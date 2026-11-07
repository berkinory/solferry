#!/bin/bash
# Runs a command as the calling user with temporary render/video membership, without editing system groups.
exec sudo -n setpriv --reuid=$(id -u) --regid=$(id -g) --groups=$(id -g),44,$(getent group render | cut -d: -f3) --no-new-privs env HOME=$HOME PATH="$PATH" BENCH_SOFT="$BENCH_SOFT" BENCH_RUNTIME="$BENCH_RUNTIME" SOLFERRY_SCHEME="$SOLFERRY_SCHEME" "$@"
