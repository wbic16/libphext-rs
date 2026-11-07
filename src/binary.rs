//! Binary scrolls: the canonical way to store arbitrary bytes inside phext.
//!
//! Phext scrolls are text (valid UTF-8). Arbitrary binary data rides as text
//! via base64. This module defines the one canonical binary scroll format, so
//! that every phext implementation (Rust, Python, Node, C#, C, ...) stores and
//! recognises binary payloads byte-for-byte identically:
//!
//! ```text
//! phext-binary:1:base64
//! <base64 body>
//! ```
//!
//! Format specification (version 1):
//! * Line 1 is exactly `phext-binary:1:base64`, followed by a single line
//!   break (0x0A).
//! * The body is RFC 4648 base64: standard alphabet (`A-Z`, `a-z`, `0-9`,
//!   `+`, `/`), padding (`=`) required, no line wrapping, no whitespace.
//! * An empty payload is the header line alone (header + line break + "").
//! * A scroll whose content begins with the header line is a binary scroll
//!   by definition. The format claims that namespace: text that matches it
//!   exactly is read as a binary scroll. A prefix-matching scroll whose body
//!   is not strict base64 is a *malformed* binary scroll.
//!
//! The ingestion rule (canonical behaviour, all ports):
//! > When non-UTF-8 data is observed in an incoming scroll update, we encode
//! > it with base64 verbatim.
//!
//! Byte-oriented entry points (`insert_bytes`, `replace_bytes`, `ingest`)
//! never reject and never panic on non-UTF-8 scroll data. Valid UTF-8 is
//! stored as ordinary text; anything else is stored as a binary scroll
//! holding the base64 of the exact observed bytes — verbatim, with no
//! normalisation — so the matching fetch returns the original bytes exactly.
//! Within a phext byte stream, the nine dimension-break bytes are always
//! structural; binary payloads that may contain them enter through the
//! update functions, armoured as base64.

use crate::phext::{self, Coordinate};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use std::collections::HashMap;

/// The first (and only header) line of a binary scroll, without its line break.
pub const BINARY_SCROLL_HEADER: &str = "phext-binary:1:base64";

/// Header line plus its terminating line break: the prefix of every binary scroll.
const BINARY_SCROLL_PREFIX: &str = "phext-binary:1:base64\n";

