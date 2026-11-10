//! Sharing with a browser on the same network, for a device that doesn't run Solferry.

mod files;
mod page;
mod serve;

use std::path::PathBuf;

pub use files::LinkFile;
pub(crate) use files::collect;
pub(crate) use serve::start;

/// A running link. Dropping it stops the server and every download or upload in flight.
#[derive(Debug)]
pub struct Link {
    urls: Vec<String>,
    server: tokio::task::AbortHandle,
}

impl Link {
    /// One address per local network this device is on. The first is the likeliest to work.
    #[must_use]
    pub fn urls(&self) -> &[String] {
        &self.urls
    }
}

impl Drop for Link {
    fn drop(&mut self) {
        self.server.abort();
    }
}

pub(crate) enum Mode {
    Share(Vec<files::Served>),
    Receive(PathBuf),
}
