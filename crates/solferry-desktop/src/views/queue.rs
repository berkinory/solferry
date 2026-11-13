use gpui_kit::{AnyElement, Context, IntoElement, ParentElement, SharedString, Styled, div, px};
use rust_i18n::t;
use solferry_core::{QUEUE_DAYS, QueuedSend};

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::{Glyph, file_glyph};
use crate::service::Command;
use crate::state::AppState;
use crate::tokens::SPACE_SM;

/// Sends waiting for their devices, each with a way to take it back.
pub fn card(state: &AppState, cx: &mut Context<MainView>) -> Option<AnyElement> {
    if state.queued.is_empty() {
        return None;
    }
    let rows: Vec<_> = state.queued.iter().map(|entry| row(entry, cx)).collect();
    let count = Some(ui::count(state.queued.len(), cx));
    let card = ui::card(Glyph::Send, &t!("queue.title"), count, cx).child(
        ui::well(cx)
            .children(rows)
            .child(ui::caption(t!("queue.hint", days = QUEUE_DAYS), cx).px(px(6.)).pb(px(2.))),
    );
    Some(card.into_any_element())
}

fn row(entry: &QueuedSend, cx: &mut Context<MainView>) -> AnyElement {
    let id = entry.id;
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .p(px(6.))
        .child(ui::tile(file_glyph(&entry.label), cx))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .child(ui::text(entry.label.clone()).truncate())
                .child(ui::caption(t!("queue.to", name = entry.peer_name), cx).truncate()),
        )
        .child(
            Button::new(SharedString::from(format!("unqueue-{id}")))
                .ghost()
                .xs()
                .icon(Glyph::Cancel)
                .tooltip(t!("queue.cancel"))
                .on_click(cx.listener(move |view, _, _, cx| {
                    view.state.update(cx, |state, cx| {
                        state.queued.retain(|entry| entry.id != id);
                        cx.notify();
                    });
                    MainView::command(Command::Unqueue(id), cx);
                })),
        )
        .gap(px(SPACE_SM))
        .into_any_element()
}
