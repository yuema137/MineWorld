## DEP-2 — Persistence: `rusqlite` behind `PersistenceBackend`

**Date** 2026-09-25 · **Status** selected, integrated in S5

**Problem.** Durable local world state and an append-only event log, with a cloud backend later
([`ARCHITECTURE.md`](ARCHITECTURE.md) §8).

**Options.** `rusqlite`, `sqlx`, `diesel`, `sea-orm`, or a file format of our own.

**Choice: `rusqlite` with the `bundled` feature**, which compiles SQLite into the binary so a
local server has no system dependency — directly serving the local-hosting requirement in
[`NETWORKING.md`](NETWORKING.md) §6.

**Why not the others.** `sqlx` is async-first with compile-time query checking, which suits a
service talking to a remote database; our persistence is local and sits behind a blocking task,
so its async model buys nothing and costs build complexity. It is also a semver hazard alongside
`rusqlite`, since only one crate version may link `libsqlite3-sys`. An ORM is the wrong shape for
event sourcing, where rows are append-only facts rather than mapped objects. Writing our own file
format would be rebuilding commodity infrastructure (`REUSE_POLICY.md` §4) and losing crash
safety we would then have to prove.

**Isolating interface.** `PersistenceBackend`, with `SqliteBackend` as the first implementation
and Postgres later through the same trait. No SQL or SQLite type appears in the kernel or in any
system.

**Accepted limitations.** Synchronous API bridged with `spawn_blocking`. Single-writer concurrency
model, which matches an authoritative single-world server and would need revisiting only for
multi-world processes.

**Implementation note, 2026-09-30 (S5, PR 07).** Integrated as the crate `mineworld-persistence`
(`persistence/`), which holds `PersistenceBackend` and its one implementation `SqliteBackend`; the
kernel, every system and the server name no SQLite type. A save is **one file**, `world.sqlite`, whose
tables are the manifest, the journal, the facts and the snapshots (`ARC-25`), so that one transaction
commits a whole revision — a separate `manifest.json` would be a second copy of a fact outside that
transaction. WAL mode; `synchronous = FULL` for a hosted world, so a revision a client has been told
survives power loss, with `NORMAL` (durable against a process crash) available to headless bulk runs.
No `spawn_blocking` is needed: the server already runs its world on a dedicated blocking thread
(`server/src/host.rs`), which is where every persistence call happens.
