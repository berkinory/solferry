use gpui_kit::{App, KeyBinding, Menu, MenuItem};
use rust_i18n::t;

gpui_kit::actions!(solferry, [Back, ChooseFiles, AddDevice, OpenSettings, Quit]);

pub const CONTEXT: &str = "Solferry";

const KEYS: [&str; 4] = if cfg!(target_os = "macos") {
    ["cmd-o", "cmd-n", "cmd-,", "cmd-q"]
} else {
    ["ctrl-o", "ctrl-n", "ctrl-,", "ctrl-q"]
};

pub fn install(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", Back, Some(CONTEXT)),
        KeyBinding::new(KEYS[0], ChooseFiles, Some(CONTEXT)),
        KeyBinding::new(KEYS[1], AddDevice, Some(CONTEXT)),
        KeyBinding::new(KEYS[2], OpenSettings, Some(CONTEXT)),
        KeyBinding::new(KEYS[3], Quit, None),
    ]);
    cx.on_action(|_: &Quit, cx| crate::shell::quit(cx));
    cx.set_menus([Menu {
        name: "Solferry".into(),
        items: vec![
            MenuItem::action(t!("menu.choose_files"), ChooseFiles),
            MenuItem::action(t!("menu.pair_with_code"), AddDevice),
            MenuItem::action(t!("menu.settings"), OpenSettings),
            MenuItem::separator(),
            MenuItem::action(t!("menu.quit"), Quit),
        ],
        disabled: false,
    }]);
}
