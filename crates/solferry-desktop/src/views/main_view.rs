use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::{ActiveTheme, Theme, TitleBar};
use gpui_kit::{
    AnyElement, AppContext, Context, Entity, ExternalPaths, FocusHandle, Focusable, FontWeight,
    InteractiveElement, IntoElement, ParentElement, PathPromptOptions, Render, Styled, Window, div,
    px, svg,
};
use iroh::EndpointId;
use solferry_core::TransferId;

use super::dialog::{self, Dialog};
use super::keys::{self, AddDevice, Back, ChooseFiles, OpenSettings};
use super::{history, home, nav, pairing, relays, send, settings};
use crate::assets::MARK;
use crate::service::{Command, Target};
use crate::shell::Shell;
use crate::state::AppState;
use crate::theme::{color, palette};
use crate::tokens::{SPACE_SM, TYPE_LABEL};

pub const COPIED_FOR: Duration = Duration::from_millis(1500);
const LAUNCH_FOR: Duration = Duration::from_millis(420);
const SENT_FOR: Duration = Duration::from_millis(1000);
const CONTENT_WIDTH: f32 = 560.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SendPhase {
    Idle,
    Launching,
    Sent,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    History,
    Pair,
    Send,
    Settings,
}

pub struct MainView {
    pub state: Entity<AppState>,
    pub page: Page,
    pub pending: Vec<PathBuf>,
    pub selected: HashSet<EndpointId>,
    pub expanded: Option<EndpointId>,
    pub dialog: Option<Dialog>,
    pub send_phase: SendPhase,
    pub nav_from: Page,
    pub nav_moves: usize,
    pub copied: bool,
    pub shared_with: Option<EndpointId>,
    targets: HashMap<EndpointId, Target>,
    pub code_input: Entity<InputState>,
    pub name_input: Entity<InputState>,
    pub relay_input: Entity<InputState>,
    focus: FocusHandle,
}

impl MainView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let state = cx.global::<Shell>().state.clone();
        cx.observe_window_appearance(window, |_, window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
        })
        .detach();
        let mut paired_count = state.read(cx).paired_count;
        let mut asking = state.read(cx).asking();
        cx.observe(&state, move |view, state, cx| {
            let (latest, waiting) = (state.read(cx).paired_count, state.read(cx).asking());
            if (latest != paired_count || waiting > asking) && view.page != Page::Send {
                view.page = Page::Home;
            }
            (paired_count, asking) = (latest, waiting);
            cx.notify();
        })
        .detach();
        let code_input = cx.new(|cx| InputState::new(window, cx).placeholder("Paste a sf1 code"));
        let name_input = cx.new(|cx| InputState::new(window, cx).placeholder("This device's name"));
        let relay_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("https://relay.example.com#token"));
        cx.subscribe_in(&code_input, window, |view, _, event, _, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                pairing::join(view, cx);
            }
        })
        .detach();
        cx.subscribe_in(&name_input, window, |view, _, event, _, cx| {
            if matches!(event, InputEvent::PressEnter { .. } | InputEvent::Blur) {
                settings::commit_name(view, cx);
            }
        })
        .detach();
        cx.subscribe_in(&relay_input, window, |view, _, event, window, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                relays::add(view, window, cx);
            }
        })
        .detach();
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self {
            state,
            page: Page::Home,
            pending: Vec::new(),
            selected: HashSet::new(),
            expanded: None,
            dialog: None,
            send_phase: SendPhase::Idle,
            nav_from: Page::Home,
            nav_moves: 0,
            copied: false,
            shared_with: None,
            targets: HashMap::new(),
            code_input,
            name_input,
            relay_input,
            focus,
        }
    }

    pub fn command(command: Command, cx: &Context<Self>) {
        cx.global::<Shell>().service.send(command);
    }

    pub fn open_page(&mut self, page: Page, window: &mut Window, cx: &mut Context<Self>) {
        if page != self.page {
            self.nav_from = self.page;
            self.nav_moves += 1;
        }
        self.page = page;
        if page == Page::Settings {
            settings::load_inputs(self, window, cx);
        }
        self.focus.focus(window, cx);
        cx.notify();
    }

    pub fn leave_page(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.page == Page::Send {
            self.clear_pending(cx);
        } else {
            self.open_page(Page::Home, window, cx);
        }
    }

    fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.dialog.take().is_some() {
            cx.notify();
        } else if self.page != Page::Home {
            self.leave_page(window, cx);
        }
    }

    pub fn send_now(&mut self, targets: Vec<Target>, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        self.clear_pending(cx);
        Self::command(Command::Send { targets, paths }, cx);
    }

    pub fn toggle_target(&mut self, id: EndpointId, target: Target, cx: &mut Context<Self>) {
        if !self.selected.remove(&id) {
            self.selected.insert(id);
            self.targets.insert(id, target);
        }
        cx.notify();
    }

    pub fn pick_files_for(target: Target, cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(file_picker());
        cx.spawn(async move |view, cx| {
            let Ok(Ok(Some(paths))) = chosen.await else { return };
            let _ = view.update(cx, |view, cx| view.send_now(vec![target], paths, cx));
        })
        .detach();
    }

    pub fn pick_pending(cx: &mut Context<Self>) {
        let chosen = cx.prompt_for_paths(file_picker());
        cx.spawn(async move |view, cx| {
            let Ok(Ok(Some(paths))) = chosen.await else { return };
            let _ = view.update(cx, |view, cx| view.stage(paths, cx));
        })
        .detach();
    }

    pub fn stage(&mut self, paths: Vec<PathBuf>, cx: &mut Context<Self>) {
        self.pending.extend(paths);
        self.page = Page::Send;
        cx.notify();
    }

    /// Hands the files over at once, then lets the button finish its animation before leaving.
    pub fn send_selected(&mut self, cx: &mut Context<Self>) {
        if self.send_phase != SendPhase::Idle {
            return;
        }
        let chosen: Vec<EndpointId> = self.selected.iter().copied().collect();
        let targets = chosen.iter().filter_map(|id| self.targets.remove(id)).collect();
        Self::command(Command::Send { targets, paths: self.pending.clone() }, cx);
        self.send_phase = SendPhase::Launching;
        cx.notify();
        cx.spawn(async move |view, cx| {
            cx.background_executor().timer(LAUNCH_FOR).await;
            let _ = view.update(cx, |view, cx| {
                view.send_phase = SendPhase::Sent;
                cx.notify();
            });
            cx.background_executor().timer(SENT_FOR).await;
            let _ = view.update(cx, |view, cx| {
                view.send_phase = SendPhase::Idle;
                view.clear_pending(cx);
            });
        })
        .detach();
    }

    pub fn clear_pending(&mut self, cx: &mut Context<Self>) {
        self.pending.clear();
        self.selected.clear();
        self.targets.clear();
        if self.page == Page::Send {
            self.page = Page::Home;
        }
        cx.notify();
    }

    pub fn remove_pending(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.pending.len() {
            self.pending.remove(index);
        }
        if self.pending.is_empty() {
            self.clear_pending(cx);
        }
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

    pub fn clear_finished(&mut self, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.transfers.retain(crate::state::Transfer::running);
            cx.notify();
        });
    }

    pub fn show_downloads(&self, cx: &mut Context<Self>) {
        if let Some(config) = &self.state.read(cx).config {
            cx.reveal_path(&config.download_dir);
        }
    }

    pub fn show_copied(&mut self, cx: &mut Context<Self>) {
        self.copied = true;
        cx.notify();
        cx.spawn(async move |view, cx| {
            cx.background_executor().timer(COPIED_FOR).await;
            let _ = view.update(cx, |view, cx| {
                view.copied = false;
                cx.notify();
            });
        })
        .detach();
    }

    pub fn answer(&mut self, key: u64, accept: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.offers.retain(|offer| offer.key != key);
            cx.notify();
        });
        Self::command(Command::Answer { offer: key, accept }, cx);
    }

    pub fn accept_always(&mut self, key: u64, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.offers.retain(|offer| offer.key != key);
            cx.notify();
        });
        Self::command(Command::AcceptAlways(key), cx);
    }

    pub fn answer_pairing(&mut self, key: u64, accept: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.pairing_prompts.retain(|prompt| prompt.key != key);
            cx.notify();
        });
        Self::command(Command::AnswerPairing { key, accept }, cx);
    }

    pub fn block_offer(&mut self, key: u64, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.offers.retain(|offer| offer.key != key);
            cx.notify();
        });
        Self::command(Command::BlockOffer(key), cx);
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

