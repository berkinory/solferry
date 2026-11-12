use std::fmt;

use serde::{Deserialize, Serialize};

use iroh::EndpointId;

use crate::error::Error;
use crate::route::Route;
use crate::trust::Device;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TransferId(pub(crate) [u8; 32]);

impl fmt::Display for TransferId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&hex::encode(&self.0[..6]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Sending,
    Receiving,
}

#[derive(Clone, Debug)]
pub enum Event {
    Paired {
        device: Device,
    },
    /// A paired device shared copied text. `auto_copy` follows its "receive without asking" choice.
    TextReceived {
        peer: EndpointId,
        peer_name: String,
        text: String,
        auto_copy: bool,
    },
    /// A send is reaching the other device and waiting for it to accept.
    Waiting {
        id: TransferId,
        peer: EndpointId,
        peer_name: String,
        label: String,
        bytes: u64,
    },
    Started {
        id: TransferId,
        peer: EndpointId,
        peer_name: String,
        label: String,
        direction: Direction,
        route: Route,
        files: usize,
        bytes: u64,
        platform: Option<crate::Platform>,
    },
    RouteChanged {
        id: TransferId,
        route: Route,
    },
    Progress {
        id: TransferId,
        transferred: u64,
        total: u64,
    },
    Reconnecting {
        id: TransferId,
        reason: Error,
    },
    Completed {
        id: TransferId,
    },
    /// A browser sent a file through a receive link. It is saved under `name`.
    LinkReceived {
        name: String,
        bytes: u64,
    },
    /// A paired device just checked whether this one is there, so it is there too.
    Present {
        peer: EndpointId,
    },
    /// A browser sent text through a receive link.
    LinkText {
        text: String,
    },
    Failed {
        id: TransferId,
        error: Error,
    },
}
