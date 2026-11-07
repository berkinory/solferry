mod confirm;
mod health;
mod inbound;
mod nearby;
mod outbound;
mod overlay;
mod pairing;

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use iroh::endpoint::Connection;
use iroh::protocol::Router;
use iroh::{Endpoint, EndpointAddr, EndpointId};
use iroh_mdns_address_lookup::MdnsAddressLookup;
use solferry_proto::{ALPN_PAIR, ALPN_TRANSFER, close};
use tokio::sync::{Semaphore, broadcast, mpsc};

pub use confirm::PairingRequest;
pub use health::RelayHealth;
pub use inbound::{IncomingOffer, OfferSummary};
pub use nearby::NearbyDevice;
pub use outbound::SendReport;
pub use pairing::Invite;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::event::{Event, TransferId};
use crate::relay::RelayLink;
use crate::route;
use crate::trust::{Device, TrustStore};
use crate::{identity, net};

const EVENT_BUFFER: usize = 1024;
const OFFER_BUFFER: usize = 16;
const MAX_INBOUND: usize = 8;
/// Unpaired devices get their own small pool, so they can never crowd out your own devices.
const MAX_STRANGERS: usize = 2;

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
    pair_listener: Mutex<Option<mpsc::Sender<PairingRequest>>>,
    pub(crate) mdns: Option<MdnsAddressLookup>,
    pub(crate) inbound_slots: Arc<Semaphore>,
    pub(crate) stranger_slots: Arc<Semaphore>,
    pub(crate) active: Mutex<HashMap<TransferId, Connection>>,
    uses_relays: bool,
}

impl fmt::Debug for Shared {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("Shared").field("id", &self.endpoint.id()).finish_non_exhaustive()
    }
}

impl Shared {
    pub(crate) fn track(&self, id: TransferId, connection: &Connection) {
        self.active_map().insert(id, connection.clone());
    }

    fn pair_listener(&self) -> MutexGuard<'_, Option<mpsc::Sender<PairingRequest>>> {
        self.pair_listener.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn has_invites(&self) -> bool {
        !self.invites.lock().unwrap_or_else(std::sync::PoisonError::into_inner).is_empty()
    }

    pub(crate) fn untrack(&self, id: TransferId) {
        self.active_map().remove(&id);
    }

    fn active_map(&self) -> MutexGuard<'_, HashMap<TransferId, Connection>> {
        self.active.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn trust(&self) -> MutexGuard<'_, TrustStore> {
        self.trust.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Your own relays plus the ones paired devices wait on, which is how a device reaches them.
fn known_relays(config: &Config, trust: &TrustStore) -> Result<Vec<RelayLink>> {
    let mut relays =
        config.relays.iter().map(|link| RelayLink::parse(link)).collect::<Result<Vec<_>>>()?;
    let shared = trust.all().iter().flat_map(|device| &device.relays);
    relays.extend(shared.filter_map(|link| RelayLink::parse(link).ok()));
    relays.sort_by(|a, b| a.url().cmp(b.url()));
    relays.dedup_by(|a, b| a.url() == b.url());
    Ok(relays)
}

impl Node {
    pub async fn start(options: NodeOptions) -> Result<(Self, mpsc::Receiver<IncomingOffer>)> {
        let NodeOptions { data_dir, isolated } = options;
        identity::prepare_private_dir(&data_dir)?;
        let config = Config::load(&data_dir)?;
        let downloads = config.download_dir.clone();
        tokio::task::spawn_blocking(move || crate::transfer::prune_abandoned(&downloads));
        let key_dir = data_dir.clone();
        let secret =
            tokio::task::spawn_blocking(move || identity::load_or_create(&key_dir, !isolated))
                .await
                .map_err(|error| Error::Protocol(error.to_string()))??;
        let trust = TrustStore::load(&data_dir)?;
        let relays = if isolated { Vec::new() } else { known_relays(&config, &trust)? };
        let uses_relays = !relays.is_empty();
        let network = net::bind(secret, &config, &relays, isolated).await?;
        let (offers, offer_queue) = mpsc::channel(OFFER_BUFFER);
        let shared = Arc::new(Shared {
            endpoint: network.endpoint.clone(),
            config,
            trust: Mutex::new(trust),
            events: broadcast::channel(EVENT_BUFFER).0,
            offers,
            invites: Mutex::new(HashMap::new()),
            pair_listener: Mutex::new(None),
            mdns: network.mdns,
            inbound_slots: Arc::new(Semaphore::new(MAX_INBOUND)),
            stranger_slots: Arc::new(Semaphore::new(MAX_STRANGERS)),
            active: Mutex::new(HashMap::new()),
            uses_relays,
        });
        if !isolated {
            overlay::answer_probes(shared.clone());
        }
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

    /// This device's address on an overlay such as Tailscale, if it joined one.
    #[must_use]
    pub fn overlay_address(&self) -> Option<std::net::Ipv4Addr> {
        route::overlay_address()
    }

    /// Decides whether this paired device may send files without asking.
    pub fn set_auto_accept(&self, id: EndpointId, enabled: bool) -> Result<()> {
        self.shared.trust().set_auto_accept(&id, enabled)
    }

    #[must_use]
    pub fn blocked(&self) -> Vec<EndpointId> {
        self.shared.trust().blocked().to_vec()
    }

    /// Refuses every connection from this device and forgets it if it was paired.
    pub fn block(&self, id: EndpointId) -> Result<()> {
        self.shared.trust().block(id)
    }

    pub fn unblock(&self, id: EndpointId) -> Result<()> {
        self.shared.trust().unblock(&id)
    }

    /// Stops a running transfer. Partial data stays, so sending again resumes it.
    pub fn cancel(&self, id: TransferId) {
        if let Some(connection) = self.shared.active_map().remove(&id) {
            connection.close(close::CANCELLED.into(), b"cancelled");
        }
    }

    /// Resolves when the endpoint can be reached through its home relay.
    pub async fn online(&self) {
        self.shared.endpoint.online().await;
    }

    /// Waits for a relay connection, but never longer than `limit`.
    pub async fn online_within(&self, limit: Duration) {
        if self.shared.uses_relays {
            let _ = tokio::time::timeout(limit, self.online()).await;
        }
    }

    pub async fn shutdown(self) -> Result<()> {
        self.router.shutdown().await.map_err(|error| Error::Protocol(error.to_string()))
    }
}
