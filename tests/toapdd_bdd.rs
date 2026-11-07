// TOAPDD #17 — Binary decision diagrams / ZDDs (TAOCP Vol 4A §7.1.4)
// Phext-native claim: a Boolean function's truth table is one scroll per
// assignment; a reduced diagram stores each DISTINCT subfunction exactly
// once, deduped by checksum() of its canonical text; evaluation is a
// coordinate walk from the root scroll, following child coordinates named
// in parent scrolls. Identical subtrees share one scroll, literally:
// two parents that need the same subfunction reference the same coordinate.
//
// Function: f(a,b,c) = (a AND b) OR (a AND c) = a AND (b OR c)
// Variable order a, b, c. Genuine sharing:
//   f: a=0 -> leaf 0,        a=1 -> g
//   g: b=0 -> h,             b=1 -> leaf 1
//   h: c=0 -> leaf 0,        c=1 -> leaf 1
// leaf 0 is shared by f and h; leaf 1 is shared by g and h.
// Unshared decision tree: 7 internal nodes + 8 leaves = 15 nodes.
// Distinct subfunctions:  {f, g, h, leaf0, leaf1} = 5 stored scrolls.
use libphext::phext;
use std::collections::HashMap;

fn f(a: bool, b: bool, c: bool) -> bool {
    (a && b) || (a && c)
}

const VARS: [&str; 3] = ["a", "b", "c"];

fn coord_str(c: phext::Coordinate) -> String {
    format!(
        "{}.{}.{}/{}.{}.{}/{}.{}.{}",
        c.z.library, c.z.shelf, c.z.series,
        c.y.collection, c.y.volume, c.y.book,
        c.x.chapter, c.x.section, c.x.scroll
    )
}

// ---------- Artefact 1: truth table ----------

fn truth_coord(i: usize) -> phext::Coordinate {
    // Coordinate encodes the assignment index (bits a,b,c of i).
    phext::to_coordinate(&format!("1.1.1/1.1.1/1.1.{}", i + 1))
}

fn build_truth_table() -> String {
    let mut p = String::new();
    for i in 0..8usize {
        let (a, b, c) = (i & 4 != 0, i & 2 != 0, i & 1 != 0);
        let out = if f(a, b, c) { "1" } else { "0" };
        p = phext::insert(p, truth_coord(i), out);
    }
    p
}

// ---------- Artefact 2: reduced diagram, deduped by checksum ----------

#[derive(Clone)]
struct NodeInfo {
    canonical: String,
    coord: phext::Coordinate,
}

struct Builder {
    diagram: String,
    by_checksum: HashMap<String, phext::Coordinate>,
    stored: usize,
    tree_nodes: usize,
    next_slot: usize,
}

impl Builder {
    fn new() -> Self {
        Builder {
            diagram: String::new(),
            by_checksum: HashMap::new(),
            stored: 0,
            tree_nodes: 0,
            next_slot: 1,
        }
    }

    fn alloc(&mut self) -> phext::Coordinate {
        let coord = phext::to_coordinate(&format!("1.1.1/1.2.1/1.1.{}", self.next_slot));
        self.next_slot += 1;
        coord
    }

    /// Recursively build the subfunction at `depth` under `prefix`
    /// (prefix[d] = assignment of variable d, for d < depth).
    /// Every call counts as one node of the UNSHARED tree; the scroll is
    /// stored only the first time its canonical text's checksum is seen.
    fn build(&mut self, depth: usize, prefix: &[bool; 3]) -> NodeInfo {
        self.tree_nodes += 1;
        let (canonical, content_children): (String, Option<(String, NodeInfo, NodeInfo)>) =
            if depth == 3 {
                let v = f(prefix[0], prefix[1], prefix[2]);
                (format!("leaf:{}", if v { 1 } else { 0 }), None)
            } else {
                let mut low_prefix = *prefix;
                low_prefix[depth] = false;
                let low = self.build(depth + 1, &low_prefix);
                let mut high_prefix = *prefix;
                high_prefix[depth] = true;
                let high = self.build(depth + 1, &high_prefix);
                // BDD reduction, elimination rule: a test whose two
                // children are the same shared subfunction is redundant
                // — store nothing and return the child itself. (This is
                // what collapses e.g. the whole a=0 subtree to leaf 0.)
                if low.coord == high.coord {
                    return low;
                }
                let canonical = format!(
                    "var:{} low:[{}] high:[{}]",
                    VARS[depth], low.canonical, high.canonical
                );
                (canonical, Some((VARS[depth].to_string(), low, high)))
            };

        // Dedup key: checksum of the canonical text wrapped as a
        // single-scroll phext (checksum is over a whole phext string).
        let single = phext::insert(
            String::new(),
            phext::to_coordinate("1.1.1/1.1.1/1.1.1"),
            &canonical,
        );
        let key = phext::checksum(&single);
        if let Some(existing) = self.by_checksum.get(&key) {
            return NodeInfo { canonical, coord: *existing };
        }
        let coord = self.alloc();
        let content = match &content_children {
            None => canonical.clone(),
            Some((var, low, high)) => format!(
                "node var={} low={} high={}",
                var,
                coord_str(low.coord),
                coord_str(high.coord)
            ),
        };
        self.diagram = phext::insert(self.diagram.clone(), coord, &content);
        self.by_checksum.insert(key, coord);
        self.stored += 1;
        NodeInfo { canonical, coord }
    }
}

