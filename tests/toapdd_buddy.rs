// TOAPDD #11 — Knuth TAOCP Vol.1 §2.5 Dynamic storage allocation / buddy system.
//
// Phext-native claim: keep the LEDGER in phext. A buddy allocator is simulated
// by the test driver over a 64-unit arena; every event (init / split / alloc /
// free / merge) is appended as a NEW scroll at a NEW coordinate
// (1.1.1/1.1.1/1.1.<event_no>) — an append-only log, never an in-place edit.
// Lineage of a block and the fifty-percent-rule ratio are both answered by
// walking the ledger scrolls alone (phokenize + fetch), not by peeking at
// the allocator's internal state.
//
// Public API only: libphext::phext::{insert, fetch, phokenize, to_coordinate}.

use libphext::phext;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Ledger: an append-only phext. Event n lives at scroll n of chapter 1.
// ---------------------------------------------------------------------------

fn event_coord(n: usize) -> phext::Coordinate {
    phext::to_coordinate(&format!("1.1.1/1.1.1/1.1.{}", n))
}

struct Ledger {
    text: String,
    events: usize,
}

impl Ledger {
    fn new() -> Ledger {
        Ledger { text: String::new(), events: 0 }
    }
    /// Append = insert at a brand-new coordinate. Existing scrolls untouched.
    fn append(&mut self, record: &str) {
        self.events += 1;
        self.text = phext::insert(self.text.clone(), event_coord(self.events), record);
    }
    fn fetch_event(&self, n: usize) -> String {
        phext::fetch(&self.text, event_coord(n))
    }
    /// All non-empty event scrolls, in coordinate (== event) order.
    fn all_events(&self) -> Vec<String> {
        let mut ps: Vec<_> = phext::phokenize(&self.text)
            .into_iter()
            .filter(|p| !p.scroll.trim().is_empty())
            .collect();
        ps.sort();
        ps.into_iter().map(|p| p.scroll).collect()
    }
}

// ---------------------------------------------------------------------------
// Buddy allocator (driver-side simulation; the phext above is the truth log)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
struct Block {
    id: u64,
    offset: usize,
    size: usize,
    allocated: bool,
}

struct Buddy {
    ledger: Ledger,
    blocks: Vec<Block>, // current partition of the 64-unit arena
    next_id: u64,
}

const ARENA: usize = 64;

impl Buddy {
    fn new() -> Buddy {
        let mut b = Buddy { ledger: Ledger::new(), blocks: vec![Block { id: 0, offset: 0, size: ARENA, allocated: false }], next_id: 1 };
        b.ledger.append("type=init;block=B0;offset=0;size=64");
        b
    }
    fn fresh_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
    fn alloc(&mut self, request: usize) -> u64 {
        let need = request.next_power_of_two().max(4);
        // Smallest free block that fits; ties broken by lowest offset.
        let mut cand: Option<usize> = None;
        for (i, blk) in self.blocks.iter().enumerate() {
            if blk.allocated || blk.size < need {
                continue;
            }
            match cand {
                None => cand = Some(i),
                Some(c) => {
                    let cb = self.blocks[c];
                    if blk.size < cb.size || (blk.size == cb.size && blk.offset < cb.offset) {
                        cand = Some(i);
                    }
                }
            }
        }
        let idx = cand.expect("arena exhausted for request");
        // Split down to exactly `need`, recording every split.
        while self.blocks[idx].size > need {
            let parent = self.blocks.remove(idx);
            let half = parent.size / 2;
            let left = Block { id: self.fresh_id(), offset: parent.offset, size: half, allocated: false };
            let right = Block { id: self.fresh_id(), offset: parent.offset + half, size: half, allocated: false };
            self.ledger.append(&format!(
                "type=split;parent=B{};poffset={};psize={};left=B{};right=B{};size={}",
                parent.id, parent.offset, parent.size, left.id, right.id, half
            ));
            self.blocks.insert(idx, right);
            self.blocks.insert(idx, left);
            // continue splitting the left child (lowest offset that fits)
        }
        self.blocks[idx].allocated = true;
        let blk = self.blocks[idx];
        self.ledger.append(&format!(
            "type=alloc;block=B{};offset={};size={};request={}",
            blk.id, blk.offset, blk.size, request
        ));
        blk.id
    }
    fn free(&mut self, id: u64) {
        let idx = self.blocks.iter().position(|b| b.id == id).expect("free of unknown block");
        assert!(self.blocks[idx].allocated, "double free in simulation");
        self.blocks[idx].allocated = false;
        let blk = self.blocks[idx];
        self.ledger.append(&format!("type=free;block=B{};offset={};size={}", blk.id, blk.offset, blk.size));
        // Merge with the buddy while the buddy is a single free block.
        // The merge cursor is explicit local state: (offset, size) of the
        // block currently trying to coalesce, updated after each merge.
        let mut cur_offset = blk.offset;
        let mut cur_size = blk.size;
        loop {
            let buddy_offset = cur_offset ^ cur_size;
            let cur_idx = self.blocks.iter().position(|b| !b.allocated && b.offset == cur_offset && b.size == cur_size)
                .expect("merge cursor must name a current free block");
            let cur = self.blocks[cur_idx];
            let buddy_idx = self.blocks.iter().position(|b| !b.allocated && b.id != cur.id && b.offset == buddy_offset && b.size == cur_size);
            let bi = match buddy_idx { Some(i) => i, None => break };
            let buddy = self.blocks[bi];
            let (lo, hi) = if cur.offset < buddy.offset { (cur, buddy) } else { (buddy, cur) };
            let parent = Block { id: self.fresh_id(), offset: lo.offset, size: cur_size * 2, allocated: false };
            self.ledger.append(&format!(
                "type=merge;left=B{};right=B{};parent=B{};offset={};size={}",
                lo.id, hi.id, parent.id, parent.offset, parent.size
            ));
            self.blocks.retain(|b| b.id != cur.id && b.id != buddy.id);
            self.blocks.push(parent);
            self.blocks.sort_by_key(|b| b.offset);
            cur_offset = parent.offset;
            cur_size = parent.size;
            if cur_size >= ARENA { break; }
        }
    }
}

