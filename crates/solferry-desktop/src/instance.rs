//! One app per data folder, since two would answer for the same device key. A second launch
//! leaves a note asking the running app to show its window, listing the files it was given, then
//! exits.

use std::fs::{File, OpenOptions, TryLockError};
use std::io::Write;
use std::path::{Path, PathBuf};

const LOCK: &str = "desktop.lock";
const WAKE: &str = "desktop.wake";
/// Where the running app moves a note before reading it, so a launch writing to it right then
/// starts a new one instead of losing its files.
const TAKEN: &str = "desktop.wake.taken";

pub struct Instance {
    _lock: Option<File>,
    wake: PathBuf,
}

impl Instance {
    /// `None` when another copy already runs on `data_dir`, which has been asked to show itself.
    pub fn claim(data_dir: &Path, files: &[PathBuf]) -> Option<Self> {
        let wake = data_dir.join(WAKE);
        let lock = std::fs::create_dir_all(data_dir).and_then(|()| {
            OpenOptions::new().create(true).truncate(false).write(true).open(data_dir.join(LOCK))
        });
        let lock = match lock.map(|file| (file.try_lock(), file)) {
            Ok((Ok(()), file)) => Some(file),
            Ok((Err(TryLockError::WouldBlock), _)) => {
                if let Err(error) = leave_note(&wake, files) {
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

    /// The files later launches passed on since the last look, when any launch asked for the
    /// window.
    pub fn woken(&self) -> Option<Vec<PathBuf>> {
        let taken = self.wake.with_file_name(TAKEN);
        std::fs::rename(&self.wake, &taken).ok()?;
        let note = std::fs::read_to_string(&taken).unwrap_or_default();
        let _ = std::fs::remove_file(&taken);
        Some(note.lines().map(PathBuf::from).collect())
    }
}

/// One path per line. A path that isn't UTF-8 can't be written down, so it's left out.
fn leave_note(wake: &Path, files: &[PathBuf]) -> std::io::Result<()> {
    let mut note = OpenOptions::new().create(true).append(true).open(wake)?;
    let mut lines = String::new();
    for path in files.iter().filter_map(|path| path.to_str()) {
        lines.push_str(path);
        lines.push('\n');
    }
    note.write_all(lines.as_bytes())
}
