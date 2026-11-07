use std::time::{Duration, Instant};

use futures::future::join_all;
use iroh::{Endpoint, RelayUrl, SecretKey};
use tokio::net::TcpStream;

use super::{Node, known_relays};
use crate::config::Config;
use crate::relay::RelayLink;
use crate::{net, route};

const CONNECT_WAIT: Duration = Duration::from_secs(5);
const ADDRESS_POLL: Duration = Duration::from_millis(200);

/// What a device can currently do with one relay, from the first thing to go wrong to the last.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelayHealth {
    pub url: RelayUrl,
    pub reachable: bool,
    pub accepted: bool,
    pub direct_paths: bool,
}

impl Node {
    /// Tests every relay this device knows, waiting up to `limit` on each.
    pub async fn relay_health(&self, limit: Duration) -> Vec<RelayHealth> {
        let relays = {
            let trust = self.shared.trust();
            known_relays(&self.shared.config, &trust).unwrap_or_default()
        };
        let config = &self.shared.config;
        join_all(relays.iter().map(|relay| check(relay, config, limit))).await
    }
}

/// Connects with a throwaway identity, because only a real connection shows whether the relay
/// accepts the token, and because a health check should not reveal which device is asking.
async fn check(relay: &RelayLink, config: &Config, limit: Duration) -> RelayHealth {
    let url = relay.url().clone();
    let reachable = reaches_server(&url).await;
    let probe = net::bind_probe(SecretKey::generate(), config, relay).await.ok();
    let (accepted, direct_paths) = match &probe {
        Some(endpoint) if reachable => {
            let accepted = tokio::time::timeout(limit, endpoint.online()).await.is_ok();
            (accepted, accepted && sees_public_address(endpoint, limit).await)
        }
        _ => (false, false),
    };
    if let Some(endpoint) = probe {
        endpoint.close().await;
    }
    RelayHealth { url, reachable, accepted, direct_paths }
}

/// A public address can only come from the relay's UDP port, so seeing one proves it open.
async fn sees_public_address(endpoint: &Endpoint, limit: Duration) -> bool {
    let started = Instant::now();
    while started.elapsed() < limit {
        if endpoint.addr().ip_addrs().any(|addr| route::is_public(addr.ip())) {
            return true;
        }
        tokio::time::sleep(ADDRESS_POLL).await;
    }
    false
}

async fn reaches_server(url: &RelayUrl) -> bool {
    let Some(host) = url.host_str() else {
        return false;
    };
    let address = (host, url.port_or_known_default().unwrap_or(443));
    tokio::time::timeout(CONNECT_WAIT, TcpStream::connect(address))
        .await
        .is_ok_and(|stream| stream.is_ok())
}
