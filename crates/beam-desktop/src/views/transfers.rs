use beam_core::{Direction, Route};
use gpui_kit::component::ActiveTheme;
use gpui_kit::component::Sizable;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement, SharedString, Styled, div, px, relative,
};

use super::main_view::MainView;
use super::ui;
use crate::errors::explain;
use crate::format;
use crate::service::Command;
use crate::state::{AppState, FailedSend, Phase, Transfer};

pub fn render(state: &AppState, cx: &mut Context<MainView>) -> AnyElement {
    let empty = state.transfers.is_empty() && state.failures.is_empty();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .children((!empty).then(|| ui::section_title("Transfers", cx)))
        .children(
            state.failures.iter().enumerate().map(|(index, failure)| failed(index, failure, cx)),
        )
        .children(state.transfers.iter().map(|transfer| row(transfer, cx)))
        .into_any_element()
}

fn route_name(route: Route) -> &'static str {
    match route {
        Route::Local => "local network",
        Route::Direct => "direct",
        Route::Relay => "relay",
    }
}

fn status(transfer: &Transfer) -> String {
    let done = format::bytes(transfer.transferred);
    let total = format::bytes(transfer.total);
    match &transfer.phase {
        Phase::Done => format!("{total}, finished"),
        Phase::Failed(error) => explain(error).next.to_owned(),
        Phase::Reconnecting => format!("{done} of {total}, reconnecting"),
        Phase::Active => {
            let route = route_name(transfer.route);
            match transfer.seconds_left() {
                Some(left) => {
                    let speed = format::bytes(format::whole(transfer.speed));
                    format!(
                        "{done} of {total}, {speed}/s, {} left, {route}",
                        format::duration(left)
                    )
                }
                None => format!("{done} of {total}, waiting, {route}"),
            }
        }
    }
}

fn row(transfer: &Transfer, cx: &mut Context<MainView>) -> AnyElement {
    let id = transfer.id;
    let verb = match transfer.direction {
        Direction::Sending => "To",
        Direction::Receiving => "From",
    };
    let running = matches!(transfer.phase, Phase::Active | Phase::Reconnecting);
    let title = match &transfer.phase {
        Phase::Failed(error) => explain(error).title.to_owned(),
        _ => transfer.title.clone(),
    };
    let (track, fill) = (cx.theme().muted_foreground.opacity(0.2), cx.theme().primary);
    let finished = !running;
    let folder = (transfer.direction == Direction::Receiving
        && matches!(transfer.phase, Phase::Done))
    .then(|| {
        Button::new(SharedString::from(format!("show-{id}")))
            .ghost()
            .xsmall()
            .label("Show folder")
            .on_click(cx.listener(|view, _, _, cx| view.show_downloads(cx)))
    });
    ui::card(cx)
        .child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(div().text_sm().child(title))
                        .child(ui::muted(format!("{verb} {}", transfer.peer), cx)),
                )
                .children(running.then(|| {
                    Button::new(SharedString::from(format!("cancel-{id}")))
                        .ghost()
                        .xsmall()
                        .label("Cancel")
                        .on_click(cx.listener(move |_, _, _, cx| {
                            MainView::command(Command::Cancel(id), cx);
                        }))
                }))
                .children(finished.then(|| {
                    div().flex().gap_1().children(folder).child(
                        Button::new(SharedString::from(format!("clear-{id}")))
                            .ghost()
                            .xsmall()
                            .label("Clear")
                            .on_click(cx.listener(move |view, _, _, cx| view.clear(id, cx))),
                    )
                })),
        )
        .children(running.then(|| {
            div()
                .h(px(6.))
                .w_full()
                .rounded(px(3.))
                .bg(track)
                .child(div().h_full().rounded(px(3.)).bg(fill).w(relative(transfer.fraction())))
        }))
        .child(ui::muted(status(transfer), cx))
        .into_any_element()
}

fn failed(index: usize, failure: &FailedSend, cx: &mut Context<MainView>) -> AnyElement {
    let explanation = explain(&failure.error);
    ui::card(cx)
        .border_color(cx.theme().danger)
        .child(div().text_sm().child(explanation.title))
        .child(ui::muted(format!("Sending to {}", failure.name), cx))
        .child(ui::muted(explanation.next, cx))
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new(SharedString::from(format!("retry-{index}")))
                        .primary()
                        .xsmall()
                        .label("Try again")
                        .on_click(cx.listener(move |view, _, _, cx| view.retry(index, cx))),
                )
                .child(
                    Button::new(SharedString::from(format!("dismiss-{index}")))
                        .ghost()
                        .xsmall()
                        .label("Dismiss")
                        .on_click(cx.listener(move |view, _, _, cx| view.dismiss(index, cx))),
                ),
        )
        .into_any_element()
}
