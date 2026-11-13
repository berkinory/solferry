use std::collections::HashMap;
use std::path::{Path, PathBuf};

use iroh::EndpointId;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::event::TransferId;
use crate::store;

const QUEUE_FILE: &str = "queue.json";
/// How long a send waits for its device before it is dropped.
pub const QUEUE_DAYS: u64 = 7;
const DAY_MS: u64 = 24 * 60 * 60 * 1000;

/// Where a queued send's files come from when it is finally delivered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum QueuedFiles {
    /// Files and folders this process can open by path.
    Paths(Vec<PathBuf>),
    /// Files only the app can reopen, such as an Android content URI. `reference` means nothing
    /// to the core; the app hands back an open file for it.
    Shared(Vec<SharedSource>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharedSource {
    pub name: String,
    pub reference: String,
}

/// A send to a paired device that wasn't there, delivered once it is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueuedSend {
    pub id: u64,
    pub peer: EndpointId,
    pub peer_name: String,
    pub label: String,
    pub files: QueuedFiles,
    pub queued_ms: u64,
}

impl QueuedSend {
    #[must_use]
    pub fn expires_ms(&self) -> u64 {
        self.queued_ms.saturating_add(QUEUE_DAYS * DAY_MS)
    }

    /// The app-owned references, which the app may release once the send is settled.
    #[must_use]
    pub fn references(&self) -> Vec<String> {
        match &self.files {
            QueuedFiles::Paths(_) => Vec::new(),
            QueuedFiles::Shared(shared) => shared.iter().map(|s| s.reference.clone()).collect(),
        }
    }
}

/// How a queued send ended.
#[derive(Clone, Debug)]
pub enum QueueOutcome {
    Delivered,
    /// The device declined, or the send failed in a way waiting won't fix.
    Failed(Error),
    Expired,
    /// The user took it back, or forgot the device.
    Cancelled,
}

#[derive(Debug)]
pub(crate) struct Queue {
    path: PathBuf,
    entries: Vec<QueuedSend>,
    /// Which transfer each entry being delivered runs as, so taking it back can stop it.
    in_flight: HashMap<u64, TransferId>,
}

impl Queue {
    /// A damaged file starts the queue over rather than stopping the app; the loss is logged.
    pub(crate) fn load(data_dir: &Path) -> Self {
        let path = data_dir.join(QUEUE_FILE);
        let entries = store::read_json(&path).unwrap_or_else(|error| {
            tracing::warn!(%error, "could not read the send queue");
            None
        });
        Self { path, entries: entries.unwrap_or_default(), in_flight: HashMap::new() }
    }

    pub(crate) fn entries(&self) -> &[QueuedSend] {
        &self.entries
    }

    pub(crate) fn add(&mut self, entry: QueuedSend) {
        self.entries.push(entry);
        self.save();
    }

    pub(crate) fn remove(&mut self, id: u64) -> Option<QueuedSend> {
        let index = self.entries.iter().position(|entry| entry.id == id)?;
        let removed = self.entries.remove(index);
        self.save();
        Some(removed)
    }

    /// Takes out every entry `stale` picks, with what it says happened to each.
    pub(crate) fn drain_stale(
        &mut self,
        stale: impl Fn(&QueuedSend) -> Option<QueueOutcome>,
    ) -> Vec<(QueuedSend, QueueOutcome)> {
        let mut drained = Vec::new();
        self.entries.retain(|entry| match stale(entry) {
            Some(outcome) => {
                drained.push((entry.clone(), outcome));
                false
            }
            None => true,
        });
        if !drained.is_empty() {
            self.save();
        }
        drained
    }

    /// Marks an entry as being delivered, unless it was taken back meanwhile.
    pub(crate) fn start(&mut self, id: u64, transfer: TransferId) -> bool {
        let queued = self.entries.iter().any(|entry| entry.id == id);
        if queued {
            self.in_flight.insert(id, transfer);
        }
        queued
    }

    pub(crate) fn finish(&mut self, id: u64) -> Option<TransferId> {
        self.in_flight.remove(&id)
    }

    fn save(&self) {
        if let Err(error) = store::write_json(&self.path, &self.entries) {
            tracing::warn!(%error, "could not save the send queue");
        }
    }
}
