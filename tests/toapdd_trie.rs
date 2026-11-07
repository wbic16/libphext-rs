//! TOAPDD #4 — Knuth TAOCP Vol. 3, §6.3 Digital searching / tries.
//!
//! Phext-native claim (ASSIGNMENTS.md, Wave 1 #4):
//!   Keys sharing a prefix share a coordinate prefix; insert is leaf
//!   creation (no other scroll moves); prefix fetch via
//!   phokenize/next_scroll enumerates exactly the keys under the prefix.
//!
//! ENCODING (documented, this test is its specification)
//! -----------------------------------------------------
//! A phext Coordinate has exactly nine components, in decreasing
//! significance order:
//!   [library, shelf, series, collection, volume, book, chapter, section, scroll]
//! Key character i (0-based) is stored in component i, so the first
//! character is the most significant component and coordinate order
//! is exactly lexicographic key order. A lowercase letter 'a'..='z'
//! maps to component value 2..=27 (letter position + 1). The value 1
//! is reserved as the end-of-key sentinel: every component past the
//! end of the key stays 1. Reserving 1 matters — with a naive
//! a=1 encoding, the key "a" and the key "aa" would collide at the
//! same coordinate; with the +1 offset, a key and any proper
//! extension of it always live at distinct coordinates, and a key
//! that is a prefix of another sorts strictly before it.
//!
//! Consequences, all exercised below:
//!   * shared key prefix <=> shared coordinate prefix (same leading
//!     components, literally — readable without decoding anything);
//!   * the scroll AT the full-key coordinate holds the value; trie
//!     interior nodes need no scrolls at all, they are just shared
//!     coordinate prefixes;
//!   * DEPTH CAP: nine components cap key length at 9 characters
//!     for this encoding. A 10th character has nowhere to go, so
//!     `encode_key` refuses it (panics) instead of truncating —
//!     truncation would silently collide distinct long keys.
//!     This is an honest limit of materialising a trie path as one
//!     phext coordinate, recorded in RESULT.md.
//!
//! Public API only: libphext::phext::{insert, fetch, phokenize,
//! next_scroll, explode, to_coordinate, Coordinate}.

use libphext::phext;
use std::collections::HashMap;

/// Components of a coordinate in decreasing significance order.
fn components(coord: &phext::Coordinate) -> [usize; 9] {
    [
        coord.z.library,
        coord.z.shelf,
        coord.z.series,
        coord.y.collection,
        coord.y.volume,
        coord.y.book,
        coord.x.chapter,
        coord.x.section,
        coord.x.scroll,
    ]
}

fn coordinate_from_components(c: [usize; 9]) -> phext::Coordinate {
    phext::to_coordinate(&format!(
        "{}.{}.{}/{}.{}.{}/{}.{}.{}",
        c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7], c[8]
    ))
}

/// Encode a lowercase a-z key (1..=9 chars) as a trie-path coordinate.
fn encode_key(key: &str) -> phext::Coordinate {
    assert!(
        !key.is_empty() && key.len() <= 9,
        "trie depth: keys must be 1..=9 chars — nine coordinate components cap trie depth"
    );
    assert!(
        key.bytes().all(|b| b.is_ascii_lowercase()),
        "this encoding covers lowercase a-z keys only"
    );
    let mut c = [1usize; 9]; // 1 = end-of-key sentinel in every unused slot
    for (i, b) in key.bytes().enumerate() {
        c[i] = (b - b'a') as usize + 2; // 'a'->2 .. 'z'->27
    }
    coordinate_from_components(c)
}

/// Decode a trie-path coordinate back to its key: read components in
/// significance order until the sentinel value 1.
fn decode_key(coord: &phext::Coordinate) -> String {
    let mut key = String::new();
    for v in components(coord) {
        if v == 1 {
            break;
        }
        assert!((2..=27).contains(&v), "component {v} is not a letter code");
        key.push((b'a' + (v - 2) as u8) as char);
    }
    key
}

/// The leading components a prefix constrains: encode the prefix's
/// characters, ignore everything after (sentinel region).
fn prefix_components(prefix: &str) -> Vec<usize> {
    assert!(prefix.len() <= 9, "trie depth: prefix longer than 9 chars");
    prefix
        .bytes()
        .map(|b| {
            assert!(b.is_ascii_lowercase(), "lowercase a-z only");
            (b - b'a') as usize + 2
        })
        .collect()
}