// ---------------------------------------------------------------------------
// Ledger-only queries: parse event scrolls, never the allocator's Vec<Block>.
// ---------------------------------------------------------------------------

fn parse(record: &str) -> HashMap<String, String> {
    record.split(';').filter_map(|kv| {
        let mut it = kv.splitn(2, '=');
        Some((it.next()?.to_string(), it.next()?.to_string()))
    }).collect()
}

/// Lineage of a block = its split ancestry, walked from ledger events alone.
/// Returns ids from the block itself up to the root (B0).
fn lineage(ledger: &Ledger, block: &str) -> Vec<String> {
    let mut parent_of: HashMap<String, String> = HashMap::new();
    for rec in ledger.all_events() {
        let f = parse(&rec);
        if f.get("type").map(|s| s.as_str()) == Some("split") {
            parent_of.insert(f["left"].clone(), f["parent"].clone());
            parent_of.insert(f["right"].clone(), f["parent"].clone());
        }
    }
    let mut chain = vec![block.to_string()];
    let mut cur = block.to_string();
    while let Some(p) = parent_of.get(&cur) {
        chain.push(p.clone());
        cur = p.clone();
    }
    chain
}

/// Replay the whole ledger and return the final partition:
/// Vec of (offset, size, allocated). Pure ledger query.
fn replay(ledger: &Ledger) -> Vec<(usize, usize, bool)> {
    let mut blocks: HashMap<String, (usize, usize, bool)> = HashMap::new();
    for rec in ledger.all_events() {
        let f = parse(&rec);
        match f.get("type").map(|s| s.as_str()) {
            Some("init") => {
                blocks.insert(f["block"].clone(), (f["offset"].parse().unwrap(), f["size"].parse().unwrap(), false));
            }
            Some("split") => {
                let (po, _, _) = blocks.remove(&f["parent"]).expect("split of unknown parent");
                let half: usize = f["size"].parse().unwrap();
                blocks.insert(f["left"].clone(), (po, half, false));
                blocks.insert(f["right"].clone(), (po + half, half, false));
            }
            Some("alloc") => {
                let e = blocks.get_mut(&f["block"]).expect("alloc of unknown block");
                e.2 = true;
            }
            Some("free") => {
                let e = blocks.get_mut(&f["block"]).expect("free of unknown block");
                e.2 = false;
            }
            Some("merge") => {
                blocks.remove(&f["left"]);
                blocks.remove(&f["right"]);
                blocks.insert(f["parent"].clone(), (f["offset"].parse().unwrap(), f["size"].parse().unwrap(), false));
            }
            other => panic!("unknown event type {:?}", other),
        }
    }
    let mut out: Vec<(usize, usize, bool)> = blocks.values().cloned().collect();
    out.sort();
    out
}

