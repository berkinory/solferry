use std::sync::mpsc::{Receiver, Sender, channel};

#[derive(Clone, Copy, Debug)]
pub enum TrayAction {
    Open,
    Quit,
}

pub struct TrayEvents {
    actions: Receiver<TrayAction>,
}

impl TrayEvents {
    pub fn drain(&self) -> Vec<TrayAction> {
        self.actions.try_iter().collect()
    }
}

pub fn start() -> TrayEvents {
    let (sender, actions) = channel();
    platform::start(sender);
    TrayEvents { actions }
}

const ICON_SIZE: u32 = 32;
const SUN: [u8; 3] = [0xff, 0xb8, 0x1f];

/// True where the mark covers the point, in the 64 unit space of assets/mark.svg.
fn covered(x: f32, y: f32) -> bool {
    let (dx, dy) = (x - 32.0, y - 34.0);
    let sun = y <= 34.0 && dx * dx + dy * dy <= 400.0;
    let bar = |left: f32, top: f32, width: f32| {
        x >= left && x <= left + width && y >= top && y <= top + 4.0
    };
    sun || bar(8.0, 40.0, 48.0) || bar(16.0, 48.0, 32.0) || bar(24.0, 56.0, 16.0)
}

/// Whether each pixel of the tray icon is part of the mark, row by row.
fn mark_pixels() -> impl Iterator<Item = bool> {
    let scale = 64.0 / 32.0;
    (0..ICON_SIZE).flat_map(move |row| {
        (0..ICON_SIZE).map(move |column| {
            let center = |value: u32| (f32::from(u16::try_from(value).unwrap_or(0)) + 0.5) * scale;
            covered(center(column), center(row))
        })
    })
}

#[cfg(target_os = "linux")]
mod platform {
    use ksni::blocking::TrayMethods;
    use ksni::menu::{MenuItem, StandardItem};

    use super::{ICON_SIZE, SUN, Sender, TrayAction, mark_pixels};

    struct SolferryTray {
        actions: Sender<TrayAction>,
    }

    impl ksni::Tray for SolferryTray {
        fn id(&self) -> String {
            "solferry".to_owned()
        }

        fn title(&self) -> String {
            "Solferry".to_owned()
        }

        fn icon_pixmap(&self) -> Vec<ksni::Icon> {
            let data = mark_pixels()
                .flat_map(|on| {
                    let alpha = if on { 0xff } else { 0 };
                    [alpha, SUN[0], SUN[1], SUN[2]]
                })
                .collect();
            let side = i32::try_from(ICON_SIZE).unwrap_or(0);
            vec![ksni::Icon { width: side, height: side, data }]
        }

        fn activate(&mut self, _x: i32, _y: i32) {
            let _ = self.actions.send(TrayAction::Open);
        }

        fn menu(&self) -> Vec<MenuItem<Self>> {
            let item = |label: &str, action: TrayAction| {
                StandardItem {
                    label: label.to_owned(),
                    activate: Box::new(move |tray: &mut Self| {
                        let _ = tray.actions.send(action);
                    }),
                    ..StandardItem::default()
                }
                .into()
            };
            vec![item("Open Solferry", TrayAction::Open), item("Quit", TrayAction::Quit)]
        }
    }

    pub fn start(actions: Sender<TrayAction>) {
        std::thread::spawn(move || match (SolferryTray { actions }).spawn() {
            Ok(handle) => std::mem::forget(handle),
            Err(error) => tracing::warn!(%error, "tray is unavailable"),
        });
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use tray_icon::menu::{Menu, MenuEvent, MenuItem};
    use tray_icon::{Icon, TrayIconBuilder};

    use super::{ICON_SIZE, SUN, Sender, TrayAction, mark_pixels};

    pub fn start(actions: Sender<TrayAction>) {
        let menu = Menu::new();
        let open = MenuItem::new("Open Solferry", true, None);
        let quit = MenuItem::new("Quit", true, None);
        let (open_id, quit_id) = (open.id().clone(), quit.id().clone());
        if menu.append_items(&[&open, &quit]).is_err() {
            return;
        }
        let pixels = mark_pixels()
            .flat_map(|on| [SUN[0], SUN[1], SUN[2], if on { 0xff } else { 0 }])
            .collect();
        let Ok(icon) = Icon::from_rgba(pixels, ICON_SIZE, ICON_SIZE) else { return };
        let tray = TrayIconBuilder::new()
            .with_tooltip("Solferry")
            .with_icon(icon)
            .with_menu(Box::new(menu))
            .build();
        match tray {
            Ok(tray) => std::mem::forget(tray),
            Err(error) => tracing::warn!(%error, "tray is unavailable"),
        }
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
    }
}
