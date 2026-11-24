//! Scroll stores for edit sequences (v0.7.0 performance work).
//!
//! The free-function buffer API in `phext` is buffer-in/buffer-out on
//! immutable strings: every edit scans the whole buffer and copies it, so
//! a sequence of N edits on a growing phext costs O(N x size). The two
//! stores in this module pay the parse once and mutate in place,
//! serializing on demand. Both are additive: no existing public API
//! changes behavior.
//!
//! * `DynamicPhext` — a capacity-managed dynamic array of scrolls in
//!   coordinate order. Capacity starts at 1, doubles when full, and
//!   shrinks ONLY at quarter-full (`n <= capacity/4`) — never at
//!   half-full, which thrashes at a resize boundary (alternating
//!   insert/remove pairs each copy the whole array). Resizes are
//!   instrumented (`resize_count`, `elements_copied`).
//! * `TransitivePhext` over `BranchingMap` — hash maps of hash maps,
//!   transitively: ONE recursive concept, a map of dimension d whose
//!   values are maps of dimension d-1, down to leaves holding scroll
//!   text. The depth is derived from `Coordinate` (9 components), never
//!   a hardcoded chain.
//!
//! Edit semantics match the string path exactly: `insert` at an existing
//! coordinate concatenates old text before new, `replace` overwrites,
//! `fetch` returns the text or `""`, and neither store ever holds an
//! empty scroll (`replace(coord, "")` removes; `insert` of `""` is a
//! no-op) — the normalized semantics the string path exhibits through
//! `remove`/`normalize`.

use crate::phext::{self, Coordinate, PositionedScroll};
use std::collections::HashMap;

/// The 9 coordinate components in (library, shelf, series, collection,
/// volume, book, chapter, section, scroll) order.
pub fn coord_key(coord: Coordinate) -> [usize; 9] {
  [
    coord.z.library, coord.z.shelf, coord.z.series,
    coord.y.collection, coord.y.volume, coord.y.book,
    coord.x.chapter, coord.x.section, coord.x.scroll,
  ]
}

/// Inverse of `coord_key`.
pub fn coord_from_key(key: [usize; 9]) -> Coordinate {
  let mut coord = phext::default_coordinate();
  coord.z.library = key[0];
  coord.z.shelf = key[1];
  coord.z.series = key[2];
  coord.y.collection = key[3];
  coord.y.volume = key[4];
  coord.y.book = key[5];
  coord.x.chapter = key[6];
  coord.x.section = key[7];
  coord.x.scroll = key[8];
  coord
}

/// Number of dimensions in a phext coordinate, derived from the type.
pub fn coordinate_depth() -> usize {
  coord_key(phext::default_coordinate()).len()
}

fn serialize_sorted(scrolls: Vec<PositionedScroll>) -> String {
  let mut map: HashMap<Coordinate, String> = HashMap::new();
  for ps in scrolls {
    if !ps.scroll.is_empty() {
      map.insert(ps.coord, ps.scroll);
    }
  }
  phext::implode(map)
}

// ---------------------------------------------------------------------------
// DynamicPhext: capacity-managed dynamic array
// ---------------------------------------------------------------------------

const MIN_CAPACITY: usize = 1;

pub struct DynamicPhext {
  data: Vec<PositionedScroll>, // len == capacity; slots >= n are dummies
  n: usize,
  capacity: usize,
  shrink_divisor: usize,
  resize_count: usize,
  elements_copied: usize,
}

impl Default for DynamicPhext {
  fn default() -> Self {
    Self::new()
  }
}

impl DynamicPhext {
  pub fn new() -> DynamicPhext {
    DynamicPhext::with_buffer("")
  }

  pub fn with_buffer(buffer: &str) -> DynamicPhext {
    let mut store = DynamicPhext {
      data: vec![PositionedScroll::new(); MIN_CAPACITY],
      n: 0,
      capacity: MIN_CAPACITY,
      shrink_divisor: 4,
      resize_count: 0,
      elements_copied: 0,
    };
    if !buffer.is_empty() {
      store.load(buffer);
    }
    store
  }

