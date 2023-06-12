use std::io;

#[derive(Debug)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Option<Vec<RespValue>>),
}


// pretty much the same as vec but typed so that inner values can use .into()
#[macro_export]
macro_rules! resp {
    ($($t:expr),*) => {
        {
            let v: Vec<RespValue> = vec![$($t.into()),*];
            v
        }
    };
}

impl From<String> for RespValue {
    fn from(value: String) -> Self {
        RespValue::SimpleString(value)
    }
}

impl From<&'_ str> for RespValue {
    fn from(value: &str) -> Self {
        RespValue::SimpleString(value.into())
    }
}

impl<T: ToString, E: ToString> From<Result<T, E>> for RespValue {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(ok) => RespValue::SimpleString(ok.to_string()),
            Err(err) => RespValue::Error(err.to_string()),
        }
    }
}

macro_rules! impl_int {
    ($($t:ty),*) => {
        $(
            impl From<$t> for RespValue {
                fn from(value: $t) -> Self {
                    RespValue::Integer(value as i64)
                }
            }
        )*
    };
}

impl_int!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl From<Option<Vec<u8>>> for RespValue {
    fn from(value: Option<Vec<u8>>) -> Self {
        RespValue::BulkString(value)
    }
}

impl From<Vec<u8>> for RespValue {
    fn from(value: Vec<u8>) -> Self {
        Some(value).into()
    }
}

impl From<Option<Vec<RespValue>>> for RespValue {
    fn from(value: Option<Vec<RespValue>>) -> Self {
        RespValue::Array(value)
    }
}

impl From<Vec<RespValue>> for RespValue {
    fn from(value: Vec<RespValue>) -> Self {
        Some(value).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn takes_resp(rv: impl Into<RespValue>) {}

    #[test]
    fn convert() {
        takes_resp("test");
        takes_resp(Ok::<_, &str>("test"));
        takes_resp(Err::<&str, _>("test"));
        takes_resp(1);
        takes_resp(Some(vec![1, 2, 3]));
        takes_resp(resp!["test"]);
    }
}