fn has_prefix(coord: &phext::Coordinate, prefix: &[usize]) -> bool {
    let c = components(coord);
    c[..prefix.len()] == prefix[..]
}

/// Build a trie phext by inserting keys in ascending key order
/// (== ascending coordinate order under this encoding).
fn build_trie(pairs: &[(&str, &str)]) -> String {
    let mut sorted: Vec<(&str, &str)> = pairs.to_vec();
    sorted.sort_by_key(|(k, _)| *k);
    let mut doc = String::new();
    for (key, value) in sorted {
        doc = phext::insert(doc, encode_key(key), value);
    }
    doc
}

/// All (key, value) leaves of a trie phext, in coordinate order,
/// via phokenize. Empty scrolls (e.g. phokenize("")'s lone scroll)
/// are not keys and are filtered out.
fn enumerate(doc: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = phext::phokenize(doc)
        .into_iter()
        .filter(|ps| !ps.scroll.is_empty())
        .map(|ps| (decode_key(&ps.coord), ps.scroll))
        .collect();
    out.sort();
    out
}

/// Prefix fetch via phokenize: exactly the keys under `prefix`.
fn keys_under_prefix(doc: &str, prefix: &str) -> Vec<String> {
    let pc = prefix_components(prefix);
    let mut out: Vec<String> = phext::phokenize(doc)
        .into_iter()
        .filter(|ps| !ps.scroll.is_empty())
        .filter(|ps| has_prefix(&ps.coord, &pc))
        .map(|ps| decode_key(&ps.coord))
        .collect();
    out.sort();
    out
}

const FIXTURE: &[(&str, &str)] = &[
    ("car", "value:car"),
    ("card", "value:card"),
    ("care", "value:care"),
    ("cart", "value:cart"),
    ("cat", "value:cat"),
    ("dog", "value:dog"),
    ("dot", "value:dot"),
    ("tree", "value:tree"),
    ("trie", "value:trie"),
    ("try", "value:try"),
];

#[test]
fn shared_prefix_is_shared_coordinate_prefix() {
    // Roundtrip: every fixture key survives encode -> decode.
    for (key, _) in FIXTURE {
        assert_eq!(decode_key(&encode_key(key)), *key);
    }
    // A key that is a prefix of another still gets its own coordinate.
    assert_ne!(encode_key("a"), encode_key("aa"));
    assert!(encode_key("car") < encode_key("card"));

    // "car" family: first three components identical, literally.
    let car = components(&encode_key("car"));
    for key in ["card", "care", "cart"] {
        let c = components(&encode_key(key));
        assert_eq!(c[..3], car[..3], "{key} must share car's 3-component prefix");
        assert_ne!(c[3], 1, "{key} extends past the prefix");
    }
    assert_eq!(car[3], 1, "car itself ends at depth 3 (sentinel)");

    // "cat" shares only the first two components with "car".
    let cat = components(&encode_key("cat"));
    assert_eq!(cat[..2], car[..2]);
    assert_ne!(cat[2], car[2]);

    // "dog" shares no component with "car".
    let dog = components(&encode_key("dog"));
    assert_ne!(dog[0], car[0]);
}

#[test]
fn exact_fetch_by_full_key_and_absent_keys() {
    let doc = build_trie(FIXTURE);
    for (key, value) in FIXTURE {
        assert_eq!(phext::fetch(&doc, encode_key(key)), *value, "fetch {key}");
    }
    // Absent keys — a shared-prefix non-key, an extension of a real
    // key, and a key under an unused branch — fetch as empty.
    for absent in ["cap", "cards", "do", "trees", "zebra"] {
        assert_eq!(phext::fetch(&doc, encode_key(absent)), "", "absent {absent}");
    }
    // The full leaf set is exactly the fixture, in key order.
    let leaves = enumerate(&doc);
    let mut expected: Vec<(String, String)> =
        FIXTURE.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    expected.sort();
    assert_eq!(leaves, expected);
}

