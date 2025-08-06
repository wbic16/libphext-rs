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
/// History Fork
/// ------------
/// Phext files are a natural fork of plain text. They add hierarchy
/// via a system of increasingly-larger dimension breaks. These breaks start
/// with normal line breaks (2D) and proceed up to library breaks (11D).
///
/// We've annotated the ascii control codes from 0x01 to 0x1f below. Phext
/// has made an effort to remain compatible with a broad swath of software.
/// It is important to note, however, that phext is a fork in the road -
/// ASCII has character codes that have fallen out of use. We've made them
/// useful again.
/// ----------------------------------------------------------------------------------------------------------

use std::collections::HashMap;

/// ----------------------------------------------------------------------------------------------------------
/// phext constants
/// ----------------------------------------------------------------------------------------------------------
pub const COORDINATE_MINIMUM: usize = 1;    // human numbering - we start at 1, not 0
pub const COORDINATE_MAXIMUM: usize = 1000; // 2 KB pages x 100^9 = 2 million petabytes
pub const LIBRARY_BREAK: char = '\x01';    // 11th dimension - replaces start of header
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

pub const ADDRESS_MICRO_BREAK: u8 = b'.'; // delimiter for micro-coordinates
pub const ADDRESS_MACRO_BREAK: u8 = b'/'; // delimiter for macro-coordinates
pub const ADDRESS_MACRO_ALT: u8 = b';';   // also allow ';' for url encoding

/// ----------------------------------------------------------------------------------------------------------
/// backwards compatibility
/// -----------------------
/// phext retains backwards compatibility for a wide portion of ascii and utf8 documents. below is a summary
/// of character codes that have been retained for future growth and/or backwards-compatibility.
/// ----------------------------------------------------------------------------------------------------------
/// not widely used
/// ---------------
/// these are most useful for transmission protocols, but since http won that war, they have fallen out of
/// common use. we could evaluate some of the other characters that were kicked out above relative to these.
///
/// * start of text = 0x02
/// * end of text = 0x03
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
    return write!(f, "{}.{}.{}", self.library, self.shelf, self.series);
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
    return write!(f, "{}.{}.{}", self.collection, self.volume, self.book);
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
    return write!(f, "{}.{}.{}", self.chapter, self.section, self.scroll);
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
    return write!(f, "{}/{}/{}", self.z, self.y, self.x);
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
    return write!(f, "{}: {}", self.coord.to_string(), self.scroll[..4].to_string());
  }
}
impl Ord for PositionedScroll {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
      return self.coord.cmp(&other.coord);
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
    return write!(f, "{}-{}", self.start, self.end);
  }
}

#[derive(Default, Debug, Clone)]
pub struct PhextParseError;
impl std::error::Error for PhextParseError {}

impl std::fmt::Display for PhextParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    return write!(f, "Phext addresses are of the form LB.SF.SR/CN.VM.BK/CH.SN.SC");
  }
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn std::convert::TryFrom
/// ----------------------------------------------------------------------------------------------------------
impl std::convert::TryFrom<&str> for Coordinate {
  type Error = PhextParseError;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
      let parts: Vec<&str> = value.split('/').collect();
      let error: PhextParseError = Default::default();
      if parts.len() != 3 {        
        return Err(error);
      }
      let z: Vec<&str> = parts[0].split('.').collect();
      let y: Vec<&str> = parts[1].split('.').collect();
      let x: Vec<&str> = parts[2].split('.').collect();
      if z.len() != 3 || y.len() != 3 || x.len() != 3 {
        return Err(error);
      }
      let mut result: Coordinate = Default::default();
      result.z.library = z[0].parse::<usize>().expect("Library missing");
      result.z.shelf = z[1].parse::<usize>().expect("Shelf missing");
      result.z.series = z[2].parse::<usize>().expect("Series missing");
      result.y.collection = y[0].parse::<usize>().expect("Collection missing");
      result.y.volume = y[1].parse::<usize>().expect("Volume missing");
      result.y.book = y[2].parse::<usize>().expect("Book missing");
      result.x.chapter = x[0].parse::<usize>().expect("Chapter missing");
      result.x.section = x[1].parse::<usize>().expect("Section missing");
      result.x.scroll = x[2].parse::<usize>().expect("Scroll missing");
      return Ok(result);
    }
}

