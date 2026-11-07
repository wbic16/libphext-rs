# TOAPDD — Knuth Concepts as Phext-Native Integration Tests

TOAPDD (The Art of Phext Driven Development) takes passages from Donald Knuth's
works, implements each concept in a phext-native way against libphext-rs, and
documents the enhancement that hindsight — and an addressable, sparse, text-native
store — offers over the original treatment.

Each test file in `tests/toapdd_*.rs` is a self-contained integration test using
only the public `libphext::phext` API. Tests only; no library code is changed by
this addition.

**Provenance:** the 20 tests were built and run one per ranch box (10 boxes,
2 waves) on 2026-11-07, task 34.7.10/9.8.14/7.7.11. Each was verified on its box
against libphext-rs v0.4.0. Per-attempt result documents (concept, exact cargo
counts, what the tests prove, API gaps) were collected alongside the sources.
Totals across the 20 files: **96 tests passed, 0 failed.**

## The 20 tests

| # | Passage (Knuth locator) | Test file | Box | Tests | Enhancement over Knuth, in one line |
|---|---|---|---|---|---|
| 1 | Literate Programming / WEB — *Literate Programming* (1992) Ch. 4, Ch. 7 | `tests/toapdd_web.rs` | scroll-central | 3 | Chunks are scrolls; tangle and weave are two traversals of the same scrolls — no preprocessor, per-chunk provenance |
| 2 | Multilinked structures / orthogonal lists — TAOCP Vol. 1 §2.4, §2.2.6 | `tests/toapdd_sparse.rs` | rock-mountain | 6 | One non-zero cell = one scroll; the coordinate carries row+column; no dense array, no special sparse encoding |
| 3 | Retrieval on secondary keys / inverted files — Vol. 3 §6.5 | `tests/toapdd_inverted.rs` | elven-path | 7 | The term is a scroll and postings append as records are written; a query is a coordinate lookup, not a batch index rebuild |
| 4 | Digital searching / tries — Vol. 3 §6.3 | `tests/toapdd_trie.rs` | halycon-vector | 7 | A coordinate is a materialised trie path; insert is leaf creation, no rebalancing of a packed node array |
| 5 | Traversing binary trees / threaded trees — Vol. 1 §2.3.1 | `tests/toapdd_traversal.rs` | logos-prime | 9 | Threading was a scarcity hack; with addresses abundant, a null is honestly just absence and traversal is a coordinate walk |
| 6 | Linked allocation — Vol. 1 §2.2.3 | `tests/toapdd_linked.rs` | free-fall | 4 | A node is a scroll at a coordinate; free is remove, and the prior version retains the node — reuse doesn't erase history |
| 7 | Dancing Links / Algorithm X — Vol. 4B §7.2.2.1 | `tests/toapdd_dlx.rs` | plain-view | 4 | Keep the sparse cover matrix as scrolls; replace the dance with versioned append-only choices — backtrack is fetching an earlier version |
| 8 | Backtrack programming — Vol. 4B §7.2.2 | `tests/toapdd_backtrack.rs` | ashfall-haven | 5 | Persist the search tree as a phext (895 states, N-queens n=6); prune by marking, resume from any stored partial state |
| 9 | Generating basic combinatorial patterns — Vol. 4A §7.2.1 | `tests/toapdd_generate.rs` | scroll-central | 5 | Knuth gives the visiting order; phext gives visited points persistent addresses — resume mid-space is "coordinates > checkpoint" |
| 10 | Multiway trees / B-trees — Vol. 3 §6.2.4 | `tests/toapdd_btree.rs` | rock-mountain | 6 | A node is a stable scroll that never moves on a split; the B-tree is demoted to an index *over* a phext, the store untouched |
| 11 | Dynamic storage allocation / buddy system — Vol. 1 §2.5 | `tests/toapdd_buddy.rs` | ni-knight | 3 | Keep the allocation ledger in phext — lineage is queryable from addressable scrolls — and let the allocator underneath be replaceable |
| 12 | Lists and garbage collection — Vol. 1 §2.3.5 | `tests/toapdd_gc.rs` | elven-path | 5 | Reachability can be read, not computed destructively; collection is an inspectable MARKS scroll and nothing is deleted |
| 13 | Boxes & glue / line breaking — *The TeXbook* Ch. 11, 12, 14 | `tests/toapdd_texbox.rs` | halycon-vector | 4 | The box tree and multiple candidate breakings live as scrolls; the chosen breaking is a single link scroll — switching touches exactly one coordinate |
| 14 | Balanced trees — Vol. 3 §6.2.3 | `tests/toapdd_balanced.rs` | logos-prime | 5 | Identity as coordinate means nothing to rebalance: 200 worst-case sorted inserts, zero rotations of the store |
| 15 | Optimum binary search trees / binary search — Vol. 3 §6.2.2, §6.2.1 | `tests/toapdd_obst.rs` | free-fall | 3 | Access counts are scrolls; placement advice is continuously revisable data, not a one-time O(n²) rebuild |
| 16 | Algorithm E (Euclid) — Vol. 1 §1.1 (deepened Vol. 2 §4.5.2–4.5.3) | `tests/toapdd_euclid.rs` | lighthouse-omen | 5 | Knuth's named, step-addressed algorithm made literal: steps, operands, and the worked trace are scrolls, and the trace replays the proof |
| 17 | Binary decision diagrams / ZDDs — Vol. 4A §7.1.4 | `tests/toapdd_bdd.rs` | plain-view | 4 | Reduction merges structurally identical nodes; phext merges scrolls that are the same record — 15 unshared nodes → 5 stored scrolls (3.0× sharing) by checksum dedup |
| 18 | Hashing — Vol. 3 §6.4 (warning / boundary test) | `tests/toapdd_hashing.rs` | ashfall-haven | 3 | Boundary proved: a one-word edit leaves identity (the coordinate) unchanged while the checksum flips 29/32 hex chars — a fingerprint is never an identity |
| 19 | External sorting — Vol. 3 §5.4 (warning / boundary test) | `tests/toapdd_extsort.rs` | ni-knight | 3 | Records are inserted at their final address; traversal is sorted on the first pass with zero merge passes |
| 20 | MIX / MMIX — Vol. 1 §1.3, §1.4 (warning / boundary test) | `tests/toapdd_mix.rs` | lighthouse-omen | 5 | Boundary proved: phext goes *around* the machine — a structureless scroll fetches whole as plain text, and execution changes only driver state (the program phext is byte-identical after) |

