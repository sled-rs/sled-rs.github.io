use std::time::{Duration, Instant};

const DESCRIPTION: &str = "
    This test will illustrate how many \"line fill buffers\" your CPU has. Each run of this test
    does the same amount of work. But at some point, the latency will significantly increase. The
    \"fill buffers\" is the number of memory locations that will be operated on in a tight loop.
    Below a certain number of memory locations, the CPU is able to perform \"write combining\" and
    buffer multiple writes to the same memory location without needing to block on effectively
    locking the memory location for access. These writes are stored in the \"line fill buffers\"
    which are usually able to store a congiguous memory region of 64 bytes (one cacheline) each.
    Once there are pending writes for more cache lines than line fill buffers, the CPU must block
    on the writes being pushed to L1 cache. For more (slightly old but generally still true)
    information, see https://mechanical-sympathy.blogspot.com/2011/07/write-combining.html
";

fn main() {
    const ITEMS: usize = 1 << 24;
    const MASK: usize = ITEMS - 1;
    const ITERATIONS: usize = 1 << 26;

    let mut vectors = vec![vec![0u8; ITEMS]; 32];

    fn run(n: usize, vectors: &mut [Vec<u8>]) -> Duration {
        let start = Instant::now();

        for chunk in vectors.chunks_mut(n) {
            for i in 0..ITERATIONS {
                let slot = i & MASK;
                let v = i as u8;
                for vector in chunk.iter_mut() {
                    vector[slot] = v;
                }
            }
        }

        start.elapsed()
    }

    println!("{}", DESCRIPTION);

    for _ in 0..2 {
        for n in 1..=16 {
            println!(
                "using {} fill buffers at a time takes {:?}",
                n,
                run(n, &mut vectors)
            );
        }
    }
}
