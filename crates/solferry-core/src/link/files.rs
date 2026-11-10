use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::{Error, Result, StorageContext};
use crate::transfer::NamedFile;

/// Something to offer through a link: a path on disk, or a file a phone handed over already open.
#[derive(Debug)]
pub enum LinkFile {
    Path(PathBuf),
    Named(NamedFile),
}

pub(crate) struct Served {
    pub(crate) name: String,
    pub(crate) size: u64,
    pub(crate) file: Arc<File>,
}

/// Opens everything up front, so a file that can't be read fails here and not in the browser.
pub(crate) fn collect(files: Vec<LinkFile>) -> Result<Vec<Served>> {
    let mut served = Vec::new();
    for file in files {
        match file {
            LinkFile::Named(named) => {
                let size = named.file.metadata().at(&named.name)?.len();
                served.push(Served { name: named.name, size, file: Arc::new(named.file) });
            }
            LinkFile::Path(path) if path.is_dir() => folder(&path, &mut served)?,
            LinkFile::Path(path) => {
                let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
                served.push(open(&path, name)?);
            }
        }
    }
    if served.is_empty() {
        return Err(Error::NothingToSend);
    }
    Ok(served)
}

fn folder(root: &Path, served: &mut Vec<Served>) -> Result<()> {
    let base = root.parent().unwrap_or(root);
    for entry in walkdir::WalkDir::new(root).sort_by_file_name() {
        let entry = entry.map_err(std::io::Error::from).at(root)?;
        if entry.file_type().is_file() {
            let relative = entry.path().strip_prefix(base).unwrap_or(entry.path());
            let parts: Vec<_> =
                relative.components().map(|part| part.as_os_str().to_string_lossy()).collect();
            served.push(open(entry.path(), parts.join("/"))?);
        }
    }
    Ok(())
}

fn open(path: &Path, name: String) -> Result<Served> {
    let file = File::open(path).at(path)?;
    let size = file.metadata().at(path)?.len();
    Ok(Served { name, size, file: Arc::new(file) })
}
