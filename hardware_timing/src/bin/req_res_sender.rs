use std::env::args;
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;

fn pew(port: u16) {
    let mut client = TcpStream::connect(("localhost", port)).unwrap();

    let data: &mut [u8] = &mut [0; 4096];

    for _ in 0..10_000 {
        client.write_all(data).unwrap();
        client.read_exact(data).unwrap();
    }

    client.flush().unwrap();
}

fn main() {
    let mut args = args();
    args.next().unwrap();

    let port = args
        .next()
        .expect("usage: prog <port> <n clients>")
        .parse::<u16>()
        .unwrap();

    let concurrency = args
        .next()
        .expect("usage: prog <port> <n clients>")
        .parse::<usize>()
        .unwrap();

    let mut threads = vec![];

    for _ in 0..concurrency {
        threads.push(thread::spawn(move || pew(port)));
    }

    for thread in threads.into_iter() {
        thread.join().unwrap();
    }
}
