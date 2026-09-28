# `tools/cli` — the `mineworld` command

```sh
mineworld server worlds/social-cafe                  # host a world
mineworld server worlds/social-cafe --listen 0.0.0.0:7878
mineworld validate worlds/social-cafe                # check one, and say what it is
```

One binary for localhost and for a LAN alike; there is no single-player mode to choose, because there
is no single-player path. `create` and `inspect` arrive with S7.

This crate is the **composition root**: the only one that depends on the transport and on the System
Packs at once. That is what lets `mineworld-server` stay a library that knows nothing about
conversation or presence, and `mineworld-worldpack` stay a loader that knows nothing about sockets.

Read next: [`server/PROTOCOL.md`](../../server/PROTOCOL.md) to write a client, and
[`worldpack/README.md`](../../worldpack/README.md) for what a World Pack is.
