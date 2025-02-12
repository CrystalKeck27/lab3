use async_std::{fs::File, io::{ReadExt, WriteExt}, net::TcpStream};
use common::{decrypt, encrypt, prompt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let mut server_public_key_file = File::open("../server_public_key.pem").await?;
    let mut client_private_key_file = File::open("../client_private_key.pem").await?;
    let server_public_key = common::file_to_pub_key(&mut server_public_key_file).await?;
    let client_private_key = common::file_to_priv_key(&mut client_private_key_file).await?;

    let username = prompt("Enter username: ").await?;

    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to server");

    loop {
        let message = prompt("> ").await?;

        if message.trim() == "exit" {
            stream.shutdown(std::net::Shutdown::Both)?;
            break;
        }

        let message = common::Message::new(&username, &message);
        let message_bytes = Vec::<u8>::from(&message);

        let sending_encrypted_message = encrypt(&server_public_key, &message_bytes)?;
        let sending_encrypted_message_length = (sending_encrypted_message.len() as u32).to_be_bytes();
        stream.write_all(&sending_encrypted_message_length).await?;
        stream.write_all(&sending_encrypted_message).await?;

        let mut recieving_encrypted_message_length = [0u8; 4];
        if let Err(e) = stream.read_exact(&mut recieving_encrypted_message_length).await {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                println!("Server closed connection");
                break;
            } else {
                return Err(e.into());
            }
        }
        let recieving_encrypted_message_length = u32::from_be_bytes(recieving_encrypted_message_length);
        let mut recieving_encrypted_message = vec![0u8; recieving_encrypted_message_length as usize];
        if let Err(e) = stream.read_exact(&mut recieving_encrypted_message).await {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                println!("Server closed connection");
                break;
            } else {
                return Err(e.into());
            }
        }
        let recieving_message_bytes = decrypt(&client_private_key, &recieving_encrypted_message)?;
        let recieving_message = common::Message::from(recieving_message_bytes.as_slice());
        

        println!("{}: {}", recieving_message.username, recieving_message.message);

    }

    Ok(())
}
