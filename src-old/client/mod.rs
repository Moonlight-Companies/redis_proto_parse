// use std::io;

// use futures::{StreamExt, SinkExt};
// use tokio_util::codec::Framed;

// use crate::resp::RespCodec;

// struct Client {
//     framer: Framed<TcpStream, RespCodec>,
// }

// impl Client {
//     pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
//         let tcp = TcpStream::connect(addr).await?;

//         let (tx, rx) = Framed::new(tcp, RespCodec::default()).split();

//         todo!()
//     }
// }


use futures::Stream;
use tokio::sync::{mpsc, oneshot};

use futures::{StreamExt, SinkExt};
use tokio_util::codec::Framed;
use tokio::net::TcpStream;

use tokio::net::{TcpStream, ToSocketAddrs};

use std::io;

use crate::resp::{RespValue, RespCodec};



// helper to manage mpsc channels
struct Pair<T> {
    tx: mpsc::Sender<T>,
    rx: mpsc::Receiver<T>,
}

impl<T> Pair<T> {
    fn new(size: usize) -> Self {
        let (tx, rx) = mpsc::channel(size);
        Self { tx, rx }
    }

    fn swapped(size: usize) -> (Self, Self) {
        let mut p1 = Self::new(size);
        let mut p2 = Self::new(size);
        p1.swap(&mut p2);
        (p1, p2)
    }

    fn split(self) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        (self.tx, self.rx)
    }

    fn swap(&mut self, other: &mut Self) {
        std::mem::swap(&mut self.rx, &mut other.rx);
    }
}







struct Subscription {
    rx: mpsc::Receiver<RespValue>,
}

struct Client {
    tx: mpsc::Sender<RespValue>,
}


impl Client {
    async fn connect(addr: impl ToSocketAddrs) -> io::Result<Self> {
        let (mut pub_rx, mut pub_tx) = {
            let stream = TcpStream::connect(&addr).await?;
            Framed::new(stream, RespCodec::default()).split()
        };
    
        let (mut sub_rx, mut sub_tx) = {
            let stream = TcpStream::connect(addr).await?;
            Framed::new(stream, RespCodec::default()).split()
        };

        let (my_pair, pub_pair) = Pair::swapped(10);

        todo!()
    }
}

async fn subber(stream: TcpStream, mpsc: Pair<>) {
    
} 





enum Mesg {
    Subscribe(String),
}










// a client interface for redis pub/sub

/*

trait Pub {
    fn publish(&mut self) -> io::Result<()>;
    fn pub
}





impl Client {
    pub async fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {}

    async fn sub

}




fn main() {
    let c = Pubsub::connect("bus.dev.moonlightcompanies.com:6379").await.unwrap();



    c.publish("groupbroadcast::test", "hello world");

    


    let sub = Sub::from_pub(&pub);




    let ps = Pubsub::conto()
}
 */



