//! Binary (non-UTF-8) payloads over Redis pub/sub.
//!
//! The high-level `Client::next()` returns `(String, String)` and so requires
//! UTF-8 payloads. For arbitrary bytes we drive `RespCodec` directly via
//! `tokio_util::codec::Framed<TcpStream, RespCodec>`.
//!
//! Run: `cargo run --example binary_payload`
//! Requires a Redis instance listening on 127.0.0.1:6379.

use futures::{SinkExt, StreamExt};
use redis_proto_parse::client::Sender;
use redis_proto_parse::resp::{value::*, RespCodec};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // subscriber: raw codec over a TCP stream so we can keep payloads as bytes
    let sub_stream = TcpStream::connect("127.0.0.1:6379").await?;
    let mut subscriber = Framed::new(sub_stream, RespCodec::default());
    subscriber
        .send(vec![bulk("SUBSCRIBE"), bulk("demo.binary")].into())
        .await?;

    let reader = tokio::spawn(async move {
        while let Some(frame) = subscriber.next().await {
            let frame = match frame {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("decode error: {e}");
                    break;
                }
            };

            // Pubsub delivers `["message", channel, payload]`. Other control
            // frames (subscribe confirmations, pongs) we skip.
            let RespValue::Array(Some(items)) = frame else { continue };
            let ty = items.first().and_then(|v| v.as_str());
            let Some("message") = ty else { continue };

            let (Some(channel), Some(payload)) = (
                items.get(1).and_then(|v| v.as_str()),
                items.get(2).and_then(|v| v.as_buf()),
            ) else {
                continue;
            };

            println!("rx {channel} <- {} bytes: {:02x?}", payload.len(), payload);
        }
    });

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // publisher: the high-level Sender is fine — `publish` takes any AsRef<[u8]>
    let mut publisher = Sender::new("127.0.0.1:6379").await?;
    let payloads: [&[u8]; 3] = [
        &[0x00, 0x01, 0x02, 0xde, 0xad, 0xbe, 0xef],
        &[0xff; 16],
        b"\x89PNG\r\n\x1a\n",
    ];
    for (i, body) in payloads.iter().enumerate() {
        let n = publisher.publish("demo.binary", body).await?;
        println!("tx demo.binary -> payload {i} ({} bytes, {n} subscribers)", body.len());
    }

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    reader.abort();
    Ok(())
}
