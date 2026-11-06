use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::store;

const CONFIG_FILE: &str = "config.json";
const APP_DIR: &str = "beam";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelayPolicy {
    /// Servers help devices find each other, file data only takes direct paths.
    #[default]
    HandshakeOnly,
    /// File data may use relays you added or that a paired device shared with you.
    OwnRelays,
    /// File data may fall back to any relay.
    Always,
    /// No servers at all: local network and overlay networks only.
    Off,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Visibility {
    #[default]
    Everyone,
    KnownDevices,
    Hidden,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub name: String,
    pub relay_policy: RelayPolicy,
    pub relays: Vec<String>,
    pub visibility: Visibility,
    pub download_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: gethostname::gethostname().to_string_lossy().into_owned(),
            relay_policy: RelayPolicy::default(),
            relays: Vec::new(),
            visibility: Visibility::default(),
            download_dir: dirs::download_dir()
                .or_else(dirs::home_dir)
                .unwrap_or_else(|| PathBuf::from(".")),
        }
    }
}

impl Config {
    pub fn load(data_dir: &Path) -> Result<Self> {
        Ok(store::read_json(&data_dir.join(CONFIG_FILE))?.unwrap_or_default())
    }

    pub fn save(&self, data_dir: &Path) -> Result<()> {
        store::write_json(&data_dir.join(CONFIG_FILE), self)
    }
}

#[must_use]
pub fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(APP_DIR)
}
