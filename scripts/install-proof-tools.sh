#!/bin/bash
# install-proof-tools.sh — install the formal-verification toolchain used by
# the proofs/ directory of this crate: Charon + Aeneas (Rust -> LLBC -> Lean)
# and the Lean 4 toolchain the Aeneas backend targets.
#
# Everything lands under one directory (default: ~/.proof-tools) so a node
# can be provisioned or wiped without touching system state:
#
#   $PROOF_TOOLS_DIR/
#     bin/opam        (only if opam had to be bootstrapped)
#     opam/           OPAMROOT: OCaml 5.3.0 switch + Aeneas/Charon-ML deps
#     aeneas/         Aeneas clone, with charon/ inside at the pinned commit
#     elan/           ELAN_HOME: Lean toolchain from aeneas/backends/lean
#
# Idempotent: safe to re-run; completed steps are skipped.
#
# Verified end-to-end 2026-11-07 on aletheia-core (Ubuntu, x86_64).
# Linux only. The recipe below encodes several failures that cost real
# time on first install; each is flagged where it is handled.
#
# Usage:  scripts/install-proof-tools.sh
# Knobs:  PROOF_TOOLS_DIR (default ~/.proof-tools), OCAML_VERSION (5.3.0),
#         OPAM_VERSION (2.2.1), AENEAS_GIT (upstream AeneasVerif/aeneas)

set -euo pipefail

PROOF_TOOLS_DIR="${PROOF_TOOLS_DIR:-$HOME/.proof-tools}"
OCAML_VERSION="${OCAML_VERSION:-5.3.0}"
OPAM_VERSION="${OPAM_VERSION:-2.2.1}"
AENEAS_GIT="${AENEAS_GIT:-https://github.com/AeneasVerif/aeneas}"

log() { echo "[proof-tools] $*"; }
die() { echo "[proof-tools] ERROR: $*" >&2; exit 1; }

[ "$(uname -s)" = "Linux" ] || die "this installer is verified on Linux only"
command -v git    >/dev/null || die "git is required"
command -v curl   >/dev/null || die "curl is required"
command -v rustup >/dev/null || die "rustup is required (Charon builds with a pinned Rust nightly)"

mkdir -p "$PROOF_TOOLS_DIR/bin"
export OPAMROOT="$PROOF_TOOLS_DIR/opam"
export OPAMYES=1
export ELAN_HOME="$PROOF_TOOLS_DIR/elan"
export PATH="$PROOF_TOOLS_DIR/bin:$ELAN_HOME/bin:$PATH"

# ---------------------------------------------------------------
# 1. opam (bootstrap the standalone binary if none is installed)
# ---------------------------------------------------------------
if ! command -v opam >/dev/null; then
  if [ ! -x "$PROOF_TOOLS_DIR/bin/opam" ]; then
    case "$(uname -m)" in
      x86_64)        OPAM_ASSET="opam-$OPAM_VERSION-x86_64-linux" ;;
      aarch64|arm64) OPAM_ASSET="opam-$OPAM_VERSION-arm64-linux" ;;
      *) die "no standalone opam asset known for arch $(uname -m); install opam manually" ;;
    esac
    log "bootstrapping opam $OPAM_VERSION ($OPAM_ASSET)"
    curl -sL -o "$PROOF_TOOLS_DIR/bin/opam" \
      "https://github.com/ocaml/opam/releases/download/$OPAM_VERSION/$OPAM_ASSET"
    chmod +x "$PROOF_TOOLS_DIR/bin/opam"
  fi
fi
log "opam: $(opam --version)"

# ---------------------------------------------------------------
# 2. OCaml switch
# ---------------------------------------------------------------
if [ ! -d "$OPAMROOT" ]; then
  # --disable-sandboxing: bubblewrap is absent on most fresh nodes, and
  # opam's sandbox wrapper fails every build without it.
  opam init --bare --disable-sandboxing -y
fi
if ! opam switch list --short 2>/dev/null | grep -qx "$OCAML_VERSION"; then
  log "creating OCaml $OCAML_VERSION switch (compiles OCaml; takes a while)"
  opam switch create "$OCAML_VERSION" -y
fi
eval "$(opam env --switch="$OCAML_VERSION")"

# ---------------------------------------------------------------
# 3. Aeneas + Charon sources (Charon at the commit Aeneas pins)
# ---------------------------------------------------------------
AENEAS_DIR="$PROOF_TOOLS_DIR/aeneas"
if [ ! -d "$AENEAS_DIR" ]; then
  log "cloning Aeneas"
  git clone "$AENEAS_GIT" "$AENEAS_DIR"
fi
CHARON_PIN="$(tail -1 "$AENEAS_DIR/charon-pin" | tr -d '[:space:]')"
if [ ! -d "$AENEAS_DIR/charon" ]; then
  log "cloning Charon into aeneas/charon"
  git clone https://github.com/AeneasVerif/charon "$AENEAS_DIR/charon"
