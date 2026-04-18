# redis_proto_parse

A small Rust library for speaking the Redis RESP2 protocol over `tokio`. Three layers:

1. **`client::Client`** — opinionated pub/sub helpers with idle-PING keepalive
2. **`RespCodec`** — a `tokio_util` Codec you can drop into `Framed<TcpStream, _>`
3. **`RespDecoder` / `resp_encode`** — raw byte-level encode/decode for custom transports

## Quick start

```rust
use redis_proto_parse::client::Client;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut c = Client::new("127.0.0.1:6379").await?;
    c.subscribe("events").await?;
    c.publish("events", "hello").await?;

    let (channel, payload) = c.next().await?;
    println!("{channel}: {payload}");
    Ok(())
}
```

Run the bundled examples (Redis on `127.0.0.1:6379`):

```
cargo run --example publish_subscribe
cargo run --example json_payload
cargo run --example binary_payload
```

See [`examples/README.md`](examples/README.md) for what each one covers, and [`SKILL.md`](SKILL.md) for an AI-oriented integration guide.

## Scope

- ✅ RESP2 encode/decode, streaming and resumable
- ✅ Pub/sub client (subscribe, psubscribe, publish) with PING keepalive
- ✅ Binary-safe bulk strings
- ❌ RESP3 (no `HELLO 3`, push types, etc.)
- ❌ General command coverage — for that use `redis` or `fred`
- ❌ TLS, connection pooling, cluster routing

## Capturing traffic with `socat`

To build regression fixtures (like the ones in `example_test_cases/`), tap a real session:

```
socat -r Tx.bin -R Rx.bin TCP-LISTEN:6379,reuseaddr,fork,bind=0.0.0.0 TCP:redis-server.internal:6379

# client terminal
redis-cli -h 127.0.0.1

# capture terminal: reset to drop connection chatter
truncate -s 0 Rx.bin Tx.bin

# client terminal: run the one command you want to capture
127.0.0.1:6379> subscribe test_channel_1
# ^C when done
```

`Rx.bin` and `Tx.bin` now contain the minimum bytes for that exchange. Leave a trailing CRLF on the last line — RESP requires it as a frame marker, and `truncate -s 0` between runs removes `redis-cli` handshake chatter.

## Layout

```
src/
  resp/
    decoder.rs   resumable decoder, iterative array stack, size caps
    encoder.rs   RespValue -> bytes, rejects CRLF in simple strings
    value.rs     RespValue enum + constructors (simple, err, int, bulk, array)
    mod.rs       RespCodec impl (Decoder + Encoder for RespValue)
  client.rs      Sender / Receiver / Client pub-sub wrapper
tests/           integration tests (encode, framing, value ops, fixtures)
examples/        runnable examples — see examples/README.md
example_test_cases/  captured Rx/Tx byte fixtures
```
