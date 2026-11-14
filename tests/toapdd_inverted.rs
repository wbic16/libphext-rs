// TOAPDD — Knuth TAOCP Vol. 3, §6.5 Retrieval on secondary keys / inverted files
// Slug: inverted
//
// Phext-native claim:
//   Term = scroll.  The scroll's content is its posting list (one record id
//   per line, in arrival order).  Appending a posting is
//       old = fetch(index, term_coord)
//       index = replace(index, term_coord, old + "\n" + new_posting)
//   — a single-scroll update.  No other term scroll is rebuilt, moved, or
//   rewritten.  A secondary-key lookup is exactly fetch(index, term_coord).
//
// Public API only: libphext::phext::{fetch, replace, explode, phokenize, to_coordinate}

use libphext::phext;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Term coordinates: each term lives at its own found address.
// Only the scroll component varies in this fixture so the terms are
// neighbours — distinct scrolls in one section.
// ---------------------------------------------------------------------------
fn coord(addr: &str) -> phext::Coordinate {
    phext::to_coordinate(addr)
}
fn phext_term() -> phext::Coordinate {
    coord("1.1.1/1.1.1/1.1.1")
}
fn retrieval_term() -> phext::Coordinate {
    coord("1.1.1/1.1.1/1.1.2")
}
fn consciousness_term() -> phext::Coordinate {
    coord("1.1.1/1.1.1/1.1.3")
}
fn memory_term() -> phext::Coordinate {
    coord("1.1.1/1.1.1/1.1.4")
}
fn absent_term() -> phext::Coordinate {
    coord("1.1.1/1.1.1/1.1.9")
}

fn term_coord(term: &str) -> phext::Coordinate {
    match term {
        "phext" => phext_term(),
        "retrieval" => retrieval_term(),
        "consciousness" => consciousness_term(),
        "memory" => memory_term(),
        other => panic!("unmapped term in fixture: {other}"),
    }
}

/// Append one posting to a term scroll: fetch old, replace with old+new.
/// This is the whole update — there is no index rebuild anywhere.
fn append_posting(index: &str, term: &str, posting: &str) -> String {
    let c = term_coord(term);
    let old = phext::fetch(index, c);
    let new = if old.is_empty() {
        posting.to_string()
    } else {
        format!("{old}\n{posting}")
    };
    phext::replace(index, c, &new)
}

/// Append every term of one record, as records are written.
fn index_record(index: &str, record_id: &str, terms: &[&str]) -> String {
    let mut cur = index.to_string();
    for t in terms {
        cur = append_posting(&cur, t, record_id);
    }
    cur
}

/// Secondary-key lookup: a coordinate fetch, nothing else.
fn lookup(index: &str, term: &str) -> String {
    phext::fetch(index, term_coord(term))
}

fn postings(index: &str, term: &str) -> Vec<String> {
    let s = lookup(index, term);
    if s.is_empty() {
        Vec::new()
    } else {
        s.lines().map(|l| l.to_string()).collect()
    }
}

/// Non-empty scrolls only (phokenize("") yields one empty scroll — filter it).
fn nonempty_scrolls(index: &str) -> HashMap<phext::Coordinate, String> {
    phext::explode(index)
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .collect()
}

fn build_three_records() -> String {
    let mut idx = String::new();
    idx = index_record(&idx, "rec-001", &["phext", "retrieval"]);
    idx = index_record(&idx, "rec-002", &["phext", "consciousness"]);
    idx = index_record(&idx, "rec-003", &["retrieval", "consciousness"]);
    idx
}

// ---------------------------------------------------------------------------
// 1. Term = scroll; secondary-key lookup = fetch.
// ---------------------------------------------------------------------------
#[test]
fn term_is_a_scroll_and_lookup_is_fetch() {
    let idx = build_three_records();

    assert_eq!(lookup(&idx, "phext"), "rec-001\nrec-002");
    assert_eq!(lookup(&idx, "retrieval"), "rec-001\nrec-003");
    assert_eq!(lookup(&idx, "consciousness"), "rec-002\nrec-003");

    // Lookup is a pure fetch: repeating it does not change the index string.
    let before = idx.clone();
    let _ = lookup(&idx, "phext");
    let _ = lookup(&idx, "consciousness");
    assert_eq!(idx, before, "fetch lookups must not mutate the index");

    // Each term is exactly one scroll in the exploded map.
    let map = nonempty_scrolls(&idx);
    assert_eq!(map.len(), 3, "three terms = three scrolls, got {map:?}");
    assert_eq!(map.get(&phext_term()).map(|s| s.as_str()), Some("rec-001\nrec-002"));
    assert_eq!(map.get(&retrieval_term()).map(|s| s.as_str()), Some("rec-001\nrec-003"));
    assert_eq!(map.get(&consciousness_term()).map(|s| s.as_str()), Some("rec-002\nrec-003"));
}

// ---------------------------------------------------------------------------
// 2. Postings append per record, in arrival order, without a rebuild.
// ---------------------------------------------------------------------------
#[test]
fn postings_append_per_record_in_arrival_order() {
    let mut idx = String::new();
    assert_eq!(lookup(&idx, "phext"), "");

    idx = index_record(&idx, "rec-001", &["phext"]);
    assert_eq!(postings(&idx, "phext"), vec!["rec-001"]);

    let after_first = lookup(&idx, "phext");
    idx = index_record(&idx, "rec-002", &["phext"]);
    let after_second = lookup(&idx, "phext");

    // Append-only growth: the new content is the old content plus one line.
    assert_eq!(after_second, format!("{after_first}\nrec-002"));
    assert!(after_second.starts_with(&after_first));

    idx = index_record(&idx, "rec-004", &["phext"]);
    assert_eq!(postings(&idx, "phext"), vec!["rec-001", "rec-002", "rec-004"]);
}

