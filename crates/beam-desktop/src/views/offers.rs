use gpui_kit::component::ActiveTheme;
use gpui_kit::component::Sizable;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::{AnyElement, Context, IntoElement, ParentElement, SharedString, Styled, div};

use super::main_view::MainView;
use super::ui;
use crate::format;
use crate::state::Offer;

pub fn card(offer: &Offer, cx: &mut Context<MainView>) -> AnyElement {
    let summary = &offer.summary;
    let key = offer.key;
    let files = match summary.files {
        1 => "1 file".to_owned(),
        count => format!("{count} files"),
    };
    let warning = summary
        .relationship
        .is_none()
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
                ),
        )
        .into_any_element()
}
