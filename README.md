# solferry

Accountless, open source file transfer between your devices and the people you trust. It works on the local network and over Tailscale or a similar overlay, with no server involved. Run your own relay and it reaches across the internet too.

There is no account and no upload step. Files go straight from one device to another, end-to-end encrypted.

> Early development. The core, CLI and desktop app work on Linux, Windows and macOS builds, and the Android app on Android 10 and later. The iOS app is written and builds, but has not been tried on a device yet.

## Why

| | solferry | LocalSend | Blip |
|---|---|---|---|
| Open source | yes | yes | no |
| No account | yes | yes | no |
| Same network | yes | yes | yes |
| Tailscale and overlays | yes | yes | no |
| Across the internet | with your own relay | no | yes |
| Resume after interruption | yes | no | yes |

## Install

Downloads are on the [releases page](https://github.com/berkinory/solferry/releases). On macOS and Linux, Homebrew works too:

```sh
brew tap berkinory/brew
brew trust berkinory/brew
brew install --cask solferry   # the macOS app, with the solferry command
brew install solferry-cli      # the command line tool alone, on Linux
```

On Windows the installer puts the `solferry` command on your PATH too.

The desktop apps update themselves unless you turn that off in Settings.

## Desktop app

```sh
cargo run --release -p solferry-desktop
```

Drop files on a device, or on the window to pick several devices. Closing the window keeps Solferry in the tray, ready to receive. The clipboard button on a paired device sends the text you copied: it lands on that device's clipboard if it receives without asking, and waits there to be copied otherwise. `Ctrl+O` (`Cmd+O` on macOS) chooses files, `Ctrl+N` adds a device and `Ctrl+,` opens settings.

On macOS, run it as an app so it can show notifications and ask for local network access:

```sh
just bundle-macos
open target/release/Solferry.app
```

## iOS

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
ios/build-rust.sh
open ios/Solferry.xcodeproj
```

Needs Xcode on a Mac. `ios/build-rust.sh` writes the Rust core and its Swift bindings to `ios/Generated`. Run it again after changing the core. Finding nearby devices on a real iPhone needs Apple's multicast entitlement.

## Android

```sh
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk
cd android && ./gradlew installDebug
```

Needs JDK 21 and the Android SDK with an NDK. Gradle builds the Rust core through `android/build-rust.sh` and generates its Kotlin bindings. Share files to Solferry from any app, or pick them from the send button. Received files land in Downloads/Solferry.

## Command line

```sh
cargo install --path crates/solferry-cli
```

On the same network, pair by comparing a code. Keep one device receiving, then on the other:

```sh
solferry pair --with my-laptop
```

Across networks, or to use a QR code, run `solferry pair` and paste what it prints on the other device:

```sh
solferry join sf1...
```

Then keep one side receiving and send from the other:

```sh
solferry receive
solferry send holiday-photos/ --to my-laptop
```

Send to several devices with repeated `--to`. Share text with a paired device's clipboard, or pipe it in with `-`:

```sh
solferry send --text "see you at 8" --to my-laptop
git log -1 | solferry send --text - --to my-laptop
```

## Who can send you files

| | Paired | Not paired |
|---|---|---|
| Same network or Tailscale | received right away | asks you, and you can block them |
| Through a relay | received right away | refused, you never see it |

Pairing never grants this on its own. A paired device still asks until you choose "Always accept" on one of its offers or run `solferry approval <device> auto`. This choice stays on your device. `solferry block` and `solferry unblock` manage blocked devices.

Tailscale devices running solferry show up by themselves, next to devices on your network.

`solferry receive --yes` takes files from every paired device without asking. Unpaired devices are still asked, and on a machine with no terminal to ask on they are refused with one line on stderr.

To keep a server or headless machine receiving, use [deploy/systemd/solferry.service](deploy/systemd/solferry.service).

## Firewalls

A strict firewall can block devices from finding each other on the same network. Allow incoming UDP on port 53318.

## Relays

solferry ships with no servers and contacts none by default. To reach devices on other networks, use Tailscale or a similar overlay, or run your own relay, see [docs/relay.md](docs/relay.md). One person sets it up and everyone they pair with uses it automatically.

## Repository

| Path | Purpose |
|---|---|
| `crates/solferry-proto` | Wire messages and framing |
| `crates/solferry-core` | Identity, trust, discovery, transfers |
| `crates/solferry-cli` | The `solferry` command |
| `crates/solferry-desktop` | Desktop app built with GPUI |
| `crates/solferry-ffi` | UniFFI bridge the mobile apps call into |
| `android` | Android app in Kotlin and Compose |
| `ios` | iOS app in SwiftUI |
| `deploy/relay` | Self-hosted relay |
| `tests/netlab` | NAT, loss and outage scenarios in Linux namespaces |
| `tests/desktop` | Driver for real input against the desktop app |
| `docs` | Architecture, protocol and relay guides |

Read [docs/architecture.md](docs/architecture.md) to understand the code, and [CONTRIBUTING.md](CONTRIBUTING.md) to work on it.

## License

MIT
