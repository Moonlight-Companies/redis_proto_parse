use bytes::BytesMut;
use redis_proto_parse::resp::{value, RespCodec};
use tokio_util::codec::Decoder;

#[test]
fn test_op_simplestring() {
    let mut rx = BytesMut::from(&vec![b'+', b'T', b'E', b'S', b'T', b'\r', b'\n'][..]);

    let mut codec = RespCodec::default();

    match codec.decode(&mut rx) {
        Ok(Some(v)) => {
            assert_eq!(v, value::simple("TEST"));
        }
        Ok(None) => {
            assert!(false, "Decode returned None, but a value was expected.");
        }
        Err(e) => {
            assert!(false, "An error occurred while decoding: {:?}", e);
        }
    }
}

#[test]
fn test_op_error() {
    let mut rx = BytesMut::from(&vec![b'-', b'T', b'E', b'S', b'T', b'\r', b'\n'][..]);

    let mut codec = RespCodec::default();

    match codec.decode(&mut rx) {
        Ok(Some(v)) => {
            assert_eq!(v, value::err("TEST"));
        }
        Ok(None) => {
            assert!(false, "Decode returned None, but a value was expected.");
        }
        Err(e) => {
            assert!(false, "An error occurred while decoding: {:?}", e);
        }
    }
}

#[test]
fn test_op_int() {
    let mut rx = BytesMut::from(&vec![b':', b'4', b'2', b'\r', b'\n'][..]);

    let mut codec = RespCodec::default();

    match codec.decode(&mut rx) {
        Ok(Some(v)) => {
            assert_eq!(v, value::int(42));
        }
        Ok(None) => {
            assert!(false, "Decode returned None, but a value was expected.");
        }
        Err(e) => {
            assert!(false, "An error occurred while decoding: {:?}", e);
        }
    }
}

#[test]
fn test_op_bulkstring() {
    let mut rx = BytesMut::from(
        &vec![
            b'$', b'4', b'\r', b'\n', b'T', b'E', b'S', b'T', b'\r', b'\n',
        ][..],
    );

    let mut codec = RespCodec::default();

    match codec.decode(&mut rx) {
        Ok(Some(v)) => {
            assert_eq!(v, value::bulk("TEST"));
        }
        Ok(None) => {
            assert!(false, "Decode returned None, but a value was expected.");
        }
        Err(e) => {
            assert!(false, "An error occurred while decoding: {:?}", e);
        }
    }
}

#[test]
fn test_op_array() {
    let mut rx = BytesMut::from(
        &vec![
            b'*', b'1', b'\r', b'\n', b'$', b'4', b'\r', b'\n', b'T', b'E', b'S', b'T', b'\r',
            b'\n',
        ][..],
    );

    let mut codec = RespCodec::default();

    match codec.decode(&mut rx) {
        Ok(Some(v)) => {
            assert_eq!(v, value::array(vec![value::bulk("TEST")]));
        }
        Ok(None) => {
            assert!(false, "Decode returned None, but a value was expected.");
        }
        Err(e) => {
            assert!(false, "An error occurred while decoding: {:?}", e);
        }
    }
}
