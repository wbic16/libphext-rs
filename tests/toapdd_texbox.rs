//! TOAPDD #13 — Knuth, The TeXbook Ch. 11 Boxes, Ch. 12 Glue,
//! Ch. 14 How TeX Breaks Paragraphs into Lines (Reframe tier).
//!
//! Phext-native claim (ASSIGNMENTS.md, Wave 2 #13):
//!   Box tree as nested coordinates; two candidate line-breakings
//!   stored as parallel scrolls; the chosen breaking is a link
//!   scroll; switching the choice touches only the link scroll.
//!
//! ENCODING (documented, this test is its specification)
//! -----------------------------------------------------
//! All coordinates share the prefix 1.1.1/1.1.1 (library..book = 1)
//! and differ only in the X arm (chapter.section.scroll):
//!
//!   chapter 1                the paragraph box tree. Chapter is the
//!                            parent prefix: every box and glue
//!                            scroll of this paragraph lives under
//!                            chapter 1, so "is inside the paragraph
//!                            box" is literally "chapter == 1".
//!     section 1, scroll 1    the paragraph (vbox) root scroll.
//!     section 2, scroll n    word hbox n, n = 1..=6, successive
//!                            scrolls in reading order. Scroll
//!                            content records the word and its
//!                            natural width.
//!     section 3, scroll n    glue after word n, n = 1..=5, in the
//!                            TeXbook's own phrasing:
//!                            "glue 10 plus 5 minus 2" (natural
//!                            width, stretch, shrink).
//!     section 10, scroll 1   candidate breaking A (parallel region).
//!     section 11, scroll 1   candidate breaking B (parallel region).
//!   chapter 1, section 20, scroll 1
//!                            the LINK scroll: its entire content is
//!                            the coordinate string of the currently
//!                            chosen breaking scroll.
//!
//! A breaking scroll lists, per line, the *coordinates* of the word
//! hboxes on that line — not copies of the words. Reading the
//! current typeset paragraph is therefore a driver walk: fetch the
//! link, fetch the breaking it points at, fetch each word scroll
//! named there, in order.
//!
//! CLASS D BOUNDARY (deliberate, per the Top 20 list, entry #13):
//!   The numeric badness / demerits optimisation is NOT implemented
//!   here and is not faked. No code in this file computes badness,
//!   demerits, or an optimum, and the driver picks nothing
//!   numerically: it follows whichever breaking the link scroll
//!   names. Phext stores the box/glue tree and the candidate
//!   breakings as addressable, inspectable scrolls; the arithmetic
//!   that would rank the candidates stays numeric, outside phext.
//!
//! Public API only: libphext::phext::{insert, fetch, replace,
//! explode, to_coordinate, Coordinate}.

use libphext::phext;
use std::collections::HashMap;

// ------------------------------------------------------------------
// Coordinates
// ------------------------------------------------------------------

fn coord(chapter: usize, section: usize, scroll: usize) -> phext::Coordinate {
    phext::to_coordinate(&format!("1.1.1/1.1.1/{chapter}.{section}.{scroll}"))
}

fn paragraph_root() -> phext::Coordinate {
    coord(1, 1, 1)
}
fn word_box(n: usize) -> phext::Coordinate {
    coord(1, 2, n)
}
fn glue(n: usize) -> phext::Coordinate {
    coord(1, 3, n)
}
fn breaking_a() -> phext::Coordinate {
    coord(1, 10, 1)
}
fn breaking_b() -> phext::Coordinate {
    coord(1, 11, 1)
}
fn link_scroll() -> phext::Coordinate {
    coord(1, 20, 1)
}

fn coord_string(c: phext::Coordinate) -> String {
    format!(
        "{}.{}.{}/{}.{}.{}/{}.{}.{}",
        c.z.library, c.z.shelf, c.z.series, c.y.collection, c.y.volume, c.y.book, c.x.chapter,
        c.x.section, c.x.scroll
    )
}

// ------------------------------------------------------------------
// Fixture: one paragraph, six word hboxes, five glues
// ------------------------------------------------------------------

const WORDS: &[(&str, usize)] = &[
    ("the", 18),
    ("quick", 30),
    ("brown", 32),
    ("fox", 20),
    ("jumps", 34),
    ("over", 26),
];

const BREAKING_A: &str = "breaking A\n\
line 1: 1.1.1/1.1.1/1.2.1 1.1.1/1.1.1/1.2.2 1.1.1/1.1.1/1.2.3\n\
line 2: 1.1.1/1.1.1/1.2.4 1.1.1/1.1.1/1.2.5 1.1.1/1.1.1/1.2.6";

