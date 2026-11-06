use std::path::{Path, PathBuf};
use std::sync::Arc;

use beam_proto::{Decision, Offer, OfferReply, close, read_frame, write_frame};
use iroh::EndpointId;
use iroh::TransportAddr;
use iroh::endpoint::{Connection, SendStream};
use iroh::protocol::{AcceptError, ProtocolHandler};
use tokio::sync::oneshot;

use super::Shared;
use crate::config::Visibility;
use crate::error::{Error, Result, StorageContext};
use crate::event::{Direction, Event, TransferId};
use crate::route::{Route, is_local};
use crate::transfer::receiver::{self, Plan};
use crate::transfer::{Meter, Staging, exchange_hello, validate_offer};
use crate::trust::{Device, Relationship};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OfferSummary {
    pub peer: EndpointId,
    pub peer_name: String,
    pub relationship: Option<Relationship>,
    pub files: usize,
    pub bytes: u64,
    pub first_path: String,
}

#[derive(Debug)]
pub struct IncomingOffer {
    pub summary: OfferSummary,
    reply: oneshot::Sender<Option<PathBuf>>,
}

impl IncomingOffer {
    pub fn accept(self, destination: PathBuf) {
        let _ = self.reply.send(Some(destination));
    }

    pub fn decline(self) {
        let _ = self.reply.send(None);
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TransferHandler(pub(crate) Arc<Shared>);

impl ProtocolHandler for TransferHandler {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let shared = self.0.clone();
        if let Err(error) = serve(&shared, &connection).await {
            tracing::warn!(peer = %connection.remote_id().fmt_short(), %error, "inbound transfer failed");
        }
        Ok(())
    }
}

async fn serve(shared: &Shared, connection: &Connection) -> Result<()> {
    let peer = connection.remote_id();
    let device = shared.trust().get(&peer).cloned();
    if device.is_none() && !admits_stranger(shared, connection) {
        connection.close(close::NOT_PAIRED.into(), b"pair first");
        return Err(Error::NotPaired);
    }
    let (mut send, mut recv) = connection
        .accept_bi()
        .await
        .map_err(|_| Error::ConnectionLost)?;
    let hello = match exchange_hello(&mut send, &mut recv, &shared.config.name).await {
        Err(error @ Error::VersionMismatch { .. }) => {
            connection.close(close::VERSION_MISMATCH.into(), b"version");
            return Err(error);
        }
        other => other?,
    };
    let offer: Offer = read_frame(&mut recv).await?;
    let id = transfer_id(&offer)?;
    let targets = match validate_offer(&offer) {
        Ok(targets) => targets,
        Err(error) => {
            reply(&mut send, Decision::InvalidOffer, Vec::new()).await?;
            return Err(error);
        }
    };

    let summary = summarize(peer, &hello.name, device.as_ref(), &offer);
    let Some(destination) = decide(shared, summary).await else {
        reply(&mut send, Decision::Declined, Vec::new()).await?;
        return Ok(());
    };
    std::fs::create_dir_all(&destination).at(&destination)?;
    let staging = Staging::open(&destination, id)?;
    let offsets = staging.offsets(&offer.entries);
    let bytes: u64 = offer.entries.iter().map(|entry| entry.size).sum();
    let resumed: u64 = offsets.iter().sum();
    if !has_room(&destination, bytes - resumed) {
        reply(&mut send, Decision::NoSpace, Vec::new()).await?;
        return Err(Error::NoSpace);
    }
    reply(&mut send, Decision::Accepted, offsets.clone()).await?;

    let files = offer.entries.iter().filter(|entry| !entry.is_dir).count();
    let _ = shared.events.send(Event::Started {
        id,
        peer,
        peer_name: hello.name,
        direction: Direction::Receiving,
        route: current_route(connection),
        files,
        bytes,
    });
    let meter = Arc::new(Meter::start(id, resumed, bytes, shared.events.clone()));
    let plan = Plan {
        destination,
        entries: offer.entries,
        targets,
        offsets,
        staging,
    };
    let result = receiver::run(connection, &mut send, plan, meter).await;
    let event = match result {
        Ok(_) => Event::Completed { id },
        Err(error) => Event::Failed { id, error },
    };
    let _ = shared.events.send(event);
    Ok(())
}

fn admits_stranger(shared: &Shared, connection: &Connection) -> bool {
    shared.config.visibility == Visibility::Everyone
        && connection
            .paths()
            .iter()
            .any(|path| match path.remote_addr() {
                TransportAddr::Ip(addr) => is_local(addr.ip()),
                _ => false,
            })
}

fn current_route(connection: &Connection) -> Route {
    let paths = connection.paths();
    let selected = paths.iter().find(iroh::endpoint::Path::is_selected);
    match selected.as_ref().map(iroh::endpoint::Path::remote_addr) {
        Some(TransportAddr::Ip(addr)) if is_local(addr.ip()) => Route::Local,
        Some(TransportAddr::Relay(_)) => Route::Relay,
        _ => Route::Direct,
    }
}

fn transfer_id(offer: &Offer) -> Result<TransferId> {
    let bytes = offer.transfer_id.as_slice().try_into();
    bytes
        .map(TransferId)
        .map_err(|_| Error::InvalidOffer("malformed transfer id".to_owned()))
}

fn summarize(peer: EndpointId, name: &str, device: Option<&Device>, offer: &Offer) -> OfferSummary {
    OfferSummary {
        peer,
        peer_name: device
            .map_or(name, |device| device.name.as_str())
            .to_owned(),
        relationship: device.map(|device| device.relationship),
        files: offer.entries.iter().filter(|entry| !entry.is_dir).count(),
        bytes: offer.entries.iter().map(|entry| entry.size).sum(),
        first_path: offer
            .entries
            .first()
            .map(|entry| entry.path.clone())
            .unwrap_or_default(),
    }
}

async fn decide(shared: &Shared, summary: OfferSummary) -> Option<PathBuf> {
    let (reply, decision) = oneshot::channel();
    shared
        .offers
        .send(IncomingOffer { summary, reply })
        .await
        .ok()?;
    decision.await.ok().flatten()
}

async fn reply(send: &mut SendStream, decision: Decision, offsets: Vec<u64>) -> Result<()> {
    Ok(write_frame(
        send,
        &OfferReply {
            decision: decision as i32,
            offsets,
        },
    )
    .await?)
}

fn has_room(destination: &Path, needed: u64) -> bool {
    fs4::available_space(destination).is_ok_and(|available| available >= needed)
}
