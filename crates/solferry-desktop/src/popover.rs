//! The quick send popover a left click on the tray icon opens: the devices, and a file picker.

use std::time::{Duration, Instant};

use gpui_kit::{
    App, AppContext, Bounds, Pixels, Point, Size, WindowBackgroundAppearance, WindowBounds,
    WindowKind, WindowOptions, point, px, size,
};

use crate::shell::Shell;
use crate::tray::Anchor;
use crate::views::{Popover, popover};

/// Room left between the popover and the icon or the edge of the screen.
#[cfg(any(target_os = "linux", target_os = "macos"))]
const GAP: f32 = 6.0;
const EDGE: f32 = 8.0;
/// Clicking the icon of an open popover closes it as it loses focus, before the click arrives.
/// That click must not open it again.
const REOPEN_AFTER: Duration = Duration::from_millis(400);

#[derive(Default)]
pub struct Slot {
    window: Option<gpui_kit::AnyWindowHandle>,
    closed_at: Option<Instant>,
}

/// Closes the popover when it shows, opens it under the icon otherwise.
pub fn toggle(anchor: Anchor, cx: &mut App) {
    if let Some(window) = cx.global_mut::<Shell>().popover.window.take()
        && window.update(cx, |_, window, _| window.remove_window()).is_ok()
    {
        return;
    }
    let closed_at = cx.global::<Shell>().popover.closed_at;
    if closed_at.is_some_and(|at| at.elapsed() < REOPEN_AFTER) {
        return;
    }
    let devices = popover::device_count(cx);
    let card = size(px(popover::WIDTH), px(popover::height(devices)));
    // The window reaches past the card by the room its shadow needs.
    let margin = px(popover::MARGIN);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds {
            origin: origin(anchor, card, cx) - point(margin, margin),
            size: size(card.width + margin * 2., card.height + margin * 2.),
        })),
        titlebar: None,
        focus: true,
        show: true,
        kind: WindowKind::PopUp,
        is_movable: false,
        is_resizable: false,
        is_minimizable: false,
        window_background: if cfg!(target_os = "macos") {
            WindowBackgroundAppearance::Transparent
        } else {
            WindowBackgroundAppearance::Opaque
        },
        app_id: Some("solferry".to_owned()),
        ..WindowOptions::default()
    };
    match gpui_kit::open_window(options, cx, |window, cx| cx.new(|cx| Popover::new(window, cx))) {
        Ok((window, _)) => cx.global_mut::<Shell>().popover.window = Some(window),
        Err(error) => tracing::warn!(%error, "couldn't open the popover"),
    }
    // A popover of an app in the background would never hear that focus moved on.
    cx.activate(true);
}

/// Notes that the popover closed on its own, as it lost focus.
pub fn closed(cx: &mut App) {
    let slot = &mut cx.global_mut::<Shell>().popover;
    slot.window = None;
    slot.closed_at = Some(Instant::now());
}

/// The popover's top left corner, in points from the top left of the main display.
fn origin(anchor: Anchor, size: Size<Pixels>, cx: &App) -> Point<Pixels> {
    #[cfg(target_os = "macos")]
    if let Some(origin) = under_icon(anchor, size) {
        return origin;
    }
    #[cfg(target_os = "linux")]
    if let Anchor::Point { x, y } = anchor
        && (x, y) != (0, 0)
    {
        return near_point(x, y, size, cx);
    }
    let _ = anchor;
    // Windows opens its own tray flyouts above the taskbar's right end, whichever icon was hit.
    corner(cfg!(windows), size, cx)
}

fn visible(cx: &App) -> Bounds<Pixels> {
    cx.primary_display().map_or_else(
        || Bounds { origin: point(px(0.), px(0.)), size: size(px(1280.), px(800.)) },
        |display| display.visible_bounds(),
    )
}

/// The top or bottom right corner of the main display, inside its bars.
fn corner(bottom: bool, size: Size<Pixels>, cx: &App) -> Point<Pixels> {
    let area = visible(cx);
    let x = area.right() - size.width - px(EDGE);
    let y = if bottom { area.bottom() - size.height - px(EDGE) } else { area.top() + px(EDGE) };
    point(x, y)
}

/// Below a click in the top half of the screen, above one in the bottom half.
#[cfg(target_os = "linux")]
#[expect(clippy::cast_precision_loss, reason = "screen coordinates fit in f32")]
fn near_point(x: i32, y: i32, size: Size<Pixels>, cx: &App) -> Point<Pixels> {
    let area = visible(cx);
    let (x, y) = (px(x as f32), px(y as f32));
    let top = if y > area.center().y { y - size.height - px(GAP) } else { y + px(GAP) };
    point(
        clamp(x - size.width / 2., area.left(), area.right() - size.width),
        clamp(top, area.top(), area.bottom() - size.height),
    )
}

#[cfg(target_os = "linux")]
fn clamp(value: Pixels, low: Pixels, high: Pixels) -> Pixels {
    value.max(low + px(EDGE)).min(high - px(EDGE))
}

/// Centered under the menu bar icon. The icon comes in the pixels of its screen, so the screen
/// whose scale puts it inside that screen is the one it sits on.
#[cfg(target_os = "macos")]
#[expect(clippy::cast_possible_truncation, reason = "screen coordinates fit in f32")]
fn under_icon(anchor: Anchor, size: Size<Pixels>) -> Option<Point<Pixels>> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::NSScreen;

    let Anchor::Icon { x, y, width, height } = anchor else { return None };
    let screens = NSScreen::screens(MainThreadMarker::new()?);
    let main_height = screens.firstObject()?.frame().size.height;
    let width_points = f64::from(f32::from(size.width));
    screens.iter().find_map(|screen| {
        let (frame, scale) = (screen.frame(), screen.backingScaleFactor());
        let top = main_height - frame.origin.y - frame.size.height;
        let (left, right) = (frame.origin.x, frame.origin.x + frame.size.width);
        let (middle, below) = ((x + width / 2.) / scale, (y + height) / scale);
        let inside =
            (left..right).contains(&middle) && (top..top + frame.size.height).contains(&below);
        inside.then(|| {
            let left = (middle - width_points / 2.)
                .clamp(left + f64::from(EDGE), right - f64::from(EDGE) - width_points);
            point(px(left as f32), px((below + f64::from(GAP)) as f32))
        })
    })
}

/// AppKit draws a frame of its own around a window without a title bar, with corners that don't
/// match the card. Without it the card is the only shape on screen.
#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
pub fn borderless(window: &gpui_kit::Window) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    const NONACTIVATING_PANEL: usize = 1 << 7;
    let Ok(handle) = HasWindowHandle::window_handle(window) else { return };
    let RawWindowHandle::AppKit(handle) = handle.as_raw() else { return };
    let view = handle.ns_view.as_ptr().cast::<AnyObject>();
    // SAFETY: the view is the window's live content view, on the main thread that owns it.
    unsafe {
        let native: *mut AnyObject = msg_send![view, window];
        if native.is_null() {
            return;
        }
        let _: () = msg_send![native, setStyleMask: NONACTIVATING_PANEL];
        let _: () = msg_send![native, setHasShadow: false];
        // A new style mask takes key status away, and keys would stop reaching the popover.
        let _: () = msg_send![native, makeKeyAndOrderFront: std::ptr::null_mut::<AnyObject>()];
        let _: bool = msg_send![native, makeFirstResponder: view];
    }
}
