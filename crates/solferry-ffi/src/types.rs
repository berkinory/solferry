use iroh::EndpointId;
use solferry_core::{
    Device, Direction, Error, Event, NearbyDevice, OfferSummary, Platform, Route, explain,
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

fn identity(id: &EndpointId, platform: Option<Platform>) -> Identity {
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
}

impl From<&Device> for PairedDevice {
    fn from(device: &Device) -> Self {
        Self {
            identity: identity(&device.id, device.platform),
            name: device.name.clone(),
            auto_accept: device.auto_accept,
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

#[derive(Clone, Debug, uniffi::Record)]
pub struct Problem {
    pub title: String,
    pub next: String,
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
    Ready {
        id: String,
        name: String,
        download_dir: String,
        visibility: String,
    },
    StartFailed {
        problem: Problem,
    },
    Devices {
        devices: Vec<PairedDevice>,
    },
    Nearby {
        devices: Vec<Nearby>,
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
