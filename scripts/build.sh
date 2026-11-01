#!/usr/bin/env bash
# libphext-rs: release build + crates.io package sanity check.
#
# Underlying commands:
#   cargo build --release            (optimized library + examples in target/release)
#   cargo package --list --allow-dirty
#     (exactly the files `cargo publish` would upload; --allow-dirty because
#      this is a local sanity check — publish.sh itself stays strict and
#      lets cargo refuse a dirty tree)
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release
echo "--- files that would be published to crates.io:"
cargo package --list --allow-dirty
