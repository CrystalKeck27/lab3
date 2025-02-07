use async_std::net::TcpStream;
use common::{prompt, ClientRequest, ClientRequestType, ServerResponse, ServerResponseType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let account_number = loop {
        let account_number_string = prompt("Enter your account number: ").await?;
        if let Ok(account_number) = u16::from_str_radix(&account_number_string, 10) {
            break account_number;
        } else {
            println!("Invalid account number. Please try again.");
        }
    };

    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;

    let key: [u8; 32] = [0; 32];
    let (mut encrypter, mut decrypter) = common::create_crypter(&key, &mut stream).await;

    loop {
        let command =
            prompt("Enter command (withdraw <amount>, deposit <amount>, balance, exit): ").await?;
        let mut splits = command.split_whitespace();
        match splits.next() {
            Some("withdraw") => {
                let Some(amount_string) = splits.next() else {
                    println!("No amount entered. Please try again.");
                    continue;
                };
                let Ok(amount) = amount_string.parse::<f32>() else {
                    println!("Invalid amount entered. Please try again.");
                    continue;
                };
                let request_type = ClientRequestType::Withdraw(amount);
                let request = ClientRequest {
                    account_number,
                    request_type,
                };

                common::send(&request, &mut stream, &mut encrypter).await?;

                let response: ServerResponse = common::recv(&mut stream, &mut decrypter).await?;
                match response.response_type {
                    ServerResponseType::Success => {
                        println!(
                            "Server response: Withdrawal of {} successful. New balance: {}",
                            amount, response.balance
                        );
                    }
                    ServerResponseType::Failure => {
                        println!(
                            "Server response: Withdrawal of {} failed. Current balance: {}",
                            amount, response.balance
                        );
                    }
                }
            }
            Some("deposit") => {
                let Some(amount_string) = splits.next() else {
                    println!("No amount entered. Please try again.");
                    continue;
                };
                let Ok(amount) = amount_string.parse::<f32>() else {
                    println!("Invalid amount entered. Please try again.");
                    continue;
                };
                let request_type = ClientRequestType::Deposit(amount);
                let request = ClientRequest {
                    account_number,
                    request_type,
                };

                common::send(&request, &mut stream, &mut encrypter).await?;

                let response: ServerResponse = common::recv(&mut stream, &mut decrypter).await?;
                match response.response_type {
                    ServerResponseType::Success => {
                        println!(
                            "Server response: Deposit of {} successful. New balance: {}",
                            amount, response.balance
                        );
                    }
                    ServerResponseType::Failure => {
                        println!(
                            "Server response: Deposit of {} failed. Current balance: {}",
                            amount, response.balance
                        );
                    }
                }
            }
            Some("balance") => {
                let request_type = ClientRequestType::Balance;
                let request = ClientRequest {
                    account_number,
                    request_type,
                };

                common::send(&request, &mut stream, &mut encrypter).await?;

                let response: ServerResponse = common::recv(&mut stream, &mut decrypter).await?;
                println!("Server response: Account balance: {}", response.balance);
            }
            Some("exit") => {
                break;
            }
            Some(_) => println!("Invalid command. Please try again."),
            None => println!("No command entered. Please try again."),
        }
    }

    Ok(())
}
