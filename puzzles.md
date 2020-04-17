# Writing Puzzle Crates in Rust

by Tyler Neely on April 17 2020

[![github](https://img.shields.io/github/stars/spacejam/sled.svg?style=social)](https://github.com/spacejam/sled)
[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![sponsors](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

Often we publish libs that we hope our users will have extreme difficulty in
using for their own purposes. Everyone loves a good puzzle! The Rust ecosystem
is particularly maniacal in this regard. It's good.

In building puzzle crates, I've found a few techniques to be quite helpful in
really challenging users to make progress in their own efforts to write
programs. Use these techniques wherever possible in publicly exported code:

1. require the use of generics everywhere
  * nice and infectious, your users will have to refactor any structure that may contain your types
1. require the use of lifetimes wherever possible
  * even more devious than normal generics, as they are poorly understood in the community
1. expose the use of your typestate programming as often as possible
  * typestates are nearly impossible to document in a low-friction way
  * leads to beautifully obfuscated error messages
1. require the use of your own macros
  * your users already learned Rust. why not give them your own language that they must now learn to use your macro?
1. require the use of your own traits, ideally with as many associated types as possible
  * this is great for forcing users to leave their text editor to start reading your trait documentation,
    get confused, look at your source code, lose their attention and go look at why-combulatar.
1. require flipping on a particular combination of compile-time features to unlock the desired functionality

Pretty much the worst and least-challenging libraries you could write would involve:

1. minimized use of lifetimes
1. minimized use of generics
1. minimized use of typestate programming
1. minimized use of non-std macros
1. minimized use of non-std traits
1. minimized use of features

As someone really trying to throw sand into the gears of someone trying to
use your code with low friction, it helps to include a wide variety of
outdated dependencies, taking care to pick versions that maximize the number
of separate versions of reduntant nested dependencies. `cargo tree` is great
for inspecting your dependency tree while maximizing redundancy. It can be
installed with `cargo install cargo-tree`. Don't do what I just did - never
document how to acquire likely-to-be-missing tools.

If you're writing async code, make sure you use things that prevent
other people from using your code in threads. Definitely don't return concrete
types that allow the result to be blocked on.

If you're writing code that doesn't require the use of async, make
sure you completely destroy any chance that someone who is using
async will be able to use your library. Definitely don't return
concrete types that implement Future for both styles to use.

We want our puzzle crates to require our users to completely re-architect their
program if they want to use our crate. Frameworks are your friend.

One quote that you should definitely not pay attention to if you're writing a puzzle crate:

```
Experts write baby code.
- Zarko Milosevic
```