## Findings for libphext-rs (surfaced by these tests)

Zero blocking API gaps were found across the 20 attempts — the 0.4.0 public API
covered every spec, including the three boundary/refusal tests. Four behaviours
are worth documenting precisely, because tests had to learn them by reading the
implementation:

1. **`subtract` is a presence mask, not a diff.** It masks scrolls by whether
   their coordinate is present in the other phext, regardless of content. Used
   as its documentation describes — masking by all *other* original scrolls —
   it isolates an edited scroll; used as a content diff, it misleads.
2. **`index()` returns byte offsets, not coordinates.** It builds a phext whose
   scrolls hold the byte position of each scroll in the source; enumerating a
   phext's coordinates goes through `phokenize`, and `phokenize(index(p))` is
   the way to walk the index itself.
3. **`offset()` is the byte position where a scroll begins** — derived from
   `index()`; for successive scrolls it equals the start byte of that scroll,
   verifiable by slicing the source string at the returned offsets.
4. **Append-only history is caller discipline, not a library guarantee.**
   An append-only log is built by `insert()`-ing each event at a fresh
   coordinate; `replace()` overwrites a scroll in place, silently. Nothing in
   the API enforces append-only usage — the convention belongs to the caller
   and should be stated wherever a phext is treated as a log or ledger.
