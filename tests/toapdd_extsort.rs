// TOAPDD #19 — Knuth TAOCP §5.4 External Sorting (WARNING / boundary test)
//
// §5.4 is scarcity engineering: sequential tapes/disks, memory too small for
// the data, so records are formed into runs and merged in k-way / polyphase
// passes over external media. This test is NOT a sorting implementation
// contest. It shows the discipline dissolving under abundance:
//
//   * On addressable, hostable text, a record is inserted directly at a
//     coordinate that encodes its sorted position (fixture 1: its rank) or
//     its key itself (fixture 2).
//   * Ordering is then a coordinate traversal (phokenize), not a merge.
//   * There is no tape, no run generation, no polyphase merge, and no
//     merging code anywhere in this file: zero merge passes is true by
//     construction, so no assertion is wasted on it — sortedness of the
//     FIRST traversal is the claim, and it is asserted directly.
//
// Honesty note (fixture 1): the driver does ONE in-memory sort to assign
// rank coordinates. That is exactly the point — address assignment replaces
// merge passes over external media. The phext artefact itself never sorts,
// never forms runs, and never merges. Fixture 2 needs not even that: the
// coordinate is derived from the key alone, so a late record can be inserted
// at its key coordinate with no re-placement of any existing scroll.

use libphext::phext;

fn coord_scroll(n: usize) -> phext::Coordinate {
    // 1.1.1/1.1.1/1.1.<n>  (n >= 1)
    phext::to_coordinate(&format!("1.1.1/1.1.1/1.1.{}", n))
}

fn non_empty_traversal(p: &str) -> Vec<String> {
    phext::phokenize(p)
        .into_iter()
        .filter(|ps| !ps.scroll.is_empty())
        .map(|ps| ps.scroll)
        .collect()
}

// ---------------------------------------------------------------------------
// Fixture 1: ~40 textual records, rank coordinates, shuffled insertion
// ---------------------------------------------------------------------------

fn fixture1_keys() -> Vec<&'static str> {
    // Deliberately unsorted source list (the "external file" as it arrives).
    vec![
        "mango", "apple", "zebra", "kiwi", "banana", "plum", "grape", "lemon",
        "peach", "melon", "cherry", "fig", "date", "olive", "quince", "pear",
        "apricot", "berry", "citron", "damson", "elder", "guava", "honeydew",
        "imbe", "jujube", "kumquat", "lime", "nectarine", "orange", "papaya",
        "rambutan", "sloe", "tangelo", "ugli", "vanilla", "walnut", "xigua",
        "yam", "ziziphus", "almond",
    ]
}

// Fixed shuffled insertion order (indices into the sorted list), no RNG.
const SHUFFLE1: [usize; 40] = [
    17, 3, 39, 0, 22, 11, 35, 7, 28, 14, 1, 31, 9, 25, 19, 5, 37, 13, 29, 2,
    33, 20, 8, 26, 15, 38, 6, 23, 12, 30, 18, 4, 34, 10, 27, 21, 36, 16, 32,
    24,
];

fn build_fixture1() -> (String, Vec<String>) {
    let mut sorted: Vec<String> =
        fixture1_keys().iter().map(|s| s.to_string()).collect();
    // The driver's ONE in-memory sort — used only to assign addresses.
    sorted.sort();
    let mut p = String::new();
    for &idx in SHUFFLE1.iter() {
        // Sorted rank (0-based) -> scroll coordinate rank+1.
        p = phext::insert(p, coord_scroll(idx + 1), &sorted[idx]);
    }
    (p, sorted)
}

#[test]
fn rank_coordinates_yield_sorted_order_with_zero_merge_passes() {
    let (p, sorted) = build_fixture1();

    // First (and only) traversal must already be sorted.
    let got = non_empty_traversal(&p);
    assert_eq!(got.len(), 40, "traversal must yield every record");
    assert_eq!(got, sorted, "phokenize order must equal sorted key order");
    for w in got.windows(2) {
        assert!(w[0] < w[1], "strictly sorted: {:?} < {:?}", w[0], w[1]);
    }
    // Each record is fetchable at exactly its rank coordinate.
    for (i, key) in sorted.iter().enumerate() {
        assert_eq!(&phext::fetch(&p, coord_scroll(i + 1)), key);
    }
}

// ---------------------------------------------------------------------------
// Fixture 2: numeric keys 0..999, key-derived coordinates, late insert
// ---------------------------------------------------------------------------

// 40 numeric keys scattered in 0..=999, in a fixed shuffled insertion order.
const KEYS2_SHUFFLED: [u32; 40] = [
    500, 7, 999, 0, 255, 42, 731, 100, 3, 888, 64, 420, 19, 950, 128, 600,
    31, 777, 200, 55, 901, 11, 345, 80, 666, 23, 470, 150, 820, 5, 290, 97,
    540, 38, 710, 175, 860, 49, 615, 260,
];
const EXTRA_KEY: u32 = 123; // not in KEYS2_SHUFFLED, lies mid-range

fn key_coord(key: u32) -> phext::Coordinate {
    coord_scroll((key + 1) as usize) // key 0 -> scroll 1
}

fn build_fixture2() -> String {
    let mut p = String::new();
    for &k in KEYS2_SHUFFLED.iter() {
        p = phext::insert(p, key_coord(k), &k.to_string());
    }
    p
}

fn sorted_keys2() -> Vec<u32> {
    let mut v: Vec<u32> = KEYS2_SHUFFLED.to_vec();
    v.sort_unstable();
    v
}

#[test]
fn key_derived_coordinates_yield_sorted_order_with_zero_merge_passes() {
    let p = build_fixture2();
    let got: Vec<u32> = non_empty_traversal(&p)
        .iter()
        .map(|s| s.parse::<u32>().unwrap())
        .collect();
    assert_eq!(got.len(), 40);
    assert_eq!(got, sorted_keys2(), "traversal must be sorted key order");
    for w in got.windows(2) {
        assert!(w[0] < w[1]);
    }
}

#[test]
fn late_insert_at_key_coordinate_replaces_nothing() {
    let p = build_fixture2();
    let before = phext::explode(&p);

    // Insert ONE more record, afterwards, at its own key coordinate.
    let p2 = phext::insert(p, key_coord(EXTRA_KEY), &EXTRA_KEY.to_string());
    let after = phext::explode(&p2);

    // Explode-diff: every prior non-empty coordinate is unchanged.
    for (coord, scroll) in before.iter() {
        if scroll.is_empty() {
            continue;
        }
        assert_eq!(
            after.get(coord),
            Some(scroll),
            "prior scroll at {} must be unchanged",
            coord
        );
    }
    assert_eq!(
        phext::fetch(&p2, key_coord(EXTRA_KEY)),
        EXTRA_KEY.to_string()
    );

    // Traversal is still sorted, now with 41 records, still zero passes.
    let got: Vec<u32> = non_empty_traversal(&p2)
        .iter()
        .map(|s| s.parse::<u32>().unwrap())
        .collect();
    let mut expect = sorted_keys2();
    expect.push(EXTRA_KEY);
    expect.sort_unstable();
    assert_eq!(got.len(), 41);
    assert_eq!(got, expect);
}
