use async_std::io::{stdin, stdout, WriteExt};
use openssl::{error::ErrorStack, symm::Crypter};
use serde::{Deserialize, Serialize};

/// Prints the provided prompt to the console and reads the user's response.
/// Returns the user's response as a `String` with all leading and trailing whitespace removed.
pub async fn prompt(prompt: &str) -> Result<String, std::io::Error> {
    print!("{}", prompt);
    stdout().flush().await?;
    let mut input = String::new();
    stdin().read_line(&mut input).await?;
    Ok(input.trim().to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerResponse {
    pub account_number: u16,
    pub balance: f32,
    pub response_type: ServerResponseType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerResponseType {
    Success,
    Failure,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientRequestType {
    Withdraw(f32),
    Deposit(f32),
    Balance,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientRequest {
    pub account_number: u16,
    pub request_type: ClientRequestType,
}

pub async fn send<A, T>(
    request: &T,
    stream: &mut A,
    encrypter: &mut EnCrypter,
) -> Result<(), std::io::Error>
where
    A: async_std::io::WriteExt + Unpin,
    T: Serialize,
{
    let mut plain = serde_json::to_vec(&request)?;
    plain.push(b'\n');
    let mut len = plain.len();
    // next multiple of 16
    len = (len + 15) & !15;
    plain.resize(len, b' ');

    let mut cypher = vec![0u8; len];
    unsafe {
        encrypter.0.update_unchecked(&plain, &mut cypher).unwrap();
    }

    stream.write_all(&cypher).await?;
    Ok(())
}

pub async fn recv<A, T>(stream: &mut A, decrypter: &mut DeCrypter) -> Result<T, std::io::Error>
where
    A: async_std::io::ReadExt + Unpin,
    T: for<'de> Deserialize<'de>,
{
    let mut cypher = [0u8; 16];
    let mut plain = [0u8; 16];
    let mut full = Vec::new();
    loop {
        stream.read_exact(&mut cypher).await?;
        unsafe {
            decrypter.0.update_unchecked(&cypher, &mut plain).unwrap();
        }

        if let Some(index) = plain.iter().position(|&x| x == b'\n') {
            full.extend_from_slice(&plain[..index]);

            // Process the response
            let response: T = serde_json::from_slice(&full)?;

            return Ok(response);
        } else {
            full.extend_from_slice(&plain);
        }
    }
}

pub async fn create_crypter<A>(key: &[u8], stream: &mut A) -> (EnCrypter, DeCrypter)
where
    A: async_std::io::ReadExt + async_std::io::WriteExt + Unpin,
{
    let mut tx_iv: [u8; 16] = [0; 16];
    openssl::rand::rand_bytes(&mut tx_iv).unwrap();

    stream.write_all(&tx_iv).await.unwrap();
    let mut rx_iv = [0u8; 16];
    stream.read_exact(&mut rx_iv).await.unwrap();

    let mut encrypter = EnCrypter::new(key, &tx_iv).unwrap();
    encrypter.0.pad(false);

    let mut decrypter = DeCrypter::new(key, &rx_iv).unwrap();
    decrypter.0.pad(false);

    (encrypter, decrypter)
}

pub struct DeCrypter(Crypter);
pub struct EnCrypter(Crypter);

impl DeCrypter {
    fn new(key: &[u8], iv: &[u8]) -> Result<Self, ErrorStack> {
        Crypter::new(
            openssl::symm::Cipher::aes_256_cbc(),
            openssl::symm::Mode::Decrypt,
            key,
            Some(iv),
        )
        .map(|crypter| DeCrypter(crypter))
    }
}

impl EnCrypter {
    fn new(key: &[u8], iv: &[u8]) -> Result<Self, ErrorStack> {
        Crypter::new(
            openssl::symm::Cipher::aes_256_cbc(),
            openssl::symm::Mode::Encrypt,
            key,
            Some(iv),
        )
        .map(|crypter| EnCrypter(crypter))
    }
}

pub fn now() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}