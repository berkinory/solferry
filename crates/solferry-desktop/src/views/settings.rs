use gpui_kit::component::Sizable;
use gpui_kit::component::input::Input;
use gpui_kit::component::switch::Switch;
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement, PathPromptOptions, SharedString, Styled,
    Window, div, px,
};
use iroh::EndpointId;
use rust_i18n::t;
use solferry_core::{BlockedDevice, Config, Visibility, update};

use super::button::Button;
use super::main_view::MainView;
use super::{language, relays, ui, updates};
use crate::assets::Glyph;
use crate::login;
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
        return ui::muted(t!("settings.starting"), cx).into_any_element();
    };
    let blocked = state.blocked.clone();
    let status = state.updater.clone();
    let id = state.id;
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .child(device_section(view, &config, cx))
        .children(super::access::card(&view.state, cx))
        .child(relays::section(view, &config, cx))
        .child(visibility_section(&config, cx))
        .child(tray_section(&config, cx))
        .child(login_section(cx))
        .child(language::section(&config, cx))
        .child(super::appearance::section(&config, cx))
        .children(update::enabled().then(|| updates::section(config.check_updates, &status, cx)))
        .children(blocked_section(&blocked, cx))
        .child(about(id, cx))
        .into_any_element()
}

fn device_section(view: &MainView, config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    ui::card(Glyph::Computer, &t!("settings.device.title"), None, cx)
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
                            .child(ui::muted(t!("settings.device.save_to"), cx))
                            .child(ui::text(config.download_dir.display().to_string()).truncate()),
                    )
                    .child(
                        Button::new("choose-folder")
                            .xs()
                            .icon(Glyph::Folder)
                            .label(t!("settings.device.change"))
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
        prompt: Some(t!("settings.device.choose").into()),
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
    let detail =
        if found { t!("settings.visibility.visible") } else { t!("settings.visibility.hidden") };
    let title = t!("settings.visibility.title");
    ui::card(Glyph::View, &title, None, cx)
        .child(
            ui::padded_well(cx).child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(SPACE_MD))
                    .child(ui::muted(detail, cx).flex_1().min_w_0())
                    .child(
                        Switch::new("findable")
                            .large()
                            .checked(found)
                            .accessibility_label(title)
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

fn tray_section(config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    let detail = match (config.show_in_tray, cfg!(target_os = "macos")) {
        (true, true) => t!("settings.tray.shown_macos"),
        (true, false) => t!("settings.tray.shown"),
        (false, true) => t!("settings.tray.hidden_macos"),
        (false, false) => t!("settings.tray.hidden"),
    };
    let label = if cfg!(target_os = "macos") {
        t!("settings.tray.title_macos")
    } else {
        t!("settings.tray.title")
    };
    ui::card(Glyph::Computer, &label, None, cx)
        .child(
            ui::padded_well(cx).child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(SPACE_MD))
                    .child(ui::muted(detail, cx).flex_1().min_w_0())
                    .child(
                        Switch::new("tray")
                            .large()
                            .checked(config.show_in_tray)
                            .accessibility_label(label)
                            .on_click(cx.listener(|view, on: &bool, _, cx| {
                                view.set_show_in_tray(*on, cx);
                            })),
                    ),
            ),
        )
        .into_any_element()
}

/// Asks the system each time, since its login entry is the only record and the user can remove it.
fn login_section(cx: &mut Context<MainView>) -> AnyElement {
    let on = login::enabled();
    let detail = if on { t!("settings.login.on") } else { t!("settings.login.off") };
    let title = t!("settings.login.title");
    ui::card(Glyph::Computer, &title, None, cx)
        .child(
            ui::padded_well(cx).child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(SPACE_MD))
                    .child(ui::muted(detail, cx).flex_1().min_w_0())
                    .child(
                        Switch::new("login")
                            .large()
                            .checked(on)
                            .accessibility_label(title)
                            .on_click(cx.listener(|_, on: &bool, _, cx| {
                                login::set(*on);
                                cx.notify();
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
        ui::card(
            Glyph::Blocked,
            &t!("settings.blocked.title"),
            Some(ui::count(blocked.len(), cx)),
            cx,
        )
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
        .gap(px(10.))
        .p(px(6.))
        .child(ui::tile(Glyph::Blocked, cx))
        .child(who)
        .child(
            Button::new(SharedString::from(format!("unblock-{id}")))
                .xs()
                .label(t!("settings.blocked.unblock"))
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
        .child(ui::caption(t!("settings.version", version = env!("CARGO_PKG_VERSION")), cx))
        .child(ui::caption(short, cx))
        .into_any_element()
}
