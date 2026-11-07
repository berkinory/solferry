# Protocol

All traffic runs over iroh QUIC connections, which are mutually authenticated and encrypted with TLS 1.3. Messages are Protocol Buffers, so unknown fields are ignored and new fields do not break older clients. Definitions live in `crates/solferry-proto/src/messages.rs`.

Control messages are framed as a big-endian `u32` length followed by the message. Frames above 64 MiB are rejected before allocation.

## Transfers: ALPN `solferry/1`

The sender opens one bidirectional control stream.

```text
sender                                receiver
  Hello            ------------------>
                   <------------------  Hello
  Offer            ------------------>
                   <------------------  OfferReply (decision, resume offsets)
  one unidirectional stream per file
  FileStart, raw bytes, 32-byte hash ->
                   <------------------  FileResult, one per file
                   <------------------  Finalized
  connection close
```

- `Hello.version` must match. A mismatch closes the connection with code 2.
- `Offer.transfer_id` is the BLAKE3 hash of the sender id and each entry's path, size and modified time.
- `OfferReply.offsets` has one entry per offered entry and gives the bytes the receiver already holds.
- A file stream carries `FileStart`, the bytes from that offset to the end, then the BLAKE3 hash of the complete file.
- `Finalized` confirms the files have moved to their destination. The sender closes after reading it.

Entry paths use `/` separators and are relative. Receivers must reject absolute paths, `.` and `..` components, backslashes, control characters, Windows reserved names and duplicates.

## Pairing: ALPN `solferry-pair/1`

An invite is the string `sf1` plus lowercase base32 of a `Ticket`. It holds the inviter's id, addresses, a 256-bit secret, an expiry and whether it is for one of the inviter's own devices.

The joiner dials the id in the ticket, so the connection proves the inviter's identity, and sends `PairRequest` with the secret. The inviter removes the secret from memory on first use, and replies with `PairReply`. The joiner stores the inviter and confirms with `Finalized`, and only then does the inviter store the joiner, so a lost reply cannot leave one side paired alone. Only a hash of the secret is kept.

## Close codes

| Code | Meaning |
|---|---|
| 0 | Done |
| 1 | Peer requires pairing |
| 2 | Protocol version mismatch |
| 3 | Internal error |

## Pairing without a code

On the same network or overlay, a device can pair with no invite. Both sides must approve, and the people compare six digits.

1. The requester sends `PairRequest` with an empty `secret` and `commitment = blake3(nonce_r)`.
2. The responder sends `PairChallenge { nonce_s }`.
3. The requester sends `PairReveal { nonce_r }`, and the responder checks the commitment.
4. Both show a code derived from the two device ids and both nonces.
5. The responder's user accepts, and the flow finishes with `PairReply` and `Finalized` as with an invite.

Each side fixes its nonce before seeing the other's, so an attacker can't search for a key that gives two connections the same code. The approval for receiving without asking stays local to each device.

## Limits before trust

Hello and pairing messages are read with a 64 KiB frame limit. The pairing ALPN closes at once when no invite exists and nothing listens for requests. Unpaired devices share a pool of two connections, separate from paired ones.
