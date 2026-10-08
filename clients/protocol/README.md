# `clients/protocol` — the Godot client protocol module

**`mineworld/`** is the module: three GDScript files a client project copies in. Everything else here
exists to prove it works.

```text
mineworld/world_client.gd   the connection: join a seat, receive observations, submit requests
mineworld/observation.gd    reading a frame without deciding anything
mineworld/space.gd          the one conversion between the world's axes and Godot's
demo/                       a demonstration scene of its own
checks/                     headless checks of the module: a reader check, and a live check against
                            a real market-town server
evidence/                   what the real engine produced, against the real server
```

## Run it

```sh
cargo run -p mineworld-cli --bin mineworld -- server worlds/social-cafe --agent alice
godot --headless --path clients/protocol --import      # once, to build the script class cache
godot --path clients/protocol                          # windowed: [1] walk closer  [E] talk  [Esc] leave
```

Scripted, which is what `evidence/` was made with:

```sh
godot --headless --path clients/protocol -- \
    --autopilot --flavour 2d --seat visitor --requests evidence/request-2d.json
godot --headless --path clients/protocol -- \
    --autopilot --flavour 3d --seat visitor --requests evidence/request-3d.json
```

The module's own checks, headless:

```sh
godot --headless --path clients/protocol --script res://checks/reader_check.gd   # no server
bash clients/protocol/run.sh affordances     # against worlds/market-town, saved to a temporary directory
```

`--flavour 2d` reports no position when it acts; `--flavour 3d` reports the one it walked to. That is
the only difference between a 2D and a 3D client at this boundary, and `AC-13` is the claim that it is
the only one — `tools/cli/tests/ac13_semantic_parity.rs` reads those two files and checks it.

## What this is, and what it is not

It is the protocol layer, meant to be **drop-in** for the 2D and the 3D reference clients, which keep
their own scene graphs, their own art and their own camera work. What changes for them is where the
world comes from.

It is not art, and the demonstration scene is not a style candidate: it draws dots and text.

It contains **no world rule**. It never measures a distance, never decides whether anybody is
available, never decides whether an action exists. Every verdict it shows arrived in an affordance the
server computed — which is why two clients can offer the same interactions without either
implementing a rule.

## Adopting it

[`ADOPTION.md`](ADOPTION.md) is the specification: the two supported ways to take the folder, the
whole API, and the four rules a client must not break. Read that; this file is the orientation.

Read next: [`server/PROTOCOL.md`](../../server/PROTOCOL.md), which governs the frames and which this
module implements.
