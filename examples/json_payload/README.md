# json_payload

Carry serialized JSON across Redis pub/sub using the `Client` wrapper.

## What it demonstrates
- Serializing a Rust struct with `serde_json` and publishing the result as a UTF-8 bulk string
- Deserializing on the subscriber side
- That `Client::publish` accepts `impl AsRef<[u8]>` (here a `&str` from `serde_json::to_string`)

## Run
```
cargo run --example json_payload
```

## Why this works
RESP bulk strings are binary-safe byte blobs with an explicit length prefix. A JSON document — which is just UTF-8 — drops in without escaping or length negotiation. The subscriber receives a `(channel, payload)` pair where `payload` is the raw bytes the publisher sent, decoded as a `String` (valid UTF-8 expected).

## Caveat
`Client::next()` currently returns `(String, String)`, so it requires the payload to be valid UTF-8. JSON always is, so this is fine. For non-UTF-8 payloads, see the `binary_payload` example, which bypasses `Client` and drives the codec directly.
