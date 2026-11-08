use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyElement, Context, Div, FontWeight, IntoElement, ParentElement, SharedString, Styled, div, px,
};

use super::button::Button;
use super::dialog::Dialog;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::format;
use crate::state::{Offer, PairingPrompt};
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_FLOOR, SPACE_SM, SPACE_XS};

const DIGIT_WIDTH: f32 = 34.0;
const DIGIT_HEIGHT: f32 = 44.0;
const DIGIT_SIZE: f32 = 20.0;

/// A card asking for a decision, marked by a primary frame and icon.
fn ask(glyph: Glyph, title: &str, cx: &mut Context<MainView>) -> Div {
    let primary = color(palette(cx).primary);
    ui::titled(ui::header(glyph, title.to_owned(), primary, cx), None, cx)
        .border_color(primary.opacity(0.55))
}

fn actions() -> Div {
    div().flex().gap(px(SPACE_XS)).pt(px(SPACE_XS))
}

pub fn card(offer: &Offer, cx: &mut Context<MainView>) -> AnyElement {
    let summary = &offer.summary;
    let key = offer.key;
    let files = match summary.files {
        1 => "1 file".to_owned(),
        count => format!("{count} files"),
    };
    let warning = (!summary.paired).then(|| {
        ui::muted("Not paired. Accept only if you expected this.", cx)
            .text_color(color(palette(cx).warning))
    });
    let name = summary.peer_name.clone();
    let block = (!summary.paired).then(|| {
        Button::new(SharedString::from(format!("block-{key}")))
            .ghost()
            .sm()
            .label("Block")
            .on_click(cx.listener(move |view, _, _, cx| {
                view.dialog = Some(Dialog::Block { key, name: name.clone() });
                cx.notify();
            }))
    });
    let always = summary.paired.then(|| {
        Button::new(SharedString::from(format!("always-{key}")))
            .sm()
            .label("Always accept")
            .tooltip(format!("Stop asking for files from {}", summary.peer_name))
            .on_click(cx.listener(move |view, _, _, cx| view.accept_always(key, cx)))
    });
    ask(Glyph::Download, "Incoming files", cx)
        .child(
            ui::padded_well(cx)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(ui::device_tile(&summary.peer, summary.platform, cx))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .min_w_0()
                                .child(ui::text(format!("{} wants to send", summary.peer_name)))
                                .child(ui::caption(
                                    format!(
                                        "{files}, {}  ·  {}",
                                        format::bytes(summary.bytes),
                                        ui::fingerprint(&summary.peer)
                                    ),
                                    cx,
                                )),
                        ),
                )
                .child(ui::muted(summary.first_path.clone(), cx).truncate())
                .children(warning)
                .child(
                    actions()
                        .child(
                            Button::new(SharedString::from(format!("accept-{key}")))
                                .primary()
                                .sm()
                                .icon(Glyph::Check)
                                .label("Accept")
                                .on_click(
                                    cx.listener(move |view, _, _, cx| view.answer(key, true, cx)),
                                ),
                        )
                        .children(always)
                        .child(
                            Button::new(SharedString::from(format!("decline-{key}")))
                                .sm()
                                .label("Decline")
                                .on_click(
                                    cx.listener(move |view, _, _, cx| view.answer(key, false, cx)),
                                ),
                        )
                        .children(block),
                ),
        )
        .into_any_element()
}

fn digit(value: char, cx: &mut Context<MainView>) -> AnyElement {
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

fn code_digits(code: &str, cx: &mut Context<MainView>) -> AnyElement {
    let digits: Vec<_> =
        code.chars().filter(char::is_ascii_digit).map(|value| digit(value, cx)).collect();
    let middle = digits.len() / 2;
    let mut digits = digits.into_iter();
    let first: Vec<_> = digits.by_ref().take(middle).collect();
    let group = || div().flex().gap(px(SPACE_XS));
    div()
        .flex()
        .justify_center()
        .gap(px(SPACE_SM))
        .py(px(SPACE_XS))
        .child(group().children(first))
        .child(group().children(digits))
        .into_any_element()
}

pub fn pairing_card(prompt: &PairingPrompt, cx: &mut Context<MainView>) -> AnyElement {
    let key = prompt.key;
    let digits = code_digits(&prompt.code, cx);
    ask(Glyph::Shield, "Pairing request", cx)
        .child(
            ui::padded_well(cx)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(ui::device_tile(&prompt.peer, prompt.platform, cx))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .min_w_0()
                                .child(ui::text(format!("{} wants to pair", prompt.name)))
                                .child(ui::caption(ui::fingerprint(&prompt.peer), cx)),
                        ),
                )
                .child(digits)
                .child(ui::muted("Accept only if the other screen shows the same code.", cx))
                .child(
                    actions()
                        .child(
                            Button::new(SharedString::from(format!("pair-yes-{key}")))
                                .primary()
                                .sm()
                                .icon(Glyph::Check)
                                .label("Same code")
                                .on_click(cx.listener(move |view, _, _, cx| {
                                    view.answer_pairing(key, true, cx);
                                })),
                        )
                        .child(
                            Button::new(SharedString::from(format!("pair-no-{key}")))
                                .sm()
                                .label("Decline")
                                .on_click(cx.listener(move |view, _, _, cx| {
                                    view.answer_pairing(key, false, cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
}

pub fn pairing_code(code: &str, cx: &mut Context<MainView>) -> AnyElement {
    let digits = code_digits(code, cx);
    ask(Glyph::Shield, "Compare codes", cx)
        .child(ui::padded_well(cx).child(digits).child(ui::muted(
            "Check that the other device shows this code, then accept there.",
            cx,
        )))
        .into_any_element()
}
