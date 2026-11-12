//! What the system has to allow for Solferry to work, and where the person allows it.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

use futures::channel::oneshot;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Permission {
    Notifications,
    #[cfg(target_os = "macos")]
    LocalNetwork,
    #[cfg(windows)]
    Firewall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Allowed,
    Off,
    /// Only macOS asks before an app may notify.
    #[cfg(target_os = "macos")]
    NotAsked,
    /// The system doesn't tell apps, so only its settings can say.
    Unknown,
}

/// What this platform asks for, in the order the person meets it.
#[cfg(target_os = "macos")]
pub const ALL: &[Permission] = &[Permission::LocalNetwork, Permission::Notifications];
#[cfg(windows)]
pub const ALL: &[Permission] = &[Permission::Firewall, Permission::Notifications];
#[cfg(not(any(target_os = "macos", windows)))]
pub const ALL: &[Permission] = &[];

pub fn status(permission: Permission) -> oneshot::Receiver<Status> {
    match permission {
        #[cfg(target_os = "macos")]
        Permission::Notifications => macos::notifications(),
        #[cfg(windows)]
        Permission::Notifications => windows::notifications(),
        #[cfg(not(any(target_os = "macos", windows)))]
        Permission::Notifications => ready(Status::Unknown),
        #[cfg(target_os = "macos")]
        Permission::LocalNetwork => ready(Status::Unknown),
        #[cfg(windows)]
        Permission::Firewall => ready(Status::Unknown),
    }
}

/// Shows the system's prompt, which only macOS has, and answers with what the person chose.
#[cfg(target_os = "macos")]
pub fn ask(permission: Permission) -> oneshot::Receiver<Status> {
    match permission {
        Permission::Notifications => macos::ask_notifications(),
        Permission::LocalNetwork => status(permission),
    }
}

/// The page of the system settings where this permission is changed.
pub fn settings_url(permission: Permission) -> &'static str {
    match permission {
        #[cfg(target_os = "macos")]
        Permission::Notifications => {
            "x-apple.systempreferences:com.apple.Notifications-Settings.extension?id=app.solferry.desktop"
        }
        #[cfg(target_os = "macos")]
        Permission::LocalNetwork => {
            "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_LocalNetwork"
        }
        #[cfg(not(target_os = "macos"))]
        Permission::Notifications => "ms-settings:notifications",
        #[cfg(windows)]
        Permission::Firewall => "windowsdefender://network/",
    }
}

fn ready(status: Status) -> oneshot::Receiver<Status> {
    let (sender, receiver) = oneshot::channel();
    let _ = sender.send(status);
    receiver
}
