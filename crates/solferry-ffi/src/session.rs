use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use iroh::EndpointId;
use solferry_core::{IncomingOffer, NearbyDevice, Node, NodeOptions, PairingRequest, TransferId};
use tokio::sync::broadcast::error::RecvError;
use tokio::task::AbortHandle;

use crate::Listener;
use crate::types::{Incoming, PairedDevice, Update};

const PROMPT_SWEEP: Duration = Duration::from_secs(1);

/// What the bridge remembers between calls: prompts waiting for an answer, what is nearby, and
/// which transfer an id from the app refers to.
#[derive(Default)]
pub struct Shelf {
    pub offers: Mutex<HashMap<u64, IncomingOffer>>,
    pub pairings: Mutex<HashMap<u64, PairingRequest>>,
    pub nearby: Mutex<HashMap<EndpointId, NearbyDevice>>,
    pub transfers: Mutex<HashMap<String, TransferId>>,
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

pub fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub fn devices(node: &Node) -> Update {
    Update::Devices { devices: node.devices().iter().map(PairedDevice::from).collect() }
}

pub async fn start(
    data_dir: std::path::PathBuf,
    shelf: &Arc<Shelf>,
    listener: &Arc<dyn Listener>,
) -> Option<Session> {
    let (node, mut offers) = match Node::start(NodeOptions { data_dir, isolated: false }).await {
        Ok(started) => started,
        Err(error) => {
            listener.update(Update::StartFailed { problem: (&error).into() });
            return None;
        }
    };
    let config = node.config();
    listener.update(Update::Ready {
        id: crate::types::device_id(&node.id()),
        name: config.name.clone(),
        download_dir: config.download_dir.display().to_string(),
        visibility: format!("{:?}", config.visibility),
    });
    listener.update(devices(&node));

    let mut tasks = Vec::new();
    let (mut events, out, kept) = (node.events(), listener.clone(), shelf.clone());
    tasks.push(spawn(async move {
        loop {
            match events.recv().await {
                Ok(event) => {
                    if let solferry_core::Event::Started { id, .. } = &event {
                        lock(&kept.transfers).insert(id.to_string(), *id);
                    }
                    out.update(Update::from_event(&event));
                }
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => break,
            }
        }
    }));

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

fn spawn(task: impl Future<Output = ()> + Send + 'static) -> AbortHandle {
    tokio::spawn(task).abort_handle()
}
