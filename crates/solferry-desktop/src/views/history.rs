use gpui_kit::component::ActiveTheme;
use gpui_kit::{AnyElement, Context, FontWeight, IntoElement, ParentElement, Styled, div, px};
use solferry_core::{Direction, PastTransfer};

use super::button::Button;
use super::home::fan_tile;
use super::main_view::MainView;
use super::ui;
use crate::assets::{Glyph, icon};
use crate::format;
use crate::service::Command;
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_FIELD, SPACE_MD, SPACE_SM, SPACE_XS};

fn empty(room: f32, cx: &Context<MainView>) -> AnyElement {
    let core = div()
        .absolute()
        .top(px(4.))
        .left(px(42.))
        .flex()
        .items_center()
        .justify_center()
        .size(px(56.))
        .rounded(px(RADIUS_FIELD))
        .bg(cx.theme().secondary)
        .border_1()
        .border_color(cx.theme().border)
        .child(icon(Glyph::History, 24.).text_color(cx.theme().muted_foreground));
    let fan = div()
        .relative()
        .w(px(140.))
        .h(px(64.))
        .child(fan_tile("past-out", Glyph::Upload, -1., false, cx))
        .child(fan_tile("past-in", Glyph::Download, 1., false, cx))
        .child(core);
    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(SPACE_SM))
        .h(px(room))
        .child(fan)
        .child(ui::text("No transfers yet").pt(px(SPACE_XS)).font_weight(FontWeight::MEDIUM))
        .child(ui::muted("Files you send or receive show up here.", cx).text_center())
        .into_any_element()
}

pub fn render(view: &MainView, room: f32, cx: &mut Context<MainView>) -> AnyElement {
    let history = view.state.read(cx).history.clone();
    if history.is_empty() {
        return empty(room, cx);
    }
    let clear = Button::new("clear-history")
        .ghost()
        .xs()
        .label("Clear")
        .on_click(cx.listener(|_, _, _, cx| MainView::command(Command::ClearHistory, cx)))
        .into_any_element();
    let rows: Vec<_> = history.iter().map(|past| row(past, cx)).collect();
    ui::card(Glyph::History, "History", Some(clear), cx)
        .child(ui::well(cx).children(rows))
        .into_any_element()
}

fn row(past: &PastTransfer, cx: &Context<MainView>) -> AnyElement {
    let sending = past.direction == Direction::Sending;
    let glyph = if sending { Glyph::Upload } else { Glyph::Download };
    let verb = if sending { "To" } else { "From" };
    let outcome = if past.completed { format::bytes(past.bytes) } else { "Failed".to_owned() };
    let line =
        format!("{verb} {}  ·  {outcome}  ·  {}", past.peer_name, format::ago(past.finished_ms));
    let detail = if past.completed {
        ui::muted(line, cx)
    } else {
        ui::muted(line, cx).text_color(color(palette(cx).danger))
    };
    div()
        .flex()
        .items_center()
        .gap(px(SPACE_MD))
        .p(px(7.))
        .child(ui::tile(glyph, cx))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .text_color(cx.theme().foreground)
                .child(ui::text(past.label.clone()).truncate())
                .child(detail.truncate()),
        )
        .into_any_element()
}
