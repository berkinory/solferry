//! Sends waiting for paired devices that weren't there. The app keeps each file reachable by a
//! reference of its own, such as a content URI with a lasting grant or a private copy, and opens
//! it again when the device is back.

use std::sync::Arc;

use iroh::EndpointId;
use solferry_core::{
    Error, Node, QUEUE_DAYS, QueueOutcome, QueuedFiles, QueuedSend, Reopen, SharedSource,
};
use tokio::task::AbortHandle;

use crate::types::{Problem, device_id};
use crate::update::Update;
use crate::{Listener, Solferry, adopt};

/// A file the app can open again later by `reference`.
#[derive(Clone, Debug, uniffi::Record)]
pub struct QueuedFile {
    pub name: String,
    pub reference: String,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct QueueEntry {
    pub id: u64,
    pub peer: String,
    pub name: String,
    pub label: String,
    pub expires_ms: u64,
    pub references: Vec<String>,
}

impl From<&QueuedSend> for QueueEntry {
    fn from(entry: &QueuedSend) -> Self {
        Self {
            id: entry.id,
            peer: device_id(&entry.peer),
            name: entry.peer_name.clone(),
            label: entry.label.clone(),
            expires_ms: entry.expires_ms(),
            references: entry.references(),
        }
    }
}

/// How a queued send ended.
#[derive(Clone, Debug, uniffi::Enum)]
pub enum QueueResult {
    Delivered,
    Failed {
        problem: Problem,
    },
    /// Expired, or taken back. Nobody needs telling.
    Dropped,
}

impl QueueResult {
    #[must_use]
    pub fn new(outcome: &QueueOutcome, name: &str) -> Self {
        match outcome {
            QueueOutcome::Delivered => Self::Delivered,
            QueueOutcome::Failed(Error::Cancelled)
            | QueueOutcome::Expired
            | QueueOutcome::Cancelled => Self::Dropped,
            QueueOutcome::Failed(error) => Self::Failed { problem: Problem::about(error, name) },
        }
    }
}

/// How many days a queued send waits, for the app to say so.
#[uniffi::export]
#[must_use]
pub fn queue_days() -> u64 {
    QUEUE_DAYS
}

pub fn queue(node: &Node) -> Update {
    Update::Queue { entries: node.queued().iter().map(QueueEntry::from).collect() }
}

/// Tells the app what is queued, then delivers it whenever its devices are back. The app opens
/// each file again and hands over a descriptor it detached.
pub fn follow(node: Node, listener: Arc<dyn Listener>) -> AbortHandle {
    listener.update(queue(&node));
    let reopen: Reopen =
        Arc::new(move |reference: &str| adopt(listener.reopen(reference.to_owned())?));
    tokio::spawn(async move { node.deliver_queued(reopen).await }).abort_handle()
}

#[uniffi::export]
impl Solferry {
    /// Queues files for a paired device that isn't there. Unpaired devices never queue.
    pub fn queue_files(&self, peer: String, files: Vec<QueuedFile>) {
        let sources: Vec<SharedSource> = files
            .into_iter()
            .map(|file| SharedSource { name: file.name, reference: file.reference })
            .collect();
        self.with_node(
            move |node, listener, _| async move {
                let id =
                    EndpointId::from_z32(&peer).map_err(|_| Error::UnknownDevice(peer.clone()));
                let name = id
                    .as_ref()
                    .ok()
                    .and_then(|id| node.devices().into_iter().find(|device| device.id == *id))
                    .map_or_else(|| peer.clone(), |device| device.name);
                match id.and_then(|id| node.queue(id, QueuedFiles::Shared(sources))) {
                    Ok(_) => listener.update(Update::Queued { name }),
                    Err(error) => {
                        let problem = Problem::about(&error, &name);
                        listener.update(Update::SendFailed { name, problem });
                    }
                }
            },
            None,
        );
    }

    pub fn unqueue(&self, id: u64) {
        self.with_node(move |node, _, _| async move { node.unqueue(id) }, None);
    }
}
