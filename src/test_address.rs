#[cfg(test)]
mod tests {
  use crate::phext::*;

  fn home() -> Coordinate { to_coordinate("1.1.1/1.1.1/1.1.1") }
  fn scroll2() -> Coordinate { to_coordinate("1.1.1/1.1.1/1.1.2") }

  fn member_a() -> String {
    let mut member = insert(String::new(), home(), "alpha bravo\ncharlie delta\necho");
    member = insert(member, scroll2(), "second scroll\nline two");
    member
  }

  fn member_b() -> String {
    insert(String::new(), home(), "uni2 home\nsecond line")
  }

  fn chain() -> Chain {
    Chain { members: vec![member_a(), member_b()], closed: false }
  }

  #[test]
  fn point_parse_canonical() {
    let p = parse_point_address("$3:1.1.1/1.1.1/1.1.1@24,5").unwrap();
    assert_eq!(p.universe, 3);
    assert_eq!(p.coordinate, home());
    assert_eq!(p.position, Some(ScrollPosition { line: 24, column: 5 }));
    assert_eq!(p.offset, None);
    assert_eq!(p.to_string(), "$3:1.1.1/1.1.1/1.1.1@24,5");
    // $1: is accepted on input, never emitted
    let p1 = parse_point_address("$1:1.1.1/1.1.1/1.1.1@24,5").unwrap();
    assert_eq!(p1.to_string(), "1.1.1/1.1.1/1.1.1@24,5");
    // leading zeros parse by value; canonical form is minimal decimal
    let pz = parse_point_address("1.1.1/1.1.1/1.1.1@024,005").unwrap();
    assert_eq!(pz.to_string(), "1.1.1/1.1.1/1.1.1@24,5");
    // bare coordinate: no position
    let bare = parse_point_address("1.1.1/1.1.1/1.1.2").unwrap();
    assert_eq!(bare.position, None);
    assert_eq!(bare.to_string(), "1.1.1/1.1.1/1.1.2");
    // offset form
    let po = parse_point_address("1.1.1/1.1.1/1.1.1@24,5+10").unwrap();
    assert_eq!(po.offset, Some(10));
    assert_eq!(po.to_string(), "1.1.1/1.1.1/1.1.1@24,5+10");
  }

  #[test]
  fn point_parse_rejects() {
    for bad in [
      "$0:1.1.1/1.1.1/1.1.1@1,1",   // universe 0
      "1.1.1/1.1.1/1.1.1@0,5",      // line 0
      "1.1.1/1.1.1/1.1.1@24,0",     // column 0
      "1.1.1/1.1.1/0.1.1@1,1",      // coordinate component 0
      "1.1.1/1.1.1@1,1",            // only two arms
      "1.1.1/1.1.1/1.1.1@24",       // no column
      "1.1.1/1.1.1/1.1.1@24,5+",    // empty offset
      "1.1.1/1.1.1/1.1.1@24,+5",    // empty column
      "1.1.1/1.1.1/1.1.1@24,5@1,1", // second @
      "1.1.1/1.1.1/1.1.1@ 24,5",    // whitespace
      "",                            // empty
    ] {
      assert!(parse_point_address(bad).is_none(), "expected rejection: {bad:?}");
    }
  }

