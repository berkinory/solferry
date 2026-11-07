# Running your own relay

A relay lets your devices connect when their networks block direct paths. It only forwards encrypted packets and cannot read your files. Whoever runs a relay can see IP addresses, device ids, timing and traffic volume, so running one makes you the operator of that service.

## Setup

You need a server with a public IP, a DNS name pointing at it, and ports 80/tcp, 443/tcp and 7842/udp open.

```sh
cd deploy/relay
RELAY_HOSTNAME=relay.example.com RELAY_EMAIL=you@example.com docker compose up -d --build
```

Certificates come from Let's Encrypt. Each client is limited to 50 MiB/s by default. Set `RELAY_BYTES_PER_SECOND` and `RELAY_MAX_BURST_BYTES` in `compose.yaml` to change that.

The relay creates an access token on first start and prints a link. Read it with:

```sh
docker compose logs relay | grep Paste
```

## Use it

On one device, paste that link:

```sh
beam relay add 'https://relay.example.com#token'
```

Or paste it in Settings. Pair as usual: the pairing code carries the link, so the other device starts using the relay without any setup. File data uses a direct path when one opens and the relay otherwise. Check the result with `beam relay show`.

## Access

Only devices with the token can use your relay, so a stranger who finds the address gets nothing. Everyone you pair with receives the token, and it stays on their device after you forget them. To shut them all out, delete `relay-token` from the `certs` volume, restart the relay and add the new link.
