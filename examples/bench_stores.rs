//! v0.7.0 benchmark harness (DISCIPLINE.md workloads A/B/C).
//! Usage: bench_stores <corpus.bin> <targets.txt>
//! String path always; store paths between STORE markers (stripped for
//! the pristine BEFORE run).
use libphext::phext;
use libphext::phext::Coordinate;
//STORE_BEGIN
use libphext::stores::{DynamicPhext, TransitivePhext};
//STORE_END
use std::time::Instant;

fn coord(a: [usize; 9]) -> Coordinate {
  let mut c = phext::default_coordinate();
  c.z.library = a[0]; c.z.shelf = a[1]; c.z.series = a[2];
  c.y.collection = a[3]; c.y.volume = a[4]; c.y.book = a[5];
  c.x.chapter = a[6]; c.x.section = a[7]; c.x.scroll = a[8];
  c
}
fn coord_a(i: usize) -> Coordinate {
  coord([1, 1, 1, 1, 1, 1, 1, (i / 50) % 40 + 1, i % 50 + 1])
}

fn read_corpus(path: &str) -> Vec<String> {
  let bytes = std::fs::read(path).unwrap();
  let mut out = Vec::new();
  let mut pos = 0usize;
  while pos < bytes.len() {
    let mut end = pos;
    while bytes[end] != b'\n' { end += 1; }
    let len: usize = std::str::from_utf8(&bytes[pos..end]).unwrap().parse().unwrap();
    let start = end + 1;
    out.push(String::from_utf8(bytes[start..start + len].to_vec()).unwrap());
    pos = start + len;
  }
  out
}

fn main() {
  let args: Vec<String> = std::env::args().collect();
  let corpus = read_corpus(&args[1]);
  let targets_raw = std::fs::read_to_string(&args[2]).unwrap();
  let targets: Vec<Coordinate> = targets_raw.lines().map(|l| {
    let v: Vec<usize> = l.split(',').map(|x| x.parse().unwrap()).collect();
    let mut a = [0usize; 9]; a.copy_from_slice(&v); coord(a)
  }).collect();
  println!("corpus: {} texts, mean {:.1} chars", corpus.len(),
    corpus.iter().map(|t| t.len()).sum::<usize>() as f64 / corpus.len() as f64);
  let text = |i: usize| corpus[i % corpus.len()].as_str();

  // ---- Workload A: string path ----
  let t0 = Instant::now();
  let mut buf = String::new();
  for i in 0..2000 { buf = phext::insert(buf, coord_a(i), text(i)); }
  let a_string = t0.elapsed().as_secs_f64() * 1000.0;
  println!("A string: edits {:.1}ms (final buffer {} bytes)", a_string, buf.len());

  //STORE_BEGIN
  let t0 = Instant::now();
  let mut dyn_store = DynamicPhext::new();
  for i in 0..2000 { dyn_store.insert(coord_a(i), text(i)); }
  let a_dyn_edits = t0.elapsed().as_secs_f64() * 1000.0;
  let t0 = Instant::now();
  let dyn_ser = dyn_store.serialize();
  let a_dyn_ser = t0.elapsed().as_secs_f64() * 1000.0;
  assert_eq!(dyn_ser, buf, "dynamic serialize != string buffer (A)");
  println!("A dynamic: edits {:.1}ms + serialize {:.1}ms = {:.1}ms (resizes {}, copies {})",
    a_dyn_edits, a_dyn_ser, a_dyn_edits + a_dyn_ser, dyn_store.resize_count(), dyn_store.elements_copied());

  let t0 = Instant::now();
  let mut tr_store = TransitivePhext::new();
  for i in 0..2000 { tr_store.insert(coord_a(i), text(i)); }
  let a_tr_edits = t0.elapsed().as_secs_f64() * 1000.0;
  let t0 = Instant::now();
  let tr_ser = tr_store.serialize();
  let a_tr_ser = t0.elapsed().as_secs_f64() * 1000.0;
  assert_eq!(tr_ser, buf, "transitive serialize != string buffer (A)");
  println!("A transitive: edits {:.1}ms + serialize {:.1}ms = {:.1}ms",
    a_tr_edits, a_tr_ser, a_tr_edits + a_tr_ser);
  //STORE_END

  // ---- Workload B ----
  let mut base = String::new();
  for i in 0..256 { base = phext::insert(base, coord_a(i), text(i)); }
  let tail = coord([1, 1, 1, 1, 1, 1, 2, 1, 1]);
  let tail_text = "tail boundary scroll\nsecond line\n";
  let t0 = Instant::now();
  let mut b = base.clone();
  for _ in 0..500 {
    b = phext::insert(b, tail, tail_text);
    b = phext::remove(&b, tail);
  }
  let b_string = t0.elapsed().as_secs_f64() * 1000.0;
  assert_eq!(b, base, "string B end state != base");
  println!("B string: {:.1}ms", b_string);

  //STORE_BEGIN
  let t0 = Instant::now();
  let mut bd = DynamicPhext::with_buffer(&base);
  for _ in 0..500 { bd.insert(tail, tail_text); bd.remove(tail); }
  let b_dyn = t0.elapsed().as_secs_f64() * 1000.0;
  assert_eq!(bd.serialize(), base, "dynamic B end state != base");
  println!("B dynamic: {:.1}ms", b_dyn);

  let t0 = Instant::now();
  let mut bt = TransitivePhext::with_buffer(&base);
  for _ in 0..500 { bt.insert(tail, tail_text); bt.remove(tail); }
  let b_tr = t0.elapsed().as_secs_f64() * 1000.0;
  assert_eq!(bt.serialize(), base, "transitive B end state != base");
  println!("B transitive: {:.1}ms", b_tr);
  //STORE_END

  // ---- Workload C ----
  let t0 = Instant::now();
  let mut string_fetches: Vec<String> = Vec::with_capacity(targets.len());
  for t in &targets { string_fetches.push(phext::fetch(&buf, *t)); }
  let c_string = t0.elapsed().as_secs_f64() * 1000.0;
  println!("C string: {:.1}ms ({} fetches)", c_string, targets.len());

  //STORE_BEGIN
  let dyn_a = DynamicPhext::with_buffer(&buf);
  let t0 = Instant::now();
  for (i, t) in targets.iter().enumerate() {
    assert_eq!(dyn_a.fetch(*t), string_fetches[i], "dynamic fetch mismatch C");
  }
  println!("C dynamic: {:.1}ms", t0.elapsed().as_secs_f64() * 1000.0);

  let tr_a = TransitivePhext::with_buffer(&buf);
  let t0 = Instant::now();
  for (i, t) in targets.iter().enumerate() {
    assert_eq!(tr_a.fetch(*t), string_fetches[i], "transitive fetch mismatch C");
  }
  println!("C transitive: {:.1}ms", t0.elapsed().as_secs_f64() * 1000.0);
  //STORE_END
  println!("ALL ASSERTIONS PASSED");
}
