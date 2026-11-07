// TOAPDD #1 — Literate Programming / WEB (Knuth, Literate Programming 1992, Ch. 4 & 7)
// Phext-native claim: a WEB source is named chunks in human order, reordered
// mechanically into compiler order (tangle) and document order (weive).
// In phext there is no preprocessor and no second file: each named chunk IS a
// scroll at its own coordinate, and tangle / weave are two different
// traversals of the SAME explode map. Editing one chunk changes exactly one
// scroll (mutation control via subtract + checksum).
//
// Public API only: libphext::phext.

use libphext::phext;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Fixture: a tiny WEB-style program, six named chunks.
// Weave (document) order is coordinate order: prose and code interleaved the
// way a human reads it. Tangle (compiler) order is root-first, code-only,
// following the chunk references inside the root chunk — a different order
// over the same scrolls.
// ---------------------------------------------------------------------------

fn coord(s: &str) -> phext::Coordinate {
    phext::to_coordinate(s)
}

fn title_c() -> phext::Coordinate { coord("1.1.1/1.1.1/1.1.1") }
fn decl_c() -> phext::Coordinate { coord("1.1.1/1.1.1/1.2.1") }
fn prose_c() -> phext::Coordinate { coord("1.1.1/1.1.1/1.3.1") }
fn compute_c() -> phext::Coordinate { coord("1.1.1/1.1.1/1.4.1") }
fn output_c() -> phext::Coordinate { coord("1.1.1/1.1.1/1.5.1") }
fn root_c() -> phext::Coordinate { coord("1.1.1/1.1.1/1.6.1") }

const TITLE: &str = "CHUNK <title>: Program Hello — count to three and report the total.";
const DECL: &str = "CHUNK <decl>: let mut total: i64 = 0;";
const PROSE: &str = "CHUNK <why compute>: We add one, two, and three; the explanation lives beside the code it explains.";
const COMPUTE: &str = "CHUNK <compute>: for n in 1..=3 { total += n; }";
const OUTPUT: &str = "CHUNK <output>: println!(\"total={}\", total);";
const ROOT: &str = "CHUNK <root>: fn main() { <decl> <compute> <output> }";

fn web_source() -> String {
    let mut map: HashMap<phext::Coordinate, String> = HashMap::new();
    map.insert(title_c(), TITLE.to_string());
    map.insert(decl_c(), DECL.to_string());
    map.insert(prose_c(), PROSE.to_string());
    map.insert(compute_c(), COMPUTE.to_string());
    map.insert(output_c(), OUTPUT.to_string());
    map.insert(root_c(), ROOT.to_string());
    phext::implode(map)
}

/// Non-empty scrolls of a phext, in native coordinate (phokenize) order.
fn scrolls_in_order(source: &str) -> Vec<(phext::Coordinate, String)> {
    phext::phokenize(source)
        .into_iter()
        .filter(|ps| !ps.scroll.is_empty())
        .map(|ps| (ps.coord, ps.scroll))
        .collect()
}

// ---------------------------------------------------------------------------
// 1. Named chunks are scrolls: the explode map is the WEB source.
// ---------------------------------------------------------------------------
#[test]
fn named_chunks_are_scrolls_in_the_explode_map() {
    let source = web_source();
    let map = phext::explode(&source);

    let nonempty: HashMap<_, _> = map
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (*k, v.clone()))
        .collect();

    assert_eq!(nonempty.len(), 6, "six named chunks = six scrolls, got {:?}", nonempty.keys().collect::<Vec<_>>());

    // Every chunk is fetchable by its own coordinate, byte for byte.
    assert_eq!(phext::fetch(&source, title_c()), TITLE);
    assert_eq!(phext::fetch(&source, decl_c()), DECL);
    assert_eq!(phext::fetch(&source, prose_c()), PROSE);
    assert_eq!(phext::fetch(&source, compute_c()), COMPUTE);
    assert_eq!(phext::fetch(&source, output_c()), OUTPUT);
    assert_eq!(phext::fetch(&source, root_c()), ROOT);

    // Round-trip: explode -> implode -> explode changes nothing.
    let rebuilt = phext::implode(map.clone());
    assert_eq!(phext::explode(&rebuilt), map);
}

