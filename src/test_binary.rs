#[cfg(test)]
mod tests {
    use crate::binary;
    use crate::phext;

    fn coord(address: &str) -> phext::Coordinate {
        return phext::to_coordinate(address);
    }

    #[test]
    fn test_encode_decode_round_trip() {
        // empty payload: header line alone
        let empty: Vec<u8> = vec![];
        let scroll = binary::encode_scroll(&empty);
        assert_eq!(scroll, "phext-binary:1:base64\n");
        assert_eq!(binary::decode_scroll(&scroll), Some(empty));

        // every byte value
        let all_bytes: Vec<u8> = (0..=255u8).collect();
        let scroll = binary::encode_scroll(&all_bytes);
        assert!(binary::is_binary_scroll(&scroll));
        assert_eq!(binary::decode_scroll(&scroll), Some(all_bytes));

        // a plainly non-UTF-8 sequence
        let foreign: Vec<u8> = vec![0xFF, 0xFE, 0x80, 0x00, 0x61, 0xC3, 0x28];
        let scroll = binary::encode_scroll(&foreign);
        assert_eq!(binary::decode_scroll(&scroll), Some(foreign));

        // >= 1 MiB payload, deterministic pattern covering all byte values
        let big: Vec<u8> = (0..(1024 * 1024 + 13)).map(|i| (i % 256) as u8).collect();
        let scroll = binary::encode_scroll(&big);
        assert_eq!(binary::decode_scroll(&scroll), Some(big));
    }

    #[test]
    fn test_decode_scroll_rejects_non_binary() {
        assert_eq!(binary::decode_scroll("just some text"), None);
        assert_eq!(binary::decode_scroll(""), None);
        // header must be a complete line: no line break, no binary scroll
        assert_eq!(binary::decode_scroll("phext-binary:1:base64"), None);
        assert!(!binary::is_binary_scroll("phext-binary:1:base64"));
        // malformed base64 body: recognisably binary, undecodable
        let malformed = "phext-binary:1:base64\n!!! not base64 !!!";
        assert!(binary::is_binary_scroll(malformed));
        assert_eq!(binary::decode_scroll(malformed), None);
    }

    #[test]
    fn test_insert_and_fetch_binary() {
        let target = coord("1.1.1/1.1.1/1.1.2");
        let payload: Vec<u8> = vec![0x00, 0x01, 0x02, 0xFE, 0xFF, 0x7F, 0x80];

        let doc = phext::insert(String::new(), coord("1.1.1/1.1.1/1.1.1"), "plain text scroll");
        let doc = binary::insert_binary(doc, target, &payload);

        // the text scroll next door is untouched
        assert_eq!(phext::fetch(&doc, coord("1.1.1/1.1.1/1.1.1")), "plain text scroll");
        // the binary scroll fetches back byte-identical
        assert_eq!(binary::fetch_binary(&doc, target), payload);
        // and its stored form is the canonical binary scroll
        let stored = phext::fetch(&doc, target);
        assert!(binary::is_binary_scroll(&stored));
        assert_eq!(binary::decode_scroll(&stored), Some(payload));
    }

    #[test]
    fn test_fetch_binary_fallbacks() {
        let doc = phext::insert(String::new(), coord("1.1.1/1.1.1/1.1.1"), "hello");
        // a plain text scroll yields its own UTF-8 bytes
        assert_eq!(binary::fetch_binary(&doc, coord("1.1.1/1.1.1/1.1.1")), b"hello".to_vec());
        // an unoccupied coordinate yields no bytes (mirrors fetch returning "")
        assert_eq!(binary::fetch_binary(&doc, coord("9.9.9/9.9.9/9.9.9")), Vec::<u8>::new());
        // a malformed binary scroll yields its own bytes verbatim, not a panic
        let doc = phext::insert(String::new(), coord("1.1.1/1.1.1/1.1.1"), "phext-binary:1:base64\n!!!");
        assert_eq!(binary::fetch_binary(&doc, coord("1.1.1/1.1.1/1.1.1")), b"phext-binary:1:base64\n!!!".to_vec());
    }

