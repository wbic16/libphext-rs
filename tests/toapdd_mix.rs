// TOAPDD #20 — Knuth TAOCP Vol.1 §1.3 MIX/MMIX boundary — WARNING / boundary test
// Claim to prove is a REFUSAL: phext does not go inside the machine.
//  (a) A phext with no break bytes at all is plain text: it fetches whole at
//      1.1.1/1.1.1/1.1.1 and phokenizes to exactly one scroll. Plain text is
//      the degenerate phext — no structure is imposed on it.
//  (b) A MIX-style program stored as scrolls is data AROUND the machine.
//      The fetch/execute loop, registers and memory live only in the driver
//      (this test code). Executing changes only driver state: the program
//      phext is byte-identical afterwards (checksum before == after), and
//      the result lives in the driver — not in any scroll — until the driver
//      deliberately writes an output scroll. Phext is the record around the
//      machine; execution is outside it.
//
// Instruction lines below are the test author's own toy lines ("LDA 100"
// style), not Knuth's prose.

use libphext::phext;
use std::collections::HashMap;

// --- coordinates -----------------------------------------------------------
fn root() -> phext::Coordinate { phext::to_coordinate("1.1.1/1.1.1/1.1.1") }
fn instr_coord(step: usize) -> phext::Coordinate {
    // chapter 1, section 1, scroll = step (1-based): one instruction per scroll
    phext::to_coordinate(&format!("1.1.1/1.1.1/1.1.{}", step))
}
fn data_coord(cell: usize) -> phext::Coordinate {
    // chapter 2, section 1, scroll = cell slot (1-based): initial data values
    phext::to_coordinate(&format!("1.1.1/1.1.1/2.1.{}", cell))
}
fn output_coord() -> phext::Coordinate { phext::to_coordinate("1.1.1/1.1.1/3.1.1") }

// Toy program: acc = mem[100] + mem[101]; mem[102] = acc; halt.
// Data cells: slot 1 <-> address 100, slot 2 <-> address 101, slot 3 <-> 102.
const PROGRAM: [&str; 4] = ["LDA 100", "ADD 101", "STA 102", "HLT"];
const DATA_ADDRS: [usize; 3] = [100, 101, 102];
const DATA_VALUES: [&str; 3] = ["40", "2", "0"];

fn build_program_phext() -> String {
    let mut p = String::new();
    for (i, line) in PROGRAM.iter().enumerate() {
        p = phext::insert(p, instr_coord(i + 1), line);
    }
    for (i, value) in DATA_VALUES.iter().enumerate() {
        p = phext::insert(p, data_coord(i + 1), value);
    }
    p
}

// --- the machine: driver-side only -----------------------------------------
// Registers, memory and the fetch/execute loop exist ONLY here, in the test
// driver. Nothing in the phext executes, and nothing here is stored back
// into the phext unless a test explicitly inserts an output scroll.
struct Machine {
    acc: i64,
    memory: HashMap<usize, i64>,
    halted: bool,
    steps_executed: usize,
}

fn run(program_phext: &str) -> Machine {
    // Seed driver memory from the data scrolls (a fetch: phext -> driver).
    let mut memory: HashMap<usize, i64> = HashMap::new();
    for (i, addr) in DATA_ADDRS.iter().enumerate() {
        let v: i64 = phext::fetch(program_phext, data_coord(i + 1))
            .trim().parse().expect("data scroll parses");
        memory.insert(*addr, v);
    }
    let mut machine = Machine { acc: 0, memory, halted: false, steps_executed: 0 };

    // Fetch/execute loop: fetch is a phext read; execute is driver code.
    let mut pc = 1usize;
    loop {
        assert!(pc <= 100, "runaway program: no HLT");
        let line = phext::fetch(program_phext, instr_coord(pc));
        assert!(!line.is_empty(), "fetched an instruction at step {}", pc);
        machine.steps_executed += 1;
        let mut parts = line.split_whitespace();
        let op = parts.next().expect("opcode present");
        let operand: Option<usize> = parts.next().map(|s| s.parse().expect("operand parses"));
        match op {
            "LDA" => machine.acc = machine.memory[&operand.expect("LDA operand")],
            "ADD" => machine.acc += machine.memory[&operand.expect("ADD operand")],
            "STA" => { machine.memory.insert(operand.expect("STA operand"), machine.acc); }
            "HLT" => { machine.halted = true; break; }
            other => panic!("unknown opcode {}", other),
        }
        pc += 1;
    }
    machine
}

// --- (a) plain text is the degenerate phext --------------------------------

