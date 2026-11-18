#[cfg(test)]
mod tests {
    use crate::phext::{self, to_coordinate,
        SCROLL_BREAK, SECTION_BREAK, CHAPTER_BREAK, BOOK_BREAK, VOLUME_BREAK,
        COLLECTION_BREAK, SERIES_BREAK, SHELF_BREAK, LIBRARY_BREAK,
        CHAIN_BREAK, CHAIN_CLOSE, LINE_BREAK};

    fn c(s: &str) -> phext::Coordinate {
        to_coordinate(s)
    }

    // spans must agree with explode (contents), phokenize (order + non-empty
    // tokens) and offset (range starts) on any document.
    fn assert_agreement(doc: &str) {
        let index = phext::spans(doc);
        let exploded = phext::explode(doc);
        assert_eq!(index.len(), exploded.len(), "span count vs explode count");
        let phokens: Vec<_> = phext::phokenize(doc).into_iter()
            .filter(|p| !p.scroll.is_empty()).collect();
        assert_eq!(index.len(), phokens.len(), "span count vs phokenize count");
        let mut previous: Option<phext::Coordinate> = None;
        for (span, phoken) in index.iter().zip(phokens.iter()) {
            assert_eq!(span.coord, phoken.coord, "coordinate order vs phokenize");
            assert_eq!(&doc[span.range.clone()], phoken.scroll.as_str(), "slice vs phokenize");
            assert_eq!(exploded.get(&span.coord).map(|s| s.as_str()),
                       Some(&doc[span.range.clone()]), "slice vs explode at {}", span.coord);
            assert_eq!(phext::offset(doc, span.coord), span.range.start,
                       "range start vs offset at {}", span.coord);
            assert_eq!(phext::fetch(doc, span.coord), &doc[span.range.clone()],
                       "slice vs fetch at {}", span.coord);
            if let Some(prev) = previous {
                assert!(prev < span.coord, "spans are in ascending coordinate order");
            }
            previous = Some(span.coord);
        }
    }

    #[test]
    fn spans_basic_two_scrolls() {
        let doc = "alpha\u{17}beta";
        let index = phext::spans(doc);
        assert_eq!(index.len(), 2);
        assert_eq!(index[0].coord, c("1.1.1/1.1.1/1.1.1"));
        assert_eq!(index[0].range, 0..5);
        assert_eq!(index[1].coord, c("1.1.1/1.1.1/1.1.2"));
        assert_eq!(index[1].range, 6..10);
        assert_eq!(&doc[index[1].range.clone()], "beta");
    }

    #[test]
    fn spans_agree_on_a_mixed_dimension_document() {
        // line breaks stay inside a scroll; every dimension break moves
        let mut doc = String::new();
        doc.push_str("root\nwith lines");
        doc.push(SCROLL_BREAK);   doc.push_str("second scroll");
        doc.push(SECTION_BREAK);  doc.push_str("new section");
        doc.push(CHAPTER_BREAK);  doc.push_str("new chapter");
        doc.push(BOOK_BREAK);     doc.push_str("new book");
        doc.push(VOLUME_BREAK);   doc.push_str("new volume");
        doc.push(COLLECTION_BREAK); doc.push_str("new collection");
        doc.push(SERIES_BREAK);   doc.push_str("new series");
        doc.push(SHELF_BREAK);    doc.push_str("new shelf");
        doc.push(LIBRARY_BREAK);  doc.push_str("new library");
        assert_agreement(&doc);
        let index = phext::spans(&doc);
        assert_eq!(index.len(), 10);
        assert_eq!(index[9].coord, c("2.1.1/1.1.1/1.1.1"));
    }

    #[test]
    fn spans_first_and_last_scroll() {
        let mut doc = String::from("first");
        doc.push(SCROLL_BREAK); doc.push_str("middle");
        doc.push(SCROLL_BREAK); doc.push_str("last");
        let index = phext::spans(&doc);
        assert_eq!(index.len(), 3);
        assert_eq!(index[0].range.start, 0, "first span starts at byte 0");
        assert_eq!(index[2].range.end, doc.len(), "last span ends at the document end");
        assert_eq!(&doc[index[2].range.clone()], "last");
    }

    // ---- the empty-scroll contract: only non-empty scrolls are reported ----

    #[test]
    fn spans_empty_document_yields_no_spans() {
        assert!(phext::spans("").is_empty());
        // phokenize, the tokenizer, reports one empty positioned scroll here;
        // spans follows explode: an empty scroll does not exist
        assert_eq!(phext::phokenize("").len(), 1);
    }

    #[test]
    fn spans_document_of_only_delimiters_yields_no_spans() {
        let mut doc = String::new();
        for b in [SCROLL_BREAK, SECTION_BREAK, CHAPTER_BREAK, BOOK_BREAK, VOLUME_BREAK,
                  COLLECTION_BREAK, SERIES_BREAK, SHELF_BREAK, LIBRARY_BREAK] {
            doc.push(b);
        }
        doc.push(SCROLL_BREAK);
        doc.push(SCROLL_BREAK);
        assert!(phext::spans(&doc).is_empty());
        assert!(phext::explode(&doc).is_empty());
    }

