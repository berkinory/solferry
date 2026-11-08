use std::time::Duration;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, App, Context, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, div, ease_out_quint, px,
    transparent_black,
};

use super::button::Button;
use super::main_view::{MainView, Page};
use super::{devices, offers, texts, transfers, ui};
use crate::assets::{Glyph, icon};
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_CONTROL, RADIUS_FIELD, SPACE_SM, SPACE_XS};
use solferry_core::explain;

/// The back button and title of a page that was opened from somewhere else.
pub fn toolbar(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let title = if view.page == Page::Pair { "Pair with a code" } else { "Send" };
    div()
        .flex()
        .flex_none()
        .items_center()
        .h(px(40.))
        .px(px(SPACE_SM))
        .gap(px(SPACE_XS))
        .child(
            Button::new("go-home")
                .ghost()
                .xs()
                .icon(Glyph::ArrowLeft)
                .tooltip("Back")
                .on_click(cx.listener(|view, _, window, cx| view.leave_page(window, cx))),
        )
        .child(ui::text(title))
        .into_any_element()
}

const FAN_GAP: f32 = 46.0;
const FAN_OPEN: f32 = 62.0;

fn fan_tile(slot: &'static str, glyph: Glyph, side: f32, open: bool, cx: &App) -> AnyElement {
    let tile = div()
        .absolute()
        .top(px(10.))
        .left(px(70. + side * FAN_GAP - 22.))
        .flex()
        .items_center()
        .justify_center()
        .size(px(44.))
        .rounded(px(RADIUS_CONTROL))
        .bg(cx.theme().secondary)
        .border_1()
        .border_color(cx.theme().border)
        .child(icon(glyph, 20.).text_color(cx.theme().muted_foreground));
    if !open {
        return tile.into_any_element();
    }
    tile.with_animation(
        SharedString::from(slot),
        Animation::new(Duration::from_millis(280)).with_easing(ease_out_quint()),
        move |tile, delta| {
            let spread = FAN_GAP + (FAN_OPEN - FAN_GAP) * delta;
            tile.left(px(70. + side * spread - 22.)).top(px(10. - 6. * delta))
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
        .left(px(42.))
        .flex()
        .items_center()
        .justify_center()
        .size(px(56.))
        .rounded(px(RADIUS_FIELD))
        .bg(color(palette.primary_fill))
        .border_1()
        .border_color(color(palette.primary_border))
        .child(icon(Glyph::Send, 24.).text_color(color(palette.on_primary)));
    let fan = div()
        .relative()
        .w(px(140.))
        .h(px(64.))
        .child(fan_tile("fan-left", Glyph::TypeImage, -1., dragging, cx))
        .child(fan_tile("fan-right", Glyph::TypePdf, 1., dragging, cx))
        .child(core);
    let mut well = ui::well(cx)
        .id("drop-zone")
        .items_center()
        .justify_center()
        .gap(px(SPACE_SM))
        .h(px(172.))
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
                    ui::text(if dragging { "Release to send" } else { "Drop files to send" })
                        .font_weight(FontWeight::MEDIUM),
                )
                .child(ui::muted(
                    if dragging { "They go to the next step" } else { "or click to choose" },
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
    let fatal = state.fatal.as_ref().map(|error| {
        let explanation = explain(error);
        ui::failure(explanation.title, explanation.next, cx)
    });
    let offers: Vec<_> = state.offers.iter().map(|offer| offers::card(offer, cx)).collect();
    let prompts: Vec<_> =
        state.pairing_prompts.iter().map(|prompt| offers::pairing_card(prompt, cx)).collect();
    let code = state.pairing_code.as_deref().map(|code| offers::pairing_code(code, cx));
    let shared = texts::render(&state, cx);
    let dragging = cx.has_active_drag();
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .children(fatal)
        .children(prompts)
        .children(code)
        .children(offers)
        .children(shared)
        .child(drop_zone(dragging, cx))
        .child(devices::render(&state, view, cx))
        .children(transfers::render(&state, cx))
        .into_any_element()
}
