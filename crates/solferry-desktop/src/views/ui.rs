use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyElement, App, BoxShadow, Div, FontWeight, Hsla, IntoElement, ParentElement, SharedString,
    Styled, black, div, point, px,
};
use iroh::EndpointId;
use solferry_core::Platform;

use crate::assets::{Glyph, icon};
use crate::theme::{color, palette};
use crate::tokens::{
    RADIUS_CARD, RADIUS_FLOOR, RADIUS_WELL, SPACE_LG, SPACE_SM, SPACE_XS, TYPE_BODY, TYPE_CAPTION,
    TYPE_LABEL,
};

/// One line and 3px of padding, so what it holds sits exactly 4px inside it.
pub fn frame(cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .flex_none()
        .p(px(3.))
        .rounded(px(RADIUS_CARD))
        .border_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .shadow(vec![BoxShadow {
            color: black().opacity(0.06),
            offset: point(px(0.), px(1.)),
            blur_radius: px(2.),
            spread_radius: px(0.),
            inset: false,
        }])
}

pub fn shadow(tone: Hsla, y: f32, blur: f32, spread: f32, inset: bool) -> BoxShadow {
    BoxShadow {
        color: tone,
        offset: point(px(0.), px(y)),
        blur_radius: px(blur),
        spread_radius: px(spread),
        inset,
    }
}

/// Set into the frame by shade and a soft inner shadow, with no line of its own.
pub fn well(cx: &App) -> Div {
    let (depth, rim) = if cx.theme().is_dark() {
        (black().opacity(0.45), gpui_kit::white().opacity(0.03))
    } else {
        (black().opacity(0.05), black().opacity(0.035))
    };
    div()
        .shadow(vec![shadow(depth, 1., 2., 0., true), shadow(rim, 0., 0., 1., true)])
        .flex()
        .flex_col()
        .gap(px(SPACE_XS))
        .p(px(SPACE_SM))
        .rounded(px(RADIUS_WELL))
        .bg(color(palette(cx).well))
}

pub fn header(glyph: Glyph, title: impl Into<SharedString>, tone: Hsla, cx: &App) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(SPACE_SM))
        .min_w_0()
        .text_size(px(TYPE_LABEL))
        .font_weight(FontWeight::MEDIUM)
        .text_color(cx.theme().foreground.opacity(0.8))
        .child(icon(glyph, 16.).text_color(tone))
        .child(title.into())
}

/// A frame with an icon and title over the well that holds its rows.
pub fn card(glyph: Glyph, title: &str, aside: Option<AnyElement>, cx: &App) -> Div {
    titled(header(glyph, title.to_owned(), cx.theme().muted_foreground, cx), aside, cx)
}

pub fn titled(heading: Div, aside: Option<AnyElement>, cx: &App) -> Div {
    frame(cx).child(
        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(34.))
            .pl(px(SPACE_LG))
            .pr(px(SPACE_SM))
            .child(heading)
            .children(aside),
    )
}

pub fn count(value: usize, cx: &App) -> AnyElement {
    caption(value.to_string(), cx).pr(px(SPACE_SM)).into_any_element()
}

pub fn padded_well(cx: &App) -> Div {
    well(cx).gap(px(SPACE_SM)).p(px(SPACE_LG))
}

pub fn failure(title: &str, next: impl Into<SharedString>, cx: &App) -> Div {
    let palette = palette(cx);
    let danger = color(palette.danger);
    titled(header(Glyph::Alert, title.to_owned(), danger, cx), None, cx)
        .border_color(danger.opacity(0.5))
        .child(padded_well(cx).bg(color(palette.danger_well)).child(muted(next, cx)))
}

pub fn text(value: impl Into<SharedString>) -> Div {
    div().text_size(px(TYPE_BODY)).child(value.into())
}

pub fn muted(value: impl Into<SharedString>, cx: &App) -> Div {
    div().text_size(px(TYPE_LABEL)).text_color(cx.theme().muted_foreground).child(value.into())
}

pub fn caption(value: impl Into<SharedString>, cx: &App) -> Div {
    div().text_size(px(TYPE_CAPTION)).text_color(cx.theme().muted_foreground).child(value.into())
}

pub fn tile(glyph: Glyph, cx: &App) -> Div {
    tinted_tile(glyph, cx.theme().secondary, cx.theme().muted_foreground)
}

pub fn tinted_tile(glyph: Glyph, fill: Hsla, tone: Hsla) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(36.))
        .rounded(px(RADIUS_FLOOR))
        .bg(fill)
        .child(icon(glyph, 18.).text_color(tone))
}

/// A device's tile: its platform logo in the color its key gives it.
/// A device's tile: filled with its platform's color while it is there, flat gray while it isn't.
pub fn device_tile(platform: Option<Platform>, online: bool, cx: &App) -> Div {
    let glyph = platform_glyph(platform);
    let palette = palette(cx);
    let brand = match platform {
        Some(Platform::Macos | Platform::Ios) => palette.platform_apple,
        Some(Platform::Android) => palette.platform_android,
        Some(Platform::Windows) => palette.platform_windows,
        Some(Platform::Linux) => palette.platform_linux,
        None => return tinted_tile(glyph, cx.theme().secondary, cx.theme().muted_foreground),
    };
    if online {
        tinted_tile(glyph, color(brand), cx.theme().foreground)
    } else {
        tinted_tile(glyph, cx.theme().secondary, cx.theme().muted_foreground.opacity(0.6))
    }
}

/// The start of the device's key, short enough to compare by eye.
pub fn fingerprint(id: &EndpointId) -> String {
    id.fmt_short().to_string()
}

pub fn platform_glyph(platform: Option<Platform>) -> Glyph {
    match platform {
        Some(Platform::Macos | Platform::Ios) => Glyph::Apple,
        Some(Platform::Windows) => Glyph::Windows,
        Some(Platform::Linux) => Glyph::Linux,
        Some(Platform::Android) => Glyph::Android,
        None => Glyph::Computer,
    }
}

pub fn dot(value: Hsla) -> Div {
    div().flex_none().size(px(6.)).rounded_full().bg(value)
}

/// Muted text after a colored dot, for anything that has a state.
pub fn status(value: Hsla, label: impl Into<SharedString>, cx: &App) -> Div {
    div().flex().items_center().gap(px(6.)).min_w_0().child(dot(value)).child(muted(label, cx))
}
