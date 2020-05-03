# working with structured data stored in sled

by Tyler Neely on May 3 2020

[![github](https://img.shields.io/github/stars/spacejam/sled.svg?style=social)](https://github.com/spacejam/sled)
[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![sponsors](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

sled can be thought of as an ordered map from raw byte keys to raw byte values.
You may create multiple isolated map "keyspaces" by using the
[`Db::open_tree`](https://docs.rs/sled/latest/sled/struct.Db.html#method.open_tree)
method. While people who write foundational infrastructure tend not to need
much more than that (we're just plumbers who care about the rate of flow rather
than the specifics of the content), people who write higher-level stateful
applications and systems tend to want to work with structured data.

This tutorial demonstrates how to get structured, SQL-like functionality with
structured keys and values without paying expensive (de)serialization costs.

The complete code for the below examples is available [in the sled
repo](https://github.com/spacejam/sled/blob/master/examples/structured.rs).

## outline

* [zero-copy serialization](#zero-copy-serialization)
* [UPSERT](#upsert) - create or update data with structured keys with endianness considerations
* [variable length fields](#variable-length-fields)
* [hash joins](#hash-joins)

We will use the [`sled`](https://docs.rs/sled) and
[`zerocopy`](https://docs.rs/zerocopy) crates in these examples.

These imports will be expected for the following examples:

```rust
use {
    byteorder::{BigEndian, LittleEndian},
    zerocopy::{
        byteorder::U64, AsBytes, FromBytes, LayoutVerified, Unaligned, U16, U32,
    },
};
```

## zero-copy serialization

We will be relying on Google's [zerocopy](docs.rs/zerocopy) crate
to work with our high-level Rust structures. It's a nice way for
treating unaligned slices of bytes as high-level Rust structures that may
be mutated in-place. It's quite light-weight.

`serde` may accomplish similar funtionality with [the borrow
attribute](https://serde.rs/lifetimes.html#borrowing-data-in-a-derived-impl),
which may reduce user friction in many cases. This can be a bit confusing and
involves more compilation effort though, so we'll use `zerocopy` for the following
examples.

**Alignment** is the requirement for a type to be placed at a specific offset
in memory. For example, the `u64` type (which is 8 bytes long) also needs to have
its first byte located at a memory address which is a multiple of 8. You can see
the alignment of a type by calling `std::mem::align_of::<u64>()`. The address
`888888` is a valid address for a `u64` because it is divisible by 8,
but `888887` is not a valid address because it is not divisible by 8. If the
bytes for the `u64` span a 64-byte boundary, they will exist on different
memory cachelines, increasing the work your machine needs to do to work with
your data. If the bytes for the `u64` span a 4k memory boundary, the data
for this simple type may span multiple DRAM rows which will further
increase the work that your hardware needs to do just to get the bytes
to your cores.

In addition to performance considerations, this matters is because the Rust
compiler expects to translate operations on `u64` into machine code that
requires our data to be located at certain offsets which minimize the amount of
work the CPU needs to perform while operating on it. However, we can use the
`repr(packed)` attribute on our types to remove the requirement, possibly reduce
the amount of space required, and possibly increase the amount of work required
to shift things around before performing operations on data on the CPU. Here's
`repr(packed)` in action:

```rust
#[repr(packed)]
struct Packed(u64);

dbg!(
  std::mem::align_of::<u64>(),
  std::mem::align_of::<Packed>(),
)
```

Which prints out:

```
[src/main.rs:5] std::mem::align_of::<u64>() = 8
[src/main.rs:5] std::mem::align_of::<Packed>() = 1
```

Trying to perform CPU instructions that require specific alignment of data on
incorrectly aligned memory can cause a process on a unix-like system to be
immediately terminated with the unhandlable signal `SIGBUS`.

In sled, data is stored in an inlinable, potentially `Arc`-backed buffer
called an `IVec`. It has no alignment guarantees, so you should not
assume your data will have any particular alignment.

**Endianness** is the order in which bytes are laid out

The `zerocopy` crate lets us derive 3 important traits:

* `FromBytes` lets us view byte slices as a higher-level type
* `AsBytes` lets us easily view the raw memory backing a type as a byte slice
* `Unaligned` guarantees that the members of our structure do not have alignment requirements

Here's what it looks like to define a structure that does not have alignment requirements:

```rust
#[derive(FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct Key {
    a: U64<BigEndian>,
    b: U64<BigEndian>,
}
```

Note the type `U64<BigEndian>`. `zerocopy::byteorder::U64` is an unaligned
`u64` that is represented as `BigEndian` bytes.

We use `BigEndian` for key types because it preserves lexicographic ordering,
which is nice if we ever want to iterate over our items in order.

We use `LittleEndian` for values because it's possibly cheaper on x86-based
machines, but the difference isn't likely to be very significant.

```rust
#[derive(FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct Value {
    count: U64<LittleEndian>,
    whatever: [u8; 16],
}
```

If we tried to add a type with an alignment requirement like `u64` to a
structure deriving `Unaligned`, we would get an error: `u64:
zerocopy::Unaligned is not satisfied`. It's a nice compile-time
safety check.

After we've defined our structures, we can use the `LayoutVerified`
wrapper from `zerocopy` to view a byte slice as our high-level
structure, possibly in a mutable way if we give it a mutable slice.

## UPSERT

The `UPSERT` command in SQL allows an item to be updated or inserted if it
does not already exist. We can achieve the same functionality in sled with
the `update_and_fetch` method, which will atomically apply a function
to an existing value, possibly retrying if it hits contention, and
returning the new value. There is also the `fetch_and_update` method
if you want to get the old value.

```rust
fn upsert(db: &sled::Db) -> sled::Result<()> {

    let key = Key { a: U64::new(21), b: U64::new(890) };

    // "UPSERT" functionality
    db.update_and_fetch(key.as_bytes(), |value_opt| {
        if let Some(existing) = value_opt {
            // We need to make a copy that will be written back
            // into the database. This allows other threads that
            // may have witnessed the old version to keep working
            // without taking out any locks. IVec will be
            // stack-allocated until it reaches 22 bytes
            let mut backing_bytes = sled::IVec::from(existing);

            // this verifies that our value is the correct length
            // and alignment (in this case we don't need it to be
            // aligned, because we use the `U64` type from zerocopy)
            let layout: LayoutVerified<&mut [u8], Value> =
                LayoutVerified::new_unaligned(&mut *backing_bytes)
                    .expect("bytes do not fit schema");

            // this lets us work with the underlying bytes as
            // a mutable structured value.
            let value: &mut Value = layout.into_mut();

            let new_count = value.count.get() + 1;

            println!("incrementing count to {}", new_count);

            value.count.set(new_count);

            Some(backing_bytes)
        } else {
            println!("setting count to 0");

            Some(sled::IVec::from(
                Value { count: U64::new(0), whatever: [0; 16] }.as_bytes(),
            ))
        }
    })?;

    Ok(())
}
```

## variable length fields

This function shows how to put a variable length component in either the beginning or the end of your value.

```rust
// Cat values will be:
// favorite_number + battles_won + <home name variable bytes>
#[derive(FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct CatValue {
    favorite_number: U64<LittleEndian>,
    battles_won: U64<LittleEndian>,
}

// Dog values will be:
// <home name variable bytes> + woof_count + postal_code
#[derive(FromBytes, AsBytes, Unaligned)]
#[repr(C)]
struct DogValue {
    woof_count: U32<LittleEndian>,
    postal_code: U16<LittleEndian>,
}

fn variable_lengths(db: &sled::Db) -> sled::Result<()> {
    // here we will show how we can use zerocopy for inserting
    // fixed-size components, mixed with variable length
    // records on the end or beginning.

    // the hash_join example below shows how to read items
    // out in a way that accounts for the variable portion,
    // using `zerocopy::LayoutVerified::{new_from_prefix, new_from_suffix}`

    let dogs = db.open_tree(b"dogs")?;

    let mut dog2000_value = vec![];
    dog2000_value.extend_from_slice(b"science zone");
    dog2000_value.extend_from_slice(
        DogValue { woof_count: U32::new(666), postal_code: U16::new(42) }
            .as_bytes(),
    );
    dogs.insert("dog2000", dog2000_value)?;

    let mut zed_pup_value = vec![];
    zed_pup_value.extend_from_slice(b"bowling alley");
    zed_pup_value.extend_from_slice(
        DogValue { woof_count: U32::new(32113231), postal_code: U16::new(0) }
            .as_bytes(),
    );
    dogs.insert("zed pup", zed_pup_value)?;

    // IMPORTANT NOTE: German dogs eat food called "barf"
    let mut klaus_value = vec![];
    klaus_value.extend_from_slice(b"barf shop");
    klaus_value.extend_from_slice(
        DogValue { woof_count: U32::new(0), postal_code: U16::new(12045) }
            .as_bytes(),
    );
    dogs.insert("klaus", klaus_value)?;

    let cats = db.open_tree(b"cats")?;

    let mut laser_cat_value = vec![];
    laser_cat_value.extend_from_slice(
        CatValue {
            favorite_number: U64::new(11),
            battles_won: U64::new(321231321),
        }
        .as_bytes(),
    );
    laser_cat_value.extend_from_slice(b"science zone");
    cats.insert("laser cat", laser_cat_value)?;

    let mut pulsar_cat_value = vec![];
    pulsar_cat_value.extend_from_slice(
        CatValue {
            favorite_number: U64::new(11),
            battles_won: U64::new(321231321),
        }
        .as_bytes(),
    );
    pulsar_cat_value.extend_from_slice(b"science zone");
    cats.insert("pulsar cat", pulsar_cat_value)?;

    let mut fluffy_value = vec![];
    fluffy_value.extend_from_slice(
        CatValue {
            favorite_number: U64::new(11),
            battles_won: U64::new(321231321),
        }
        .as_bytes(),
    );
    fluffy_value.extend_from_slice(b"bowling alley");
    cats.insert("fluffy", fluffy_value)?;

    Ok(())
}
```

## hash joins

```rust
fn hash_join(db: &sled::Db) -> sled::Result<()> {
    // here we will try to find cats and dogs who
    // live in the same home.

    let cats = db.open_tree(b"cats")?;
    let dogs = db.open_tree(b"dogs")?;

    let mut join = std::collections::HashMap::new();

    for name_value_res in &cats {
        // cats are stored as name -> favorite_number + battles_won + home name variable bytes
        let (name, value_bytes) = name_value_res?;
        let (_, home_name): (LayoutVerified<&[u8], CatValue>, &[u8]) =
            LayoutVerified::new_from_prefix(&*value_bytes).unwrap();
        let (ref mut cat_names, _dog_names) =
            join.entry(home_name.to_vec()).or_insert((vec![], vec![]));
        cat_names.push(std::str::from_utf8(&*name).unwrap().to_string());
    }

    for name_value_res in &dogs {
        // dogs are stored as name -> home name variable bytes + woof count + postal code
        let (name, value_bytes) = name_value_res?;

        // note that this is reversed from the cat example above, where
        // the variable bytes are at the other end of the value, and are
        // extracted using new_from_prefix instead of new_from_suffix.
        let (home_name, _dog_value): (_, LayoutVerified<&[u8], DogValue>) =
            LayoutVerified::new_from_suffix(&*value_bytes).unwrap();

        if let Some((_cat_names, ref mut dog_names)) = join.get_mut(home_name) {
            dog_names.push(std::str::from_utf8(&*name).unwrap().to_string());
        }
    }

    for (home, (cats, dogs)) in join {
        println!(
            "the cats {:?} and the dogs {:?} live in the same home of {}",
            cats,
            dogs,
            std::str::from_utf8(&home).unwrap()
        );
    }

    Ok(())
}
```
