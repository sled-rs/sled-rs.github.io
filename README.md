# sled

[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![Open Collective backers](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

A modern embedded database. Written in Rust, usable on servers and phones from any C-compatible language.

```rust
use sled::Db;

let db = Db::open(path)?;   // as in fs::open
db.insert(k, v)?;           // as in BTreeMap::insert
db.get(&k)?;                // as in BTreeMap::get
for kv in db.range(k..) {}  // as in BTreeMap::range
db.remove(&k)?;             // as in BTreeMap::remove
drop(db);                   // fsync and close file
```

# what's an embedded database?

An embedded database is a library that performs storage operations.
Why not use files directly?
[It's basically impossible to get right, even for experts](https://danluu.com/file-consistency/).
An embedded database can store data on disk more quickly and with far fewer bugs than if you were to use files directly.
Despite the simple API, it often takes years of hard work to get the underlying storage system into a nice place.

Embedded databases may use techniques like:

* transactions that allow data to be read and mutated by multiple threads at the same time without causing data races
* lock-free indexing that allows data to be accessed without blocking other threads
* recovery techniques that avoid corruption even when the database crashes while writing data
* caching techniques that are scan-resistant
* deserialized object caching that facilitates transactions on objects directly, without [paying hefty serialization or networking costs required to interact with external databases](https://ai.google/research/pubs/pub48030)

# popular uses of embedded databases

* app state storage without an external database
* high-performance or transactional shared state
* larger-than-memory datasets such as higher-level databases, queues

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

<p><small>Hosted on GitHub Pages &mdash; Theme by <a href="https://github.com/orderedlist">orderedlist</a></small></p>
