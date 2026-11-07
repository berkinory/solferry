use gpui_kit::component::{ActiveTheme, Theme, ThemeRegistry};
use gpui_kit::{App, Hsla, Rgba, rgb};
use solferry_core::Route;

use crate::tokens::{DARK, LIGHT};

const THEMES: &str = include_str!("theme.json");

/// Registers the generated themes, so every later appearance change loads them.
pub fn install(cx: &mut App) {
    let registry = ThemeRegistry::global_mut(cx);
    registry.load_themes_from_str(THEMES).expect("theme.json is generated from valid tokens");
    let named = |name: &str| registry.themes().get(name).cloned();
    let (light, dark) = (named("Solferry Light"), named("Solferry Dark"));
    let theme = Theme::global_mut(cx);
    theme.light_theme = light.expect("theme.json defines the light theme");
    theme.dark_theme = dark.expect("theme.json defines the dark theme");
}

/// The color that identifies how a transfer travels.
pub fn route_color(route: Route, cx: &App) -> Hsla {
    let palette = if cx.theme().is_dark() { &DARK } else { &LIGHT };
    let color: Rgba = rgb(match route {
        Route::Local => palette.route_local,
        Route::Overlay => palette.route_overlay,
        Route::Direct => palette.route_direct,
        Route::Relay => palette.route_relay,
    });
    color.into()
}
