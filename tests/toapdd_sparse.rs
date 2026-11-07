// TOAPDD — Knuth TAOCP §2.4 + §2.2.6 Multilinked structures / orthogonal lists
// Slug: sparse · Box: rock-mountain
//
// Sparse matrix as scrolls:
//   absent cells occupy no scroll; fetch by coordinate = cell lookup;
//   inserting one cell touches one coordinate;
//   row and column traversals both recover the matrix.
//
// Encoding (documented here, per spec):
//   cell (row, col)  ->  coordinate 1.1.1/1.1.1/<chapter>.<section>.1
//                        where chapter = row + 1, section = col + 1, scroll = 1
//   (phext coordinate components are >= 1, so 0-based matrix indices are
//    shifted by +1. All other components stay at their default 1.)
//   The scroll content is the cell value. An absent cell has NO scroll.
//   Orthogonal links in Knuth's multilinked list become coordinate
//   references: row links are "same chapter, varying section" and column
//   links are "same section, varying chapter" — readable from the
//   coordinate itself, with no pointer fields stored anywhere.
//
// Fixture matrix: 4 rows x 5 cols, 6 non-zero cells out of 20.
//   row 0: col 1 = "a", col 4 = "b"
//   row 1: col 0 = "c"
//   row 2: col 2 = "d", col 4 = "e"
//   row 3: col 3 = "f"

use libphext::phext;
use std::collections::HashMap;

const ROWS: usize = 4;
const COLS: usize = 5;

fn cell_coord(row: usize, col: usize) -> phext::Coordinate {
    phext::to_coordinate(&format!("1.1.1/1.1.1/{}.{}.1", row + 1, col + 1))
}

fn nonzeros() -> Vec<(usize, usize, &'static str)> {
    vec![
        (0, 1, "a"),
        (0, 4, "b"),
        (1, 0, "c"),
        (2, 2, "d"),
        (2, 4, "e"),
        (3, 3, "f"),
    ]
}

fn fixture() -> String {
    let mut map: HashMap<phext::Coordinate, String> = HashMap::new();
    for (r, c, v) in nonzeros() {
        map.insert(cell_coord(r, c), v.to_string());
    }
    phext::implode(map)
}

/// Populated cells only: explode may surface empty scrolls (e.g. the
/// default scroll from phokenize("")), so empties are filtered out —
/// an empty string is not a matrix entry.
fn populated(phext_str: &str) -> HashMap<phext::Coordinate, String> {
    phext::explode(phext_str)
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .collect()
}

fn row_of(coord: &phext::Coordinate) -> usize {
    coord.x.chapter - 1
}
fn col_of(coord: &phext::Coordinate) -> usize {
    coord.x.section - 1
}

#[test]
fn absent_cells_occupy_no_scroll() {
    let m = fixture();
    let cells = populated(&m);
    // 6 non-zero cells, not 4*5 = 20: absent cells occupy no scroll,
    // and there is no dense array / special sparse encoding behind this.
    assert_eq!(cells.len(), 6, "only non-zero cells may occupy scrolls");
    assert_eq!(cells.len(), nonzeros().len());
    assert!(cells.len() < ROWS * COLS);
    // phokenize agrees once empty scrolls are filtered out.
    let tokenized: Vec<_> = phext::phokenize(&m)
        .into_iter()
        .filter(|ps| !ps.scroll.is_empty())
        .collect();
    assert_eq!(tokenized.len(), 6);
    // Every populated coordinate decodes back to a declared non-zero cell.
    for (coord, _) in &cells {
        assert_eq!(coord.x.scroll, 1);
        assert!(
            nonzeros().iter().any(|(r, c, _)| *r == row_of(coord) && *c == col_of(coord)),
            "populated scroll at {} is not a declared non-zero cell",
            coord
        );
    }
}

#[test]
fn fetch_by_coordinate_is_cell_lookup() {
    let m = fixture();
    for r in 0..ROWS {
        for c in 0..COLS {
            let got = phext::fetch(&m, cell_coord(r, c));
            let expected = nonzeros()
                .iter()
                .find(|(rr, cc, _)| *rr == r && *cc == c)
                .map(|(_, _, v)| *v)
                .unwrap_or("");
            assert_eq!(got, expected, "cell ({}, {}) lookup mismatch", r, c);
        }
    }
    // Spot-check the two identities a coordinate carries at once:
    // (0,4) is in row 0 AND column 4, fetched by the one coordinate.
    assert_eq!(phext::fetch(&m, cell_coord(0, 4)), "b");
    assert_eq!(phext::fetch(&m, cell_coord(3, 0)), "");
}

