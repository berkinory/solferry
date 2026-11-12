#!/bin/bash
# Prints the CHANGELOG.md section for a version, the release notes. Fails when there is none.
set -euo pipefail

VERSION=$1
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
NOTES=$(awk -v heading="## $VERSION" '
    $0 == heading || index($0, heading " ") == 1 { found = 1; next }
    found && /^## / { exit }
    found && (started || $0 != "") {
        started = 1
        pending = pending $0 "\n"
        if ($0 != "") { printf "%s", pending; pending = "" }
    }
' "$ROOT/CHANGELOG.md")
[ -n "$NOTES" ] || {
    echo "CHANGELOG.md has no notes for $VERSION" >&2
    exit 1
}
echo "$NOTES"
