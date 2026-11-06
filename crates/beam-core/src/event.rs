use std::fmt;

use iroh::EndpointId;

use crate::error::Error;
use crate::route::Route;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TransferId(pub(crate) [u8; 32]);

impl fmt::Display for TransferId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&hex::encode(&self.0[..6]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Sending,
    Receiving,
}

#[derive(Clone, Debug)]
pub enum Event {
    Started {
        id: TransferId,
        peer: EndpointId,
        peer_name: String,
        direction: Direction,
        route: Route,
        files: usize,
        bytes: u64,
    },
    Progress {
        id: TransferId,
        transferred: u64,
        total: u64,
    },
    Completed {
        id: TransferId,
    },
    Failed {
        id: TransferId,
        error: Error,
    },
}
