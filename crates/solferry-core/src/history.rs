use std::path::{Path, PathBuf};

use iroh::EndpointId;
use serde::{Deserialize, Serialize};

use crate::event::Direction;
use crate::store;

const HISTORY_FILE: &str = "history.json";
const KEPT: usize = 200;

/// A transfer that is over, for the history screen.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PastTransfer {
    pub peer: EndpointId,
    pub peer_name: String,
    pub label: String,
    pub direction: Direction,
    pub files: usize,
    pub bytes: u64,
    pub finished_ms: u64,
    pub completed: bool,
}

#[derive(Debug)]
pub(crate) struct History {
    path: PathBuf,
    entries: Vec<PastTransfer>,
}

impl History {
    /// History is a convenience, so a damaged file starts it over instead of stopping the app.
    pub(crate) fn load(data_dir: &Path) -> Self {
        let path = data_dir.join(HISTORY_FILE);
        let entries = store::read_json(&path).ok().flatten().unwrap_or_default();
        Self { path, entries }
    }

    pub(crate) fn entries(&self) -> &[PastTransfer] {
        &self.entries
    }

    pub(crate) fn add(&mut self, entry: PastTransfer) {
        self.entries.insert(0, entry);
        self.entries.truncate(KEPT);
        self.save();
    }

    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.save();
    }

    fn save(&self) {
        if let Err(error) = store::write_json(&self.path, &self.entries) {
            tracing::warn!(%error, "could not save the transfer history");
        }
    }
}
