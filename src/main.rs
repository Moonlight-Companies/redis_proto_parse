use futures::StreamExt;
use tokio::fs::File;
use tokio_util::codec::FramedRead;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

use redis_pp::RedisCodec;

fn redis_subscribe(channel_name: String) -> String {
    format!("*2\r\n$9\r\nsubscribe\r\n${}\r\n{}\r\n", channel_name.len(), channel_name)
}

fn redis_psubscribe(channel_name: String) -> String {
    format!("*2\r\n$10\r\npsubscribe\r\n${}\r\n{}\r\n", channel_name.len(), channel_name)
}

#[tokio::main]
async fn main() {
    //let file = File::open("./src/proto_traffic.bin").await.unwrap();

    let mut socket = TcpStream::connect("bus.dev.moonlightcompanies.com:6380").await.unwrap();

    //socket.write_all(redis_psubscribe("groupbroadcast::gpio::*".into()).as_bytes()).await.unwrap();
    socket.write_all(redis_psubscribe("cv::*::camera".into()).as_bytes()).await.unwrap();

    let mut frames = FramedRead::new(socket, RedisCodec {});

    while let Some(res) = frames.next().await {
        match res {
            Ok(val) => println!("message"),
            //Ok(val) => println!("{:?}", val),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
