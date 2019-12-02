# sled

[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![Open Collective backers](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

Welcome to the introduction for the sled database! We'll keep this short and sweet.

Sled can be thought of as a BTreeMap<Vec<u8>, Vec<u8>> that stores its data on disk.
Sled is an [embedded database](https://en.wikipedia.org/wiki/Embedded_database).

Embedded databases are useful in several cases:

* you want to store data on disk, without facing [the complexity of files](https://danluu.com/file-consistency/)
* you want to be simple, without operating an external database
* you want to be fast, without paying network costs
* using disk storage as a building block in your system

# Let's get going!

Open your rust project, or create one with `cargo new sled-intro`.

In `Cargo.toml`, add sled to the dependencies section:

```toml
[dependencies]
sled = "0.29"
```

Now, in your Rust code:

```rust
fn main() -> sled::Result<()> {
    let path = "my_storage_directory";

    // works like std::fs::open
    let db = sled::Db::open(path)?;

    let key = "my key";
    let value = vec![42];

    dbg!(
        db.insert(key, value)?, // as in BTreeMap::insert
        db.get(key)?,           // as in BTreeMap::get
        db.remove(key)?,        // as in BTreeMap::remove
    );

    Ok(())
}
```

This will create a new directory, `my_storage_directory`, write
a new item into the database inside, retrieve the item, and then remove it.

The key and value types can be a `Vec<u8>`, a `[u8]`, or a `str`.
These will be converted to the sled `IVec` type automatically.
`IVec` is an `Arc<[u8]>` that will not allocate if the value is small.

All sled operations return a `sled::Result` that should never
be ignored. If this is an `Err`, it means a serious unexpected issue
has happened. Operations that may fail in expected ways, like
`compare_and_swap`, have a return type of `sled::Result<CompareAndSwapResult>`
where the expected failure is nested inside the unexpected failure.
This allows users to use the try operator on every sled operation, and
locally reason about errors that are likely to be encountered.
This allows your error handling logic to take full advantage of Rust's exhaustive pattern matching.

# additional features

* fully serializable multi-key and multi-Tree [transactions](https://docs.rs/sled/latest/sled/struct.Tree.html#method.transaction) involving up to 69 separate Trees!
* fully atomic single-key operations, supports [compare and swap](https://docs.rs/sled/latest/sled/struct.Tree.html#method.compare_and_swap)
* [write batch support](https://docs.rs/sled/latest/sled/struct.Tree.html#method.apply_batch)
* [subscription/watch semantics on key prefixes](https://github.com/spacejam/sled/wiki/reactive-semantics)
* [multiple keyspace/Tree support](https://docs.rs/sled/latest/sled/struct.Db.html#method.open_tree)
* [merge operators](https://github.com/spacejam/sled/wiki/merge-operators)
* forward and reverse iterators
* a crash-safe monotonic [ID generator](https://docs.rs/sled/latest/sled/struct.Db.html#method.generate_id) capable of generating 75-125 million unique ID's per second
* [zstd](https://github.com/facebook/zstd) compression (use the `compression` build feature)
