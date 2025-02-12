
use async_std::{fs::File, io::{stdin, stdout, ReadExt, WriteExt}};
use openssl::{error::ErrorStack, pkey::{HasPublic, PKeyRef}};

pub struct Message {
    pub username: String,
    pub message: String,
}

impl Message {
    pub fn new(username: &str, message: &str) -> Self {
        Self {
            username: username.to_string(),
            message: message.to_string(),
        }
    }
}

impl From<&[u8]> for Message {
    fn from(value: &[u8]) -> Self {
        let username_length = value[0] as usize;
        let value = &value[1..];
        let username = String::from_utf8(value[0..username_length].to_vec()).unwrap();
        let value = &value[username_length..];
        let message_length = u16::from_be_bytes([value[0], value[1]]) as usize;
        let value = &value[2..];
        let message = String::from_utf8(value[0..message_length].to_vec()).unwrap();
        Self { username, message }
    }
}

impl From<&Message> for Vec<u8> {
    fn from(value: &Message) -> Vec<u8> {
        let username_length = value.username.len() as u8;
        let username = value.username.as_bytes();
        let message_length = value.message.len() as u16;
        let message = value.message.as_bytes();
        let mut result = Vec::new();
        result.push(username_length);
        result.extend(username);
        result.extend(&message_length.to_be_bytes());
        result.extend(message);
        result
    }
}

/// Prints the provided prompt to the console and reads the user's response.
/// Returns the user's response as a `String` with all leading and trailing whitespace removed.
pub async fn prompt(prompt: &str) -> Result<String, std::io::Error> {
    print!("{}", prompt);
    stdout().flush().await?;
    let mut input = String::new();
    stdin().read_line(&mut input).await?;
    Ok(input.trim().to_string())
}

pub fn now() -> String {
    let now = chrono::Local::now();
    now.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn encrypt<'a, T>(pkey: &'a PKeyRef<T>, plaintext: &[u8]) -> Result<Vec<u8>, ErrorStack>  where T: HasPublic {
    // EVP_PKEY_encrypt_init
    let encrypter = openssl::encrypt::Encrypter::new(pkey)?;
    // EVP_PKEY_encrypt with null ptr
    let cyphertext_len = encrypter.encrypt_len(plaintext)?;
    let mut cyphertext = vec![0; cyphertext_len];
    // EVP_PKEY_encrypt
    encrypter.encrypt(plaintext, &mut cyphertext)?;
    Ok(cyphertext)
}

pub fn decrypt<'a, T>(pkey: &'a PKeyRef<T>, cyphertext: &[u8]) -> Result<Vec<u8>, ErrorStack> where T: openssl::pkey::HasPrivate {
    // EVP_PKEY_decrypt_init
    let decrypter = openssl::encrypt::Decrypter::new(pkey)?;
    // EVP_PKEY_decrypt with null ptr
    let plaintext_len = decrypter.decrypt_len(cyphertext)?;
    let mut plaintext = vec![0; plaintext_len];
    // EVP_PKEY_decrypt
    decrypter.decrypt(cyphertext, &mut plaintext)?;
    Ok(plaintext)
}

pub async fn file_to_pub_key(file: &mut File) -> Result<openssl::pkey::PKey<openssl::pkey::Public>, std::io::Error> {
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await?;
    Ok(openssl::pkey::PKey::public_key_from_pem(&buf)?)
}

pub async fn file_to_priv_key(file: &mut File) -> Result<openssl::pkey::PKey<openssl::pkey::Private>, std::io::Error> {
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).await?;
    Ok(openssl::pkey::PKey::private_key_from_pem(&buf)?)
}