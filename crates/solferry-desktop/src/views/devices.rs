use gpui_kit::component::ActiveTheme;
use gpui_kit::component::switch::Switch;
use gpui_kit::{
    AnyElement, App, Context, ElementId, ExternalPaths, Hsla, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
    transparent_black,
};
use iroh::{EndpointAddr, EndpointId};
use solferry_core::{NearbyDevice, Platform, Route};

use super::button::Button;
use super::dialog::Dialog;
use super::main_view::{MainView, Page};
use super::ui;
use crate::assets::{Glyph, icon};
use crate::service::{Command, Target};
use crate::state::AppState;
use crate::theme::{color, palette, route_color};
use crate::tokens::{RADIUS_CONTROL, RADIUS_FIELD, SPACE_SM, SPACE_XS};

struct Row {
    id: EndpointId,
    addr: EndpointAddr,
    name: String,
    platform: Option<Platform>,
    detail: &'static str,
    route: Option<Route>,
    approval: Option<bool>,
}

fn near_detail(device: &NearbyDevice) -> (&'static str, Route) {
    if device.overlay {
        ("On your overlay network", Route::Overlay)
    } else {
        ("On this network", Route::Local)
    }
}

fn rows(state: &AppState) -> (Vec<Row>, Vec<Row>) {
    let near = |id: &EndpointId| state.nearby.iter().find(|device| device.addr.id == *id);
    let mine = state
        .devices
        .iter()
        .map(|device| {
            let found = near(&device.id);
            Row {
                id: device.id,
                addr: found.map_or_else(|| device.addr(), |found| found.addr.clone()),
                name: device.name.clone(),
                platform: device.platform.or_else(|| found.and_then(|found| found.platform)),
                detail: found.map_or("Paired", |found| near_detail(found).0),
                route: found.map(|found| near_detail(found).1),
                approval: Some(device.auto_accept),
            }
        })
        .collect();
    let others = state
        .nearby
        .iter()
        .filter(|device| !device.known)
        .map(|device| Row {
            id: device.addr.id,
            addr: device.addr.clone(),
            name: device.name.clone().unwrap_or_else(|| "Unnamed device".to_owned()),
            platform: device.platform,
            detail: if device.overlay { "Overlay, not paired" } else { "Not paired" },
            route: None,
            approval: None,
        })
        .collect();
    (mine, others)
}

pub fn render(state: &AppState, view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let (mine, others) = rows(state);
    if mine.is_empty() && others.is_empty() {
        return empty(cx);
    }
    let section = |glyph, title, rows: Vec<Row>, cx: &mut Context<MainView>| {
        (!rows.is_empty()).then(|| {
            let count = ui::count(rows.len(), cx);
            let elements: Vec<_> = rows.into_iter().map(|row| device_row(row, view, cx)).collect();
            ui::card(glyph, title, Some(count), cx).child(ui::well(cx).children(elements))
        })
    };
    let paired = section(Glyph::Devices, "Devices", mine, cx);
    let nearby = section(Glyph::Nearby, "Nearby", others, cx);
    div().flex().flex_col().gap(px(SPACE_SM)).children(paired).children(nearby).into_any_element()
}

fn empty(cx: &mut Context<MainView>) -> AnyElement {
    ui::frame(cx)
        .child(
            ui::well(cx)
                .items_center()
                .gap(px(SPACE_SM))
                .py(px(28.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(44.))
                        .rounded(px(RADIUS_FIELD))
                        .bg(cx.theme().secondary)
                        .child(icon(Glyph::Devices, 22.).text_color(cx.theme().muted_foreground)),
                )
                .child(ui::text("No devices yet").pt(px(SPACE_XS)))
                .child(
                    ui::muted("Open Solferry on another device on this network.", cx)
                        .text_center()
                        .max_w(px(280.)),
                )
                .child(
                    div().pt(px(SPACE_XS)).child(
                        Button::new("empty-add")
                            .sm()
                            .icon(Glyph::Qr)
                            .label("Add with a code")
                            .on_click(cx.listener(|view, _, window, cx| {
                                view.open_page(Page::Pair, window, cx);
                            })),
                    ),
                ),
        )
        .into_any_element()
}

fn tile(id: &EndpointId, platform: Option<Platform>, selected: bool, cx: &App) -> AnyElement {
    if selected {
        let palette = palette(cx);
        ui::tinted_tile(Glyph::Check, color(palette.primary_fill), color(palette.on_primary))
            .into_any_element()
    } else {
        ui::device_tile(id, platform, cx).into_any_element()
    }
}

fn detail(row: &Row, cx: &App) -> AnyElement {
    let tone: Option<Hsla> = row.route.map(|route| route_color(route, cx));
    tone.map_or_else(
        || ui::muted(row.detail, cx).into_any_element(),
        |tone| ui::status(tone, row.detail, cx).into_any_element(),
    )
}

