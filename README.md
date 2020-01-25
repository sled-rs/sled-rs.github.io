# sled
[![github](https://img.shields.io/github/stars/spacejam/sled.svg?style=social)](https://github.com/spacejam/sled)
[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![Open Collective backers](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

A modern [embedded database](https://en.wikipedia.org/wiki/Embedded_database).
Written in Rust, usable on servers and phones from any C-compatible language.

```rust
let db = sled::open(path)?; // as in fs::open
db.insert(k, v)?;           // as in BTreeMap::insert
db.get(&k)?;                // as in BTreeMap::get
for kv in db.range(k..) {}  // as in BTreeMap::range
db.remove(&k)?;             // as in BTreeMap::remove
drop(db);                   // fsync and close file
```

Embedded databases are useful in several cases:

* you want to store data on disk, without facing [the complexity of files](https://danluu.com/file-consistency/)
* you want to be simple, without operating an external database
* you want to be fast, without paying network costs
* using disk storage as a building block in your system

# sled features

* [API](https://docs.rs/sled) similar to a threadsafe `BTreeMap<[u8], [u8]>`
* fully serializable multi-key and multi-Tree [transactions](https://docs.rs/sled/latest/sled/struct.Tree.html#method.transaction)
* fully atomic single-key operations, including [compare and swap](https://docs.rs/sled/latest/sled/struct.Tree.html#method.compare_and_swap) and [update and fetch](https://docs.rs/sled/latest/sled/struct.Tree.html#method.update_and_fetch)
* zero-copy reads
* [write batch support](https://docs.rs/sled/latest/sled/struct.Tree.html#method.apply_batch)
* [subscription/watch semantics on key prefixes](https://github.com/spacejam/sled/wiki/reactive-semantics)
* [multiple keyspaces](https://docs.rs/sled/latest/sled/struct.Db.html#method.open_tree)
* [merge operators](https://github.com/spacejam/sled/wiki/merge-operators)
* forward and reverse iterators
* a crash-safe monotonic [ID generator](https://docs.rs/sled/latest/sled/struct.Db.html#method.generate_id) capable of generating over 100 million unique ID's per second
* [zstd](https://github.com/facebook/zstd) compression (use the `compression` build feature)
* cpu-scalable lock-free implementation
* SSD-optimized log-structured storage
* prefix encoded and suffix truncated keys reducing the storage cost of complex keys

# references

* [architectural outlook](https://github.com/spacejam/sled/wiki/sled-architectural-outlook)
* [The Bw-Tree: A B-tree for New Hardware Platforms](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/bw-tree-icde2013-final.pdf)
* [LLAMA: A Cache/Storage Subsystem for Modern Hardware](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/llama-vldb2013.pdf)
* [Cicada: Dependably Fast Multi-Core In-Memory Transactions](http://15721.courses.cs.cmu.edu/spring2018/papers/06-mvcc2/lim-sigmod2017.pdf)
* [The Design and Implementation of a Log-Structured File System](https://people.eecs.berkeley.edu/~brewer/cs262/LFS.pdf)
* [TinyLFU: A Highly Efficient Cache Admission Policy](https://arxiv.org/abs/1512.00727)
