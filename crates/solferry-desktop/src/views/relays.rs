use std::borrow::Cow;

use gpui_kit::component::Sizable;
use gpui_kit::component::input::Input;
use gpui_kit::{
    AnyElement, ClipboardItem, Context, Hsla, IntoElement, ParentElement, SharedString, Styled,
    Window, div, px,
};
use rust_i18n::t;
use solferry_core::{Config, RelayHealth, RelayLink};

use super::button::Button;
use super::main_view::MainView;
use super::settings::change;
use super::{relay_limit, ui};
use crate::assets::Glyph;
use crate::state::Tone;
use crate::theme::{color, palette};
use crate::tokens::SPACE_XS;
use solferry_core::explain;

const GUIDE: &str = "https://github.com/berkinory/solferry/blob/main/docs/relay.md";

pub fn section(view: &MainView, config: &Config, cx: &mut Context<MainView>) -> AnyElement {
    let health = view.state.read(cx).relay_health.clone();
    let rows: Vec<_> = config.relays.iter().map(|url| row(url, &health, cx)).collect();
    let guide = Button::new("relay-guide")
        .ghost()
        .xs()
        .icon(Glyph::External)
        .label(t!("relays.guide"))
        .on_click(cx.listener(|_, _, _, cx| cx.open_url(GUIDE)))
        .into_any_element();
    ui::card(Glyph::Relay, &t!("relays.title"), Some(guide), cx)
        .child(
            ui::padded_well(cx)
                .child(ui::muted(t!("relays.hint"), cx))
                .children(rows)
                .child(
                    div()
                        .flex()
                        .gap(px(SPACE_XS))
                        .child(
                            div().flex_1().min_w_0().child(Input::new(&view.relay_input).large()),
                        )
                        .child(
                            Button::new("add-relay")
                                .icon(Glyph::Add)
                                .label(t!("relays.add"))
                                .on_click(cx.listener(|view, _, window, cx| add(view, window, cx))),
                        ),
                )
                .child(relay_limit::field(view, cx)),
        )
        .into_any_element()
}

fn row(url: &str, health: &[RelayHealth], cx: &mut Context<MainView>) -> AnyElement {
    let (removed, copied) = (url.to_owned(), url.to_owned());
    let parsed = RelayLink::parse(url);
    let found = parsed
        .as_ref()
        .ok()
        .and_then(|link| health.iter().find(|health| health.url == *link.url()));
    let shown = parsed.map_or_else(|_| url.to_owned(), |link| link.masked());
    let line = match found {
        Some(found) => {
            let (tone, label) = status(found, cx);
            ui::status(tone, label, cx).into_any_element()
        }
        None => ui::muted(t!("relays.checking"), cx).into_any_element(),
    };
    div()
        .flex()
        .items_center()
        .gap(px(10.))
        .child(ui::tile(Glyph::Relay, cx))
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .child(ui::text(shown).truncate())
                .child(line),
        )
        .child(
            Button::new(SharedString::from(format!("copy-{url}")))
                .ghost()
                .xs()
                .icon(Glyph::Copy)
                .tooltip(t!("relays.copy"))
                .on_click(cx.listener(move |view, _, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                    view.state.update(cx, |state, cx| {
                        state.toast(Tone::Success, t!("relays.copied"), None);
                        cx.notify();
                    });
                })),
        )
        .child(
            Button::new(SharedString::from(format!("remove-{url}")))
                .ghost()
                .xs()
                .icon(Glyph::Delete)
                .tooltip(t!("relays.remove"))
                .on_click(cx.listener(move |view, _, _, cx| {
                    let removed = removed.clone();
                    change(view, cx, |config| config.relays.retain(|url| *url != removed));
                })),
        )
        .into_any_element()
}

fn status(health: &RelayHealth, cx: &mut Context<MainView>) -> (Hsla, Cow<'static, str>) {
    let palette = palette(cx);
    match (health.reachable, health.accepted, health.direct_paths) {
        (false, ..) => (color(palette.danger), t!("relays.unreachable")),
        (true, false, _) => (color(palette.danger), t!("relays.refused")),
        (true, true, false) => (color(palette.warning), t!("relays.udp_blocked")),
        (true, true, true) => (color(palette.success), t!("relays.connected")),
    }
}

pub fn add(view: &MainView, window: &mut Window, cx: &mut Context<MainView>) {
    let url = view.relay_input.read(cx).value().trim().to_owned();
    if url.is_empty() {
        return;
    }
    let added = match RelayLink::parse(&url) {
        Ok(added) => added,
        Err(error) => {
            view.state.update(cx, |state, cx| {
                state.toast(Tone::Error, explain(&error).title, None);
                cx.notify();
            });
            return;
        }
    };
    // A new link for a relay already here is its new token, so it replaces the old one.
    change(view, cx, |config| {
        config.relays.retain(|existing| {
            RelayLink::parse(existing).map_or(true, |existing| !existing.same_server(&added))
        });
        config.relays.push(url);
    });
    view.relay_input.update(cx, |input, cx| input.set_value("", window, cx));
}
