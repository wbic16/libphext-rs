//------------------------------------------------------------------------------------------------------------
// file: test_toc_suffix.rs
// purpose: unit tests for toc_scrolls_by_suffix — the relationship primitive of
//          the progressive TOC modes (scrolls sharing a trailing suffix).
//------------------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
  use crate::phext::*;
  use std::collections::HashMap;

  // scrolls at 1.1.1/1.1.1/1.2.3, 2.3.4/5.6.7/1.2.3, 1.1.1/9.9.9/4.5.6, 3.3.3/3.3.3/1.2.3
  fn fixture() -> String {
    let mut map: HashMap<Coordinate, String> = Default::default();
    map.insert(to_coordinate("1.1.1/1.1.1/1.2.3"), "one".to_string());
    map.insert(to_coordinate("2.3.4/5.6.7/1.2.3"), "two-two".to_string());
    map.insert(to_coordinate("1.1.1/9.9.9/4.5.6"), "three".to_string());
    map.insert(to_coordinate("3.3.3/3.3.3/1.2.3"), "four".to_string());
    implode(map)
  }

  #[test]
  fn suffix_finds_relatives_across_parents() {
    let doc = fixture();
    let kin = toc_scrolls_by_suffix(&doc, &[1, 2, 3]);
    assert_eq!(kin.len(), 3);
    let coords: Vec<String> = kin.iter().map(|s| s.coord.to_string()).collect();
    assert!(coords.contains(&"2.3.4/5.6.7/1.2.3".to_string()));
    assert!(coords.contains(&"3.3.3/3.3.3/1.2.3".to_string()));
    // bytes travel with the listing
    let two = kin.iter().find(|s| s.coord.to_string() == "2.3.4/5.6.7/1.2.3").unwrap();
    assert_eq!(two.bytes, 7);
  }

  #[test]
  fn suffix_lengths_behave() {
    let doc = fixture();
    // trailing 1 component: scroll == 3
    assert_eq!(toc_scrolls_by_suffix(&doc, &[3]).len(), 3);
    // trailing 6: book 1 + chapter.section.scroll 1.2.3 -> two of them (books 1 and 7? no: y.book components are 1 and 7 and 9 and 3)
    let six = toc_scrolls_by_suffix(&doc, &[1, 1, 1, 1, 2, 3]);
    assert_eq!(six.len(), 1);
    assert_eq!(six[0].coord.to_string(), "1.1.1/1.1.1/1.2.3");
    // full coordinate as suffix: at most one
    assert_eq!(toc_scrolls_by_suffix(&doc, &[2, 3, 4, 5, 6, 7, 1, 2, 3]).len(), 1);
    assert_eq!(toc_scrolls_by_suffix(&doc, &[9, 9, 9, 9, 9, 9, 9, 9, 9]).len(), 0);
    // empty suffix = everything; overlong suffix = nothing
    assert_eq!(toc_scrolls_by_suffix(&doc, &[]).len(), 4);
    assert!(toc_scrolls_by_suffix(&doc, &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).is_empty());
    // no match
    assert!(toc_scrolls_by_suffix(&doc, &[7, 7, 7]).is_empty());
  }
}
