#!/usr/bin/env bash
#
# build_rust_xcframework.sh
# Compile the Rust core (../core) for iOS device + simulator and bundle it as
# an XCFramework that the Xcode app links against. Run this on macOS before
# building the app in Xcode, or let the GitHub Actions workflow run it.
#
# Output: ios/WortMeisterCore.xcframework
#
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
CORE="$HERE/../core"
BUILD="$HERE/.rustbuild"
OUT="$HERE/WortMeisterCore.xcframework"

LIB_NAME="libwortmeister_core.a"

# iOS targets: arm64 device, arm64 simulator (Apple Silicon Macs) and
# x86_64 simulator (Intel Macs / older CI runners).
TARGETS=(
  aarch64-apple-ios          # iPhone/iPad device
  aarch64-apple-ios-sim      # simulator on Apple Silicon
  x86_64-apple-ios           # simulator on Intel
)

echo "==> Installing Rust targets"
for t in "${TARGETS[@]}"; do
  rustup target add "$t"
done

echo "==> Building release static libs"
for t in "${TARGETS[@]}"; do
  cargo build --release --manifest-path "$CORE/Cargo.toml" --target "$t"
done

rm -rf "$BUILD" "$OUT"
mkdir -p "$BUILD"

# Combine both simulator archs (arm64 + x86_64) into one fat lib via lipo.
echo "==> Creating fat simulator lib"
mkdir -p "$BUILD/sim"
lipo -create \
  "$CORE/target/aarch64-apple-ios-sim/release/$LIB_NAME" \
  "$CORE/target/x86_64-apple-ios/release/$LIB_NAME" \
  -output "$BUILD/sim/$LIB_NAME"

# Headers directory shared by both slices.
HEADERS="$CORE/include"

echo "==> Creating XCFramework"
xcodebuild -create-xcframework \
  -library "$CORE/target/aarch64-apple-ios/release/$LIB_NAME" -headers "$HEADERS" \
  -library "$BUILD/sim/$LIB_NAME" -headers "$HEADERS" \
  -output "$OUT"

echo "==> Done: $OUT"