/// ----------------------------------------------------------------------------------------------------------
/// @fn is_binary_scroll
///
/// true when `scroll` carries the binary scroll header. Decoding success is a
/// separate question — see `decode_scroll`.
/// ----------------------------------------------------------------------------------------------------------
pub fn is_binary_scroll(scroll: &str) -> bool {
  return scroll.starts_with(BINARY_SCROLL_PREFIX);
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn encode_scroll
///
/// encodes raw bytes as a binary scroll (header line + base64 body)
/// ----------------------------------------------------------------------------------------------------------
pub fn encode_scroll(payload: &[u8]) -> String {
  let mut out = String::with_capacity(BINARY_SCROLL_PREFIX.len() + (payload.len() + 2) / 3 * 4);
  out.push_str(BINARY_SCROLL_PREFIX);
  STANDARD.encode_string(payload, &mut out);
  return out;
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn decode_scroll
///
/// decodes a binary scroll back to its raw bytes. Returns None when the
/// scroll is not a binary scroll, or when its body is not strict base64.
/// ----------------------------------------------------------------------------------------------------------
pub fn decode_scroll(scroll: &str) -> Option<Vec<u8>> {
  let body = scroll.strip_prefix(BINARY_SCROLL_PREFIX)?;
  return STANDARD.decode(body).ok();
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn insert_binary
///
/// stores `payload` at `location` as a binary scroll, unconditionally: the
/// caller has declared this scroll binary, even if the bytes happen to be
/// valid UTF-8.
/// ----------------------------------------------------------------------------------------------------------
pub fn insert_binary(input: String, location: Coordinate, payload: &[u8]) -> String {
  return phext::insert(input, location, &encode_scroll(payload));
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn replace_binary
///
/// replaces the scroll at `location` with `payload`, stored as a binary
/// scroll (see `insert_binary`).
/// ----------------------------------------------------------------------------------------------------------
pub fn replace_binary(input: &str, location: Coordinate, payload: &[u8]) -> String {
  return phext::replace(input, location, &encode_scroll(payload));
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn fetch_binary
///
/// retrieves the bytes held at `target`. A well-formed binary scroll decodes
/// to its payload. Any other scroll — plain text, a malformed binary scroll,
/// or no scroll at all — yields the scroll's own UTF-8 bytes (empty when the
/// coordinate is unoccupied), mirroring how `phext::fetch` returns "" there.
/// This function is total: it never panics and never loses scroll content.
/// ----------------------------------------------------------------------------------------------------------
pub fn fetch_binary(input: &str, target: Coordinate) -> Vec<u8> {
  let scroll = phext::fetch(input, target);
  match decode_scroll(&scroll) {
    Some(bytes) => return bytes,
    None => return scroll.into_bytes(),
  }
}

/// The stored form of an incoming scroll update, per the ingestion rule:
/// valid UTF-8 stays text; anything else becomes a binary scroll holding the
/// base64 of the exact observed bytes, verbatim.
fn stored_form(payload: &[u8]) -> String {
  match std::str::from_utf8(payload) {
    Ok(text) => return text.to_string(),
    Err(_) => return encode_scroll(payload),
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn insert_bytes
///
/// byte-oriented insert: stores `payload` at `location` under the ingestion
/// rule (UTF-8 as text, non-UTF-8 as a base64-verbatim binary scroll).
/// ----------------------------------------------------------------------------------------------------------
pub fn insert_bytes(input: String, location: Coordinate, payload: &[u8]) -> String {
  return phext::insert(input, location, &stored_form(payload));
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn replace_bytes
///
/// byte-oriented replace: stores `payload` at `location` under the ingestion
/// rule (UTF-8 as text, non-UTF-8 as a base64-verbatim binary scroll).
/// ----------------------------------------------------------------------------------------------------------
pub fn replace_bytes(input: &str, location: Coordinate, payload: &[u8]) -> String {
  return phext::replace(input, location, &stored_form(payload));
}

fn is_structural_break(byte: u8) -> bool {
  return byte == phext::SCROLL_BREAK_U8 ||
         byte == phext::SECTION_BREAK_U8 ||
         byte == phext::CHAPTER_BREAK_U8 ||
         byte == phext::BOOK_BREAK_U8 ||
         byte == phext::VOLUME_BREAK_U8 ||
         byte == phext::COLLECTION_BREAK_U8 ||
         byte == phext::SERIES_BREAK_U8 ||
         byte == phext::SHELF_BREAK_U8 ||
         byte == phext::LIBRARY_BREAK_U8;
}

fn apply_break(location: &mut Coordinate, byte: u8) {
  if byte == phext::SCROLL_BREAK_U8     { location.scroll_break();     }
  if byte == phext::SECTION_BREAK_U8    { location.section_break();    }
  if byte == phext::CHAPTER_BREAK_U8    { location.chapter_break();    }
  if byte == phext::BOOK_BREAK_U8       { location.book_break();       }
  if byte == phext::VOLUME_BREAK_U8     { location.volume_break();     }
  if byte == phext::COLLECTION_BREAK_U8 { location.collection_break(); }
  if byte == phext::SERIES_BREAK_U8     { location.series_break();     }
  if byte == phext::SHELF_BREAK_U8      { location.shelf_break();      }
  if byte == phext::LIBRARY_BREAK_U8    { location.library_break();    }
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn phokenize_bytes
///
/// the byte-level twin of `phext::phokenize`: walks a raw byte stream and
/// returns each scroll's coordinate and raw bytes, with identical
/// segmentation. Dimension-break bytes are ASCII, so byte offsets are exact;
/// scroll bodies are returned uninterpreted, valid UTF-8 or not.
/// ----------------------------------------------------------------------------------------------------------
pub fn phokenize_bytes(input: &[u8]) -> Vec<(Coordinate, Vec<u8>)> {
  let mut result: Vec<(Coordinate, Vec<u8>)> = Vec::new();
  let max = input.len();
  let mut coord = phext::default_coordinate();
  let mut pi: usize = 0;
  loop {
    let start = coord;
    let mut begin = start;
    let mut text_start: Option<usize> = None;
    let mut text_end = max;
    while pi < max {
      let byte = input[pi];
      if is_structural_break(byte) {
        apply_break(&mut coord, byte);
        if text_start.is_some() {
          text_end = pi;
          pi += 1;
          break;
        }
      } else if text_start.is_none() {
        text_start = Some(pi);
        begin = coord;
      }
      pi += 1;
    }
    let scroll = match text_start {
      Some(first) => input[first..text_end].to_vec(),
      None => Vec::new(),
    };
    result.push((begin, scroll));
    if pi >= max { break; }
  }

  return result;
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn ingest
///
/// ingests an externally supplied phext byte stream. Every scroll is
/// observed as raw bytes: valid UTF-8 scrolls pass through verbatim, and any
/// scroll whose bytes are not valid UTF-8 is stored as a binary scroll
/// holding the base64 of the exact observed bytes (the ingestion rule), so
/// `fetch_binary` at its coordinate returns the original bytes exactly.
/// Never rejects, never panics. The result is the canonical serialization
/// (the same normal form as `phext::implode` of `phext::explode`).
/// ----------------------------------------------------------------------------------------------------------
pub fn ingest(bytes: &[u8]) -> String {
  let parts = phokenize_bytes(bytes);
  let mut map: HashMap<Coordinate, String> = HashMap::new();
  for (coord, raw) in parts {
    let text = match String::from_utf8(raw) {
      Ok(text) => text,
      Err(error) => encode_scroll(&error.into_bytes()),
    };
    map.insert(coord, text);
  }
  return phext::implode(map);
}
