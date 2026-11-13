#[cfg(test)]
mod tests {
    use crate::phext::{self, Chain, CHAIN_BREAK, CHAIN_CLOSE, CHAIN_BREAK_U8, CHAIN_CLOSE_U8};

    /// parse + serialize must reproduce every stream byte-for-byte
    fn roundtrip(stream: &str) {
        let chains = phext::parse_chains(stream);
        assert_eq!(phext::serialize_chains(&chains), stream, "round trip {stream:?}");
    }

    #[test]
    fn chain_free_input_is_one_unclosed_chain() {
        // a phext with no chain bytes: the whole stream is the single member
        // of one unclosed chain, and it serializes back unchanged
        let doc = "alpha\x17beta\x01gamma";
        let chains = phext::parse_chains(doc);
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].members, vec![doc.to_string()]);
        assert!(!chains[0].closed);
        assert_eq!(phext::serialize_chains(&chains), doc);

        // single-phext behaviour on the same bytes is untouched: the chain
        // bytes are not dimension breaks, so nothing in the member moves
        assert_eq!(phext::fetch(doc, phext::to_coordinate("1.1.1/1.1.1/1.1.2")), "beta");
        assert_eq!(phext::fetch(doc, phext::to_coordinate("2.1.1/1.1.1/1.1.1")), "gamma");
        roundtrip(doc);
    }

    #[test]
    fn two_phexts_chained_and_closed_round_trip() {
        let stream = format!("edition one{CHAIN_BREAK}edition two{CHAIN_CLOSE}");
        let chains = phext::parse_chains(&stream);
        assert_eq!(chains.len(), 1);
        assert_eq!(
            chains[0].members,
            vec!["edition one".to_string(), "edition two".to_string()]
        );
        assert!(chains[0].closed);
        roundtrip(&stream);
    }

    #[test]
    fn unclosed_chain_at_eof_reports_truncated() {
        let chains = phext::parse_chains("one\x02two");
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].members, vec!["one".to_string(), "two".to_string()]);
        assert!(!chains[0].closed, "a chain ending at EOF must report unclosed");
        roundtrip("one\x02two");
    }

    #[test]
    fn two_chains_in_one_stream_parse_as_two_units() {
        let stream = "a\x02b\x03c\x02d\x03";
        let chains = phext::parse_chains(stream);
        assert_eq!(chains.len(), 2);
        assert_eq!(chains[0].members, vec!["a".to_string(), "b".to_string()]);
        assert!(chains[0].closed);
        assert_eq!(chains[1].members, vec!["c".to_string(), "d".to_string()]);
        assert!(chains[1].closed);
        roundtrip(stream);

        // a closed chain followed by an unclosed one stays two distinct chains
        let mixed = phext::parse_chains("a\x03b");
        assert_eq!(mixed.len(), 2);
        assert_eq!(mixed[0].members, vec!["a".to_string()]);
        assert!(mixed[0].closed);
        assert_eq!(mixed[1].members, vec!["b".to_string()]);
        assert!(!mixed[1].closed);
        roundtrip("a\x03b");
    }

    #[test]
    fn chained_members_carry_full_phexts() {
        // members are complete phexts: coordinates resolve inside each member,
        // and each member restarts at 1.1.1/1.1.1/1.1.1
        let first = "root\x17second scroll";
        let second = phext::insert(
            String::new(),
            phext::to_coordinate("1.1.1/1.1.1/1.1.3"),
            "third edition",
        );
        let stream = format!("{first}\x02{second}\x03");
        let chains = phext::parse_chains(&stream);
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].members.len(), 2);
        assert_eq!(
            phext::fetch(&chains[0].members[0], phext::to_coordinate("1.1.1/1.1.1/1.1.2")),
            "second scroll"
        );
        assert_eq!(
            phext::fetch(&chains[0].members[1], phext::to_coordinate("1.1.1/1.1.1/1.1.3")),
            "third edition"
        );
        assert_eq!(
            phext::fetch(&chains[0].members[1], phext::to_coordinate("1.1.1/1.1.1/1.1.1")),
            ""
        );
        roundtrip(&stream);
    }

    #[test]
    fn editions_in_succession_read_in_order() {
        // the chain is the time axis: members arrive oldest-first and stay ordered
        let stream = "v1\x02v2\x02v3\x03";
        let chains = phext::parse_chains(stream);
        assert_eq!(chains.len(), 1);
        assert_eq!(
            chains[0].members,
            vec!["v1".to_string(), "v2".to_string(), "v3".to_string()]
        );
        assert!(chains[0].closed);
        roundtrip(stream);
    }

    #[test]
    fn empty_and_break_only_streams_round_trip() {
        // the empty stream is one unclosed chain holding one empty phext
        let chains = phext::parse_chains("");
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].members, vec![String::new()]);
        assert!(!chains[0].closed);
        roundtrip("");

        // a lone close terminates a chain holding one empty phext
        let chains = phext::parse_chains("\x03");
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].members, vec![String::new()]);
        assert!(chains[0].closed);
        roundtrip("\x03");

        // leading and trailing breaks preserve their empty members
        assert_eq!(
            phext::parse_chains("a\x02")[0].members,
            vec!["a".to_string(), String::new()]
        );
        assert_eq!(
            phext::parse_chains("\x02b")[0].members,
            vec![String::new(), "b".to_string()]
        );
        assert_eq!(phext::parse_chains("\x02\x02")[0].members.len(), 3);
        roundtrip("a\x02");
        roundtrip("\x02b");
        roundtrip("\x02\x02");
        roundtrip("\x02");

        // back-to-back closes are two chains, the second holding an empty phext
        let chains = phext::parse_chains("a\x03\x03");
        assert_eq!(chains.len(), 2);
        assert!(chains[0].closed && chains[1].closed);
        assert_eq!(chains[1].members, vec![String::new()]);
        roundtrip("a\x03\x03");
    }

    #[test]
    fn serialize_from_constructed_chains() {
        let mut chain = Chain::new();
        assert!(!chain.closed);
        assert_eq!(chain.serialize(), "");
        chain.members.push("v1".to_string());
        chain.members.push("v2".to_string());
        assert_eq!(chain.serialize(), "v1\x02v2");
        chain.closed = true;
        assert_eq!(chain.serialize(), "v1\x02v2\x03");
        assert_eq!(phext::serialize_chains(&[chain]), "v1\x02v2\x03");
    }

    #[test]
    fn canon_shaped_documents_parse_unchanged() {
        // the regression fixture stands in for the canon phexts (CYOA,
        // incipit.phext, emise.phext): verified to contain no 0x02 or 0x03
        // bytes, so each reads as one unclosed chain holding one member,
        // byte-identical in and out
        let doc = std::fs::read_to_string("regression-1-helios.phext")
            .expect("Unable to open helios.phext");
        assert!(!doc.as_bytes().contains(&CHAIN_BREAK_U8));
        assert!(!doc.as_bytes().contains(&CHAIN_CLOSE_U8));
        let chains = phext::parse_chains(&doc);
        assert_eq!(chains.len(), 1);
        assert_eq!(chains[0].members, vec![doc.clone()]);
        assert!(!chains[0].closed);
        assert_eq!(phext::serialize_chains(&chains), doc);
    }

    #[test]
    fn chain_layer_does_not_touch_member_structure() {
        // splitting happens at the stream level only: a member's own scrolls
        // and breaks are exactly what the single-phext functions see
        let chains = phext::parse_chains("x\x17y\x02z\x17w\x03");
        assert_eq!(
            chains[0].members,
            vec!["x\x17y".to_string(), "z\x17w".to_string()]
        );
        assert_eq!(phext::phokenize(&chains[0].members[1]).len(), 2);
        // and a chain byte inside what a 0.5 reader saw as one document is
        // now the stream layer's: phokenize alone no longer spans members
        roundtrip("x\x17y\x02z\x17w\x03");
    }
}

#[test]
fn multiverse_addressing() {
    let chain = crate::phext::Chain {
        members: vec!["U1 home".to_string(), "U2 home".to_string()],
        closed: true,
    };
    assert_eq!(chain.fetch("1.1.1/1.1.1/1.1.1"), "U1 home");
    assert_eq!(chain.fetch("$1:1.1.1/1.1.1/1.1.1"), "U1 home");
    assert_eq!(chain.fetch("$2:1.1.1/1.1.1/1.1.1"), "U2 home");
    assert_eq!(chain.fetch("$3:1.1.1/1.1.1/1.1.1"), "");
    assert_eq!(crate::phext::parse_universe_address("$0:1.1.1/1.1.1/1.1.1"), None);
    let (n, _) = crate::phext::parse_universe_address("$12:2.1.1/1.1.1/1.1.1").unwrap();
    assert_eq!(n, 12);
}
