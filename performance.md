# sled performance guide

[![github](https://img.shields.io/github/stars/spacejam/sled.svg?style=social)](https://github.com/spacejam/sled)
[![documentation](https://docs.rs/sled/badge.svg)](https://docs.rs/sled)
[![chat](https://img.shields.io/discord/509773073294295082.svg?logo=discord)](https://discord.gg/Z6VsXds)
[![sponsors](https://img.shields.io/opencollective/backers/sled)](https://github.com/sponsors/spacejam)

## overview

This guide contains basic information for getting
started with performance-sensitive engineering.

The target audience is the Rust ecosystem, where many
people are now trying their hands at optimization
for the first time. But the vast majority of this
document applies generally to optimizing programs
running on machines, with a few of the hardware
effects mentioned being specific to circa-2020 x86.

Performance is being thoughtful about the metrics
that matter to us and allowing ourselves to be aware
of them while making decisions.

These materials are based on Tyler Neely's
Rust workshop content, and have been inspired
by the writings of
[Dmitry Vyukov](http://www.1024cores.net/home/parallel-computing/cache-oblivious-algorithms),
[Mark Callaghan](http://smalldatum.blogspot.com/2019/05/crum-conjecture-read-write-space-and.html),
[Brendan Gregg](http://www.brendangregg.com/usemethod.html),
[Martin Thompson](https://mechanical-sympathy.blogspot.com/2013/02/cpu-cache-flushing-fallacy.html),
[Pedro Ramalhete](http://concurrencyfreaks.blogspot.com/2019/11/is-left-right-generic-concurrency.html)
and others.

## contents

* [principles](#principles)
* [metrics](#metrics)
* [experimental design](#experimental-design)
* [USE Method](#use-method)
* [rust](#rust)
* [cpus](#cpus)
  * [frequency scaling](#frequency-scaling)
  * [4k aliasing](#4k-aliasing)
  * [bandwidth saturation](#bandwidth-saturation)
  * [branch misprediction](#branch-misprediction)
  * [branch target misprediction](#branch-target-misprediction)
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
* [memory](#memory)
* [threads](#threads)
* [async tasks](#async-tasks)
* [syscalls](#syscalls)
* [hardware effects](#hardware-effects)
* [universal scalability law](#universal-scalability-law)
* [flamegraphs](#flamegraphs)
* [cachegrind](#cachegrind)
* [massif](#massif)
* [dhat](#dhat)
* [benchmarketing](#benchmarketing)

## principles

```
    You are not a Bayesian homunculus whose
    reasoning is “corrupted” by cognitive biases.

    You just are cognitive biases.

```

[- Luke Muehlhauser, via R:AZ](https://www.readthesequences.com/Rationality-An-Introduction).

The first thing to consider is that our minds
are pure shit and everything we know is wrong.
We must accept our fallibility before embarking
down the path to fast-as-fuck machinery.
Our assumptions are all wrong to some extent,
and they often have a short shelf-life that we
habitually fail to check the expiration date on.
We build towers of assumptions that are bound
to specific contexts, and when the conditions
that caused us to form these beliefs change,
we tend not to revisit the now-invalidated
beliefs. Cache invalidation is hard when
we are so rarely aware of the dependency
graphs of what we believe.

So, we measure. Even when we're convinced
that we're right. Because we are always
wrong to some extent, and we are fundamentally
incapable of altering this fact. But we can
be responsible in the face of that.

Corollary: allow yourself to be wrong.
Allowing yourself to be wrong with yourself,
your collaborators, and in public is a key
optimization for learning faster and building
better things with less effort and in less time.

Luckily for us, machines tend to be quite
amenable to measurement. Constructing them
to be somewhat measurable in the first place
is the only reason we've been able to produce
them at all despite our many shortcomings. We took
the predecessor to your current machine,
chose some metrics to improve, made a huge
number of mistakes while continuing to measure,
and occasionally we got lucky and the metrics
we cared about improved enough to alter the
production lines - crystallizing the successful
results into new production processes that
eventually put your machine in front of you.

#### your programs

The only thing that matters is that real
programs on real hardware see statistically
significant improvements in real cost metrics
like total cost of ownership, responsiveness,
etc... If a metric doesn't help a human,
it's just a vanity pursuit that may make
the important metrics worse due to
under-investment.

One of the most frequently overlooked
performance metrics is the cognitive
complexity of a codebase. If engineers
experience high friction when trying to
change a codebase, all efforts to make
the code faster will be dramatically
hindered. A codebase that is a joy
for engineers to work with is a codebase
that will see the most long-term optimizations.
Codebases that burn people out will not
see long-term success unless they receive
tons of funding to replace people who
flee the project after short periods of
activity. [Organizational instability
is a high-quality predictive metric
for the bugginess of a codebase](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/tr-2008-11.pdf).

Putting energy into reducing the complexity
of your code will often make it:

* easier for humans to read (hence faster for
  them to optimize over time)
* easier for compilers to optimize
* faster to compile at all, resulting in a more
  responsive edit-measure loop, resulting in
  more optimizations per human time unit spent
* have less machine code, improving instruction
  cache at runtime (especially when running
  outside of microbenchmarks that conceal
  realistic cache effects)

"Experts write baby code." - Zarko Milosevic

So, we must pick our meaningful metrics,
measure them after considerate experimental
design, make decisions while having these
results at hand, and repeat.

Our unmeasured assumptions are incorrect.
Optimizing without measuring is how you
end up with unmaintainable codebases
that have been inflicted by many displays
of "performative-optimization" written
with the metric of "demonstrates
intellectual superiority" over metrics
like "long-term collaborator happiness".

Let's strive to be clear about our
metrics, at the very least.

## metrics

Performance metrics come in many shapes and sizes.
Workloads will have a few metrics that matter far
more than others.

It's at this point that I'm obligated to bash
[benchmarketing](#benchmarketing), but honestly
it's often an important tool for projects to
see success - you just need to be clear about
what your metrics actually are. Don't trick
people. Give people the means to reproduce
your findings. All that good science shit.

Most systems performance metrics boil down
to these two:

* latency - the time that an operation takes
* throughput - how many operations can be performed in some unit of time

At higher scales, both of these metrics become
factors in major concerns like:

* total cost of ownership
  * how many servers do I need to pay for to get my shit done?
  * how many hours do engineers spend taking care of this shit?
  * how much power does this shit draw?

In trying to determine how many servers do I need
to pay for to get my shit done, we need to consider
both latency and throughput.

If we have 1000 requests arriving per second
at an exponential distribution (as opposed to one
arriving each millisecond on the dot), our
system actually needs to process requests
faster than one each millisecond. Queue
theory tells us that as
our arrival rate approaches our processing
rate, our queue depth approaches infinity.
Nobody's got that kind of time to lay
around in line to be served. Queue
theory provides a number of key intuitions
for reasoning about the relationship
between latency and throughput. See
[this site](https://witestlab.poly.edu/blog/average-queue-length-of-an-m-m-1-queue/)
for pretty graphs illustrating this on an
[M/M/1](https://en.wikipedia.org/wiki/Kendall%27s_notation)
queue analyzing a network system.

Some other important general-purpose metrics are:

* utilization - the proportion of time that a system
  (server, disk, hashmap, etc...) is busy handling requests
* saturation - the extent to which requests must queue
  before being handled by the system, usually
  measured in terms of queue depth (length).

Latency and throughput considerations are often
in direct contradiction with each other. If we
want to optimize the throughput of a server,
we want to increase the chance that when a server
is finished processing one request that it already
has another one lined up and ready to go. 100%
utilization means that the server is always
doing useful work. If there is not work already
waiting to be served when the previous item completes,
the utilization drops, along with the throughput.
Having things waiting to go in a queue is a
common way to increase throughput.

But waiting (saturation) is bad for latency.
All other things being equal, sending more
requests to a system will cause latency to
suffer because the chance that a request
will have to wait in line before being served
will increase as well. If we want to minimize
the latency of a server, we want to increase the
chance that there is an empty queue leading
into it, because waiting in that queue will slow
down each request.

Latency vs throughput is a fundamental relationship that has
tremendous consequences for performance-sensitive
engineering. We are constantly faced with
decisions about whether we want our requests
to be fast, or if we want the system to generally
handle many requests per second, with some being
quite slow.

If you want to improve both latency and throughput,
you need to make the unit of work cheaper to perform.

Different systems will have different relationships
between utilization and saturation. Network adapters
are often designed to be able to keep receiving more
and more work and avoid saturation until relatively
high utilization. Other devices, like spinning disks,
will start saturating quite quickly, because the work
causes other work to get slower by needing to drag
the disk spindle to another physical location before
it's able to handle the request. Here's a place
where smart scheduling can make a huge difference for
the relationship between utilization and saturation.

Further reading:

* http://www.brendangregg.com/usemethod.html
* Systems Performance: Enterprise and the Cloud by
  Brendan Gregg (buy the book just to read chapter 2: Methodology)
* Quantitative Analysis of Computer Systems by Clement
  Leung - awesome intro to queue theory.


#### latency pitfalls

If you're measuring latency for a large number
of requests, there are a number of ways that you
can derive meaning from the measurements.

The one that many people reach for immediately
is average. But the average is not very interesting
for computer systems because it hides the impact
of outliers.

Some people claim that the geometric mean
instead of the arithmetic mean is a better
choice for some metrics, but for reasoning
about highly discrete systems (nearly
everything in the world of systems)
it's still a pretty low-quality metric.
Our systems do not fit nicely with
normal distributions and any sort of
average tells us very little about what
the distribution of latencies looks like.

Instead, we usually use histograms so that we
can understand the distribution of our data.
The 50th percentile is the median. The 90th
percentile is the latency that 90% of all
measured latencies are beneath. It's pretty
cheap to measure histograms by using logarithmic
bucketing to index into an array of buckets
that are sized to be within 1% of the true
observed values. The [historian](http://docs.rs/historian)
crate was extracted from sled to assist
with these measurements in a super cheap
manner.

Imagine this scenario:

* a front-end system sends 100 requests to a back-end system
* the front-end system is able to send each request in parallel
* the latency distribution for the back-end system is a
  steady 1ms until the 99th percentile where it jumps to 1s.
* the front-end system must wait for the slowest response
  before it can respond to the user

How long does the front-end system need to wait for?

The probability of needing to wait 1 second for a
single request is 1% (99th percentile is 1s). The
probability of needing to wait 1 second for 2 requests
is 1.9% (`1 - (0.99 ^ 2)`). Intuition: if we sent
1,000,000 requests, the percentage would not become
1,000,000 * 1%, or 10,000%, because 100% is the max
probability an event can have.

The probability of needing to wait 1 second for
100 requests is `1 - (0.99 ^ 100)`, or 63%. Even
though the event only happens 1% of the time, our
front-end system will have to wait 1 second in
63% of all cases, due to needing to send multiple
requests.

Our systems are full of subcomponents that are
accessed many times to satisfy a higher-level
request. The more often something happens, the
higher the percentile we should care about is.
For many workloads, looking at the 100th percentile
(max measurement) is quite helpful, even though
it only happened once, because it can help
to motivate capacity planning for other
systems that depend on it.

Further reading:

* [The Tail at Scale by Jeff Dean](https://cseweb.ucsd.edu/~gmporter/classes/fa17/cse124/post/schedule/p74-dean.pdf)

#### metrics case-study: sled

Here are some other metrics that are interesting
for sled:

* Single operation worst case latency: this
  is our primary metric because we are
  prioritizing transactional workloads above
  analytical workloads. We want users to
  have reliably responsive access to their
  data. We pay particular attention to the very
  worst case latency because it is fairly
  important from an operational perspective.
* Peak memory utilization: we want a high
  fraction of all allocated memory to be
  made up of user data that is likely
  to be accessed. This lets us keep our
  cache hit rates higher given the available
  memory, reducing the latency of more
  operations.
* Recovery latency. How long does it take
  to start the database after crashing?
* Peak memory throughput: we want to avoid
  short-lived allocations that may be more
  efficiently stored on the stack. This also
  allows us to have more predictable latency
  as our memory usage grows, because most
  allocators start to degrade in various ways
  as they are pushed harder.
* Bulk-loading throughput: we want users to
  be able to insert large amounts of data
  into sled quickly so they can start using it.
* Peak disk space utilization: we don't want
  sled to use 10x the space that user data
  requires. It's normal for databases to
  use 1-2x the actual data size because
  of various defragmenting efforts, but
  we reduce the number of deployment
  possibilities when this "space amplification"
  is high.
* Peak disk throughput: there is a trade-off
  between new data that can be written and
  the amount of disk throughput we spend
  rewriting old data to defragment the storage
  file and use less total space. If we are careful
  about minimizing the amount of data that we
  write at all, we can increase our range of
  choice between smaller files and higher write
  throughput.
* Disk durability: the more we write data at all,
  the sooner our drives will die. We should avoid
  moving data around too much. A huge amount of
  the work of building a high quality storage
  engine boils down to treating the disk kindly,
  often at the expense of write throughput.



## experimental design

We seek to make sled more efficient by changing code.

Running the same program twice will result
in two different measurements. But the difference
in performance is NOT necessarily because the
code is faster for realistic workloads.
[CPU frequency scaling](#frequency-scaling)
is a major source of variance, for instance.

If you spend more time compiling and applying
more optimizations, the program may run slower
if executed immediately after compilation,
because frequency scaling has kicked in
already.

Many code changes that run faster in microbenchmarks
will run more slowly when combined with
real business logic, because the microbenchmark
causes CPU caches to behave differently.

Often, code that runs faster in microbenchmarks
causes CPUs to heat up more, causing frequency
scaling to kick in more, and result in a slower
system when running for longer periods of time.
Faster code often consumes more heat, as well.
Maybe a 3% throughput improvement is not worth
a 100% power consumption increase.

Experimental design is about trying to
extract useful measurements despite known
and unknown sources of variance.

Only through careful measurement can we
increase our confidence that our observed
measurements correspond to the changes we
introduced in code.

Failing to exercise experimental discipline
will result in a lot of "optimizations"
that are assumed to improve the situation
but in fact only add complexity to the
codebase, reducing maintainability, and
making it harder to properly measure
future optimizations.

It's quite easy to justify a performance regression
as an improvement when you see a workload
running faster after changing code. But code
changes are far from the only things that
impact how long it takes to run a program,
or how fast the code runs.

There are a large number of known and unknown
factors that will introduce variance into
workload measurements.
Even if we run a program twice in a row,
we will experience variance in our observed
latencies and throughputs.

There are lots of ways to make sled faster in a
single run of a workload, and we need to
make sure that when we take measurements,
we are not actually measuring the effects
of things that do not relate to the code
that we are trying to optimize.


#### Bad:

```
* time compile and run workload 1
* time compile and run workload 2
* compare total times
```

#### Better:

```
* compile workload 1
* compile workload 2
* cool-down
* time workload 1
* time workload 2
* time workload 1
* time workload 2
...
* time workload 1
* time workload 2
* view distribution of results
```

#### Further reading:

* The Art of Computer Systems Performance Analysis by Raj Jain



## universal scalability law

http://smalldatum.blogspot.com/2019/10/usl-universal-scalability-law-is-good.html

## queue theory

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


If you have an Intel CPU, you can use the `i7z` command,
to see what your cores are currently doing. It is
 available in most Linux package managers.

```
CPU speed from cpuinfo 1607.00MHz

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


This list has been extracted from [Kobzol's wonderful hardware effects GitHub repo](https://github.com/Kobzol/hardware-effects).
[Ben Titzer - What Spectre Means for Language Implementors](https://www.youtube.com/watch?v=FGX-KD5Nh2g)

### 4k-aliasing

When you read a value that was just written, CPUs will

### bandwidth saturation

### branch misprediction

###
###


## memory
### numa
## threads
## syscalls
## filesystems
## disks
## networks
## hardware effects

Modern servers and laptops are






## flamegraphs
## cachegrind
## massif
## dhat

http://www.brendangregg.com/blog/2018-02-09/kpti-kaiser-meltdown-performance.html
http://www.brendangregg.com/offcpuanalysis.html

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

## benchmarketing

Sometimes publishing performance numbers is an important aspect of
marketing your system. When performed by a project that is favored
by someone, they will usually feel pride about those numbers. When
performed by a non-preferred project, the same person may call-out
the publishing of metrics as a nefarious effort to trick people into
using a system using cherry-picked metrics.

The fact is, in our attention-scarce internet spheres of communication,
metrics are often an effective means of capturing interest. Two bar charts
without any labels other than something like "higher is better" is
deceptive. We can capture interest in ethical ways by being clear
about what, specifically, we are measuring.

There are, of course, perverse incentives to minimize this context,
because it clutters up the call-to-action to get someone to try
out the project that you have put so much hard work into. Attention
is scarce, and you do need to be careful about how you present
context.

You should mention any hardware in the critical path relating to the
benchmark's outcome. You should mention the workload employed.
Ideally you should link to the code.

There is a time and place for
[benchmarketing](http://smalldatum.blogspot.com/2014/06/benchmarketing.html)
as long as it is not
