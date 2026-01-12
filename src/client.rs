/* Client Requirement
    Async CLI
    Accept host, port, username
    Auto-connect on startup
    Interactive prompt
    Send messages
    Receive messages concurrently
    Clean exit on leave
*/

use anyhow::Result;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

pub struct ChatClient {
    host: String,
    port: u16,
    username: String,
}

impl ChatClient {
    pub fn new(host: impl Into<String>, port: u16, username: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port,
            username: username.into(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let addr = format!("{}:{}", self.host, self.port);
        let stream = TcpStream::connect(&addr).await?;
        println!("Connected to {}", addr);

        let (reader, mut writer) = stream.into_split();

        // Send JOIN
        writer
            .write_all(format!("JOIN {}\n", self.username).as_bytes())
            .await?;

        // Task 1: read messages from server
        let mut server_reader = BufReader::new(reader).lines();
        let read_task = tokio::spawn(async move {
            while let Ok(Some(line)) = server_reader.next_line().await {
                println!("\n{}", line);
                print!("> ");
            }
        });

        // Task 2: read user input
        let stdin = io::stdin();
        let mut input = BufReader::new(stdin).lines();

        print!("> ");
        while let Some(line) = input.next_line().await? {
            let line = line.trim();

            if line == "leave" {
                writer.write_all(b"LEAVE\n").await?;
                break;
            }

            if let Some(msg) = line.strip_prefix("send ") {
                writer
                    .write_all(format!("MSG {}\n", msg).as_bytes())
                    .await?;
            } else {
                println!("Unknown command. Use: send <msg> | leave");
            }

            print!("> ");
        }

        read_task.abort();
        println!("Disconnected");
        Ok(())
    }
}
