// TOAPDD #8 — Knuth TAOCP Vol 4B §7.2.2 Backtrack programming.
// N-queens (n=6) search tree persisted as a phext.
//
// Encoding (documented, found from the choice path itself):
//   A partial state is the list of columns chosen for rows 0..k-1, e.g. [1,3]
//   means row0->col1, row1->col3 (0-based columns in content).
//   Its coordinate uses one component per row across the z/y arms:
//     z.library  = row0 choice, z.shelf = row1, z.series = row2,
//     y.collection = row3, y.volume = row4, y.book = row5,
//   where a present choice c is stored as c+2 (2..=7) and an absent
//   row is 1, so absence is distinct from column 0.
//     x.chapter = 1, x.section = depth k + 1, x.scroll = 1.
//   The full path is also stored in the scroll content, so a stored
//   partial can be resumed by fetch alone.
// Content: "OPEN depth=k path=c0,c1,..." | "SOLUTION path=..." |
//          "PRUNED depth=k path=... reason=attack".
// Pruning is by marking: a dead child gets a PRUNED scroll and is
// never expanded; the scroll is never removed.
//
// Driver/library split, stated plainly: the DFS below is driver code in
// this file (it is cross-checked against an independent brute-force
// scan). What libphext contributes — and what the tests pin — is the
// persistence: every visited state is a scroll at its choice-path
// coordinate, fetched back exactly, with coordinate<->path consistency
// checked for all 895 stored states, and implode(explode(..)) fidelity.

use libphext::phext;
use std::collections::{BTreeSet, HashMap};

const N: usize = 6;

fn coord_for_path(path: &[usize]) -> phext::Coordinate {
    let mut comp = [1usize; 6];
    for (i, &c) in path.iter().enumerate() {
        comp[i] = c + 2;
    }
    phext::to_coordinate(&format!(
        "{}.{}.{}/{}.{}.{}/1.{}.1",
        comp[0], comp[1], comp[2], comp[3], comp[4], comp[5],
        path.len() + 1
    ))
}

fn path_from_coord(coord: &phext::Coordinate) -> Vec<usize> {
    let depth = coord.x.section - 1;
    let comps = [
        coord.z.library, coord.z.shelf, coord.z.series,
        coord.y.collection, coord.y.volume, coord.y.book,
    ];
    comps[..depth].iter().map(|v| v - 2).collect()
}

fn fmt_path(path: &[usize]) -> String {
    path.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(",")
}

fn parse_path(content: &str) -> Vec<usize> {
    // content contains "path=c0,c1,..." (possibly followed by more text)
    let start = content.find("path=").expect("path= in content") + 5;
    let rest = &content[start..];
    let token: String = rest.chars().take_while(|ch| ch.is_ascii_digit() || *ch == ',').collect();
    if token.is_empty() { return Vec::new(); }
    token.split(',').map(|s| s.parse().unwrap()).collect()
}

fn attacks(path: &[usize], col: usize) -> bool {
    let row = path.len();
    path.iter().enumerate().any(|(r, &c)| c == col || c.abs_diff(col) == row - r)
}

fn valid_solution(sol: &[usize]) -> bool {
    if sol.len() != N { return false; }
    let mut prefix: Vec<usize> = Vec::new();
    for &c in sol {
        if attacks(&prefix, c) { return false; }
        prefix.push(c);
    }
    true
}

/// DFS that records every visited partial state into `map`.
fn expand(map: &mut HashMap<phext::Coordinate, String>, path: &[usize], solutions: &mut Vec<Vec<usize>>) {
    for col in 0..N {
        let mut child = path.to_vec();
        child.push(col);
        let coord = coord_for_path(&child);
        if attacks(path, col) {
            map.insert(coord, format!("PRUNED depth={} path={} reason=attack", child.len(), fmt_path(&child)));
        } else if child.len() == N {
            map.insert(coord, format!("SOLUTION path={}", fmt_path(&child)));
            solutions.push(child);
        } else {
            map.insert(coord, format!("OPEN depth={} path={}", child.len(), fmt_path(&child)));
            expand(map, &child, solutions);
        }
    }
}

fn build_tree_from(start: &[usize]) -> (String, Vec<Vec<usize>>) {
    let mut map: HashMap<phext::Coordinate, String> = HashMap::new();
    let mut solutions = Vec::new();
    if start.len() == N {
        map.insert(coord_for_path(start), format!("SOLUTION path={}", fmt_path(start)));
        solutions.push(start.to_vec());
        return (phext::implode(map), solutions);
    }
    let label = if start.is_empty() { "OPEN depth=0 path=".to_string() }
        else { format!("OPEN depth={} path={}", start.len(), fmt_path(start)) };
    map.insert(coord_for_path(start), label);
    expand(&mut map, start, &mut solutions);
    solutions.sort();
    (phext::implode(map), solutions)
}

fn build_tree() -> (String, Vec<Vec<usize>>) { build_tree_from(&[]) }

fn known_solutions() -> BTreeSet<Vec<usize>> {
    // The 4 solutions of 6-queens (columns per row, 0-based), independently
    // cross-checked below against a brute-force permutation scan.
    vec![
        vec![1, 3, 5, 0, 2, 4],
        vec![2, 5, 1, 4, 0, 3],
        vec![3, 0, 4, 1, 5, 2],
        vec![4, 2, 0, 5, 3, 1],
    ].into_iter().collect()
}

