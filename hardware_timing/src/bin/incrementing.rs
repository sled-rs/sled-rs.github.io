use std::{
    sync::{
        atomic::{fence, AtomicUsize, Ordering},
        Arc, Barrier, Mutex,
    },
    thread,
    time::Instant,
};

const N: usize = 50_000_000;

fn black_box<T>(item: &T) -> T {
    unsafe { std::ptr::read_volatile(item) }
}

fn main() {
    println!("trying to increment a number 500 million times");

    for _ in 0..100000 {
        Instant::now().elapsed();
    }

    let timer = Instant::now();
    println!("measurement overhead: {:?}", timer.elapsed());

    let timer = Instant::now();
    let mut count = 0;
    for i in 0..black_box(&N) {
        unsafe { std::ptr::write_volatile(&mut count, i) };
    }
    black_box(&count);
    println!("single-threaded with write_volatile: {:?}", timer.elapsed());

    let timer = Instant::now();
    let mut count = 0;
    for i in 0..black_box(&N) {
        unsafe { std::ptr::write_volatile(&mut count, i) };
        fence(Ordering::Release);
    }
    black_box(&count);
    println!(
        "single-threaded write_volatile + Release raw fence: {:?}",
        timer.elapsed()
    );

    let timer = Instant::now();
    let mut count = 0;
    for i in 0..black_box(&N) {
        unsafe { std::ptr::write_volatile(&mut count, i) };
        fence(Ordering::SeqCst);
    }
    black_box(&count);
    println!(
        "single-threaded write_volatile + SeqCst raw fence: {:?}",
        timer.elapsed()
    );

    let timer = Instant::now();
    let count = AtomicUsize::new(0);
    for i in 0..black_box(&N) {
        count.compare_and_swap(i, i + 1, Ordering::SeqCst);
    }
    black_box(&count);
    println!("single-threaded with SeqCst CAS: {:?}", timer.elapsed());

    for threads in 1..3 {
        for ordering in
            [Ordering::Relaxed, Ordering::SeqCst, Ordering::AcqRel].iter()
        {
            let barrier1 = Arc::new(Barrier::new(threads + 1));
            let barrier2 = Arc::new(Barrier::new(threads + 1));
            let count = Arc::new(AtomicUsize::new(0));
            for _ in 0..threads {
                let barrier1 = barrier1.clone();
                let barrier2 = barrier2.clone();
                let count = count.clone();
                thread::spawn(move || {
                    let mut current = count.load(Ordering::Acquire);
                    barrier1.wait();
                    while current < black_box(&N) {
                        let res = count.compare_and_swap(
                            current,
                            current + 1,
                            *ordering,
                        );
                        if res == current {
                            current = current + 1;
                        } else {
                            current = res;
                        }
                    }
                    barrier2.wait();
                });
            }
            barrier1.wait();
            let timer = Instant::now();
            barrier2.wait();
            println!(
                "{} threads with {:?} CAS: {:?}",
                threads,
                ordering,
                timer.elapsed()
            );
        }
    }

    for threads in 1..3 {
        let barrier1 = Arc::new(Barrier::new(threads + 1));
        let barrier2 = Arc::new(Barrier::new(threads + 1));
        let count = Arc::new(Mutex::new(0));
        for _ in 0..threads {
            let barrier1 = barrier1.clone();
            let barrier2 = barrier2.clone();
            let count = count.clone();
            thread::spawn(move || {
                barrier1.wait();
                let mut current = 0;
                while current < black_box(&N) {
                    let mut locked = count.lock().unwrap();
                    *locked += 1;
                    current = *locked;
                }
                barrier2.wait();
            });
        }
        barrier1.wait();
        let timer = Instant::now();
        barrier2.wait();
        println!("{} threads with a Mutex: {:?}", threads, timer.elapsed());
    }
}
