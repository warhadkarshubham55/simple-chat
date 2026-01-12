/*  ***Server Requirement***
    Async TCP server
    Unique usernames
    JOIN / MSG / LEAVE
    Broadcast (excluding sender)
    Cleanup on disconnect
    High concurrency (Tokio + DashMap)
*/

use anyhow::{anyhow, Result};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

type ClientTx = mpsc::UnboundedSender<String>;
type Clients = Arc<DashMap<String, ClientTx>>;

pub struct ChatServer {
    addr: String,
    clients: Clients,
}

impl ChatServer {
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            clients: Arc::new(DashMap::new()),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Chat server listening on {}", self.addr);

        loop {
            let (stream, peer) = listener.accept().await?;
            println!("New connection from {}", peer);

            let clients = self.clients.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_client(stream, clients).await {
                    eprintln!("Client error: {e}");
                }
            });
        }
    }
}

async fn handle_client(stream: TcpStream, clients: Clients) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader).lines();

    //Expect JOIN
    let join_line = reader
        .next_line()
        .await?
        .ok_or_else(|| anyhow!("Disconnected before JOIN"))?;

    let username = parse_join(&join_line)?;

    if clients.contains_key(&username) {
        writer.write_all(b"ERROR Username already taken\n").await?;
        return Ok(());
    }

    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    clients.insert(username.clone(), tx);

    println!("{username} joined");

    //Writer task
    let writer_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if writer.write_all(msg.as_bytes()).await.is_err() {
                break;
            }
        }
    });

    //Read loop
    while let Some(line) = reader.next_line().await? {
        match line.as_str() {
            "LEAVE" => break,
            _ => {
                if let Some(msg) = parse_msg(&line) {
                    broadcast(&clients, &username, msg);
                }
            }
        }
    }

    //Cleanup
    clients.remove(&username);
    writer_task.abort();

    println!("{username} left");
    Ok(())
}

fn parse_join(line: &str) -> Result<String> {
    let mut parts = line.splitn(2, ' ');
    match (parts.next(), parts.next()) {
        (Some("JOIN"), Some(username)) if !username.is_empty() => Ok(username.to_string()),
        _ => Err(anyhow!("Invalid JOIN command")),
    }
}

fn parse_msg(line: &str) -> Option<String> {
    line.strip_prefix("MSG ").map(|s| s.to_string())
}

fn broadcast(clients: &Clients, sender: &str, msg: String) {
    let payload = format!("FROM {} {}\n", sender, msg);

    for entry in clients.iter() {
        if entry.key() != sender {
            let _ = entry.value().send(payload.clone());
        }
    }
}
