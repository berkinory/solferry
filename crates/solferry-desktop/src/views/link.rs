use gpui_kit::{AnyElement, ClipboardItem, Context, IntoElement, ParentElement, Styled, div, px};
use rust_i18n::t;

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::qr;
use crate::service::Command;
use crate::state::LinkView;
use crate::tokens::SPACE_SM;

const PREVIEW_LINES: usize = 3;

/// A link for a browser on the same network. Closing the sheet stops it.
pub fn sheet(view: &MainView, link: &LinkView, cx: &mut Context<MainView>) -> AnyElement {
    let url = link.urls.first().cloned().unwrap_or_default();
    let copied = url.clone();
    let (glyph, label) = if view.copied {
        (Glyph::Check, t!("link.copied"))
    } else {
        (Glyph::Copy, t!("link.copy"))
    };
    let hint = if link.receiving { t!("link.receive_hint") } else { t!("link.share_hint") };
    let code = div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(SPACE_SM))
        .children(qr::code(&url))
        .child(ui::text(url.clone()))
        .child(ui::muted(hint, cx).text_center())
        .child(Button::new("copy-link").sm().icon(glyph).label(label).on_click(cx.listener(
            move |view, _, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                view.show_copied(cx);
            },
        )));
    let received = (!link.received.is_empty() || !link.texts.is_empty()).then(|| {
        let texts: Vec<_> =
            link.texts.iter().enumerate().map(|(index, text)| text_row(index, text, cx)).collect();
        let files = link.received.iter().map(|name| ui::text(name.clone()).truncate().p(px(5.)));
        ui::card(Glyph::Download, &t!("link.received"), None, cx)
            .child(ui::well(cx).children(texts).children(files))
    });
    let stop = Button::new("stop-link").danger().label(t!("link.stop")).on_click(cx.listener(
        |view, _, window, cx| {
            view.dismiss(|_, _, cx| MainView::command(Command::StopLink, cx), window, cx);
        },
    ));
    let content = div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .child(code)
        .children(received)
        .child(div().flex().justify_center().pt(px(SPACE_SM)).child(stop));
    let title = if link.receiving { t!("link.receive_title") } else { t!("link.share_title") };
    super::sheet::present(
        view,
        title,
        content,
        |_, _, cx| MainView::command(Command::StopLink, cx),
        cx,
    )
}

/// Text a browser sent, a few lines of it, with a way to copy all of it.
fn text_row(index: usize, text: &str, cx: &mut Context<MainView>) -> AnyElement {
    let copied = text.to_owned();
    div()
        .flex()
        .items_start()
        .gap(px(SPACE_SM))
        .p(px(5.))
        .child(ui::text(text.to_owned()).line_clamp(PREVIEW_LINES).flex_1().min_w_0())
        .child(
            Button::new(("copy-link-text", index))
                .ghost()
                .xs()
                .icon(Glyph::Copy)
                .tooltip(t!("texts.copy"))
                .on_click(cx.listener(move |_, _, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                })),
        )
        .into_any_element()
}

pub fn card(cx: &mut Context<MainView>) -> AnyElement {
    ui::card(Glyph::Link, &t!("link.no_app.title"), None, cx)
        .child(
            ui::padded_well(cx).child(ui::muted(t!("link.no_app.body"), cx)).child(
                div().flex().pt(px(SPACE_SM)).child(
                    Button::new("receive-link")
                        .sm()
                        .icon(Glyph::Download)
                        .label(t!("link.no_app.receive"))
                        .on_click(cx.listener(|_, _, _, cx| {
                            MainView::command(Command::Link(Vec::new()), cx);
                        })),
                ),
            ),
        )
        .into_any_element()
}
