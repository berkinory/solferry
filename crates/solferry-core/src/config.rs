use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::store;

const CONFIG_FILE: &str = "config.json";
const APP_DIR: &str = "solferry";
const MIB: u64 = 1024 * 1024;

/// The relay speed limits the apps offer, in bytes per second.
pub const RELAY_SPEED_LIMITS: [u64; 6] = [MIB, 2 * MIB, 5 * MIB, 10 * MIB, 25 * MIB, 50 * MIB];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Visibility {
    #[default]
    Everyone,
    Hidden,
}

/// Light or dark, for the apps that draw their own windows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Appearance {
    /// Follows the system, and changes with it.
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
#[expect(clippy::struct_excessive_bools, reason = "each switch is a separate setting")]
pub struct Config {
    pub name: String,
    /// Relays you run. Empty means this device never contacts a server.
    pub relays: Vec<String>,
    /// PEM file with extra root certificates, for relays behind a private CA.
    pub relay_ca_file: Option<PathBuf>,
    /// Bytes per second each way while files go through a relay. Direct paths are never limited.
    pub relay_speed_limit: Option<u64>,
    pub visibility: Visibility,
    pub download_dir: PathBuf,
    /// Whether the desktop and Android apps look for new releases.
    pub check_updates: bool,
    /// Whether the desktop app keeps an icon in the tray or menu bar.
    pub show_in_tray: bool,
    /// The desktop app's language, such as `tr`. `None` follows the system.
    pub language: Option<String>,
    pub appearance: Appearance,
    /// Whether the macOS app sends HEIC photos as JPEG to Linux and Windows devices.
    pub convert_heic: bool,
    /// Whether the desktop app has shown its first launch screen.
    pub onboarded: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: gethostname::gethostname().to_string_lossy().into_owned(),
            relays: Vec::new(),
            relay_ca_file: None,
            relay_speed_limit: None,
            visibility: Visibility::default(),
            download_dir: dirs::download_dir()
                .or_else(dirs::home_dir)
                .unwrap_or_else(|| PathBuf::from(".")),
            check_updates: true,
            show_in_tray: true,
            language: None,
            appearance: Appearance::System,
            convert_heic: true,
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
