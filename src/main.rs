use std::str;
use std::io::Cursor;
use std::io::Read;
use std::io::{ Seek, SeekFrom };

const REDIS_PROTO_STRING: u8=b'$';
const REDIS_PROTO_LIST: u8=b'*';
const REDIS_PROTO_INT: u8=b':';
const REDIS_PROTO_OK: u8=b'+';
const REDIS_PROTO_ERROR: u8=b'-';
const REDIS_PROTO_CR: u8=13;
const REDIS_PROTO_LF: u8=10;


pub struct RedisProtocol {
    remaining_data: Vec<u8>,
}

impl RedisProtocol {
    pub fn new() -> Self {
        RedisProtocol {
            remaining_data: Vec::new(),
        }
    }

    pub fn more_data(&mut self, data: Vec<u8>) -> Result<usize, &'static str> {
        self.remaining_data.extend(data);
        if self.remaining_data.len() > (1024*1024*512) {
            // 512mb of unparsable data
            return Err("protocol error")
        }
        return Ok(self.remaining_data.len())
    }

    pub fn consume(&mut self) -> Option<RedisValue> {
        let mut cursor = Cursor::new(&self.remaining_data[..]);
        let result = RedisProtocol::read_redis(&mut cursor);
        
        if result.is_some() {
            let pos = cursor.position() as usize;

            self.remaining_data.drain(..pos);
        }

        result
    }

    fn read_length(cursor: &mut Cursor<&[u8]>) -> Option<i32> {
        let mut buf = Vec::new();
        let mut byte = [0; 1];
        loop {
            if let Ok(()) = cursor.read_exact(&mut byte) {
                if byte[0] == REDIS_PROTO_CR {
                    if let Ok(()) = cursor.read_exact(&mut byte) {
                        if byte[0] == REDIS_PROTO_LF {
                            break
                        }
                    } else {
                        return None
                    }
                }
                buf.extend(&byte);
            } else {
                return None
            }
        }

        let value: i32 = str::from_utf8(&buf).ok()?.trim().parse().ok()?;

        //println!("read_length: {}", value);

        Some(value)
    }

    fn remove_crlf(cursor: &mut Cursor<&[u8]>) -> Option<bool> {
        let mut crlf = [0; 2];
        if let Ok(()) = cursor.read_exact(&mut crlf) {
            if crlf[0] == REDIS_PROTO_CR && crlf[1] == REDIS_PROTO_LF {
                return Some(true)
            } else {
                panic!("protocol error - no crlf {:?}", crlf);
            }
        }

        None
    }

    fn read_redis(cursor: &mut Cursor<&[u8]>) -> Option<RedisValue> {
        let mut buf = [0u8; 1];
        cursor.read_exact(&mut buf).ok()?;
        //println!("ENTER RedisProtocol::redis_read <{}>", buf[0]);
        match buf[0] {
            REDIS_PROTO_STRING => {
                //println!("RedisProtocol::redis_read::REDIS_PROTO_STRING");
                let string = RedisProtocol::read_redis_string(cursor)?;
                return Some(RedisValue::RedisString(string));
            },
            REDIS_PROTO_INT => {
                //println!("RedisProtocol::redis_read::REDIS_PROTO_INT");
                let integer = RedisProtocol::read_redis_int(cursor)?;
                return Some(RedisValue::RedisInt(integer));
            },
            REDIS_PROTO_LIST => {
                //println!("RedisProtocol::redis_read::REDIS_PROTO_LIST");
                return Some(RedisProtocol::read_redis_list(cursor)?);
            },
            REDIS_PROTO_OK => {
                //println!("RedisProtocol::redis_read::REDIS_PROTO_OK");
                let string = RedisProtocol::read_redis_generic_crlf_string(cursor)?;
                return Some(RedisValue::RedisOk(string));
            }
            REDIS_PROTO_ERROR => {
                //println!("RedisProtocol::redis_read::REDIS_PROTO_ERROR");
                let string = RedisProtocol::read_redis_generic_crlf_string(cursor)?;
                return Some(RedisValue::RedisError(string));
            }
            _ => {
                panic!("RedisProtocol::redis_read::REDIS_PROTO_ERROR [{:?}]", buf[0]);
            }
        }
    }

    fn read_redis_list(cursor: &mut Cursor<&[u8]>) -> Option<RedisValue> {
        let mut parts = Vec::new();

        let num_parts=RedisProtocol::read_length(cursor)?;

        if num_parts == -1 {
            // null list has "*-1\r\n"
            return Some(RedisValue::RedisList(parts))
        }

        for _ in 0..num_parts {
            if let Some(v)=RedisProtocol::read_redis(cursor) {
                parts.push(v);
            } else {
                return None
            }
        }

        Some(RedisValue::RedisList(parts))
    }

    fn read_redis_string(cursor: &mut Cursor<&[u8]>) -> Option<String> {
        let string_length=RedisProtocol::read_length(cursor)?;

        if string_length == -1 {
            // "null" string has "$-1\r\n"
            return Some("".into());
        }

        let mut buf = vec![0; string_length as usize];
        cursor.read_exact(&mut buf).ok()?;

        RedisProtocol::remove_crlf(cursor)?;
    
        let string = str::from_utf8(&buf).ok()?.trim().to_string();

        //println!("REDIS_PROTO_STRING [{};{:?}]", string_length, string);

        Some(string)
    }

    fn read_redis_int(cursor: &mut Cursor<&[u8]>) -> Option<i32> {
        let mut buf = Vec::new();
        let mut byte = [0; 1];
        loop {
            if let Ok(()) = cursor.read_exact(&mut byte) {
                if byte[0] == REDIS_PROTO_CR {
                    if let Ok(()) = cursor.read_exact(&mut byte) {
                        if byte[0] == REDIS_PROTO_LF {
                            break
                        } else {
                            return None
                        }
                    }
                } else {
                    buf.extend(&byte);
                }
            } else {
                return None
            }
        }

        let value: i32 = str::from_utf8(&buf).ok()?.trim().parse().ok()?;
        //println!("REDIS_PROTO_INT: {}", value);
        Some(value)
    }

    fn read_redis_generic_crlf_string(cursor: &mut Cursor<&[u8]>) -> Option<String> {
        let mut buf: Vec<u8> = Vec::new();
        let mut byte = [0; 1];
        loop {
            if let Ok(()) = cursor.read_exact(&mut byte) {
                //println!("RedisProtocol::read_redis_generic_crlf_string: {:?}", byte);
                if byte[0] == REDIS_PROTO_CR {
                    if let Ok(()) = cursor.read_exact(&mut byte) {
                        if byte[0] == REDIS_PROTO_LF {
                            break;
                        } else {
                            return None
                        }
                    }
                } else {
                    buf.extend(&byte);
                }
            } else {
                return None;
            }
        }

        println!("read_redis_crlf BUF:{:?}", buf);

        Some(str::from_utf8(&buf).ok()?.trim().to_string())
    }
}

