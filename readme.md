# libphext

This Rust project provides the standard Phext implementation (11-dimensional plain hypertext). For more information about the phext format, head over to https://phext.io.

## Elevator Pitch

Phext is hierarchical digital memory. It enables seamless knowledge transfer between humans and computers. Let's learn how to think at planet-scale. :)

## Zero Dependencies*

Phext is just 11-dimensional text. As such, you only need phext.rs and the standard libraries to work with it.* This tiny dependency gives you hierarchical superpowers. Use them wisely!

Note: We depend upon `xxh3` for checksum content hashes.

## Phext Motivation

In the 1980s, computers could write 25 KB/sec to a floppy disk. In the 2020s, it became possible to write 2 GB/sec to an SSD. This changed the definition of a "small" file. Unfortunately, most of our file abstractions (especially on Windows) have not scaled to take advantage of these performance gains. For the most part, this isn't much of a problem: humans are still rate-limited at 300 bps using keyboards. At some point in the next 25 years, however, we will have high-bandwidth brain interconnects - at which point we will need a high-bandwidth multi-dimensional text format: phext!

The introduction of Large Language Models (LLMs) has accelerated our transition to this future. You can use phext to interact with agents and groups of humans at scale - think of visualizing 9 billion computer screens at once. Phext is like being given a coordinate system of coordinate systems, allowing you to walk the latent space of any problem space efficiently.

## Phext Coordinate Formats

* Canonical Format: Orders coordinates to avoid the need for labels
  * example: z3.z2.z1/y3.y2.y1/x3.x2.x1
  * z3 - Library (LB)
  * z2 - Shelf (SF)
  * z1 - Series (SR)
  * y3 - Collection (CN)
  * y2 - Volume (VM)
  * y1 - Book (BK)
  * x3 - Chapter (CH)
  * x2 - Section (SN)
  * x1 - Scroll (SC)
* URL Format: the same as the canonical format, but with semi-colons instead of slashes
  * this allows us to use coordinates in routes
  * example: z3.z2.z1;y3.y2.y1;x3.x2.x1

> **Dense graphs:** phext is sparse by design. Coordinate components are unbounded, but a fully-populated dense graph contains the product of its dimensions' extents in scrolls — a dense block spanning 1000 in each of 9 dimensions implies 10^27 scrolls. Materializing, exploding, or indexing such a graph costs memory proportional to its populated scroll count. The old 999 cap never protected against this (999^9 was already unreachable); sparsity does. Keep graphs sparse.

## Binary Scrolls (feature: `binary`)

