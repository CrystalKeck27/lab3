use async_std::io;
use async_std::net::TcpListener;
use async_std::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let tcp_listener = TcpListener::bind("127.0.0.1:8080").await?;

    let mut incoming = tcp_listener.incoming();

    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let (reader, writer) = &mut (&stream, &stream);
        io::copy(reader, writer).await?;
    }

    Ok(())
}
