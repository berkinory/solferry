use std::f32::consts::{FRAC_PI_2, TAU};
use std::time::Duration;

use gpui_kit::component::ActiveTheme;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, Bounds, Context, Div, FontWeight, Hsla,
    InteractiveElement, IntoElement, ParentElement, PathBuilder, Pixels, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window, canvas, div, ease_out_quint, point, px, rgb, white,
};
use solferry_core::Direction;

use super::main_view::MainView;
use super::toast_timing::EXIT;
use crate::assets::{Glyph, icon};
use crate::format;
use crate::service::Command;
use crate::state::{AppState, Phase, Toast, Tone, Transfer};
use crate::theme::{color, palette};

const WIDTH: f32 = 340.0;
const RADIUS: f32 = 20.0;
const HEIGHT: f32 = 42.0;
const RING: f32 = 16.0;
const ROW_HEIGHT: f32 = 48.0;
const ENTER: Duration = Duration::from_millis(380);
const RECONNECTING: u32 = 0x00f5_9e0b;
#[cfg(target_os = "macos")]
const CONTROLS: f32 = 84.0;
#[cfg(not(target_os = "macos"))]
const CONTROLS: f32 = 104.0;

/// Foreground fill and background text, sized by what it says, so it flips with the theme.
fn shell(cx: &Context<MainView>) -> Div {
    div()
        .relative()
        .flex()
        .flex_col()
        .min_w_0()
        .max_w(px(WIDTH))
        .rounded(px(RADIUS))
        .bg(cx.theme().foreground)
        .text_color(cx.theme().background)
        .shadow_lg()
}

fn faded(cx: &Context<MainView>) -> Hsla {
    cx.theme().background.opacity(0.76)
}

fn arc(bounds: Bounds<Pixels>, sweep: f32, tone: Hsla, window: &mut Window) {
    let center = bounds.center();
    let radius = f32::from(bounds.size.width) / 2. - 1.;
    let at = |angle: f32| {
        let turned = angle - FRAC_PI_2;
        point(center.x + px(turned.cos() * radius), center.y + px(turned.sin() * radius))
    };
    let mut path = PathBuilder::stroke(px(2.));
    path.move_to(at(0.));
    for step in 1..=48u8 {
        path.line_to(at(sweep * f32::from(step) / 48.));
    }
    if let Ok(built) = path.build() {
        window.paint_path(built, tone);
    }
}

/// How far along a transfer is, drawn by hand because only paths can curve.
fn ring(fraction: f32, tone: Hsla) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, (), window, _| {
            arc(bounds, TAU, tone.opacity(0.25), window);
            if fraction > 0. {
                arc(bounds, TAU * fraction.min(1.), tone, window);
            }
        },
    )
    .flex_none()
    .size(px(RING))
}

/// A filled circle with a mark that lands with a small overshoot.
fn badge(id: SharedString, tone: Tone, cx: &Context<MainView>) -> AnyElement {
    let palette = palette(cx);
    let (fill, glyph) = match tone {
        Tone::Success => (color(palette.success_fill), Glyph::Check),
        Tone::Error => (color(palette.danger_fill), Glyph::Cancel),
    };
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .rounded_full()
        .size(px(RING))
        .bg(fill)
        .child(icon(glyph, 10.).text_color(white()))
        .with_animation(
            id,
            Animation::new(Duration::from_millis(480)).with_easing(ease_out_quint()),
            |badge, delta| {
                let swell = (delta * TAU / 2.).sin() * 0.14;
                badge.size(px(RING * (0.86 + 0.14 * delta + swell)))
            },
        )
        .into_any_element()
}

fn dismiss(id: SharedString, cx: &Context<MainView>) -> Stateful<Div> {
    let hover = cx.theme().background.opacity(0.12);
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(24.))
        .rounded(px(6.))
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .child(icon(Glyph::Cancel, 12.).text_color(faded(cx)))
}

