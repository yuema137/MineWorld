# `tools/cli` — the `mineworld` command

```sh
mineworld server worlds/social-cafe                  # host a world
mineworld server worlds/social-cafe --listen 0.0.0.0:7878
mineworld server worlds/social-cafe --agent alice     # and drive one of its people with a rule
mineworld server worlds/social-cafe --save saves/cafe # keep the world: stop it, start it, same world
mineworld replay worlds/social-cafe --save saves/cafe # re-run a save's whole history and check it
mineworld validate worlds/social-cafe                # check one, and say what it is

mineworld run worlds/social-cafe --headless --seed 7 --days 300   # 300 days, no renderer, no model
mineworld run worlds/social-cafe --headless --seed 7 --days 300 --save saves/run
mineworld inspect saves/run                           # what a save holds; every fact's cause checked
mineworld create worlds/my-town                       # a new World Pack to start from
```

`run` drives every seat with a seeded rule — people greet, talk, walk about and step out of the
door — and prints a line per simulated day and a summary. The same seed gives the same world, byte
for byte; kill a `--save` run and run the same command again, and it finishes the same world.

`--save DIR` creates the world in `DIR/world.sqlite` the first time and resumes it every time after —
even after the process was killed. The resumed world is the same world: the same instance, the same
people, the same history, continuing where it stopped. `replay` re-executes the save from its beginning
and fails if a single fact or snapshot does not reproduce.

`--agent SEAT` occupies that seat with a deterministic controller, in this process, over the same
path a client's connection uses — the same roster, the same actor check, the same server-allocated
identity. Repeat it for more than one. Nothing about the world distinguishes such a Person from a
played one, which is `INV-1`.

One binary for localhost and for a LAN alike; there is no single-player mode to choose, because there
is no single-player path. The full command surface is specified in
[`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §8.1.

This crate is the **composition root**: the only one that depends on the transport, the System Packs
and a controller at once. That is what lets `mineworld-server` stay a library that knows nothing about
conversation or presence, `mineworld-worldpack` stay a loader that knows nothing about sockets, and
`mineworld-rule-controller` stay a decision that knows nothing about how a request reaches a world.

Read next: [`server/PROTOCOL.md`](../../server/PROTOCOL.md) to write a client, and
[`worldpack/README.md`](../../worldpack/README.md) for what a World Pack is.
