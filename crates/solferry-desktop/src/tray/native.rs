use rust_i18n::t;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use super::{ICON_SIZE, INK, Sender, TrayAction, mark_alpha};

pub struct Tray(Option<TrayIcon>);

impl Tray {
    pub fn start(actions: Sender<TrayAction>, visible: bool) -> Self {
        let tray = Self(build(actions));
        tray.set_visible(visible);
        tray
    }

    pub fn set_visible(&self, visible: bool) {
        if let Some(icon) = &self.0
            && let Err(error) = icon.set_visible(visible)
        {
            tracing::warn!(%error, "couldn't change the tray icon");
        }
    }
}

fn build(actions: Sender<TrayAction>) -> Option<TrayIcon> {
    let menu = Menu::new();
    let open = MenuItem::new(t!("tray.open"), true, None);
    let quit = MenuItem::new(t!("tray.quit"), true, None);
    let (open_id, quit_id) = (open.id().clone(), quit.id().clone());
    menu.append_items(&[&open, &quit]).ok()?;
    let pixels = mark_alpha().flat_map(|alpha| [INK[0], INK[1], INK[2], alpha]).collect();
    let icon = Icon::from_rgba(pixels, ICON_SIZE, ICON_SIZE).ok()?;
    let tray = TrayIconBuilder::new().with_tooltip("Solferry");
    #[cfg(target_os = "macos")]
    let tray = tray.with_icon_templated(icon);
    #[cfg(not(target_os = "macos"))]
    let tray = tray.with_icon(icon);
    let tray = tray
        .with_menu(Box::new(menu))
        .build()
        .inspect_err(|error| tracing::warn!(%error, "tray is unavailable"))
        .ok()?;
    std::thread::spawn(move || {
        while let Ok(event) = MenuEvent::receiver().recv() {
            let action = if event.id == open_id {
                TrayAction::Open
            } else if event.id == quit_id {
                TrayAction::Quit
            } else {
                continue;
            };
            let _ = actions.send(action);
        }
    });
    Some(tray)
}
