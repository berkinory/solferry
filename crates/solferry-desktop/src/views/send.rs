use std::path::Path;

use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px,
};

use super::button::Button;
use super::devices::{self, Row};
use super::main_view::MainView;
use super::{radar, ui};
use crate::assets::{Glyph, icon};
use crate::format;
use crate::service::Target;
use crate::tokens::{RADIUS_CONTROL, SPACE_SM, SPACE_XS};

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned())
}

fn file_row(index: usize, path: &Path, cx: &mut Context<MainView>) -> AnyElement {
    let glyph = if path.is_dir() { Glyph::Folder } else { Glyph::File };
    let size = std::fs::metadata(path)
        .ok()
        .filter(std::fs::Metadata::is_file)
        .map(|meta| format::bytes(meta.len()));
    div()
        .flex()
        .items_center()
        .gap(px(SPACE_SM))
        .pl(px(SPACE_SM))
        .h(px(32.))
        .child(icon(glyph, 16.).text_color(cx.theme().muted_foreground))
        .child(ui::text(file_name(path)).flex_1().min_w_0().truncate())
        .children(size.map(|size| ui::mono(size, cx)))
        .child(
            Button::new(SharedString::from(format!("unpick-{index}")))
                .ghost()
                .xs()
                .icon(Glyph::Cancel)
                .tooltip("Remove")
                .on_click(cx.listener(move |view, _, _, cx| view.remove_pending(index, cx))),
        )
        .into_any_element()
}

fn target_row(row: &Row, view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let id = row.id;
    let selected = view.selected.contains(&id);
    let hover = cx.theme().accent;
    let (addr, name) = (row.addr.clone(), row.name.clone());
    let mut body = div()
        .id(SharedString::from(format!("target-{id}")))
        .flex()
        .items_center()
        .gap(px(10.))
        .p(px(7.))
        .rounded(px(RADIUS_CONTROL))
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .on_click(cx.listener(move |view, _, _, cx| {
            view.toggle_target(id, Target { addr: addr.clone(), name: name.clone() }, cx);
        }))
        .child(devices::tile(&row.id, row.platform, selected, cx))
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

pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let (mine, others) = devices::rows(view.state.read(cx));
    let files: Vec<_> =
        view.pending.iter().enumerate().map(|(index, path)| file_row(index, path, cx)).collect();
    let add = Button::new("add-more")
        .ghost()
        .xs()
        .icon(Glyph::Add)
        .label("Add")
        .on_click(cx.listener(|_, _, _, cx| MainView::pick_pending(cx)))
        .into_any_element();
    let targets: Vec<_> = mine.iter().chain(&others).map(|row| target_row(row, view, cx)).collect();
    let well = if targets.is_empty() {
        ui::well(cx).child(radar::scanning(cx))
    } else {
        ui::well(cx).children(targets)
    };
    let chosen = view.selected.len();
    let action = if chosen == 0 {
        ui::muted("Choose who gets these", cx).py(px(SPACE_SM)).text_center().into_any_element()
    } else {
        let label =
            if chosen == 1 { "Send".to_owned() } else { format!("Send to {chosen} devices") };
        Button::new("send-selected")
            .primary()
            .grow()
            .icon(Glyph::Send)
            .label(label)
            .on_click(cx.listener(|view, _, _, cx| view.send_selected(cx)))
            .into_any_element()
    };
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .child(
            ui::card(Glyph::File, "Files", Some(add), cx)
                .child(ui::well(cx).gap(px(0.)).children(files)),
        )
        .child(ui::card(Glyph::Devices, "Send to", None, cx).child(well))
        .child(div().flex().flex_col().pt(px(SPACE_XS)).child(action))
        .into_any_element()
}
