#!/bin/sh
# Keeps the relay's access tokens in /data. While a rotation runs, the previous token still works.
# Usage: relay-token ensure | rotate | finish | tokens | link
set -eu

CURRENT=/data/relay-token
PREVIOUS=/data/relay-token.previous
umask 077

new_token() {
    od -An -tx1 -N32 /dev/urandom | tr -d ' \n' > "$CURRENT.new"
    mv "$CURRENT.new" "$CURRENT"
}

case "${1:-}" in
    ensure)
        [ -s "$CURRENT" ] || new_token
        ;;
    rotate)
        if [ -s "$PREVIOUS" ]; then
            echo "error: the last rotation is not finished. Run: sh rotate.sh finish" >&2
            exit 1
        fi
        cp "$CURRENT" "$PREVIOUS"
        new_token
        ;;
    finish)
        rm -f "$PREVIOUS"
        ;;
    tokens)
        printf '"%s"' "$(cat "$CURRENT")"
        if [ -s "$PREVIOUS" ]; then printf ', "%s"' "$(cat "$PREVIOUS")"; fi
        ;;
    link)
        echo "https://$RELAY_HOSTNAME#$(cat "$CURRENT")"
        ;;
    *)
        echo "usage: relay-token ensure | rotate | finish | tokens | link" >&2
        exit 2
        ;;
esac
