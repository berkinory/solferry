mod inbound;
mod nearby;
mod outbound;
mod pairing;

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};

use beam_proto::{ALPN_PAIR, ALPN_TRANSFER};
use iroh::protocol::Router;
use iroh::{Endpoint, EndpointAddr, EndpointId};
use iroh_mdns_address_lookup::MdnsAddressLookup;
use tokio::sync::{Semaphore, broadcast, mpsc};

pub use inbound::{IncomingOffer, OfferSummary};
pub use nearby::NearbyDevice;
pub use outbound::SendReport;
pub use pairing::Invite;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::event::Event;
use crate::trust::{Device, TrustStore};
use crate::{identity, net};

const EVENT_BUFFER: usize = 1024;
const OFFER_BUFFER: usize = 16;
const MAX_INBOUND: usize = 8;

#[derive(Clone, Debug)]
pub struct NodeOptions {
    pub data_dir: PathBuf,
    /// Disables relays and every discovery service. Peers must be dialed by explicit address.
    pub isolated: bool,
}

#[derive(Clone, Debug)]
pub struct Node {
    shared: Arc<Shared>,
    router: Router,
}

pub(crate) struct Shared {
    pub(crate) endpoint: Endpoint,
    pub(crate) config: Config,
    pub(crate) trust: Mutex<TrustStore>,
    pub(crate) events: broadcast::Sender<Event>,
    pub(crate) offers: mpsc::Sender<IncomingOffer>,
    pub(crate) invites: Mutex<HashMap<[u8; 32], pairing::PendingInvite>>,
    pub(crate) mdns: Option<MdnsAddressLookup>,
    pub(crate) inbound_slots: Arc<Semaphore>,
}

impl fmt::Debug for Shared {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Shared").field("id", &self.endpoint.id()).finish_non_exhaustive()
    }
}

impl Shared {
    pub(crate) fn trust(&self) -> MutexGuard<'_, TrustStore> {
        self.trust.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Node {
    pub async fn start(options: NodeOptions) -> Result<(Self, mpsc::Receiver<IncomingOffer>)> {
        let NodeOptions { data_dir, isolated } = options;
        identity::prepare_private_dir(&data_dir)?;
        let config = Config::load(&data_dir)?;
        let secret = identity::load_or_create(&data_dir)?;
        let network = net::bind(secret, &config, isolated).await?;
        let (offers, offer_queue) = mpsc::channel(OFFER_BUFFER);
        let shared = Arc::new(Shared {
            endpoint: network.endpoint.clone(),
            config,
            trust: Mutex::new(TrustStore::load(&data_dir)?),
            events: broadcast::channel(EVENT_BUFFER).0,
            offers,
            invites: Mutex::new(HashMap::new()),
            mdns: network.mdns,
            inbound_slots: Arc::new(Semaphore::new(MAX_INBOUND)),
        });
        let router = Router::builder(network.endpoint)
            .accept(ALPN_TRANSFER, inbound::TransferHandler(shared.clone()))
            .accept(ALPN_PAIR, pairing::PairHandler(shared.clone()))
            .spawn();
        Ok((Self { shared, router }, offer_queue))
    }

    #[must_use]
    pub fn id(&self) -> EndpointId {
        self.shared.endpoint.id()
    }

    #[must_use]
    pub fn addr(&self) -> EndpointAddr {
        self.shared.endpoint.addr()
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.shared.config
    }

    #[must_use]
    pub fn events(&self) -> broadcast::Receiver<Event> {
        self.shared.events.subscribe()
    }

    #[must_use]
    pub fn devices(&self) -> Vec<Device> {
        self.shared.trust().all().to_vec()
    }

    pub fn find_device(&self, query: &str) -> Result<Device> {
        self.shared.trust().resolve(query).cloned()
    }

    pub fn forget(&self, query: &str) -> Result<Device> {
        let mut trust = self.shared.trust();
        let device = trust.resolve(query)?.clone();
        trust.remove(&device.id)?;
        Ok(device)
    }

    /// Resolves when the endpoint can be reached through its home relay.
    pub async fn online(&self) {
        self.shared.endpoint.online().await;
    }

    pub async fn shutdown(self) -> Result<()> {
        self.router.shutdown().await.map_err(|error| Error::Protocol(error.to_string()))
    }
}
