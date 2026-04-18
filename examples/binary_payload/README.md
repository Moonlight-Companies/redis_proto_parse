# binary_payload

Arbitrary bytes (non-UTF-8) carried over Redis pub/sub by driving `RespCodec` directly.

## What it demonstrates
- Wrapping a raw `TcpStream` with `Framed<TcpStream, RespCodec>` to get a `Stream`/`Sink` of `RespValue`
- Matching pubsub control frames (`subscribe`/`message`) manually
- Using `RespValue::as_buf()` to extract the payload as `&[u8]` instead of `&str`
- Publishing raw bytes via `Sender::publish`, which accepts any `impl AsRef<[u8]>`

## Run
```
cargo run --example binary_payload
```

## Why bypass `Client::next()`?
`Client::next()` returns `(String, String)` — convenient, but panics the pubsub loop on non-UTF-8 payloads with `InvalidData`. If you have binary content (e.g. protobuf frames, compressed blobs, file chunks), work with `RespValue::as_buf()` directly as shown here.

## Payload examples shown
- `[0x00 0x01 0x02 0xde 0xad 0xbe 0xef]` — sentinel bytes, including null
- `[0xff; 16]` — all high-bit bytes, not valid UTF-8
- PNG magic prefix `\x89PNG\r\n\x1a\n` — note the embedded CRLF, which RESP carries without issue because bulk strings are length-prefixed

## Caveat
This example skips the idle-PING keepalive that `Client::next()` provides. For long-lived binary subscribers, you'll want to layer your own keepalive using the same `tokio::time::timeout` pattern: on idle, send `PING` via the sink half; treat any frame as liveness.
