//! Time the whole-document operations on a real phext.
//!
//! ```text
//! cargo run --release --example explode_bench -- big.phext
//! ```

use std::time::Instant;

use libphext::phext;

fn main() {
    let path = std::env::args().nth(1).expect("usage: explode_bench <file.phext>");
    let doc = std::fs::read_to_string(&path).expect("read phext");
    println!("{path}: {:.2} MB", doc.len() as f64 / 1_048_576.0);

    let start = Instant::now();
    let map = phext::explode(&doc);
    println!("explode   {:>10.1} ms  ({} scrolls)", ms(start), map.len());

    let start = Instant::now();
    let rebuilt = phext::implode(map);
    println!("implode   {:>10.1} ms  ({} bytes)", ms(start), rebuilt.len());

    let start = Instant::now();
    let toc = phext::textmap(&doc);
    println!("textmap   {:>10.1} ms  ({} lines)", ms(start), toc.lines().count());

    let start = Instant::now();
    let index = phext::index(&doc);
    println!("index     {:>10.1} ms  ({} bytes)", ms(start), index.len());
}

fn ms(start: Instant) -> f64 {
    start.elapsed().as_secs_f64() * 1000.0
}
