//! The Dock icon on macOS, which only shows while the window is open when the menu bar icon is on.

/// Shows or hides the app in the Dock and the app switcher. Does nothing off macOS.
pub fn show(visible: bool) {
    #[cfg(target_os = "macos")]
    {
        use objc2::MainThreadMarker;
        use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};

        let Some(main) = MainThreadMarker::new() else { return };
        let policy = if visible {
            NSApplicationActivationPolicy::Regular
        } else {
            NSApplicationActivationPolicy::Accessory
        };
        NSApplication::sharedApplication(main).setActivationPolicy(policy);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = visible;
}
