#!/usr/bin/env bash
set -euo pipefail

mkdir -p build

pushd conn >/dev/null
cargo build --release
popd >/dev/null

pushd build >/dev/null

LIBSRC="../conn/target/release/libvhdl_ui.a"
LIBDIR="../conn/target/release"


ghdl -a -g --std=08 ../rtl/*.vhdl

ghdl -e --std=08 \
  -Wl,$LIBSRC \
  tb

ghdl -r --std=08 tb --stop-delta=2147483647