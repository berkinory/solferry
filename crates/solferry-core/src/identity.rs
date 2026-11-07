use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use iroh::SecretKey;

use crate::error::{Result, StorageContext};

const KEY_FILE: &str = "device.key";
const KEY_LEN: usize = 32;

pub(crate) fn prepare_private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path).at(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).at(path)?;
    }
    Ok(())
}

/// Loads this device's key, keeping it in the Keychain on macOS and the Credential Manager on
/// Windows. A key still in the data folder moves there once it reads back intact. Elsewhere, or
/// when the store does not answer, the key stays in the folder, readable only by this user, the
/// way SSH keeps keys: Linux secret services can be locked, missing, or wait on a password prompt.
pub(crate) fn load_or_create(data_dir: &Path, use_keychain: bool) -> Result<SecretKey> {
    let path = data_dir.join(KEY_FILE);
    let stored = read_file(&path)?;
    let keychain = use_keychain.then(|| keychain::Slot::open(data_dir)).flatten();
    let Some(slot) = keychain else {
        return match stored {
            Some(key) => Ok(key),
            None => create_file(&path),
        };
    };
    match slot.read() {
        Ok(Some(key)) => {
            if stored.is_some() {
                let _ = fs::remove_file(&path);
            }
            Ok(key)
        }
        Ok(None) => {
            let key = stored.unwrap_or_else(SecretKey::generate);
            if slot.store(&key) {
                let _ = fs::remove_file(&path);
                Ok(key)
            } else if path.exists() {
                Ok(key)
            } else {
                write_private(&path, &key.to_bytes()).at(&path)?;
                Ok(key)
            }
        }
        Err(()) => match stored {
            Some(key) => Ok(key),
            None => create_file(&path),
        },
    }
}

fn read_file(path: &Path) -> Result<Option<SecretKey>> {
    match fs::read(path) {
        Ok(bytes) => {
            let key: std::result::Result<[u8; KEY_LEN], _> = bytes.try_into();
            let key = key.map_err(|_| std::io::Error::from(std::io::ErrorKind::InvalidData));
            Ok(Some(SecretKey::from_bytes(&key.at(path)?)))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).at(path),
    }
}

fn create_file(path: &Path) -> Result<SecretKey> {
    let key = SecretKey::generate();
    write_private(path, &key.to_bytes()).at(path)?;
    Ok(key)
}

fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
mod keychain {
    use std::path::Path;
    use std::sync::{Arc, mpsc};
    use std::time::Duration;

    use iroh::SecretKey;
    use keyring::Entry;

    use super::KEY_LEN;

    const SERVICE: &str = "Solferry";
    /// The store may wait on the person to allow access. Past this, start with the file instead.
    const ANSWER_WAIT: Duration = Duration::from_secs(30);

    pub(super) struct Slot(Arc<Entry>);

    /// Runs a store call on its own thread, so one that never answers cannot hold up startup.
    fn guarded<T: Send + 'static>(call: impl FnOnce() -> T + Send + 'static) -> Option<T> {
        let (answer, waiting) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = answer.send(call());
        });
        waiting.recv_timeout(ANSWER_WAIT).ok()
    }

    impl Slot {
        /// One entry per data folder, so separate profiles on one machine keep separate keys.
        pub(super) fn open(data_dir: &Path) -> Option<Self> {
            let folder = data_dir.canonicalize().unwrap_or_else(|_| data_dir.to_owned());
            let account = format!("device key {}", folder.display());
            Entry::new(SERVICE, &account).ok().map(|entry| Self(Arc::new(entry)))
        }

        /// `Ok(None)` when the keychain works but holds no key, `Err` when there is no keychain.
        pub(super) fn read(&self) -> Result<Option<SecretKey>, ()> {
            let entry = self.0.clone();
            let Some(answer) = guarded(move || entry.get_secret()) else {
                tracing::info!("the keychain did not answer, keeping the device key in a file");
                return Err(());
            };
            match answer {
                Ok(bytes) => {
                    let key: [u8; KEY_LEN] = bytes.try_into().map_err(|_| ())?;
                    Ok(Some(SecretKey::from_bytes(&key)))
                }
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(error) => {
                    tracing::info!(%error, "no usable keychain, keeping the device key in a file");
                    Err(())
                }
            }
        }

        /// Stores the key and reads it back, so a keychain that silently drops it is never trusted.
        pub(super) fn store(&self, key: &SecretKey) -> bool {
            let (entry, bytes) = (self.0.clone(), key.to_bytes());
            let saved = guarded(move || entry.set_secret(&bytes).is_ok()).unwrap_or(false);
            saved && matches!(self.read(), Ok(Some(stored)) if stored.to_bytes() == key.to_bytes())
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod keychain {
    use std::path::Path;

    use iroh::SecretKey;

    /// No keychain here, so no slot can exist.
    pub(super) enum Slot {}

    impl Slot {
        pub(super) fn open(_: &Path) -> Option<Self> {
            None
        }

        pub(super) fn read(&self) -> Result<Option<SecretKey>, ()> {
            match *self {}
        }

        pub(super) fn store(&self, _: &SecretKey) -> bool {
            match *self {}
        }
    }
}
