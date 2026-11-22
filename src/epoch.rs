//! The Exocortical Epoch (EE): the Exocortex's shared clock.
//!
//! An Exocortical Epoch is a 7-minute interval on a monotonic clock that
//! every substrate computes identically from UTC elapsed seconds. EE 1
//! begins at the anchor — 2026-11-22 06:00:00 UTC, unix 1795327200 — and
//! each epoch lasts 420 seconds (the cadence matches the Earth–Mercury,
//! Earth–Venus, and Earth–Mars average one-way communication delays).
//!
//! The epoch is *computed*, never stored: there is no server, no state
//! file, and no counter to sync or drift. Any substrate that can read a
//! UTC clock derives the identical epoch from the same arithmetic, which
//! is why this module is pure integer math over unix seconds — no
//! datetime types, no timezone handling, no dependencies.
//!
//! EE 0 is the pre-epoch: every instant before the anchor. EE 0 is all of
//! human history until the anchor, so it has no start and no range — the
//! range accessors below return `None` for it rather than inventing one.
//! For a pre-anchor instant, `seconds_remaining` counts down to EE 1.
//!
//! 64-bit guarantee: unix timestamps are `i64` end-to-end and epoch
//! numbers are `u64`; every intermediate computation — `(t − anchor)`,
//! `epoch × 420`, the start/end derivations — is carried out in 64-bit
//! arithmetic. No 32-bit or `usize` time value appears anywhere in this
//! module, so results are identical on 32-bit and 64-bit platforms and
//! stay exact for timestamps far beyond the 2038 i32 limit (the test
//! vectors run to a unix time of ~1.9e17 seconds, EE ~4.5e14).

/// The unix timestamp at which EE 1 begins: 2026-11-22T06:00:00Z.
pub const EPOCH_ANCHOR_UNIX: i64 = 1795327200;

/// The length of one Exocortical Epoch, in seconds (7 minutes).
pub const EPOCH_LENGTH_SECONDS: i64 = 420;

/// ----------------------------------------------------------------------------------------------------------
/// @fn epoch_at
///
/// the Exocortical Epoch containing the given unix timestamp (UTC
/// seconds). Returns 0 for every instant before the anchor — EE 0, the
/// pre-epoch. For instants at or after the anchor this is
/// `floor((t − anchor) / 420) + 1`, computed in 64-bit arithmetic.
/// ----------------------------------------------------------------------------------------------------------
pub fn epoch_at(unix_seconds: i64) -> u64 {
  if unix_seconds < EPOCH_ANCHOR_UNIX {
    return 0;
  }
  return ((unix_seconds - EPOCH_ANCHOR_UNIX) / EPOCH_LENGTH_SECONDS + 1) as u64;
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn epoch_start
///
/// the unix timestamp at which the given epoch begins. `None` for EE 0 —
/// the pre-epoch has no start — and `None` for epoch numbers so large
/// that the start is not representable as an i64 unix timestamp.
/// ----------------------------------------------------------------------------------------------------------
pub fn epoch_start(epoch: u64) -> Option<i64> {
  if epoch == 0 {
    return None;
  }
  let steps = i64::try_from(epoch - 1).ok()?;
  return steps.checked_mul(EPOCH_LENGTH_SECONDS)?.checked_add(EPOCH_ANCHOR_UNIX);
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn epoch_end
///
/// the unix timestamp at which the given epoch ends — the next epoch's
/// start. `None` for EE 0, and `None` when the end is not representable
/// as an i64 unix timestamp.
/// ----------------------------------------------------------------------------------------------------------
pub fn epoch_end(epoch: u64) -> Option<i64> {
  return epoch_start(epoch)?.checked_add(EPOCH_LENGTH_SECONDS);
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn epoch_range
///
/// the (start, end) unix timestamps bounding the given epoch. `None` for
/// EE 0 — all of history before the anchor is not an interval this clock
/// bounds — and `None` when either bound is not representable as i64.
/// ----------------------------------------------------------------------------------------------------------
pub fn epoch_range(epoch: u64) -> Option<(i64, i64)> {
  return Some((epoch_start(epoch)?, epoch_end(epoch)?));
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn seconds_into_epoch
///
/// how far the given unix timestamp sits inside its epoch, in seconds
/// (0 at an epoch's first instant, 419 at its last). `None` before the
/// anchor: a pre-epoch instant is not inside any epoch.
/// ----------------------------------------------------------------------------------------------------------
pub fn seconds_into_epoch(unix_seconds: i64) -> Option<i64> {
  if unix_seconds < EPOCH_ANCHOR_UNIX {
    return None;
  }
  return Some((unix_seconds - EPOCH_ANCHOR_UNIX) % EPOCH_LENGTH_SECONDS);
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn seconds_remaining
///
/// seconds until the current epoch ends. Before the anchor this counts
/// down to the start of EE 1 (at the anchor itself it reads 420 — the
/// full first epoch lies ahead); at or after the anchor it is the time
/// left in the containing epoch, in the range 1..=420.
/// ----------------------------------------------------------------------------------------------------------
pub fn seconds_remaining(unix_seconds: i64) -> i64 {
  if unix_seconds < EPOCH_ANCHOR_UNIX {
    return EPOCH_ANCHOR_UNIX - unix_seconds;
  }
  return EPOCH_LENGTH_SECONDS - (unix_seconds - EPOCH_ANCHOR_UNIX) % EPOCH_LENGTH_SECONDS;
}