Phext scrolls are text (valid UTF-8). Arbitrary binary data rides as text via base64, in one canonical format that every phext implementation (Rust, Python, Node, C#, C, ...) must match byte-for-byte:

```text
phext-binary:1:base64
<base64 body>
```

* Line 1 is exactly `phext-binary:1:base64`, followed by a single line break (0x0A).
* The body is RFC 4648 base64: standard alphabet (`A-Z`, `a-z`, `0-9`, `+`, `/`), padding (`=`) required, no line wrapping, no whitespace.
* An empty payload is the header line alone.
* A scroll whose content begins with the header line is a binary scroll by definition — the format claims that namespace. A prefix-matching scroll whose body is not strict base64 is a *malformed* binary scroll; `fetch_binary` returns its bytes verbatim rather than failing.

API: `binary::insert_binary` / `replace_binary` store a payload as a binary scroll unconditionally; `binary::fetch_binary` returns the bytes at a coordinate (the decoded payload for a binary scroll, the scroll's own UTF-8 bytes otherwise, empty for an unoccupied coordinate). `encode_scroll` / `decode_scroll` / `is_binary_scroll` expose the format itself.

### The Ingestion Rule (canonical behaviour)

> When non-UTF-8 data is observed in an incoming scroll update, we encode it with base64 verbatim.

Byte-oriented entry points — `binary::insert_bytes`, `binary::replace_bytes`, and `binary::ingest` for whole externally supplied phext byte streams — never reject and never panic on non-UTF-8 scroll data. Valid UTF-8 is stored as ordinary text; anything else is stored as a binary scroll holding the base64 of the exact observed bytes, verbatim, with no normalisation, so the matching fetch returns the original bytes exactly. Within a phext byte stream the nine dimension-break bytes are always structural; binary payloads that may contain them enter through the update functions, armoured as base64.

## The XZ Seal (feature: `xz`)

Compression is a seal, not a working format. A phext is authored, edited, and queried as plain text; when a phext is finished — archived, published, or transported — it may be sealed: xz-compressed as a whole. `seal::seal` produces a standard xz stream (liblzma) over the phext's exact bytes, encoded at preset 6 (the preset is an encoder choice, not a format requirement; any valid xz stream unseals). `seal::unseal` restores the exact original bytes, and sealing is deterministic for a given encoder and preset. To unseal foreign content that may carry non-UTF-8 scrolls, unseal to bytes and pass them through `binary::ingest`.

Both features are optional and off by default: the core crate keeps its single `xxh3` dependency. Enable them with `features = ["binary", "xz"]`.

## Chains (0.6.0)

A phext stream can hold more than one phext. The chain break (0x02) separates whole phexts within a **chain**, and the chain close (0x03) terminates a chain — one stream may hold many chains in succession. The chain bytes form a stream-level layer, one rung above the library break: the nine dimension breaks structure a single phext, the chain bytes structure a stream of phexts. Read in order, a chain's members are editions in sequence — the chain layer is how a phext stream encodes time.

A chain that ends at EOF without a chain close is *unclosed* — possibly truncated — and the API keeps that distinction explicit (`Chain.closed`). A stream with no chain bytes at all parses as a single unclosed chain holding one phext, so pre-0.6 documents read exactly as before, and every single-phext function is untouched: chain bytes are not dimension breaks.

API: `phext::parse_chains` splits a stream into `Chain { members, closed }` values — each member a complete phext string for use with `fetch`, `explode`, and friends — and `phext::serialize_chains` writes them back byte-for-byte.

## Build

1. Clone this repo
2. Install Rust
3. Run `cargo build`

## Test

1. Complete the build steps above
2. Run `cargo test`

## Run

1. After building and testing the project, start the rocket server.
2. Run `cargo run`

### Phext Basics

* explode: Splits an input buffer into a hashmap of scrolls
* implode: Collapses a hashmap of scrolls back into a serialized phext buffer
* test_more_cowbell: Ensures that you've got more cowbell!
* line_break: Proves that we're using ASCII line breaks
* coordinate_parsing: Verifies that string -> coordinate -> string produces the same result
* scrolls: Verifies that SCROLL_BREAK reliably splits 3 scrolls
* sections: Verifies that SECTION_BREAK reliably splits 3 sections
* chapters: Verifies that CHAPTER_BREAK reliably splits 3 chapters
* books: Verifies that BOOK_BREAK reliably splits 3 books
* volumes: Verifies that VOLUME_BREAK reliably splits 3 volumes
* collections: Verifies that COLLECTION_BREAK reliably splits 3 collections
* series: Verifies that SERIES_BREAK reliably splits 3 series
* shelves: Verifies that SHELF_BREAK reliably splits 3 shelves
* libraries: Verifies that LIBRARY_BREAK reliably splits 3 libraries
* coordinates_invalid: tests for invalid coordinate detection
* coordinates_valid: ensures that a realistic coordinate is valid
* realistic_parse: Verifies that a coordinate with many delimiters parses correctly
* dead_reckoning: Verifies that we can accurately calculate coordinates on existing phext documents

### Tests

* next_scroll: verifies that we can tokenize subspace by scroll
* phokenize: verifies that we can build subspace phokens (phext tokens)
* test_url_encoding: tests for alternate url format with semicolons
* coordinate_based_insert: Verifies that random insertion by phext coordinate works
* coordinate_based_replace: Verifies that random replacement by phext coordinate works
* coordinate_based_remove: Verifies that random scroll removal by phext coordinate works
* range_based_replace: Verifies that a range of phext coordinates can be used to replace text
* expand: verifies that delimiters can be grown larger by 1 dimension
* contract: verifies that delimiters can be shrunk by 1 dimension
* merge: verifies that two phext documents can be zipper-merged (intersection)
* subtract: verifies that we can prune all of the coordinates from a second phext document
* normalize: verifies that empty scrolls are pruned from the given phext document

### Regressions

1. While working on the exollama project, I found an input that caused libphext to stall - I was trying to insert a scroll with index=100, which wasn't supported prior to v0.2.0. Performance tuning for exollama will be coming soon, so I bumped the coordinate limit to 1000 for now.