const BREAKING_B: &str = "breaking B\n\
line 1: 1.1.1/1.1.1/1.2.1 1.1.1/1.1.1/1.2.2\n\
line 2: 1.1.1/1.1.1/1.2.3 1.1.1/1.1.1/1.2.4\n\
line 3: 1.1.1/1.1.1/1.2.5 1.1.1/1.1.1/1.2.6";

/// Build the full phext, inserting in ascending coordinate order.
fn build() -> String {
    let mut doc = String::new();
    doc = phext::insert(
        doc,
        paragraph_root(),
        "box: paragraph\nchildren: words at 1.2.*, glue at 1.3.*",
    );
    for (i, (word, width)) in WORDS.iter().enumerate() {
        doc = phext::insert(
            doc,
            word_box(i + 1),
            &format!("hbox: word={word} width={width}"),
        );
    }
    for n in 1..WORDS.len() {
        doc = phext::insert(doc, glue(n), "glue 10 plus 5 minus 2");
    }
    doc = phext::insert(doc, breaking_a(), BREAKING_A);
    doc = phext::insert(doc, breaking_b(), BREAKING_B);
    // The choice starts at breaking A; the link scroll holds only
    // the chosen breaking's coordinate.
    doc = phext::insert(doc, link_scroll(), &coord_string(breaking_a()));
    doc
}

fn nonempty(doc: &str) -> HashMap<phext::Coordinate, String> {
    phext::explode(doc)
        .into_iter()
        .filter(|(_, v)| !v.is_empty())
        .collect()
}

// ------------------------------------------------------------------
// Driver: follow the link, then the breaking, then the word boxes
// ------------------------------------------------------------------

/// Word coordinates named by a breaking scroll, line by line.
fn breaking_lines(breaking: &str) -> Vec<Vec<phext::Coordinate>> {
    breaking
        .lines()
        .filter(|l| l.starts_with("line "))
        .map(|l| {
            let (_, coords) = l.split_once(':').expect("line has a colon");
            coords
                .split_whitespace()
                .map(phext::to_coordinate)
                .collect()
        })
        .collect()
}

fn word_of(scroll: &str) -> String {
    // "hbox: word=the width=18" -> "the"
    let after = scroll.split("word=").nth(1).expect("hbox names a word");
    after.split_whitespace().next().expect("word text").to_string()
}

/// The current typeset lines: link -> breaking -> word scrolls.
/// This driver makes no numeric choice of its own (Class D boundary
/// above); it reads whichever breaking the link scroll names.
fn current_lines(doc: &str) -> Vec<Vec<String>> {
    let target = phext::fetch(doc, link_scroll());
    assert!(!target.is_empty(), "link scroll must be present");
    let breaking_coord = phext::to_coordinate(target.trim());
    let breaking = phext::fetch(doc, breaking_coord);
    assert!(!breaking.is_empty(), "link must point at a real breaking");
    breaking_lines(&breaking)
        .iter()
        .map(|line| {
            line.iter()
                .map(|c| word_of(&phext::fetch(doc, *c)))
                .collect()
        })
        .collect()
}

// ------------------------------------------------------------------
// Tests
// ------------------------------------------------------------------

#[test]
fn box_tree_is_nested_by_coordinate_prefix() {
    let doc = build();
    let map = nonempty(&doc);

    // Root + 6 word hboxes + 5 glues + 2 breakings + 1 link = 15.
    assert_eq!(map.len(), 15);

    // The paragraph root is the box; its fetch is exact.
    assert_eq!(
        phext::fetch(&doc, paragraph_root()),
        "box: paragraph\nchildren: words at 1.2.*, glue at 1.3.*"
    );

    // Nesting is a coordinate prefix: every box/glue scroll of the
    // paragraph shares the paragraph's chapter (and all higher
    // components), i.e. parent coordinate prefix = box nesting.
    for c in map.keys() {
        assert_eq!(c.z.library, 1);
        assert_eq!(c.z.shelf, 1);
        assert_eq!(c.z.series, 1);
        assert_eq!(c.y.collection, 1);
        assert_eq!(c.y.volume, 1);
        assert_eq!(c.y.book, 1);
        assert_eq!(c.x.chapter, 1, "every scroll is inside chapter 1 (the paragraph)");
    }

    // Word hboxes sit at successive coordinates in reading order and
    // fetch their word + natural width.
    for (i, (word, width)) in WORDS.iter().enumerate() {
        let c = word_box(i + 1);
        assert_eq!(c.x.section, 2);
        assert_eq!(c.x.scroll, i + 1);
        assert_eq!(
            phext::fetch(&doc, c),
            format!("hbox: word={word} width={width}")
        );
    }

    // Glue scrolls sit between the words they separate (glue n is
    // after word n) and record natural width + stretch + shrink in
    // the TeXbook's phrasing.
    for n in 1..WORDS.len() {
        let c = glue(n);
        assert_eq!(c.x.section, 3);
        assert_eq!(c.x.scroll, n);
        assert_eq!(phext::fetch(&doc, c), "glue 10 plus 5 minus 2");
    }
    // There is no glue after the last word.
    assert_eq!(phext::fetch(&doc, glue(WORDS.len())), "");
}

