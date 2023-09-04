use futures::{StreamExt, SinkExt};
use tokio_util::codec::Framed;
use tokio::net::TcpStream;

use redis_pp::resp::{RespValue, RespCodec};


// creates a redis subscribe command
fn redis_subscribe(ch: &str) -> RespValue {
    vec![
        RespValue::bulk("into_bs"),
        RespValue::bulk(ch)
    ].into()
}

// creates a redis psubscribe command
fn redis_psubscribe(ch: &str) -> RespValue {
    vec![
        RespValue::bulk("psubscribe"),
        RespValue::bulk(ch)
    ].into()
}


#[tokio::main]
async fn main() {

    let (mut rx, mut tx) = Framed::new(stream, RespCodec::default()).split();

    let _ = rx.send(redis_psubscribe("groupbroadcast::*")).await;

    while let Some(Ok(val)) = tx.next().await {
        // we can do processing on the value!!
    };
}

