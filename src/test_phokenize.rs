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

    /// The non-empty scrolls of a document, as `explode` sees them.
    fn content(doc: &str) -> std::collections::BTreeMap<String, String> {
        phext::explode(doc)
            .into_iter()
            .filter(|(_, text)| !text.is_empty())
            .map(|(coord, text)| (coord.to_string(), text))
            .collect()
    }

    fn documents(seed: u64, count: usize) -> Vec<String> {
        let mut rng = Lcg(seed);
        let mut docs = edge_cases();
        docs.extend([
            "a\x18b\x19c".to_string(),
            "a\x17b\x17".to_string(),
            "x\x17".to_string(),
            "\x19\x1aa".to_string(),
            "a\x18b\x19c\x1ad\x1ce\x1df\x1eg\x1fh\x01i".to_string(),
        ]);
        for _ in 0..count {
            docs.push(random_doc(&mut rng));
        }
        if let Ok(helios) = std::fs::read_to_string("regression-1-helios.phext") {
            docs.push(helios);
        }
        docs
    }

    /// fetch, replace, insert, and remove each act on exactly the scroll at the target coordinate,
    /// whether it holds text, is empty, falls between two scrolls, or lies past the end.
    #[test]
    fn edits_act_on_exactly_the_target_scroll() {
        let mut rng = Lcg(0xc0de);
        for doc in documents(0xc0de, 1_500) {
            let before = content(&doc);
            for target in subspace_targets(&doc, &mut rng) {
                let key = target.to_string();
                let current = before.get(&key).cloned().unwrap_or_default();

                assert_eq!(phext::fetch(&doc, target), current, "fetch {doc:?} at {key}");

                let mut want = before.clone();
                want.insert(key.clone(), "X".to_string());
                assert_eq!(content(&phext::replace(&doc, target, "X")), want, "replace {doc:?} at {key}");

                let mut want = before.clone();
                want.insert(key.clone(), format!("{current}Y"));
                assert_eq!(content(&phext::insert(doc.clone(), target, "Y")), want, "insert {doc:?} at {key}");

                let mut want = before.clone();
                want.remove(&key);
                assert_eq!(content(&phext::remove(&doc, target)), want, "remove {doc:?} at {key}");

                let _ = phext::offset(&doc, target);
            }
        }
    }

    #[test]
    fn missing_coordinates_between_scrolls_are_left_alone() {
        let doc = "a\x18b\x19c";
        let between = phext::to_coordinate("1.1.1/1.1.1/1.1.2");
        assert_eq!(phext::fetch(doc, between), "");
        assert_eq!(phext::replace(doc, between, "X"), "a\x17X\x18b\x19c");
        assert_eq!(phext::insert(doc.to_string(), between, "Y"), "a\x17Y\x18b\x19c");
        assert_eq!(phext::remove(doc, between), doc);
        let after = phext::to_coordinate("1.1.1/1.1.1/1.1.2");
        assert_eq!(phext::insert("x\x17".to_string(), after, "Y"), "x\x17Y");
        assert_eq!(phext::fetch("a\x17b\x17", after), "b");
    }

    #[test]
    fn subtract_removes_only_scrolls_with_content_on_the_right() {
        let docs = documents(0x5b7, 1_000);
        for (index, left) in docs.iter().enumerate() {
            let right = &docs[(index + 1) % docs.len()];
            let mask = content(right);
            let want: std::collections::BTreeMap<String, String> = content(left)
                .into_iter()
                .filter(|(coord, _)| !mask.contains_key(coord))
                .collect();
            assert_eq!(content(&phext::subtract(left, right)), want, "{left:?} - {right:?}");
        }
        assert_eq!(phext::subtract("\x17\x17lead", "a\x18b"), "\x17\x17lead");
    }

    #[test]
    fn offset_does_not_panic_before_the_first_scroll() {
        assert_eq!(phext::offset("\x19\x1aa", phext::to_coordinate("1.1.1/1.1.1/1.1.1")), 0);
        assert_eq!(phext::offset("", phext::to_coordinate("2.2.2/2.2.2/2.2.2")), 0);
        assert_eq!(phext::offset("ab\x17cd", phext::to_coordinate("1.1.1/1.1.1/1.1.2")), 3);
        assert_eq!(phext::offset("ab\x17cd", phext::to_coordinate("1.1.1/1.1.1/1.1.9")), 3);
    }

    #[test]
    fn summaries_keep_multibyte_characters_whole() {
        // U+010A's low byte is 0x0A, a line break
        assert_eq!(phext::create_summary("\u{010A}bc"), "\u{010A}bc");
        assert_eq!(phext::create_summary("日本\x17rest"), "日本...");
        assert_eq!(phext::create_summary("one\ntwo"), "one...");
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
