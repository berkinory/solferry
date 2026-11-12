//! One app per data folder, since two would answer for the same device key. A second launch
//! leaves a note asking the running app to show its window, then exits.

use std::fs::{File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};

const LOCK: &str = "desktop.lock";
const WAKE: &str = "desktop.wake";

pub struct Instance {
    _lock: Option<File>,
    wake: PathBuf,
}

impl Instance {
    /// `None` when another copy already runs on `data_dir`, which has been asked to show itself.
    pub fn claim(data_dir: &Path) -> Option<Self> {
        let wake = data_dir.join(WAKE);
        let lock = std::fs::create_dir_all(data_dir).and_then(|()| {
            OpenOptions::new().create(true).truncate(false).write(true).open(data_dir.join(LOCK))
        });
        let lock = match lock.map(|file| (file.try_lock(), file)) {
            Ok((Ok(()), file)) => Some(file),
            Ok((Err(TryLockError::WouldBlock), _)) => {
                if let Err(error) = std::fs::write(&wake, b"") {
                    tracing::warn!(%error, "couldn't reach the running app");
                }
                return None;
            }
            Ok((Err(TryLockError::Error(error)), _)) | Err(error) => {
                tracing::warn!(%error, "couldn't make sure only one app runs");
                None
            }
        };
        let _ = std::fs::remove_file(&wake);
        Some(Self { _lock: lock, wake })
    }

    /// Whether a later launch asked for the window since the last look.
    pub fn woken(&self) -> bool {
        std::fs::remove_file(&self.wake).is_ok()
    }
}
