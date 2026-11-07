#!/bin/bash
# Regenerate the Lean model of libphext under proofs/ from the Rust sources.
#
# Pipeline: Charon extracts LLBC from the crate (with explode/implode kept
# opaque - they are the crate's only HashMap users, and HashMap has no Lean
# model in Aeneas), then Aeneas translates the LLBC to Lean.
#
# Expects charon and aeneas on PATH (see scripts/install-proof-tools.sh),
# and RUSTUP_TOOLCHAIN set to the nightly Charon needs
# (nightly-2026-09-17 at the pin this was developed against).
set -e
cd "$(dirname "$0")/.."

charon cargo --preset=aeneas \
  --opaque=libphext::phext::explode --opaque=libphext::phext::implode \
  --dest-file=libphext.llbc

rm -rf proofs-gen
aeneas -backend lean libphext.llbc -dest proofs-gen -subdir /Libphext/Code \
  -split-files -namespace Libphext -no-progress-bar

cp proofs-gen/Libphext/Code/Types.lean proofs-gen/Libphext/Code/Funs.lean \
  proofs/Libphext/Code/
cp proofs-gen/Libphext/Code/FunsExternal_Template.lean \
  proofs/Libphext/Code/FunsExternal.lean
cp proofs-gen/Libphext/Code/TypesExternal_Template.lean \
  proofs/Libphext/Code/TypesExternal.lean
echo "model regenerated under proofs/Libphext/Code/"
