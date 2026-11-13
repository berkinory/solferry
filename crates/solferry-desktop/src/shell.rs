use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use gpui_kit::component::{Theme, TitleBar};
use std::path::PathBuf;

use gpui_kit::{
    App, AppContext, AsyncApp, Bounds, ClipboardItem, Entity, Global, SharedString,
    SystemNotification, SystemNotificationAction, SystemNotificationResponse, WeakEntity,
    WindowBounds, px, size,
};
use rust_i18n::t;
use solferry_core::{Direction, Event};

use crate::instance::Instance;
use crate::opened::OpenedFiles;
use crate::service::{Command, Handle, Update};
use crate::state::AppState;
use crate::tray::{Tray, TrayAction, TrayEvents};
use crate::views::MainView;

const WINDOW_WIDTH: f32 = 420.0;
const WINDOW_HEIGHT: f32 = 620.0;
/// Keeps the window clear of a menu bar and a dock or taskbar on small screens.
const SCREEN_MARGIN: f32 = 96.0;
/// How often the tray menu, later launches and opened files are checked for.
const POLL_EVERY: Duration = Duration::from_millis(150);

pub struct Shell {
    pub state: Entity<AppState>,
    pub service: Handle,
    pub tray: Tray,
    pub popover: crate::popover::Slot,
    window: Option<(gpui_kit::AnyWindowHandle, WeakEntity<MainView>)>,
}

impl Global for Shell {}

pub fn start(
    service: Handle,
    updates: UnboundedReceiver<Update>,
    (tray, actions): (Tray, TrayEvents),
    (instance, hidden, opened): (Instance, bool, OpenedFiles),
    cx: &mut App,
) {
    let state = cx.new(|_| AppState::default());
    cx.set_global(Shell {
        state,
        service,
        tray,
        popover: crate::popover::Slot::default(),
        window: None,
    });
    if !hidden {
        show_window(cx);
    }
    cx.on_system_notification_response(|response, cx| respond(&response, cx));
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
            match instance.woken() {
                Some(files) if !files.is_empty() => cx.update(|cx| send_files(files, cx)),
                Some(_) => cx.update(show_window),
                None => {}
            }
            let files = opened.drain();
            if !files.is_empty() {
                cx.update(|cx| send_files(files, cx));
            }
            for action in actions.drain() {
                cx.update(|cx| match action {
                    TrayAction::Open => show_window(cx),
                    TrayAction::Quit => quit(cx),
                    TrayAction::Popover(anchor) => crate::popover::toggle(anchor, cx),
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
    if let Update::OfferGone(key) = update {
        cx.dismiss_system_notification(&offer_tag(key));
    }
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
    let window = cx.global::<Shell>().window.as_ref().map(|(window, _)| *window);
    if window.is_some_and(|w| w.update(cx, |_, w, _| w.is_window_active()).unwrap_or(false)) {
        return;
    }
    let state = cx.global::<Shell>().state.read(cx);
    let (tag, actions) = match update {
        Update::Offer { key, .. } => (offer_tag(*key), offer_actions()),
        _ => (SharedString::from("solferry"), Vec::new()),
    };
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
        tag,
        title: SharedString::from(title),
        body: SharedString::from(body),
        actions,
    });
}

/// Opens the send sheet with `paths`, adding them to any files already waiting there.
pub fn send_files(paths: Vec<PathBuf>, cx: &mut App) {
    show_window(cx);
    if let Some((_, view)) = cx.global::<Shell>().window.clone() {
        let _ = view.update(cx, |view, cx| view.stage(paths, cx));
    }
}

const OFFER_TAG: &str = "offer-";
const ACCEPT: &str = "accept";
const DECLINE: &str = "decline";

fn offer_tag(key: u64) -> SharedString {
    SharedString::from(format!("{OFFER_TAG}{key}"))
}

/// Plain answers only: always accepting a device is a choice to make in the app. Linux gets none,
/// since GPUI can't take an XDG notification back once the offer is gone.
fn offer_actions() -> Vec<SystemNotificationAction> {
    if cfg!(target_os = "linux") {
        return Vec::new();
    }
    vec![
        SystemNotificationAction { id: ACCEPT.into(), label: t!("offers.accept").into() },
        SystemNotificationAction { id: DECLINE.into(), label: t!("offers.decline").into() },
    ]
}

fn respond(response: &SystemNotificationResponse, cx: &mut App) {
    let key = response.tag.strip_prefix(OFFER_TAG).and_then(|key| key.parse().ok());
    match (key, response.action_id.as_deref()) {
        (Some(offer), Some(action @ (ACCEPT | DECLINE))) => {
            settle_offer(offer, Command::Answer { offer, accept: action == ACCEPT }, cx);
        }
        _ => show_window(cx),
    }
}

/// Answers an offer and takes back its notification, wherever the answer came from.
pub fn settle_offer(key: u64, answer: Command, cx: &mut App) {
    cx.global::<Shell>().state.clone().update(cx, |state, cx| {
        state.offers.retain(|offer| offer.key != key);
        cx.notify();
    });
    cx.dismiss_system_notification(&offer_tag(key));
    cx.global::<Shell>().service.send(answer);
}

pub fn show_window(cx: &mut App) {
    if let Some((window, _)) = cx.global::<Shell>().window
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
    if let Ok((window, view)) = opened {
        cx.global_mut::<Shell>().window = Some((window, view.downgrade()));
    }
    cx.activate(true);
}

pub fn quit(cx: &mut App) {
    cx.global::<Shell>().service.send(Command::Quit);
    cx.quit();
}
