// TOAPDD #9 — Generating basic combinatorial patterns
// (Knuth, TAOCP Vol. 4A, §7.2.1, esp. 7.2.1.1 n-tuples, 7.2.1.2 permutations)
// Top-20 entry #9.
//
// Phext-native claim: if each generated object is stored at the coordinate
// its generation path implies, then enumeration, resumption mid-space, and
// splitting the space are the same operation. Knuth gives the visiting
// order; phext gives the visited points addresses that persist after the
// visit.
//
// Driver/library split, stated plainly: the generation drivers below
// (Lehmer-code ranks, tuple enumeration) are code in this file. What
// libphext contributes — and what the tests pin — is that phokenize
// traversal yields the stored objects in coordinate order, that the
// coordinate order equals the generation order under these encodings,
// and that insert()-ing later objects never disturbs earlier scrolls,
// so resumption is a coordinate filter, not a replay.
//
// Encodings (documented, and verified by the tests below):
//
// * Permutations of [0,1,2,3], lexicographic order. The generation path is
//   the Lehmer code (factoradic) d0,d1,d2,d3: at step i, d_i is the index
//   of the chosen element among the still-unused elements in sorted order
//   (d3 is always 0). Lexicographic order on permutations is lexicographic
//   order on their Lehmer codes. Coordinate:
//       1.1.1 / 1.1.1 / (d0+1).(d1+1).(d2+1)
//   i.e. chapter=d0+1, section=d1+1, scroll=d2+1, all other components 1.
//   Coordinate Ord compares z, then y, then x (chapter, then section, then
//   scroll), so coordinate order == Lehmer lex order == permutation lex
//   order. The rank is recoverable from the coordinate alone:
//       rank = d0*3! + d1*2! + d2*1!
//   so a checkpoint coordinate decodes to exactly where generation stopped.
//
// * Binary n-tuples (a,b,c,d) in {0,1}^4, lexicographic order. Four path
//   digits need four successive components in Ord order:
//       1.1.1 / 1.1.(a+1) / (b+1).(c+1).(d+1)
//   i.e. book=a+1, chapter=b+1, section=c+1, scroll=d+1. Ord compares book
//   before chapter before section before scroll, so coordinate order ==
//   tuple lex order. Rank = a*8 + b*4 + c*2 + d, from the coordinate alone.
//
// * Ternary n-tuples (a,b,c) in {0,1,2}^3, lexicographic order:
//       1.1.1 / 1.1.1 / (a+1).(b+1).(c+1)
//   Rank = a*9 + b*3 + c, from the coordinate alone.
//
// Public API only: libphext::phext.

use libphext::phext;
use std::collections::{HashMap, HashSet};

fn coord(s: &str) -> phext::Coordinate {
    phext::to_coordinate(s)
}

fn scrolls_in_order(source: &str) -> Vec<(phext::Coordinate, String)> {
    phext::phokenize(source)
        .into_iter()
        .filter(|ps| !ps.scroll.is_empty())
        .map(|ps| (ps.coord, ps.scroll))
        .collect()
}

fn build(entries: &[(phext::Coordinate, String)]) -> String {
    let mut map: HashMap<phext::Coordinate, String> = HashMap::new();
    for (c, s) in entries {
        map.insert(*c, s.clone());
    }
    phext::implode(map)
}

fn assert_no_repeats(items: &[(phext::Coordinate, String)], what: &str) {
    let coords: HashSet<_> = items.iter().map(|(c, _)| *c).collect();
    let texts: HashSet<_> = items.iter().map(|(_, s)| s.clone()).collect();
    assert_eq!(coords.len(), items.len(), "{}: duplicate coordinates", what);
    assert_eq!(texts.len(), items.len(), "{}: duplicate objects", what);
}

// ---------------------------------------------------------------------------
// Permutation generation (driver): rank -> (permutation, Lehmer code)
// ---------------------------------------------------------------------------

fn perm_at_rank(mut rank: usize) -> (Vec<usize>, [usize; 4]) {
    assert!(rank < 24);
    let mut remaining = vec![0usize, 1, 2, 3];
    let mut perm = Vec::new();
    let mut lehmer = [0usize; 4];
    for i in 0..4 {
        let f = (1..=(3 - i)).product::<usize>(); // (3-i)!
        let idx = rank / f;
        rank %= f;
        lehmer[i] = idx;
        perm.push(remaining.remove(idx));
    }
    (perm, lehmer)
}