impl Focusable for MainView {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for MainView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let overlay = self.dialog.as_ref().map(|open| dialog::render(open, cx));
        let pushed = matches!(self.page, Page::Pair | Page::Send);
        let tabs = (!pushed).then(|| nav::render(self, cx));
        let body = match self.page {
            Page::Home => home::render(self, cx),
            Page::History => history::render(self, cx),
            Page::Pair => pairing::render(self, cx),
            Page::Send => send::render(self, cx),
            Page::Settings => settings::render(self, cx),
        };
        div()
            .id("main")
            .key_context(keys::CONTEXT)
            .track_focus(&self.focus)
            .group("window")
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family(cx.theme().font_family.clone())
            .on_action(cx.listener(|view, _: &Back, window, cx| view.back(window, cx)))
            .on_action(cx.listener(|_, _: &ChooseFiles, _, cx| Self::pick_pending(cx)))
            .on_action(cx.listener(|view, _: &AddDevice, window, cx| {
                view.open_page(Page::Pair, window, cx);
            }))
            .on_action(cx.listener(|view, _: &OpenSettings, window, cx| {
                view.open_page(Page::Settings, window, cx);
            }))
            .on_drop(cx.listener(|view, paths: &ExternalPaths, _, cx| {
                view.stage(paths.paths().to_vec(), cx);
            }))
            .child(TitleBar::new().child(brand(cx)).child(device_name(self, cx)))
            .children(pushed.then(|| home::toolbar(self, cx)))
            .child(
                div().flex_1().min_h_0().child(
                    div()
                        .id("content")
                        .size_full()
                        .p(px(SPACE_SM))
                        .pt(px(4.))
                        .pb(px(if pushed { SPACE_SM } else { nav::CLEARANCE }))
                        .child(div().mx_auto().w_full().max_w(px(CONTENT_WIDTH)).child(body))
                        .overflow_y_scrollbar(),
                ),
            )
            .children(tabs)
            .children(overlay)
    }
}

fn device_name(view: &MainView, cx: &Context<MainView>) -> AnyElement {
    let name =
        view.state.read(cx).config.as_ref().map(|config| config.name.clone()).unwrap_or_default();
    div()
        .pr(px(SPACE_SM))
        .text_size(px(TYPE_LABEL))
        .text_color(cx.theme().muted_foreground)
        .child(name)
        .into_any_element()
}

fn brand(cx: &Context<MainView>) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .child(svg().path(MARK).size(px(16.)).text_color(color(palette(cx).brand)))
        .child(div().text_size(px(TYPE_LABEL)).font_weight(FontWeight::MEDIUM).child("Solferry"))
}
