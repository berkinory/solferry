use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyElement, BoxShadow, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
    StatefulInteractiveElement, Styled, black, div, point, px,
};
use iroh::EndpointId;

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::service::Command;
use crate::tokens::{RADIUS_CARD, SPACE_LG, SPACE_SM, SPACE_XS};

const WIDTH: f32 = 350.0;

pub enum Dialog {
    Forget { id: EndpointId, name: String },
    Block { key: u64, name: String },
}

struct Copy {
    title: String,
    body: &'static str,
    action: &'static str,
}

fn copy(dialog: &Dialog) -> Copy {
    match dialog {
        Dialog::Forget { name, .. } => Copy {
            title: format!("Forget {name}?"),
            body: "It goes back to being an unpaired device until you pair again.",
            action: "Forget",
        },
        Dialog::Block { name, .. } => Copy {
            title: format!("Block {name}?"),
            body: "It can't send files or ask to pair. You can unblock it in Settings.",
            action: "Block",
        },
    }
}

fn confirm(view: &mut MainView, cx: &mut Context<MainView>) {
    match view.dialog.take() {
        Some(Dialog::Forget { id, .. }) => {
            view.expanded = None;
            MainView::command(Command::Forget(id), cx);
        }
        Some(Dialog::Block { key, .. }) => view.block_offer(key, cx),
        None => {}
    }
    cx.notify();
}

pub fn render(dialog: &Dialog, cx: &mut Context<MainView>) -> AnyElement {
    let text = copy(dialog);
    let theme = cx.theme();
    let sheet = div()
        .id("dialog")
        .flex()
        .flex_col()
        .gap(px(SPACE_XS))
        .w(px(WIDTH))
        .max_w_full()
        .p(px(SPACE_LG))
        .rounded(px(RADIUS_CARD))
        .bg(theme.background)
        .border_1()
        .border_color(theme.border)
        .shadow(vec![
            BoxShadow {
                color: black().opacity(0.06),
                offset: point(px(0.), px(1.)),
                blur_radius: px(2.),
                spread_radius: px(0.),
                inset: false,
            },
            BoxShadow {
                color: black().opacity(0.18),
                offset: point(px(0.), px(12.)),
                blur_radius: px(32.),
                spread_radius: px(0.),
                inset: false,
            },
        ])
        .on_click(|_, _, cx| cx.stop_propagation())
        .child(ui::text(text.title).font_weight(FontWeight::MEDIUM))
        .child(ui::muted(text.body, cx))
        .child(
            div()
                .flex()
                .justify_end()
                .gap(px(SPACE_SM))
                .pt(px(SPACE_LG))
                .child(Button::new("dialog-cancel").sm().label("Cancel").on_click(cx.listener(
                    |view, _, _, cx| {
                        view.dialog = None;
                        cx.notify();
                    },
                )))
                .child(
                    Button::new("dialog-confirm")
                        .danger()
                        .sm()
                        .label(text.action)
                        .on_click(cx.listener(|view, _, _, cx| confirm(view, cx))),
                ),
        );
    div()
        .id("dialog-backdrop")
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .p(px(SPACE_LG))
        .bg(black().opacity(0.4))
        .occlude()
        .on_click(cx.listener(|view, _, _, cx| {
            view.dialog = None;
            cx.notify();
        }))
        .child(sheet)
        .into_any_element()
}
