#[cfg(test)]
mod tests {
    use crate::phext::{self, to_coordinate, SCROLL_BREAK, VOLUME_BREAK};
    use std::collections::HashMap;

    fn c(s: &str) -> phext::Coordinate {
        to_coordinate(s)
    }

    // ---- append ------------------------------------------------------------

    #[test]
    fn append_lands_at_the_exact_coordinate() {
        let doc = phext::append(String::new(), c("1.1.1/1.1.1/1.1.1"), "first").unwrap();
        let doc = phext::append(doc, c("1.1.1/1.1.1/1.1.2"), "second").unwrap();
        assert_eq!(phext::fetch(&doc, c("1.1.1/1.1.1/1.1.1")), "first");
        assert_eq!(phext::fetch(&doc, c("1.1.1/1.1.1/1.1.2")), "second");
        // the second append changed only the suffix
        assert!(doc.starts_with("first"));
    }

    #[test]
    fn append_rejects_occupied_earlier_and_empty() {
        let doc = phext::append(String::new(), c("1.1.1/1.1.1/1.1.2"), "x").unwrap();
        assert!(phext::append(doc.clone(), c("1.1.1/1.1.1/1.1.2"), "y").is_none(), "occupied");
        assert!(phext::append(doc.clone(), c("1.1.1/1.1.1/1.1.1"), "y").is_none(), "earlier");
        assert!(phext::append(doc, c("1.1.1/1.1.1/1.1.3"), "").is_none(), "empty scroll");
    }

    #[test]
    fn append_ignores_trailing_breaks() {
        // trailing delimiters encode only empty scrolls; they must not
        // shift the appended scroll's address
        let mut doc = "a".to_string();
        doc.push(SCROLL_BREAK);
        doc.push(SCROLL_BREAK);
        let doc = phext::append(doc, c("1.1.1/1.1.1/1.1.2"), "b").unwrap();
        assert_eq!(phext::fetch(&doc, c("1.1.1/1.1.1/1.1.2")), "b");
        assert_eq!(phext::explode(&doc).len(), 2);
    }

    #[test]
    fn append_across_a_volume_boundary_mints_no_empty_scrolls() {
        let doc = phext::append("root".to_string(), c("1.1.1/1.2.1/1.1.1"), "next volume").unwrap();
        assert_eq!(phext::fetch(&doc, c("1.1.1/1.1.1/1.1.1")), "root");
        assert_eq!(phext::fetch(&doc, c("1.1.1/1.2.1/1.1.1")), "next volume");
        assert_eq!(phext::explode(&doc).len(), 2);
    }

    #[test]
    fn append_batch_lands_all_and_validates_order() {
        let entries = [
            (c("1.1.1/1.1.1/1.1.1"), "one"),
            (c("1.1.1/1.1.1/1.1.2"), "two"),
            (c("1.1.1/1.1.1/1.2.1"), "three"),
        ];
        let doc = phext::append_batch(String::new(), &entries).unwrap();
        assert_eq!(phext::fetch(&doc, entries[2].0), "three");
        assert_eq!(phext::explode(&doc).len(), 3);
        let out_of_order = [entries[1], entries[0]];
        assert!(phext::append_batch(String::new(), &out_of_order).is_none());
        assert!(phext::append_batch(doc, &[(c("1.1.1/1.1.1/1.1.2"), "late")]).is_none());
    }

    // ---- checked scrolls ---------------------------------------------------

    #[test]
    fn checked_scroll_roundtrips_and_detects_tampering() {
        let checked = phext::checked_scroll("line one\nline two");
        assert!(phext::verify_checked_scroll(&checked));
        assert_eq!(phext::uncheck_scroll(&checked).unwrap(), "line one\nline two");
        let tampered = checked.replace("one", "eno");
        assert!(!phext::verify_checked_scroll(&tampered));
        assert!(phext::uncheck_scroll(&tampered).is_none());
        assert_eq!(phext::checked_scroll(""), "");
        assert!(!phext::verify_checked_scroll(""));
        assert!(phext::uncheck_scroll("no trailer here").is_none());
    }

    #[test]
    fn checked_scroll_detects_a_torn_tail() {
        let checked = phext::checked_scroll("a complete entry");
        let torn = &checked[..checked.len() - 6];
        assert!(!phext::verify_checked_scroll(torn));
        assert!(phext::uncheck_scroll(torn).is_none());
    }

    #[test]
    fn checked_scroll_reserves_only_the_final_marker_line() {
        // a payload that itself ends with a marker-shaped line still
        // round-trips: verification reads the LAST marker, and the
        // checksum binds the whole payload before it
        let payload = "data\nphext-check:xxh3-128:00000000000000000000000000000000";
        let checked = phext::checked_scroll(payload);
        assert!(phext::verify_checked_scroll(&checked));
        assert_eq!(phext::uncheck_scroll(&checked).unwrap(), payload);
    }

