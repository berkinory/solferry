use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;

use gpui_kit::component::input::{InputEvent, InputState, TextareaState};
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::{ActiveTheme, Theme, TitleBar};
use gpui_kit::{
    AnyElement, AppContext, Context, Entity, ExternalPaths, FocusHandle, Focusable, FontWeight,
    InteractiveElement, IntoElement, ParentElement, PathPromptOptions, Render, Styled, Window, div,
    px, svg,
};
use iroh::EndpointId;
use rust_i18n::t;

use super::dialog::{self, Dialog};
use super::keys::{self, AddDevice, Back, ChooseFiles, OpenSettings};
use super::onboarding::Onboarding;
use super::{
    confirm, history, home, nav, pairing, relays, send, settings, texts, toast, toast_timing,
};
use crate::assets::MARK;
use crate::service::{Command, Target};
use crate::shell::Shell;
use crate::state::AppState;
use crate::theme::{color, palette};
use crate::tokens::{SPACE_SM, TYPE_LABEL};

pub const COPIED_FOR: Duration = Duration::from_millis(1500);
const FLY_FOR: Duration = Duration::from_millis(380);
const CONTENT_WIDTH: f32 = 720.0;
const HEADER_HEIGHT: f32 = 50.0;
const CONTENT_TOP: f32 = 4.0;
/// The mark's art sits low in its box, and letters read high in theirs.
const MARK_LIFT: f32 = -2.0;
const TITLE_BAR: f32 = 34.0;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SendPhase {
    Idle,
    Flying,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Home,
    History,
    Pair,
    Send,
    Settings,
    Text,
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
    pub texting: Option<Target>,
    pub toasts_seen: u64,
    pub pill_seen: u64,
    pub transfers_open: bool,
    pub fatal_open: bool,
    pub sheet_motion: super::sheet::Motion,
    targets: HashMap<EndpointId, Target>,
    pub code_input: Entity<InputState>,
    pub address_input: Entity<InputState>,
    pub name_input: Entity<InputState>,
    pub relay_input: Entity<InputState>,
    pub text_input: Entity<TextareaState>,
    onboarding: Option<Entity<Onboarding>>,
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
            toast_timing::track(view, &state, cx);
            cx.notify();
        })
        .detach();
        let code_input =
            cx.new(|cx| InputState::new(window, cx).placeholder(t!("pairing.code_placeholder")));
        let address_input = cx.new(|cx| InputState::new(window, cx).placeholder("192.168.1.20"));
        let text_input =
            cx.new(|cx| TextareaState::new(window, cx).placeholder(t!("texts.placeholder")));
        let name_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder(t!("settings.device.name_placeholder"))
        });
        let relay_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("https://relay.example.com#token"));
        cx.subscribe_in(&code_input, window, |view, _, event, _, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                pairing::join(view, cx);
            }
        })
        .detach();
        cx.subscribe_in(&address_input, window, |view, _, event, _, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                pairing::pair_address(view, cx);
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
        super::access::refresh(&state, cx);
        cx.observe_window_activation(window, |view, window, cx| {
            if window.is_window_active() {
                super::access::refresh(&view.state, cx);
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
            texting: None,
            toasts_seen: 0,
            pill_seen: 0,
            transfers_open: false,
            fatal_open: false,
            sheet_motion: super::sheet::Motion::Rising,
            targets: HashMap::new(),
            code_input,
            address_input,
            name_input,
            relay_input,
            text_input,
            onboarding: None,
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

    pub fn set_check_updates(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            if let Some(config) = &mut state.config {
                config.check_updates = enabled;
            }
            cx.notify();
        });
        Self::command(Command::SetCheckUpdates(enabled), cx);
    }

    pub fn set_show_in_tray(&mut self, shown: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            if let Some(config) = &mut state.config {
                config.show_in_tray = shown;
            }
            cx.notify();
        });
        cx.global::<Shell>().tray.set_visible(shown);
        Self::command(Command::SetShowInTray(shown), cx);
    }

    pub fn leave_page(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, _| {
            state.join_error = None;
            state.address_error = None;
        });
        if self.page == Page::Send {
            self.clear_pending(cx);
        } else {
            self.open_page(Page::Home, window, cx);
        }
    }

    fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.dialog.take().is_some() {
            cx.notify();
        } else if matches!(self.page, Page::Send | Page::Pair | Page::Text) {
            self.dismiss(Self::leave_page, window, cx);
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

    /// Hands the files over and leaves once the cursor has flown. The pill takes it from there:
    /// it waits for the other device, then shows progress, and a toast says if a send failed.
    pub fn send_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.send_phase != SendPhase::Idle {
            return;
        }
        let targets: Vec<Target> =
            self.selected.iter().filter_map(|id| self.targets.get(id).cloned()).collect();
        Self::command(Command::Send { targets, paths: self.pending.clone() }, cx);
        self.send_phase = SendPhase::Flying;
        cx.notify();
        cx.spawn_in(window, async move |view, cx| {
            cx.background_executor().timer(FLY_FOR).await;
            let _ = view.update_in(cx, |view, window, cx| {
                view.dismiss(|view, _, cx| view.clear_pending(cx), window, cx);
            });
        })
        .detach();
    }

    pub fn clear_pending(&mut self, cx: &mut Context<Self>) {
        self.send_phase = SendPhase::Idle;
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

    pub fn stop_comparing(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.pairing_code = None;
            cx.notify();
        });
    }

    pub fn decline_first_pairing(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(key) = self.state.read(cx).pairing_prompts.first().map(|prompt| prompt.key) {
            self.answer_pairing(key, false, cx);
        }
    }

    pub fn answer_pairing(&mut self, key: u64, accept: bool, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            state.pairing_prompts.retain(|prompt| prompt.key != key);
            cx.notify();
        });
        Self::command(Command::AnswerPairing { key, accept }, cx);
    }
}

