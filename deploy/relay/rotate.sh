#!/bin/sh
# Gives the relay a new access token. Usage: sudo sh rotate.sh [finish]
#   rotate.sh         new token; the old one keeps working until you finish
#   rotate.sh finish  the old token stops working
set -eu

CONTAINER=${RELAY_CONTAINER:-solferry-relay-relay-1}

say() { printf '%s\n' "$*"; }
stop() { say "error: $*" >&2; exit 1; }

docker inspect "$CONTAINER" >/dev/null 2>&1 || stop "No relay container named $CONTAINER. Run install.sh first."

case "${1:-start}" in
    start)
        docker exec "$CONTAINER" relay-token rotate
        docker restart "$CONTAINER" >/dev/null
        say "The relay accepts the new token and the old one."
        say ""
        say "Paste the new link into one of your devices, under Settings or with solferry relay add:"
        say ""
        say "  $(docker exec "$CONTAINER" relay-token link)"
        say ""
        say "Devices paired with it get the new link the next time they reach it."
        say "Once they all have, stop the old token with: sh rotate.sh finish"
        ;;
    finish)
        docker exec "$CONTAINER" relay-token finish
        docker restart "$CONTAINER" >/dev/null
        say "Done. Only the new token works now."
        ;;
    *)
        stop "usage: rotate.sh [finish]"
        ;;
esac
