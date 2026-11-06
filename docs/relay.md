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

On each device:

```sh
beam relay add https://relay.example.com
```

Devices paired after this share the relay address automatically. Adding a relay switches the default policy to `own-relays`, so file data may use it but never a public relay. Check the result with `beam relay show`.

## Keep it private

A relay with no authentication serves anyone who finds it. If bandwidth or exposure matters, restrict it with iroh-relay access control or a firewall.
