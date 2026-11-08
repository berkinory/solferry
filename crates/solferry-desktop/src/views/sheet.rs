use std::time::Duration;

use gpui_kit::component::ActiveTheme;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, Context, FontWeight, InteractiveElement, IntoElement,
    ParentElement, StatefulInteractiveElement, Styled, Window, black, div, ease_out_quint, px,
};

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::tokens::{RADIUS_CARD, SPACE_MD, SPACE_SM};

const TOP: f32 = 40.0;
const TRAVEL: f32 = 180.0;

/// Comes up from the bottom over the home screen, the way a phone presents a sheet.
pub fn present(
    title: &'static str,
    content: impl IntoElement,
    close: fn(&mut MainView, &mut Window, &mut Context<MainView>),
    cx: &mut Context<MainView>,
) -> AnyElement {
    let header = div()
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .h(px(46.))
        .child(ui::text(title).font_weight(FontWeight::MEDIUM))
        .child(
            div().absolute().left(px(SPACE_SM)).child(
                Button::new("close-sheet")
                    .ghost()
                    .xs()
                    .icon(Glyph::Cancel)
                    .tooltip("Close")
                    .on_click(cx.listener(move |view, _, window, cx| close(view, window, cx))),
            ),
        );
    let body = div()
        .id("sheet-content")
        .flex_1()
        .min_h_0()
        .child(div().flex().flex_col().px(px(SPACE_MD)).pb(px(SPACE_SM)).child(content))
        .overflow_y_scrollbar();
    let panel = div()
        .absolute()
        .left_0()
        .right_0()
        .bottom_0()
        .top(px(TOP))
        .flex()
        .flex_col()
        .rounded_t(px(RADIUS_CARD))
        .bg(cx.theme().background)
        .border_1()
        .border_color(cx.theme().border)
        .child(header)
        .child(body)
        .with_animation(
            "sheet-rise",
            Animation::new(Duration::from_millis(360)).with_easing(ease_out_quint()),
            |panel, delta| panel.top(px(TOP + (1. - delta) * TRAVEL)),
        );
    div()
        .id("sheet")
        .absolute()
        .inset_0()
        .occlude()
        .child(
            div()
                .id("sheet-backdrop")
                .absolute()
                .inset_0()
                .bg(black().opacity(0.45))
                .on_click(cx.listener(move |view, _, window, cx| close(view, window, cx)))
                .with_animation(
                    "sheet-dim",
                    Animation::new(Duration::from_millis(240)),
                    Styled::opacity,
                ),
        )
        .child(panel)
        .into_any_element()
}