fn device_row(row: Row, view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let picking = !view.pending.is_empty();
    let selected = view.selected.contains(&row.id);
    let expanded = view.expanded == Some(row.id);
    let (hover, highlight) = (cx.theme().accent, color(palette(cx).primary));
    let group = SharedString::from(format!("row-{}", row.id));
    let id: ElementId = group.clone().into();
    let target = Target { addr: row.addr.clone(), name: row.name.clone() };
    let drop_target = Target { addr: row.addr.clone(), name: row.name.clone() };
    let device = row.id;
    let hint = (!picking && row.approval.is_some()).then(|| {
        div()
            .flex_none()
            .opacity(0.)
            .group_hover(group.clone(), |style| style.opacity(1.))
            .child(icon(Glyph::Send, 14.).text_color(cx.theme().muted_foreground))
    });
    let details = row
        .approval
        .filter(|_| expanded)
        .map(|auto_accept| device_details(row.id, row.name.clone(), auto_accept, cx));
    let action = if row.approval.is_some() {
        more_button(row.id, cx)
    } else {
        pair_button(row.id, row.addr.clone(), cx)
    };
    let shown = tile(&row.id, row.platform, selected, cx);
    let line = detail(&row, cx);
    let mut body = div()
        .id(id)
        .group(group)
        .flex()
        .items_center()
        .gap(px(10.))
        .p(px(7.))
        .rounded(px(RADIUS_CONTROL))
        .border_1()
        .border_color(transparent_black())
        .cursor_pointer()
        .hover(move |style| style.bg(hover))
        .drag_over::<ExternalPaths>(move |style, _, _, _| style.border_color(highlight).bg(hover))
        .on_drop(cx.listener(move |view, paths: &ExternalPaths, _, cx| {
            cx.stop_propagation();
            let target = Target { addr: drop_target.addr.clone(), name: drop_target.name.clone() };
            view.send_now(vec![target], paths.paths().to_vec(), cx);
        }))
        .on_click(cx.listener(move |view, _, _, cx| {
            let target = Target { addr: target.addr.clone(), name: target.name.clone() };
            view.activate_device(device, target, cx);
        }))
        .child(shown)
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .child(ui::text(row.name).truncate())
                .child(line),
        )
        .children(hint)
        .child(action);
    if selected || expanded {
        body = body.bg(hover);
    }
    div().flex().flex_col().child(body).children(details).into_any_element()
}

fn more_button(id: EndpointId, cx: &mut Context<MainView>) -> AnyElement {
    Button::new(SharedString::from(format!("more-{id}")))
        .ghost()
        .xs()
        .icon(Glyph::More)
        .tooltip("Options")
        .on_click(cx.listener(move |view, _, _: &mut Window, cx| {
            cx.stop_propagation();
            view.expanded = if view.expanded == Some(id) { None } else { Some(id) };
            cx.notify();
        }))
        .into_any_element()
}

fn device_details(
    id: EndpointId,
    name: String,
    auto_accept: bool,
    cx: &mut Context<MainView>,
) -> AnyElement {
    let line = || div().flex().items_center().justify_between().h(px(36.)).pl(px(50.)).pr(px(7.));
    div()
        .flex()
        .flex_col()
        .child(
            line().child(ui::muted("Key", cx)).child(ui::mono(ui::fingerprint(&id), cx).pr(px(4.))),
        )
        .child(
            line().child(ui::muted("Receive without asking", cx)).child(
                Switch::new(SharedString::from(format!("approval-{id}")))
                    .checked(auto_accept)
                    .accessibility_label("Receive without asking")
                    .on_click(cx.listener(move |_, enabled: &bool, _, cx| {
                        MainView::command(Command::SetAutoAccept { id, enabled: *enabled }, cx);
                    })),
            ),
        )
        .child(
            line().child(ui::muted("Remove this device", cx)).child(
                Button::new(SharedString::from(format!("forget-{id}")))
                    .xs()
                    .icon(Glyph::Delete)
                    .label("Forget")
                    .on_click(cx.listener(move |view, _, _, cx| {
                        view.dialog = Some(Dialog::Forget { id, name: name.clone() });
                        cx.notify();
                    })),
            ),
        )
        .into_any_element()
}

fn pair_button(id: EndpointId, addr: EndpointAddr, cx: &mut Context<MainView>) -> AnyElement {
    Button::new(SharedString::from(format!("pair-{id}")))
        .xs()
        .label("Pair")
        .on_click(cx.listener(move |view, _, _: &mut Window, cx| {
            cx.stop_propagation();
            let auto_accept = !view.state.read(cx).ask_after_pairing;
            MainView::command(Command::PairNearby { addr: addr.clone(), auto_accept }, cx);
        }))
        .into_any_element()
}
