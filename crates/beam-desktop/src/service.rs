use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use beam_core::{
    Config, Device, Error, Event, IncomingOffer, NearbyDevice, Node, NodeOptions, OfferSummary,
    PairingRequest, RelayHealth, TransferId,
};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures::{SinkExt, StreamExt};
use iroh::{EndpointAddr, EndpointId};
use tokio::sync::Mutex;
use tokio::task::AbortHandle;

use crate::notify::ArrivalNotices;

const ONLINE_WAIT: Duration = Duration::from_secs(5);

pub struct Target {
    pub addr: EndpointAddr,
    pub name: String,
}

pub enum Command {
    Send { targets: Vec<Target>, paths: Vec<PathBuf> },
    Cancel(TransferId),
    Answer { offer: u64, accept: bool },
    BlockOffer(u64),
    PairNearby { addr: EndpointAddr, auto_accept: bool },
    AnswerPairing { key: u64, accept: bool, auto_accept: bool },
    Invite { auto_accept: bool },
    Join { code: String, auto_accept: bool },
    Forget(EndpointId),
    SetAutoAccept { id: EndpointId, enabled: bool },
    Unblock(EndpointId),
    Apply(Config),
    Quit,
}

pub enum Update {
    Ready { id: EndpointId, config: Box<Config>, devices: Vec<Device>, blocked: Vec<EndpointId> },
    StartFailed(Error),
    Devices(Vec<Device>),
    Blocked(Vec<EndpointId>),
    RelayHealth(Vec<RelayHealth>),
    Nearby(Vec<NearbyDevice>),
    Transfer(Event),
    Offer { key: u64, summary: OfferSummary },
    PairingRequest { key: u64, name: String, code: String },
    PairingCode(String),
    Invite(String),
    Paired(Device),
    PairingFailed(Error),
    SendFailed { name: String, addr: EndpointAddr, paths: Vec<PathBuf>, error: Error },
}

pub struct Handle {
    pub commands: UnboundedSender<Command>,
}

impl Handle {
    pub fn send(&self, command: Command) {
        let _ = self.commands.unbounded_send(command);
    }
}

pub fn spawn(data_dir: PathBuf) -> (Handle, UnboundedReceiver<Update>) {
    let (commands, command_queue) = unbounded();
    let (updates, update_queue) = unbounded();
    std::thread::Builder::new()
        .name("beam-core".to_owned())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            runtime.block_on(run(data_dir, command_queue, updates));
        })
        .expect("core thread");
    (Handle { commands }, update_queue)
}

type Pending = Arc<Mutex<HashMap<u64, IncomingOffer>>>;
type PendingPairings = Arc<Mutex<HashMap<u64, PairingRequest>>>;

struct Session {
    node: Node,
    tasks: Vec<AbortHandle>,
}