  /// Test-visible variant reproducing the thrash-prone half-full shrink
  /// strategy (`shrink_divisor = 2`). Production code uses 4.
  pub fn with_shrink_divisor(buffer: &str, shrink_divisor: usize) -> DynamicPhext {
    assert!(shrink_divisor >= 2);
    let mut store = DynamicPhext::with_buffer(buffer);
    store.shrink_divisor = shrink_divisor;
    store
  }

  fn resize(&mut self, new_capacity: usize) {
    let mut new_data = vec![PositionedScroll::new(); new_capacity];
    for i in 0..self.n {
      new_data[i] = self.data[i].clone();
    }
    self.elements_copied += self.n;
    self.resize_count += 1;
    self.data = new_data;
    self.capacity = new_capacity;
  }

  fn grow_if_full(&mut self) {
    if self.n == self.capacity {
      self.resize(self.capacity * 2);
    }
  }

  fn shrink_if_sparse(&mut self) {
    if self.capacity > MIN_CAPACITY && self.n <= self.capacity / self.shrink_divisor {
      let target = std::cmp::max(MIN_CAPACITY, self.capacity / 2);
      self.resize(target);
    }
  }

  pub fn capacity(&self) -> usize { self.capacity }
  pub fn resize_count(&self) -> usize { self.resize_count }
  pub fn elements_copied(&self) -> usize { self.elements_copied }
  pub fn reset_stats(&mut self) {
    self.resize_count = 0;
    self.elements_copied = 0;
  }
  pub fn len(&self) -> usize { self.n }
  pub fn is_empty(&self) -> bool { self.n == 0 }

  pub fn load(&mut self, buffer: &str) {
    self.data = vec![PositionedScroll::new(); MIN_CAPACITY];
    self.capacity = MIN_CAPACITY;
    self.n = 0;
    self.reset_stats();
    for ps in phext::phokenize(buffer) {
      if !ps.scroll.is_empty() {
        self.push(ps);
      }
    }
  }

  fn push(&mut self, ps: PositionedScroll) {
    self.grow_if_full();
    self.data[self.n] = ps;
    self.n += 1;
  }

  pub fn scrolls(&self) -> Vec<PositionedScroll> {
    self.data[..self.n].to_vec()
  }

  pub fn serialize(&self) -> String {
    serialize_sorted(self.scrolls())
  }

  fn find(&self, coord: Coordinate) -> (usize, bool) {
    let mut lo = 0usize;
    let mut hi = self.n;
    while lo < hi {
      let mid = (lo + hi) / 2;
      match self.data[mid].coord.cmp(&coord) {
        std::cmp::Ordering::Less => lo = mid + 1,
        std::cmp::Ordering::Greater => hi = mid,
        std::cmp::Ordering::Equal => return (mid, true),
      }
    }
    (lo, false)
  }

  pub fn fetch(&self, coord: Coordinate) -> String {
    let (index, found) = self.find(coord);
    if found { self.data[index].scroll.clone() } else { String::new() }
  }

  pub fn contains(&self, coord: Coordinate) -> bool {
    self.find(coord).1
  }

  pub fn append(&mut self, coord: Coordinate, text: &str) {
    assert!(
      self.n == 0 || coord > self.data[self.n - 1].coord,
      "append requires a coordinate past the current tail"
    );
    if text.is_empty() { return; }
    self.push(PositionedScroll { coord, scroll: text.to_string() });
  }

  pub fn update(&mut self, coord: Coordinate, text: &str, overwrite: bool) {
    let (index, found) = self.find(coord);
    if found {
      if text.is_empty() {
        if overwrite { self.remove_at(index); }
        return;
      }
      let new_text = if overwrite {
        text.to_string()
      } else {
        let mut t = self.data[index].scroll.clone();
        t.push_str(text);
        t
      };
      self.data[index] = PositionedScroll { coord, scroll: new_text };
      return;
    }
    if text.is_empty() { return; }
    self.grow_if_full();
    let mut i = self.n;
    while i > index {
      self.data[i] = self.data[i - 1].clone();
      i -= 1;
    }
    self.data[index] = PositionedScroll { coord, scroll: text.to_string() };
    self.n += 1;
  }

