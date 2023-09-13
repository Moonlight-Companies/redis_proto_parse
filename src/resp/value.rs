use std::str;

#[derive(Debug, PartialEq, Eq)]
pub enum RespValue {
    SimpleString(Box<str>),
    SimpleError(Box<str>),
    Integer(i64),
    BulkString(Option<Box<[u8]>>),
    Array(Option<Vec<RespValue>>),
}

use RespValue::*;

impl RespValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            BulkString(Some(buf)) => str::from_utf8(&buf[..]).ok(),
            // todo: fill out this guy
            _ => None,
        }
    }

    pub fn as_buf(&self) -> Option<&str> {
        None
    }
}

pub fn array(values: Vec<RespValue>) -> RespValue {
    Array(Some(values))
}

pub fn simple(s: impl AsRef<str>) -> RespValue {
    SimpleString(Box::from(s.as_ref()))
}

pub fn err(s: impl AsRef<str>) -> RespValue {
    SimpleError(Box::from(s.as_ref()))
}

pub fn int(s: impl Into<i64>) -> RespValue {
    Integer(s.into())
}

pub fn bulk(bs: impl AsRef<[u8]>) -> RespValue {
    BulkString(Some(Box::from(bs.as_ref())))
}

pub const BULK_NONE: RespValue = BulkString(None);

impl From<Vec<RespValue>> for RespValue {
    fn from(value: Vec<RespValue>) -> Self {
        RespValue::Array(Some(value))
    }
}

pub const ARRAY_NONE: RespValue = Array(None);
