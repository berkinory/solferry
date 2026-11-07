# Architecture

beam is one Rust core with thin clients around it. Everything hard lives in `beam-core`, so each client only draws screens and talks to its platform.

```text
 CLI, desktop, Android next
              |
          beam-core        identity, trust, discovery, transfers
              |
          beam-proto       wire format
              |
            iroh           QUIC, hole punching, relays
```

## Identity and trust

A device is its Ed25519 key, stored in `device.key` in the data directory. There are no accounts. iroh authenticates every connection against that key, so a device cannot be impersonated.

`devices.json` lists paired devices. A device is either `own` or `contact`. Clients decide what that means: the CLI accepts transfers from your own devices automatically and asks for everything else.

Peers that are not paired can only reach you over a local address, and only while visibility is `everyone`. Anything else is closed before a byte of file metadata is read.

## Reaching a device

iroh dials by public key and picks the best path. beam adds three address sources:

- mDNS for the local network
- n0 DNS and the Mainline DHT for the internet
- relays stored at pairing time

Overlay networks such as Tailscale need no special code, because they are ordinary network interfaces.

## Relay policy

A relay plays two roles: it helps two devices punch through NAT, and it can carry data when that fails. `RelayPolicy` separates them.

| Policy | Servers used | File data over relays |
|---|---|---|
| `handshake-only` (default) | yes | no |
| `own-relays` | yes | relays you added or a paired device shared |
| `always` | yes | yes |
| `off` | no | no |

`route.rs` enforces this on the sending side. It gives hole punching a head start, waits for an allowed path before file data flows, and fails the transfer with `NoDirectPath` if the connection stays on a forbidden relay.

## Transfers

The sender scans its paths into a `Manifest`. The transfer id is a hash of the sender and the file list, so sending the same files again resumes automatically.

Each file travels on its own QUIC stream, up to 16 at a time. Reading, writing and hashing run on blocking threads connected to the network tasks by bounded channels, which keeps the async runtime free and memory use flat.

Received data stays in `.beam-partial/<id>/` inside the destination. Files move to their final names only after every file has been verified, so an interrupted transfer never leaves half a file in place.

A resumed file is hashed from the first byte on both sides, so a damaged partial file is caught and sent again. The same check covers bit rot and a source file that changes between attempts.

## Congestion control

Connections use BBR. Consumer Wi-Fi and mobile links lose packets for reasons unrelated to congestion, and loss-based Cubic collapses there. In `tests/netlab`, a 300 Mbit link with 0.5% loss moved 26 MiB/s with BBR and 5 MiB/s with Cubic, and a 200 Mbit link with 1% loss and 80 ms RTT moved 11 MiB/s with BBR and did not finish 300 MiB in 200 seconds with Cubic.

## Stream window

Each stream has a 2 MiB receive window. noq closes a connection that exceeds 1024 gaps in one stream, and a larger window makes that reachable under heavy packet loss. See the constant in `net.rs` before changing it. The cost is throughput on high latency links, which parallel streams per file can solve later.

## Desktop

`beam-desktop` runs the core on its own thread with a tokio runtime. `service.rs` is the only bridge: the UI sends `Command`s and receives `Update`s over channels, and `state.rs` turns updates into plain data that views render. Closing the window keeps the process and tray alive, because the app exists to receive files.

## Errors

`beam_core::Error` is an exhaustive enum and is safe to match on. Clients must turn every variant into a message that says what happened and what to do next. The CLI does this in `explain.rs` with no catch-all arm, so adding a variant fails compilation until it has a message.
