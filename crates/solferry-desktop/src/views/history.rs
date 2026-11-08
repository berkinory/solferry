use gpui_kit::component::ActiveTheme;
use gpui_kit::{AnyElement, Context, IntoElement, ParentElement, Styled, div, px};
use solferry_core::{Direction, PastTransfer};

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::format;
use crate::service::Command;
use crate::theme::{color, palette};
use crate::tokens::SPACE_MD;

pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let history = view.state.read(cx).history.clone();
    if history.is_empty() {
        return ui::card(Glyph::History, "History", None, cx)
            .child(
                ui::padded_well(cx)
                    .items_center()
                    .py(px(24.))
                    .child(ui::text("No transfers yet"))
                    .child(ui::muted("Files you send or receive are listed here.", cx)),
            )
            .into_any_element();
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
