//! Files the system asks the app to open: dropped on the Dock icon, or handed over by the share
//! extension. They arrive before the app has finished launching too, so they wait in a channel.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};

use url::Url;

pub struct OpenedFiles(Receiver<PathBuf>);

impl OpenedFiles {
    pub fn drain(&self) -> Vec<PathBuf> {
        self.0.try_iter().collect()
    }
}

/// The handler goes to `Application::on_open_urls`, the receiver to the shell.
pub fn listen() -> (impl FnMut(Vec<String>) + 'static, OpenedFiles) {
    let (sender, receiver): (Sender<PathBuf>, _) = channel();
    let handler = move |urls: Vec<String>| {
        for url in urls {
            let path = Url::parse(&url)
                .ok()
                .filter(|url| url.scheme() == "file")
                .and_then(|url| url.to_file_path().ok());
            if let Some(path) = path {
                let _ = sender.send(path);
            } else {
                tracing::warn!(%url, "ignored a URL that isn't a local file");
            }
        }
    };
    (handler, OpenedFiles(receiver))
}
