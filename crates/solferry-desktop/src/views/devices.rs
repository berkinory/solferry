use gpui_kit::component::ActiveTheme;
use gpui_kit::component::switch::Switch;
use gpui_kit::{
    AnyElement, App, Context, ElementId, ExternalPaths, Hsla, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div, px,
    transparent_black,
};
use iroh::{EndpointAddr, EndpointId};
use solferry_core::{NearbyDevice, Platform};

use super::button::Button;
use super::dialog::Dialog;
use super::main_view::MainView;
use super::{radar, ui};
use crate::assets::Glyph;
use crate::format;
use crate::service::{Command, Target};
use crate::state::AppState;
use crate::theme::{color, palette};
use crate::tokens::{RADIUS_CONTROL, SPACE_SM, SPACE_XS};

pub enum Dot {
    Online,
    Offline,
}

pub struct Row {
    pub id: EndpointId,
    pub addr: EndpointAddr,
    pub name: String,
    pub platform: Option<Platform>,
    detail: &'static str,
    dot: Option<Dot>,
    approval: Option<bool>,
    paired_at_ms: u64,
    pub online: bool,
}

/// What a paired device's line says: where it was seen nearby, else whether it answered.
fn presence(
    found: Option<&NearbyDevice>,
    id: &EndpointId,
    state: &AppState,
) -> (&'static str, Option<Dot>) {
    if found.is_some() {
        return ("Online", Some(Dot::Online));
    }
    match &state.reachable {
        None => ("Paired", None),
        Some(reachable) if reachable.contains(id) => ("Online", Some(Dot::Online)),
        Some(_) => ("Offline", Some(Dot::Offline)),
    }
}

pub fn rows(state: &AppState) -> (Vec<Row>, Vec<Row>) {
    let near = |id: &EndpointId| state.nearby.iter().find(|device| device.addr.id == *id);
    let mine = state
        .devices
        .iter()
        .map(|device| {
            let found = near(&device.id);
            let (detail, dot) = presence(found, &device.id, state);
            let online = matches!(dot, Some(Dot::Online));
            Row {
                id: device.id,
                addr: found.map_or_else(|| device.addr(), |found| found.addr.clone()),
                name: device.name.clone(),
                platform: device.platform.or_else(|| found.and_then(|found| found.platform)),
                detail,
                dot,
                approval: Some(device.auto_accept),
                paired_at_ms: device.paired_at_ms,
                online,
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
            detail: "Not paired",
            dot: None,
            approval: None,
            paired_at_ms: 0,
            online: true,
        })
        .collect();
    (mine, others)
}

pub fn render(state: &AppState, view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let (mine, others) = rows(state);
    if mine.is_empty() && others.is_empty() {
        return radar::hero(cx);
    }
    let paired = (!mine.is_empty()).then(|| {
        let count = ui::count(mine.len(), cx);
        let elements: Vec<_> = mine.into_iter().map(|row| device_row(row, view, cx)).collect();
        ui::card(Glyph::Devices, "Devices", Some(count), cx).child(ui::well(cx).children(elements))
    });
    let found: Vec<_> = others.into_iter().map(|row| device_row(row, view, cx)).collect();
    let well = if found.is_empty() {
        ui::well(cx).child(radar::scanning(cx))
    } else {
        ui::well(cx).children(found)
    };
    let code = radar::use_code("Use a code", cx);
    let nearby = ui::card(Glyph::Nearby, "Nearby", Some(code), cx).child(well);
    div().flex().flex_col().gap(px(SPACE_SM)).children(paired).child(nearby).into_any_element()
}

pub fn tile(platform: Option<Platform>, online: bool, selected: bool, cx: &App) -> AnyElement {
    if selected {
        let palette = palette(cx);
        ui::tinted_tile(Glyph::Check, color(palette.primary_fill), color(palette.on_primary))
            .into_any_element()
    } else {
        ui::device_tile(platform, online, cx).into_any_element()
    }
}

pub fn detail(row: &Row, cx: &App) -> AnyElement {
    let tone: Option<Hsla> = row.dot.as_ref().map(|dot| match dot {
        Dot::Online => color(palette(cx).success),
        Dot::Offline => cx.theme().muted_foreground.opacity(0.5),
    });
    tone.map_or_else(
        || ui::muted(row.detail, cx).into_any_element(),
        |tone| ui::status(tone, row.detail, cx).into_any_element(),
    )
}

fn device_row(row: Row, view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let expanded = view.expanded == Some(row.id);
    let (hover, highlight) = (cx.theme().accent, color(palette(cx).primary));
    let group = SharedString::from(format!("row-{}", row.id));
    let id: ElementId = group.clone().into();
    let target = Target { addr: row.addr.clone(), name: row.name.clone() };
    let drop_target = Target { addr: row.addr.clone(), name: row.name.clone() };
    let details = row
        .approval
        .filter(|_| expanded)
        .map(|auto_accept| device_details(&row, auto_accept, view, cx));
    let share = row.approval.is_some().then(|| share_button(&row, view, cx));
    let action = if row.approval.is_some() {
        more_button(row.id, cx)
    } else {
        pair_button(row.id, row.addr.clone(), cx)
    };
    let shown = tile(row.platform, row.online, false, cx);
    let line = detail(&row, cx);
    let mut body = div()
        .id(id)
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
        .on_click(cx.listener(move |_, _, _, cx| {
            let target = Target { addr: target.addr.clone(), name: target.name.clone() };
            MainView::pick_files_for(target, cx);
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
        .children(share)
        .child(action);
    if expanded {
        body = body.bg(hover);
    }
    div().flex().flex_col().child(body).children(details).into_any_element()
}

fn share_button(row: &Row, view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let id = row.id;
    let target = Target { addr: row.addr.clone(), name: row.name.clone() };
    let shared = view.shared_with == Some(id);
    Button::new(SharedString::from(format!("share-{id}")))
        .ghost()
        .xs()
        .icon(if shared { Glyph::Check } else { Glyph::Clipboard })
        .tooltip(if shared { "Clipboard sent" } else { "Send clipboard" })
        .on_click(cx.listener(move |view, _, _: &mut Window, cx| {
            cx.stop_propagation();
            let target = Target { addr: target.addr.clone(), name: target.name.clone() };
            view.share_clipboard(id, target, cx);
        }))
        .into_any_element()
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
    row: &Row,
    auto_accept: bool,
    view: &MainView,
    cx: &mut Context<MainView>,
) -> AnyElement {
    let (id, name) = (row.id, row.name.clone());
    let last = view
        .state
        .read(cx)
        .history
        .iter()
        .find(|past| past.peer == id && past.completed)
        .map(|past| past.finished_ms);
    let (what, moment) = last.map_or_else(
        || ("Paired", format::ago(row.paired_at_ms)),
        |ms| ("Last transfer", format::ago(ms)),
    );
    let when = div()
        .flex()
        .gap(px(SPACE_XS))
        .child(ui::muted(what, cx))
        .child(ui::muted(moment, cx).opacity(0.6));
    let line = || div().flex().items_center().justify_between().h(px(36.)).pl(px(50.)).pr(px(7.));
    div()
        .flex()
        .flex_col()
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
            line().child(when).child(
                Button::new(SharedString::from(format!("forget-{id}")))
                    .danger()
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
        .icon(Glyph::Link)
        .label("Pair")
        .on_click(cx.listener(move |_, _, _: &mut Window, cx| {
            cx.stop_propagation();
            MainView::command(Command::PairNearby { addr: addr.clone() }, cx);
        }))
        .into_any_element()
}