/// One line that comes up from below whenever it changes.
fn label(id: SharedString, text: String, wide: bool) -> impl IntoElement {
    let line = if wide { div().flex_1() } else { div() };
    line.relative()
        .min_w_0()
        .text_size(px(14.))
        .font_weight(FontWeight::MEDIUM)
        .truncate()
        .child(text)
        .with_animation(
            id,
            Animation::new(Duration::from_millis(320)).with_easing(ease_out_quint()),
            |text, delta| text.top(px((1. - delta) * 10.)).opacity(delta),
        )
}

fn percent(fraction: f32, cx: &Context<MainView>) -> impl IntoElement {
    div()
        .flex_none()
        .text_size(px(13.))
        .text_color(faded(cx))
        .child(format!("{}%", format::whole(f64::from(fraction) * 100.)))
}

fn row(close: bool) -> Div {
    div().flex().items_center().gap(px(10.)).h(px(HEIGHT)).pl(px(16.)).pr(px(if close {
        10.
    } else {
        18.
    }))
}

fn running_row(transfer: &Transfer, wide: bool, cx: &mut Context<MainView>) -> Div {
    let id = transfer.id;
    let waiting = matches!(transfer.phase, Phase::Reconnecting);
    let verb = match (waiting, transfer.direction) {
        (true, _) => "Reconnecting",
        (_, Direction::Sending) => "Sending",
        (_, Direction::Receiving) => "Receiving",
    };
    let tone = if waiting { rgb(RECONNECTING).into() } else { cx.theme().background };
    let key = SharedString::from(format!("pill-text-{id}-{waiting}"));
    row(true)
        .child(ring(transfer.fraction(), tone))
        .child(label(key, format!("{verb} {}", transfer.title), wide))
        .child(percent(transfer.fraction(), cx))
        .child(
            dismiss(SharedString::from(format!("pill-cancel-{id}")), cx).on_click(
                cx.listener(move |_, _, _, cx| MainView::command(Command::Cancel(id), cx)),
            ),
        )
}

fn ended_row(transfer: &Transfer, cx: &mut Context<MainView>) -> Div {
    let id = transfer.id;
    let (tone, title) = transfer.outcome();
    let received = tone == Tone::Success && transfer.direction == Direction::Receiving;
    let hover = cx.theme().background.opacity(0.12);
    row(received)
        .child(badge(SharedString::from(format!("pill-badge-{id}")), tone, cx))
        .child(label(SharedString::from(format!("pill-end-{id}")), title, false))
        .children(received.then(|| {
            div()
                .id("pill-folder")
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .size(px(24.))
                .rounded(px(6.))
                .cursor_pointer()
                .hover(move |style| style.bg(hover))
                .child(icon(Glyph::FolderOpen, 14.).text_color(faded(cx)))
                .on_click(cx.listener(|view, _, _, cx| view.show_downloads(cx)))
        }))
}

fn share(running: &[&Transfer]) -> f32 {
    let done: u64 = running.iter().map(|item| item.transferred).sum();
    let total: u64 = running.iter().map(|item| item.total).sum();
    #[expect(clippy::cast_possible_truncation, reason = "a progress ratio fits in f32")]
    if total == 0 { 1. } else { (format::precise(done) / format::precise(total)) as f32 }
}

fn many_row(running: &[&Transfer], cx: &mut Context<MainView>) -> AnyElement {
    let fraction = share(running);
    let key = SharedString::from(format!("pill-many-{}", running.len()));
    row(false)
        .id("pill-many")
        .cursor_pointer()
        .on_click(cx.listener(|view, _, _, cx| {
            view.transfers_open = !view.transfers_open;
            cx.notify();
        }))
        .child(ring(fraction, cx.theme().background))
        .child(label(key, format!("{} transfers", running.len()), true))
        .child(percent(fraction, cx))
        .into_any_element()
}

fn shake(element: Div, id: impl Into<SharedString>) -> impl IntoElement {
    element.with_animation(id.into(), Animation::new(Duration::from_millis(280)), |shell, delta| {
        shell.left(px((delta * TAU * 2.).sin() * 3. * (1. - delta)))
    })
}

