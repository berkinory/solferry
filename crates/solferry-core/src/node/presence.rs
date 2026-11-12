use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use futures::future::join_all;
use iroh::EndpointId;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use solferry_proto::{ALPN_PRESENCE, close};

use super::{Node, Shared};
use crate::event::Event;

/// A handshake on a local network takes milliseconds, and one through a relay about a second.
const ANSWER_WAIT: Duration = Duration::from_secs(3);
/// Apps check every 5 seconds, so this covers two missed rounds and a slow answer.
const CHECK_IN_FRESH: Duration = Duration::from_secs(12);

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
            lock(&self.0.checked_in).insert(peer, Instant::now());
            let _ = self.0.events.send(Event::Present { peer });
        }
        std::future::ready(Ok(()))
    }
}

impl Node {
    /// Paired devices that answer right now, over any path, and those that reached this one
    /// lately. Each probe costs a single handshake.
    pub async fn reachable(&self) -> HashSet<EndpointId> {
        let endpoint = &self.shared.endpoint;
        let fresh: Vec<EndpointId> = lock(&self.shared.checked_in)
            .iter()
            .filter(|(_, at)| at.elapsed() < CHECK_IN_FRESH)
            .map(|(id, _)| *id)
            .collect();
        let probes = self.devices().into_iter().map(|device| async move {
            let probe = endpoint.connect(device.addr(), ALPN_PRESENCE);
            let connection = tokio::time::timeout(ANSWER_WAIT, probe).await.ok()?.ok()?;
            connection.close(close::DONE.into(), b"seen");
            Some(device.id)
        });
        let mut answered: HashSet<EndpointId> =
            join_all(probes).await.into_iter().flatten().collect();
        answered.extend(fresh);
        answered
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}
