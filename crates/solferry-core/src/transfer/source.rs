use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Where a sent file's bytes come from: a path to open, or a file someone already opened, such
/// as a phone's shared content, which may not be reachable by any path this app can open.
#[derive(Debug, Clone)]
pub(crate) enum Source {
    Path(PathBuf),
    Open { name: PathBuf, file: Arc<File> },
}

impl Source {
    /// What errors name this source by.
    pub(crate) fn label(&self) -> &Path {
        match self {
            Self::Path(path) => path,
            Self::Open { name, .. } => name,
        }
    }

    /// A handle that segments read at their own offsets, so they never share a file position.
    pub(crate) fn reader(&self) -> io::Result<Arc<File>> {
        match self {
            Self::Path(path) => File::open(path).map(Arc::new),
            Self::Open { file, .. } => Ok(file.clone()),
        }
    }
}

pub(crate) fn read_at(file: &File, buffer: &mut [u8], offset: u64) -> io::Result<usize> {
    #[cfg(unix)]
    {
        std::os::unix::fs::FileExt::read_at(file, buffer, offset)
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::FileExt::seek_read(file, buffer, offset)
    }
}
