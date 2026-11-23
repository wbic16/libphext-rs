// TOAPDD attempt #6 — Knuth TAOCP Vol. 1, §2.2.3 Linked allocation.
//
// Phext-native claim: a linked list is a set of scrolls, one per node, where
// each node's content carries its successor's *coordinate* as text
// ("value|next=<coordinate or END>"). The list is walked by a driver that
// fetches a node, parses its next pointer, and fetches again — no contiguous
// arena, no pointer surgery inside a shared buffer.
//
// Proven here:
//  * the walk follows next pointers in order;
//  * replace of one node leaves every other node byte-identical
//    (fetch equality per coordinate + explode-map diff of exactly one key);
//  * insert (allocation of a fresh node scroll) leaves every existing node
//    byte-identical; linking it in is a separate, single-scroll replace of
//    its predecessor;
//  * remove (free) empties exactly the freed coordinate and leaves every
//    other node byte-identical at its original coordinate, and the freed
//    node's content is still recoverable from a prior version string held
//    by the test — reuse of the freed coordinate does not erase that history.

use libphext::phext;
use libphext::phext::Coordinate;
use std::collections::HashMap;

/// Node n lives at 1.1.1/1.1.1/1.1.n — the scroll dimension is the slot.
fn node_coord(n: usize) -> Coordinate {
    phext::to_coordinate(&format!("1.1.1/1.1.1/1.1.{}", n))
}

fn node_ref(n: usize) -> String {
    node_coord(n).to_string()
}

/// Node payload: a value plus a textual link to the successor's coordinate.
fn node_content(value: &str, next: Option<usize>) -> String {
    match next {
        Some(n) => format!("{}|next={}", value, node_ref(n)),
        None => format!("{}|next=END", value),
    }
}

/// Allocate the list node-by-node with `insert`, in address order.
fn build(values: &[&str]) -> String {
    let mut doc = String::new();
    for (i, value) in values.iter().enumerate() {
        let n = i + 1;
        let next = if n < values.len() { Some(n + 1) } else { None };
        doc = phext::insert(doc, node_coord(n), &node_content(value, next));
    }
    doc
}

/// The list walk: fetch head, read its next pointer, fetch again, until END.
fn walk(doc: &str, head: usize) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = Some(head);
    let mut guard = 0;
    while let Some(n) = current {
        guard += 1;
        assert!(guard <= 100, "walk failed to terminate (cycle?)");
        let content = phext::fetch(doc, node_coord(n));
        assert!(!content.is_empty(), "walk hit a missing node at {}", node_ref(n));
        let mut parts = content.splitn(2, "|next=");
        values.push(parts.next().unwrap().to_string());
        let next = parts.next().expect("node content must carry a next pointer");
        current = if next == "END" {
            None
        } else {
            // The pointer is a coordinate in text form; resolve it back.
            let coord = phext::to_coordinate(next);
            Some(coord.x.scroll)
        };
    }
    values
}

/// Non-empty scrolls only (phokenize("") yields one empty scroll; drop empties).
fn snapshot(doc: &str) -> HashMap<Coordinate, String> {
    phext::explode(doc)
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .collect()
}

/// Coordinates whose content differs between two snapshots.
fn changed_coords(
    before: &HashMap<Coordinate, String>,
    after: &HashMap<Coordinate, String>,
) -> Vec<Coordinate> {
    let mut changed: Vec<Coordinate> = before
        .keys()
        .chain(after.keys())
        .filter(|k| before.get(*k) != after.get(*k))
        .cloned()
        .collect();
    changed.sort();
    changed.dedup();
    changed
}

#[test]
fn list_walk_follows_next_pointers_in_order() {
    let doc = build(&["alpha", "bravo", "charlie", "delta"]);

    // Each node is fetchable at its own coordinate with its exact content.
    assert_eq!(phext::fetch(&doc, node_coord(1)), "alpha|next=1.1.1/1.1.1/1.1.2");
    assert_eq!(phext::fetch(&doc, node_coord(2)), "bravo|next=1.1.1/1.1.1/1.1.3");
    assert_eq!(phext::fetch(&doc, node_coord(3)), "charlie|next=1.1.1/1.1.1/1.1.4");
    assert_eq!(phext::fetch(&doc, node_coord(4)), "delta|next=END");

    // Four nodes, no more: the list is exactly four scrolls.
    assert_eq!(snapshot(&doc).len(), 4);

    // The driver walk recovers the list in link order.
    assert_eq!(walk(&doc, 1), vec!["alpha", "bravo", "charlie", "delta"]);
}

