use redis_proto_parse::resp::{value::*, RespCodec};
use tokio_util::codec::Decoder;
use bytes::BytesMut;

fn test_generic(data: &[u8], expected_resp_value: RespValue) {
    let mut data_mut = BytesMut::from(data);
    let mut codec = RespCodec::default();

    match codec.decode(&mut data_mut) {
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

fn test_generic_multiple(data: &[u8], expected: Vec<Box<RespValue>>) {
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

    test_generic(&rx_bytes[..], simple("PONG"));
    
    test_generic(&tx_bytes[..], array(vec![
        bulk("ping"),
    ]));
}

#[test]
fn test_ping_bulk() {
    let rx_bytes = include_bytes!("../example_test_cases/ping_bulk/Rx.bin");
    let tx_bytes = include_bytes!("../example_test_cases/ping_bulk/Tx.bin");

    test_generic(&rx_bytes[..], bulk("hello world"));
    
    test_generic(&tx_bytes[..], array(vec![
        bulk("ping"),
        bulk("hello world"),
    ]));
}

#[test]
fn test_subscribe_single_channel() {
    let rx_bytes = include_bytes!("../example_test_cases/subscribe_single_channel/Rx.bin");
    let tx_bytes = include_bytes!("../example_test_cases/subscribe_single_channel/Tx.bin");

    test_generic(&rx_bytes[..], array(vec![
        bulk("subscribe"),
        bulk("test_channel_1"),
        int(1),
    ]));

    test_generic(&tx_bytes[..], array(vec![
        bulk("subscribe"),
        bulk("test_channel_1"),
    ]));
}

#[test]
fn test_subscribe_multiple_channels() {
    let rx_bytes = include_bytes!("../example_test_cases/subscribe_multiple_channels/Rx.bin");
    let tx_bytes = include_bytes!("../example_test_cases/subscribe_multiple_channels/Tx.bin");

    test_generic_multiple(&rx_bytes[..], vec![Box::new(array(vec![
        bulk("subscribe"),
        bulk("test_channel_1"),
        int(1),
    ])), Box::new(array(vec![
        bulk("subscribe"),
        bulk("test_channel_2"),
        int(2),
    ])), Box::new(array(vec![
        bulk("subscribe"),
        bulk("test_channel_3"),
        int(3),
    ]))]);
    
    test_generic_multiple(&tx_bytes[..], vec![Box::new(array(vec![
        bulk("subscribe"),
        bulk("test_channel_1"),
        bulk("test_channel_2"),
        bulk("test_channel_3"),
    ]))]);
}
