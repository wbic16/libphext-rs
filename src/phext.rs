/// ----------------------------------------------------------------------------------------------------------
/// Phext Buffer Library
/// ported from: https://github.com/wbic16/libphext/blob/main/phext.h
///
/// Copyright: (c) 2024-2025 Will Bickford (Phext, Inc.)
/// License: MIT
///
/// Overview
/// --------
/// Phext is composable, relational text. It is composed of layers upon layers of plain text (scrolls). All
/// text in a phext document is stored in subspace: a traditional buffer of utf8 text with a terminating
/// null byte. Subspace enables you to orient yourself within petabyte volumes of text with an age-old
/// mechanism: dead reckoning.
///
/// We've been using this process to keep track of columns and lines in plain text since the dawn of the
/// information age. Phext extends dead reckoning from 2D to 11D in a natural way. Please refer to this
/// article for details: https://phext.io/api.php?seed=raap&token=research&coordinate=1.1.1/1.1.1/1.1.1.
///
/// Coordinate Systems as Points
/// ----------------------------
/// To understand how truly huge a phext coordinate space is: try imagining a 3D coordinate system that
/// has been compressed fractally into a point. That's phext in a nutshell. We will make use of a series
/// of delimiters of unusual size (DOUS) to make sense of this address space.
///
/// Encoding
/// --------
/// Traditionally, text files have only contained one document type. This severely limits our ability
/// to represent arbitrary ideas and data. Phext splits file types to only have meaning within the context
/// of a scroll of text. This allows us to embed entire file systems and networks of knowledge within
/// one phext buffer. We achieve this by re-purposing historic ASCII control codes that have fallen out
/// of common use.
///
/// We've shortened these dimension names to two-letter acronyms in the table below to ensure it fits.
///
///   Dimension  Designation  Description
///   ---------  -----------  ----------- 
///   1          CL           Column
///   2          LN           Line
///   3          SC           Scroll
///   4          SN           Section
///   5          CH           Chapter
///   6          BK           Book
///   7          VM           Volume
///   8          CN           Collection
///   9          SR           Series
///   10         SF           Shelf
///   11         LB           Library
///
///   delimiter         value     CL   LN   SC   SN   CH   BK   VM   CN   SR   SF   LB
///   ---------         -----     --   --   --   --   --   --   --   --   --   --   --
///   character         implicit  +1
///   line break        0x0A      =1   +1
///   scroll break      0x17      =1   =1   +1
///   section break     0x18      =1   =1   =1   +1
///   chapter break     0x19      =1   =1   =1   =1   +1
///   book break        0x1A      =1   =1   =1   =1   =1   +1
///   volume break      0x1C      =1   =1   =1   =1   =1   =1   +1
///   collection break  0x1D      =1   =1   =1   =1   =1   =1   =1   +1
///   series break      0x1E      =1   =1   =1   =1   =1   =1   =1   =1   +1
///   shelf break       0x1F      =1   =1   =1   =1   =1   =1   =1   =1   =1   +1
///   library break     0x01      =1   =1   =1   =1   =1   =1   =1   =1   =1   =1   +1
///
/// Above the library break sits the stream level (0.6.0): whole phexts joined into chains.
///
///   chain break       0x02      separates whole phexts within a chain
///   chain close       0x03      terminates a chain; a chain ending without it is unclosed
///
/// History Fork
/// ------------
/// Phext files are a natural fork of plain text. They add hierarchy
/// via a system of increasingly-larger dimension breaks. These breaks start
/// with normal line breaks (2D) and proceed up to library breaks (11D).
///
/// We've annotated the ascii control codes from 0x01 to 0x1f below. Phext
/// has made an effort to remain compatible with a broad swath of software.
/// It is important to note, however, that phext is a fork in the road -
/// ASCII includes character codes that have fallen out of use. We've made them
/// useful again.
/// ----------------------------------------------------------------------------------------------------------

use std::collections::HashMap;

/// ----------------------------------------------------------------------------------------------------------
/// phext constants
/// ----------------------------------------------------------------------------------------------------------
pub const COORDINATE_MINIMUM: usize = 1;    // human numbering - we start at 1, not 0
// coordinate components are unbounded as of 0.4.0 (the 999 cap was removed; see the
// dense-graph warning on `Coordinate` below)
pub const LIBRARY_BREAK: char = '\x01';    // 11th dimension - replaces start of header
pub const CHAIN_BREAK: char = '\x02';      // stream level - separates whole phexts within a chain; replaces start of text
pub const CHAIN_CLOSE: char = '\x03';      // stream level - terminates a chain; replaces end of text
pub const MORE_COWBELL: char = '\x07';     // i've got a fever, and the only prescription...is more cowbell!
pub const LINE_BREAK: char = '\x0A';       // same as plain text \o/
pub const SCROLL_BREAK: char = '\x17';     // 3D Break - replaces End Transmission Block
pub const SECTION_BREAK: char = '\x18';    // 4D Break - replaces Cancel Block
pub const CHAPTER_BREAK: char = '\x19';    // 5D Break - replaces End of Tape
pub const BOOK_BREAK: char = '\x1A';       // 6D Break - replaces Substitute
pub const VOLUME_BREAK: char = '\x1C';     // 7D Break - replaces file separator
pub const COLLECTION_BREAK: char = '\x1D'; // 8D Break - replaces group separator
pub const SERIES_BREAK: char = '\x1E';     // 9D Break - replaces record separator
pub const SHELF_BREAK: char = '\x1F';      // 10D Break - replaces unit separator

// byte forms of the char constants above, for use in byte-oriented code.
// Aeneas cannot translate casts between char and int today, so byte code
// compares and pushes these twins instead of converting the char constants.
pub const LIBRARY_BREAK_U8: u8 = 0x01;    // byte form of LIBRARY_BREAK
pub const CHAIN_BREAK_U8: u8 = 0x02;      // byte form of CHAIN_BREAK
pub const CHAIN_CLOSE_U8: u8 = 0x03;      // byte form of CHAIN_CLOSE
pub const MORE_COWBELL_U8: u8 = 0x07;     // byte form of MORE_COWBELL
pub const LINE_BREAK_U8: u8 = 0x0A;       // byte form of LINE_BREAK
pub const SCROLL_BREAK_U8: u8 = 0x17;     // byte form of SCROLL_BREAK
pub const SECTION_BREAK_U8: u8 = 0x18;    // byte form of SECTION_BREAK
pub const CHAPTER_BREAK_U8: u8 = 0x19;    // byte form of CHAPTER_BREAK
pub const BOOK_BREAK_U8: u8 = 0x1A;       // byte form of BOOK_BREAK
pub const VOLUME_BREAK_U8: u8 = 0x1C;     // byte form of VOLUME_BREAK
pub const COLLECTION_BREAK_U8: u8 = 0x1D; // byte form of COLLECTION_BREAK
pub const SERIES_BREAK_U8: u8 = 0x1E;     // byte form of SERIES_BREAK
pub const SHELF_BREAK_U8: u8 = 0x1F;      // byte form of SHELF_BREAK

pub const ADDRESS_MICRO_BREAK: u8 = b'.'; // delimiter for micro-coordinates
pub const ADDRESS_MACRO_BREAK: u8 = b'/'; // delimiter for macro-coordinates
pub const ADDRESS_MACRO_ALT: u8 = b';';   // also allow ';' for url encoding

/// ----------------------------------------------------------------------------------------------------------
/// backwards compatibility
/// -----------------------
/// phext retains backwards compatibility for a wide portion of ascii and utf8 documents. below is a summary
/// of character codes that have been retained for future growth and/or backwards-compatibility.
/// ----------------------------------------------------------------------------------------------------------
/// claimed in 0.6.0
/// ----------------
/// * start of text = 0x02: the chain break (see CHAIN_BREAK)
/// * end of text = 0x03: the chain close (see CHAIN_CLOSE)
///
/// not widely used
/// ---------------
/// these are most useful for transmission protocols, but since http won that war, they have fallen out of
/// common use. we could evaluate some of the other characters that were kicked out above relative to these.
///
/// * end of transmission = 0x04
/// * enquery = 0x05
/// * ack = 0x06
/// * nak = 0x15
/// * syn = 0x16
/// * escape = 0x1b
///
/// actively used
/// -------------
/// * backspace = 0x08: most editors don't record backspaces, but this seems useful
/// * tab = 0x09: i had the opportunity to end the tabs vs spaces war, and chose peace
/// * vertical tab = 0x0b: seems useful
/// * form feed = 0x0c: dot matrix printers!?
/// * carriage return = 0x0d: needed for windows compatibility
/// * shift out = 0x0e: seems useful
/// * shift in = 0x0f: seems useful
/// * data link escape = 0x10: assuming printers need these
/// * device control 1 = 0x11: assuming printers need these
/// * device control 2 = 0x12: assuming printers need these
/// * device control 3 = 0x13: assuming printers need these
/// * device control 4 = 0x14: assuming printers need these
/// ----------------------------------------------------------------------------------------------------------