async fn start(
    data_dir: &Path,
    updates: &UnboundedSender<Update>,
    pending: &Pending,
    pairing: &PendingPairings,
) -> Option<Session> {
    let options = NodeOptions { data_dir: data_dir.to_owned(), isolated: false };
    let (node, mut offers) = match Node::start(options).await {
        Ok(started) => started,
        Err(error) => {
            let _ = updates.unbounded_send(Update::StartFailed(error));
            return None;
        }
    };
    let ready = Update::Ready {
        id: node.id(),
        config: Box::new(node.config().clone()),
        devices: node.devices(),
        blocked: node.blocked(),
    };
    let _ = updates.unbounded_send(ready);

    let mut tasks = Vec::new();
    let mut events = node.events();
    let outgoing = updates.clone();
    tasks.push(
        tokio::spawn(async move {
            let mut notices = ArrivalNotices::default();
            loop {
                match events.recv().await {
                    Ok(event) => {
                        notices.observe(&event);
                        let _ = outgoing.unbounded_send(Update::Transfer(event));
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        })
        .abort_handle(),
    );

    let waiting = pending.clone();
    let outgoing = updates.clone();
    let downloads = node.config().download_dir.clone();
    tasks.push(
        tokio::spawn(async move {
            let mut next = 0;
            while let Some(offer) = offers.recv().await {
                if offer.summary.auto_accept {
                    offer.accept(downloads.clone());
                    continue;
                }
                next += 1;
                let summary = offer.summary.clone();
                waiting.lock().await.insert(next, offer);
                let _ = outgoing.unbounded_send(Update::Offer { key: next, summary });
            }
        })
        .abort_handle(),
    );

    let checker = node.clone();
    let outgoing = updates.clone();
    tasks.push(
        tokio::spawn(async move {
            let health = checker.relay_health(ONLINE_WAIT).await;
            let _ = outgoing.unbounded_send(Update::RelayHealth(health));
        })
        .abort_handle(),
    );

    let mut requests = node.pairing_requests();
    let waiting = pairing.clone();
    let outgoing = updates.clone();
    tasks.push(
        tokio::spawn(async move {
            let mut next = 0;
            while let Some(request) = requests.recv().await {
                next += 1;
                let (name, code) = (request.peer_name.clone(), request.code.clone());
                waiting.lock().await.insert(next, request);
                let _ = outgoing.unbounded_send(Update::PairingRequest { key: next, name, code });
            }
        })
        .abort_handle(),
    );

    let scanner = node.clone();
    let outgoing = updates.clone();
    tasks.push(
        tokio::spawn(async move {
            scanner
                .follow_nearby(|found| {
                    let _ = outgoing.unbounded_send(Update::Nearby(found));
                })
                .await;
        })
        .abort_handle(),
    );
    Some(Session { node, tasks })
}

async fn run(
    data_dir: PathBuf,
    mut commands: UnboundedReceiver<Command>,
    updates: UnboundedSender<Update>,
) {
    let pending: Pending = Arc::default();
    let pairing: PendingPairings = Arc::default();
    let mut session = start(&data_dir, &updates, &pending, &pairing).await;
    while let Some(command) = commands.next().await {
        match command {
            Command::Quit => break,
            Command::Apply(config) => {
                if let Some(active) = session.take() {
                    stop(active).await;
                }
                if let Err(error) = config.save(&data_dir) {
                    let _ = updates.unbounded_send(Update::StartFailed(error));
                }
                session = start(&data_dir, &updates, &pending, &pairing).await;
            }
            Command::AnswerPairing { key, accept, auto_accept } => {
                let Some(request) = pairing.lock().await.remove(&key) else { continue };
                if accept { request.accept(auto_accept) } else { request.decline() }
            }
            Command::Answer { offer, accept } => {
                let Some(offer) = pending.lock().await.remove(&offer) else { continue };
                match (&session, accept) {
                    (Some(active), true) => offer.accept(active.node.config().download_dir.clone()),
                    _ => offer.decline(),
                }
            }
            Command::BlockOffer(offer) => {
                let Some(offer) = pending.lock().await.remove(&offer) else { continue };
                let peer = offer.summary.peer;
                offer.decline();
                if let Some(active) = &session
                    && active.node.block(peer).is_ok()
                {
                    let _ = updates.unbounded_send(Update::Devices(active.node.devices()));
                    let _ = updates.unbounded_send(Update::Blocked(active.node.blocked()));
                }
            }
            other => {
                if let Some(active) = &session {
                    dispatch(&active.node, other, &updates);
                }
            }
        }
    }
    if let Some(active) = session.take() {
        stop(active).await;
    }
}

async fn stop(session: Session) {
    for task in session.tasks {
        task.abort();
    }
    let _ = session.node.shutdown().await;
}

fn dispatch(node: &Node, command: Command, updates: &UnboundedSender<Update>) {
    let mut updates = updates.clone();
    let node = node.clone();
    tokio::spawn(async move {
        match command {
            Command::Send { targets, paths } => {
                for Target { addr, name } in targets {
                    let (node, paths, mut updates) = (node.clone(), paths.clone(), updates.clone());
                    tokio::spawn(async move {
                        if let Err(error) = node.send(addr.clone(), paths.clone()).await {
                            let failed = Update::SendFailed { name, addr, paths, error };
                            let _ = updates.send(failed).await;
                        }
                    });
                }
            }
            Command::Cancel(id) => node.cancel(id),
            Command::PairNearby { addr, auto_accept } => {
                let shown = updates.clone();
                let show = move |code: &str| {
                    let _ = shown.unbounded_send(Update::PairingCode(code.to_owned()));
                };
                let result = match node.pair_nearby(addr, auto_accept, show).await {
                    Ok(device) => Update::Paired(device),
                    Err(error) => Update::PairingFailed(error),
                };
                let _ = updates.send(result).await;
                let _ = updates.send(Update::Devices(node.devices())).await;
            }
            Command::Invite { auto_accept } => {
                node.online_within(ONLINE_WAIT).await;
                let code = node.invite(auto_accept).code().to_owned();
                let _ = updates.send(Update::Invite(code)).await;
            }
            Command::Join { code, auto_accept } => {
                let result = match node.join(&code, auto_accept).await {
                    Ok(device) => Update::Paired(device),
                    Err(error) => Update::PairingFailed(error),
                };
                let _ = updates.send(result).await;
                let _ = updates.send(Update::Devices(node.devices())).await;
            }
            Command::Forget(id) => {
                let _ = node.forget(&id.to_z32());
                let _ = updates.send(Update::Devices(node.devices())).await;
            }
            Command::SetAutoAccept { id, enabled } => {
                let _ = node.set_auto_accept(id, enabled);
                let _ = updates.send(Update::Devices(node.devices())).await;
            }
            Command::Unblock(id) => {
                let _ = node.unblock(id);
                let _ = updates.send(Update::Blocked(node.blocked())).await;
            }
            Command::Answer { .. }
            | Command::AnswerPairing { .. }
            | Command::BlockOffer(_)
            | Command::Apply(_)
            | Command::Quit => {}
        }
    });
}