fn pill(state: &AppState, view: &MainView, cx: &mut Context<MainView>) -> Option<AnyElement> {
    let running: Vec<_> = state.transfers.iter().filter(|item| item.running()).collect();
    let body = match running.as_slice() {
        [] => ended_row(state.transfers.first()?, cx).into_any_element(),
        [only] => running_row(only, false, cx).into_any_element(),
        many => many_row(many, cx),
    };
    let list: Vec<_> = if view.transfers_open && running.len() > 1 {
        running.iter().map(|item| running_row(item, true, cx).into_any_element()).collect()
    } else {
        Vec::new()
    };
    let open = !list.is_empty();
    let shell = shell(cx).when(open, Styled::w_full).child(body).children(list);
    let failed = state.transfers.iter().any(|item| matches!(item.phase, Phase::Failed(_)));
    let shell = if failed {
        shake(shell, "pill-shake").into_any_element()
    } else {
        shell.into_any_element()
    };
    let leaving = state.pill_leaving;
    let id = if leaving { "pill-out" } else { "pill-in" };
    let span = if leaving { EXIT } else { ENTER };
    Some(
        div()
            .relative()
            .flex()
            .justify_center()
            .w_full()
            .px(px(CONTROLS))
            .child(shell)
            .with_animation(
                id,
                Animation::new(span).with_easing(ease_out_quint()),
                move |pill, delta| {
                    let shown = if leaving { 1. - delta } else { delta };
                    pill.top(px((1. - shown) * -12.)).opacity(shown)
                },
            )
            .into_any_element(),
    )
}

fn toast_row(toast: &Toast, cx: &mut Context<MainView>) -> AnyElement {
    let key = toast.key;
    let leaving = toast.leaving;
    let failed = toast.tone == Tone::Error;
    let retry = toast.retry.is_some().then(|| {
        let toast = toast.clone();
        div()
            .id(SharedString::from(format!("toast-retry-{key}")))
            .flex_none()
            .text_size(px(13.))
            .font_weight(FontWeight::MEDIUM)
            .cursor_pointer()
            .child("Try again")
            .on_click(cx.listener(move |view, _, _, cx| view.retry_toast(&toast, cx)))
    });
    let close = failed.then(|| {
        dismiss(SharedString::from(format!("toast-close-{key}")), cx)
            .on_click(cx.listener(move |view, _, _, cx| view.dismiss_toast(key, cx)))
    });
    let body = row(failed)
        .child(badge(SharedString::from(format!("toast-badge-{key}")), toast.tone, cx))
        .child(label(SharedString::from(format!("toast-text-{key}")), toast.title.clone(), false))
        .children(retry)
        .children(close);
    let shell = shell(cx).child(body);
    let shell = if failed && !leaving {
        shake(shell, format!("toast-shake-{key}")).into_any_element()
    } else {
        shell.into_any_element()
    };
    let id = SharedString::from(format!("toast-{key}-{leaving}"));
    let span = if leaving { EXIT } else { ENTER };
    div()
        .flex()
        .justify_center()
        .w_full()
        .px(px(8.))
        .pt(px(6.))
        .h(px(ROW_HEIGHT))
        .child(shell)
        .with_animation(
            id,
            Animation::new(span).with_easing(ease_out_quint()),
            move |row, delta| {
                let shown = if leaving { 1. - delta } else { delta };
                row.h(px(ROW_HEIGHT * shown)).opacity(shown)
            },
        )
        .into_any_element()
}

/// One pill for what is moving over the brand, and short messages underneath it.
pub fn render(view: &MainView, cx: &mut Context<MainView>) -> Option<AnyElement> {
    let state = view.state.read(cx).clone();
    let pill = pill(&state, view, cx);
    if pill.is_none() && state.toasts.is_empty() {
        return None;
    }
    let toasts: Vec<_> = state.toasts.iter().map(|toast| toast_row(toast, cx)).collect();
    Some(
        div()
            .absolute()
            .top(px(5.))
            .left_0()
            .right_0()
            .flex()
            .flex_col()
            .children(pill)
            .children(toasts)
            .into_any_element(),
    )
}
