use std::convert::TryInto;
use std::time::{Duration, Instant};

const DESCRIPTION: &str = "
";

fn main() {
    const ITERATIONS: usize = 1 << 24;

    fn run(stride: usize, buf: &[u8]) -> Duration {
        let start = Instant::now();

        let mut sum = 0_usize;
        for i in 0..ITERATIONS {
            let slot = (i * stride) % buf.len();
            sum += buf[slot] as usize;
        }

        if sum % 99 == 32 {
            println!("neva gonna happen");
        }

        start.elapsed()
    }

    println!("{}", DESCRIPTION);

    for _ in 0..3 {
        for n in 0..20 {
            let buf = vec![n; 1024 * 1024 * 1024];

            let duration = run(8 << n, &buf);
            let per_op = duration / ITERATIONS.try_into().unwrap();
            println!(
                "using {:10.1} stride length while iterating \t{:?}, \t{:?}/op",
                8 << n,
                duration,
                per_op
            );
        }
    }
}
