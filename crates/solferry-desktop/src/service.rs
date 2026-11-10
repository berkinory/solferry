use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures::{SinkExt, StreamExt};
use iroh::{EndpointAddr, EndpointId};
use solferry_core::{
    BlockedDevice, Config, Device, Error, Event, IncomingOffer, NearbyDevice, Node, NodeOptions,
    OfferSummary, PairingRequest, PastTransfer, Platform, RelayHealth, TransferId,
};
use tokio::sync::Mutex;
use tokio::task::AbortHandle;

use crate::updater::Updater;
use crate::watch;

const ONLINE_WAIT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct Target {
    pub addr: EndpointAddr,
    pub name: String,
}

pub enum Command {
    Send { targets: Vec<Target>, paths: Vec<PathBuf> },
    SendText { target: Target, text: String },
    Cancel(TransferId),
    Answer { offer: u64, accept: bool },
    AcceptAlways(u64),
    BlockOffer(u64),
    PairNearby { addr: EndpointAddr },
    PairAddress { address: String },
    AnswerPairing { key: u64, accept: bool },
    Invite,
    Join { code: String },
    Forget(EndpointId),
    SetAutoAccept { id: EndpointId, enabled: bool },
    Unblock(EndpointId),
    ClearHistory,
    Apply(Config),
    CheckForUpdates,
    InstallUpdate,
    SetCheckUpdates(bool),
    Quit,
}

pub enum Update {
    Updater(crate::updater::Status),
    InstallFailed,
    Restarting,
    Ready {
        id: EndpointId,
        config: Box<Config>,
        devices: Vec<Device>,
        blocked: Vec<BlockedDevice>,
    },
    StartFailed(Error),
    Devices(Vec<Device>),
    Blocked(Vec<BlockedDevice>),
    RelayHealth(Vec<RelayHealth>),
    Nearby(Vec<NearbyDevice>),
    Reachable(HashSet<EndpointId>),
    Transfer(Event),
    History(Vec<PastTransfer>),
    Offer {
        key: u64,
        summary: OfferSummary,
    },
    OfferGone(u64),
    PairingRequest {
        key: u64,
        peer: EndpointId,
        platform: Option<Platform>,
        name: String,
        code: String,
    },
    PairingGone(u64),
    PairingCode(String),
    Invite(String),
    Paired(Device),
    PairingFailed(Error),
    AddressFailed(Error),
    SendFailed {
        name: String,
        addr: EndpointAddr,
        paths: Vec<PathBuf>,
        error: Error,
    },
    TextShared {
        name: String,
        result: Result<(), Error>,
    },
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
        .name("solferry-core".to_owned())
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

pub type Pending = Arc<Mutex<HashMap<u64, IncomingOffer>>>;
pub type PendingPairings = Arc<Mutex<HashMap<u64, PairingRequest>>>;

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
    let _ = updates.unbounded_send(Update::History(node.history()));

    let mut tasks = Vec::new();
    let mut events = node.events();
    let (outgoing, recorder) = (updates.clone(), node.clone());
    tasks.push(
        tokio::spawn(async move {
            loop {
                match events.recv().await {
                    Ok(event) => {
                        let over = matches!(event, Event::Completed { .. } | Event::Failed { .. });
                        let _ = outgoing.unbounded_send(Update::Transfer(event));
                        if over {
                            let _ = outgoing.unbounded_send(Update::History(recorder.history()));
                        }
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
                let (peer, platform) = (request.peer, request.platform);
                waiting.lock().await.insert(next, request);
                let prompt = Update::PairingRequest { key: next, peer, platform, name, code };
                let _ = outgoing.unbounded_send(prompt);
            }
        })
        .abort_handle(),
    );

    tasks.push(watch::sweep_prompts(pending.clone(), pairing.clone(), updates.clone()));

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
    tasks.push(watch::follow_presence(node.clone(), updates.clone()));
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
    let releases = Updater::new(data_dir.clone(), updates.clone());
    releases.follow();
    while let Some(command) = commands.next().await {
        match command {
            Command::Quit => break,
            Command::CheckForUpdates => releases.check_now(),
            Command::InstallUpdate => releases.install(),
            Command::SetCheckUpdates(enabled) => {
                if let Ok(config) = Config::load(&data_dir) {
                    let _ = Config { check_updates: enabled, ..config }.save(&data_dir);
                }
            }
            Command::Apply(config) => {
                if let Some(active) = session.take() {
                    stop(active).await;
                }
                if let Err(error) = config.save(&data_dir) {
                    let _ = updates.unbounded_send(Update::StartFailed(error));
                }
                session = start(&data_dir, &updates, &pending, &pairing).await;
            }
            Command::AnswerPairing { key, accept } => {
                let Some(request) = pairing.lock().await.remove(&key) else { continue };
                if accept { request.accept() } else { request.decline() }
            }
            Command::AcceptAlways(offer) => {
                let Some(offer) = pending.lock().await.remove(&offer) else { continue };
                let Some(active) = &session else {
                    offer.decline();
                    continue;
                };
                let downloads = active.node.config().download_dir.clone();
                let _ = offer.accept_always(&active.node, downloads);
                let _ = updates.unbounded_send(Update::Devices(active.node.devices()));
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
                let (peer, name) = (offer.summary.peer, offer.summary.peer_name.clone());
                offer.decline();
                if let Some(active) = &session
                    && active.node.block(peer, Some(name)).is_ok()
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
            Command::SendText { target: Target { addr, name }, text } => {
                let result = node.send_text(addr, text).await;
                let _ = updates.send(Update::TextShared { name, result }).await;
            }
            Command::Cancel(id) => node.cancel(id),
            Command::PairNearby { addr } => pair(&node, addr, &mut updates).await,
            Command::PairAddress { address } => match node.find_address(&address).await {
                Ok(device) => pair(&node, device.addr, &mut updates).await,
                Err(error) => {
                    let _ = updates.send(Update::AddressFailed(error)).await;
                }
            },
            Command::Invite => {
                node.online_within(ONLINE_WAIT).await;
                let code = node.invite().code().to_owned();
                let _ = updates.send(Update::Invite(code)).await;
            }
            Command::Join { code } => {
                let result = match node.join(&code).await {
                    Ok(device) => Update::Paired(device),
                    Err(error) => Update::PairingFailed(error),
                };
                let _ = updates.send(result).await;
                let _ = updates.send(Update::Devices(node.devices())).await;
            }
            Command::ClearHistory => {
                node.clear_history();
                let _ = updates.send(Update::History(Vec::new())).await;
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
            | Command::AcceptAlways(_)
            | Command::AnswerPairing { .. }
            | Command::BlockOffer(_)
            | Command::Apply(_)
            | Command::CheckForUpdates
            | Command::InstallUpdate
            | Command::SetCheckUpdates(_)
            | Command::Quit => {}
        }
    });
}

async fn pair(node: &Node, addr: EndpointAddr, updates: &mut UnboundedSender<Update>) {
    let shown = updates.clone();
    let show = move |code: &str| {
        let _ = shown.unbounded_send(Update::PairingCode(code.to_owned()));
    };
    let result = match node.pair_nearby(addr, show).await {
        Ok(device) => Update::Paired(device),
        Err(error) => Update::PairingFailed(error),
    };
    let _ = updates.send(result).await;
    let _ = updates.send(Update::Devices(node.devices())).await;
}
