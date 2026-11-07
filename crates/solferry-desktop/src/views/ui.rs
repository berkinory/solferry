use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyElement, App, Div, FontWeight, IntoElement, ParentElement, SharedString, Styled, div, px,
};

use crate::theme::{color, palette};
use crate::tokens::{
    FONT_MONO, RADIUS_CARD, RADIUS_CONTROL, RADIUS_WELL, SPACE_MD, SPACE_SM, SPACE_XS, TYPE_BODY,
    TYPE_CAPTION, TYPE_LABEL,
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
}

pub fn well(cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_XS))
        .p(px(SPACE_SM))
        .rounded(px(RADIUS_WELL))
        .bg(color(palette(cx).well))
}

pub fn card(title: impl Into<SharedString>, aside: Option<AnyElement>, cx: &App) -> Div {
    frame(cx).child(
        div()
            .flex()
            .items_center()
            .justify_between()
            .h(px(32.))
            .pl(px(SPACE_MD))
            .pr(px(SPACE_SM))
            .text_size(px(TYPE_LABEL))
            .font_weight(FontWeight::MEDIUM)
            .text_color(cx.theme().foreground.opacity(0.8))
            .child(title.into())
            .children(aside),
    )
}

pub fn count(value: usize, cx: &App) -> AnyElement {
    mono(value.to_string(), cx).into_any_element()
}

/// A card that wants a decision or reports a failure, marked by its frame.
pub fn notice(line: u32, cx: &App) -> Div {
    frame(cx).border_color(color(line))
}

pub fn notice_well(cx: &App) -> Div {
    well(cx).gap(px(SPACE_SM)).p(px(SPACE_MD))
}

pub fn failure(title: impl Into<SharedString>, next: impl Into<SharedString>, cx: &App) -> Div {
    let palette = palette(cx);
    notice(palette.danger, cx).child(
        notice_well(cx)
            .gap(px(2.))
            .bg(color(palette.danger_well))
            .child(text(title))
            .child(muted(next, cx)),
    )
}

pub fn text(value: impl Into<SharedString>) -> Div {
    div().text_size(px(TYPE_BODY)).child(value.into())
}

pub fn muted(value: impl Into<SharedString>, cx: &App) -> Div {
    div().text_size(px(TYPE_LABEL)).text_color(cx.theme().muted_foreground).child(value.into())
}

pub fn mono(value: impl Into<SharedString>, cx: &App) -> Div {
    div()
        .font_family(FONT_MONO)
        .text_size(px(TYPE_CAPTION))
        .text_color(cx.theme().muted_foreground)
        .child(value.into())
}

pub fn tile(cx: &App) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(32.))
        .rounded(px(RADIUS_CONTROL))
        .bg(cx.theme().secondary)
        .text_color(cx.theme().muted_foreground)
        .text_size(px(TYPE_LABEL))
        .font_weight(FontWeight::MEDIUM)
}

pub fn initial(name: &str, cx: &App) -> Div {
    let letter: String =
        name.chars().next().map(|c| c.to_uppercase().collect()).unwrap_or_default();
    tile(cx).child(letter)
}

pub fn dot(value: gpui_kit::Hsla) -> Div {
    div().flex_none().size(px(6.)).rounded_full().bg(value)
}

/// Muted text after a colored dot, for anything that has a state.
pub fn status(value: gpui_kit::Hsla, label: impl Into<SharedString>, cx: &App) -> Div {
    div().flex().items_center().gap(px(6.)).min_w_0().child(dot(value)).child(muted(label, cx))
}
