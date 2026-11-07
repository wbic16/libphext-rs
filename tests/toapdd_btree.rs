// TOAPDD — Knuth TAOCP §6.2.4 Multiway trees / B-trees
// (B-trees live inside §6.2.4; there is no section titled "B-trees".)
// Slug: btree · Box: rock-mountain
//
// Two layers, kept distinct — this distinction IS the phext claim:
//
//   (a) THE STORE. One scroll per key record, at a stable coordinate
//       derived from the key itself. Records never move, ever:
//           key k  ->  coordinate 1.1.1/1.1.1/<k>.1.1
//                      (z.series = 1 is the store region;
//                       x.chapter = k, x.section = 1, x.scroll = 1)
//       Content: "record key=<k> value=data-<k>".
//
//   (b) THE INDEX. A B-tree held in a SEPARATE coordinate region
//       (z.series = 2). Each B-tree node is one scroll:
//           node n  ->  coordinate 1.1.2/1.1.1/<n>.1.1
//       Content (line-oriented, parseable by the traversal below):
//           node <n>
//           keys <k1> <k2> ...
//           children <c1> <c2> ...        (absent/empty for a leaf)
//       Node capacity is deliberately small — max 3 keys per node —
//       so a handful of insertions forces splits.
//
// Split rule (the hindsight change over Knuth's disk-block B-tree):
//   a split births NEW scrolls and no existing address moves.
//   - A non-root node that overflows keeps its left half at its own
//     coordinate; the right half is born as a new node scroll; the
//     parent node scroll's content is REPLACED at the SAME parent
//     coordinate to record the promoted key and the new child.
//   - The root (node 1) that overflows is replaced at the SAME root
//     coordinate with the promoted key and two NEW child scrolls
//     holding the two halves. The root address never changes.
//   The store region is not touched by a split at all.
//
// Fixture keys: 10, 20, ..., 120 (12 keys, inserted in ascending
// order). With max 3 keys/node this produces exactly 3 splits:
// the root splits on key 40, a leaf splits on key 70, and a leaf
// splits on key 100, leaving 5 index nodes with root keys [30,60,90].

use libphext::phext;
use std::collections::HashMap;

const MAX_KEYS: usize = 3;
const KEYS: [u32; 12] = [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110, 120];

// ---------------------------------------------------------------------------
// Coordinates — the two regions are distinguished by z.series alone.
// ---------------------------------------------------------------------------

fn store_coord(key: u32) -> phext::Coordinate {
    phext::to_coordinate(&format!("1.1.1/1.1.1/{}.1.1", key))
}

fn index_coord(node_id: u32) -> phext::Coordinate {
    phext::to_coordinate(&format!("1.1.2/1.1.1/{}.1.1", node_id))
}

fn record_content(key: u32) -> String {
    format!("record key={} value=data-{}", key, key)
}

// ---------------------------------------------------------------------------
// Node scroll content: write and parse.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
struct Node {
    id: u32,
    keys: Vec<u32>,
    children: Vec<u32>, // node ids; empty iff leaf
}

impl Node {
    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
    fn content(&self) -> String {
        let keys = self
            .keys
            .iter()
            .map(|k| k.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let children = self
            .children
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        format!("node {}\nkeys {}\nchildren {}\n", self.id, keys, children)
    }
}

/// Parse a node scroll back into (keys, children). Index traversal uses
/// ONLY this plus fetch — it never looks at the driver's in-memory model.
fn parse_node(content: &str) -> (Vec<u32>, Vec<u32>) {
    let mut keys = Vec::new();
    let mut children = Vec::new();
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("keys") {
            keys = rest
                .split_whitespace()
                .map(|t| t.parse::<u32>().expect("key in node scroll"))
                .collect();
        } else if let Some(rest) = line.strip_prefix("children") {
            children = rest
                .split_whitespace()
                .map(|t| t.parse::<u32>().expect("child in node scroll"))
                .collect();
        }
    }
    (keys, children)
}

// ---------------------------------------------------------------------------
// The driver: owns the phext document and the B-tree index model, and
// keeps the index scrolls in the document in sync with the model.
// ---------------------------------------------------------------------------

struct Driver {
    doc: String,
    nodes: HashMap<u32, Node>,
    root: u32,
    next_id: u32,
    splits: usize,
}

