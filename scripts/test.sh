#!/usr/bin/env bash
# libphext-rs: run the full test suite.
#
# Underlying command: cargo test
# The exit code is propagated honestly. NOTE: one failure is pre-existing
# at HEAD — regressions::regressions::helios_stalled_parse (the test expects
# offset 370; the file's first scroll break sits at offset 358). It fails
# identically on the pristine tree and is unrelated to the 0.4.0 changes;
# see the Publish section of readme.md before releasing.
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"
cargo test
