use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use iroh::EndpointId;
use solferry_proto::Entry;
use walkdir::WalkDir;

use super::source::Source;
use crate::error::{Error, Result};
use crate::event::TransferId;

#[derive(Debug)]
pub(crate) struct Manifest {
    pub(crate) id: TransferId,
    pub(crate) entries: Vec<Entry>,
    pub(crate) sources: Vec<Source>,
}

impl Manifest {
    pub(crate) fn scan(roots: &[PathBuf], sender: EndpointId) -> Result<Self> {
        let mut items = Vec::new();
        for root in roots {
            collect(root, &mut items)?;
        }
        if items.is_empty() {
            return Err(Error::NothingToSend);
        }
        items.sort_by(|a, b| a.0.path.cmp(&b.0.path));
        let id = transfer_id(sender, items.iter().map(|(entry, _)| entry));
        let (entries, sources) =
            items.into_iter().map(|(entry, path)| (entry, Source::Path(path))).unzip();
        Ok(Self { id, entries, sources })
    }
}

/// A file someone handed over already open, as a phone's share sheet does: `name` is what the
/// receiver sees.
#[derive(Debug)]
pub struct NamedFile {
    pub name: String,
    pub file: std::fs::File,
}

impl Manifest {
    pub(crate) fn named(files: Vec<NamedFile>, sender: EndpointId) -> Result<Self> {
        let mut items = Vec::with_capacity(files.len());
        for file in files {
            let label = PathBuf::from(&file.name);
            let metadata = file
                .file
                .metadata()
                .map_err(|error| Error::Storage { path: label.clone(), source: Arc::new(error) })?;
            let name = file.name.rsplit(['/', '\\']).next().unwrap_or_default().to_owned();
            if name.is_empty() || metadata.is_dir() {
                return Err(Error::UnsupportedName(label));
            }
            let entry = Entry {
                path: name,
                size: metadata.len(),
                modified_ms: modified_ms(&metadata),
                is_dir: false,
            };
            items.push((entry, Source::Open { name: label, file: Arc::new(file.file) }));
        }
        if items.is_empty() {
            return Err(Error::NothingToSend);
        }
        let id = transfer_id(sender, items.iter().map(|(entry, _)| entry));
        let (entries, sources) = items.into_iter().unzip();
        Ok(Self { id, entries, sources })
    }
}

fn modified_ms(metadata: &std::fs::Metadata) -> u64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |elapsed| u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX))
}

fn collect(root: &Path, items: &mut Vec<(Entry, PathBuf)>) -> Result<()> {
    let base = root.parent().unwrap_or(Path::new(""));
    for item in WalkDir::new(root).sort_by_file_name() {
        let item = item.map_err(|error| Error::Storage {
            path: error.path().unwrap_or(root).to_owned(),
            source: Arc::new(error.into()),
        })?;
        if item.file_type().is_symlink() {
            continue;
        }
        let relative = item.path().strip_prefix(base).unwrap_or(item.path());
        let path = relative.components().try_fold(String::new(), |mut joined, part| {
            let part = part.as_os_str().to_str()?;
            if !joined.is_empty() {
                joined.push('/');
            }
            joined.push_str(part);
            Some(joined)
        });
        let path = path.ok_or_else(|| Error::UnsupportedName(item.path().to_owned()))?;
        let metadata = item.metadata().map_err(|error| Error::Storage {
            path: item.path().to_owned(),
            source: Arc::new(error.into()),
        })?;
        let modified_ms = modified_ms(&metadata);
        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };
        items.push((Entry { path, size, modified_ms, is_dir }, item.path().to_owned()));
    }
    Ok(())
}

fn transfer_id<'a>(sender: EndpointId, entries: impl Iterator<Item = &'a Entry>) -> TransferId {
    let mut hasher = blake3::Hasher::new();
    hasher.update(sender.as_bytes());
    for entry in entries {
        hasher.update(&(entry.path.len() as u64).to_le_bytes());
        hasher.update(entry.path.as_bytes());
        hasher.update(&entry.size.to_le_bytes());
        hasher.update(&entry.modified_ms.to_le_bytes());
    }
    TransferId(*hasher.finalize().as_bytes())
}