/// ----------------------------------------------------------------------------------------------------------
/// @struct ZCoordinate
///
/// The large-scale Z arm of a phext coordinate (see `Coordinate` below)
/// ----------------------------------------------------------------------------------------------------------
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Copy, Clone)]
pub struct ZCoordinate {
  pub library: usize,
  pub shelf: usize,
  pub series: usize
}
impl ZCoordinate {
  pub fn new() -> ZCoordinate {
    ZCoordinate { library: 1, shelf: 1, series: 1 }
  }
}
impl Default for ZCoordinate {
  fn default() -> ZCoordinate {
    ZCoordinate::new()
  }
}
impl std::fmt::Display for ZCoordinate {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}.{}.{}", self.library, self.shelf, self.series)
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// @struct YCoordinate
///
/// The large-scale Y arm of a phext coordinate (see `Coordinate` below)
/// ----------------------------------------------------------------------------------------------------------
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Copy, Clone)]
pub struct YCoordinate {
  pub collection: usize,
  pub volume: usize,
  pub book: usize
}
impl YCoordinate {
  pub fn new() -> YCoordinate {
    YCoordinate { collection: 1, volume: 1, book: 1 }
  }
}
impl Default for YCoordinate {
  fn default() -> YCoordinate {
    YCoordinate::new()
  }
}
impl std::fmt::Display for YCoordinate {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}.{}.{}", self.collection, self.volume, self.book)
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// @struct XCoordinate
///
/// The large-scale X arm of a phext coordinate (see `Coordinate` below)
/// ----------------------------------------------------------------------------------------------------------
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Copy, Clone)]
pub struct XCoordinate {
  pub chapter: usize,
  pub section: usize,
  pub scroll: usize
}
impl XCoordinate {
  pub fn new() -> XCoordinate {
    XCoordinate { chapter: 1, section: 1, scroll: 1 }
  }
}
impl Default for XCoordinate {
  fn default() -> XCoordinate {
    XCoordinate::new()
  }
}
impl std::fmt::Display for XCoordinate {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}.{}.{}", self.chapter, self.section, self.scroll)
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// @struct Coordinate
///
/// provides access to a default-initialized coordinate at 1.1.1/1.1.1/1.1.1
///
/// phext coordinates are formed along a 9-dimensional hierarchy with three main arms
/// of the form z3.z2.z1/y3.y2.y1.x3.x2.x1 where:
///
/// Z - this arm contains the library (z3), shelf (z2), and series (z1) dimensions
/// Y - this arm contains the collection (y3), volume (y2), and book (y1) dimensions
/// X - this arm contains the chapter (x3), section (x2), and scroll (x1) dimensions
///
/// Dense graphs: phext is sparse by design. Coordinate components are unbounded, but a
/// fully-populated dense graph contains the product of its dimensions' extents in
/// scrolls — a dense block spanning 1000 in each of 9 dimensions implies 10^27 scrolls.
/// Materializing, exploding, or indexing such a graph costs memory proportional to its
/// populated scroll count. The old 999 cap never protected against this (999^9 was
/// already unreachable); sparsity does. Keep graphs sparse.
/// ----------------------------------------------------------------------------------------------------------
#[derive(Default, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Copy, Clone)]
pub struct Coordinate {
  pub z: ZCoordinate,
  pub y: YCoordinate,
  pub x: XCoordinate,
}
impl Coordinate {
  pub fn new() -> Coordinate {
    Coordinate { z: ZCoordinate::new(), y: YCoordinate::new(), x: XCoordinate::new() }
  }
}
impl std::fmt::Display for Coordinate {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}/{}/{}", self.z, self.y, self.x)
  }
}

#[derive(Default, Debug, PartialEq, Eq, Hash, PartialOrd, Clone)]
pub struct PositionedScroll {
  pub coord: Coordinate,
  pub scroll: String
}
impl PositionedScroll {
  pub fn new() -> PositionedScroll {
    PositionedScroll { coord: to_coordinate("1.1.1/1.1.1/1.1.1"), scroll: "".to_string() }
  }
}
impl std::fmt::Display for PositionedScroll {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}: {}", self.coord.to_string(), &self.scroll[..4])
  }
}
impl Ord for PositionedScroll {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
      self.coord.cmp(&other.coord)
  }
}

#[derive(Default, Debug, PartialEq, PartialOrd, Copy, Clone)]
pub struct Range {
  pub start: Coordinate,
  pub end: Coordinate
}
impl Range {
  pub fn new() -> Range {
    Range { start: to_coordinate("1.1.1/1.1.1/1.1.1"), end: to_coordinate("1.1.1/1.1.1/1.1.1") }
  }
}
impl std::fmt::Display for Range {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}-{}", self.start, self.end)
  }
}

#[derive(Default, Debug, Clone)]
pub struct PhextParseError;
impl std::error::Error for PhextParseError {}

impl std::fmt::Display for PhextParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "Phext addresses are of the form LB.SF.SR/CN.VM.BK/CH.SN.SC")
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// split_on: manual equivalent of str::split(char), matching its semantics exactly (empty segments
/// and a trailing empty tail are preserved). Exists because str::split is Pattern-generic, and
/// core's Pattern trait carries a generic associated type that Aeneas cannot translate today.
/// The two passes keep borrows of the input out of the iterator loop: first collect the delimiter
/// indices (owned data), then slice between them in a separate loop for the translator.
/// The pieces are owned Strings because the translator cannot carry a vector of borrows of the
/// input built inside a function; the semantics are unchanged.
/// ----------------------------------------------------------------------------------------------------------
fn split_on(s: &str, delim: char) -> Vec<String> {
  let mut cuts: Vec<usize> = Vec::new();
  for (i, ch) in s.char_indices() {
    if ch == delim { cuts.push(i); }
  }
  let mut out: Vec<String> = Vec::new();
  let mut start: usize = 0;
  for cut in cuts {
    out.push(s[start..cut].to_string());
    start = cut + delim.len_utf8();
  }
  out.push(s[start..].to_string());
  out
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn std::convert::TryFrom
/// ----------------------------------------------------------------------------------------------------------
impl std::convert::TryFrom<&str> for Coordinate {
  type Error = PhextParseError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
      let parts: Vec<String> = split_on(value, '/');
      let error: PhextParseError = Default::default();
      if parts.len() != 3 {        
        return Err(error);
      }
      let z: Vec<String> = split_on(parts[0].as_str(), '.');
      let y: Vec<String> = split_on(parts[1].as_str(), '.');
      let x: Vec<String> = split_on(parts[2].as_str(), '.');
      if z.len() != 3 || y.len() != 3 || x.len() != 3 {
        return Err(error);
      }
      let mut result: Coordinate = Default::default();
      // A component that fails to parse (non-numeric, empty, negative, or
      // beyond usize) is a parse error, not a panic: return Err.
      match z[0].parse::<usize>() {
        Ok(v) => result.z.library = v,
        Err(_) => return Err(error),
      }
      match z[1].parse::<usize>() {
        Ok(v) => result.z.shelf = v,
        Err(_) => return Err(error),
      }
      match z[2].parse::<usize>() {
        Ok(v) => result.z.series = v,
        Err(_) => return Err(error),
      }
      match y[0].parse::<usize>() {
        Ok(v) => result.y.collection = v,
        Err(_) => return Err(error),
      }
      match y[1].parse::<usize>() {
        Ok(v) => result.y.volume = v,
        Err(_) => return Err(error),
      }
      match y[2].parse::<usize>() {
        Ok(v) => result.y.book = v,
        Err(_) => return Err(error),
      }
      match x[0].parse::<usize>() {
        Ok(v) => result.x.chapter = v,
        Err(_) => return Err(error),
      }
      match x[1].parse::<usize>() {
        Ok(v) => result.x.section = v,
        Err(_) => return Err(error),
      }
      match x[2].parse::<usize>() {
        Ok(v) => result.x.scroll = v,
        Err(_) => return Err(error),
      }
      Ok(result)
    }
}

