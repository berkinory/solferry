use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement, SharedString, Styled, div, px, relative,
};
use solferry_core::{Direction, Route};

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::format;
use crate::service::Command;
use crate::state::{AppState, FailedSend, Phase, Transfer};
use crate::theme::{color, palette, route_color};
use crate::tokens::{SPACE_SM, SPACE_XS};
use solferry_core::explain;

pub fn render(state: &AppState, cx: &mut Context<MainView>) -> Option<AnyElement> {
    if state.transfers.is_empty() && state.failures.is_empty() {
        return None;
    }
    let failures: Vec<_> = state
        .failures
        .iter()
        .enumerate()
        .map(|(index, failure)| failed(index, failure, cx))
        .collect();
    let rows: Vec<_> = state.transfers.iter().map(|transfer| row(transfer, cx)).collect();
    let finished = state.transfers.iter().filter(|transfer| !transfer.running()).count();
    let aside = (finished > 1).then(|| {
        Button::new("clear-finished")
            .ghost()
            .xs()
            .label("Clear finished")
            .on_click(cx.listener(|view, _, _, cx| view.clear_finished(cx)))
            .into_any_element()
    });
    let card = (!rows.is_empty()).then(|| {
        ui::card(Glyph::Transfers, "Transfers", aside, cx)
            .child(ui::well(cx).gap(px(2.)).children(rows))
    });
    Some(
        div()
            .flex()
            .flex_col()
            .gap(px(SPACE_SM))
            .children(failures)
            .children(card)
            .into_any_element(),
    )
}

fn route_name(route: Route) -> &'static str {
    match route {
        Route::Local => "local network",
        Route::Overlay => "overlay network",
        Route::Direct => "direct",
        Route::Relay => "relay",
    }
}

fn peer_line(transfer: &Transfer) -> String {
    let verb = match transfer.direction {
        Direction::Sending => "To",
        Direction::Receiving => "From",
    };
    if transfer.running() {
        format!("{verb} {}, {}", transfer.peer, route_name(transfer.route))
    } else {
        format!("{verb} {}", transfer.peer)
    }
}

fn numbers(transfer: &Transfer, cx: &mut Context<MainView>) -> Option<AnyElement> {
    let done = format::bytes(transfer.transferred);
    let total = format::bytes(transfer.total);
    let right = match &transfer.phase {
        Phase::Done | Phase::Failed(_) => return None,
        Phase::Reconnecting => "reconnecting".to_owned(),
        Phase::Active => transfer.seconds_left().map_or_else(
            || "waiting".to_owned(),
            |left| {
                let speed = format::bytes(format::whole(transfer.speed));
                format!("{speed}/s, {} left", format::duration(left))
            },
        ),
    };
    Some(
        div()
            .flex()
            .justify_between()
            .gap(px(SPACE_SM))
            .child(ui::caption(format!("{done} / {total}"), cx))
            .child(ui::caption(right, cx))
            .into_any_element(),
    )
}

fn outcome(transfer: &Transfer, cx: &mut Context<MainView>) -> Option<AnyElement> {
    let palette = palette(cx);
    let line = match &transfer.phase {
        Phase::Done => ui::status(color(palette.success), format::bytes(transfer.total), cx),
        Phase::Failed(error) => ui::status(color(palette.danger), explain(error).next, cx),
        _ => return None,
    };
    Some(line.into_any_element())
}

fn progress(transfer: &Transfer, cx: &mut Context<MainView>) -> AnyElement {
    let palette = palette(cx);
    let fill = if matches!(transfer.phase, Phase::Reconnecting) {
        color(palette.warning)
    } else {
        color(palette.primary)
    };
    div()
        .h(px(6.))
        .w_full()
        .my(px(2.))
        .rounded(px(3.))
        .bg(cx.theme().secondary)
        .child(div().h_full().rounded(px(3.)).bg(fill).w(relative(transfer.fraction())))
        .into_any_element()
}

fn row(transfer: &Transfer, cx: &mut Context<MainView>) -> AnyElement {
    let id = transfer.id;
    let running = transfer.running();
    let title = match &transfer.phase {
        Phase::Failed(error) => explain(error).title.to_owned(),
        _ => transfer.title.clone(),
    };
    let received =
        transfer.direction == Direction::Receiving && matches!(transfer.phase, Phase::Done);
    let folder = received.then(|| {
        Button::new(SharedString::from(format!("show-{id}")))
            .ghost()
            .xs()
            .icon(Glyph::FolderOpen)
            .tooltip("Show in folder")
            .on_click(cx.listener(|view, _, _, cx| view.show_downloads(cx)))
    });
    let close = Button::new(SharedString::from(format!("close-{id}")))
        .ghost()
        .xs()
        .icon(Glyph::Cancel)
        .tooltip(if running { "Cancel" } else { "Clear" })
        .on_click(cx.listener(move |view, _, _, cx| {
            if running {
                MainView::command(Command::Cancel(id), cx);
            } else {
                view.clear(id, cx);
            }
        }));
    let glyph = match transfer.direction {
        Direction::Sending => Glyph::Upload,
        Direction::Receiving => Glyph::Download,
    };
    let tone = if running { route_color(transfer.route, cx) } else { cx.theme().muted_foreground };
    let leading = ui::tinted_tile(glyph, cx.theme().secondary, tone);
    let bar = running.then(|| progress(transfer, cx));
    let measures = numbers(transfer, cx);
    let result = outcome(transfer, cx);
    div()
        .flex()
        .gap(px(10.))
        .p(px(7.))
        .child(leading)
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(px(2.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(SPACE_XS))
                        .child(ui::text(title).flex_1().min_w_0().truncate())
                        .children(folder)
                        .child(close),
                )
                .child(ui::muted(peer_line(transfer), cx).truncate())
                .children(bar)
                .children(measures)
                .children(result),
        )
        .into_any_element()
}

fn failed(index: usize, failure: &FailedSend, cx: &mut Context<MainView>) -> AnyElement {
    let explanation = explain(&failure.error);
    ui::failure(explanation.title, format!("{} {}", failure.name, explanation.next), cx)
        .child(
            div()
                .flex()
                .gap(px(SPACE_XS))
                .p(px(SPACE_XS))
                .child(
                    Button::new(SharedString::from(format!("retry-{index}")))
                        .primary()
                        .sm()
                        .label("Try again")
                        .on_click(cx.listener(move |view, _, _, cx| view.retry(index, cx))),
                )
                .child(
                    Button::new(SharedString::from(format!("dismiss-{index}")))
                        .ghost()
                        .sm()
                        .label("Dismiss")
                        .on_click(cx.listener(move |view, _, _, cx| view.dismiss(index, cx))),
                ),
        )
        .into_any_element()
}
