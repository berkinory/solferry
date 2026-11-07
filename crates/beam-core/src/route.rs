use std::net::IpAddr;
use std::time::{Duration, Instant};

use futures::StreamExt;
use iroh::endpoint::{Connection, Path, PathList};
use iroh::{RelayUrl, TransportAddr};

use crate::error::{Error, Result};

const RELAY_GRACE: Duration = Duration::from_secs(20);
const DIRECT_HEAD_START: Duration = Duration::from_secs(4);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Local,
    Overlay,
    Direct,
    Relay,
}

#[derive(Clone, Debug)]
pub(crate) struct RouteRule {
    pub(crate) any_relay: bool,
    pub(crate) relays: Vec<RelayUrl>,
}

impl RouteRule {
    fn allows(&self, path: &Path<'_>) -> bool {
        match path.remote_addr() {
            TransportAddr::Relay(url) => self.any_relay || self.relays.contains(url),
            _ => true,
        }
    }

    fn evaluate(&self, paths: &PathList<'_>) -> Option<Route> {
        paths
            .iter()
            .find(|path| path.is_selected() && self.allows(path))
            .map(|path| classify(&path))
    }
}

fn classify(path: &Path<'_>) -> Route {
    match path.remote_addr() {
        TransportAddr::Ip(addr) if is_overlay(addr.ip()) => Route::Overlay,
        TransportAddr::Ip(addr) if is_local(addr.ip()) => Route::Local,
        TransportAddr::Relay(_) => Route::Relay,
        _ => Route::Direct,
    }
}

/// Addresses Tailscale and similar overlays hand out. Carrier NAT uses the same IPv4 block, so
/// these must never count as the local network when deciding who to trust.
fn is_overlay(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(ip) => ip.octets()[0] == 100 && (ip.octets()[1] & 0xc0) == 64,
        IpAddr::V6(ip) => ip.segments()[..3] == [0xfd7a, 0x115c, 0xa1e0],
    }
}

pub(crate) fn is_local(ip: IpAddr) -> bool {
    if is_overlay(ip) {
        return false;
    }
    match ip.to_canonical() {
        IpAddr::V4(ip) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
        IpAddr::V6(ip) => ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local(),
    }
}

/// Gives hole punching a head start before settling for an allowed relay.
pub(crate) async fn wait_for_route(
    connection: &Connection,
    rule: &RouteRule,
    limit: Duration,
) -> Result<Route> {
    let mut snapshots = connection.paths_stream();
    let started = Instant::now();
    let mut relay_allowed = false;
    loop {
        let settled = started.elapsed() >= DIRECT_HEAD_START;
        if relay_allowed && settled {
            return Ok(Route::Relay);
        }
        let next_deadline = if settled || relay_allowed { DIRECT_HEAD_START } else { limit };
        let wait = next_deadline.saturating_sub(started.elapsed());
        match tokio::time::timeout(wait, snapshots.next()).await {
            Ok(Some(paths)) => match rule.evaluate(&paths) {
                Some(Route::Relay) => relay_allowed = true,
                Some(route) => return Ok(route),
                None => relay_allowed = false,
            },
            Ok(None) => return Err(Error::ConnectionLost),
            Err(_) if relay_allowed => return Ok(Route::Relay),
            Err(_) => return Err(Error::NoDirectPath),
        }
    }
}

/// The route file data is using right now.
pub(crate) fn current_route(connection: &Connection) -> Route {
    let paths = connection.paths();
    paths.iter().find(Path::is_selected).map_or(Route::Direct, |path| classify(&path))
}

pub(crate) async fn enforce_route(connection: &Connection, rule: &RouteRule) -> Error {
    let mut snapshots = connection.paths_stream();
    let mut violated_at: Option<Instant> = None;
    loop {
        let wait =
            violated_at.map_or(RELAY_GRACE, |since| RELAY_GRACE.saturating_sub(since.elapsed()));
        match tokio::time::timeout(wait, snapshots.next()).await {
            Ok(Some(paths)) if rule.evaluate(&paths).is_some() => violated_at = None,
            Ok(Some(_)) => violated_at = violated_at.or_else(|| Some(Instant::now())),
            Ok(None) => return Error::ConnectionLost,
            Err(_) if violated_at.is_some() => return Error::NoDirectPath,
            Err(_) => {}
        }
    }
}
