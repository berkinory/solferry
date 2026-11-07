use gpui_kit::component::ActiveTheme;
use gpui_kit::{AnyElement, App, Div, IntoElement, ParentElement, SharedString, Styled, div, px};

pub fn section_title(text: impl Into<SharedString>, cx: &App) -> Div {
    div().text_xs().text_color(cx.theme().muted_foreground).child(text.into())
}

pub fn card(cx: &App) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .p_3()
        .rounded(px(10.))
        .bg(cx.theme().secondary)
        .border_1()
        .border_color(cx.theme().border)
}

pub fn muted(text: impl Into<SharedString>, cx: &App) -> AnyElement {
    div().text_sm().text_color(cx.theme().muted_foreground).child(text.into()).into_any_element()
}
