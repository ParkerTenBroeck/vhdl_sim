#!/usr/bin/env bash
set -euo pipefail


pushd relay >/dev/null
cargo run --release -- "$@"
popd >/dev/null
