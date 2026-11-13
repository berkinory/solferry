use std::sync::mpsc::{Receiver, Sender, channel};

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
    #[cfg_attr(target_os = "linux", expect(dead_code, reason = "ksni gives a point"))]
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

#[cfg(target_os = "linux")]
mod linux;
#[cfg(not(target_os = "linux"))]
mod native;

#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(not(target_os = "linux"))]
use native as platform;

pub use platform::Tray;

/// The icon is created hidden when `visible` is false, so the setting can bring it back later.
pub fn start(visible: bool) -> (Tray, TrayEvents) {
    let (sender, actions) = channel();
    (platform::Tray::start(sender, visible), TrayEvents { actions })
}

const ICON_SIZE: u32 = 44;
/// The brand's violet reads on light and dark panels alike. macOS tints a template icon itself.
const INK: [u8; 3] = [0x85, 0x80, 0xff];
/// The mark's bounds in the 64 unit space of assets/mark.svg, centered in the icon.
const MARK_CENTER: (f32, f32) = (32.0, 37.0);
const MARK_SPAN: f32 = 52.0;
/// Samples per pixel side, so curved edges come out smooth instead of stepped.
const SAMPLES: u32 = 4;

/// True where the mark covers the point, in the 64 unit space of assets/mark.svg.
fn covered(x: f32, y: f32) -> bool {
    let (dx, dy) = (x - 32.0, y - 34.0);
    let sun = y <= 34.0 && dx * dx + dy * dy <= 400.0;
    let bar = |left: f32, top: f32, width: f32| {
        let (dx, dy) = (x - x.clamp(left + 2.0, left + width - 2.0), y - (top + 2.0));
        dx * dx + dy * dy <= 4.0
    };
    sun || bar(8.0, 40.0, 48.0) || bar(16.0, 48.0, 32.0) || bar(24.0, 56.0, 16.0)
}

/// How much of each pixel of the tray icon the mark covers, row by row.
fn mark_alpha() -> impl Iterator<Item = u8> {
    let float = |value: u32| f32::from(u16::try_from(value).unwrap_or(0));
    let unit = MARK_SPAN / float(ICON_SIZE * SAMPLES);
    let origin = (MARK_CENTER.0 - MARK_SPAN / 2.0, MARK_CENTER.1 - MARK_SPAN / 2.0);
    (0..ICON_SIZE).flat_map(move |row| {
        (0..ICON_SIZE).map(move |column| {
            let hits = (0..SAMPLES * SAMPLES)
                .filter(|sample| {
                    let x = float(column * SAMPLES + sample % SAMPLES) + 0.5;
                    let y = float(row * SAMPLES + sample / SAMPLES) + 0.5;
                    covered(origin.0 + x * unit, origin.1 + y * unit)
                })
                .count();
            u8::try_from(hits * 255 / (SAMPLES * SAMPLES) as usize).unwrap_or(u8::MAX)
        })
    })
}