fi
if [ "$(git -C "$AENEAS_DIR/charon" rev-parse HEAD)" != "$CHARON_PIN" ]; then
  git -C "$AENEAS_DIR/charon" checkout -q "$CHARON_PIN"
fi
log "charon at pin $CHARON_PIN"

# ---------------------------------------------------------------
# 4. Charon (Rust binary + driver) on its pinned nightly
# ---------------------------------------------------------------
# Charon's rust-toolchain file names a nightly and several cross targets.
# Only the host target is needed to build and run Charon here, and setting
# RUSTUP_TOOLCHAIN keeps rustup from trying to install the cross targets
# listed in that file.
CHARON_CHANNEL="$(sed -n 's/.*channel *= *"\(.*\)".*/\1/p' "$AENEAS_DIR/charon/rust-toolchain" | head -1)"
[ -n "$CHARON_CHANNEL" ] || die "could not read Rust channel from charon/rust-toolchain"
HOST_TARGET="$(rustc -vV | sed -n 's/^host: //p')"
if ! rustup toolchain list | grep -q "^$CHARON_CHANNEL"; then
  log "installing Rust $CHARON_CHANNEL (host target only)"
  rustup toolchain install "$CHARON_CHANNEL" \
    --component rustc-dev,llvm-tools,rust-src,miri \
    --target "$HOST_TARGET"
fi
if [ ! -x "$AENEAS_DIR/charon/bin/charon" ]; then
  log "building Charon (serial make; cargo parallelises internally)"
  (cd "$AENEAS_DIR/charon" && RUSTUP_TOOLCHAIN="$CHARON_CHANNEL" make)
fi
[ -x "$AENEAS_DIR/charon/bin/charon" ] || die "charon build did not produce bin/charon"

# ---------------------------------------------------------------
# 5. OCaml dependencies for Aeneas AND for Charon-ML
# ---------------------------------------------------------------
# The ./ in "./src" and "./charon" is load-bearing: opam treats a bare
# word as a package name, and `opam install src` silently installs
# nothing (exit 0, "Nothing to do") — the deps simply never land and
# the failure surfaces much later as `dune: not found`.
# Charon-ML's deps (ppx_deriving_hash and friends) are a separate
# package set from Aeneas's own; both must be installed.
log "installing opam dependencies (aeneas + charon-ml)"
(cd "$AENEAS_DIR" && opam install ./src --deps-only --with-test -y)
(cd "$AENEAS_DIR" && opam install ./charon --deps-only --with-test -y)

# ---------------------------------------------------------------
# 6. Aeneas itself
# ---------------------------------------------------------------
# Build SERIALLY. Aeneas's Makefile fans out to build-bin / build-lib /
# build-runner targets that share one dune _build directory; parallel
# make makes them collide ("Another Dune instance is currently
# running"). Dune parallelises inside each invocation on its own.
if [ ! -x "$AENEAS_DIR/bin/aeneas" ]; then
  log "building Aeneas (serial make)"
  (cd "$AENEAS_DIR" && make)
fi
[ -x "$AENEAS_DIR/bin/aeneas" ] || die "aeneas build did not produce bin/aeneas"

# ---------------------------------------------------------------
# 7. Lean, at the version Aeneas's backend pins
# ---------------------------------------------------------------
if [ ! -x "$ELAN_HOME/bin/elan" ]; then
  log "installing elan"
  curl -sSf https://raw.githubusercontent.com/leanprover/elan/master/elan-init.sh \
    | sh -s -- -y --default-toolchain none
fi
LEAN_TOOLCHAIN="$(tr -d '[:space:]' < "$AENEAS_DIR/backends/lean/lean-toolchain")"
if ! elan toolchain list | grep -qF "$LEAN_TOOLCHAIN"; then
  log "installing Lean toolchain $LEAN_TOOLCHAIN"
  elan toolchain install "$LEAN_TOOLCHAIN"
fi
elan default "$LEAN_TOOLCHAIN"

# ---------------------------------------------------------------
# 8. Verify and report
# ---------------------------------------------------------------
log "installed:"
"$AENEAS_DIR/charon/bin/charon" --version 2>/dev/null | head -1 || true
"$AENEAS_DIR/bin/aeneas" --help 2>&1 | head -1 || true
lean --version
cat <<EOF

Proof tools ready under $PROOF_TOOLS_DIR.

To extract and translate this crate:

  export PATH="$AENEAS_DIR/bin:$AENEAS_DIR/charon/bin:\$PATH"
  export RUSTUP_TOOLCHAIN="$CHARON_CHANNEL"
  charon cargo --preset=aeneas --dest-file=libphext.llbc
  aeneas -backend lean libphext.llbc \\
    -dest proofs -subdir /Libphext/Code -split-files -namespace Libphext

Trust note: proofs produced this way are checked by Lean's kernel.
Charon and Aeneas themselves are trusted, unverified translators;
hand-written models of external crates (see proofs/) are trusted too.
EOF
