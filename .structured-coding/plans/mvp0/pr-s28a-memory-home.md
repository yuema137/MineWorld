# PR S28-a — The memory home: schema 4, shared fact records, seat views, positional dumps, deterministic GC, and the transparency harness

## DESIGN FROZEN — 2026-10-15 (operator approved the freeze in the primary session: "同意冻结并合并")

```text
Design revision:        revision 1 (2026-10-15), FROZEN 2026-10-15
Author:                 S28-a / S27-d PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s28a-s27d, branch docs/s28a-s27d-designs,
                        planning base origin/main @ 2d07c6c (#215; FG-b2 #213 merged at e67af07)
Approved by / evidence: §14 ruled 2026-10-15: QS28a-1 and QS28a-3 by the operator (answered directly in
                        the primary session), QS28a-2 and QS28a-4 by the primary; relayed by the
                        coordinator. The freeze header is the primary session's to write
Implementation base:    origin/main at C0; C0 records the commit, and that S28-0 and FG-b2 have merged
Start condition:        step-28 §11.1: S28-0 merged (ARC-98 … ARC-101, DEP-58 in docs/DECISIONS.md; NOT
                        merged at 2d07c6c), FG-b2 merged (yes, #213), this design frozen, the primary
                        session's dispatch
Execution contract:     §12 (filled, endpoint authority included)
Implementer:            Opus-class (step-28 §11.1)
Lifecycle:              DESIGN FROZEN 2026-10-15
```

