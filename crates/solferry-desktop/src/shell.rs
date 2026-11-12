use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use gpui_kit::component::{Theme, TitleBar};
use gpui_kit::{
    App, AppContext, AsyncApp, Bounds, ClipboardItem, Entity, Global, SharedString,
    SystemNotification, WindowBounds, px, size,
};
use rust_i18n::t;
use solferry_core::{Direction, Event};

use crate::instance::Instance;
use crate::service::{Command, Handle, Update};
use crate::state::AppState;
use crate::tray::{Tray, TrayAction, TrayEvents};
use crate::views::MainView;

const WINDOW_WIDTH: f32 = 420.0;
const WINDOW_HEIGHT: f32 = 620.0;
/// Keeps the window clear of a menu bar and a dock or taskbar on small screens.
const SCREEN_MARGIN: f32 = 96.0;
/// How often the tray menu and later launches are checked for.
const POLL_EVERY: Duration = Duration::from_millis(150);

pub struct Shell {
    pub state: Entity<AppState>,
    pub service: Handle,
    pub tray: Tray,
    window: Option<gpui_kit::AnyWindowHandle>,
}

impl Global for Shell {}

pub fn start(
    service: Handle,
    updates: UnboundedReceiver<Update>,
    (tray, actions): (Tray, TrayEvents),
    (instance, hidden): (Instance, bool),
    cx: &mut App,
) {
    let state = cx.new(|_| AppState::default());
    cx.set_global(Shell { state, service, tray, window: None });
    if !hidden {
        show_window(cx);
    }
    let mut updates = updates;
    cx.spawn(async move |cx: &mut AsyncApp| {
        while let Some(update) = updates.next().await {
            cx.update(|cx| handle_update(update, cx));
        }
    })
    .detach();
    cx.spawn(async move |cx: &mut AsyncApp| {
        loop {
            cx.background_executor().timer(POLL_EVERY).await;
            if instance.woken() {
                cx.update(show_window);
            }
            for action in actions.drain() {
                cx.update(|cx| match action {
                    TrayAction::Open => show_window(cx),
                    TrayAction::Quit => quit(cx),
                });
            }
        }
    })
    .detach();
}

fn handle_update(update: Update, cx: &mut App) {
    if matches!(update, Update::Restarting) {
        quit(cx);
        return;
    }
    notify(&update, cx);
    if let Update::Transfer(Event::TextReceived { text, auto_copy: true, .. }) = &update {
        cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
    }
    let wants_attention = matches!(update, Update::Offer { .. } | Update::PairingRequest { .. });
    let state = cx.global::<Shell>().state.clone();
    state.update(cx, |state, cx| {
        state.apply(update);
        cx.notify();
    });
    if wants_attention {
        show_window(cx);
    }
}

fn notify(update: &Update, cx: &mut App) {
    let window = cx.global::<Shell>().window;
    if window.is_some_and(|w| w.update(cx, |_, w, _| w.is_window_active()).unwrap_or(false)) {
        return;
    }
    let state = cx.global::<Shell>().state.read(cx);
    let (title, body) = match update {
        Update::Offer { summary, .. } => (
            t!("notification.offer.title"),
            t!("notification.offer.body", name = summary.peer_name),
        ),
        Update::Transfer(Event::TextReceived { peer_name, auto_copy, .. }) => {
            let body = if *auto_copy {
                t!("notification.text.copied")
            } else {
                t!("notification.text.copy_in_app")
            };
            (t!("notification.text.title", name = peer_name), body)
        }
        Update::PairingRequest { name, .. } => {
            (t!("notification.pairing.title"), t!("notification.pairing.body", name = name))
        }
        Update::Transfer(Event::Completed { id }) => {
            let received = state
                .transfers
                .iter()
                .find(|transfer| transfer.id == *id && transfer.direction == Direction::Receiving);
            let Some(transfer) = received else { return };
            let body =
                t!("notification.received.body", title = transfer.title, peer = transfer.peer);
            (t!("notification.received.title"), body)
        }
        _ => return,
    };
    cx.show_system_notification(SystemNotification {
        tag: "solferry".into(),
        title: SharedString::from(title),
        body: SharedString::from(body),
        actions: Vec::new(),
    });
}

pub fn show_window(cx: &mut App) {
    if let Some(window) = cx.global::<Shell>().window
        && window.update(cx, |_, window, _| window.activate_window()).is_ok()
    {
        return;
    }
    let room = cx
        .primary_display()
        .map_or(px(WINDOW_HEIGHT), |display| display.bounds().size.height - px(SCREEN_MARGIN));
    let bounds = Bounds::centered(None, size(px(WINDOW_WIDTH), px(WINDOW_HEIGHT).min(room)), cx);
    let mut options = TitleBar::window_options();
    options.window_bounds = Some(WindowBounds::Windowed(bounds));
    options.window_min_size = Some(size(px(360.), px(500.)));
    options.app_id = Some("solferry".to_owned());
    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        window.on_window_should_close(cx, |_, cx| {
            cx.global_mut::<Shell>().window = None;
            // Without a tray icon nothing could bring the window back, except the dock on macOS.
            let in_tray = cx
                .global::<Shell>()
                .state
                .read(cx)
                .config
                .as_ref()
                .is_none_or(|config| config.show_in_tray);
            if !in_tray && !cfg!(target_os = "macos") {
                quit(cx);
            }
            true
        });
        Theme::sync_system_appearance(Some(window), cx);
        cx.new(|cx| MainView::new(window, cx))
    });
    if let Ok((window, _)) = opened {
        cx.global_mut::<Shell>().window = Some(window);
    }
    cx.activate(true);
}

pub fn quit(cx: &mut App) {
    cx.global::<Shell>().service.send(Command::Quit);
    cx.quit();
}
