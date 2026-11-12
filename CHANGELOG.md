# Changelog

Notable changes are recorded here. The format follows Keep a Changelog.

## Unreleased

### Added

- Device identity, pairing and trust store
- Resumable, verified file transfers over iroh
- Local-first networking: no servers by default, optional self-hosted relays shared through pairing
- Per-device approval and blocking
- Pairing nearby devices by comparing a code
- Tailscale devices listed automatically
- Android option to keep receiving while the app is closed
- iOS app in SwiftUI
- Transfer history, a separate send screen and tab navigation
- `solferry relay check` and a one-command relay installer
- Live route updates during a transfer
- `solferry` command with JSON event output
- Self-hosted relay deployment
- Network scenarios for NAT, loss, outages and restarts
- Desktop app for Linux, Windows and macOS with tray, pairing and settings
- Online and offline status for paired devices
- "Always accept" on incoming files, so pairing no longer grants it
- Android: your relays, blocked devices, long transfers as user-initiated jobs, signed release APK
- Device platforms shared on pairing and discovery, shown as platform icons
- Clipboard text sharing with paired devices, from the desktop and with `solferry send --text`
- Large files sent as up to 16 parallel segments, each verified and resumable on its own
- Identity colors and key fingerprints, so devices with the same name still look different
- Abandoned partial transfers cleared at startup
- Android app: send from the picker or share sheet, pair by code or nearby, share clipboard text, receive into Downloads/Solferry
