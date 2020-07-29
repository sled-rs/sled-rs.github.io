use std::{
    convert::TryInto,
    io,
    net::{SocketAddr, ToSocketAddrs, UdpSocket},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use smol::Async;

async fn echo_async() {
    let socket =
        Async::<UdpSocket>::bind(([127, 0, 0, 1], 7001))
            .unwrap();

    let mut buf = [0; 4096];
    loop {
        let (n, from) =
            socket.recv_from(&mut buf).await.unwrap();
        socket.send_to(&buf[..n], from).await.unwrap();
    }
}

fn echo_sync() {
    let socket = UdpSocket::bind("127.0.0.1:7000").unwrap();

    let mut buf = [0; 4096];
    loop {
        let (n, from) = socket.recv_from(&mut buf).unwrap();
        socket.send_to(&buf[..n], from).unwrap();
    }
}

fn client(port: u16, sz: usize) -> Duration {
    let start = Instant::now();

    let socket = UdpSocket::bind("127.0.0.1:7002").unwrap();

    let mut buf = vec![0; sz];

    let dst: SocketAddr = format!("127.0.0.1:{}", port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    for _ in 0..100000 {
        socket.send_to(&buf, dst).unwrap();
        let (_, _) = socket.recv_from(&mut buf).unwrap();
    }

    start.elapsed()
}

fn main() {
    let sync_server = thread::spawn(echo_sync);
    thread::sleep(Duration::from_secs(1));

    let async_server =
        thread::spawn(|| smol::run(echo_async()));

    let sync_time = client(7000, 4096);
    let async_time = client(7001, 4096);

    let sync_rate = 1000 * 100000 / sync_time.as_millis();
    let async_rate = 1000 * 100000 / async_time.as_millis();

    println!(
        "sync: {}/s async: {}/s",
        sync_rate, async_rate
    );
}
