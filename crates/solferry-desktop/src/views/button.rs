use gpui_kit::component::ActiveTheme;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::{
    App, ClickEvent, ElementId, FontWeight, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, black, div, px, white,
};

use super::ui::shadow;
use crate::assets::{Glyph, icon};
use crate::theme::{color, palette};
use crate::tokens::RADIUS_CONTROL;

const GROUP: &str = "button";

type Handler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Variant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Size {
    Xs,
    Sm,
    Md,
}

/// The app's one button, styled like invisit's: a lit primary, a framed secondary, quiet ghosts.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    glyph: Option<Glyph>,
    variant: Variant,
    size: Size,
    grow: bool,
    tooltip: Option<SharedString>,
    on_click: Option<Handler>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            glyph: None,
            variant: Variant::Secondary,
            size: Size::Md,
            grow: false,
            tooltip: None,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, glyph: Glyph) -> Self {
        self.glyph = Some(glyph);
        self
    }

    pub fn primary(mut self) -> Self {
        self.variant = Variant::Primary;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.variant = Variant::Ghost;
        self
    }

    pub fn danger(mut self) -> Self {
        self.variant = Variant::Danger;
        self
    }

    pub fn xs(mut self) -> Self {
        self.size = Size::Xs;
        self
    }

    pub fn sm(mut self) -> Self {
        self.size = Size::Sm;
        self
    }

    pub fn grow(mut self) -> Self {
        self.grow = true;
        self
    }

    pub fn tooltip(mut self, text: impl Into<SharedString>) -> Self {
        self.tooltip = Some(text.into());
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

struct Look {
    fill: Option<Hsla>,
    hover: Hsla,
    text: Hsla,
    hover_text: Hsla,
    border: Option<Hsla>,
}

fn look(variant: Variant, cx: &App) -> Look {
    let theme = cx.theme();
    let palette = palette(cx);
    match variant {
        Variant::Primary => Look {
            fill: Some(color(palette.primary_fill)),
            hover: color(palette.primary_hover),
            text: color(palette.on_primary),
            hover_text: color(palette.on_primary),
            border: Some(color(palette.primary_border)),
        },
        Variant::Secondary => Look {
            fill: Some(theme.background),
            hover: theme.accent,
            text: theme.foreground,
            hover_text: theme.foreground,
            border: Some(theme.border),
        },
        Variant::Ghost => Look {
            fill: None,
            hover: theme.accent,
            text: theme.muted_foreground,
            hover_text: theme.foreground,
            border: None,
        },
        Variant::Danger => Look {
            fill: Some(color(palette.danger_fill)),
            hover: color(palette.danger_fill).opacity(0.9),
            text: color(palette.on_primary),
            hover_text: color(palette.on_primary),
            border: None,
        },
    }
}

impl RenderOnce for Button {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let (height, padding, text, glyph_size) = match self.size {
            Size::Xs => (28.0, 8.0, 12.0, 14.0),
            Size::Sm => (32.0, 12.0, 13.0, 16.0),
            Size::Md => (36.0, 14.0, 14.0, 16.0),
        };
        let look = look(self.variant, cx);
        let (hover, hover_text) = (look.hover, look.hover_text);
        let square = self.label.is_none();
        let mut button = div()
            .id(self.id)
            .relative()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(6.))
            .h(px(height))
            .rounded(px(RADIUS_CONTROL))
            .text_size(px(text))
            .font_weight(FontWeight::MEDIUM)
            .text_color(look.text)
            .group(GROUP)
            .cursor_pointer()
            .hover(move |style| style.bg(hover).text_color(hover_text));
        button = if square { button.w(px(height)) } else { button.px(px(padding)) };
        if self.grow {
            button = button.flex_1();
        }
        if let Some(fill) = look.fill {
            button = button.bg(fill);
        }
        if let Some(border) = look.border {
            button = button.border_1().border_color(border);
        }
        match self.variant {
            Variant::Primary => {
                let fill = look.fill.unwrap_or(hover);
                let deep = color(palette(cx).primary_deep).opacity(0.3);
                button = button.active(move |style| style.bg(fill)).shadow(vec![
                    shadow(white().opacity(0.22), 1., 0., 0., true),
                    shadow(deep, -1., 0., 0., true),
                ]);
            }
            Variant::Secondary => {
                button = button.shadow(vec![shadow(black().opacity(0.05), 1., 2., 0., false)]);
            }
            _ => {}
        }
        if let Some(glyph) = self.glyph {
            button = button.child(
                icon(glyph, glyph_size)
                    .text_color(look.text)
                    .group_hover(GROUP, move |style| style.text_color(hover_text)),
            );
        }
        if let Some(label) = self.label {
            button = button.child(label);
        }
        if let Some(text) = self.tooltip {
            button = button.tooltip(move |window, cx| Tooltip::new(text.clone()).build(window, cx));
        }
        if let Some(handler) = self.on_click {
            button = button.on_click(handler);
        }
        button
    }
}