pub enum RedisValue {
    RedisString(String),
    RedisInt(i32),
    RedisList(Vec<RedisValue>),
    RedisOk(String),
    RedisError(String)
}

impl RedisValue {
    fn debugprint(&self) {
        self._debugprint(0)
    }

    fn _debugprint(&self, depth: usize) {
        print!("{}", std::iter::repeat("  ").take(depth).collect::<String>());
        match self {
            RedisValue::RedisString(value) => println!("RedisString: {};{}", value.len(), value),
            RedisValue::RedisInt(value) => println!("RedisInt: {}", value),
            RedisValue::RedisList(value) => {
                println!("RedisList: {}", value.len());
                for item in value {
                    item._debugprint(depth + 1 as usize);
                }
            }
            RedisValue::RedisOk(value) => println!("OK! {}", value),
            RedisValue::RedisError(value) => println!("ERROR! {}", value),
        }
    }
}

fn main() {
    let data=include_bytes!("proto_traffic.bin");
    
    let mut protocol = RedisProtocol::new();

    // example subscribe
    // *2
    // $9
    // subscribe
    // $27    <--- length of string below
    // groupbroadcast::control::46        <---- channel name

    //protocol.more_data(data.to_vec());

    // // test the whole stream at once
    // loop {
    //     if let Some(cmd)=protocol.consume() {
    //         cmd.debugprint();
    //     } else {
    //         break;
    //     }
    // }

    // test one byte at a time
    for byte in data.iter() {
        protocol.more_data(vec![*byte]);
        //println!("append more data {}", protocol.more_data(vec![*byte]).expect("err".into()));
        loop {
            if let Some(cmd)=protocol.consume() {
                cmd.debugprint();
            } else {
                break;
            }
        }
    }
}
