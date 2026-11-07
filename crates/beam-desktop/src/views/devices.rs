use std::path::Path;

use beam_core::NearbyDevice;
use gpui_kit::component::ActiveTheme;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Disableable, Sizable};
use gpui_kit::{
    AnyElement, Context, ElementId, ExternalPaths, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, Window, div, px,
};
use iroh::{EndpointAddr, EndpointId};

use super::main_view::MainView;
use super::ui;
use crate::service::{Command, Target};
use crate::state::AppState;

struct Row {
    id: EndpointId,
    addr: EndpointAddr,
    name: String,
    detail: &'static str,
    approval: Option<bool>,
}

fn rows(state: &AppState) -> (Vec<Row>, Vec<Row>) {
    let near = |id: &EndpointId| state.nearby.iter().find(|device| device.addr.id == *id);
    let mine = state
        .devices
        .iter()
        .map(|device| Row {
            id: device.id,
            addr: near(&device.id).map_or_else(|| device.addr(), |found| found.addr.clone()),
            name: device.name.clone(),
            detail: if near(&device.id).is_some() { "On this network" } else { "Paired" },
            approval: Some(device.auto_accept),
        })
        .collect();
    let others = state
        .nearby
        .iter()
        .filter(|device: &&NearbyDevice| !device.known)
        .map(|device| Row {
            id: device.addr.id,
            addr: device.addr.clone(),
            name: device.name.clone().unwrap_or_else(|| "Unnamed device".to_owned()),
            detail: "Not paired, they will be asked first",
            approval: None,
        })
        .collect();
    (mine, others)
}

pub fn render(state: &AppState, view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let (mine, others) = rows(state);
    let empty = mine.is_empty() && others.is_empty();
    div()
        .flex()
        .flex_col()
        .gap_2()
        .when_some_rows("Devices", mine, view, cx)
        .when_some_rows("Nearby", others, view, cx)
        .children(empty.then(|| {
            ui::card(cx).child(ui::muted(
                "No devices yet. Open beam on another device, or add one with a code.",
                cx,
            ))
        }))
        .into_any_element()
}

trait DeviceSections {
    fn when_some_rows(
        self,
        title: &'static str,
        rows: Vec<Row>,
        view: &MainView,
        cx: &mut Context<MainView>,
    ) -> Self;
}

impl DeviceSections for gpui_kit::Div {
    fn when_some_rows(
        self,
        title: &'static str,
        rows: Vec<Row>,
        view: &MainView,
        cx: &mut Context<MainView>,
    ) -> Self {
        if rows.is_empty() {
            return self;
        }
        let elements: Vec<AnyElement> =
            rows.into_iter().map(|row| device_row(row, view, cx)).collect();
        self.child(ui::section_title(title, cx)).children(elements)
    }
}

fn device_row(row: Row, view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let selected = view.selected.contains(&row.id);
    let theme = cx.theme();
    let (border, hover) = (theme.border, theme.accent);
    let highlight = theme.primary;
    let target = Target { addr: row.addr.clone(), name: row.name.clone() };
    let id: ElementId = SharedString::from(format!("device-{}", row.id)).into();
    let (click_id, click_target) = (row.id, row.addr.clone());
    let click_name = row.name.clone();
    div()
        .id(id)
        .flex()
        .items_center()
        .justify_between()
        .gap_2()
        .p_3()
        .rounded(px(10.))
        .border_1()
        .border_color(if selected { highlight } else { border })
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .drag_over::<ExternalPaths>(move |style, _, _, _| style.border_color(highlight).bg(hover))
        .on_drop(cx.listener(move |view, paths: &ExternalPaths, _, cx| {
            let target = Target { addr: target.addr.clone(), name: target.name.clone() };
            view.send_now(vec![target], paths.paths().to_vec(), cx);
        }))
        .on_click(cx.listener(move |view, _, _, cx| {
            let target = Target { addr: click_target.clone(), name: click_name.clone() };
            view.activate_device(click_id, target, cx);
        }))
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().text_sm().child(row.name))
                .child(ui::muted(row.detail, cx)),
        )
        .children(row.approval.map(|auto_accept| {
            div()
                .flex()
                .gap_1()
                .child(approval_button(row.id, auto_accept, cx))
                .child(forget_button(row.id, cx))
        }))
        .into_any_element()
}

fn approval_button(id: EndpointId, auto_accept: bool, cx: &mut Context<MainView>) -> AnyElement {
    Button::new(SharedString::from(format!("approval-{id}")))
        .ghost()
        .xsmall()
        .label(if auto_accept { "Receives without asking" } else { "Asks first" })
        .on_click(cx.listener(move |_, _, _: &mut Window, cx| {
            MainView::command(Command::SetAutoAccept { id, enabled: !auto_accept }, cx);
        }))
        .into_any_element()
}

fn forget_button(id: EndpointId, cx: &mut Context<MainView>) -> AnyElement {
    Button::new(SharedString::from(format!("forget-{id}")))
        .ghost()
        .xsmall()
        .label("Forget")
        .on_click(cx.listener(move |_, _, _: &mut Window, cx| {
            MainView::command(Command::Forget(id), cx);
        }))
        .into_any_element()
}

pub fn pending_bar(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let count = view.pending.len();
    let chosen = view.selected.len();
    let summary = match count {
        1 => file_name(&view.pending[0]),
        _ => format!("{count} items"),
    };
    ui::card(cx)
        .child(div().text_sm().child(format!("Ready to send {summary}")))
        .child(ui::muted("Choose devices, then send.", cx))
        .child(
            div()
                .flex()
                .gap_2()
                .child(
                    Button::new("send-selected")
                        .primary()
                        .small()
                        .disabled(chosen == 0)
                        .label(if chosen > 1 {
                            format!("Send to {chosen} devices")
                        } else {
                            "Send".to_owned()
                        })
                        .on_click(cx.listener(|view, _, _, cx| view.send_selected(cx))),
                )
                .child(
                    Button::new("clear-pending")
                        .ghost()
                        .small()
                        .label("Cancel")
                        .on_click(cx.listener(|view, _, _, cx| view.clear_pending(cx))),
                ),
        )
        .into_any_element()
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned())
}
