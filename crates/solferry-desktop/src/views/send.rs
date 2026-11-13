use std::path::Path;
use std::time::Duration;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    Animation, AnimationExt, AnyElement, Context, FontWeight, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Transformation, div,
    ease_out_quint, percentage, point, px,
};
use rust_i18n::t;

use super::button::Button;
use super::devices::{self, Row};
use super::main_view::{MainView, SendPhase};
use super::{radar, ui};
use crate::assets::{Glyph, file_glyph, icon};
use crate::format;
use crate::service::{Command, Target};
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_CONTROL, RADIUS_FIELD, SPACE_LG, SPACE_SM};

const BUTTON_WIDTH: f32 = 189.0;

#[expect(clippy::cast_possible_truncation, reason = "a list position fits in f32")]
fn fraction(value: usize) -> f32 {
    format::precise(value as u64) as f32
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned())
}

fn file_row(index: usize, path: &Path, cx: &mut Context<MainView>) -> AnyElement {
    let name = file_name(path);
    let glyph = if path.is_dir() { Glyph::Folder } else { file_glyph(&name) };
    let size = std::fs::metadata(path)
        .ok()
        .filter(std::fs::Metadata::is_file)
        .map(|meta| format::bytes(meta.len()));
    let danger = color(palette(cx).danger);
    let step = fraction(index);
    div()
        .flex()
        .items_center()
        .gap(px(SPACE_SM))
        .h(px(32.))
        .px(px(SPACE_SM))
        .child(icon(glyph, 18.).text_color(cx.theme().muted_foreground))
        .child(ui::text(name).flex_1().min_w_0().truncate())
        .children(size.map(|size| ui::muted(size, cx)))
        .child(
            div()
                .id(SharedString::from(format!("unpick-{index}")))
                .flex()
                .items_center()
                .justify_center()
                .size(px(25.))
                .rounded(px(RADIUS_CONTROL))
                .cursor_pointer()
                .hover(|style| style.opacity(0.7))
                .on_click(cx.listener(move |view, _, _, cx| view.remove_pending(index, cx)))
                .child(icon(Glyph::Delete, 14.).text_color(danger)),
        )
        .with_animation(
            SharedString::from(format!("file-in-{index}")),
            Animation::new(Duration::from_millis(320 + 45 * index as u64))
                .with_easing(ease_out_quint()),
            move |row, delta| {
                let shown = ((delta * (1. + step * 0.14)) - step * 0.14).clamp(0., 1.);
                row.opacity(shown).ml(px((1. - shown) * -14.))
            },
        )
        .into_any_element()
}

fn target_row(row: &Row, view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let id = row.id;
    let selected = view.selected.contains(&id);
    let hover = cx.theme().accent;
    let target = Target { addr: row.addr.clone(), name: row.name.clone(), platform: row.platform };
    let mut body = div()
        .id(SharedString::from(format!("target-{id}")))
        .flex()
        .items_center()
        .gap(px(10.))
        .p(px(6.))
        .rounded(px(RADIUS_CONTROL))
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(cx.listener(move |view, _, _, cx| {
            view.toggle_target(id, target.clone(), cx);
        }))
        .child(devices::tile(row.platform, row.online, selected, cx))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .child(ui::text(row.name.clone()).truncate())
                .child(devices::detail(row, cx)),
        );
    if selected {
        body = body.bg(hover);
    }
    body.into_any_element()
}

fn cursor_at(delta: f32) -> Transformation {
    Transformation::rotate(percentage(0.25))
        .with_translation(point(px(delta * 34.), px(delta * -34.)))
}

fn send_button(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let palette = palette(cx);
    let chosen = view.selected.len();
    let phase = view.send_phase;
    let label = match (phase, chosen) {
        (_, 0 | 1) => t!("send.to_one"),
        (_, many) => t!("send.to_many", count = many),
    };
    let on_primary = color(palette.on_primary);
    let symbol: AnyElement = match phase {
        SendPhase::Idle => icon(Glyph::Cursor, 18.)
            .text_color(on_primary)
            .with_transformation(cursor_at(0.))
            .into_any_element(),
        SendPhase::Flying => icon(Glyph::Cursor, 18.)
            .text_color(on_primary)
            .with_animation(
                "cursor-launch",
                Animation::new(Duration::from_millis(360)),
                |glyph, delta| glyph.with_transformation(cursor_at(delta * delta)),
            )
            .into_any_element(),
    };
    let fill = color(palette.primary_fill);
    let button = div()
        .id("send-selected")
        .flex()
        .items_center()
        .justify_center()
        .gap(px(SPACE_SM))
        .w(px(BUTTON_WIDTH))
        .h(px(45.))
        .rounded(px(RADIUS_FIELD))
        .bg(fill)
        .overflow_hidden()
        .text_color(on_primary)
        .font_weight(FontWeight::MEDIUM)
        .child(symbol)
        .child(label);
    let button: AnyElement = match phase {
        _ if chosen == 0 => button.opacity(0.5).into_any_element(),
        SendPhase::Idle => button
            .cursor_pointer()
            .hover(|style| style.opacity(0.92))
            .on_click(cx.listener(|view, _, window, cx| view.send_selected(window, cx)))
            .into_any_element(),
        SendPhase::Flying => button.into_any_element(),
    };
    div().flex().justify_center().pt(px(SPACE_LG)).pb(px(SPACE_SM)).child(button).into_any_element()
}

pub fn sheet(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let (mine, others) = devices::rows(view.state.read(cx));
    let files: Vec<_> =
        view.pending.iter().enumerate().map(|(index, path)| file_row(index, path, cx)).collect();
    let targets: Vec<_> = mine.iter().chain(&others).map(|row| target_row(row, view, cx)).collect();
    let well = if targets.is_empty() {
        ui::well(cx).child(radar::scanning(cx))
    } else {
        ui::well(cx).children(targets)
    };
    let content = div()
        .flex()
        .flex_col()
        .child(div().flex().flex_col().children(files))
        .child(ui::frame(cx).child(well))
        .child(send_button(view, cx))
        .child(
            div().flex().justify_center().child(
                Button::new("share-link")
                    .ghost()
                    .sm()
                    .icon(Glyph::Link)
                    .label(t!("send.share_link"))
                    .on_click(cx.listener(|view, _, _, cx| {
                        MainView::command(Command::Link(view.pending.clone()), cx);
                        view.clear_pending(cx);
                    })),
            ),
        );
    super::sheet::present(view, t!("send.title"), content, |view, _, cx| view.clear_pending(cx), cx)
}
