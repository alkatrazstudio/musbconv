#!/usr/bin/env bash
set -e
cd "$(dirname -- "${BASH_SOURCE[0]}")"
RUSTFLAGS="-C link-arg=-s" cargo build --release
