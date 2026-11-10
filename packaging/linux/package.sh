#!/bin/bash
# Turns a built desktop binary into the published Linux files: an AppImage, which can update
# itself, and a .deb, which leaves updates to the package manager.
set -euo pipefail

BINARY=$1
PLATFORM=$2
DIST=$3
ARCH=${PLATFORM#linux-}
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
ASSETS="$ROOT/crates/solferry-desktop/assets"
VERSION=$(grep -m1 '^version' "$ROOT/Cargo.toml" | cut -d'"' -f2)
TOOL_VERSION=1.9.1
WORK=$(mktemp -d)

case $ARCH in
    x86_64) DEB_ARCH=amd64 ;;
    aarch64) DEB_ARCH=arm64 ;;
esac

APPDIR="$WORK/Solferry.AppDir"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/lib"
cp "$BINARY" "$APPDIR/usr/bin/solferry-desktop"
cp "$ASSETS/solferry.desktop" "$APPDIR/"
cp "$ASSETS/icon.png" "$APPDIR/solferry.png"
for library in libxkbcommon.so.0 libxkbcommon-x11.so.0 libxcb-xkb.so.1; do
    cp -L "$(ldconfig -p | awk -v name="$library" '$1 == name { print $NF; exit }')" "$APPDIR/usr/lib/"
done
cat > "$APPDIR/AppRun" <<'RUN'
#!/bin/sh
HERE=$(dirname "$(readlink -f "$0")")
export LD_LIBRARY_PATH="$HERE/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
exec "$HERE/usr/bin/solferry-desktop" "$@"
RUN
chmod +x "$APPDIR/AppRun"
TOOL="$WORK/appimagetool"
curl -fsSL -o "$TOOL" "https://github.com/AppImage/appimagetool/releases/download/$TOOL_VERSION/appimagetool-$ARCH.AppImage"
chmod +x "$TOOL"
ARCH=$ARCH "$TOOL" --appimage-extract-and-run --comp zstd "$APPDIR" "$DIST/Solferry-$PLATFORM.AppImage"

PACKAGE="$WORK/deb"
install -Dm755 "$BINARY" "$PACKAGE/usr/bin/solferry-desktop"
install -Dm644 "$ASSETS/solferry.desktop" "$PACKAGE/usr/share/applications/solferry.desktop"
install -Dm644 "$ASSETS/icon.png" "$PACKAGE/usr/share/icons/hicolor/512x512/apps/solferry.png"
mkdir "$PACKAGE/DEBIAN"
cat > "$PACKAGE/DEBIAN/control" <<CONTROL
Package: solferry
Version: $VERSION
Architecture: $DEB_ARCH
Maintainer: Solferry <https://github.com/berkinory/solferry/issues>
Depends: libxkbcommon0, libxkbcommon-x11-0, libxcb1
Section: net
Priority: optional
Description: Send files between your devices, no account needed
CONTROL
dpkg-deb --build --root-owner-group -Zxz "$PACKAGE" "$DIST/Solferry-$PLATFORM.deb"
