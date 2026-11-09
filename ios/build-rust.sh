#!/bin/bash
# Builds the Rust core for iOS and writes the XCFramework and Swift bindings the Xcode project uses.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
OUT=$ROOT/ios/Generated
export IPHONEOS_DEPLOYMENT_TARGET=26.0
cd "$ROOT"

# Cargo.lock holds libc at 0.2.188: from 0.2.189 it hides BSD routing items netwatch needs on iOS.

for target in aarch64-apple-ios aarch64-apple-ios-sim; do
    cargo rustc -q -p solferry-ffi --lib --release --target "$target" --crate-type staticlib
done

# Release builds are stripped, so the bindings come from a host build that keeps UniFFI's metadata.
cargo build -q -p solferry-ffi
rm -rf "$OUT"
mkdir -p "$OUT/bindings" "$OUT/headers"
cargo run -q -p solferry-ffi --features bindgen --bin uniffi-bindgen -- generate \
    --library target/debug/libsolferry_ffi.dylib --language swift --no-format --out-dir "$OUT/bindings"
mv "$OUT/bindings/solferry_ffi.swift" "$OUT/"
mv "$OUT/bindings/solferry_ffiFFI.h" "$OUT/headers/"
mv "$OUT/bindings/solferry_ffiFFI.modulemap" "$OUT/headers/module.modulemap"
rm -rf "$OUT/bindings"

xcodebuild -create-xcframework \
    -library target/aarch64-apple-ios/release/libsolferry_ffi.a -headers "$OUT/headers" \
    -library target/aarch64-apple-ios-sim/release/libsolferry_ffi.a -headers "$OUT/headers" \
    -output "$OUT/SolferryFFI.xcframework"
