#[cfg(test)]
mod tests {
    use crate::phext::{self, PositionedScroll};
    use std::time::{Duration, Instant};

    /// The earlier phokenize: `next_scroll` in a loop. Quadratic, but the
    /// definition of the expected output.
    fn reference(phext: &str) -> Vec<PositionedScroll> {
        let mut result = Vec::new();
        let mut coord = phext::default_coordinate();
        let mut temp = phext.to_string();
        loop {
            let item;
            (item, coord, temp) = phext::next_scroll(temp.as_str(), coord);
            result.push(item);
            if temp.is_empty() {
                break;
            }
        }
        result
    }

    fn pairs(tokens: &[PositionedScroll]) -> Vec<(String, String)> {
        tokens
            .iter()
            .map(|token| (token.coord.to_string(), token.scroll.clone()))
            .collect()
    }

    const BREAKS: [char; 9] = [
        '\x17', '\x18', '\x19', '\x1A', '\x1C', '\x1D', '\x1E', '\x1F', '\x01',
    ];

    fn edge_cases() -> Vec<String> {
        vec![
            String::new(),
            "one scroll\nwith two lines".into(),
            "a\x17b\x18c\x19d\x1Ae\x1Cf\x1Dg\x1Eh\x1Fi\x01j".into(),
            "\x17\x17lead\x17\x17\x17gap\x01\x01\x17deep\x17".into(),
            "trailing one\x17".into(),
            "trailing two\x17\x17".into(),
            "\x01\x1F\x1E\x1D\x1C\x1A\x19\x18\x17".into(),
            "ünïcödé ✓\x1Fshelf ☃\x1Aboek ü\x17🜁".into(),
            "\n\n\x17\n".into(),
        ]
    }

    /// Small deterministic generator, so a failure always reproduces.
    struct Lcg(u64);
    impl Lcg {
        fn next(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0 >> 33
        }
    }

    fn random_doc(rng: &mut Lcg) -> String {
        let pieces = ["a", "bc", "ü", "☃", "🜁", "\n", " ", "text"];
        let mut doc = String::new();
        for _ in 0..(rng.next() % 60) {
            if rng.next() % 3 == 0 {
                let run = 1 + rng.next() % 3;
                for _ in 0..run {
                    doc.push(BREAKS[(rng.next() % 9) as usize]);
                }
            } else {
                doc.push_str(pieces[(rng.next() % pieces.len() as u64) as usize]);
            }
        }
        doc
    }

    #[test]
    fn phokenize_matches_the_next_scroll_loop() {
        for doc in edge_cases() {
            assert_eq!(pairs(&phext::phokenize(&doc)), pairs(&reference(&doc)), "{doc:?}");
        }
        let mut rng = Lcg(0x5eed);
        for _ in 0..5_000 {
            let doc = random_doc(&mut rng);
            assert_eq!(pairs(&phext::phokenize(&doc)), pairs(&reference(&doc)), "{doc:?}");
        }
    }

    /// The earlier get_subspace_coordinates: full comparisons on every byte, always to the end.
    fn subspace_reference(subspace: &[u8], target: phext::Coordinate) -> (usize, usize, phext::Coordinate) {
        let mut walker = phext::default_coordinate();
        let mut best = phext::default_coordinate();
        let mut subspace_index: usize = 0;
        let mut start: usize = 0;
        let mut end: usize = 0;
        let mut stage = 0;
        let max = subspace.len();
        while subspace_index < max {
            let next = subspace[subspace_index];
            let compare = next as char;
            if stage == 0 {
                if walker == target {
                    stage = 1;
                    start = subspace_index;
                    best = walker;
                }
                if walker < target {
                    best = walker;
                }
            }
            if stage < 2 && walker > target {
                if stage == 0 {
                    start = subspace_index - 1;
                }
                end = subspace_index - 1;
                stage = 2;
            }
            if phext::is_phext_break(next) {
                if compare == phext::SCROLL_BREAK { walker.scroll_break(); }
                if compare == phext::SECTION_BREAK { walker.section_break(); }
                if compare == phext::CHAPTER_BREAK { walker.chapter_break(); }
                if compare == phext::BOOK_BREAK { walker.book_break(); }
                if compare == phext::VOLUME_BREAK { walker.volume_break(); }
                if compare == phext::COLLECTION_BREAK { walker.collection_break(); }
                if compare == phext::SERIES_BREAK { walker.series_break(); }
                if compare == phext::SHELF_BREAK { walker.shelf_break(); }
                if compare == phext::LIBRARY_BREAK { walker.library_break(); }
            }
            if stage < 2 && walker > target {
                end = subspace_index;
                stage = 2;
            }
            subspace_index += 1;
        }
        if stage == 1 && walker == target {
            end = max;
        }
        if stage == 0 {
            start = max;
            end = max;
        }
        (start, end, best)
    }