    #[test]
    fn test_update_rule_non_utf8_is_base64_verbatim() {
        let target = coord("1.1.1/1.1.1/1.1.1");
        // payload deliberately includes dimension-break bytes (0x17, 0x1C):
        // armoured as base64, they round-trip as content, not structure
        let payload: Vec<u8> = vec![0xFF, 0x17, 0x1C, 0x80, 0x00];
        let doc = binary::replace_bytes("", target, &payload);

        let stored = phext::fetch(&doc, target);
        assert!(binary::is_binary_scroll(&stored), "non-UTF-8 update must be stored as a binary scroll");
        assert_eq!(binary::fetch_binary(&doc, target), payload);

        // same rule through the insert entry point
        let doc = binary::insert_bytes(String::new(), target, &payload);
        assert_eq!(binary::fetch_binary(&doc, target), payload);
    }

    #[test]
    fn test_update_rule_utf8_stays_text() {
        let target = coord("1.1.1/1.1.1/1.1.1");
        let doc = binary::replace_bytes("", target, "ordinary text ✓".as_bytes());
        assert_eq!(phext::fetch(&doc, target), "ordinary text ✓");
        assert!(!binary::is_binary_scroll(&phext::fetch(&doc, target)));
        assert_eq!(binary::fetch_binary(&doc, target), "ordinary text ✓".as_bytes().to_vec());
    }

    #[test]
    fn test_ingest_foreign_byte_stream() {
        // a phext byte stream whose second scroll is not valid UTF-8
        let mut stream: Vec<u8> = Vec::new();
        stream.extend_from_slice(b"hello");
        stream.push(phext::SCROLL_BREAK_U8);
        stream.extend_from_slice(&[0xFF, 0xFE, 0x41, 0x80]);

        let doc = binary::ingest(&stream);

        // text scroll passes through verbatim
        assert_eq!(phext::fetch(&doc, coord("1.1.1/1.1.1/1.1.1")), "hello");
        // foreign scroll: stored as a binary scroll, reads back as the original bytes
        let foreign_at = coord("1.1.1/1.1.1/1.1.2");
        assert!(binary::is_binary_scroll(&phext::fetch(&doc, foreign_at)));
        assert_eq!(binary::fetch_binary(&doc, foreign_at), vec![0xFF, 0xFE, 0x41, 0x80]);
    }

    #[test]
    fn test_ingest_canonical_phext_is_identity() {
        let mut doc = phext::insert(String::new(), coord("1.1.1/1.1.1/1.1.1"), "first");
        doc = phext::insert(doc, coord("1.1.1/1.1.1/1.1.2"), "second ✓ scroll");
        doc = phext::insert(doc, coord("2.1.1/1.3.1/1.1.1"), "far away");
        assert_eq!(binary::ingest(doc.as_bytes()), doc);
    }

    #[test]
    fn test_ingest_empty_and_break_only_streams() {
        // no panic, no content
        let doc = binary::ingest(&[]);
        assert_eq!(phext::fetch(&doc, coord("1.1.1/1.1.1/1.1.1")), "");
        let doc = binary::ingest(&[phext::SCROLL_BREAK_U8, phext::SECTION_BREAK_U8]);
        assert_eq!(phext::fetch(&doc, coord("1.1.1/1.1.1/1.1.1")), "");
    }

    #[test]
    fn test_phokenize_bytes_matches_phokenize() {
        let doc = "alpha\x17beta\x18gamma";
        let text_parts = phext::phokenize(doc);
        let byte_parts = binary::phokenize_bytes(doc.as_bytes());
        assert_eq!(text_parts.len(), byte_parts.len());
        for (text_part, byte_part) in text_parts.iter().zip(byte_parts.iter()) {
            assert_eq!(text_part.coord, byte_part.0);
            assert_eq!(text_part.scroll.as_bytes(), byte_part.1.as_slice());
        }
    }
}
