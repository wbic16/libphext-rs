//------------------------------------------------------------------------------------------------------------
// toc_fixtures.rs — conformance for the progressive TOC modes over real corpus
// phexts: files this crate did not write.
//
// Set PHEXT_CORPUS_DIR to a directory of .phext files (e.g. ~/workspace/phext).
// Without it, the test skips — the invariants it checks are input-independent
// and are covered synthetically in src/test_toc.rs; this file exists so the
// modes are also proven against the living corpus, at corpus scale.
//------------------------------------------------------------------------------------------------------------
use libphext::phext::*;

fn components_of(coord: &Coordinate) -> [usize; 9] {
  [coord.z.library, coord.z.shelf, coord.z.series,
   coord.y.collection, coord.y.volume, coord.y.book,
   coord.x.chapter, coord.x.section, coord.x.scroll]
}

fn check_document(name: &str, doc: &str) {
  let index = spans(doc);
  let summary = toc_summary(doc, &[]);
  assert_eq!(summary.scrolls, index.len(), "{}: root summary != spans", name);
  let total_bytes: usize = index.iter().map(|s| s.range.len()).sum();
  assert_eq!(summary.bytes, total_bytes, "{}: root bytes != spans", name);

  // children partition the root, and the root's children sum to the full TOC
  let root = toc_children(doc, &[]);
  assert_eq!(root.iter().map(|c| c.scrolls).sum::<usize>(), index.len(),
             "{}: root children do not sum to the scroll count", name);
  assert_eq!(root.iter().map(|c| c.bytes).sum::<usize>(), total_bytes,
             "{}: root children bytes do not sum", name);

  // every span appears in toc_scrolls with the same size, and a random-walk
  // prefix drill: children of children partition their parent two levels down
  let listed = toc_scrolls(doc, &[]);
  assert_eq!(listed.len(), index.len(), "{}: toc_scrolls count", name);
  for span in &index {
    let found = listed.iter().find(|e| e.coord == span.coord)
      .unwrap_or_else(|| panic!("{}: {} missing from toc_scrolls", name, span.coord));
    assert_eq!(found.bytes, span.range.len(), "{}: size at {}", name, span.coord);
  }
  for child in &root {
    let sub = toc_children(doc, &[child.value]);
    let parent = toc_summary(doc, &[child.value]);
    assert_eq!(sub.iter().map(|c| c.scrolls).sum::<usize>(), parent.scrolls,
               "{}: level-2 partition under library {}", name, child.value);
    // the first grandchild's scrolls listing matches a manual prefix filter
    if let Some(grand) = sub.first() {
      let prefix = [child.value, grand.value];
      let manual = index.iter()
        .filter(|s| { let c = components_of(&s.coord); c[0] == prefix[0] && c[1] == prefix[1] })
        .count();
      assert_eq!(toc_scrolls(doc, &prefix).len(), manual,
                 "{}: scrolls under {:?}", name, prefix);
    }
  }
}

#[test]
fn toc_modes_conform_on_corpus_files() {
  let dir = match std::env::var("PHEXT_CORPUS_DIR") {
    Ok(d) => d,
    Err(_) => {
      eprintln!("PHEXT_CORPUS_DIR unset — corpus conformance skipped");
      return;
    }
  };
  let mut checked = 0;
  let mut paths: Vec<_> = std::fs::read_dir(&dir)
    .expect("corpus dir readable")
    .filter_map(|e| e.ok().map(|e| e.path()))
    .filter(|p| p.extension().map(|x| x == "phext").unwrap_or(false))
    .collect();
  paths.sort();
  for path in paths {
    let doc = std::fs::read_to_string(&path)
      .unwrap_or_else(|e| panic!("read {:?}: {}", path, e));
    check_document(&path.display().to_string(), &doc);
    checked += 1;
  }
  assert!(checked > 0, "no .phext files found in {}", dir);
  eprintln!("toc conformance: {} corpus files checked", checked);
}
