#!/bin/bash
# Prints the CHANGELOG.md section for a version, the release notes. Fails when there is none.
set -euo pipefail

VERSION=$1
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
NOTES=$(awk -v heading="## $VERSION" '
    $0 == heading || index($0, heading " ") == 1 { found = 1; next }
    found && /^## / { exit }
    found { print }
' "$ROOT/CHANGELOG.md" | sed -e '/./,$!d' -e ':a' -e '/^\n*$/{$d;N;ba' -e '}')
[ -n "$NOTES" ] || {
    echo "CHANGELOG.md has no notes for $VERSION" >&2
    exit 1
}
echo "$NOTES"
