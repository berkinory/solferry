use gpui_kit::component::ActiveTheme;
use gpui_kit::{AnyElement, App, Div, IntoElement, ParentElement, SharedString, Styled, div, px};

use crate::tokens::{RADIUS_MD, SPACE_MD, SPACE_SM};

pub fn section_title(text: impl Into<SharedString>, cx: &App) -> Div {
    div().text_xs().text_color(cx.theme().muted_foreground).child(text.into())
}

pub fn card(cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .p(px(SPACE_MD))
        .rounded(px(RADIUS_MD))
        .bg(cx.theme().secondary)
        .border_1()
        .border_color(cx.theme().border)
}

pub fn muted(text: impl Into<SharedString>, cx: &App) -> AnyElement {
    div().text_sm().text_color(cx.theme().muted_foreground).child(text.into()).into_any_element()
}
