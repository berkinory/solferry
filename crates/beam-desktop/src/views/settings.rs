use beam_core::{Config, RelayLink, Visibility};
use gpui_kit::component::Sizable;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::{
    AnyElement, Context, IntoElement, ParentElement, PathPromptOptions, SharedString, Styled,
    Window, div,
};

use iroh::EndpointId;

use super::main_view::{MainView, Page};
use super::ui;
use crate::errors::explain;
use crate::service::Command;

const VISIBILITY: [(Visibility, &str); 3] = [
    (Visibility::Everyone, "Everyone nearby"),
    (Visibility::KnownDevices, "Paired devices"),
    (Visibility::Hidden, "Hidden"),
];

pub fn load_inputs(view: &MainView, window: &mut Window, cx: &mut Context<MainView>) {
    let name = view.state.read(cx).config.as_ref().map(|config| config.name.clone());
    if let Some(name) = name {
        view.name_input.update(cx, |input, cx| input.set_value(name, window, cx));
    }
}

fn change(view: &MainView, cx: &mut Context<MainView>, edit: impl FnOnce(&mut Config)) {
    let Some(mut config) = view.state.read(cx).config.clone() else { return };
    edit(&mut config);
    MainView::command(Command::Apply(config), cx);
}

pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let Some(config) = view.state.read(cx).config.clone() else {
        return ui::muted("Starting...", cx);
    };
    let blocked = view.state.read(cx).blocked.clone();
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(name_section(view, cx))
        .child(folder_section(&config, cx))
        .child(relay_section(view, &config, cx))
        .child(visibility_section(&config, cx))
        .children(blocked_section(&blocked, cx))
        .into_any_element()
}

fn name_section(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    ui::card(cx)
        .child(ui::section_title("Device name", cx))
        .child(Input::new(&view.name_input))
        .child(Button::new("save-name").small().label("Save").on_click(cx.listener(
            |view, _, _, cx| {
                let name = view.name_input.read(cx).value().trim().to_owned();
                if !name.is_empty() {
                    change(view, cx, |config| config.name = name);
                    view.page = Page::Home;
                    cx.notify();
                }
            },
        )))
        .into_any_element()
}

fn folder_section(config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    ui::card(cx)
        .child(ui::section_title("Save files to", cx))
        .child(ui::muted(config.download_dir.display().to_string(), cx))
        .child(
            Button::new("choose-folder")
                .small()
                .label("Change")
                .on_click(cx.listener(|_, _, _, cx| choose_folder(cx))),
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

fn relay_section(view: &MainView, config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    let relays: Vec<_> = config.relays.iter().map(|url| relay_row(url, cx)).collect();
    let failure = view.state.read(cx).relay_error.as_ref().map(explain);
    let failure = failure.map(|explanation| ui::muted(explanation.next, cx));
    ui::card(cx)
        .child(ui::section_title("Your relays", cx))
        .child(ui::muted("Reach devices on other networks through a relay you run.", cx))
        .children(relays)
        .children(failure)
        .child(Input::new(&view.relay_input))
        .child(
            Button::new("add-relay")
                .small()
                .label("Add")
                .on_click(cx.listener(|view, _, window, cx| add_relay(view, window, cx))),
        )
        .into_any_element()
}

fn relay_row(url: &str, cx: &mut Context<MainView>) -> AnyElement {
    let removed = url.to_owned();
    let parsed = RelayLink::parse(url);
    let shown = parsed.map_or_else(|_| url.to_owned(), |link| link.url().to_string());
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(ui::muted(shown, cx))
        .child(
            Button::new(SharedString::from(format!("remove-{url}")))
                .ghost()
                .xsmall()
                .label("Remove")
                .on_click(cx.listener(move |view, _, _, cx| {
                    let removed = removed.clone();
                    change(view, cx, |config| config.relays.retain(|url| *url != removed));
                })),
        )
        .into_any_element()
}

fn add_relay(view: &MainView, window: &mut Window, cx: &mut Context<MainView>) {
    let url = view.relay_input.read(cx).value().trim().to_owned();
    if url.is_empty() {
        return;
    }
    if let Err(error) = RelayLink::parse(&url) {
        view.state.update(cx, |state, cx| {
            state.relay_error = Some(error);
            cx.notify();
        });
        return;
    }
    view.state.update(cx, |state, _| state.relay_error = None);
    change(view, cx, |config| {
        config.relays.retain(|existing| *existing != url);
        config.relays.push(url);
    });
    view.relay_input.update(cx, |input, cx| input.set_value("", window, cx));
}

fn visibility_section(config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    let options: Vec<_> = VISIBILITY
        .iter()
        .map(|&(mode, title)| {
            let button = Button::new(SharedString::from(title)).small().label(title);
            let button = if config.visibility == mode { button.primary() } else { button.ghost() };
            button.on_click(cx.listener(move |view, _, _, cx| {
                change(view, cx, |config| config.visibility = mode);
            }))
        })
        .collect();
    ui::card(cx)
        .child(ui::section_title("Who can see this device", cx))
        .child(div().flex().gap_2().children(options))
        .into_any_element()
}

fn blocked_section(blocked: &[EndpointId], cx: &mut Context<MainView>) -> Option<AnyElement> {
    if blocked.is_empty() {
        return None;
    }
    let rows: Vec<_> = blocked.iter().map(|&id| blocked_row(id, cx)).collect();
    Some(ui::card(cx).child(ui::section_title("Blocked", cx)).children(rows).into_any_element())
}

fn blocked_row(id: EndpointId, cx: &mut Context<MainView>) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(ui::muted(id.fmt_short().to_string(), cx))
        .child(
            Button::new(SharedString::from(format!("unblock-{id}")))
                .ghost()
                .xsmall()
                .label("Unblock")
                .on_click(cx.listener(move |_, _, _, cx| {
                    MainView::command(Command::Unblock(id), cx);
                })),
        )
        .into_any_element()
}
