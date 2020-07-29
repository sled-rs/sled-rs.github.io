use std::io::{self, prelude::*};
use std::net::{TcpListener, TcpStream};

/// Echoes messages from the client back to it.
fn sync_echo(mut src: TcpStream) -> io::Result<()> {
    let buf: &mut [u8] = &mut [0; 4096];
    let mut dst = src.try_clone().unwrap();
    loop {
        let read = src.read(buf).unwrap();
        if read == 0 {
            return Ok(());
        }
        dst.write_all(&mut buf[..read]).unwrap();
    }
}

fn main() -> io::Result<()> {
    // Create a listener.
    let listener = TcpListener::bind("127.0.0.1:7000")?;
    println!("Listening on {}", listener.local_addr()?);
    println!("Now start a TCP client.");

    // Accept clients in a loop.

    for stream_res in listener.incoming() {
        let stream = stream_res.unwrap();
        println!(
            "Accepted client: {:?}",
            stream.peer_addr().unwrap()
        );

        // Spawn a task that echoes messages from the client back to it.
        std::thread::spawn(move || sync_echo(stream));
    }
    Ok(())
}
