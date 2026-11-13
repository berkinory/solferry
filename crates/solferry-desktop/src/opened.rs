//! Files the app is asked to send: dropped on the Dock icon, handed over by the share extension,
//! or named on the command line. They arrive before the app has finished launching too, so they
//! wait in a channel.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};

use url::Url;

pub struct OpenedFiles(Receiver<PathBuf>);

impl OpenedFiles {
    pub fn drain(&self) -> Vec<PathBuf> {
        self.0.try_iter().collect()
    }
}

/// Paths this launch was given, made absolute so another running copy can find them too. Flags
/// such as the one for a launch at login aren't paths.
pub fn from_args() -> Vec<PathBuf> {
    std::env::args_os()
        .skip(1)
        .filter(|arg| !arg.to_string_lossy().starts_with('-'))
        .filter_map(|arg| std::path::absolute(Path::new(&arg)).ok())
        .filter(|path| path.exists())
        .collect()
}

/// Starts with `launched`. The handler goes to `Application::on_open_urls`, the receiver to the
/// shell.
pub fn listen(launched: Vec<PathBuf>) -> (impl FnMut(Vec<String>) + 'static, OpenedFiles) {
    let (sender, receiver): (Sender<PathBuf>, _) = channel();
    for path in launched {
        let _ = sender.send(path);
    }
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
