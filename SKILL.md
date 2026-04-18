---
name: redis_proto_parse
description: Parse and encode the Redis RESP2 protocol in Rust. Provides a tokio_util Codec for streaming decode/encode plus an opinionated pub/sub client built on top. Use when consuming this crate from another Rust project.
---

# redis_proto_parse — integration skill

This skill teaches an AI (or a reader) how to pull `redis_proto_parse` into a Rust project and use it correctly. The crate has three API tiers; pick the lowest one that fits.

## When to use this crate

Reach for `redis_proto_parse` when you need:

- A **pub/sub-focused** Redis client in pure Rust on `tokio`
- A **streaming RESP codec** that integrates cleanly with `tokio_util::codec::Framed`
- Low-level **RESP value** encode/decode for parsing captured traffic, building proxies, or writing a custom transport

Do **not** use it for:

- General Redis command coverage (GET, SET, HSET, cluster routing, etc.) — use `redis` or `fred`
- RESP3 (the crate speaks RESP2 only; no `HELLO 3`, no push types, no maps/sets at the protocol level)
- TLS — the client opens plain TCP; wrap yourself if you need `rustls`

## API tiers

### Tier 1 — `client::Client` / `Sender` / `Receiver`

Opinionated pub/sub helpers. `Client` holds two connections (one subscribed, one command) and exposes `subscribe`, `psubscribe`, `publish`, and `next`. The subscribed connection runs an **idle-PING keepalive**: any received frame is treated as liveness; after a full idle window with no traffic, it sends `PING`; after `MAX_OUTSTANDING_PINGS` consecutive idle windows with no response it returns `TimedOut`.

```rust
use redis_proto_parse::client::Client;

let mut c = Client::new("127.0.0.1:6379").await?;
c.subscribe("events").await?;
c.publish("events", "hello").await?;
let (channel, payload) = c.next().await?;   // payload is String — requires UTF-8
```

Notes:

- `publish` accepts `impl AsRef<[u8]>`, so bytes work for the sender. The receiver side (`next`) returns `(String, String)` — for non-UTF-8 payloads, drop to Tier 2.
- `Client::split()` hands you the `Sender` / `Receiver` pair if you need to move them across tasks.
- Pattern subs: `psubscribe("chan.*")`; sharded pub/sub receive types (`smessage`/`ssubscribe`/`sunsubscribe`) are tolerated by `Receiver::next` but there are no `ssubscribe`-send methods on `Receiver` yet — send them manually via a `Framed` sink if you need them.

### Tier 2 — `Framed<TcpStream, RespCodec>`

The raw codec. You own command dispatch, the read loop, and keepalive.

```rust
use futures::{SinkExt, StreamExt};
use redis_proto_parse::resp::{value::*, RespCodec, RespValue};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

let stream = TcpStream::connect("127.0.0.1:6379").await?;
let mut framed = Framed::new(stream, RespCodec::default());

framed.send(vec![bulk("PING")].into()).await?;
let reply: Option<std::io::Result<RespValue>> = framed.next().await;
```

Use this tier for binary-safe receive, custom command dispatch (non–pub/sub), or a proxy. The codec is resumable: partial frames are buffered and completed on the next poll.

### Tier 3 — raw `RespDecoder` / `resp_encode`

Exposed as `redis_proto_parse::resp::{RespDecoder, encoder::resp_encode}`. Useful for parsing captured byte buffers (e.g., from `socat` dumps — see the project README) or writing a transport that isn't `tokio::net::TcpStream`.

```rust
use bytes::BytesMut;
use redis_proto_parse::resp::{RespCodec};
use tokio_util::codec::Decoder;

let mut codec = RespCodec::default();
let mut buf = BytesMut::from(&b"+PONG\r\n"[..]);
let value = codec.decode(&mut buf)?;   // Some(SimpleString("PONG"))
```

## RespValue construction

Free-function constructors live in `resp::value`:

| Function | Produces | Notes |
|---|---|---|
| `simple("OK")` | `SimpleString` | Must not contain CR/LF — encoder rejects |
| `err("WRONGTYPE …")` | `SimpleError` | Same CR/LF rule |
| `int(42)` | `Integer` | Takes `impl Into<i64>` |
| `bulk(&[u8] or &str)` | `BulkString(Some(_))` | Binary-safe, length-prefixed |
| `array(vec![…])` | `Array(Some(_))` | Nest freely |
| `BULK_NONE` / `ARRAY_NONE` | `$-1\r\n` / `*-1\r\n` | Null bulk / null array |

`From<Vec<RespValue>>` produces an `Array(Some(…))`, so `vec![bulk("SET"), bulk("k"), bulk("v")].into()` is a valid command.

### Accessors

- `RespValue::as_str() -> Option<&str>` — works on `SimpleString`, `SimpleError`, and UTF-8 `BulkString`
- `RespValue::as_buf() -> Option<&[u8]>` — same three variants but returns raw bytes; use for binary payloads

## Error handling contract

- Decode errors (`io::ErrorKind::InvalidData`) leave the codec in a partially consumed state. **Drop the `Framed` on decode error** — do not try to resume.
- Encode errors only fire for CR/LF in `SimpleString`/`SimpleError`. Safe to keep using the codec after.
- `Receiver::next` treats any decoded frame as liveness; it returns `TimedOut` only after `MAX_OUTSTANDING_PINGS` consecutive idle windows with no frames at all.

## Safety guarantees in the decoder

- Rejects negative bulk-string and array lengths other than the null sentinel (`-1`), preventing capacity-overflow panics on malformed input
- Caps the CRLF scan at `MAX_CRLF_SCAN` bytes (512 MB), so an attacker can't stall us scanning a pathologically long line
- Uses an iterative array stack, so deeply-nested arrays do not blow the native stack

## Dependencies the consumer inherits

From `Cargo.toml`:

```
bytes = "1.4"
futures = "0.3"
tokio = { version = "1.28", features = ["net", "macros", "time"] }
tokio-util = { version = "0.7", features = ["codec"] }
```

No TLS, no connection pool, no cluster support. If you add those, layer them on top.

## Common integration patterns

**Adding your own keepalive on top of Tier 2**: mirror the pattern from `Receiver::next` — wrap `framed.next()` in `tokio::time::timeout`, send `PING` as `vec![bulk("PING")].into()` on timeout, count consecutive pings without any frame, fail when the count crosses your threshold.

**Running a subscriber loop**:
```rust
loop {
    match client.next().await {
        Ok((channel, payload)) => handle(channel, payload),
        Err(e) if e.kind() == io::ErrorKind::TimedOut => reconnect().await?,
        Err(e) => return Err(e),
    }
}
```

**Parsing a pcap / socat capture**: feed bytes into a `RespCodec` in a loop, collecting `Ok(Some(v))` results and stopping on `Ok(None)` (partial frame at end of buffer) or `Err`.

## See also

- `examples/publish_subscribe/` — Tier 1 minimal pub/sub
- `examples/json_payload/` — Tier 1 with JSON serialization
- `examples/binary_payload/` — Tier 2 for non-UTF-8 payloads
- `example_test_cases/` — captured byte fixtures used by the test suite
