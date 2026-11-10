use std::net::IpAddr;

use futures::future::join_all;
use netdev::ipnet::Ipv4Net;

use super::nearby::NearbyDevice;
use super::{Shared, probe};
use crate::route;

/// However wide a network is, only the 254 hosts around this device are asked.
const WIDEST_PREFIX: u8 = 24;

/// Asks every host on this device's local networks whether it runs solferry, for networks that
/// drop the multicast mDNS needs.
pub(super) async fn scan(shared: &Shared) -> Vec<NearbyDevice> {
    let scans = route::lan_networks().into_iter().filter_map(|net| {
        let own = net.addr();
        let nearest = Ipv4Net::new(own, net.prefix_len().max(WIDEST_PREFIX)).ok()?.trunc();
        let hosts: Vec<IpAddr> =
            nearest.hosts().filter(|host| *host != own).map(IpAddr::V4).collect();
        Some(async move { probe::ask(shared, own.into(), &hosts, probe::ANSWER_WAIT).await })
    });
    join_all(scans).await.into_iter().flatten().collect()
}
