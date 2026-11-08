use std::time::Duration;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, BoxShadow, Context, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, black, div, ease_out_quint,
    point, px,
};

use super::main_view::{MainView, Page};
use crate::assets::{Glyph, icon};
use crate::theme::{color, palette};

/// Room the scrolling content leaves so nothing ends up under the bar.
pub const CLEARANCE: f32 = 80.0;

const ITEM_WIDTH: f32 = 84.0;
const INSET: f32 = 4.0;
const SLIDE: Duration = Duration::from_millis(240);
const TABS: [(Page, Glyph, &str); 3] = [
    (Page::Home, Glyph::Home, "Home"),
    (Page::History, Glyph::History, "History"),
    (Page::Settings, Glyph::Settings, "Settings"),
];

fn slot(page: Page) -> f32 {
    let index = TABS.iter().position(|(tab, ..)| *tab == page).unwrap_or(0);
    INSET + ITEM_WIDTH * index as f32
}

pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let primary = color(palette(cx).primary);
    let (from, to) = (slot(view.nav_from), slot(view.page));
    let indicator = div()
        .absolute()
        .top(px(INSET))
        .left(px(to))
        .w(px(ITEM_WIDTH))
        .h(px(46.))
        .rounded_full()
        .bg(cx.theme().secondary)
        .with_animation(
            SharedString::from(format!("nav-slide-{}", view.nav_moves)),
            Animation::new(SLIDE).with_easing(ease_out_quint()),
            move |pill, delta| pill.left(px(from + (to - from) * delta)),
        );
    let items: Vec<_> = TABS
        .into_iter()
        .map(|(page, glyph, label)| {
            let tone = if view.page == page { primary } else { cx.theme().muted_foreground };
            div()
                .id(SharedString::from(label))
                .relative()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .w(px(ITEM_WIDTH))
                .h(px(46.))
                .cursor_pointer()
                .text_size(px(11.))
                .text_color(tone)
                .on_click(cx.listener(move |view, _, window, cx| view.open_page(page, window, cx)))
                .child(icon(glyph, 18.).text_color(tone))
                .child(label)
                .into_any_element()
        })
        .collect();
    div()
        .absolute()
        .bottom(px(14.))
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .child(
            div()
                .relative()
                .flex()
                .p(px(INSET))
                .rounded_full()
                .bg(cx.theme().background)
                .border_1()
                .border_color(cx.theme().border)
                .shadow(vec![BoxShadow {
                    color: black().opacity(0.28),
                    offset: point(px(0.), px(10.)),
                    blur_radius: px(30.),
                    spread_radius: px(0.),
                    inset: false,
                }])
                .child(indicator)
                .children(items),
        )
        .into_any_element()
}
