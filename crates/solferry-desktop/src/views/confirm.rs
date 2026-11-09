use gpui_kit::component::ActiveTheme;
use gpui_kit::{AnyElement, Context, Div, FontWeight, IntoElement, ParentElement, Styled, div, px};

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::state::PairingPrompt;
use crate::tokens::{RADIUS_FLOOR, SPACE_MD, SPACE_SM, SPACE_XS};

const DIGIT_WIDTH: f32 = 50.0;
const DIGIT_HEIGHT: f32 = 68.0;
const DIGIT_SIZE: f32 = 30.0;

fn digit(value: char, cx: &Context<MainView>) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .w(px(DIGIT_WIDTH))
        .h(px(DIGIT_HEIGHT))
        .rounded(px(RADIUS_FLOOR))
        .bg(cx.theme().background)
        .border_1()
        .border_color(cx.theme().border)
        .text_size(px(DIGIT_SIZE))
        .font_weight(FontWeight::MEDIUM)
        .child(value.to_string())
        .into_any_element()
}

fn digits(code: &str, cx: &Context<MainView>) -> Div {
    let all: Vec<_> = code.chars().filter(char::is_ascii_digit).map(|d| digit(d, cx)).collect();
    let middle = all.len() / 2;
    let mut all = all.into_iter();
    let first: Vec<_> = all.by_ref().take(middle).collect();
    let group = || div().flex().gap(px(SPACE_XS));
    ui::frame(cx).child(
        ui::well(cx)
            .flex_row()
            .justify_center()
            .gap(px(SPACE_MD))
            .py(px(23.))
            .child(group().children(first))
            .child(group().children(all)),
    )
}

fn request(prompt: &PairingPrompt, cx: &mut Context<MainView>) -> AnyElement {
    let key = prompt.key;
    let peer = div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(SPACE_XS))
        .py(px(SPACE_XS))
        .child(ui::device_tile(prompt.platform, true, cx))
        .child(ui::text(format!("{} wants to pair", prompt.name)).truncate().max_w_full())
        .child(ui::caption(ui::fingerprint(&prompt.peer), cx));
    let answers =
        div()
            .flex()
            .justify_center()
            .gap(px(SPACE_SM))
            .pt(px(SPACE_MD))
            .child(
                Button::new("pair-accept").primary().icon(Glyph::Check).label("Confirm").on_click(
                    cx.listener(move |view, _, _, cx| view.answer_pairing(key, true, cx)),
                ),
            )
            .child(
                Button::new("pair-decline").warning().label("Decline").on_click(
                    cx.listener(move |view, _, _, cx| view.answer_pairing(key, false, cx)),
                ),
            );
    let content = div()
        .flex()
        .flex_col()
        .child(peer)
        .child(digits(&prompt.code, cx))
        .child(
            div()
                .flex()
                .justify_center()
                .pt(px(SPACE_SM))
                .child(ui::muted("Confirm only if the other screen shows the same code.", cx)),
        )
        .child(answers);
    super::sheet::present("Pairing request", content, MainView::decline_first_pairing, cx)
}

fn compare(code: &str, cx: &mut Context<MainView>) -> AnyElement {
    let content = div().flex().flex_col().pt(px(SPACE_XS)).child(digits(code, cx)).child(
        div().flex().justify_center().pt(px(SPACE_SM)).child(ui::muted(
            "Check that the other device shows this code, then accept there.",
            cx,
        )),
    );
    super::sheet::present("Compare codes", content, MainView::stop_comparing, cx)
}

/// Pairing takes over the screen the way sending does, ahead of anything else open.
pub fn sheet(view: &MainView, cx: &mut Context<MainView>) -> Option<AnyElement> {
    let state = view.state.read(cx);
    let (prompt, code) = (state.pairing_prompts.first().cloned(), state.pairing_code.clone());
    prompt.map(|prompt| request(&prompt, cx)).or_else(|| code.map(|code| compare(&code, cx)))
}
