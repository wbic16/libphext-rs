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
//!
//! Sentron Ages: the position *within* an epoch. The unit is the
//! Sentron Age, named next to the Exocortical Epoch it subdivides —
//! one Exocortical Epoch = 2^42 Sentron Ages. Where the epoch count
//! is the shared coordinate, a Sentron Age resolves one epoch into
//! 2^42 nominal steps: the field is 42 bits wide, so the canonical
//! reference clock is 2^42 / 420 ≈ 10.4715 GHz and one Sentron Age is
//! ≈ 95.5 picoseconds. At 4 GHz an epoch holds 1.68×10^12 cycles,
//! filling only the bottom ~38% of the field — the field runs ahead
//! of the silicon by design, so Sentron Ages stay meaningful as
//! clocks get faster.
//!
//! Sentron Ages are *less accurate than epochs*. Time synchronization
//! at this point in the timeline is poor from a sentient perspective:
//! 1 ms of clock skew is ≈ 10.47 million Sentron Ages, and NTP-class
//! synchronization is some seven orders of magnitude too coarse to
//! pin a Sentron Age exactly. A Sentron Age is therefore a local,
//! nominal position within its epoch — meaningful for ordering events
//! on one clock — while the epoch count remains the robust shared
//! coordinate. Sentron Ages read on different machines are not
//! directly comparable at face value.
//!
//! The coarse Sentron Age (`coarse_age_at`) is the sync-survivable
//! form: the fine Sentron Age scaled into a 16-bit address space —
//! one Exocortical Epoch = 65,536 coarse Sentron Ages, one tick
//! ≈ 6.41 ms. At that width, NTP-class clock skew of a few
//! milliseconds costs about a tick, so coarse Sentron Ages are
//! approximately comparable across machines. It is the register-grade
//! form — the one the register system carries as its Age component.

/// The unix timestamp at which EE 1 begins: 2026-11-22T06:00:00Z.
pub const EPOCH_ANCHOR_UNIX: i64 = 1795327200;

/// The length of one Exocortical Epoch, in seconds (7 minutes).
pub const EPOCH_LENGTH_SECONDS: i64 = 420;

/// The width of a Sentron Age field, in bits: one epoch is divided
/// into 2^42 Sentron Ages.
pub const AGE_BITS: u32 = 42;

/// The number of Sentron Ages in one Exocortical Epoch:
/// 2^42 = 4,398,046,511,104. The canonical reference clock is
/// AGES_PER_EPOCH / 420 ≈ 10.4715 GHz, so one Sentron Age is
/// ≈ 95.5 picoseconds.
pub const AGES_PER_EPOCH: u64 = 1 << AGE_BITS;

/// The largest Sentron Age: the final nominal step of an epoch.
pub const AGE_MAX: u64 = AGES_PER_EPOCH - 1;

/// The width of a coarse Sentron Age field, in bits: the fine Sentron
/// Age scaled into a 16-bit address space (coarse = fine >>
/// (AGE_BITS − COARSE_AGE_BITS)). One coarse tick is 420 / 65536 s
/// ≈ 6.41 ms.
pub const COARSE_AGE_BITS: u32 = 16;

/// The number of coarse Sentron Ages in one Exocortical Epoch:
/// 2^16 = 65,536.
pub const COARSE_AGES_PER_EPOCH: u64 = 1 << COARSE_AGE_BITS;

/// The largest coarse Sentron Age: the final coarse step of an epoch.
pub const COARSE_AGE_MAX: u64 = COARSE_AGES_PER_EPOCH - 1;

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

