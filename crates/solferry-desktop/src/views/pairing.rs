use gpui_kit::component::input::Input;
use gpui_kit::component::switch::Switch;
use gpui_kit::{AnyElement, ClipboardItem, Context, IntoElement, ParentElement, Styled, div, px};

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::errors::explain;
use crate::qr;
use crate::service::Command;
use crate::tokens::{SPACE_MD, SPACE_SM, SPACE_XS};

pub fn join(view: &MainView, cx: &mut Context<MainView>) {
    let code = view.code_input.read(cx).value().trim().to_owned();
    if code.is_empty() {
        return;
    }
    let auto_accept = !view.state.read(cx).ask_after_pairing;
    view.state.update(cx, |state, cx| {
        state.joining = true;
        state.pairing_error = None;
        cx.notify();
    });
    MainView::command(Command::Join { code, auto_accept }, cx);
}

pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let state = view.state.read(cx).clone();
    let auto_accept = !state.ask_after_pairing;
    let error = state.pairing_error.as_ref().map(|error| {
        let explanation = explain(error);
        ui::failure(explanation.title, explanation.next, cx)
    });
    let code = match state.invite.as_deref() {
        Some(code) => invite(code, view.copied, cx),
        None => invite_prompt(auto_accept, cx),
    };
    let join_label = if state.joining { "Pairing..." } else { "Pair" };
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .children(error)
        .child(ui::card(Glyph::Qr, "Show a code", None, cx).child(ui::padded_well(cx).child(code)))
        .child(
            ui::card(Glyph::Key, "Enter a code", None, cx).child(
                ui::padded_well(cx).child(
                    div()
                        .flex()
                        .gap(px(SPACE_XS))
                        .child(div().flex_1().min_w_0().child(Input::new(&view.code_input)))
                        .child(
                            Button::new("join")
                                .primary()
                                .sm()
                                .label(join_label)
                                .on_click(cx.listener(|view, _, _, cx| join(view, cx))),
                        ),
                ),
            ),
        )
        .child(
            ui::card(Glyph::Shield, "Trust", None, cx)
                .child(ui::padded_well(cx).child(approval_toggle(auto_accept, cx))),
        )
        .into_any_element()
}

fn invite_prompt(auto_accept: bool, cx: &mut Context<MainView>) -> AnyElement {
    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(SPACE_SM))
        .child(ui::muted(
            "Nearby devices pair from the home screen. For anything else, show a code here and enter it there.",
            cx,
        ))
        .child(Button::new("invite").sm().icon(Glyph::Qr).label("Show code").on_click(
            cx.listener(move |_, _, _, cx| {
                MainView::command(Command::Invite { auto_accept }, cx);
            }),
        ))
        .into_any_element()
}

fn approval_toggle(auto_accept: bool, cx: &mut Context<MainView>) -> AnyElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(SPACE_MD))
        .child(
            div()
                .flex()
                .flex_col()
                .child(ui::text("Receive without asking"))
                .child(ui::muted("You can change this for each device later.", cx)),
        )
        .child(
            Switch::new("pairing-approval")
                .checked(auto_accept)
                .accessibility_label("Receive without asking")
                .on_click(cx.listener(|view, enabled: &bool, _, cx| {
                    let ask = !*enabled;
                    view.state.update(cx, |state, cx| {
                        state.ask_after_pairing = ask;
                        cx.notify();
                    });
                })),
        )
        .into_any_element()
}

fn invite(code: &str, copied: bool, cx: &mut Context<MainView>) -> AnyElement {
    let text = code.to_owned();
    let (glyph, label) = if copied { (Glyph::Check, "Copied") } else { (Glyph::Copy, "Copy code") };
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(SPACE_SM))
        .children(qr::code(code))
        .child(ui::muted("Works once, for 10 minutes.", cx))
        .child(Button::new("copy-code").sm().icon(glyph).label(label).on_click(cx.listener(
            move |view, _, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
                view.show_copied(cx);
            },
        )))
        .into_any_element()
}