fn file_picker() -> PathPromptOptions {
    PathPromptOptions {
        files: true,
        directories: true,
        multiple: true,
        prompt: Some(t!("send.prompt").into()),
    }
}

impl Focusable for MainView {
    fn focus_handle(&self, _: &gpui_kit::App) -> FocusHandle {
        self.focus.clone()
    }
}

impl Render for MainView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(screen) = self.onboarding(window, cx) {
            return screen;
        }
        let overlay = self.dialog.as_ref().map(|open| dialog::render(open, cx));
        let link = self.state.read(cx).link.clone();
        let sheet = confirm::sheet(self, cx)
            .or_else(|| link.map(|link| super::link::sheet(self, &link, cx)))
            .or_else(|| match self.page {
                Page::Send => Some(send::sheet(self, cx)),
                Page::Pair => Some(pairing::sheet(self, cx)),
                Page::Text => Some(texts::sheet(self, cx)),
                _ => None,
            });
        let toasts = toast::render(self, f32::from(window.viewport_size().width), cx);
        let room = f32::from(window.viewport_size().height) - HEADER_HEIGHT - nav::CLEARANCE;
        let tabs = nav::render(self, cx);
        let body = match self.page {
            Page::Home | Page::Send | Page::Pair | Page::Text => home::render(self, cx),
            Page::History => history::render(self, room, cx),
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
            .child(TitleBar::new())
            .child(div().flex_none().h(px(HEADER_HEIGHT - TITLE_BAR)))
            .child(brand(cx))
            .child(
                div().flex_1().min_h_0().child(
                    div()
                        .id("content")
                        .size_full()
                        .p(px(SPACE_SM))
                        .pt(px(CONTENT_TOP))
                        .pb(px(nav::CLEARANCE))
                        .child(div().mx_auto().w_full().max_w(px(CONTENT_WIDTH)).child(body))
                        .overflow_y_scrollbar(),
                ),
            )
            .child(tabs)
            .children(sheet)
            .children(overlay)
            .children(toasts)
            .into_any_element()
    }
}

impl MainView {
    /// The first launch screen, until it has been finished once.
    fn onboarding(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        let config = self.state.read(cx).config.clone().filter(|config| !config.onboarded);
        let Some(config) = config else {
            self.onboarding = None;
            return None;
        };
        let state = self.state.clone();
        let screen = self
            .onboarding
            .get_or_insert_with(|| cx.new(|cx| Onboarding::new(state, config.name, window, cx)))
            .clone();
        let frame = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .font_family(cx.theme().font_family.clone())
            .child(TitleBar::new())
            .child(div().flex_1().min_h_0().child(screen));
        Some(frame.into_any_element())
    }
}

/// Centered over the title bar, which keeps its own window buttons and drag area.
fn brand(cx: &Context<MainView>) -> impl IntoElement {
    div()
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .h(px(HEADER_HEIGHT + CONTENT_TOP))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(6.))
        .child(
            svg().path(MARK).size(px(14.)).mt(px(MARK_LIFT)).text_color(color(palette(cx).brand)),
        )
        .child(div().text_size(px(TYPE_LABEL)).font_weight(FontWeight::MEDIUM).child("Solferry"))
}
