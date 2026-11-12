#!/bin/bash
# Builds the Rust core for Android and writes its Kotlin bindings where Gradle picks them up.
set -euo pipefail

OUT=${1:?usage: build-rust.sh <output dir>}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
ABIS=${SOLFERRY_ABIS:-arm64-v8a,x86_64}
cd "$ROOT"

targets=()
for abi in ${ABIS//,/ }; do
    targets+=(-t "$abi")
done
# Published builds set SOLFERRY_RUST_PROFILE=dist, which links slower and comes out smaller.
cargo ndk "${targets[@]}" -P 29 -o "$OUT/jniLibs" build --profile "${SOLFERRY_RUST_PROFILE:-release}" -p solferry-ffi
find "$OUT/jniLibs" -name '*.so' ! -name libsolferry_ffi.so -delete

# Release builds are stripped, so the bindings come from a host build that keeps UniFFI's metadata.
cargo build -q -p solferry-ffi
cargo run -q -p solferry-ffi --features bindgen --bin uniffi-bindgen -- generate \
    --library target/debug/libsolferry_ffi.so --language kotlin --no-format --out-dir "$OUT/kotlin"