**Effort:** `mvp0` · **Step:** [`step-28-shared-memory.md`](step-28-shared-memory.md) (STEP DESIGN FROZEN
2026-10-14; rulings §14.1; `ARC-98` … `ARC-101`, `DEP-58`) §§0, 3 (I-S28-1 … I-S28-10), 4, 6, 7, 8, 9 (C-1,
C-2, C-5, C-6), 10, 11 (the S28-a row), 15 (AS28-1, 3–9, 12–14) · **Parent:** [`overall.md`](overall.md) ·
**Revises the code of:** [`pr-s10-p4-memory.md`](pr-s10-p4-memory.md) (the store, `ARC-59`, `DEP-37`),
[`pr-fg-b-retention-host.md`](pr-fg-b-retention-host.md), [`pr-fg-b2-size-trigger.md`](pr-fg-b2-size-trigger.md)
(merged as #213), [`pr-fg-c-lm-summarise.md`](pr-fg-c-lm-summarise.md) (merged as #201) · **Followed by:**
S28-b (the retention host's remaining S28 work, §10) · **Coordinates with:** P6b (PR #207, open; it opens
`MemoryStore`, §10 R-P6b-a-1) · **Binding parents:** `CLAUDE.md` §§2, 3, 4 (items 1, 2, 3, 7, 8, 9, 10, 11,
12, 13, 16); `docs/CORE_CONCEPTS.md` §§2 (INV-4, INV-13), 5.3; `docs/ARCHITECTURE.md` §9.3;
`docs/DECISIONS.md` `ARC-25`, `ARC-27`, `ARC-59`, `ARC-87`, `ARC-98` … `ARC-101`, `DEP-37`, `DEP-58`.

**Decision ids** (confirmed free on `origin/main` by `git grep` on 2026-10-15). `ARC-109` (assigned by the primary 2026-10-15; `ARC-105` … `ARC-108` and `DEP-60` went to S29, #218):
the memory home replaces `MemoryStore`, the seat view is memory's one storage API, and the store schema is
**4**. `DEP-61` (assigned 2026-10-15): `HomeBusy` uses SQLite's own exclusive locking, and the `filelock`
package is declined (§4). Both are written by C2 (§9).

No hosted model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Scope (step-28 §11, S28-a row).** "`memory.home` and `memory.scopes.world`: schema v3, the fact,
  fact_who and access tables, insert-or-verify, `SeatView` and its dump, positional canonical form (QS28-5,
  ruled), one transaction per ingest, GC, `verify`, `HomeBusy`, v2 refusal. The transparency harness (12
  seats, `none` vs `home`, frames, interleavings, kills). Golden v2 renders."
- **Start (step-28 §11.1).** "S28-0 merged. **FG-b2 merged** (bound, size trigger). Golden v2 renders captured
  at C0 from that `main`." Implementer **Opus**; reviewer the primary plus an Opus logic review.
- **Rulings (step-28 §14.1).** QS28-1 (MVP-0), QS28-2 (per process in MVP-0), QS28-4 (**a person's size is the
  seat view, shared records counted in full**), QS28-5 (**adopt the positional canonical encoding, in S28-a**),
  QS28-8 (**refuse the old schema and rebuild from the perceived stream; no importer**), QS28-10 (stack in
  `decisions.yaml`, placement in `cognition.toml`). None is re-asked here.
- **Integration checkpoint (step-28 §11.1).** "Every PR ends with the transparency harness green (12 seats,
  `none` vs `home`)."
- **R-LOAD-1 (coordinator rule, in force).** Locally: only the targeted test files of the commit, plus ruff,
  ruff format and pyright. Never a local `cargo test --workspace` or a whole-repository suite while other
  agents run. CI on the exact head is the gate.

---

## 1. Goal

Replace the per-seat `MemoryStore` with a **memory home**: one SQLite database per cognition process and
world instance that stores each perceived fact's perspective-free envelope **once**, as a fact record keyed
by its `EventId`, and gives each seat an **access row** carrying only its own rendering. Every consumer of
memory (ingestion, the roll-up, names, retrieval and the FG-b/FG-b2/FG-c retention host) reads and writes
through a **seat view**, whose canonical dump is defined in the **positional** encoding (QS28-5) and is
byte-identical whether the seat shares its home with no one or with eleven others (I-S28-2). Shared records
live exactly as long as some seat holds them, collected in the releasing transaction (I-S28-4). The
per-person bound (FG-b2: 20 MiB default, raise-only) measures the seat view, records counted in full
(QS28-4). Nothing changes in the kernel, contracts, server or protocol (I-S28-10).

---

## 2. Scope

### 2.1 In scope

1. **`memory/home/`** (new package, `mineworld_cognition.memory.home`): `schema.py` (schema 4 DDL, the dumped
   tables and their order, `SCHEMA_VERSION = 4`), `encoding.py` (the positional canonical line, §5.4),
   `home.py` (`MemoryHome`: open, identity, lock, seats, transaction, home dump, `verify`), `view.py`
   (`SeatView`: the projection, its dump, sizes and meter, `hold`, `release`, meta, cursor), `errors.py`.
2. **`memory/scopes/world.py`** (new; `memory/scopes/__init__.py` a docstring-only package): fact records —
   insert-or-verify, the holder anti-join GC, the `verify` checks. No `MemoryScope` interface yet (S28-c).
3. **Deletion of `memory/store.py`** (`MemoryStore`, `NotAMemoryStore`, `StoreIdentityMismatch`) with no
   alias (`CLAUDE.md` §4 item 12). `records.SCHEMA_VERSION` moves to `home/schema.py`; `StoreIdentity` stays
   as the **seat identity**.
4. **The mechanical port of every consumer to `SeatView`** (§5.8): `memory/{ingest.py, names.py,
   retrieve.py, __init__.py}`, `compress/{episodes.py, chapters.py, stable.py, model_summarizer.py}`,
   `memory/retention/{absorb.py, host.py, plan.py, bound.py, items.py, forgotten.py, assisted.py}`. "Mechanical"
   means: the same statements, with the seat predicate added, `l0` reads replaced by the access ⋈ fact join,
   and `l0` deletes replaced by `SeatView.release`. No retention semantics change (§14 QS28a-2, ruled yes).
5. **The tests' port** to the new API (every file of §3.3), and the new tests of §7.
6. **Golden renders** captured from the schema-3 code at C1 (C-2), and the **transparency harness**
   (AS28-1, AS28-3, AS28-4, AS28-5, AS28-6, AS28-7, AS28-8, AS28-9, AS28-12, AS28-13, AS28-14 as §8 states them).
7. `docs/DECISIONS.md`: `ARC-109`, `DEP-61`; a dated note under `ARC-101` (the concrete grammar of
   §5.4) and under `ARC-87` (the bound measures the seat view).
8. `cognition/lm-controller/README.md`: one line (memory is a home of seat views; the old store is refused).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Forgotten-range attribution, C-4 (a)–(c), FU-S28-1 (prose names), FG-b's real-binary Gate 2 on seat views, the 300-day bound and AS28-9 under the forced backstop | S28-b (§10 R-S28b-a-1) |
| `MemoryScope`, `ScopeRegistry`, `decisions.yaml memory.scopes`, the `person` scope re-homing, parameter models, AS28-2, AS28-19 | S28-c |
| `place` / `item` anchors, `Rendered.anchors` | S28-d |
| `mineworld-memory inspect`, `export` / `import`, AS28-11, AS28-16 | S28-e |
| P6b wiring (`cognition.toml [memory]`: `sharing`, `home`, `cache_records`, `page_cache_kib`), AS28-10 | S28-f |
| The per-home record LRU (`cache_records`, step §4.5) | **Unassigned in the step** (F-S28a-6): recommended S28-f |
| 12 × 300 and 100 × 300 days, AS28-17 | S28-g |
| Any change under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` | never (I-S28-10) |

### 2.3 Dependencies

Merged: P4, FG-b (#186), FG-b2 (#213, `e67af07`), FG-c (#201), DL-a. **Required before C0:** S28-0 (spec edits
and `ARC-98` … `ARC-101`, `DEP-58`; not on `main` at `2d07c6c`). Not required: P6b (#207, open): whichever of
P6b and S28-a merges second adapts its `MemoryStore` use (R-P6b-a-1).

---

## 3. Audit (`origin/main @ 2d07c6c`)

### 3.1 Source anchors (all under `cognition/lm-controller/src/mineworld_cognition/`)

| # | Fact | Where |
| --- | --- | --- |
| A-1 | **The schema on `main` is already 3.** `SCHEMA_VERSION = 3`: "2 since FG-b …; 3 since FG-b2: the pass log is keyed by its triggering fact and records its cause, so a day may hold several passes". | `memory/records.py:27-31`; FG-b2 design §4.5 |
| A-2 | A store of another schema is refused as `StoreIdentityMismatch(field="schema")` (not a named `UnsupportedStoreSchema`); a new file whose identity carries another version is refused the same way. | `memory/store.py:119-129`, `:371-381` |
| A-3 | One file per seat; tables `meta`, `l0`, `l0_who`, `episodes`, `episode_who`, `chapters`, `chapter_ranges`, `stable`, `names`, `forgotten`, `retention_passes`, plus the FTS5 index `l0_text` (never dumped). `l0` carries `event_key`, `event_id`, `at`, `kind`, `place`, `salience`, `mine`, `gist`, `digest`, `importance`, `involved`. | `memory/store.py:37-76` |
| A-4 | The dump is JSON Lines of sorted-key objects with a `"table"` key, one encoder; tables in `_DUMPED` order, rows in key order. | `memory/store.py:83-116`, `:212-216` |
| A-5 | **FG-b2's meter.** `content_size()` = the dump's bytes less `meta`'s lines, kept by `TEMP` triggers per metered table calling Python functions with `(table, rowid, NEW.*)`; reset on rollback; `size()` and `row_bytes()` read the meter once it is in use. | `memory/store.py:96-101`, `:218-292`; FG-b2 §4.2 |
| A-6 | `transaction()` is `BEGIN IMMEDIATE … COMMIT/ROLLBACK` on the store's one connection; `auto_vacuum = INCREMENTAL` set before the first table; `reclaim()` runs `PRAGMA incremental_vacuum`. | `memory/store.py:175-186`, `:304-306`, `:383-385` |
| A-7 | Ingest: per fact, re-delivery is `SELECT digest FROM l0 WHERE event_key` (equal → no-op, else `ConflictingFact`); a fact not held at or below the cursor where retention merged or forgot is skipped (`_represented`); `involved = me in fact.participants` is stored but read only at ingest (by `Stable.feed`). | `memory/ingest.py:96-122`, `:145-157`, `:160-202`; `compress/stable.py:25-27` |
| A-8 | `digest(fact)` = sha256 hex of the fact's canonical JSON. | `memory/ingest.py:64-69` |
| A-9 | Counterparts = the fact's participants, deduplicated in envelope order, minus me; `l0_who` is read back in insertion (`rowid`) order, but dumped sorted by entity. | `memory/render/__init__.py:69-70`; `memory/store.py:87`, `:337-341` |
| A-10 | SQL against the store's tables outside `store.py` (every site the port touches): `compress/episodes.py:70,85,89,99-103,117,122,129,151,159`; `compress/chapters.py:29,32,40,48,52,69,96,103`; `compress/stable.py:28,36,42,52`; `compress/model_summarizer.py:96,115,120`; `memory/names.py:38,49`; `memory/ingest.py:102,150,168,185,190`; `memory/retrieve.py:215-231,238,250`; `memory/retention/absorb.py:109,136`; `memory/retention/bound.py:117,198`; `memory/retention/forgotten.py:31,56,65,72-74,80`; `memory/retention/host.py:106,111,334,339,346,367,373,380,384,392,396,404`; `memory/retention/plan.py:99-161`. | as listed |
| A-11 | Retention items read rows **with their dump sizes** through `row_bytes(table)` (`_Extraction.rows`); an L0 item's size is its `l0` line plus its `l0_who` lines. | `memory/retention/items.py:173-212` |
| A-12 | Retention execution deletes L0 rows through `delete_records` (FTS entry, `l0_who`, `l0`) inside the pass's transaction. | `memory/retention/plan.py:121-161` |
| A-13 | FG-b2's bounds: `StoreBounds(mib)`, `bound = mib × MiB`, `target = bound × 3 // 4`, `trigger = bound − bound // 16`, `limit = target − 512`; `absorb` passes `stop_when = content_size() > trigger`. | `memory/retention/bound.py`; FG-b2 §4.1, §4.3 |
| A-14 | FG-c's replay test pins the sha256 of a replayed **dump**: an encoding change changes it. | `tests/test_lm_summarise_replay.py:40`, `:99-103` |
| A-15 | The isolation scan forbids, in `memory/`, `compress/`, `forgetting/`, imports beyond a short allow-list. | `tests/test_memory_isolation.py:20-30` |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S28a-1** | The step calls the home's schema "v3" and its refusal `UnsupportedStoreSchema(found=2, wanted=3)` (step §§4.4, 5.5, 10). FG-b2 shipped schema **3** (A-1) after the step was written. Calling the home "3" would accept FG-b2 files as homes. | **The home is schema 4.** Every older store (2, 3) is refused by `UnsupportedStoreSchema(found, wanted=4)` with the rebuild instruction (QS28-8's rule, unchanged). The step's "v3 store" reads "schema 4 seat view" everywhere; the primary applies the wording (§11). Not a question: QS28-8 already rules the migration; only the number is new. |
| **F-S28a-2** | **I-S28-6 cannot hold strictly.** A record held by exactly one seat costs the home its fact line, its `fact_who` lines and one access line, which repeats the key and names the seat; the seat view costs one `l0` line and its `l0_who` lines. With any encoding that names the holder, the home is larger than the view for that record by a fixed overhead (the access line's table tag, seat and key; and `me` among the participants). Under `sharing: none` every record is singly held, so `MemoryHome.dump() ≤ SeatView.dump()` fails by construction, and AS28-9 as written fails on its own baseline. | §14 **QS28a-1, ruled 2026-10-15 (operator):** I-S28-6 is replaced by the two exact statements I-a-6 (a) and (b) (§6), with the named constant `K` (§5.4). |
| **F-S28a-3** | The step's split gives S28-a the store and S28-b "the retention host on seat views: items, execution with GC, FG-b2's size trigger on the seat view". But the store is the retention host's storage: retention SQL (A-10, A-12) reads `l0` and deletes from it. Replacing the store without porting those statements leaves `main` red; keeping `MemoryStore` beside the home is the compatibility layer QS28-8 and `CLAUDE.md` §4 item 12 rule out. | §14 **QS28a-2, ruled 2026-10-15 (primary): yes.** S28-a ports every SQL site mechanically (items, execution through `release` with GC, the meter, the size trigger now measure the seat view **by construction**); S28-b keeps the semantic retention work (§10). |
| **F-S28a-4** | The step's "how" (`subject` / `participant` / `witness`, §4.3) and S28-d's anchor role `subject` (§5.2.3) need the fact's **subjects**, but §4.2's record keeps only participants (`fact_who`). `PerceivedEvent.subjects` is not stored anywhere today. | §14 **QS28a-3, ruled 2026-10-15 (operator): `fact_who.role` now** (§5.2). |
| **F-S28a-5** | `HomeBusy(path, holder_pid)` (step §5.5): SQLite's lock (the reuse choice, §4) cannot report the holder's pid; a pid needs a side lock file. | §14 **QS28a-4, ruled 2026-10-15 (primary): `holder_pid` dropped**; the error is `HomeBusy(path)`. |
| **F-S28a-6** | The step's per-home LRU of decoded records (`cache_records`, §4.5, §5.4) is in no PR's scope (§11's rows list `sharing` and `cache` wiring only for S28-f). | Recorded for the primary: recommended in S28-f, measured first (the SQLite page cache may make it unnecessary — `CLAUDE.md` §4 item 11). Not built here. |
| **F-S28a-7** | `involved` (A-3, A-7) is stored but never read after ingest; it is derivable from the record (`me ∈ participants`). | Dropped from schema 4; ingest computes it from the fact, as today. |
| **F-S28a-8** | The 20 MiB promise's day count (step §8.3) depends on the positional bytes per record; FG-b2's §4.1 margin argument ("1.25 MiB is a fact naming ≈ 3 000 participants") was stated in object bytes. | The ratios and numbers of `StoreBounds` are unchanged (step §7.2: "S28 inherits FG-b2's numbers and changes only what they measure"); C8 re-measures bytes per held record and records the margin in positional bytes (§8 AS28-12). |

### 3.3 Tests that open the store today (the port's test surface)

`tests/retention_driver.py` (5 uses), `test_ac10.py` (3), `test_assisted_host.py` (2), `test_lm_summarise_replay.py`
(4), `test_lm_summarise.py` (4), `test_memory_compress.py` (3), `test_memory_isolation.py` (2),
`test_memory_render.py` (4), `test_memory_retrieve.py` (3), `test_memory_store.py` (11),
`test_memory_summarizers.py` (6), `test_retention_host.py` (9), `test_retention_plan.py` (3),
`test_retention_policies.py` (3), `test_retention_real.py` (5), `test_retention_retrieve.py` (4),
`test_retention_size.py` (13), `test_retention_store.py` (9). Fixtures: `memory_fixtures.py`
(`alice_identity`, the fact builders), `retention_stream.py` (the seeded single-seat stream), `conftest.py`
(`social_cafe_100`, built from the real binary), `binary.py` (`mineworld(…)`).

### 3.4 What does not exist

No home, no shared record, no seat column anywhere, no positional encoder, no multi-seat stream fixture, no
kill harness for memory, no `UnsupportedStoreSchema`, no `HomeBusy`.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`; `DEP-58` covers the store)

| Need | Decision | Why |
| --- | --- | --- |
| The store | SQLite (`sqlite3`, `DEP-37`, `DEP-58`): one file per home, one connection, rollback journal | One transaction covers every seat's writes (step §6.3); `ATTACH` across files is declined there |
| Byte metering | FG-b2's `TEMP` triggers for private tables; for the access join, the home's own `hold`/`release` (the only writers of access rows) add and subtract the view lines' sizes | Python's `sqlite3` has no update hook (FG-b2 §4.2); a trigger cannot see the joined record without SQL inside a Python callback, which `sqlite3` forbids on the same connection |
| Single-writer lock (`HomeBusy`) | **SQLite's own exclusive locking** (`PRAGMA locking_mode = EXCLUSIVE`, then `BEGIN EXCLUSIVE; COMMIT` at open, `timeout = 0`) — `DEP-61` | Cross-platform by SQLite's VFS (POSIX locks, Windows `LockFileEx`), no dependency, no side file to go stale after a SIGKILL. Declined: `filelock` (Unlicense; mature) — a second lock beside SQLite's own that can disagree with it; a hand-written `fcntl`/`msvcrt` lock file — two code paths and stale files. The prototype check is C2's test on all three OSes (R-a-3) |
| Canonical encoding | our own, ≈ 120 lines (`encoding.py`) on the standard `json` encoder | The encoding **is** the determinism and bound contract (`ARC-101`); a serialization library would put that contract outside our control |
| Kill harness | the standard library (`subprocess`, `os._exit`) | No library models "crash a SQLite writer at a chosen statement" |
| Content addressing, ref-counted blob stores, Letta/Zep/mem0 | not adopted (step §16, `DEP-58`) | ruled |

---

## 5. Design

### 5.1 Module layout

```text
memory/home/__init__.py   MemoryHome, SeatView, the errors (the package's public names)
memory/home/schema.py     SCHEMA_VERSION = 4; DDL; HOME_TABLES; VIEW_TABLES (name, columns, order); codecs
memory/home/encoding.py   line(table, cells, codecs) -> str; line_bytes(...) -> int    (§5.4)
memory/home/home.py       MemoryHome                                                   (§5.2, §5.5)
memory/home/view.py       SeatView                                                     (§5.3, §5.6, §5.7)
memory/home/errors.py     NotAMemoryHome, HomeIdentityMismatch, HomeBusy, SeatNotInHome,
                          UnsupportedStoreSchema, HolderInvariantBroken, FactRecordMissing
memory/scopes/__init__.py docstring only: "memory scopes (step-28 §5); the interface is S28-c's"
memory/scopes/world.py    insert_or_verify, collect (GC), check_holders (verify)       (§5.6)
memory/store.py           deleted
```

Only `memory/home/` and `memory/scopes/world.py` call `sqlite3.connect` or write `fact`, `fact_who`,
`access` or `access_text` (AS28-13, AS28-14). Every other module writes only its own private tables, through
the `SeatView`'s connection, with the seat predicate.

### 5.2 Schema 4 (the home)

```sql
-- home identity; never per seat
CREATE TABLE home_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)   -- world_instance, schema
CREATE TABLE seats (seat INTEGER PRIMARY KEY, key TEXT NOT NULL UNIQUE, self_entity TEXT NOT NULL)
-- the world scope: shared, immutable, keyed by EventKey (step §4.2)
CREATE TABLE fact (event_key TEXT PRIMARY KEY, event_id TEXT NOT NULL, at INTEGER NOT NULL,
                   kind TEXT NOT NULL, place TEXT, digest TEXT NOT NULL)          -- full 64-hex sha256
CREATE TABLE fact_who (event_key TEXT NOT NULL, ordinal INTEGER NOT NULL, entity TEXT NOT NULL,
                       role INTEGER NOT NULL CHECK (role IN (1, 2, 3)),          -- QS28a-3, ruled
                       PRIMARY KEY (event_key, ordinal))
-- the person scope's base: one row per (seat, held fact) (step §4.3)
CREATE TABLE access (seat INTEGER NOT NULL, event_key TEXT NOT NULL, gist TEXT NOT NULL,
                     salience TEXT NOT NULL CHECK (salience IN ('notable','ambient')),
                     mine INTEGER NOT NULL CHECK (mine IN (0,1)), importance INTEGER NOT NULL,
                     PRIMARY KEY (seat, event_key))
CREATE INDEX access_by_key ON access (event_key)                                  -- the GC anti-join
-- P4/FG-b's private tables, each with a leading seat column, otherwise as schema 3 (A-3):
meta(seat, key, value)  PK (seat, key)                 -- cursor, newest, retention_day, forgotten_below
episodes(seat, episode, …)  PK (seat, episode); INDEX (seat, last_key)
episode_who(seat, episode, entity)  PK (seat, episode, entity)
chapters(seat, week, …)  PK (seat, week);  chapter_ranges(seat, week, first_key, last_key) PK (seat, week, first_key)
stable(seat, entity, …)  PK (seat, entity);  names(seat, entity, name, source_key) PK (seat, entity)
forgotten(seat, first_key, last_key)  PK (seat, first_key)
retention_passes(seat, trigger, …)  PK (seat, trigger)
-- index, never dumped (A-3's l0_text): rowid = access.rowid
CREATE VIRTUAL TABLE access_text USING fts5(gist, seat UNINDEXED, tokenize='unicode61 remove_diacritics 2')
```

- `seats.seat` is an internal integer, assigned on first `MemoryHome.seat(identity)`. It **never appears in
  any dump** (a dump names seats by `key`), so the order in which seats were opened cannot reach a dump
  (I-S28-5).
- `fact_who.role` (QS28a-3, ruled 2026-10-15): `1` participant only, `2` subject only, `3` both. The rows
  are the participants, deduplicated in envelope order (ordinals 0…), then the subjects that are not
  participants, in envelope order. Counterparts (A-9) = the rows with role 1 or 3, in ordinal order, minus
  `me`. "How" (step §4.3) is derived from `me`'s row: role 2 or 3 → `subject`, role 1 → `participant`, no
  row → `witness`.
- `PRAGMA auto_vacuum = INCREMENTAL` before the first table (A-6); rollback journal (no WAL; step §6.3).

### 5.3 The seat view (the projection, step §4.4)

`MemoryHome.seat(identity) -> SeatView` creates the `seats` row on first use (refusing `self_entity` that
differs from a stored one by `HomeIdentityMismatch("self_entity", …)`). A `SeatView` exposes exactly what
`MemoryStore` exposed (`identity`, `me`, `connection`, `transaction()`, `meta`/`set_meta`, `cursor`, `newest`,
`dump`, `size`, `content_size`, `row_bytes(table)`, `reclaim`, `fts`) plus `seat_id` (the internal integer, for
the port's `seat = ?` predicates), `hold(...)`, `release(keys)` and `holds(key)`.

The view's tables, in dump order (the `_DUMPED` order of A-4, `l0_who` after `l0`):

| View table | Rows | Columns, in dump order |
| --- | --- | --- |
| `meta` | `home_meta.world_instance`, `schema`; `seats.key` (as `seat`), `self_entity`; this seat's `meta` rows | `key, value` (key order) |
| `names` | this seat's `names` | `entity, name, source_key` |
| `l0` | `access(seat) ⋈ fact` | `event_id, at, kind, place, salience, mine, gist, digest, importance` |
| `l0_who` | for each held key: `fact_who` rows with role 1 or 3 and `entity ≠ me`, ordinal order | `event_id, entity` |
| `episodes`, `episode_who`, `chapters`, `chapter_ranges`, `stable`, `forgotten`, `retention_passes` | this seat's rows, `seat` dropped | schema 3's columns, in declared order |

Every read the port makes goes through `seat = view.seat_id` or through the access join for that seat; no
statement in a consumer names `fact` without joining `access` on its seat (AS28-14's scan).

### 5.4 The positional canonical encoding (QS28-5, `ARC-101`; the grammar this PR fixes)

- **A line** is the compact JSON array `[table, c1, …, cn]` (`separators=(",", ":")`, `ensure_ascii=False`,
  one `json.JSONEncoder` built once), then `"\n"`. `table` is the view table's name as a string. Columns are
  in the order of §5.3's table (the view) or §5.2's DDL (the home dump), with the `seat` column removed in a
  view and replaced by the seat's `key` in the home dump.
- **Codecs, per column** (`schema.CODECS`, one table, unit-tested): an `EventKey` column (`event_key`,
  `first_key`, `last_key`, `level_key`, `source_key`, `trigger`, and the `meta` values of `newest` and
  `forgotten_below`) and an `EventId` column (`event_id`, the `meta` value of `cursor`) are written as the
  **unpadded decimal integer** (`int(value)`; u64 fits Python's `int` exactly; JSON numbers, no quotes);
  `fact.digest` and the view's `l0.digest` are written as the **first 32 hex characters** (128 bits; the full
  64 stays in the table and is the re-delivery check, step §8.1); every other value as stored (`INTEGER` →
  number, `TEXT` → string, `NULL` → `null`). No float exists in any table.
- **`meta`** lines are `["meta", key, value]`; the identity rows (`world_instance`, `seat`, `self_entity`,
  `schema`) carry strings, `schema` the string `"4"`.
- **Row order** is each table's key order (§5.3); `l0_who` and `fact_who` by `(event key, ordinal)`.
- **The constant `K`** (QS28a-1, ruled): `schema.K_ACCESS_OVERHEAD_BYTES`, the largest number of bytes a
  one-seat home's dump spends on one held record beyond that record's lines in the seat view (the `access`
  line's tag, brackets, seat key and repeated id, and `me`'s `fact_who` line). It is a function of the encoding
  alone, never of the data's text. C8 derives it from §5.4's grammar for the longest seat key (64 bytes,
  `EntityKey`'s identifier bound, `sdk/python/src/mineworld_sdk/wire/ids.py:66-73`) and the longest u64
  `EventId` and `EntityId` (20 decimal digits each), pins it in `schema.py` with that derivation in its
  docstring, and AC-a-8 (b) checks it on real data. Under realistic seat keys the measured overhead is expected
  to be ≤ 48 B per held record; the pinned `K` is the derived worst case.
- **`SeatView.dump()`** = every view table's lines, in §5.3's order, concatenated. It is the standalone seat
  view: what I-S28-2 compares, what the bound measures (QS28-4), what S28-e exports.
- **`MemoryHome.dump()`** = `home_meta` lines; `fact` lines; `fact_who` lines; then, for each seat in `key`
  order, `["seat", key, self_entity]`, that seat's `access` lines (`["access", event_id, gist, salience,
  mine, importance]`) and its private tables' lines in §5.3's order (without the identity rows). It is the
  determinism artefact of the whole home (I-S28-5).

### 5.5 Opening, identity, lock, refusal

`MemoryHome.open(path: Path | Literal[":memory:"], world_instance: WorldInstanceId, *, verify: Verify =
"off") -> MemoryHome`:

1. `sqlite3.connect(path, isolation_level=None, timeout=0)`.
2. For a file: `PRAGMA locking_mode = EXCLUSIVE`, then `BEGIN EXCLUSIVE` + `COMMIT`. `sqlite3.OperationalError`
   "database is locked" → `HomeBusy(path)` (QS28a-4) after closing the connection. The lock is held until
   `close()` (or the process dies: the OS releases it, so a SIGKILL leaves nothing stale).
3. Empty database → create schema 4 (`auto_vacuum` first) and `home_meta`. A database with tables but no
   `home_meta` → if it has a `meta` table with a `schema` row (a P4/FG-b store) →
   `UnsupportedStoreSchema(found=<that value>, wanted=4)`, message: "rebuild it from the perceived stream:
   delete the file and start the seat with a null cursor (`perceived: { since: null }`)"; otherwise
   `NotAMemoryHome(path)`. `home_meta.schema ≠ "4"` → `UnsupportedStoreSchema`. `world_instance` differs →
   `HomeIdentityMismatch("world_instance", stored, given)`.
4. `verify`: `"off"` | `"pass"` | `"every-commit"` (step §5.4). The home checks I-S28-4 (`check_holders`) and
   every seat's meter against its dump after every retention pass (`"pass"`) or every commit
   (`"every-commit"`), raising `HolderInvariantBroken(keys)` or `HolderInvariantBroken(seat=…)`.

`SeatNotInHome(seat)` is raised by `MemoryHome.view_of(key)` for a seat key the home has no row for (the
read-only lookup S28-e's inspect will use); `seat(identity)` creates. `remove_seat(key)` releases every key
the seat holds (with GC, §5.6) and deletes its private rows, in one transaction (step §6.4 "a seat leaving the
home").

All errors subclass `ValueError` as today's do, carry their fields, and start their message with
`memory.home:` or `memory.scopes.world:` (step §5.5).

### 5.6 Hold, release and GC (one transaction; step §6.3, §6.4)

- `SeatView.hold(fact: PerceivedEvent, digest: str, rendered: Rendered) -> L0Record` — inside the caller's
  transaction: `world.insert_or_verify(db, fact, digest)` (absent → insert `fact` and `fact_who`; present →
  compare the full stored digest; different → `ConflictingFact(fact.id)`, raised from
  `memory.scopes.world`; `ConflictingFact` moves there and `memory.ingest` re-exports nothing — callers import
  it from `memory`), then insert the `access` row, then (notable) the `access_text` row, then add the seat's
  view bytes for this key to the meter (§5.7). Order is fixed: record before access.
- `SeatView.release(keys: Iterable[EventKey])` — inside the caller's transaction: delete this seat's
  `access_text`, `access` rows for `keys`, subtract their meter bytes, then `world.collect(db, keys)`:
  `DELETE FROM fact_who / fact WHERE event_key IN (keys) AND NOT EXISTS (SELECT 1 FROM access WHERE
  access.event_key = fact.event_key)`. The holders are the access rows; there is no stored count.
- **Ingest's three cases** (A-7, per seat): the seat holds the key → re-delivery, compare the digest with the
  record's → no-op or `ConflictingFact`; the seat does not hold it and `_represented` (per seat) → skipped and
  counted; otherwise → render, `hold`. A record present because a peer holds it is verified by `hold`, never
  trusted.
- `incremental_vacuum` runs after a retention pass's commit (A-6, unchanged), never inside a transaction.

### 5.7 Sizes, the meter and the bound (QS28-4; FG-b2 unchanged in its numbers)

- `SeatView.size()` = `len(SeatView.dump())` in bytes; `content_size()` = that less the `meta` lines (FG-b2's
  definition, A-5), kept incrementally per seat:
  - private tables: FG-b2's `TEMP` triggers, now passing `NEW.seat` / `OLD.seat`; the home keeps
    `sizes[(table, rowid)] = (seat, bytes)` and one running total per seat;
  - `l0` and `l0_who` view lines: added by `hold` and subtracted by `release` (they are the only writers of
    `access`, and records are immutable), computed with the same encoder from the record and the rendering;
  - a rollback drops every seat's cached sizes (A-5's rule), so the next call measures in full.
- `row_bytes(table)` returns `(cells, bytes)` for a view table in view order, the cells keyed by the **schema-3
  column names** the items code reads (`event_key` included, padded, as a cell — it is not dumped), so
  `retention/items.py` keeps its reads (A-11) and its sizes are seat-view bytes by construction.
- **The bound is unchanged** (`StoreBounds`, A-13): 20 MiB default, raise-only, `target` 75 %, `trigger`
  bound − bound/16, measured on `content_size()` of the **seat view**, each held record counted in full however
  many seats hold it. Nothing a peer holds, keeps or forgets changes a seat's size (I-S28-3).

### 5.8 The port, statement by statement (mechanical; A-10)

| Pattern today | Schema 4 |
| --- | --- |
| `… FROM <private table> WHERE …` / `INSERT INTO <private> (…)` / `UPDATE <private> … WHERE …` / `DELETE FROM <private> WHERE …` | the same statement with `seat = ?` added to the predicate (or the `seat` column first in the insert), `view.seat_id` bound first |
| `max(episode)` (`episodes.py:117`) | `max(episode) … WHERE seat = ?`: episode numbers are per seat |
| `SELECT … FROM l0 WHERE …` (ingest, retrieve, episodes' importance/mine, absorb's `_newest_at`, host's `check_structure`, items) | `SELECT … FROM access a JOIN fact f USING (event_key) WHERE a.seat = ? AND …`, same `ORDER BY`/`LIMIT` |
| `l0_who` reads (`records_between`) | `fact_who` rows of the held keys, role 1 or 3, `entity ≠ me`, `ORDER BY event_key, ordinal` |
| `l0_text JOIN l0 ON rowid` (retrieve FTS) | `access_text JOIN access a ON a.rowid = access_text.rowid … WHERE a.seat = ?` |
| `delete_records(db, keys, fts)` (plan.py) | `view.release(keys)` |
| `store.connection`, `MemoryStore` parameters and annotations | `view.connection`, `SeatView` |
| functions taking `db: sqlite3.Connection` (`plan.execute`, `host._log`, `_write_prose`, `check_structure`, `forgotten.*`) | take the `SeatView` (they need its `seat_id`) |

`records_between`, `NameBook.load/learn`, `Stable`, `Episodes`, `close_weeks`, `chapters`, `load_episodes`,
`stable_facts` and `retrieve` keep their signatures with `SeatView` for `MemoryStore`.

### 5.9 The tests' world: multi-seat streams

- **Synthetic** (`tests/home_streams.py`, test-only, imports no code under test): `world(seed, days, seats=12)`
  → one ordered list of `(PerceivedEvent, audience: frozenset[EntityKey])`, every draw SplitMix64 of
  `(seed, index, salt)` as `retention_stream.py` does; each seat's stream is the facts whose audience holds it,
  **the same `PerceivedEvent` object** for every member (F-S28-3: envelopes identical across observers). A
  fact's audience is drawn so the holder histogram resembles F-S28-2's (1 to 12 holders, about 13 % singly
  held). Every renderer's fact kind appears (`memory_fixtures` builders), plus `named` facts for name learning.
- **Real** (`conftest.social_cafe_12`, `real_server`-free: it runs `mineworld run worlds/social-cafe --headless
  --seed 7 --days 2 --save DIR` and `mineworld perceived … --person <key> --json` for the 12 people, once per
  session, through `binary.py`).

### 5.10 The kill harness (AS28-8)

`tests/home_kill.py` is a child script run as `sys.executable tests/home_kill.py <home> <scenario> <seed>
<kill-at>`. It ingests (and runs passes, through `absorb`) the synthetic world into a file home and, at the
`kill-at`-th call of a chosen point, calls `os._exit(137)` (no `finally`, no flush beyond what SQLite already
wrote; the same on all three OSes). The points are seams the child patches **in its own process** (test code;
nothing in production names them): `SeatView.hold` after the record insert and before the access insert;
`SeatView.release` after the access deletes and before `collect`; the home's `COMMIT`. The parent re-runs the
child without a kill point from the persisted cursors (re-delivery from each seat's cursor) and compares
`MemoryHome.dump()` with the uninterrupted run's.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-a-1 (I-S28-1) | A seat's access rows are exactly the keys `hold` was called for on its perceived stream, less those it released; no API reads a record except through the seat's access join | AS28-1; the AS28-14 scan |
| I-a-2 (I-S28-2) | Every `SeatView.dump()` and every rendered section is byte-identical under `none` and `home` | AS28-3, AS28-4 |
| I-a-3 (I-S28-3) | Nothing a home-mate perceives, keeps or forgets changes a seat's dump, size, pass decisions or sections | AS28-5 |
| I-a-4 (I-S28-4) | After every commit: `fact` keys = the union of `access` keys; every `fact_who` key has its `fact` | AS28-6, AS28-7 |
| I-a-5 (I-S28-5) | `MemoryHome.dump()` is a function of each seat's perceived prefix and configuration only | AS28-3 (interleavings), AS28-8 |
| I-a-6 (I-S28-6 as ruled, QS28a-1) | (a) **Sharing never costs disk:** `len(MemoryHome.dump())` of a shared home ≤ Σ over its seats of `len(MemoryHome.dump())` of that seat's one-seat home. (b) **Disk stays bounded:** each one-seat home's dump ≤ its seat view's dump + `K` × (records it holds) (§5.4), so a home is ≤ N × bound × (1 + `K` / smallest record line) | AS28-9 (AC-a-8) |
| I-a-7 (I-S28-7) | `memory/`, `compress/`, `forgetting/` read no save and no world log; only `memory/home/` connects | AS28-13 |
| I-a-8 (I-S28-10) | No path under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` changes | AS28-13 diff scan |
| I-a-9 | P4-1 … P4-n, FGb-1 … FGb-7 and FG-b2's FGb2-1 … FGb2-n hold for every seat view (the ported suites stay green) | §3.3's files |
| I-a-10 | A schema 2 or 3 store is refused by name and never modified | `test_home_open.py` |
| I-a-11 | One `ingest` call or one pass is one transaction over the home; a crash rolls back all of it | AS28-8 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | ruff, ruff format, pyright strict over `cognition/lm-controller` | CI `fast`, `python` |
| Unit | the encoder and codecs (`test_home_encoding.py`: one line per codec; a u64 key; a 128-bit digest; `null`; a non-ASCII gist); open/refuse/identity/lock (`test_home_open.py`); hold/release/GC and the meter (`test_home_store.py`, the port of `test_memory_store.py`; `test_home_gc.py`) | `cognition/lm-controller/tests/` |
| Ported | every file of §3.3, with only the API rename and re-derived (never hand-pinned) byte figures | same |
| Golden (C-2) | `test_home_golden.py` over `tests/golden/s28a_renders.json` | same |
| Harness | `test_home_transparency.py` (AS28-3/4/5), `test_home_kill.py` (AS28-8), `test_home_scale.py` (AS28-1, AS28-9, AS28-12, on the real 12-seat stream) | same |
| Scans | `test_memory_isolation.py` extended (AS28-13 source, AS28-14 writers) | same |
| Diff scan | AS28-13's "no change under kernel/ …" | C9, `git diff --name-only` (recorded) |
| Real binary | `social_cafe_12` (2 days) and the existing `social_cafe_100` users (`test_ac10.py`, `test_retention_real.py`) | `python` job, three OSes |
| Gate 1 (real LLM) | NOT REQUIRED: no model is called; FG-c's cassette replays (no live call) | — |

The tests' oracles are the dumps and the sections, never a value the home reports about itself; byte figures
are computed from the dump in the test, never pinned by hand (test rules §25).

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-a-1 | **Access oracle.** On the real 12-seat, 2-day stream, every key of every seat's access index and every citation of every line of 200 seeded `RecallQuery`s per seat is in that seat's `mineworld perceived` export; 0 exceptions | AS28-1 |
| AC-a-2 | **Transparency.** For 12 seats of the synthetic world (seed 7, 10 days), `SeatView.dump()` under one shared home equals the dump under 12 one-seat homes, byte for byte, at every pass point and at the end, for frames of 1, 7, 256 and 4 096 facts and three seeded seat interleavings (frame 1 on the first 2 days only, R-a-2); and for the real 12-seat stream at frame 256, one interleaving | AS28-3 |
| AC-a-3 | **Section identity.** For the same runs, 200 seeded `RecallQuery`s per seat render byte-identical `MemorySection.render()` under both | AS28-4 |
| AC-a-4 | **Isolation.** Seat A's dump and sections are identical when its eleven home-mates run `keep-all`, `decay` with threshold 1 000, or `importance-recency-cap` with every k at 0, compared with A alone in its own home | AS28-5 |
| AC-a-5 | **GC exactness.** Under `verify="every-commit"`, over the synthetic world, after every commit both anti-joins (`fact` without holder, held key without `fact`) are empty | AS28-6 |
| AC-a-6 | **Last holder.** A fact held by exactly two seats survives the first seat's `release` and is collected in the second's transaction; reversing the order gives the same final home dump | AS28-7 |
| AC-a-7 | **Kill/resume.** 50 seeded kills across the three points of §5.10 (17/17/16); after resume and re-delivery, the home dump equals the uninterrupted run's every time | AS28-8 |
| AC-a-8 | **Physical vs logical (QS28a-1, ruled).** At every pass point of AC-a-2: (a) the shared home's dump length ≤ Σ of the twelve one-seat homes' dump lengths; (b) each one-seat home's dump length ≤ its seat view's dump length + `K` × its held records, and the largest per-record overhead measured is recorded beside `K`; and every seat view ≤ its bound | AS28-9 |
| AC-a-9 | **Efficiency.** On the real 12-seat stream before retention (plain `ingest`), the home's bytes per world-day ≤ 0.30 × the schema-3 stores' (Σ of 12 `MemoryStore.dump()` lengths, measured at C1 on the same stream and recorded); also recorded, not gated: seat-view bytes per held record, and the days 20 MiB holds | AS28-12 |
| AC-a-10 | **Golden renders (C-2).** Every section in `s28a_renders.json` (captured at C1 from schema-3 code) is reproduced by schema 4 for every query, at every pass point at which schema 3 performed no bound-forced removal | C-2 |
| AC-a-11 | **No truth, no kernel.** Source scan: no `persistence` import, no save path, no `sqlite3.connect` outside `memory/home/` in `memory/`, `compress/`, `forgetting/`; no `INSERT`/`DELETE`/`UPDATE` naming `fact`, `fact_who`, `access`, `access_text` outside `memory/home/` and `memory/scopes/world.py`. Diff scan: nothing under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` | AS28-13, AS28-14 |
| AC-a-12 | **Refusal.** A schema-3 store file and a schema-2 one each raise `UnsupportedStoreSchema(found, 4)`, the file's bytes unchanged; a second `MemoryHome.open` of a held file raises `HomeBusy` on Linux, macOS and Windows | step §10; F-S28a-1 |
| AC-a-13 | **The FG-c cassette** replays with zero `CassetteMiss`, and its rendered sections equal the C1 golden; the pinned dump digest is re-pinned with that evidence (A-14) | AS28-10's FG-c half |
| AC-a-14 | **The ported suites** (§3.3) pass; each real-binary file within its pre-port wall time + 25 % on Linux CI (R-a-1) | I-a-9 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason. Targeted
validation per commit; the PR's CI is the one full run.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s28-a`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full,
   before editing it.
2. Make the numbered implementation steps, in order. Do not do work that belongs to a later commit.
3. Run the commit's validation commands. Record for each: the command, the test count, the wall time, and
   PASS / FAIL / INCONCLUSIVE (a test not run is not a pass).
4. For each mutation the commit lists: plant it, run the named test, record that it turned red (with the
   failing test's name), revert it from a byte copy (`cmp` clean), run the test again green.
5. Static checks: `uv run --locked ruff check cognition/lm-controller`; `uv run --locked ruff format --check
   cognition/lm-controller`; `uv run --locked pyright`. Tests one file at a time: `uv run --locked pytest
   cognition/lm-controller/tests/<file>` (one path, so the member's `addopts` and network guard apply).
   **R-LOAD-1:** never the whole member suite, never `cargo test --workspace` locally; CI is the gate.
   Markdown: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
6. Review: answer each review question of the commit in §15 in one or two sentences.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files**. Otherwise
   revert it, or stop and record why.
8. Update §15 (ledger row, evidence, findings) and the handoff in the same commit.
9. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` with the message given plus the
   session's attribution line. No heredoc, no `sed -i`, no `python3 -c`.
10. Stop and report (a material stop, §12) instead of improvising.

#### C0 — Freeze, base and ledger (Markdown only)

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s28a.md` (new).
- Steps:
  1. Record the base commit; confirm in `docs/DECISIONS.md` that `ARC-98` … `ARC-101` and `DEP-58` exist
     (S28-0 merged) and that `git log` shows FG-b2 (#213). Either missing → material stop.
  2. Re-read A-1 … A-15 on the base; record any line that moved (bounded) or any statement that changed (stop).
  3. Record which of P6b (#207) and S28-b have merged (R-P6b-a-1).
  4. Write the handoff (the contract's fields, the sequence C0 … C9).
- [ ] Implementation. [ ] Validation: the two doc scripts. [ ] Review: the base satisfies §2.3.
- **Commit message:** `docs(plan): S28-a — open the ledger at the base`

#### C1 — Multi-seat streams and golden renders, from the schema-3 code

- **Files:** `cognition/lm-controller/tests/{home_streams.py, capture_golden_renders.py,
  test_home_golden.py, golden/s28a_renders.json, golden/SOURCE.md}`, `tests/conftest.py` (`social_cafe_12`).
- Steps:
  1. `home_streams.py` as §5.9 (synthetic). Its own unit check lives in `test_home_golden.py`: the same seed
     gives the same streams; every fact of a seat's stream is the identical object in every member's stream;
     the holder histogram has every count 1 … 12.
  2. `conftest.social_cafe_12` as §5.9 (real), session-scoped, through `binary.mineworld`.
  3. `capture_golden_renders.py` (a script run by hand, documented in `golden/SOURCE.md` with the command and
     the base SHA): for the synthetic world (seed 7, 10 days, 12 seats), one `MemoryStore` per seat (schema 3,
     today's code), `absorb` with the default `consolidate` in day frames; at every pass point and at the end,
     200 `RecallQuery`s per seat drawn by SplitMix64 from `(seed, seat, point)` (time within the last day,
     0–3 counterparts from the cast, 0–3 words from the stream's vocabulary); write per (seat, point) the
     sha256 of the concatenated `render()` outputs and, for 3 queries, the full text (for human review). Also
     record Σ `dump()` lengths of 12 plain-`ingest` stores on `social_cafe_12` for AC-a-9 (`golden/SOURCE.md`).
  4. `test_home_golden.py`: today it opens `MemoryStore`s and asserts the golden; C4 switches it to seat views.
- Validation: `uv run --locked pytest cognition/lm-controller/tests/test_home_golden.py`; the script run once
  (record its wall time).
- [ ] Implementation: steps 1–3. [ ] Validation: the golden reproduces on the base code. [ ] Review: no
  query draws from a clock or `random`; the 3 sample texts read correctly (reviewed by hand).
- **Commit message:** `test(cognition): multi-seat streams and golden renders from schema 3 (S28-a)`

#### C2 — The home: schema 4, encoding, open, lock, refusal; decision records

- **Files:** `memory/home/{__init__.py, schema.py, encoding.py, home.py, errors.py}`, `memory/scopes/{__init__.py,
  world.py}`, `tests/{test_home_encoding.py, test_home_open.py}`, `docs/DECISIONS.md`.
- Steps:
  1. `schema.py`: §5.2 DDL exactly (with `fact_who.role`, QS28a-3), `SCHEMA_VERSION = 4`, the view
     tables of §5.3 with their columns and orders, `CODECS` (§5.4).
  2. `encoding.py`: `line(table, cells, codecs) -> str` and `line_bytes(...) -> int` (newline included).
  3. `errors.py`: the seven errors of §5.1 with their fields; message prefixes per §5.5.
  4. `home.py`: `MemoryHome.open` (§5.5 steps 1–3), `close`, `__enter__/__exit__`, `transaction()` (A-6's
     shape, dropping every seat's cached sizes on rollback), `seat(identity)`, `view_of(key)`, `seats()`,
     `dump()` (§5.4), `reclaim()`.
  5. `scopes/world.py`: `insert_or_verify`, `collect`, `check_holders` (§5.6).
  6. `docs/DECISIONS.md`: `ARC-109` and `DEP-61` in the file's record format, at the end of the file; dated notes under `ARC-101` (the §5.4 grammar) and `ARC-87` (the bound on the seat
     view, QS28-4).
  7. Tests: every codec of `CODECS` once; `open` creates, reopens, refuses a schema-3 file (the test writes a
     `meta` table by hand with the identity rows and `schema = "3"`, as schema 3 did, A-3), refuses a schema-2
     file the same way, refuses a save-like database (`NotAMemoryHome`), refuses another world instance; a
     second `open` of a held file raises `HomeBusy`. The second open runs in another process: the test starts
     `sys.executable tests/home_kill.py busy <path>` with `subprocess` (create `home_kill.py` here with only
     the `busy` scenario; C7 extends it), which prints the error's class name and exits.
- Validation: the two test files and `tests/home_kill.py busy` through `test_home_open.py`; doc scripts.
  Mutation **M-a-1** (`timeout=5` and no `BEGIN EXCLUSIVE`) → `test_a_held_home_is_busy` red.
- [ ] Implementation: steps 1–6. [ ] Validation: step 7, M-a-1. [ ] Review: the encoder is built once; no
  float; the schema-3 file's bytes are unchanged after the refusal (`sha256` before/after in the test).
- **Commit message:** `feat(cognition): the memory home — schema 4, positional dumps, lock and refusal (S28-a)`

#### C3 — The seat view: projection, dump, meter, hold, release, GC, verify

- **Files:** `memory/home/view.py`, `memory/home/home.py`, `memory/home/__init__.py`,
  `tests/{test_home_store.py, test_home_gc.py}`.
- Steps:
  1. `view.py`: `SeatView` (§5.3) with `dump`, `size`, `content_size`, `row_bytes`, the meter (§5.7), `hold`,
     `release`, `holds`, `meta`/`set_meta`, `cursor`, `newest`, `fts`.
  2. `home.py`: `verify` modes (§5.5 step 4); `remove_seat`.
  3. `test_home_store.py`: port every test of `test_memory_store.py` that concerns the store's own behaviour
     to a one-seat home; add: `content_size()` equals `size()` less meta after every `hold`/`release`/private
     write and after a rollback (two seats in one home); `row_bytes("l0")` sums to the `l0` lines of `dump()`.
  4. `test_home_gc.py`: AC-a-5 on the synthetic world with plain `hold`/`release` driven by a seeded script;
     AC-a-6 (both orders); `remove_seat` leaves no orphan.
- Validation: the two files. Mutations **M-a-2** (`collect` without `NOT EXISTS`) →
  `test_the_last_holder_collects` red; **M-a-3** (`hold` inserts access before the record and skips the digest
  check when a peer holds it) → `test_a_peers_record_is_verified_not_trusted` red; **M-a-4** (the meter forgets
  the `l0_who` lines) → `test_the_meter_is_the_dump` red.
- [ ] Implementation: steps 1–2. [ ] Validation: steps 3–4 and the mutations. [ ] Review: no statement in
  `view.py` reads `fact` without the seat's access join; GC runs only inside the releasing transaction.
- **Commit message:** `feat(cognition): seat views over shared fact records, with exact GC (S28-a)`

#### C4 — Ingest, names, the roll-up and retrieval on seat views

- **Files:** `memory/{ingest.py, names.py, retrieve.py, records.py, __init__.py}`, `compress/{episodes.py,
  chapters.py, stable.py, model_summarizer.py}`, `memory/store.py` (deleted), `tests/{memory_fixtures.py,
  test_memory_compress.py, test_memory_render.py, test_memory_retrieve.py, test_memory_summarizers.py,
  test_memory_isolation.py, test_home_golden.py, test_memory_store.py}` (the last deleted: C3 ported it).
- Steps:
  1. Port every A-10 site of these modules by §5.8; `ingest` uses `holds`/`hold` (§5.6); `involved` is
     computed from the fact and not stored (F-S28a-7); `ConflictingFact` is raised from `memory.scopes.world`
     and exported by `memory/__init__.py`.
  2. Delete `memory/store.py`; move `SCHEMA_VERSION` out of `records.py`; `memory/__init__.py` exports
     `MemoryHome`, `SeatView` and the errors instead of `MemoryStore`, `NotAMemoryStore`,
     `StoreIdentityMismatch` (no alias).
  3. Tests: replace `MemoryStore.open(":memory:", identity)` by a helper `memory_fixtures.seat_view(identity)`
     (a one-seat `":memory:"` home); `test_home_golden.py` now builds seat views (12 one-seat homes — the
     `none` arm) and must reproduce the C1 golden **for the plain-ingest points** (the pass points need C5).
- Validation: each listed test file. Mutation **M-a-5** (retrieve's L0 query without `a.seat = ?`) → in a
  two-seat home, `test_home_golden.py::test_sections_name_only_held_facts` red (add that test: two seats with
  disjoint streams, every citation held).
- [ ] Implementation: steps 1–2. [ ] Validation: step 3, M-a-5. [ ] Review: every ported statement has the seat
  predicate or the access join (grep `FROM episodes|FROM stable|FROM names|FROM chapters` in the diff).
- **Commit message:** `refactor(cognition): ingestion, roll-up and retrieval read seat views (S28-a)`

#### C5 — The retention host on seat views (mechanical)

- **Files:** `memory/retention/{absorb.py, host.py, plan.py, bound.py, items.py, forgotten.py, assisted.py}`,
  `tests/{retention_driver.py, test_assisted_host.py, test_lm_summarise.py, test_lm_summarise_replay.py,
  test_retention_host.py, test_retention_plan.py, test_retention_policies.py, test_retention_real.py,
  test_retention_retrieve.py, test_retention_size.py, test_retention_store.py, test_ac10.py, test_home_golden.py}`.
- Steps:
  1. Port A-10's retention sites by §5.8; `plan.delete_records` becomes `view.release`; functions taking
     `db` take the `SeatView`; `items.py` reads `row_bytes` unchanged (§5.7).
  2. Tests: the API rename; **every byte figure re-derived from the dump in the test** (never pinned by hand);
     `test_retention_size.py`'s streams lengthened where needed so that each test still crosses the trigger
     (positional bytes are smaller) — the bound is never lowered (FGb2-1); `test_lm_summarise_replay.py`'s
     dump digest re-pinned **only after** AC-a-13's two checks pass (zero `CassetteMiss`, sections equal to a
     golden captured at C1 by the same script for that cassette's stream — add that capture to C1's script if
     C1 did not; record the old and new digests in §15).
  3. `test_home_golden.py`: the pass points now (AC-a-10).
- Validation: each listed file, one at a time; record wall times against the base's (R-a-1). Mutations
  **M-a-6** (`release` drops the access rows but `items.py` keeps counting their bytes — e.g. the meter not
  updated in `release`) → `test_retention_size.py` (a named size test) or `test_the_meter_is_the_dump` red;
  **M-a-7** (a pass releases keys of another seat: `release` without `seat = ?`) → `test_home_gc.py` or
  `test_home_transparency.py` (C6) — run C3's `test_home_gc.py::test_a_pass_touches_only_its_seat` (add it here).
- [ ] Implementation: step 1. [ ] Validation: steps 2–3, mutations. [ ] Review: no policy, item rule, bound
  number or pass order changed (diff of `forgetting/` empty; diff of `bound.py` touches only the seat
  predicate).
- **Commit message:** `refactor(cognition): the retention host executes on seat views, with GC (S28-a)`

#### C6 — The transparency harness

- **Files:** `tests/test_home_transparency.py`, `tests/home_streams.py` (interleavings helper).
- Steps:
  1. `interleave(streams, frame, seed)`: the seats' frames merged in a seeded order (each seat's own order
     preserved).
  2. AC-a-2, AC-a-3 on the synthetic world (both arms built in one test per frame size, compared per seat at
     every pass point: `absorb` reports its passes; the harness snapshots each seat's dump and 200 sections
     after each), and on `social_cafe_12` (frame 256, one interleaving).
  3. AC-a-4 (isolation), with the home-mates' `Retention` from `retention_driver.retention(...)`.
- Validation: the file. Mutations **M-a-8** (a seat's `l0_who` view includes `me`) → red by name;
  **M-a-9** (the home dump orders seats by `seats.seat` instead of `key`) → the interleaving test red.
- [ ] Implementation: steps 1–3. [ ] Validation: the file, mutations. [ ] Review: the oracle is the `none` arm's
  dump, computed in the test, never the home's report.
- **Commit message:** `test(cognition): sharing is invisible — the 12-seat transparency harness (S28-a)`

#### C7 — Kill and resume

- **Files:** `tests/home_kill.py`, `tests/test_home_kill.py`.
- Steps: §5.10; 50 seeded kills (17 at the record insert, 17 in `release` before `collect`, 16 at `COMMIT`);
  resume = re-run with the persisted cursors; compare `MemoryHome.dump()`.
- Validation: the file (record wall time; R-a-2). Mutation **M-a-10** (`hold` commits the record in its own
  transaction before the access row) → AC-a-7 red.
- [ ] Implementation. [ ] Validation. [ ] Review: the seams are patched only in the child; nothing in
  `memory/` reads an environment variable or names a kill point.
- **Commit message:** `test(cognition): a killed home resumes to the uninterrupted dump (S28-a)`

#### C8 — Scale, efficiency and the scans

- **Files:** `tests/test_home_scale.py`, `tests/test_memory_isolation.py`.
- Steps: derive and pin `schema.K_ACCESS_OVERHEAD_BYTES` (§5.4; add `memory/home/schema.py` to this commit's
  **Files**); AC-a-1, AC-a-8 (a) and (b), AC-a-9 (measure; record bytes per held record and the days 20 MiB
  holds; F-S28a-8's margin), AC-a-11's source scan.
- Validation: the two files. Mutation **M-a-11** (an `INSERT INTO access` added in `compress/episodes.py`) →
  the writers scan red.
- [ ] Implementation. [ ] Validation. [ ] Review: AC-a-9's baseline is C1's recorded schema-3 figure, not a
  re-measurement with deleted code.
- **Commit message:** `test(cognition): access oracle, physical vs logical and efficiency on social-cafe (S28-a)`

#### C9 — Close-out

- **Files:** this file (§15), `handoff-s28a.md`, `cognition/lm-controller/README.md`.
- Steps: AC-a-11's diff scan (`git diff --name-only origin/main...HEAD`); the README line; fill §8's evidence;
  push `mvp0/pr-s28a`; `gh pr create`; CI on the exact head (Linux, macOS, Windows); repair to green.
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S28-a close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-S28b-a-1 (on S28-b).** S28-b keeps: forgotten-range attribution (step §5.6), C-4 (a)–(c), FU-S28-1, FG-b's
  real-binary Gate 2 on seat views, AS28-9 under `keep-all` at the bound, and the check that FG-b2's
  `trigger` margin holds in positional bytes for the largest single fact (F-S28a-8). It inherits the ported
  host; it does not port again.
- **R-S28c-a-1 (on S28-c).** `memory/scopes/world.py`'s functions become the `world` scope; the `person` scope
  re-homes `ingest`/`compress`/`retrieve`; `fact_who.role` (if ruled) is what anchor roles read.
- **R-S28e-a-1 (on S28-e).** `inspect access` derives "how" from `fact_who.role` (§5.2; QS28a-3, ruled).
- **R-P6b-a-1 (on P6b or this PR, whichever merges second).** P6b opens `MemoryHome.open(store/<instance>/
  memory.sqlite, instance)` and `home.seat(identity)` instead of `MemoryStore.open(…/<seat>.sqlite, identity)`
  (step §10's path); the full `sharing` wiring stays S28-f's.
- **R-S28f-a-1.** The record LRU (F-S28a-6).

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-28-shared-memory.md` §§4.1, 4.4, 5.2.2, 6.6, 8.2, 9.2, 10, 11, 12, 15 | "schema v3" → "schema 4"; `UnsupportedStoreSchema(found=2, wanted=3)` → `(found ∈ {2, 3}, wanted=4)`; golden renders from the schema-3 code at C1 (F-S28a-1). **Applied in PR #219 at the primary's request, with a dated note in the step's header** |
| `step-28-shared-memory.md` §3 I-S28-6, §15 AS28-9 | replace I-S28-6 with I-a-6 (a)/(b) and the constant `K` (QS28a-1, ruled) |
| `step-28-shared-memory.md` §4.2, §4.3, §5.2.3 | `fact_who.role` 1/2/3; "how" derived from it (QS28a-3, ruled) |
| `step-28-shared-memory.md` §5.5 | `HomeBusy(path)`, no `holder_pid` (QS28a-4, ruled) |
| `step-28-shared-memory.md` §11 | the S28-a / S28-b split (QS28a-2, ruled); the record LRU's owner (F-S28a-6) |
| `overall.md` | S28-a frozen / merged; `ARC-109`, `DEP-61` |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S28-a — the memory home (schema 4, seat views, positional dumps,
                         GC, transparency harness)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s28a-memory-home.md
RELATED / BINDING DOCS:  step-28-shared-memory.md (§§3, 4, 6, 7, 8, 9, 10, 11, 14.1, 15); pr-s10-p4-memory.md;
                         pr-fg-b-retention-host.md; pr-fg-b2-size-trigger.md; pr-fg-c-lm-summarise.md;
                         docs/CORE_CONCEPTS.md §5.3; docs/ARCHITECTURE.md §9.3; docs/DECISIONS.md ARC-59,
                         ARC-87, ARC-98…ARC-101, DEP-37, DEP-58; .structured-coding/standards.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s28-a, its own, held by one session
BRANCH:                  mvp0/pr-s28a, created from origin/main
START CONDITION:         S28-0 merged AND FG-b2 merged (yes) AND this design frozen, then the primary
                         session's dispatch                                               (primary)
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–8, as §14's rulings shape them                      (primary)
FROZEN INVARIANTS:       step-28 I-S28-1 … I-S28-10 (I-S28-6 replaced by I-a-6 (a)/(b), QS28a-1); §6 I-a-1 … I-a-11;
                         FGb-1 … FGb-7 and FG-b2's bound numbers unchanged; §8 as written
SEQUENCE:                C0 … C9 (§9)
ALLOWED COMMANDS:        uv run --locked *; uv sync --locked; cargo build -p mineworld-cli (the binary for
                         social_cafe_12 / social_cafe_100); git; gh (never merge); python3 scripts/*;
                         target/*/mineworld *; mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or the
                         whole member suite locally (R-LOAD-1); reading any key or .env; calling any model;
                         any record mode for cassettes; edits outside APPROVED SCOPE
MATERIAL STOPS:          S28-0 not merged; a needed change to a policy, an item rule, a bound number or the
                         pass order; a needed change outside cognition/lm-controller and docs/DECISIONS.md;
                         a new dependency or a uv.lock change; AC-a-2/3/4 failing for a reason the port
                         cannot fix (a peer-dependence in the design); AC-a-9 > 0.30 after C8; the HomeBusy
                         test failing on a platform (DEP-61's fallback is a decision, not an edit); a
                         re-recorded cassette (never)
PLATFORMS:               Linux, macOS, Windows (python job on all three)
VALIDATION BUDGET:       unit and harness files unrestricted locally one at a time; each real-binary file
                         ≤ 90 s on Linux CI; the harness ≤ 120 s per file on Linux CI (R-a-2); CI: the PR's
                         runs. Gate 1: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s28a.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                                                          (primary, 2026-10-15)
                         semantic commits: authorized                     (working rules §14)
                         branch push: authorized                          (working rules §21)
                         PR creation / update: authorized                 (working rules §21)
                         CI repair to review readiness: authorized        (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized — never by the implementation session; only with the
                         operator's explicit authorization after the primary's and an Opus logic review
                         (working rules §22)
POST-MERGE SYNC OWNER:   primary: step-28, overall.md, decision ids; implementation session: this ledger
STOP CONDITION:          S28-a READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-a-1 | The access ⋈ fact join makes retrieval and passes slower | `access(seat, event_key)` PK and `fact` PK serve every join; AC-a-14 bounds wall time; a regression > 25 % is recorded with a profile and is a material stop only if it breaks a real-binary budget |
| R-a-2 | The harness (12 seats × 4 frames × 3 interleavings × 2 arms) is heavy | synthetic streams; frame 1 on 2 days only; real data at one configuration; ≤ 120 s per file on Linux CI, else split across files |
| R-a-3 | SQLite exclusive locking behaves differently on Windows or network filesystems | the `HomeBusy` test runs on all three OSes in CI; network filesystems are out of scope (documented); fallback is `DEP-61`'s recorded alternative, by decision |
| R-a-4 | A subtle peer dependence (rowids, FTS rowids, seat ordinals) leaks into a dump | dumps never contain rowids or `seats.seat`; seats are ordered by `key`; AC-a-2 with interleavings; M-a-9 |
| R-a-5 | P6b (#207) and this PR conflict on the store API | R-P6b-a-1; whichever merges second adapts; the open path is the step's |
| R-a-6 | The PR is large (≈ 30 files) | ten semantic commits; the port is mechanical and table-driven (§5.8); Opus implementer and an Opus logic review |
| R-a-7 | Positional figures change what FG-b2's size tests cross | streams lengthened, bound never lowered (C5 step 2) |
| R-a-8 | Users with schema-3 stores on disk lose memory | QS28-8 (ruled): memory is a derivation, rebuilt from the perceived stream; P6b is not merged, so no store exists outside test runs |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **QS28a-1 [operator — frozen invariant I-S28-6 / AS28-9]** | I-S28-6 says the home's canonical bytes are ≤ the sum of its seat views. For a record held by exactly one seat the home stores the record plus an access line that names the seat and repeats the key, so the home is larger than that seat's view by a fixed overhead per singly held record (F-S28a-2). Under `sharing: none` every record is singly held and the invariant fails by construction. Amend it? | **Yes, to two exact statements that keep its purpose:** (a) **sharing never costs disk** — `len(MemoryHome.dump())` of a shared home ≤ Σ over its seats of `len(MemoryHome.dump())` of each seat's one-seat home (true by construction: the shared home stores each record once); (b) **disk stays bounded** — each one-seat home's dump ≤ its seat view's dump + K × (records it holds), with K the encoding's fixed per-access overhead, measured at C8 and pinned as a named constant (expected ≤ 48 B, ≈ 25 % of a record's view bytes), so the home is ≤ N × bound × (1 + K / smallest record line). AS28-9 checks both. The 20 MiB promise (QS28-4) is untouched: it measures the view. The alternative — inlining singly held records into access rows until a second holder appears — makes GC promote and demote rows and is rejected as complexity for a few bytes |
| **QS28a-2 [primary / operator — the step's PR split]** | Replacing the store requires porting the retention host's SQL (it reads and deletes `l0`), which the step assigns to S28-b. Keeping `MemoryStore` alive until S28-b is a compatibility layer. Move the **mechanical** port of retention (items read seat-view bytes, execution releases with GC, the size trigger reads the seat view) into S28-a? | **Yes.** It is one storage swap, and `main` stays green at each merge. S28-b keeps everything that changes behaviour or adds checks (R-S28b-a-1). S28-a's implementer is already Opus, as the step wants for both |
| **QS28a-3 [operator — the shared record's shape]** | The step's "how" (subject / participant / witness) and S28-d's anchor role `subject` need the fact's subjects, which §4.2's record does not keep (F-S28a-4). Add a `role` to `fact_who` now? | **Yes, now.** `fact_who` rows become the union of participants and subjects with `role` 1/2/3 (§5.2). It costs a few bytes per row (charged in full, QS28-4), and it avoids a schema 5 when S28-d lands. Counterparts and recall are unchanged |
| **QS28a-4 [primary — a named error's fields]** | `HomeBusy(path, holder_pid)`: SQLite's own lock, chosen for reuse and crash-safety (§4), cannot tell the holder's pid. Drop the field, or add a side lock file that records it? | **Drop `holder_pid`.** `HomeBusy(path)` names the file; the operator finds the process with the platform's tools. A side file can go stale after a SIGKILL and is a second lock that can disagree with SQLite's |

### 14.1 Rulings, 2026-10-15 (relayed by the coordinator; binding)

| Question | Ruling | By |
| --- | --- | --- |
| QS28a-1 | **Replace I-S28-6 with the two exact statements, as recommended; name the constant `K`.** Applied: I-a-6 (a)/(b) (§6), AC-a-8 (§8), `schema.K_ACCESS_OVERHEAD_BYTES` (§5.4), C8 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| QS28a-2 | **Yes.** The mechanical port of the retention SQL moves into S28-a. Applied: §2.1 item 4, C5; S28-b keeps R-S28b-a-1 | Ruled 2026-10-15 (primary) |
| QS28a-3 | **Add the `role` column to `fact_who` now.** Applied: §5.2, C2, R-S28e-a-1 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| QS28a-4 | **Drop `holder_pid`.** Applied: `HomeBusy(path)` (§5.5, F-S28a-5) | Ruled 2026-10-15 (primary) |

No question remains open. Decision ids: `ARC-109` (the home) and `DEP-61` (SQLite locking), assigned by the
primary 2026-10-15 and confirmed free on `origin/main` (S29, #218, took `ARC-105` … `ARC-108` and `DEP-60`).

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C9 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; M-a-1 … M-a-11 as §9 lists them)

### Deviations and findings

F-S28a-1 … F-S28a-8 (§3.2), recorded at design time.
