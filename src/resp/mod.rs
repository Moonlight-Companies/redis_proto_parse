use std::io::{self, ErrorKind};

use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

pub use self::value::RespValue;

mod decoder;
mod encoder;
mod value;

#[derive(Default)]
pub struct RespCodec {
    dec: decoder::RespDecoder,
}

impl Decoder for RespCodec {
    type Item = RespValue;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        match self.dec.resume_decode(src) {
            // if we get a value, return it
            Ok(val) => Ok(Some(val)),
            // if we get an unexpected EOF, we need to wait for more data
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => Ok(None),
            // if we get any other error, we need to return it
            Err(e) => Err(e),
        }
    }
}

impl Encoder<RespValue> for RespCodec {
    type Error = io::Error;

    fn encode(&mut self, item: RespValue, dst: &mut BytesMut) -> Result<(), Self::Error> {
        encoder::resp_encode(item, dst);

        Ok(())
    }
}