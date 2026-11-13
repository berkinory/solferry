use solferry_core::{Direction, Error, Event};

use crate::queue::{QueueEntry, QueueResult};
use crate::types::{
    BlockedDevice, HistoryEntry, Identity, Incoming, Nearby, PairedDevice, Problem, Relay,
    TransferRoute, device_id, identity,
};

/// Everything the app learns from the core, in the order it happens.
#[derive(Clone, Debug, uniffi::Enum)]
pub enum Update {
    Relays {
        relays: Vec<Relay>,
    },
    RelayFailed {
        problem: Problem,
    },
    Blocked {
        devices: Vec<BlockedDevice>,
    },
    /// Paired devices that answered the last presence check.
    Reachable {
        ids: Vec<String>,
    },
    /// A paired device just reached this one, so it is there.
    Present {
        id: String,
    },
    /// A paired device said it is going away.
    Absent {
        id: String,
    },
    Ready {
        id: String,
        fingerprint: String,
        name: String,
        download_dir: String,
        visibility: String,
        /// `None` when this build has no update channel.
        check_updates: Option<bool>,
        /// Bytes per second while files go through a relay. `None` means no limit.
        relay_speed_limit: Option<u64>,
    },
    UpdateAvailable {
        version: String,
        page: String,
    },
    StartFailed {
        problem: Problem,
        reason: String,
    },
    Devices {
        devices: Vec<PairedDevice>,
    },
    History {
        entries: Vec<HistoryEntry>,
    },
    Nearby {
        devices: Vec<Nearby>,
    },
    Waiting {
        id: String,
        peer_name: String,
        label: String,
        bytes: u64,
    },
    Started {
        id: String,
        peer_name: String,
        label: String,
        sending: bool,
        route: TransferRoute,
        bytes: u64,
    },
    Progress {
        id: String,
        transferred: u64,
        total: u64,
    },
    RouteChanged {
        id: String,
        route: TransferRoute,
    },
    Reconnecting {
        id: String,
    },
    Completed {
        id: String,
    },
    Cancelled {
        id: String,
    },
    Failed {
        id: String,
        problem: Problem,
    },
    Offer {
        incoming: Incoming,
    },
    OfferGone {
        key: u64,
    },
    PairingRequest {
        key: u64,
        peer: Identity,
        name: String,
        code: String,
    },
    PairingGone {
        key: u64,
    },
    PairingCode {
        code: String,
    },
    Invite {
        code: String,
    },
    Paired {
        device: PairedDevice,
    },
    AddressFailed {
        problem: Problem,
    },
    PairingFailed {
        problem: Problem,
    },
    SendFailed {
        name: String,
        problem: Problem,
    },
    TextReceived {
        name: String,
        text: String,
        auto_copy: bool,
    },
    TextShared {
        name: String,
        problem: Option<Problem>,
    },
    /// A link is up. `urls` has one address per local network, the likeliest first.
    LinkReady {
        urls: Vec<String>,
        receiving: bool,
    },
    LinkFailed {
        problem: Problem,
    },
    LinkStopped,
    /// A browser sent a file through the receive link.
    LinkReceived {
        name: String,
        bytes: u64,
    },
    /// A browser sent text through the receive link.
    LinkText {
        text: String,
    },
    /// Sends waiting for paired devices that weren't there, oldest first.
    Queue {
        entries: Vec<QueueEntry>,
    },
    /// A send to a device that wasn't there went into the queue.
    Queued {
        name: String,
    },
    /// A queued send left the queue. The app may release `references` once no other entry
    /// uses them.
    QueueSettled {
        id: u64,
        name: String,
        label: String,
        references: Vec<String>,
        result: QueueResult,
    },
}

impl Update {
    /// What the app hears about `event`. The queue's list comes from the node instead, so a
    /// change to it maps to nothing here.
    #[must_use]
    pub fn from_event(event: &Event) -> Option<Self> {
        Some(match event {
            Event::Paired { device } => Self::Paired { device: device.into() },
            Event::TextReceived { peer_name, text, auto_copy, .. } => Self::TextReceived {
                name: peer_name.clone(),
                text: text.clone(),
                auto_copy: *auto_copy,
            },
            Event::Waiting { id, peer_name, label, bytes, .. } => Self::Waiting {
                id: id.to_string(),
                peer_name: peer_name.clone(),
                label: label.clone(),
                bytes: *bytes,
            },
            Event::Started { id, peer_name, label, direction, route, bytes, .. } => Self::Started {
                id: id.to_string(),
                peer_name: peer_name.clone(),
                label: label.clone(),
                sending: *direction == Direction::Sending,
                route: (*route).into(),
                bytes: *bytes,
            },
            Event::RouteChanged { id, route } => {
                Self::RouteChanged { id: id.to_string(), route: (*route).into() }
            }
            Event::Progress { id, transferred, total } => {
                Self::Progress { id: id.to_string(), transferred: *transferred, total: *total }
            }
            Event::Reconnecting { id, .. } => Self::Reconnecting { id: id.to_string() },
            Event::Completed { id } => Self::Completed { id: id.to_string() },
            Event::LinkReceived { name, bytes } => {
                Self::LinkReceived { name: name.clone(), bytes: *bytes }
            }
            Event::LinkText { text } => Self::LinkText { text: text.clone() },
            Event::Present { peer } => Self::Present { id: device_id(peer) },
            Event::Absent { peer } => Self::Absent { id: device_id(peer) },
            Event::Failed { id, error: Error::Cancelled } => Self::Cancelled { id: id.to_string() },
            Event::Failed { id, error } => {
                Self::Failed { id: id.to_string(), problem: error.into() }
            }
            Event::QueueSettled { entry, outcome } => Self::QueueSettled {
                id: entry.id,
                name: entry.peer_name.clone(),
                label: entry.label.clone(),
                references: entry.references(),
                result: QueueResult::new(outcome, &entry.peer_name),
            },
            Event::QueueChanged => return None,
        })
    }

    #[must_use]
    pub fn pairing_request(key: u64, request: &solferry_core::PairingRequest) -> Self {
        Self::PairingRequest {
            key,
            peer: identity(&request.peer, request.platform),
            name: request.peer_name.clone(),
            code: request.code.clone(),
        }
    }
}
