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

#[cfg(target_os = "linux")]
mod platform {
    use ksni::blocking::TrayMethods;
    use ksni::menu::{MenuItem, StandardItem};

    use super::{Sender, TrayAction};

    struct BeamTray {
        actions: Sender<TrayAction>,
    }

    impl ksni::Tray for BeamTray {
        fn id(&self) -> String {
            "beam".to_owned()
        }

        fn title(&self) -> String {
            "beam".to_owned()
        }

        fn icon_name(&self) -> String {
            "folder-download".to_owned()
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
            vec![item("Open beam", TrayAction::Open), item("Quit", TrayAction::Quit)]
        }
    }

    pub fn start(actions: Sender<TrayAction>) {
        std::thread::spawn(move || match (BeamTray { actions }).spawn() {
            Ok(handle) => std::mem::forget(handle),
            Err(error) => tracing::warn!(%error, "tray is unavailable"),
        });
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    use tray_icon::menu::{Menu, MenuEvent, MenuItem};
    use tray_icon::{Icon, TrayIconBuilder};

    use super::{Sender, TrayAction};

    const ICON_SIZE: u32 = 32;

    pub fn start(actions: Sender<TrayAction>) {
        let menu = Menu::new();
        let open = MenuItem::new("Open beam", true, None);
        let quit = MenuItem::new("Quit", true, None);
        let (open_id, quit_id) = (open.id().clone(), quit.id().clone());
        if menu.append_items(&[&open, &quit]).is_err() {
            return;
        }
        let pixels = [0x4f, 0x8c, 0xff, 0xff].repeat((ICON_SIZE * ICON_SIZE) as usize);
        let Ok(icon) = Icon::from_rgba(pixels, ICON_SIZE, ICON_SIZE) else { return };
        let tray = TrayIconBuilder::new()
            .with_tooltip("beam")
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
