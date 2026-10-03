#!/usr/bin/env bash
# Build the German vocabulary trainer for macOS.
# Run this ON A MAC that has Xcode command line tools and Rust installed.
#
#   Install Rust:  https://rustup.rs
#   Install Xcode CLT:  xcode-select --install
#
# Usage:
#   chmod +x build_macos.sh
#   ./build_macos.sh                 # build for the Mac you are running on
#   ./build_macos.sh universal       # build a universal (Intel + Apple Silicon) binary
#
# Old-OS note: MACOSX_DEPLOYMENT_TARGET controls the minimum macOS the binary
# declares. Only the x86_64 slice can run on macOS 10.x (all 10.x Macs are
# Intel); the arm64 slice always needs macOS 11+.

set -euo pipefail

APP_NAME="DeutschWorttrainer"
BIN_NAME="deutsch-worttrainer"
DIST_DIR="dist/macos"

# Minimum macOS version the produced binary targets. Can be overridden:
#   MACOSX_DEPLOYMENT_TARGET=10.15 ./build_macos.sh universal
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-10.13}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust was not found. Install it from https://rustup.rs and re-run this script."
  exit 1
fi

echo "Targeting macOS >= $MACOSX_DEPLOYMENT_TARGET"
mkdir -p "$DIST_DIR"

if [[ "${1:-}" == "universal" ]]; then
  echo "Building a universal macOS binary (x86_64 + arm64)..."
  rustup target add x86_64-apple-darwin aarch64-apple-darwin
  cargo build --release --target x86_64-apple-darwin
  cargo build --release --target aarch64-apple-darwin
  lipo -create -output "$DIST_DIR/$BIN_NAME" \
    "target/x86_64-apple-darwin/release/$BIN_NAME" \
    "target/aarch64-apple-darwin/release/$BIN_NAME"
else
  echo "Building a native macOS binary for this machine..."
  cargo build --release
  cp "target/release/$BIN_NAME" "$DIST_DIR/$BIN_NAME"
fi

# Assemble a minimal .app bundle so it can be launched by double-click.
APP_BUNDLE="$DIST_DIR/$APP_NAME.app"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cp "$DIST_DIR/$BIN_NAME" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"
chmod +x "$APP_BUNDLE/Contents/MacOS/$APP_NAME"
cp "data/vocabulary.csv" "$APP_BUNDLE/Contents/Resources/vocabulary.csv"
# Also drop the CSV next to the raw binary for the plain-binary workflow.
cp "data/vocabulary.csv" "$DIST_DIR/vocabulary.csv"

cat > "$APP_BUNDLE/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>
    <string>Deutsch Worttrainer</string>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIdentifier</key>
    <string>com.deutschcoachpro.worttrainer</string>
    <key>CFBundleVersion</key>
    <string>0.0.7</string>
    <key>CFBundleShortVersionString</key>
    <string>0.0.7</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>$MACOSX_DEPLOYMENT_TARGET</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

echo
echo "Build complete:"
echo "  App bundle : $APP_BUNDLE"
echo "  Plain binary: $DIST_DIR/$BIN_NAME"
