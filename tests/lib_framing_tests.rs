use redis_proto_parse::resp::{value, RespCodec};
use tokio_util::codec::Decoder;
use bytes::BytesMut;

#[test]
fn test_missing_frame_terminator() {
    let mut rx = BytesMut::from(&vec![ 0x2b, 0x50, 0x4f, 0x4e, 0x47 ][..]);

    let mut codec = RespCodec::default();

    match codec.decode(&mut rx) {
        Ok(Some(_)) => {
            // no value should return, test data has no CRLF
            panic!("unexpected value");
        }
        Ok(None) => {
            // op should be equal to SimpleString, remaining buffer should be "PONG", codec waiting for CRLF
            assert_eq!(rx.len(), 4 as usize);
        }
        Err(e) => {
            panic!("An error occurred: {:?}", e);
        }
    }
}

#[test]
fn test_bad_op() {
    let skip= vec![b'+', b'-', b':',  b'$', b'*'];

    // test each opcode byte from 0..=255 excluding actual opcodes
    for i in 0..=255 {
        if skip.contains(&i) {
            continue;
        }
        let mut data = BytesMut::from(&vec![ i as u8, 0x50, 0x4f, 0x4e, 0x47 ][..]);
        let mut codec = RespCodec::default();
        match codec.decode(&mut data) {
            Ok(Some(_)) => {
                // no value should return, test data has no CRLF
                panic!("expected error because of invalid op (some)")
            }
            Ok(None) => {
                // should not get None here because there is a byte (i) and no op set
                panic!("expected error because of invalid op (none)")
            }
            Err(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
                assert_eq!(e.to_string(), format!("invalid opcode byte: {:#04x}", i));
            }
        }
    }
}

#[test]
fn test_remaining_buffer_len() {
    let mut rx = BytesMut::from(&vec![ b'$', b'4', b'\r', b'\n', b'T', b'E', b'S', b'T', b'\r', b'\n', b'$' ][..]);

    let mut codec = RespCodec::default();

    match codec.decode(&mut rx) {
        Ok(Some(v)) => {
            assert_eq!(v, value::bulk("TEST"));
            assert_eq!(rx.len(), 1 as usize);
        }
        Ok(None) => {
            assert!(false, "Decode returned None, but a value was expected.");
        }
        Err(e) => {
            assert!(false, "An error occurred while decoding: {:?}", e);
        }
    }
}