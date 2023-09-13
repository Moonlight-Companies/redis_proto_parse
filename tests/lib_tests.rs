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

fn help_create_bulkstring(txt: &str) -> RespValue {
    RespValue::BulkString(Some(Box::from(txt.as_bytes().to_vec())))
}

fn help_create_simplestring(txt: &str) -> RespValue {
    RespValue::SimpleString(Box::from(txt.to_string()))
}

fn help_create_integer(num: i64) -> RespValue {
    RespValue::Integer(num)
}

fn help_create_array(values: Vec<RespValue>) -> RespValue {
    RespValue::Array(Some(values))
}

#[test]
fn test_ping_simple() {
    let rx_bytes = include_bytes!("../example_test_cases/ping_simple/Rx.bin");
    let tx_bytes = include_bytes!("../example_test_cases/ping_simple/Tx.bin");

    test_generic(&rx_bytes[..], vec![Box::new(
        help_create_simplestring("PONG")
    )]);
    
    test_generic(&tx_bytes[..], vec![Box::new(help_create_array(vec![
        help_create_bulkstring("ping"),
    ]))]);
}

#[test]
fn test_ping_bulk() {
    let rx_bytes = include_bytes!("../example_test_cases/ping_bulk/Rx.bin");
    let tx_bytes = include_bytes!("../example_test_cases/ping_bulk/Tx.bin");

    test_generic(&rx_bytes[..], vec![Box::new(help_create_bulkstring("hello world"))]);
    
    test_generic(&tx_bytes[..], vec![Box::new(help_create_array(vec![
        help_create_bulkstring("ping"),
        help_create_bulkstring("hello world"),
    ]))]);
}

#[test]
fn test_subscribe_single_channel() {
    let rx_bytes = include_bytes!("../example_test_cases/subscribe_single_channel/Rx.bin");
    let tx_bytes = include_bytes!("../example_test_cases/subscribe_single_channel/Tx.bin");

    test_generic(&rx_bytes[..], vec![Box::new(help_create_array(vec![
        help_create_bulkstring("subscribe"),
        help_create_bulkstring("test_channel_1"),
        help_create_integer(1),
    ]))]);

    test_generic(&tx_bytes[..], vec![Box::new(help_create_array(vec![
        help_create_bulkstring("subscribe"),
        help_create_bulkstring("test_channel_1"),
    ]))]);
}


#[test]
fn test_subscribe_multiple_channels() {
    let rx_bytes = include_bytes!("../example_test_cases/subscribe_multiple_channels/Rx.bin");
    let tx_bytes = include_bytes!("../example_test_cases/subscribe_multiple_channels/Tx.bin");

    test_generic(&rx_bytes[..], vec![Box::new(help_create_array(vec![
        help_create_bulkstring("subscribe"),
        help_create_bulkstring("test_channel_1"),
        help_create_integer(1),
    ])), Box::new(help_create_array(vec![
        help_create_bulkstring("subscribe"),
        help_create_bulkstring("test_channel_2"),
        help_create_integer(2),
    ])), Box::new(help_create_array(vec![
        help_create_bulkstring("subscribe"),
        help_create_bulkstring("test_channel_3"),
        help_create_integer(3),
    ]))]);
    
    test_generic(&tx_bytes[..], vec![Box::new(RespValue::Array(Some(vec![
        help_create_bulkstring("subscribe"),
        help_create_bulkstring("test_channel_1"),
        help_create_bulkstring("test_channel_2"),
        help_create_bulkstring("test_channel_3"),
    ])))]);
}
