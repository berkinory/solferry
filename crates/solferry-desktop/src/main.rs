#![cfg_attr(windows, windows_subsystem = "windows")]

mod assets;
mod format;
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

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("SOLFERRY_LOG").unwrap_or_else(|_| EnvFilter::new("off")),
        )
        .with_writer(std::io::stderr)
        .init();
    let data_dir = std::env::var_os("SOLFERRY_DATA_DIR").map_or_else(default_data_dir, Into::into);
    let show_in_tray = Config::load(&data_dir).map_or(true, |config| config.show_in_tray);
    let (handle, updates) = service::spawn(data_dir);
    let app =
        gpui_kit::application().with_quit_mode(QuitMode::Explicit).with_assets(assets::Assets);
    app.on_reopen(shell::show_window);
    app.run(move |cx| {
        gpui_kit::init(cx);
        theme::install(cx);
        views::keys::install(cx);
        shell::start(handle, updates, tray::start(show_in_tray), cx);
    });
}
