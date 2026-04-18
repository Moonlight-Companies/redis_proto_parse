//! Basic text pub/sub using the Client wrapper.
//!
//! Run: `cargo run --example publish_subscribe`
//! Requires a Redis instance listening on 127.0.0.1:6379.

use redis_proto_parse::client::Client;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut subscriber = Client::new("127.0.0.1:6379").await?;
    subscriber.subscribe("demo.text").await?;

    let reader = tokio::spawn(async move {
        loop {
            match subscriber.next().await {
                Ok((channel, message)) => println!("rx {channel} <- {message}"),
                Err(e) => {
                    eprintln!("receive error: {e}");
                    break;
                }
            }
        }
    });

    // small delay so the SUBSCRIBE confirmation lands before we publish
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut publisher = Client::new("127.0.0.1:6379").await?;
    for i in 0..5 {
        let n = publisher
            .publish("demo.text", format!("hello {i}"))
            .await?;
        println!("tx demo.text -> hello {i} (delivered to {n} subscribers)");
    }

    // give the reader a moment to drain, then bail
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    reader.abort();
    Ok(())
}
