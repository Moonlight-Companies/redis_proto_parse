data capture, run socat and connect to localhost:6379 as if it was redis-server.internal redis server, replace redis-server.internal
```
socat -r Tx.bin -R Rx.bin TCP-LISTEN:6379,reuseaddr,fork,bind=0.0.0.0 TCP:redis-server.internal:6379
```

replay Rx.bin

