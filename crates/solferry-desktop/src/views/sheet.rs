use std::time::Duration;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, Context, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, black, div,
    ease_in_out, ease_out_quint, px,
};
use rust_i18n::t;

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::tokens::{RADIUS_CARD, SPACE_MD, SPACE_SM};

const TOP: f32 = 33.0;
const TRAVEL: f32 = 150.0;
const RISE: Duration = Duration::from_millis(360);
const FALL: Duration = Duration::from_millis(220);

/// Which way the open sheet is moving.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    Rising,
    Falling,
}

impl MainView {
    /// Lets the open sheet slide away, then runs `close`, which takes it off the screen.
    pub fn dismiss(
        &mut self,
        close: impl FnOnce(&mut MainView, &mut Window, &mut Context<MainView>) + 'static,
        window: &mut Window,
        cx: &mut Context<MainView>,
    ) {
        if self.sheet_motion == Motion::Falling {
            return;
        }
        self.sheet_motion = Motion::Falling;
        cx.notify();
        cx.spawn_in(window, async move |view, cx| {
            cx.background_executor().timer(FALL).await;
            let _ = view.update_in(cx, |view, window, cx| {
                view.sheet_motion = Motion::Rising;
                close(view, window, cx);
            });
        })
        .detach();
    }
}

/// Comes up from the bottom over the home screen, only as tall as what it holds.
pub fn present(
    view: &MainView,
    title: impl Into<SharedString>,
    content: impl IntoElement,
    close: fn(&mut MainView, &mut Window, &mut Context<MainView>),
    cx: &mut Context<MainView>,
) -> AnyElement {
    let closing = view.sheet_motion == Motion::Falling;
    let header = div()
        .relative()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .h(px(41.))
        .child(ui::text(title).font_weight(FontWeight::MEDIUM))
        .child(
            div().absolute().left(px(SPACE_SM)).child(
                Button::new("close-sheet")
                    .ghost()
                    .xs()
                    .icon(Glyph::Cancel)
                    .tooltip(t!("sheet.close"))
                    .on_click(
                        cx.listener(move |view, _, window, cx| view.dismiss(close, window, cx)),
                    ),
            ),
        );
    let body = div()
        .id("sheet-content")
        .min_h_0()
        .child(div().flex().flex_col().px(px(SPACE_MD)).pb(px(SPACE_MD)).child(content))
        .overflow_y_scroll();
    let panel = div()
        .relative()
        .max_h_full()
        .flex()
        .flex_col()
        .rounded_t(px(RADIUS_CARD))
        .bg(cx.theme().background)
        .border_1()
        .border_color(cx.theme().border)
        .child(header)
        .child(body);
    let panel = if closing {
        panel
            .with_animation(
                "sheet-fall",
                Animation::new(FALL).with_easing(ease_in_out),
                |panel, delta| panel.top(px(delta * TRAVEL)).opacity(1. - delta),
            )
            .into_any_element()
    } else {
        panel
            .with_animation(
                "sheet-rise",
                Animation::new(RISE).with_easing(ease_out_quint()),
                |panel, delta| panel.top(px((1. - delta) * TRAVEL)),
            )
            .into_any_element()
    };
    let backdrop = div()
        .id("sheet-backdrop")
        .absolute()
        .inset_0()
        .bg(black().opacity(0.45))
        .on_click(cx.listener(move |view, _, window, cx| view.dismiss(close, window, cx)));
    let backdrop = if closing {
        backdrop
            .with_animation("sheet-undim", Animation::new(FALL), |backdrop, delta| {
                backdrop.opacity(1. - delta)
            })
            .into_any_element()
    } else {
        backdrop
            .with_animation(
                "sheet-dim",
                Animation::new(Duration::from_millis(240)),
                Styled::opacity,
            )
            .into_any_element()
    };
    div()
        .id("sheet")
        .absolute()
        .inset_0()
        .occlude()
        .child(backdrop)
        .child(
            div()
                .absolute()
                .left_0()
                .right_0()
                .bottom_0()
                .top(px(TOP))
                .flex()
                .flex_col()
                .justify_end()
                .child(panel),
        )
        .into_any_element()
}
