use std::sync::{Arc, Mutex, PoisonError};

use ksni::blocking::{Handle, TrayMethods};
use ksni::menu::{MenuItem, StandardItem};
use rust_i18n::t;

use super::{Anchor, ICON_SIZE, INK, Sender, TrayAction, icon};

struct SolferryTray {
    actions: Sender<TrayAction>,
    progress: Option<f32>,
}

impl ksni::Tray for SolferryTray {
    fn id(&self) -> String {
        "solferry".to_owned()
    }

    fn title(&self) -> String {
        "Solferry".to_owned()
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let data = icon::alpha(self.progress)
            .into_iter()
            .flat_map(|alpha| [alpha, INK[0], INK[1], INK[2]])
            .collect();
        let side = i32::try_from(ICON_SIZE).unwrap_or(0);
        vec![ksni::Icon { width: side, height: side, data }]
    }

    fn activate(&mut self, x: i32, y: i32) {
        let _ = self.actions.send(TrayAction::Popover(Anchor::Point { x, y }));
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
    progress: Option<f32>,
    handle: Option<Handle<SolferryTray>>,
}

pub struct Tray {
    actions: Sender<TrayAction>,
    slot: Arc<Mutex<Slot>>,
}

impl Tray {
    pub fn start(actions: Sender<TrayAction>, visible: bool) -> Self {
        let tray = Self {
            actions,
            slot: Arc::new(Mutex::new(Slot { wanted: false, progress: None, handle: None })),
        };
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
                (true, None) => {
                    match (SolferryTray { actions: actions.clone(), progress: slot.progress })
                        .spawn()
                    {
                        Ok(handle) => slot.handle = Some(handle),
                        Err(error) => {
                            tracing::warn!(%error, "tray is unavailable");
                            // Many desktops have no tray, so the window has to stay reachable.
                            let _ = actions.send(TrayAction::Open);
                        }
                    }
                }
                (true, Some(handle)) => slot.handle = Some(handle),
                (false, Some(handle)) => handle.shutdown().wait(),
                (false, None) => {}
            }
        });
    }

    pub fn set_progress(&self, progress: Option<f32>) {
        self.slot.lock().unwrap_or_else(PoisonError::into_inner).progress = progress;
        let slot = self.slot.clone();
        std::thread::spawn(move || {
            let slot = slot.lock().unwrap_or_else(PoisonError::into_inner);
            if let Some(handle) = &slot.handle {
                let progress = slot.progress;
                handle.update(|tray| tray.progress = progress);
            }
        });
    }
}
