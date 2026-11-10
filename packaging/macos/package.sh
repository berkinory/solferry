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

# Apple can sit on a submission for days, so past five minutes it carries on without the build.
notarize() {
    local auth=(--apple-id "$APPLE_ID" --team-id "$APPLE_TEAM_ID" --password "$APPLE_APP_PASSWORD")
    local id status
    id=$(xcrun notarytool submit "$1" "${auth[@]}" --output-format plist | plutil -extract id raw -o - -)
    xcrun notarytool wait "$id" "${auth[@]}" --timeout 5m || true
    status=$(xcrun notarytool info "$id" "${auth[@]}" --output-format plist | plutil -extract status raw -o - -)
    case $status in
        Accepted) xcrun stapler staple "$2" ;;
        "In Progress") echo "::warning::Apple is still notarizing $(basename "$2"), submission $id. It ships unstapled." ;;
        *)
            xcrun notarytool log "$id" "${auth[@]}" || true
            echo "::error::Notarizing $(basename "$2") ended as $status, submission $id"
            exit 1
            ;;
    esac
}

"$ROOT/packaging/macos/bundle.sh" "$RELEASE"
if [ -n "$IDENTITY" ]; then
    SUBMISSION=$(mktemp -d)/Solferry.zip
    ditto -c -k --keepParent "$APP" "$SUBMISSION"
    notarize "$SUBMISSION" "$APP"
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
    notarize "$DMG" "$DMG"
fi

tar -C "$RELEASE" -czf "$DIST/solferry-cli-$PLATFORM.tar.gz" solferry
