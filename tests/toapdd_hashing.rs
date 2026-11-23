// TOAPDD #18 — Knuth TAOCP §6.4 Hashing (WARNING / boundary test)
//
// This test must prove the REFUSAL, not force hashing into phext:
//   * A phext coordinate is an identity found from meaning: scrolls on one
//     topic sit at nearby coordinates, a different topic sits far away.
//   * checksum() (xxh3-128, 32 hex chars in libphext-rs src/phext.rs) is a
//     fingerprint: a one-word edit changes it completely, while the
//     coordinate — the identity — does not move at all.
//   * Two different texts at the same coordinate share the identity and
//     differ only in fingerprint. Identity != fingerprint.

use libphext::phext;

fn coord(address: &str) -> phext::Coordinate {
    phext::to_coordinate(address)
}

/// Component-wise Manhattan distance over all 9 coordinate fields.
fn distance(a: phext::Coordinate, b: phext::Coordinate) -> usize {
    let av = [
        a.z.library, a.z.shelf, a.z.series,
        a.y.collection, a.y.volume, a.y.book,
        a.x.chapter, a.x.section, a.x.scroll,
    ];
    let bv = [
        b.z.library, b.z.shelf, b.z.series,
        b.y.collection, b.y.volume, b.y.book,
        b.x.chapter, b.x.section, b.x.scroll,
    ];
    av.iter().zip(bv.iter()).map(|(x, y)| x.abs_diff(*y)).sum()
}

const GARDEN_A: &str = "1.1.1/1.1.1/1.1.1";
const GARDEN_B: &str = "1.1.1/1.1.1/1.1.2";
const GARDEN_C: &str = "1.1.1/1.1.1/1.2.1";
const FAR: &str = "1.1.1/1.1.1/9.1.1";

const GARDEN_A_TEXT: &str = "garden roses need water and sun";
const GARDEN_B_TEXT: &str = "garden tomatoes need water and sun";
const GARDEN_C_TEXT: &str = "garden herbs need shade and water";
const FAR_TEXT: &str = "orbital mechanics need thrust and timing";

fn fixture() -> String {
    let mut p = String::new();
    p = phext::insert(p, coord(GARDEN_A), GARDEN_A_TEXT);
    p = phext::insert(p, coord(GARDEN_B), GARDEN_B_TEXT);
    p = phext::insert(p, coord(GARDEN_C), GARDEN_C_TEXT);
    p = phext::insert(p, coord(FAR), FAR_TEXT);
    p
}

fn non_empty_coords(p: &str) -> Vec<phext::Coordinate> {
    phext::phokenize(p)
        .into_iter()
        .filter(|ps| !ps.scroll.is_empty())
        .map(|ps| ps.coord)
        .collect()
}

// Note: the neighbourhood here is BY CONSTRUCTION — this fixture assigns
// garden scrolls to nearby coordinates by hand, so the distance asserts
// check the fixture's geometry. What it demonstrates is that meaning-found
// addresses CAN cluster (a hash of the texts could not); the refusal
// itself is proved by the two checksum tests below.
#[test]
fn coordinate_identity_preserves_meaning_neighbourhood() {
    let p = fixture();

    // The fixture really holds exactly the four scrolls, at the
    // coordinates the meaning assigned them.
    let coords = non_empty_coords(&p);
    let expected = vec![coord(GARDEN_A), coord(GARDEN_B), coord(GARDEN_C), coord(FAR)];
    assert_eq!(coords, expected);
    assert_eq!(phext::fetch(&p, coord(GARDEN_A)), GARDEN_A_TEXT);
    assert_eq!(phext::fetch(&p, coord(FAR)), FAR_TEXT);

    // Neighbourhood: every pair inside the garden cluster (chapter 1)
    // is strictly nearer than every garden-to-far (chapter 9) pair.
    // A hash of these texts would scatter all four uniformly — no
    // neighbourhood would survive to assert on.
    let garden = [coord(GARDEN_A), coord(GARDEN_B), coord(GARDEN_C)];
    let far = coord(FAR);
    let mut max_within = 0usize;
    for i in 0..garden.len() {
        for j in (i + 1)..garden.len() {
            max_within = max_within.max(distance(garden[i], garden[j]));
        }
    }
    let mut min_to_far = usize::MAX;
    for g in garden {
        min_to_far = min_to_far.min(distance(g, far));
    }
    // Fixture geometry, stated exactly: within-cluster max is 2
    // (1.1.2 vs 1.2.1 in section/scroll), nearest far distance is 8
    // (the chapter gap 1 -> 9 alone).
    assert_eq!(max_within, 2);
    assert_eq!(min_to_far, 8);
    assert!(max_within < min_to_far,
        "near meanings must be neighbours: within={max_within} to_far={min_to_far}");
}

