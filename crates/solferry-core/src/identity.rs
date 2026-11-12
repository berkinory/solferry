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

/// Loads this device's key, or creates one. It stays in a file readable only by this user, the
/// way SSH keeps keys: OS keychains ask the person to allow every binary that did not create the
/// entry, and the app and the CLI share one key.
pub(crate) fn load_or_create(data_dir: &Path) -> Result<SecretKey> {
    let path = data_dir.join(KEY_FILE);
    match read_file(&path)? {
        Some(key) => Ok(key),
        None => create_file(&path),
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
