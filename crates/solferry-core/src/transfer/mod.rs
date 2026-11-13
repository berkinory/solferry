mod manifest;
mod meter;
pub(crate) mod paths;
mod quarantine;
pub(crate) mod receiver;
pub(crate) mod segments;
pub(crate) mod sender;
mod session;
mod source;
mod staging;
mod throttle;

pub(crate) use manifest::Manifest;
pub use manifest::NamedFile;
pub(crate) use meter::Meter;
pub(crate) use session::{
    exchange_hello, remote_failure, transfer_label, validate_offer, within_handshake,
};
pub(crate) use staging::free_path;
pub(crate) use staging::{Staging, prune_abandoned};
pub(crate) use throttle::{RelayLimit, Throttle};

const CHUNK_SIZE: usize = 1024 * 1024;
