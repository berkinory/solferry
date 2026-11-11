use std::borrow::Cow;

use gpui_kit::component::Sizable;
use gpui_kit::component::switch::Switch;
use gpui_kit::{AnyElement, Context, IntoElement, ParentElement, Styled, div, px};
use rust_i18n::t;

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::service::Command;
use crate::theme::{color, palette};
use crate::tokens::SPACE_MD;
use crate::updater::Status;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn line(status: &Status) -> Cow<'static, str> {
    match status {
        Status::Idle => t!("updates.idle", version = VERSION),
        Status::Checking => t!("updates.checking"),
        Status::UpToDate => t!("updates.up_to_date", version = VERSION),
        Status::Downloading { version, percent } => {
            t!("updates.downloading", version = version, percent = percent)
        }
        Status::Ready { version } => t!("updates.ready", version = version),
        Status::Available { version, .. } => t!("updates.available", version = version),
        Status::Failed => t!("updates.failed"),
    }
}

fn action(status: &Status, cx: &mut Context<MainView>) -> Option<AnyElement> {
    match status {
        Status::Ready { .. } => Some(
            Button::new("update-restart")
                .primary()
                .xs()
                .label(t!("updates.restart"))
                .on_click(cx.listener(|_, _, _, cx| MainView::command(Command::InstallUpdate, cx)))
                .into_any_element(),
        ),
        Status::Available { page, .. } => {
            let page = page.clone();
            Some(
                Button::new("update-page")
                    .xs()
                    .icon(Glyph::External)
                    .label(t!("updates.download"))
                    .on_click(cx.listener(move |_, _, _, cx| cx.open_url(&page)))
                    .into_any_element(),
            )
        }
        _ => None,
    }
}

pub fn section(enabled: bool, status: &Status, cx: &mut Context<MainView>) -> AnyElement {
    let check = Button::new("update-check")
        .ghost()
        .xs()
        .label(t!("updates.check"))
        .on_click(cx.listener(|_, _, _, cx| MainView::command(Command::CheckForUpdates, cx)))
        .into_any_element();
    let shown = if *status == Status::Failed {
        ui::muted(line(status), cx).text_color(color(palette(cx).danger))
    } else {
        ui::muted(line(status), cx)
    };
    let row = || div().flex().items_center().justify_between().gap(px(SPACE_MD));
    ui::card(Glyph::Download, &t!("updates.title"), Some(check), cx)
        .child(
            ui::padded_well(cx)
                .child(
                    row().child(ui::muted(t!("updates.automatic"), cx).flex_1()).child(
                        Switch::new("check-updates")
                            .large()
                            .checked(enabled)
                            .accessibility_label(t!("updates.automatic"))
                            .on_click(cx.listener(|view, on: &bool, _, cx| {
                                view.set_check_updates(*on, cx);
                            })),
                    ),
                )
                .child(row().child(shown.flex_1()).children(action(status, cx))),
        )
        .into_any_element()
}
