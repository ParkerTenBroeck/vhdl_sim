#!/usr/bin/env bash
set -euo pipefail

mkdir -p build

# Build Rust shared library
pushd conn >/dev/null
cargo build --release
popd >/dev/null

pushd build >/dev/null

LIBSRC="../conn/target/release/libvhdl_ui.a"
LIBDIR="../conn/target/release"


ghdl -a --std=08 ../rtl/*.vhdl

ghdl -e --std=08 \
  -Wl,"$LIBSRC" \
  -Wl,-Wl,-rpath -Wl,-Wl,$LIBDIR \
   tb


echo "=== Running sim ==="
echo "Connect and stream inputs using:"
echo "  nc 127.0.0.1 5555"
echo "Then type lines like:"
echo "  sw=1"
echo "  key=15"
echo "  key=7   (press KEY3 if bit3 becomes 0, etc; active-low)"
echo

ghdl -r --std=08 tb