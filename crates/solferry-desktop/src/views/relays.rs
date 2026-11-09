use gpui_kit::component::Sizable;
use gpui_kit::component::input::Input;
use gpui_kit::{
    AnyElement, Context, Hsla, IntoElement, ParentElement, SharedString, Styled, Window, div, px,
};
use solferry_core::{Config, RelayHealth, RelayLink};

use super::button::Button;
use super::main_view::MainView;
use super::settings::change;
use super::ui;
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
        .label("Setup Guide")
        .on_click(cx.listener(|_, _, _, cx| cx.open_url(GUIDE)))
        .into_any_element();
    ui::card(Glyph::Relay, "Relay", Some(guide), cx)
        .child(
            ui::padded_well(cx)
                .child(ui::muted("Reach devices on other networks through a relay you run.", cx))
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
                                .label("Add")
                                .on_click(cx.listener(|view, _, window, cx| add(view, window, cx))),
                        ),
                ),
        )
        .into_any_element()
}

fn row(url: &str, health: &[RelayHealth], cx: &mut Context<MainView>) -> AnyElement {
    let removed = url.to_owned();
    let parsed = RelayLink::parse(url);
    let found = parsed
        .as_ref()
        .ok()
        .and_then(|link| health.iter().find(|health| health.url == *link.url()));
    let shown = parsed.map_or_else(|_| url.to_owned(), |link| link.url().to_string());
    let line = match found {
        Some(found) => {
            let (tone, label) = status(found, cx);
            ui::status(tone, label, cx).into_any_element()
        }
        None => ui::muted("Checking", cx).into_any_element(),
    };
    div()
        .flex()
        .items_center()
        .gap(px(14.))
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
            Button::new(SharedString::from(format!("remove-{url}")))
                .ghost()
                .xs()
                .icon(Glyph::Delete)
                .tooltip("Remove")
                .on_click(cx.listener(move |view, _, _, cx| {
                    let removed = removed.clone();
                    change(view, cx, |config| config.relays.retain(|url| *url != removed));
                })),
        )
        .into_any_element()
}

fn status(health: &RelayHealth, cx: &mut Context<MainView>) -> (Hsla, &'static str) {
    let palette = palette(cx);
    match (health.reachable, health.accepted, health.direct_paths) {
        (false, ..) => (color(palette.danger), "Can't reach the server"),
        (true, false, _) => (color(palette.danger), "Refused, check the link"),
        (true, true, false) => (color(palette.warning), "Connected, UDP 7842 is blocked"),
        (true, true, true) => (color(palette.success), "Connected"),
    }
}

pub fn add(view: &MainView, window: &mut Window, cx: &mut Context<MainView>) {
    let url = view.relay_input.read(cx).value().trim().to_owned();
    if url.is_empty() {
        return;
    }
    if let Err(error) = RelayLink::parse(&url) {
        view.state.update(cx, |state, cx| {
            state.toast(Tone::Error, explain(&error).title, None);
            cx.notify();
        });
        return;
    }
    change(view, cx, |config| {
        config.relays.retain(|existing| *existing != url);
        config.relays.push(url);
    });
    view.relay_input.update(cx, |input, cx| input.set_value("", window, cx));
}