fn perm_coord(lehmer: &[usize; 4]) -> phext::Coordinate {
    assert_eq!(lehmer[3], 0, "last Lehmer digit of a 4-permutation is 0");
    coord(&format!(
        "1.1.1/1.1.1/{}.{}.{}",
        lehmer[0] + 1,
        lehmer[1] + 1,
        lehmer[2] + 1
    ))
}

/// Decode a permutation coordinate back to (Lehmer code, rank).
fn perm_rank_from_coord(c: phext::Coordinate) -> ([usize; 4], usize) {
    let lehmer = [
        c.x.chapter - 1,
        c.x.section - 1,
        c.x.scroll - 1,
        0usize,
    ];
    let rank = lehmer[0] * 6 + lehmer[1] * 2 + lehmer[2];
    (lehmer, rank)
}

fn perm_text(perm: &[usize]) -> String {
    perm.iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn all_permutations() -> Vec<(phext::Coordinate, String)> {
    (0..24)
        .map(|r| {
            let (perm, lehmer) = perm_at_rank(r);
            (perm_coord(&lehmer), perm_text(&perm))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// 1. All 24 permutations of 4, in lex order, at their generation-path
//    coordinates; phokenize traversal yields exactly the generation order,
//    and each coordinate decodes back to the object it addresses.
// ---------------------------------------------------------------------------
#[test]
fn permutations_of_4_at_generation_path_coordinates() {
    let expected = all_permutations();

    // The driver really generates lexicographic order.
    let mut sorted = expected.clone();
    sorted.sort_by(|a, b| a.1.cmp(&b.1));
    // (lex order on "0,1,2,3"-style strings coincides with lex order on the
    // digit sequences here because every digit is a single character)
    assert_eq!(expected, sorted, "generation order must be lexicographic");

    // Coordinate order == generation order, by construction of the encoding.
    let coords: Vec<_> = expected.iter().map(|(c, _)| *c).collect();
    let mut coords_sorted = coords.clone();
    coords_sorted.sort();
    assert_eq!(coords, coords_sorted, "coordinate Ord must match generation order");
    for w in coords.windows(2) {
        assert!(w[0] < w[1], "coordinates strictly increase along the walk");
    }

    let source = build(&expected);

    // The stored set has exactly 24 objects: coordinates are unique keys,
    // so a duplicate generation could not hide as a second copy.
    let map = phext::explode(&source);
    let nonempty: HashMap<_, _> = map
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| (*k, v.clone()))
        .collect();
    assert_eq!(nonempty.len(), 24, "exactly 24 permutation scrolls stored");

    // Traversal yields exactly the 24 objects, in generation order.
    let walked = scrolls_in_order(&source);
    assert_eq!(walked, expected, "phokenize order must be generation order");
    assert_no_repeats(&walked, "permutations");

    // Every object is fetchable at its path coordinate, and the coordinate
    // alone decodes (via its Lehmer digits) back to that object and rank.
    for (rank, (c, text)) in expected.iter().enumerate() {
        assert_eq!(phext::fetch(&source, *c), *text);
        let (lehmer, decoded_rank) = perm_rank_from_coord(*c);
        assert_eq!(decoded_rank, rank, "coordinate must encode its rank");
        let (perm, lehmer2) = perm_at_rank(rank);
        assert_eq!(lehmer, lehmer2);
        assert_eq!(perm_text(&perm), *text);
    }

    // Spot-check the ends of the space.
    assert_eq!(expected[0].1, "0,1,2,3");
    assert_eq!(expected[0].0, coord("1.1.1/1.1.1/1.1.1"));
    assert_eq!(expected[23].1, "3,2,1,0");
    assert_eq!(expected[23].0, coord("1.1.1/1.1.1/4.3.2"));
}

// ---------------------------------------------------------------------------
// 2. Resume by coordinate filter: from the 10th object's coordinate,
//    enumerate only scrolls at greater coordinates — exactly objects
//    11..24, no repeats, none missing.
// ---------------------------------------------------------------------------
#[test]
fn resume_from_10th_permutation_yields_11_to_24() {
    let expected = all_permutations();
    let source = build(&expected);
    let walked = scrolls_in_order(&source);

    let checkpoint = walked[9].0; // the 10th object (rank 9)
    let (_, checkpoint_rank) = perm_rank_from_coord(checkpoint);
    assert_eq!(checkpoint_rank, 9);

    let resumed: Vec<_> = walked
        .iter()
        .filter(|(c, _)| *c > checkpoint)
        .cloned()
        .collect();

    assert_eq!(resumed.len(), 14, "exactly 14 objects remain after the 10th");
    assert_eq!(resumed, expected[10..].to_vec(), "resume yields objects 11..24 in order");
    assert_no_repeats(&resumed, "resumed permutations");

    // Disjoint from the visited prefix; union is the whole space.
    let visited: HashSet<_> = walked[..10].iter().map(|(_, s)| s.clone()).collect();
    for (_, s) in &resumed {
        assert!(!visited.contains(s), "resumed object must not repeat a visited one: {}", s);
    }
    let union: HashSet<_> = walked[..10]
        .iter()
        .chain(resumed.iter())
        .map(|(_, s)| s.clone())
        .collect();
    assert_eq!(union.len(), 24, "visited + resumed = the full space, nothing missing");
}

// ---------------------------------------------------------------------------
// 3. Resume by generation: stop after 10 objects, keep only the partial
//    phext and the checkpoint coordinate, decode the rank from the
//    coordinate, and generate exactly the remaining objects into the same
//    phext. The first 10 scrolls are never regenerated or touched.
// ---------------------------------------------------------------------------
#[test]
fn resume_generation_from_checkpoint_coordinate() {
    let expected = all_permutations();

    // Partial run: only the first 10 objects exist.
    let mut source = build(&expected[..10]);
    assert_eq!(scrolls_in_order(&source).len(), 10);

    let checkpoint = expected[9].0;
    assert_eq!(phext::fetch(&source, checkpoint), expected[9].1);

    // The checkpoint coordinate alone says where to resume.
    let (_, rank) = perm_rank_from_coord(checkpoint);
    assert_eq!(rank, 9);

    // Generate only ranks 10..24 and insert each at its path coordinate.
    for r in (rank + 1)..24 {
        let (perm, lehmer) = perm_at_rank(r);
        assert_eq!(perm_coord(&lehmer), expected[r].0);
        source = phext::insert(source, perm_coord(&lehmer), &perm_text(&perm));
    }

    // The completed phext is exactly the full space, in order, and the
    // originally generated prefix survived resumption byte-for-byte.
    assert_eq!(scrolls_in_order(&source), expected);
    for (c, text) in &expected[..10] {
        assert_eq!(phext::fetch(&source, *c), *text, "prefix scroll untouched by resume");
    }
    assert_no_repeats(&scrolls_in_order(&source), "resumed generation");
}

// ---------------------------------------------------------------------------
// 4. Binary n-tuples: all 2^4 tuples at path coordinates
//    (book, chapter, section, scroll) = (a+1, b+1, c+1, d+1), with the same
//    mid-space resume property at the 6th object.
// ---------------------------------------------------------------------------

fn binary_coord(t: &[usize; 4]) -> phext::Coordinate {
    coord(&format!(
        "1.1.1/1.1.{}/{}.{}.{}",
        t[0] + 1,
        t[1] + 1,
        t[2] + 1,
        t[3] + 1
    ))
}

fn binary_rank_from_coord(c: phext::Coordinate) -> ([usize; 4], usize) {
    let t = [
        c.y.book - 1,
        c.x.chapter - 1,
        c.x.section - 1,
        c.x.scroll - 1,
    ];
    (t, t[0] * 8 + t[1] * 4 + t[2] * 2 + t[3])
}

#[test]
fn binary_tuples_2_to_4_at_path_coordinates_with_resume() {
    let mut expected: Vec<(phext::Coordinate, String)> = Vec::new();
    for a in 0..2 {
        for b in 0..2 {
            for c_ in 0..2 {
                for d in 0..2 {
                    let t = [a, b, c_, d];
                    expected.push((binary_coord(&t), format!("{},{},{},{}", a, b, c_, d)));
                }
            }
        }
    }
    assert_eq!(expected.len(), 16);

    let coords: Vec<_> = expected.iter().map(|(c, _)| *c).collect();
    let mut coords_sorted = coords.clone();
    coords_sorted.sort();
    assert_eq!(coords, coords_sorted, "tuple coordinate order must be lex order");

    let source = build(&expected);
    let map = phext::explode(&source);
    let nonempty = map.values().filter(|v| !v.is_empty()).count();
    assert_eq!(nonempty, 16, "exactly 16 binary tuple scrolls stored");

    let walked = scrolls_in_order(&source);
    assert_eq!(walked, expected, "traversal yields tuples in lex generation order");
    assert_no_repeats(&walked, "binary tuples");

    // Each coordinate decodes back to its tuple and rank.
    for (rank, (c, text)) in expected.iter().enumerate() {
        assert_eq!(phext::fetch(&source, *c), *text);
        let (t, decoded) = binary_rank_from_coord(*c);
        assert_eq!(decoded, rank);
        assert_eq!(format!("{},{},{},{}", t[0], t[1], t[2], t[3]), *text);
    }

    // Resume from the 6th object (rank 5): exactly the remaining 10.
    let checkpoint = walked[5].0;
    let (_, checkpoint_rank) = binary_rank_from_coord(checkpoint);
    assert_eq!(checkpoint_rank, 5);
    let resumed: Vec<_> = walked.iter().filter(|(c, _)| *c > checkpoint).cloned().collect();
    assert_eq!(resumed.len(), 10);
    assert_eq!(resumed, expected[6..].to_vec());
    assert_no_repeats(&resumed, "resumed binary tuples");
}

// ---------------------------------------------------------------------------
// 5. Ternary n-tuples: all 3^3 tuples at path coordinates
//    (chapter, section, scroll) = (a+1, b+1, c+1), resuming at the 14th.
// ---------------------------------------------------------------------------

fn ternary_coord(t: &[usize; 3]) -> phext::Coordinate {
    coord(&format!(
        "1.1.1/1.1.1/{}.{}.{}",
        t[0] + 1,
        t[1] + 1,
        t[2] + 1
    ))
}

fn ternary_rank_from_coord(c: phext::Coordinate) -> ([usize; 3], usize) {
    let t = [c.x.chapter - 1, c.x.section - 1, c.x.scroll - 1];
    (t, t[0] * 9 + t[1] * 3 + t[2])
}

#[test]
fn ternary_tuples_3_to_3_at_path_coordinates_with_resume() {
    let mut expected: Vec<(phext::Coordinate, String)> = Vec::new();
    for a in 0..3 {
        for b in 0..3 {
            for c_ in 0..3 {
                let t = [a, b, c_];
                expected.push((ternary_coord(&t), format!("{},{},{}", a, b, c_)));
            }
        }
    }
    assert_eq!(expected.len(), 27);

    let coords: Vec<_> = expected.iter().map(|(c, _)| *c).collect();
    let mut coords_sorted = coords.clone();
    coords_sorted.sort();
    assert_eq!(coords, coords_sorted, "ternary coordinate order must be lex order");

    let source = build(&expected);
    let map = phext::explode(&source);
    let nonempty = map.values().filter(|v| !v.is_empty()).count();
    assert_eq!(nonempty, 27, "exactly 27 ternary tuple scrolls stored");

    let walked = scrolls_in_order(&source);
    assert_eq!(walked, expected, "traversal yields tuples in lex generation order");
    assert_no_repeats(&walked, "ternary tuples");

    for (rank, (c, text)) in expected.iter().enumerate() {
        assert_eq!(phext::fetch(&source, *c), *text);
        let (t, decoded) = ternary_rank_from_coord(*c);
        assert_eq!(decoded, rank);
        assert_eq!(format!("{},{},{}", t[0], t[1], t[2]), *text);
    }

    // Resume from the 14th object (rank 13): exactly the remaining 13.
    let checkpoint = walked[13].0;
    let (_, checkpoint_rank) = ternary_rank_from_coord(checkpoint);
    assert_eq!(checkpoint_rank, 13);
    let resumed: Vec<_> = walked.iter().filter(|(c, _)| *c > checkpoint).cloned().collect();
    assert_eq!(resumed.len(), 13);
    assert_eq!(resumed, expected[14..].to_vec());
    assert_no_repeats(&resumed, "resumed ternary tuples");

    // Splitting the space is the same operation: the two halves are the
    // scrolls at coordinates <= checkpoint and > checkpoint, and their
    // merge (union of the two stored sets) is the whole space.
    let first_half: Vec<_> = walked.iter().filter(|(c, _)| *c <= checkpoint).cloned().collect();
    assert_eq!(first_half.len(), 14);
    assert_eq!(first_half, expected[..14].to_vec());
    let mut halves = first_half.clone();
    halves.extend(resumed.iter().cloned());
    assert_eq!(halves, expected, "the two halves partition the space exactly");
}
