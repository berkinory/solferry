//! The languages the app ships and the picker that chooses one.

use gpui_kit::component::ActiveTheme;
use gpui_kit::component::popover::Popover;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::{
    Anchor, AnyElement, App, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, img, px,
};
use rust_i18n::t;
use solferry_core::Config;

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::{Glyph, icon};
use crate::service::Command;
use crate::tokens::{RADIUS_FLOOR, SPACE_MD, TYPE_LABEL};

/// Each language under its own name, so anyone can find theirs, with the flag it is shown by.
const LANGUAGES: &[(&str, &str, &str)] = &[
    ("en", "English", "gb"),
    ("tr", "Türkçe", "tr"),
    ("de", "Deutsch", "de"),
    ("fr", "Français", "fr"),
    ("it", "Italiano", "it"),
    ("es", "Español", "es"),
    ("pt", "Português", "br"),
    ("ru", "Русский", "ru"),
    ("uk", "Українська", "ua"),
    ("ja", "日本語", "jp"),
    ("ko", "한국어", "kr"),
    ("zh", "简体中文", "cn"),
    ("nl", "Nederlands", "nl"),
    ("pl", "Polski", "pl"),
    ("he", "עברית", "il"),
];
const FLAG: f32 = 15.0;
const MENU_HEIGHT: f32 = 267.0;

/// Shows the app in `language`, or in the system's when there is none.
pub fn apply(language: Option<&str>) {
    match language {
        Some(tag) => solferry_core::set_locale(tag),
        None => solferry_core::set_locale(&sys_locale::get_locale().unwrap_or_default()),
    }
}

fn name(language: Option<&str>) -> SharedString {
    LANGUAGES
        .iter()
        .find(|(tag, ..)| Some(*tag) == language)
        .map_or_else(|| t!("settings.language.system").into(), |(_, name, _)| (*name).into())
}

fn choose(view: &MainView, language: Option<&str>, cx: &mut Context<MainView>) {
    apply(language);
    view.state.update(cx, |state, cx| {
        if let Some(config) = &mut state.config {
            config.language = language.map(str::to_owned);
        }
        cx.notify();
    });
    MainView::command(Command::SetLanguage(language.map(str::to_owned)), cx);
}

pub fn section(config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    let current = config.language.clone();
    let title = t!("settings.language.title");
    let main = cx.entity().downgrade();
    let picker = Popover::new("language")
        .anchor(Anchor::TopRight)
        .trigger(
            Button::new("language-trigger")
                .xs()
                .label(name(current.as_deref()))
                .icon(Glyph::ChevronDown),
        )
        .content(move |_, _, cx| {
            let popover = cx.entity();
            let choices = iter_choices().map(|language| {
                let chosen = language == current.as_deref();
                let (main, popover) = (main.clone(), popover.clone());
                div()
                    .id(SharedString::from(format!("language-{}", language.unwrap_or("system"))))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(SPACE_MD))
                    .px(px(9.))
                    .py(px(5.))
                    .rounded(px(RADIUS_FLOOR))
                    .text_size(px(TYPE_LABEL))
                    .cursor_pointer()
                    .hover(|style| style.bg(cx.theme().accent))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(9.))
                            .child(badge(language, cx))
                            .child(name(language)),
                    )
                    .children(
                        chosen.then(|| icon(Glyph::Check, 13.).text_color(cx.theme().foreground)),
                    )
                    .on_click(move |_, window, cx| {
                        popover.update(cx, |state, cx| state.dismiss(window, cx));
                        let _ = main.update(cx, |view, cx| choose(view, language, cx));
                    })
            });
            div()
                .id("language-menu")
                .w(px(184.))
                .max_h(px(MENU_HEIGHT))
                .p(px(4.))
                .overflow_y_scrollbar()
                .children(choices)
        });
    ui::card(Glyph::Language, &title, None, cx)
        .child(
            ui::padded_well(cx).child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(SPACE_MD))
                    .child(ui::muted(t!("settings.language.detail"), cx).flex_1().min_w_0())
                    .child(picker),
            ),
        )
        .into_any_element()
}

fn iter_choices() -> impl Iterator<Item = Option<&'static str>> {
    std::iter::once(None).chain(LANGUAGES.iter().map(|(tag, ..)| Some(*tag)))
}

/// The language's flag, or the globe for the system's choice.
fn badge(language: Option<&str>, cx: &App) -> AnyElement {
    match LANGUAGES.iter().find(|(tag, ..)| Some(*tag) == language) {
        Some((_, _, country)) => {
            img(format!("flags/{country}.svg")).size(px(FLAG)).flex_none().into_any_element()
        }
        None => {
            icon(Glyph::Language, FLAG).text_color(cx.theme().muted_foreground).into_any_element()
        }
    }
}
