#!/bin/bash
# Builds Solferry.app next to the given release binary and signs it ad hoc.
set -euo pipefail

RELEASE=${1:-target/release}
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VERSION=$(grep -m1 '^version' "$ROOT/Cargo.toml" | cut -d'"' -f2)
APP="$RELEASE/Solferry.app"
ICONS=$(mktemp -d)/solferry.iconset

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" "$ICONS"
cp "$RELEASE/solferry-desktop" "$APP/Contents/MacOS/"
sed "s/VERSION/$VERSION/g" "$ROOT/packaging/macos/Info.plist" > "$APP/Contents/Info.plist"
for size in 16 32 128 256 512; do
    sips -z $size $size "$ROOT/crates/solferry-desktop/assets/icon.png" --out "$ICONS/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -z $double $double "$ROOT/crates/solferry-desktop/assets/icon.png" --out "$ICONS/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONS" -o "$APP/Contents/Resources/solferry.icns"
codesign --force --deep --sign - "$APP"
echo "$APP"
