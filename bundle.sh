#!/usr/bin/env bash
# Build and bundle the plugin as a VST3 and CLAP package.
# Usage: ./bundle.sh [--release|--debug]
set -e

PROFILE="release"
if [ "$1" = "--debug" ]; then
    PROFILE="debug"
fi

NAME="ruhrvibe"
DISPLAY_NAME="Ruhrvibe"

echo ">>> Building $NAME ($PROFILE)..."
if [ "$PROFILE" = "release" ]; then
    cargo build --release
    SRC_DIR="target/release"
else
    cargo build
    SRC_DIR="target/debug"
fi

SRC_DLL="$SRC_DIR/${NAME}.dll"
if [ ! -f "$SRC_DLL" ]; then
    echo "ERROR: Built DLL not found at $SRC_DLL"
    exit 1
fi

OUT_DIR="target/bundled"
mkdir -p "$OUT_DIR"

# VST3 bundle (Windows layout)
VST3_BUNDLE="$OUT_DIR/${DISPLAY_NAME}.vst3"
VST3_INNER="$VST3_BUNDLE/Contents/x86_64-win"
rm -rf "$VST3_BUNDLE"
mkdir -p "$VST3_INNER"
cp "$SRC_DLL" "$VST3_INNER/${DISPLAY_NAME}.vst3"
echo ">>> VST3 bundle: $VST3_BUNDLE"

# CLAP bundle (Windows is just a renamed DLL, no folder structure required)
CLAP_FILE="$OUT_DIR/${DISPLAY_NAME}.clap"
cp "$SRC_DLL" "$CLAP_FILE"
echo ">>> CLAP file:   $CLAP_FILE"

echo
echo "Install by copying the .vst3 bundle to:"
echo "  C:\\Program Files\\Common Files\\VST3\\"
echo "and/or the .clap file to:"
echo "  C:\\Program Files\\Common Files\\CLAP\\"
