# Changelog

Written by hand for people using Solferry. Each release's section becomes its GitHub release notes.

## Unreleased

### Changed

- The desktop app is smaller, closer to the size of the apps around it.
- Sheets slide away when they close, on desktop and Android.
- The macOS menu bar icon follows the menu bar's color and is sharp on Retina screens.
- The device key is kept in a file only you can read, on every platform, so macOS no longer asks for Keychain access. Devices paired with an earlier version on macOS or Windows need to be paired again.

## 0.0.2 - 2026-11-12

### Added

- A first-launch screen that asks what to call your device.
- Solferry in 15 languages, following your system or chosen in Settings: English, Türkçe, Deutsch, Français, Italiano, Español, Português, Русский, Українська, 日本語, 한국어, 简体中文, Nederlands, Polski and עברית.
- Every CLI command can answer in JSON, and `receive` takes its answers over stdin, so scripts and agents can drive Solferry.

### Changed

- Separate macOS downloads for Apple silicon and Intel.
- The Android app is 16 MB instead of 40 MB, and every download is smaller.

## 0.0.1 - 2026-11-12

Initial release.
