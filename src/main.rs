use futures::StreamExt;
use tokio::fs::File;
use tokio_util::codec::FramedRead;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

use redis_pp::{ RedisCodec, RedisValue, PubSubEvent, PubSubMessage };

fn redis_subscribe(channel_name: String) -> String {
    format!("*2\r\n$9\r\nsubscribe\r\n${}\r\n{}\r\n", channel_name.len(), channel_name)
}

fn redis_psubscribe(channel_name: String) -> String {
    format!("*2\r\n$10\r\npsubscribe\r\n${}\r\n{}\r\n", channel_name.len(), channel_name)
}

#[tokio::main]
async fn main() {
    //let file = File::open("./src/proto_traffic.bin").await.unwrap();

    // for regular json data
    let mut socket = TcpStream::connect("bus.dev.moonlightcompanies.com:6379").await.unwrap();
    socket.write_all(redis_psubscribe("groupbroadcast::*".into()).as_bytes()).await.unwrap();

    // for camera data
    // let mut socket = TcpStream::connect("bus.dev.moonlightcompanies.com:6380").await.unwrap();
    // socket.write_all(redis_psubscribe("cv::*::camera".into()).as_bytes()).await.unwrap();

    let mut frames = FramedRead::new(socket, RedisCodec {});

    while let Some(res) = frames.next().await {
        match res {
            // Ok(val) => {
            //     match PubSubMessage::try_from(val) {
            //         Ok(temp) => {
            //             println!("{:?} {} bytes", temp.channel_name, temp.data.len());
            //         },
            //         Err(e) => println!("Error in conversion: {:?}", e),
            //     }
            // },
            Ok(v) => {
                if let Ok(psmsg)=PubSubEvent::try_from(v) {
                    match psmsg {
                        PubSubEvent::Message(msg) => {
                            println!("MESSAGE:{}", msg.channel_name);
                        },
                        _ => {}
                    }
                }
            }
            Err(e) => println!("Error: {:?}", e),
        }
    }
}
