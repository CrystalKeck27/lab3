use std::sync::Arc;

use async_std::fs::File;
use async_std::prelude::*;
use async_std::{net::TcpListener, sync::Mutex};
use common::{decrypt, encrypt, prompt};

type LogFile = Arc<Mutex<File>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client_public_key_file = File::open("../client_public_key.pem").await?;
    let mut server_private_key_file = File::open("../server_private_key.pem").await?;
    let client_public_key = common::file_to_pub_key(&mut client_public_key_file).await?;
    let server_private_key = common::file_to_priv_key(&mut server_private_key_file).await?;

    let logfile = Arc::new(Mutex::new(File::create("transactions.log").await?));

    let username = prompt("Enter username: ").await?;

    let tcp_listener = TcpListener::bind("0.0.0.0:8080").await?;
    println!("Server listening on port 8080");

    let Ok((mut stream, _)) = tcp_listener.accept().await else {
        println!("No connection");
        return Ok(());
    };

    drop(tcp_listener);

    println!("Connection established");

    loop {
        let message = prompt("> ").await?;

        if message.trim() == "exit" {
            stream.shutdown(std::net::Shutdown::Both)?;
            break;
        }

        let message = common::Message::new(&username, &message);

        let mut logfile = logfile.lock().await;
        let now = common::now();
        logfile.write_all(now.as_bytes()).await?;
        logfile.write_all(b"\n").await?;
        logfile.write_all(message.username.as_bytes()).await?;
        logfile.write_all(b": ").await?;
        logfile.write_all(message.message.as_bytes()).await?;
        logfile.write_all(b"\n").await?;
        let message_bytes = Vec::<u8>::from(&message);

        let sending_encrypted_message = encrypt(&client_public_key, &message_bytes)?;
        let sending_encrypted_message_length =
            (sending_encrypted_message.len() as u32).to_be_bytes();
        stream.write_all(&sending_encrypted_message_length).await?;
        stream.write_all(&sending_encrypted_message).await?;

        let mut recieving_encrypted_message_length = [0u8; 4];
        if let Err(e) = stream
            .read_exact(&mut recieving_encrypted_message_length)
            .await
        {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                println!("Server closed connection");
                break;
            } else {
                return Err(e.into());
            }
        }
        let recieving_encrypted_message_length =
            u32::from_be_bytes(recieving_encrypted_message_length);
        let mut recieving_encrypted_message =
            vec![0u8; recieving_encrypted_message_length as usize];
        if let Err(e) = stream.read_exact(&mut recieving_encrypted_message).await {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                println!("Server closed connection");
                break;
            } else {
                return Err(e.into());
            }
        }
        let recieving_message_bytes = decrypt(&server_private_key, &recieving_encrypted_message)?;
        let recieving_message = common::Message::from(recieving_message_bytes.as_slice());

        let now = common::now();
        logfile.write_all(now.as_bytes()).await?;
        logfile.write_all(b"\n").await?;
        logfile.write_all(recieving_message.username.as_bytes()).await?;
        logfile.write_all(b": ").await?;
        logfile.write_all(recieving_message.message.as_bytes()).await?;
        logfile.write_all(b"\n").await?;
        logfile.flush().await?;



        // print!("\u{001B}[s");
        // print!("\u{001B}[A");
        // print!("\u{001B}[999D");
        // print!("\u{001B}[S");
        // print!("\u{001B}[L");
        // print!("Server response: {}", recieving_message);
        // print!("\u{001B}[u");
        println!("{}: {}", recieving_message.username, recieving_message.message);
    }

    Ok(())
}
