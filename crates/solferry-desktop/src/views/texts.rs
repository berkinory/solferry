use gpui_kit::component::ActiveTheme;
use gpui_kit::component::input::Textarea;
use gpui_kit::{
    AnyElement, ClipboardItem, Context, IntoElement, ParentElement, SharedString, Styled, Window,
    div, px,
};
use rust_i18n::t;

use super::button::Button;
use super::links;
use super::main_view::{MainView, Page};
use super::ui;
use crate::assets::Glyph;
use crate::service::{Command, Target};
use crate::state::{AppState, SharedText};
use crate::tokens::{SPACE_SM, SPACE_XS};

const PREVIEW_LINES: usize = 4;
const TEXT_HEIGHT: f32 = 134.0;

pub fn render(state: &AppState, cx: &mut Context<MainView>) -> Option<AnyElement> {
    state.text.as_ref().map(|shared| card(shared, cx))
}

fn card(shared: &SharedText, cx: &mut Context<MainView>) -> AnyElement {
    let key = shared.key;
    let text = shared.text.clone();
    let dismiss = Button::new(SharedString::from(format!("text-dismiss-{key}")))
        .ghost()
        .xs()
        .icon(Glyph::Cancel)
        .tooltip(t!("texts.dismiss"))
        .on_click(cx.listener(move |view, _, _, cx| view.dismiss_text(key, cx)))
        .into_any_element();
    let (glyph, label) = if shared.copied {
        (Glyph::Check, t!("texts.copied"))
    } else {
        (Glyph::Copy, t!("texts.copy"))
    };
    let heading = ui::header(
        Glyph::Clipboard,
        t!("texts.from", name = shared.name),
        cx.theme().muted_foreground,
        cx,
    );
    ui::titled(heading, Some(dismiss), cx)
        .child(
            ui::padded_well(cx)
                .child(
                    links::text(SharedString::from(format!("text-{key}")), shared.text.clone(), cx)
                        .line_clamp(PREVIEW_LINES),
                )
                .child(
                    div().flex().gap(px(SPACE_SM)).pt(px(SPACE_XS)).child(
                        Button::new(SharedString::from(format!("text-copy-{key}")))
                            .sm()
                            .icon(glyph)
                            .label(label)
                            .on_click(cx.listener(move |view, _, _, cx| {
                                cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
                                view.mark_copied(key, cx);
                            })),
                    ),
                ),
        )
        .into_any_element()
}

/// Text to send to one device, starting from what is on the clipboard.
pub fn sheet(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let name = view.texting.as_ref().map_or_else(String::new, |target| target.name.clone());
    let content = div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(ui::muted(t!("texts.to", name = name), cx))
                .child(
                    Button::new("paste-text")
                        .xs()
                        .icon(Glyph::Clipboard)
                        .label(t!("texts.paste"))
                        .on_click(cx.listener(|view, _, window, cx| view.paste_text(window, cx))),
                ),
        )
        .child(Textarea::new(&view.text_input).h(px(TEXT_HEIGHT)))
        .child(
            div().flex().justify_center().pt(px(SPACE_SM)).child(
                Button::new("send-text")
                    .primary()
                    .icon(Glyph::Send)
                    .label(t!("texts.send"))
                    .on_click(cx.listener(|view, _, window, cx| view.send_text(window, cx))),
            ),
        );
    super::sheet::present(view, t!("texts.title"), content, MainView::leave_page, cx)
}

impl MainView {
    fn paste_text(&mut self, window: &mut Window, cx: &mut Context<MainView>) {
        let Some(pasted) = cx.read_from_clipboard().and_then(|item| item.text()) else { return };
        let text = self.text_input.read(cx).value().to_string() + &pasted;
        self.text_input.update(cx, |input, cx| input.set_value(text, window, cx));
    }

    pub fn compose_text(
        &mut self,
        target: Target,
        window: &mut Window,
        cx: &mut Context<MainView>,
    ) {
        let pasted = cx.read_from_clipboard().and_then(|item| item.text()).unwrap_or_default();
        self.text_input.update(cx, |input, cx| input.set_value(pasted, window, cx));
        self.texting = Some(target);
        self.open_page(Page::Text, window, cx);
    }

    fn send_text(&mut self, window: &mut Window, cx: &mut Context<MainView>) {
        let text = self.text_input.read(cx).value().to_string();
        let Some(target) = self.texting.clone().filter(|_| !text.trim().is_empty()) else { return };
        MainView::command(Command::SendText { target, text }, cx);
        self.dismiss(MainView::leave_page, window, cx);
    }

    pub fn mark_copied(&mut self, key: u64, cx: &mut Context<MainView>) {
        self.state.update(cx, |state, cx| {
            if let Some(shared) = state.text.as_mut().filter(|shared| shared.key == key) {
                shared.copied = true;
            }
            cx.notify();
        });
    }

    pub fn dismiss_text(&mut self, key: u64, cx: &mut Context<MainView>) {
        self.state.update(cx, |state, cx| {
            state.text.take_if(|shared| shared.key == key);
            cx.notify();
        });
    }
}
