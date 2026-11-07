use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{Error, Result, StorageContext};

pub(crate) fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).at(path),
    };
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(|source| Error::CorruptData { path: path.to_owned(), source: source.into() })
}

pub(crate) fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|source| Error::CorruptData { path: path.to_owned(), source: source.into() })?;
    let tmp = path.with_extension("tmp");
    let mut file = File::create(&tmp).at(&tmp)?;
    file.write_all(&bytes).at(&tmp)?;
    file.sync_all().at(&tmp)?;
    fs::rename(&tmp, path).at(path)
}
