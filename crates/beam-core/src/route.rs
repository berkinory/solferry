use std::net::IpAddr;
use std::time::{Duration, Instant};

use futures::StreamExt;
use iroh::endpoint::{Connection, Path, PathList};
use iroh::{RelayUrl, TransportAddr};

use crate::error::{Error, Result};

const RELAY_GRACE: Duration = Duration::from_secs(20);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Local,
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
        TransportAddr::Ip(addr) if is_local(addr.ip()) => Route::Local,
        TransportAddr::Relay(_) => Route::Relay,
        _ => Route::Direct,
    }
}

pub(crate) fn is_local(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(ip) => {
            let carrier_grade = ip.octets()[0] == 100 && (ip.octets()[1] & 0xc0) == 64;
            ip.is_private() || ip.is_loopback() || ip.is_link_local() || carrier_grade
        }
        IpAddr::V6(ip) => ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local(),
    }
}

pub(crate) async fn wait_for_route(
    connection: &Connection,
    rule: &RouteRule,
    limit: Duration,
) -> Result<Route> {
    let mut snapshots = connection.paths_stream();
    let wait = async {
        while let Some(paths) = snapshots.next().await {
            if let Some(route) = rule.evaluate(&paths) {
                return Ok(route);
            }
        }
        Err(Error::ConnectionLost)
    };
    tokio::time::timeout(limit, wait)
        .await
        .unwrap_or(Err(Error::NoDirectPath))
}

pub(crate) async fn enforce_route(connection: &Connection, rule: &RouteRule) -> Error {
    let mut snapshots = connection.paths_stream();
    let mut violated_at: Option<Instant> = None;
    loop {
        let wait = violated_at.map_or(RELAY_GRACE, |since| {
            RELAY_GRACE.saturating_sub(since.elapsed())
        });
        match tokio::time::timeout(wait, snapshots.next()).await {
            Ok(Some(paths)) if rule.evaluate(&paths).is_some() => violated_at = None,
            Ok(Some(_)) => violated_at = violated_at.or_else(|| Some(Instant::now())),
            Ok(None) => return Error::ConnectionLost,
            Err(_) if violated_at.is_some() => return Error::NoDirectPath,
            Err(_) => {}
        }
    }
}
