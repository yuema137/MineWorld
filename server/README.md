# server/

What hosts a world: **`mineworld-server`**, a library. One authoritative world, HTTP for control and
status, WebSocket for the live connection.

```text
mineworld server worlds/social-cafe --listen 127.0.0.1:7878
[mineworld] invite 3f9c… — join with: 127.0.0.1:7878 seat=alice invite=3f9c…

GET /health   is this process up
GET /status   what the world is: time, entities, systems and their verbs, seats, connected clients
GET /ws       the live connection
```

Every client joins with the server's invite and a nickname — on loopback too. Give one with
`--invite TOKEN` or `MINEWORLD_INVITE`, or let the server make one and print it, as above.

The binary is [`tools/cli`](../tools/cli), not this crate, and deliberately: hosting a world means
loading a World Pack, which means installing System Packs, and a transport that depended on those
would invert the one-way dependency rule. `WorldHost::spawn` takes a closure that assembles a world
and serves whatever it is given — the CLI is what supplies one.

A hosted world may have a save (`HostedWorld::persisted`, `--save DIR` on the command line). Then
every request is written to it before it is answered, and each frame tells the client which saved
`revision` of the world it describes.

One controller per seat. `--agent SEAT` and `--town` drive seats with in-server controllers on the
world thread, so a hosted town lives with nobody connected; a player joining such a seat simply takes
it over, and gives it back on leaving. A dropped socket's seat is held (`--hold`, 30 s) for the
`resume` its welcome carried; `take_over: true` takes a seat from another connection.
`--time-scale N` runs the world's clock N times faster than the wall's; NPCs still walk at wall pace.

```text
mineworld server worlds/market-town --town --save saves/town
```

Each connected client gets what **its** observer perceives — not a world dump, and not a filtered
copy of one. It submits requests; the server allocates their identity and the world decides what
happens. A client can say three things: which seat it wants, what it would like to happen, and that
it is leaving.

Read next:

- [`PROTOCOL.md`](PROTOCOL.md) — the frames, in full. Write a client against that.
- [`docs/NETWORKING.md`](../docs/NETWORKING.md) — why the server is authoritative, and why there is
  only one binary.

- [`tools/cli/README.md`](../tools/cli/README.md) — the command that runs a world.
