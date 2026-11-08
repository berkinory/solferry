use gpui_kit::{App, KeyBinding, Menu, MenuItem};

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
            MenuItem::action("Choose Files...", ChooseFiles),
            MenuItem::action("Pair with a Code...", AddDevice),
            MenuItem::action("Settings...", OpenSettings),
            MenuItem::separator(),
            MenuItem::action("Quit Solferry", Quit),
        ],
        disabled: false,
    }]);
}
