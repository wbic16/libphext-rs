// TOAPDD #15 — Optimum binary search trees (§6.2.2) + binary search (§6.2.1, Algorithm B)
// Public API only: libphext::phext
//
// Scope, stated plainly: no optimum search tree is constructed in this
// file. What §6.2.2 contributes here is its premise — shape should follow
// use — carried as data: access counts are scrolls, updated by fetch +
// replace on the count scroll alone, so placement advice stays revisable
// without rebuilding anything (test 3 pins exactly which scrolls change).
// The search itself is Knuth's Algorithm B re-implemented as a driver
// over fetch() (test 1), and test 2 pins offset()'s real semantics
// against the raw serialized string.
//
// Design:
//  * Key records: one scroll per key, at successive coordinates
//    1.1.1/1.1.1/1.1.<rank+1>  (chapter 1, section 1, scroll = rank+1).
//    The scroll index IS the rank: keys are stored in sorted order.
//  * Access counts: a parallel region, one count scroll per key, at
//    1.1.1/1.1.1/1.2.<rank+1>  (chapter 1, section 2, scroll = rank+1).
//    Counting is fetch + replace on the count scroll only.
//  * Binary search lives in this driver: it probes fetch() at the mid-rank
//    coordinate, compares, and halves the rank range (Knuth's Algorithm B
//    over an ordered table, where the "table" is the sorted scroll range).
//  * offset() is exercised against its actual semantics (see test 2).

use libphext::phext;
use std::collections::HashMap;

const KEYS: [&str; 16] = [
    "apple", "birch", "cedar", "delta", "ember", "flint", "glade", "harbor",
    "iris", "juniper", "kestrel", "linden", "maple", "north", "onyx", "pine",
];

fn key_coord(rank: usize) -> phext::Coordinate {
    phext::to_coordinate(&format!("1.1.1/1.1.1/1.1.{}", rank + 1))
}

fn count_coord(rank: usize) -> phext::Coordinate {
    phext::to_coordinate(&format!("1.1.1/1.1.1/1.2.{}", rank + 1))
}

fn build_keys_only() -> String {
    let mut doc = String::new();
    for (rank, key) in KEYS.iter().enumerate() {
        doc = phext::insert(doc, key_coord(rank), key);
    }
    doc
}

fn build_store() -> String {
    let mut doc = build_keys_only();
    for rank in 0..KEYS.len() {
        doc = phext::insert(doc, count_coord(rank), "0");
    }
    doc
}

/// Algorithm B over ranks: returns (rank if found, number of fetch probes).
fn binary_search(doc: &str, target: &str) -> (Option<usize>, usize) {
    let mut lo: i64 = 0;
    let mut hi: i64 = KEYS.len() as i64 - 1;
    let mut probes: usize = 0;
    while lo <= hi {
        let mid = ((lo + hi) / 2) as usize;
        probes += 1;
        let value = phext::fetch(doc, key_coord(mid));
        match target.cmp(value.as_str()) {
            std::cmp::Ordering::Equal => return (Some(mid), probes),
            std::cmp::Ordering::Less => hi = mid as i64 - 1,
            std::cmp::Ordering::Greater => lo = mid as i64 + 1,
        }
    }
    (None, probes)
}

fn probe_bound(n: usize) -> usize {
    // ceil(log2(n)) + 1
    let mut p = 0usize;
    let mut v = 1usize;
    while v < n {
        v *= 2;
        p += 1;
    }
    p + 1
}

#[test]
fn binary_search_finds_every_key_within_log_bound_and_reports_missing() {
    let doc = build_store();

    // sanity: the store really holds the keys in sorted rank order
    for (rank, key) in KEYS.iter().enumerate() {
        assert_eq!(phext::fetch(&doc, key_coord(rank)), *key, "rank {}", rank);
    }

    let bound = probe_bound(KEYS.len());
    assert_eq!(bound, 5, "ceil(log2(16))+1");

    for (rank, key) in KEYS.iter().enumerate() {
        let (found, probes) = binary_search(&doc, key);
        assert_eq!(found, Some(rank), "key {}", key);
        assert!(
            probes <= bound,
            "key {} took {} probes, bound {}",
            key, probes, bound
        );
    }

    // missing keys: one inside the range, one past the end
    for missing in ["falcon", "quartz"] {
        assert!(!KEYS.contains(&missing));
        let (found, probes) = binary_search(&doc, missing);
        assert_eq!(found, None, "missing key {}", missing);
        assert!(
            probes <= bound,
            "missing {} took {} probes, bound {}",
            missing, probes, bound
        );
    }
}

#[test]
fn offset_is_the_byte_position_where_each_key_scroll_begins() {
    // offset() in libphext-rs 0.4.0 is derived from index(): index_phokens
    // walks the coordinate breaks needed to reach each scroll, counting
    // 1 per break byte, and records the running total before adding that
    // scroll's byte length. For scrolls at successive scroll coordinates
    // (one 1-byte SCROLL_BREAK between neighbours) that running total is
    // exactly the byte offset at which the scroll's text begins in the
    // serialized phext string. This test asserts THAT, directly, by slicing.
    let doc = build_keys_only();

    let mut expected: usize = 0;
    let mut previous: Option<usize> = None;
    for (rank, key) in KEYS.iter().enumerate() {
        let off = phext::offset(&doc, key_coord(rank));
        assert_eq!(off, expected, "offset of rank {} ({})", rank, key);
        if let Some(prev) = previous {
            assert!(off > prev, "offsets strictly increasing with rank");
        }
        previous = Some(off);
        // the scroll's text begins exactly at offset() in the raw string
        assert!(
            doc[off..].starts_with(key),
            "slice at offset {} does not start with {:?}: {:?}",
            off, key, &doc[off..off + key.len().min(doc.len() - off)]
        );
        assert_eq!(&doc[off..off + key.len()], *key);
        expected += key.len() + 1; // +1 for the single SCROLL_BREAK byte
    }
    assert_eq!(phext::offset(&doc, key_coord(0)), 0);
}

