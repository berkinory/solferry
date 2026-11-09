use gpui_kit::component::ActiveTheme;
use gpui_kit::{
    AnyElement, ClipboardItem, Context, IntoElement, ParentElement, SharedString, Styled, div, px,
};
use iroh::EndpointId;

use super::button::Button;
use super::main_view::{COPIED_FOR, MainView};
use super::ui;
use crate::assets::Glyph;
use crate::service::{Command, Target};
use crate::state::{AppState, SharedText};
use crate::tokens::{SPACE_SM, SPACE_XS};

const PREVIEW_LINES: usize = 4;

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
        .tooltip("Dismiss")
        .on_click(cx.listener(move |view, _, _, cx| view.dismiss_text(key, cx)))
        .into_any_element();
    let (glyph, label) =
        if shared.copied { (Glyph::Check, "Copied") } else { (Glyph::Copy, "Copy") };
    let heading = ui::header(
        Glyph::Clipboard,
        format!("Text from {}", shared.name),
        cx.theme().muted_foreground,
        cx,
    );
    ui::titled(heading, Some(dismiss), cx)
        .child(
            ui::padded_well(cx)
                .child(ui::text(shared.text.clone()).line_clamp(PREVIEW_LINES))
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

impl MainView {
    /// Sends what is on this device's clipboard to a paired device.
    pub fn share_clipboard(&mut self, id: EndpointId, target: Target, cx: &mut Context<MainView>) {
        let text =
            cx.read_from_clipboard().and_then(|item| item.text()).filter(|text| !text.is_empty());
        let Some(text) = text else {
            self.state.update(cx, |state, cx| {
                state.text_failed(&target.name, &solferry_core::Error::NothingToSend);
                cx.notify();
            });
            return;
        };
        MainView::command(Command::SendText { target, text }, cx);
        self.shared_with = Some(id);
        cx.notify();
        cx.spawn(async move |view, cx| {
            cx.background_executor().timer(COPIED_FOR).await;
            let _ = view.update(cx, |view, cx| {
                view.shared_with = None;
                cx.notify();
            });
        })
        .detach();
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
