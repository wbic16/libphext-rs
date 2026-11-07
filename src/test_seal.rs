#[cfg(test)]
mod tests {
    use crate::seal;

    #[test]
    fn test_seal_round_trip_real_phext() {
        // the crate's own test corpus: a real, multi-scroll phext
        let corpus = include_str!("../unit-test.phext");
        let sealed = seal::seal(corpus).expect("sealing the corpus must succeed");
        assert!(sealed.len() > 0);
        // an xz stream starts with the xz magic bytes
        assert_eq!(&sealed[0..6], b"\xFD7zXZ\x00");
        let restored = seal::unseal_string(&sealed).expect("unsealing must succeed");
        assert_eq!(restored, corpus);
    }

    #[test]
    fn test_seal_round_trip_empty_and_small() {
        let sealed = seal::seal("").expect("seal");
        assert_eq!(seal::unseal_string(&sealed).expect("unseal"), "");

        let sealed = seal::seal("hello phext").expect("seal");
        assert_eq!(seal::unseal(&sealed).expect("unseal"), b"hello phext".to_vec());
    }

    #[test]
    fn test_seal_carrying_binary_scrolls() {
        // a phext holding base64 binary scrolls (the canonical literal form)
        // seals and unseals with every scroll intact
        let doc = "cover text\x17phext-binary:1:base64\nAAEC/f7/fw==\x17after";
        let sealed = seal::seal(doc).expect("seal");
        let restored = seal::unseal_string(&sealed).expect("unseal");
        assert_eq!(restored, doc);
    }

    #[test]
    fn test_seal_is_deterministic() {
        let doc = "the same phext, sealed twice\x17the same phext, sealed twice";
        let first = seal::seal(doc).expect("seal");
        let second = seal::seal(doc).expect("seal");
        assert_eq!(first, second);
    }

    #[test]
    fn test_unseal_rejects_non_xz_data() {
        assert!(seal::unseal(b"this is not an xz stream").is_err());
        assert!(seal::unseal(&[]).is_err());
        assert!(seal::unseal_string(b"phext, but unsealed").is_err());
    }
}
