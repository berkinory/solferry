use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use iroh::SecretKey;

use crate::error::{Result, StorageContext};

const KEY_FILE: &str = "device.key";

pub(crate) fn load_or_create(data_dir: &Path) -> Result<SecretKey> {
    let path = data_dir.join(KEY_FILE);
    match fs::read(&path) {
        Ok(bytes) => {
            let key: [u8; 32] = bytes.try_into().map_err(|_| std::io::ErrorKind::InvalidData);
            Ok(SecretKey::from_bytes(&key.map_err(std::io::Error::from).at(&path)?))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let key = SecretKey::generate();
            write_private(&path, &key.to_bytes()).at(&path)?;
            Ok(key)
        }
        Err(error) => Err(error).at(&path),
    }
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
