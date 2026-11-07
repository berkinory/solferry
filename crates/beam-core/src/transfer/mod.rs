mod manifest;
mod meter;
pub(crate) mod paths;
pub(crate) mod receiver;
pub(crate) mod sender;
mod session;
mod staging;

pub(crate) use manifest::Manifest;
pub(crate) use meter::Meter;
pub(crate) use session::{
    exchange_hello, remote_failure, transfer_label, validate_offer, within_handshake,
};
pub(crate) use staging::Staging;

const CHUNK_SIZE: usize = 1024 * 1024;
