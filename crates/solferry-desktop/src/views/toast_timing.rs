use std::time::Duration;

use gpui_kit::{AsyncApp, Context, Entity};

use super::main_view::MainView;
use crate::service::{Command, Target};
use crate::state::{AppState, Phase, Toast, Tone, Transfer};

pub const EXIT: Duration = Duration::from_millis(220);
const SUCCESS_STAYS: Duration = Duration::from_secs(3);
const ERROR_STAYS: Duration = Duration::from_secs(5);
const RETRY_STAYS: Duration = Duration::from_secs(9);

fn stays(tone: Tone, retry: bool) -> Duration {
    match (tone, retry) {
        (_, true) => RETRY_STAYS,
        (Tone::Success, _) => SUCCESS_STAYS,
        (Tone::Error, _) => ERROR_STAYS,
    }
}

/// Starts the way out of anything that appeared since the last look.
pub fn track(view: &mut MainView, state: &Entity<AppState>, cx: &mut Context<MainView>) {
    let read = state.read(cx);
    let fresh: Vec<_> = read
        .toasts
        .iter()
        .filter(|toast| toast.key > view.toasts_seen)
        .map(|toast| (toast.key, stays(toast.tone, toast.retry.is_some())))
        .collect();
    let settled = read.pill_settled;
    let running = read.transfers.iter().any(Transfer::running);
    let failed = read.transfers.iter().any(|item| matches!(item.phase, Phase::Failed(_)));
    let idle = !running && !read.transfers.is_empty();
    view.toasts_seen = read.toasts.last().map_or(view.toasts_seen, |toast| toast.key);
    for (key, delay) in fresh {
        cx.spawn(async move |view, cx| {
            cx.background_executor().timer(delay).await;
            let _ = view.update(cx, |view, cx| view.dismiss_toast(key, cx));
        })
        .detach();
    }
    if settled != view.pill_seen {
        view.pill_seen = settled;
        if idle {
            let delay = if failed { ERROR_STAYS } else { SUCCESS_STAYS };
            let state = state.clone();
            cx.spawn(async move |_, cx| {
                cx.background_executor().timer(delay).await;
                leave_pill(&state, settled, cx).await;
            })
            .detach();
        }
    }
}

async fn leave_pill(state: &Entity<AppState>, settled: u64, cx: &mut AsyncApp) {
    let current = |state: &AppState| state.pill_settled == settled;
    state.update(cx, |state, cx| {
        if current(state) {
            state.pill_leaving = true;
            cx.notify();
        }
    });
    cx.background_executor().timer(EXIT).await;
    state.update(cx, |state, cx| {
        if current(state) {
            state.pill_leaving = false;
            state.transfers.retain(Transfer::running);
            cx.notify();
        }
    });
}

impl MainView {
    pub fn dismiss_toast(&mut self, key: u64, cx: &mut Context<Self>) {
        let state = self.state.clone();
        state.update(cx, |state, cx| {
            state.toasts.iter_mut().filter(|t| t.key == key).for_each(|t| t.leaving = true);
            cx.notify();
        });
        cx.spawn(async move |_, cx| {
            cx.background_executor().timer(EXIT).await;
            state.update(cx, |state, cx| {
                state.toasts.retain(|toast| toast.key != key);
                cx.notify();
            });
        })
        .detach();
    }

    pub fn retry_toast(&mut self, toast: &Toast, cx: &mut Context<Self>) {
        if let Some(failure) = toast.retry.clone() {
            let target = Target { addr: failure.addr, name: failure.name };
            Self::command(Command::Send { targets: vec![target], paths: failure.paths }, cx);
        }
        self.dismiss_toast(toast.key, cx);
    }
}