#[test]
fn both_candidate_breakings_coexist_and_fetch_correctly() {
    let doc = build();

    // Parallel regions: same chapter and scroll, sections 10 vs 11.
    assert_eq!(breaking_a().x.chapter, breaking_b().x.chapter);
    assert_eq!(breaking_a().x.scroll, breaking_b().x.scroll);
    assert_eq!(breaking_a().x.section, 10);
    assert_eq!(breaking_b().x.section, 11);

    // Both coexist, byte-exact, in the same phext.
    assert_eq!(phext::fetch(&doc, breaking_a()), BREAKING_A);
    assert_eq!(phext::fetch(&doc, breaking_b()), BREAKING_B);
    assert_ne!(BREAKING_A, BREAKING_B);

    // Each breaking is a genuine partition of the paragraph: every
    // word box named exactly once, in reading order.
    let all_words: Vec<phext::Coordinate> = (1..=WORDS.len()).map(word_box).collect();
    for breaking in [BREAKING_A, BREAKING_B] {
        let flat: Vec<phext::Coordinate> =
            breaking_lines(breaking).into_iter().flatten().collect();
        assert_eq!(flat, all_words, "breaking must cover each word exactly once, in order");
    }

    // And they really are different breakings: A is 3+3, B is 2+2+2.
    let a_sizes: Vec<usize> = breaking_lines(BREAKING_A).iter().map(|l| l.len()).collect();
    let b_sizes: Vec<usize> = breaking_lines(BREAKING_B).iter().map(|l| l.len()).collect();
    assert_eq!(a_sizes, vec![3, 3]);
    assert_eq!(b_sizes, vec![2, 2, 2]);
}

#[test]
fn current_lines_follow_the_link_scroll() {
    let doc = build();

    // The link initially names breaking A...
    assert_eq!(
        phext::fetch(&doc, link_scroll()),
        coord_string(breaking_a())
    );
    // ...and reading the current lines is following it: fetch the
    // link, fetch that breaking, fetch each word scroll it names.
    assert_eq!(
        current_lines(&doc),
        vec![
            vec!["the", "quick", "brown"],
            vec!["fox", "jumps", "over"],
        ]
    );
}

#[test]
fn switching_choice_touches_only_the_link_scroll() {
    let before = build();
    let before_map = nonempty(&before);

    // THE SWITCH: replace only the link scroll's content so it
    // points at breaking B instead of breaking A. Nothing else is
    // written — no breaking is rebuilt, no box moves.
    let after = phext::replace(&before, link_scroll(), &coord_string(breaking_b()));
    let after_map = nonempty(&after);

    // Explode-diff (NOT subtract: subtract masks by coordinate
    // presence, not content, per the wave-1 web finding): same
    // coordinate set, and exactly one coordinate changed content.
    assert_eq!(before_map.len(), after_map.len());
    assert_eq!(
        before_map.keys().collect::<Vec<_>>().len(),
        after_map.keys().collect::<Vec<_>>().len()
    );
    let mut changed: Vec<phext::Coordinate> = Vec::new();
    for (coord, value) in &before_map {
        match after_map.get(coord) {
            Some(v) if v == value => {}
            Some(_) => changed.push(*coord),
            None => panic!("scroll at {coord} vanished during the switch"),
        }
    }
    for coord in after_map.keys() {
        assert!(before_map.contains_key(coord), "new scroll at {coord} during the switch");
    }
    assert_eq!(changed, vec![link_scroll()], "exactly the link scroll may change");

    // Both breaking scrolls and every box/glue scroll are
    // byte-identical after the switch (already implied by the diff
    // above; stated explicitly because it is the claim).
    assert_eq!(phext::fetch(&after, breaking_a()), BREAKING_A);
    assert_eq!(phext::fetch(&after, breaking_b()), BREAKING_B);
    for (i, (word, width)) in WORDS.iter().enumerate() {
        assert_eq!(
            phext::fetch(&after, word_box(i + 1)),
            format!("hbox: word={word} width={width}")
        );
    }
    for n in 1..WORDS.len() {
        assert_eq!(phext::fetch(&after, glue(n)), "glue 10 plus 5 minus 2");
    }

    // And the switch has its intended effect: the same driver, still
    // making no numeric choice, now reads breaking B's lines.
    assert_eq!(phext::fetch(&after, link_scroll()), coord_string(breaking_b()));
    assert_eq!(
        current_lines(&after),
        vec![
            vec!["the", "quick"],
            vec!["brown", "fox"],
            vec!["jumps", "over"],
        ]
    );
}
