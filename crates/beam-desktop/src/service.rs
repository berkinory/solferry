use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use beam_core::{
    Config, Device, Error, Event, IncomingOffer, NearbyDevice, Node, NodeOptions, OfferSummary,
    Relationship, RelayPolicy, TransferId,
};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures::{SinkExt, StreamExt};
use iroh::{EndpointAddr, EndpointId};
use tokio::sync::Mutex;
use tokio::task::AbortHandle;

const ONLINE_WAIT: Duration = Duration::from_secs(5);

pub struct Target {
    pub addr: EndpointAddr,
    pub name: String,
}

pub enum Command {
    Send { targets: Vec<Target>, paths: Vec<PathBuf> },
    Cancel(TransferId),
    Answer { offer: u64, accept: bool },
    Invite(Relationship),
    Join(String),
    Forget(EndpointId),
    Apply(Config),
    Quit,
}

pub enum Update {
    Ready { id: EndpointId, config: Box<Config>, devices: Vec<Device> },
    StartFailed(Error),
    Devices(Vec<Device>),
    Nearby(Vec<NearbyDevice>),
    Transfer(Event),
    Offer { key: u64, summary: OfferSummary },
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

struct Session {
    node: Node,
    tasks: Vec<AbortHandle>,
}

async fn start(
    data_dir: &Path,
    updates: &UnboundedSender<Update>,
    pending: &Pending,
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
    };
    let _ = updates.unbounded_send(ready);

    let mut tasks = Vec::new();
    let mut events = node.events();
    let outgoing = updates.clone();
    tasks.push(
        tokio::spawn(async move {
            loop {
                match events.recv().await {
                    Ok(event) => {
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
                if offer.summary.relationship == Some(Relationship::Own) {
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
    let mut session = start(&data_dir, &updates, &pending).await;
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
                session = start(&data_dir, &updates, &pending).await;
            }
            Command::Answer { offer, accept } => {
                let Some(offer) = pending.lock().await.remove(&offer) else { continue };
                match (&session, accept) {
                    (Some(active), true) => offer.accept(active.node.config().download_dir.clone()),
                    _ => offer.decline(),
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
            Command::Invite(relationship) => {
                if node.config().relay_policy != RelayPolicy::Off {
                    node.online_within(ONLINE_WAIT).await;
                }
                let _ =
                    updates.send(Update::Invite(node.invite(relationship).code().to_owned())).await;
            }
            Command::Join(code) => {
                let result = match node.join(&code).await {
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
            Command::Answer { .. } | Command::Apply(_) | Command::Quit => {}
        }
    });
}
