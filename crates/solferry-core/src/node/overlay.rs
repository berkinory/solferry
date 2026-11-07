use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use iroh::{EndpointAddr, EndpointId, TransportAddr};
use serde::Deserialize;
use tokio::net::UdpSocket;
use tokio::process::Command;

use super::Shared;
use super::nearby::NearbyDevice;
use crate::config::Visibility;
use crate::route;
use crate::transfer::paths::clean_label;

/// Answers only on the overlay address, so nothing listens on the internet or the local network.
const PORT: u16 = 53319;
const QUESTION: &[u8] = b"solferry-overlay-1";
const QUESTION_LEN: usize = QUESTION.len();
const ANSWER_WAIT: Duration = Duration::from_millis(800);
const ID_LEN: usize = 32;
const MAX_ANSWER: usize = 512;
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

fn answer_for(shared: &Shared) -> Vec<u8> {
    let port = shared.endpoint.bound_sockets().first().map_or(0, SocketAddr::port);
    let mut answer = shared.endpoint.id().as_bytes().to_vec();
    answer.extend_from_slice(&port.to_be_bytes());
    answer.extend_from_slice(shared.config.name.as_bytes());
    answer.truncate(MAX_ANSWER);
    answer
}

/// Lets other solferry devices on this overlay find this one, for as long as the node runs.
pub(super) fn answer_probes(shared: Arc<Shared>) {
    if shared.config.visibility != Visibility::Everyone {
        return;
    }
    let Some(own) = route::overlay_address() else {
        return;
    };
    tokio::spawn(async move {
        let Ok(socket) = UdpSocket::bind((own, PORT)).await else {
            return;
        };
        let answer = answer_for(&shared);
        let mut buffer = [0; QUESTION.len()];
        loop {
            tokio::select! {
                () = shared.endpoint.closed() => return,
                received = socket.recv_from(&mut buffer) => {
                    if let Ok((QUESTION_LEN, from)) = received
                        && buffer == QUESTION
                        && route::is_overlay(from.ip())
                    {
                        let _ = socket.send_to(&answer, from).await;
                    }
                }
            }
        }
    });
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

fn parse_answer(answer: &[u8], from: SocketAddr) -> Option<(EndpointId, EndpointAddr, String)> {
    let (id, rest) = answer.split_first_chunk::<ID_LEN>()?;
    let (port, name) = rest.split_first_chunk::<2>()?;
    let id = EndpointId::from_bytes(id).ok()?;
    let address = SocketAddr::new(from.ip(), u16::from_be_bytes(*port));
    let addr = EndpointAddr::from_parts(id, [TransportAddr::Ip(address)]);
    Some((id, addr, clean_label(&String::from_utf8_lossy(name))))
}

/// Asks every online overlay peer whether it runs solferry. Identity is proven when connecting, so a
/// wrong answer can only add a device that then fails the same checks as any stranger.
pub(super) async fn scan(shared: &Shared) -> Vec<NearbyDevice> {
    let Some(own) = route::overlay_address() else {
        return Vec::new();
    };
    let peers = online_peers().await;
    let Ok(socket) = UdpSocket::bind((own, 0)).await else {
        return Vec::new();
    };
    for peer in peers {
        let _ = socket.send_to(QUESTION, (peer, PORT)).await;
    }
    let mut found = Vec::new();
    let mut buffer = [0; MAX_ANSWER];
    let collect = async {
        loop {
            let Ok((length, from)) = socket.recv_from(&mut buffer).await else { return };
            let Some((id, addr, name)) = parse_answer(&buffer[..length], from) else { continue };
            if id != shared.endpoint.id() {
                found.push(NearbyDevice { addr, name: Some(name), known: false, overlay: true });
            }
        }
    };
    let _ = tokio::time::timeout(ANSWER_WAIT, collect).await;
    found
}