fn brute_force_solutions() -> BTreeSet<Vec<usize>> {
    // Independent check: all permutations of 0..6 with no diagonal attacks.
    fn permute(v: &mut Vec<usize>, k: usize, out: &mut Vec<Vec<usize>>) {
        if k == v.len() { out.push(v.clone()); return; }
        for i in k..v.len() { v.swap(k, i); permute(v, k + 1, out); v.swap(k, i); }
    }
    let mut perms = Vec::new();
    permute(&mut (0..N).collect(), 0, &mut perms);
    perms.into_iter().filter(|p| {
        (0..N).all(|r1| (r1 + 1..N).all(|r2| p[r1].abs_diff(p[r2]) != r2 - r1))
    }).collect()
}

#[test]
fn fresh_run_finds_exactly_four_known_solutions() {
    assert_eq!(brute_force_solutions(), known_solutions(), "fixture must match brute force");
    let (tree, solutions) = build_tree();
    assert_eq!(solutions.len(), 4, "n=6 has exactly 4 solutions");
    let set: BTreeSet<Vec<usize>> = solutions.clone().into_iter().collect();
    assert_eq!(set, known_solutions());
    assert!(solutions.iter().all(|s| valid_solution(s)));
    // The SOLUTION scrolls stored in the phext agree with the driver.
    let stored: BTreeSet<Vec<usize>> = phext::explode(&tree).values()
        .filter(|v| v.starts_with("SOLUTION"))
        .map(|v| parse_path(v)).collect();
    assert_eq!(stored, known_solutions());
}

#[test]
fn partial_states_are_addressable_by_choice_path() {
    let (tree, _) = build_tree();
    // Root and a mid-search partial are fetchable at their path coordinates.
    assert_eq!(phext::fetch(&tree, coord_for_path(&[])), "OPEN depth=0 path=");
    let mid = phext::fetch(&tree, coord_for_path(&[1, 3]));
    assert!(mid.starts_with("OPEN"), "got: {mid}");
    assert_eq!(parse_path(&mid), vec![1, 3]);
    // Coordinate <-> path round-trips for every stored scroll.
    for (coord, content) in phext::explode(&tree) {
        if content.is_empty() { continue; }
        assert_eq!(path_from_coord(&coord), parse_path(&content), "coord {coord} vs {content}");
    }
    // Round-trip through implode(explode(..)) preserves a solution scroll.
    let sol_coord = coord_for_path(&[1, 3, 5, 0, 2, 4]);
    let before = phext::fetch(&tree, sol_coord);
    assert!(before.starts_with("SOLUTION"), "got: {before}");
    let rebuilt = phext::implode(phext::explode(&tree));
    assert_eq!(phext::fetch(&rebuilt, sol_coord), before);
}

#[test]
fn prune_marks_not_deletes_and_subtree_not_expanded() {
    let (tree, _) = build_tree();
    // [0,1] is a diagonal attack: it must exist as a PRUNED scroll.
    let pruned_coord = coord_for_path(&[0, 1]);
    let content = phext::fetch(&tree, pruned_coord);
    assert!(content.starts_with("PRUNED"), "pruned scroll must be marked, got: {content:?}");
    let map = phext::explode(&tree);
    assert!(map.contains_key(&pruned_coord), "pruned coordinate must still be a key in explode");
    assert!(map.values().filter(|v| v.starts_with("PRUNED")).count() > 0);
    // Its subtree was not expanded: no stored path strictly extends [0,1].
    for (coord, c) in &map {
        if c.is_empty() { continue; }
        let p = path_from_coord(coord);
        assert!(!(p.len() > 2 && p[..2] == [0, 1]), "pruned subtree expanded at {p:?}");
    }
    // Marking via replace (the API operation) also keeps the scroll present.
    let mut small = phext::insert(String::new(), coord_for_path(&[2]), "OPEN depth=1 path=2");
    small = phext::replace(&small, coord_for_path(&[2]), "PRUNED depth=1 path=2 reason=attack");
    assert_eq!(phext::fetch(&small, coord_for_path(&[2])), "PRUNED depth=1 path=2 reason=attack");
    assert!(phext::explode(&small).contains_key(&coord_for_path(&[2])));
}

#[test]
fn resume_from_stored_partial_matches_fresh() {
    let (tree, fresh) = build_tree();
    // Resume from the root stored in the phext reproduces the fresh run.
    let root = phext::fetch(&tree, coord_for_path(&[]));
    let (_, from_root) = build_tree_from(&parse_path(&root));
    assert_eq!(from_root, fresh);
    // Resume from a named mid-search partial fetched from the stored phext.
    let stored_mid = phext::fetch(&tree, coord_for_path(&[1, 3]));
    assert!(stored_mid.starts_with("OPEN"));
    let start = parse_path(&stored_mid);
    let (_, resumed) = build_tree_from(&start);
    let expected: Vec<Vec<usize>> = fresh.iter().cloned()
        .filter(|s| s[..start.len()] == start[..]).collect();
    assert_eq!(resumed, expected);
    assert_eq!(resumed, vec![vec![1, 3, 5, 0, 2, 4]], "exactly one solution extends [1,3]");
}

#[test]
fn disjoint_resumes_partition_solution_set() {
    let (tree, fresh) = build_tree();
    // Two workers resume independently from the SAME stored phext string,
    // with no shared mutable stack, on disjoint first-row prefixes.
    let mut union: BTreeSet<Vec<usize>> = BTreeSet::new();
    for first in [1usize, 2] {
        let stored = phext::fetch(&tree, coord_for_path(&[first]));
        assert!(stored.starts_with("OPEN"), "prefix [{first}] stored: {stored}");
        let (_, sols) = build_tree_from(&parse_path(&stored));
        for s in sols {
            assert!(union.insert(s), "disjoint prefixes must not duplicate solutions");
        }
    }
    let expected: BTreeSet<Vec<usize>> = fresh.iter().cloned()
        .filter(|s| s[0] == 1 || s[0] == 2).collect();
    assert_eq!(union, expected);
    assert_eq!(union.len(), 2);
}
