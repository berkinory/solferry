use gpui_kit::component::{ActiveTheme, Theme};
use gpui_kit::{App, Hsla, Rgba, rgb};
use solferry_core::Route;

use crate::tokens::{DARK, LIGHT, Palette};

fn color(value: u32) -> Hsla {
    let color: Rgba = rgb(value);
    color.into()
}

/// The color that identifies how a transfer travels.
pub fn route_color(route: Route, cx: &App) -> Hsla {
    let palette = if cx.theme().is_dark() { &DARK } else { &LIGHT };
    color(match route {
        Route::Local => palette.route_local,
        Route::Overlay => palette.route_overlay,
        Route::Direct => palette.route_direct,
        Route::Relay => palette.route_relay,
    })
}

/// Applies the design tokens over the toolkit's default theme for the current appearance.
pub fn apply(cx: &mut App) {
    Theme::update(cx, |theme| {
        let palette: &Palette = if theme.is_dark() { &DARK } else { &LIGHT };
        let sun = color(palette.sun);
        let on_sun = color(palette.on_sun);
        let surface = color(palette.surface);
        let raised = color(palette.surface_raised);
        let ink = color(palette.ink);
        let muted = color(palette.ink_muted);
        let line = color(palette.line);

        theme.background = color(palette.canvas);
        theme.foreground = ink;
        theme.border = line;
        theme.input = line;
        theme.ring = sun;
        theme.caret = ink;
        theme.primary = sun;
        theme.primary_foreground = on_sun;
        theme.primary_hover = sun.opacity(0.9);
        theme.primary_active = sun.opacity(0.8);
        theme.button_primary = sun;
        theme.button_primary_foreground = on_sun;
        theme.button_primary_hover = sun.opacity(0.9);
        theme.button_primary_active = sun.opacity(0.8);
        theme.secondary = surface;
        theme.secondary_foreground = ink;
        theme.secondary_hover = raised;
        theme.secondary_active = raised;
        theme.accent = raised;
        theme.accent_foreground = ink;
        theme.muted = raised;
        theme.muted_foreground = muted;
        theme.title_bar = theme.background;
        theme.title_bar_border = line;
        theme.danger = color(palette.danger);
        theme.success = color(palette.success);
        theme.warning = color(palette.warning);
        theme.link = color(palette.harbor);
    });
}