// ---------------------------------------------------------------------------
// 3. Core §6.5 claim: appending to one term leaves every other term
//    byte-identical (fetch before/after + whole-map explode comparison).
// ---------------------------------------------------------------------------
#[test]
fn appending_to_one_term_leaves_other_terms_byte_identical() {
    let idx = build_three_records();
    let before_map = nonempty_scrolls(&idx);
    let before_phext = lookup(&idx, "phext");
    let before_consciousness = lookup(&idx, "consciousness");
    let before_retrieval = lookup(&idx, "retrieval");

    // One new record mentioning only "retrieval".
    let idx2 = index_record(&idx, "rec-004", &["retrieval"]);

    // The touched term grew by exactly one appended posting.
    assert_eq!(lookup(&idx2, "retrieval"), format!("{before_retrieval}\nrec-004"));

    // Every other term is byte-identical by fetch...
    assert_eq!(lookup(&idx2, "phext"), before_phext);
    assert_eq!(lookup(&idx2, "consciousness"), before_consciousness);

    // ...and by explode: same key set, same values except the touched term.
    let after_map = nonempty_scrolls(&idx2);
    assert_eq!(after_map.len(), before_map.len(), "no scroll created or lost");
    for (c, v) in &before_map {
        let after = after_map.get(c).expect("term scroll vanished");
        if *c == retrieval_term() {
            assert_eq!(after, &format!("{v}\nrec-004"));
        } else {
            assert_eq!(after, v, "untouched term at {c} changed");
        }
    }
}

// ---------------------------------------------------------------------------
// 4. A later record re-uses existing terms and still touches only its terms.
// ---------------------------------------------------------------------------
#[test]
fn later_record_touches_only_its_own_terms() {
    let idx = build_three_records();
    let before = nonempty_scrolls(&idx);

    let idx2 = index_record(&idx, "rec-005", &["phext", "consciousness"]);
    let after = nonempty_scrolls(&idx2);

    assert_eq!(after.len(), before.len());
    // retrieval was not in rec-005: byte-identical.
    assert_eq!(lookup(&idx2, "retrieval"), lookup(&idx, "retrieval"));
    // The two named terms each gained exactly that one posting, appended last.
    assert_eq!(postings(&idx2, "phext"), vec!["rec-001", "rec-002", "rec-005"]);
    assert_eq!(
        postings(&idx2, "consciousness"),
        vec!["rec-002", "rec-003", "rec-005"]
    );
}

// ---------------------------------------------------------------------------
// 5. A brand-new term is born as one new scroll; the old index is untouched.
//    Knuth's batch rebuild is replaced by incremental birth.
// ---------------------------------------------------------------------------
#[test]
fn new_term_births_one_scroll_without_touching_existing_terms() {
    let idx = build_three_records();
    let before = nonempty_scrolls(&idx);
    assert_eq!(before.len(), 3);
    assert_eq!(phext::fetch(&idx, memory_term()), "");

    let idx2 = index_record(&idx, "rec-006", &["memory"]);
    let after = nonempty_scrolls(&idx2);

    assert_eq!(after.len(), 4, "exactly one new scroll for the new term");
    assert_eq!(lookup(&idx2, "memory"), "rec-006");
    for (c, v) in &before {
        assert_eq!(after.get(c), Some(v), "existing term at {c} changed");
    }
}

// ---------------------------------------------------------------------------
// 6. Absent secondary key: lookup returns empty; no scroll is conjured.
// ---------------------------------------------------------------------------
#[test]
fn absent_term_lookup_is_empty_and_creates_nothing() {
    let idx = build_three_records();
    assert_eq!(phext::fetch(&idx, absent_term()), "");
    assert_eq!(lookup(&idx, "memory"), "");
    assert_eq!(nonempty_scrolls(&idx).len(), 3);

    // phokenize agrees, once empty scrolls are filtered out.
    let live = phext::phokenize(&idx)
        .into_iter()
        .filter(|ps| !ps.scroll.is_empty())
        .count();
    assert_eq!(live, 3);
}

// ---------------------------------------------------------------------------
// 7. Field-qualified postings (§6.5, per field): the same append mechanism
//    carries "record:field" postings, so a term can say where it occurred.
// ---------------------------------------------------------------------------
#[test]
fn field_qualified_postings_append_the_same_way() {
    let mut idx = String::new();
    idx = append_posting(&idx, "phext", "rec-001:title");
    idx = append_posting(&idx, "phext", "rec-002:body");
    idx = append_posting(&idx, "phext", "rec-001:body");

    assert_eq!(
        postings(&idx, "phext"),
        vec!["rec-001:title", "rec-002:body", "rec-001:body"]
    );
    // A caller can recover the distinct records for the term from postings.
    let mut records: Vec<String> = postings(&idx, "phext")
        .iter()
        .map(|p| p.split(':').next().unwrap().to_string())
        .collect();
    records.sort();
    records.dedup();
    assert_eq!(records, vec!["rec-001", "rec-002"]);
}
