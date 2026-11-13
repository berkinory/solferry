//! How fast files may go through a relay, typed in MB/s. 0 means no limit. Direct paths are never
//! limited.

use gpui_kit::component::Sizable;
use gpui_kit::component::input::Input;
use gpui_kit::{AnyElement, Context, IntoElement, ParentElement, Styled, Window, div, px};
use rust_i18n::t;

use super::main_view::MainView;
use super::settings::change;
use super::ui;
use crate::tokens::{SPACE_SM, SPACE_XS};

const MIB: u64 = 1024 * 1024;

pub fn field(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let hint = format!("{} 0 = {}", t!("relays.limit.hint"), t!("relays.limit.none"));
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_XS))
        .pt(px(SPACE_SM))
        .child(ui::text(t!("relays.limit.title")))
        .child(ui::muted(hint, cx))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(SPACE_XS))
                .child(div().w(px(120.)).child(Input::new(&view.relay_limit_input).large()))
                .child(ui::muted("MB/s", cx)),
        )
        .into_any_element()
}

/// Puts the saved limit in the field, 0 when there is none.
pub fn load(view: &MainView, window: &mut Window, cx: &mut Context<MainView>) {
    let Some(limit) = view.state.read(cx).config.as_ref().map(|config| config.relay_speed_limit)
    else {
        return;
    };
    let shown = limit.map_or(0, |bytes| bytes / MIB).to_string();
    view.relay_limit_input.update(cx, |input, cx| input.set_value(shown, window, cx));
}

/// Saves what was typed. Anything that isn't a whole number puts the saved limit back.
pub fn commit(view: &MainView, window: &mut Window, cx: &mut Context<MainView>) {
    let Some(current) = view.state.read(cx).config.as_ref().map(|config| config.relay_speed_limit)
    else {
        return;
    };
    let Ok(megabytes) = view.relay_limit_input.read(cx).value().trim().parse::<u64>() else {
        return load(view, window, cx);
    };
    let limit = (megabytes > 0).then(|| megabytes.saturating_mul(MIB));
    if limit != current {
        change(view, cx, |config| config.relay_speed_limit = limit);
    }
}
