// TOAPDD attempt #5 — Knuth TAOCP Vol. 1, §2.3.1: Traversing binary trees / threaded trees.
//
// Phext-native claim to prove:
//   A binary tree is stored as scrolls whose *coordinates encode the tree path*.
//   A null child is not a thread bit, not a null pointer, and not a stored field:
//   it is simply the absence of a scroll at the child's coordinate
//   (`fetch` returns ""). Traversal completes anyway.
//
// Coordinate encoding (documented, found from the tree's own structure):
//   A phext Coordinate is nine components in lexicographic order:
//     [library, shelf, series, collection, volume, book, chapter, section, scroll]
//   The path from the root is written one step per component, in that order:
//     component = 1  -> no step here (this node, or an ancestor, ended above)
//     component = 2  -> the step at this depth went LEFT
//     component = 3  -> the step at this depth went RIGHT
//   The root is therefore 1.1.1/1.1.1/1.1.1. A node's children are obtained by
//   setting the component at index = depth to 2 (left) or 3 (right); every
//   deeper component stays 1. Depth is at most 9 in this encoding (one step
//   per component) — the fixture tree uses depth 3.
//
// Honesty note — which order is the document order?
//   `next_scroll` walks in coordinate (document) order. Under this encoding
//   document order IS preorder: lexicographic order visits a coordinate
//   before any coordinate it is a prefix of (the node before its subtrees),
//   and a left step (2) always sorts before a right step (3) at the first
//   differing component (the whole left subtree before the right subtree).
//   So preorder is proved by the `next_scroll` walk itself. Inorder and
//   postorder are NOT document order here; they are implemented below as
//   driver walks over `fetch`, computing child coordinates from the
//   encoding. No traversal stores anything the others need.
//
// The stored scroll content is exactly the node's value ("F", "B", ...).
// No thread bits, no link fields, no parent pointers are stored anywhere —
// asserted explicitly in `no_thread_bits_or_link_fields_are_stored`.

use libphext::phext::{self, Coordinate};

const L: bool = false; // left step
const R: bool = true; // right step

fn components(coord: Coordinate) -> [usize; 9] {
    [
        coord.z.library,
        coord.z.shelf,
        coord.z.series,
        coord.y.collection,
        coord.y.volume,
        coord.y.book,
        coord.x.chapter,
        coord.x.section,
        coord.x.scroll,
    ]
}

fn from_components(c: [usize; 9]) -> Coordinate {
    phext::to_coordinate(&format!(
        "{}.{}.{}/{}.{}.{}/{}.{}.{}",
        c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7], c[8]
    ))
}

/// Coordinate of the node reached by `path` from the root ([] = root).
fn coord_for_path(path: &[bool]) -> Coordinate {
    assert!(path.len() <= 9, "encoding holds at most one step per component");
    let mut c = [1usize; 9];
    for (depth, &step) in path.iter().enumerate() {
        c[depth] = if step { 3 } else { 2 };
    }
    from_components(c)
}

/// Coordinate of the child of the node at `coord`/`depth` in direction `right`.
fn child(coord: Coordinate, depth: usize, right: bool) -> Coordinate {
    let mut c = components(coord);
    assert_eq!(c[depth], 1, "child is written into the first unused component");
    c[depth] = if right { 3 } else { 2 };
    from_components(c)
}

// The fixture tree — a binary search tree with deliberate interior nulls:
//
//            F
//          /   \
//         B     G
//        / \     \
//       A   D     I
//          /     /
//         C     H
//
// G has no left child, D has no right child, I has no right child, and
// A, C, H are leaves (both children null). A complete tree of this shape
// would have 15 node slots; only 8 scrolls exist.
fn fixture() -> Vec<(&'static [bool], &'static str)> {
    vec![
        (&[], "F"),
        (&[L], "B"),
        (&[R], "G"),
        (&[L, L], "A"),
        (&[L, R], "D"),
        (&[R, R], "I"),
        (&[L, R, L], "C"),
        (&[R, R, L], "H"),
    ]
}

