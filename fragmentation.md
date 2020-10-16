by Tyler Neely on October 16 2020

[![github](https://img.shields.io/github/stars/spacejam/sled.svg?style=social)](https://github.com/spacejam/sled)
[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![sponsors](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

## Complexity and Fragmentation

I think one of the many reasons why Golang can feel nice to use while collaborating
with others is because there aren't all that many features that should be avoided
in most situations, as is the case with C++, Haskell or Rust. Complexity
really causes a programming language to fragment.

Scala is a great example of this. From a distance, it seemed quite familiar to
Java users who were able to use all of their favorite libraries from the
expansive JVM ecosystem while cutting away large amounts of boilerplate. In
some ways it let them write cleaner Java. But one reason for the language to
exist was to provide a platform for programming language research, and it
supports a large number of advanced features such as the mythical
"higher-kinded types" which allows for the expression of a wide variety of
[interesting and complex language
patterns](https://github.com/lemastero/scala_typeclassopedia#category-theory).

Scala soon became the language that Haskell programmers who "wanted jobs" would
use at work, where they immediately began to translate their holy text, the
[Typeclassopedia](https://wiki.haskell.org/Typeclassopedia), into their work
projects in Scala. Understandably, this led to a bitter rift between the Java
and Haskell camps, as both sides completely hated the way that the other camp
wrote their code. The two camps were then forced into shared confinement when
Twitter's Finagle service framework gained popularity, forcing users to puzzle
over long confusing error messages stemming from use of futures. There ended up
being 3 major, incompatible versions of futures between Finagle, the Akka
actor-based library, and the standard library's own actor system. It was not
uncommon for codebases to contain a variety of coding styles and modules using
incompatible futures frameworks. To run a Scala project was to live in a
perpetual war of enforcing coding guidelines.

Eventually, Rust decided to use Finagle as the inspiration for its futures
approach. I see a lot of parallels between the Rust async history and the Scala
async history that it was based on, especially in terms of this friction
between camps. I'm anticipating a similar Typeclassopedia-style split to happen
if Rust gets GATs, which are isomorphic to HKTs.