/// The scripted sequence from the spec: fills the 64-unit arena, then frees
/// in an order that forces a merge cascade. Returns (buddy, [a,b,c,d] ids).
fn scripted() -> (Buddy, [u64; 4]) {
    let mut b = Buddy::new();
    let a = b.alloc(8);
    let bb = b.alloc(16);
    let c = b.alloc(8);
    let d = b.alloc(20); // rounds to 32
    let ids = [a, bb, c, d];
    b.free(ids[1]); // free the 16 — buddy (offset 0, size 16) does not exist as one block: no merge
    b.free(ids[0]); // free the first 8 — buddy (offset 8) still allocated: no merge
    b.free(ids[2]); // free the second 8 — cascades: 8+8 -> 16, then 16+16 -> 32
    (b, ids)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn append_only_ledger_events_are_never_modified() {
    let (buddy, _) = scripted();
    // Snapshot-as-written check: re-run the same sequence, capturing each
    // event scroll's bytes at the moment the NEXT event was appended is not
    // possible post-hoc — so instead verify the stronger post-hoc property
    // available here: fetching every event after all appends returns exactly
    // the parsed event stream phokenize yields, event count matches the log,
    // and each event scroll is non-empty and mentions its own type.
    let n = buddy.ledger.events;
    assert_eq!(n, 13, "scripted log is deterministic: init + 3 splits + 4 allocs + 3 frees + 2 merges");
    let walked = buddy.ledger.all_events();
    assert_eq!(walked.len(), n, "phokenize must yield exactly one scroll per event");
    for (i, ev) in walked.iter().enumerate() {
        let fetched = buddy.ledger.fetch_event(i + 1);
        assert_eq!(&fetched, ev, "event {} changed between phokenize and fetch", i + 1);
        assert!(ev.contains("type="), "event {} lacks a type field: {}", i + 1, ev);
    }
    // First and last events pin the log's shape.
    assert!(walked[0].starts_with("type=init"), "log must open with init, got {}", walked[0]);
    assert!(walked[n - 1].starts_with("type=merge"), "scripted log must end with the merge cascade, got {}", walked[n - 1]);

    // True append-only check: build incrementally and snapshot along the way.
    let mut b = Buddy::new();
    let mut snapshots: Vec<String> = vec![b.ledger.fetch_event(1)];
    let a = b.alloc(8);
    for i in 2..=b.ledger.events { snapshots.push(b.ledger.fetch_event(i)); }
    let _ = b.alloc(16);
    let _ = b.free(a);
    // Every snapshot taken earlier must still fetch byte-identically now,
    // after later appends (including merges) have landed.
    for (i, snap) in snapshots.iter().enumerate() {
        assert_eq!(&b.ledger.fetch_event(i + 1), snap, "event {} was modified by a later append", i + 1);
    }
}

#[test]
fn split_merge_recorded_and_lineage_fetchable_from_ledger() {
    let mut b = Buddy::new();
    let _a = b.alloc(8);
    let _bb = b.alloc(16);
    let c = b.alloc(8);
    let d = b.alloc(20);

    let types: Vec<String> = b.ledger.all_events().iter().map(|r| parse(r)["type"].clone()).collect();
    assert!(types.iter().filter(|t| *t == "split").count() >= 3, "allocations must record splits: {:?}", types);
    assert_eq!(types.iter().filter(|t| *t == "alloc").count(), 4);

    // Lineage of the 8-unit block at offset 8 (third alloc): root -> 32 -> 16 -> 8.
    let cname = format!("B{}", c);
    let chain = lineage(&b.ledger, &cname);
    assert_eq!(chain.len(), 4, "lineage of {} should be [self, 16-parent, 32-parent, B0], got {:?}", cname, chain);
    assert_eq!(chain.last().unwrap(), "B0", "lineage must terminate at the arena root");

    // Lineage of the 32-unit block (fourth alloc): direct right child of root.
    let dname = format!("B{}", d);
    let dchain = lineage(&b.ledger, &dname);
    assert_eq!(dchain, vec![dname.clone(), "B0".to_string()], "32-unit block is a direct split child of the root");

    // Now the frees: merges must appear in the ledger, in cascade order
    // (an 8+8 merge strictly before the 16+16 merge it enables).
    let (b2, _) = scripted();
    let evs = b2.ledger.all_events();
    let merge_sizes: Vec<usize> = evs.iter().filter_map(|r| {
        let f = parse(r);
        if f.get("type").map(|s| s.as_str()) == Some("merge") { Some(f["size"].parse().unwrap()) } else { None }
    }).collect();
    assert_eq!(merge_sizes, vec![16, 32], "expected exactly the 8+8->16 then 16+16->32 merge cascade");
}

#[test]
fn fifty_percent_rule_is_answerable_as_a_ledger_query() {
    let (buddy, _) = scripted();
    // Replay ONLY the ledger; do not consult buddy.blocks for the verdict
    // (we compare against it afterwards as a cross-check of the replay).
    let final_partition = replay(&buddy.ledger);
    let total: usize = final_partition.iter().map(|(_, s, _)| *s).sum();
    let free: usize = final_partition.iter().filter(|(_, _, a)| !a).map(|(_, s, _)| *s).sum();
    assert_eq!(total, ARENA, "ledger replay must account for the whole arena");
    assert!(free <= total, "free cannot exceed total");
    let ratio = free as f64 / total as f64;
    assert!((0.0..=1.0).contains(&ratio), "ratio must be a well-defined number in [0,1], got {}", ratio);

    // Cross-check: the ledger replay agrees with the allocator's own state.
    let mut driver_state: Vec<(usize, usize, bool)> = buddy.blocks.iter().map(|b| (b.offset, b.size, b.allocated)).collect();
    driver_state.sort();
    assert_eq!(final_partition, driver_state, "ledger replay must reproduce the allocator's final partition");

    // The measured steady state of THIS scripted sequence: one 32-unit block
    // held, one 32-unit block free — the fifty-percent rule's textbook case,
    // observed here, not asserted as a universal law.
    assert_eq!(final_partition.len(), 2, "steady state should be two 32-unit blocks, got {:?}", final_partition);
    assert_eq!(ratio, 0.5, "scripted steady state sits exactly at the fifty-percent point (observation)");
}
