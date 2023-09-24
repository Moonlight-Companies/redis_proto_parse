use redis_proto_parse::resp::{value, RespCodec, encoder};
use tokio_util::codec::Decoder;
use bytes::BytesMut;

#[macro_export]
macro_rules! test_encode_decode {
    ($encodefn:expr, $encodeval:expr, $encodestr:expr) => {
        {
            let input=$encodefn($encodeval);
            let mut data = BytesMut::new();
            encoder::resp_encode(input, &mut data);
            assert_eq!(data, BytesMut::from($encodestr));

            let mut codec = RespCodec::default();
            match codec.decode(&mut data) {
                Ok(Some(resp_value)) => {
                    assert_eq!(resp_value, $encodefn($encodeval));
                }
                Ok(None) => {
                    panic!("Unexpected EOF");
                }
                Err(e) => {
                    panic!("An error occurred: {:?}", e);
                }
            }
        }
    };
}

#[test]
fn test_simple_encode_value_bulk() {
    test_encode_decode!(value::bulk, "bulk", "$4\r\nbulk\r\n")
}

#[test]
fn test_simple_encode_value_int() {
    test_encode_decode!(value::int, 42, ":42\r\n")
}

#[test]
fn test_simple_encode_value_simple() {
    test_encode_decode!(value::simple, "pass", "+pass\r\n")
}

#[test]
fn test_simple_encode_value_err() {
    test_encode_decode!(value::err, "fail", "-fail\r\n")
}

#[test]
fn test_simple_encode_value_array() {
    test_encode_decode!(value::array, vec![], "*0\r\n")
}
