# persistence/

**`mineworld-persistence`** — how a MineWorld world survives the death of the process hosting it.

A save is one SQLite file, `world.sqlite`, holding three append-only things:

```text
facts       every Event that happened             the world's history
journal     every input that moved the world      requests, and time passing with something due
snapshots   the whole state at some revisions     checkpoints
```

A restart restores the newest snapshot and re-runs the journal after it through the kernel's own
pipeline. Every re-run input must reproduce the facts the save logged, byte for byte, or the save is
refused. `verify` re-runs everything from genesis and checks every snapshot too.

Each journaled input is one **revision**: the number a client is told the world is at.

Facts and journal are never deleted. Snapshots are kept by a fixed rule — genesis, one anchor every
4 096 revisions, and the newest two — and each is stored zstd-compressed, so a save grows with its
history rather than with its snapshots (`ARC-81`).

```rust
let (world, _) = PersistentWorld::create(Box::new(SqliteBackend::create(dir, durability)?), assembled, creation)?;
let (world, how) = PersistentWorld::resume(Box::new(SqliteBackend::open(dir, durability)?), composed)?;
```

The rules, and why, are in [`docs/DECISIONS.md`](../docs/DECISIONS.md) `ARC-25` and `DEP-2`; the
design is `.structured-coding/plans/mvp0/step-06-persistence.md`.
