//! Carries a relay's new token to paired devices, so rotating it on the server strands nobody.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use tokio::task::JoinSet;

use iroh::EndpointId;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use solferry_proto::{ALPN_RELAYS, RelayLinks, close, read_small_frame, write_frame};

use super::pairing::{adopt_relays, relay_links};
use super::{Node, Shared};
use crate::error::{Error, Result};
use crate::relay::RelayLink;
use crate::transfer::within_handshake;

/// Long enough for a handshake through a relay, short enough not to pile up behind a dead peer.
const SEND_WAIT: Duration = Duration::from_secs(10);
/// How long quitting waits for links still on their way, so a short CLI run still delivers them.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(3);
/// How often a node with a new relay token looks for a moment without transfers to restart in.
const IDLE_CHECK: Duration = Duration::from_secs(5);

type Sent = Arc<Mutex<HashSet<EndpointId>>>;

/// Relay links on their way to paired devices.
#[derive(Debug, Default)]
pub(crate) struct Shares {
    /// Paired devices that already have this device's relay links since it started.
    sent: Sent,
    running: Mutex<JoinSet<()>>,
    /// Set once a relay's new token arrived from a paired device.
    rotated: tokio::sync::watch::Sender<bool>,
}

impl Shares {
    /// Waits a moment for links still on their way.
    pub(crate) async fn finish(&self) {
        let running = std::mem::take(&mut *lock(&self.running));
        let _ = tokio::time::timeout(SHUTDOWN_WAIT, running.join_all()).await;
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The links a paired device may need: the relays this device waits on, as pairing hands them
/// out, and the ones added here by hand, which a device dialing out may not have reached yet.
fn current_links(shared: &Shared) -> Vec<String> {
    let mut links = relay_links(shared, &[]);
    for own in &shared.config.relays {
        let Ok(parsed) = RelayLink::parse(own) else { continue };
        let listed = links
            .iter()
            .any(|link| RelayLink::parse(link).is_ok_and(|link| link.same_server(&parsed)));
        if !listed {
            links.push(own.clone());
        }
    }
    links
}

impl Node {
    /// Resolves once this device took a relay's new token from a paired device and no transfer is
    /// running. A relay connection keeps the token it started with, so restart the node then, or
    /// it drops off the relay when the old token stops working.
    pub async fn wants_restart(&self) {
        let mut rotated = self.shared.relay_shares.rotated.subscribe();
        let _ = rotated.wait_for(|rotated| *rotated).await;
        while !self.shared.active_map().is_empty() {
            tokio::time::sleep(IDLE_CHECK).await;
        }
    }
}

impl Shared {
    /// Sends a paired device the relay links this device hands out, once per run, the first time
    /// the two reach each other. A failed send is tried again on the next contact.
    pub(crate) fn share_relays(&self, peer: EndpointId) {
        let Some(addr) = self.trust().get(&peer).map(crate::trust::Device::addr) else { return };
        let links = current_links(self);
        if links.is_empty() || !lock(&self.relay_shares.sent).insert(peer) {
            return;
        }
        let (endpoint, sent) = (self.endpoint.clone(), self.relay_shares.sent.clone());
        let mut running = lock(&self.relay_shares.running);
        while running.try_join_next().is_some() {}
        running.spawn(async move {
            let delivered = tokio::time::timeout(SEND_WAIT, async {
                let connection = endpoint.connect(addr, ALPN_RELAYS).await.ok()?;
                let mut stream = connection.open_uni().await.ok()?;
                write_frame(&mut stream, &RelayLinks { links }).await.ok()?;
                stream.finish().ok()?;
                connection.closed().await;
                Some(())
            })
            .await;
            if !matches!(delivered, Ok(Some(()))) {
                lock(&sent).remove(&peer);
            }
        });
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RelaysHandler(pub(crate) Arc<Shared>);

impl ProtocolHandler for RelaysHandler {
    async fn accept(&self, connection: Connection) -> Result<(), AcceptError> {
        let peer = connection.remote_id();
        if self.0.trust().get(&peer).is_none() {
            connection.close(close::NOT_PAIRED.into(), b"not paired");
            return Ok(());
        }
        let received = receive(&connection).await;
        connection.close(close::DONE.into(), b"thanks");
        match received {
            Ok(links) => adopt_rotated(&self.0, peer, &links).await,
            Err(error) => tracing::debug!(%error, "relay links not received"),
        }
        Ok(())
    }
}

async fn receive(connection: &Connection) -> Result<Vec<String>> {
    within_handshake(async {
        let mut stream = connection.accept_uni().await.map_err(|_| Error::ConnectionLost)?;
        let message: RelayLinks = read_small_frame(&mut stream).await?;
        Ok(message.links)
    })
    .await
}

/// Starts using new tokens for relays `peer` gave this device, then passes them on to the
/// devices this one gave those relays to.
async fn adopt_rotated(shared: &Shared, peer: EndpointId, links: &[String]) {
    let rotated = match shared.trust().rotate_relays(&peer, links) {
        Ok(rotated) => rotated,
        Err(error) => {
            tracing::warn!(%error, "could not store a relay's new token");
            return;
        }
    };
    if rotated.is_empty() {
        return;
    }
    tracing::info!(count = rotated.len(), "took a relay's new token from a paired device");
    // Relays added here by hand keep the link the user gave.
    let own: Vec<RelayLink> =
        shared.config.relays.iter().filter_map(|link| RelayLink::parse(link).ok()).collect();
    let adopted: Vec<String> = rotated
        .into_iter()
        .filter(|link| {
            RelayLink::parse(link).is_ok_and(|link| !own.iter().any(|own| own.same_server(&link)))
        })
        .collect();
    adopt_relays(shared, &adopted).await;
    lock(&shared.relay_shares.sent).clear();
    shared.relay_shares.rotated.send_replace(true);
}
