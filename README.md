# sled

| key | value |
| :-: | --- |
| [documentation](https://docs.rs/sled) | [![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled) |
| [chat about databases with us](https://discord.gg/Z6VsXds) | [![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds) |
| [help us build what you want to use](https://opencollective.com/sled) | [![Open Collective backers](https://img.shields.io/opencollective/backers/sled)](https://opencollective.com/sled) |

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
let mut iter = tree.range(k..);
assert_eq!(iter.next(), Some(Ok((k, v2))));
assert_eq!(iter.next(), None);

// deletion
tree.remove(&k);

// block until all operations are on-disk
// (flush_async also available to get a Future)
tree.flush();
```

# features

* API similar to a threadsafe `BTreeMap<Vec<u8>, Vec<u8>>`
* ACID, constant crash and concurrency testing
* SSD-optimized log-structured storage
* cpu-scalable lock-free implementation
* [LSM tree](https://en.wikipedia.org/wiki/Log-structured_merge-tree)-like write performance
  with [B+ tree](https://en.wikipedia.org/wiki/B%2B_tree)-like read performance
* multiple keyspace support
* subscription/watch semantics on key prefixes
* forward, reverse, range iterators
* a crash-safe monotonic ID generator capable of generating 125+ million IDs per second
* [zstd](https://github.com/facebook/zstd) compression (use the `compression` build feature)
* [merge operators](https://github.com/spacejam/sled/wiki/merge-operators)

# goals

1. don't make the user think. the interface should be obvious.
1. don't surprise users with performance traps.
1. don't wake up operators. bring reliability techniques from academia into real-world practice.
1. don't use so much electricity. our data structures should play to modern hardware's strengths.

# architecture

lock-free tree on a lock-free pagecache on a lock-free log. the pagecache scatters
partial page fragments across the log, rather than rewriting entire pages at a time
as B+ trees for spinning disks historically have. on page reads, we concurrently
scatter-gather reads across the log to materialize the page from its fragments.
check out the [architectural outlook](https://github.com/spacejam/sled/wiki/sled-architectural-outlook)
for a more detailed overview of where we're at and where we see things going!

# References

* [The Bw-Tree: A B-tree for New Hardware Platforms](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/bw-tree-icde2013-final.pdf)
* [LLAMA: A Cache/Storage Subsystem for Modern Hardware](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/llama-vldb2013.pdf)
* [Cicada: Dependably Fast Multi-Core In-Memory Transactions](http://15721.courses.cs.cmu.edu/spring2018/papers/06-mvcc2/lim-sigmod2017.pdf)
* [The Design and Implementation of a Log-Structured File System](https://people.eecs.berkeley.edu/~brewer/cs262/LFS.pdf)
