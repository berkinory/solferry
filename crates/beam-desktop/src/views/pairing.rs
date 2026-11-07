use beam_core::Relationship;
use gpui_kit::component::ActiveTheme;
use gpui_kit::component::Sizable;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::{AnyElement, ClipboardItem, Context, IntoElement, ParentElement, Styled, div};

use super::main_view::{MainView, Page};
use super::ui;
use crate::errors::explain;
use crate::qr;
use crate::service::Command;

pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let state = view.state.read(cx).clone();
    let invite = state.invite;
    let failure = state.pairing_error.as_ref().map(explain);
    let shown = invite.map(|code| invite_card(code, cx));
    let error = failure.map(|explanation| {
        ui::card(cx)
            .border_color(cx.theme().danger)
            .child(div().text_sm().child(explanation.title))
            .child(ui::muted(explanation.next, cx))
    });
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            ui::card(cx)
                .child(div().text_sm().child("Add a device"))
                .child(ui::muted("Show a code here, then enter it on the other device.", cx))
                .child(
                    div()
                        .flex()
                        .gap_2()
                        .child(
                            Button::new("invite-own")
                                .primary()
                                .small()
                                .label("It's my device")
                                .on_click(cx.listener(|view, _, _, cx| {
                                    view.command(Command::Invite(Relationship::Own), cx);
                                })),
                        )
                        .child(
                            Button::new("invite-contact")
                                .small()
                                .label("It's someone else's")
                                .on_click(cx.listener(|view, _, _, cx| {
                                    view.command(Command::Invite(Relationship::Contact), cx);
                                })),
                        ),
                ),
        )
        .children(shown)
        .child(
            ui::card(cx)
                .child(div().text_sm().child("Have a code?"))
                .child(Input::new(&view.code_input))
                .child(
                    Button::new("join")
                        .primary()
                        .small()
                        .label("Pair")
                        .on_click(cx.listener(|view, _, _, cx| {
                            let code = view.code_input.read(cx).value().to_string();
                            view.command(Command::Join(code), cx);
                            view.page = Page::Home;
                            cx.notify();
                        })),
                ),
        )
        .children(error)
        .into_any_element()
}

fn invite_card(code: String, cx: &mut Context<MainView>) -> AnyElement {
    let copied = code.clone();
    ui::card(cx)
        .items_center()
        .children(qr::code(&code))
        .child(ui::muted("Works once, for 10 minutes.", cx))
        .child(
            Button::new("copy-code")
                .small()
                .label("Copy code")
                .on_click(cx.listener(move |_, _, _, cx| {
                    cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
                })),
        )
        .into_any_element()
}
