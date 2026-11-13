# Building from source

You need stable Rust. The core and the CLI support Rust 1.91, and the desktop app needs 1.98 or newer, following GPUI. [CONTRIBUTING.md](../CONTRIBUTING.md) lists the tools for linting and tests.

```sh
git clone https://github.com/berkinory/solferry.git
cd solferry
```

## Command line

```sh
cargo install --path crates/solferry-cli
```

## Desktop

```sh
cargo run --release -p solferry-desktop
```

On Linux, the desktop app needs the development packages for X11, Wayland, Vulkan, FreeType and fontconfig. On Ubuntu:

```sh
sudo apt-get install libxkbcommon-x11-dev libwayland-dev libvulkan-dev libfreetype-dev libfontconfig-dev
```

On macOS, run it as an app bundle, so it can show notifications, ask for local network access and include the share extension:

```sh
just bundle-macos
open target/release/Solferry.app
```

The bundle is signed ad hoc unless `SOLFERRY_SIGN_IDENTITY` names a Developer ID certificate.

## Android

```sh
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk
cd android && ./gradlew installDebug
```

Needs JDK 21 and the Android SDK with an NDK. Gradle builds the Rust core through `android/build-rust.sh` and generates its Kotlin bindings.

## iOS

```sh
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
ios/build-rust.sh
open ios/Solferry.xcodeproj
```

Needs Xcode on a Mac. `ios/build-rust.sh` writes the Rust core and its Swift bindings to `ios/Generated`, so run it again after changing the core. The share extension shares an App Group with the app, and finding nearby devices on a real iPhone needs Apple's multicast entitlement.

## Relay

The relay is iroh's relay server in a container. See [Run a relay](relay.md) and `deploy/relay`.

## Repository layout

| Path | Contents |
|---|---|
| `crates/solferry-proto` | Wire messages and framing |
| `crates/solferry-core` | Identity, trust, discovery, transfers, the queue |
| `crates/solferry-cli` | The `solferry` command |
| `crates/solferry-desktop` | Desktop app built with GPUI |
| `crates/solferry-ffi` | UniFFI bridge the mobile apps call into |
| `android` | Android app in Kotlin and Compose |
| `ios` | iOS app and share extension in SwiftUI |
| `packaging` | macOS bundle and share extension, Windows installer, Linux AppImage, Homebrew |
| `deploy` | Self-hosted relay and a systemd unit for headless receiving |
| `design` | Design tokens, icons and the generated themes |
| `gallery` | Screenshots of every screen on every platform |
| `tests/netlab` | NAT, loss and outage scenarios in Linux network namespaces |
| `tests/desktop` | Driver for real input against the desktop app |
| `docs` | This documentation |

Read [Architecture](architecture.md) for how the pieces fit together.