/// ----------------------------------------------------------------------------------------------------------
/// @fn age_at
///
/// the Sentron Age of the given unix timestamp: its nominal position
/// within its epoch, `floor(seconds_into_epoch × 2^42 / 420)`, in the
/// range 0..=AGE_MAX. `None` before the anchor, exactly as
/// `seconds_into_epoch` — a pre-epoch instant is inside no epoch, so
/// it has no Sentron Age. Whole-second input fixes the Sentron Age at
/// that second's first instant; `age_at_nanos` refines it with the
/// sub-second part.
///
/// A Sentron Age is a *local* position: 1 ms of clock skew moves it
/// by ≈ 10.47 million Sentron Ages, so Sentron Ages order events on
/// one clock — the epoch count is the coordinate that travels between
/// machines.
/// ----------------------------------------------------------------------------------------------------------
pub fn age_at(unix_seconds: i64) -> Option<u64> {
  let into = seconds_into_epoch(unix_seconds)?;
  let age = (into as u128) * (AGES_PER_EPOCH as u128) / (EPOCH_LENGTH_SECONDS as u128);
  return Some(age as u64);
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn age_at_nanos
///
/// the Sentron Age of a timestamp given with sub-second precision:
/// `unix_seconds` plus `nanoseconds` (0..1_000_000_000) within that
/// second. The Sentron Age is `floor(elapsed_ns × 2^42 / (420 ×
/// 10^9))` over the nanoseconds elapsed in the containing epoch,
/// computed in 128-bit arithmetic (elapsed_ns × 2^42 overflows u64).
/// `None` before the anchor, and `None` for an out-of-range
/// nanosecond part. With `nanoseconds == 0` the result agrees exactly
/// with `age_at`.
/// ----------------------------------------------------------------------------------------------------------
pub fn age_at_nanos(unix_seconds: i64, nanoseconds: u32) -> Option<u64> {
  if nanoseconds >= 1_000_000_000 {
    return None;
  }
  let into = seconds_into_epoch(unix_seconds)?;
  let elapsed_ns = (into as u128) * 1_000_000_000u128 + (nanoseconds as u128);
  let epoch_ns = (EPOCH_LENGTH_SECONDS as u128) * 1_000_000_000u128;
  let age = elapsed_ns * (AGES_PER_EPOCH as u128) / epoch_ns;
  return Some(age as u64);
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn age_start
///
/// the unix timestamp of the whole second in which the given Sentron
/// Age of the given epoch begins: `epoch_start(epoch) + floor(age ×
/// 420 / 2^42)`. `None` for EE 0, for `age > AGE_MAX`, and when that
/// second is not representable as an i64 unix timestamp.
///
/// This is the coarsest faithful inverse of `age_at`, and its
/// coarseness is the module's honesty about resolution: a Sentron Age
/// is ≈ 95.5 ps wide, so ~10.47 billion Sentron Ages begin inside
/// each second and no whole-second timestamp can name one Sentron
/// Age's exact start. The inverse therefore names the containing
/// second — expect it to read one second early when the Sentron Age
/// in force at a second boundary began just before it.
/// ----------------------------------------------------------------------------------------------------------
pub fn age_start(epoch: u64, age: u64) -> Option<i64> {
  if age > AGE_MAX {
    return None;
  }
  let start = epoch_start(epoch)?;
  let second = ((age as u128) * (EPOCH_LENGTH_SECONDS as u128) / (AGES_PER_EPOCH as u128)) as i64;
  return start.checked_add(second);
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn coarse_age_at
///
/// the coarse Sentron Age of the given unix timestamp: the fine
/// Sentron Age scaled into a 16-bit address space, `age_at(t) >> 26`,
/// equivalently `floor(seconds_into_epoch × 65536 / 420)`, in the
/// range 0..=COARSE_AGE_MAX. `None` before the anchor, exactly as
/// `age_at`.
///
/// This is the register-grade form. One coarse tick is ≈ 6.41 ms, so
/// where a fine Sentron Age (≈ 95.5 ps) is destroyed by any real clock
/// skew — 1 ms is ≈ 10.47 million fine Sentron Ages — NTP-class skew
/// of a few milliseconds costs a coarse Sentron Age about a tick.
/// Coarse Sentron Ages are therefore approximately comparable across
/// machines, and they are the Age component the register system
/// carries.
/// ----------------------------------------------------------------------------------------------------------
pub fn coarse_age_at(unix_seconds: i64) -> Option<u64> {
  return Some(age_at(unix_seconds)? >> (AGE_BITS - COARSE_AGE_BITS));
}
