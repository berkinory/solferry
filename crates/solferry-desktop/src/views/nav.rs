use std::time::Duration;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, BoxShadow, Context, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, black, div, ease_out_quint,
    point, px,
};
use rust_i18n::t;

use super::main_view::{MainView, Page};
use crate::assets::{Glyph, icon};
use crate::format;
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_CONTROL, RADIUS_FIELD, TYPE_LABEL};

/// Room the scrolling content leaves so nothing ends up under the bar.
pub const CLEARANCE: f32 = 62.0;

const ITEM_WIDTH: f32 = 94.0;
const ITEM_HEIGHT: f32 = 33.0;
const INSET: f32 = 4.0;
const SLIDE: Duration = Duration::from_millis(240);
const TABS: [(Page, Glyph, &str); 3] = [
    (Page::Home, Glyph::Home, "nav.transfer"),
    (Page::History, Glyph::History, "nav.history"),
    (Page::Settings, Glyph::Settings, "nav.settings"),
];

#[expect(clippy::cast_possible_truncation, reason = "a tab position fits in f32")]
fn slot(page: Page) -> f32 {
    let index = TABS.iter().position(|(tab, ..)| *tab == page).unwrap_or(0);
    INSET + ITEM_WIDTH * format::precise(index as u64) as f32
}

pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let primary = color(palette(cx).primary);
    let (from, to) = (slot(view.nav_from), slot(view.page));
    let indicator = div()
        .absolute()
        .top(px(INSET))
        .left(px(to))
        .w(px(ITEM_WIDTH))
        .h(px(ITEM_HEIGHT))
        .rounded(px(RADIUS_CONTROL))
        .bg(cx.theme().secondary)
        .with_animation(
            SharedString::from(format!("nav-slide-{}", view.nav_moves)),
            Animation::new(SLIDE).with_easing(ease_out_quint()),
            move |pill, delta| pill.left(px(from + (to - from) * delta)),
        );
    let items: Vec<_> = TABS
        .into_iter()
        .map(|(page, glyph, key)| {
            let tone = if view.page == page { primary } else { cx.theme().muted_foreground };
            div()
                .id(SharedString::from(key))
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(6.))
                .w(px(ITEM_WIDTH))
                .h(px(ITEM_HEIGHT))
                .cursor_pointer()
                .text_size(px(TYPE_LABEL))
                .text_color(tone)
                .on_click(cx.listener(move |view, _, window, cx| view.open_page(page, window, cx)))
                .child(icon(glyph, 13.).text_color(tone))
                .child(t!(key))
                .into_any_element()
        })
        .collect();
    div()
        .absolute()
        .bottom(px(10.))
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .child(
            div()
                .relative()
                .flex()
                .p(px(INSET))
                .rounded(px(RADIUS_FIELD))
                .bg(cx.theme().background)
                .border_1()
                .border_color(cx.theme().border)
                .shadow(vec![BoxShadow {
                    color: black().opacity(0.2),
                    offset: point(px(0.), px(5.)),
                    blur_radius: px(15.),
                    spread_radius: px(0.),
                    inset: false,
                }])
                .child(indicator)
                .children(items),
        )
        .into_any_element()
}
