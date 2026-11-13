# User guide

The desktop, Android and iOS apps work the same way. Where they differ, this guide says so.

## The home screen

- **Send files**: drop files on it, or tap to choose them.
- **Devices**: devices you paired with, and whether each one is online now.
- **Nearby**: other devices running Solferry on this network or your Tailscale network, ready to pair or send to.
- **No app on the other device**: share files with a browser, or receive from one.

## Pairing

Pairing makes two devices remember each other. Paired devices find each other on every network they share, can use each other's relay, can send each other texts, and can choose to skip the question when sending files. You don't need to pair to send a file once.

**On the same network.** Press **Pair** next to a device under Nearby. Both screens show the same six digits. Check that they match, then confirm on the other device. A different code means you're talking to someone else, so decline.

**On different networks.** Choose **Use a code**, then **Show code**. Enter that code on the other device, or scan its QR code. A code works once, for 10 minutes, and carries your relay, so treat it like a password.

**By address.** If a device on your network doesn't show up, enter its IP address under Use a code. Each device shows its own address on that screen.

Paired devices stay paired until you choose **Forget** in a device's options.

## Sending

Pick the files first or the device first, whichever is quicker:

- Drop files on the window, or choose them, then pick one or more devices and press **Send**.
- Open a device's options and send from there.
- **Desktop:** drop files on the Dock icon on macOS, or click the menu bar or tray icon for a small popover with your devices and a file picker. `Ctrl+O` (`Cmd+O` on macOS) chooses files, `Ctrl+N` adds a device and `Ctrl+,` opens Settings.
- **macOS and Android:** share files to Solferry from any app's share sheet. Solferry opens straight into the send screen with them.

Folders keep their structure. Progress shows in a toast at the bottom, which opens into the full list.

**Interrupted transfers resume.** A dropped connection reconnects by itself. If either app restarts, send the same files to the same device again and the transfer picks up where it stopped. Everything already received is checked again first.

**Photos from iPhones.** HEIC photos are converted to JPEG when the receiving device runs Windows or Linux, which can't open HEIC without extra software. The conversion happens on the sending device, keeps the photo's date, location and orientation, and isn't noticeable at the quality used. Turn it off in Settings with **Convert HEIC photos to JPEG**. macOS, iOS and Android receive the original.

## Sending to a device that's away

If a paired device is offline, the send waits in a **Waiting to send** list on the home screen instead of failing. Solferry delivers it when the device comes back, and the other side is still asked first if it asks for that device. You get a notification when it's delivered or declined.

A waiting send can be cancelled from the list. It gives up after 7 days.

Delivery needs Solferry running on the sending device. On Android that means background receiving is on, or the app is open. iOS suspends apps soon after they leave the screen, so waiting sends go out the next time you open Solferry.

## Receiving

When someone sends you files, an offer appears with **Accept**, **Decline** and, for unpaired devices, **Block**. The notification has Accept and Decline too, so you can answer without opening Solferry.

For a paired device, **Always accept** skips the question from then on. You can change it later in that device's options. The choice is stored only on your device.

Received files go to your download folder, or the folder you chose in Settings. On macOS and Windows, received files carry the same download marker as files from the internet, so the system checks them before they first run.

## Texts and links

On a paired device's row, the clipboard button sends the text you copied. If that device always accepts from you, the text lands straight on its clipboard. Otherwise it waits there with a **Copy** button.

Links in received text are underlined and open when you tap them. Only `http`, `https` and `mailto` links are recognized, and nothing opens by itself.

## Sharing with a browser

For a device without Solferry, use **Share via link** on the send screen, or **Receive via link** on the home screen. Solferry shows a link and a QR code that any browser on the same network can open, to download your files or send you files and text. The link stops working when you close it.

## Visibility

**Let others find me** in Settings decides whether unpaired devices can see this one. Turned off, Solferry stays hidden from Nearby lists and only paired devices can reach it.

On Android, the **Visibility** Quick Settings tile turns it on and off from the notification shade. Long-press the tile to open Solferry.

## Blocking

**Block** on an offer refuses every future connection from that device, including pairing requests. Blocked devices are listed in Settings, where you can unblock them.

## History

The History tab lists finished and failed transfers in both directions. **Clear** empties it. History stays on this device.

## Settings

| Setting | What it does |
|---|---|
| Device name | What other devices see |
| Save files to | Where received files go |
| Permissions | Local network, notifications and, on Windows, the firewall, with a button to each system setting |
| Relay server | Relays you run, and a speed limit for traffic through them. See [Run a relay](relay.md) |
| Let others find me | Whether unpaired devices can see this one |
| Convert HEIC photos to JPEG | macOS, Android and iOS, on by default. For devices that can't open HEIC |
| Appearance | System, light or dark |
| Language | Follows the system until you pick one |
| Open at login, Show in tray | Desktop only |
| Receive in the background | Android only, keeps a notification while it runs |
| Updates | Desktop only, automatic checks and Check now |
| Blocked devices | Unblock a device |
