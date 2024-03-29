use std::io::{self, Read};

pub mod resp;
pub mod client;

pub use redis_protocol::{PubSubEvent, PubSubMessage, RedisCodec, RedisValue};

mod redis_protocol {
    use super::*;
    use std::io::{Error, ErrorKind::*};

    use bytes::{Buf, BytesMut};
    use tokio_util::codec::Decoder;

    const PROTO_STRING: u8 = b'$';
    const PROTO_LIST: u8 = b'*';
    const PROTO_INT: u8 = b':';
    const PROTO_OK: u8 = b'+';
    const PROTO_ERROR: u8 = b'-';
    const PROTO_CRLF: &[u8; 2] = &[0x0d, 0x0a];

    #[derive(Debug)]
    pub enum RedisValue {
        String(Vec<u8>),
        Int(i64),
        List(Vec<RedisValue>),
        Ok(String),
        Error(String),
    }
    pub struct PubSubMessage {
        pub channel_name: String,
        pub channel_pattern: Option<String>,
        pub data: Vec<u8>,
    }

    impl PubSubMessage {
        fn to_str(&self) -> Result<String, io::Error> {
            Ok(std::str::from_utf8(&self.data)
                .or(Err(Error::new(
                    InvalidData,
                    "buffer cannot be represented by a utf8 string",
                )))?
                .into())
        }
    }

    pub enum PubSubEvent {
        Message(PubSubMessage),
        Pong(String),
        List((String, Vec<RedisValue>)),
        String(String),
        Int(i64),
        Ok(String),
        Error(String),
    }

    impl TryFrom<RedisValue> for PubSubEvent {
        type Error = io::Error;

        fn try_from(value: RedisValue) -> Result<Self, io::Error> {
            match value {
                RedisValue::List(v) => {
                    let mut v = v.into_iter();
                    if let Some(message_kind) = v.next() {
                        match message_kind.as_str().as_str() {
                            "message" => {
                                if let (Some(rv_channel), Some(rv_data)) = (v.next(), v.next()) {
                                    return Ok(PubSubEvent::Message(PubSubMessage {
                                        channel_name: rv_channel.as_str(),
                                        channel_pattern: None,
                                        data: rv_data.take_buffer(),
                                    }));
                                };

                                return Err(Error::new(InvalidData, "protocol error - 'message' missing some parameters (expects channel, data)"));
                            }
                            "pmessage" => {
                                if let (Some(rv_pattern), Some(rv_channel), Some(rv_data)) =
                                    (v.next(), v.next(), v.next())
                                {
                                    return Ok(PubSubEvent::Message(PubSubMessage {
                                        channel_name: rv_channel.as_str(),
                                        channel_pattern: Some(rv_pattern.as_str()),
                                        data: rv_data.take_buffer(),
                                    }));
                                };

                                return Err(Error::new(InvalidData, "protocol error - 'message' missing some parameters (expects pattern, channel, data)"));
                            }
                            "subscribe" => {
                                // "subscribe" response can end up here [ "subscribe", "channel_name", RedisValue::Int ]
                                //
                                // todo : is N the number of channels in the list of channels subscribed to, subscribe can take multiple arguments
                                //
                                // https://redis.io/commands/subscribe/ -> for each channel, one message with the first element being the string "subscribe" is pushed as a confirmation that the command succeeded.
                                //
                                // *3
                                // $9
                                // subscribe
                                // $20
                                // groupbroadcast::gpio
                                // :1
                                //
                                return Ok(PubSubEvent::List(("subscribe".into(), v.collect())));
                            }
                            "unsubscribe" => {
                                return Ok(PubSubEvent::List(("unsubscribe".into(), v.collect())))
                            }
                            txt => return Ok(PubSubEvent::List((txt.into(), v.collect()))),
                        }
                    } else {
                        return Err(Error::new(InvalidData, "pubsub stream error - zero length list - capture stream with socat for bug report"));
                    }
                }
                RedisValue::String(v) => {
                    // ping response can come as a $<l>string
                    //
                    // https://redis.io/commands/ping/ -> Bulk string reply the argument provided, when applicable.
                    //
                    // we don't know the contents of this so just return string?
                    // also, can any other string show up, seems like this could just be Pong(v)
                    //
                    // todo : create test case for bulk string ping reply
                    //
                    return Ok(PubSubEvent::Pong(String::from_utf8_lossy(&v).to_string()));
                    //return Ok(PubSubEvent::String(String::from_utf8_lossy(&v).to_string()))
                }
                RedisValue::Int(v) => {
                    // not expected, can this happen? SUBSCRIBE returns number of channels in [ "SUBSCRIBE", x ] i believe
                    //
                    // todo : confirm Rx message on Tx SUBSCRIBE
                    //
                    return Ok(PubSubEvent::Int(v));
                }
                RedisValue::Ok(v) => {
                    //
                    // https://redis.io/commands/ping/ -> Simple string reply, and specifically PONG, when no argument is provided.
                    //
                    // redis> PING
                    // "PONG"   -> +PONG<crlf>
                    //
                    // todo : create test case for simple string reply
                    //

                    if v == "PONG" {
                        return Ok(PubSubEvent::Pong(v.into()));
                    }

                    return Ok(PubSubEvent::Ok(v));
                }
                RedisValue::Error(v) => return Ok(PubSubEvent::Error(v)),
            }
        }
    }
}