#[test]
fn replace_one_node_leaves_all_other_nodes_byte_identical() {
    let doc = build(&["alpha", "bravo", "charlie", "delta"]);
    let before = snapshot(&doc);

    // Replace node 2's payload, keeping its successor pointer unchanged.
    let doc2 = phext::replace(&doc, node_coord(2), &node_content("bravo-II", Some(3)));

    assert_eq!(
        phext::fetch(&doc2, node_coord(2)),
        "bravo-II|next=1.1.1/1.1.1/1.1.3"
    );
    // Every other node: byte-identical, per-coordinate fetch and whole-map diff.
    for n in [1usize, 3, 4] {
        assert_eq!(
            phext::fetch(&doc2, node_coord(n)),
            phext::fetch(&doc, node_coord(n)),
            "node {} changed under a replace of node 2",
            n
        );
    }
    assert_eq!(changed_coords(&before, &snapshot(&doc2)), vec![node_coord(2)]);
    assert_eq!(walk(&doc2, 1), vec!["alpha", "bravo-II", "charlie", "delta"]);
}

#[test]
fn insert_allocates_a_node_without_touching_existing_nodes() {
    let doc = build(&["alpha", "bravo", "charlie"]);
    let before = snapshot(&doc);

    // Allocation: a brand-new node scroll at a fresh coordinate (scroll 5;
    // scroll 4 stays an unallocated gap). Not yet linked into the list.
    let doc2 = phext::insert(doc.clone(), node_coord(5), &node_content("echo", None));

    assert_eq!(phext::fetch(&doc2, node_coord(5)), "echo|next=END");
    assert_eq!(phext::fetch(&doc2, node_coord(4)), "");
    for n in [1usize, 2, 3] {
        assert_eq!(
            phext::fetch(&doc2, node_coord(n)),
            phext::fetch(&doc, node_coord(n)),
            "existing node {} changed under an insert elsewhere",
            n
        );
    }
    // The insert added exactly one scroll and changed none.
    let after = snapshot(&doc2);
    assert_eq!(after.len(), before.len() + 1);
    assert_eq!(changed_coords(&before, &after), vec![node_coord(5)]);
    // Unlinked, so the walk is unchanged.
    assert_eq!(walk(&doc2, 1), vec!["alpha", "bravo", "charlie"]);

    // Linking is its own operation: one replace on the predecessor only.
    let doc3 = phext::replace(&doc2, node_coord(3), &node_content("charlie", Some(5)));
    assert_eq!(
        changed_coords(&snapshot(&doc2), &snapshot(&doc3)),
        vec![node_coord(3)]
    );
    assert_eq!(walk(&doc3, 1), vec!["alpha", "bravo", "charlie", "echo"]);
}

#[test]
fn remove_frees_node_others_identical_and_prior_version_recovers_it() {
    let doc = build(&["alpha", "bravo", "charlie", "delta"]);
    // The prior version string, held by the test before the free.
    let prior_version = doc.clone();
    let freed_content = phext::fetch(&doc, node_coord(3));
    assert_eq!(freed_content, "charlie|next=1.1.1/1.1.1/1.1.4");
    let before = snapshot(&doc);

    // Free = remove. Exactly the freed coordinate empties.
    let doc2 = phext::remove(&doc, node_coord(3));
    assert_eq!(phext::fetch(&doc2, node_coord(3)), "");
    assert!(!snapshot(&doc2).contains_key(&node_coord(3)));
    // Every surviving node is byte-identical at its *original* coordinate —
    // node 4 did not slide down into the freed slot.
    for n in [1usize, 2, 4] {
        assert_eq!(
            phext::fetch(&doc2, node_coord(n)),
            phext::fetch(&doc, node_coord(n)),
            "surviving node {} changed under a remove of node 3",
            n
        );
    }
    assert_eq!(changed_coords(&before, &snapshot(&doc2)), vec![node_coord(3)]);

    // History: the freed node's content is recoverable from the prior
    // version string the test still holds — freeing erased nothing there.
    assert_eq!(phext::fetch(&prior_version, node_coord(3)), freed_content);
    assert_eq!(walk(&prior_version, 1), vec!["alpha", "bravo", "charlie", "delta"]);

    // The caller relinks the predecessor (one replace, as in Knuth), and
    // the list walks cleanly over the gap.
    let doc3 = phext::replace(&doc2, node_coord(2), &node_content("bravo", Some(4)));
    assert_eq!(walk(&doc3, 1), vec!["alpha", "bravo", "delta"]);

    // Reuse of the freed coordinate: write a new node into scroll 3 and
    // link it in. The prior version string is untouched by the reuse, so
    // the original freed content is still recoverable from it afterwards.
    let doc4 = phext::replace(&doc3, node_coord(3), &node_content("recycled", Some(4)));
    let doc5 = phext::replace(&doc4, node_coord(2), &node_content("bravo", Some(3)));
    assert_eq!(walk(&doc5, 1), vec!["alpha", "bravo", "recycled", "delta"]);
    assert_eq!(prior_version, doc);
    assert_eq!(phext::fetch(&prior_version, node_coord(3)), "charlie|next=1.1.1/1.1.1/1.1.4");
}
