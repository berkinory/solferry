# Running your own relay

A relay lets your devices connect when their networks block direct paths. It only forwards encrypted packets and cannot read your files. Whoever runs a relay can see IP addresses, device ids, timing and traffic volume, so running one makes you the operator of that service.

## Setup

You need a server with a public IP, a DNS name pointing at it with no CDN proxy, and ports 443/tcp and 7842/udp open. Without 7842/udp the relay still carries data, but devices can't open a direct path through it.

Clone this repository on the server, then:

```sh
sudo sh deploy/relay/install.sh relay.example.com you@example.com
```

It checks DNS and ports first, starts the relay and prints the link to paste into solferry. Add `--check` to run the checks and change nothing. If Traefik already holds port 443, as on Coolify, it passes the relay through Traefik without touching your other sites. On `ufw-docker` servers it opens UDP 7842 for you.

Certificates come from Let's Encrypt. Each client is limited to 50 MiB/s by default. Set `RELAY_BYTES_PER_SECOND` and `RELAY_MAX_BURST_BYTES` in `compose.yaml` to change that.

## Use it

On one device, paste that link:

```sh
solferry relay add 'https://relay.example.com#token'
```

Or paste it in Settings. Pair as usual: the pairing code carries the link, so the other device starts using the relay without any setup. File data uses a direct path when one opens and the relay otherwise. Run `solferry relay check` to test it: it says whether the server, the token and the UDP port each work, and what to fix.

## Access

Only devices with the token can use your relay, so a stranger who finds the address gets nothing. Everyone you pair with receives the token, and it stays on their device after you forget them. To shut them all out, delete `relay-token` from the `certs` volume, restart the relay and add the new link.
