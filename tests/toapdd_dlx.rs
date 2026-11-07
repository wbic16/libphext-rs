// TOAPDD #7 — Dancing Links / Algorithm X (TAOCP Vol 4B §7.2.2.1)
// Phext-native claim: sparse exact-cover matrix as scrolls; solving uses
// versioned phext strings instead of in-place dance/undo; backtrack =
// return to an earlier version string. Solution verifies as exact cover.
// NOTE: "Algorithm X" here is exact cover, NOT extended Euclid (§4.5.2).
use libphext::phext;
use std::collections::{BTreeMap, BTreeSet, HashMap};

// Universe: columns 1..=4. Rows (1-indexed):
//  R1={1,2} R2={1,3} R3={2,4} R4={3,4} R5={1,4} R6={2,3}
// Exact covers: {R1,R4}, {R2,R3}, {R5,R6} — three solutions.
const ROWS: [(usize, [usize; 2]); 6] = [
    (1, [1, 2]),
    (2, [1, 3]),
    (3, [2, 4]),
    (4, [3, 4]),
    (5, [1, 4]),
    (6, [2, 3]),
];
const NCOLS: usize = 4;

fn cell(row: usize, col: usize) -> phext::Coordinate {
    phext::to_coordinate(&format!("1.1.1/1.1.{}/1.1.{}", row, col))
}

fn build_matrix() -> String {
    let mut p = String::new();
    for (row, cols) in ROWS.iter() {
        for col in cols.iter() {
            p = phext::insert(p, cell(*row, *col), "1");
        }
    }
    p
}

/// Parse a phext version string back into row -> set of columns.
fn parse(p: &str) -> BTreeMap<usize, BTreeSet<usize>> {
    let map = phext::explode(p);
    let mut rows: BTreeMap<usize, BTreeSet<usize>> = BTreeMap::new();
    for (coord, val) in map.iter() {
        if val.trim().is_empty() {
            continue;
        }
        let row = coord.y.book;
        let col = coord.x.scroll;
        rows.entry(row).or_default().insert(col);
    }
    rows
}

/// Cover: given a version string and a chosen row, produce a NEW phext
/// string containing only rows that share no column with the chosen row
/// (and excluding the chosen row itself). No mutation of the input.
fn cover(version: &str, chosen: usize) -> String {
    let rows = parse(version);
    let chosen_cols = rows.get(&chosen).cloned().unwrap_or_default();
    let mut out: HashMap<phext::Coordinate, String> = HashMap::new();
    for (row, cols) in rows.iter() {
        if *row == chosen {
            continue;
        }
        if cols.iter().any(|c| chosen_cols.contains(c)) {
            continue;
        }
        for col in cols.iter() {
            out.insert(cell(*row, *col), "1".to_string());
        }
    }
    phext::implode(out)
}

