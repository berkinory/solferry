#!/bin/bash
# Turns a built release into the published macOS files: the DMG people install, the zip the
# updater swaps in, and the CLI. Signs and notarizes when SOLFERRY_SIGN_IDENTITY is set.
set -euo pipefail

RELEASE=$1
PLATFORM=$2
DIST=$3
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
APP="$RELEASE/Solferry.app"
IDENTITY=${SOLFERRY_SIGN_IDENTITY:-}

notarize() {
    xcrun notarytool submit "$1" --apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" \
        --password "$APPLE_APP_PASSWORD" --wait --timeout 30m
}

"$ROOT/packaging/macos/bundle.sh" "$RELEASE"
if [ -n "$IDENTITY" ]; then
    SUBMISSION=$(mktemp -d)/Solferry.zip
    ditto -c -k --keepParent "$APP" "$SUBMISSION"
    notarize "$SUBMISSION"
    xcrun stapler staple "$APP"
    codesign --force --options runtime --timestamp --sign "$IDENTITY" "$RELEASE/solferry"
fi

ditto -c -k --keepParent "$APP" "$DIST/Solferry-$PLATFORM.zip"

STAGE=$(mktemp -d)
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
DMG="$DIST/Solferry-$PLATFORM.dmg"
hdiutil create -volname Solferry -srcfolder "$STAGE" -ov -format ULFO "$DMG"
if [ -n "$IDENTITY" ]; then
    codesign --force --timestamp --sign "$IDENTITY" "$DMG"
    notarize "$DMG"
    xcrun stapler staple "$DMG"
fi

tar -C "$RELEASE" -czf "$DIST/solferry-cli-$PLATFORM.tar.gz" solferry
