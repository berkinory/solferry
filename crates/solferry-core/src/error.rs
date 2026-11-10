use std::path::PathBuf;
use std::sync::Arc;

use solferry_proto::FrameError;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Error)]
pub enum Error {
    #[error("cannot access {path}")]
    Storage {
        path: PathBuf,
        #[source]
        source: Arc<std::io::Error>,
    },
    #[error("corrupt data file {path}")]
    CorruptData {
        path: PathBuf,
        #[source]
        source: Arc<serde_json::Error>,
    },
    #[error("network stack failed to start: {0}")]
    NetworkStart(String),
    #[error("invalid relay url {0}")]
    InvalidRelayUrl(String),
    #[error("a relay is not working")]
    RelayCheckFailed,
    #[error("device is unreachable: {0}")]
    Unreachable(String),
    #[error("the device only accepts paired peers")]
    NotPaired,
    #[error("the device runs protocol {theirs}, this one runs {ours}")]
    VersionMismatch { ours: u32, theirs: u32 },
    #[error("the device is busy with other transfers")]
    Busy,
    #[error("the transfer was cancelled")]
    Cancelled,
    #[error("the transfer was declined")]
    Declined,
    #[error("the receiver is out of disk space")]
    NoSpace,
    #[error("the receiver could not save the files")]
    DeliveryFailed,
    #[error("the offer is invalid: {0}")]
    InvalidOffer(String),
    #[error("{0} changed while it was being sent")]
    SourceChanged(PathBuf),
    #[error("file name is not valid UTF-8: {0}")]
    UnsupportedName(PathBuf),
    #[error("nothing to send")]
    NothingToSend,
    #[error("{0} failed integrity verification")]
    HashMismatch(String),
    #[error("connection lost")]
    ConnectionLost,
    #[error("invalid pairing code")]
    InvalidTicket,
    #[error("pairing code expired")]
    TicketExpired,
    #[error("update failed: {0}")]
    Update(String),
    #[error("pairing code belongs to this device")]
    OwnTicket,
    #[error("pairing was refused")]
    PairingRefused,
    #[error("unknown device {0}")]
    UnknownDevice(String),
    #[error("{0} matches more than one device")]
    AmbiguousDevice(String),
    #[error("protocol violation: {0}")]
    Protocol(String),
}

impl Error {
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::ConnectionLost | Self::Unreachable(_))
    }
}

impl From<FrameError> for Error {
    fn from(error: FrameError) -> Self {
        match error {
            FrameError::Io(_) => Self::ConnectionLost,
            other => Self::Protocol(other.to_string()),
        }
    }
}

pub(crate) trait StorageContext<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> StorageContext<T> for std::io::Result<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T> {
        self.map_err(|source| Error::Storage { path: path.into(), source: source.into() })
    }
}
