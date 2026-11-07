// TOAPDD attempt #14 — Knuth TAOCP Vol. 3, §6.2.3: Balanced trees.
//
// Phext-native claim to prove:
//   Insert 200 keys in strictly sorted order — the worst case for a naive
//   BST, which degenerates to a 200-deep chain, and the case balanced trees
//   exist to repair with rotations — into a phext STORE. The store never
//   rotates, never rebalances, and never moves: every record is written once
//   at a coordinate derived from its own rank, and every previously written
//   record fetches byte-identical at its original coordinate after every
//   later insertion. There is nothing to rebalance because a record's
//   identity is its coordinate, not its position in a balanced shape.
//
// Coordinate derivation (documented, found from the key's rank):
//   Keys are the 200 fixed strings "key-000" .. "key-199", in that order,
//   so key number r (rank, 0-based) is known before any insertion.
//   Store coordinate for rank r:
//        1.1.1 / 1.1.1 / 1.1.(r+1)
//   i.e. library/shelf/series, collection/volume/book, and
//   chapter/section all stay 1, and the scroll component is rank+1.
//   Coordinate components are unbounded usize, so 200 keys (and far more)
//   fit in the single scroll component; no packing, splitting, or node
//   fanout is needed. Document (coordinate) order is therefore exactly
//   sorted key order.
//
// Derived index (the contrast artefact):
//   A balanced tree is not discarded — it is demoted to a DERIVED index.
//   One index scroll lives at a fixed coordinate outside the store region,
//        2.1.1 / 1.1.1 / 1.1.1        (library 2)
//   Its content names the median (root) key of the keys present so far and
//   lists those keys in sorted order — the root pointer and shape summary
//   a balanced index over the same coordinates would maintain. The driver
//   REBUILDS that scroll with replace() at checkpoints. The test proves,
//   by explode-diff, that a rebuild changes only the index coordinate:
//   every store coordinate is byte-identical before and after.
//
// What was actually asserted, per insert (not just at checkpoints):
//   After EVERY one of the 200 insertions the full explode map is taken
//   and asserted to be exactly the previous map plus the one new entry
//   (no entry changed, no entry lost), AND every previously written key
//   is fetch()ed and asserted byte-identical. 200 explodes + ~20k fetches
//   on a small growing phext; no sampling was needed.

use libphext::phext::{self, Coordinate};
use std::collections::HashMap;

const N: usize = 200;

fn key(rank: usize) -> String {
    format!("key-{:03}", rank)
}

fn record(rank: usize) -> String {
    format!("{}:record-{:03}", key(rank), rank)
}

fn store_coord(rank: usize) -> Coordinate {
    phext::to_coordinate(&format!("1.1.1/1.1.1/1.1.{}", rank + 1))
}

fn index_coord() -> Coordinate {
    phext::to_coordinate("2.1.1/1.1.1/1.1.1")
}

/// Non-empty scrolls only (explode of a fresh phext can carry an empty
/// scroll at the default coordinate; phokenize("") behaves the same way).
fn live_map(doc: &str) -> HashMap<Coordinate, String> {
    phext::explode(doc)
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .collect()
}

/// Content of the derived balanced-index scroll over the first `count`
/// keys: the median key as root, plus the sorted key list.
fn index_content(count: usize) -> String {
    assert!(count > 0);
    let root = key((count - 1) / 2);
    let keys: Vec<String> = (0..count).map(key).collect();
    format!(
        "balanced-index n={} root={} keys={}",
        count,
        root,
        keys.join(",")
    )
}

/// Build just the store, inserts in strictly sorted (worst-case) order.
fn build_store() -> String {
    let mut doc = String::new();
    for r in 0..N {
        doc = phext::insert(doc, store_coord(r), &record(r));
    }
    doc
}

#[test]
fn sorted_insertion_never_moves_or_changes_any_store_scroll() {
    // The fixture keys really are in strictly sorted order — the worst
    // case that degenerates a naive BST into a chain of depth 200.
    for r in 1..N {
        assert!(key(r - 1) < key(r), "keys must be strictly sorted");
    }

    let mut doc = String::new();
    let mut previous: HashMap<Coordinate, String> = HashMap::new();

    for r in 0..N {
        doc = phext::insert(doc, store_coord(r), &record(r));

        // Strongest form, after EVERY insert: the full explode map only
        // ever GAINS the one new entry; no earlier entry changes or moves.
        let now = live_map(&doc);
        assert_eq!(now.len(), previous.len() + 1, "insert {} changed scroll count unexpectedly", r);
        for (coord, value) in &previous {
            assert_eq!(
                now.get(coord),
                Some(value),
                "insert {} changed or moved the scroll at {}",
                r,
                coord.to_string()
            );
        }
        assert_eq!(now.get(&store_coord(r)), Some(&record(r)));

        // And fetch() — not just the map — stays correct for every key
        // written so far, at its original coordinate.
        for prior in 0..=r {
            assert_eq!(
                phext::fetch(&doc, store_coord(prior)),
                record(prior),
                "fetch broke for rank {} after insert {}",
                prior,
                r
            );
        }

        previous = now;
    }

    // Final: exactly 200 store scrolls, and all 200 keys fetch correctly.
    assert_eq!(previous.len(), N);
    for r in 0..N {
        assert_eq!(phext::fetch(&doc, store_coord(r)), record(r));
    }
}

