* support common file formats
  * tar
  * zip
  * sqlite
  * csv
  * sql
* non-linear flows
  Q: what happens to information as it flows along a path of phext coordinates?
  say we want to define stable regions early in the file...
  we could define a phext-based mask to assist with indexing
* hierarchical mobs
* DB emulation
* memory-mapped I/O
* investigate data sources
  * https://huggingface.co/learn/nlp-course/chapter2/4?fw=pt
  * https://commoncrawl.org/get-started
  * https://medium.com/@zolayola/public-data-sets-in-the-era-of-llms-0a4e89bda658* wikibench targets (phext-lattice/bench/wiki/FINDINGS.md, 38k-article / 49 MB phext)
  * indexed fetch: `fetch` scans linearly (12.8 ms p50 at 49 MB vs 0.7 µs with phext-lattice's span index); upstream a span index type
  * splice API: `replace` + rewrite takes 124 ms at 49 MB; return (offset, old_len, new bytes) so callers can patch or journal
  * persisted index: an index (and name map) that survives across opens, so open time stops growing with file size
