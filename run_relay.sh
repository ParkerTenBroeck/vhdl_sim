#!/usr/bin/env bash
set -euo pipefail

pushd conn >/dev/null
cargo build --release
popd >/dev/null


pushd relay >/dev/null
cargo run --release
popd >/dev/null