use std::io::{self, Error, ErrorKind::*};

use bytes::{Buf, BytesMut};

use crate::resp::RespValue;
use RespValue::*;

use super::value::*;

#[derive(Debug)]
enum Op {
    SimpleString,
    Error,
    Integer,
    BulkString,
    Array,
}

#[derive(Default)]
struct ArrayContext {
    rem: i64,
    items: Vec<RespValue>,
}

impl ArrayContext {
    fn new(len: i64) -> Self {
        Self {
            rem: len,
            items: Vec::with_capacity(len as usize),
        }
    }

    fn push(&mut self, item: RespValue) {
        self.items.push(item);

        self.rem -= 1;
        debug_assert!(self.rem >= 0);
    }

    fn is_complete(&self) -> bool {
        self.rem == 0
    }

    fn items(self) -> Vec<RespValue> {
        self.items
    }
}

#[derive(Default)]
pub struct RespDecoder {
    ptr: usize,
    cached_len: Option<i64>,
    op: Option<Op>,
    stack: Vec<ArrayContext>,
}

impl RespDecoder {
    /// Returns the next operation, storing it in case of partial read.
    fn get_op(&mut self, src: &mut BytesMut) -> io::Result<&Op> {
        if self.op.is_none() {
            if src.is_empty() {
                return Err(Error::new(UnexpectedEof, ""));
            }

            let op = match src.get_u8() {
                b'+' => Op::SimpleString,
                b'-' => Op::Error,
                b':' => Op::Integer,
                b'$' => Op::BulkString,
                b'*' => Op::Array,
                _ => return Err(Error::new(InvalidData, "invalid prefix")),
            };

            self.op = Some(op);
        }
        // safety: is set 100%
        unsafe { Ok(self.op.as_ref().unwrap_unchecked()) }
    }

    /// Returns the index of the next CRLF, or an error if EOF is reached.
    fn next_crlf(&mut self, src: &mut BytesMut) -> io::Result<usize> {
        loop {
            let crlf = src
                .get(self.ptr..self.ptr + 2)
                .ok_or_else(|| Error::new(UnexpectedEof, ""))?;

            if self.ptr > 512_000_000 {
                return Err(Error::new(InvalidData, "too long"));
            }

            if crlf == [b'\r', b'\n'] {
                let ptr = self.ptr;
                self.ptr = 0;
                return Ok(ptr);
            };

            self.ptr += 1;
        }
    }

    /// Takes a String and its CRLF delimiter out of the BytesMut instance.
    fn inner_string(&mut self, src: &mut BytesMut) -> io::Result<String> {
        let idx = self.next_crlf(src)?;

        // todo: investigate if this can be done without a copy
        let window = src.split_to(idx);
        let slice_as_str =
            std::str::from_utf8(&window).map_err(|_| Error::new(InvalidData, "invalid utf8"))?;

        src.advance(2);
        Ok(slice_as_str.into())
    }

    /// Takes an i64 and its CRLF delimiter out of the BytesMut instance.
    fn inner_i64(&mut self, src: &mut BytesMut) -> io::Result<i64> {
        let idx = self.next_crlf(src)?;

        let window = src.split_to(idx);
        let num = std::str::from_utf8(&window)
            .map_err(|_| Error::new(InvalidData, "invalid utf8"))?
            .parse()
            .map_err(|_| Error::new(InvalidData, "invalid integer"))?;

        src.advance(2);
        Ok(num)
    }

    fn get_simple_string(&mut self, src: &mut BytesMut) -> io::Result<RespValue> {
        Ok(simple(self.inner_string(src)?))
    }

    fn get_error(&mut self, src: &mut BytesMut) -> io::Result<RespValue> {
        Ok(err(self.inner_string(src)?))
    }

    fn get_integer(&mut self, src: &mut BytesMut) -> io::Result<RespValue> {
        Ok(Integer(self.inner_i64(src)?))
    }

    fn get_bulk_string(&mut self, src: &mut BytesMut) -> io::Result<RespValue> {
        // if the length has already been calculated, use it
        let len = match self.cached_len {
            Some(len) => len,
            None => {
                let len = self.inner_i64(src)?;

                if len == -1 {
                    return Ok(BulkString(None));
                }

                self.cached_len = Some(len);
                len
            }
        };

        if len + 2 > src.len() as i64 {
            return Err(Error::new(UnexpectedEof, ""));
        }

        self.cached_len = None;
        let buf: Box<[_]> = src.split_to(len as usize)[..].into();
        src.advance(2);

        Ok(bulk(buf))
    }

    /// Returns an ArrayContext instead of a RedisValue. When resume_decode
    /// gets a RedisValue from one of the above functions, it will push it
    /// to the topmost ArrayContext on the stack, which keeps track of how
    /// many items are left to be decoded.
    fn get_array_context(&mut self, src: &mut BytesMut) -> io::Result<Option<ArrayContext>> {
        let len = self.inner_i64(src)?;

        if len == -1 {
            return Ok(None);
        }

        Ok(Some(ArrayContext::new(len)))
    }

    /// Begin decoding the BytesMut instance, or resume where it left off.
    pub(crate) fn resume_decode(&mut self, src: &mut BytesMut) -> io::Result<RespValue> {
        loop {
            let mut val = match self.get_op(src)? {
                Op::SimpleString => self.get_simple_string(src)?,
                Op::Error => self.get_error(src)?,
                Op::Integer => self.get_integer(src)?,
                Op::BulkString => self.get_bulk_string(src)?,
                Op::Array => match self.get_array_context(src)? {
                    None => Array(None),
                    Some(ctx) if ctx.is_complete() => ctx.items().into(),
                    Some(ctx) => {
                        self.stack.push(ctx);
                        self.op = None;
                        continue;
                    }
                },
            };

            self.op = None;

            loop {
                let Some(mut ctx) = self.stack.pop() else { return Ok(val) };

                ctx.push(val);
                if !ctx.is_complete() {
                    self.stack.push(ctx);
                    break;
                }

                val = ctx.items().into();
            }
        }
    }
}
