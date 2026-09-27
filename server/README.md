# server/

The process that hosts a world: **`mineworld-server`**, a crate and a binary. One authoritative
world, HTTP for control and status, WebSocket for the live connection.

```text
mineworld-server --listen 127.0.0.1:7878

GET /health   is this process up
GET /status   what the world is: time, entities, systems, seats, connected clients
GET /ws       the live connection
```

Each connected client gets what **its** observer perceives — not a world dump, and not a filtered
copy of one. It submits requests; the server allocates their identity and the world decides what
happens. A client can say two things: which seat it wants, and what it would like to happen.

Read next:

- [`PROTOCOL.md`](PROTOCOL.md) — the frames, in full. Write a client against that.
- [`docs/NETWORKING.md`](../docs/NETWORKING.md) — why the server is authoritative, and why there is
  only one binary.

The world this binary hosts is empty until a World Pack can be loaded, which is the next PR. The
crate is what a world is hosted *by*: `WorldHost::spawn` takes a closure that assembles the world,
its systems, its seats and its perception, and serves whatever it is given.
