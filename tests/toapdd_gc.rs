// TOAPDD #12 — Knuth TAOCP Vol.1 §2.3.5 Lists and garbage collection.
//
// Phext-native claim: reachability can be READ instead of computed
// destructively. Roots and references are links between scrolls, so the
// live set is a traversal over fetch(); "collection" writes one inspectable
// MARKS scroll and deletes nothing — every node stays byte-identical, and
// the marks scroll alone reproduces the live/garbage partition afterwards.
//
// Honesty note: the mark-phase BFS below is driver code in this file; what
// libphext contributes — and what the tests pin — is fetch-based traversal,
// phokenize/index enumeration of exactly the stored nodes, and insert of
// the MARKS scroll leaving all other scrolls untouched.
// Graph encoding: node scrolls link via "-> <coordinate>" lines in content.
use libphext::phext::{fetch, index, insert, phokenize, to_coordinate};
use std::collections::{BTreeSet, HashSet, VecDeque};

fn c(s: &str) -> libphext::phext::Coordinate { to_coordinate(s) }

fn root_coord() -> libphext::phext::Coordinate { c("1.1.1/1.1.1/1.1.1") }
fn a() -> libphext::phext::Coordinate { c("1.1.1/1.1.1/1.1.2") }
fn b() -> libphext::phext::Coordinate { c("1.1.1/1.1.1/1.1.3") }
fn cc() -> libphext::phext::Coordinate { c("1.1.1/1.1.1/1.1.4") }
fn d() -> libphext::phext::Coordinate { c("1.1.1/1.1.1/1.1.5") }
fn x() -> libphext::phext::Coordinate { c("1.1.1/1.1.1/1.1.6") }
fn y() -> libphext::phext::Coordinate { c("1.1.1/1.1.1/1.1.7") }
fn u() -> libphext::phext::Coordinate { c("1.1.1/1.1.1/1.1.8") }
fn marks_coord() -> libphext::phext::Coordinate { c("1.1.1/1.1.1/1.1.9") }

fn all_nodes() -> Vec<libphext::phext::Coordinate> { vec![a(), b(), cc(), d(), x(), y(), u()] }

fn build_graph() -> String {
    let mut p = String::new();
    p = insert(p, root_coord(), "ROOTS\n-> 1.1.1/1.1.1/1.1.2\n-> 1.1.1/1.1.1/1.1.3");
    p = insert(p, a(), "A\n-> 1.1.1/1.1.1/1.1.4");
    p = insert(p, b(), "B leaf");
    p = insert(p, cc(), "C transitive\n-> 1.1.1/1.1.1/1.1.5");
    p = insert(p, d(), "D transitive leaf");
    p = insert(p, x(), "X island\n-> 1.1.1/1.1.1/1.1.7");
    p = insert(p, y(), "Y island\n-> 1.1.1/1.1.1/1.1.6");
    p = insert(p, u(), "U unlinked");
    p
}

fn parse_links(content: &str) -> Vec<libphext::phext::Coordinate> {
    let mut out = Vec::new();
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("-> ") {
            out.push(to_coordinate(rest.trim()));
        }
    }
    out
}

fn roots_of(phext: &str) -> Vec<libphext::phext::Coordinate> {
    parse_links(&fetch(phext, root_coord()))
}

/// Mark phase: BFS from roots using fetch only.
fn mark_phase(phext: &str, roots: &[libphext::phext::Coordinate]) -> BTreeSet<String> {
    let mut live: BTreeSet<String> = BTreeSet::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut q: VecDeque<libphext::phext::Coordinate> = VecDeque::new();
    for r in roots { q.push_back(*r); }
    while let Some(cur) = q.pop_front() {
        let key = cur.to_string();
        if !seen.insert(key.clone()) { continue; }
        live.insert(key);
        let content = fetch(phext, cur);
        for nxt in parse_links(&content) { q.push_back(nxt); }
    }
    live
}

fn expected_live() -> BTreeSet<String> {
    [a(), b(), cc(), d()].iter().map(|v| v.to_string()).collect()
}
fn expected_garbage() -> BTreeSet<String> {
    [x(), y(), u()].iter().map(|v| v.to_string()).collect()
}

/// Collection: write MARKS scroll listing LIVE/GARBAGE per node. No deletion.
fn collect(phext: &str) -> String {
    let roots = roots_of(phext);
    let live = mark_phase(phext, &roots);
    let mut lines: Vec<String> = Vec::new();
    for n in all_nodes() {
        let k = n.to_string();
        if live.contains(&k) { lines.push(format!("LIVE {}", k)); }
        else { lines.push(format!("GARBAGE {}", k)); }
    }
    lines.sort();
    insert(phext.to_string(), marks_coord(), &lines.join("\n"))
}

