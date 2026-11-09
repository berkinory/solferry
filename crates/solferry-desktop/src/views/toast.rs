use std::f32::consts::TAU;
use std::time::Duration;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, Context, Div, FontWeight, Hsla, InteractiveElement,
    IntoElement, ParentElement, SharedString, StatefulInteractiveElement, Styled, Transformation,
    div, ease_out_quint, percentage, px, relative,
};
use solferry_core::{Direction, Route};

use super::main_view::MainView;
use super::toast_timing::EXIT;
use crate::assets::{Glyph, icon};
use crate::format;
use crate::service::Command;
use crate::state::{AppState, Phase, Toast, Tone, Transfer};
use crate::theme::{color, palette};

const WIDTH: f32 = 340.0;
const RADIUS: f32 = 20.0;
const PILL_HEIGHT: f32 = 44.0;
const ROW_HEIGHT: f32 = 54.0;
const ENTER: Duration = Duration::from_millis(380);
#[cfg(target_os = "macos")]
const SIDES: (f32, f32) = (84.0, 84.0);
#[cfg(not(target_os = "macos"))]
const SIDES: (f32, f32) = (8.0, 112.0);

/// Foreground fill and background text, so the toast flips with the theme by itself.
fn shell(cx: &Context<MainView>) -> Div {
    div()
        .relative()
        .w_full()
        .rounded(px(RADIUS))
        .bg(cx.theme().foreground)
        .text_color(cx.theme().background)
        .shadow_lg()
}

fn faded(cx: &Context<MainView>) -> Hsla {
    cx.theme().background.opacity(0.62)
}

fn spinner(cx: &Context<MainView>) -> AnyElement {
    icon(Glyph::Loading, 18.)
        .text_color(cx.theme().background)
        .with_animation(
            "toast-spin",
            Animation::new(Duration::from_millis(800)).repeat(),
            |spin, delta| spin.with_transformation(Transformation::rotate(percentage(delta))),
        )
        .into_any_element()
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
        .size(px(18.))
        .bg(fill)
        .child(icon(glyph, 11.).text_color(gpui_kit::white()))
        .with_animation(
            id,
            Animation::new(Duration::from_millis(480)).with_easing(ease_out_quint()),
            |badge, delta| {
                let swell = (delta * TAU / 2.).sin() * 0.14;
                badge.size(px(18. * (0.86 + 0.14 * delta + swell)))
            },
        )
        .into_any_element()
}

fn close(id: SharedString, cx: &Context<MainView>) -> gpui_kit::Stateful<Div> {
    let hover = cx.theme().background.opacity(0.12);
    div()
        .id(id)
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(24.))
        .rounded(px(8.))
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .child(icon(Glyph::Cancel, 13.).text_color(faded(cx)))
}

fn route_name(route: Route) -> &'static str {
    match route {
        Route::Local => "local network",
        Route::Overlay => "overlay network",
        Route::Direct => "direct",
        Route::Relay => "relay",
    }
}

fn motion(transfer: &Transfer) -> String {
    let (verb, peer) = match transfer.direction {
        Direction::Sending => ("To", &transfer.peer),
        Direction::Receiving => ("From", &transfer.peer),
    };
    if matches!(transfer.phase, Phase::Reconnecting) {
        return format!("{verb} {peer}  ·  Reconnecting");
    }
    let route = route_name(transfer.route);
    transfer.seconds_left().map_or_else(
        || format!("{verb} {peer}  ·  {route}"),
        |left| {
            let speed = format::bytes(format::whole(transfer.speed));
            format!("{verb} {peer}  ·  {speed}/s  ·  {} left  ·  {route}", format::duration(left))
        },
    )
}

fn lines(
    id: SharedString,
    title: String,
    detail: String,
    cx: &Context<MainView>,
) -> impl IntoElement {
    div()
        .relative()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .child(div().text_size(px(13.)).font_weight(FontWeight::MEDIUM).truncate().child(title))
        .child(div().text_size(px(11.5)).text_color(faded(cx)).truncate().child(detail))
        .with_animation(
            id,
            Animation::new(Duration::from_millis(320)).with_easing(ease_out_quint()),
            |text, delta| text.top(px((1. - delta) * 9.)).opacity(delta),
        )
}

fn percent(fraction: f32) -> String {
    format!("{}%", format::whole(f64::from(fraction) * 100.))
}

fn progress(fraction: f32, cx: &Context<MainView>) -> impl IntoElement {
    let primary = color(palette(cx).primary);
    div()
        .absolute()
        .left(px(18.))
        .right(px(18.))
        .bottom(px(3.))
        .h(px(2.))
        .rounded_full()
        .bg(cx.theme().background.opacity(0.16))
        .child(div().h_full().rounded_full().bg(primary).w(relative(fraction)))
}

fn row_base() -> Div {
    div().flex().items_center().gap(px(10.)).pl(px(14.)).pr(px(8.)).h(px(PILL_HEIGHT))
}

fn running_row(transfer: &Transfer, cx: &mut Context<MainView>) -> Div {
    let id = transfer.id;
    let key = SharedString::from(format!("pill-text-{id}"));
    row_base()
        .child(spinner(cx))
        .child(lines(key, transfer.title.clone(), motion(transfer), cx))
        .child(div().text_size(px(12.)).text_color(faded(cx)).child(percent(transfer.fraction())))
        .child(
            close(SharedString::from(format!("pill-cancel-{id}")), cx).on_click(
                cx.listener(move |_, _, _, cx| MainView::command(Command::Cancel(id), cx)),
            ),
        )
}