#[test]
fn store_has_no_balance_structure_and_no_degenerate_shape() {
    let doc = build_store();

    // Coordinate derivation is exactly rank+1 in the scroll component.
    assert_eq!(store_coord(0).to_string(), "1.1.1/1.1.1/1.1.1");
    assert_eq!(store_coord(199).to_string(), "1.1.1/1.1.1/1.1.200");

    // Every store scroll is exactly its record — no balance factor, no
    // height, no child/parent pointers, no rotation bookkeeping is stored.
    let forbidden = [
        "balance", "height", "left", "right", "parent", "child", "rotate",
        "rotation", "root", "avl", "red", "black",
    ];
    let map = live_map(&doc);
    assert_eq!(map.len(), N);
    for r in 0..N {
        let scroll = &map[&store_coord(r)];
        assert_eq!(scroll, &record(r));
        let lower = scroll.to_lowercase();
        for word in forbidden {
            assert!(
                !lower.contains(word),
                "store scroll for rank {} stores balance structure: {:?}",
                r,
                scroll
            );
        }
    }

    // Distinct keys get distinct coordinates (identity is the coordinate).
    let mut coords: Vec<Coordinate> = (0..N).map(store_coord).collect();
    coords.sort();
    coords.dedup();
    assert_eq!(coords.len(), N);
}

#[test]
fn document_order_is_sorted_order_without_any_search_structure() {
    let doc = build_store();
    // phokenize enumerates in coordinate order; under the rank derivation
    // that is exactly sorted key order — ordering is a traversal of the
    // addresses, not a walk of a balanced shape.
    let present: Vec<String> = phext::phokenize(&doc)
        .into_iter()
        .filter(|ps| !ps.scroll.is_empty())
        .map(|ps| ps.scroll)
        .collect();
    let expected: Vec<String> = (0..N).map(record).collect();
    assert_eq!(present, expected);
}

#[test]
fn derived_index_rebuild_touches_only_the_index_coordinate() {
    let checkpoints = [50usize, 100, 150, 200];
    let mut doc = String::new();

    for r in 0..N {
        doc = phext::insert(doc, store_coord(r), &record(r));
        let count = r + 1;
        if !checkpoints.contains(&count) {
            continue;
        }

        // Rebuild the derived balanced index at its FIXED coordinate.
        let before = live_map(&doc);
        doc = phext::replace(&doc, index_coord(), &index_content(count));
        let after = live_map(&doc);

        // The rebuild touched only the index coordinate: it is the sole
        // coordinate that gained or changed; store coordinates all match.
        assert_eq!(phext::fetch(&doc, index_coord()), index_content(count));
        for (coord, value) in &before {
            if *coord == index_coord() {
                continue; // the index itself is allowed to change
            }
            assert_eq!(
                after.get(coord),
                Some(value),
                "index rebuild at n={} changed store scroll at {}",
                count,
                coord.to_string()
            );
        }
        let changed: Vec<&Coordinate> = after
            .keys()
            .filter(|c| before.get(*c) != after.get(*c))
            .collect();
        assert_eq!(changed, vec![&index_coord()], "rebuild at n={} changed {:?}", count, changed);
        assert_eq!(after.len(), before.len() + if before.contains_key(&index_coord()) { 0 } else { 1 });
    }

    // Final accounting, counted separately: 200 store scrolls + 1 index.
    let map = live_map(&doc);
    let store_count = map.keys().filter(|c| **c != index_coord()).count();
    assert_eq!(store_count, N);
    assert_eq!(map.len(), N + 1);

    // The final index names the balanced root (median) over all 200 keys
    // and lists them sorted; the store underneath it never moved.
    assert_eq!(phext::fetch(&doc, index_coord()), index_content(N));
    assert!(index_content(N).contains("root=key-099"));
    for r in 0..N {
        assert_eq!(phext::fetch(&doc, store_coord(r)), record(r));
    }
}

#[test]
fn index_can_be_rebuilt_from_scratch_over_an_unchanged_store() {
    // A derived index is disposable: dropping it (remove) and rebuilding
    // it (replace) leaves the store byte-identical throughout.
    let mut doc = build_store();
    doc = phext::replace(&doc, index_coord(), &index_content(N));
    let with_index = live_map(&doc);

    doc = phext::remove(&doc, index_coord());
    let store_only = live_map(&doc);
    assert_eq!(store_only.len(), N);
    assert_eq!(phext::fetch(&doc, index_coord()), "");

    doc = phext::replace(&doc, index_coord(), &index_content(N));
    assert_eq!(live_map(&doc), with_index);
    for r in 0..N {
        assert_eq!(phext::fetch(&doc, store_coord(r)), record(r));
    }
}
