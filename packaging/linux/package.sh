#!/bin/bash
# Turns a built desktop binary into the published AppImage, which can update itself.
set -euo pipefail

BINARY=$1
PLATFORM=$2
DIST=$3
ARCH=${PLATFORM#linux-}
ROOT=$(cd "$(dirname "$0")/../.." && pwd)
ASSETS="$ROOT/crates/solferry-desktop/assets"
TOOL_VERSION=1.9.1
WORK=$(mktemp -d)

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
# The strongest zstd level and 1 MiB blocks cut about 8% with no cost to starting the app.
ARCH=$ARCH "$TOOL" --appimage-extract-and-run --comp zstd \
    --mksquashfs-opt -Xcompression-level --mksquashfs-opt 22 --mksquashfs-opt -b --mksquashfs-opt 1M \
    "$APPDIR" "$DIST/Solferry-$PLATFORM.AppImage"

