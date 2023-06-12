use futures::{StreamExt, SinkExt};
use tokio_util::codec::Framed;
use tokio::net::TcpStream;

use redis_pp::resp::{RespValue, RespCodec};


// creates a redis subscribe command
fn redis_subscribe(ch: &str) -> RespValue {
    RespValue::Array(Some(vec![
        RespValue::BulkString(Some("subscribe".into())),
        RespValue::BulkString(Some(ch.into())),
    ]))
}

// creates a redis psubscribe command
fn redis_psubscribe(ch: &str) -> RespValue {
    RespValue::Array(Some(vec![
        RespValue::BulkString(Some("psubscribe".into())),
        RespValue::BulkString(Some(ch.into())),
    ]))
}


#[tokio::main]
async fn main() {
    let stream = TcpStream::connect("bus.dev.moonlightcompanies.com:6379").await.unwrap();

    let (mut rx, mut tx) = Framed::new(stream, RespCodec::default()).split();

    let _ = rx.send(redis_psubscribe("groupbroadcast::*")).await;

    while let Some(Ok(val)) = tx.next().await {
        // we can do processing on the value!!
    };
}






