use std::path::Path;

use gpui_kit::component::ActiveTheme;
use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::{Icon, IconName, Sizable};
use gpui_kit::{
    AnyElement, Context, ExternalPaths, InteractiveElement, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, div, px,
};

use super::main_view::{MainView, Page};
use super::{devices, offers, transfers, ui};
use crate::errors::explain;
use crate::tokens::{RADIUS_WELL, SPACE_MD, SPACE_SM, SPACE_XS};

const SHOWN_FILES: usize = 3;

pub fn toolbar(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let row = div().flex().flex_none().items_center().justify_between().h(px(36.)).px(px(SPACE_SM));
    if view.page != Page::Home {
        let title = if view.page == Page::Pair { "Add device" } else { "Settings" };
        return row
            .justify_start()
            .gap(px(SPACE_XS))
            .child(
                Button::new("go-home")
                    .ghost()
                    .small()
                    .icon(IconName::ArrowLeft)
                    .accessibility_label("Back")
                    .tooltip("Back")
                    .on_click(cx.listener(|view, _, window, cx| {
                        view.open_page(Page::Home, window, cx);
                    })),
            )
            .child(ui::text(title))
            .into_any_element();
    }
    let name = view.state.read(cx).config.as_ref().map(|config| config.name.clone());
    row.pl(px(SPACE_MD))
        .child(div().min_w_0().children(name.map(|name| ui::muted(name, cx).truncate())))
        .child(
            div()
                .flex()
                .flex_none()
                .items_center()
                .gap(px(SPACE_XS))
                .child(
                    Button::new("add-device")
                        .ghost()
                        .xsmall()
                        .icon(IconName::Plus)
                        .label("Add device")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_page(Page::Pair, window, cx);
                        })),
                )
                .child(
                    Button::new("open-settings")
                        .ghost()
                        .xsmall()
                        .icon(Icon::new(IconName::Settings))
                        .accessibility_label("Settings")
                        .tooltip("Settings")
                        .on_click(cx.listener(|view, _, window, cx| {
                            view.open_page(Page::Settings, window, cx);
                        })),
                ),
        )
        .into_any_element()
}

fn drop_zone(cx: &mut Context<MainView>) -> AnyElement {
    let theme = cx.theme();
    let (hover, sun, on_sun) = (theme.accent, theme.primary, theme.primary_foreground);
    ui::frame(cx)
        .child(
            ui::well(cx)
                .id("drop-zone")
                .flex_row()
                .items_center()
                .gap(px(SPACE_MD))
                .h(px(84.))
                .px(px(14.))
                .border_1()
                .border_color(gpui_kit::transparent_black())
                .cursor_pointer()
                .hover(move |style| style.bg(hover))
                .group_drag_over::<ExternalPaths>("window", move |style| {
                    style.border_color(sun).bg(hover)
                })
                .on_click(cx.listener(|_, _, _, cx| MainView::pick_pending(cx)))
                .child(
                    ui::tile(cx)
                        .size(px(40.))
                        .bg(sun)
                        .child(Icon::new(IconName::ArrowUp).size(px(18.)).text_color(on_sun)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(ui::text("Drop files to send"))
                        .child(ui::muted("or click to choose", cx)),
                ),
        )
        .into_any_element()
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(|| path.display().to_string(), |name| name.to_string_lossy().into_owned())
}

fn pending_row(index: usize, path: &Path, cx: &mut Context<MainView>) -> AnyElement {
    let icon = if path.is_dir() { IconName::Folder } else { IconName::File };
    div()
        .flex()
        .items_center()
        .gap(px(SPACE_SM))
        .pl(px(SPACE_SM))
        .h(px(30.))
        .child(Icon::new(icon).size(px(14.)).text_color(cx.theme().muted_foreground))
        .child(ui::text(file_name(path)).flex_1().min_w_0().truncate())
        .child(
            Button::new(SharedString::from(format!("unpick-{index}")))
                .ghost()
                .xsmall()
                .icon(IconName::Close)
                .accessibility_label("Remove")
                .on_click(cx.listener(move |view, _, _, cx| view.remove_pending(index, cx))),
        )
        .into_any_element()
}

fn pending_card(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let chosen = view.selected.len();
    let total = view.pending.len();
    let rows: Vec<_> = view
        .pending
        .iter()
        .take(SHOWN_FILES)
        .enumerate()
        .map(|(index, path)| pending_row(index, path, cx))
        .collect();
    let more = (total > SHOWN_FILES).then(|| {
        ui::muted(format!("and {} more", total - SHOWN_FILES), cx).pl(px(SPACE_SM)).pb(px(SPACE_XS))
    });
    let send = if chosen == 0 {
        ui::muted("Choose devices below", cx).flex_1().pl(px(SPACE_SM)).into_any_element()
    } else {
        let label =
            if chosen == 1 { "Send".to_owned() } else { format!("Send to {chosen} devices") };
        Button::new("send-selected")
            .primary()
            .flex_1()
            .label(label)
            .on_click(cx.listener(|view, _, _, cx| view.send_selected(cx)))
            .into_any_element()
    };
    ui::card("Ready to send", Some(ui::count(total, cx)), cx)
        .child(
            ui::well(cx).gap(px(0.)).rounded(px(RADIUS_WELL)).children(rows).children(more).child(
                div()
                    .flex()
                    .gap(px(SPACE_XS))
                    .pt(px(SPACE_SM))
                    .items_center()
                    .child(send)
                    .child(
                        Button::new("add-more")
                            .ghost()
                            .icon(IconName::Plus)
                            .accessibility_label("Add more")
                            .tooltip("Add more")
                            .on_click(cx.listener(|_, _, _, cx| MainView::pick_pending(cx))),
                    )
                    .child(
                        Button::new("clear-pending")
                            .ghost()
                            .label("Cancel")
                            .on_click(cx.listener(|view, _, _, cx| view.clear_pending(cx))),
                    ),
            ),
        )
        .into_any_element()
}

pub fn render(view: &MainView, cx: &mut Context<MainView>) -> AnyElement {
    let state = view.state.read(cx).clone();
    let fatal = state.fatal.as_ref().map(|error| {
        let explanation = explain(error);
        ui::failure(explanation.title, explanation.next, cx)
    });
    let offers: Vec<_> = state.offers.iter().map(|offer| offers::card(offer, cx)).collect();
    let prompts: Vec<_> =
        state.pairing_prompts.iter().map(|prompt| offers::pairing_card(prompt, cx)).collect();
    let code = state.pairing_code.as_deref().map(|code| offers::pairing_code(code, cx));
    let send = if view.pending.is_empty() { drop_zone(cx) } else { pending_card(view, cx) };
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_SM))
        .children(fatal)
        .children(prompts)
        .children(code)
        .children(offers)
        .child(send)
        .child(devices::render(&state, view, cx))
        .children(transfers::render(&state, cx))
        .into_any_element()
}
