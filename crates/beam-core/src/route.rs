use std::net::IpAddr;

use iroh::TransportAddr;
use iroh::endpoint::{Connection, Path};

/// Interface name prefixes of overlay clients. Unknown names fail closed: carrier interfaces use
/// the same addresses, and paired devices do not depend on this check.
const TUNNEL_PREFIXES: [&str; 5] = ["tailscale", "utun", "tun", "wg", "zt"];

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
fn is_overlay(ip: IpAddr) -> bool {
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

fn joined_overlay() -> bool {
    netdev::get_interfaces().iter().any(|interface| {
        let name = interface.friendly_name.as_deref().unwrap_or(&interface.name).to_lowercase();
        TUNNEL_PREFIXES.iter().any(|prefix| name.starts_with(prefix))
            && interface.ipv4.iter().any(|net| is_overlay(IpAddr::V4(net.addr())))
    })
}

/// Whether an unpaired device at `ip` may ask to send files: never across a relay or the internet.
pub(crate) fn shares_network(ip: IpAddr) -> bool {
    is_local(ip) || (is_overlay(ip) && joined_overlay())
}

/// The route file data is using right now.
pub(crate) fn current_route(connection: &Connection) -> Route {
    let paths = connection.paths();
    paths.iter().find(Path::is_selected).map_or(Route::Direct, |path| classify(&path))
}