  #[test]
  fn point_resolve() {
    let c = chain();
    let p = parse_point_address("1.1.1/1.1.1/1.1.1@2,3").unwrap();
    // line 1 "alpha bravo" = 11 chars + newline = 12; column 3 -> +2
    assert_eq!(c.resolve_point(&p), Some((1, home(), 14)));
    // last valid boundary of line 1
    let end = parse_point_address("1.1.1/1.1.1/1.1.1@1,12").unwrap();
    assert_eq!(c.resolve_point(&end), Some((1, home(), 11)));
    // past the line, past the scroll, missing scroll, missing universe
    assert_eq!(c.resolve_point(&parse_point_address("1.1.1/1.1.1/1.1.1@1,13").unwrap()), None);
    assert_eq!(c.resolve_point(&parse_point_address("1.1.1/1.1.1/1.1.1@4,1").unwrap()), None);
    assert_eq!(c.resolve_point(&parse_point_address("1.1.1/1.1.1/1.1.9@1,1").unwrap()), None);
    assert_eq!(c.resolve_point(&parse_point_address("$3:1.1.1/1.1.1/1.1.1@1,1").unwrap()), None);
    // bare point resolves to offset 0; universe 2 resolves in its own member
    assert_eq!(c.resolve_point(&parse_point_address("1.1.1/1.1.1/1.1.2").unwrap()), Some((1, scroll2(), 0)));
    assert_eq!(c.resolve_point(&parse_point_address("$2:1.1.1/1.1.1/1.1.1@2,1").unwrap()), Some((2, home(), 10)));
  }

  #[test]
  fn point_fetch_offset() {
    let c = chain();
    assert_eq!(c.fetch_point("1.1.1/1.1.1/1.1.1@1,1+5"), "alpha");
    assert_eq!(c.fetch_point("1.1.1/1.1.1/1.1.1@2,1+7"), "charlie");
    // across a line boundary: offset 9 is 'v' of "bravo"
    assert_eq!(c.fetch_point("1.1.1/1.1.1/1.1.1@1,10+5"), "vo\nch");
    // clipped at the scroll boundary, never into the next scroll
    assert_eq!(c.fetch_point("1.1.1/1.1.1/1.1.1@3,1+100"), "echo");
    assert_eq!(c.fetch_point("1.1.1/1.1.1/1.1.1@1,1+0"), "");
    // a bare point is a caret: fetches nothing; failures fetch nothing
    assert_eq!(c.fetch_point("1.1.1/1.1.1/1.1.1@1,1"), "");
    assert_eq!(c.fetch_point("1.1.1/1.1.1/1.1.1@9,1+3"), "");
    assert_eq!(c.fetch_point("$2:1.1.1/1.1.1/1.1.1@1,1+4"), "uni2");
  }

  #[test]
  fn point_fetch_offset_unicode() {
    let member = insert(String::new(), home(), "héllo wörld");
    let c = Chain { members: vec![member], closed: false };
    // columns and offsets count scalars: é is one character (two bytes)
    assert_eq!(c.resolve_point(&parse_point_address("1.1.1/1.1.1/1.1.1@1,3").unwrap()), Some((1, home(), 2)));
    assert_eq!(c.fetch_point("1.1.1/1.1.1/1.1.1@1,2+3"), "éll");
  }

  #[test]
  fn range_parse_canonical() {
    let r = parse_range_address("1.1.1/1.1.1/1.1.1@24,5-1.1.1/1.1.1/1.1.1@24,9").unwrap();
    assert_eq!(r.to_string(), "1.1.1/1.1.1/1.1.1@24,5-1.1.1/1.1.1/1.1.1@24,9");
    // reversed input normalizes: direction is discarded
    let rev = parse_range_address("1.1.1/1.1.1/1.1.1@24,9-1.1.1/1.1.1/1.1.1@24,5").unwrap();
    assert_eq!(rev, r);
    // $1: on input is dropped in canonical form, even beside a $3: endpoint
    let cross = parse_range_address("$1:1.1.1/1.1.1/1.1.1@24,5-$3:1.1.1/1.1.1/1.1.1@31,12").unwrap();
    assert_eq!(cross.to_string(), "1.1.1/1.1.1/1.1.1@24,5-$3:1.1.1/1.1.1/1.1.1@31,12");
    // offset forms are not range endpoints
    assert!(parse_range_address("1.1.1/1.1.1/1.1.1@1,1+3-1.1.1/1.1.1/1.1.1@1,5").is_none());
    // one hyphen only
    assert!(parse_range_address("1.1.1/1.1.1/1.1.1-1.1.1/1.1.1/1.1.2-1.1.1/1.1.1/1.1.3").is_none());
  }

