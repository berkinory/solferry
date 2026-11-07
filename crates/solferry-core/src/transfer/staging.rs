use std::fs::{self, File, FileTimes};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use bytes::Bytes;
use iroh::EndpointId;
use solferry_proto::Entry;
use tokio::sync::mpsc;

use super::CHUNK_SIZE;
use super::paths::crosses_symlink;
use super::quarantine::mark_downloaded;
use super::segments::{Progress, segments};
use crate::error::{Error, Result, StorageContext};
use crate::event::TransferId;

const PARTIAL_DIR: &str = ".solferry-partial";
const ABANDONED_AFTER: Duration = Duration::from_hours(7 * 24);
/// How often a segment writer records how far it got, so a restart resumes close to where it stopped.
const PROGRESS_STEP: u64 = 8 * 1024 * 1024;

#[derive(Debug)]
pub(crate) struct Staging {
    root: PathBuf,
}

impl Staging {
    /// The sender picks the transfer id, so the peer is part of the name: nobody can continue
    /// or poison another device's partial files by repeating its id.
    pub(crate) fn open(destination: &Path, peer: EndpointId, id: TransferId) -> Result<Self> {
        let parent = destination.join(PARTIAL_DIR);
        prune_abandoned(&parent);
        let owner = blake3::Hasher::new().update(peer.as_bytes()).update(&id.0).finalize();
        let root = parent.join(owner.to_hex().as_str());
        fs::create_dir_all(&root).at(&root)?;
        Ok(Self { root })
    }

    pub(crate) fn part(&self, index: usize) -> PathBuf {
        self.root.join(index.to_string())
    }

    pub(crate) fn progress_file(&self, index: usize, segment: usize) -> PathBuf {
        self.root.join(format!("{index}.{segment}"))
    }

    /// What each segment already holds. Anything unreadable or impossible counts as nothing.
    pub(crate) fn progress(&self, entries: &[Entry]) -> Progress {
        let held = |index: usize, segment: usize, range: &Range<u64>| {
            if !self.part(index).exists() {
                return 0;
            }
            let recorded = fs::read(self.progress_file(index, segment))
                .ok()
                .and_then(|bytes| bytes.try_into().ok().map(u64::from_le_bytes));
            recorded.filter(|&value| value <= range.end - range.start).unwrap_or(0)
        };
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| {
                let ranges = segments(entry);
                ranges
                    .iter()
                    .enumerate()
                    .map(|(segment, range)| held(index, segment, range))
                    .collect()
            })
            .collect()
    }

    pub(crate) fn finalize(
        &self,
        destination: &Path,
        entries: &[Entry],
        targets: &[PathBuf],
    ) -> Result<Vec<PathBuf>> {
        let mut delivered = Vec::with_capacity(entries.len());
        for (index, (entry, target)) in entries.iter().zip(targets).enumerate() {
            if crosses_symlink(destination, target) {
                return Err(Error::InvalidOffer(format!("{} leaves the destination", entry.path)));
            }
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
            mark_downloaded(&target);
            delivered.push(target);
        }
        fs::remove_dir_all(&self.root).at(&self.root)?;
        if let Some(parent) = self.root.parent() {
            let _ = fs::remove_dir(parent);
        }
        Ok(delivered)
    }
}

fn prune_abandoned(parent: &Path) {
    let Ok(transfers) = fs::read_dir(parent) else {
        return;
    };
    for transfer in transfers.flatten() {
        let abandoned = transfer
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| modified.elapsed().ok())
            .is_some_and(|age| age > ABANDONED_AFTER);
        if abandoned {
            let _ = fs::remove_dir_all(transfer.path());
        }
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
    let stem = target.file_stem().unwrap_or_default().to_string_lossy().into_owned();
    let extension = target.extension().map(|extension| extension.to_string_lossy().into_owned());
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

/// One segment of a partial file: where it lives, which bytes it covers, and how many it has.
pub(crate) struct Segment {
    pub(crate) part: PathBuf,
    pub(crate) progress: PathBuf,
    pub(crate) range: Range<u64>,
    pub(crate) held: u64,
}

/// Writes incoming chunks after the bytes the segment already holds and returns the hash of the
/// whole segment, recording progress as it goes.
pub(crate) fn write_part(
    segment: &Segment,
    mut chunks: mpsc::Receiver<Bytes>,
) -> Result<blake3::Hash> {
    let path = &segment.part;
    let mut file =
        File::options().read(true).write(true).create(true).truncate(false).open(path).at(path)?;
    let mut hasher = blake3::Hasher::new();
    file.seek(SeekFrom::Start(segment.range.start)).at(path)?;
    let mut prefix = (&mut file).take(segment.held);
    let mut buffer = vec![0; CHUNK_SIZE];
    loop {
        let read = prefix.read(&mut buffer).at(path)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    file.seek(SeekFrom::Start(segment.range.start + segment.held)).at(path)?;
    let mut writer = BufWriter::with_capacity(CHUNK_SIZE, file);
    let mut held = segment.held;
    let mut recorded = held;
    let outcome = loop {
        let Some(chunk) = chunks.blocking_recv() else { break Ok(()) };
        if let Err(error) = writer.write_all(&chunk) {
            break Err(error);
        }
        hasher.update(&chunk);
        held += chunk.len() as u64;
        if held - recorded >= PROGRESS_STEP {
            if let Err(error) = writer.flush() {
                break Err(error);
            }
            record(&segment.progress, held);
            recorded = held;
        }
    };
    outcome.and_then(|()| writer.flush()).at(path)?;
    record(&segment.progress, held);
    Ok(hasher.finalize())
}

fn record(progress: &Path, held: u64) {
    let _ = fs::write(progress, held.to_le_bytes());
}

/// Forgets what a segment held, so the next attempt sends it again from the start.
pub(crate) fn forget(segment: &Segment) {
    let _ = fs::remove_file(&segment.progress);
}
