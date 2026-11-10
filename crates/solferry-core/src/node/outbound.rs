use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use iroh::endpoint::{ConnectOptions, Connection, RecvStream, SendStream};
use iroh::{EndpointAddr, EndpointId};
use solferry_proto::{ALPN_TRANSFER, Decision, Offer, OfferReply, read_frame, write_frame};

use super::{Node, Shared};
use crate::error::{Error, Result};
use crate::event::{Direction, Event, TransferId};
use crate::net::local_transport_config;
use crate::route::{self, Route, current_route};
use crate::transfer::segments;
use crate::transfer::{Manifest, Meter, NamedFile, exchange_hello, remote_failure};
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
        let sender_id = self.id();
        let manifest = tokio::task::spawn_blocking(move || Manifest::scan(&paths, sender_id))
            .await
            .map_err(|error| Error::Protocol(error.to_string()))??;
        self.deliver(target.into(), Arc::new(manifest)).await
    }

    /// Sends files that arrive with their own names, such as shared content on a phone.
    pub async fn send_named(
        &self,
        target: impl Into<EndpointAddr>,
        files: Vec<NamedFile>,
    ) -> Result<SendReport> {
        let sender_id = self.id();
        let manifest = tokio::task::spawn_blocking(move || Manifest::named(files, sender_id))
            .await
            .map_err(|error| Error::Protocol(error.to_string()))??;
        self.deliver(target.into(), Arc::new(manifest)).await
    }

    async fn deliver(&self, target: EndpointAddr, manifest: Arc<Manifest>) -> Result<SendReport> {
        let started = Instant::now();
        let peer_name = self.shared.trust().get(&target.id).map(|device| device.name.clone());
        let _ = self.shared.events.send(Event::Waiting {
            id: manifest.id,
            peer: target.id,
            peer_name: peer_name.unwrap_or_else(|| target.id.fmt_short().to_string()),
            label: transfer_label(&manifest.entries),
            bytes: manifest.entries.iter().map(|entry| entry.size).sum(),
        });
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
                    self.shared.publish(event);
                    return Err(error);
                }
                Ok(report) => {
                    self.shared.publish(Event::Completed { id: manifest.id });
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
    let mut options = ConnectOptions::new();
    let local = route::on_local_network(&target);
    tracing::info!(local, "connecting to send");
    if local {
        options = options.with_transport_config(local_transport_config()?);
    }
    let unreachable = |error: &dyn std::fmt::Display| Error::Unreachable(error.to_string());
    let connecting = shared
        .endpoint
        .connect_with_opts(target, ALPN_TRANSFER, options)
        .await
        .map_err(|error| unreachable(&error))?;
    let connection = connecting.await.map_err(|error| unreachable(&error))?;
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
    let progress = segments::unflatten(&manifest.entries, &reply.offsets)
        .ok_or_else(|| Error::Protocol("resume table does not match the offer".to_owned()))?;

    let route = current_route(connection);
    let bytes: u64 = manifest.entries.iter().map(|entry| entry.size).sum();
    let resumed = segments::total(&progress);
    let files = manifest.entries.iter().filter(|entry| !entry.is_dir).count();
    shared.publish(Event::Started {
        id: manifest.id,
        peer,
        peer_name: hello.name.clone(),
        label: transfer_label(&manifest.entries),
        direction: Direction::Sending,
        route,
        files,
        bytes,
        platform: crate::Platform::parse(&hello.platform),
    });
    let meter = Meter::start(manifest.id, resumed, bytes, shared.events.clone());
    let final_route = tokio::select! {
        finished = sender::run(connection, &mut recv, manifest, &progress, &meter) => finished?,
        () = route::report_changes(connection, manifest.id, route, &shared.events) => unreachable!(),
    };
    Ok(SendReport {
        id: manifest.id,
        peer_name: hello.name,
        route: final_route,
        bytes_sent: bytes - resumed,
        resumed_bytes: resumed,
    })
}