/// Build the phext by inserting each node's scroll at its path coordinate,
/// in preorder, so the inserts land in document order.
fn build_tree() -> String {
    let mut order = fixture();
    // preorder of the fixture: F B A D C G I H
    order.sort_by_key(|(path, _)| coord_for_path(path));
    let mut doc = String::new();
    for (path, value) in order {
        doc = phext::insert(doc, coord_for_path(path), value);
    }
    doc
}

/// The document-order walk: `next_scroll` over the remaining string, exactly
/// the loop the library's own phokenize reference implementation uses.
fn document_order(doc: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut coord = phext::default_coordinate();
    let mut rest = doc.to_string();
    loop {
        let (item, next, remaining) = phext::next_scroll(rest.as_str(), coord);
        coord = next;
        rest = remaining;
        if !item.scroll.is_empty() {
            out.push(item.scroll);
        }
        if rest.is_empty() {
            break;
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq)]
enum Order {
    Pre,
    In,
    Post,
}

/// Driver traversal over `fetch`: a subtree exists iff its root coordinate
/// fetches non-empty. A missing child fetches "" and the walk simply
/// returns — no thread, no stack fix-up, no stored link is consulted.
fn driver_walk(doc: &str, coord: Coordinate, depth: usize, order: Order, out: &mut Vec<String>) {
    let value = phext::fetch(doc, coord);
    if value.is_empty() {
        return; // null child / absent subtree: absence is the whole encoding
    }
    assert!(depth < 9, "fixture depth stays inside the 9-component encoding");
    let left = child(coord, depth, L);
    let right = child(coord, depth, R);
    match order {
        Order::Pre => {
            out.push(value);
            driver_walk(doc, left, depth + 1, order, out);
            driver_walk(doc, right, depth + 1, order, out);
        }
        Order::In => {
            driver_walk(doc, left, depth + 1, order, out);
            out.push(value);
            driver_walk(doc, right, depth + 1, order, out);
        }
        Order::Post => {
            driver_walk(doc, left, depth + 1, order, out);
            driver_walk(doc, right, depth + 1, order, out);
            out.push(value);
        }
    }
}

fn traverse(doc: &str, order: Order) -> Vec<String> {
    let mut out = Vec::new();
    driver_walk(doc, coord_for_path(&[]), 0, order, &mut out);
    out
}

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| s.to_string()).collect()
}

#[test]
fn encoding_places_each_node_at_its_path_coordinate() {
    // root, and one node per interesting shape of path
    assert_eq!(coord_for_path(&[]).to_string(), "1.1.1/1.1.1/1.1.1");
    assert_eq!(coord_for_path(&[L]).to_string(), "2.1.1/1.1.1/1.1.1");
    assert_eq!(coord_for_path(&[R]).to_string(), "3.1.1/1.1.1/1.1.1");
    assert_eq!(coord_for_path(&[L, R, L]).to_string(), "2.3.2/1.1.1/1.1.1");
    assert_eq!(coord_for_path(&[R, R, L]).to_string(), "3.3.2/1.1.1/1.1.1");

    // children are derived from the parent's coordinate + depth, not stored
    let f = coord_for_path(&[]);
    assert_eq!(child(f, 0, L), coord_for_path(&[L]));
    assert_eq!(child(f, 0, R), coord_for_path(&[R]));
    let d = coord_for_path(&[L, R]);
    assert_eq!(child(d, 2, L), coord_for_path(&[L, R, L]));
    assert_eq!(child(d, 2, R), coord_for_path(&[L, R, R]));

    // every fixture node fetches back its own value at its path coordinate
    let doc = build_tree();
    for (path, value) in fixture() {
        assert_eq!(phext::fetch(&doc, coord_for_path(path)), value);
    }
}

#[test]
fn document_order_walk_with_next_scroll_is_preorder() {
    let doc = build_tree();
    // Document (coordinate) order under the path encoding is preorder —
    // see the honesty note at the top of this file.
    assert_eq!(document_order(&doc), strings(&["F", "B", "A", "D", "C", "G", "I", "H"]));

    // phokenize, the library's own coordinate-order enumeration, agrees,
    // and sees exactly the 8 present nodes — no scrolls for null children.
    let present: Vec<String> = phext::phokenize(&doc)
        .into_iter()
        .filter(|ps| !ps.scroll.is_empty())
        .map(|ps| ps.scroll)
        .collect();
    assert_eq!(present, document_order(&doc));
}

