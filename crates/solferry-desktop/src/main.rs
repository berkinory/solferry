#![cfg_attr(windows, windows_subsystem = "windows")]

mod assets;
mod format;
mod instance;
mod login;
mod opened;
#[cfg_attr(
    not(any(target_os = "macos", windows)),
    allow(dead_code, reason = "Linux asks for no permissions")
)]
mod permissions;
mod qr;
mod service;
mod shell;
mod state;
mod theme;
#[allow(dead_code)]
mod tokens;
mod tray;
mod updater;
mod views;
mod watch;

use gpui_kit::QuitMode;
use solferry_core::{Config, default_data_dir};
use tracing_subscriber::EnvFilter;

/// The bundle identifier on macOS, and the `AppUserModelID` the installer gives the Start menu entry.
const APP_ID: &str = "app.solferry.desktop";

rust_i18n::i18n!("locales", fallback = "en");

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("SOLFERRY_LOG").unwrap_or_else(|_| EnvFilter::new("off")),
        )
        .with_writer(std::io::stderr)
        .init();
    let data_dir = std::env::var_os("SOLFERRY_DATA_DIR").map_or_else(default_data_dir, Into::into);
    let config = Config::load(&data_dir).unwrap_or_default();
    views::language::apply(config.language.as_deref());
    let launched = opened::from_args();
    let Some(instance) = instance::Instance::claim(&data_dir, &launched) else { return };
    let show_in_tray = config.show_in_tray;
    // A launch at login waits in the tray, or in the Dock on macOS, unless nothing would show it.
    let hidden = std::env::args().any(|arg| arg == login::HIDDEN)
        && (show_in_tray || cfg!(target_os = "macos"));
    let (handle, updates) = service::spawn(data_dir);
    let app =
        gpui_kit::application().with_quit_mode(QuitMode::Explicit).with_assets(assets::Assets);
    app.on_reopen(shell::show_window);
    let (open_handler, opened) = opened::listen(launched);
    app.on_open_urls(open_handler);
    app.run(move |cx| {
        // Windows shows no notifications from an app without one.
        cx.set_app_identity(APP_ID, "Solferry");
        gpui_kit::init(cx);
        theme::install(cx);
        views::keys::install(cx);
        shell::start(handle, updates, tray::start(show_in_tray), (instance, hidden, opened), cx);
    });
}
