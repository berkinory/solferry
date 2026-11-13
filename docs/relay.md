# Running your own relay

A relay lets your devices connect when their networks block direct paths. It only forwards encrypted packets and cannot read your files. Whoever runs a relay can see IP addresses, device ids, timing and traffic volume, so running one makes you the operator of that service.

## Setup

You need a server with a public IP, a DNS name pointing at it with no CDN proxy, and ports 443/tcp and 7842/udp open. Without 7842/udp the relay still carries data, but devices can't open a direct path through it.

Clone this repository on the server, then:

```sh
sudo sh deploy/relay/install.sh relay.example.com
```

It checks DNS and ports first, starts the relay and prints the link to paste into solferry. Add `--check` to run the checks and change nothing. If Traefik already holds port 443, as on Coolify, it passes the relay through Traefik without touching your other sites. On `ufw-docker` servers it opens UDP 7842 for you.

Certificates come from Let's Encrypt and need no email address.

The relay sets no speed limit of its own. To cap how fast each device may send through it, add `RELAY_BYTES_PER_SECOND` to `deploy/relay/.env`, and `RELAY_MAX_BURST_BYTES` for the burst it may go over that, then run `install.sh` again:

```sh
RELAY_BYTES_PER_SECOND=10485760
```

## Use it

On one device, paste that link:

```sh
solferry relay add 'https://relay.example.com#token'
```

Or paste it in Settings. Pair as usual: the pairing code carries the link, so the other device starts using the relay without any setup. File data uses a direct path when one opens and the relay otherwise. Run `solferry relay check` to test it: it says whether the server, the token and the UDP port each work, and what to fix.

Settings and the CLI show the link as `https://relay.example.com#••••`. The copy button next to it copies the whole link.

To keep a relay from using all of a device's bandwidth, set Relay speed limit in Settings, or run `solferry relay limit 5` for 5 MB/s and `solferry relay limit 0` to remove it. In Settings you type the number in MB/s, and 0 means no limit. It only applies while files go through the relay. A direct path is never limited.

## Access

Only devices with the token can use your relay, so a stranger who finds the address gets nothing. Everyone you pair with receives the token, and it stays on their device after you forget them. To shut them out, give the relay a new token.

## Changing the token

On the server, in the same directory:

```sh
sudo sh deploy/relay/rotate.sh
```

It prints a new link. For now the relay accepts both the new token and the old one, so nothing stops working. Paste the new link into one of your devices, in Settings or with `solferry relay add`, which replaces the old link. Every device paired with it gets the new link the next time the two reach each other, and passes it on to the devices it gave the relay to. A device only takes a new link for a relay from the device that gave it that relay, so a paired device can't push a relay on you.

If you added the link by hand on more than one device, paste the new one on each of them.

When your devices have all been online, stop the old token:

```sh
sudo sh deploy/relay/rotate.sh finish
```

From then on, devices you forgot can't use the relay. A device that was offline the whole time can't either: pair it again, or paste the new link on it.
