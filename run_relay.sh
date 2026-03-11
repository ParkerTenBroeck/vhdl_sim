#!/usr/bin/env bash
set -euo pipefail


pushd relay >/dev/null
cargo +nightly run -Z bindeps --release -- "$@"
popd >/dev/null