  pub fn insert(&mut self, coord: Coordinate, text: &str) {
    self.update(coord, text, false);
  }

  pub fn replace(&mut self, coord: Coordinate, text: &str) {
    self.update(coord, text, true);
  }

  fn remove_at(&mut self, index: usize) {
    for i in index..self.n - 1 {
      self.data[i] = self.data[i + 1].clone();
    }
    self.n -= 1;
    self.data[self.n] = PositionedScroll::new();
    self.shrink_if_sparse();
  }

  pub fn remove(&mut self, coord: Coordinate) {
    let (index, found) = self.find(coord);
    if found { self.remove_at(index); }
  }
}

// ---------------------------------------------------------------------------
// BranchingMap: one recursive concept, dimension-generic
// ---------------------------------------------------------------------------

enum Node<V> {
  Branch(HashMap<usize, Node<V>>),
  Leaf(V),
}

/// A hash map of dimension `depth` branching into hash maps of one fewer
/// dimension, recursively, down to leaves holding values. Keys are
/// component arrays of exactly `depth` entries. Every operation is
/// defined recursively over the hierarchy: one hash lookup per level.
pub struct BranchingMap<V> {
  depth: usize,
  root: HashMap<usize, Node<V>>,
  count: usize,
}

impl<V> BranchingMap<V> {
  pub fn new(depth: usize) -> BranchingMap<V> {
    assert!(depth >= 1, "depth must be >= 1");
    BranchingMap { depth, root: HashMap::new(), count: 0 }
  }

  pub fn depth(&self) -> usize { self.depth }
  pub fn len(&self) -> usize { self.count }
  pub fn is_empty(&self) -> bool { self.count == 0 }

  fn check(&self, key: &[usize]) {
    assert_eq!(key.len(), self.depth, "key length must equal map depth");
  }

  pub fn fetch(&self, key: &[usize]) -> Option<&V> {
    self.check(key);
    Self::fetch_at(&self.root, key, 0)
  }

  fn fetch_at<'a>(node: &'a HashMap<usize, Node<V>>, key: &[usize], level: usize) -> Option<&'a V> {
    match node.get(&key[level]) {
      None => None,
      Some(Node::Leaf(value)) => Some(value),
      Some(Node::Branch(child)) => Self::fetch_at(child, key, level + 1),
    }
  }

  pub fn contains(&self, key: &[usize]) -> bool {
    self.fetch(key).is_some()
  }

  /// Store `value` at `key`; returns true when a new leaf was created.
  pub fn put(&mut self, key: &[usize], value: V) -> bool {
    self.check(key);
    let created = Self::put_at(&mut self.root, key, 0, self.depth, value);
    if created { self.count += 1; }
    created
  }

  fn put_at(node: &mut HashMap<usize, Node<V>>, key: &[usize], level: usize, depth: usize, value: V) -> bool {
    if level == depth - 1 {
      let created = !node.contains_key(&key[level]);
      node.insert(key[level], Node::Leaf(value));
      return created;
    }
    let child = match node.entry(key[level]).or_insert_with(|| Node::Branch(HashMap::new())) {
      Node::Branch(child) => child,
      Node::Leaf(_) => unreachable!("leaf above the leaf level"),
    };
    Self::put_at(child, key, level + 1, depth, value)
  }

  pub fn remove(&mut self, key: &[usize]) -> bool {
    self.check(key);
    let removed = Self::remove_at(&mut self.root, key, 0, self.depth);
    if removed { self.count -= 1; }
    removed
  }

  fn remove_at(node: &mut HashMap<usize, Node<V>>, key: &[usize], level: usize, depth: usize) -> bool {
    if level == depth - 1 {
      return node.remove(&key[level]).is_some();
    }
    let (removed, prune) = match node.get_mut(&key[level]) {
      None => return false,
      Some(Node::Leaf(_)) => return false,
      Some(Node::Branch(child)) => {
        let removed = Self::remove_at(child, key, level + 1, depth);
        (removed, removed && child.is_empty())
      }
    };
    if prune { node.remove(&key[level]); }
    removed
  }

  /// All (key, value) pairs in lexicographic (coordinate) order: keys are
  /// sorted at every level of the hierarchy.
  pub fn items(&self) -> Vec<(Vec<usize>, &V)> {
    let mut out = Vec::new();
    Self::items_at(&self.root, Vec::new(), &mut out);
    out
  }

  fn items_at<'a>(node: &'a HashMap<usize, Node<V>>, prefix: Vec<usize>, out: &mut Vec<(Vec<usize>, &'a V)>) {
    let mut keys: Vec<usize> = node.keys().copied().collect();
    keys.sort_unstable();
    for k in keys {
      let mut p = prefix.clone();
      p.push(k);
      match &node[&k] {
        Node::Leaf(value) => out.push((p, value)),
        Node::Branch(child) => Self::items_at(child, p, out),
      }
    }
  }
}

