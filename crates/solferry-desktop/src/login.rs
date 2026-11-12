//! Opening Solferry when the user logs in: a launch agent on macOS, the Run key on Windows and an
//! XDG autostart entry on Linux. The system's entry is the only record, so nothing can drift.

use auto_launch::{AutoLaunch, AutoLaunchBuilder, WindowsEnableMode};

/// Tells a launch at login to start in the tray instead of opening the window.
pub const HIDDEN: &str = "--hidden";

/// What a login should start. An `AppImage` runs from a mount that changes every launch, so it
/// names the image file instead.
fn entry() -> Option<AutoLaunch> {
    let path = std::env::var_os("APPIMAGE")
        .filter(|_| cfg!(target_os = "linux"))
        .map(Into::into)
        .or_else(|| std::env::current_exe().ok())?;
    let path = path.to_str()?;
    // Windows and Linux get one command line, where a path with spaces needs quotes. The macOS
    // launch agent lists the arguments apart.
    let path = if cfg!(target_os = "macos") { path.to_owned() } else { format!("\"{path}\"") };
    AutoLaunchBuilder::new()
        .set_app_name("Solferry")
        .set_app_path(&path)
        .set_args(&[HIDDEN])
        .set_windows_enable_mode(WindowsEnableMode::CurrentUser)
        // Lets the macOS login items list show the app instead of the developer's name.
        .set_bundle_identifiers(&["app.solferry.desktop"])
        .build()
        .ok()
}

pub fn enabled() -> bool {
    entry().is_some_and(|entry| entry.is_enabled().unwrap_or(false))
}

pub fn set(on: bool) {
    let Some(entry) = entry() else { return };
    if let Err(error) = if on { entry.enable() } else { entry.disable() } {
        tracing::warn!(%error, "changing open at login failed");
    }
}