pub fn check_for_cowbell(phext: &str) -> bool {
  for byte in phext.as_bytes() {
    if *byte == MORE_COWBELL as u8 {
      return true;
    }
  }

  return false;
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn get_subspace_coordinates
///
/// finds the start and end offsets for the given coordinate
/// ----------------------------------------------------------------------------------------------------------
pub fn get_subspace_coordinates(subspace: &[u8], target: Coordinate) -> (usize, usize, Coordinate) {
  let mut walker: Coordinate = default_coordinate();
  let mut best: Coordinate = default_coordinate();
  let mut subspace_index: usize = 0;
  let mut start: usize = 0;
  let mut end: usize = 0;
  let mut stage = 0;
  let max = subspace.len();

  while subspace_index < max {
    let next = subspace[subspace_index];
    let compare = next as char;

    if stage == 0 {
      if walker == target {
        stage = 1;
        start = subspace_index;
        best = walker;
      }
      if walker < target {
        best = walker;
      }
    }

    if stage < 2 && walker > target {
      if stage == 0 {
        start = subspace_index - 1;
      }
      end = subspace_index - 1;
      stage = 2;
    }

    if is_phext_break(next) {
      if compare == SCROLL_BREAK     { walker.scroll_break();     }
      if compare == SECTION_BREAK    { walker.section_break();    }
      if compare == CHAPTER_BREAK    { walker.chapter_break();    }
      if compare == BOOK_BREAK       { walker.book_break();       }
      if compare == VOLUME_BREAK     { walker.volume_break();     }
      if compare == COLLECTION_BREAK { walker.collection_break(); }
      if compare == SERIES_BREAK     { walker.series_break();     }
      if compare == SHELF_BREAK      { walker.shelf_break();      }
      if compare == LIBRARY_BREAK    { walker.library_break();    }
    }

    if stage < 2 && walker > target {
      end = subspace_index;
      stage = 2;
    }

    subspace_index += 1;
  }

  if stage == 1 && walker == target {
    end = max;
    stage = 2;
  }

  if stage == 0 {
    start = max;
    end = max;
  }

  return (start, end, best);
}

/// ----------------------------------------------------------------------------------------------------------
pub fn remove(phext: &str, location: Coordinate) -> String {
  let phase1 = replace(phext, location, "");
  return normalize(phase1.as_str());
}

/// ----------------------------------------------------------------------------------------------------------
pub fn create_summary(phext: &str) -> String {
  let buffer = phext.as_bytes();
  let mut limit = 32;
  if buffer.len() < 32 { limit = buffer.len(); }
  let mut i = 0;
  let mut summary: Vec<u8> = Default::default();
  while i < limit {
    let ith = buffer[i];    
    if is_phext_break(ith) {
      break;
    }
    summary.push(ith);
    i += 1;
  }
  if summary.len() < buffer.len() {
    summary.push('.' as u8);
    summary.push('.' as u8);
    summary.push('.' as u8);
  }

  let result: String = String::from_utf8(summary).expect("invalid utf8");
  return result;
}

/// ----------------------------------------------------------------------------------------------------------
pub fn navmap(urlbase: &str, phext: &str) -> String {
  let phokens = phokenize(phext);
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

  return result;
}

/// ----------------------------------------------------------------------------------------------------------
pub fn textmap(phext: &str) -> String {
  let phokens = phokenize(phext);
  let mut result = String::new();
  for phoken in phokens {
    result += &format!("* {}: {}\n", phoken.coord.to_string(), create_summary(&phoken.scroll)).to_string();
  }

  return result;
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn checksum
///
/// Provides a 128-bit content checksum using the fastest algorithm Rust provides: xxh3
/// ----------------------------------------------------------------------------------------------------------
pub fn checksum(phext: &str) -> String {
  let buffer = phext.as_bytes();
  let hash = xxhash_rust::xxh3::xxh3_128(buffer);
  return format!("{:0>32}", format!("{:x}", hash));
}

/// ----------------------------------------------------------------------------------------------------------
pub fn manifest(phext: &str) -> String {
  let mut phokens = phokenize(phext);
  let mut i = 0;
  while i < phokens.len() {
    phokens[i].scroll = checksum(phokens[i].scroll.as_str());
    i += 1;
  }

  let result = dephokenize(&mut phokens);

  return result;
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
    if letter1.contains(c as char) { value += 1; continue; }
    if letter2.contains(c as char) { value += 2; continue; }
    if letter3.contains(c as char) { value += 3; continue; }
    if letter4.contains(c as char) { value += 4; continue; }
    if letter5.contains(c as char) { value += 5; continue; }
    if letter6.contains(c as char) { value += 6; continue; }
  }

  return (value % 99).to_string();
}

/// ----------------------------------------------------------------------------------------------------------
pub fn soundex_v1(phext: &str) -> String {
  let mut phokens = phokenize(phext);
  
  for ith in &mut phokens {
    ith.scroll = soundex_internal(ith.scroll.clone());
  }

  return dephokenize(&mut phokens);
}

/// ----------------------------------------------------------------------------------------------------------
fn index_phokens(phext: &str) -> Vec<PositionedScroll> {
  let phokens = phokenize(phext);
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
    
    output.push(PositionedScroll { coord, scroll: format!("{}", offset)});
    offset += phokens[i].scroll.len();
    i += 1;
  }

  return output;
}

