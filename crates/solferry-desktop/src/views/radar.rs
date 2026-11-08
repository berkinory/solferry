use std::f32::consts::TAU;
use std::time::Duration;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, Context, Div, FontWeight, Hsla, IntoElement,
    ParentElement, Styled, div, px, white,
};

use super::button::Button;
use super::main_view::{MainView, Page};
use super::ui;
use crate::assets::{Glyph, icon};
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_FIELD, SPACE_SM, SPACE_XS};

const SIZE: f32 = 176.0;
const CORE: f32 = 56.0;
const ORBITS: [f32; 2] = [108.0, 172.0];
const PULSE: Duration = Duration::from_millis(2800);
const BREATH: Duration = Duration::from_millis(3600);
const TILE: f32 = 32.0;

/// The kinds of device that could turn up, resting on the orbit where they would appear.
const GHOSTS: [(&str, Glyph, f32, f32, f32); 4] = [
    ("ghost-apple", Glyph::Apple, -66.0, -42.0, 0.0),
    ("ghost-windows", Glyph::Windows, 68.0, -30.0, 0.25),
    ("ghost-android", Glyph::Android, -54.0, 58.0, 0.5),
    ("ghost-linux", Glyph::Linux, 62.0, 52.0, 0.75),
];

fn centered(area: f32, size: f32) -> Div {
    let inset = (area - size) / 2.0;
    div().absolute().left(px(inset)).top(px(inset)).size(px(size)).rounded_full()
}

/// A ring leaving the center and fading on its way out.
fn pulse(id: &'static str, phase: f32, area: f32, from: f32, tone: Hsla) -> impl IntoElement {
    centered(area, from).border_1().border_color(tone).with_animation(
        id,
        Animation::new(PULSE).repeat(),
        move |ring, delta| {
            let progress = (delta + phase).fract();
            let size = from + (area - from) * progress;
            let inset = (area - size) / 2.0;
            ring.left(px(inset)).top(px(inset)).size(px(size)).opacity(0.7 * (1.0 - progress))
        },
    )
}

fn ghost(entry: (&'static str, Glyph, f32, f32, f32), cx: &Context<MainView>) -> impl IntoElement {
    let (id, glyph, x, y, phase) = entry;
    let theme = cx.theme();
    div()
        .absolute()
        .left(px((SIZE - TILE) / 2.0 + x))
        .top(px((SIZE - TILE) / 2.0 + y))
        .child(ui::tinted_tile(glyph, theme.secondary, theme.muted_foreground))
        .with_animation(id, Animation::new(BREATH).repeat(), move |tile, delta| {
            tile.opacity(0.75f32.mul_add((TAU * (delta + phase)).sin() * 0.5 + 0.5, 0.25))
        })
}

fn radar(cx: &Context<MainView>) -> Div {
    let palette = palette(cx);
    let primary = color(palette.primary);
    let core = centered(SIZE, CORE)
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(RADIUS_FIELD))
        .bg(color(palette.primary_fill))
        .border_1()
        .border_color(color(palette.primary_border))
        .shadow(vec![ui::shadow(white().opacity(0.22), 1., 0., 0., true)])
        .child(icon(Glyph::Nearby, 24.).text_color(color(palette.on_primary)));
    let line = cx.theme().border;
    div()
        .relative()
        .size(px(SIZE))
        .children(ORBITS.map(|orbit| centered(SIZE, orbit).border_1().border_color(line)))
        .child(pulse("pulse-first", 0.0, SIZE, CORE, primary))
        .child(pulse("pulse-second", 0.5, SIZE, CORE, primary))
        .children(GHOSTS.map(|entry| ghost(entry, cx)))
        .child(core)
}

pub fn use_code(label: &'static str, cx: &mut Context<MainView>) -> AnyElement {
    Button::new("use-code")
        .ghost()
        .xs()
        .icon(Glyph::Qr)
        .label(label)
        .on_click(cx.listener(|view, _, window, cx| view.open_page(Page::Pair, window, cx)))
        .into_any_element()
}

/// The whole home screen for someone with no devices yet.
pub fn hero(cx: &mut Context<MainView>) -> AnyElement {
    let code = Button::new("pair-with-code")
        .sm()
        .icon(Glyph::Qr)
        .label("Pair with code")
        .on_click(cx.listener(|view, _, window, cx| view.open_page(Page::Pair, window, cx)));
    ui::frame(cx)
        .child(
            ui::well(cx)
                .items_center()
                .gap(px(SPACE_SM))
                .py(px(24.))
                .child(radar(cx))
                .child(
                    ui::text("Looking for nearby devices")
                        .pt(px(SPACE_SM))
                        .font_weight(FontWeight::MEDIUM),
                )
                .child(
                    ui::muted(
                        "Open Solferry on another device on this network and it shows up here.",
                        cx,
                    )
                    .text_center()
                    .max_w(px(300.)),
                )
                .child(div().pt(px(SPACE_XS)).child(code)),
        )
        .into_any_element()
}

/// One quiet line for when nothing new is around but the search is still on.
pub fn scanning(cx: &Context<MainView>) -> AnyElement {
    let primary = color(palette(cx).primary);
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .p(px(7.))
        .child(
            div()
                .relative()
                .flex_none()
                .size(px(TILE))
                .child(pulse("scan-pulse", 0.0, TILE, 8.0, primary))
                .child(centered(TILE, 8.0).bg(primary)),
        )
        .child(ui::muted("Looking for new devices on this network", cx))
        .into_any_element()
}
