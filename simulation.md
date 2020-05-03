# sled simulation guide

[![github](https://img.shields.io/github/stars/spacejam/sled.svg?style=social)](https://github.com/spacejam/sled)
[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![sponsors](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

## overview

This guide contains basic information about deterministic testing.


Step 1: build a simulator. Anyone who doesn’t do this is building a very buggy system.

After you have the experience of building your first distributed system on top of a simulator that induces partitions / delays etc… you will forever consider writing them without a simulator to be like driving drunk and blindfolded. So many bugs will pop out. Our laptops don’t behave like networked systems. If you built a plane in windtunnel with zero induced turbulence effects, would you then fly that plane? Because that’s how people are building the distributed systems you use today, and fixes only happen occasionally when someone is lucky enough to realize there’s even anything wrong. They would save so much time debugging high-cost issues in production if they had tested the thing with a simulator.

But it’s not complex to build a simulator. Much simpler than the subtleties of CRDT convergence with dynamic membership or raft with a single partition that causes dueling candidacies etc… And if you write a few simple tests, like “is the sequence of observed client responses actually possible to observe with any combination of client requests (possibly dropping ones where responses were not observed) executed serially?” which gives you a straightforward linearizability test. You can write simpler invariants that get executed after each message is delivered and processed, like “are there more than 1 leaders that can convince a majority of the cluster to do as they say?” (split brain).

Simulators give you implementations that Jepsen will not find bugs in, at least as far as the core state machine for your distributed algorithms is concerned. It takes 5 minutes to run 1 jepsen test on a cluster, usually after spending 1 month to implement jepsen for the system, or paying someone hundreds of thousands of dollars to do it for you. You can run thousands of interesting workloads per second on a laptop, stretching the possible delivery horizons for messages way farther, and finding far more bugs.

You can choose your timing assumption model, but the simplest to implement, and also the one that guarantees the freedom from the most bugs in the wild, is the asynchronous model, where any message can be arbitrarily delayed (maybe forever, dropped) or reordered with others.

This is one possible interface that has worked well for me:

```rust
// called when this state machine receives a message,
// responds with outgoing messages
fn receive(msg, at) -> [(msg, destination)]

// periodically called for handling periodic functionality,
// like leader election etc...
fn tick(at) -> [(msg, destination)]
```

Recipe:

1. write your algorithm around a state machine that receives messages from other nodes, and responds with the set of outgoing messages. this can be easily run on the simulator in testing, and on top of a real tcp/whatever transport in production. if you have things like periodic leader elections etc… you can implement a tick method also that occasionally gets called. having this clear mapping from input to output is an amazing advantage for taming the complexity of your system, and gives you lots of wonderful introspection opportunities as well if you’re using tracing or anything like that. having total separation between IO and algorithmic concerns will allow you to be much more flexible with how your nodes communicate with each other over time, and will make integration costs lower as well.
1. randomly generate a set of client requests that happen at a specific time to “seed” the cluster, optionally also seed it with ticks etc… you can be creative here based on what your system actually is doing
1. stuff all messages / events in the system into a priority queue keyed on next delivery time
1. iterate over the priority queue, delivering messages to the intended state machine
1. for each outgoing message in the set that the state machine generated in response, deterministically assign a “delivery time” to the message (or drop it). insert each scheduled message into the priority queue
1. iterate over the priority queue until empty / some other budget
1. for each observed client request -> response pair, make assertions about validity of that observed response. Did a write request observe a successful response before a read request, but that read request returned the old value? This will be highly specific to your system, but will save you time by specifying in code.

This general pattern is called “discrete event simulation”. If you’re coming into distributed systems, if you learn this technique, you will have a massive advantage over anyone who claims to be a distributed systems expert but just tests in production / their laptop / jepsen.
