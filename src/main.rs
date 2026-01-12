mod client;
mod server;

use anyhow::Result;
use client::ChatClient;
use server::ChatServer;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "server" => {
            // Usage: cargo run -- server 127.0.0.1:8080
            let addr = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| "127.0.0.1:8080".to_string());

            let server = ChatServer::new(addr);
            server.run().await?;
        }

        "client" => {
            // Usage: cargo run -- client <host> <port> <username>
            if args.len() != 5 {
                print_usage();
                return Ok(());
            }

            let host = &args[2];
            let port: u16 = args[3].parse()?;
            let username = &args[4];

            let client = ChatClient::new(host, port, username);
            client.run().await?;
        }

        _ => print_usage(),
    }

    Ok(())
}

fn print_usage() {
    println!("Usage:");
    println!("  Start server:");
    println!("    cargo run -- server [addr]");
    println!("    example: cargo run -- server 127.0.0.1:8080");
    println!();
    println!("  Start client:");
    println!("    cargo run -- client <host> <port> <username>");
    println!("    example: cargo run -- client 127.0.0.1 8080 alice");
}