    #[test]
    fn fetch_checked_requires_a_valid_trailer() {
        let doc = phext::insert(String::new(), c("1.1.1/1.1.1/1.1.1"), &phext::checked_scroll("sealed"));
        let doc = phext::insert(doc, c("1.1.1/1.1.1/1.1.2"), "plain");
        assert_eq!(phext::fetch_checked(&doc, c("1.1.1/1.1.1/1.1.1")).unwrap(), "sealed");
        assert!(phext::fetch_checked(&doc, c("1.1.1/1.1.1/1.1.2")).is_none(), "unchecked scroll");
        assert!(phext::fetch_checked(&doc, c("1.1.1/1.1.1/1.1.9")).is_none(), "missing scroll");
    }

    // ---- commit ------------------------------------------------------------

    #[test]
    fn commit_marker_verifies_and_detects_tears_and_tampering() {
        let a = c("1.1.1/1.3.1/1.1.1");
        let b = c("1.1.1/1.3.1/1.1.2");
        let commit_at = c("1.1.1/1.3.1/1.1.3");
        let one = phext::checked_scroll("one");
        let two = phext::checked_scroll("two");
        let marker = phext::commit_scroll(&[(a, &one), (b, &two)]);
        let doc = phext::append_batch(String::new(), &[(a, &one), (b, &two), (commit_at, &marker)]).unwrap();
        assert!(phext::verify_commit(&doc, &[a, b], commit_at));
        assert_eq!(phext::fetch_checked(&doc, a).unwrap(), "one");

        // torn: the write stopped inside the commit marker
        let torn = &doc[..doc.len() - 5];
        assert!(!phext::verify_commit(torn, &[a, b], commit_at));

        // tampered: an entry changed after the marker was computed
        let altered = doc.replace("one", "eno");
        assert!(!phext::verify_commit(&altered, &[a, b], commit_at));

        // wrong entry list: count binds the batch size
        assert!(!phext::verify_commit(&doc, &[a], commit_at));
    }

    // ---- empty-scroll semantics --------------------------------------------

    #[test]
    fn explode_omits_empty_scrolls() {
        let mut doc = "a".to_string();
        doc.push(VOLUME_BREAK);
        doc.push(VOLUME_BREAK);
        doc.push_str("b");
        let map = phext::explode(&doc);
        assert_eq!(map.len(), 2, "delimiter runs are addressing, not content");
        assert_eq!(phext::fetch(&doc, c("1.1.1/1.3.1/1.1.1")), "b");

        let sparse = phext::insert(String::new(), c("1.1.1/1.3.1/1.1.1"), "deep");
        assert_eq!(phext::explode(&sparse).len(), 1);
    }

    #[test]
    fn implode_skips_empty_values_and_roundtrips_sparse_documents() {
        let mut map = HashMap::new();
        map.insert(c("1.1.1/1.1.1/1.1.1"), "home".to_string());
        map.insert(c("1.1.1/1.2.1/1.1.1"), "".to_string());
        map.insert(c("1.1.1/1.3.1/1.1.1"), "deep".to_string());
        let doc = phext::implode(map);
        let back = phext::explode(&doc);
        assert_eq!(back.len(), 2);
        assert_eq!(back[&c("1.1.1/1.3.1/1.1.1")], "deep");
    }

    // ---- binary fields -------------------------------------------------------

    #[cfg(feature = "binary")]
    #[test]
    fn binary_field_roundtrips_arbitrary_bytes_on_one_line() {
        use crate::binary;
        let payloads: Vec<Vec<u8>> = vec![
            Vec::new(),
            b"plain".to_vec(),
            b"line one\nline two".to_vec(),
            vec![0x17, 0x1C, 0x00, 0xFF, 0x01],
            (0..1024u32).map(|i| (i % 251) as u8).collect(),
        ];
        for payload in &payloads {
            let field = binary::encode_field(payload);
            assert!(!field.contains('\n'), "field must stay on one line");
            assert!(
                !field.bytes().any(|b| matches!(b, 0x01 | 0x17..=0x1F)),
                "field must contain no dimension-break bytes"
            );
            assert_eq!(binary::decode_field(&field).unwrap(), *payload);
        }
    }

    #[cfg(feature = "binary")]
    #[test]
    fn binary_field_rejects_plain_text_and_bad_base64() {
        use crate::binary;
        assert!(binary::decode_field("just text").is_none());
        assert!(binary::decode_field("phext-field:base64:!!!").is_none());
        assert!(binary::decode_field("phext-binary:1:base64\nAAAA").is_none(), "a binary scroll is not a field");
    }
}
