# libphext

This project is the primary implementation of 11-dimensional plain hypertext, or Phext. For more information about the phext format, head over to https://phext.io.

## Elevator Pitch

Phext is hierarchical digital memory. It enables seamless knowledge transfer between humans and computers. Let's learn how to think at multiversal-scale. :)

## Zero Dependencies*

Phext is just 11-dimensional text. As such, you only need phext.rs and the standard libraries to work with it.* This tiny dependency gives you hierarchical superpowers. Use them wisely!

Note: We depend upon `xxh3` for checksum content hashes.

## Phext Motivation

In the 1980s, computers could write 25 KB/sec to a floppy disk. In the 2020s, it became possible to write 2 GB/sec to an SSD. This changed the definition of a "small" file. Unfortunately, most of our file abstractions (especially on Windows) have not scaled to take advantage of these performance gains. For the most part, this isn't much of a problem: humans are still rate-limited at 300 bps using keyboards. At some point in the next 25 years, however, we will have high-bandwidth brain interconnects - at which point we will need a high-bandwidth multi-dimensional text format: phext!

The introduction of Large Language Models (LLMs) has <em>accelerated</em> our transition to this future. You can use phext to interact with agents and groups of humans at scale - think of visualizing 9 billion computer screens at once. Phext is like being given a coordinate system of coordinate systems, allowing you to walk the latent space of any problem space efficiently.

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