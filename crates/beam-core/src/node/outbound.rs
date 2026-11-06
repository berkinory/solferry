use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use beam_proto::{ALPN_TRANSFER, Decision, Offer, OfferReply, read_frame, write_frame};
use iroh::endpoint::{Connection, RecvStream, SendStream};
use iroh::{EndpointAddr, EndpointId, RelayUrl};

use super::{Node, Shared};
use crate::config::RelayPolicy;
use crate::error::{Error, Result};
use crate::event::{Direction, Event, TransferId};
use crate::net::parse_relays;
use crate::route::{Route, RouteRule, enforce_route, wait_for_route};
use crate::transfer::sender;
use crate::transfer::{Manifest, Meter, exchange_hello, remote_failure};

const DIRECT_PATH_WAIT: Duration = Duration::from_secs(12);
const RETRY_WINDOW: Duration = Duration::from_secs(300);
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
        loop {
            let outcome = attempt(&self.shared, target.clone(), &manifest).await;
            match outcome {
                Err(error) if error.is_retryable() && started.elapsed() < RETRY_WINDOW => {
                    tracing::info!(%error, "retrying transfer");
                    tokio::time::sleep(delay).await;
                    delay = (delay * 2).min(RETRY_MAX);
                }
                Err(error) => {
                    let event = Event::Failed {
                        id: manifest.id,
                        error: error.clone(),
                    };
                    let _ = self.shared.events.send(event);
                    return Err(error);
                }
                Ok(report) => {
                    let _ = self
                        .shared
                        .events
                        .send(Event::Completed { id: manifest.id });
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
) -> Result<SendReport> {
    let peer = target.id;
    let connection = shared
        .endpoint
        .connect(target, ALPN_TRANSFER)
        .await
        .map_err(|error| Error::Unreachable(error.to_string()))?;
    let result = async {
        let (send, recv) = connection
            .open_bi()
            .await
            .map_err(|_| Error::ConnectionLost)?;
        let rule = route_rule(shared, peer)?;
        tokio::select! {
            report = negotiate(shared, &connection, send, recv, manifest, &rule, peer) => report,
            violation = enforce_route(&connection, &rule) => Err(violation),
        }
    }
    .await;
    result.map_err(|error| remote_failure(&connection, error))
}

fn route_rule(shared: &Shared, peer: EndpointId) -> Result<RouteRule> {
    let mut relays: Vec<RelayUrl> = parse_relays(&shared.config.relays)?;
    if let Some(device) = shared.trust().get(&peer) {
        relays.extend(parse_relays(&device.relays)?);
    }
    Ok(RouteRule {
        any_relay: shared.config.relay_policy == RelayPolicy::Always,
        relays,
    })
}

async fn negotiate(
    shared: &Shared,
    connection: &Connection,
    mut send: SendStream,
    mut recv: RecvStream,
    manifest: &Arc<Manifest>,
    rule: &RouteRule,
    peer: EndpointId,
) -> Result<SendReport> {
    let hello = exchange_hello(&mut send, &mut recv, &shared.config.name).await?;
    let offer = Offer {
        transfer_id: manifest.id.0.to_vec(),
        entries: manifest.entries.clone(),
    };
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
        return Err(Error::Protocol(
            "resume table does not match the offer".to_owned(),
        ));
    }
    let oversized = reply
        .offsets
        .iter()
        .zip(&manifest.entries)
        .any(|(offset, entry)| *offset > entry.size);
    if oversized {
        return Err(Error::Protocol(
            "resume offset exceeds file size".to_owned(),
        ));
    }

    let route = wait_for_route(connection, rule, DIRECT_PATH_WAIT).await?;
    let bytes: u64 = manifest.entries.iter().map(|entry| entry.size).sum();
    let resumed: u64 = reply.offsets.iter().sum();
    let files = manifest
        .entries
        .iter()
        .filter(|entry| !entry.is_dir)
        .count();
    let _ = shared.events.send(Event::Started {
        id: manifest.id,
        peer,
        peer_name: hello.name.clone(),
        direction: Direction::Sending,
        route,
        files,
        bytes,
    });
    let meter = Meter::start(manifest.id, resumed, bytes, shared.events.clone());
    sender::run(connection, &mut recv, manifest, &reply.offsets, &meter).await?;
    Ok(SendReport {
        id: manifest.id,
        peer_name: hello.name,
        route,
        bytes_sent: bytes - resumed,
        resumed_bytes: resumed,
    })
}
