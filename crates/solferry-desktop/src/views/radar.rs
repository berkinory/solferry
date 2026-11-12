use std::f32::consts::TAU;
use std::time::Duration;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, Context, Div, FontWeight, Hsla, IntoElement,
    ParentElement, Styled, div, px,
};
use rust_i18n::t;

use super::button::Button;
use super::main_view::{MainView, Page};
use super::ui;
use crate::assets::{Glyph, icon};
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_FIELD, SPACE_XS};

const SIZE: f32 = 103.0;
const CORE: f32 = 33.0;
const ORBITS: [f32; 2] = [60.0, 95.0];
const PULSE: Duration = Duration::from_millis(2800);
const BREATH: Duration = Duration::from_millis(3600);
const TILE: f32 = 31.0;

/// The kinds of device that could turn up, resting on the orbit where they would appear.
const GHOSTS: [(&str, Glyph, f32, f32, f32); 4] = [
    ("ghost-apple", Glyph::Apple, -44.0, -27.0, 0.0),
    ("ghost-windows", Glyph::Windows, 44.0, -20.0, 0.25),
    ("ghost-android", Glyph::Android, -35.0, 38.0, 0.5),
    ("ghost-linux", Glyph::Linux, 41.0, 34.0, 0.75),
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
        .bg(cx.theme().secondary)
        .border_1()
        .border_color(cx.theme().border)
        .child(icon(Glyph::Nearby, 17.).text_color(cx.theme().muted_foreground));
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

pub fn use_code(cx: &mut Context<MainView>) -> AnyElement {
    Button::new("use-code")
        .ghost()
        .xs()
        .icon(Glyph::Qr)
        .label(t!("radar.use_code"))
        .on_click(cx.listener(|view, _, window, cx| view.open_page(Page::Pair, window, cx)))
        .into_any_element()
}

/// What the nearby card holds for someone with no devices yet.
pub fn searching(cx: &mut Context<MainView>) -> Div {
    ui::well(cx)
        .items_center()
        .gap(px(SPACE_XS))
        .py(px(10.))
        .child(radar(cx))
        .child(ui::text(t!("radar.looking")).pt(px(SPACE_XS)).font_weight(FontWeight::MEDIUM))
        .child(ui::muted(t!("radar.hint"), cx).text_center().max_w(px(275.)))
}

/// One quiet line for when nothing new is around but the search is still on.
pub fn scanning(cx: &Context<MainView>) -> AnyElement {
    let primary = color(palette(cx).primary);
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .p(px(6.))
        .child(
            div()
                .relative()
                .flex_none()
                .size(px(TILE))
                .child(pulse("scan-pulse", 0.0, TILE, 8.0, primary))
                .child(centered(TILE, 7.0).bg(primary)),
        )
        .child(ui::muted(t!("radar.scanning"), cx))
        .into_any_element()
}
