use gpui_kit::component::ActiveTheme;
use gpui_kit::component::Sizable;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::input::Input;
use gpui_kit::{
    AnyElement, ClipboardItem, Context, IntoElement, ParentElement, SharedString, Styled, div,
};

use super::main_view::MainView;
use super::ui;
use crate::errors::explain;
use crate::qr;
use crate::service::Command;

pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let state = view.state.read(cx).clone();
    let invite = state.invite.clone();
    let failure = state.pairing_error.as_ref().map(explain);
    let auto_accept = !state.ask_after_pairing;
    let shown = invite.as_deref().map(|code| invite_card(code, cx));
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
        .children(error)
        .child(
            ui::card(cx)
                .child(div().text_sm().child("Add a device"))
                .child(ui::muted("Show a code here, then enter it on the other device.", cx))
                .child(approval_toggle(state.ask_after_pairing, cx))
                .child(Button::new("invite").primary().small().label("Show code").on_click(
                    cx.listener(move |_, _, _, cx| {
                        MainView::command(Command::Invite { auto_accept }, cx);
                    }),
                )),
        )
        .children(shown)
        .child(
            ui::card(cx)
                .child(div().text_sm().child("Have a code?"))
                .child(Input::new(&view.code_input))
                .child(approval_toggle(state.ask_after_pairing, cx))
                .child(
                    Button::new("join")
                        .primary()
                        .small()
                        .label(if state.joining { "Pairing..." } else { "Pair" })
                        .loading(state.joining)
                        .on_click(cx.listener(move |view, _, _, cx| {
                            let code = view.code_input.read(cx).value().trim().to_owned();
                            view.state.update(cx, |state, cx| {
                                state.joining = true;
                                state.pairing_error = None;
                                cx.notify();
                            });
                            MainView::command(Command::Join { code, auto_accept }, cx);
                        })),
                ),
        )
        .into_any_element()
}

fn approval_toggle(ask: bool, cx: &mut Context<MainView>) -> AnyElement {
    let label = if ask { "Ask before receiving" } else { "Receive without asking" };
    Button::new(SharedString::from("pairing-approval"))
        .ghost()
        .small()
        .label(label)
        .on_click(cx.listener(move |view, _, _, cx| {
            view.state.update(cx, |state, cx| {
                state.ask_after_pairing = !ask;
                cx.notify();
            });
        }))
        .into_any_element()
}

fn invite_card(code: &str, cx: &mut Context<MainView>) -> AnyElement {
    let copied = code.to_owned();
    ui::card(cx)
        .items_center()
        .children(qr::code(code))
        .child(ui::muted("Works once, for 10 minutes.", cx))
        .child(Button::new("copy-code").small().label("Copy code").on_click(cx.listener(
            move |_, _, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()));
            },
        )))
        .into_any_element()
}
