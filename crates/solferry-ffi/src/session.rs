use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use iroh::EndpointId;
use solferry_core::{
    Event, IncomingOffer, NearbyDevice, Node, NodeOptions, PairingRequest, RelayHealth, RelayLink,
    TransferId, update,
};
use tokio::sync::broadcast::error::RecvError;
use tokio::task::AbortHandle;

use crate::types::{
    BlockedDevice, HistoryEntry, Incoming, PairedDevice, Relay, RelayStatus, Update, identity,
};
use crate::{Edit, Listener};

const PROMPT_SWEEP: Duration = Duration::from_secs(1);
const RELAY_WAIT: Duration = Duration::from_secs(5);

/// What the bridge remembers between calls: prompts waiting for an answer, what is nearby, and
/// which transfer an id from the app refers to.
#[derive(Default)]
pub struct Shelf {
    pub offers: Mutex<HashMap<u64, IncomingOffer>>,
    pub pairings: Mutex<HashMap<u64, PairingRequest>>,
    pub nearby: Mutex<HashMap<EndpointId, NearbyDevice>>,
    pub transfers: Mutex<HashMap<String, TransferId>>,
    pub link: Mutex<Option<solferry_core::Link>>,
}

pub struct Session {
    pub node: Node,
    tasks: Vec<AbortHandle>,
}

impl Session {
    pub async fn stop(self) {
        for task in self.tasks {
            task.abort();
        }
        let _ = self.node.shutdown().await;
    }
}

/// What restarting the node needs, held apart from the bridge so a running node can ask for it.
#[derive(Clone)]
pub struct Restarter {
    pub runtime: tokio::runtime::Handle,
    pub data_dir: PathBuf,
    pub listener: Arc<dyn Listener>,
    pub shelf: Arc<Shelf>,
    pub session: Arc<tokio::sync::Mutex<Option<Session>>>,
}

impl Restarter {
    /// Stops the node, applies `edit` to the saved settings, and starts it again. The new node
    /// restarts itself the same way once it takes a relay's new token.
    pub fn restart(&self, edit: Option<Edit>) {
        let this = self.clone();
        self.runtime.spawn(async move {
            let mut active = this.session.lock().await;
            if let Some(running) = active.take() {
                running.stop().await;
            }
            if let Some(edit) = edit {
                let mut config = solferry_core::Config::load(&this.data_dir).unwrap_or_default();
                edit(&mut config);
                let _ = config.save(&this.data_dir);
            }
            *active = start(this.data_dir.clone(), &this.shelf, &this.listener).await;
            if let Some(started) = active.as_mut() {
                let (node, again) = (started.node.clone(), this.clone());
                started.tasks.push(spawn(async move {
                    node.wants_restart().await;
                    again.restart(None);
                }));
            }
        });
    }
}

pub fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn devices(node: &Node) -> Update {
    Update::Devices { devices: node.devices().iter().map(PairedDevice::from).collect() }
}

pub fn history(node: &Node) -> Update {
    Update::History { entries: node.history().iter().map(HistoryEntry::from).collect() }
}

pub fn blocked(node: &Node) -> Update {
    let devices = node
        .blocked()
        .into_iter()
        .map(|device| BlockedDevice { identity: identity(&device.id, None), name: device.name });
    Update::Blocked { devices: devices.collect() }
}

/// The relays this device was given, with what is known about each so far.
fn relays(node: &Node, health: &[RelayHealth]) -> Update {
    let row = |link: &String| {
        let parsed = RelayLink::parse(link).ok()?;
        let found = health.iter().find(|health| health.url == *parsed.url());
        Some(Relay {
            link: link.clone(),
            url: parsed.url().to_string(),
            shown: parsed.masked(),
            status: found.map_or(RelayStatus::Checking, Into::into),
        })
    };
    Update::Relays { relays: node.config().relays.iter().filter_map(row).collect() }
}

fn ready(node: &Node) -> Update {
    let config = node.config();
    Update::Ready {
        id: crate::types::device_id(&node.id()),
        fingerprint: node.id().fmt_short().to_string(),
        name: config.name.clone(),
        download_dir: config.download_dir.display().to_string(),
        visibility: format!("{:?}", config.visibility),
        check_updates: update::enabled().then_some(config.check_updates),
        relay_speed_limit: config.relay_speed_limit,
    }
}

