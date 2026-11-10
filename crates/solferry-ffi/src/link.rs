use std::future::Future;

use solferry_core::{Link, LinkFile, NamedFile, Node};

use crate::session::lock;
use crate::types::Update;
use crate::{SharedFile, Solferry, adopt};

#[uniffi::export]
impl Solferry {
    /// Offers files to browsers on the local network until `stop_link`.
    pub fn share_link(&self, files: Vec<SharedFile>) {
        let files: Vec<LinkFile> = files
            .into_iter()
            .filter_map(|shared| {
                let file = adopt(shared.fd)?;
                Some(LinkFile::Named(NamedFile { file, name: shared.name }))
            })
            .collect();
        self.start_link(false, move |node| async move { node.share_link(files).await });
    }

    /// Takes files from browsers on the local network until `stop_link`.
    pub fn receive_link(&self) {
        self.start_link(true, |node| async move { node.receive_link().await });
    }

    pub fn stop_link(&self) {
        lock(&self.shelf.link).take();
        self.listener.update(Update::LinkStopped);
    }
}

impl Solferry {
    fn start_link<W, F>(&self, receiving: bool, start: W)
    where
        W: FnOnce(Node) -> F + Send + 'static,
        F: Future<Output = solferry_core::Result<Link>> + Send + 'static,
    {
        let kept = self.shelf.clone();
        self.with_node(
            move |node, listener, _| async move {
                lock(&kept.link).take();
                match start(node).await {
                    Ok(link) => {
                        let urls = link.urls().to_vec();
                        *lock(&kept.link) = Some(link);
                        listener.update(Update::LinkReady { urls, receiving });
                    }
                    Err(error) => listener.update(Update::LinkFailed { problem: (&error).into() }),
                }
            },
            None,
        );
    }
}
