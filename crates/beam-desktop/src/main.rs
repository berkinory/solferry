#![cfg_attr(windows, windows_subsystem = "windows")]

mod errors;
mod format;
mod notify;
mod qr;
mod service;
mod shell;
mod state;
mod tray;
mod views;

use beam_core::default_data_dir;
use gpui_kit::QuitMode;
use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("BEAM_LOG").unwrap_or_else(|_| EnvFilter::new("off")),
        )
        .with_writer(std::io::stderr)
        .init();
    let data_dir = std::env::var_os("BEAM_DATA_DIR").map_or_else(default_data_dir, Into::into);
    let (handle, updates) = service::spawn(data_dir);
    let tray = tray::start();
    gpui_kit::application()
        .with_quit_mode(QuitMode::Explicit)
        .with_assets(gpui_kit::assets::Assets)
        .run(move |cx| {
            gpui_kit::init(cx);
            shell::start(handle, updates, tray, cx);
        });
}