#[test]
fn insert_is_leaf_creation_no_other_scroll_moves() {
    // Base trie without "care".
    let base_pairs: Vec<(&str, &str)> =
        FIXTURE.iter().cloned().filter(|(k, _)| *k != "care").collect();
    let base = build_trie(&base_pairs);
    let before: HashMap<phext::Coordinate, String> = phext::explode(&base)
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .collect();
    assert_eq!(before.len(), 9);

    // Insert one new leaf, in the middle of the coordinate range
    // (card < care < cart). This is the whole update.
    let after_doc = phext::insert(base, encode_key("care"), "value:care");
    let after: HashMap<phext::Coordinate, String> = phext::explode(&after_doc)
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .collect();

    // Exactly one scroll was created...
    assert_eq!(after.len(), before.len() + 1);
    assert_eq!(
        after.get(&encode_key("care")).map(|s| s.as_str()),
        Some("value:care")
    );
    // ...and every pre-existing scroll is byte-identical at the
    // same coordinate: no rebalancing, no packed-array rewrite,
    // no other scroll moved.
    for (coord, value) in &before {
        assert_eq!(
            after.get(coord),
            Some(value),
            "scroll at {coord} changed during a leaf insert"
        );
        assert_eq!(phext::fetch(&after_doc, *coord), *value);
    }
}

#[test]
fn prefix_enumeration_via_phokenize_is_exact() {
    let doc = build_trie(FIXTURE);

    assert_eq!(keys_under_prefix(&doc, "car"), vec!["car", "card", "care", "cart"]);
    assert_eq!(
        keys_under_prefix(&doc, "ca"),
        vec!["car", "card", "care", "cart", "cat"]
    );
    assert_eq!(keys_under_prefix(&doc, "do"), vec!["dog", "dot"]);
    assert_eq!(keys_under_prefix(&doc, "tr"), vec!["tree", "trie", "try"]);
    assert_eq!(keys_under_prefix(&doc, "trie"), vec!["trie"]);
    // A prefix with no keys under it enumerates nothing — including
    // a prefix that extends a real key.
    assert_eq!(keys_under_prefix(&doc, "cards"), Vec::<String>::new());
    assert_eq!(keys_under_prefix(&doc, "z"), Vec::<String>::new());
    // The empty prefix enumerates the whole trie, sorted.
    let mut all: Vec<String> = FIXTURE.iter().map(|(k, _)| k.to_string()).collect();
    all.sort();
    assert_eq!(keys_under_prefix(&doc, ""), all);
}

#[test]
fn prefix_enumeration_via_next_scroll_walk_is_exact() {
    let doc = build_trie(FIXTURE);

    // Walk the raw phext with next_scroll: each call returns the
    // next positioned scroll, the coordinate after it, and the
    // unparsed remainder to feed back in.
    let mut walked: Vec<(String, String)> = Vec::new();
    let mut coord = phext::to_coordinate("1.1.1/1.1.1/1.1.1");
    let mut rest = doc.clone();
    loop {
        let (ps, next_coord, remaining) = phext::next_scroll(&rest, coord);
        if !ps.scroll.is_empty() {
            walked.push((decode_key(&ps.coord), ps.scroll.clone()));
        }
        coord = next_coord;
        let done = remaining.is_empty();
        rest = remaining;
        if done {
            break;
        }
    }
    walked.sort();
    assert_eq!(walked, enumerate(&doc), "next_scroll walk must match phokenize");

    // Filtering the walk by a coordinate prefix gives exactly the
    // keys under that prefix.
    let pc = prefix_components("car");
    let under: Vec<String> = walked
        .iter()
        .map(|(k, _)| k.clone())
        .filter(|k| has_prefix(&encode_key(k), &pc))
        .collect();
    assert_eq!(under, vec!["car", "card", "care", "cart"]);
}

#[test]
fn nine_char_key_uses_all_nine_components() {
    // The deepest key this encoding can hold: one char per component.
    let deep = encode_key("abcdefghi");
    assert!(components(&deep).iter().all(|v| *v != 1));
    assert_eq!(decode_key(&deep), "abcdefghi");

    let doc = build_trie(&[("abcdefghi", "value:deep"), ("abc", "value:abc")]);
    assert_eq!(phext::fetch(&doc, deep), "value:deep");
    assert_eq!(keys_under_prefix(&doc, "abc"), vec!["abc", "abcdefghi"]);
}

#[test]
#[should_panic(expected = "trie depth")]
fn tenth_character_is_refused_not_truncated() {
    // A 10-char key would need a 10th component. Refusing it is the
    // honest behaviour: silently dropping the last char would make
    // "abcdefghij" and "abcdefghix" collide at one coordinate.
    let _ = encode_key("abcdefghij");
}
