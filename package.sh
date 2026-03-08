#!/bin/sh
set -e

echo "Cleaning previous build artifacts..."
cargo clean

echo "Building release binary..."
cargo build --release
if [ $? -ne 0 ]; then
    echo "Build failed."
    exit 1
fi
echo "Build complete."

echo "Running tests..."
cargo test
if [ $? -ne 0 ]; then
    echo "Tests failed."
    exit 1
fi
echo "Tests passed."

BIN=target/release/hourglass

STAGING=$(mktemp -d)
trap 'rm -rf "$STAGING"' EXIT

mkdir -p "$STAGING/hourglass"
cp "$BIN"                       "$STAGING/hourglass/"
cp install.sh                   "$STAGING/hourglass/"
cp uninstall.sh                 "$STAGING/hourglass/"
cp -r config                    "$STAGING/hourglass/"

TARBALL="hourglass.tar.gz"
tar -czf "$TARBALL" -C "$STAGING" hourglass

echo "Created $TARBALL"
