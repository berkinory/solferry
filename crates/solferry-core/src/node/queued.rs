//! Sends waiting for paired devices that weren't there, delivered when they come back.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::sync::Arc;
use std::time::Duration;

use iroh::EndpointId;
use rust_i18n::t;
use tokio::sync::broadcast::error::RecvError;
use tokio::task::JoinSet;

use super::{Node, pairing::now_ms, presence};
use crate::error::{Error, Result};
use crate::event::Event;
use crate::queue::{QueueOutcome, QueuedFiles, QueuedSend};
use crate::transfer::{Manifest, NamedFile};
use crate::trust::Device;

/// How often devices with waiting sends are checked, apart from whenever one reaches this device.
const RECHECK: Duration = Duration::from_secs(20);

/// Opens a file the app shared by reference, or `None` when it is gone.
pub type Reopen = Arc<dyn Fn(&str) -> Option<File> + Send + Sync>;

impl Node {
    #[must_use]
    pub fn queued(&self) -> Vec<QueuedSend> {
        self.shared.queue().entries().to_vec()
    }

    /// Keeps a send for a paired device until it is there. Unpaired devices never queue.
    pub fn queue(&self, peer: EndpointId, files: QueuedFiles) -> Result<QueuedSend> {
        let peer_name = self.shared.trust().get(&peer).map(|device| device.name.clone());
        let peer_name = peer_name.ok_or(Error::NotPaired)?;
        let names: Vec<String> = match &files {
            QueuedFiles::Paths(paths) => paths
                .iter()
                .map(|path| {
                    path.file_name().map_or_else(
                        || path.display().to_string(),
                        |name| name.to_string_lossy().into_owned(),
                    )
                })
                .collect(),
            QueuedFiles::Shared(shared) => shared.iter().map(|file| file.name.clone()).collect(),
        };
        let first = names.first().ok_or(Error::NothingToSend)?;
        let label = if names.len() == 1 {
            first.clone()
        } else {
            t!("transfer.and_more", first = first).into_owned()
        };
        let entry =
            QueuedSend { id: rand::random(), peer, peer_name, label, files, queued_ms: now_ms() };
        self.shared.queue().add(entry.clone());
        self.shared.queue_added.notify_one();
        let _ = self.shared.events.send(Event::QueueChanged);
        Ok(entry)
    }

    /// Takes a queued send back, stopping it if it is being delivered.
    pub fn unqueue(&self, id: u64) {
        let (removed, running) = {
            let mut queue = self.shared.queue();
            (queue.remove(id), queue.finish(id))
        };
        if let Some(transfer) = running {
            self.cancel(transfer);
        }
        if let Some(entry) = removed {
            self.settle(entry, QueueOutcome::Cancelled);
        }
    }

    /// Delivers queued sends whenever their devices are there. Apps run this for as long as they
    /// run; nothing is delivered while no app is.
    pub async fn deliver_queued(&self, reopen: Reopen) {
        let mut events = self.events();
        let mut recheck = tokio::time::interval(RECHECK);
        recheck.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let mut busy: HashMap<tokio::task::Id, EndpointId> = HashMap::new();
        let mut running = JoinSet::new();
        loop {
            let only = tokio::select! {
                _ = recheck.tick() => None,
                () = self.shared.queue_added.notified() => None,
                Some(done) = running.join_next_with_id() => {
                    let id = done.map_or_else(|error| error.id(), |(id, ())| id);
                    busy.remove(&id);
                    continue;
                }
                event = events.recv() => match event {
                    Ok(Event::Present { peer }) => Some(peer),
                    Err(RecvError::Closed) => return,
                    _ => continue,
                },
            };
            self.drop_stale();
            for peer in self.queued_peers() {
                let skipped = only.is_some_and(|only| only != peer);
                if skipped || busy.values().any(|known| *known == peer) {
                    continue;
                }
                let task = running.spawn(deliver_to(self.clone(), peer, reopen.clone()));
                busy.insert(task.id(), peer);
            }
        }
    }

    /// Drops what expired, and what waits for devices that are no longer paired.
    pub(super) fn drop_stale(&self) {
        let now = now_ms();
        let paired: HashSet<EndpointId> =
            self.shared.trust().all().iter().map(|device| device.id).collect();
        let drained = self.shared.queue().drain_stale(|entry| {
            if !paired.contains(&entry.peer) {
                Some(QueueOutcome::Cancelled)
            } else if entry.expires_ms() <= now {
                Some(QueueOutcome::Expired)
            } else {
                None
            }
        });
        for (entry, outcome) in drained {
            self.settle(entry, outcome);
        }
    }

    fn queued_peers(&self) -> Vec<EndpointId> {
        let mut peers: Vec<EndpointId> =
            self.shared.queue().entries().iter().map(|entry| entry.peer).collect();
        peers.sort_unstable();
        peers.dedup();
        peers
    }

    fn settle(&self, entry: QueuedSend, outcome: QueueOutcome) {
        let _ = self.shared.events.send(Event::QueueSettled { entry, outcome });
        let _ = self.shared.events.send(Event::QueueChanged);
    }

    async fn deliver_entry(
        &self,
        device: &Device,
        entry: &QueuedSend,
        reopen: &Reopen,
    ) -> Result<()> {
        let (sender, files, reopen) = (self.id(), entry.files.clone(), reopen.clone());
        let manifest = tokio::task::spawn_blocking(move || match files {
            QueuedFiles::Paths(paths) => Manifest::scan(&paths, sender),
            QueuedFiles::Shared(shared) => {
                let named = shared.into_iter().map(|source| {
                    let file = reopen(&source.reference).ok_or_else(|| Error::Storage {
                        path: source.name.clone().into(),
                        source: Arc::new(std::io::ErrorKind::NotFound.into()),
                    })?;
                    Ok(NamedFile { name: source.name, file })
                });
                Manifest::named(named.collect::<Result<_>>()?, sender)
            }
        })
        .await
        .map_err(|error| Error::Protocol(error.to_string()))??;
        if !self.shared.queue().start(entry.id, manifest.id) {
            return Err(Error::Cancelled);
        }
        let result = self.deliver(device.addr(), Arc::new(manifest)).await;
        self.shared.queue().finish(entry.id);
        result.map(|_| ())
    }
}

/// Probes the device once, then delivers its sends oldest first while it takes them.
async fn deliver_to(node: Node, peer: EndpointId, reopen: Reopen) {
    let Some(device) = node.shared.trust().get(&peer).cloned() else { return };
    if !presence::answers(&node.shared.endpoint, device.addr()).await {
        return;
    }
    loop {
        let next = node.shared.queue().entries().iter().find(|entry| entry.peer == peer).cloned();
        let Some(entry) = next else { return };
        let outcome = match node.deliver_entry(&device, &entry, &reopen).await {
            Ok(()) => QueueOutcome::Delivered,
            Err(Error::Unreachable(_) | Error::ConnectionLost | Error::Busy) => return,
            Err(Error::Cancelled) => QueueOutcome::Cancelled,
            Err(error) => QueueOutcome::Failed(error),
        };
        tracing::info!(?outcome, "queued send settled");
        if let Some(entry) = node.shared.queue().remove(entry.id) {
            node.settle(entry, outcome);
        }
    }
}
