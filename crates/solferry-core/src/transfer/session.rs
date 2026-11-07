use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use iroh::endpoint::{Connection, ConnectionError, RecvStream, SendStream};
use solferry_proto::{
    Entry, HASH_LEN, Hello, Offer, PROTOCOL_VERSION, close, read_small_frame, write_frame,
};

use super::paths::{clean_label, sanitize};
use crate::error::{Error, Result};

const MAX_ENTRIES: usize = 1_000_000;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(20);

/// Fails instead of waiting forever on a peer that connects and then says nothing.
pub(crate) async fn within_handshake<T>(step: impl Future<Output = Result<T>>) -> Result<T> {
    tokio::time::timeout(HANDSHAKE_TIMEOUT, step).await.unwrap_or(Err(Error::ConnectionLost))
}

pub(crate) async fn exchange_hello(
    send: &mut SendStream,
    recv: &mut RecvStream,
    name: &str,
) -> Result<Hello> {
    let hello = Hello { version: PROTOCOL_VERSION, name: name.to_owned(), capabilities: 0 };
    write_frame(send, &hello).await?;
    let theirs: Hello = read_small_frame(recv).await?;
    if theirs.version != PROTOCOL_VERSION {
        return Err(Error::VersionMismatch { ours: PROTOCOL_VERSION, theirs: theirs.version });
    }
    Ok(Hello { name: clean_label(&theirs.name), ..theirs })
}

/// Prefers the reason the peer gave when it closed over the local symptom.
pub(crate) fn remote_failure(connection: &Connection, fallback: Error) -> Error {
    let closed = match connection.close_reason() {
        Some(ConnectionError::ApplicationClosed(closed)) => closed,
        Some(ConnectionError::LocallyClosed) => return Error::Cancelled,
        _ => return fallback,
    };
    let code = closed.error_code.into_inner();
    if code == u64::from(close::CANCELLED) {
        Error::Cancelled
    } else if code == u64::from(close::NOT_PAIRED) {
        Error::NotPaired
    } else if code == u64::from(close::BUSY) {
        Error::Busy
    } else if code == u64::from(close::VERSION_MISMATCH) {
        Error::VersionMismatch { ours: PROTOCOL_VERSION, theirs: 0 }
    } else {
        fallback
    }
}

/// Short description of a transfer for progress displays.
pub(crate) fn transfer_label(entries: &[Entry]) -> String {
    let top = |entry: &Entry| entry.path.split('/').next().unwrap_or_default().to_owned();
    let first = entries.first().map(top).unwrap_or_default();
    let others = entries.iter().map(top).filter(|name| *name != first).count();
    if others == 0 { first } else { format!("{first} and more") }
}

pub(crate) fn validate_offer(offer: &Offer) -> Result<Vec<PathBuf>> {
    let invalid = |reason: &str| Error::InvalidOffer(reason.to_owned());
    if offer.transfer_id.len() != HASH_LEN {
        return Err(invalid("malformed transfer id"));
    }
    if offer.entries.is_empty() || offer.entries.len() > MAX_ENTRIES {
        return Err(invalid("unacceptable entry count"));
    }
    if offer.entries.iter().try_fold(0u64, |sum, entry| sum.checked_add(entry.size)).is_none() {
        return Err(invalid("total size overflows"));
    }
    let mut seen = HashSet::with_capacity(offer.entries.len());
    offer
        .entries
        .iter()
        .map(|Entry { path, .. }| {
            let safe = sanitize(path).ok_or_else(|| invalid("unsafe path"))?;
            if seen.insert(path.to_lowercase()) { Ok(safe) } else { Err(invalid("duplicate path")) }
        })
        .collect()
}
