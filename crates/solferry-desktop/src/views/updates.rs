use gpui_kit::component::Sizable;
use gpui_kit::component::switch::Switch;
use gpui_kit::{AnyElement, Context, IntoElement, ParentElement, Styled, div, px};

use super::button::Button;
use super::main_view::MainView;
use super::ui;
use crate::assets::Glyph;
use crate::service::Command;
use crate::theme::{color, palette};
use crate::tokens::SPACE_MD;
use crate::updater::Status;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn line(status: &Status) -> String {
    match status {
        Status::Idle => format!("Solferry {VERSION}"),
        Status::Checking => "Checking for updates...".to_owned(),
        Status::UpToDate => format!("Solferry {VERSION} is up to date"),
        Status::Downloading { version, percent } => format!("Downloading {version}, {percent}%"),
        Status::Ready { version } => format!("Solferry {version} is ready"),
        Status::Available { version, .. } => format!("Solferry {version} is available"),
        Status::Failed => "Couldn't check for updates".to_owned(),
    }
}

fn action(status: &Status, cx: &mut Context<MainView>) -> Option<AnyElement> {
    match status {
        Status::Ready { .. } => Some(
            Button::new("update-restart")
                .primary()
                .xs()
                .label("Restart")
                .on_click(cx.listener(|_, _, _, cx| MainView::command(Command::InstallUpdate, cx)))
                .into_any_element(),
        ),
        Status::Available { page, .. } => {
            let page = page.clone();
            Some(
                Button::new("update-page")
                    .xs()
                    .icon(Glyph::External)
                    .label("Download")
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
        .label("Check now")
        .on_click(cx.listener(|_, _, _, cx| MainView::command(Command::CheckForUpdates, cx)))
        .into_any_element();
    let shown = if *status == Status::Failed {
        ui::muted(line(status), cx).text_color(color(palette(cx).danger))
    } else {
        ui::muted(line(status), cx)
    };
    let row = || div().flex().items_center().justify_between().gap(px(SPACE_MD));
    ui::card(Glyph::Download, "Updates", Some(check), cx)
        .child(
            ui::padded_well(cx)
                .child(
                    row().child(ui::muted("Check for updates automatically", cx).flex_1()).child(
                        Switch::new("check-updates")
                            .large()
                            .checked(enabled)
                            .accessibility_label("Check for updates automatically")
                            .on_click(cx.listener(|view, on: &bool, _, cx| {
                                view.set_check_updates(*on, cx);
                            })),
                    ),
                )
                .child(row().child(shown.flex_1()).children(action(status, cx))),
        )
        .into_any_element()
}
