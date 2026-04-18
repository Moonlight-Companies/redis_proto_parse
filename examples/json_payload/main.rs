//! JSON payloads over Redis pub/sub.
//!
//! RESP bulk strings are just bytes; this example serializes a struct with
//! serde_json and ships the JSON string as the message body.
//!
//! Run: `cargo run --example json_payload`
//! Requires a Redis instance listening on 127.0.0.1:6379.

use redis_proto_parse::client::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    id: u64,
    kind: String,
    payload: serde_json::Value,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut subscriber = Client::new("127.0.0.1:6379").await?;
    subscriber.subscribe("demo.events").await?;

    let reader = tokio::spawn(async move {
        loop {
            match subscriber.next().await {
                Ok((channel, json)) => match serde_json::from_str::<Event>(&json) {
                    Ok(event) => println!("rx {channel} <- {event:?}"),
                    Err(e) => eprintln!("rx {channel}: invalid json ({e}): {json}"),
                },
                Err(e) => {
                    eprintln!("receive error: {e}");
                    break;
                }
            }
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let mut publisher = Client::new("127.0.0.1:6379").await?;
    for i in 0..3 {
        let event = Event {
            id: i,
            kind: "greeting".into(),
            payload: serde_json::json!({ "msg": format!("hello {i}"), "i": i }),
        };
        let body = serde_json::to_string(&event).expect("serialize");
        publisher.publish("demo.events", &body).await?;
        println!("tx demo.events -> {body}");
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    reader.abort();
    Ok(())
}
