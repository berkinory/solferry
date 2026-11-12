# Changelog

Written by hand for people using Solferry. Each release's section becomes its GitHub release notes.

## Unreleased

### Added

- Browsers can send text through a receive link, not just files. It shows up in the link sheet with a copy button.
- Clicking a paired device opens its options, on desktop and Android, instead of a file picker.
- Two paired devices with the same name show their key next to their status, so you can tell which is which.
- A Permissions card in onboarding and Settings shows what the system allows Solferry: notifications everywhere, local network on macOS, the firewall on Windows. Each says whether it's on and opens the right settings page.
- Light, dark or system appearance in Settings, on desktop and Android 12 and later.
- Onboarding on Android offers to keep receiving in the background.

### Changed

- The desktop window and everything in it are smaller, closer to the apps around it.
- Sheets slide away when they close instead of vanishing, on desktop and Android.
- Android asks for notifications from onboarding instead of popping the prompt up on its own, and macOS asks from a button instead of at the first incoming file.
- The longest hints are a sentence each.
- The macOS menu bar icon follows the menu bar's color, is sharp on Retina screens and sits centered.
- The device key is kept in a file only you can read, on every platform, so macOS no longer asks for Keychain access. Devices paired with an earlier version on macOS or Windows need to be paired again.

### Fixed

- A device you just paired no longer stays in Nearby with a Pair button.
- Paired devices show as online right away when one is on a router's 2.4 GHz band and the other on 5 GHz, or when a phone drops multicast to save power. Before, they could look offline for minutes.
- Choosing a tab on desktop no longer also presses the button underneath it.
- Windows shows notifications for incoming files, texts and pairing requests. Before, it showed none.

## 0.0.2 - 2026-11-12

### Added

- A first-launch screen that asks what to call your device.
- Solferry in 15 languages: English, Türkçe, Deutsch, Français, Italiano, Español, Português, Русский, Українська, 日本語, 한국어, 简体中文, Nederlands, Polski and עברית. It follows your system, or you pick one in Settings and it switches right away.
- Every CLI command can print JSON with `--json`, errors included as one JSON line, and `receive` takes its answers over stdin. `--help` explains it all, so scripts and agents can drive Solferry.

### Changed

- Separate macOS downloads for Apple silicon and Intel, each smaller than one download for both.
- The Android app is 16 MB instead of 40 MB.
- Every download is smaller, compressed harder without losing anything, and the relay image is built on Alpine.

## 0.0.1 - 2026-11-12

Initial release.
