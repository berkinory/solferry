use std::fs::{self, File, FileTimes};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use beam_proto::Entry;
use bytes::Bytes;
use tokio::sync::mpsc;

use super::CHUNK_SIZE;
use crate::error::{Result, StorageContext};
use crate::event::TransferId;

const PARTIAL_DIR: &str = ".beam-partial";

#[derive(Debug)]
pub(crate) struct Staging {
    root: PathBuf,
}

impl Staging {
    pub(crate) fn open(destination: &Path, id: TransferId) -> Result<Self> {
        let root = destination.join(PARTIAL_DIR).join(hex::encode(id.0));
        fs::create_dir_all(&root).at(&root)?;
        Ok(Self { root })
    }

    pub(crate) fn part(&self, index: usize) -> PathBuf {
        self.root.join(index.to_string())
    }

    pub(crate) fn offsets(&self, entries: &[Entry]) -> Vec<u64> {
        let offset = |(index, entry): (usize, &Entry)| {
            let existing = fs::metadata(self.part(index)).map_or(0, |metadata| metadata.len());
            if entry.is_dir || existing > entry.size {
                0
            } else {
                existing
            }
        };
        entries.iter().enumerate().map(offset).collect()
    }

    pub(crate) fn finalize(
        &self,
        destination: &Path,
        entries: &[Entry],
        targets: &[PathBuf],
    ) -> Result<Vec<PathBuf>> {
        let mut delivered = Vec::with_capacity(entries.len());
        for (index, (entry, target)) in entries.iter().zip(targets).enumerate() {
            let target = destination.join(target);
            if entry.is_dir {
                fs::create_dir_all(&target).at(&target)?;
                continue;
            }
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).at(parent)?;
            }
            let target = free_path(target);
            let part = self.part(index);
            restore_modified(&part, entry.modified_ms);
            fs::rename(&part, &target).at(&target)?;
            delivered.push(target);
        }
        fs::remove_dir_all(&self.root).at(&self.root)?;
        if let Some(parent) = self.root.parent() {
            let _ = fs::remove_dir(parent);
        }
        Ok(delivered)
    }
}

fn restore_modified(path: &Path, modified_ms: u64) {
    let modified = UNIX_EPOCH + Duration::from_millis(modified_ms);
    if let Ok(file) = File::options().write(true).open(path) {
        let _ = file.set_times(FileTimes::new().set_modified(modified));
    }
}

fn free_path(target: PathBuf) -> PathBuf {
    if !target.exists() {
        return target;
    }
    let stem = target
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let extension = target
        .extension()
        .map(|extension| extension.to_string_lossy().into_owned());
    (1..=u32::MAX)
        .map(|attempt| {
            let name = match &extension {
                Some(extension) => format!("{stem} ({attempt}).{extension}"),
                None => format!("{stem} ({attempt})"),
            };
            target.with_file_name(name)
        })
        .find(|candidate| !candidate.exists())
        .unwrap_or(target)
}

/// Appends incoming chunks to a partial file and returns the hash of the complete content.
pub(crate) fn write_part(
    path: &Path,
    offset: u64,
    mut chunks: mpsc::Receiver<Bytes>,
) -> Result<blake3::Hash> {
    let mut file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .at(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut prefix = (&mut file).take(offset);
    let mut buffer = vec![0; CHUNK_SIZE];
    loop {
        let read = prefix.read(&mut buffer).at(path)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    file.set_len(offset).at(path)?;
    file.seek(SeekFrom::Start(offset)).at(path)?;
    let mut writer = BufWriter::with_capacity(CHUNK_SIZE, file);
    while let Some(chunk) = chunks.blocking_recv() {
        writer.write_all(&chunk).at(path)?;
        hasher.update(&chunk);
    }
    writer.flush().at(path)?;
    Ok(hasher.finalize())
}
