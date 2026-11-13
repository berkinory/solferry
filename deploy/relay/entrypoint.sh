#!/bin/sh
set -eu

: "${RELAY_HOSTNAME:?set RELAY_HOSTNAME to the public DNS name of this server}"

relay-token ensure

whole_number() {
    case "$2" in
        '' | *[!0-9]* | 0*) echo "$1 must be a whole number of bytes above zero" >&2; exit 1 ;;
    esac
}

# Unlimited unless set. iroh-relay limits what each client sends into the relay.
LIMITS=""
if [ -n "${RELAY_BYTES_PER_SECOND:-}" ]; then
    whole_number RELAY_BYTES_PER_SECOND "$RELAY_BYTES_PER_SECOND"
    LIMITS="[limits.client.rx]
bytes_per_second = $RELAY_BYTES_PER_SECOND"
    if [ -n "${RELAY_MAX_BURST_BYTES:-}" ]; then
        whole_number RELAY_MAX_BURST_BYTES "$RELAY_MAX_BURST_BYTES"
        LIMITS="$LIMITS
max_burst_bytes = $RELAY_MAX_BURST_BYTES"
    fi
elif [ -n "${RELAY_MAX_BURST_BYTES:-}" ]; then
    echo "RELAY_MAX_BURST_BYTES needs RELAY_BYTES_PER_SECOND" >&2
    exit 1
fi

umask 077
cat > /tmp/relay.toml <<CONFIG
http_bind_addr = "[::]:8080"
enable_quic_addr_discovery = true
access.shared_token = [$(relay-token tokens)]

[tls]
https_bind_addr = "[::]:8443"
hostname = "$RELAY_HOSTNAME"
contact = "${RELAY_EMAIL:-relay@$RELAY_HOSTNAME}"
cert_mode = "LetsEncrypt"
cert_dir = "/data"

$LIMITS
CONFIG

echo "Paste this into solferry: $(relay-token link)"
exec iroh-relay --config-path /tmp/relay.toml
