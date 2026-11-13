//! How fast files may go through a relay. Direct paths are never limited.

use gpui_kit::base::Selectable;
use gpui_kit::{AnyElement, Context, IntoElement, ParentElement, SharedString, Styled, div, px};
use rust_i18n::t;
use solferry_core::{Config, RELAY_SPEED_LIMITS};

use super::button::Button;
use super::main_view::MainView;
use super::settings::change;
use super::ui;
use crate::tokens::{SPACE_SM, SPACE_XS};

const MIB: u64 = 1024 * 1024;

pub fn choices(config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    let current = config.relay_speed_limit;
    let options = std::iter::once(None).chain(RELAY_SPEED_LIMITS.into_iter().map(Some));
    let choices = options.map(|limit| {
        let label = limit.map_or_else(
            || t!("relays.limit.none"),
            |bytes| t!("relays.limit.speed", amount = bytes / MIB),
        );
        Button::new(SharedString::from(format!("relay-limit-{}", limit.unwrap_or(0))))
            .ghost()
            .xs()
            .label(label)
            .selected(current == limit)
            .on_click(cx.listener(move |view, _, _, cx| {
                change(view, cx, |config| config.relay_speed_limit = limit);
            }))
    });
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_XS))
        .pt(px(SPACE_SM))
        .child(ui::text(t!("relays.limit.title")))
        .child(ui::muted(t!("relays.limit.hint"), cx))
        .child(div().flex().flex_wrap().gap(px(SPACE_XS)).children(choices))
        .into_any_element()
}
