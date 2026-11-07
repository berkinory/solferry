use gpui_kit::component::ActiveTheme;
use gpui_kit::component::Sizable;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::{AnyElement, Context, IntoElement, ParentElement, SharedString, Styled, div};

use super::main_view::MainView;
use super::ui;
use crate::format;
use crate::state::{Offer, PairingPrompt};

pub fn card(offer: &Offer, cx: &mut Context<MainView>) -> AnyElement {
    let summary = &offer.summary;
    let key = offer.key;
    let files = match summary.files {
        1 => "1 file".to_owned(),
        count => format!("{count} files"),
    };
    let warning = (!summary.paired)
        .then(|| ui::muted("This device isn't paired. Accept only if you expected this.", cx));
    ui::card(cx)
        .border_color(cx.theme().primary)
        .child(div().text_sm().child(format!("{} wants to send you", summary.peer_name)))
        .child(div().text_sm().child(format!(
            "{}, {}: {}",
            files,
            format::bytes(summary.bytes),
            summary.first_path
        )))
        .children(warning)
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new(SharedString::from(format!("accept-{key}")))
                        .primary()
                        .small()
                        .label("Accept")
                        .on_click(cx.listener(move |view, _, _, cx| view.answer(key, true, cx))),
                )
                .child(
                    Button::new(SharedString::from(format!("decline-{key}")))
                        .ghost()
                        .small()
                        .label("Decline")
                        .on_click(cx.listener(move |view, _, _, cx| view.answer(key, false, cx))),
                )
                .child(
                    Button::new(SharedString::from(format!("block-{key}")))
                        .ghost()
                        .small()
                        .label("Block")
                        .on_click(cx.listener(move |view, _, _, cx| view.block_offer(key, cx))),
                ),
        )
        .into_any_element()
}

pub fn pairing_card(prompt: &PairingPrompt, cx: &mut Context<MainView>) -> AnyElement {
    let key = prompt.key;
    ui::card(cx)
        .border_color(cx.theme().primary)
        .child(div().text_sm().child(format!("{} wants to pair", prompt.name)))
        .child(div().text_xl().child(prompt.code.clone()))
        .child(ui::muted("Accept only if the other screen shows the same code.", cx))
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new(SharedString::from(format!("pair-yes-{key}")))
                        .primary()
                        .small()
                        .label("Same code")
                        .on_click(
                            cx.listener(move |view, _, _, cx| view.answer_pairing(key, true, cx)),
                        ),
                )
                .child(
                    Button::new(SharedString::from(format!("pair-no-{key}")))
                        .ghost()
                        .small()
                        .label("Decline")
                        .on_click(
                            cx.listener(move |view, _, _, cx| view.answer_pairing(key, false, cx)),
                        ),
                ),
        )
        .into_any_element()
}

pub fn pairing_code(code: &str, cx: &mut Context<MainView>) -> AnyElement {
    ui::card(cx)
        .border_color(cx.theme().primary)
        .child(div().text_xl().child(code.to_owned()))
        .child(ui::muted("Check that the other device shows this code, then accept there.", cx))
        .into_any_element()
}
