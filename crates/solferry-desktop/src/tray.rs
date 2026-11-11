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

pub use platform::Tray;

/// The icon is created hidden when `visible` is false, so the setting can bring it back later.
pub fn start(visible: bool) -> (Tray, TrayEvents) {
    let (sender, actions) = channel();
    (platform::Tray::start(sender, visible), TrayEvents { actions })
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
    use std::sync::{Arc, Mutex, PoisonError};

    use ksni::blocking::{Handle, TrayMethods};
    use ksni::menu::{MenuItem, StandardItem};
    use rust_i18n::t;

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
            vec![item(&t!("tray.open"), TrayAction::Open), item(&t!("tray.quit"), TrayAction::Quit)]
        }
    }

    struct Slot {
        wanted: bool,
        handle: Option<Handle<SolferryTray>>,
    }

    pub struct Tray {
        actions: Sender<TrayAction>,
        slot: Arc<Mutex<Slot>>,
    }

    impl Tray {
        pub fn start(actions: Sender<TrayAction>, visible: bool) -> Self {
            let tray =
                Self { actions, slot: Arc::new(Mutex::new(Slot { wanted: false, handle: None })) };
            tray.set_visible(visible);
            tray
        }

        /// Talking to the tray host can block, so the work runs off the UI thread. The lock keeps
        /// quick toggles in order.
        pub fn set_visible(&self, visible: bool) {
            self.slot.lock().unwrap_or_else(PoisonError::into_inner).wanted = visible;
            let (slot, actions) = (self.slot.clone(), self.actions.clone());
            std::thread::spawn(move || {
                let mut slot = slot.lock().unwrap_or_else(PoisonError::into_inner);
                match (slot.wanted, slot.handle.take()) {
                    (true, None) => match (SolferryTray { actions }).spawn() {
                        Ok(handle) => slot.handle = Some(handle),
                        Err(error) => tracing::warn!(%error, "tray is unavailable"),
                    },
                    (true, Some(handle)) => slot.handle = Some(handle),
                    (false, Some(handle)) => handle.shutdown().wait(),
                    (false, None) => {}
                }
            });
        }
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use rust_i18n::t;
    use tray_icon::menu::{Menu, MenuEvent, MenuItem};
    use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

    use super::{ICON_SIZE, SUN, Sender, TrayAction, mark_pixels};

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
        let pixels = mark_pixels()
            .flat_map(|on| [SUN[0], SUN[1], SUN[2], if on { 0xff } else { 0 }])
            .collect();
        let icon = Icon::from_rgba(pixels, ICON_SIZE, ICON_SIZE).ok()?;
        let tray = TrayIconBuilder::new()
            .with_tooltip("Solferry")
            .with_icon(icon)
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
}
