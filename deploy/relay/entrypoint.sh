#!/bin/sh
set -eu

: "${RELAY_HOSTNAME:?set RELAY_HOSTNAME to the public DNS name of this server}"
: "${RELAY_EMAIL:?set RELAY_EMAIL for Let's Encrypt notices}"

cat > /tmp/relay.toml <<CONFIG
http_bind_addr = "[::]:80"
enable_quic_addr_discovery = true

[tls]
hostname = "$RELAY_HOSTNAME"
contact = "$RELAY_EMAIL"
cert_mode = "LetsEncrypt"
cert_dir = "/data"

[limits.client.rx]
bytes_per_second = ${RELAY_BYTES_PER_SECOND:-52428800}
max_burst_bytes = ${RELAY_MAX_BURST_BYTES:-104857600}
CONFIG

exec iroh-relay --config-path /tmp/relay.toml