/// ----------------------------------------------------------------------------------------------------------
pub fn index(phext: &str) -> String {
  let mut output = index_phokens(phext);

  return dephokenize(&mut output);
}

/// ----------------------------------------------------------------------------------------------------------
pub fn offset(phext: &str, coord: Coordinate) -> usize {
  let mut output = index_phokens(phext);

  let mut best = default_coordinate();
  let mut matched = false;
  let mut fetch_coord = coord;
  for phoken in output.clone() {
    if phoken.coord <= coord {
      best = phoken.coord;
    }
    if phoken.coord == coord {
      matched = true;
    }
  }

  if matched == false {
    fetch_coord = best;
  }
  let index = dephokenize(&mut output);
  
  return fetch(index.as_str(), fetch_coord).parse::<usize>().unwrap();
}

/// ----------------------------------------------------------------------------------------------------------
pub fn replace(phext: &str, location: Coordinate, scroll: &str) -> String {
  let bytes = phext.as_bytes();
  let parts = get_subspace_coordinates(bytes, location);
  let start: usize = parts.0;
  let mut end: usize = parts.1;
  let mut fixup: Vec<u8> = vec![];
  let mut subspace_coordinate: Coordinate = parts.2;

  while subspace_coordinate.z.library < location.z.library {
    fixup.push(LIBRARY_BREAK as u8);
    subspace_coordinate.library_break();
  }
  while subspace_coordinate.z.shelf < location.z.shelf {
    fixup.push(SHELF_BREAK as u8);
    subspace_coordinate.shelf_break();
  }
  while subspace_coordinate.z.series < location.z.series {
    fixup.push(SERIES_BREAK as u8);
    subspace_coordinate.series_break();
  }
  while subspace_coordinate.y.collection < location.y.collection {
    fixup.push(COLLECTION_BREAK as u8);
    subspace_coordinate.collection_break();
  }
  while subspace_coordinate.y.volume < location.y.volume {
    fixup.push(VOLUME_BREAK as u8);
    subspace_coordinate.volume_break();
  }
  while subspace_coordinate.y.book < location.y.book {
    fixup.push(BOOK_BREAK as u8);
    subspace_coordinate.book_break();
  }
  while subspace_coordinate.x.chapter < location.x.chapter {
    fixup.push(CHAPTER_BREAK as u8);
    subspace_coordinate.chapter_break();
  }
  while subspace_coordinate.x.section < location.x.section {
    fixup.push(SECTION_BREAK as u8);
    subspace_coordinate.section_break();
  }
  while subspace_coordinate.x.scroll < location.x.scroll {
    fixup.push(SCROLL_BREAK as u8);
    subspace_coordinate.scroll_break();
  }

  let text: std::slice::Iter<u8> = scroll.as_bytes().iter();
  let max = bytes.len();
  if end > max { end = max; }
  let left = &bytes[..start];
  let right = &bytes[end..];
  let temp:Vec<u8> = left.iter().chain(fixup.iter()).chain(text).chain(right.iter()).cloned().collect();
  let result: String = String::from_utf8(temp).expect("invalid utf8");
  return result;
}

