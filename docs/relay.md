# Running your own relay

A relay lets your devices connect when their networks block direct paths. It only forwards encrypted packets and cannot read your files. Whoever runs a relay can see IP addresses, device ids, timing and traffic volume, so running one makes you the operator of that service.

## Setup

You need a server with a public IP, a DNS name pointing at it, and ports 80/tcp, 443/tcp and 7842/udp open.

```sh
cd deploy/relay
RELAY_HOSTNAME=relay.example.com RELAY_EMAIL=you@example.com docker compose up -d --build
```

Certificates come from Let's Encrypt. Each client is limited to 50 MiB/s by default. Set `RELAY_BYTES_PER_SECOND` and `RELAY_MAX_BURST_BYTES` in `compose.yaml` to change that.

## Use it

On one device:

```sh
beam relay add https://relay.example.com
```

Or paste the address in Settings. Pair as usual: the pairing code carries the relay, so the other device starts using it without any setup. File data uses a direct path when one opens and the relay otherwise. Check the result with `beam relay show`.

## Keep it private

A relay with no authentication serves anyone who finds it, and the bandwidth is yours. If that matters, restrict it with iroh-relay access control or a firewall.
