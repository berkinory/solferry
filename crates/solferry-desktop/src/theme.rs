use std::borrow::Cow;

use gpui_kit::component::{ActiveTheme, Theme, ThemeMode, ThemeRegistry};
use gpui_kit::{App, Hsla, Rgba, Window, rgb};
use solferry_core::Appearance;

use crate::tokens::{DARK, LIGHT, Palette};

const THEMES: &str = include_str!("theme.json");
const FONTS: [&[u8]; 2] = [
    include_bytes!("../assets/fonts/InterTight-Regular.ttf"),
    include_bytes!("../assets/fonts/InterTight-Medium.ttf"),
];

/// Embeds the fonts and registers the generated themes, so every later appearance change loads them.
pub fn install(cx: &mut App) {
    let fonts = FONTS.iter().map(|font| Cow::Borrowed(*font)).collect();
    cx.text_system().add_fonts(fonts).expect("the embedded fonts are valid");
    let registry = ThemeRegistry::global_mut(cx);
    registry.load_themes_from_str(THEMES).expect("theme.json is generated from valid tokens");
    let named = |name: &str| registry.themes().get(name).cloned();
    let (light, dark) = (named("Solferry Light"), named("Solferry Dark"));
    let theme = Theme::global_mut(cx);
    theme.light_theme = light.expect("theme.json defines the light theme");
    theme.dark_theme = dark.expect("theme.json defines the dark theme");
}

/// Draws the app light or dark, or as the system does now.
pub fn apply(appearance: Appearance, window: &mut Window, cx: &mut App) {
    match appearance {
        Appearance::System => Theme::sync_system_appearance(Some(window), cx),
        Appearance::Light => Theme::change(ThemeMode::Light, Some(window), cx),
        Appearance::Dark => Theme::change(ThemeMode::Dark, Some(window), cx),
    }
}

pub fn palette(cx: &App) -> &'static Palette {
    if cx.theme().is_dark() { &DARK } else { &LIGHT }
}

pub fn color(value: u32) -> Hsla {
    let color: Rgba = rgb(value);
    color.into()
}
