#!/bin/sh
set -eu

: "${RELAY_HOSTNAME:?set RELAY_HOSTNAME to the public DNS name of this server}"

TOKEN_FILE=/data/relay-token
if [ ! -s "$TOKEN_FILE" ]; then
    (umask 077 && od -An -tx1 -N32 /dev/urandom | tr -d ' \n' > "$TOKEN_FILE")
fi
TOKEN=$(cat "$TOKEN_FILE")

umask 077
cat > /tmp/relay.toml <<CONFIG
http_bind_addr = "[::]:8080"
enable_quic_addr_discovery = true
access.shared_token = ["$TOKEN"]

[tls]
https_bind_addr = "[::]:8443"
hostname = "$RELAY_HOSTNAME"
contact = "${RELAY_EMAIL:-relay@$RELAY_HOSTNAME}"
cert_mode = "LetsEncrypt"
cert_dir = "/data"

[limits.client.rx]
bytes_per_second = ${RELAY_BYTES_PER_SECOND:-52428800}
max_burst_bytes = ${RELAY_MAX_BURST_BYTES:-104857600}
CONFIG

echo "Paste this into beam: https://$RELAY_HOSTNAME#$TOKEN"
exec iroh-relay --config-path /tmp/relay.toml