  #[test]
  fn range_fetch_within_scroll() {
    let c = chain();
    // characters 1-5 of line 1 (boundaries 1..6, half-open)
    assert_eq!(c.fetch_range("1.1.1/1.1.1/1.1.1@1,1-1.1.1/1.1.1/1.1.1@1,6"), "alpha");
    // the whole of line 2, newline included
    assert_eq!(c.fetch_range("1.1.1/1.1.1/1.1.1@2,1-1.1.1/1.1.1/1.1.1@3,1"), "charlie delta\n");
    // empty range is a caret
    assert_eq!(c.fetch_range("1.1.1/1.1.1/1.1.1@1,3-1.1.1/1.1.1/1.1.1@1,3"), "");
    // reversed fetches the same extent
    assert_eq!(c.fetch_range("1.1.1/1.1.1/1.1.1@1,6-1.1.1/1.1.1/1.1.1@1,1"), "alpha");
  }

  #[test]
  fn range_fetch_cross_scroll() {
    let c = chain();
    let frag = c.fetch_range("1.1.1/1.1.1/1.1.1@3,2-1.1.1/1.1.1/1.1.2@1,7");
    assert_eq!(frag, "cho\u{17}second");
    // the fragment round-trips as a phext: its scrolls fetch back the segments
    let parsed = parse_chains(&frag);
    assert_eq!(parsed[0].fetch("1.1.1/1.1.1/1.1.1"), "cho");
    assert_eq!(parsed[0].fetch("1.1.1/1.1.1/1.1.2"), "second");
    // whole scrolls, no @ on either endpoint: the member itself
    assert_eq!(c.fetch_range("1.1.1/1.1.1/1.1.1-1.1.1/1.1.1/1.1.2"), member_a());
  }

  #[test]
  fn range_fetch_multiverse() {
    let c = chain();
    let frag = c.fetch_range("1.1.1/1.1.1/1.1.1@3,2-$2:1.1.1/1.1.1/1.1.1@1,5");
    // universe 1's scroll 2 lies strictly between the endpoints: a whole segment
    assert_eq!(frag, "cho\u{17}second scroll\nline two\u{2}uni2");
    let parsed = parse_chains(&frag);
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].members.len(), 2);
    assert_eq!(parsed[0].fetch("$2:1.1.1/1.1.1/1.1.1"), "uni2");
    // resolve_range exposes the segments
    let range = parse_range_address("1.1.1/1.1.1/1.1.1@3,2-$2:1.1.1/1.1.1/1.1.1@1,5").unwrap();
    let segments = c.resolve_range(&range).unwrap();
    assert_eq!(segments, vec![
      (1, home(), "cho".to_string()),
      (1, scroll2(), "second scroll\nline two".to_string()),
      (2, home(), "uni2".to_string()),
    ]);
  }

  #[test]
  fn range_resolution_errors() {
    let c = chain();
    // universe past the end of the chain
    assert_eq!(c.fetch_range("1.1.1/1.1.1/1.1.1@1,1-$3:1.1.1/1.1.1/1.1.1@1,1"), "");
    assert!(c.resolve_range(&parse_range_address("1.1.1/1.1.1/1.1.1@1,1-$3:1.1.1/1.1.1/1.1.1@1,1").unwrap()).is_none());
    // missing endpoint scroll
    assert!(c.resolve_range(&parse_range_address("1.1.1/1.1.1/1.1.1@1,1-1.1.1/1.1.1/1.1.9@1,1").unwrap()).is_none());
    // line past the scroll's end: an error, never a clamp
    assert!(c.resolve_range(&parse_range_address("1.1.1/1.1.1/1.1.1@1,1-1.1.1/1.1.1/1.1.1@9,1").unwrap()).is_none());
    // column past the line's last boundary
    assert!(c.resolve_range(&parse_range_address("1.1.1/1.1.1/1.1.1@1,1-1.1.1/1.1.1/1.1.1@1,13").unwrap()).is_none());
  }
}