#[test]
fn preorder_driver_walk_agrees_with_document_order() {
    let doc = build_tree();
    assert_eq!(traverse(&doc, Order::Pre), document_order(&doc));
    assert_eq!(traverse(&doc, Order::Pre), strings(&["F", "B", "A", "D", "C", "G", "I", "H"]));
}

#[test]
fn inorder_driver_walk_recovers_sorted_order() {
    let doc = build_tree();
    // The fixture is a BST, so inorder is the sorted sequence — obtained
    // purely by fetch()ing computed child coordinates, never by document order.
    assert_eq!(traverse(&doc, Order::In), strings(&["A", "B", "C", "D", "F", "G", "H", "I"]));
}

#[test]
fn postorder_driver_walk() {
    let doc = build_tree();
    assert_eq!(traverse(&doc, Order::Post), strings(&["A", "C", "D", "B", "H", "I", "G", "F"]));
}

#[test]
fn null_children_are_absent_scrolls_and_traversal_still_completes() {
    let doc = build_tree();

    // Every null child of the fixture fetches "" — G's left child, D's right
    // child, I's right child, and both children of each leaf.
    let null_paths: &[&[bool]] = &[
        &[R, L],       // G's left
        &[L, R, R],    // D's right
        &[R, R, R],    // I's right
        &[L, L, L],    // A's left
        &[L, L, R],    // A's right
        &[L, R, L, L], // C's left
        &[R, R, L, R], // H's right
    ];
    for path in null_paths {
        assert_eq!(phext::fetch(&doc, coord_for_path(path)), "", "null at {:?}", path);
    }

    // Exactly 8 non-empty scrolls exist in the whole document.
    let count = phext::explode(&doc).values().filter(|v| !v.is_empty()).count();
    assert_eq!(count, 8);

    // And all three traversals still visit exactly the 8 present nodes:
    // absence terminates a subtree walk instead of breaking it.
    for order in [Order::Pre, Order::In, Order::Post] {
        let mut got = traverse(&doc, order);
        got.sort();
        assert_eq!(got, strings(&["A", "B", "C", "D", "F", "G", "H", "I"]));
    }
}

#[test]
fn no_thread_bits_or_link_fields_are_stored() {
    let doc = build_tree();
    let forbidden = [
        "thread", "ltag", "rtag", "left", "right", "parent", "link", "pointer", "successor",
        "predecessor", "->",
    ];
    for (coord, scroll) in phext::explode(&doc) {
        if scroll.is_empty() {
            continue;
        }
        // a node scroll is its value and nothing else
        assert_eq!(scroll.len(), 1, "scroll at {} holds more than its value", coord.to_string());
        let lower = scroll.to_lowercase();
        for word in forbidden {
            assert!(!lower.contains(word), "scroll at {} stores a link field: {:?}", coord.to_string(), scroll);
        }
    }
}

#[test]
fn empty_and_single_node_trees_traverse() {
    // empty phext: the root itself is absent, every order is empty, no panic
    for order in [Order::Pre, Order::In, Order::Post] {
        assert!(traverse("", order).is_empty());
    }
    assert!(document_order("").is_empty());

    // a lone root with two null children: each order is just the root
    let doc = phext::insert(String::new(), coord_for_path(&[]), "F");
    for order in [Order::Pre, Order::In, Order::Post] {
        assert_eq!(traverse(&doc, order), strings(&["F"]));
    }
    assert_eq!(document_order(&doc), strings(&["F"]));
}

#[test]
fn subtree_traversal_from_a_non_root_coordinate() {
    let doc = build_tree();
    // Start the driver at B (depth 1): its subtree walks independently of
    // the rest of the tree, using the same coordinate arithmetic.
    let mut out = Vec::new();
    driver_walk(&doc, coord_for_path(&[L]), 1, Order::In, &mut out);
    assert_eq!(out, strings(&["A", "B", "C", "D"]));

    let mut out = Vec::new();
    driver_walk(&doc, coord_for_path(&[R]), 1, Order::Post, &mut out);
    assert_eq!(out, strings(&["H", "I", "G"]));
}