#[test]
fn one_word_edit_keeps_coordinate_but_changes_checksum_completely() {
    let p = fixture();
    let target = coord(GARDEN_A);
    let before_checksum = phext::checksum(&p);

    // Edit exactly ONE word: roses -> tulips.
    let edited_text = GARDEN_A_TEXT.replace("roses", "tulips");
    assert_ne!(edited_text, GARDEN_A_TEXT);
    let edited = phext::replace(&p, target, &edited_text);

    // Identity survived the edit: same coordinates, in the same order,
    // and the edited text is fetched at the SAME coordinate.
    assert_eq!(phext::fetch(&edited, target), edited_text);
    assert_eq!(non_empty_coords(&edited), non_empty_coords(&p));

    // The fingerprint did not survive — and it did not change a little,
    // it changed completely. checksum() is xxh3-128 rendered as 32
    // lowercase hex chars (verified in src/phext.rs).
    let after_checksum = phext::checksum(&edited);
    println!("checksum before one-word edit: {before_checksum}");
    println!("checksum after  one-word edit: {after_checksum}");
    assert_eq!(before_checksum.len(), 32);
    assert_eq!(after_checksum.len(), 32);
    assert_ne!(before_checksum, after_checksum);
    let differing = before_checksum.chars()
        .zip(after_checksum.chars())
        .filter(|(a, b)| a != b)
        .count();
    let common_prefix = before_checksum.chars()
        .zip(after_checksum.chars())
        .take_while(|(a, b)| a == b)
        .count();
    println!("differing hex chars: {differing}/32, common prefix: {common_prefix}");
    // A majority of the 32 hex characters differ, and there is no long
    // shared prefix — the new fingerprint carries no trace of locality
    // from the old one, unlike the coordinate, which carried all of it.
    assert!(differing > 16, "majority of checksum chars must differ, got {differing}/32");
    assert!(common_prefix <= 2, "no long common prefix expected, got {common_prefix}");
}

#[test]
fn same_coordinate_different_text_same_identity_different_fingerprint() {
    let target = coord(GARDEN_A);
    let text1 = "garden roses need water and sun";
    let text2 = "orbital mechanics need thrust and timing";

    let p1 = phext::insert(String::new(), target, text1);
    let p2 = phext::insert(String::new(), target, text2);

    // Same identity: phokenize places both texts at the identical
    // coordinate, and fetch resolves each at that coordinate.
    assert_eq!(non_empty_coords(&p1), vec![target]);
    assert_eq!(non_empty_coords(&p2), vec![target]);
    assert_eq!(non_empty_coords(&p1), non_empty_coords(&p2));
    assert_eq!(phext::fetch(&p1, target), text1);
    assert_eq!(phext::fetch(&p2, target), text2);

    // Different fingerprints, at both whole-phext and scroll level.
    // If checksum were the identity, these two records could not be
    // told apart from two different records — or, read the other way,
    // the edited record in the previous test would be a new record.
    // Both readings are the refusal: a fingerprint is never an identity.
    assert_ne!(phext::checksum(&p1), phext::checksum(&p2));
    assert_ne!(phext::checksum(text1), phext::checksum(text2));
    println!("p1 checksum: {}", phext::checksum(&p1));
    println!("p2 checksum: {}", phext::checksum(&p2));
}
