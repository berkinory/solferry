use rust_i18n::t;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use super::{Anchor, ICON_SIZE, INK, Sender, TrayAction, icon};

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

    pub fn set_progress(&self, progress: Option<f32>) {
        let Some(tray) = &self.0 else { return };
        let Some(icon) = picture(progress) else { return };
        #[cfg(target_os = "macos")]
        let result = tray.set_icon_templated(Some(icon));
        #[cfg(not(target_os = "macos"))]
        let result = tray.set_icon(Some(icon));
        if let Err(error) = result {
            tracing::warn!(%error, "couldn't redraw the tray icon");
        }
    }
}

fn picture(progress: Option<f32>) -> Option<Icon> {
    let pixels = icon::alpha(progress)
        .into_iter()
        .flat_map(|alpha| [INK[0], INK[1], INK[2], alpha])
        .collect();
    Icon::from_rgba(pixels, ICON_SIZE, ICON_SIZE).ok()
}

fn build(actions: Sender<TrayAction>) -> Option<TrayIcon> {
    let menu = Menu::new();
    let open = MenuItem::new(t!("tray.open"), true, None);
    let quit = MenuItem::new(t!("tray.quit"), true, None);
    let (open_id, quit_id) = (open.id().clone(), quit.id().clone());
    menu.append_items(&[&open, &quit]).ok()?;
    let icon = picture(None)?;
    let tray = TrayIconBuilder::new().with_tooltip("Solferry");
    #[cfg(target_os = "macos")]
    let tray = tray.with_icon_templated(icon);
    #[cfg(not(target_os = "macos"))]
    let tray = tray.with_icon(icon);
    let tray = tray
        .with_menu(Box::new(menu))
        // A left click opens the popover, and the menu stays on a right click.
        .with_menu_on_left_click(false)
        .build()
        .inspect_err(|error| tracing::warn!(%error, "tray is unavailable"))
        .ok()?;
    let clicks = actions.clone();
    std::thread::spawn(move || {
        while let Ok(event) = TrayIconEvent::receiver().recv() {
            if let TrayIconEvent::Click {
                rect,
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let anchor = Anchor::Icon {
                    x: rect.position.x,
                    y: rect.position.y,
                    width: f64::from(rect.size.width),
                    height: f64::from(rect.size.height),
                };
                let _ = clicks.send(TrayAction::Popover(anchor));
            }
        }
    });
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
