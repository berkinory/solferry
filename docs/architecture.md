# Architecture

solferry is one Rust core with thin clients around it. Everything hard lives in `solferry-core`, so each client only draws screens and talks to its platform.

```text
 CLI, desktop, Android, iOS
              |
          solferry-core        identity, trust, discovery, transfers
              |
          solferry-proto       wire format
              |
            iroh           QUIC, hole punching, relays
```

## Identity and trust

A device is its Ed25519 key, stored in `device.key` in the data directory. There are no accounts. iroh authenticates every connection against that key, so a device cannot be impersonated.

`devices.json` lists paired devices, each with its name, platform, relays, last known addresses and whether its files arrive without asking. That last flag starts off and only ever changes on this device.

Peers that are not paired can only reach you over a local address or an overlay you joined, and only while visibility is `everyone`. Blocked devices and everything else are closed before a byte of file metadata is read.

Whether a paired device needs approval is stored per device, on the device that receives. It is chosen on screen when pairing, never taken from the pairing code.

## Reaching a device

iroh dials by public key and picks the best path. solferry has two address sources and contacts no server unless a relay is configured. Without a relay it also leaves router port mapping off, so it never opens a port on your router:

- mDNS for the local network
- relays and direct addresses stored at pairing time

solferry prefers UDP port 53318 so stored addresses survive restarts and firewalls can allow it. If the port is taken it falls back to a random one.

Overlay networks such as Tailscale are ordinary network interfaces for reaching paired devices. To list unpaired ones, `node/overlay.rs` reads the online peers from the local `tailscale` command and asks each on UDP 53319. That port listens only on the overlay address and only while visibility is `everyone`. Admitting unpaired ones needs one check: `100.64.0.0/10` is shared with carrier NAT, so `route.rs` also requires a local tunnel interface with an overlay address. Unknown interface names fail closed.

## Relays

A relay helps two devices punch through NAT and carries data when that fails. solferry only knows relays you added and relays a paired device waits on, so there is nothing to restrict: every known relay is one somebody in the pairing chose. With none, no server is contacted at all.

A pairing code carries the inviter's relay. The joining device starts using it and passes it back, so only one person has to set a relay up. Both sides reload these relays on start.

## Transfers

The sender scans its paths into a `Manifest`. The transfer id is a hash of the sender and the file list, so sending the same files again resumes automatically.

Each file travels on its own QUIC stream, up to 16 at a time. Reading, writing and hashing run on blocking threads connected to the network tasks by bounded channels, which keeps the async runtime free and memory use flat.

Received data stays in `.solferry-partial/<id>/` inside the destination. Files move to their final names only after every file has been verified, so an interrupted transfer never leaves half a file in place.

A resumed file is hashed from the first byte on both sides, so a damaged partial file is caught and sent again. The same check covers bit rot and a source file that changes between attempts.

## Queued sends

A send to a paired device that can't be reached can wait in `queue.json` instead of failing. Unpaired devices never queue. An entry holds paths, or for phones a reference only the app understands: an Android content URI with a lasting grant or a private copy when the provider gives none, and a private copy on iOS. The app reopens a reference into a descriptor when the device is back.

`Node::deliver_queued` runs for as long as an app runs. It probes waiting devices every 20 seconds and at once when one of them reaches this device, then sends oldest first through the normal path, so the receiver is asked as for any send. Unreachable, busy or lost connections keep an entry; a decline or any other failure ends it. Entries expire after 7 days, and forgetting or blocking a device drops its entries.

Nothing delivers while no app runs. On Android the process stays only with the background receiving option on; otherwise delivery waits for the next start, as it does on iOS, which suspends apps soon after they leave the screen. The CLI delivers from `receive`, which reads the queue when it starts.

## Congestion control

Connections use BBR. Consumer Wi-Fi and mobile links lose packets for reasons unrelated to congestion, and loss-based Cubic collapses there. In `tests/netlab`, a 300 Mbit link with 0.5% loss moved 26 MiB/s with BBR and 5 MiB/s with Cubic, and a 200 Mbit link with 1% loss and 80 ms RTT moved 11 MiB/s with BBR and did not finish 300 MiB in 200 seconds with Cubic.

## Stream window

Each stream has a 2 MiB receive window. noq closes a connection that exceeds 1024 gaps in one stream, and a larger window makes that reachable under heavy packet loss. See the constant in `net.rs` before changing it.

One stream therefore moves at most 2 MiB per round trip, so large files travel as up to 16 segments in parallel, each on its own stream with its own hash and resume point. A file in flight can then use 32 MiB of window, more than QUIC stacks usually auto-tune to (quic-go stops at 6 MiB per stream and 15 MiB per connection), while no single stream gets large enough to hit the gap limit. On a 1 Gbit link with 100 ms round trips (`far-link` in `tests/netlab`), one 300 MiB file went from 17 MiB/s to 59 MiB/s, and on the 1% loss link from 11 MiB/s to 16 MiB/s. LAN speed is unchanged.

## Desktop

`solferry-desktop` runs the core on its own thread with a tokio runtime. `service.rs` is the only bridge: the UI sends `Command`s and receives `Update`s over channels, and `state.rs` turns updates into plain data that views render. Closing the window keeps the process and tray alive, because the app exists to receive files.

## Errors

`solferry_core::Error` is an exhaustive enum and is safe to match on. Clients must turn every variant into a message that says what happened and what to do next. The CLI does this in `explain.rs` with no catch-all arm, so adding a variant fails compilation until it has a message.
