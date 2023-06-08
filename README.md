data capture, run socat and connect to localhost:6379 as if it was the redis server
```
socat -r Tx.bin -R Rx.bin TCP-LISTEN:6379,reuseaddr,fork,bind=0.0.0.0 TCP:bus.dev.moonlightcompanies.com:6379
```

replay Rx.bin