pub fn check_for_cowbell(input: &str) -> bool {
  for byte in input.as_bytes() {
    if *byte == MORE_COWBELL_U8 {
      return true;
    }
  }

  false
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn get_subspace_coordinates
///
/// finds the start and end offsets for the given coordinate
///
/// Returns `(start, end, best)`:
/// - when the document reaches `target`, `start..end` is that scroll's text (empty if it has none) and
///   `best` is `target`;
/// - otherwise `start == end` is where the scroll would go: just before the break that steps past
///   `target`, or the end of the document. `best` is the coordinate in effect there, so writing breaks
///   from `best` up to `target` and then the text places it correctly.
///
/// The walker only moves on a dimension break, so it is compared with `target` only at the start and
/// after each break, and the scan stops as soon as it steps past `target`.
///
/// Earlier versions returned `start = 0` for a missing coordinate that falls between two scrolls, so
/// `fetch`, `replace`, and `remove` acted on the scroll before it, and they missed a scroll reached only
/// by breaks at the very end of the document.
/// ----------------------------------------------------------------------------------------------------------
pub fn get_subspace_coordinates(subspace: &[u8], target: Coordinate) -> (usize, usize, Coordinate) {
  let mut walker: Coordinate = default_coordinate();
  let mut best: Coordinate = default_coordinate();
  let mut found: Option<usize> = None;
  let max = subspace.len();
  let mut position: usize = 0;
  let mut moved = true;

  loop {
    // `walker` is the coordinate in effect at `position`
    if moved && found.is_none() {
      if walker == target {
        found = Some(position);
        best = walker;
      } else if walker < target {
        best = walker;
      } else {
        // before the first scroll: `target` sorts ahead of 1.1.1/1.1.1/1.1.1
        return (position, position, best);
      }
      moved = false;
    }
    if position == max {
      break;
    }

    let next = subspace[position];
    if is_dimension_break(next) {
      apply_break(&mut walker, next);
      moved = true;
      if walker > target {
        return match found {
          Some(start) => (start, position, best),
          None => (position, position, best),
        };
      }
      // since 0.4.0 a break always advances the walker, so a break after `found` steps past
      // `target` and returns above; this adjustment is retained defensively for any future
      // break type that might not move the walker (it would leave the walker on `target`,
      // and the scroll's text would start after the break)
      if found == Some(position) {
        found = Some(position + 1);
      }
    }
    position += 1;
  }

  match found {
    Some(start) => (start, max, best),
    None => (max, max, best),
  }
}

/// ----------------------------------------------------------------------------------------------------------
pub fn remove(input: &str, location: Coordinate) -> String {
  let phase1 = replace(input, location, "");
  normalize(phase1.as_str())
}

/// ----------------------------------------------------------------------------------------------------------
pub fn create_summary(input: &str) -> String {
  // scan bytes, not chars: every break is an ASCII byte below 0x20, and UTF-8
  // multi-byte sequences never contain bytes below 0x80, so the two-byte form
  // of U+010A cannot be mistaken for a line break. `end` lands either on a
  // break byte or at the full length, so it is always a character boundary.
  let bytes = input.as_bytes();
  let mut end: usize = 0;
  while end < bytes.len() {
    if is_phext_break(bytes[end]) { break; }
    end += 1;
  }
  let mut summary = input[..end].to_string();
  if end < bytes.len() {
    summary.push_str("...");
  }
  summary
}

/// ----------------------------------------------------------------------------------------------------------
pub fn navmap(urlbase: &str, input: &str) -> String {
  let phokens = phokenize(input);
  let mut result = String::new();
  let max = phokens.len();
  if max > 0 {
    result += "<ul>\n";
  }
  for phoken in phokens {
    result += &format!("<li><a href=\"{}{}\">{} {}</a></li>\n", urlbase, phoken.coord.to_urlencoded(), phoken.coord.to_string(), create_summary(&phoken.scroll)).to_string();
  }
  if max > 0 {
    result += "</ul>\n";
  }

  result
}

/// ----------------------------------------------------------------------------------------------------------
pub fn textmap(input: &str) -> String {
  let phokens = phokenize(input);
  let mut result = String::new();
  for phoken in phokens {
    result += &format!("* {}: {}\n", phoken.coord.to_string(), create_summary(&phoken.scroll)).to_string();
  }

  result
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn checksum
///
/// Provides a 128-bit content checksum using the fastest algorithm Rust provides: xxh3
/// ----------------------------------------------------------------------------------------------------------
pub fn checksum(input: &str) -> String {
  let buffer = input.as_bytes();
  let hash = xxhash_rust::xxh3::xxh3_128(buffer);
  format!("{:0>32}", format!("{:x}", hash))
}

/// ----------------------------------------------------------------------------------------------------------
pub fn manifest(input: &str) -> String {
  let mut phokens = phokenize(input);
  let mut i = 0;
  while i < phokens.len() {
    phokens[i].scroll = checksum(phokens[i].scroll.as_str());
    i += 1;
  }

  

  dephokenize(&mut phokens)
}

/// ----------------------------------------------------------------------------------------------------------
/// soundex_internal
/// inspired by https://sites.rootsweb.com/~nedodge/transfer/soundexlist.htm
/// ----------------------------------------------------------------------------------------------------------
fn soundex_internal(byte: String) -> String {
  let letter1 = "bpfv";
  let letter2 = "cskgjqxz";
  let letter3 = "dt";
  let letter4 = "l";
  let letter5 = "mn";
  let letter6 = "r";

  let mut value: usize = 1; // 1-100
  for c in byte.to_string().into_bytes() {
    // byte-slice contains: equivalent to str::contains(char) for these pure-ASCII literals, and avoids the Pattern trait
    if letter1.as_bytes().contains(&c) { value += 1; continue; }
    if letter2.as_bytes().contains(&c) { value += 2; continue; }
    if letter3.as_bytes().contains(&c) { value += 3; continue; }
    if letter4.as_bytes().contains(&c) { value += 4; continue; }
    if letter5.as_bytes().contains(&c) { value += 5; continue; }
    if letter6.as_bytes().contains(&c) { value += 6; continue; }
  }

  (value % 99).to_string()
}

/// ----------------------------------------------------------------------------------------------------------
pub fn soundex_v1(input: &str) -> String {
  let mut phokens = phokenize(input);
  
  for ith in &mut phokens {
    ith.scroll = soundex_internal(ith.scroll.clone());
  }

  dephokenize(&mut phokens)
}

/// ----------------------------------------------------------------------------------------------------------
fn index_phokens(input: &str) -> Vec<PositionedScroll> {
  let phokens = phokenize(input);
  let mut offset: usize = 0;
  let mut coord = default_coordinate();
  let mut output: Vec<PositionedScroll> = Vec::new();
  let mut i: usize = 0;
  while i < phokens.len() {
    let reference = phokens[i].coord;
    while coord.z.library < reference.z.library { coord.library_break(); offset += 1; }
    while coord.z.shelf < reference.z.shelf { coord.shelf_break(); offset += 1; }
    while coord.z.series < reference.z.series { coord.series_break(); offset += 1; }
    while coord.y.collection < reference.y.collection { coord.collection_break(); offset += 1; }
    while coord.y.volume < reference.y.volume { coord.volume_break(); offset += 1; }
    while coord.y.book < reference.y.book { coord.book_break(); offset += 1; }
    while coord.x.chapter < reference.x.chapter { coord.chapter_break(); offset += 1; }
    while coord.x.section < reference.x.section { coord.section_break(); offset += 1; }
    while coord.x.scroll < reference.x.scroll { coord.scroll_break(); offset += 1; }
    
    output.push(PositionedScroll { coord, scroll: format!("{offset}")});
    offset += phokens[i].scroll.len();
    i += 1;
  }

  output
}

/// ----------------------------------------------------------------------------------------------------------
pub fn index(input: &str) -> String {
  let mut output = index_phokens(input);

  dephokenize(&mut output)
}

/// ----------------------------------------------------------------------------------------------------------
///
/// The offset of the scroll at `coord`, or of the last scroll before it. 0 when no scroll comes at or before
/// `coord` (earlier versions panicked there).
pub fn offset(input: &str, coord: Coordinate) -> usize {
  let output = index_phokens(input);

  let mut result: usize = 0;
  for phoken in output {
    if phoken.coord > coord {
      break;
    }
    result = phoken.scroll.parse::<usize>().unwrap_or(result);
  }

  result
}

/// ----------------------------------------------------------------------------------------------------------
pub fn replace(input: &str, location: Coordinate, scroll: &str) -> String {
  let bytes = input.as_bytes();
  let parts = get_subspace_coordinates(bytes, location);
  let start: usize = parts.0;
  let mut end: usize = parts.1;
  let mut fixup: Vec<u8> = vec![];
  let mut subspace_coordinate: Coordinate = parts.2;

  while subspace_coordinate.z.library < location.z.library {
    fixup.push(LIBRARY_BREAK_U8);
    subspace_coordinate.library_break();
  }
  while subspace_coordinate.z.shelf < location.z.shelf {
    fixup.push(SHELF_BREAK_U8);
    subspace_coordinate.shelf_break();
  }
  while subspace_coordinate.z.series < location.z.series {
    fixup.push(SERIES_BREAK_U8);
    subspace_coordinate.series_break();
  }
  while subspace_coordinate.y.collection < location.y.collection {
    fixup.push(COLLECTION_BREAK_U8);
    subspace_coordinate.collection_break();
  }
  while subspace_coordinate.y.volume < location.y.volume {
    fixup.push(VOLUME_BREAK_U8);
    subspace_coordinate.volume_break();
  }
  while subspace_coordinate.y.book < location.y.book {
    fixup.push(BOOK_BREAK_U8);
    subspace_coordinate.book_break();
  }
  while subspace_coordinate.x.chapter < location.x.chapter {
    fixup.push(CHAPTER_BREAK_U8);
    subspace_coordinate.chapter_break();
  }
  while subspace_coordinate.x.section < location.x.section {
    fixup.push(SECTION_BREAK_U8);
    subspace_coordinate.section_break();
  }
  while subspace_coordinate.x.scroll < location.x.scroll {
    fixup.push(SCROLL_BREAK_U8);
    subspace_coordinate.scroll_break();
  }

  let text: std::slice::Iter<u8> = scroll.as_bytes().iter();
  let max = bytes.len();
  if end > max { end = max; }
  let left = &bytes[..start];
  let right = &bytes[end..];
  let mut temp: Vec<u8> = Vec::new();
  for b in left.iter() { temp.push(*b); }
  for b in fixup.iter() { temp.push(*b); }
  for b in text { temp.push(*b); }
  for b in right.iter() { temp.push(*b); }
  let result: String = String::from_utf8(temp).expect("invalid utf8");
  result
}

/// ----------------------------------------------------------------------------------------------------------
/// errata: libphext-py opted for ranges that operate on a stack of positioned scrolls
/// this method applies the range edit to subspace directly...we probably need both
pub fn range_replace(input: &str, location: Range, scroll: &str) -> String {
  let bytes = input.as_bytes();
  let parts_start = get_subspace_coordinates(bytes, location.start);
  let parts_end = get_subspace_coordinates(bytes, location.end);
  let start: usize = parts_start.0;
  let mut end: usize = parts_end.1;

  let text: std::slice::Iter<u8> = scroll.as_bytes().iter();
  let max = bytes.len();
  if end > max { end = max; }
  let left = &bytes[..start];
  let right = &bytes[end..];
  let mut temp: Vec<u8> = Vec::new();
  for b in left.iter() { temp.push(*b); }
  for b in text { temp.push(*b); }
  for b in right.iter() { temp.push(*b); }
  let result: String = String::from_utf8(temp).expect("invalid utf8");
  result
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn insert
///
/// inserts the content specified in `scroll` at the coordinate within `phext` specified by `location`
/// ----------------------------------------------------------------------------------------------------------
pub fn insert(input: String, location: Coordinate, scroll: &str) -> String {
  let bytes: &[u8] = input.as_bytes();
  let parts: (usize, usize, Coordinate) = get_subspace_coordinates(bytes, location);
  let end: usize = parts.1;
  let mut fixup: Vec<u8> = vec![];
  let mut subspace_coordinate: Coordinate = parts.2;

  while subspace_coordinate.z.library < location.z.library {
    fixup.push(LIBRARY_BREAK_U8);
    subspace_coordinate.library_break();
  }
  while subspace_coordinate.z.shelf < location.z.shelf {
    fixup.push(SHELF_BREAK_U8);
    subspace_coordinate.shelf_break();
  }
  while subspace_coordinate.z.series < location.z.series {
    fixup.push(SERIES_BREAK_U8);
    subspace_coordinate.series_break();
  }
  while subspace_coordinate.y.collection < location.y.collection {
    fixup.push(COLLECTION_BREAK_U8);
    subspace_coordinate.collection_break();
  }
  while subspace_coordinate.y.volume < location.y.volume {
    fixup.push(VOLUME_BREAK_U8);
    subspace_coordinate.volume_break();
  }
  while subspace_coordinate.y.book < location.y.book {
    fixup.push(BOOK_BREAK_U8);
    subspace_coordinate.book_break();
  }
  while subspace_coordinate.x.chapter < location.x.chapter {
    fixup.push(CHAPTER_BREAK_U8);
    subspace_coordinate.chapter_break();
  }
  while subspace_coordinate.x.section < location.x.section {
    fixup.push(SECTION_BREAK_U8);
    subspace_coordinate.section_break();
  }
  while subspace_coordinate.x.scroll < location.x.scroll {
    fixup.push(SCROLL_BREAK_U8);
    subspace_coordinate.scroll_break();
  }

  let text: std::slice::Iter<u8> = scroll.as_bytes().iter();
  let left = &bytes[..end];
  let right = &bytes[end..];
  let mut temp = Vec::with_capacity(left.len() + fixup.len() + text.len() + right.len());
  temp.extend_from_slice(left);
  temp.extend_from_slice(fixup.as_slice());
  temp.extend_from_slice(text.as_slice());
  temp.extend_from_slice(right);
  let result: String = String::from_utf8(temp).expect("invalid utf8");

  result
}


/// ----------------------------------------------------------------------------------------------------------
/// @fn next_scroll
///
/// retrieves the next scroll from the given string, assuming an arbitrary starting point
/// ----------------------------------------------------------------------------------------------------------
pub fn next_scroll(input: &str, start: Coordinate) -> (PositionedScroll, Coordinate, String) {
  let mut location = start;
  let p = input.as_bytes();
  let mut output: Vec<u8> = vec![];
  let mut remaining: Vec<u8> = vec![];
  let mut pi: usize = 0;
  let mut begin: Coordinate = start;
  let pmax = p.len();
  while pi < pmax
  {
    let mut dimension_break: bool = false;
    if p[pi] == SCROLL_BREAK_U8     { location.scroll_break();     dimension_break = true; }
    if p[pi] == SECTION_BREAK_U8    { location.section_break();    dimension_break = true; }
    if p[pi] == CHAPTER_BREAK_U8    { location.chapter_break();    dimension_break = true; }
    if p[pi] == BOOK_BREAK_U8       { location.book_break();       dimension_break = true; }
    if p[pi] == VOLUME_BREAK_U8     { location.volume_break();     dimension_break = true; }
    if p[pi] == COLLECTION_BREAK_U8 { location.collection_break(); dimension_break = true; }
    if p[pi] == SERIES_BREAK_U8     { location.series_break();     dimension_break = true; }
    if p[pi] == SHELF_BREAK_U8      { location.shelf_break();      dimension_break = true; }
    if p[pi] == LIBRARY_BREAK_U8    { location.library_break();    dimension_break = true; }

    if dimension_break {
      if !output.is_empty() {
        pi += 1;
        break;
      }
    } else {      
      begin = location;
      output.push(p[pi]);
    }
    pi += 1;
  }

  while pi < p.len()
  {
    remaining.push(p[pi]);
    pi += 1;
  }

  let out_scroll: PositionedScroll = PositionedScroll{coord: begin, scroll: String::from_utf8(output).expect("valid UTF-8")};
  (out_scroll, location, String::from_utf8(remaining).expect("valid UTF-8"))
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn is_dimension_break
///
/// true for the nine delimiters that move to a new scroll (line breaks stay inside a scroll)
/// ----------------------------------------------------------------------------------------------------------
fn is_dimension_break(byte: u8) -> bool {
  byte == SCROLL_BREAK_U8 ||
         byte == SECTION_BREAK_U8 ||
         byte == CHAPTER_BREAK_U8 ||
         byte == BOOK_BREAK_U8 ||
         byte == VOLUME_BREAK_U8 ||
         byte == COLLECTION_BREAK_U8 ||
         byte == SERIES_BREAK_U8 ||
         byte == SHELF_BREAK_U8 ||
         byte == LIBRARY_BREAK_U8
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn apply_break
///
/// advances `location` across one dimension break
/// ----------------------------------------------------------------------------------------------------------
fn apply_break(location: &mut Coordinate, byte: u8) {
  if byte == SCROLL_BREAK_U8     { location.scroll_break();     }
  if byte == SECTION_BREAK_U8    { location.section_break();    }
  if byte == CHAPTER_BREAK_U8    { location.chapter_break();    }
  if byte == BOOK_BREAK_U8       { location.book_break();       }
  if byte == VOLUME_BREAK_U8     { location.volume_break();     }
  if byte == COLLECTION_BREAK_U8 { location.collection_break(); }
  if byte == SERIES_BREAK_U8     { location.series_break();     }
  if byte == SHELF_BREAK_U8      { location.shelf_break();      }
  if byte == LIBRARY_BREAK_U8    { location.library_break();    }
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn phokenize
///
/// Transforms a packed phext buffer into a phext token (phoken) stream
///
/// One pass over the bytes. Each step matches a call to `next_scroll`: skip leading breaks, take a run
/// of text, then consume the break that ends it. Earlier versions called `next_scroll` in a loop, which
/// copied the rest of the document for every scroll and made this quadratic in the document size.
/// ----------------------------------------------------------------------------------------------------------
pub fn phokenize(input: &str) -> Vec<PositionedScroll> {
  let mut result: Vec<PositionedScroll> = Vec::new();
  let bytes = input.as_bytes();
  let max = bytes.len();
  let mut coord = default_coordinate();
  let mut pi: usize = 0;
  loop {
    let start = coord;
    let mut begin = start;
    let mut text_start: Option<usize> = None;
    let mut text_end = max;
    while pi < max {
      let byte = bytes[pi];
      if is_dimension_break(byte) {
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
      // breaks are ASCII, so these offsets always fall on character boundaries
      Some(first) => input[first..text_end].to_string(),
      None => String::new(),
    };
    result.push(PositionedScroll{coord: begin, scroll});
    if pi >= max { break; }
  }

  result
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn explode
///
/// Explodes an input phext into a hierarchical hashap, suitable for fast access and updates
/// ----------------------------------------------------------------------------------------------------------
pub fn explode(input: &str) -> HashMap<Coordinate, String> {
  let parts = phokenize(input);
  let mut hash = HashMap::new();
  for row in parts {
    // A scroll exists iff it holds at least one byte — that is the phext
    // definition. Delimiter runs between distant scrolls tokenize as empty
    // positioned slots; they are addressing, not content, and never
    // materialize here. (phokenize still reports them: it is the
    // tokenizer. Existence semantics live in explode.)
    if !row.scroll.is_empty() {
      hash.insert(row.coord, row.scroll);
    }
  }
  hash
}

/// ----------------------------------------------------------------------------------------------------------
/// @struct ScrollSpan
///
/// The borrowed form of a `PositionedScroll`: a scroll's coordinate plus the byte range of its
/// text within the input document. Slicing the input with `range` yields exactly the scroll's
/// text — building the index allocates no `String` per scroll.
/// ----------------------------------------------------------------------------------------------------------
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ScrollSpan {
  pub coord: Coordinate,
  pub range: std::ops::Range<usize>
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn spans
///
/// Builds a borrowed span index over `input`: one `ScrollSpan` per scroll, in document order,
/// giving each scroll's coordinate and the byte range of its text in `input`.
///
/// The walk is the `phokenize` walk — the same dimension breaks and the same coordinate
/// arithmetic — except that it records byte ranges instead of allocating a `String` for every
/// scroll. For every returned span, `&input[span.range]` is exactly the scroll `explode`
/// returns at `span.coord`, and `span.range.start` is the offset `offset` reports for that
/// coordinate.
///
/// Empty-scroll contract: a scroll exists iff it holds at least one byte — the definition
/// `explode` encodes — so `spans` reports only non-empty scrolls. Delimiter runs that encode
/// empty scrolls (leading breaks, runs between distant scrolls, trailing breaks) produce no
/// span; an empty document, or a document of only delimiters, produces no spans. `phokenize`
/// remains the tokenizer of record for empty slots: it additionally reports a final empty
/// positioned scroll in those trailing cases, where `spans` reports none.
///
/// Chain bytes (0x02 / 0x03) are not dimension breaks, so — exactly as in `phokenize` and
/// `explode` — they are scroll content here. This index covers a single phext (one universe);
/// chain structure is the `parse_chains` layer above it.
///
/// Motivating consumer: holo-phext, whose `src/index.rs` carries a 323-line re-implementation
/// of this walk, plus a duplicated private dimension-break predicate in its core, because
/// libphext exported only owned-`String` tokenization (`phokenize`/`explode`) and a
/// single-scroll `offset`. Downstream stores should index with `spans` instead of re-walking
/// the format.
///
/// # Example
///
/// ```
/// use libphext::phext::{spans, to_coordinate};
/// let input = "alpha\u{17}beta";
/// let index = spans(input);
/// assert_eq!(index.len(), 2);
/// assert_eq!(index[0].coord, to_coordinate("1.1.1/1.1.1/1.1.1"));
/// assert_eq!(&input[index[0].range.clone()], "alpha");
/// assert_eq!(&input[index[1].range.clone()], "beta");
/// ```
/// ----------------------------------------------------------------------------------------------------------
pub fn spans(input: &str) -> Vec<ScrollSpan> {
  let mut result: Vec<ScrollSpan> = Vec::new();
  let bytes = input.as_bytes();
  let max = bytes.len();
  let mut coord = default_coordinate();
  let mut pi: usize = 0;
  loop {
    let mut begin = coord;
    let mut text_start: Option<usize> = None;
    let mut text_end = max;
    while pi < max {
      let byte = bytes[pi];
      if is_dimension_break(byte) {
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
    // breaks are ASCII, so these offsets always fall on character boundaries
    if let Some(first) = text_start {
      result.push(ScrollSpan{coord: begin, range: first..text_end});
    }
    if pi >= max { break; }
  }

  result
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn implode
///
/// Serializes a hierarchical hash of scrolls into a monolithic phext document.
/// ----------------------------------------------------------------------------------------------------------
pub fn implode(map: HashMap::<Coordinate, String>) -> String {
  let mut vec: Vec<PositionedScroll> = Vec::new();
  for (key, value) in map.into_iter() {
    let ps = PositionedScroll{coord: key, scroll: value};
    vec.push(ps);
  }
  vec.sort();
  dephokenize(&mut vec)
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn merge
///
/// combines `left` and `right` into a new phext document, with content merged on a per-scroll basis
/// ----------------------------------------------------------------------------------------------------------
pub fn merge(left: &str, right: &str) -> String {
  let tl = phokenize(left);
  let tr = phokenize(right);
  let mut tli = 0;
  let mut tri = 0;
  let maxtl = tl.len();
  let maxtr = tr.len();
  let mut result: String = Default::default();
  let mut coord = default_coordinate();

  loop {
    let have_left = tli < maxtl;
    let have_right = tri < maxtr;
    
    let pick_left = have_left && (!have_right || tl[tli].coord <= tr[tri].coord);
    let pick_right = have_right && (!have_left || tr[tri].coord <= tl[tli].coord);

    if pick_left {
      append_scroll(&mut result, &tl[tli], coord);
      coord = tl[tli].coord;
      tli += 1;
    }
    if pick_right {
      append_scroll(&mut result, &tr[tri], coord);
      coord = tr[tri].coord;
      tri += 1;
    }

    if !pick_left && !pick_right {
      break;
    }
  }

  result
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn fetch
///
/// retrieves the scroll located at the given coordinates.
///
/// @param phext  the raw phext buffer to search
/// @param coord  coordinate to select the scroll from
/// ----------------------------------------------------------------------------------------------------------
pub fn fetch(input: &str, target: Coordinate) -> String {
  let bytes: &[u8] = input.as_bytes();
  let parts = get_subspace_coordinates(bytes, target);

  let start = parts.0 as usize;
  let end = parts.1 as usize;

  if end > start
  {
    let glyphs: usize = end - start;
    let mut temp: Vec<u8> = Vec::new();
    let stop = if start + glyphs < bytes.len() { start + glyphs } else { bytes.len() };
    let mut idx = start;
    while idx < stop {
      temp.push(bytes[idx]);
      idx += 1;
    }
    let result: String = String::from_utf8(temp).expect("invalid utf8");
    return result;
  }

  "".to_owned()
}

/// ----------------------------------------------------------------------------------------------------------
pub fn expand(input: &str) -> String {
  let mut buffer: Vec<u8> = input.to_string().into_bytes();
  let max = buffer.len();
  let mut p: usize = 0;
  loop {
    if p == max { break; }
    if buffer[p] == LINE_BREAK_U8 {
      buffer[p] = SCROLL_BREAK_U8;
    } else if buffer[p] == SCROLL_BREAK_U8 {
      buffer[p] = SECTION_BREAK_U8;
    } else if buffer[p] == SECTION_BREAK_U8 {
      buffer[p] = CHAPTER_BREAK_U8;
    } else if buffer[p] == CHAPTER_BREAK_U8 {
      buffer[p] = BOOK_BREAK_U8;
    } else if buffer[p] == BOOK_BREAK_U8 {
      buffer[p] = VOLUME_BREAK_U8;
    } else if buffer[p] == VOLUME_BREAK_U8{
      buffer[p] = COLLECTION_BREAK_U8;
    } else if buffer[p] == COLLECTION_BREAK_U8 {
      buffer[p] = SERIES_BREAK_U8;
    } else if buffer[p] == SERIES_BREAK_U8 {
      buffer[p] = SHELF_BREAK_U8;
    } else if buffer[p] == SHELF_BREAK_U8 {
      buffer[p] = LIBRARY_BREAK_U8;
    }
    p += 1;
  }

  let result: String = String::from_utf8(buffer).expect("invalid utf8");
  result
}

/// ----------------------------------------------------------------------------------------------------------
pub fn contract(input: &str) -> String {
  let mut buffer: Vec<u8> = input.to_string().into_bytes();
  let max = buffer.len();
  let mut p: usize = 0;
  loop {
    if p == max { break; }
    
    if buffer[p] == LIBRARY_BREAK_U8 {
      buffer[p] = SHELF_BREAK_U8;
    } else if buffer[p] == SHELF_BREAK_U8 {
      buffer[p] = SERIES_BREAK_U8;
    } else if buffer[p] == SERIES_BREAK_U8 {
      buffer[p] = COLLECTION_BREAK_U8;
    } else if buffer[p] == COLLECTION_BREAK_U8{
      buffer[p] = VOLUME_BREAK_U8;
    } else if buffer[p] == VOLUME_BREAK_U8 {
      buffer[p] = BOOK_BREAK_U8;
    } else if buffer[p] == BOOK_BREAK_U8 {
      buffer[p] = CHAPTER_BREAK_U8;
    } else if buffer[p] == CHAPTER_BREAK_U8 {
      buffer[p] = SECTION_BREAK_U8;
    } else if buffer[p] == SECTION_BREAK_U8 {
      buffer[p] = SCROLL_BREAK_U8;
    } else if buffer[p] == SCROLL_BREAK_U8 {
      buffer[p] = LINE_BREAK_U8;
    }   
    p += 1;
  }

  let result: String = String::from_utf8(buffer).expect("invalid utf8");
  result
}

/// ----------------------------------------------------------------------------------------------------------
fn dephokenize(tokens: &mut Vec<PositionedScroll>) -> String {
  let mut result: String = Default::default();
  let mut coord = default_coordinate();
  for ps in tokens {
    if !ps.scroll.is_empty() {
      append_scroll(&mut result, ps, coord);
      // The tracked coordinate advances only when bytes were emitted.
      // Advancing it for a skipped empty scroll undercounts the breaks
      // the next scroll needs and misplaces every scroll after it —
      // the mirror image of the explode defect (empty scrolls do not
      // exist, so they cannot move the write position).
      coord = ps.coord;
    }
  }
  result
}

/// ----------------------------------------------------------------------------------------------------------
fn append_scroll(output: &mut String, token: &PositionedScroll, mut coord: Coordinate) {
  while coord < token.coord {
    if coord.z.library < token.coord.z.library       { output.push(LIBRARY_BREAK);    coord.library_break();    continue; }
    if coord.z.shelf < token.coord.z.shelf           { output.push(SHELF_BREAK);      coord.shelf_break();      continue; }
    if coord.z.series < token.coord.z.series         { output.push(SERIES_BREAK);     coord.series_break();     continue; }
    if coord.y.collection < token.coord.y.collection { output.push(COLLECTION_BREAK); coord.collection_break(); continue; }
    if coord.y.volume < token.coord.y.volume         { output.push(VOLUME_BREAK);     coord.volume_break();     continue; }
    if coord.y.book < token.coord.y.book             { output.push(BOOK_BREAK);       coord.book_break();       continue; }
    if coord.x.chapter < token.coord.x.chapter       { output.push(CHAPTER_BREAK);    coord.chapter_break();    continue; }
    if coord.x.section < token.coord.x.section       { output.push(SECTION_BREAK);    coord.section_break();    continue; }
    if coord.x.scroll < token.coord.x.scroll         { output.push(SCROLL_BREAK);     coord.scroll_break();     continue; }
  }
  output.push_str(&token.scroll);
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn subtract
///
/// Subtracts the scrolls from `left` where there is also content at those coordinates in `right`
///
/// Note: This makes it easy to generate a mask for a given phext archive.
/// ----------------------------------------------------------------------------------------------------------
pub fn subtract(left: &str, right: &str) -> String {
  let pl = phokenize(left);
  let pr = phokenize(right);
  let mut result: String = Default::default();
  let mut pri = 0;
  let max = pr.len();
  let mut coord = default_coordinate();
  for token in pl {
    if token.scroll.is_empty() {
      continue;
    }
    // both lists are in coordinate order: skip right-hand scrolls that come before this one
    while pri < max && pr[pri].coord < token.coord {
      pri += 1;
    }
    let masked = pri < max && pr[pri].coord == token.coord && !pr[pri].scroll.is_empty();
    if !masked {
      append_scroll(&mut result, &token, coord);
      coord = token.coord;
    }
  }

  result
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn is_phext_break
///
/// returns true if `byte` corresponds to one of our phext delimiters (line breaks included)
/// ----------------------------------------------------------------------------------------------------------
pub fn is_phext_break(byte: u8) -> bool {
  byte == LINE_BREAK_U8 ||
         byte == SCROLL_BREAK_U8 ||
         byte == SECTION_BREAK_U8 ||
         byte == CHAPTER_BREAK_U8 ||
         byte == BOOK_BREAK_U8 ||
         byte == VOLUME_BREAK_U8 ||
         byte == COLLECTION_BREAK_U8 ||
         byte == SERIES_BREAK_U8 ||
         byte == SHELF_BREAK_U8 ||
         byte == LIBRARY_BREAK_U8
}

/// ----------------------------------------------------------------------------------------------------------
pub fn normalize(input: &str) -> String {
  let mut arr = phokenize(input);
  dephokenize(&mut arr)
}

/// ----------------------------------------------------------------------------------------------------------
/// @struct Chain
///
/// A chain: an ordered series of complete phexts within one stream, joined by CHAIN_BREAK (0x02) and
/// optionally terminated by CHAIN_CLOSE (0x03). The chain layer is a stream-level layer, one rung
/// above the library break: the nine dimension breaks structure a single phext, while the two chain
/// bytes structure a stream of phexts. Each member is a whole phext, worked with the ordinary phext
/// functions in this module; read in succession, a chain's members are editions in sequence — the
/// chain is how a phext stream encodes time.
///
/// `closed` records how the chain ended: true when a CHAIN_CLOSE terminated it, false when the
/// stream simply ran out (EOF). A chain that ends at EOF is unclosed — possibly truncated — and is
/// never silently promoted to closed. A stream with no chain bytes at all is a single unclosed
/// chain holding one member phext: every pre-0.6 phext reads this way.
/// ----------------------------------------------------------------------------------------------------------
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Clone)]
pub struct Chain {
  pub members: Vec<String>,
  pub closed: bool
}
impl Chain {
  pub fn new() -> Chain {
    Chain { members: Vec::new(), closed: false }
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn serialize
  ///
  /// serializes this chain alone: its members joined by CHAIN_BREAK, with a trailing CHAIN_CLOSE
  /// when the chain is closed. See `serialize_chains` for whole streams.
  /// ------------------------------------------------------------------------------------------------------
  pub fn serialize(&self) -> String {
    let mut result = String::new();
    let mut first = true;
    for member in &self.members {
      if !first { result.push(CHAIN_BREAK); }
      first = false;
      result.push_str(member);
    }
    if self.closed { result.push(CHAIN_CLOSE); }
    result
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn parse_chains
///
/// splits a stream into its chains. CHAIN_BREAK separates the member phexts of a chain; CHAIN_CLOSE
/// terminates a chain, and whatever follows it opens the next one — so one stream may hold many
/// chains. The stream's final chain is closed only when the stream ends with CHAIN_CLOSE.
///
/// Splitting follows `split_on` semantics, so empty members and empty chains are preserved exactly:
/// parse and serialize are byte-exact inverses for every stream.
/// ----------------------------------------------------------------------------------------------------------
pub fn parse_chains(input: &str) -> Vec<Chain> {
  let ends_closed = input.ends_with(CHAIN_CLOSE);
  let mut segments = split_on(input, CHAIN_CLOSE);
  if ends_closed {
    // split_on preserves the empty tail after a trailing delimiter; that
    // tail is the end of the stream, not the start of another chain
    segments.pop();
  }
  let count = segments.len();
  let mut chains: Vec<Chain> = Vec::new();
  let mut i = 0;
  for segment in segments {
    let closed = ends_closed || i + 1 < count;
    chains.push(Chain { members: split_on(segment.as_str(), CHAIN_BREAK), closed });
    i += 1;
  }
  chains
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn serialize_chains
///
/// serializes a stream of chains back into stream bytes: the inverse of `parse_chains`.
/// ----------------------------------------------------------------------------------------------------------
pub fn serialize_chains(chains: &[Chain]) -> String {
  let mut result = String::new();
  for chain in chains {
    result.push_str(chain.serialize().as_str());
  }
  result
}

/// ----------------------------------------------------------------------------------------------------------
pub fn default_coordinate() -> Coordinate {
  
  Coordinate {
    z: ZCoordinate {
      library: 1,
      shelf: 1,
      series: 1
    },
    y: YCoordinate {
      collection: 1,
      volume: 1,
      book: 1
    },
    x: XCoordinate {
      chapter: 1,
      section: 1,
      scroll: 1
    }
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn to_coordinate
///
/// translates a phext string to a strongly-typed address
///
/// @param address  text to parse
/// ----------------------------------------------------------------------------------------------------------
pub fn to_coordinate(address: &str) -> Coordinate {
  let mut result: Coordinate = default_coordinate();

  let mut index: u8 = 0;
  let mut value:u64 = 0;
  let exp:u64 = 10;

  for next in address.as_bytes() {
    let byte = *next;

    if byte == ADDRESS_MICRO_BREAK || byte == ADDRESS_MACRO_BREAK || byte == ADDRESS_MACRO_ALT {            
      match index {
        1 => { result.z.library = value as usize; index += 1; },
        2 => { result.z.shelf = value as usize; index += 1; },
        3 => { result.z.series = value as usize; index += 1; },
        4 => { result.y.collection = value as usize; index += 1; },
        5 => { result.y.volume = value as usize; index += 1; },
        6 => { result.y.book = value as usize; index += 1; },
        7 => { result.x.chapter = value as usize; index += 1; },
        8 => { result.x.section = value as usize; index += 1; },
        _ => {}
      }
      value = 0;
    }

    if (0x30..=0x39).contains(&byte)
    {
      value = exp.saturating_mul(value).saturating_add((byte - 0x30) as u64);
      if index == 0 { index = 1; }
    }
  }

  if index > 0 {
    result.x.scroll = value as usize;
  }

  result
}

fn validate_dimension_index(index: usize) -> bool {
  index >= COORDINATE_MINIMUM
}

impl Coordinate {
  /// ----------------------------------------------------------------------------------------------------------
  /// @fn validate_coordinate
  ///
  /// determines if coord points to a valid phext address
  ///
  /// @param coord: the coordinate to reset
  /// ----------------------------------------------------------------------------------------------------------
  pub fn validate_coordinate(&self) -> bool {
    
    validate_dimension_index(self.z.library) &&
                   validate_dimension_index(self.z.shelf) &&
                   validate_dimension_index(self.z.series) &&
                   validate_dimension_index(self.y.collection) &&
                   validate_dimension_index(self.y.volume) &&
                   validate_dimension_index(self.y.book) &&
                   validate_dimension_index(self.x.chapter) &&
                   validate_dimension_index(self.x.section) &&
                   validate_dimension_index(self.x.scroll)
  }

  /// ----------------------------------------------------------------------------------------------------------
  /// @fn to_string
  ///
  /// produces a quoted string for the given phext address in canonical format (z3.z2.z1/y3.y2.y1/x3.x2.x1)
  ///
  /// @param coord  the coordinate to translate
  /// ----------------------------------------------------------------------------------------------------------
  pub fn to_string(&self) -> String {
    if !self.validate_coordinate() {
      return "".to_owned();
    }
    format!("{}.{}.{}/{}.{}.{}/{}.{}.{}",
      self.z.library, self.z.shelf, self.z.series,
      self.y.collection, self.y.volume, self.y.book,
      self.x.chapter, self.x.section, self.x.scroll)
  }

  pub fn to_urlencoded(&self) -> String {
    if !self.validate_coordinate() {
      return "".to_owned();
    }
    format!("{}.{}.{};{}.{}.{};{}.{}.{}",
      self.z.library, self.z.shelf, self.z.series,
      self.y.collection, self.y.volume, self.y.book,
      self.x.chapter, self.x.section, self.x.scroll)
  }

  /// ----------------------------------------------------------------------------------------------------------
  /// @fn advance_coordinate
  ///
  /// ----------------------------------------------------------------------------------------------------------
  fn advance_coordinate(index: usize) -> usize {
    index + 1// components are unbounded: advancing never saturates
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn scroll_break
  /// ------------------------------------------------------------------------------------------------------
  pub fn scroll_break(&mut self) {
    self.x.scroll = Self::advance_coordinate(self.x.scroll);
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn section_break
  /// ------------------------------------------------------------------------------------------------------
  pub fn section_break(&mut self) {
    self.x.section = Self::advance_coordinate(self.x.section);
    self.x.scroll = 1;
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn chapter_break
  /// ------------------------------------------------------------------------------------------------------
  pub fn chapter_break(&mut self) {
    self.x.chapter = Self::advance_coordinate(self.x.chapter);
    self.x.section = 1;
    self.x.scroll = 1;
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn book_break
  /// ------------------------------------------------------------------------------------------------------
  pub fn book_break(&mut self) {
    self.y.book = Self::advance_coordinate(self.y.book);
    self.x.chapter = 1;
    self.x.section = 1;
    self.x.scroll = 1;
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn volume_break
  /// ------------------------------------------------------------------------------------------------------
  pub fn volume_break(&mut self) {
    self.y.volume = Self::advance_coordinate(self.y.volume);
    self.y.book = 1;
    self.x.chapter = 1;
    self.x.section = 1;
    self.x.scroll = 1;
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn collection_break
  /// ------------------------------------------------------------------------------------------------------
  pub fn collection_break(&mut self) {
    self.y.collection = Self::advance_coordinate(self.y.collection);
    self.y.volume = 1;
    self.y.book = 1;
    self.x.chapter = 1;
    self.x.section = 1;
    self.x.scroll = 1;
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn series_break
  /// ------------------------------------------------------------------------------------------------------
  pub fn series_break(&mut self) {
    self.z.series = Self::advance_coordinate(self.z.series);
    self.y.collection = 1;
    self.y.volume = 1;
    self.y.book = 1;
    self.x.chapter = 1;
    self.x.section = 1;
    self.x.scroll = 1;
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn shelf_break
  /// ------------------------------------------------------------------------------------------------------
  pub fn shelf_break(&mut self) {
    self.z.shelf = Self::advance_coordinate(self.z.shelf);
    self.z.series = 1;
    self.y.collection = 1;
    self.y.volume = 1;
    self.y.book = 1;
    self.x.chapter = 1;
    self.x.section = 1;
    self.x.scroll = 1;
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn library_break
  /// ------------------------------------------------------------------------------------------------------
  pub fn library_break(&mut self) {
    self.z.library = Self::advance_coordinate(self.z.library);
    self.z.shelf = 1;
    self.z.series = 1;
    self.y.collection = 1;
    self.y.volume = 1;
    self.y.book = 1;
    self.x.chapter = 1;
    self.x.section = 1;
    self.x.scroll = 1;
  }
}
// -----------------------------------------------------------------------------------------------------------
// Multiverse addressing (Will, 2026-11-13): inside a chain (a multiverse), each member phext is a
// universe indexed from 1. "$N:<address>" names a scroll in universe N; the "1:" may be omitted,
// and a chain-free phext is a single universe at multiverse 1.
// -----------------------------------------------------------------------------------------------------------
pub fn parse_universe_address(address: &str) -> Option<(usize, Coordinate)> {
    let (universe, coord_part) = match address.strip_prefix('$') {
        Some(rest) => {
            let (num, coord) = rest.split_once(':')?;
            let n: usize = num.parse().ok()?;
            if n < 1 { return None; }
            (n, coord)
        }
        None => (1, address),
    };
    Some((universe, to_coordinate(coord_part)))
}

impl Chain {
    /// Fetch a scroll by multiverse address: "$N:coord", or a bare coordinate (universe 1).
    /// Out-of-range universes and missing scrolls yield an empty string, as fetch does.
    pub fn fetch(&self, address: &str) -> String {
        match parse_universe_address(address) {
            Some((universe, coord)) => match self.members.get(universe - 1) {
                Some(member) => fetch(member, coord),
                None => String::new(),
            },
            None => String::new(),
        }
    }
}

// -----------------------------------------------------------------------------------------------------------
// Line/column addressing (v0.6.1, Will 2026-11-15; range syntax ratified 2026-11-15 —
// RANGE-SYNTAX-PROPOSAL.md in the ports program workspace).
//
// A point address is "[$N:]<9D coordinate>@<line>,<column>": the multiverse prefix and the
// coordinate exactly as in parse_universe_address, plus a within-scroll caret position.
// Line and column are 1-based; a position names a caret boundary (column 1 is the boundary
// before the line's first character; a line of n characters has valid columns 1..n+1), and
// columns count Unicode scalar values. A point may carry "+<offsetChars>": select forward
// that many characters (scalars) from the point, counted within the scroll, terminating at
// the scroll boundary.
//
// A range address is two point addresses joined by one hyphen: "<point>-<point>". The
// endpoints may span scrolls and universes; an endpoint without "@" names its whole scroll
// (as start: its first boundary; as end: its last). Canonical form: endpoints in ascending
// (universe, coordinate, line, column) order — direction is discarded — "$1:" never written,
// integers in minimal decimal form. Out-of-range endpoints fail at resolution, never clamp.
// -----------------------------------------------------------------------------------------------------------

/// @struct ScrollPosition
///
/// A caret boundary inside one scroll: 1-based line and column (see the section note above).
#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct ScrollPosition {
  pub line: usize,
  pub column: usize
}

/// @struct PointAddress
///
/// One parsed point address: universe (1-based; 1 when the "$N:" prefix is absent), the 9D
/// coordinate, an optional caret position, and an optional forward offset in characters.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PointAddress {
  pub universe: usize,
  pub coordinate: Coordinate,
  pub position: Option<ScrollPosition>,
  pub offset: Option<usize>
}
impl std::fmt::Display for PointAddress {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    if self.universe > 1 { write!(f, "${}:", self.universe)?; }
    write!(f, "{}", self.coordinate)?;
    if let Some(pos) = self.position {
      write!(f, "@{},{}", pos.line, pos.column)?;
      if let Some(offset) = self.offset {
        write!(f, "+{}", offset)?;
      }
    }
    Ok(())
  }
}

/// @struct RangeAddress
///
/// A parsed range: two point endpoints, normalized to ascending order at parse time.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RangeAddress {
  pub start: PointAddress,
  pub end: PointAddress
}
impl std::fmt::Display for RangeAddress {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{}-{}", self.start, self.end)
  }
}

fn parse_address_uint(text: &str) -> Option<usize> {
  if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) { return None; }
  text.parse::<usize>().ok()
}

fn parse_address_coordinate(text: &str) -> Option<Coordinate> {
  let arms: Vec<&str> = text.split('/').collect();
  if arms.len() != 3 { return None; }
  let mut values: Vec<usize> = Vec::new();
  for arm in arms {
    let parts: Vec<&str> = arm.split('.').collect();
    if parts.len() != 3 { return None; }
    for part in parts {
      let value = parse_address_uint(part)?;
      if value < 1 { return None; }
      values.push(value);
    }
  }
  let canonical = format!("{}.{}.{}/{}.{}.{}/{}.{}.{}",
    values[0], values[1], values[2], values[3], values[4], values[5], values[6], values[7], values[8]);
  Some(to_coordinate(&canonical))
}

/// ------------------------------------------------------------------------------------------------------
/// @fn parse_point_address
///
/// parses a v0.6.1 point address: "[$N:]<coordinate>[@<line>,<column>[+<offsetChars>]]".
/// Returns None on any malformed input: universe 0, a coordinate component below 1, a line or
/// column below 1, a sign anywhere, or trailing structure. "$1:" is accepted on input; the
/// canonical Display form never emits it.
/// ------------------------------------------------------------------------------------------------------
pub fn parse_point_address(address: &str) -> Option<PointAddress> {
  let (universe, rest) = match address.strip_prefix('$') {
    Some(stripped) => {
      let (num, coord) = stripped.split_once(':')?;
      let n = parse_address_uint(num)?;
      if n < 1 { return None; }
      (n, coord)
    }
    None => (1, address),
  };
  let (coord_part, pos_part) = match rest.split_once('@') {
    Some((coord, pos)) => {
      if pos.contains('@') { return None; }
      (coord, Some(pos))
    }
    None => (rest, None),
  };
  let coordinate = parse_address_coordinate(coord_part)?;
  let mut position = None;
  let mut offset = None;
  if let Some(pos) = pos_part {
    let (line_part, col_part) = pos.split_once(',')?;
    let line = parse_address_uint(line_part)?;
    let (col_text, offset_value) = match col_part.split_once('+') {
      Some((col, off)) => (col, Some(parse_address_uint(off)?)),
      None => (col_part, None),
    };
    let column = parse_address_uint(col_text)?;
    if line < 1 || column < 1 { return None; }
    position = Some(ScrollPosition { line, column });
    offset = offset_value;
  }
  Some(PointAddress { universe, coordinate, position, offset })
}

fn endpoint_position_key(point: &PointAddress, is_start: bool) -> (usize, usize) {
  match point.position {
    Some(pos) => (pos.line, pos.column),
    None => if is_start { (1, 1) } else { (usize::MAX, usize::MAX) },
  }
}

fn endpoint_cmp(a: &PointAddress, a_is_start: bool, b: &PointAddress, b_is_start: bool) -> std::cmp::Ordering {
  use std::cmp::Ordering;
  if a.universe != b.universe { return a.universe.cmp(&b.universe); }
  let coord = a.coordinate.cmp(&b.coordinate);
  if coord != Ordering::Equal { return coord; }
  endpoint_position_key(a, a_is_start).cmp(&endpoint_position_key(b, b_is_start))
}

/// ------------------------------------------------------------------------------------------------------
/// @fn parse_range_address
///
/// parses a v0.6.1 range address: exactly two point addresses joined by one hyphen. The
/// endpoints are normalized to ascending order (direction is discarded, per the ratified
/// spec). Range endpoints may not carry a "+offset" — the offset form never takes a second
/// endpoint. Returns None on malformed input.
/// ------------------------------------------------------------------------------------------------------
pub fn parse_range_address(address: &str) -> Option<RangeAddress> {
  let (first, second) = address.split_once('-')?;
  if second.contains('-') { return None; }
  let a = parse_point_address(first)?;
  let b = parse_point_address(second)?;
  if a.offset.is_some() || b.offset.is_some() { return None; }
  if endpoint_cmp(&a, true, &b, false) == std::cmp::Ordering::Greater {
    Some(RangeAddress { start: b, end: a })
  } else {
    Some(RangeAddress { start: a, end: b })
  }
}

/// Resolves a caret position inside one scroll to a character (scalar) offset.
/// None when the line or column is past the scroll's extent — never clamps.
fn resolve_scroll_offset(text: &str, pos: ScrollPosition) -> Option<usize> {
  if pos.line < 1 || pos.column < 1 { return None; }
  let mut line = 1usize;
  let mut column = 1usize;
  let mut offset = 0usize;
  for ch in text.chars() {
    if line == pos.line && column == pos.column { return Some(offset); }
    if ch == '\n' { line += 1; column = 1; } else { column += 1; }
    offset += 1;
  }
  if line == pos.line && column == pos.column { Some(offset) } else { None }
}

/// The scroll text at a coordinate in one member phext, in the phokenize sense of presence:
/// a coordinate is present exactly when phokenize enumerates it.
fn addressed_scroll_text(member: &str, target: Coordinate) -> Option<String> {
  phokenize(member).into_iter().find(|ps| ps.coord == target).map(|ps| ps.scroll)
}

impl Chain {
  /// ------------------------------------------------------------------------------------------------------
  /// @fn resolve_point
  ///
  /// resolves a point address against this chain: (universe, coordinate, character offset
  /// within the scroll). A point without "@" resolves to offset 0 of its scroll. None when
  /// the universe or scroll is absent or the position is out of range — never clamps.
  /// ------------------------------------------------------------------------------------------------------
  pub fn resolve_point(&self, point: &PointAddress) -> Option<(usize, Coordinate, usize)> {
    if point.universe < 1 || point.universe > self.members.len() { return None; }
    let text = addressed_scroll_text(&self.members[point.universe - 1], point.coordinate)?;
    let offset = match point.position {
      None => 0,
      Some(pos) => resolve_scroll_offset(&text, pos)?,
    };
    Some((point.universe, point.coordinate, offset))
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn fetch_point
  ///
  /// fetches a point address string. A bare point is a caret: it fetches the empty string.
  /// The offset form fetches the selected characters: from the point, forward `offset`
  /// characters (scalars) within the scroll, terminating at the scroll boundary. Any parse
  /// or resolution failure fetches the empty string, as Chain::fetch does.
  /// ------------------------------------------------------------------------------------------------------
  pub fn fetch_point(&self, address: &str) -> String {
    let point = match parse_point_address(address) { Some(p) => p, None => return String::new() };
    let count = match point.offset { Some(n) => n, None => return String::new() };
    let (_, _, offset) = match self.resolve_point(&point) { Some(v) => v, None => return String::new() };
    let text = addressed_scroll_text(&self.members[point.universe - 1], point.coordinate).unwrap_or_default();
    text.chars().skip(offset).take(count).collect()
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn resolve_range
  ///
  /// resolves a range against this chain into per-scroll segments: (universe, coordinate,
  /// segment text) in ascending order. Middle scrolls contribute their whole text; the
  /// endpoint scrolls contribute from/to their endpoint boundaries (an endpoint without
  /// "@" contributes its whole scroll). None when either endpoint fails to resolve —
  /// an out-of-range endpoint is an error, never a clamp.
  /// ------------------------------------------------------------------------------------------------------
  pub fn resolve_range(&self, range: &RangeAddress) -> Option<Vec<(usize, Coordinate, String)>> {
    let start = &range.start;
    let end = &range.end;
    if start.universe < 1 || end.universe > self.members.len() || start.universe > end.universe { return None; }
    // both endpoint scrolls must be present, and both endpoint positions must resolve
    self.resolve_point(&PointAddress { universe: start.universe, coordinate: start.coordinate, position: start.position, offset: None })?;
    self.resolve_point(&PointAddress { universe: end.universe, coordinate: end.coordinate, position: end.position, offset: None })?;
    let mut segments: Vec<(usize, Coordinate, String)> = Vec::new();
    for universe in start.universe..=end.universe {
      let member = &self.members[universe - 1];
      let mut scrolls = phokenize(member);
      scrolls.sort_by(|a, b| a.coord.cmp(&b.coord));
      for ps in &scrolls {
        let coord = ps.coord;
        if universe == start.universe && coord < start.coordinate { continue; }
        if universe == end.universe && coord > end.coordinate { continue; }
        let chars: Vec<char> = ps.scroll.chars().collect();
        let mut lo = 0usize;
        let mut hi = chars.len();
        if universe == start.universe && coord == start.coordinate {
          lo = match start.position { None => 0, Some(pos) => resolve_scroll_offset(&ps.scroll, pos)? };
        }
        if universe == end.universe && coord == end.coordinate {
          hi = match end.position { None => chars.len(), Some(pos) => resolve_scroll_offset(&ps.scroll, pos)? };
        }
        if lo > hi { return None; }
        segments.push((universe, coord, chars[lo..hi].iter().collect()));
      }
    }
    Some(segments)
  }

  /// ------------------------------------------------------------------------------------------------------
  /// @fn fetch_range
  ///
  /// fetches a range address string: the spanned text with phext structure bytes preserved —
  /// the covered segments re-assembled at their coordinates into a phext (chain) fragment
  /// (scroll breaks between covered scrolls inside a universe, CHAIN_BREAK between
  /// universes, never a trailing CHAIN_CLOSE: a fragment is an excerpt, not a terminated
  /// chain), so the result round-trips as a phext fragment. Empty segments contribute
  /// nothing. Any parse or resolution failure fetches the empty string, as Chain::fetch does.
  /// ------------------------------------------------------------------------------------------------------
  pub fn fetch_range(&self, address: &str) -> String {
    let range = match parse_range_address(address) { Some(r) => r, None => return String::new() };
    let segments = match self.resolve_range(&range) { Some(s) => s, None => return String::new() };
    let mut members: Vec<String> = Vec::new();
    for universe in range.start.universe..=range.end.universe {
      let mut member = String::new();
      for (_seg_universe, coord, text) in segments.iter().filter(|(u, _, _)| *u == universe) {
        if !text.is_empty() {
          member = insert(member, *coord, text);
        }
      }
      members.push(member);
    }
    Chain { members, closed: false }.serialize()
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// v0.6.2 — append, integrity, and commit primitives.
///
/// These exist because a phext *store* (see the Holostore re-implementation
/// in wbic16/holostore, PHEXT.md round 2) needs three things the document
/// API did not say: how to extend a document at its end without rebuilding
/// it, how to tell a complete scroll from a torn one, and how to tell a
/// complete batch from a torn one. Each primitive is small, uses no new
/// dependencies, and states its own limit.
/// ----------------------------------------------------------------------------------------------------------

/// The trailer line of a checked scroll: a newline, this marker, and the
/// xxh3-128 checksum (the existing `checksum`) of everything before it.
const CHECK_MARKER: &str = "\nphext-check:xxh3-128:";

/// ----------------------------------------------------------------------------------------------------------
/// @fn checked_scroll
///
/// Returns `scroll` with an integrity trailer appended: one final line
/// `phext-check:xxh3-128:<checksum of the payload>`. In a checked scroll
/// the final line is reserved — verification always reads the LAST marker
/// line, and the checksum binds the entire payload before it, so a marker
/// line inside the payload cannot forge a trailer.
///
/// The checksum detects accidental corruption and torn writes. xxh3 is
/// not a cryptographic hash: this is integrity, not authentication.
/// An empty scroll stays empty (a scroll with no bytes does not exist,
/// so there is nothing to check).
/// ----------------------------------------------------------------------------------------------------------
pub fn checked_scroll(scroll: &str) -> String {
  if scroll.is_empty() {
    return String::new();
  }
  format!("{}{}{}", scroll, CHECK_MARKER, checksum(scroll))
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn verify_checked_scroll
///
/// true when `scroll` ends with a well-formed integrity trailer whose
/// checksum matches the payload before it.
/// ----------------------------------------------------------------------------------------------------------
pub fn verify_checked_scroll(scroll: &str) -> bool {
  match scroll.rfind(CHECK_MARKER) {
    Some(at) => {
      let (payload, trailer) = scroll.split_at(at);
      let hex = &trailer[CHECK_MARKER.len()..];
      hex.len() == 32 && hex.bytes().all(|b| b.is_ascii_hexdigit()) && checksum(payload) == hex
    }
    None => false,
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn uncheck_scroll
///
/// Returns the payload of a checked scroll when its trailer verifies;
/// None for unchecked, malformed, or corrupted scrolls.
/// ----------------------------------------------------------------------------------------------------------
pub fn uncheck_scroll(scroll: &str) -> Option<String> {
  if verify_checked_scroll(scroll) {
    let at = scroll.rfind(CHECK_MARKER)?;
    Some(scroll[..at].to_string())
  } else {
    None
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn fetch_checked
///
/// Fetches the scroll at `target` and returns its payload, but only when
/// the scroll is checked and verifies. A missing, unchecked, or torn
/// scroll yields None — recovery code can treat None as "not committed".
/// ----------------------------------------------------------------------------------------------------------
pub fn fetch_checked(input: &str, target: Coordinate) -> Option<String> {
  uncheck_scroll(&fetch(input, target))
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn break_dimension
///
/// the dimension index of a dimension-break byte, lowest (scroll) first:
/// scroll 0, section 1, chapter 2, book 3, volume 4, collection 5,
/// series 6, shelf 7, library 8. None for any other byte.
/// ----------------------------------------------------------------------------------------------------------
fn break_dimension(byte: u8) -> Option<usize> {
  match byte {
    SCROLL_BREAK_U8 => Some(0),
    SECTION_BREAK_U8 => Some(1),
    CHAPTER_BREAK_U8 => Some(2),
    BOOK_BREAK_U8 => Some(3),
    VOLUME_BREAK_U8 => Some(4),
    COLLECTION_BREAK_U8 => Some(5),
    SERIES_BREAK_U8 => Some(6),
    SHELF_BREAK_U8 => Some(7),
    LIBRARY_BREAK_U8 => Some(8),
    _ => None,
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn break_candidates
///
/// SWAR scan of one 8-byte lane: a bitmask whose bit i is set when byte i
/// of the lane is below 0x20. Every dimension break is below 0x20, so a
/// lane with an empty mask holds no breaks and can be skipped whole. The
/// subtraction can borrow across lanes and flag a byte spuriously, but a
/// byte below 0x20 always flags its own lane — flagged bytes are checked
/// against the real break table, so a spurious flag costs a lookup,
/// never a missed break. (The bit-packing multiply is exact for all 256
/// combinations of lane flags.)
/// ----------------------------------------------------------------------------------------------------------
fn break_candidates(chunk: u64) -> u64 {
  let flagged = chunk.wrapping_sub(0x2020_2020_2020_2020) & !chunk & 0x8080_8080_8080_8080;
  ((flagged >> 7) & 0x0101_0101_0101_0101).wrapping_mul(0x0102_0408_1020_4080) >> 56
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn last_scroll_coordinate
///
/// the coordinate of the last non-empty scroll in `bytes` (a document
/// prefix with no trailing delimiter bytes: the last byte is content).
///
/// A scroll's coordinate is the fold of every dimension break before it,
/// and a break increments its own dimension while resetting every lower
/// dimension to 1 — so, read backwards, each component is 1 plus the
/// count of its own breaks after the last higher break. This scan walks
/// the bytes backwards with O(1) state and stops as soon as every
/// component is settled: a library break settles all lower dimensions at
/// once, leaving only a plain count of library breaks over the head.
/// The scan works 64 bytes at a time: `break_candidates` flags the bytes
/// that could be breaks with a few word-wide operations, and only flagged
/// bytes are visited individually — content bytes are never walked one
/// by one. Nothing is tokenized and nothing is allocated: the cost is
/// the delimiter scan itself, not a parse of the document.
///
/// This replaces the whole-document `phokenize` the 0.6.2 append paid on
/// every call, whose per-call cost grew linearly with document size and
/// made chained appends quadratic.
/// ----------------------------------------------------------------------------------------------------------
fn last_scroll_coordinate(bytes: &[u8]) -> Coordinate {
  // count[d]: breaks of dimension d in the suffix scanned so far
  let mut count = [0usize; 9];
  // resolved[d]: component value of dimension d, once a higher break
  // (or the start of the document) fixes it
  let mut resolved: [Option<usize>; 9] = [None; 9];
  let mut end = bytes.len();
  'scan: while end > 0 {
    let start = end.saturating_sub(64);
    let chunk = &bytes[start..end];
    // bit i of `candidates` marks chunk[i] as a possible break byte
    let mut candidates: u64 = 0;
    let mut lanes = chunk.chunks_exact(8);
    let mut base = 0;
    for lane in &mut lanes {
      let word = u64::from_le_bytes(lane.try_into().unwrap());
      candidates |= break_candidates(word) << base;
      base += 8;
    }
    for (i, byte) in lanes.remainder().iter().enumerate() {
      if *byte < 0x20 {
        candidates |= 1 << (base + i);
      }
    }
    // visit candidates from the last byte backwards
    while candidates != 0 {
      let bit = 63 - candidates.leading_zeros();
      candidates &= !(1 << bit);
      let j = start + bit as usize;
      if let Some(d) = break_dimension(bytes[j]) {
        for e in 0..d {
          if resolved[e].is_none() {
            resolved[e] = Some(1 + count[e]);
          }
        }
        count[d] += 1;
        if d == 8 {
          // every lower dimension is settled; the library component is
          // 1 plus the library breaks in the unscanned head — a plain
          // byte count, with no coordinate state left to track
          count[8] += bytes[..j].iter().filter(|b| **b == LIBRARY_BREAK_U8).count();
          resolved[8] = Some(1 + count[8]);
          break 'scan;
        }
      }
    }
    end = start;
  }
  let value = |d: usize| resolved[d].unwrap_or(1 + count[d]);
  Coordinate {
    z: ZCoordinate { library: value(8), shelf: value(7), series: value(6) },
    y: YCoordinate { collection: value(5), volume: value(4), book: value(3) },
    x: XCoordinate { chapter: value(2), section: value(1), scroll: value(0) },
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn append
///
/// Appends `scroll` at `location`, which must sort strictly after the
/// document's last non-empty scroll. The result is the input with only a
/// delimiter run and the new content added at its end — a host can write
/// just that suffix to a file opened in append mode and sync once.
///
/// Returns None when the scroll is empty (empty scrolls do not exist) or
/// `location` does not sort after the document's end. Trailing delimiter
/// bytes in the input encode only empty scrolls, so they are dropped
/// first and never shift the appended scroll's address.
///
/// On None the append was rejected; the input String was consumed, so
/// pass a clone if the caller needs it after a rejected append.
///
/// The document's end is located with `last_scroll_coordinate`, a
/// backward delimiter scan over the bytes — existing text is never
/// re-tokenized, so a call's cost tracks the delimiter run at the end
/// of the document, not the size of the document.
/// ----------------------------------------------------------------------------------------------------------
pub fn append(input: String, location: Coordinate, scroll: &str) -> Option<String> {
  if scroll.is_empty() {
    return None;
  }
  let keep = input
    .as_bytes()
    .iter()
    .rposition(|b| !is_dimension_break(*b))
    .map(|i| i + 1)
    .unwrap_or(0);
  let token = PositionedScroll { coord: location, scroll: scroll.to_string() };
  if keep == 0 {
    let mut out = String::new();
    append_scroll(&mut out, &token, default_coordinate());
    return Some(out);
  }
  let last = last_scroll_coordinate(&input.as_bytes()[..keep]);
  if location <= last {
    return None;
  }
  // the input is owned: drop its trailing delimiters in place and grow
  // it, rather than copying the kept prefix into a fresh string
  let mut out = input;
  out.truncate(keep);
  append_scroll(&mut out, &token, last);
  Some(out)
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn append_checked
///
/// `append` with an integrity trailer: the stored scroll is
/// `checked_scroll(scroll)`, verifiable later with `fetch_checked`.
/// ----------------------------------------------------------------------------------------------------------
pub fn append_checked(input: String, location: Coordinate, scroll: &str) -> Option<String> {
  append(input, location, &checked_scroll(scroll))
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn append_batch
///
/// Appends several scrolls in one call. Entries must be non-empty and in
/// strictly ascending coordinate order, the first sorting after the
/// document's end. The whole batch is a single suffix of the result, so
/// a host persists it with one write and one sync.
///
/// This is construction atomicity, not storage atomicity: whether the
/// suffix lands completely is the host filesystem's business (see the
/// durability profile in the crate docs). Pair the batch with a
/// `commit_scroll` as its final entry to make a torn batch *detectable*
/// at recovery — that is the part a format library can own.
/// ----------------------------------------------------------------------------------------------------------
pub fn append_batch(input: String, entries: &[(Coordinate, &str)]) -> Option<String> {
  if entries.is_empty() {
    return None;
  }
  for pair in entries.windows(2) {
    if pair[1].0 <= pair[0].0 {
      return None;
    }
  }
  let mut out = append(input, entries[0].0, entries[0].1)?;
  let mut coord = entries[0].0;
  for (location, scroll) in &entries[1..] {
    if scroll.is_empty() {
      return None;
    }
    append_scroll(&mut out, &PositionedScroll { coord: *location, scroll: scroll.to_string() }, coord);
    coord = *location;
  }
  Some(out)
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn commit_scroll
///
/// Builds the commit marker for a batch: a small scroll binding the
/// batch's entry count and a checksum over each entry's coordinate and
/// content, in order. Append it (via `append_batch`) as the batch's
/// final scroll, at a coordinate after every entry. `entries` here are
/// the (coordinate, content) pairs exactly as appended — checked
/// contents included, when the batch uses checked scrolls.
/// ----------------------------------------------------------------------------------------------------------
pub fn commit_scroll(entries: &[(Coordinate, &str)]) -> String {
  let mut material = String::new();
  for (coord, content) in entries {
    material.push_str(&coord.to_string());
    material.push('\n');
    material.push_str(content);
    material.push('\n');
  }
  format!("phext-commit:1\ncount={}\ncheck={}", entries.len(), checksum(&material))
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn verify_commit
///
/// true when the scroll at `commit` is a commit marker whose count
/// matches `entries` and whose checksum matches the entries' current
/// contents, fetched in order. Any missing, altered, or torn entry —
/// or a missing marker — yields false: the batch did not commit.
/// ----------------------------------------------------------------------------------------------------------
pub fn verify_commit(input: &str, entries: &[Coordinate], commit: Coordinate) -> bool {
  let text = fetch(input, commit);
  let mut lines = text.lines();
  if lines.next() != Some("phext-commit:1") {
    return false;
  }
  let mut count: Option<usize> = None;
  let mut check: Option<&str> = None;
  for line in lines {
    if let Some(v) = line.strip_prefix("count=") {
      count = v.parse().ok();
    }
    if let Some(v) = line.strip_prefix("check=") {
      check = Some(v);
    }
  }
  if count != Some(entries.len()) {
    return false;
  }
  let mut material = String::new();
  for coord in entries {
    let content = fetch(input, *coord);
    if content.is_empty() {
      return false;
    }
    material.push_str(&coord.to_string());
    material.push('\n');
    material.push_str(&content);
    material.push('\n');
  }
  check == Some(checksum(&material).as_str())
}
