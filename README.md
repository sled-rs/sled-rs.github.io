# sled

[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![Open Collective backers](https://img.shields.io/opencollective/backers/sled)](https://opencollective.com/sled)

A modern embedded database. Written in Rust, usable on servers and phones from any C-compatible language.

```rust
use sled::Db;

let tree = Db::open(path)?;

// set and get
tree.insert(k, v1);
assert_eq!(tree.get(&k), Ok(Some(v1)));

// compare and swap
tree.compare_and_swap(k, Some(v1), Some(v2));

// scan forward
for kv in tree.range(k..) {}

// deletion
tree.remove(&k);

// block until all operations are on-disk
tree.flush();

// or if using async
tree.flush_async().await;
```

# features

* [API](https://docs.rs/sled) similar to a threadsafe `BTreeMap<[u8], [u8]>`
* fully serializable multi-key and multi-Tree [transactions](https://docs.rs/sled/latest/sled/struct.Tree.html#method.transaction) involving up to 69 separate Trees!
* fully atomic single-key operations, supports [compare and swap](https://docs.rs/sled/latest/sled/struct.Tree.html#method.compare_and_swap)
* zero-copy reads
* [write batch support](https://docs.rs/sled/latest/sled/struct.Tree.html#method.apply_batch)
* [subscription/watch semantics on key prefixes](https://github.com/spacejam/sled/wiki/reactive-semantics)
* [multiple keyspace/Tree support](https://docs.rs/sled/latest/sled/struct.Db.html#method.open_tree)
* [merge operators](https://github.com/spacejam/sled/wiki/merge-operators)
* forward and reverse iterators
* a crash-safe monotonic [ID generator](https://docs.rs/sled/latest/sled/struct.Db.html#method.generate_id) capable of generating 75-125 million unique ID's per second
* [zstd](https://github.com/facebook/zstd) compression (use the `compression` build feature)
* cpu-scalable lock-free implementation
* SSD-optimized log-structured storage
* prefix encoded keys reducing the storage cost of complex keys

# goals

1. don't make the user think. the interface should be obvious.
1. don't surprise users with performance traps.
1. don't wake up operators. bring reliability techniques from academia into real-world practice.
1. don't use so much electricity. our data structures should play to modern hardware's strengths.


# references

* [architectural outlook](https://github.com/spacejam/sled/wiki/sled-architectural-outlook)
* [The Bw-Tree: A B-tree for New Hardware Platforms](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/bw-tree-icde2013-final.pdf)
* [LLAMA: A Cache/Storage Subsystem for Modern Hardware](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/llama-vldb2013.pdf)
* [Cicada: Dependably Fast Multi-Core In-Memory Transactions](http://15721.courses.cs.cmu.edu/spring2018/papers/06-mvcc2/lim-sigmod2017.pdf)
* [The Design and Implementation of a Log-Structured File System](https://people.eecs.berkeley.edu/~brewer/cs262/LFS.pdf)

<p><small>Hosted on GitHub Pages &mdash; Theme by <a href="https://github.com/orderedlist">orderedlist</a></small></p>