fn partition_from_marks(phext: &str) -> (BTreeSet<String>, BTreeSet<String>) {
    let content = fetch(phext, marks_coord());
    let mut live = BTreeSet::new();
    let mut garbage = BTreeSet::new();
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("LIVE ") { live.insert(rest.trim().to_string()); }
        else if let Some(rest) = t.strip_prefix("GARBAGE ") { garbage.insert(rest.trim().to_string()); }
    }
    (live, garbage)
}

#[test]
fn mark_traversal_finds_exactly_reachable_set() {
    let p = build_graph();
    let roots = roots_of(&p);
    assert_eq!(roots, vec![a(), b()]);
    let live = mark_phase(&p, &roots);
    assert_eq!(live, expected_live(), "live must be exactly reachable incl. transitive C,D");
    // transitive check explicitly: D is only reachable via A -> C -> D
    assert!(live.contains(&d().to_string()));
    assert!(live.contains(&cc().to_string()));
}

#[test]
fn garbage_is_exactly_island_plus_unlinked() {
    let p = build_graph();
    let live = mark_phase(&p, &roots_of(&p));
    let all: BTreeSet<String> = all_nodes().iter().map(|v| v.to_string()).collect();
    let garbage: BTreeSet<String> = all.difference(&live).cloned().collect();
    assert_eq!(garbage, expected_garbage());
    // island nodes link to each other but are still garbage (cycle does not make them live)
    assert!(garbage.contains(&x().to_string()));
    assert!(garbage.contains(&y().to_string()));
}

#[test]
fn index_and_phokenize_enumerate_all_nodes() {
    let p = build_graph();
    // phokenize enumerates every populated scroll
    let scrolls: Vec<_> = phokenize(&p).into_iter().filter(|s| !s.scroll.trim().is_empty()).collect();
    let coords: BTreeSet<String> = scrolls.iter().map(|s| s.coord.to_string()).collect();
    for n in all_nodes() { assert!(coords.contains(&n.to_string()), "missing {}", n); }
    assert!(coords.contains(&root_coord().to_string()));
    // index() returns a phext whose scrolls are byte offsets; phokenizing it
    // must enumerate exactly the same coordinates (auditable enumeration).
    let idx = index(&p);
    assert!(!idx.is_empty());
    let idx_coords: BTreeSet<String> = phokenize(&idx).into_iter()
        .filter(|s| !s.scroll.trim().is_empty())
        .map(|s| s.coord.to_string()).collect();
    assert_eq!(idx_coords, coords, "index enumeration must match phokenize enumeration");
}

#[test]
fn collection_marks_without_deleting() {
    let p = build_graph();
    // snapshot every node + root before collection
    let mut before: Vec<(libphext::phext::Coordinate, String)> = Vec::new();
    for n in std::iter::once(root_coord()).chain(all_nodes()) {
        before.push((n, fetch(&p, n)));
    }
    let collected = collect(&p);
    // marks scroll is fetchable and inspectable, listing every node
    let marks = fetch(&collected, marks_coord());
    assert!(!marks.is_empty(), "marks scroll must be fetchable");
    for n in all_nodes() {
        assert!(marks.contains(&n.to_string()), "marks missing {}", n);
    }
    let (live, garbage) = partition_from_marks(&collected);
    assert_eq!(live, expected_live());
    assert_eq!(garbage, expected_garbage());
    // NOTHING silently deleted: every node byte-identical after collection
    for (n, orig) in &before {
        assert_eq!(&fetch(&collected, *n), orig, "node {} changed by collection", n);
    }
}

#[test]
fn marks_scroll_reproduces_partition_without_rewalking() {
    let p = build_graph();
    let collected = collect(&p);
    // Second traversal reads ONLY the marks scroll — no graph walk.
    let (live, garbage) = partition_from_marks(&collected);
    assert_eq!(live, expected_live());
    assert_eq!(garbage, expected_garbage());
    // partition is complete and disjoint
    assert!(live.is_disjoint(&garbage));
    let union: BTreeSet<String> = live.union(&garbage).cloned().collect();
    let all: BTreeSet<String> = all_nodes().iter().map(|v| v.to_string()).collect();
    assert_eq!(union, all);
}
