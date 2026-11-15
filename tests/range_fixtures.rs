//! Conformance test consuming the shared libphext 0.6.1 range fixtures in
//! tests/fixtures/ranges/ (see FIXTURES-RANGES.md in the ports program
//! workspace copy; the set extends the 0.6 chain fixtures).
//!
//! For every case: parse the input stream with parse_chains, take chain 0,
//! and run every recorded op against the v0.6.1 addressing API
//! (parse_point_address / parse_range_address / Chain::resolve_point /
//! Chain::resolve_range / Chain::fetch_point / Chain::fetch_range),
//! comparing canonical forms, resolutions, and fetched bytes exactly.

use libphext::phext;
use std::fs;
use std::path::PathBuf;

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ranges")
}

fn field<'a>(fields: &[&'a str], key: &str) -> &'a str {
    for f in fields {
        if let Some((k, v)) = f.split_once('=') {
            if k == key {
                return v;
            }
        }
    }
    panic!("missing field {key}");
}

#[test]
fn range_fixtures_conform() {
    let root = fixtures_root();
    let index = fs::read_to_string(root.join("cases.tsv")).expect("cases.tsv");
    let mut case_count = 0usize;
    let mut op_count = 0usize;
    for line in index.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let case = line.split('\t').next().unwrap().to_string();
        case_count += 1;
        op_count += run_case(&root.join(&case), &case);
    }
    assert!(case_count >= 6, "expected at least 6 fixture cases, got {case_count}");
    assert!(op_count >= 40, "expected at least 40 fixture ops, got {op_count}");
}

fn run_case(dir: &PathBuf, case: &str) -> usize {
    let tsv = fs::read_to_string(dir.join("expected.tsv")).expect("expected.tsv");
    let input_bytes = fs::read(dir.join("input.phext")).expect("input.phext");
    let input = String::from_utf8(input_bytes).expect("fixture input is valid UTF-8");
    let chains = phext::parse_chains(&input);
    let chain = &chains[0];
    let mut ops = 0usize;
    for line in tsv.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.first() != Some(&"op") {
            continue;
        }
        ops += 1;
        let kind = fields[1];
        let input_addr = fields[2];
        let at = format!("case {case} op {} ({kind} {input_addr})", ops);
        match kind {
            "parse_point" => {
                let error = field(&fields, "error") == "true";
                match phext::parse_point_address(input_addr) {
                    None => assert!(error, "{at}: expected parse error, got success"),
                    Some(p) => {
                        assert!(!error, "{at}: expected success, got parse error");
                        assert_eq!(p.to_string(), field(&fields, "canonical"), "{at}: canonical mismatch");
                    }
                }
            }
            "parse_range" => {
                let error = field(&fields, "error") == "true";
                match phext::parse_range_address(input_addr) {
                    None => assert!(error, "{at}: expected parse error, got success"),
                    Some(r) => {
                        assert!(!error, "{at}: expected success, got parse error");
                        assert_eq!(r.to_string(), field(&fields, "canonical"), "{at}: canonical mismatch");
                    }
                }
            }
            "resolve_point" => {
                let error = field(&fields, "error") == "true";
                let point = phext::parse_point_address(input_addr).expect("resolve op input parses");
                match chain.resolve_point(&point) {
                    None => assert!(error, "{at}: expected resolution error, got success"),
                    Some((universe, coord, offset)) => {
                        assert!(!error, "{at}: expected success, got resolution error");
                        let want = format!("{universe}|{coord}|{offset}");
                        assert_eq!(want, field(&fields, "resolve"), "{at}: resolution mismatch");
                    }
                }
            }
            "fetch_point" => {
                let resolved = field(&fields, "resolved") == "true";
                let point = phext::parse_point_address(input_addr);
                let actual_resolved = point.as_ref().map(|p| chain.resolve_point(p).is_some()).unwrap_or(false);
                assert_eq!(actual_resolved, resolved, "{at}: resolved flag mismatch");
                let expected = fs::read(dir.join(field(&fields, "file"))).expect("expected result file");
                let actual = chain.fetch_point(input_addr);
                assert_eq!(actual.as_bytes(), expected.as_slice(), "{at}: fetch_point bytes mismatch");
            }
            "fetch_range" => {
                let resolved = field(&fields, "resolved") == "true";
                let range = phext::parse_range_address(input_addr);
                let actual_resolved = range.as_ref().map(|r| chain.resolve_range(r).is_some()).unwrap_or(false);
                assert_eq!(actual_resolved, resolved, "{at}: resolved flag mismatch");
                let expected = fs::read(dir.join(field(&fields, "file"))).expect("expected result file");
                let actual = chain.fetch_range(input_addr);
                assert_eq!(actual.as_bytes(), expected.as_slice(), "{at}: fetch_range bytes mismatch");
            }
            other => panic!("{at}: unknown op kind {other}"),
        }
    }
    ops
}
