use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::process::Stdio;

use serde::Deserialize;
use tokio::process::Command;

use super::nearby::NearbyDevice;
use super::{Shared, probe};
use crate::route;

const TAILSCALE_PATHS: [&str; 3] = [
    "tailscale",
    "/Applications/Tailscale.app/Contents/MacOS/Tailscale",
    r"C:\Program Files\Tailscale\tailscale.exe",
];

/// Without this, a desktop app flashes a console window on every scan.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Deserialize)]
struct Status {
    #[serde(rename = "Peer", default)]
    peers: HashMap<String, Peer>,
}

#[derive(Deserialize)]
struct Peer {
    #[serde(rename = "TailscaleIPs", default)]
    addresses: Vec<IpAddr>,
    #[serde(rename = "Online", default)]
    online: bool,
}

async fn online_peers() -> Vec<Ipv4Addr> {
    for program in TAILSCALE_PATHS {
        let mut command = Command::new(program);
        command.args(["status", "--json"]).stdin(Stdio::null()).stderr(Stdio::null());
        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);
        let status = command.output().await;
        let Ok(output) = status else { continue };
        let Ok(status) = serde_json::from_slice::<Status>(&output.stdout) else { continue };
        return status
            .peers
            .into_values()
            .filter(|peer| peer.online)
            .filter_map(|peer| {
                peer.addresses.into_iter().find_map(|address| match address {
                    IpAddr::V4(address) => Some(address),
                    IpAddr::V6(_) => None,
                })
            })
            .collect();
    }
    Vec::new()
}

/// Asks every online overlay peer whether it runs solferry.
pub(super) async fn scan(shared: &Shared) -> Vec<NearbyDevice> {
    let Some(own) = route::overlay_address() else {
        return Vec::new();
    };
    let peers: Vec<IpAddr> = online_peers().await.into_iter().map(IpAddr::V4).collect();
    probe::ask(shared, own.into(), &peers, probe::ANSWER_WAIT).await
}
