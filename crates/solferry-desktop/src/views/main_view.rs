use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use gpui_kit::component::Sizable;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::InputState;
use gpui_kit::component::{ActiveTheme, Icon, IconName, Theme, TitleBar};
use gpui_kit::{
    AnyElement, AppContext, Context, Entity, ExternalPaths, FontWeight, InteractiveElement,
    IntoElement, ParentElement, PathPromptOptions, Render, StatefulInteractiveElement, Styled,
    Window, div, px,
};
use iroh::EndpointId;
use solferry_core::TransferId;

use super::{devices, offers, pairing, settings, transfers, ui};
use crate::errors::explain;
use crate::service::{Command, Target};
use crate::shell::Shell;
use crate::state::AppState;
use crate::tokens::{RADIUS_LG, SPACE_LG};

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
    pub expanded: Option<EndpointId>,
    targets: HashMap<EndpointId, Target>,
    pub code_input: Entity<InputState>,
    pub name_input: Entity<InputState>,
    pub relay_input: Entity<InputState>,
}

impl MainView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = cx.global::<Shell>().state.clone();
        cx.observe_window_appearance(window, |_, window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
        })
        .detach();
        let mut paired_count = state.read(cx).paired_count;
        cx.observe(&state, move |view, state, cx| {
            let latest = state.read(cx).paired_count;
            if latest != paired_count {
                paired_count = latest;
                view.page = Page::Home;
            }
            cx.notify();
        })
        .detach();
        Self {
            state,
            page: Page::Home,
            pending: Vec::new(),
            selected: HashSet::new(),
            expanded: None,
            targets: HashMap::new(),
            code_input: cx.new(|cx| InputState::new(window, cx).placeholder("Paste a sf1 code")),
            name_input: cx.new(|cx| InputState::new(window, cx).placeholder("This device's name")),
            relay_input: cx
                .new(|cx| InputState::new(window, cx).placeholder("https://relay.example.com")),
        }
    }

    pub fn command(command: Command, cx: &Context<Self>) {
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
        Self::command(Command::Send { targets, paths }, cx);
    }

    pub fn activate_device(&mut self, id: EndpointId, target: Target, cx: &mut Context<Self>) {
        if self.pending.is_empty() {
            Self::pick_files_for(target, cx);
        } else {
            if !self.selected.remove(&id) {
                self.selected.insert(id);
                self.targets.insert(id, target);
            }
            cx.notify();
        }
    }

    fn pick_files_for(target: Target, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(file_picker());
        cx.spawn(async move |view, cx| {
            let Ok(Ok(Some(paths))) = chosen.await else { return };
            let _ = view.update(cx, |view, cx| view.send_now(vec![target], paths, cx));
        })
        .detach();
    }

    fn pick_pending(cx: &mut Context<Self>) {
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
            Self::command(Command::Send { targets: vec![target], paths: failure.paths }, cx);
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

    pub fn clear(&mut self, id: TransferId, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.transfers.retain(|transfer| transfer.id != id);
            cx.notify();
        });
    }

    pub fn show_downloads(&self, cx: &mut Context<Self>) {
        if let Some(config) = &self.state.read(cx).config {
            cx.reveal_path(&config.download_dir);
        }
    }

    pub fn answer(&mut self, key: u64, accept: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.offers.retain(|offer| offer.key != key);
            cx.notify();
        });
        Self::command(Command::Answer { offer: key, accept }, cx);
    }

    pub fn answer_pairing(&mut self, key: u64, accept: bool, cx: &mut Context<Self>) {
        let auto_accept = !self.state.read(cx).ask_after_pairing;
        self.state.update(cx, |state, cx| {
            state.pairing_prompts.retain(|prompt| prompt.key != key);
            cx.notify();
        });
        Self::command(Command::AnswerPairing { key, accept, auto_accept }, cx);
    }

    pub fn block_offer(&mut self, key: u64, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.offers.retain(|offer| offer.key != key);
            cx.notify();
        });
        Self::command(Command::BlockOffer(key), cx);
    }

    fn header(&self, cx: &mut Context<Self>) -> AnyElement {
        let name = self.state.read(cx).config.as_ref().map(|config| config.name.clone());
        TitleBar::new()
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().text_sm().font_weight(FontWeight::SEMIBOLD).child("solferry"))
                    .children(name.map(|name| ui::muted(name, cx))),
            )
            .into_any_element()
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> AnyElement {
        let home = self.page == Page::Home;
        let leading = if home {
            Button::new("add-device")
                .ghost()
                .small()
                .icon(IconName::Plus)
                .label("Add device")
                .on_click(cx.listener(|view, _, window, cx| {
                    view.open_page(Page::Pair, window, cx);
                }))
        } else {
            Button::new("go-home").ghost().small().icon(IconName::ArrowLeft).label("Back").on_click(
                cx.listener(|view, _, window, cx| {
                    view.open_page(Page::Home, window, cx);
                }),
            )
        };
        div()
            .flex()
            .items_center()
            .justify_between()
            .px(px(10.))
            .pt(px(8.))
            .child(leading)
            .children(home.then(|| {
                Button::new("open-settings")
                    .ghost()
                    .small()
                    .icon(Icon::new(IconName::Settings))
                    .accessibility_label("Settings")
                    .on_click(cx.listener(|view, _, window, cx| {
                        view.open_page(Page::Settings, window, cx);
                    }))
            }))
            .into_any_element()
    }

    fn drop_zone(cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme();
        let (surface, line, hover) = (theme.secondary, theme.border, theme.accent);
        let (sun, on_sun) = (theme.primary, theme.primary_foreground);
        div()
            .id("drop-zone")
            .flex()
            .items_center()
            .gap(px(SPACE_LG))
            .p(px(SPACE_LG))
            .rounded(px(RADIUS_LG))
            .bg(surface)
            .border_1()
            .border_color(line)
            .cursor_pointer()
            .hover(move |style| style.bg(hover))
            .drag_over::<ExternalPaths>(move |style, _, _, _| style.border_color(sun).bg(hover))
            .on_click(cx.listener(|_, _, _, cx| Self::pick_pending(cx)))
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .size(px(44.))
                    .rounded_full()
                    .bg(sun)
                    .child(Icon::new(IconName::ArrowUp).size(px(22.)).text_color(on_sun)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Drop files to send"),
                    )
                    .child(ui::muted("or click to choose", cx)),
            )
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
        let prompts: Vec<_> =
            state.pairing_prompts.iter().map(|prompt| offers::pairing_card(prompt, cx)).collect();
        let code = state.pairing_code.as_deref().map(|code| offers::pairing_code(code, cx));
        let pending = (!self.pending.is_empty()).then(|| devices::pending_bar(self, cx));
        let drop_zone = self.pending.is_empty().then(|| Self::drop_zone(cx));
        div()
            .flex()
            .flex_col()
            .gap_3()
            .children(fatal)
            .children(prompts)
            .children(code)
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
            .child(self.toolbar(cx))
            .child(div().id("content").flex_1().overflow_y_scroll().p(px(14.)).child(body))
    }
}
