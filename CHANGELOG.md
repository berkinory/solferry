# Changelog

Written by hand for people using Solferry. Each release's section becomes its GitHub release notes.

## Unreleased

### Added

- Solferry shows up in the share sheet on iPhone, iPad and Mac. Sharing photos or files to it opens the send sheet with them.
- A click on the tray or menu bar icon opens a small popover with your devices. Pick one and choose files, or choose files first. Right-click still shows the menu.
- Dropping files on the Dock icon opens the send sheet with them. On Windows and Linux, files passed to the app when it starts do the same.
- `solferry send -` sends whatever is piped in as one file. `--name` sets its name, otherwise it arrives as stdin.txt, or stdin.bin when it isn't text.
- `solferry send --to last` sends to the device this one last finished sending files to.
- Android has a Quick Settings tile that turns visibility on and off. Long-press it to open Solferry.
- Give your relay a new token without cutting anyone off: `deploy/relay/rotate.sh` prints a new link and keeps the old token working until you run `rotate.sh finish`. Paste the new link on one device and the devices paired with it pick it up the next time they reach it.
- Relay speed limit in Settings, and `solferry relay limit`, caps how fast files go through a relay. Direct connections are never limited.
- A copy button next to each relay. Relay links show as `https://relay.example.com#••••` so the token stays off screens.
- HEIC photos sent from iPhone, Android or a Mac to a Linux or Windows device arrive as JPEG, converted on the sending device at full quality with their date, location and orientation kept. Turn it off in Settings under "Convert HEIC photos to JPEG".
- Incoming file notifications have Accept and Decline buttons on Android, iOS, macOS and Windows, so you can answer without opening the app. The notification goes away once the offer is answered anywhere, withdrawn or expired. Linux keeps the plain notification.
- Links in received text are underlined and open when you tap or click them. Only links written out with http, https or mailto count, and nothing opens by itself.

### Changed

- `solferry receive --yes` only takes files from paired devices without asking. Unpaired devices are asked as usual, or refused with a line on stderr when there is no terminal to ask on.
- The relay server sets no speed limit of its own unless you set `RELAY_BYTES_PER_SECOND`.
- Adding a new link for a relay you already have replaces the old one.
- The relay is called a relay server, and "Relay" is no longer translated into words like "Röle". Other stiff translations were reworded in several languages.
- The language list has no "system default" entry anymore. The system's language starts out selected, and picking another one sticks.
- iOS catches up: onboarding, the Permissions card, the appearance setting, and the same translations as the other apps.

### Removed

- Hebrew. Without a right-to-left layout it read worse than English.

## 0.0.3 - 2026-11-12

### Added

- Browsers can send text through a receive link, not just files. It shows up in the link sheet with a copy button.
- Clicking a paired device opens its options, on desktop and Android, instead of a file picker.
- Two paired devices with the same name show their key next to their status, so you can tell which is which.
- A Permissions card in onboarding and Settings shows what the system allows Solferry: notifications everywhere, local network on macOS, the firewall on Windows. Each says whether it's on and opens the right settings page.
- Light, dark or system appearance in Settings, on desktop and Android 12 and later.
- Onboarding offers to keep receiving in the background on Android, and to open at login on desktop.

### Changed

- A new app icon: the mark as frosted glass, lit from behind.
- The desktop window and everything in it are smaller, closer to the apps around it.
- Sheets slide away when they close instead of vanishing, on desktop and Android.
- Android asks for notifications from onboarding instead of popping the prompt up on its own, and macOS asks from a button instead of at the first incoming file.
- The longest hints are a sentence each.
- The macOS menu bar icon follows the menu bar's color, is sharp on Retina screens and sits centered.
- The device key is kept in a file only you can read, on every platform, so macOS no longer asks for Keychain access. Devices paired with an earlier version on macOS or Windows need to be paired again.

### Fixed

- A device you just paired no longer stays in Nearby with a Pair button.
- Paired devices show as online or offline within seconds instead of within 20. A device that quits, or a phone that goes to the background, tells its paired devices, so they show it offline at once.
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
