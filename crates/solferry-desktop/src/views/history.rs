use gpui_kit::component::ActiveTheme;
use gpui_kit::{AnyElement, Context, FontWeight, IntoElement, ParentElement, Styled, div, px};
use rust_i18n::t;
use solferry_core::{Direction, PastTransfer};

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::{Glyph, icon};
use crate::format;
use crate::service::Command;
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_FIELD, SPACE_MD, SPACE_SM, SPACE_XS, TYPE_LABEL};

fn empty(room: f32, cx: &Context<MainView>) -> AnyElement {
    let core = div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(53.))
        .rounded(px(RADIUS_FIELD))
        .bg(cx.theme().secondary)
        .border_1()
        .border_color(cx.theme().border)
        .child(icon(Glyph::History, 24.).text_color(cx.theme().muted_foreground));
    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(SPACE_SM))
        .h(px(room))
        .child(core)
        .child(ui::text(t!("history.empty.title")).pt(px(SPACE_XS)).font_weight(FontWeight::MEDIUM))
        .child(ui::muted(t!("history.empty.body"), cx).text_center())
        .into_any_element()
}

pub fn render(view: &MainView, room: f32, cx: &mut Context<MainView>) -> AnyElement {
    let state = view.state.read(cx).clone();
    if state.history.is_empty() {
        return empty(room, cx);
    }
    let clear = Button::new("clear-history")
        .ghost()
        .xs()
        .label(t!("history.clear"))
        .on_click(cx.listener(|_, _, _, cx| MainView::command(Command::ClearHistory, cx)))
        .into_any_element();
    let rows: Vec<_> = state.history.iter().map(|past| row(past, cx)).collect();
    ui::card(Glyph::History, &t!("history.title"), Some(clear), cx)
        .child(ui::well(cx).children(rows))
        .into_any_element()
}

fn row(past: &PastTransfer, cx: &Context<MainView>) -> AnyElement {
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
        .child(icon(ui::platform_glyph(past.platform), 11.).text_color(muted))
        .child(div().flex_1().min_w_0().truncate().child(past.peer_name.clone()))
        .child(ui::caption(format::ago(past.finished_ms), cx).flex_none());
    div()
        .flex()
        .items_center()
        .gap(px(SPACE_MD))
        .p(px(6.))
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
        )
        .into_any_element()
}
