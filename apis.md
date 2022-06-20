# Writing APIs in Rust

by Tyler Neely on May 3 2020, updated January 26 2022

[![github](https://img.shields.io/github/stars/spacejam/sled.svg?style=social)](https://github.com/spacejam/sled)
[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![sponsors](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

Often we publish libraries that we hope may solve problems for users. Bringing
in a new library to hopefully solve a problem may involve many costs. Often
these costs are higher than the value of the library itself. We can take
concrete steps to minimize these costs.

The virtues we should strive for:
* easy to understand for beginners without needing to traverse documentation beyond looking at provided function signatures on docs.rs.
* composes well with plain functions and data structures - old code should play nicely with new code if it will retain its value.
* easy to debug - high quality infrastructure often requires 3-4x the time on debugging than the time it takes to write features. While using techniques like [fuzzing](https://fuzzcheck.neocities.org/tutorial1_function.html), [model-based testing](https://medium.com/@tylerneely/reliable-systems-series-model-based-property-testing-e89a433b360), [fault injection](https://docs.rs/fail/latest/fail/) and [simulation](https://sled.rs/simulation) can make it significantly cheaper to write high quality infrastructure, debugging will still take most of the time budget over time. Even when someone is focused on writing new features, the majority of their time is spent interacting with the compiler until it works. This process should be optimized for through API design decisions that guide users toward correct usage, and providing high quality information for them when normal development cycle breakage occurs.
* quick to compile so we have a lower chance of getting distracted between runs

Additionally, by minimizing costs for users, you often improve your own experience as an author.

## General Guidelines

1. minimize use of generics
  * infectious, your users will have to refactor any structure that may contain your types
  * removing generics improves compile times
  * removing generics makes code easier to reason about. even if you think you
    really understand something it will still feel lighter if you make it
    non-generic for a single real use case. ripping out viral generic parameters
    in containing types feels amazing.
1. minimize use of lifetimes
  * even less ergonomic than normal generics, as they are poorly understood by many Rust programmers
1. consider the friction of typestate programming in non-correctness-critical components
  * typestates can be effective for imposing well-typed state transitions
  * typestates may be challenging to document in a clear way
  * error messages encountered due to a violated typestate often lack specificity
  * take care to minimize user friction when using typestates in public interfaces
1. minimize use of non-std macros
  * every macro is its own DSL that users must learn.
1. minimize use of non-std traits
  * traits are not as bad as macros, but they still force users to take their minds off their code to learn your trait
1. minimize use of proc macros
  * while proc macros enable a lot of cuteness, that cuteness tends to compose extremely poorly, increases documentation effort, increases debugging effort, tends to break caching and generally make compile times explode.
1. minimize use of conditional compilation
  * difficult to document, difficult to test
1. be considerate of the dependency tree that you force on your users
  * `cargo tree` is great for guiding pruning efforts
    * can be installed with `cargo install cargo-tree`
  * nobody wants to build 50 more dependencies when they are trying to solve a simple problem with your crate
1. don't use async unless it makes something measurable better
  * I have never found this to be true for any of my work on distributed systems or databases.
  * It reduces composability, increases debugging costs, significantly increases reliance on non-std code for trivial functionality which in-turn exacerbates compilation latency.
1. be cautious about const generics in situations where you are unable to fuzz confidence-inspiring ranges
  * const parameters must be concrete at compile-time, making it high-friction to fuzz interesting values for them.
1. don't be a framework unless you actually need to create an entire ecosystem
  that is simply incompatible with the standard forms of computation, such as
  stackful coroutines or something.
1. be extremely cautious of any subcommunity that is obsessed with solving problems with their own stuff rather than pre-existing problems that their stuff makes easier to solve.

```
Experts write baby code.
- Zarko Milosevic
```