fn build_diagram() -> (Builder, phext::Coordinate) {
    let mut b = Builder::new();
    let root = b.build(0, &[false; 3]);
    (b, root.coord)
}

fn children(content: &str) -> (String, String) {
    let mut low = String::new();
    let mut high = String::new();
    for tok in content.split_whitespace() {
        if let Some(v) = tok.strip_prefix("low=") {
            low = v.to_string();
        } else if let Some(v) = tok.strip_prefix("high=") {
            high = v.to_string();
        }
    }
    (low, high)
}

/// Evaluation = coordinate walk: fetch root scroll, follow the child
/// coordinate named for the current variable's assigned value, repeat
/// until a leaf scroll is reached.
fn eval(diagram: &str, root: phext::Coordinate, assign: &[bool; 3]) -> bool {
    let mut coord = root;
    let mut depth = 0usize;
    loop {
        let content = phext::fetch(diagram, coord);
        assert!(!content.is_empty(), "walk reached a missing scroll at depth {}", depth);
        if let Some(v) = content.strip_prefix("leaf:") {
            return v == "1";
        }
        assert!(content.contains(&format!("var={}", VARS[depth])),
            "node at depth {} must test variable {}: {}", depth, VARS[depth], content);
        let (low, high) = children(&content);
        let next = if assign[depth] { &high } else { &low };
        coord = phext::to_coordinate(next);
        depth += 1;
        assert!(depth <= 3, "walk must reach a leaf within 3 steps");
    }
}

#[test]
fn truth_table_scrolls_match_function_for_all_assignments() {
    let p = build_truth_table();
    let map = phext::explode(&p);
    let nonempty: Vec<_> = map.values().filter(|v| !v.trim().is_empty()).collect();
    assert_eq!(nonempty.len(), 8, "one scroll per assignment");
    for i in 0..8usize {
        let (a, b, c) = (i & 4 != 0, i & 2 != 0, i & 1 != 0);
        let expected = if f(a, b, c) { "1" } else { "0" };
        assert_eq!(phext::fetch(&p, truth_coord(i)), expected,
            "assignment a={} b={} c={}", a, b, c);
    }
}

#[test]
fn reduced_diagram_dedup_by_checksum_shares_subtrees() {
    let (b, _root) = build_diagram();
    // Unshared tree: full depth-3 binary tree = 15 nodes.
    assert_eq!(b.tree_nodes, 15, "unshared tree nodes");
    // Distinct subfunctions: f, g, h, leaf0, leaf1.
    assert_eq!(b.stored, 5, "distinct subfunctions stored once each");
    assert!(b.stored < b.tree_nodes);
    // The stored diagram really holds exactly 5 non-empty scrolls.
    let map = phext::explode(&b.diagram);
    let nonempty: Vec<_> = map.values().filter(|v| !v.trim().is_empty()).collect();
    assert_eq!(nonempty.len(), 5);
    // Exactly one scroll in the whole diagram holds each leaf text:
    // the leaves are shared records, not copies.
    assert_eq!(map.values().filter(|v| v.trim() == "leaf:0").count(), 1);
    assert_eq!(map.values().filter(|v| v.trim() == "leaf:1").count(), 1);
    // Sharing ratio 15:5 = 3.0 (reported in RESULT.md).
    assert_eq!(b.tree_nodes as f64 / b.stored as f64, 3.0);
}

#[test]
fn evaluation_is_a_coordinate_walk_and_matches_function() {
    let (b, root) = build_diagram();
    for i in 0..8usize {
        let assign = [i & 4 != 0, i & 2 != 0, i & 1 != 0];
        assert_eq!(eval(&b.diagram, root, &assign), f(assign[0], assign[1], assign[2]),
            "walk for assignment {:?}", assign);
    }
    // Diagram evaluation and truth table agree on every assignment.
    let tt = build_truth_table();
    for i in 0..8usize {
        let assign = [i & 4 != 0, i & 2 != 0, i & 1 != 0];
        let from_table = phext::fetch(&tt, truth_coord(i)) == "1";
        assert_eq!(eval(&b.diagram, root, &assign), from_table);
    }
}

#[test]
fn two_parents_share_a_subfunction_by_the_same_coordinate() {
    let (b, root) = build_diagram();
    // Root is f (tests a). Its high child is g (tests b); g's low child is h (tests c).
    let root_content = phext::fetch(&b.diagram, root);
    let (f_low, f_high) = children(&root_content);
    let g_content = phext::fetch(&b.diagram, phext::to_coordinate(&f_high));
    let (g_low, g_high) = children(&g_content);
    let h_content = phext::fetch(&b.diagram, phext::to_coordinate(&g_low));
    let (h_low, h_high) = children(&h_content);
    // f and h both need leaf 0 — and reference the SAME coordinate for it.
    assert_eq!(f_low, h_low, "f.low and h.low must be the same shared leaf-0 scroll");
    // g and h both need leaf 1 — same coordinate again.
    assert_eq!(g_high, h_high, "g.high and h.high must be the same shared leaf-1 scroll");
    // And those coordinates fetch the leaf texts.
    assert_eq!(phext::fetch(&b.diagram, phext::to_coordinate(&f_low)), "leaf:0");
    assert_eq!(phext::fetch(&b.diagram, phext::to_coordinate(&g_high)), "leaf:1");
}
