//------------------------------------------------------------------------------------------------------------
// file: test_toc.rs
// purpose: unit tests for the progressive TOC modes (toc_children / toc_summary /
//          toc_scrolls). The partition invariants asserted here — children sum to
//          their parent summary at every level, and toc_scrolls agrees with
//          explode — are input-independent: they must hold for any phext.
//          tests/toc_fixtures.rs runs the same invariants over real corpus files.
//------------------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
  use crate::phext::*;
  use std::collections::HashMap;

  // a document with structure in several arms:
  //   1.1.1/1.1.1/1.1.1 "alpha"        1.1.1/1.1.1/1.1.2 "beta"
  //   1.1.1/1.1.1/1.2.1 "gamma"        1.1.2/1.1.1/1.1.1 "delta"
  //   1.2.1/3.1.1/1.1.1 "epsilon-long" 2.1.1/1.1.1/9.9.9 "zeta"
  fn fixture() -> String {
    let mut map: HashMap<Coordinate, String> = Default::default();
    map.insert(to_coordinate("1.1.1/1.1.1/1.1.1"), "alpha".to_string());
    map.insert(to_coordinate("1.1.1/1.1.1/1.1.2"), "beta".to_string());
    map.insert(to_coordinate("1.1.1/1.1.1/1.2.1"), "gamma".to_string());
    map.insert(to_coordinate("1.1.2/1.1.1/1.1.1"), "delta".to_string());
    map.insert(to_coordinate("1.2.1/3.1.1/1.1.1"), "epsilon-long".to_string());
    map.insert(to_coordinate("2.1.1/1.1.1/9.9.9"), "zeta".to_string());
    implode(map)
  }

  fn components_of(coord: &Coordinate) -> [usize; 9] {
    [coord.z.library, coord.z.shelf, coord.z.series,
     coord.y.collection, coord.y.volume, coord.y.book,
     coord.x.chapter, coord.x.section, coord.x.scroll]
  }

  #[test]
  fn children_at_root_list_libraries() {
    let doc = fixture();
    let root = toc_children(&doc, &[]);
    assert_eq!(root.len(), 2);
    assert_eq!(root[0].value, 1);
    assert_eq!(root[0].scrolls, 5);
    assert_eq!(root[0].bytes, 5 + 4 + 5 + 5 + 12);
    assert_eq!(root[1].value, 2);
    assert_eq!(root[1].scrolls, 1);
    assert_eq!(root[1].bytes, 4);
  }

  #[test]
  fn children_walk_down_one_level_at_a_time() {
    let doc = fixture();
    // library 1 -> shelves {1, 2}
    let shelves = toc_children(&doc, &[1]);
    assert_eq!(shelves.iter().map(|c| c.value).collect::<Vec<_>>(), vec![1, 2]);
    // 1.2 -> series {1}
    let series = toc_children(&doc, &[1, 2]);
    assert_eq!(series.len(), 1);
    assert_eq!(series[0].scrolls, 1);
    // 1.1.1/1.1.1 -> chapters {1}; then sections {1, 2}; then scrolls {1, 2}
    let sections = toc_children(&doc, &[1, 1, 1, 1, 1, 1, 1]);
    assert_eq!(sections.iter().map(|c| c.value).collect::<Vec<_>>(), vec![1, 2]);
    let scrolls = toc_children(&doc, &[1, 1, 1, 1, 1, 1, 1, 1]);
    assert_eq!(scrolls.iter().map(|c| c.value).collect::<Vec<_>>(), vec![1, 2]);
    assert_eq!(scrolls[0].bytes, 5); // "alpha"
    assert_eq!(scrolls[1].bytes, 4); // "beta"
  }

  #[test]
  fn children_of_full_or_missing_prefix_is_empty() {
    let doc = fixture();
    assert!(toc_children(&doc, &[1, 1, 1, 1, 1, 1, 1, 1, 1]).is_empty());
    assert!(toc_children(&doc, &[7]).is_empty());
    assert!(toc_children(&doc, &[1, 9, 9]).is_empty());
  }

  #[test]
  fn summary_counts_the_subtree() {
    let doc = fixture();
    let whole = toc_summary(&doc, &[]);
    assert_eq!(whole.scrolls, 6);
    assert_eq!(whole.bytes, 35);
    assert_eq!(toc_summary(&doc, &[2]).scrolls, 1);
    let single = toc_summary(&doc, &[1, 2, 1, 3, 1, 1, 1, 1, 1]);
    assert_eq!(single.scrolls, 1);
    assert_eq!(single.bytes, 12);
    assert_eq!(toc_summary(&doc, &[4, 4, 4]).scrolls, 0);
  }

  #[test]
  fn scrolls_lists_planets_with_sizes() {
    let doc = fixture();
    let all = toc_scrolls(&doc, &[]);
    assert_eq!(all.len(), 6);
    let system = toc_scrolls(&doc, &[1, 1, 1]);
    assert_eq!(system.len(), 3);
    let coords: Vec<String> = system.iter().map(|s| s.coord.to_string()).collect();
    assert!(coords.contains(&"1.1.1/1.1.1/1.2.1".to_string()));
    // bytes agree with explode, coordinate by coordinate
    let map = explode(&doc);
    for entry in &all {
      assert_eq!(entry.bytes, map[&entry.coord].len());
    }
    let one = toc_scrolls(&doc, &[2, 1, 1, 1, 1, 1, 9, 9, 9]);
    assert_eq!(one.len(), 1);
    assert_eq!(one[0].bytes, 4);
    assert!(toc_scrolls(&doc, &[8]).is_empty());
  }

  // The partition invariant, recursively over every populated prefix of the
  // fixture: children counts sum to the parent summary, and the leaves of the
  // recursion are exactly the toc_scrolls listing.
  #[test]
  fn children_partition_every_prefix() {
    let doc = fixture();
    fn walk(doc: &str, prefix: &mut Vec<usize>, leaves: &mut usize) {
      let summary = toc_summary(doc, prefix);
      if prefix.len() == 9 {
        assert!(summary.scrolls <= 1);
        *leaves += summary.scrolls;
        return;
      }
      let children = toc_children(doc, prefix);
      let counted: usize = children.iter().map(|c| c.scrolls).sum();
      let bytes: usize = children.iter().map(|c| c.bytes).sum();
      assert_eq!(counted, summary.scrolls, "count mismatch at {:?}", prefix);
      assert_eq!(bytes, summary.bytes, "bytes mismatch at {:?}", prefix);
      for child in children {
        prefix.push(child.value);
        walk(doc, prefix, leaves);
        prefix.pop();
      }
    }
    let mut leaves = 0;
    walk(&doc, &mut Vec::new(), &mut leaves);
    assert_eq!(leaves, toc_scrolls(&doc, &[]).len());
    assert_eq!(leaves, explode(&doc).len());
  }

  #[test]
  fn empty_and_delimiter_only_documents_have_no_structure() {
    assert!(toc_children("", &[]).is_empty());
    assert_eq!(toc_summary("", &[]).scrolls, 0);
    assert!(toc_scrolls("", &[]).is_empty());
    let breaks = "\u{17}\u{18}\u{19}";
    assert!(toc_children(breaks, &[]).is_empty());
    assert_eq!(toc_summary(breaks, &[]).scrolls, 0);
  }

  #[test]
  fn modes_agree_with_spans_and_explode() {
    let doc = fixture();
    assert_eq!(toc_summary(&doc, &[]).scrolls, spans(&doc).len());
    for entry in toc_scrolls(&doc, &[]) {
      let components = components_of(&entry.coord);
      // every listed scroll is reachable by walking children from the root
      let mut prefix: Vec<usize> = Vec::new();
      for depth in 0..9 {
        let children = toc_children(&doc, &prefix);
        assert!(children.iter().any(|c| c.value == components[depth]),
                "dimension {} value {} missing at {:?}", depth, components[depth], prefix);
        prefix.push(components[depth]);
      }
    }
  }
}
