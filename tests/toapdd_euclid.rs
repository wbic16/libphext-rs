// TOAPDD #16 — Knuth TAOCP Vol.1 §1.1, Algorithm E (Euclid's algorithm for gcd)
// Phext-native claim: the algorithm's steps, its operands, and its worked
// trace are all scrolls at their own coordinates in one phext. A driver
// (execution lives here, in the test, nowhere in the phext) reads operands
// from scrolls, runs the steps named by the step-scrolls, and appends each
// remainder step as a new trace scroll. The trace IS the phext: replaying
// it (phokenize the trace region in order, re-check every line) replays
// the proof that gcd(m, n) is what the driver claimed.
//
// Step texts are paraphrases written for this test, not Knuth's prose.

use libphext::phext;

// --- coordinates -----------------------------------------------------------
fn e1() -> phext::Coordinate { phext::to_coordinate("1.1.1/1.1.1/1.1.1") }
fn e2() -> phext::Coordinate { phext::to_coordinate("1.1.1/1.1.1/1.1.2") }
fn e3() -> phext::Coordinate { phext::to_coordinate("1.1.1/1.1.1/1.1.3") }
fn op_m() -> phext::Coordinate { phext::to_coordinate("1.1.1/1.1.1/2.1.1") }
fn op_n() -> phext::Coordinate { phext::to_coordinate("1.1.1/1.1.1/2.1.2") }
fn trace_coord(step: usize) -> phext::Coordinate {
    // chapter 3 = trace; section = step index (1-based); scroll 1
    phext::to_coordinate(&format!("1.1.1/1.1.1/3.{}.1", step))
}

const E1_TEXT: &str = "E1 [Find remainder]: Divide the larger current value by the smaller and keep the remainder r.";
const E2_TEXT: &str = "E2 [Is it zero?]: If r is zero, the smaller current value is the greatest common divisor; stop.";
const E3_TEXT: &str = "E3 [Reduce]: Replace the pair with (smaller, r) and return to E1.";

/// Build the initial phext: three step-scrolls + two operand scrolls, no trace.
fn build_algorithm_phext(m: u64, n: u64) -> String {
    let mut p = String::new();
    p = phext::insert(p, e1(), E1_TEXT);
    p = phext::insert(p, e2(), E2_TEXT);
    p = phext::insert(p, e3(), E3_TEXT);
    p = phext::insert(p, op_m(), &m.to_string());
    p = phext::insert(p, op_n(), &n.to_string());
    p
}

/// Driver: reads operands from scrolls, appends one trace scroll per
/// remainder step, returns (phext_with_trace, gcd).
fn run_euclid(phext_in: &str) -> (String, u64) {
    // The driver checks the step-scrolls are present (keyword asserts
    // below) and then executes the arithmetic compiled into it — it does
    // not interpret the step texts. The phext-facing claim is storage:
    // steps, operands, and the resulting trace are all fetchable scrolls,
    // and the trace replays as a proof (see replay()).
    assert!(phext::fetch(phext_in, e1()).contains("remainder"));
    assert!(phext::fetch(phext_in, e2()).contains("zero"));
    assert!(phext::fetch(phext_in, e3()).contains("return to E1"));

    let mut a: u64 = phext::fetch(phext_in, op_m()).trim().parse().expect("m parses");
    let mut b: u64 = phext::fetch(phext_in, op_n()).trim().parse().expect("n parses");
    if a < b { std::mem::swap(&mut a, &mut b); }

    let mut out = phext_in.to_string();
    let mut step = 0usize;
    loop {
        step += 1;
        let q = a / b;
        let r = a % b;
        let line = format!("{} = {} * {} + {}", a, q, b, r);
        out = phext::insert(out, trace_coord(step), &line);
        if r == 0 {
            return (out, b);
        }
        a = b;
        b = r;
    }
}

/// Replay: collect trace scrolls in coordinate order, parse each
/// "a = q * b + r" line, verify the arithmetic and the chaining, and
/// return the gcd the trace proves (the b of the final r == 0 line).
fn replay(phext_in: &str) -> u64 {
    let mut trace: Vec<(usize, String)> = phext::phokenize(phext_in)
        .into_iter()
        .filter(|ps| !ps.scroll.trim().is_empty())
        .filter(|ps| ps.coord.x.chapter == 3)
        .map(|ps| (ps.coord.x.section, ps.scroll.trim().to_string()))
        .collect();
    trace.sort_by_key(|(section, _)| *section);
    assert!(!trace.is_empty(), "trace must contain at least one step");

    let mut prev: Option<(u64, u64)> = None; // (b, r) of previous line
    let mut gcd: Option<u64> = None;
    for (idx, (section, line)) in trace.iter().enumerate() {
        assert_eq!(*section, idx + 1, "trace sections are consecutive from 1");
        // parse "a = q * b + r"
        let nums: Vec<u64> = line
            .split(|c: char| !(c.is_ascii_digit()))
            .filter(|s| !s.is_empty())
            .map(|s| s.parse().expect("trace number parses"))
            .collect();
        assert_eq!(nums.len(), 4, "line {:?} has a, q, b, r", line);
        let (a, q, b, r) = (nums[0], nums[1], nums[2], nums[3]);
        assert_eq!(a, q * b + r, "arithmetic checks out: {}", line);
        assert!(r < b, "remainder is smaller than divisor: {}", line);
        if let Some((pb, pr)) = prev {
            assert_eq!(a, pb, "chain: next a is previous b");
            assert_eq!(b, pr, "chain: next b is previous r");
        }
        if r == 0 {
            assert_eq!(idx, trace.len() - 1, "zero remainder only at the end");
            gcd = Some(b);
        }
        prev = Some((b, r));
    }
    gcd.expect("trace ends with a zero remainder, proving a gcd")
}

