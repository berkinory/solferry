use gpui_kit::component::ActiveTheme;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::{
    AnyElement, Context, FontWeight, IntoElement, ParentElement, SharedString, Styled, div, px,
};

use super::main_view::MainView;
use super::ui;
use crate::format;
use crate::state::{Offer, PairingPrompt};
use crate::theme::{color, palette};
use crate::tokens::{FONT_MONO, RADIUS_FLOOR, SPACE_SM, SPACE_XS};

const DIGIT_WIDTH: f32 = 34.0;
const DIGIT_HEIGHT: f32 = 44.0;
const DIGIT_SIZE: f32 = 20.0;

fn sun_card(cx: &mut Context<MainView>) -> gpui_kit::Div {
    ui::notice(palette(cx).sun, cx)
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
    let block = (!summary.paired).then(|| {
        Button::new(SharedString::from(format!("block-{key}")))
            .ghost()
            .label("Block")
            .on_click(cx.listener(move |view, _, _, cx| view.block_offer(key, cx)))
    });
    sun_card(cx)
        .child(
            ui::notice_well(cx)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(ui::initial(&summary.peer_name, cx))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .min_w_0()
                                .child(ui::text(format!("{} wants to send", summary.peer_name)))
                                .child(ui::mono(
                                    format!("{files}, {}", format::bytes(summary.bytes)),
                                    cx,
                                )),
                        ),
                )
                .child(ui::muted(summary.first_path.clone(), cx).truncate())
                .children(warning)
                .child(
                    div()
                        .flex()
                        .gap(px(SPACE_XS))
                        .child(
                            Button::new(SharedString::from(format!("accept-{key}")))
                                .primary()
                                .label("Accept")
                                .on_click(
                                    cx.listener(move |view, _, _, cx| view.answer(key, true, cx)),
                                ),
                        )
                        .child(
                            Button::new(SharedString::from(format!("decline-{key}")))
                                .ghost()
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
        .bg(cx.theme().secondary)
        .font_family(FONT_MONO)
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
    sun_card(cx)
        .child(
            ui::notice_well(cx)
                .child(ui::text(format!("{} wants to pair", prompt.name)))
                .child(digits)
                .child(ui::muted("Accept only if the other screen shows the same code.", cx))
                .child(
                    div()
                        .flex()
                        .gap(px(SPACE_XS))
                        .child(
                            Button::new(SharedString::from(format!("pair-yes-{key}")))
                                .primary()
                                .label("Same code")
                                .on_click(cx.listener(move |view, _, _, cx| {
                                    view.answer_pairing(key, true, cx);
                                })),
                        )
                        .child(
                            Button::new(SharedString::from(format!("pair-no-{key}")))
                                .ghost()
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
    sun_card(cx)
        .child(
            ui::notice_well(cx)
                .child(ui::text("Check this code on the other device"))
                .child(digits)
                .child(ui::muted("Then accept there.", cx)),
        )
        .into_any_element()
}
