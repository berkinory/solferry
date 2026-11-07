use serde::{Deserialize, Serialize};

/// The operating system a device runs, so other devices can show the right icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Macos,
    Windows,
    Linux,
    Android,
    Ios,
}

const ALL: [Platform; 5] =
    [Platform::Macos, Platform::Windows, Platform::Linux, Platform::Android, Platform::Ios];

impl Platform {
    #[must_use]
    pub const fn current() -> Option<Self> {
        if cfg!(target_os = "macos") {
            Some(Self::Macos)
        } else if cfg!(target_os = "windows") {
            Some(Self::Windows)
        } else if cfg!(target_os = "android") {
            Some(Self::Android)
        } else if cfg!(target_os = "ios") {
            Some(Self::Ios)
        } else if cfg!(target_os = "linux") {
            Some(Self::Linux)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Macos => "macos",
            Self::Windows => "windows",
            Self::Linux => "linux",
            Self::Android => "android",
            Self::Ios => "ios",
        }
    }

    #[must_use]
    pub fn parse(code: &str) -> Option<Self> {
        ALL.into_iter().find(|platform| platform.code() == code)
    }
}

pub(crate) fn current_code() -> String {
    Platform::current().map(Platform::code).unwrap_or_default().to_owned()
}

/// Where only a name fits, the platform rides in front of it, like `macos/Studio`.
pub(crate) fn label(name: &str) -> String {
    match Platform::current() {
        Some(platform) => format!("{}/{name}", platform.code()),
        None => name.to_owned(),
    }
}

pub(crate) fn split_label(label: &str) -> (Option<Platform>, &str) {
    label
        .split_once('/')
        .and_then(|(code, name)| Platform::parse(code).map(|platform| (Some(platform), name)))
        .unwrap_or((None, label))
}
