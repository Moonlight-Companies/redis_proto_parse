* common test cases
```
Once the client enters the subscribed state, it is not supposed to issue any other commands, except for additional SUBSCRIBE, SSUBSCRIBE, PSUBSCRIBE, UNSUBSCRIBE, SUNSUBSCRIBE, PUNSUBSCRIBE, PING, RESET, and QUIT commands. However, if RESP3 is used (see HELLO), it is possible for a client to issue any commands while in the subscribed state.
```

**List of commands:**
```
- SUBSCRIBE (single, multi)
- SSUBSCRIBE (single, multi)
- PSUBSCRIBE (single, multi)
- UNSUBSCRIBE (single, multi)
- SUNSUBSCRIBE (single, multi)
- PUNSUBSCRIBE (single, multi)
- PING (plain, with argument)
- RESET (expects `+RESET<CRLF>` simple string. Refer to the [Redis RESET command](https://redis.io/commands/reset/) documentation.)
- QUIT
```