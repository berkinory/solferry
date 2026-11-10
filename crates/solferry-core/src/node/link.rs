use super::Node;
use crate::error::{Error, Result};
use crate::link::{self, Link, LinkFile, Mode};

impl Node {
    /// Serves `files` to any browser on this device's local networks until the link is dropped.
    pub async fn share_link(&self, files: Vec<LinkFile>) -> Result<Link> {
        let served = tokio::task::spawn_blocking(move || link::collect(files))
            .await
            .map_err(|error| Error::Protocol(error.to_string()))??;
        link::start(Mode::Share(served), self.shared.events.clone()).await
    }

    /// Takes files from any browser on this device's local networks until the link is dropped.
    pub async fn receive_link(&self) -> Result<Link> {
        let folder = self.shared.config.download_dir.clone();
        link::start(Mode::Receive(folder), self.shared.events.clone()).await
    }
}
