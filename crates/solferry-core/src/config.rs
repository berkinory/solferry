use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::store;

const CONFIG_FILE: &str = "config.json";
const APP_DIR: &str = "solferry";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Visibility {
    #[default]
    Everyone,
    Hidden,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub name: String,
    /// Relays you run. Empty means this device never contacts a server.
    pub relays: Vec<String>,
    /// PEM file with extra root certificates, for relays behind a private CA.
    pub relay_ca_file: Option<PathBuf>,
    pub visibility: Visibility,
    pub download_dir: PathBuf,
    /// Whether the desktop and Android apps look for new releases.
    pub check_updates: bool,
    /// Whether the desktop app keeps an icon in the tray or menu bar.
    pub show_in_tray: bool,
    /// The desktop app's language, such as `tr`. `None` follows the system.
    pub language: Option<String>,
    /// Whether the desktop app has shown its first launch screen.
    pub onboarded: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: gethostname::gethostname().to_string_lossy().into_owned(),
            relays: Vec::new(),
            relay_ca_file: None,
            visibility: Visibility::default(),
            download_dir: dirs::download_dir()
                .or_else(dirs::home_dir)
                .unwrap_or_else(|| PathBuf::from(".")),
            check_updates: true,
            show_in_tray: true,
            language: None,
            onboarded: false,
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
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join(APP_DIR)
}
