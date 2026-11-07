//! The xz seal: canonical compression for a finished phext.
//!
//! Compression is a seal, not a working format. A phext is authored, edited,
//! and queried as plain text; when a phext is finished — archived, published,
//! or transported — it may be sealed: xz-compressed as a whole. Unsealing
//! restores the exact original bytes.
//!
//! Seal specification:
//! * The seal is a standard xz stream (liblzma) over the phext's exact
//!   bytes. Decoders in any implementation must accept any valid xz stream.
//! * This implementation encodes at preset 6. The preset is an encoder
//!   choice, not a format requirement; seals produced at other presets
//!   unseal identically.
//! * Sealing is deterministic for a given encoder and preset: the same
//!   phext bytes seal to the same sealed bytes.

use std::io::{self, Read, Write};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

/// The xz preset this implementation seals with.
pub const SEAL_PRESET: u32 = 6;

/// ----------------------------------------------------------------------------------------------------------
/// @fn seal
///
/// seals a phext: xz-compresses its full UTF-8 bytes.
/// ----------------------------------------------------------------------------------------------------------
pub fn seal(input: &str) -> io::Result<Vec<u8>> {
  return seal_bytes(input.as_bytes());
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn seal_bytes
///
/// seals raw phext bytes (see `seal`).
/// ----------------------------------------------------------------------------------------------------------
pub fn seal_bytes(input: &[u8]) -> io::Result<Vec<u8>> {
  let mut encoder = XzEncoder::new(Vec::new(), SEAL_PRESET);
  encoder.write_all(input)?;
  return encoder.finish();
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn unseal
///
/// unseals a sealed phext back to its exact original bytes. Returns the
/// io error when the data is not a valid xz stream.
/// ----------------------------------------------------------------------------------------------------------
pub fn unseal(data: &[u8]) -> io::Result<Vec<u8>> {
  let mut decoder = XzDecoder::new(data);
  let mut out = Vec::new();
  decoder.read_to_end(&mut out)?;
  return Ok(out);
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn unseal_string
///
/// unseals directly to text. Fails with InvalidData when the unsealed bytes
/// are not valid UTF-8; to unseal foreign content that may carry non-UTF-8
/// scrolls, `unseal` to bytes and pass them through `binary::ingest`, which
/// applies the base64-verbatim ingestion rule instead of failing.
/// ----------------------------------------------------------------------------------------------------------
pub fn unseal_string(data: &[u8]) -> io::Result<String> {
  let bytes = unseal(data)?;
  return String::from_utf8(bytes).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
}
