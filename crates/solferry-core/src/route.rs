use std::net::{IpAddr, Ipv4Addr};

use futures::StreamExt;
use iroh::endpoint::{Connection, Path};
use iroh::{EndpointAddr, TransportAddr};
use netdev::ipnet::Ipv4Net;
use tokio::sync::broadcast;

use crate::event::{Event, TransferId};

/// Interface name prefixes of overlay clients. Unknown names fail closed: carrier interfaces use
/// the same addresses, and paired devices do not depend on this check.
const TUNNEL_PREFIXES: [&str; 5] = ["tailscale", "utun", "tun", "wg", "zt"];

/// Interface name prefixes of containers and virtual machines. Their addresses mean nothing to
/// another device, and leaving them out keeps pairing codes short enough for a QR code.
const VIRTUAL_PREFIXES: [&str; 11] = [
    "docker",
    "br-",
    "virbr",
    "veth",
    "vmnet",
    "vboxnet",
    "podman",
    "cni",
    "flannel",
    "lxc",
    "vethernet",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Route {
    Local,
    Overlay,
    Direct,
    Relay,
}

fn classify(path: &Path<'_>) -> Route {
    match path.remote_addr() {
        TransportAddr::Ip(addr) if is_overlay(addr.ip()) => Route::Overlay,
        TransportAddr::Ip(addr) if is_local(addr.ip()) => Route::Local,
        TransportAddr::Relay(_) => Route::Relay,
        _ => Route::Direct,
    }
}

/// Addresses Tailscale and similar overlays hand out. Carrier NAT shares the IPv4 block.
pub(crate) fn is_overlay(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(ip) => ip.octets()[0] == 100 && (ip.octets()[1] & 0xc0) == 64,
        IpAddr::V6(ip) => ip.segments()[..3] == [0xfd7a, 0x115c, 0xa1e0],
    }
}

fn is_local(ip: IpAddr) -> bool {
    if is_overlay(ip) {
        return false;
    }
    match ip.to_canonical() {
        IpAddr::V4(ip) => ip.is_private() || ip.is_loopback() || ip.is_link_local(),
        IpAddr::V6(ip) => ip.is_loopback() || ip.is_unique_local() || ip.is_unicast_link_local(),
    }
}

pub(crate) fn is_public(ip: IpAddr) -> bool {
    !is_local(ip) && !is_overlay(ip)
}

fn named(interface: &netdev::Interface, prefixes: &[&str]) -> bool {
    let name = interface.friendly_name.as_deref().unwrap_or(&interface.name).to_lowercase();
    prefixes.iter().any(|prefix| name.starts_with(prefix))
}

/// This device's address on an overlay it joined, if any.
pub(crate) fn overlay_address() -> Option<Ipv4Addr> {
    netdev::get_interfaces().iter().find_map(|interface| {
        if !named(interface, &TUNNEL_PREFIXES) {
            return None;
        }
        interface.ipv4.iter().find_map(|net| {
            let ip = net.addr();
            is_overlay(ip.into()).then_some(ip)
        })
    })
}

/// This device's private IPv4 networks, leaving out overlays, containers and virtual machines.
pub(crate) fn lan_networks() -> Vec<Ipv4Net> {
    netdev::get_interfaces()
        .into_iter()
        .filter(|interface| interface.is_up() && !interface.is_loopback())
        .filter(|interface| {
            !named(interface, &TUNNEL_PREFIXES) && !named(interface, &VIRTUAL_PREFIXES)
        })
        .flat_map(|interface| interface.ipv4)
        .filter(|net| net.addr().is_private())
        .collect()
}

/// Whether an unpaired device at `ip` may ask to send files: never across a relay or the internet.
pub(crate) fn shares_network(ip: IpAddr) -> bool {
    is_local(ip) || (is_overlay(ip) && overlay_address().is_some())
}

/// Whether `addr` lists an address inside a subnet this device is on right now. Link-local
/// addresses are left out: every interface shares their prefix, so they prove nothing.
pub(crate) fn on_local_network(addr: &EndpointAddr) -> bool {
    let interfaces = netdev::get_interfaces();
    addr.ip_addrs().any(|target| match target.ip().to_canonical() {
        IpAddr::V4(ip) if ip.is_private() => {
            interfaces.iter().any(|interface| interface.ipv4.iter().any(|net| net.contains(&ip)))
        }
        IpAddr::V6(ip) if ip.is_unique_local() => {
            interfaces.iter().any(|interface| interface.ipv6.iter().any(|net| net.contains(&ip)))
        }
        _ => false,
    })
}

/// The route file data is using right now.
pub(crate) fn current_route(connection: &Connection) -> Route {
    let paths = connection.paths();
    paths.iter().find(Path::is_selected).map_or(Route::Direct, |path| classify(&path))
}

/// Tells listeners when a running transfer moves, for example from a relay to a direct path.
/// Runs until the caller drops it, and borrows so a dead connection can still close.
pub(crate) async fn report_changes(
    connection: &Connection,
    id: TransferId,
    mut route: Route,
    events: &broadcast::Sender<Event>,
) {
    let mut snapshots = connection.paths_stream();
    while let Some(paths) = snapshots.next().await {
        let selected = paths.iter().find(Path::is_selected).map(|path| classify(&path));
        if let Some(current) = selected.filter(|current| *current != route) {
            route = current;
            let _ = events.send(Event::RouteChanged { id, route });
        }
    }
    std::future::pending::<()>().await;
}

/// This device's own addresses that are worth giving to another device.
pub(crate) fn shareable_addrs(addrs: impl Iterator<Item = std::net::SocketAddr>) -> Vec<String> {
    let interfaces = netdev::get_interfaces();
    let is_virtual = |ip: IpAddr| {
        interfaces.iter().any(|interface| {
            named(interface, &VIRTUAL_PREFIXES)
                && (interface.ipv4.iter().any(|net| IpAddr::V4(net.addr()) == ip)
                    || interface.ipv6.iter().any(|net| IpAddr::V6(net.addr()) == ip))
        })
    };
    addrs.filter(|addr| !is_virtual(addr.ip())).map(|addr| addr.to_string()).collect()
}
