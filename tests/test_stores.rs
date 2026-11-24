//! Equivalence + structure tests for the v0.7.0 scroll stores.
//! Seeded random edit sequences (>=400 ops) assert byte-identical
//! serialization vs the original string path AFTER EVERY OP, for both
//! stores; fetch agreement is sampled throughout.
use libphext::phext;
use libphext::phext::Coordinate;
use libphext::stores::{BranchingMap, DynamicPhext, TransitivePhext};

fn coord(a: [usize; 9]) -> Coordinate {
  let mut c = phext::default_coordinate();
  c.z.library = a[0]; c.z.shelf = a[1]; c.z.series = a[2];
  c.y.collection = a[3]; c.y.volume = a[4]; c.y.book = a[5];
  c.x.chapter = a[6]; c.x.section = a[7]; c.x.scroll = a[8];
  c
}

// xorshift64* — deterministic, stdlib only.
struct Rng(u64);
impl Rng {
  fn next(&mut self) -> u64 {
    let mut x = self.0;
    x ^= x >> 12; x ^= x << 25; x ^= x >> 27;
    self.0 = x;
    x.wrapping_mul(0x2545F4914F6CDD1D)
  }
  fn below(&mut self, n: usize) -> usize { (self.next() % n as u64) as usize }
}

fn pool() -> Vec<Coordinate> {
  // bounded pool: 3 sections x 60 scrolls = 180 coords (mirrors py test)
  let mut v = Vec::new();
  for section in 1..=3usize {
    for scroll in 1..=60usize {
      v.push(coord([1, 1, 1, 1, 1, 1, 1, section, scroll]));
    }
  }
  v
}

#[test]
fn random_edit_sequence_equivalence_both_stores() {
  let pool = pool();
  let mut rng = Rng(0x20261124);
  let mut buf = String::new();
  let mut dyn_store = DynamicPhext::new();
  let mut tr_store = TransitivePhext::new();
  let words = ["alpha", "beta gamma", "delta\nepsilon", "zeta", "eta theta iota", "kappa"];
  for op in 0..600 {
    let c = pool[rng.below(pool.len())];
    let text = words[rng.below(words.len())];
    match rng.below(10) {
      0..=4 => {
        buf = phext::insert(buf, c, text);
        dyn_store.insert(c, text);
        tr_store.insert(c, text);
      }
      5..=7 => {
        buf = phext::replace(&buf, c, text);
        dyn_store.replace(c, text);
        tr_store.replace(c, text);
      }
      _ => {
        buf = phext::remove(&buf, c);
        dyn_store.remove(c);
        tr_store.remove(c);
      }
    }
    assert_eq!(dyn_store.serialize(), buf, "dynamic diverged after op {}", op);
    assert_eq!(tr_store.serialize(), buf, "transitive diverged after op {}", op);
    if op % 25 == 0 {
      for _ in 0..8 {
        let f = pool[rng.below(pool.len())];
        let expected = phext::fetch(&buf, f);
        assert_eq!(dyn_store.fetch(f), expected, "dynamic fetch op {}", op);
        assert_eq!(tr_store.fetch(f), expected, "transitive fetch op {}", op);
      }
    }
  }
}

#[test]
fn quarter_full_no_thrash_vs_half_full() {
  // 64-scroll boundary, 50 alternating append/remove pairs.
  let build = |divisor: usize| {
    let mut s = DynamicPhext::with_shrink_divisor("", divisor);
    for i in 0..64 {
      s.append(coord([1, 1, 1, 1, 1, 1, 1, 1, i + 1]), "x");
    }
    s.reset_stats();
    s
  };
  let tail = coord([1, 1, 1, 1, 1, 1, 1, 1, 65]);
  let mut quarter = build(4);
  for _ in 0..50 { quarter.append(tail, "x"); quarter.remove(tail); }
  assert!(quarter.elements_copied() <= 64,
    "quarter-full copied {} elements", quarter.elements_copied());

  let mut half = build(2);
  for _ in 0..50 { half.append(tail, "x"); half.remove(tail); }
  assert!(half.elements_copied() >= 3200,
    "half-full copied only {} elements", half.elements_copied());
}

#[test]
fn growth_is_amortized() {
  let mut s = DynamicPhext::new();
  for i in 0..1000 {
    s.append(coord([1, 1, 1, 1, 1, 1, 1, (i / 50) % 40 + 1, i % 50 + 1]), "x");
  }
  assert_eq!(s.resize_count(), 10);
  assert!(s.elements_copied() < 2000, "copied {}", s.elements_copied());
  // no shrink at half full: capacity holds at 64 with 32 live
  let mut s2 = DynamicPhext::new();
  for i in 0..64 { s2.append(coord([1, 1, 1, 1, 1, 1, 1, 1, i + 1]), "x"); }
  assert_eq!(s2.capacity(), 64);
  for i in (32..64).rev() { s2.remove(coord([1, 1, 1, 1, 1, 1, 1, 1, i + 1])); }
  assert_eq!(s2.len(), 32);
  assert_eq!(s2.capacity(), 64);
}

#[test]
fn branching_map_is_dimension_generic() {
  let mut m3: BranchingMap<String> = BranchingMap::new(3);
  m3.put(&[1, 2, 3], "a".to_string());
  m3.put(&[1, 2, 4], "b".to_string());
  assert_eq!(m3.fetch(&[1, 2, 3]).map(|s| s.as_str()), Some("a"));
  assert_eq!(m3.len(), 2);
  assert!(m3.remove(&[1, 2, 3]));
  assert_eq!(m3.fetch(&[1, 2, 3]), None);
  let mut m1: BranchingMap<String> = BranchingMap::new(1);
  m1.put(&[7], "solo".to_string());
  assert_eq!(m1.fetch(&[7]).map(|s| s.as_str()), Some("solo"));
  let items: Vec<Vec<usize>> = m3.items().into_iter().map(|(k, _)| k).collect();
  assert_eq!(items, vec![vec![1, 2, 4]]);
}

#[test]
fn transitive_depth_derived_from_coordinate() {
  assert_eq!(TransitivePhext::new().depth(), 9);
  assert_eq!(libphext::stores::coordinate_depth(), 9);
}

#[test]
fn empty_text_semantics_match_normalized_string_path() {
  let c = coord([1, 1, 1, 1, 1, 1, 1, 1, 1]);
  let mut d = DynamicPhext::new();
  let mut t = TransitivePhext::new();
  d.insert(c, ""); t.insert(c, "");
  assert_eq!(d.serialize(), "");
  assert_eq!(t.serialize(), "");
  d.insert(c, "body"); t.insert(c, "body");
  d.replace(c, ""); t.replace(c, "");
  assert_eq!(d.serialize(), phext::remove(&phext::insert(String::new(), c, "body"), c));
  assert_eq!(t.serialize(), "");
}
