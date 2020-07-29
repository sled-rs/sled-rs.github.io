use std::net::{TcpListener, TcpStream};

use futures::io;
use smol::{Async, Task};

/// Echoes messages from the client back to it.
async fn async_echo(
    stream: Async<TcpStream>,
) -> io::Result<()> {
    io::copy(&stream, &mut &stream).await?;
    Ok(())
}

fn main() -> io::Result<()> {
    smol::run(async {
        // Create a listener.
        let listener = Async::<TcpListener>::bind((
            [127, 0, 0, 1],
            7001,
        ))?;
        println!(
            "Listening on {}",
            listener.get_ref().local_addr()?
        );
        println!("Now start a TCP client.");

        // Accept clients in a loop.
        let mut clients = vec![];
        loop {
            let (stream, peer_addr) =
                listener.accept().await?;
            println!("Accepted client: {}", peer_addr);

            // Spawn a task that echoes messages from the client back to it.
            let client = Task::spawn(async_echo(stream));
            clients.push(client);
        }
    })
}
