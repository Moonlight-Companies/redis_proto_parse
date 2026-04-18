use std::io::{self, Error, ErrorKind};

use bytes::{BufMut, BytesMut};

use crate::resp::RespValue;

fn contains_crlf(s: &str) -> bool {
    s.as_bytes().iter().any(|&b| b == b'\r' || b == b'\n')
}

pub fn resp_encode(item: RespValue, dst: &mut BytesMut) -> io::Result<()> {
    match item {
        RespValue::SimpleString(s) => {
            if contains_crlf(&s) {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "SimpleString contains CR or LF",
                ));
            }
            dst.reserve(s.len() + 3);
            dst.put_slice(b"+");
            dst.put_slice(s.as_bytes());
            dst.put_slice(b"\r\n");
        }
        RespValue::SimpleError(e) => {
            if contains_crlf(&e) {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "SimpleError contains CR or LF",
                ));
            }
            dst.reserve(e.len() + 3);
            dst.put_slice(b"-");
            dst.put_slice(e.as_bytes());
            dst.put_slice(b"\r\n");
        }
        RespValue::Integer(i) => {
            let str = i.to_string();
            dst.reserve(str.len() + 3);
            dst.put_slice(b":");
            dst.put_slice(str.as_bytes());
            dst.put_slice(b"\r\n");
        }
        RespValue::BulkString(s) => {
            if let Some(vec) = s {
                let len = vec.len().to_string();
                dst.reserve(len.len() + 3);
                dst.put_slice(b"$");
                dst.put_slice(len.as_bytes());
                dst.put_slice(b"\r\n");

                dst.reserve(vec.len() + 2);
                dst.put_slice(&vec);
                dst.put_slice(b"\r\n");
            } else {
                dst.reserve(5);
                dst.put_slice(b"$-1\r\n");
            }
        }
        RespValue::Array(a) => {
            if let Some(vec) = a {
                let len = vec.len().to_string();
                dst.reserve(len.len() + 3);
                dst.put_slice(b"*");
                dst.put_slice(len.as_bytes());
                dst.put_slice(b"\r\n");

                for item in vec {
                    resp_encode(item, dst)?;
                }
            } else {
                dst.reserve(5);
                dst.put_slice(b"*-1\r\n");
            }
        }
    }
    Ok(())
}
