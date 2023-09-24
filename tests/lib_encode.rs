use redis_proto_parse::resp::{value, RespCodec, encoder};
use tokio_util::codec::Decoder;
use bytes::BytesMut;

#[test]
fn test_simple_encode_value_bulk() {
    let input=value::bulk("bulk");
    let mut data = BytesMut::new();

    encoder::resp_encode(input, &mut data);

    assert_eq!(data, BytesMut::from("$4\r\nbulk\r\n"));
}

#[test]
fn test_simple_encode_value_int() {
    let input=value::int(42);
    let mut data = BytesMut::new();

    encoder::resp_encode(input, &mut data);

    assert_eq!(data, BytesMut::from(":42\r\n"));
}

#[test]
fn test_simple_encode_value_simple() {
    let input=value::simple("test");
    let mut data = BytesMut::new();

    encoder::resp_encode(input, &mut data);

    assert_eq!(data, BytesMut::from("+test\r\n"));
}

#[test]
fn test_simple_encode_value_err() {
    let input=value::err("fail");
    let mut data = BytesMut::new();

    encoder::resp_encode(input, &mut data);

    assert_eq!(data, BytesMut::from("-fail\r\n"));
}

#[test]
fn test_simple_encode_value_array() {
    let input=value::array(vec![]);
    let mut data = BytesMut::new();

    encoder::resp_encode(input, &mut data);

    assert_eq!(data, BytesMut::from("*0\r\n"));
}

#[test]
fn test_simple_encode_values() {
    // test to make sure encoded values produce the expected results in basic cases
    // lots of other tests use encode on the input and output side of tests
    let v=value::array(vec![
        value::bulk("bulk"),
        value::int(1),
        value::simple("foo"),
        value::err("bar"),
    ]);

    let mut data = BytesMut::new();

    encoder::resp_encode(v, &mut data);

    assert_eq!(data, BytesMut::from("*4\r\n$4\r\nbulk\r\n:1\r\n+foo\r\n-bar\r\n"));
}
