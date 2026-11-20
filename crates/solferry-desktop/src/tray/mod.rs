use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub enum TrayAction {
    Open,
    Quit,
    /// A left click, which opens or closes the quick send popover.
    Popover(Anchor),
}

/// Where the tray icon sits, as the platform tells it.
#[derive(Clone, Copy, Debug)]
pub enum Anchor {
    /// The icon's rectangle in physical pixels, from the top left of the main display.
    #[cfg_attr(
        not(target_os = "macos"),
        expect(
            dead_code,
            reason = "ksni gives a point, and Windows places its flyouts by the corner"
        )
    )]
    Icon { x: f64, y: f64, width: f64, height: f64 },
    /// The point a Linux tray host passed with the click. Many pass (0, 0) instead.
    #[cfg_attr(not(target_os = "linux"), expect(dead_code, reason = "only ksni gives a point"))]
    Point { x: i32, y: i32 },
}

pub struct TrayEvents {
    actions: Receiver<TrayAction>,
}

impl TrayEvents {
    pub fn drain(&self) -> Vec<TrayAction> {
        self.actions.try_iter().collect()
    }
}

mod icon;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(target_os = "linux"))]
mod native;

use icon::{ICON_SIZE, INK};
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(not(target_os = "linux"))]
use native as platform;

/// The ring moves in steps of 2%, so the icon only redraws for a change you could see.
const STEPS: f32 = 50.0;
/// Bounds the redraws of a fast transfer, which could cross a step on every poll.
const REDRAW_EVERY: Duration = Duration::from_millis(250);

/// What the icon shows. `Unknown` after it was hidden, so it redraws when it shows again.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Shown {
    Unknown,
    Mark,
    Step(u8),
}

pub struct Tray {
    icon: platform::Tray,
    shown: Shown,
    drawn_at: Instant,
}

impl Tray {
    pub fn set_visible(&self, visible: bool) {
        self.icon.set_visible(visible);
    }

    /// Shows how far the running transfers are, or the plain mark for `None`. A hidden icon is
    /// left alone and redrawn once it shows again.
    pub fn show_progress(&mut self, progress: Option<f32>, visible: bool) {
        if !visible {
            self.shown = Shown::Unknown;
            return;
        }
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to 0 through 50 first"
        )]
        let wanted = progress.map_or(Shown::Mark, |progress| {
            Shown::Step((progress.clamp(0.0, 1.0) * STEPS).round() as u8)
        });
        let moving = matches!((self.shown, wanted), (Shown::Step(_), Shown::Step(_)));
        if self.shown == wanted || (moving && self.drawn_at.elapsed() < REDRAW_EVERY) {
            return;
        }
        let drawn = match wanted {
            Shown::Step(step) => Some(f32::from(step) / STEPS),
            Shown::Mark | Shown::Unknown => None,
        };
        self.icon.set_progress(drawn);
        self.shown = wanted;
        self.drawn_at = Instant::now();
    }
}

/// The icon is created hidden when `visible` is false, so the setting can bring it back later.
pub fn start(visible: bool) -> (Tray, TrayEvents) {
    let (sender, actions) = channel();
    let icon = platform::Tray::start(sender, visible);
    (Tray { icon, shown: Shown::Mark, drawn_at: Instant::now() }, TrayEvents { actions })
}
