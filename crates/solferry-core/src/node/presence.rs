//! Which paired devices are there. Each side probes the others, a probe that arrives counts as
//! an answer the other way, and a device about to go away says so.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use futures::future::join_all;
use iroh::EndpointId;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use solferry_proto::{ALPN_LEAVING, ALPN_PRESENCE, close};

use super::{Node, Shared};
use crate::event::Event;

/// A handshake on a local network takes milliseconds, and one through a relay about a second.
const ANSWER_WAIT: Duration = Duration::from_secs(3);
/// Apps check every 3 seconds, so this covers a missed round and a slow answer.
const CHECK_IN_FRESH: Duration = Duration::from_secs(8);
/// Leaving can't hold up quitting for long; a device that misses it finds out on its next probe.
const LEAVE_WAIT: Duration = Duration::from_secs(1);

/// What this device has heard from paired devices, apart from its own probes.
#[derive(Debug, Default)]
pub(crate) struct Heard {
    /// When each one last probed this device.
    checked_in: HashMap<EndpointId, Instant>,
    /// Said they were going away and haven't probed since. A frozen app may still answer for a
    /// moment, so their answers don't count until they reach out again.
    left: HashSet<EndpointId>,
}

/// A finished handshake is the whole answer, so nothing is ever said on this protocol.
///
/// A paired device that reached this one is there too, even when this side can't reach it:
/// multicast often stops between a router's 2.4 and 5 GHz bands, or at a phone saving power.
#[derive(Debug, Clone)]
pub(crate) struct PresenceHandler(pub(crate) Arc<Shared>);

impl ProtocolHandler for PresenceHandler {
    fn accept(
        &self,
        connection: Connection,
    ) -> impl Future<Output = Result<(), AcceptError>> + Send {
        let peer = connection.remote_id();
        connection.close(close::DONE.into(), b"here");
        if self.0.trust().get(&peer).is_some() {
            let mut heard = lock(&self.0.heard);
            heard.checked_in.insert(peer, Instant::now());
            heard.left.remove(&peer);
            drop(heard);
            let _ = self.0.events.send(Event::Present { peer });
            self.0.share_relays(peer);
        }
        std::future::ready(Ok(()))
    }
}

/// The handshake alone says the device is going away.
#[derive(Debug, Clone)]
pub(crate) struct LeavingHandler(pub(crate) Arc<Shared>);

impl ProtocolHandler for LeavingHandler {
    fn accept(
        &self,
        connection: Connection,
    ) -> impl Future<Output = Result<(), AcceptError>> + Send {
        let peer = connection.remote_id();
        connection.close(close::DONE.into(), b"bye");
        if self.0.trust().get(&peer).is_some() {
            let mut heard = lock(&self.0.heard);
            heard.checked_in.remove(&peer);
            heard.left.insert(peer);
            drop(heard);
            let _ = self.0.events.send(Event::Absent { peer });
        }
        std::future::ready(Ok(()))
    }
}

impl Node {
    /// Paired devices that are there: those that reached this one lately, and those that answer
    /// a probe now. A device that just checked in isn't probed, so two devices watching each
    /// other cost one handshake per round, not two.
    pub async fn reachable(&self) -> HashSet<EndpointId> {
        let endpoint = &self.shared.endpoint;
        let (fresh, left) = {
            let heard = lock(&self.shared.heard);
            let fresh: HashSet<EndpointId> = heard
                .checked_in
                .iter()
                .filter(|(_, at)| at.elapsed() < CHECK_IN_FRESH)
                .map(|(id, _)| *id)
                .collect();
            (fresh, heard.left.clone())
        };
        let devices = self.devices().into_iter().filter(|device| !fresh.contains(&device.id));
        let probes = devices.map(|device| async move {
            answers(endpoint, device.addr()).await.then_some(device.id)
        });
        let answered: Vec<EndpointId> = join_all(probes).await.into_iter().flatten().collect();
        for peer in &answered {
            self.shared.share_relays(*peer);
        }
        answered.into_iter().filter(|id| !left.contains(id)).chain(fresh).collect()
    }

    /// Tells the paired devices that this one is going away, so they stop showing it at once.
    /// Waits about a second at most.
    pub async fn leave(&self) {
        let endpoint = &self.shared.endpoint;
        let goodbyes = self.devices().into_iter().map(|device| async move {
            let goodbye = endpoint.connect(device.addr(), ALPN_LEAVING);
            if let Ok(Ok(connection)) = tokio::time::timeout(LEAVE_WAIT, goodbye).await {
                connection.close(close::DONE.into(), b"bye");
            }
        });
        join_all(goodbyes).await;
    }
}

/// Whether the device answers a presence probe right now.
pub(super) async fn answers(endpoint: &iroh::Endpoint, addr: iroh::EndpointAddr) -> bool {
    let probe = endpoint.connect(addr, ALPN_PRESENCE);
    let Ok(Ok(connection)) = tokio::time::timeout(ANSWER_WAIT, probe).await else { return false };
    connection.close(close::DONE.into(), b"seen");
    true
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}
