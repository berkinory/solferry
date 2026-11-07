use std::time::Duration;

use futures::StreamExt;
use futures::channel::mpsc::UnboundedReceiver;
use gpui_kit::component::{Theme, TitleBar};
use gpui_kit::{
    App, AppContext, AsyncApp, Bounds, ClipboardItem, Entity, Global, SharedString,
    SystemNotification, WindowBounds, px, size,
};
use solferry_core::{Direction, Event};

use crate::service::{Command, Handle, Update};
use crate::state::AppState;
use crate::tray::{TrayAction, TrayEvents};
use crate::views::MainView;

const TRAY_POLL: Duration = Duration::from_millis(150);

pub struct Shell {
    pub state: Entity<AppState>,
    pub service: Handle,
    window: Option<gpui_kit::AnyWindowHandle>,
}

impl Global for Shell {}

pub fn start(service: Handle, updates: UnboundedReceiver<Update>, tray: TrayEvents, cx: &mut App) {
    let state = cx.new(|_| AppState::default());
    cx.set_global(Shell { state, service, window: None });
    show_window(cx);
    let mut updates = updates;
    cx.spawn(async move |cx: &mut AsyncApp| {
        while let Some(update) = updates.next().await {
            cx.update(|cx| handle_update(update, cx));
        }
    })
    .detach();
    cx.spawn(async move |cx: &mut AsyncApp| {
        loop {
            cx.background_executor().timer(TRAY_POLL).await;
            for action in tray.drain() {
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
    notify(&update, cx);
    if let Update::Transfer(Event::TextReceived { text, auto_copy: true, .. }) = &update {
        cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
    }
    let wants_attention = matches!(update, Update::Offer { .. });
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
    let state = cx.global::<Shell>().state.read(cx);
    let (title, body) = match update {
        Update::Offer { summary, .. } => {
            ("Incoming files".to_owned(), format!("{} wants to send you files", summary.peer_name))
        }
        Update::Transfer(Event::TextReceived { peer_name, auto_copy, .. }) => {
            let body = if *auto_copy {
                "Copied to your clipboard".to_owned()
            } else {
                "Open Solferry to copy it".to_owned()
            };
            (format!("Text from {peer_name}"), body)
        }
        Update::PairingRequest { name, .. } => {
            ("Pairing request".to_owned(), format!("{name} wants to pair with this device"))
        }
        Update::Transfer(Event::Completed { id }) => {
            let received = state
                .transfers
                .iter()
                .find(|transfer| transfer.id == *id && transfer.direction == Direction::Receiving);
            let Some(transfer) = received else { return };
            ("Files received".to_owned(), format!("{} from {}", transfer.title, transfer.peer))
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
    let bounds = Bounds::centered(None, size(px(440.), px(720.)), cx);
    let mut options = TitleBar::window_options();
    options.window_bounds = Some(WindowBounds::Windowed(bounds));
    options.window_min_size = Some(size(px(360.), px(480.)));
    options.app_id = Some("solferry".to_owned());
    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        window.on_window_should_close(cx, |_, cx| {
            cx.global_mut::<Shell>().window = None;
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
