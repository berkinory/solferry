use gpui_kit::component::ActiveTheme;
use std::time::Duration;

use gpui_kit::{
    Animation, AnimationExt, AnyElement, App, AsyncApp, Context, Entity, ExternalPaths,
    FocusHandle, FontWeight, InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Window, black, div, ease_in_out,
    ease_out_quint, px, svg, transparent_black,
};
use rust_i18n::t;
use solferry_core::Appearance;

use super::button::Button;
use super::devices::{self, Row};
use super::main_view::file_picker;
use super::{radar, ui};
use crate::assets::{Glyph, MARK};
use crate::service::{Command, Target};
use crate::shell::{self, Shell};
use crate::state::AppState;
use crate::theme::{self, color, palette};
use crate::tokens::{RADIUS_CARD, RADIUS_CONTROL, SPACE_SM, TYPE_LABEL};

pub const WIDTH: f32 = 300.0;
const HEADER: f32 = 44.0;
const ROW: f32 = 43.0;
const FOOTER: f32 = 52.0;
/// Beyond this many devices the list scrolls.
const ROWS_SHOWN: usize = 5;
/// Transparent room around the card on macOS, where it draws its own shadow.
pub const MARGIN: f32 = if cfg!(target_os = "macos") { 20.0 } else { 0.0 };
const OPEN: Duration = Duration::from_millis(180);
const CLOSE: Duration = Duration::from_millis(120);
/// How far the card drops into place as it appears.
const TRAVEL: f32 = 6.0;

pub fn device_count(cx: &App) -> usize {
    let (mine, others) = devices::rows(cx.global::<Shell>().state.read(cx));
    mine.len() + others.len()
}

#[expect(clippy::cast_precision_loss, reason = "a handful of rows")]
pub fn height(devices: usize) -> f32 {
    let rows = devices.clamp(1, ROWS_SHOWN) as f32;
    HEADER + rows * ROW + 2. * SPACE_SM + FOOTER
}

/// Pick a device, then the files go to it. Or pick the files first, and the send sheet opens.
pub struct Popover {
    state: Entity<AppState>,
    focus: FocusHandle,
    was_active: bool,
    closing: bool,
}

impl Popover {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        #[cfg(target_os = "macos")]
        crate::popover::borderless(window);
        let state = cx.global::<Shell>().state.clone();
        let appearance =
            state.read(cx).config.as_ref().map_or(Appearance::System, |c| c.appearance);
        theme::apply(appearance, window, cx);
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        cx.observe_window_activation(window, |view: &mut Self, window, cx| {
            if window.is_window_active() {
                view.was_active = true;
            } else if view.was_active {
                crate::popover::closed(cx);
                view.dismiss(window, cx);
            }
        })
        .detach();
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self { state, focus, was_active: false, closing: false }
    }

    /// Fades the card out on macOS before the window goes. Elsewhere the window is opaque, so it
    /// goes at once.
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if MARGIN == 0. {
            window.remove_window();
            return;
        }
        if self.closing {
            return;
        }
        self.closing = true;
        cx.notify();
        cx.spawn_in(window, async move |_, cx| {
            cx.background_executor().timer(CLOSE).await;
            let _ = cx.update(|window, _| window.remove_window());
        })
        .detach();
    }
}

/// On macOS the card is the window's only shape: rounded, with its own shadow, and moving in
/// and out.
fn shaped(card: gpui_kit::Stateful<gpui_kit::Div>, closing: bool, cx: &App) -> AnyElement {
    if MARGIN == 0. {
        return card.into_any_element();
    }
    let card = card
        .rounded(px(RADIUS_CARD))
        .border_1()
        .border_color(cx.theme().border)
        .shadow(vec![ui::shadow(black().opacity(0.3), 6., 14., 0., false)]);
    let card = if closing {
        card.with_animation(
            "popover-close",
            Animation::new(CLOSE).with_easing(ease_in_out),
            |card, delta| card.top(px(-delta * TRAVEL / 2.)).opacity(1. - delta),
        )
        .into_any_element()
    } else {
        card.with_animation(
            "popover-open",
            Animation::new(OPEN).with_easing(ease_out_quint()),
            |card, delta| card.top(px(-(1. - delta) * TRAVEL)).opacity(delta),
        )
        .into_any_element()
    };
    div().size_full().p(px(MARGIN)).child(card).into_any_element()
}

