data capture, run socat and connect to localhost:6379 as if it was redis-server.internal redis server, replace redis-server.internal
```
socat -r Tx.bin -R Rx.bin TCP-LISTEN:6379,reuseaddr,fork,bind=0.0.0.0 TCP:redis-server.internal:6379

(connect your client) $ redis-cli -h 127.0.0.1

(in another terminal) $ truncate -s 0 Rx.bin Tx.bin

(execute test command) 127.0.0.1:6379> subscribe test_channel_1
CTRL+C

now Rx.bin and Tx.bin should have just the Rx and Tx test bytes

```

note - its important to leave a trailing CRLF on the end of the last line when carving out a test Rx and Tx, RESP requires these as frame markers, and using truncate -s 0 removes redis-cli chatter to help make a minimum test case