#[test]
fn plain_text_with_no_breaks_fetches_whole_at_root() {
    // A single line, deliberately containing NONE of the phext dimension
    // break bytes (and no line break either): no structure at all.
    let plain = "A scroll with no structure is plain text and phext leaves it whole";
    let break_bytes: [u8; 9] = [0x01, 0x17, 0x18, 0x19, 0x1A, 0x1C, 0x1D, 0x1E, 0x1F];
    for b in plain.bytes() {
        assert!(!break_bytes.contains(&b), "fixture must contain no break bytes");
        assert_ne!(b, b'\n', "fixture must contain no line break");
    }

    // It fetches whole at the root coordinate.
    assert_eq!(phext::fetch(plain, root()), plain);
    // Exactly one scroll, at the root, holding the whole string.
    let scrolls = phext::phokenize(plain);
    assert_eq!(scrolls.len(), 1, "no breaks => exactly one scroll");
    assert_eq!(scrolls[0].coord, root());
    assert_eq!(scrolls[0].scroll, plain);
    // Nothing lives anywhere else.
    assert_eq!(phext::fetch(plain, instr_coord(2)), "");
    assert_eq!(phext::fetch(plain, output_coord()), "");
}

// --- (b) the program is data around the machine ----------------------------

#[test]
fn program_is_scrolls_one_instruction_per_scroll_plus_data() {
    let p = build_program_phext();
    for (i, line) in PROGRAM.iter().enumerate() {
        assert_eq!(phext::fetch(&p, instr_coord(i + 1)), *line);
    }
    assert_eq!(phext::fetch(&p, instr_coord(PROGRAM.len() + 1)), "");
    for (i, value) in DATA_VALUES.iter().enumerate() {
        assert_eq!(phext::fetch(&p, data_coord(i + 1)), *value);
    }
    // No output exists before anything runs.
    assert_eq!(phext::fetch(&p, output_coord()), "");
}

#[test]
fn execution_changes_only_driver_state_phext_is_byte_identical() {
    let p = build_program_phext();
    let checksum_before = phext::checksum(&p);
    let bytes_before = p.clone();

    let machine = run(&p);

    // The driver computed: 40 + 2 = 42, stored into its OWN memory cell 102.
    assert!(machine.halted);
    assert_eq!(machine.steps_executed, 4);
    assert_eq!(machine.acc, 42);
    assert_eq!(machine.memory[&102], 42);
    assert_eq!(machine.memory[&100], 40);
    assert_eq!(machine.memory[&101], 2);

    // Execution touched nothing in the phext: byte-identical, same checksum.
    assert_eq!(p, bytes_before);
    assert_eq!(phext::checksum(&p), checksum_before);

    // In particular STA did NOT write back: the phext's data scroll for
    // address 102 still holds its initial "0", and no result scroll exists.
    // The result value lives in the driver, not in any scroll.
    assert_eq!(phext::fetch(&p, data_coord(3)), "0");
    assert_eq!(phext::fetch(&p, output_coord()), "");
    for (i, line) in PROGRAM.iter().enumerate() {
        assert_eq!(phext::fetch(&p, instr_coord(i + 1)), *line);
    }
}

#[test]
fn rerunning_from_the_same_phext_gives_the_same_result() {
    // No hidden state in the phext: it is an immutable record the driver
    // reads. Two runs from the same string agree, and the string is
    // unchanged between and after them — separation, not shared state.
    let p = build_program_phext();
    let first = run(&p);
    let second = run(&p);
    assert_eq!(first.acc, second.acc);
    assert_eq!(first.memory[&102], second.memory[&102]);
    assert_eq!(p, build_program_phext());
}

#[test]
fn deliberate_output_is_a_new_scroll_written_by_the_driver() {
    let p = build_program_phext();
    let machine = run(&p);
    assert_eq!(phext::fetch(&p, output_coord()), "", "no output before the driver writes one");

    // The ONLY way a result enters the phext: the driver explicitly
    // inserts a new output scroll. That is a deliberate act of recording,
    // not a side effect of execution.
    let recorded = phext::insert(p.clone(), output_coord(), &machine.memory[&102].to_string());
    assert_eq!(phext::fetch(&recorded, output_coord()), "42");

    // The recorded version's program and data scrolls are unchanged, and
    // the original program phext still has no output scroll at all.
    for (i, line) in PROGRAM.iter().enumerate() {
        assert_eq!(phext::fetch(&recorded, instr_coord(i + 1)), *line);
    }
    for (i, value) in DATA_VALUES.iter().enumerate() {
        assert_eq!(phext::fetch(&recorded, data_coord(i + 1)), *value);
    }
    assert_eq!(phext::fetch(&p, output_coord()), "");
    assert_ne!(phext::checksum(&recorded), phext::checksum(&p),
        "the deliberate write is a new version; execution alone changed nothing");
}
