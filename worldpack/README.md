# `mineworld-worldpack`

Reads a **World Pack** — a directory of YAML describing a world's places, people and capabilities —
and loads the world it describes.

```rust
let pack = WorldPack::read("worlds/social-cafe")?;   // validate, or say what is wrong and where
let loaded = pack.load(WorldTime::EPOCH)?;           // install, create, resolve, state
```

Three things are worth knowing before reading the code:

- **Authoring keys resolve to ids deterministically.** Places in key order, then people in key
  order. That order reaches the event log, so it is a property of the pack and never of the run.
- **Initial state is a recorded fact, not a write.** Each authored position becomes a genesis event
  the owning system reduces, so *where did Alice's initial position come from* has an answer that
  survives a replay.

- **It names no System Pack but two.** It reaches every pack through the build's installed set,
  [`systems/installed`](../systems/installed/), so installing a pack edits nothing here. Only
  `presence` and `movement` are named, because `location` and `passages` are their fields
  ([`DECISIONS.md` `ARC-33`](../docs/DECISIONS.md)).

What a pack may say is specified in [`docs/MODULE_SPEC.md`](../docs/MODULE_SPEC.md) §4 and §4.1.
The reasoning behind the loading route is in
[`.structured-coding/plans/mvp0/pr-05c-world-packs.md`](../.structured-coding/plans/mvp0/pr-05c-world-packs.md).
