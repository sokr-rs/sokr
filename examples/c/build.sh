#!/bin/bash
# Build script for SOKR C example
#
# This script compiles the hello_compute.c example against the built
# sokr and sokr-cpu libraries.

set -e

EXAMPLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOKR_DIR="$(cd "$EXAMPLE_DIR/../.." && pwd)"
SOKR_PLUGINS_DIR="$(cd "$SOKR_DIR/.." && pwd)/sokr-plugins"

CFLAGS="-Wall -Wextra -O2"
INCLUDE_DIR="$SOKR_DIR/include"
SOKR_LIB_DIR="$SOKR_DIR/target/release"
SOKR_CPU_LIB_DIR="$SOKR_PLUGINS_DIR/target/release"

echo "Building SOKR C example: hello_compute"
echo "======================================="
echo "Sokr dir:        $SOKR_DIR"
echo "Include dir:     $INCLUDE_DIR"
echo "Sokr lib dir:    $SOKR_LIB_DIR"
echo "Sokr-cpu lib dir: $SOKR_CPU_LIB_DIR"
echo ""

# Ensure libraries are built
echo "Building sokr libraries..."
cd "$SOKR_DIR"
cargo build --lib --release --quiet
cd "$SOKR_PLUGINS_DIR"
cargo build -p sokr-cpu --release --quiet
cd "$EXAMPLE_DIR"

echo "Compiling hello_compute..."
gcc $CFLAGS \
    -I"$INCLUDE_DIR" \
    -L"$SOKR_LIB_DIR" \
    -L"$SOKR_CPU_LIB_DIR" \
    -o hello_compute hello_compute.c \
    -lsokr_cpu -lsokr

echo "✓ Build successful: ./hello_compute"
echo ""
echo "To run: LD_LIBRARY_PATH=$SOKR_LIB_DIR:$SOKR_CPU_LIB_DIR ./hello_compute"
