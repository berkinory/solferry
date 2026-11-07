use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use gpui_kit::component::Sizable;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::InputState;
use gpui_kit::component::{ActiveTheme, Icon, IconName, TitleBar};
use gpui_kit::{
    AnyElement, AppContext, Context, Entity, ExternalPaths, FontWeight, InteractiveElement, IntoElement, ParentElement,
    PathPromptOptions, Render, StatefulInteractiveElement, Styled, Window, div, px,
};
use iroh::EndpointId;

use super::{devices, offers, pairing, settings, transfers, ui};
use crate::errors::explain;
use crate::service::{Command, Target};
use crate::shell::Shell;
use crate::state::AppState;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    Pair,
    Settings,
}

pub struct MainView {
    pub state: Entity<AppState>,
    pub page: Page,
    pub pending: Vec<PathBuf>,
    pub selected: HashSet<EndpointId>,
    targets: HashMap<EndpointId, Target>,
    pub code_input: Entity<InputState>,
    pub name_input: Entity<InputState>,
    pub relay_input: Entity<InputState>,
}

impl MainView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = cx.global::<Shell>().state.clone();
        cx.observe(&state, |_, _, cx| cx.notify()).detach();
        Self {
            state,
            page: Page::Home,
            pending: Vec::new(),
            selected: HashSet::new(),
            targets: HashMap::new(),
            code_input: cx.new(|cx| InputState::new(window, cx).placeholder("Paste a beam1 code")),
            name_input: cx.new(|cx| InputState::new(window, cx).placeholder("This device's name")),
            relay_input: cx
                .new(|cx| InputState::new(window, cx).placeholder("https://relay.example.com")),
        }
    }

    pub fn command(&self, command: Command, cx: &Context<Self>) {
        cx.global::<Shell>().service.send(command);
    }

    pub fn open_page(&mut self, page: Page, window: &mut Window, cx: &mut Context<Self>) {
        self.page = page;
        if page == Page::Settings {
            settings::load_inputs(self, window, cx);
        }
        cx.notify();
    }

    pub fn send_now(&mut self, targets: Vec<Target>, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        self.clear_pending(cx);
        self.command(Command::Send { targets, paths }, cx);
    }

    pub fn activate_device(&mut self, id: EndpointId, target: Target, cx: &mut Context<Self>) {
        if self.pending.is_empty() {
            self.pick_files_for(target, cx);
        } else {
            if !self.selected.remove(&id) {
                self.selected.insert(id);
                self.targets.insert(id, target);
            }
            cx.notify();
        }
    }

    fn pick_files_for(&self, target: Target, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(file_picker());
        cx.spawn(async move |view, cx| {
            let Ok(Ok(Some(paths))) = chosen.await else { return };
            let _ = view.update(cx, |view, cx| view.send_now(vec![target], paths, cx));
        })
        .detach();
    }

    fn pick_pending(&self, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(file_picker());
        cx.spawn(async move |view, cx| {
            let Ok(Ok(Some(paths))) = chosen.await else { return };
            let _ = view.update(cx, |view, cx| {
                view.pending = paths;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn send_selected(&mut self, cx: &mut Context<Self>) {
        let chosen: Vec<EndpointId> = self.selected.iter().copied().collect();
        let targets = chosen.iter().filter_map(|id| self.targets.remove(id)).collect();
        let paths = std::mem::take(&mut self.pending);
        self.send_now(targets, paths, cx);
    }

    pub fn clear_pending(&mut self, cx: &mut Context<Self>) {
        self.pending.clear();
        self.selected.clear();
        self.targets.clear();
        cx.notify();
    }

    pub fn retry(&mut self, index: usize, cx: &mut Context<Self>) {
        let failure = self.state.update(cx, |state, cx| {
            cx.notify();
            (index < state.failures.len()).then(|| state.failures.remove(index))
        });
        if let Some(failure) = failure {
            let target = Target { addr: failure.addr, name: failure.name };
            self.command(Command::Send { targets: vec![target], paths: failure.paths }, cx);
        }
    }

    pub fn dismiss(&mut self, index: usize, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            if index < state.failures.len() {
                state.failures.remove(index);
            }
            cx.notify();
        });
    }

    pub fn answer(&mut self, key: u64, accept: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.offers.retain(|offer| offer.key != key);
            cx.notify();
        });
        self.command(Command::Answer { offer: key, accept }, cx);
    }

    fn header(&self, cx: &mut Context<Self>) -> AnyElement {
        let home = self.page == Page::Home;
        let name = self.state.read(cx).config.as_ref().map(|config| config.name.clone());
        TitleBar::new().child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .pr_2()
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child("beam"))
                        .children(name.map(|name| ui::muted(name, cx))),
                )
                .child(
                    div()
                        .flex()
                        .gap_1()
                        .child(
                            Button::new("add-device")
                                .ghost()
                                .xsmall()
                                .icon(IconName::Plus)
                                .tooltip("Add device")
                                .on_click(cx.listener(|view, _, window, cx| {
                                    view.open_page(Page::Pair, window, cx);
                                })),
                        )
                        .child(
                            Button::new("open-settings")
                                .ghost()
                                .xsmall()
                                .icon(Icon::new(if home {
                                    IconName::Settings
                                } else {
                                    IconName::Close
                                }))
                                .tooltip(if home { "Settings" } else { "Back" })
                                .on_click(cx.listener(move |view, _, window, cx| {
                                    let next = if home { Page::Settings } else { Page::Home };
                                    view.open_page(next, window, cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
    }

    fn drop_zone(&self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let (border, hover, muted) = (theme.border, theme.accent, theme.muted_foreground);
        div()
            .id("drop-zone")
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_2()
            .h(px(132.))
            .rounded(px(14.))
            .border_2()
            .border_dashed()
            .border_color(border)
            .cursor_pointer()
            .hover(move |style| style.bg(hover))
            .on_click(cx.listener(|view, _, _, cx| view.pick_pending(cx)))
            .child(Icon::new(IconName::FolderOpen).size(px(26.)).text_color(muted))
            .child(div().text_sm().child("Drop files to send"))
            .child(ui::muted("or click to choose", cx))
            .into_any_element()
    }

    fn home(&self, cx: &mut Context<Self>) -> AnyElement {
        let state = self.state.read(cx).clone();
        let fatal = state.fatal.as_ref().map(|error| {
            let explanation = explain(error);
            ui::card(cx)
                .border_color(cx.theme().danger)
                .child(div().text_sm().child(explanation.title))
                .child(ui::muted(explanation.next, cx))
        });
        let offers: Vec<_> = state.offers.iter().map(|offer| offers::card(offer, cx)).collect();
        let pending = (!self.pending.is_empty()).then(|| devices::pending_bar(self, cx));
        let drop_zone = self.pending.is_empty().then(|| self.drop_zone(cx));
        div()
            .flex()
            .flex_col()
            .gap_3()
            .children(fatal)
            .children(offers)
            .children(drop_zone)
            .children(pending)
            .child(devices::render(&state, self, cx))
            .child(transfers::render(&state, cx))
            .into_any_element()
    }
}

fn file_picker() -> PathPromptOptions {
    PathPromptOptions {
        files: true,
        directories: true,
        multiple: true,
        prompt: Some("Send".into()),
    }
}

impl Render for MainView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let highlight = cx.theme().accent;
        let body = match self.page {
            Page::Home => self.home(cx),
            Page::Pair => pairing::render(self, cx),
            Page::Settings => settings::render(self, cx),
        };
        div()
            .id("main")
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family(cx.theme().font_family.clone())
            .drag_over::<ExternalPaths>(move |style, _, _, _| style.bg(highlight))
            .on_drop(cx.listener(|view, paths: &ExternalPaths, _, cx| {
                view.page = Page::Home;
                view.pending.extend_from_slice(paths.paths());
                cx.notify();
            }))
            .child(self.header(cx))
            .child(div().id("content").flex_1().overflow_y_scroll().p(px(14.)).child(body))
    }
}
