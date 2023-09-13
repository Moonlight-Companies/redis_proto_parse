use redis_proto_parse::resp::{value::*, RespCodec};
use tokio_util::codec::Decoder;
use bytes::BytesMut;

fn test_generic(data: &[u8], expected: Vec<Box<RespValue>>) {
    let mut data_mut = BytesMut::from(data);
    let mut codec = RespCodec::default();

    for expected_resp_value in expected {
        match codec.decode(&mut data_mut) {
            Ok(Some(resp_value)) => {
                assert_eq!(resp_value, *expected_resp_value);
            }
            Ok(None) => {
                panic!("Unexpected EOF");
            }
            Err(e) => {
                panic!("An error occurred: {:?}", e);
            }
        }
    }
}

#[test]
fn test_ping_simple() {
    let rx_bytes = include_bytes!("../example_test_cases/ping_simple/Rx.bin");
    let tx_bytes = include_bytes!("../example_test_cases/ping_simple/Tx.bin");

    test_generic(&rx_bytes[..], vec![Box::new(RespValue::SimpleString(Box::from("PONG".to_string())))]);
    
    test_generic(&tx_bytes[..], vec![Box::new(RespValue::Array(Some(vec![
        RespValue::BulkString(Some(Box::from("ping".as_bytes().to_vec()))),
    ])))]);
}

#[test]
fn test_ping_bulk() {
    let rx_bytes = include_bytes!("../example_test_cases/ping_bulk/Rx.bin");
    let tx_bytes = include_bytes!("../example_test_cases/ping_bulk/Tx.bin");

    test_generic(&rx_bytes[..], vec![Box::new(RespValue::BulkString(Some(Box::from("hello world".as_bytes().to_vec()))))]);
    
    test_generic(&tx_bytes[..], vec![Box::new(RespValue::Array(Some(vec![
        RespValue::BulkString(Some(Box::from("ping".as_bytes().to_vec()))),
        RespValue::BulkString(Some(Box::from("hello world".as_bytes().to_vec()))),
    ])))]);
}

#[test]
fn test_subscribe_multiple_channels() {
    let rx_bytes = include_bytes!("../example_test_cases/subscribe_multiple_channels/Rx.bin");
    let tx_bytes = include_bytes!("../example_test_cases/subscribe_multiple_channels/Tx.bin");

    test_generic(&rx_bytes[..], vec![Box::new(RespValue::Array(Some(vec![
        RespValue::BulkString(Some(Box::from("subscribe".as_bytes().to_vec()))),
        RespValue::BulkString(Some(Box::from("test_channel_1".as_bytes().to_vec()))),
        RespValue::Integer(1),
    ]))), Box::new(RespValue::Array(Some(vec![
        RespValue::BulkString(Some(Box::from("subscribe".as_bytes().to_vec()))),
        RespValue::BulkString(Some(Box::from("test_channel_2".as_bytes().to_vec()))),
        RespValue::Integer(2),
    ]))), Box::new(RespValue::Array(Some(vec![
        RespValue::BulkString(Some(Box::from("subscribe".as_bytes().to_vec()))),
        RespValue::BulkString(Some(Box::from("test_channel_3".as_bytes().to_vec()))),
        RespValue::Integer(3),
    ])))]);
    
    test_generic(&tx_bytes[..], vec![Box::new(RespValue::Array(Some(vec![
        RespValue::BulkString(Some(Box::from("subscribe".as_bytes().to_vec()))),
        RespValue::BulkString(Some(Box::from("test_channel_1".as_bytes().to_vec()))),
        RespValue::BulkString(Some(Box::from("test_channel_2".as_bytes().to_vec()))),
        RespValue::BulkString(Some(Box::from("test_channel_3".as_bytes().to_vec()))),
    ])))]);
}
