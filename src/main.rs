use futures::StreamExt;
use tokio::fs::File;
use tokio_util::codec::FramedRead;

use redis_pp::RedisCodec;

#[tokio::main]
async fn main() {
    let file = File::open("./src/proto_traffic.bin").await.unwrap();

    let mut frames = FramedRead::new(file, RedisCodec {});

    while let Some(res) = frames.next().await {
        match res {
            Ok(val) => println!("{:?}", val),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