fn ended_row(transfer: &Transfer, cx: &mut Context<MainView>) -> Div {
    let id = transfer.id;
    let (tone, title, detail) = transfer.outcome();
    let received = tone == Tone::Success && transfer.direction == Direction::Receiving;
    row_base()
        .child(badge(SharedString::from(format!("pill-badge-{id}")), tone, cx))
        .child(lines(SharedString::from(format!("pill-end-{id}")), title, detail, cx))
        .children(received.then(|| {
            div()
                .id("pill-folder")
                .flex_none()
                .cursor_pointer()
                .child(icon(Glyph::FolderOpen, 15.).text_color(faded(cx)))
                .on_click(cx.listener(|view, _, _, cx| view.show_downloads(cx)))
        }))
}

fn pill(state: &AppState, view: &MainView, cx: &mut Context<MainView>) -> Option<AnyElement> {
    let running: Vec<_> = state.transfers.iter().filter(|item| item.running()).collect();
    let body = match running.as_slice() {
        [] => ended_row(state.transfers.first()?, cx).into_any_element(),
        [only] => running_row(only, cx).into_any_element(),
        many => many_row(many, cx),
    };
    let fraction = match running.as_slice() {
        [] => None,
        many => {
            let done: u64 = many.iter().map(|item| item.transferred).sum();
            let total: u64 = many.iter().map(|item| item.total).sum();
            #[expect(clippy::cast_possible_truncation, reason = "a progress ratio fits in f32")]
            Some(if total == 0 {
                1.
            } else {
                (format::precise(done) / format::precise(total)) as f32
            })
        }
    };
    let list: Vec<_> = if view.transfers_open && running.len() > 1 {
        running.iter().map(|item| running_row(item, cx).into_any_element()).collect()
    } else {
        Vec::new()
    };
    let shell = shell(cx).child(body).children(list).children(fraction.map(|f| progress(f, cx)));
    let shell = if state.transfers.iter().any(|item| matches!(item.phase, Phase::Failed(_))) {
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
            .w_full()
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

fn many_row(running: &[&Transfer], cx: &mut Context<MainView>) -> AnyElement {
    let sending = running.iter().filter(|item| item.direction == Direction::Sending).count();
    let receiving = running.len() - sending;
    let detail = match (sending, receiving) {
        (0, count) => format!("Receiving {count}"),
        (count, 0) => format!("Sending {count}"),
        (out, inn) => format!("Sending {out}, receiving {inn}"),
    };
    let key = SharedString::from(format!("pill-many-{}", running.len()));
    row_base()
        .id("pill-many")
        .cursor_pointer()
        .on_click(cx.listener(|view, _, _, cx| {
            view.transfers_open = !view.transfers_open;
            cx.notify();
        }))
        .child(spinner(cx))
        .child(lines(key, format!("{} transfers", running.len()), detail, cx))
        .child(icon(Glyph::More, 15.).text_color(faded(cx)).mr(px(5.)))
        .into_any_element()
}

fn shake(element: Div, id: impl Into<SharedString>) -> impl IntoElement {
    element.with_animation(id.into(), Animation::new(Duration::from_millis(280)), |shell, delta| {
        shell.left(px((delta * TAU * 2.).sin() * 3. * (1. - delta)))
    })
}

fn toast_row(toast: &Toast, cx: &mut Context<MainView>) -> AnyElement {
    let key = toast.key;
    let leaving = toast.leaving;
    let retry = toast.retry.is_some().then(|| {
        let toast = toast.clone();
        div()
            .id(SharedString::from(format!("toast-retry-{key}")))
            .flex_none()
            .px(px(6.))
            .text_size(px(12.))
            .font_weight(FontWeight::MEDIUM)
            .cursor_pointer()
            .child("Try again")
            .on_click(cx.listener(move |view, _, _, cx| view.retry_toast(&toast, cx)))
    });
    let body = row_base()
        .child(badge(SharedString::from(format!("toast-badge-{key}")), toast.tone, cx))
        .child(lines(
            SharedString::from(format!("toast-text-{key}")),
            toast.title.clone(),
            toast.detail.clone(),
            cx,
        ))
        .children(retry)
        .child(
            close(SharedString::from(format!("toast-close-{key}")), cx)
                .on_click(cx.listener(move |view, _, _, cx| view.dismiss_toast(key, cx))),
        );
    let shell = shell(cx).child(body);
    let shell = if toast.tone == Tone::Error && !leaving {
        shake(shell, format!("toast-shake-{key}")).into_any_element()
    } else {
        shell.into_any_element()
    };
    let id = SharedString::from(format!("toast-{key}-{leaving}"));
    let span = if leaving { EXIT } else { ENTER };
    div()
        .w_full()
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
            .top(px(4.))
            .left(px(SIDES.0))
            .right(px(SIDES.1))
            .flex()
            .flex_col()
            .items_center()
            .child(
                div().flex().flex_col().w_full().max_w(px(WIDTH)).children(pill).children(toasts),
            )
            .into_any_element(),
    )
}
