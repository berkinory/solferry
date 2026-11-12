//! Light, dark, or whatever the system shows.

use gpui_kit::base::Selectable;
use gpui_kit::component::Theme;
use gpui_kit::{
    AnyElement, Context, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
};
use rust_i18n::t;
use solferry_core::{Appearance, Config};

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::service::Command;
use crate::state::AppState;
use crate::theme;
use crate::tokens::SPACE_XS;

fn chosen(state: &AppState) -> Appearance {
    state.config.as_ref().map_or(Appearance::System, |config| config.appearance)
}

/// Draws the window in the chosen appearance, and keeps it in step with the system on "System".
pub fn follow(state: &Entity<AppState>, window: &mut Window, cx: &mut Context<MainView>) {
    cx.observe_window_appearance(window, |view, window, cx| {
        if chosen(view.state.read(cx)) == Appearance::System {
            Theme::sync_system_appearance(Some(window), cx);
        }
    })
    .detach();
    let mut shown = chosen(state.read(cx));
    theme::apply(shown, window, cx);
    cx.observe_in(state, window, move |_, state, window, cx| {
        let wanted = chosen(state.read(cx));
        if wanted != shown {
            shown = wanted;
            theme::apply(wanted, window, cx);
        }
    })
    .detach();
}

pub fn section(config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    let options = [
        (Appearance::System, t!("settings.appearance.system")),
        (Appearance::Light, t!("settings.appearance.light")),
        (Appearance::Dark, t!("settings.appearance.dark")),
    ];
    let choices = options.into_iter().map(|(appearance, label)| {
        Button::new(SharedString::from(format!("appearance-{appearance:?}")))
            .ghost()
            .xs()
            .label(label)
            .selected(config.appearance == appearance)
            .on_click(cx.listener(move |view, _, _, cx| choose(view, appearance, cx)))
    });
    ui::card(Glyph::Appearance, &t!("settings.appearance.title"), None, cx)
        .child(
            ui::padded_well(cx).child(div().flex().flex_wrap().gap(px(SPACE_XS)).children(choices)),
        )
        .into_any_element()
}

/// Saves the choice. The main view sees the config change and redraws in it.
fn choose(view: &MainView, appearance: Appearance, cx: &mut Context<MainView>) {
    view.state.update(cx, |state, cx| {
        if let Some(config) = &mut state.config {
            config.appearance = appearance;
        }
        cx.notify();
    });
    MainView::command(Command::SetAppearance(appearance), cx);
}