    fn subspace_targets(doc: &str, rng: &mut Lcg) -> Vec<phext::Coordinate> {
        // every scroll's own coordinate, its neighbours, and some arbitrary ones
        let mut targets: Vec<phext::Coordinate> = Vec::new();
        for token in phext::phokenize(doc) {
            targets.push(token.coord);
            let mut after = token.coord;
            after.scroll_break();
            targets.push(after);
            let mut deeper = token.coord;
            deeper.section_break();
            targets.push(deeper);
        }
        for _ in 0..4 {
            let address = format!(
                "{}.{}.{}/{}.{}.{}/{}.{}.{}",
                1 + rng.next() % 3, 1 + rng.next() % 3, 1 + rng.next() % 3,
                1 + rng.next() % 3, 1 + rng.next() % 3, 1 + rng.next() % 3,
                1 + rng.next() % 3, 1 + rng.next() % 3, 1 + rng.next() % 5,
            );
            targets.push(phext::to_coordinate(&address));
        }
        targets
    }

    #[test]
    fn subspace_coordinates_match_the_per_byte_scan() {
        let mut rng = Lcg(0xc0de);
        let mut docs = edge_cases();
        for _ in 0..1_500 {
            docs.push(random_doc(&mut rng));
        }
        if let Ok(helios) = std::fs::read_to_string("regression-1-helios.phext") {
            docs.push(helios);
        }
        for doc in docs {
            let bytes = doc.as_bytes();
            for target in subspace_targets(&doc, &mut rng) {
                let (s, e, b) = phext::get_subspace_coordinates(bytes, target);
                let (rs, re, rb) = subspace_reference(bytes, target);
                assert_eq!(
                    (s, e, b.to_string()),
                    (rs, re, rb.to_string()),
                    "{doc:?} at {target}"
                );
            }
        }
    }

    #[test]
    fn explode_and_implode_roundtrip_the_same_scrolls() {
        let mut rng = Lcg(0xfab1e);
        for _ in 0..2_000 {
            let doc = random_doc(&mut rng);
            let map = phext::explode(&doc);
            let rebuilt = phext::implode(map.clone());
            let again = phext::explode(&rebuilt);
            let filled = |m: &std::collections::HashMap<phext::Coordinate, String>| {
                let mut v: Vec<(String, String)> = m
                    .iter()
                    .filter(|(_, text)| !text.is_empty())
                    .map(|(c, t)| (c.to_string(), t.clone()))
                    .collect();
                v.sort();
                v
            };
            assert_eq!(filled(&map), filled(&again), "{doc:?}");
        }
    }

    #[test]
    fn a_large_phext_explodes_in_linear_time() {
        let mut doc = String::new();
        for index in 0..50_000 {
            if index > 0 {
                // 40 scrolls a book, 40 books a volume: every coordinate stays in range
                doc.push(if index % 1600 == 0 {
                    '\x1C'
                } else if index % 40 == 0 {
                    '\x1A'
                } else {
                    '\x17'
                });
            }
            doc.push_str("kind: message\nauthor: bench\n---\nsome words about the topic here");
        }
        let start = Instant::now();
        let map = phext::explode(&doc);
        let rebuilt = phext::implode(map.clone());
        let toc = phext::textmap(&doc);
        let spent = start.elapsed();
        assert_eq!(map.len(), 50_000);
        assert_eq!(rebuilt, doc);
        assert_eq!(toc.lines().count(), 50_000);
        assert!(spent < Duration::from_secs(2), "took {spent:?}");
    }
}