pub async fn start(
    data_dir: std::path::PathBuf,
    shelf: &Arc<Shelf>,
    listener: &Arc<dyn Listener>,
) -> Option<Session> {
    let (node, mut offers) = match Node::start(NodeOptions { data_dir, isolated: false }).await {
        Ok(started) => started,
        Err(error) => {
            listener.update(Update::StartFailed {
                problem: (&error).into(),
                reason: error.to_string(),
            });
            return None;
        }
    };
    let config = node.config();
    listener.update(ready(&node));
    listener.update(devices(&node));
    listener.update(blocked(&node));
    listener.update(history(&node));
    listener.update(relays(&node, &[]));

    let mut tasks = Vec::new();
    if config.check_updates {
        let out = listener.clone();
        tasks.push(spawn(async move {
            if let Ok(Some(release)) = update::check().await {
                out.update(Update::UpdateAvailable {
                    version: release.version,
                    page: release.page,
                });
            }
        }));
    }
    let (checker, out) = (node.clone(), listener.clone());
    tasks.push(spawn(async move {
        let health = checker.relay_health(RELAY_WAIT).await;
        out.update(relays(&checker, &health));
    }));
    tasks.push(forward_events(&node, listener.clone(), shelf.clone()));

    let (out, kept) = (listener.clone(), shelf.clone());
    let downloads = node.config().download_dir.clone();
    tasks.push(spawn(async move {
        let mut next = 0;
        while let Some(offer) = offers.recv().await {
            if offer.summary.auto_accept {
                offer.accept(downloads.clone());
                continue;
            }
            next += 1;
            let incoming = Incoming::new(next, &offer.summary);
            lock(&kept.offers).insert(next, offer);
            out.update(Update::Offer { incoming });
        }
    }));

    let (mut requests, out, kept) = (node.pairing_requests(), listener.clone(), shelf.clone());
    tasks.push(spawn(async move {
        let mut next = 0;
        while let Some(request) = requests.recv().await {
            next += 1;
            out.update(Update::pairing_request(next, &request));
            lock(&kept.pairings).insert(next, request);
        }
    }));

    let (out, kept) = (listener.clone(), shelf.clone());
    tasks.push(spawn(async move {
        let mut sweep = tokio::time::interval(PROMPT_SWEEP);
        loop {
            sweep.tick().await;
            lock(&kept.offers).retain(|key, offer| {
                let open = !offer.is_closed();
                if !open {
                    out.update(Update::OfferGone { key: *key });
                }
                open
            });
            lock(&kept.pairings).retain(|key, request| {
                let open = !request.is_closed();
                if !open {
                    out.update(Update::PairingGone { key: *key });
                }
                open
            });
        }
    }));

    let (scanner, out, kept) = (node.clone(), listener.clone(), shelf.clone());
    tasks.push(spawn(async move {
        scanner
            .follow_nearby(|found| {
                let shown = found.iter().map(Into::into).collect();
                *lock(&kept.nearby) =
                    found.into_iter().map(|device| (device.addr.id, device)).collect();
                out.update(Update::Nearby { devices: shown });
            })
            .await;
    }));
    Some(Session { node, tasks })
}

/// Passes transfer events to the app, with the history again whenever a transfer ends.
fn forward_events(node: &Node, out: Arc<dyn Listener>, kept: Arc<Shelf>) -> AbortHandle {
    let (mut events, recorder) = (node.events(), node.clone());
    spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    if let Event::Waiting { id, .. } | Event::Started { id, .. } = &event {
                        lock(&kept.transfers).insert(id.to_string(), *id);
                    }
                    out.update(Update::from_event(&event));
                    if matches!(event, Event::Completed { .. } | Event::Failed { .. }) {
                        out.update(history(&recorder));
                    }
                }
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => break,
            }
        }
    })
}

fn spawn(task: impl Future<Output = ()> + Send + 'static) -> AbortHandle {
    tokio::spawn(task).abort_handle()
}
