* content checksums
  * incorporate the content soundex from raap
    see: https://phext.io/api.php?seed=raap&token=research&coordinate=1.1.1/1.1.1/1.1.2
* conversion methods
  * tar
  * zip
  * sqlite
  * local file system
* non-linear flows
  Q: what happens to information as it flows along a path of phext coordinates?
  say we want to define stable regions early in the file...
  we could define a phext-based mask to assist with indexing
* hierarchical mobs
* DB emulation
* Liquid Peanut Butter
  * it's a bit nutty!
* fast indexing
  * checksum forking: record expected offsets and checksums in .checksum files
  * hierarchy map in memory
  * memory-mapped I/O
* investigate data sources
  * https://huggingface.co/learn/nlp-course/chapter2/4?fw=pt
  * https://commoncrawl.org/get-started
  * https://medium.com/@zolayola/public-data-sets-in-the-era-of-llms-0a4e89bda658