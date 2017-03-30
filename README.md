
Distributed key-value store
============================

[![Build Status](https://travis-ci.org/PureW/tokio-key-value-store.svg?branch=master)](https://travis-ci.org/PureW/tokio-key-value-store)

Introduction
------------

Simple daemon implementing (a subset) of Redis' [RESP-protocol](https://redis.io/topics/protocol) built on top of [Tokio](https://tokio.rs/).

In order to not need a client, the [inline-commands](https://redis.io/topics/protocol#inline-commands) are used.

Example
-------

Launch the daemon:
```bash
cargo run
```

In a second terminal:
```bash
$ telnet localhost 12345
...
$ set foo bar<enter>
+OK
$ badcommand
-ERR unknown command
```
