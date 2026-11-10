use std::collections::{HashMap, HashSet};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use futures::future::join_all;
use iroh::{EndpointAddr, EndpointId, TransportAddr};
use tokio::net::UdpSocket;
use tokio::task::JoinHandle;

use super::Shared;
use super::nearby::NearbyDevice;
use crate::config::Visibility;
use crate::route;
use crate::transfer::paths::clean_label;

/// Answers only on overlay and private LAN addresses, never on a public one.
const PORT: u16 = 53319;
const QUESTION: &[u8] = b"solferry-overlay-1";
const QUESTION_LEN: usize = QUESTION.len();
pub(super) const ANSWER_WAIT: Duration = Duration::from_millis(800);
const ID_LEN: usize = 32;
const MAX_ANSWER: usize = 512;
/// Questions to hosts nobody has resolved yet wait in the kernel, charged to the socket that sent
/// them. Past about a hundred, a Linux socket stops sending until those lookups fail seconds later.
const TARGETS_PER_SOCKET: usize = 32;
/// Addresses come and go with Wi-Fi networks, so the listening sockets follow them.
const REBIND_EVERY: Duration = Duration::from_secs(20);

fn answer_for(shared: &Shared) -> Vec<u8> {
    let port = shared.endpoint.bound_sockets().first().map_or(0, SocketAddr::port);
    let mut answer = shared.endpoint.id().as_bytes().to_vec();
    answer.extend_from_slice(&port.to_be_bytes());
    answer.extend_from_slice(crate::platform::label(&shared.config.name).as_bytes());
    answer.truncate(MAX_ANSWER);
    answer
}

fn parse_answer(answer: &[u8], from: SocketAddr) -> Option<(EndpointId, EndpointAddr, String)> {
    let (id, rest) = answer.split_first_chunk::<ID_LEN>()?;
    let (port, name) = rest.split_first_chunk::<2>()?;
    let id = EndpointId::from_bytes(id).ok()?;
    let address = SocketAddr::new(from.ip(), u16::from_be_bytes(*port));
    let addr = EndpointAddr::from_parts(id, [TransportAddr::Ip(address)]);
    Some((id, addr, clean_label(&String::from_utf8_lossy(name))))
}

/// Lets other solferry devices on this overlay or local network find this one, for as long as the
/// node runs.
pub(super) fn answer_probes(shared: Arc<Shared>) {
    if shared.config.visibility != Visibility::Everyone {
        return;
    }
    tokio::spawn(async move {
        let answer: Arc<[u8]> = answer_for(&shared).into();
        let mut listening: HashMap<Ipv4Addr, JoinHandle<()>> = HashMap::new();
        let mut rebind = tokio::time::interval(REBIND_EVERY);
        loop {
            tokio::select! {
                () = shared.endpoint.closed() => break,
                _ = rebind.tick() => {
                    let lan = route::lan_networks().into_iter().map(|net| net.addr());
                    let wanted: HashSet<Ipv4Addr> = lan.chain(route::overlay_address()).collect();
                    listening.retain(|own, task| {
                        let keep = wanted.contains(own) && !task.is_finished();
                        if !keep {
                            task.abort();
                        }
                        keep
                    });
                    for own in wanted {
                        let answer = answer.clone();
                        listening.entry(own).or_insert_with(|| tokio::spawn(answer_on(own, answer)));
                    }
                }
            }
        }
        for task in listening.into_values() {
            task.abort();
        }
    });
}

async fn answer_on(own: Ipv4Addr, answer: Arc<[u8]>) {
    let Ok(socket) = UdpSocket::bind((own, PORT)).await else {
        return;
    };
    let mut buffer = [0; QUESTION_LEN];
    loop {
        if let Ok((QUESTION_LEN, from)) = socket.recv_from(&mut buffer).await
            && buffer == QUESTION
            && route::shares_network(from.ip())
        {
            let _ = socket.send_to(&answer, from).await;
        }
    }
}

/// Asks each target whether it runs solferry, sending from `own`, and returns once every target
/// answered or `wait` ran out. Identity is proven when connecting, so a wrong answer can only add
/// a device that then fails the same checks as any stranger.
pub(super) async fn ask(
    shared: &Shared,
    own: IpAddr,
    targets: &[IpAddr],
    wait: Duration,
) -> Vec<NearbyDevice> {
    let groups =
        targets.chunks(TARGETS_PER_SOCKET).map(|group| ask_group(shared, own, group, wait));
    join_all(groups).await.into_iter().flatten().collect()
}

async fn ask_group(
    shared: &Shared,
    own: IpAddr,
    targets: &[IpAddr],
    wait: Duration,
) -> Vec<NearbyDevice> {
    let Ok(socket) = UdpSocket::bind((own, 0)).await else {
        return Vec::new();
    };
    let mut found = Vec::new();
    let mut answered = HashSet::new();
    let mut buffer = [0; MAX_ANSWER];
    let exchange = async {
        for target in targets {
            let _ = socket.send_to(QUESTION, (*target, PORT)).await;
        }
        while answered.len() < targets.len() {
            let Ok((length, from)) = socket.recv_from(&mut buffer).await else { return };
            let Some((id, addr, name)) = parse_answer(&buffer[..length], from) else { continue };
            if id != shared.endpoint.id() && answered.insert(from.ip()) {
                let (platform, name) = crate::platform::split_label(&name);
                let name = Some(name.to_owned());
                let overlay = route::is_overlay(from.ip());
                found.push(NearbyDevice { addr, name, known: false, overlay, platform });
            }
        }
    };
    let _ = tokio::time::timeout(wait, exchange).await;
    found
}
