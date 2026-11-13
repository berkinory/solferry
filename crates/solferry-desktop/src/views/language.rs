//! The languages the app ships and the picker that chooses one.

use gpui_kit::component::ActiveTheme;
use gpui_kit::component::popover::Popover;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::{
    Anchor, AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString,
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

fn choose(view: &MainView, language: &str, cx: &mut Context<MainView>) {
    apply(Some(language));
    view.state.update(cx, |state, cx| {
        if let Some(config) = &mut state.config {
            config.language = Some(language.to_owned());
        }
        cx.notify();
    });
    MainView::command(Command::SetLanguage(Some(language.to_owned())), cx);
}

pub fn section(config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    // Until someone picks one, the language shown is the one the system's resolved to.
    let current = config.language.clone().unwrap_or_else(|| rust_i18n::locale().to_string());
    let current_name =
        LANGUAGES.iter().find(|(tag, ..)| *tag == current).map_or("English", |l| l.1);
    let title = t!("settings.language.title");
    let main = cx.entity().downgrade();
    let picker = Popover::new("language")
        .anchor(Anchor::TopRight)
        .trigger(Button::new("language-trigger").xs().label(current_name).icon(Glyph::ChevronDown))
        .content(move |_, _, cx| {
            let popover = cx.entity();
            let choices = LANGUAGES.iter().map(|&(language, name, country)| {
                let chosen = language == current;
                let (main, popover) = (main.clone(), popover.clone());
                div()
                    .id(SharedString::from(format!("language-{language}")))
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
                            .child(img(format!("flags/{country}.svg")).size(px(FLAG)).flex_none())
                            .child(name),
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
