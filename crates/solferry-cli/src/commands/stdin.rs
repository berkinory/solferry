use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use solferry_core::{Error, NamedFile, Result};

const SNIFF_LEN: usize = 8 * 1024;

/// Copies stdin to an unnamed file, because transfers read segments at offsets, know every size
/// up front and resume by re-reading. It sits in the data dir rather than a temp dir that may live
/// in memory, and disappears when the process exits, even on a crash.
pub async fn spool(data_dir: &Path, name: Option<String>) -> Result<NamedFile> {
    let data_dir = data_dir.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut file =
            tempfile::tempfile_in(&data_dir).map_err(|error| storage(&data_dir, error))?;
        std::io::copy(&mut std::io::stdin().lock(), &mut file)
            .map_err(|error| storage(Path::new("stdin"), error))?;
        let name = match name {
            Some(name) => name,
            None => default_name(&file).map_err(|error| storage(&data_dir, error))?,
        };
        Ok(NamedFile { name, file })
    })
    .await
    .map_err(|error| Error::Protocol(error.to_string()))?
}

/// A pipe has no name, so the start of it decides whether it arrives as text or as bytes.
fn default_name(file: &std::fs::File) -> std::io::Result<String> {
    let mut start = Vec::with_capacity(SNIFF_LEN);
    let mut reader = file;
    std::io::Seek::rewind(&mut reader)?;
    reader.take(SNIFF_LEN as u64).read_to_end(&mut start)?;
    // A character cut off at the end of the sample still counts as text.
    let text = match std::str::from_utf8(&start) {
        Ok(_) => true,
        Err(error) => error.error_len().is_none(),
    };
    Ok(if text { "stdin.txt" } else { "stdin.bin" }.to_owned())
}

fn storage(path: &Path, error: std::io::Error) -> Error {
    Error::Storage { path: PathBuf::from(path), source: Arc::new(error) }
}