/// ----------------------------------------------------------------------------------------------------------
/// errata: libphext-py opted for ranges that operate on a stack of positioned scrolls
/// this method applies the range edit to subspace directly...we probably need both
pub fn range_replace(phext: &str, location: Range, scroll: &str) -> String {
  let bytes = phext.as_bytes();
  let parts_start = get_subspace_coordinates(bytes, location.start);
  let parts_end = get_subspace_coordinates(bytes, location.end);
  println!("Start: {} vs {}", location.start, parts_start.2.to_string());
  println!("End: {} vs {}", location.end, parts_end.2.to_string());
  let start: usize = parts_start.0;
  let mut end: usize = parts_end.1;
  println!("Subspace start: {}, end: {}", start, end);

  let text: std::slice::Iter<u8> = scroll.as_bytes().iter();
  let max = bytes.len();
  if end > max { end = max; }
  let left = &bytes[..start];
  let right = &bytes[end..];
  let temp:Vec<u8> = left.iter().chain(text).chain(right.iter()).cloned().collect();
  let result: String = String::from_utf8(temp).expect("invalid utf8");
  return result;
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn insert
///
/// inserts the content specified in `scroll` at the coordinate within `phext` specified by `location`
/// ----------------------------------------------------------------------------------------------------------
pub fn insert(phext: String, location: Coordinate, scroll: &str) -> String {
  let bytes: &[u8] = phext.as_bytes();
  let parts: (usize, usize, Coordinate) = get_subspace_coordinates(bytes, location);
  let end: usize = parts.1;
  let mut fixup: Vec<u8> = vec![];
  let mut subspace_coordinate: Coordinate = parts.2;

  while subspace_coordinate.z.library < location.z.library {
    fixup.push(LIBRARY_BREAK as u8);
    subspace_coordinate.library_break();
  }
  while subspace_coordinate.z.shelf < location.z.shelf {
    fixup.push(SHELF_BREAK as u8);
    subspace_coordinate.shelf_break();
  }
  while subspace_coordinate.z.series < location.z.series {
    fixup.push(SERIES_BREAK as u8);
    subspace_coordinate.series_break();
  }
  while subspace_coordinate.y.collection < location.y.collection {
    fixup.push(COLLECTION_BREAK as u8);
    subspace_coordinate.collection_break();
  }
  while subspace_coordinate.y.volume < location.y.volume {
    fixup.push(VOLUME_BREAK as u8);
    subspace_coordinate.volume_break();
  }
  while subspace_coordinate.y.book < location.y.book {
    fixup.push(BOOK_BREAK as u8);
    subspace_coordinate.book_break();
  }
  while subspace_coordinate.x.chapter < location.x.chapter {
    fixup.push(CHAPTER_BREAK as u8);
    subspace_coordinate.chapter_break();
  }
  while subspace_coordinate.x.section < location.x.section {
    fixup.push(SECTION_BREAK as u8);
    subspace_coordinate.section_break();
  }
  while subspace_coordinate.x.scroll < location.x.scroll {
    fixup.push(SCROLL_BREAK as u8);
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

  return result;
}


/// ----------------------------------------------------------------------------------------------------------
/// @fn next_scroll
///
/// retrieves the next scroll from the given string, assuming an arbitrary starting point
/// ----------------------------------------------------------------------------------------------------------
pub fn next_scroll(phext: &str, start: Coordinate) -> (PositionedScroll, Coordinate, String) {
  let mut location = start;
  let p = phext.as_bytes();
  let mut output: Vec<u8> = vec![];
  let mut remaining: Vec<u8> = vec![];
  let mut pi: usize = 0;
  let mut begin: Coordinate = start;
  let pmax = p.len();
  while pi < pmax
  {
    let test = p[pi] as char;
    let mut dimension_break: bool = false;
    if test == SCROLL_BREAK     { location.scroll_break();     dimension_break = true; }
    if test == SECTION_BREAK    { location.section_break();    dimension_break = true; }
    if test == CHAPTER_BREAK    { location.chapter_break();    dimension_break = true; }
    if test == BOOK_BREAK       { location.book_break();       dimension_break = true; }
    if test == VOLUME_BREAK     { location.volume_break();     dimension_break = true; }
    if test == COLLECTION_BREAK { location.collection_break(); dimension_break = true; }
    if test == SERIES_BREAK     { location.series_break();     dimension_break = true; }
    if test == SHELF_BREAK      { location.shelf_break();      dimension_break = true; }
    if test == LIBRARY_BREAK    { location.library_break();    dimension_break = true; }

    if dimension_break {
      if output.len() > 0 {
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
  return (out_scroll, location, String::from_utf8(remaining).expect("valid UTF-8"));
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn phokenize
///
/// Transforms a packed phext buffer into a phext token (phoken) stream
/// ----------------------------------------------------------------------------------------------------------
pub fn phokenize(phext: &str) -> Vec<PositionedScroll> {
  let mut result: Vec<PositionedScroll> = Vec::new();
  let mut coord = default_coordinate();
  let mut temp: String = phext.to_string();
  loop {
    let item: PositionedScroll;
    (item, coord, temp) = next_scroll(temp.as_str(), coord);
    result.push(item);
    if temp.len() == 0 { break; }
  }

  return result;
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn explode
///
/// Explodes an input phext into a hierarchical hashap, suitable for fast access and updates
/// ----------------------------------------------------------------------------------------------------------
pub fn explode(phext: &str) -> HashMap<Coordinate, String> {
  let parts = phokenize(phext);
  let mut hash = HashMap::new();
  for row in parts {
    hash.insert(row.coord, row.scroll);
  }
  return hash;
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
  return dephokenize(&mut vec);
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
    
    let pick_left = have_left && (have_right == false || tl[tli].coord <= tr[tri].coord);
    let pick_right = have_right && (have_left == false || tr[tri].coord <= tl[tli].coord);

    if pick_left {
      result.push_str(&append_scroll(tl[tli].clone(), coord));
      coord = tl[tli].coord;
      tli += 1;
    }
    if pick_right {
      result.push_str(&append_scroll(tr[tri].clone(), coord));
      coord = tr[tri].coord;
      tri += 1;
    }

    if pick_left == false && pick_right == false {
      break;
    }
  }

  return result;
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn fetch
///
/// retrieves the plain text string located at the given coordinates.
/// important: this can be optimized with hash tables and memo-ized parsing - for now let's keep it simple
/// see my C# implementation in https://github.com/wbic16/terse-editor if you want to do that
///
/// @param phext  the raw phext buffer to search
/// @param coord  coordinate to select the scroll from
/// ----------------------------------------------------------------------------------------------------------
pub fn fetch(phext: &str, target: Coordinate) -> String {
  let bytes: &[u8] = phext.as_bytes();
  let parts = get_subspace_coordinates(bytes, target);

  let start = parts.0 as usize;
  let end = parts.1 as usize;

  if end > start
  {
    let glyphs: usize = end - start;
    let temp: Vec<u8> = bytes.iter().skip(start).take(glyphs).cloned().collect();
    let result: String = String::from_utf8(temp).expect("invalid utf8");
    return result;
  }

  return "".to_owned();
}

/// ----------------------------------------------------------------------------------------------------------
pub fn expand(phext: &str) -> String {
  let mut copy = phext.to_string().clone();
  unsafe {
  let buffer = copy.as_bytes_mut();
  let max = buffer.len();
  let mut p: usize = 0;
  loop {
    if p == max { break; }
    if buffer[p] == LINE_BREAK as u8 {
      buffer[p] = SCROLL_BREAK as u8;
    } else if buffer[p] == SCROLL_BREAK as u8 {
      buffer[p] = SECTION_BREAK as u8;
    } else if buffer[p] == SECTION_BREAK as u8 {
      buffer[p] = CHAPTER_BREAK as u8;
    } else if buffer[p] == CHAPTER_BREAK as u8 {
      buffer[p] = BOOK_BREAK as u8;
    } else if buffer[p] == BOOK_BREAK as u8 {
      buffer[p] = VOLUME_BREAK as u8;
    } else if buffer[p] == VOLUME_BREAK as u8{
      buffer[p] = COLLECTION_BREAK as u8;
    } else if buffer[p] == COLLECTION_BREAK as u8 {
      buffer[p] = SERIES_BREAK as u8;
    } else if buffer[p] == SERIES_BREAK as u8 {
      buffer[p] = SHELF_BREAK as u8;
    } else if buffer[p] == SHELF_BREAK as u8 {
      buffer[p] = LIBRARY_BREAK as u8;
    }
    p += 1;
  }

  let temp: Vec<u8> = buffer.iter().cloned().collect();
  let result: String = String::from_utf8(temp).expect("invalid utf8");
  return result;
  }
}

/// ----------------------------------------------------------------------------------------------------------
pub fn contract(phext: &str) -> String {
  let mut copy = phext.to_string().clone();
  unsafe {
  let buffer = copy.as_bytes_mut();
  let max = buffer.len();
  let mut p: usize = 0;
  loop {
    if p == max { break; }
    
    if buffer[p] == LIBRARY_BREAK as u8 {
      buffer[p] = SHELF_BREAK as u8;
    } else if buffer[p] == SHELF_BREAK as u8 {
      buffer[p] = SERIES_BREAK as u8;
    } else if buffer[p] == SERIES_BREAK as u8 {
      buffer[p] = COLLECTION_BREAK as u8;
    } else if buffer[p] == COLLECTION_BREAK as u8{
      buffer[p] = VOLUME_BREAK as u8;
    } else if buffer[p] == VOLUME_BREAK as u8 {
      buffer[p] = BOOK_BREAK as u8;
    } else if buffer[p] == BOOK_BREAK as u8 {
      buffer[p] = CHAPTER_BREAK as u8;
    } else if buffer[p] == CHAPTER_BREAK as u8 {
      buffer[p] = SECTION_BREAK as u8;
    } else if buffer[p] == SECTION_BREAK as u8 {
      buffer[p] = SCROLL_BREAK as u8;
    } else if buffer[p] == SCROLL_BREAK as u8 {
      buffer[p] = LINE_BREAK as u8;
    }   
    p += 1;
  }

  let temp: Vec<u8> = buffer.iter().cloned().collect();
  let result: String = String::from_utf8(temp).expect("invalid utf8");
  return result;
  }
}

/// ----------------------------------------------------------------------------------------------------------
fn dephokenize(tokens: &mut Vec<PositionedScroll>) -> String {
  let mut result: String = Default::default();
  let mut coord = default_coordinate();
  for ps in tokens {
    if ps.scroll.len() > 0 {
      result.push_str(&append_scroll(ps.clone(), coord));
    }
    coord = ps.coord;
  }
  return result;
}

/// ----------------------------------------------------------------------------------------------------------
fn append_scroll(token: PositionedScroll, mut coord: Coordinate) -> String {
  let mut output: String = Default::default();
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
  return output;
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
    let mut do_append = pri == max;

    if pri < max {
      let compare = pr[pri].clone();
      if token.coord < compare.coord {
        do_append = true;
      } else if token.coord == compare.coord {
        pri += 1;
      }
    }

    if do_append {
      result.push_str(&append_scroll(token.clone(), coord));
      coord = token.coord;
    }
  }

  return result;
}

/// ----------------------------------------------------------------------------------------------------------
/// @fn is_phext_break
///
/// returns true if `byte` corresponds to one of our phext delimiters (line breaks included)
/// ----------------------------------------------------------------------------------------------------------
pub fn is_phext_break(byte: u8) -> bool {
  return byte == LINE_BREAK as u8 ||
         byte == SCROLL_BREAK as u8 ||
         byte == SECTION_BREAK as u8 ||
         byte == CHAPTER_BREAK as u8 ||
         byte == BOOK_BREAK as u8 ||
         byte == VOLUME_BREAK as u8 ||
         byte == COLLECTION_BREAK as u8 ||
         byte == SERIES_BREAK as u8 ||
         byte == SHELF_BREAK as u8 ||
         byte == LIBRARY_BREAK as u8;
}

/// ----------------------------------------------------------------------------------------------------------
pub fn normalize(phext: &str) -> String {
  let mut arr = phokenize(phext);
  return dephokenize(&mut arr);
}

/// ----------------------------------------------------------------------------------------------------------
pub fn default_coordinate() -> Coordinate {
  let coord = Coordinate {
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
  };
  return coord;
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
  let mut value:u32 = 0;
  let exp:u32 = 10;

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

    if byte >= 0x30 && byte <= 0x39
    {
      value = exp * value + ((byte - 0x30) as u32);
      if index == 0 { index = 1; }
    }
  }

  if index > 0 {
    result.x.scroll = value as usize;
  }

  return result;
}

fn validate_dimension_index(index: usize) -> bool {
  return index >= COORDINATE_MINIMUM && index <= COORDINATE_MAXIMUM;
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
    let ok = validate_dimension_index(self.z.library) &&
                   validate_dimension_index(self.z.shelf) &&
                   validate_dimension_index(self.z.series) &&
                   validate_dimension_index(self.y.collection) &&
                   validate_dimension_index(self.y.volume) &&
                   validate_dimension_index(self.y.book) &&
                   validate_dimension_index(self.x.chapter) &&
                   validate_dimension_index(self.x.section) &&
                   validate_dimension_index(self.x.scroll);
    return ok;
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
    return format!("{}.{}.{}/{}.{}.{}/{}.{}.{}",
      self.z.library, self.z.shelf, self.z.series,
      self.y.collection, self.y.volume, self.y.book,
      self.x.chapter, self.x.section, self.x.scroll);
  }

  pub fn to_urlencoded(&self) -> String {
    if !self.validate_coordinate() {
      return "".to_owned();
    }
    return format!("{}.{}.{};{}.{}.{};{}.{}.{}",
      self.z.library, self.z.shelf, self.z.series,
      self.y.collection, self.y.volume, self.y.book,
      self.x.chapter, self.x.section, self.x.scroll);
  }

  /// ----------------------------------------------------------------------------------------------------------
  /// @fn advance_coordinate
  ///
  /// ----------------------------------------------------------------------------------------------------------
  fn advance_coordinate(index: usize) -> usize {
    let next: usize = index + 1;
    if next < COORDINATE_MAXIMUM {
      return next;
    }

    return index; // can't advance beyond the maximum
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