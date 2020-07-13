* ##### CHAPTER 0b0010: THE MACHINE
  * [computation](#computation)
  * [threads](#threads)
  * [syscalls](#syscalls)
  * [flash storage](#flash-storage)
  * [network stacks](#network-stacks)
  * [cache](#cache)
  * [frequency scaling](#frequency-scaling)
  * [branch misprediction](#branch-misprediction)
  * [branch target misprediction](#branch-target-misprediction)
  * [4k aliasing](#4k-aliasing)
  * [bandwidth saturation](#bandwidth-saturation)
  * [cache conflicts](#cache-conflicts)
  * [cache/memory hierarchy bandwidth](#cache/memory-hierarchy-bandwidth)
  * [data dependencies](#data-dependencies)
  * [denormal floating point numbers](#denormal-floating-point-numbers)
  * [DRAM refresh interval](#DRAM-refresh-interval)
  * [false sharing](#false-sharing)
  * [hardware prefetching](#hardware-prefetching)
  * [memory-bound program](#memory-bound-program)
  * [misaligned accesses](#misaligned-accesses)
  * [non-temporal stores](#non-temporal-stores)
  * [software prefetching](#software-prefetching)
  * [store buffer capacity](#store-buffer-capacity)
  * [write combining](#write-combining)
* ##### CHAPTER 0b011: ANAL TOOLZ
  * [visualizing time with flamegraphs](#flamegraphs)
  * [modeling CPU behavior with llvm-mca](#llvm-mca)
  * [determining viable optimizations with coz](#coz)
  * [allocation lifetime analysis with dhat](#dhat)
  * [heap profiling with massif](#massif)
  * [top-down analysis with toplev](#top-down-analysis)
  * [CPU cache simulation with cachegrind](#cachegrind)
* ##### CHAPTER 0b100: TECHNIQUE
  * data structure design
  * critical path analysis
  * concurrency
  * parallelism
  * batching
  * flat-combining
* ##### CHAPTER 0b101: RUST
  * allocation analysis with the `GlobalAlloc` trait
  * [async tasks](#async-tasks)

# CHAPTER 0b0010: THE MACHINE

The goal of this chapter is for readers to walk away feeling dizzy yet
familiar with how our machines work at a high level.

## computation

Human-readable code is translated into instructions and data that the CPU will
zip together while executing your program.

When a program executes, it refers
to other memory locations that contain more instructions and data for the CPU
to fetch, interpret, and combine. Fetching instructions and data from main
memory takes a really long time, and by being considerate of how the CPU
fetches memory and publishes changes, you can sometimes make your code several
orders of magnitude faster.

Modern CPUs often execute 4 instructions per cycle, and execute over 3 billion
cycles per second. The time that it takes to retrieve instructions or data from
main memory is around 65 nanoseconds (or 105 to get memory from a different NUMA
node).

That means we can execute 780 instructions in the time that it takes to pull a
single instruction or byte of data from main memory.

This is why we have several levels of caches in front of the main memory. It
lets us have quick access to memory that the hardware believes we may need
to use again.

Lots of people coming to Rust have become a bit fixated on trying to minimize
the number of instructions that their programs need. But this usually doesn't
matter so much compared to cache performance. Intel will sometimes recommend
high performance sorting algorithms that look like they should be terrible
because of their high instruction counts, but because they play nicely with
prefetchers and branch predictors, the cache is much better utilized, and the
overall algorithm runs faster than lower instruction sorting algorithms that pay
less attention to hardware friendliness.

## threads

## syscalls

In a sense, our programs are essentially just DSL's for orchestrating syscalls.
Kernels do most of the interesting heavy-lifting when it comes to IO,
and it's our job as authors of programs that run on top of kernels to stay
out of the kernel's way as much as possible.

http://www.brendangregg.com/blog/2018-02-09/kpti-kaiser-meltdown-performance.html

## flash storage

Our new flash-based storage devices behave differently from the spinning
disks of the past. Instead of needing to drag a physical spindle from one location
to another (taking several milliseconds to get there) our flash storage
has no moving parts. Every 32mb or so will end up on a different
chip inside the drive. Data is

## network stacks

## cache

https://mechanical-sympathy.blogspot.com/2013/08/lock-based-vs-lock-free-concurrent.html
* graphs showing lock-free is fast

https://www.real-logic.co.uk/training.html
* syllabus for concurrent programming

https://mechanical-sympathy.blogspot.com/2011/08/inter-thread-latency.html
* avoiding memory barriers via batching

https://mechanical-sympathy.blogspot.com/2011/07/write-combining.html
* memory buffers are like unchained hash maps w/ 64-byte buckets (cachelines)
* cachelines have 64-bit bitmap that records dirtiness
* cacheline is the unit of memory transfer
* evicting the previous tenant causes write-back, maybe all the way to dram
* storing data means writing to L1, but if it's not there, need to do RFO
* when going to L2, the CPU performs a request for ownership
* until the RFO completes, the CPU stores the item to be written in a
  cacheline-sized buffer among the "line fill buffers"
* these buffers hold speculative stores until the cacheline can be acquired
* the biggest benefit happens here when the longest delays happen: getting from DRAM
* if multiple writes happen to the same cacheline, they can happen on the same buffer
* reads will also snoop this buffer first
* on some intel chips, there are 4 line fill buffers (on my laptop this is 8, see code)
* this means that if we write to more than 4 separate cachelines in a loop, it slows down
* with hyperthreading, there is more competition for these same buffers


https://mechanical-sympathy.blogspot.com/2012/08/memory-access-patterns-are-important.html
* cache access latencies are ~1ns, ~4ns, ~15ns
* caches are effectively hash tables with a fixed number of slots for each hash value
  * called "ways", 8-way = 8-slots per hash value
* these each store 64 bytes, pulled in adjacently
* memory gets evicted in LRU order
* on eviction, memory gets written back, possibly all the way back to dram
* each level of cache includes TLB mappings for virtual memory, 4k or 2mb pages
* prefetching tends to access cache lines when accessed 2kb or less fixed stride apart
* when we hit DRAM, memory is arranged in rows that are 4k (a page) wide. the entire
  page is loaded into a row buffer. it has a queue that reorders requests to
  the same page so that they can share the work of pulling a page into a row
  buffer.
* with NUMA, each hop adds 20ns to access times. on an 8-socket system, memory
  may be 3 hops away.

https://mechanical-sympathy.blogspot.com/2011/09/single-writer-principle.html
* optimism can cause effective queuing effects just like locking
* managing contention vs real work
* message passing and letting threads do work without memory barriers is nice

Incrementing a 64-bit counter 500 million times using a variety of techniques on my laptop with an i7-10710U:

<table style="width:100%">
  <tr>
    <td> method </td>
    <td> time(ms) </td>
  </tr>
  <tr>
    <td>one thread, write_volatile</td>
    <td>130</td>
  </tr>
  <tr>
    <td>one thread + Release memory barrier</td>
    <td>130</td>
  </tr>
  <tr>
    <td>one thread + SeqCst memory barrier</td>
    <td>5,500</td>
  </tr>
  <tr>
    <td>one thread with CAS</td>
    <td>3,200</td>
  </tr>
  <tr>
    <td>two threads with Relaxed CAS</td>
    <td>12,200</td>
  </tr>
  <tr>
    <td>two threads with SeqCst CAS</td>
    <td>12,400</td>
  </tr>
  <tr>
    <td>one thread with lock</td>
    <td>9,000</td>
  </tr>
  <tr>
    <td>two threads with a lock</td>
    <td>65,000</td>
  </tr>
</table>

https://mechanical-sympathy.blogspot.com/2011/07/memory-barriersfences.html
* each cpu core has 6 execution units that can execute instructions in parallel
* execution units access registers
* execution units read from load buffers and write to store buffers
* load and store buffers interact with the L1 cache
* store buffers feed into the write combining buffer
* the write combining buffer feeds into the L2 cache
* the load and store buffers can be read from efficiently
* reads will access the buffers first, avoiding cache if possible
* barriers guarantee the visibility ordering to other cores
* barriers propagate data in-order to the cache subsystem
* store barriers push all data into the L1 cache
* load barriers wait for all in-progress loads to complete before the next loads happen
* full barriers combine load and store barriers
* it is better to "batch" ordered work as much as possible to reduce barrier overhead

https://mechanical-sympathy.blogspot.com/2011/07/false-sharing.html
* to modify memory, your core needs to acquire exclusive access for the cacheline
* this may involve going through the l3 cache (or worse) to invalidate the previous owner
* multiple pieces of data may share the same cache line, which makes it impossible
  to make progress in parallel
* good graph showing false sharing [REPLICATE THIS IN RUST]

https://mechanical-sympathy.blogspot.com/2013/02/cpu-cache-flushing-fallacy.html
* MESIF - to write, a RFO must happen that invalidates other copies
* cache coherency traffic is on its own bus
* cache controller is a module on each L3 cache segment
  * connected to on-socket ring-bus network
  * sockets are connected to each other via this ring-bus network as well
  * everything shares this ring-bus network
    * cores
    * L3 cache segments
    * QPI/hypertransport controller (links sockets)
    * memory controller
    * integrated graphics subsystem
  * the ring-bus network has 4 lanes
    * request
    * snoop
    * acknowledge
    * 32-bytes data per cycle
* l3 cache is inclusive of l1/l2
  * facilitates identification of which core has copies of which cachelines
* read request from a core goes to the ring bus
  * will read from main memory if uncached
  * will read from l3 if clean
  * will snoop from another core if modified
  * the returned read will never be stale
* TLB may need to be flushed depending on the address indexing policy on a context switch

https://stackoverflow.com/questions/54876208/size-of-store-buffers-on-intel-hardware-what-exactly-is-a-store-buffer/54880249#54880249
https://nicknash.me/2018/04/07/speculating-about-store-buffer-capacity/
https://preshing.com/20120930/weak-vs-strong-memory-models/
http://www.1024cores.net/home/parallel-computing/cache-oblivious-algorithms


This list has been extracted from [Kobzol's wonderful hardware effects GitHub repo](https://github.com/Kobzol/hardware-effects).
[Ben Titzer - What Spectre Means for Language Implementors](https://www.youtube.com/watch?v=FGX-KD5Nh2g)

Further reading:

* https://mechanical-sympathy.blogspot.com/2013/02/cpu-cache-flushing-fallacy.html
* https://mechanical-sympathy.blogspot.com/2011/07/memory-barriersfences.html
* https://bartoszmilewski.com/2008/11/05/who-ordered-memory-fences-on-an-x86/
* https://www.scylladb.com/2017/07/06/scyllas-approach-improve-performance-cpu-bound-workloads/
* https://www.nickwilcox.com/blog/arm_vs_x86_memory_model/

## frequency scaling

CPUs constantly shift their frequencies to use less power and generate less
heat while meeting demand. This has major implications for measurements. Many
people run a workload, record some latency stats, make a change, run the
workload again, and record the new stats. It is a mistake to assume that the
delta between the two measurements is explained by whatever code changed
in-between. Often, changes that may cause the compiler to spend more effort
performing optimizations will cause frequency scaling to kick in to a greater
extent before the workload under measurement gets a chance to run, causing the
CPU to run the new workload at a diminished frequency, and making it appear to
perform worse.

Frequency scaling must be accounted for in your performance analysis. We must
take multiple measurements.

Also, we must be careful about the residual effects of compilation. Rust has a
lot of nice compilation options that will trade more compilation time for faster
runtime. When the compiler works harder, it can often cause the CPU to scale
down more aggressively to account for the heat being generated, and it will make
it seem like a workload is slower even though it is much faster, but more
heavily throttled.

If you have an Intel CPU, you can use the `i7z` command, to see what your cores
are currently doing. It is available in most Linux package managers.

## branch misprediction
## branch target misprediction
## 4k aliasing
## bandwidth saturation
## cache conflicts
## cache/memory hierarchy bandwidth
## data dependencies
## denormal floating point numbers
## DRAM refresh interval
## false sharing
## hardware prefetching
## memory-bound program
## misaligned accesses
## non-temporal stores
## software prefetching
## store buffer capacity
## write combining

# CHAPTER 0b101: Rust specifics

```
It is hard to free fools
from the chains they revere.

- Voltaire
```

Rust's borrowing rules ensure that there will only exist a single mutable
reference to some memory at a time.

As this is taken advantage of, it allows the Rust compiler to approach
Fortran-level performance (much faster than C/C++ in many cases).

See [rust/54878](https://github.com/rust-lang/rust/issues/54878) for the current
status of the effort to support this. It's a big deal. There's a reason we still
use Fortran libraries in much of our linear algebra (and implicitly, our machine
learning) libraries.

[Cheap tricks for high-performance Rust](https://deterministic.space/high-performance-rust.html)

## async tasks

Rust's async tasks are a simple mechanism for describing blocking behavior. By
using `async` blocks and functions, we can describe state machines that will
block on certain dependencies while executing. Within an
`async` block, we may use the `await` feature to suspend execution of the state
machine until progress may be made. Most magic-feeling functionality in Rust
happens because a particular trait was implemented, and async code is no different.
In Rust, asynchronicity relies on the `Future` trait. `async` functions and blocks
compile into objects which implement the `Future` trait.

The `Future` trait has one method: `poll`:

```rust
pub trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}
```

A `Future` expects to have `poll` called by an "executor" which provides a
`Context` object. The `Context` object provides a way for the polled future to
register a callback that will be notified when it is ready to be polled again,
if it is not already `Ready`.

In the section on the [universal scalability law](#universal-scalability-law)
we discussed how concurrency often undermines possible performance gains of
parallelization. Looking at [a minimal
executor](http://github.com/spacejam/extreme), we can see a number of potential
sources for contention and coherency costs which will drag down our scalability
curve if we are not careful.

There is currently no way to tell a `Context` / `Waker` that your task is
blocked due to a particular kind of event, which makes using this interface
challenging when an author desires to implement priority-based scheduling
techniques like the ones [discussed in the scheduling section
above](#scheduling). If this interface is used while builidng such a scheduler,
one must feed information out-of-band via something like thread local
variables.

Memory usage is often cited as a benefit for using async tasks, however,
it's quite easy for these to balloon to several megabytes in production
workloads, because in a sense, it's the most pessimistic stack possible.
Make sure you use memory profilers to determine the actual memory use
of your systems, and perform responsible capacity planning to ensure
that you are not just assuming that it will be low-memory due to
using async tasks instead of threads.
[Tyler Mandry from Google touches on this at his talk at RustFest Barcelona](https://www.youtube.com/watch?v=ZHP9sUqB3Qs).

Async tasks are generally a great fit for then you must block many tasks that
perform very little computation, such as when you build a load balancer. Only
then do context switches become measurable compared to the actual workload
being scheduled. This effect is often distorted by the way that people tend to
run microbenchmarks which don’t perform computation and memory accesses in ways
that match what they are likely to encounter in production, making it seem like
the proportion of CPU budget consumed by context switches is large, but for
anything de/serializing json, the context switch is often noise in comparison.
Measure performance of realistic workloads on realistic hardware.

```
Everybody is ignorant, only on different subjects
```
- Will Rogers

```
It's never what we don't know that stops us. It's what we do know that just ain't so.
```
- Mark Twain

```
Micro-benchmarks are like a microscope.
Magnification is high, but what the
heck are you looking at?
```
- Cliff Click

https://assets.azul.com/files/Cliff_Click_Art_of_Java_Benchmarking.pdf

[Five ways not to fool yourself](https://timharris.uk/misc/five-ways.pdf)
  * test algorithms while being smallest possible as well as clearly overflowing caches
  * be aware of runaway unfairness issues in concurrent code due to caching etc...
  * ideally, keep your correctness assertions on everywhere, even under perf analysis. don't measure broken code.
  * the main risk is that variance is more significant than actual effects
  * nail down the experimental environment
    * pin threads to cpus
    * pin memory to sockets
    * hyperthreading is disabled
    * power management features disabled
    * double check the socket that owns important memory using libnuma etc...
  * establish baseline overhead vs non-concurrent version
  * expect speedup to be linear with each thread, if not, why not?
  * link these 3 things to increase chances of causal link:
    * timing or throughput info
    * resource usage: instructions, memory use, last-level cache misses, etc...
    * actual code changes
  * before making something faster, duplicate the operation you want to optimize, see if it makes measurements different
  * know what your system limits are, be aware of what the code is doing, and make sure you trade-off away from scarce resources where possible
  * after we tune, start putting the system back to the production state without pinning, power management, etc...

- criticism
  * a lot of time may be spent setting up an unrealistic environment
    frequency scaling needs to be accounted for because it can

https://randomascii.wordpress.com/2018/02/04/what-we-talk-about-when-we-talk-about-performance/
  * "90% faster" and "90% speed-up" etc... can easily be misinterpreted
  * use ratios instead of performance

A guide to experimental algorithmics - Catherine C McGeoch
  terms:
  * experimental algorithmics - the study of algorithms (and programs, data
    structures, etc...) through experimentation

  analysis:
    performance predictions about inputs and machines in terms of time, space, quality, etc...
  design:
    building faster and better algorithms (structures, programs, etc...)

[Brief announcement: persistent unfairness arising from cache residency imbalance](https://dl.acm.org/doi/10.1145/2612669.2612703)
  * concurrent algorithms often become extremely unfair, favoring the winner of the last operation,
    making their caches hotter and allowing them to increase the unfairness

my techniques:
* gaze into the future. before optimizing at all, try to just comment out the
  code and see how fast the system would be if the optimization were infinitely
  successful. don't bother optimizing things that can't meaningfully improve your
  metrics even if improved infinitely. This also gives you a great sense of
  priorities by knowing proportionally what aspects of your system are really
  slow.
* "if you didn't optimize it, it ain't optimized"

[COZ:  Finding  Code  that  Counts  with  Causal  Profiling](https://arxiv.org/pdf/1608.03676v1.pdf)
* inserts delays, causing potential relative speedups to be illuminated

https://github.com/alexcrichton/coz-rs

anti-normative-positivism
  positivistLc tendencies due to engineering education - 2003

The Art of Multiprocessor Programming by Maurice Herlihy and Nir Shavit


https://doc.rust-lang.org/stable/std/alloc/trait.GlobalAlloc.html

[Thread State Analysis](http://www.brendangregg.com/tsamethod.html)
show how much time each interesting thread is spent doing:
  * executing on the CPU
  * runnable but waiting to get a turn
  * anonymous paging: could run but desired memory is not resident yet
  * sleeping: waiting for I/O or data/text page-ins
  * lock: waiting to acquire a lock
  * idle: waiting for work



[Optimizable Code](https://deplinenoise.wordpress.com/2013/12/28/optimizable-code/)

[Array Layouts For Comparison-Based Searching](https://arxiv.org/pdf/1509.05053.pdf)

https://profiler.firefox.com/docs/#/./guide-perf-profiling
https://profiler.firefox.com/
https://github.com/KDAB/hotspot
* [Off-CPU Analysis](http://www.brendangregg.com/offcpuanalysis.html)

[Elimination back-of stack](https://max-inden.de/blog/2020-03-28-elimination-backoff-stack)
[A Scalable Lockfree Stack Algorithm](https://www.cs.bgu.ac.il/%7Ehendlerd/papers/p206-hendler.pdf)
* elimination stack

<iframe width="560" height="315" src="https://www.youtube.com/embed/CIdXPIN3j38" frameborder="0" allow="accelerometer; autoplay; encrypted-media; gyroscope; picture-in-picture" allowfullscreen></iframe>


