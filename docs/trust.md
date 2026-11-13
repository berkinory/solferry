# Trust and privacy

Solferry has no accounts and no servers of its own. This page explains who can reach your device, what protects your files and what anyone in between can see.

## Devices are keys

Each device creates an Ed25519 key on first launch and is identified by it from then on. Names are only labels: two devices can share a name, but never a key. Every connection is authenticated against the other device's key, so a device can't pretend to be one you paired with.

The key lives in the data folder, readable only by your user, the way SSH keeps keys. See [where Solferry keeps things](install.md#where-solferry-keeps-things).

## Who can send you files

| | Paired | Not paired |
|---|---|---|
| Same network or Tailscale | asks you, or arrives right away if you chose that for the device | asks you, and you can block them |
| Through a relay | asks you, or arrives right away if you chose that for the device | refused, you never see it |

- **Pairing never grants anything on its own.** A paired device asks until you pick **Always accept** on one of its offers, or run `solferry approval <device> auto`. That choice is stored only on your device. A pairing code can't set it.
- **Unpaired devices can only knock on a local network or an overlay you joined**, and only while **Let others find me** is on.
- **Blocked devices are refused** before they can send a single byte of file information.

## What is protected

- **Encryption.** File contents, names and texts are end-to-end encrypted and authenticated with TLS 1.3 over QUIC. Relays and networks in between see only encrypted packets.
- **Integrity.** Every file is checked against the sender's BLAKE3 hash before it's moved into place. A damaged or tampered transfer is caught and never delivered.
- **Partial files.** Half-received files stay in a hidden folder until everything is verified, so you never see half a file. They belong to the device that started them, so no other device can continue or poison them.
- **Paths.** Names and folders from the sender are checked before anything touches your disk. A sender can't write outside the folder you receive into, or through a link.
- **Downloads.** On macOS and Windows, received files carry the same marker as files from the internet, so the system checks them before they first run.

## What others can see

- **Your network.** Devices on the same network see your device's name while **Let others find me** is on. Turn it off and only paired devices can find you.
- **Tailscale.** Devices on your Tailscale network see the same as the local network.
- **A relay.** Whoever runs a relay can see IP addresses, device ids, timing and how much data goes through, never what's in it. Solferry uses only relays you added and relays a paired device gave you, and contacts no server at all without one.
- **Pairing codes.** A code carries your relay and its access token, so share it only with people you'd let use your relay. It works once, for 10 minutes.

## What Solferry doesn't do

- It contacts no server unless you add a relay. The desktop apps check GitHub for updates, which you can turn off in Settings.
- It collects no analytics, crash reports or usage data.
- Without a relay, it never asks your router to open a port.

## Limits

Solferry hasn't had an independent security audit yet. Please report vulnerabilities privately, as described in [SECURITY.md](../SECURITY.md).
