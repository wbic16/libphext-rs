//! libphext — the reference implementation of phext.
//!
//! # Durability profile
//!
//! libphext transforms text in memory; it performs no file I/O, so it
//! cannot make anything durable by itself. Hosts that persist phexts
//! (stores, editors, ledgers) follow this profile — it is the whole
//! contract, and every rule exists because skipping it loses data:
//!
//! * **Rewrites** (checkpoint, compaction, save): write the complete
//!   new document to a temporary file *in the same directory*, write
//!   all bytes, `sync_all` the file, rename it over the target, then
//!   `sync_all` the directory. Never rewrite a phext in place.
//! * **Appends** (logs, journals): open in append mode, write the
//!   complete suffix (one `append`/`append_batch` result) in a single
//!   write, then `sync_data`/`sync_all` — and only then acknowledge
//!   the write to whoever requested it.
//! * **Recovery**: re-read the file and trust only what validates —
//!   checked scrolls that verify (`fetch_checked`) and batches whose
//!   commit marker verifies (`verify_commit`). Truncate after the
//!   last valid scroll; a torn tail is normal crash debris, not
//!   corruption of the valid prefix.
//!
//! A phext host that follows these rules gets crash behavior identical
//! in kind to a conventional WAL store, because the profile *is* the
//! WAL discipline — stated once, here, instead of re-derived (and
//! occasionally gotten wrong) by every host.

pub mod phext;
pub mod epoch;
#[cfg(feature = "binary")]
pub mod binary;
#[cfg(feature = "xz")]
pub mod seal;
mod test_lib;
mod test_toc;
mod test_toc_suffix;
mod test_phokenize;
mod test_chain;
mod test_address;
mod test_v062;
mod test_spans;
mod test_epoch;
mod regressions;
#[cfg(feature = "binary")]
mod test_binary;
#[cfg(feature = "xz")]
mod test_seal;