// ---------------------------------------------------------------------------
// 2. Tangle order and weave order are two traversals of the SAME map.
//    No preprocessor, no tangled intermediate file: both readings consume
//    the identical source string, and reading does not modify it.
// ---------------------------------------------------------------------------
#[test]
fn tangle_and_weave_are_two_traversals_of_the_same_map() {
    let source = web_source();
    let before = phext::checksum(&source);
    let map = phext::explode(&source);

    // WEAVE: native coordinate traversal — prose and code interleaved,
    // exactly the order a reader meets them.
    let weave: Vec<String> = scrolls_in_order(&source).into_iter().map(|(_, s)| s).collect();
    assert_eq!(
        weave,
        vec![TITLE, DECL, PROSE, COMPUTE, OUTPUT, ROOT],
        "weave order is coordinate order over the map"
    );

    // TANGLE: follow the references inside <root> — root first, code only,
    // prose chunks are not part of the compiled program.
    let tangle_order = vec![root_c(), decl_c(), compute_c(), output_c()];
    let tangle: Vec<String> = tangle_order
        .iter()
        .map(|c| map.get(c).expect("tangle target must be a chunk in the same map").clone())
        .collect();
    assert_eq!(tangle, vec![ROOT, DECL, COMPUTE, OUTPUT]);

    // The two traversals genuinely differ (order and membership) ...
    assert_ne!(weave, tangle);
    assert!(weave.len() > tangle.len(), "weave carries the prose tangle drops");

    // ... yet every tangled chunk is verbatim a scroll of the one source,
    // fetched from the same string the weave read.
    for c in &tangle_order {
        assert_eq!(phext::fetch(&source, *c), *map.get(c).unwrap());
    }

    // Traversal is read-only: no preprocessor pass rewrote the source.
    assert_eq!(phext::checksum(&source), before);
    assert_eq!(phext::explode(&source), map);
}

// ---------------------------------------------------------------------------
// 3. Mutation control: editing one chunk changes exactly one scroll.
//    subtract() is used as its docs intend — a coordinate mask — to isolate
//    the changed scroll; checksum() proves the whole changed while the
//    per-scroll comparison proves nothing else did.
// ---------------------------------------------------------------------------
#[test]
fn editing_one_chunk_changes_exactly_one_scroll() {
    let original = web_source();
    let edited_text = "CHUNK <compute>: for n in 1..=4 { total += n; }";
    let edited = phext::replace(&original, compute_c(), edited_text);

    // The whole document's fingerprint moves ...
    assert_ne!(phext::checksum(&original), phext::checksum(&edited));

    // ... but the explode maps differ at exactly one coordinate.
    let before_map = phext::explode(&original);
    let after_map = phext::explode(&edited);
    assert_eq!(before_map.len(), after_map.len(), "no scroll was born or destroyed");
    let mut changed: Vec<phext::Coordinate> = Vec::new();
    for (c, v) in &before_map {
        let after_v = after_map.get(c).expect("every original coordinate survives an edit");
        if after_v != v {
            changed.push(*c);
        }
    }
    assert_eq!(changed, vec![compute_c()], "exactly the edited chunk changed");
    assert_eq!(phext::fetch(&edited, compute_c()), edited_text);

    // subtract as mask: masking the edited phext by every OTHER original
    // scroll leaves exactly the edited chunk, carrying its new text and its
    // original coordinate (per-chunk provenance survives the edit).
    let mut others: HashMap<phext::Coordinate, String> = HashMap::new();
    for (c, v) in &before_map {
        if *c != compute_c() && !v.is_empty() {
            others.insert(*c, v.clone());
        }
    }
    let mask = phext::implode(others);
    let isolated = phext::subtract(&edited, &mask);
    let isolated_scrolls = scrolls_in_order(&isolated);
    assert_eq!(isolated_scrolls.len(), 1, "subtract isolates the single changed scroll, got {:?}", isolated_scrolls);
    assert_eq!(isolated_scrolls[0].0, compute_c());
    assert_eq!(isolated_scrolls[0].1, edited_text);

    // And masking the original the same way recovers the pre-edit chunk:
    // both versions are addressable at the same coordinate.
    let isolated_before = phext::subtract(&original, &mask);
    let before_scrolls = scrolls_in_order(&isolated_before);
    assert_eq!(before_scrolls.len(), 1);
    assert_eq!(before_scrolls[0].0, compute_c());
    assert_eq!(before_scrolls[0].1, COMPUTE);
}
