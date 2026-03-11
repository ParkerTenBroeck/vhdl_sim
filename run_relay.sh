#!/usr/bin/env bash
set -euo pipefail


pushd relay >/dev/null
cargo +nightly build -Z bindeps --release
popd >/dev/null

./target/release/relay "$@"
