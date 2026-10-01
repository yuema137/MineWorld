# `tools/cli` — the `mineworld` command

```sh
mineworld server worlds/social-cafe                  # host a world
mineworld server worlds/social-cafe --listen 0.0.0.0:7878
mineworld server worlds/social-cafe --agent alice     # and drive one of its people with a rule
mineworld server worlds/social-cafe --save saves/cafe # keep the world: stop it, start it, same world
mineworld replay worlds/social-cafe --save saves/cafe # re-run a save's whole history and check it
mineworld validate worlds/social-cafe                # check one, and say what it is
```

`--save DIR` creates the world in `DIR/world.sqlite` the first time and resumes it every time after —
even after the process was killed. The resumed world is the same world: the same instance, the same
people, the same history, continuing where it stopped. `replay` re-executes the save from its beginning
and fails if a single fact or snapshot does not reproduce.

`--agent SEAT` occupies that seat with a deterministic controller, in this process, over the same
path a client's connection uses — the same roster, the same actor check, the same server-allocated
identity. Repeat it for more than one. Nothing about the world distinguishes such a Person from a
played one, which is `INV-1`.

One binary for localhost and for a LAN alike; there is no single-player mode to choose, because there
is no single-player path. `create` and `inspect` arrive with S7.

This crate is the **composition root**: the only one that depends on the transport, the System Packs
and a controller at once. That is what lets `mineworld-server` stay a library that knows nothing about
conversation or presence, `mineworld-worldpack` stay a loader that knows nothing about sockets, and
`mineworld-rule-controller` stay a decision that knows nothing about how a request reaches a world.

Read next: [`server/PROTOCOL.md`](../../server/PROTOCOL.md) to write a client, and
[`worldpack/README.md`](../../worldpack/README.md) for what a World Pack is.
