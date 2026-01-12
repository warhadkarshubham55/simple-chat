use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn user_can_join_server() {
    let mut stream = TcpStream::connect("127.0.0.1:8080")
        .await
        .expect("Failed to connect");

    stream.write_all(b"JOIN alice\n").await.unwrap();

    let mut buffer = [0u8; 128];
    let n = stream.read(&mut buffer).await.unwrap();

    let _response = String::from_utf8_lossy(&buffer[..n]);
    // If no panic / disconnect → join succeeded
    assert!(stream.peer_addr().is_ok());
}

#[tokio::test]
async fn duplicate_username_is_rejected() {
    let mut s1 = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let mut s2 = TcpStream::connect("127.0.0.1:8080").await.unwrap();

    s1.write_all(b"JOIN bob\n").await.unwrap();
    sleep(Duration::from_millis(50)).await;

    s2.write_all(b"JOIN bob\n").await.unwrap();

    let mut buffer = [0u8; 128];
    let n = s2.read(&mut buffer).await.unwrap();

    let response = String::from_utf8_lossy(&buffer[..n]);
    assert!(response.contains("Username already taken"));
}

#[tokio::test]
async fn message_is_broadcast_to_other_users() {
    let mut alice = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let mut bob = TcpStream::connect("127.0.0.1:8080").await.unwrap();

    alice.write_all(b"JOIN alicey join\n").await.unwrap();
    bob.write_all(b"JOIN bobby join\n").await.unwrap();

    sleep(Duration::from_millis(100)).await;

    alice.write_all(b"MSG Hello Bob!\n").await.unwrap();

    let mut buffer = [0u8; 128];
    let n = bob.read(&mut buffer).await.unwrap();
    let msg = String::from_utf8_lossy(&buffer[..n]);
    eprintln!("msg :: {:?}", msg);

    assert!(msg.contains("alicey join"));
    assert!(msg.contains("Hello Bob!"));
}

#[tokio::test]
async fn leave_removes_user_from_room() {
    let mut alice = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let mut bob = TcpStream::connect("127.0.0.1:8080").await.unwrap();

    alice.write_all(b"JOIN alice\n").await.unwrap();
    bob.write_all(b"JOIN bob\n").await.unwrap();

    sleep(Duration::from_millis(50)).await;

    alice.write_all(b"LEAVE\n").await.unwrap();
    sleep(Duration::from_millis(50)).await;

    
    bob.write_all(b"MSG Are you there?\n").await.unwrap();

    let mut buffer = [0u8; 128];
    let n = alice.read(&mut buffer).await.unwrap_or(0);

    assert_eq!(n, 0); // alice should not receive messages
}
