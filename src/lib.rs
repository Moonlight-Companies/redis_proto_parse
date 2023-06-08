use std::io::{self, Read};

pub use redis_protocol::{RedisCodec, RedisValue};

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
        String(String),
        Int(i32),
        List(Vec<RedisValue>),
        Ok(String),
        Error(String),
    }

    pub struct RedisCodec;

    impl Decoder for RedisCodec {
        type Item = RedisValue;
        type Error = io::Error;

        fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
            // obtain a new slice pointing to the source
            // mut slices have cursor functionality built
            // into the read implemenation
            let reader = &mut src.as_ref();

            match read_value(reader) {
                Ok(val) => {
                    src.advance(src.len() - reader.len());
                    Ok(Some(val))
                }
                // if we get an unexpected EOF, we need to wait for more data
                Err(e) if e.kind() == UnexpectedEof => Ok(None),
                Err(e) => Err(e),
            }
        }
    }

    fn read_length(src: &mut &[u8]) -> io::Result<i32> {
        for i in 0.. {
            let Some([l, r]) = src.get(i..i+2) else {
                return Err(Error::new(UnexpectedEof, ""))
            };

            if [*l, *r] == *PROTO_CRLF {
                let value = std::str::from_utf8(&src[..=i])
                    .or(Err(Error::new(
                        InvalidData,
                        "len read failed (not a string)",
                    )))?
                    .trim()
                    .parse()
                    .or(Err(Error::new(
                        InvalidData,
                        "len read failed (string not a number)",
                    )))?;
                take_vec(src, i + 2)?;

                return Ok(value);
            }
        }

        unreachable!()
    }

    fn pop_crlf(src: &mut &[u8]) -> io::Result<()> {
        if take_arr(src)? != *PROTO_CRLF {
            return Err(Error::new(InvalidData, "expected CRLF"));
        }

        Ok(())
    }

    fn read_value(src: &mut &[u8]) -> io::Result<RedisValue> {
        Ok(match take_u8(src)? {
            PROTO_STRING => read_redis_string(src)?,
            PROTO_INT => read_redis_int(src)?,
            PROTO_LIST => read_redis_list(src)?,
            PROTO_OK => RedisValue::Ok(read_redis_generic_crlf_string(src)?),
            PROTO_ERROR => RedisValue::Error(read_redis_generic_crlf_string(src)?),
            _ => return Err(Error::new(InvalidData, "invalid type")),
        })
    }

    fn read_redis_list(src: &mut &[u8]) -> io::Result<RedisValue> {
        let len = read_length(src)?;

        if len == -1 {
            // null list has "*-1\r\n"
            return Ok(RedisValue::List(Vec::new()));
        }

        let mut parts = Vec::with_capacity(len as usize);
        for _ in 0..len {
            parts.push(read_value(src)?);
        }

        Ok(RedisValue::List(parts))
    }

    fn read_redis_string(src: &mut &[u8]) -> io::Result<RedisValue> {
        let string_length = read_length(src)?;

        if string_length == -1 {
            // "null" string has "$-1\r\n"
            return Ok(RedisValue::String("".into()));
        }

        let buf = take_vec(src, string_length as usize)?;
        pop_crlf(src)?;

        let str = std::str::from_utf8(&buf)
            .or(Err(Error::new(
                InvalidData,
                "string read failed (not a string)",
            )))?
            .trim();

        Ok(RedisValue::String(str.into()))
    }

    fn read_redis_int(src: &mut &[u8]) -> io::Result<RedisValue> {
        Ok(RedisValue::Int(read_length(src)?))
    }

    fn read_redis_generic_crlf_string(src: &mut &[u8]) -> io::Result<String> {
        for i in 0.. {
            let Some([l, r]) = src.get(i..i+2) else {
                return Err(Error::new(UnexpectedEof, ""))
            };

            if [*l, *r] == *PROTO_CRLF {
                let value = std::str::from_utf8(&src[..=i])
                    .or(Err(Error::new(
                        InvalidData,
                        "string read failed (not a string)",
                    )))?
                    .trim();
                take_vec(src, i + 2)?;

                return Ok(value.into());
            }
        }

        unreachable!()
    }
}

fn take_arr<const N: usize>(src: &mut impl Read) -> io::Result<[u8; N]> {
    let mut buf = [0; N];
    src.read_exact(&mut buf)?;
    Ok(buf)
}

fn take_vec(src: &mut impl Read, n: usize) -> io::Result<Vec<u8>> {
    let mut buf = vec![0_u8; n];
    src.read_exact(&mut buf)?;
    Ok(buf)
}

fn take_u8(src: &mut impl Read) -> io::Result<u8> {
    take_arr::<1>(src).map(|[x]| x)
}
