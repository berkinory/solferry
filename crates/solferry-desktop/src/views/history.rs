use gpui_kit::component::ActiveTheme;
use gpui_kit::{AnyElement, Context, FontWeight, IntoElement, ParentElement, Styled, div, px};
use solferry_core::{Direction, PastTransfer, Platform};

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::{Glyph, icon};
use crate::format;
use crate::service::Command;
use crate::state::AppState;
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_FIELD, SPACE_MD, SPACE_SM, SPACE_XS, TYPE_LABEL};

fn empty(room: f32, cx: &Context<MainView>) -> AnyElement {
    let core = div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(60.))
        .rounded(px(RADIUS_FIELD))
        .bg(cx.theme().secondary)
        .border_1()
        .border_color(cx.theme().border)
        .child(icon(Glyph::History, 27.).text_color(cx.theme().muted_foreground));
    div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(SPACE_SM))
        .h(px(room))
        .child(core)
        .child(ui::text("No transfers yet").pt(px(SPACE_XS)).font_weight(FontWeight::MEDIUM))
        .child(ui::muted("Files you send or receive show up here.", cx).text_center())
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
        .label("Clear")
        .on_click(cx.listener(|_, _, _, cx| MainView::command(Command::ClearHistory, cx)))
        .into_any_element();
    let rows: Vec<_> =
        state.history.iter().map(|past| row(past, platform(&state, past), cx)).collect();
    ui::card(Glyph::History, "History", Some(clear), cx)
        .child(ui::well(cx).children(rows))
        .into_any_element()
}

fn platform(state: &AppState, past: &PastTransfer) -> Option<Platform> {
    let paired = state.devices.iter().find(|device| device.id == past.peer);
    let nearby = || state.nearby.iter().find(|device| device.addr.id == past.peer);
    paired
        .and_then(|device| device.platform)
        .or_else(|| nearby().and_then(|device| device.platform))
}

fn row(past: &PastTransfer, platform: Option<Platform>, cx: &Context<MainView>) -> AnyElement {
    let sending = past.direction == Direction::Sending;
    let glyph = if sending { Glyph::Upload } else { Glyph::Download };
    let verb = if sending { "To" } else { "From" };
    let outcome = if past.completed { format::bytes(past.bytes) } else { "Failed".to_owned() };
    let tone = if past.completed { cx.theme().muted_foreground } else { color(palette(cx).danger) };
    let detail = div()
        .flex()
        .items_center()
        .gap(px(SPACE_XS))
        .min_w_0()
        .text_size(px(TYPE_LABEL))
        .text_color(tone)
        .child(div().flex_none().child(verb))
        .child(icon(ui::platform_glyph(platform), 13.).text_color(tone))
        .child(div().min_w_0().truncate().child(past.peer_name.clone()))
        .child(div().flex_none().child(format!("·  {outcome}")));
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
                .child(detail),
        )
        .child(ui::caption(format::ago(past.finished_ms), cx).flex_none())
        .into_any_element()
}
