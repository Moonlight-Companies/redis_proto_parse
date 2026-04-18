# publish_subscribe

Basic text pub/sub over a single channel using the high-level `Client` wrapper.

## What it demonstrates
- Connecting two clients (one subscriber, one publisher) to Redis
- `subscribe`/`publish` round-trip
- Draining messages via `Client::next()` in a reader task

## Run
Start a Redis instance on `127.0.0.1:6379`, then:

```
cargo run --example publish_subscribe
```

## Expected output
```
tx demo.text -> hello 0 (delivered to 1 subscribers)
rx demo.text <- hello 0
tx demo.text -> hello 1 (delivered to 1 subscribers)
rx demo.text <- hello 1
...
```

## Notes
- The subscriber task is spawned before the publisher connects, but we sleep 100ms to make sure Redis has processed the SUBSCRIBE before we PUBLISH. In production code you'd want a proper synchronization signal.
- `Client` opens two TCP connections under the hood — one for the subscribed reader, one for command replies — because a subscribed connection is restricted in what it can send.