    #[test]
    fn spans_trailing_delimiter_run_yields_no_empty_span() {
        let mut doc = String::from("only");
        doc.push(SCROLL_BREAK);
        doc.push(SCROLL_BREAK);
        doc.push(SECTION_BREAK);
        let index = phext::spans(&doc);
        assert_eq!(index.len(), 1);
        assert_eq!(index[0].range, 0..4);
        // phokenize additionally reports the trailing empty slot; spans does not
        assert_eq!(phext::phokenize(&doc).len(), 2);
        assert_agreement(&doc);
    }

    #[test]
    fn spans_empty_slots_between_distant_scrolls_are_skipped() {
        // a delimiter run encodes empty scrolls at the skipped coordinates;
        // they are addressing, not content, and produce no span
        let mut doc = String::from("a");
        doc.push(SCROLL_BREAK);
        doc.push(SCROLL_BREAK);
        doc.push_str("b");
        let index = phext::spans(&doc);
        assert_eq!(index.len(), 2);
        assert_eq!(index[1].coord, c("1.1.1/1.1.1/1.1.3"));
        assert_eq!(index[1].range, 3..4);
        assert_agreement(&doc);
    }

    #[test]
    fn spans_leading_breaks_address_the_first_scroll() {
        let mut doc = String::new();
        doc.push(SCROLL_BREAK);
        doc.push(SCROLL_BREAK);
        doc.push_str("x");
        let index = phext::spans(&doc);
        assert_eq!(index.len(), 1);
        assert_eq!(index[0].coord, c("1.1.1/1.1.1/1.1.3"));
        assert_eq!(index[0].range, 2..3);
    }

    // ---- chain bytes are content to the single-phext parser ----

    #[test]
    fn spans_chain_bytes_are_scroll_content() {
        // 0x02 / 0x03 structure chains (parse_chains' layer); the single-phext
        // walk treats them as content, and spans must match it exactly
        let mut doc = String::from("alpha");
        doc.push(CHAIN_BREAK);
        doc.push_str("gamma");
        doc.push(CHAIN_CLOSE);
        let index = phext::spans(&doc);
        assert_eq!(index.len(), 1);
        assert_eq!(&doc[index[0].range.clone()], doc.as_str());
        assert_agreement(&doc);
    }

    #[test]
    fn spans_non_break_control_bytes_and_line_breaks_are_content() {
        // bytes in the break *range* of ASCII that phext does not claim as
        // dimension breaks — line break, cowbell, tab, escape, and friends —
        // are scroll data, not boundaries
        let mut scroll = String::from("a");
        for b in ['\u{04}', '\u{05}', '\u{06}', '\u{07}', '\u{08}', '\u{09}',
                  '\u{0b}', '\u{0c}', '\u{0d}', '\u{0e}', '\u{0f}', '\u{10}',
                  '\u{15}', '\u{16}', '\u{1b}'] {
            scroll.push(b);
            scroll.push('x');
        }
        scroll.push(LINE_BREAK);
        scroll.push_str("next line");
        let index = phext::spans(&scroll);
        assert_eq!(index.len(), 1);
        assert_eq!(index[0].range, 0..scroll.len());
        assert_agreement(&scroll);
    }

    #[test]
    fn spans_ranges_are_byte_ranges_over_utf8() {
        let doc = "héllo\u{17}wörld";
        let index = phext::spans(doc);
        assert_eq!(index.len(), 2);
        assert_eq!(index[0].range, 0..6, "é is two bytes");
        assert_eq!(&doc[index[0].range.clone()], "héllo");
        assert_eq!(&doc[index[1].range.clone()], "wörld");
        assert_agreement(doc);
    }

    // ---- coordinate edges ----

    #[test]
    fn spans_coordinate_components_beyond_999() {
        // components are unbounded since 0.4.0: 1005 scroll breaks land the
        // next scroll at scroll 1006
        let mut doc = String::from("a");
        for _ in 0..1005 { doc.push(SCROLL_BREAK); }
        doc.push('b');
        let index = phext::spans(&doc);
        assert_eq!(index.len(), 2);
        assert_eq!(index[1].coord, c("1.1.1/1.1.1/1.1.1006"));
        assert_eq!(index[1].range, 1006..1007);
        assert_agreement(&doc);
    }

    #[test]
    fn spans_agree_on_a_1500_scroll_document() {
        let mut doc = String::new();
        for i in 1..=1500 {
            if i > 1 { doc.push(SCROLL_BREAK); }
            doc.push_str(&format!("scroll {}", i));
        }
        let index = phext::spans(&doc);
        assert_eq!(index.len(), 1500);
        assert_eq!(index[1499].coord, c("1.1.1/1.1.1/1.1.1500"));
        assert_agreement(&doc);
    }

    #[test]
    fn saturating_address_components_match_the_existing_parser() {
        // span coordinates come from break arithmetic (unbounded increment,
        // never saturates); *address parsing* is where saturation lives, and
        // it is unchanged: a 20-digit component saturates at u64::MAX
        let coord = c("1.1.1/1.1.1/1.1.99999999999999999999");
        assert_eq!(coord.x.scroll, u64::MAX as usize);
        // a coordinate past every scroll resolves through offset/fetch as before
        let doc = "only scroll";
        assert_eq!(phext::fetch(doc, coord), "");
        assert_eq!(phext::offset(doc, coord), 0);
    }
}
