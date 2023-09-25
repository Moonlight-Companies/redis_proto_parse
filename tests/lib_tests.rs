use redis_proto_parse::resp::{value, RespCodec};
use tokio_util::codec::Decoder;
use bytes::BytesMut;

fn test_generic(data: &mut BytesMut, expected_resp_value: value::RespValue) {
    let mut codec = RespCodec::default();

    match codec.decode(data) {
        Ok(Some(resp_value)) => {
            assert_eq!(resp_value, expected_resp_value);
        }
        Ok(None) => {
            panic!("Unexpected EOF");
        }
        Err(e) => {
            panic!("An error occurred: {:?}", e);
        }
    }
}

fn test_generic_multiple(data: &mut BytesMut, expected: Vec<value::RespValue>) {
    for expected_resp_value in expected {
        test_generic(data, expected_resp_value)
    }
}

#[macro_export]
macro_rules! prepare_data {
    ($path:expr) => {
        (
            bytes::BytesMut::from(&include_bytes!(concat!("../example_test_cases/", $path, "/Rx.bin"))[..]),
            bytes::BytesMut::from(&include_bytes!(concat!("../example_test_cases/", $path, "/Tx.bin"))[..]),
        )
    };
}

#[test]
fn test_ping_simple() {
    let (mut rx, mut tx) = prepare_data!("ping_simple");

    test_generic(&mut rx, value::simple("PONG"));

    test_generic(&mut tx, value::array(vec![
        value::bulk("ping"),
    ]));
}

#[test]
fn test_ping_bulk() {
    let (mut rx, mut tx) = prepare_data!("ping_bulk");

    test_generic(&mut rx, value::bulk("hello world"));

    test_generic(&mut tx, value::array(vec![
        value::bulk("ping"),
        value::bulk("hello world"),
    ]));
}

#[test]
fn test_subscribe_single_channel() {
    let (mut rx, mut tx) = prepare_data!("subscribe_single_channel");

    test_generic(&mut rx, value::array(vec![
        value::bulk("subscribe"),
        value::bulk("test_channel_1"),
        value::int(1),
    ]));

    test_generic(&mut tx, value::array(vec![
        value::bulk("subscribe"),
        value::bulk("test_channel_1"),
    ]));
}

#[test]
fn test_subscribe_multiple_channels() {
    let (mut rx, mut tx) = prepare_data!("subscribe_multiple_channels");

    test_generic_multiple(&mut rx, vec![value::array(vec![
        value::bulk("subscribe"),
        value::bulk("test_channel_1"),
        value::int(1),
    ]), value::array(vec![
        value::bulk("subscribe"),
        value::bulk("test_channel_2"),
        value::int(2),
    ]), value::array(vec![
        value::bulk("subscribe"),
        value::bulk("test_channel_3"),
        value::int(3),
    ])]);

    test_generic_multiple(&mut tx, vec![value::array(vec![
        value::bulk("subscribe"),
        value::bulk("test_channel_1"),
        value::bulk("test_channel_2"),
        value::bulk("test_channel_3"),
    ])]);
}

#[test]
fn test_debug_fmt() {
    let v=value::array(vec![
        value::bulk("subscribe"),
        value::bulk("test_channel_1"),
        value::int(1),
        value::simple("foo"),
        value::err("bar"),
    ]);

    assert_eq!(format!("{:?}", v), "Array<5>([BulkString(\"subscribe\"), BulkString(\"test_channel_1\"), Integer(1), SimpleString(\"foo\"), SimpleError(\"bar\")]))")
}
