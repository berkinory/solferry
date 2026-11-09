use gpui_kit::component::Sizable;
use gpui_kit::component::input::Input;
use gpui_kit::component::switch::Switch;
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement, PathPromptOptions, SharedString, Styled,
    Window, div, px,
};
use iroh::EndpointId;
use solferry_core::{BlockedDevice, Config, Visibility};

use super::button::Button;
use super::main_view::MainView;
use super::{relays, ui};
use crate::assets::Glyph;
use crate::service::Command;
use crate::tokens::{SPACE_MD, SPACE_SM, SPACE_XS};

pub fn load_inputs(view: &MainView, window: &mut Window, cx: &mut Context<MainView>) {
    let name = view.state.read(cx).config.as_ref().map(|config| config.name.clone());
    if let Some(name) = name {
        view.name_input.update(cx, |input, cx| input.set_value(name, window, cx));
    }
}

pub fn change(view: &MainView, cx: &mut Context<MainView>, edit: impl FnOnce(&mut Config)) {
    let Some(mut config) = view.state.read(cx).config.clone() else { return };
    edit(&mut config);
    MainView::command(Command::Apply(config), cx);
}

pub fn commit_name(view: &MainView, cx: &mut Context<MainView>) {
    let name = view.name_input.read(cx).value().trim().to_owned();
    let unchanged = view.state.read(cx).config.as_ref().is_none_or(|config| config.name == name);
    if !name.is_empty() && !unchanged {
        change(view, cx, |config| config.name = name);
    }
}

pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let state = view.state.read(cx);
    let Some(config) = state.config.clone() else {
        return ui::muted("Starting...", cx).into_any_element();
    };
    let blocked = state.blocked.clone();
    let id = state.id;
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .child(device_section(view, &config, cx))
        .child(visibility_section(&config, cx))
        .child(relays::section(view, &config, cx))
        .children(blocked_section(&blocked, cx))
        .child(about(id, cx))
        .into_any_element()
}

fn device_section(view: &MainView, config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    ui::card(Glyph::Computer, "This device", None, cx)
        .child(
            ui::padded_well(cx).child(Input::new(&view.name_input).large()).child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(SPACE_SM))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .child(ui::muted("Save files to", cx))
                            .child(ui::text(config.download_dir.display().to_string()).truncate()),
                    )
                    .child(
                        Button::new("choose-folder")
                            .xs()
                            .icon(Glyph::Folder)
                            .label("Change")
                            .on_click(cx.listener(|_, _, _, cx| choose_folder(cx))),
                    ),
            ),
        )
        .into_any_element()
}

fn choose_folder(cx: &mut Context<MainView>) {
    let options = PathPromptOptions {
        files: false,
        directories: true,
        multiple: false,
        prompt: Some("Choose".into()),
    };
    let chosen = cx.prompt_for_paths(options);
    cx.spawn(async move |view, cx| {
        let Ok(Ok(Some(mut paths))) = chosen.await else { return };
        let Some(path) = paths.pop() else { return };
        let _ = view.update(cx, |view, cx| {
            change(view, cx, |config| config.download_dir = path);
        });
    })
    .detach();
}

fn visibility_section(config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    let found = config.visibility == Visibility::Everyone;
    let detail = if found {
        "Devices on your network can see this one and ask to send."
    } else {
        "Nobody new can find this device. Paired devices can still reach it."
    };
    ui::card(Glyph::View, "Let others find me", None, cx)
        .child(
            ui::padded_well(cx).child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(SPACE_MD))
                    .child(ui::muted(detail, cx).flex_1())
                    .child(
                        Switch::new("findable")
                            .large()
                            .checked(found)
                            .accessibility_label("Let others find me")
                            .on_click(cx.listener(move |view, on: &bool, _, cx| {
                                let chosen =
                                    if *on { Visibility::Everyone } else { Visibility::Hidden };
                                change(view, cx, |config| config.visibility = chosen);
                            })),
                    ),
            ),
        )
        .into_any_element()
}

fn blocked_section(blocked: &[BlockedDevice], cx: &mut Context<MainView>) -> Option<AnyElement> {
    if blocked.is_empty() {
        return None;
    }
    let rows: Vec<_> = blocked.iter().map(|device| blocked_row(device, cx)).collect();
    Some(
        ui::card(Glyph::Blocked, "Blocked", Some(ui::count(blocked.len(), cx)), cx)
            .child(ui::well(cx).children(rows))
            .into_any_element(),
    )
}

fn blocked_row(device: &BlockedDevice, cx: &mut Context<MainView>) -> AnyElement {
    let id = device.id;
    let short = id.fmt_short().to_string();
    let who = div()
        .flex()
        .flex_col()
        .flex_1()
        .min_w_0()
        .child(ui::text(device.name.clone().unwrap_or_else(|| short.clone())).truncate())
        .children(device.name.is_some().then(|| ui::caption(short, cx)));
    div()
        .flex()
        .items_center()
        .gap(px(14.))
        .p(px(9.))
        .child(ui::tile(Glyph::Blocked, cx))
        .child(who)
        .child(
            Button::new(SharedString::from(format!("unblock-{id}")))
                .xs()
                .label("Unblock")
                .on_click(cx.listener(move |_, _, _, cx| {
                    MainView::command(Command::Unblock(id), cx);
                })),
        )
        .into_any_element()
}

fn about(id: Option<EndpointId>, cx: &mut Context<MainView>) -> AnyElement {
    let short = id.map(|id| id.fmt_short().to_string()).unwrap_or_default();
    div()
        .flex()
        .justify_between()
        .px(px(SPACE_MD))
        .py(px(SPACE_XS))
        .child(ui::caption(format!("Solferry {}", env!("CARGO_PKG_VERSION")), cx))
        .child(ui::caption(short, cx))
        .into_any_element()
}
