//! The permissions card: what the system allows Solferry, and the way to change each.

use futures::channel::oneshot;
use gpui_kit::{AnyElement, App, Entity, IntoElement, ParentElement, Styled, div, px};
use rust_i18n::t;

use super::button::Button;
use super::ui;
use crate::assets::Glyph;
use crate::permissions::{self, Permission, Status};
use crate::state::AppState;
use crate::theme::{color, palette};

/// Asks the system again, for when the person may have changed something in its settings.
pub fn refresh(state: &Entity<AppState>, cx: &mut App) {
    for &permission in permissions::ALL {
        follow(state, permission, permissions::status(permission), cx);
    }
}

fn follow(
    state: &Entity<AppState>,
    permission: Permission,
    answer: oneshot::Receiver<Status>,
    cx: &mut App,
) {
    let state = state.downgrade();
    cx.spawn(async move |cx| {
        let Ok(status) = answer.await else { return };
        let _ = state.update(cx, |state, cx| {
            state.permissions.insert(permission, status);
            cx.notify();
        });
    })
    .detach();
}

/// One row per permission this platform has, or nothing where it has none.
pub fn card(state: &Entity<AppState>, cx: &mut App) -> Option<AnyElement> {
    if permissions::ALL.is_empty() {
        return None;
    }
    let known = state.read(cx).permissions.clone();
    let rows: Vec<_> = permissions::ALL
        .iter()
        .map(|&permission| row(state, permission, known.get(&permission).copied(), cx))
        .collect();
    let well = ui::padded_well(cx).gap(px(14.)).children(rows);
    Some(ui::card(Glyph::Shield, &t!("permissions.title"), None, cx).child(well).into_any_element())
}

fn row(
    #[cfg_attr(
        not(target_os = "macos"),
        expect(unused_variables, reason = "only macOS can ask from the app")
    )]
    state: &Entity<AppState>,
    permission: Permission,
    status: Option<Status>,
    cx: &mut App,
) -> AnyElement {
    let (glyph, title, detail) = match permission {
        Permission::Notifications => (
            Glyph::Notification,
            t!("permissions.notifications.title"),
            t!("permissions.notifications.detail"),
        ),
        #[cfg(target_os = "macos")]
        Permission::LocalNetwork => (
            Glyph::Nearby,
            t!("permissions.local_network.title"),
            t!("permissions.local_network.detail"),
        ),
        #[cfg(windows)]
        Permission::Firewall => {
            (Glyph::Shield, t!("permissions.firewall.title"), t!("permissions.firewall.detail"))
        }
    };
    let palette = palette(cx);
    let line = match status {
        Some(Status::Allowed) => {
            Some(ui::status(color(palette.success), t!("permissions.allowed"), cx))
        }
        Some(Status::Off) => Some(ui::status(color(palette.danger), t!("permissions.off"), cx)),
        #[cfg(target_os = "macos")]
        Some(Status::NotAsked) => {
            Some(ui::status(color(palette.warning), t!("permissions.not_asked"), cx))
        }
        Some(Status::Unknown) | None => None,
    };
    let id = format!("permission-{permission:?}");
    let action = match status {
        Some(Status::Allowed) => None,
        #[cfg(target_os = "macos")]
        Some(Status::NotAsked) => {
            let state = state.clone();
            Some(Button::new(id).xs().primary().label(t!("permissions.allow")).on_click(
                move |_, _, cx| follow(&state, permission, permissions::ask(permission), cx),
            ))
        }
        _ => Some(
            Button::new(id)
                .xs()
                .icon(Glyph::External)
                .label(t!("permissions.open_settings"))
                .on_click(move |_, _, cx| cx.open_url(permissions::settings_url(permission))),
        ),
    };
    div()
        .flex()
        .items_start()
        .gap(px(10.))
        .child(ui::tile(glyph, cx))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap(px(2.))
                .child(ui::text(title))
                .children(line)
                .child(ui::caption(detail, cx)),
        )
        .children(action)
        .into_any_element()
}
