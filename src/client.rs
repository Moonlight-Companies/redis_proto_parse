use std::io;

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio_util::codec::Framed;

use crate::resp::{value::*, RespCodec};

pub struct Sender {
    f_conn: Framed<TcpStream, RespCodec>,
}

pub struct Receiver {
    tx: SplitSink<Framed<TcpStream, RespCodec>, RespValue>,
    rx: SplitStream<Framed<TcpStream, RespCodec>>,
}

impl Sender {
    pub async fn new(addr: impl ToSocketAddrs) -> io::Result<Self> {
        let stream = tokio::net::TcpStream::connect(addr).await?;

        Ok(Self {
            f_conn: Framed::new(stream, RespCodec::default()),
        })
    }

    pub async fn publish(&mut self, channel: &str, mesg: &str) -> io::Result<i64> {
        let resp = vec![bulk("PUBLISH"), bulk(channel), bulk(mesg)].into();

        self.f_conn.send(resp).await?;
        // todo: is unwrap ok here
        let ret = self.f_conn.next().await.ok_or(io::ErrorKind::BrokenPipe)?;

        match ret {
            // happy case :)
            Ok(RespValue::Integer(i)) => Ok(i),
            // error case :(
            Err(e) => Err(e),
            Ok(RespValue::SimpleError(err)) => {
                Err(io::Error::new(io::ErrorKind::Other, String::from(err)))
            }
            Ok(_) => Err(io::Error::from(io::ErrorKind::InvalidData)),
        }
    }
}

impl Receiver {
    pub async fn new(addr: impl ToSocketAddrs) -> io::Result<Self> {
        let stream = tokio::net::TcpStream::connect(addr).await?;
        let framed = Framed::new(stream, RespCodec::default());
        let (tx, rx) = framed.split();

        Ok(Self { rx, tx })
    }

    pub async fn subscribe(&mut self, channel: &str) -> io::Result<()> {
        let resp = vec![bulk("SUBSCRIBE"), bulk(channel)].into();

        self.tx.send(resp).await?;

        Ok(())
    }

    pub async fn unsubscribe(&mut self, channel: &str) -> io::Result<()> {
        let resp = vec![bulk("UNSUBSCRIBE"), bulk(channel)].into();

        self.tx.send(resp).await?;

        Ok(())
    }

    pub async fn unsubscribe_all(&mut self) -> io::Result<()> {
        let resp = vec![bulk("UNSUBSCRIBE")].into();

        self.tx.send(resp).await?;

        Ok(())
    }

    pub async fn psubscribe(&mut self, pattern: &str) -> io::Result<()> {
        let resp = vec![bulk("PSUBSCRIBE"), bulk(pattern)].into();

        self.tx.send(resp).await?;

        Ok(())
    }

    pub async fn punsubscribe(&mut self, pattern: &str) -> io::Result<()> {
        let resp = vec![bulk("PUNSUBSCRIBE"), bulk(pattern)].into();

        self.tx.send(resp).await?;

        Ok(())
    }

    pub async fn punsubscribe_all(&mut self) -> io::Result<()> {
        let resp = vec![bulk("PUNSUBSCRIBE")].into();

        self.tx.send(resp).await?;

        Ok(())
    }

    pub async fn next(&mut self) -> io::Result<(String, String)> {
        let mut received_pong = true;

        loop {
            // future which will ping the server
            // after ten seconds
            let pingfut = async {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                if !received_pong {
                    return Err(io::Error::from(io::ErrorKind::TimedOut));
                }
                received_pong = false;
                // println!("sending ping");
                self.tx.send(vec![bulk("PING")].into()).await
            };

            // future representing the next frame
            let mut next = self.rx.next();

            let frame = tokio::select! {
                res = pingfut => {
                    res?;      // fail if future failed
                    next.await // await socket for pong
                },
                val = &mut next => val
            }
            .ok_or(io::ErrorKind::BrokenPipe)??;

            let items = match frame {
                RespValue::Array(Some(items)) => items,
                RespValue::SimpleString(p) if &*p == "PONG" => {
                    received_pong = true;
                    continue;
                }
                _ => return Err(io::Error::from(io::ErrorKind::InvalidData)),
            };

            let ty = items[0].as_str();

            let items = match ty {
                Some("message") => items.get(1..3),
                Some("pmessage") => items.get(2..4),
                Some("unsubscribe") | Some("punsubscribe") => continue,
                Some("subscribe") | Some("psubscribe") => continue,
                _ => return Err(io::Error::from(io::ErrorKind::InvalidData)),
            };

            let Some([a, b]) = items else {
                return Err(io::Error::from(io::ErrorKind::InvalidData))
            };

            let channel = a.as_str().ok_or(io::ErrorKind::InvalidData)?;
            let mesg = b.as_str().ok_or(io::ErrorKind::InvalidData)?;

            return Ok((channel.into(), mesg.into()));
        }
    }
}

pub struct Client {
    sender: Sender,
    receiver: Receiver,
}

impl Client {
    pub async fn new(addr: impl ToSocketAddrs) -> io::Result<Self> {
        let (sender, receiver) = tokio::join!(Sender::new(&addr), Receiver::new(&addr));

        Ok(Self {
            sender: sender?,
            receiver: receiver?,
        })
    }

    pub async fn publish(&mut self, channel: &str, mesg: &str) -> io::Result<i64> {
        self.sender.publish(channel, mesg).await
    }

    pub async fn subscribe(&mut self, channel: &str) -> io::Result<()> {
        self.receiver.subscribe(channel).await
    }

    pub async fn unsubscribe(&mut self, channel: &str) -> io::Result<()> {
        self.receiver.unsubscribe(channel).await
    }

    pub async fn unsubscribe_all(&mut self) -> io::Result<()> {
        self.receiver.unsubscribe_all().await
    }

    pub async fn psubscribe(&mut self, pattern: &str) -> io::Result<()> {
        self.receiver.psubscribe(pattern).await
    }

    pub async fn punsubscribe(&mut self, pattern: &str) -> io::Result<()> {
        self.receiver.punsubscribe(pattern).await
    }

    pub async fn punsubscribe_all(&mut self) -> io::Result<()> {
        self.receiver.punsubscribe_all().await
    }

    pub async fn next(&mut self) -> io::Result<(String, String)> {
        self.receiver.next().await
    }

    pub fn split(self) -> (Sender, Receiver) {
        (self.sender, self.receiver)
    }

    pub fn join(sender: Sender, receiver: Receiver) -> Self {
        Self { sender, receiver }
    }
}
