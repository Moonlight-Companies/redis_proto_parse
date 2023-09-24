use std::str;

#[derive(PartialEq, Eq, Clone)]
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
            SimpleString(val) => Some(val),
            SimpleError(val) => Some(val),
            // no other types can be converted to a str
            _ => None,
        }
    }

    pub fn as_buf(&self) -> Option<&str> {
        None
    }
}

use std::fmt;
impl fmt::Debug for RespValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RespValue::SimpleString(val) => write!(f, "SimpleString({:?})", val),
            RespValue::SimpleError(val) => write!(f, "SimpleError({:?})", val),
            RespValue::Integer(val) => write!(f, "Integer({})", val),
            RespValue::BulkString(Some(buf)) => write!(f, "BulkString({:?})", String::from_utf8_lossy(buf)),
            RespValue::BulkString(None) => write!(f, "BulkString(None)"),
            RespValue::Array(Some(arr)) => write!(f, "Array<{}>({:?}))", arr.len(), arr),
            RespValue::Array(None) => write!(f, "Array(None)"),
        }
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