/// The file picker would open behind the popover, so the popover goes first.
fn pick_files(
    window: &mut Window,
    cx: &mut App,
    then: impl FnOnce(Vec<std::path::PathBuf>, &mut App) + 'static,
) {
    window.remove_window();
    cx.activate(true);
    let chosen = cx.prompt_for_paths(file_picker());
    cx.spawn(async move |cx: &mut AsyncApp| {
        let Ok(Ok(Some(paths))) = chosen.await else { return };
        cx.update(|cx| then(paths, cx));
    })
    .detach();
}

fn device_row(row: &Row, cx: &mut Context<Popover>) -> impl IntoElement {
    let target = Target { addr: row.addr.clone(), name: row.name.clone(), platform: row.platform };
    let (hover, ring) = (cx.theme().accent, color(palette(cx).primary));
    let drop_target = target.clone();
    div()
        .id(SharedString::from(format!("quick-{}", row.id)))
        .flex()
        .flex_none()
        .items_center()
        .gap(px(10.))
        .h(px(ROW))
        .px(px(6.))
        .rounded(px(RADIUS_CONTROL))
        .border_1()
        .border_color(transparent_black())
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .drag_over::<ExternalPaths>(move |style, _, _, _| style.border_color(ring).bg(hover))
        .on_drop(cx.listener(move |view, paths: &ExternalPaths, window, cx| {
            cx.stop_propagation();
            let (targets, paths) = (vec![drop_target.clone()], paths.paths().to_vec());
            cx.global::<Shell>().service.send(Command::Send { targets, paths });
            view.dismiss(window, cx);
        }))
        .on_click(cx.listener(move |_, _, window, cx| {
            let target = target.clone();
            pick_files(window, cx, move |paths, cx| {
                cx.global::<Shell>().service.send(Command::Send { targets: vec![target], paths });
            });
        }))
        .child(devices::tile(row.platform, row.online, false, cx))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .child(ui::text(row.name.clone()).truncate())
                .child(devices::detail(row, cx)),
        )
}

fn header(cx: &mut Context<Popover>) -> impl IntoElement {
    div()
        .flex()
        .flex_none()
        .items_center()
        .gap(px(6.))
        .h(px(HEADER))
        .px(px(SPACE_SM + 4.))
        .child(svg().path(MARK).size(px(14.)).text_color(color(palette(cx).brand)))
        .child(
            div()
                .flex_1()
                .text_size(px(TYPE_LABEL))
                .font_weight(FontWeight::MEDIUM)
                .child("Solferry"),
        )
        .child(
            Button::new("open-window")
                .ghost()
                .xs()
                .icon(Glyph::External)
                .tooltip(t!("tray.open"))
                .on_click(|_, window, cx| {
                    window.remove_window();
                    shell::show_window(cx);
                }),
        )
}

impl Render for Popover {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (mine, others) = devices::rows(self.state.read(cx));
        let rows: Vec<_> =
            mine.iter().chain(&others).map(|row| device_row(row, cx).into_any_element()).collect();
        let list = if rows.is_empty() {
            div().child(radar::scanning(cx)).into_any_element()
        } else {
            div()
                .id("quick-devices")
                .size_full()
                .flex()
                .flex_col()
                .children(rows)
                .overflow_y_scroll()
                .into_any_element()
        };
        let frame = div()
            .id("popover")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|view, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "escape" {
                    view.dismiss(window, cx);
                }
            }))
            .on_drop(cx.listener(|view, paths: &ExternalPaths, window, cx| {
                view.dismiss(window, cx);
                shell::send_files(paths.paths().to_vec(), cx);
            }))
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family(cx.theme().font_family.clone());
        let card = frame
            .child(header(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .px(px(SPACE_SM))
                    .child(ui::well(cx).size_full().child(list)),
            )
            .child(
                div().flex().flex_none().items_center().justify_center().h(px(FOOTER)).child(
                    Button::new("quick-choose")
                        .sm()
                        .icon(Glyph::FolderOpen)
                        .label(t!("menu.choose_files"))
                        .on_click(|_, window, cx| pick_files(window, cx, shell::send_files)),
                ),
            );
        shaped(card, self.closing, cx)
    }
}
