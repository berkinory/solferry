//! The first launch screen: what Solferry is, and the name nearby devices will see.

use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{ActiveTheme, Sizable};
use gpui_kit::{
    AppContext, Context, Entity, FontWeight, IntoElement, ParentElement, Render, Styled, Window,
    div, px, svg,
};
use rust_i18n::t;

use super::button::Button;
use super::ui;
use crate::assets::MARK;
use crate::service::Command;
use crate::shell::Shell;
use crate::state::AppState;
use crate::theme::{color, palette};
use crate::tokens::{SPACE_LG, SPACE_SM, SPACE_XL, TYPE_BODY};

const WIDTH: f32 = 400.0;

pub struct Onboarding {
    state: Entity<AppState>,
    name: Entity<InputState>,
}

impl Onboarding {
    pub fn new(
        state: Entity<AppState>,
        name: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let name = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(t!("settings.device.name_placeholder"))
                .default_value(name)
        });
        name.update(cx, |input, cx| input.focus(window, cx));
        cx.subscribe_in(&name, window, |view, _, event, _, cx| match event {
            InputEvent::PressEnter { .. } => view.finish(cx),
            InputEvent::Change => cx.notify(),
            _ => {}
        })
        .detach();
        Self { state, name }
    }

    fn finish(&self, cx: &mut Context<Self>) {
        let name = self.name.read(cx).value().trim().to_owned();
        let Some(mut config) = self.state.read(cx).config.clone() else { return };
        if name.is_empty() {
            return;
        }
        config.name = name;
        config.onboarded = true;
        self.state.update(cx, |state, cx| {
            state.config = Some(config.clone());
            cx.notify();
        });
        cx.global::<Shell>().service.send(Command::Apply(config));
    }
}

impl Render for Onboarding {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let blank = self.name.read(cx).value().trim().is_empty();
        let name = ui::padded_well(cx)
            .child(ui::text(t!("onboarding.name_label")).font_weight(FontWeight::MEDIUM))
            .child(Input::new(&self.name).large())
            .child(ui::caption(t!("onboarding.name_hint"), cx));
        let start = Button::new("get-started").label(t!("onboarding.start"));
        let start = if blank {
            start
        } else {
            start.primary().on_click(cx.listener(|view, _, _, cx| view.finish(cx)))
        };
        div().size_full().flex().items_center().justify_center().p(px(SPACE_LG)).child(
            div()
                .w_full()
                .max_w(px(WIDTH))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(SPACE_SM))
                .child(svg().path(MARK).size(px(56.)).text_color(color(palette(cx).brand)))
                .child(
                    div()
                        .mt(px(SPACE_SM))
                        .text_size(px(26.))
                        .font_weight(FontWeight::MEDIUM)
                        .child(t!("onboarding.title")),
                )
                .child(
                    div()
                        .text_center()
                        .text_size(px(TYPE_BODY))
                        .text_color(cx.theme().muted_foreground)
                        .child(t!("onboarding.body")),
                )
                .child(div().w_full().mt(px(SPACE_XL)).child(name))
                .child(div().mt(px(SPACE_LG)).child(start)),
        )
    }
}
