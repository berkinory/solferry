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
    let received = (!link.received.is_empty()).then(|| {
        let rows = link.received.iter().map(|name| ui::text(name.clone()).truncate().p(px(6.)));
        ui::card(Glyph::Download, &t!("link.received"), None, cx).child(ui::well(cx).children(rows))
    });
    let content = div().flex().flex_col().gap(px(SPACE_SM)).child(code).children(received).child(
        Button::new("stop-link")
            .danger()
            .label(t!("link.stop"))
            .on_click(cx.listener(|_, _, _, cx| MainView::command(Command::StopLink, cx))),
    );
    let title = if link.receiving { t!("link.receive_title") } else { t!("link.share_title") };
    super::sheet::present(title, content, |_, _, cx| MainView::command(Command::StopLink, cx), cx)
}

pub fn card(cx: &mut Context<MainView>) -> AnyElement {
    ui::card(Glyph::Link, &t!("link.no_app.title"), None, cx)
        .child(
            ui::padded_well(cx).child(ui::muted(t!("link.no_app.body"), cx)).child(
                div().pt(px(SPACE_SM)).child(
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