#[test]
fn access_counts_are_scrolls_hot_key_ranking_exact_and_keys_never_move() {
    let mut doc = build_store();

    // snapshot before counting: per-key fetch, whole-store explode map,
    // and the byte offset of every key scroll.
    let keys_before: Vec<String> = (0..KEYS.len())
        .map(|r| phext::fetch(&doc, key_coord(r)))
        .collect();
    let exploded_before: HashMap<phext::Coordinate, String> = phext::explode(&doc);
    let key_offsets_before: Vec<usize> = (0..KEYS.len())
        .map(|r| phext::offset(&doc, key_coord(r)))
        .collect();

    // Scripted access sequence, driven through the binary search itself:
    // distinct final frequencies so the hot-key ranking is exact.
    // (rank, times): pine x9, apple x7, harbor x5, cedar x3, onyx x1.
    let script: [(usize, usize); 5] = [(15, 9), (0, 7), (7, 5), (2, 3), (14, 1)];
    let mut expected_counts = vec![0usize; KEYS.len()];
    for (rank, times) in script {
        for _ in 0..times {
            let (found, _) = binary_search(&doc, KEYS[rank]);
            assert_eq!(found, Some(rank));
            let current: usize = phext::fetch(&doc, count_coord(rank))
                .parse()
                .expect("count scroll must hold a decimal count");
            doc = phext::replace(&doc, count_coord(rank), &(current + 1).to_string());
            expected_counts[rank] += 1;
        }
    }

    // Frequency traversal: phokenize over the count region (section 2),
    // paired with index() over the same region to prove the traversal is
    // over the indexed store, not a side table.
    let idx = phext::index(&doc);
    // index() yields one offset scroll per populated scroll; the count
    // region contributes exactly KEYS.len() of them.
    let count_region_in_index = phext::phokenize(&idx)
        .into_iter()
        .filter(|ps| ps.coord.x.section == 2)
        .count();
    assert_eq!(count_region_in_index, KEYS.len());

    let mut ranking: Vec<(usize, usize)> = phext::phokenize(&doc)
        .into_iter()
        .filter(|ps| ps.coord.x.section == 2 && !ps.scroll.is_empty())
        .map(|ps| {
            let rank = ps.coord.x.scroll - 1;
            let count: usize = ps.scroll.parse().expect("count scroll is decimal");
            (rank, count)
        })
        .collect();
    assert_eq!(ranking.len(), KEYS.len(), "one count scroll per key");
    // exact counts, rank by rank, before sorting
    let mut by_rank = ranking.clone();
    by_rank.sort_by_key(|(rank, _)| *rank);
    for (rank, count) in &by_rank {
        assert_eq!(*count, expected_counts[*rank], "count for {}", KEYS[*rank]);
    }
    // hot-key ranking: count descending, rank ascending as tie-break
    ranking.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let hot_names: Vec<&str> = ranking.iter().take(5).map(|(r, _)| KEYS[*r]).collect();
    assert_eq!(hot_names, vec!["pine", "apple", "harbor", "cedar", "onyx"]);
    assert_eq!(
        ranking.iter().take(5).map(|(_, c)| *c).collect::<Vec<_>>(),
        vec![9, 7, 5, 3, 1]
    );
    // full ranking is exactly the scripted frequency table
    let mut expected_ranking: Vec<(usize, usize)> = expected_counts
        .iter()
        .enumerate()
        .map(|(r, c)| (r, *c))
        .collect();
    expected_ranking.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    assert_eq!(ranking, expected_ranking);

    // Placement advice was data, not a rebuild: the key-record scrolls are
    // byte-identical, at the same byte offsets, after all the counting.
    for rank in 0..KEYS.len() {
        assert_eq!(phext::fetch(&doc, key_coord(rank)), keys_before[rank]);
        assert_eq!(
            phext::offset(&doc, key_coord(rank)),
            key_offsets_before[rank],
            "key {} moved during counting",
            KEYS[rank]
        );
    }
    // explode-diff: exactly the five touched count scrolls changed,
    // nothing else in the whole store.
    let exploded_after: HashMap<phext::Coordinate, String> = phext::explode(&doc);
    let mut changed: Vec<phext::Coordinate> = exploded_before
        .keys()
        .chain(exploded_after.keys())
        .copied()
        .collect();
    changed.sort();
    changed.dedup();
    let changed: Vec<phext::Coordinate> = changed
        .into_iter()
        .filter(|c| exploded_before.get(c) != exploded_after.get(c))
        .collect();
    let mut expected_changed: Vec<phext::Coordinate> =
        script.iter().map(|(r, _)| count_coord(*r)).collect();
    expected_changed.sort();
    assert_eq!(changed, expected_changed);
}
