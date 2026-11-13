use iroh::EndpointId;
use solferry_core::{
    Device, Direction, Error, NearbyDevice, OfferSummary, PastTransfer, Platform, RelayHealth,
    Route, explain,
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
    pub overlay: bool,
}

impl From<&NearbyDevice> for Nearby {
    fn from(device: &NearbyDevice) -> Self {
        Self {
            identity: identity(&device.addr.id, device.platform),
            name: device.name.clone(),
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
    /// What was added, including any access token. Removing a relay names it by this, and
    /// copying it gives this.
    pub link: String,
    pub url: String,
    /// What to show: the server, with any token hidden.
    pub shown: String,
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
        Self { title: explanation.title.into_owned(), next: explanation.next.into_owned() }
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
