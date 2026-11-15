//! Conformance test consuming the shared libphext 0.6 chain fixtures in
//! tests/fixtures/chains/ (see that directory's manifest.json and the
//! FIXTURES.md format spec in the ports program workspace copy).
//!
//! For every case: parse the input with parse_chains, check the chain /
//! member / scroll structure recorded in expected.tsv (which was generated
//! by calling this library), check every recorded fetch against
//! Chain::fetch, and require serialize_chains(parse_chains(input)) to be
//! byte-identical to the input.

use libphext::phext;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/chains")
}

fn kv(field: &str) -> (&str, &str) {
    let (k, v) = field.split_once('=').expect("key=value field");
    (k, v)
}

#[test]
fn chain_fixtures_conform() {
    let root = fixtures_root();
    let index = fs::read_to_string(root.join("cases.tsv")).expect("cases.tsv");
    let mut case_count = 0usize;
    for line in index.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let case = line.split('\t').next().unwrap().to_string();
        case_count += 1;
        run_case(&root.join(&case), &case);
    }
    assert!(case_count >= 10, "expected at least 10 fixture cases, got {case_count}");
}

fn run_case(dir: &PathBuf, case: &str) {
    let tsv = fs::read_to_string(dir.join("expected.tsv")).expect("expected.tsv");
    let input_bytes = fs::read(dir.join("input.phext")).expect("input.phext");
    let input = String::from_utf8(input_bytes.clone()).expect("fixture input is valid UTF-8");

    let mut expected_chains: Option<usize> = None;
    let mut chain_info: HashMap<usize, (bool, usize)> = HashMap::new();
    let mut member_info: HashMap<(usize, usize), (usize, usize)> = HashMap::new(); // -> (bytes, scrolls)
    let mut fetches: Vec<(usize, String, String)> = Vec::new(); // (chain, address, file)

    for line in tsv.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        match fields[0] {
            "input_bytes" => assert_eq!(fields[1].parse::<usize>().unwrap(), input_bytes.len(), "{case}: input_bytes"),
            "chains" => expected_chains = Some(fields[1].parse().unwrap()),
            "chain" => {
                let ci: usize = fields[1].parse().unwrap();
                let mut closed = false;
                let mut members = 0usize;
                for f in &fields[2..] {
                    match kv(f) {
                        ("closed", v) => closed = v == "true",
                        ("members", v) => members = v.parse().unwrap(),
                        _ => {}
                    }
                }
                chain_info.insert(ci, (closed, members));
            }
            "member" => {
                let ci: usize = fields[1].parse().unwrap();
                let mi: usize = fields[2].parse().unwrap();
                let mut bytes = 0usize;
                let mut scrolls = 0usize;
                for f in &fields[3..] {
                    match kv(f) {
                        ("bytes", v) => bytes = v.parse().unwrap(),
                        ("scrolls", v) => scrolls = v.parse().unwrap(),
                        _ => {}
                    }
                }
                member_info.insert((ci, mi), (bytes, scrolls));
            }
            "fetch" => {
                let mut ci = 0usize;
                let mut address = String::new();
                let mut file = String::new();
                for f in &fields[1..] {
                    match kv(f) {
                        ("chain", v) => ci = v.parse().unwrap(),
                        ("address", v) => address = v.to_string(),
                        ("file", v) => file = v.to_string(),
                        _ => {}
                    }
                }
                fetches.push((ci, address, file));
            }
            _ => {}
        }
    }

    let chains = phext::parse_chains(&input);
    assert_eq!(Some(chains.len()), expected_chains, "{case}: chain count");
    for (ci, chain) in chains.iter().enumerate() {
        let (closed, members) = chain_info[&ci];
        assert_eq!(chain.closed, closed, "{case}: chain {ci} closed");
        assert_eq!(chain.members.len(), members, "{case}: chain {ci} member count");
        for (mi, member) in chain.members.iter().enumerate() {
            let (bytes, scrolls) = member_info[&(ci, mi)];
            assert_eq!(member.len(), bytes, "{case}: member {ci}/{mi} bytes");
            assert_eq!(phext::phokenize(member).len(), scrolls, "{case}: member {ci}/{mi} scroll count");
        }
    }

    // Round-trip expectation for every case: byte-identical re-serialization.
    assert_eq!(phext::serialize_chains(&chains).as_bytes(), input_bytes.as_slice(), "{case}: round trip");

    for (ci, address, file) in &fetches {
        let expected = fs::read_to_string(dir.join(file)).expect("fetch expected file");
        assert_eq!(&chains[*ci].fetch(address), &expected, "{case}: fetch {address} (chain {ci})");
    }
}
