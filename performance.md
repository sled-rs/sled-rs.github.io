# sled performance guide

[![github](https://img.shields.io/github/stars/spacejam/sled.svg?style=social)](https://github.com/spacejam/sled)
[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![sponsors](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

## contents

* [overview](#overview)
* [experimental design](#experimental-design)
* [rust](#rust)
* [cpus](#cpus)
  * [frequency scaling](#frequency-scaling)
* [memory](#memory)
* [threads](#threads)
* [async tasks](#async-tasks)
* [syscalls](#syscalls)
* [USE Method](#use-method)
* [universal scalability law](#universal-scalability-law)
* [queue theory](#queue-theory)
* [flamegraphs](#flamegraphs)
* [cachegrind](#cachegrind)
* [massif](#massif)
* [dhat](#dhat)


## overview

This guide showcases some basic information for getting
started with performance-sensitive engineering work.

It is hoped that this will provide enough background
to be successful in optimizing the sled database when
suboptimal behavior is discovered.

These materials are extracted from Tyler Neely's
Rust workshops.

## experimental design

We seek to make sled faster.

sled may be faster if:
* your web browser is closed
* your laptop is plugged in
* you run it on a larger machine
* you run it on a machine with [frequency scaling](#frequency-scaling) disabled with a custom kernel

Many factors influence our measurements. Is your
web browser running? Is your laptop plugged in?
Did you just

Further reading: Quantitative Analysis of Computer Systems by Clement Leung.

## rust

Rust's borrowing rules ensure that there will only exist
a single mutable reference to some memory at a time.

As this is taken advantage of, it allows the Rust compiler
to approach Fortran-level performance (much faster than
C/C++ in many cases).

See [rust/54878](https://github.com/rust-lang/rust/issues/54878)
for the current status of the effort to support this. It's a big
deal. There's a reason we still use Fortran libraries in much of
our linear algebra (and implicitly, our machine learning) libraries.


## cpus

CPUs combine instructions and data.
The rest of this section assumes x86_64.

### frequency scaling

The first thing to know about real CPUs is that
they constantly shift their frequencies to use
less power and generate less heat while meeting demand.
This has major implications for measurements. Many people
run a workload, record some latency stats, make a change,
run the workload again, and record the new stats. It is
a mistake to assume that the delta between the two
measurements is explained by whatever code changed
in-between. Often, changes that may cause the compiler
to spend more effort performing optimizations will
cause frequency scaling to kick in to a greater extent
before the workload under measurement gets a chance
to run, causing the CPU to run the new workload at
a diminished frequency, and making it appear to
perform worse.

Frequency scaling must be accounted for in your performance
analysis. We must take multiple measurements.

Also, we must be careful about the residual effects of compilation.
Rust has a lot of nice compilation options that will
trade more compilation time for faster runtime. When the compiler
works harder, it can often cause the CPU to scale down more
aggressively to account for the heat being generated,
and it will make it seem like a workload is slower
even though it is much faster, but more heavily throttled.

Bad:

```
* time compile and run workload 1
* time compile and run workload 2
* compare total times
```

Better:

```
* compile workload 1
* compile workload 2
* cooldown
* time workload 1
* time workload 2
* time workload 1
* time workload 2
...
* time workload 1
* time workload 2
* view distribution of results
```

If you have an intel CPU, you can use the `i7z` command,
to see what your cores are currently doing. It is
 available in most linux package managers.

```
Cpu speed from cpuinfo 1607.00Mhz

True Frequency (without accounting Turbo) 1607 MHz
  CPU Multiplier 16x || Bus clock frequency (BCLK) 100.44 MHz

Socket [0] - [physical cores=6, logical cores=12, max online cores ever=6]
  TURBO ENABLED on 6 Cores, Hyper Threading ON
  Max Frequency without considering Turbo 1707.44 MHz (100.44 x [17])
  Max TURBO Multiplier (if Enabled) with 1/2/3/4/5/6 Cores is  47x/47x/41x/41x/39x/39x
  Real Current Frequency 1257.57 MHz [100.44 x 12.52] (Max of below)
        Core [core-id]  :Actual Freq (Mult.)      C0%   Halt(C1)%  C3 %   C6 %  Temp      VCore
        Core 1 [0]:       1122.43 (11.18x)      3.61    76.4      21       0    36      0.6615
        Core 2 [1]:       1102.71 (10.98x)      1.11    99.2       0       0    36      0.6613
        Core 3 [2]:       1257.57 (12.52x)      5.05      96       0       0    35      0.6666
        Core 4 [3]:       1098.65 (10.94x)      2.13    98.5       0       0    36      0.6616
        Core 5 [4]:       1053.30 (10.49x)         1    99.5       0       0    37      1.0566
        Core 6 [5]:       947.05 (9.43x)           1    99.5       0       0    36      1.0566

C0 = Processor running without halting
C1 = Processor running with halts (States >C0 are power saver modes with cores idling)
C3 = Cores running with PLL turned off and core cache turned off
C6, C7 = Everything in C3 + core state saved to last level cache, C7 is deeper than C6
  Above values in table are in percentage over the last 1 sec
```

###


## memory
### numa
## threads
## syscalls
## filesystems
## disks
## networks
## USE Method
## universal scalability law
## queue theory

Further reading: Quantitative Analysis of Computer Systems by Clement Leung.

## flamegraphs
## cachegrind
## massif
## dhat
