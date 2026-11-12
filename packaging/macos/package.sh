#!/bin/bash
# Turns a built release into the published DMG, which people install and the updater unpacks.
# Signs and notarizes when SOLFERRY_SIGN_IDENTITY is set.
set -euo pipefail

RELEASE=$1
PLATFORM=$2
DIST=$3
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
APP="$RELEASE/Solferry.app"
IDENTITY=${SOLFERRY_SIGN_IDENTITY:-}

# Apple can sit on a first submission for days. A dry run moves on after a few minutes, but a
# release stops, so a DMG Gatekeeper would refuse never goes out.
NOTARY_WAIT=5m
if [ "${SOLFERRY_PUBLISH:-}" = true ]; then
    NOTARY_WAIT=30m
fi

notarize() {
    local auth=(--apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_APP_PASSWORD")
    local id status
    id=$(xcrun notarytool submit "$1" "${auth[@]}" --output-format plist | plutil -extract id raw -o - -)
    xcrun notarytool wait "$id" "${auth[@]}" --timeout "$NOTARY_WAIT" || true
    status=$(xcrun notarytool info "$id" "${auth[@]}" --output-format plist | plutil -extract status raw -o - -)
    if [ "$status" = Accepted ]; then
        xcrun stapler staple "$2"
    elif [ "$status" = "In Progress" ] && [ "${SOLFERRY_PUBLISH:-}" != true ]; then
        echo "::warning::Apple is still notarizing $(basename "$2"), submission $id"
    else
        xcrun notarytool log "$id" "${auth[@]}" || true
        echo "::error::Notarizing $(basename "$2") ended as $status, submission $id"
        exit 1
    fi
}

"$ROOT/packaging/macos/bundle.sh" "$RELEASE"
if [ -n "$IDENTITY" ]; then
    SUBMISSION=$(mktemp -d)/Solferry.zip
    ditto -c -k --keepParent "$APP" "$SUBMISSION"
    notarize "$SUBMISSION" "$APP"
fi

STAGE=$(mktemp -d)
cp -R "$APP" "$STAGE/"
ln -s /Applications "$STAGE/Applications"
DMG="$DIST/Solferry-$PLATFORM.dmg"
hdiutil create -volname Solferry -srcfolder "$STAGE" -ov -format ULFO "$DMG"
if [ -n "$IDENTITY" ]; then
    codesign --force --timestamp --sign "$IDENTITY" "$DMG"
    notarize "$DMG" "$DMG"
fi
