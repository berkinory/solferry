use std::time::Duration;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, App, Context, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Transformation, div,
    ease_out_quint, px, radians, transparent_black,
};
use rust_i18n::t;

use super::main_view::MainView;
use super::{devices, link, offers, texts, ui};
use crate::assets::{Glyph, icon};
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_FIELD, SPACE_SM};

const FAN_WIDTH: f32 = 150.0;
const FAN_GAP: f32 = 41.0;
const FAN_OPEN: f32 = 58.0;
const FAN_SIZE: f32 = 60.0;
const FAN_TILT: f32 = 0.2;

/// A file card leaning away from the middle. Only SVGs can rotate, so the card is drawn as one.
fn fan_tile(slot: &'static str, glyph: Glyph, side: f32, open: bool, cx: &App) -> AnyElement {
    let turn = || Transformation::rotate(radians(side * FAN_TILT));
    let layer = |size: f32, inset: f32| {
        icon(Glyph::FanCard, size)
            .absolute()
            .top(px(inset))
            .left(px(inset))
            .with_transformation(turn())
    };
    let tile = div()
        .absolute()
        .top(px(4.))
        .left(px(FAN_WIDTH / 2. + side * FAN_GAP - FAN_SIZE / 2.))
        .flex()
        .items_center()
        .justify_center()
        .size(px(FAN_SIZE))
        .child(layer(FAN_SIZE, 0.).text_color(cx.theme().border))
        .child(layer(FAN_SIZE - 2., 1.).text_color(cx.theme().secondary))
        .child(
            icon(glyph, 23.).text_color(cx.theme().muted_foreground).with_transformation(turn()),
        );
    if !open {
        return tile.into_any_element();
    }
    tile.with_animation(
        SharedString::from(slot),
        Animation::new(Duration::from_millis(280)).with_easing(ease_out_quint()),
        move |tile, delta| {
            let spread = FAN_GAP + (FAN_OPEN - FAN_GAP) * delta;
            tile.left(px(FAN_WIDTH / 2. + side * spread - FAN_SIZE / 2.))
        },
    )
    .into_any_element()
}

fn drop_zone(dragging: bool, cx: &mut Context<MainView>) -> AnyElement {
    let theme = cx.theme();
    let palette = palette(cx);
    let (hover, ring) = (theme.accent, color(palette.primary));
    let core = div()
        .absolute()
        .top(px(4.))
        .left(px((FAN_WIDTH - FAN_SIZE) / 2.))
        .flex()
        .items_center()
        .justify_center()
        .size(px(FAN_SIZE))
        .rounded(px(RADIUS_FIELD))
        .bg(color(palette.primary_fill))
        .border_1()
        .border_color(color(palette.primary_border))
        .child(icon(Glyph::Send, 27.).text_color(color(palette.on_primary)));
    let fan = div()
        .relative()
        .w(px(FAN_WIDTH))
        .h(px(70.))
        .child(fan_tile("fan-left", Glyph::TypeImage, -1., dragging, cx))
        .child(fan_tile("fan-right", Glyph::TypePdf, 1., dragging, cx))
        .child(core);
    let mut well = ui::well(cx)
        .id("drop-zone")
        .items_center()
        .justify_center()
        .gap(px(SPACE_SM))
        .h(px(177.))
        .border_1()
        .border_color(transparent_black())
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(cx.listener(|_, _, _, cx| MainView::pick_pending(cx)))
        .child(fan)
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .child(
                    ui::text(if dragging { t!("home.release") } else { t!("home.drop") })
                        .font_weight(FontWeight::MEDIUM),
                )
                .child(ui::muted(
                    if dragging { t!("home.next_step") } else { t!("home.click") },
                    cx,
                )),
        );
    if dragging {
        well = well.border_color(ring).bg(hover);
    }
    ui::frame(cx).child(well).into_any_element()
}

pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let state = view.state.read(cx).clone();
    let offers: Vec<_> = state.offers.iter().map(|offer| offers::card(offer, cx)).collect();
    let shared = texts::render(&state, cx);
    let dragging = cx.has_active_drag();
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .children(offers)
        .children(shared)
        .child(drop_zone(dragging, cx))
        .child(devices::render(&state, view, cx))
        .child(link::card(cx))
        .into_any_element()
}