/// Recursive Algorithm X over version strings. Each choice pushes a new
/// version onto `versions`; backtracking simply re-reads an earlier entry.
fn solve(
    version_idx: usize,
    versions: &mut Vec<String>,
    chosen: &mut Vec<usize>,
    solutions: &mut Vec<Vec<usize>>,
) {
    let current = versions[version_idx].clone();
    let rows = parse(&current);
    let covered: BTreeSet<usize> = chosen
        .iter()
        .flat_map(|r| {
            ROWS.iter()
                .find(|(id, _)| id == r)
                .unwrap()
                .1
                .iter()
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect();
    if covered.len() == NCOLS {
        let mut s = chosen.clone();
        s.sort();
        solutions.push(s);
        return;
    }
    if rows.is_empty() {
        return; // dead end — backtrack by returning
    }
    // Pick first uncovered column present in remaining matrix.
    let mut uncovered: Vec<usize> = (1..=NCOLS)
        .filter(|c| !covered.contains(c))
        .collect();
    uncovered.sort();
    let col = match uncovered.first() {
        Some(c) => *c,
        None => return,
    };
    // Candidate rows in the current version covering `col`, in row order.
    let candidates: Vec<usize> = rows
        .iter()
        .filter(|(_, cols)| cols.contains(&col))
        .map(|(r, _)| *r)
        .collect();
    for cand in candidates {
        let next = cover(&current, cand);
        versions.push(next);
        let next_idx = versions.len() - 1;
        chosen.push(cand);
        solve(next_idx, versions, chosen, solutions);
        chosen.pop(); // backtrack: earlier version string is untouched
    }
}

fn all_solutions() -> (Vec<String>, Vec<Vec<usize>>) {
    let v0 = build_matrix();
    let mut versions = vec![v0];
    let mut chosen = Vec::new();
    let mut solutions = Vec::new();
    solve(0, &mut versions, &mut chosen, &mut solutions);
    for s in solutions.iter_mut() {
        s.sort();
    }
    solutions.sort();
    (versions, solutions)
}

#[test]
fn sparse_matrix_only_one_cells_occupy_scrolls() {
    let p = build_matrix();
    let map = phext::explode(&p);
    let nonempty: Vec<_> = map.values().filter(|v| !v.trim().is_empty()).collect();
    assert_eq!(nonempty.len(), 12, "6 rows x 2 cols = 12 one-cells");
    // Present cell fetches "1"; absent cell fetches empty (no scroll).
    assert_eq!(phext::fetch(&p, cell(1, 1)), "1");
    assert_eq!(phext::fetch(&p, cell(1, 2)), "1");
    assert_eq!(phext::fetch(&p, cell(1, 3)), "");
    assert_eq!(phext::fetch(&p, cell(1, 4)), "");
    assert_eq!(phext::fetch(&p, cell(6, 3)), "1");
    // Parse round-trips the intended rows.
    let rows = parse(&p);
    assert_eq!(rows.len(), 6);
    assert_eq!(rows[&4], BTreeSet::from([3, 4]));
}

#[test]
fn solver_finds_all_exact_covers_and_they_verify() {
    let (_versions, solutions) = all_solutions();
    assert_eq!(
        solutions,
        vec![vec![1, 4], vec![2, 3], vec![5, 6]],
        "expected the three exact covers"
    );
    // Verify each solution: every column covered exactly once.
    for sol in &solutions {
        let mut counts = [0usize; NCOLS + 1];
        for r in sol {
            let cols = ROWS.iter().find(|(id, _)| id == r).unwrap().1;
            for c in cols {
                counts[c] += 1;
            }
        }
        for c in 1..=NCOLS {
            assert_eq!(counts[c], 1, "column {} must be covered exactly once", c);
        }
    }
}

#[test]
fn versioned_choices_are_immutable_no_dance_undo() {
    let v0 = build_matrix();
    let v0_snapshot = v0.clone();
    let v0_checksum = phext::checksum(&v0);
    // Make a choice: cover with row 1 -> new version string v1.
    let v1 = cover(&v0, 1);
    // v0 is unchanged (no in-place dance to undo).
    assert_eq!(v0, v0_snapshot);
    assert_eq!(phext::checksum(&v0), v0_checksum);
    // v1 contains only rows disjoint from R1={1,2}: R4={3,4} alone remains
    // (R2,R3,R5,R6 all share col 1 or 2).
    let rows1 = parse(&v1);
    assert_eq!(rows1.len(), 1);
    assert!(rows1.contains_key(&4));
    // A further choice from v1 does not alter v1 either.
    let v1_snapshot = v1.clone();
    let _v2 = cover(&v1, 4);
    assert_eq!(v1, v1_snapshot);
    // Backtrack = fetch the earlier version: parsing v0 again yields all rows.
    assert_eq!(parse(&v0).len(), 6);
}

#[test]
fn backtrack_by_earlier_version_finds_alternative_solution() {
    let (versions, solutions) = all_solutions();
    // The search created multiple versions and still found all solutions,
    // which is only possible by returning to earlier version strings.
    assert!(versions.len() > 3, "expected branching versions, got {}", versions.len());
    assert_eq!(solutions.len(), 3);
    // Version 0 is still the full matrix after the whole search.
    assert_eq!(parse(&versions[0]).len(), 6);
}
