# Error Handling in a Correctness-Critical Rust Project

by Tyler Neely on April 8 2020

[![github](https://img.shields.io/github/stars/spacejam/sled.svg?style=social)](https://github.com/spacejam/sled)
[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![sponsors](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

Let's begin with two excerpts from the paper [Simple Testing Can Prevent Most
Critical Failures: An Analysis of Production Failures in Distributed
Data-intensive
Systems](http://www.eecg.toronto.edu/~yuan/papers/failure_analysis_osdi14.pdf)

```
almost all (92%) of the catastrophic system failures
are the result of incorrect handling of non-fatal errors
explicitly signaled in software.
```

```
in 58% of the catastrophic failures, the underlying
faults could easily have been detected through simple
testing of error handling code.
```

These stats haunt me. They cause me to frequently ask myself "how can I design
my systems to increase the chances that errors will be handled correctly?"

This leads to two goals:

1. when an error happens, it is handled correctly
2. error handling logic is triggered under test

# error handling in Rust

In Rust, error handling is centered around the `Result` enum and the try `?`
operator.

`Result` is [defined like this](https://doc.rust-lang.org/std/result/enum.Result.html):

```rust
pub enum Result<T, E> {
    Ok(T),
    Err(E),
}

// This `use` lets us write `Ok(Happy)` instead
// of `Result::Ok(Happy)` as we need to do with
// other enums by default.
pub use Result::{Ok, Err};
```

and it is used like this:

```rust
fn may_fail() -> Result<Happy, Sad> {
  if /* function succeeded */ {
    Ok(Happy)
  } else {
    Err(Sad)
  }
}
```

We use `Result` to represent an operation which may succeed or fail. We tend
not to write many functions that accept `Result`s as arguments, because a
`Result` fundamentally represents uncertainty about whether an operation will
succeed or not. By the time we have an actual `Result` object, we no longer
have uncertainty about whether the operation was successful or not. We know
what happened. Results tend to flow backwards to callers, rather than forwards
into newly called functions.

Error handling may begin once it is known that an error has occurred.
However, we often do not wish to handle an error at the exact point
in which it is known to have happened. Imagine this code:

```rust
fn may_fail() -> Result<Happy, Sad> {
  /* either returns Ok(Happy) or Err(Sad) */
}

fn caller() {
  match may_fail() {
    Ok(happy) => println!(":)"),
    Err(sad) => {
      eprintln!(":(");
      /* handle error */
      return;
    }
  }
  match may_fail() {
    Ok(happy) => println!(":)"),
    Err(sad) => {
      eprintln!(":(");
      /* handle error */
      return;
    }
  }
  match may_fail() {
    Ok(happy) => println!(":)"),
    Err(sad) => {
      eprintln!(":(");
      /* handle error */
      return;
    }
  }
  println!("I am so happy right now");
}
```

Error handling can easily become repetitive and error-prone. Error handling
logic is an area where applying the [single responsibility
principle](https://en.wikipedia.org/wiki/Single-responsibility_principle) can
really reduce bugs over time. If handling of a particular kind of
error can happen in one place, you can eliminate the chance that a bug
will happen because you forgot to refactor 1 out of 5 places where a
concern is handled in your codebase. These bugs are really easy to
introduce when refactoring Rust, as we tend to spend so much
energy fixing compiler errors during refactors that we may forget
to sweep through the codebase and check to make sure that it has
remained coherent and that all separate locations of similar
techniques have remained in-sync with each other.

It's quite easy to do a refactor of the above code and end
up with something like this, where the last instance missed
the newly changed logic:

```rust
match may_fail() {
  Ok(happy) => println!(":)"),
  Err(sad) => {
    eprintln!(":(");
    /* handle error */
    /* new and improved extra step */
    return;
  }
}
match may_fail() {
  Ok(happy) => println!(":)"),
  Err(sad) => {
    eprintln!(":(");
    /* handle error */
    /* new and improved extra step */
    return;
  }
}
match may_fail() {
  Ok(happy) => println!(":)"),
  Err(sad) => {
    eprintln!(":(");
    /* handle error */
                    <----- we forgot to update this
    return;
  }
}
```

So, it helps to centralize the error handling logic:

```rust
fn may_fail() -> Result<Happy, Sad> {
  /* either returns Ok(Happy) or Err(Sad) */
}

fn call_and_handle() {
  match may_fail() {
    Ok(happy) => println!(":)"),
    Err(sad) => {
      eprintln!(":(");
      /* handle error */
      /* new and improved extra step */
      return;
    }
  }
}

fn caller() {
  call_and_handle();
  call_and_handle();
  call_and_handle();
  println!("I am so happy right now");
}
```

Unfortunately, the intent of the original program has been distorted. We will
now print "I am so happy right now" even after experiencing some failures from
the `may_fail()` function. We also keep calling `call_and_handle()` even if
the last call failed. We want to short-circuit that print statement, as well
as subsequent calls to `call_and_handle`, as soon as the first one encounters
issues.

Here is where we start to get into precarious territory by introducing
the try `?` operator to add short-circuiting logic.

```rust
fn may_fail() -> Result<Happy, Sad> {
  /* either returns Ok(Happy) or Err(Sad) */
}

fn call_and_handle() -> Result<(), ()> {
  match may_fail() {
    Ok(happy) => {
      println!(":)");
      Ok(())
    },
    Err(sad) => {
      eprintln!(":(");
      /* handle error */
      /* new and improved extra step */
      Err(())
    }
  }
}

fn caller() -> Result<(), ()> {
  call_and_handle()?;
  call_and_handle()?;
  call_and_handle()?;
  println!("I am so happy right now");
  Ok(())
}
```

This fulfils the above constraints, but we have now made `caller` return a
`Result`, even when we have already handled any errors that it may have
encountered. Callers of `caller` don't need to care about any issues that have
cropped up during its execution, because it has already handled them. We do not
want our handled errors to propagate any information at all to the caller,
because it only allows them to begin to be concerned about an issue that they
are not responsible for handling in any way. This is as unhealthy in programs
as it can sometimes be in human interpersonal communication, as it encourages
core concerns to be handled by an entity that has less information about that
core concern, resulting in harmful coupling and more bugs over time.

Any callers of the `caller()` function will start to get compiler warnings
if they don't use the `Result` that is returned:

```
warning: unused `std::result::Result` that must be used
 --> src/main.rs:6:5
  |
  |     caller();
  |     ^^^^^^^^^
  |
  = note: `#[warn(unused_must_use)]` on by default
  = note: this `Result` may be an `Err` variant,
          which should be handled
```

So, you could easily imagine someone writing code like this, with a `main()`
function that uses the try `?` operator to get rid of that compiler warning:

```rust
fn may_fail() -> Result<Happy, Sad> {
  /* either returns Ok(Happy) or Err(Sad) */
}

fn call_and_handle() -> Result<(), ()> {
  match may_fail() {
    Ok(happy) => {
      println!(":)");
      Ok(())
    },
    Err(sad) => {
      eprintln!(":(");
      /* handle error */
      /* new and improved extra step */
      Err(())
    }
  }
}

fn caller() -> Result<(), ()> {
  call_and_handle()?;
  call_and_handle()?;
  call_and_handle()?;
  println!("I am so happy right now");
  Ok(())
}

fn main() -> Result<(), ()> {
  caller()?;
  caller()?;
  caller()
}
```

This gets rid of the compiler warning. However, this is buggy, because the
`caller()` function already handles its concerns, and we don't need to care about
its success. If the intent is to call it 3 times, we now are encouraged by the
compiler to early-exit as well.

The try `?` operator can save us a lot of effort, but it also has risks. We are
enticed by the easy early return. But try `?` is fundamentally about
propagation. And we must only use it when we require that the caller handles
the issue that has popped up.

Our intention is to call `caller()` 3 times in `main()`, regardless of whether
`caller()` needs to handle errors internally or not. This is what we really
want:

```rust
fn may_fail() -> Result<Happy, Sad> {
  /* either returns Ok(Happy) or Err(Sad) */
}

fn call_and_handle() -> bool {
  match may_fail() {
    Ok(happy) => {
      println!(":)");
      true
    },
    Err(sad) => {
      eprintln!(":(");
      /* handle error */
      /* new and improved extra step */
      false
    }
  }
}

fn caller() {
  // using && will also short-circuit evaluation
  if call_and_handle()
    && call_and_handle()
    && call_and_handle() {
    println!("I am so happy right now");
  }
}

fn main() {
  // our intention is to call `caller()` 3 times,
  // whether it needs to handle errors internally
  // or not.
  caller();
  caller();
  caller();
}
```

# why does this matter?

There is a tendency in the Rust community to throw all errors into a single
global error type, which is a big enum that holds the various possible errors
that may have been encountered at any point anywhere in the program. It's easy
to see how this makes working with errors super easy.

But it's easy for the wrong reasons.

Remember our first goal from the beginning of this article:

```
1. when an error happens, it is handled correctly
```

If we have different errors that pop up in different parts of our programs,
our goal is for those errors to be handled correctly. This is almost
impossible to do correctly when all possible errors may end up being
converted into this big-ball-of-mud single error enum.

Imagine this system where we are expecting to encounter both handleable simple
errors and much more serious fatal errors. We will use the global error enum
style that has become popular:

```rust
struct LocalError;
struct FatalError;

enum Error {
  Local(LocalError),
  Fatal(FatalError),
}

// these conversions allow the try `?` operator
// to automatically turn a specific error into
// the global `Error` when early-returning
// from functions
impl From<LocalError> for Error {
  // Error::Local(LocalError)
}
impl From<FatalError> for Error {
  // Error::Fatal(FatalError)
}

fn subtask_a() -> Result<(), LocalError> {
  /* perform work, maybe fail */
}

fn subtask_b() -> Result<(), FatalError> {
  /* perform work, maybe fail */
}

// the try `?` operator uses the `From` impl to convert
// from `LocalError` and `FatalError` into `Error`
fn perform_work() -> Result<(), Error> {
  subtask_a()?;
  subtask_b()?;
  subtask_a()?;
  Ok(())
}

fn main() -> Result<(), Error> {
  loop {
    perform_work()?;
  }
}

```

Everything looks pretty normal. Especially because there's actually no
error handling that is happening, in violation of goal #1 above.
Let's handle our local errors:

```rust
fn subtask_a() -> Result<(), LocalError> {
  /* perform work, maybe fail */
}

fn subtask_b() -> Result<(), FatalError> {
  /* perform work, maybe fail */
}

fn perform_work() -> Result<(), Error> {
  subtask_a()?;
  subtask_b()?;
  subtask_a()?;
  Ok(())
}

fn call_and_handle_local_error() -> Result<(), Error> {
  match perform_work() {
    Err(Error::Local(local_error)) => {
      /* handle error */
      Ok(())
    }
    other => other
  }
}

fn perform_work() -> Result<(), Error> {
  call_and_handle_local_error()?;
  call_and_handle_local_error()?;
  call_and_handle_local_error()
}
```

Ok, everything is alright. We're handling the local errors by performing
a partial pattern match, and having successes and fatal errors propagate
by handling it specifically in a particular place. But we all know how
code changes over time. At some point, somebody is going to write code
that looks like this:

```rust
fn subtask_a() -> Result<(), LocalError> {
  /* perform work, maybe fail */
}

fn subtask_b() -> Result<(), FatalError> {
  /* perform work, maybe fail */
}

fn perform_work() -> Result<(), Error> {
  subtask_a()?;
  subtask_b()?;
  subtask_a()?;
  Ok(())
}

fn call_and_handle_local_error() -> Result<(), Error> {
  match perform_work() {
    Err(Error::Local(local_error)) => {
      /* handle error */
      Ok(())
    }
    other => other
  }
}

fn perform_work() -> Result<(), Error> {
  call_and_handle_local_error()?;
  subtask_a()?;  <----- unhandled local error
  call_and_handle_local_error()?;
  subtask_b()?;
  call_and_handle_local_error()
}
```

The compiler won't even bat an eye. It might not fail for a long time. But the
local errors are not being handled, and a catastrophic system failure may be
possible now. This happens all the time in real code.

# case study: sled's compare and swap error

sled has an `Error` enum of its own. It can store various types of horrific
failures that you really want the user to be aware of, like if operations on
the backing file start to fail. We basically want to shut down the system
immediately to minimize the chance that data loss will happen without the user
being aware of it.

sled has a method that allows the user to atomically change the value of a key,
if they can correctly guess the current value. This primitive is quite common
in lock-free programming, and it forms the basis of many more complex
algorithms that we all rely on every day. In the past, it basically had this
signature:

```rust
fn compare_and_swap(
  &mut self,
  key: Key,
  old_value: Value,
  new_value: Value
) -> Result<(), sled::Error>
```

where the `Error` enum had a few different variants, and looked something like this:

```rust
enum Error {
  Io(std::io::Error),
  CompareAndSwap(CompareAndSwapError),
}
```

If you correctly guessed the previous value associated with the given key, sled would
atomically update the value to the new one you provided, and return `Ok(())`. If you
guessed the wrong old value, it would return `Err(sled::Error::CompareAndSwap(current_value))`.

However, it would also return an error if an IO issue was encountered at some point during
the execution of the operation.

The main problem was that these two error classes require completely different responses.
The IO error basically requires shutting down the system immediately until the problem
with the underlying system can be addressed. But the compare and swap is totally expected
to fail all the time. It's completely normal behavior. Unfortunately, users were no longer
able to rely on the try operator at all, because they had to actually do a partial
pattern match on the returned result object:

```rust
let result = sled.compare_and_swap(
  "dogs",
  "pickles",
  "catfood"
);
match result {
  Ok(()) => {},
  Err(sled::Error::Io(io_error)) =>
    return Err(io_error.into()),
  Err(sled::Error::CompareAndSwap(cas_error)) => {
    // handle expected issue
  }
}
```

And this was really gross. Additionally, inside the sled codebase, internal
systems were performing their own atomic CAS operations, and relying on the
same `Error` enum to signal success, expected failure, or fatal failure. It
made the codebase a nightmare to work with. Dozens and dozens of bugs happened over
years of development where the underlying issue boiled down to either accidentally
using the try `?` operator somewhere that a local error should have been handled,
or by performing a partial pattern match that included an over-optimistic wildcard
match. It was a challenging time.

# making bugs jump out

Over time, I developed several strategies for finding these bugs. The most
successful efforts that resulted in finding the most bugs boiled down to
randomly causing different operations to fail by triggering them through
PingCAP's [`fail` crate](https://docs.rs/fail) and [combining it with property
testing to cause various combinations of failures to be triggered under
test](https://github.com/spacejam/sled/blob/05de9a415c8794a775817fc1e1fd123e8ad20d84/tests/test_tree_failpoints.rs).
This kind of testing is among the highest bug:test code ratios that I've written
for sled so far.

[It triggered all kinds of deterministically-replayable bugs that would only
crop up under specific combinations of errors and high-level
operations](https://github.com/spacejam/sled/blob/05de9a415c8794a775817fc1e1fd123e8ad20d84/tests/test_tree_failpoints.rs#L479-L509).

The high level idea is that when you write a function that might fail,
add some conditionally-compiled logic that checks to see if it is
supposed to return an error instead of execute the happy path.
When you write tests, cause that injected failure to happen sometimes.
Maybe randomly, maybe in desired ways, whatever works best for you.
I got the most milage out of combining it with property testing
because I prefer to have machines write tests for me, while
I focus on making claims about what should happen.

But even just flipping a global static `AtomicBool` that is compiled during testing
that causes your potentially failing codepaths to intentionally fail sometimes
will cause so many bugs to jump out in your systems.

This caused many of the above bugs relating to the error
enum handling to jump out. But they kept getting introduced,
because it was difficult to always keep in my mind where
it might be possible for a compare and swap-related failure
to crop up, as there is a lot of lock-free conditional
logic in the sled codebase.

If you don't want to pull in an external crate that relies on an older version
of rand etc... the core functionality behind the `fail` crate is fairly simple,
and I've implemented a [simpler
internal](https://github.com/spacejam/sled/blob/05de9a415c8794a775817fc1e1fd123e8ad20d84/src/fail.rs)
version for sled to keep testing compile times shorter. Speedy tests = more tests
over time = less bugs.

The important thing: most catastrophic systems bugs exist in our error handling
code. It's not very much work to trigger that error handling logic in our
tests. You will find lots of important bugs as soon as you start manually
triggering these failure handling paths. Many bugs will usually be found in a
system the first time these kinds of simple tests are applied.

# making unhandled errors unrepresentable

Eventually this led me to go for what felt like the nuclear solution, but after
seeing how many bugs it immediately rooted out by simply refactoring the
codebase, I'm convinced that this is the only way to do error handling in
systems where we have multiple error handling concerns in Rust today.

That solution: make the global `Error` enum specifically only hold errors that
should cause the overall system to halt - reserved for situations that require
human intervention. Keep errors which relate to separate concerns in totally separate error types.
By keeping errors that must be handled separately in their own types, we reduce
the chance that the try `?` operator will accidentally push a local concern
into a caller that can't deal with it. If you make such errors representable,
they will happen. And they certainly have as I've been developing sled.

Today, the compare and swap operation has a signature that looks clunkier, but
it is far easier to use, and internally the system has become far more stable by
isolating concerns to their own types.

```rust
fn compare_and_swap(
  &mut self,
  key: Key,
  old_value: Value,
  new_value: Value
) -> Result<Result<(), CompareAndSwapError>, sled::Error>
```

While on first glance this looks way less cute, it significantly
improves the chances that users will properly handle their
compare and swap-related errors properly:

```rust
// we can actually use try `?` now
let cas_result = sled.compare_and_swap(
  "dogs",
  "pickles",
  "catfood"
)?;

if let Err(cas_error) = cas_result {
    // handle expected issue
}
```

By using nested `Result`s, we allow ourselves to take advantage of the
wonderful short-circuit and error propagation properties of the try `?`
operator, without exposing ourselves to an endless deluge of bugs relating to
mashing all of our errors into a gross single type. We are able to rely on the
beautiful bug-reducing capabilities of performing exhaustive pattern matching
again, without making guesses about which variants may or may not need to be
handled at specific places. By separating error types, we increase the chances
that we correctly handle errors at all.

Use try `?` for propagating errors. Use exhaustive pattern matching on concerns
you need to handle. Do not implement conversions from local concerns into
global enums, or your local concerns will find themselves in inappropriate
places over time. Using separate types will lock them out of where they don't
belong.

# in summary

There is a strong tendency in the Rust community to throw all of our errors
into a single global error enum. This makes usage of the try `?` operator
bug-prone, and it increases the chances that we will allow a local error to
slip through and propagate to a caller that is not capable of handling it.

By keeping errors that must be handled in separate ways in separate types, we
make this confusion impossible to represent. The price we pay is a signature
that involves nested `Result`s, which definitely makes the code look less cute,
but for maintaining a real project, cute doesn't count for much compared to
spending less time on bugs that could have been avoided.

We can easily test our failure handling logic
by using tools like PingCAP's [`fail` crate](https://docs.rs/fail).

Let's end with the same two
[quotes](http://www.eecg.toronto.edu/~yuan/papers/failure_analysis_osdi14.pdf)
that we began with:

```
almost all (92%) of the catastrophic system failures
are the result of incorrect handling of non-fatal errors
explicitly signaled in software.
```

```
in 58% of the catastrophic failures, the underlying
faults could easily have been detected through simple
testing of error handling code.
```

Thanks for reading!

If you found this article to be useful, please consider [supporting my
efforts](https://github.com/sponsors/spacejam) to share knowledge and
productionize cutting edge database research with implementations in Rust :)
