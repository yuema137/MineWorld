## ARC-98 — Shared controller memory is a cognition-side home of perceived envelopes, and perception is the only access right

**Date** 2026-10-14 · **Status** accepted (step-28 design frozen 2026-10-14); implemented by S28-a · **Approved by** the operator's step-28 rulings QS28-1 … QS28-10 (2026-10-14) · **Relates to** `ARC-25`, `ARC-27`, `ARC-43`, `ARC-57`, `ARC-58`, `ARC-59`, `ARC-86`, `ARC-87`, `ARC-89`, `ARC-99`, `ARC-100`, `ARC-101`, `DEP-37`, `DEP-58`, `INV-4`, `INV-11`, `INV-13`, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §5.3, [`ARCHITECTURE.md`](ARCHITECTURE.md) §9.3 · **Design** `.structured-coding/plans/mvp0/step-28-shared-memory.md` §§4, 6; `.structured-coding/plans/mvp0/pr-s28a-memory-home.md`

**Problem.** Today each seat stores its own copy of every fact it perceived. In the 12-person `social-cafe` at seed 7 over 30 days, 86.7 % of the 84 639 distinct facts are perceived by more than one person, and the seats hold 449 726 copies, 5.3 per fact (step-28 §2, F-S28-1 and F-S28-2). Shared events must be stored once, and a person's access to them must be a right, loaded when needed (operator, 2026-10-14). The world log cannot be the shared store: a controller is a client that sees only its perceived stream (INV-6, INV-13), and the log is world truth, not a subjective layer (INV-4).

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) One store per seat, as today | Declined: 5.3 copies per fact; the 20 MiB bound would hold about 45 days of undigested detail |
| (b) Shared records in the kernel or in the world log, read by controllers | Declined: the kernel would learn what memory and a holder are (INV-12, `CLAUDE.md` §4.2), and a controller would read truth it must not read (INV-13) |
| (c) **One cognition-side home per process: fact records keyed by `EventId` and verified by digest, and one access row per person** | **Chosen** |
| (d) Content-addressed records keyed by a hash of the content | Declined as the key: the world already gives each fact a stable, unique id (INV-11), and a hash would add a second identity. The method is adopted (`DEP-58`) |
| (e) Gists deduplicated by content hash | Declined: a gist is the person's own rendering and depends on that person and their name book at ingestion (I-S28-2); the saving is under 4 % of world bytes |
| (f) Per-seat files joined by `ATTACH` in one connection | Declined: atomic commit across attached files holds only in rollback-journal mode, a restriction to defend indefinitely. One file per home gives one transaction |
| (g) A reference count stored on each record | Declined: a stored count can drift from its rows. The anti-join over the access rows is exact by construction |

**Choice.** Shared controller memory is a cognition-side home of perceived envelopes, and perception is the only access right. Fact records are stored once per home, keyed by `EventId` and verified by digest. Access rows are per person. GC is the anti-join in the releasing transaction. There is no kernel, server or protocol change in MVP-0.

The concrete rules that follow, stated in step-28 §§4 and 6: the home is one SQLite database per world instance and cognition process, and it is the only writer of its tables. A fact record exists exactly while at least one access row cites its key (I-S28-1, I-S28-4). Insert-or-verify raises `ConflictingFact` on a digest difference. Each ingest call and each retention pass is one `BEGIN IMMEDIATE` transaction. Forgetting removes an access row or demotes it into a private summary; it never redirects to a peer's copy.

**Why not ourselves, and why not them.** Storage and transactions are commodity, so SQLite is reused (`DEP-37`, `DEP-58`). The access rule, the garbage-collection rule and the determinism contract are MineWorld's own, because no framework's execution model may own them (`REUSE_POLICY.md`).

**Interface that isolates it.** `mineworld_cognition.memory.home` (`MemoryHome`, `SeatView`). The fact and access tables are written only by the home.

**Limitation accepted.** Sharing is per process in MVP-0. Seats in different processes have different homes, so duplication is bounded by the number of processes, not of persons (step-28 §6.6). Cross-process sharing is MVP-1, through the server as the cold tier (QS28-2, path (i)). A cognition-side shared service is declined as the default because it cannot verify perception.

**Note, 2026-10-19 (PR S28-f; `pr-s28f-home-wiring.md` §§5, 14; operator rulings QS28f-1 … QS28f-4).** The placement as built. (1) The layout is fixed, not a path template: under `[memory] sharing = "home"` (the default) the seats of one cognition process share `<store>/<instance>/memory.sqlite` per world instance, and under `"none"` each seat has `<store>/<instance>/<seat>.sqlite`; `service` is refused at load time with `SharingModeUnavailable`. (2) One `asyncio.Lock` per home serializes every write of its seats, because an assisted retention pass awaits its model inside its `BEGIN IMMEDIATE` transaction on the home's one connection. (3) No record LRU (`cache_records`) is built: SQLite's page cache (`page_cache_kib`, default 8 192 KiB) is the only cache until S28-g measures a need (FU-S28f-1). (4) `[controller] recall_episodes` is kept beside `person.sections.L1.lines`, and every retention pass is written to the seat's decision log as one `pass` line.

**Revisit** when MVP-1 opens cross-process sharing (S28-h), or when a world needs more than one process per instance.
