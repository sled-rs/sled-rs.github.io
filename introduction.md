# sled

[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![sponsors](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

Welcome to the introduction for the sled embedded database! We'll keep this short and sweet.

Sled can be thought of as a `BTreeMap<[u8], [u8]>` that stores its data on disk.

[Embedded databases](https://en.wikipedia.org/wiki/Embedded_database) are useful in several cases:

* you want to store data on disk, without facing [the complexity of files](https://danluu.com/file-consistency/)
* you want to be simple, without operating an external database
* you want to be fast, without paying network costs
* using disk storage as a building block in your system

# Let's get going!

Open your rust project, or create one with `cargo new sled-intro`.

In `Cargo.toml`, add sled to the dependencies section:

```toml
[dependencies]
sled = "0.31"
```

Now, in your Rust code:

```rust
fn main() -> sled::Result<()> {
    // this directory will be created if it does not exist
    let path = "my_storage_directory";

    // works like std::fs::open
    let db = sled::open(path)?;

    // key and value types can be `Vec<u8>`, `[u8]`, or `str`.
    let key = "my key";

    // `generate_id`
    let value = db.generate_id()?.to_be_bytes();

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
If `remove` were not used, the data would be stored safely
on disk for future access.

## Key and Value Types

The key and value types can be a `Vec<u8>`, a `[u8]`, or a `str`.
These will be converted to the sled `IVec` type automatically.
`IVec` is an `Arc<[u8]>` that will not allocate if the value is small.
It has implemented the `Deref<Target=[u8]>` trait, which means it
may use all of the `&[u8]` slice methods.

## Error Handling

All sled operations return a `sled::Result` that should never
be ignored. If this is an `Err`, it means a serious unexpected issue
has happened. Operations that may fail in expected ways, like
`compare_and_swap`, have a return type of `sled::Result<CompareAndSwapResult>`
where the expected failure is nested inside the unexpected failure.
This allows users to use the try operator on every sled operation, and
locally reason about errors that are likely to be encountered.
This allows your error handling logic to take full advantage of Rust's exhaustive pattern matching.

## Thread Safety

All operations in sled are thread-safe. The `Db` may be cloned and shared across threads
without needing to use `Arc` or `Mutex` etc... Internally, sled relies on
atomic operations to guarantee correctness when being used by multiple threads.
Sled has been designed from the beginning to perform well with highly concurrent
workloads.


# Advanced Features

Many more advanced features are supported, which might be useful for creators of higher performance stateful systems.

* fully serializable multi-key and multi-Tree [transactions](https://docs.rs/sled/latest/sled/struct.Tree.html#method.transaction) involving up to 69 separate Trees!
* fully atomic single-key operations, supports [compare and swap](https://docs.rs/sled/latest/sled/struct.Tree.html#method.compare_and_swap)
* [write batch support](https://docs.rs/sled/latest/sled/struct.Tree.html#method.apply_batch)
* [subscription/watch semantics on key prefixes](https://github.com/spacejam/sled/wiki/reactive-semantics)
* [multiple keyspace/Tree support](https://docs.rs/sled/latest/sled/struct.Db.html#method.open_tree)
* [merge operators](https://github.com/spacejam/sled/wiki/merge-operators)
* a crash-safe monotonic [ID generator](https://docs.rs/sled/latest/sled/struct.Db.html#method.generate_id) capable of generating 75-125 million unique ID's per second
* [zstd](https://github.com/facebook/zstd) compression (use the `compression` build feature)
