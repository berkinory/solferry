# beam

Accountless, open source file transfer between your devices and the people you trust. It finds the fastest path by itself: the local network, Tailscale or a similar overlay, or a direct connection across the internet.

There is no account and no upload step. Files go straight from one device to another, end-to-end encrypted.

> Early development. The core and CLI work. Desktop and Android apps come next, iOS last.

## Why

| | beam | LocalSend | Blip |
|---|---|---|---|
| Open source | yes | yes | no |
| No account | yes | yes | no |
| Same network | yes | yes | yes |
| Across the internet | yes | no | yes |
| Resume after interruption | yes | no | yes |

## Quick start

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

Send to several devices with repeated `--to`. Devices on the same network can be reached without pairing, see `beam nearby`, and each transfer asks for confirmation.

## Relays

By default servers only introduce two devices, and file data never passes through them. When two networks refuse direct connections, beam tells you instead of silently routing your files elsewhere. You then have three options:

- Use Tailscale or a similar overlay network.
- Run your own relay, see [docs/relay.md](docs/relay.md). Adding it allows your data through that relay only.
- Allow public relays with `beam relay policy always`.

`beam relay policy off` removes servers completely, which leaves the local network and overlay networks.

## Repository

| Path | Purpose |
|---|---|
| `crates/beam-proto` | Wire messages and framing |
| `crates/beam-core` | Identity, trust, discovery, transfers |
| `crates/beam-cli` | The `beam` command |
| `deploy/relay` | Self-hosted relay |
| `tests/netlab` | NAT, loss and outage scenarios in Linux namespaces |
| `docs` | Architecture, protocol and relay guides |

Read [docs/architecture.md](docs/architecture.md) to understand the code, and [CONTRIBUTING.md](CONTRIBUTING.md) to work on it.

## License

Apache-2.0