#[test]
fn steps_are_scrolls_at_their_own_coordinates() {
    let p = build_algorithm_phext(1071, 462);
    assert_eq!(phext::fetch(&p, e1()), E1_TEXT);
    assert_eq!(phext::fetch(&p, e2()), E2_TEXT);
    assert_eq!(phext::fetch(&p, e3()), E3_TEXT);
    // "see Algorithm E" resolves inside the same text: E3 names E1, and E1
    // is fetchable from the same phext.
    assert!(phext::fetch(&p, e3()).contains("E1"));
    // operands live in their own scrolls, distinct from the steps
    assert_eq!(phext::fetch(&p, op_m()), "1071");
    assert_eq!(phext::fetch(&p, op_n()), "462");
    // no trace yet
    assert_eq!(phext::fetch(&p, trace_coord(1)), "");
}

#[test]
fn driver_trace_is_the_phext_and_gcd_is_correct() {
    // Independently computed expectation:
    //   1071 = 2*462 + 147 ; 462 = 3*147 + 21 ; 147 = 7*21 + 0  => gcd 21
    let p = build_algorithm_phext(1071, 462);
    let (traced, gcd) = run_euclid(&p);
    assert_eq!(gcd, 21);
    assert_eq!(phext::fetch(&traced, trace_coord(1)), "1071 = 2 * 462 + 147");
    assert_eq!(phext::fetch(&traced, trace_coord(2)), "462 = 3 * 147 + 21");
    assert_eq!(phext::fetch(&traced, trace_coord(3)), "147 = 7 * 21 + 0");
    assert_eq!(phext::fetch(&traced, trace_coord(4)), "");
}

#[test]
fn replay_of_trace_reproves_the_gcd() {
    let p = build_algorithm_phext(1071, 462);
    let (traced, gcd) = run_euclid(&p);
    // fetch(trace) replays the proof — no driver state needed.
    assert_eq!(replay(&traced), gcd);
    assert_eq!(replay(&traced), 21);

    // a second pair, Knuth's classic style example shape (m=119, n=544):
    // 544 = 4*119 + 68 ; 119 = 1*68 + 51 ; 68 = 1*51 + 17 ; 51 = 3*17 + 0
    let p2 = build_algorithm_phext(119, 544);
    let (traced2, gcd2) = run_euclid(&p2);
    assert_eq!(gcd2, 17);
    assert_eq!(replay(&traced2), 17);
    assert_eq!(phext::fetch(&traced2, trace_coord(1)), "544 = 4 * 119 + 68");
    assert_eq!(phext::fetch(&traced2, trace_coord(4)), "51 = 3 * 17 + 0");
}

#[test]
fn algorithm_text_is_unchanged_after_execution() {
    let p = build_algorithm_phext(1071, 462);
    let before = (
        phext::fetch(&p, e1()),
        phext::fetch(&p, e2()),
        phext::fetch(&p, e3()),
        phext::fetch(&p, op_m()),
        phext::fetch(&p, op_n()),
    );
    let (traced, _) = run_euclid(&p);
    let after = (
        phext::fetch(&traced, e1()),
        phext::fetch(&traced, e2()),
        phext::fetch(&traced, e3()),
        phext::fetch(&traced, op_m()),
        phext::fetch(&traced, op_n()),
    );
    assert_eq!(before, after, "steps are data; execution only appends trace scrolls");
}

#[test]
fn a_correction_touches_exactly_one_step_scroll() {
    let p = build_algorithm_phext(1071, 462);
    let corrected = phext::replace(
        &p,
        e2(),
        "E2 [Is it zero?]: If r is zero, stop: the last non-zero remainder's divisor is the answer.",
    );
    assert!(phext::fetch(&corrected, e2()).contains("stop"));
    assert_eq!(phext::fetch(&corrected, e1()), E1_TEXT);
    assert_eq!(phext::fetch(&corrected, e3()), E3_TEXT);
    assert_eq!(phext::fetch(&corrected, op_m()), "1071");
    assert_eq!(phext::fetch(&corrected, op_n()), "462");
}
