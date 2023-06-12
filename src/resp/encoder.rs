use bytes::{BytesMut, BufMut};

use crate::resp::RespValue;

pub fn resp_encode(item: RespValue, dst: &mut BytesMut) {
    match item {
        RespValue::SimpleString(s) => {
            dst.reserve(s.len() + 3);
            dst.put_slice(b"+");
            dst.put_slice(s.as_bytes());
            dst.put_slice(b"\r\n");
        }
        RespValue::Error(e) => {
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
                    resp_encode(item, dst);
                }
            } else {
                dst.reserve(5);
                dst.put_slice(b"*-1\r\n");
            }
        }
    }
}