use gpui_kit::component::Sizable;
use gpui_kit::component::input::Input;
use gpui_kit::{AnyElement, ClipboardItem, Context, IntoElement, ParentElement, Styled, div, px};
use rust_i18n::t;

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::qr;
use crate::service::Command;
use crate::state::AppState;
use crate::theme::{color, palette};
use crate::tokens::{SPACE_SM, SPACE_XS};

pub fn join(view: &MainView, cx: &mut Context<MainView>) {
    let code = view.code_input.read(cx).value().trim().to_owned();
    if code.is_empty() {
        return;
    }
    view.state.update(cx, |state, cx| {
        state.joining = true;
        state.join_error = None;
        cx.notify();
    });
    MainView::command(Command::Join { code }, cx);
}

pub fn pair_address(view: &MainView, cx: &mut Context<MainView>) {
    let address = view.address_input.read(cx).value().trim().to_owned();
    if address.is_empty() {
        return;
    }
    view.state.update(cx, |state, cx| {
        state.pairing_address = true;
        state.address_error = None;
        cx.notify();
    });
    MainView::command(Command::PairAddress { address }, cx);
}

pub fn sheet(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let state = view.state.read(cx).clone();
    let code = match state.invite.as_deref() {
        Some(code) => invite(code, view.copied, cx),
        None => invite_prompt(cx),
    };
    let join_label = if state.joining { t!("pairing.pairing") } else { t!("pairing.pair") };
    let content =
        div()
            .flex()
            .flex_col()
            .gap(px(SPACE_SM))
            .child(
                ui::card(Glyph::Qr, &t!("pairing.show_code"), None, cx)
                    .child(ui::padded_well(cx).child(code)),
            )
            .child(
                ui::card(Glyph::Key, &t!("pairing.enter_code"), None, cx).child(
                    ui::padded_well(cx)
                        .child(
                            div()
                                .flex()
                                .gap(px(SPACE_XS))
                                .child(
                                    div()
                                        .flex_1()
                                        .min_w_0()
                                        .child(Input::new(&view.code_input).large()),
                                )
                                .child(
                                    Button::new("join")
                                        .primary()
                                        .label(join_label)
                                        .on_click(cx.listener(|view, _, _, cx| join(view, cx))),
                                ),
                        )
                        .children(state.join_error.clone().map(|error| {
                            ui::muted(error, cx).text_color(color(palette(cx).danger))
                        })),
                ),
            )
            .child(address_card(view, &state, cx));
    super::sheet::present(t!("pairing.title"), content, MainView::leave_page, cx)
}

/// For a device on another subnet, or on a network that keeps devices from finding each other.
fn address_card(view: &MainView, state: &AppState, cx: &mut Context<MainView>) -> AnyElement {
    let label = if state.pairing_address { t!("pairing.pairing") } else { t!("pairing.pair") };
    let own = solferry_core::lan_addresses();
    let own = (!own.is_empty()).then(|| {
        let list: Vec<String> = own.iter().map(ToString::to_string).collect();
        ui::muted(t!("pairing.own_addresses", addresses = list.join(", ")), cx)
    });
    let error = state
        .address_error
        .clone()
        .map(|error| ui::muted(error, cx).text_color(color(palette(cx).danger)));
    ui::card(Glyph::Link, &t!("pairing.address_title"), None, cx)
        .child(
            ui::padded_well(cx)
                .child(
                    div()
                        .flex()
                        .gap(px(SPACE_XS))
                        .child(
                            div().flex_1().min_w_0().child(Input::new(&view.address_input).large()),
                        )
                        .child(
                            Button::new("pair-address")
                                .primary()
                                .label(label)
                                .on_click(cx.listener(|view, _, _, cx| pair_address(view, cx))),
                        ),
                )
                .children(error)
                .children(own),
        )
        .into_any_element()
}

fn invite_prompt(cx: &mut Context<MainView>) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(SPACE_SM))
        .child(ui::muted(t!("pairing.invite_hint"), cx))
        .child(
            Button::new("invite")
                .sm()
                .icon(Glyph::Qr)
                .label(t!("pairing.invite"))
                .on_click(cx.listener(move |_, _, _, cx| MainView::command(Command::Invite, cx))),
        )
        .into_any_element()
}

fn invite(code: &str, copied: bool, cx: &mut Context<MainView>) -> AnyElement {
    let text = code.to_owned();
    let (glyph, label) = if copied {
        (Glyph::Check, t!("pairing.copied"))
    } else {
        (Glyph::Copy, t!("pairing.copy"))
    };
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(SPACE_SM))
        .children(qr::code(code))
        .child(ui::muted(t!("pairing.expires"), cx))
        .child(Button::new("copy-code").sm().icon(glyph).label(label).on_click(cx.listener(
            move |view, _, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
                view.show_copied(cx);
            },
        )))
        .into_any_element()
}
