use iroh::EndpointId;
use solferry_core::{
    Device, Direction, Error, Event, NearbyDevice, OfferSummary, PastTransfer, Platform,
    RelayHealth, Route, explain,
};

/// A short way to name a device on the other side of the bridge: its key in z-base-32.
#[must_use]
pub fn device_id(id: &EndpointId) -> String {
    id.to_z32()
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum DevicePlatform {
    Macos,
    Windows,
    Linux,
    Android,
    Ios,
}

impl From<Platform> for DevicePlatform {
    fn from(platform: Platform) -> Self {
        match platform {
            Platform::Macos => Self::Macos,
            Platform::Windows => Self::Windows,
            Platform::Linux => Self::Linux,
            Platform::Android => Self::Android,
            Platform::Ios => Self::Ios,
        }
    }
}

/// Who a device is, as every screen shows it.
#[derive(Clone, Debug, uniffi::Record)]
pub struct Identity {
    pub id: String,
    /// The start of the key, short enough to compare by eye.
    pub fingerprint: String,
    /// Which of the eight identity colors the key picks.
    pub tone: u8,
    pub platform: Option<DevicePlatform>,
}

pub fn identity(id: &EndpointId, platform: Option<Platform>) -> Identity {
    Identity {
        id: device_id(id),
        fingerprint: id.fmt_short().to_string(),
        tone: id.as_bytes()[0] % 8,
        platform: platform.map(Into::into),
    }
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct PairedDevice {
    pub identity: Identity,
    pub name: String,
    pub auto_accept: bool,
    pub paired_at_ms: u64,
}

impl From<&Device> for PairedDevice {
    fn from(device: &Device) -> Self {
        Self {
            identity: identity(&device.id, device.platform),
            name: device.name.clone(),
            auto_accept: device.auto_accept,
            paired_at_ms: device.paired_at_ms,
        }
    }
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct Nearby {
    pub identity: Identity,
    pub name: Option<String>,
    pub known: bool,
    pub overlay: bool,
}

impl From<&NearbyDevice> for Nearby {
    fn from(device: &NearbyDevice) -> Self {
        Self {
            identity: identity(&device.addr.id, device.platform),
            name: device.name.clone(),
            known: device.known,
            overlay: device.overlay,
        }
    }
}

/// A transfer that is over, as the history screen lists it.
#[derive(Clone, Debug, uniffi::Record)]
pub struct BlockedDevice {
    pub identity: Identity,
    pub name: Option<String>,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct HistoryEntry {
    pub peer: String,
    pub name: String,
    pub label: String,
    pub sending: bool,
    pub files: u64,
    pub bytes: u64,
    pub finished_ms: u64,
    pub completed: bool,
    pub platform: Option<DevicePlatform>,
}

impl From<&PastTransfer> for HistoryEntry {
    fn from(past: &PastTransfer) -> Self {
        Self {
            peer: device_id(&past.peer),
            name: past.peer_name.clone(),
            label: past.label.clone(),
            sending: past.direction == Direction::Sending,
            files: past.files as u64,
            bytes: past.bytes,
            finished_ms: past.finished_ms,
            completed: past.completed,
            platform: past.platform.map(Into::into),
        }
    }
}

/// How far a connection to one of your relays gets, from the first thing to go wrong.
#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum RelayStatus {
    Checking,
    Connected,
    UdpBlocked,
    Refused,
    Unreachable,
}

impl From<&RelayHealth> for RelayStatus {
    fn from(health: &RelayHealth) -> Self {
        match (health.reachable, health.accepted, health.direct_paths) {
            (false, ..) => Self::Unreachable,
            (true, false, _) => Self::Refused,
            (true, true, false) => Self::UdpBlocked,
            (true, true, true) => Self::Connected,
        }
    }
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct Relay {
    /// What was added, including any access token. Removing a relay names it by this.
    pub link: String,
    pub url: String,
    pub status: RelayStatus,
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct Problem {
    pub title: String,
    pub next: String,
}

impl Problem {
    /// The same advice, with a title that names the device it happened with.
    #[must_use]
    pub fn about(error: &solferry_core::Error, name: &str) -> Self {
        Self { title: solferry_core::about(error, name), ..error.into() }
    }
}

impl From<&Error> for Problem {
    fn from(error: &Error) -> Self {
        let explanation = explain(error);
        Self { title: explanation.title.to_owned(), next: explanation.next.to_owned() }
    }
}

#[derive(Clone, Debug, uniffi::Record)]
pub struct Incoming {
    pub key: u64,
    pub sender: Identity,
    pub name: String,
    pub paired: bool,
    pub files: u64,
    pub bytes: u64,
    pub first_path: String,
}

impl Incoming {
    #[must_use]
    pub fn new(key: u64, summary: &OfferSummary) -> Self {
        Self {
            key,
            sender: identity(&summary.peer, summary.platform),
            name: summary.peer_name.clone(),
            paired: summary.paired,
            files: summary.files as u64,
            bytes: summary.bytes,
            first_path: summary.first_path.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum TransferRoute {
    Local,
    Overlay,
    Direct,
    Relay,
}

impl From<Route> for TransferRoute {
    fn from(route: Route) -> Self {
        match route {
            Route::Local => Self::Local,
            Route::Overlay => Self::Overlay,
            Route::Direct => Self::Direct,
            Route::Relay => Self::Relay,
        }
    }
}

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
    Ready {
        id: String,
        fingerprint: String,
        name: String,
        download_dir: String,
        visibility: String,
        /// `None` when this build has no update channel.
        check_updates: Option<bool>,
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
}

impl Update {
    #[must_use]
    pub fn from_event(event: &Event) -> Self {
        match event {
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
            Event::Failed { id, error: Error::Cancelled } => Self::Cancelled { id: id.to_string() },
            Event::Failed { id, error } => {
                Self::Failed { id: id.to_string(), problem: error.into() }
            }
        }
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
