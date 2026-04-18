# Examples

Runnable examples for `redis_proto_parse`. Each one expects a Redis server on `127.0.0.1:6379`.

| Example | What it shows |
|---|---|
| [`publish_subscribe`](publish_subscribe/) | Minimal text pub/sub using the `Client` wrapper |
| [`json_payload`](json_payload/) | Serializing a Rust struct as JSON and shipping it through pub/sub |
| [`binary_payload`](binary_payload/) | Non-UTF-8 byte payloads by driving `RespCodec` directly |

Run any example with:

```
cargo run --example <name>
```

## Which API tier fits your use case?

- **`Client` / `Sender` / `Receiver`** (`src/client.rs`) — opinionated pub/sub helpers with idle-PING keepalive. Payloads go in as `impl AsRef<[u8]>`; messages come out as `(String, String)`, so payloads must be valid UTF-8 on the receive side.
- **`Framed<TcpStream, RespCodec>`** (`src/resp/mod.rs`) — raw encoder/decoder. You own the command dispatch, the read loop, and any keepalive. Use this for binary payloads, custom commands, or when you want direct control over framing.
- **`RespDecoder` / `resp_encode`** — the lowest tier, useful for parsing captured traffic (see the top-level README) or building your own transport.
