#!/bin/bash
# Runs a command as berk with temporary render/video membership, without editing system groups.
exec sudo -n setpriv --reuid=1000 --regid=1000 --groups=1000,44,$(getent group render | cut -d: -f3) --no-new-privs env HOME=/home/berk PATH="$PATH" BENCH_SOFT="$BENCH_SOFT" BENCH_RUNTIME="$BENCH_RUNTIME" "$@"
