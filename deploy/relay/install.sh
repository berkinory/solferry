#!/bin/sh
# Installs a solferry relay with Docker. Usage: sudo sh install.sh [--check] relay.example.com
set -eu

CHECK_ONLY=0
if [ "${1:-}" = "--check" ]; then
    CHECK_ONLY=1
    shift
fi
HOSTNAME_ARG=${1:?usage: install.sh [--check] relay.example.com}
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
FILES="-f compose.yaml"

say() { printf '%s\n' "$*"; }
stop() { say "error: $*" >&2; exit 1; }

command -v docker >/dev/null || stop "Docker is not installed. See https://docs.docker.com/engine/install/"
docker compose version >/dev/null 2>&1 || stop "Docker Compose is not available."

ADDRESS=$(getent ahostsv4 "$HOSTNAME_ARG" 2>/dev/null | awk 'NR==1{print $1}')
[ -n "$ADDRESS" ] || stop "$HOSTNAME_ARG has no DNS record yet. Point it at this server, with no CDN proxy."
say "DNS       $HOSTNAME_ARG -> $ADDRESS"

port_owner() {
    docker ps --format '{{.Names}} {{.Image}} {{.Ports}}' | grep -E "(:|\[::\]:)$1->" | head -1 | awk '{print $1, $2}'
}
if ss -tlnH 2>/dev/null | grep -qE ':443\b'; then
    OWNER=$(port_owner 443)
    case "$OWNER" in
        *solferry-relay*) say "Port 443  already used by this relay" ;;
        *traefik*)
            say "Port 443  held by Traefik ($OWNER), passing the relay through it"
            FILES="$FILES -f compose.traefik.yaml"
            ;;
        *) stop "Port 443 is in use by ${OWNER:-another program}. Free it, or put Traefik in front." ;;
    esac
else
    say "Port 443  free"
fi
if ss -ulnH 2>/dev/null | grep -qE ':7842\b' && ! port_owner 7842 | grep -q solferry-relay; then
    stop "UDP port 7842 is in use."
fi

if [ "$CHECK_ONLY" = 1 ]; then
    say "Check finished, nothing was changed."
    exit 0
fi

cd "$HERE"
umask 077
# Keeps settings such as RELAY_BYTES_PER_SECOND that were added to .env by hand.
KEPT=$(grep -v '^RELAY_HOSTNAME=' .env 2>/dev/null || true)
printf 'RELAY_HOSTNAME=%s\n' "$HOSTNAME_ARG" > .env
[ -z "$KEPT" ] || printf '%s\n' "$KEPT" >> .env
# shellcheck disable=SC2086
docker compose -p solferry-relay $FILES pull --quiet --ignore-pull-failures
# shellcheck disable=SC2086
docker compose -p solferry-relay $FILES up -d

if command -v ufw-docker >/dev/null; then
    ufw-docker allow solferry-relay-relay-1 7842/udp || say "Could not open UDP 7842, direct paths may not work."
fi

for _ in 1 2 3 4 5 6 7 8 9 10; do
    LINK=$(docker logs solferry-relay-relay-1 2>&1 | grep -o 'https://[^ ]*#[0-9a-f]*' | tail -1 || true)
    [ -n "$LINK" ] && break
    sleep 2
done
[ -n "${LINK:-}" ] || stop "The relay did not start. See: docker logs solferry-relay-relay-1"

say ""
say "Done. Paste this into solferry, under Settings or with solferry relay add:"
say ""
say "  $LINK"
say ""
say "Treat it like a password. Then test it with: solferry relay check"
