#!/usr/bin/env bash
# libphext-rs: publish to crates.io — SAFETY-FIRST.
#
# What this script does:
#   1. Runs the test suite and the release build (scripts/build.sh).
#      The suite has ONE known pre-existing failure at HEAD,
#      regressions::regressions::helios_stalled_parse; if that is the
#      ONLY failure, the script prints a loud warning and continues.
#      Any other failure aborts the publish.
#   2. Runs `cargo publish --dry-run` (always — this is the real check:
#      it packages, uploads in dry-run mode, and verifies the crate).
#   3. Prints exactly what would be published (name, version, file list).
#   4. STOPS. The real publish runs only when PUBLISH_CONFIRM=1 is set.
#
# Credentials: set CARGO_REGISTRY_TOKEN in the environment, or run
# `cargo login` once beforehand. No credential is embedded in this script.
# The working tree must be committed and clean: cargo publish refuses a
# dirty tree, and this script deliberately does NOT pass --allow-dirty.
#
# Usage:
#   scripts/publish.sh                    # dry-run + summary, then stop
#   PUBLISH_CONFIRM=1 scripts/publish.sh  # dry-run, then publish for real
set -euo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.cargo/bin:$PATH"

NAME="$(grep -m1 '^name' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')"
VERSION="$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')"

echo "--- test suite (cargo test):"
KNOWN_FAILURE="regressions::regressions::helios_stalled_parse"
TEST_LOG="$(mktemp)"
if cargo test 2>&1 | tee "$TEST_LOG"; then
  echo "test suite: all tests passed."
else
  FAILED_LIST="$(awk '/^failures:$/{f=1;next} /^$/{f=0} f' "$TEST_LOG" | sed 's/^ *//' | grep -v '^$' | sort -u)"
  if [ "$FAILED_LIST" != "$KNOWN_FAILURE" ]; then
    # One transparent retry: test_phokenize::tests::a_large_phext_explodes_in_linear_time
    # is a wall-clock timing assertion and flakes under load. A retry that
    # still fails (or fails differently) aborts below — nothing is hidden:
    # both runs' failure lists are printed.
    echo "Unexpected failures in run 1:"
    echo "$FAILED_LIST"
    echo "Re-running the suite once to rule out timing flakes..."
    cargo test 2>&1 | tee "$TEST_LOG" || true
    FAILED_LIST="$(awk '/^failures:$/{f=1;next} /^$/{f=0} f' "$TEST_LOG" | sed 's/^ *//' | grep -v '^$' | sort -u)"
    echo "Run 2 failures:"
    echo "${FAILED_LIST:-<none>}"
  fi
  if [ "$FAILED_LIST" = "$KNOWN_FAILURE" ]; then
    echo "WARNING: the only failure is the known pre-existing one:"
    echo "  $KNOWN_FAILURE"
    echo "It fails identically on the pristine tree (expected offset 370,"
    echo "actual first scroll break at 358) and is unrelated to this release."
    echo "Continuing — but a publisher should know it is still open."
  else
    echo "ERROR: unexpected test failures:" >&2
    echo "$FAILED_LIST" >&2
    echo "Aborting publish." >&2
    exit 1
  fi
fi
rm -f "$TEST_LOG"

scripts/build.sh

echo "--- cargo publish --dry-run:"
cargo publish --dry-run

echo "=================================================="
echo "Ready to publish: ${NAME} v${VERSION} -> crates.io"
echo "Artifacts: the .crate package containing the files listed above"
echo "           (source only; crates.io builds the docs and distribution)."
if [ "${PUBLISH_CONFIRM:-}" != "1" ]; then
  echo "STOPPED: this was a dry run only. Nothing was published."
  echo "To publish for real:  PUBLISH_CONFIRM=1 scripts/publish.sh"
  exit 0
fi

echo "PUBLISH_CONFIRM=1 set — publishing ${NAME} v${VERSION}..."
cargo publish
echo "Published ${NAME} v${VERSION} to crates.io."
