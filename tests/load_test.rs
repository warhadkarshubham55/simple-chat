use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn load_test_100_clients() {
    let client_count = 100;
    let mut handles = Vec::new();

    for i in 0..client_count {
        handles.push(tokio::spawn(async move {
            let mut stream = TcpStream::connect("127.0.0.1:8080")
                .await
                .expect("connect failed");

            let username = format!("user{}", i);
            stream
                .write_all(format!("JOIN {}\n", username).as_bytes())
                .await
                .unwrap();

            stream
                .write_all(b"MSG Hello from load test\n")
                .await
                .unwrap();

            sleep(Duration::from_millis(50)).await;
        }));
    }

    for h in handles {
        h.await.unwrap();
    }
}
