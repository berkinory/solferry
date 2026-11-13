#!/bin/bash
# Builds Solferry.app next to the given release binaries, with the CLI inside when it was built
# and the share extension compiled for the same architecture. Signs it with
# SOLFERRY_SIGN_IDENTITY, or ad hoc without one.
set -euo pipefail

RELEASE=${1:-target/release}
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VERSION=$(grep -m1 '^version' "$ROOT/Cargo.toml" | cut -d'"' -f2)
APP="$RELEASE/Solferry.app"
APPEX="$APP/Contents/PlugIns/SolferryShare.appex"
MACOS="$ROOT/packaging/macos"
ICONS=$(mktemp -d)/solferry.iconset

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" "$APPEX/Contents/MacOS" "$ICONS"
cp "$RELEASE/solferry-desktop" "$APP/Contents/MacOS/"
sed "s/VERSION/$VERSION/g" "$MACOS/Info.plist" > "$APP/Contents/Info.plist"
sed "s/VERSION/$VERSION/g" "$MACOS/share/Info.plist" > "$APPEX/Contents/Info.plist"
ARCH=$(lipo -archs "$RELEASE/solferry-desktop")
xcrun --sdk macosx swiftc -O -parse-as-library -application-extension \
    -target "$ARCH-apple-macos${MACOSX_DEPLOYMENT_TARGET:-12.0}" -module-name SolferryShare \
    -Xlinker -e -Xlinker _NSExtensionMain \
    -o "$APPEX/Contents/MacOS/SolferryShare" "$MACOS/share/ShareViewController.swift"
for size in 16 32 128 256 512; do
    sips -z $size $size "$ROOT/crates/solferry-desktop/assets/icon.png" --out "$ICONS/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -z $double $double "$ROOT/crates/solferry-desktop/assets/icon.png" --out "$ICONS/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONS" -o "$APP/Contents/Resources/solferry.icns"
if [ -n "${SOLFERRY_SIGN_IDENTITY:-}" ]; then
    SIGN=(--options runtime --timestamp --sign "$SOLFERRY_SIGN_IDENTITY")
else
    SIGN=(--sign -)
fi
# Nested code is signed before the bundle that seals it.
if [ -f "$RELEASE/solferry" ]; then
    cp "$RELEASE/solferry" "$APP/Contents/MacOS/"
    codesign --force "${SIGN[@]}" "$APP/Contents/MacOS/solferry"
fi
codesign --force "${SIGN[@]}" --entitlements "$MACOS/share/share.entitlements" "$APPEX"
codesign --force "${SIGN[@]}" --entitlements "$MACOS/Solferry.entitlements" "$APP"
echo "$APP"
