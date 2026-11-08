use std::collections::HashSet;
use std::time::Duration;

use futures::future::join_all;
use iroh::EndpointId;
use iroh::endpoint::Connection;
use iroh::protocol::{AcceptError, ProtocolHandler};
use solferry_proto::{ALPN_PRESENCE, close};

use super::Node;

const ANSWER_WAIT: Duration = Duration::from_secs(5);

/// A finished handshake is the whole answer, so nothing is ever said on this protocol.
#[derive(Debug, Clone)]
pub(crate) struct PresenceHandler;

impl ProtocolHandler for PresenceHandler {
    fn accept(
        &self,
        connection: Connection,
    ) -> impl Future<Output = Result<(), AcceptError>> + Send {
        connection.close(close::DONE.into(), b"here");
        std::future::ready(Ok(()))
    }
}

impl Node {
    /// Paired devices that answer right now, over any path. Each one costs a single handshake.
    pub async fn reachable(&self) -> HashSet<EndpointId> {
        let endpoint = &self.shared.endpoint;
        let probes = self.devices().into_iter().map(|device| async move {
            let probe = endpoint.connect(device.addr(), ALPN_PRESENCE);
            let connection = tokio::time::timeout(ANSWER_WAIT, probe).await.ok()?.ok()?;
            connection.close(close::DONE.into(), b"seen");
            Some(device.id)
        });
        join_all(probes).await.into_iter().flatten().collect()
    }
}
