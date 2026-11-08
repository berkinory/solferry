use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyElement, Context, ExternalPaths, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, transparent_black,
};

use super::button::Button;
use super::main_view::{MainView, Page};
use super::{devices, offers, texts, transfers, ui};
use crate::assets::{Glyph, icon};
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_CONTROL, SPACE_MD, SPACE_SM, SPACE_XS};
use solferry_core::explain;

pub fn toolbar(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let row = div().flex().flex_none().items_center().justify_between().h(px(40.)).px(px(SPACE_SM));
    if matches!(view.page, Page::Pair | Page::Send) {
        let title = if view.page == Page::Pair { "Pair with a code" } else { "Send" };
        return row
            .justify_start()
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
            .into_any_element();
    }
    let name = view.state.read(cx).config.as_ref().map(|config| config.name.clone());
    row.pl(px(SPACE_MD))
        .child(div().min_w_0().children(name.map(|name| ui::muted(name, cx).truncate())))
        .child(
            div()
                .flex()
                .flex_none()
                .gap(px(SPACE_XS))
                .child(tab(view, Page::Home, Glyph::Home, "Home", cx))
                .child(tab(view, Page::History, Glyph::History, "History", cx))
                .child(tab(view, Page::Settings, Glyph::Settings, "Settings", cx)),
        )
        .into_any_element()
}

fn tab(
    view: &MainView,
    page: Page,
    glyph: Glyph,
    label: &'static str,
    cx: &mut Context<MainView>,
) -> Button {
    let button = Button::new(label)
        .xs()
        .icon(glyph)
        .label(label)
        .on_click(cx.listener(move |view, _, window, cx| view.open_page(page, window, cx)));
    if view.page == page { button } else { button.ghost() }
}

fn drop_zone(cx: &mut Context<MainView>) -> AnyElement {
    let theme = cx.theme();
    let palette = palette(cx);
    let (hover, ring) = (theme.accent, color(palette.primary));
    ui::frame(cx)
        .child(
            ui::well(cx)
                .id("drop-zone")
                .flex_row()
                .items_center()
                .gap(px(SPACE_MD))
                .h(px(84.))
                .px(px(14.))
                .border_1()
                .border_color(transparent_black())
                .cursor_pointer()
                .hover(move |style| style.bg(hover))
                .group_drag_over::<ExternalPaths>("window", move |style| {
                    style.border_color(ring).bg(hover)
                })
                .on_click(cx.listener(|_, _, _, cx| MainView::pick_pending(cx)))
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .size(px(40.))
                        .rounded(px(RADIUS_CONTROL))
                        .bg(color(palette.primary_fill))
                        .border_1()
                        .border_color(color(palette.primary_border))
                        .child(icon(Glyph::Send, 18.).text_color(color(palette.on_primary))),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(ui::text("Drop files to send"))
                        .child(ui::muted("or click to choose", cx)),
                ),
        )
        .into_any_element()
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
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .children(fatal)
        .children(prompts)
        .children(code)
        .children(offers)
        .children(shared)
        .child(drop_zone(cx))
        .child(devices::render(&state, view, cx))
        .children(transfers::render(&state, cx))
        .into_any_element()
}