#[test]
fn inserting_one_cell_touches_one_coordinate() {
    let before_str = fixture();
    let before = populated(&before_str);

    // Insert a previously absent cell (1, 2).
    let after_str = phext::insert(before_str.clone(), cell_coord(1, 2), "new");
    let after = populated(&after_str);

    assert_eq!(phext::fetch(&after_str, cell_coord(1, 2)), "new");
    assert_eq!(after.len(), before.len() + 1, "exactly one new scroll");

    // Every pre-existing cell is byte-identical, and the only added
    // coordinate is the inserted cell's coordinate.
    for (coord, value) in &before {
        assert_eq!(after.get(coord), Some(value), "cell at {} changed", coord);
    }
    let added: Vec<_> = after.keys().filter(|k| !before.contains_key(*k)).collect();
    assert_eq!(added.len(), 1);
    assert_eq!(*added[0], cell_coord(1, 2));
    // No coordinate was removed.
    assert!(before.keys().all(|k| after.contains_key(k)));
}

#[test]
fn row_traversal_recovers_the_matrix() {
    let m = fixture();
    let cells = populated(&m);
    for r in 0..ROWS {
        // Row traversal = all scrolls sharing this row's chapter,
        // ordered by section (column).
        let mut row_cells: Vec<_> = cells
            .iter()
            .filter(|(coord, _)| row_of(coord) == r)
            .map(|(coord, v)| (col_of(coord), v.clone()))
            .collect();
        row_cells.sort_by_key(|(c, _)| *c);
        let expected: Vec<(usize, String)> = nonzeros()
            .iter()
            .filter(|(rr, _, _)| *rr == r)
            .map(|(_, c, v)| (*c, v.to_string()))
            .collect();
        assert_eq!(row_cells, expected, "row {} traversal mismatch", r);
        // The same row is also recoverable cell-by-cell via fetch.
        for (c, v) in &row_cells {
            assert_eq!(&phext::fetch(&m, cell_coord(r, *c)), v);
        }
    }
}

#[test]
fn column_traversal_recovers_the_matrix() {
    let m = fixture();
    let cells = populated(&m);
    for c in 0..COLS {
        // Column traversal = all scrolls sharing this column's section,
        // ordered by chapter (row) — the orthogonal direction.
        let mut col_cells: Vec<_> = cells
            .iter()
            .filter(|(coord, _)| col_of(coord) == c)
            .map(|(coord, v)| (row_of(coord), v.clone()))
            .collect();
        col_cells.sort_by_key(|(r, _)| *r);
        let expected: Vec<(usize, String)> = nonzeros()
            .iter()
            .filter(|(_, cc, _)| *cc == c)
            .map(|(rr, _, v)| (*rr, v.to_string()))
            .collect();
        assert_eq!(col_cells, expected, "column {} traversal mismatch", c);
        for (r, v) in &col_cells {
            assert_eq!(&phext::fetch(&m, cell_coord(*r, c)), v);
        }
    }
}

#[test]
fn row_and_column_traversals_reconstruct_the_same_dense_matrix() {
    let m = fixture();
    // Reconstruct a dense 4x5 grid purely from row traversal...
    let mut by_row = vec![vec![String::new(); COLS]; ROWS];
    for (coord, v) in populated(&m) {
        by_row[row_of(&coord)][col_of(&coord)] = v;
    }
    // ...and purely from column traversal (grouping by column first).
    let mut by_col = vec![vec![String::new(); COLS]; ROWS];
    let cells = populated(&m);
    for c in 0..COLS {
        for (coord, v) in cells.iter().filter(|(coord, _)| col_of(coord) == c) {
            by_col[row_of(coord)][c] = v.clone();
        }
    }
    assert_eq!(by_row, by_col, "orthogonal traversals must agree");

    // The reconstructed grid matches the declared matrix exactly,
    // with "" precisely where cells are absent.
    let mut expected = vec![vec![String::new(); COLS]; ROWS];
    for (r, c, v) in nonzeros() {
        expected[r][c] = v.to_string();
    }
    assert_eq!(by_row, expected);
}
