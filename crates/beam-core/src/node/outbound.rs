use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use beam_proto::{ALPN_TRANSFER, Decision, Offer, OfferReply, read_frame, write_frame};
use iroh::endpoint::{Connection, RecvStream, SendStream};
use iroh::{EndpointAddr, EndpointId};

use super::{Node, Shared};
use crate::error::{Error, Result};
use crate::event::{Direction, Event, TransferId};
use crate::route::{Route, current_route};
use crate::transfer::{Manifest, Meter, exchange_hello, remote_failure};
use crate::transfer::{sender, transfer_label};

const FIRST_CONTACT_WINDOW: Duration = Duration::from_secs(20);
const RESUME_WINDOW: Duration = Duration::from_secs(300);
const RETRY_BASE: Duration = Duration::from_secs(1);
const RETRY_MAX: Duration = Duration::from_secs(15);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SendReport {
    pub id: TransferId,
    pub peer_name: String,
    pub route: Route,
    pub bytes_sent: u64,
    pub resumed_bytes: u64,
}

impl Node {
    /// Sends `paths` to a device, reconnecting and resuming after network interruptions.
    pub async fn send(
        &self,
        target: impl Into<EndpointAddr>,
        paths: Vec<PathBuf>,
    ) -> Result<SendReport> {
        let target = target.into();
        let sender_id = self.id();
        let manifest = tokio::task::spawn_blocking(move || Manifest::scan(&paths, sender_id))
            .await
            .map_err(|error| Error::Protocol(error.to_string()))??;
        let manifest = Arc::new(manifest);
        let started = Instant::now();
        let mut delay = RETRY_BASE;
        let mut reached = false;
        loop {
            let outcome = attempt(&self.shared, target.clone(), &manifest, &mut reached).await;
            let window = if reached { RESUME_WINDOW } else { FIRST_CONTACT_WINDOW };
            match outcome {
                Err(error) if error.is_retryable() && started.elapsed() < window => {
                    tracing::info!(%error, "retrying transfer");
                    let id = manifest.id;
                    let _ = self.shared.events.send(Event::Reconnecting { id, reason: error });
                    tokio::time::sleep(delay).await;
                    delay = (delay * 2).min(RETRY_MAX);
                }
                Err(error) => {
                    let event = Event::Failed { id: manifest.id, error: error.clone() };
                    let _ = self.shared.events.send(event);
                    return Err(error);
                }
                Ok(report) => {
                    let _ = self.shared.events.send(Event::Completed { id: manifest.id });
                    return Ok(report);
                }
            }
        }
    }
}

async fn attempt(
    shared: &Shared,
    target: EndpointAddr,
    manifest: &Arc<Manifest>,
    reached: &mut bool,
) -> Result<SendReport> {
    let peer = target.id;
    let connection = shared
        .endpoint
        .connect(target, ALPN_TRANSFER)
        .await
        .map_err(|error| Error::Unreachable(error.to_string()))?;
    *reached = true;
    shared.track(manifest.id, &connection);
    let result = async {
        let (send, recv) = connection.open_bi().await.map_err(|_| Error::ConnectionLost)?;
        negotiate(shared, &connection, send, recv, manifest, peer).await
    }
    .await;
    shared.untrack(manifest.id);
    result.map_err(|error| remote_failure(&connection, error))
}

async fn negotiate(
    shared: &Shared,
    connection: &Connection,
    mut send: SendStream,
    mut recv: RecvStream,
    manifest: &Arc<Manifest>,
    peer: EndpointId,
) -> Result<SendReport> {
    let hello = exchange_hello(&mut send, &mut recv, &shared.config.name).await?;
    let offer = Offer { transfer_id: manifest.id.0.to_vec(), entries: manifest.entries.clone() };
    write_frame(&mut send, &offer).await?;
    let reply: OfferReply = read_frame(&mut recv).await?;
    match reply.decision() {
        Decision::Accepted => {}
        Decision::Declined => return Err(Error::Declined),
        Decision::NoSpace => return Err(Error::NoSpace),
        Decision::InvalidOffer => return Err(Error::InvalidOffer("rejected by peer".to_owned())),
        Decision::Unspecified => return Err(Error::Protocol("empty decision".to_owned())),
    }
    if reply.offsets.len() != manifest.entries.len() {
        return Err(Error::Protocol("resume table does not match the offer".to_owned()));
    }
    let oversized =
        reply.offsets.iter().zip(&manifest.entries).any(|(offset, entry)| *offset > entry.size);
    if oversized {
        return Err(Error::Protocol("resume offset exceeds file size".to_owned()));
    }

    let bytes: u64 = manifest.entries.iter().map(|entry| entry.size).sum();
    let resumed: u64 = reply.offsets.iter().sum();
    let files = manifest.entries.iter().filter(|entry| !entry.is_dir).count();
    let _ = shared.events.send(Event::Started {
        id: manifest.id,
        peer,
        peer_name: hello.name.clone(),
        label: transfer_label(&manifest.entries),
        direction: Direction::Sending,
        route: current_route(connection),
        files,
        bytes,
    });
    let meter = Meter::start(manifest.id, resumed, bytes, shared.events.clone());
    let final_route = sender::run(connection, &mut recv, manifest, &reply.offsets, &meter).await?;
    Ok(SendReport {
        id: manifest.id,
        peer_name: hello.name,
        route: final_route,
        bytes_sent: bytes - resumed,
        resumed_bytes: resumed,
    })
}
