# Writing APIs in Rust

by Tyler Neely on May 3 2020

[![github](https://img.shields.io/github/stars/spacejam/sled.svg?style=social)](https://github.com/spacejam/sled)
[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![sponsors](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

Often we publish libraries that we hope may solve problems for users. Bringing
in a new library to hopefully solve a problem may involve many costs. Often
these costs are higher than the value of the library itself. We can take
concrete steps to minimize these costs.

Additionally, by minimizing costs for users, you often improve your own experience as an author.

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
1. don't be a framework unless you actually need to create an entire ecosystem
  that is simply incompatible with the standard forms of computation, such as
  stackful coroutines or something.

```
Experts write baby code.
- Zarko Milosevic
```
