use async_std::net::TcpStream;
use async_std::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    stream.write_all(b"hello world").await?;

    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;

    // Print as text
    println!("{}", std::str::from_utf8(&buf[..n])?);

    Ok(())
}