// ---------------------------------------------------------------------------
// TransitivePhext: BranchingMap at coordinate depth
// ---------------------------------------------------------------------------

pub struct TransitivePhext {
  map: BranchingMap<String>,
  depth: usize,
}

impl Default for TransitivePhext {
  fn default() -> Self {
    Self::new()
  }
}

impl TransitivePhext {
  pub fn new() -> TransitivePhext {
    TransitivePhext::with_buffer("")
  }

  pub fn with_buffer(buffer: &str) -> TransitivePhext {
    let depth = coordinate_depth();
    let mut store = TransitivePhext { map: BranchingMap::new(depth), depth };
    if !buffer.is_empty() {
      store.load(buffer);
    }
    store
  }

  pub fn depth(&self) -> usize { self.depth }
  pub fn len(&self) -> usize { self.map.len() }
  pub fn is_empty(&self) -> bool { self.map.is_empty() }

  pub fn load(&mut self, buffer: &str) {
    self.map = BranchingMap::new(self.depth);
    for ps in phext::phokenize(buffer) {
      if !ps.scroll.is_empty() {
        self.map.put(&coord_key(ps.coord), ps.scroll);
      }
    }
  }

  pub fn scrolls(&self) -> Vec<PositionedScroll> {
    self.map
      .items()
      .into_iter()
      .map(|(key, text)| {
        let mut arr = [0usize; 9];
        arr.copy_from_slice(&key);
        PositionedScroll { coord: coord_from_key(arr), scroll: text.clone() }
      })
      .collect()
  }

  pub fn serialize(&self) -> String {
    serialize_sorted(self.scrolls())
  }

  pub fn fetch(&self, coord: Coordinate) -> String {
    match self.map.fetch(&coord_key(coord)) {
      Some(text) => text.clone(),
      None => String::new(),
    }
  }

  pub fn contains(&self, coord: Coordinate) -> bool {
    self.map.contains(&coord_key(coord))
  }

  pub fn update(&mut self, coord: Coordinate, text: &str, overwrite: bool) {
    let key = coord_key(coord);
    if text.is_empty() {
      if overwrite { self.map.remove(&key); }
      return;
    }
    let new_text = if overwrite {
      text.to_string()
    } else {
      match self.map.fetch(&key) {
        Some(existing) => {
          let mut t = existing.clone();
          t.push_str(text);
          t
        }
        None => text.to_string(),
      }
    };
    self.map.put(&key, new_text);
  }

  pub fn insert(&mut self, coord: Coordinate, text: &str) {
    self.update(coord, text, false);
  }

  pub fn replace(&mut self, coord: Coordinate, text: &str) {
    self.update(coord, text, true);
  }

  pub fn remove(&mut self, coord: Coordinate) {
    self.map.remove(&coord_key(coord));
  }

  /// A hash hierarchy has no tail; append is insert. Kept for API parity
  /// with `DynamicPhext`.
  pub fn append(&mut self, coord: Coordinate, text: &str) {
    self.insert(coord, text);
  }
}