impl Driver {
    fn new() -> Driver {
        let mut d = Driver {
            doc: String::new(),
            nodes: HashMap::new(),
            root: 1,
            next_id: 2,
            splits: 0,
        };
        d.nodes.insert(1, Node { id: 1, keys: vec![], children: vec![] });
        d.sync_node(1);
        d
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Write a scroll: insert if absent, replace if changed, no-op if same.
    fn write_scroll(&mut self, coord: phext::Coordinate, content: &str) {
        let existing = phext::fetch(&self.doc, coord);
        if existing == content {
            return;
        }
        if existing.is_empty() {
            self.doc = phext::insert(self.doc.clone(), coord, content);
        } else {
            self.doc = phext::replace(&self.doc, coord, content);
        }
    }

    fn sync_node(&mut self, id: u32) {
        let content = self.nodes.get(&id).expect("node exists").content();
        self.write_scroll(index_coord(id), &content);
    }

    /// STORE layer only: write the record scroll for a key. The index
    /// is not touched.
    fn add_record(&mut self, key: u32) {
        self.write_scroll(store_coord(key), &record_content(key));
    }

    /// Full insert: record into the store, then key into the index.
    fn insert(&mut self, key: u32) {
        self.add_record(key);
        self.index_insert(key);
    }

    /// INDEX layer only: insert a key into the B-tree model, syncing
    /// node scrolls as nodes change / are born. The store is not touched.
    fn index_insert(&mut self, key: u32) {
        if let Some((promoted, right_id)) = self.insert_into(self.root, key) {
            // Root split, in place: the root keeps coordinate node 1.
            // Its left half is copied into a NEW child scroll, the right
            // half already lives in the new scroll `right_id`, and the
            // root scroll is REPLACED at the same coordinate with just
            // the promoted key and the two child pointers.
            let left = self.nodes.get(&self.root).expect("root").clone();
            let left_id = self.alloc_id();
            self.nodes.insert(
                left_id,
                Node { id: left_id, keys: left.keys.clone(), children: left.children.clone() },
            );
            let root = self.nodes.get_mut(&self.root).expect("root");
            root.keys = vec![promoted];
            root.children = vec![left_id, right_id];
            self.sync_node(left_id);
            self.sync_node(right_id);
            self.sync_node(self.root);
        }
    }

    /// Returns Some((promoted_key, new_right_node_id)) iff node `id` split.
    fn insert_into(&mut self, id: u32, key: u32) -> Option<(u32, u32)> {
        let node = self.nodes.get(&id).expect("node exists").clone();
        if node.is_leaf() {
            let mut keys = node.keys.clone();
            let pos = keys.iter().position(|k| key < *k).unwrap_or(keys.len());
            keys.insert(pos, key);
            self.nodes.get_mut(&id).expect("node").keys = keys;
            self.sync_node(id);
            if self.nodes.get(&id).expect("node").keys.len() > MAX_KEYS {
                return Some(self.split_node(id));
            }
            return None;
        }
        // Internal node: descend into the child that should hold `key`.
        let child_idx = node.keys.iter().filter(|k| key > **k).count();
        let child_id = node.children[child_idx];
        if let Some((promoted, new_right)) = self.insert_into(child_id, key) {
            let n = self.nodes.get_mut(&id).expect("node");
            n.keys.insert(child_idx, promoted);
            n.children.insert(child_idx + 1, new_right);
            self.sync_node(id);
            if self.nodes.get(&id).expect("node").keys.len() > MAX_KEYS {
                return Some(self.split_node(id));
            }
        }
        None
    }

    /// Split an over-full node: left half stays at the node's own
    /// coordinate, right half is born as a new node scroll, the median
    /// key is promoted to the caller (the parent).
    fn split_node(&mut self, id: u32) -> (u32, u32) {
        let node = self.nodes.get(&id).expect("node").clone();
        let median = node.keys.len() / 2;
        let promoted = node.keys[median];
        let right_keys = node.keys[median + 1..].to_vec();
        let right_children = if node.is_leaf() {
            Vec::new()
        } else {
            node.children[median + 1..].to_vec()
        };
        let right_id = self.alloc_id();
        self.nodes.insert(
            right_id,
            Node { id: right_id, keys: right_keys, children: right_children },
        );
        let n = self.nodes.get_mut(&id).expect("node");
        n.keys.truncate(median);
        if !n.is_leaf() {
            n.children.truncate(median + 1);
        }
        self.splits += 1;
        self.sync_node(id);
        self.sync_node(right_id);
        (promoted, right_id)
    }
}

// ---------------------------------------------------------------------------
// Region views over the document, via explode (empties filtered).
// ---------------------------------------------------------------------------

fn region(doc: &str, series: usize) -> HashMap<phext::Coordinate, String> {
    phext::explode(doc)
        .into_iter()
        .filter(|(coord, v)| coord.z.series == series && !v.is_empty())
        .collect()
}

fn store_region(doc: &str) -> HashMap<phext::Coordinate, String> {
    region(doc, 1)
}

fn index_region(doc: &str) -> HashMap<phext::Coordinate, String> {
    region(doc, 2)
}

/// In-order traversal driven ONLY by the index scrolls in the document:
/// fetch the node scroll, parse it, walk children interleaved with keys.
fn traverse(doc: &str, node_id: u32) -> Vec<u32> {
    let content = phext::fetch(doc, index_coord(node_id));
    assert!(!content.is_empty(), "index traversal hit missing node {}", node_id);
    let (keys, children) = parse_node(&content);
    if children.is_empty() {
        return keys;
    }
    assert_eq!(children.len(), keys.len() + 1, "node {} is malformed", node_id);
    let mut out = Vec::new();
    for (i, child) in children.iter().enumerate() {
        out.extend(traverse(doc, *child));
        if i < keys.len() {
            out.push(keys[i]);
        }
    }
    out
}

fn build(keys: &[u32]) -> Driver {
    let mut d = Driver::new();
    for k in keys {
        d.insert(*k);
    }
    d
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn store_records_live_at_stable_key_derived_coordinates() {
    let d = build(&KEYS);
    // Every key fetches its record at exactly the coordinate the key
    // itself implies — no index consultation needed.
    for k in KEYS {
        assert_eq!(phext::fetch(&d.doc, store_coord(k)), record_content(k));
    }
    // One scroll per record, in the store region, at exactly those coords.
    let store = store_region(&d.doc);
    assert_eq!(store.len(), KEYS.len());
    for k in KEYS {
        assert_eq!(store.get(&store_coord(k)), Some(&record_content(k)));
    }
    // Store and index are different regions of the same document.
    assert!(store.keys().all(|c| c.z.series == 1));
    assert!(index_region(&d.doc).keys().all(|c| c.z.series == 2));
}

#[test]
fn root_split_births_new_scrolls_and_parent_address_does_not_move() {
    let mut d = Driver::new();
    for k in [10, 20, 30] {
        d.insert(k);
    }
    assert_eq!(d.splits, 0, "3 keys fit in the root leaf, no split yet");
    let root_coord_before = index_coord(d.root);
    let root_before = phext::fetch(&d.doc, root_coord_before);
    let (keys_before, children_before) = parse_node(&root_before);
    assert_eq!(keys_before, vec![10, 20, 30]);
    assert!(children_before.is_empty());
    let index_before = index_region(&d.doc);
    assert_eq!(index_before.len(), 1, "only the root node scroll exists");

    // The 4th key overflows the root (capacity 3): a split must occur.
    d.insert(40);
    assert_eq!(d.splits, 1, "inserting the 4th key must split the root");

    // The parent (root) address did not move: same node id, same coord.
    assert_eq!(d.root, 1);
    assert_eq!(index_coord(d.root), root_coord_before);

    // Fetching the SAME parent coordinate now reflects the split:
    // promoted median key 30, and two child pointers.
    let root_after = phext::fetch(&d.doc, root_coord_before);
    assert_ne!(root_after, root_before);
    let (keys_after, children_after) = parse_node(&root_after);
    assert_eq!(keys_after, vec![30]);
    assert_eq!(children_after.len(), 2);

    // The split birthed exactly two NEW node scrolls, at coordinates
    // that held nothing before the split.
    let index_after = index_region(&d.doc);
    assert_eq!(index_after.len(), 3);
    for child in &children_after {
        let coord = index_coord(*child);
        assert!(!index_before.contains_key(&coord), "child scroll must be new");
        assert!(index_after.contains_key(&coord));
    }
    // The two children hold the two halves, in order.
    let (left_keys, _) = parse_node(&phext::fetch(&d.doc, index_coord(children_after[0])));
    let (right_keys, _) = parse_node(&phext::fetch(&d.doc, index_coord(children_after[1])));
    assert_eq!(left_keys, vec![10, 20]);
    assert_eq!(right_keys, vec![40]);
}

#[test]
fn store_is_byte_identical_across_a_split() {
    // Root split, isolated: write key 40's RECORD first, snapshot the
    // whole store region, then drive the split through the INDEX only.
    // Explode-diff of the store region must be empty.
    let mut d = Driver::new();
    for k in [10, 20, 30] {
        d.insert(k);
    }
    d.add_record(40);
    let store_before = store_region(&d.doc);
    d.index_insert(40);
    assert_eq!(d.splits, 1);
    assert_eq!(store_region(&d.doc), store_before, "root split touched the store");

    // Leaf split, isolated the same way: build to the point where the
    // right leaf is full ([40,50,60] under root [30]), record 70 in
    // the store, snapshot, then split via the index only.
    for k in [50, 60] {
        d.insert(k);
    }
    d.add_record(70);
    let store_before = store_region(&d.doc);
    let splits_before = d.splits;
    d.index_insert(70);
    assert_eq!(d.splits, splits_before + 1, "key 70 must split the full leaf");
    assert_eq!(store_region(&d.doc), store_before, "leaf split touched the store");
}

#[test]
fn keys_fetched_before_a_split_are_fetchable_at_the_same_data_after_it() {
    let mut d = Driver::new();
    for k in [10, 20, 30] {
        d.insert(k);
    }
    // Snapshot every key's store coordinate and content BEFORE any split.
    let before: HashMap<u32, (phext::Coordinate, String)> = [10, 20, 30]
        .iter()
        .map(|k| (*k, (store_coord(*k), phext::fetch(&d.doc, store_coord(*k)))))
        .collect();

    // Drive the remaining inserts: 3 splits reshape the index entirely.
    for k in [40, 50, 60, 70, 80, 90, 100, 110, 120] {
        d.insert(k);
    }
    assert_eq!(d.splits, 3);

    // The pre-split keys are fetchable at the SAME coordinate with the
    // SAME content — the store did not notice the index changed.
    for (k, (coord, content)) in &before {
        assert_eq!(&store_coord(*k), coord);
        assert_eq!(&phext::fetch(&d.doc, *coord), content, "key {} moved or changed", k);
    }
    // And so is every later key, at its own key-derived coordinate.
    for k in KEYS {
        assert_eq!(phext::fetch(&d.doc, store_coord(k)), record_content(k));
    }
}

#[test]
fn index_traversal_finds_all_keys() {
    let d = build(&KEYS);
    // Walking only the node scrolls from the root recovers every key,
    // in sorted (in-order) sequence.
    let found = traverse(&d.doc, d.root);
    assert_eq!(found, KEYS.to_vec());
    // Every key the index names resolves to its record in the store.
    for k in &found {
        assert_eq!(phext::fetch(&d.doc, store_coord(*k)), record_content(*k));
    }
}

#[test]
fn repeated_splits_never_move_the_root_or_any_record() {
    let d = build(&KEYS);
    assert_eq!(d.splits, 3, "fixture should force exactly 3 splits");
    // The root is still node 1 at its original coordinate, now holding
    // the three promoted medians and four child pointers.
    assert_eq!(d.root, 1);
    let (root_keys, root_children) =
        parse_node(&phext::fetch(&d.doc, index_coord(1)));
    assert_eq!(root_keys, vec![30, 60, 90]);
    assert_eq!(root_children.len(), 4);
    // 5 index node scrolls total (root + 4 leaves), all in region 2.
    assert_eq!(index_region(&d.doc).len(), 5);
    // 12 record scrolls, all in region 1, none moved or rewritten.
    let store = store_region(&d.doc);
    assert_eq!(store.len(), KEYS.len());
    for k in KEYS {
        assert_eq!(store.get(&store_coord(k)), Some(&record_content(k)));
    }
}
