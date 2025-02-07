use std::collections::HashMap;
use std::sync::Arc;

use async_std::fs::File;
use async_std::{net::TcpListener, sync::Mutex};
use async_std::prelude::*;
use common::{send, ClientRequest, ClientRequestType, ServerResponse, ServerResponseType};

type Database = Arc<Mutex<HashMap<u16, f32>>>;
type LogFile = Arc<Mutex<File>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let database: Database = Arc::new(Mutex::new(HashMap::new()));
    let logfile = Arc::new(Mutex::new(File::create("transactions.log").await?));

    let tcp_listener = TcpListener::bind("127.0.0.1:8080").await?;
    let mut incoming = tcp_listener.incoming();

    println!("Server listening on port 8080");

    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let database = database.clone();
        let logfile = logfile.clone();
        tokio::spawn(async move {
            handle_client(stream, database, logfile).await;
        });
    }

    Ok(())
}

async fn handle_client(mut stream: async_std::net::TcpStream, mut database: Database, mut logfile: LogFile) {
    let key: [u8; 32] = [0; 32];
    let (mut encrypter, mut decrypter) = common::create_crypter(&key, &mut stream).await;
    while let Ok(request) = common::recv(&mut stream, &mut decrypter).await {
        let response = handle_request(request, &mut database, &mut logfile).await;
        send(&response, &mut stream, &mut encrypter).await.unwrap();
    }
}

async fn handle_request(request: ClientRequest, database: &mut Database, logfile: &mut LogFile) -> ServerResponse {
    let mut db = database.lock().await;
    let balance = db.entry(request.account_number).or_insert(1000.0);
    let mut log = logfile.lock().await;

    match request.request_type {
        ClientRequestType::Withdraw(amount) => {
            if *balance < amount {
                let now = common::now();
                log.write_all(now.as_bytes()).await.unwrap();
                log.write_all(b"\n").await.unwrap();
                let line = format!(" — Account: {} — Withdrawal of {} failed. Balance: {}\n", request.account_number, amount, *balance);
                log.write_all(line.as_bytes()).await.unwrap();
                log.flush().await.unwrap();
                ServerResponse {
                    account_number: request.account_number,
                    balance: *balance,
                    response_type: ServerResponseType::Failure,
                }
            } else {
            *balance -= amount;
            let now = common::now();
            log.write_all(now.as_bytes()).await.unwrap();
            log.write_all(b"\n").await.unwrap();
            let line = format!(" — Account: {} — Withdrawal of {} successful. New balance: {}\n", request.account_number, amount, *balance);
            log.write_all(line.as_bytes()).await.unwrap();
            log.flush().await.unwrap();
            ServerResponse {
                account_number: request.account_number,
                balance: *balance,
                response_type: ServerResponseType::Success,
            }}
        }
        ClientRequestType::Deposit(amount) => {
            *balance += amount;
            let now = common::now();
            log.write_all(now.as_bytes()).await.unwrap();
            log.write_all(b"\n").await.unwrap();
            let line = format!(" — Account: {} — Deposit of {}. New balance: {}\n", request.account_number, amount, *balance);
            log.write_all(line.as_bytes()).await.unwrap();
            log.flush().await.unwrap();
            ServerResponse {
                account_number: request.account_number,
                balance: *balance,
                response_type: ServerResponseType::Success,
            }
        }
        ClientRequestType::Balance => {
            ServerResponse {
                account_number: request.account_number,
                balance: *balance,
                response_type: ServerResponseType::Success,
            }
        }
    }
}
