# beam

Accountless, open source file transfer between your devices and the people you trust. It works on the local network and over Tailscale or a similar overlay, with no server involved. Run your own relay and it reaches across the internet too.

There is no account and no upload step. Files go straight from one device to another, end-to-end encrypted.

> Early development. The core, CLI and desktop app work on Linux, Windows and macOS builds. Android comes next, iOS last.

## Why

| | beam | LocalSend | Blip |
|---|---|---|---|
| Open source | yes | yes | no |
| No account | yes | yes | no |
| Same network | yes | yes | yes |
| Tailscale and overlays | yes | yes | no |
| Across the internet | with your own relay | no | yes |
| Resume after interruption | yes | no | yes |

## Desktop app

```sh
cargo run --release -p beam-desktop
```

Drop files on a device, or on the window to pick several devices. Closing the window keeps beam in the tray, ready to receive.

## Command line

```sh
cargo install --path crates/beam-cli
```

On the first device:

```sh
beam pair
```

On the second device, paste the code it printed:

```sh
beam join beam1...
```

Then keep one side receiving and send from the other:

```sh
beam receive
beam send holiday-photos/ --to my-laptop
```

Send to several devices with repeated `--to`.

## Who can send you files

| | Paired | Not paired |
|---|---|---|
| Same network or overlay | received right away | asks you, and you can block them |
| Through a relay | received right away | refused, you never see it |

To be asked for a paired device too, use `beam pair --ask`, `beam join <code> --ask` or `beam approval <device> ask`. This choice stays on your device, a pairing code cannot change it. `beam block` and `beam unblock` manage blocked devices.

## Firewalls

A strict firewall can block devices from finding each other on the same network. Allow incoming UDP on port 53318.

## Relays

beam ships with no servers and contacts none by default. To reach devices on other networks, use Tailscale or a similar overlay, or run your own relay, see [docs/relay.md](docs/relay.md). One person sets it up and everyone they pair with uses it automatically.

## Repository

| Path | Purpose |
|---|---|
| `crates/beam-proto` | Wire messages and framing |
| `crates/beam-core` | Identity, trust, discovery, transfers |
| `crates/beam-cli` | The `beam` command |
| `crates/beam-desktop` | Desktop app built with GPUI |
| `deploy/relay` | Self-hosted relay |
| `tests/netlab` | NAT, loss and outage scenarios in Linux namespaces |
| `tests/desktop` | Driver for real input against the desktop app |
| `docs` | Architecture, protocol and relay guides |

Read [docs/architecture.md](docs/architecture.md) to understand the code, and [CONTRIBUTING.md](CONTRIBUTING.md) to work on it.

## License

Apache-2.0
