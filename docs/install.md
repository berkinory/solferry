# Installation

Every release is on the [releases page](https://github.com/berkinory/solferry/releases). Each one has the same nine files:

| File | For |
|---|---|
| `Solferry-macos-aarch64.dmg` | Macs with Apple silicon, macOS 12 or later |
| `Solferry-macos-x86_64.dmg` | Intel Macs, macOS 12 or later |
| `Solferry-windows-setup.exe` | Windows 10 or later, x64 |
| `Solferry-linux-x86_64.AppImage` | Linux on x86_64 with glibc 2.35 or later |
| `Solferry-linux-aarch64.AppImage` | Linux on ARM64 with glibc 2.35 or later |
| `Solferry-android.apk` | Android 10 or later |
| `solferry-cli-linux-x86_64.tar.xz` | The command line tool alone, any x86_64 Linux |
| `solferry-cli-linux-aarch64.tar.xz` | The command line tool alone, any ARM64 Linux |
| `latest.json` | What the desktop apps read to update themselves |

## macOS

Open the `.dmg` and drag Solferry to Applications. The app is signed and notarized, so it opens without warnings.

On first launch macOS asks whether Solferry may find devices on your local network. Allow it, or nearby devices won't show up. You can change it later in System Settings, Privacy & Security, Local Network.

The `solferry` command ships inside the app. To put it on your PATH:

```sh
sudo ln -s /Applications/Solferry.app/Contents/MacOS/solferry /usr/local/bin/solferry
```

With Homebrew, `brew install --cask solferry` does both.

## Windows

Run `Solferry-windows-setup.exe`. The installer isn't code signed yet, so SmartScreen may warn about an unknown publisher: choose **More info**, then **Run anyway**. The installer also puts the `solferry` command on your PATH.

The first time Solferry receives, Windows Firewall may ask whether to allow it on private networks. Allow it, or other devices can't reach this one.

## Linux

Make the AppImage executable and run it:

```sh
chmod +x Solferry-linux-x86_64.AppImage
./Solferry-linux-x86_64.AppImage
```

Settings has options to open Solferry at login and to show it in the system tray. With the tray icon on, closing the window keeps Solferry running so it can receive. Turn it off and closing the window quits.

For the command line tool alone, unpack the tarball somewhere on your PATH:

```sh
tar xf solferry-cli-linux-x86_64.tar.xz
sudo mv solferry /usr/local/bin/
```

Or with Homebrew on Linux: `brew install solferry-cli`.

## Android

Download `Solferry-android.apk` on the phone and open it. Android asks once to allow installs from your browser or file manager.

Solferry asks for notifications, so incoming files and pairing requests can reach you. To receive while the app is closed, turn on background receiving in Settings. It keeps a quiet notification while it runs.

## iOS

The iOS app isn't released yet. It is built and tested on the simulator, and you can [build it yourself](building.md#ios) with Xcode.

## Updates

The desktop apps check for updates and download them in the background, then show a Restart button. Every update is verified against a signing key built into the app. Turn checking off in Settings, under Updates.

Android tells you when a new version is out and links to it. Install the new APK over the old one, and your pairings and settings stay.

## Where Solferry keeps things

| | Settings, keys and paired devices | Received files |
|---|---|---|
| macOS | `~/Library/Application Support/solferry` | `~/Downloads` |
| Windows | `%APPDATA%\solferry` | `Downloads` |
| Linux | `~/.local/share/solferry` | `~/Downloads` |
| Android | the app's private storage | `Downloads/Solferry`, or a folder you pick |

You can change the download folder in Settings. The command line tool uses the same data folder as the app on that machine, unless you give it `--data-dir`.
