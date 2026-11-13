use gpui_kit::component::ActiveTheme;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::{
    App, AsyncApp, Context, Entity, FocusHandle, FontWeight, InteractiveElement, IntoElement,
    KeyDownEvent, ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window,
    div, px, svg,
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
}

impl Popover {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
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
                window.remove_window();
            }
        })
        .detach();
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self { state, focus, was_active: false }
    }
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
    let target = Target { addr: row.addr.clone(), name: row.name.clone() };
    let hover = cx.theme().accent;
    div()
        .id(SharedString::from(format!("quick-{}", row.id)))
        .flex()
        .flex_none()
        .items_center()
        .gap(px(10.))
        .h(px(ROW))
        .px(px(6.))
        .rounded(px(RADIUS_CONTROL))
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
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
                .overflow_y_scrollbar()
                .into_any_element()
        };
        let mut frame = div()
            .id("popover")
            .track_focus(&self.focus)
            .on_key_down(|event: &KeyDownEvent, window, _| {
                if event.keystroke.key == "escape" {
                    window.remove_window();
                }
            })
            .flex()
            .flex_col()
            .size_full()
            .overflow_hidden()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family(cx.theme().font_family.clone());
        if cfg!(target_os = "macos") {
            frame = frame.rounded(px(RADIUS_CARD)).border_1().border_color(cx.theme().border);
        }
        frame
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
            )
    }
}
