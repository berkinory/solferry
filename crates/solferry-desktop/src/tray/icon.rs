//! The tray icon, drawn in code: the mark, inside a ring that fills while files move.

use std::f32::consts::TAU;

pub const ICON_SIZE: u32 = 44;
/// The brand's violet reads on light and dark panels alike. macOS tints a template icon itself.
pub const INK: [u8; 3] = [0x85, 0x80, 0xff];
/// The mark's bounds in the 64 unit space of assets/mark.svg, centered in the icon.
const MARK_CENTER: (f32, f32) = (32.0, 37.0);
/// macOS draws the icon 22 points tall, so the mark gets more room around it there to come out
/// near 16 points, the size of the system's own menu bar icons.
const MARK_SPAN: f32 = if cfg!(target_os = "macos") { 66.0 } else { 52.0 };
/// How far the mark reaches from its center, in the same units.
const MARK_REACH: f32 = 25.0;
/// The ring's outer radius in pixels, kept near the mark's own size on macOS for the same reason.
const RING_OUTER: f32 = if cfg!(target_os = "macos") { 19.5 } else { 21.5 };
const RING_WIDTH: f32 = 3.5;
/// Space between the ring and the smaller mark inside it.
const RING_GAP: f32 = 2.5;
/// Coverage weights per sample. The unfilled part of the ring stays faint.
const FULL: u32 = 10;
const TRACK: u32 = 3;
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

/// The ring's weight at a point given in pixels from the icon's center. It fills clockwise
/// from 12 o'clock.
fn ring(dx: f32, dy: f32, progress: f32) -> u32 {
    let distance = dx.hypot(dy);
    if distance > RING_OUTER || distance < RING_OUTER - RING_WIDTH {
        return 0;
    }
    // y grows downward, so this angle starts at the top and turns clockwise.
    let turn = dx.atan2(-dy).rem_euclid(TAU) / TAU;
    if turn < progress { FULL } else { TRACK }
}

/// How opaque each pixel of the tray icon is, row by row. `progress` is `None` while nothing
/// transfers, which draws the plain mark.
pub fn alpha(progress: Option<f32>) -> Vec<u8> {
    let float = |value: u32| f32::from(u16::try_from(value).unwrap_or(0));
    let half = float(ICON_SIZE) / 2.0;
    // How many mark units the icon spans. The plain mark keeps the same sums it always had.
    let span = match progress {
        None => MARK_SPAN,
        Some(_) => MARK_REACH * float(ICON_SIZE) / (RING_OUTER - RING_WIDTH - RING_GAP),
    };
    let unit = span / float(ICON_SIZE * SAMPLES);
    let origin = (MARK_CENTER.0 - span / 2.0, MARK_CENTER.1 - span / 2.0);
    let mut pixels = Vec::with_capacity((ICON_SIZE * ICON_SIZE) as usize);
    for row in 0..ICON_SIZE {
        for column in 0..ICON_SIZE {
            let weight: u32 = (0..SAMPLES * SAMPLES)
                .map(|sample| {
                    let x = float(column * SAMPLES + sample % SAMPLES) + 0.5;
                    let y = float(row * SAMPLES + sample / SAMPLES) + 0.5;
                    if covered(origin.0 + x * unit, origin.1 + y * unit) {
                        return FULL;
                    }
                    let (dx, dy) = (x / float(SAMPLES) - half, y / float(SAMPLES) - half);
                    progress.map_or(0, |progress| ring(dx, dy, progress))
                })
                .sum();
            let alpha = weight * 255 / (FULL * SAMPLES * SAMPLES);
            pixels.push(u8::try_from(alpha).unwrap_or(u8::MAX));
        }
    }
    pixels
}
