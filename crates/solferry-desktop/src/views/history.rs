use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyElement, App, Context, Entity, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, div, px, uniform_list,
};
use rust_i18n::t;
use solferry_core::{Direction, PastTransfer};

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::{Glyph, icon};
use crate::format;
use crate::service::Command;
use crate::state::{AppState, Tone};
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_CONTROL, RADIUS_FIELD, SPACE_MD, SPACE_SM, SPACE_XS, TYPE_LABEL};

fn empty(cx: &Context<MainView>) -> AnyElement {
    let core = div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(50.))
        .rounded(px(RADIUS_FIELD))
        .bg(cx.theme().secondary)
        .border_1()
        .border_color(cx.theme().border)
        .child(icon(Glyph::History, 23.).text_color(cx.theme().muted_foreground));
    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(SPACE_SM))
        .h_full()
        .child(core)
        .child(ui::text(t!("history.empty.title")).pt(px(SPACE_XS)).font_weight(FontWeight::MEDIUM))
        .child(ui::muted(t!("history.empty.body"), cx).text_center())
        .into_any_element()
}

/// Only the rows in view are drawn, so a long history scrolls as lightly as a short one.
pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let count = view.state.read(cx).history.len();
    if count == 0 {
        return empty(cx);
    }
    let clear = Button::new("clear-history")
        .ghost()
        .xs()
        .label(t!("history.clear"))
        .on_click(cx.listener(|_, _, _, cx| MainView::command(Command::ClearHistory, cx)))
        .into_any_element();
    let state = view.state.clone();
    let rows = uniform_list("history-rows", count, move |range, _, cx| {
        let history = &state.read(cx).history;
        let rows = history.get(range.clone()).unwrap_or_default();
        // The well spaced its rows with a gap, which a list has no room for.
        range
            .zip(rows)
            .map(|(index, past)| div().pb(px(SPACE_XS)).child(row(index, past, &state, cx)))
            .collect()
    })
    .size_full()
    .py(px(SPACE_SM));
    ui::card(Glyph::History, &t!("history.title"), Some(clear), cx)
        .h_full()
        .child(ui::well(cx).flex_1().min_h_0().py_0().child(rows))
        .into_any_element()
}

fn row(index: usize, past: &PastTransfer, state: &Entity<AppState>, cx: &App) -> AnyElement {
    let glyph = if past.direction == Direction::Sending { Glyph::Upload } else { Glyph::Download };
    let outcome =
        if past.completed { format::bytes(past.bytes) } else { t!("history.failed").into_owned() };
    let tone = if past.completed { cx.theme().muted_foreground } else { color(palette(cx).danger) };
    let title = div()
        .flex()
        .items_center()
        .gap(px(SPACE_SM))
        .child(ui::text(past.label.clone()).flex_1().min_w_0().truncate())
        .child(div().flex_none().text_size(px(TYPE_LABEL)).text_color(tone).child(outcome));
    let muted = cx.theme().muted_foreground;
    let detail = div()
        .flex()
        .items_center()
        .gap(px(SPACE_XS))
        .text_size(px(TYPE_LABEL))
        .text_color(muted)
        .child(icon(ui::platform_glyph(past.platform), 10.).text_color(muted))
        .child(div().flex_1().min_w_0().truncate().child(past.peer_name.clone()))
        .child(ui::caption(format::ago(past.finished_ms), cx).flex_none());
    let body = div()
        .id(("history-row", index))
        .flex()
        .items_center()
        .gap(px(SPACE_MD))
        .p(px(6.))
        .rounded(px(RADIUS_CONTROL))
        .child(ui::tile(glyph, cx))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .text_color(cx.theme().foreground)
                .child(title)
                .child(detail),
        );
    let Some(path) = past.path.clone().filter(|_| past.completed) else {
        return body.into_any_element();
    };
    let (state, hover) = (state.clone(), cx.theme().accent);
    body.cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(move |_, _, cx| {
            // Checked on click rather than while drawing, so scrolling never touches the disk.
            if path.exists() {
                cx.reveal_path(&path);
            } else {
                state.update(cx, |state, cx| {
                    state.toast(Tone::Error, t!("history.missing"), None);
                    cx.notify();
                });
            }
        })
        .into_any_element()
}
