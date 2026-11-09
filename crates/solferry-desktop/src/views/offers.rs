use gpui_kit::{
    AnyElement, Context, Div, IntoElement, ParentElement, SharedString, Styled, div, px,
};

use super::button::Button;
use super::dialog::Dialog;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::format;
use crate::state::Offer;
use crate::theme::{color, palette};
use crate::tokens::SPACE_XS;

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
                        .gap(px(12.))
                        .child(ui::device_tile(summary.platform, true, cx))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .min_w_0()
                                .child(
                                    ui::text(format!("{} wants to send", summary.peer_name))
                                        .truncate(),
                                )
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
