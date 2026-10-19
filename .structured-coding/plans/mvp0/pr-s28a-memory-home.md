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

**Amendment to AC-a-9 (operator, 2026-10-18, ruled in the primary session; landed in PR 12d #171, step-11 §19.13
TD-D16 item 13).**
- **The new rule.** On the real 12-seat stream before retention, the seat views hold at most
  `PER_RECORD_BOUND` = 180 bytes per held record (`test_home_scale.py`). The bound is normalised to the history.
- **Why.** The original rule compared the home with C1's schema-3 stores, which were measured on another
  history: the pre-12d café, with code since deleted.
- **The value.** 12d's café (bodies, persona, appearance) measured 145.5 B per record, with a home of
  4 174 666 B, that is 0.356 of the schema-3 figure; the bound adds a 24 % margin.
- **What remains of the old rule.** The 0.30 ratio against 11 712 772 B is printed as advisory only.

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
| C0 | (this commit) | [x] base, decisions, A-1 … A-15 re-read, handoff | [x] doc scripts PASS | [x] §2.3 satisfied |
| C0 | `817d202b` | [x] | [x] | [x] |
| C1 | `e2e5dab9` | [x] streams, `social_cafe_12`, capture script, golden, test | [x] golden reproduces on the base code (4/4 PASS) | [x] no clock/`random`; samples read by hand |
| C2 | `1cc4ce33` | [x] schema, encoding, errors, home (open/lock/refuse/transaction/dump), `scopes.world`, `ARC-109`, `DEP-61`, notes | [x] 4 + 8 tests PASS; M-a-1 red by name | [x] one encoder; no float; refused file byte-identical |
| C3 | `402426b5` | [x] `view.py` (projection, dump, meter, hold/release, `records_between`), `home.py` (seat, view_of, remove_seat, meter, verify) | [x] 7 + 4 tests PASS; M-a-2, M-a-3, M-a-4 red by name | [x] `fact` read only through the seat's access join; GC only in the releasing transaction |
| C4 | `7ec76cfd` | [x] ingest, names, retrieve, records, `memory/__init__`, compress ×4, controller ×3 (D-a-1) ported; `store.py` deleted | [x] 9 files PASS; golden plain-ingest points PASS (retention arms red until C5, D-a-9); M-a-5 red by name | [x] every ported statement has the seat predicate or the access join |
| C5 | `419df400` | [x] retention host ported (absorb, host, plan, bound, forgotten; items reads `row_bytes` unchanged); `delete_records` → `SeatView.release` | [x] 20 files PASS one at a time; golden all arms PASS; digest re-pinned after AC-a-13; M-a-6, M-a-7 red | [x] no policy, item rule, bound number or pass order changed; `forgetting/` untouched |
| C6 | `e7b35625` | [x] `interleave`, `home_harness.py`, the harness in four files (D-a-13) | [x] AC-a-2/3/4 and I-S28-5 PASS (synthetic: 6 + 6 + 4 tests; real: 1); M-a-8, M-a-9 red | [x] the oracle is the `none` arm, computed in the test |
| C8 | `d5b745b7` | [x] `K_ACCESS_OVERHEAD_BYTES` = 93 derived and pinned; `test_home_scale.py`; writers and save scans | [x] 4 + 5 tests PASS; AC-a-9 ratio 0.257; M-a-11 red | [x] AC-a-9's baseline is C1's recorded schema-3 figure |
| C7 | `24c5ca07` | [x] `home_kill.py kill` (three seams, patched in the child), `drive` (re-delivery from each seat's cursor), `test_home_kill.py` | [x] 50 kills PASS (17/17/16); M-a-10 red | [x] seams patched only in the child (and, for counting, by the test's `MonkeyPatch`); nothing in `memory/` names a kill point or reads the environment |
| merge | `09ea595c` | [x] `origin/main` merged; `docs/DECISIONS.md` rebuilt: main's records, S28-a's two notes, `ARC-109` and `DEP-61` appended last | [x] decision ids 150 distinct, headings 240 none duplicated; ruff/pyright PASS | [x] no other file conflicted |
| C9 | (this commit) | [x] README line, diff scan, §8 evidence, handoff | [ ] CI on the exact head (filled after the run) | [x] deviations recorded (D-a-1 … D-a-13, F-S28a-9, F-S28a-10) |

### Evidence

**C0 (2026-10-15).** Implementation base `origin/main @ b37733b9` (#227, S28-0). `docs/DECISIONS.md` holds
`ARC-98` (l. 8414), `ARC-99` (8446), `ARC-100` (8470), `ARC-101` (8496), `DEP-58` (8521); `ARC-109` and `DEP-61`
are absent (free). FG-b2 is on the base (`dffe8458` merges #213; the size trigger is in
`memory/retention/{absorb,bound}.py`). P6b **merged** as #207 (`574e5eeb`); S28-b not merged.
A-1 … A-15 re-read on the base: every statement holds; line numbers are unchanged within ±2 lines. **New sites
since the audit** (P6b merged after `2d07c6c`): `controller/seat.py` (opens `MemoryStore` at
`<store_dir>/<seat>.sqlite`), `controller/triggers.py` (`SELECT entity FROM stable`), `controller/memory_view.py`
(an annotation), and `tests/{test_controller_parts.py, test_controller_seat.py, test_assignment.py}` — see D-a-1.
Validation: `python3 scripts/check_doc_headings.py` → "239 numbered sections across 33 documents, none
duplicated" PASS; `python3 scripts/check_decision_ids.py` → "138 decision ids, all distinct" PASS.

**C1 (2026-10-15).** `tests/home_streams.py` (synthetic `world(seed, days)`, 60 facts a day, one quiet day in about
nine; `seat_streams`; `day_frames`; `queries`; the real `cafe12`), `tests/conftest.py::social_cafe_12`,
`tests/capture_golden_renders.py`, `tests/golden/{s28a_renders.json, SOURCE.md}`, `tests/test_home_golden.py`.
- Capture: `uv run --locked python cognition/lm-controller/tests/capture_golden_renders.py --cafe` on the schema-3
  code (C0 head `817d202b`): 21.9 s wall, load 2.8. AC-a-9's baseline: Σ of the 12 schema-3 plain-`ingest`
  dumps on `social_cafe_12` = **11 712 772 B** (2 world-days; 5 964 facts in the world, 31 651 perceived).
- `uv run --locked pytest cognition/lm-controller/tests/test_home_golden.py`: 4 passed in 18.95 s (load 3.9) —
  **PASS**: the golden reproduces on the base code; the stream check holds (same seed, same streams; one
  `PerceivedEvent` object per fact across its audience; holder counts 1 … 12 all present; 8–18 % singly held).
- ruff check, ruff format --check, pyright (strict, `cognition/lm-controller`): PASS (0 errors).
- Review: every draw is SplitMix64 of `(seed, index, salt)` (`retention_stream.draw`); `queries` draws from
  `(seed, seat, point)`; no clock and no `random` import. The three sample renders (Alice, end of `ingest`)
  were read by hand: names learned from `named` facts (strangers named mid-stream are known only to their
  audience, unnamed ones are `someone`), places are `somewhere` (no place is named), every line ends with its
  citation, the four headers appear in P4's order.
- Real-stream probe (scratch, not evidence for a criterion): 5 964 facts, every envelope identical across its
  perceivers; holders 1: 794, 2: 1 796, …, 12: 1 289; **5 facts (`passage-opened`) have two subjects and no
  participant** (F-S28a-9).

**C2 (2026-10-15).** `memory/home/{__init__, schema, encoding, errors, home}.py`, `memory/scopes/{__init__,
world}.py`, `tests/{test_home_encoding.py, test_home_open.py, home_kill.py}`, `docs/DECISIONS.md` (`ARC-109`,
`DEP-61`, notes under `ARC-87` and `ARC-101`), `tests/test_memory_store.py` (D-a-7).
- `uv run --locked pytest …/test_home_encoding.py`: 4 passed in 0.01 s — PASS. `…/test_home_open.py`: 8 passed in
  0.40 s — PASS (create/reopen with `auto_vacuum = 2`; schema 2 and 3 refused with `found`, `wanted = 4`, the
  rebuild instruction and the file's sha256 unchanged; schema 5 refused; a save-like database is
  `NotAMemoryHome`; another instance is `HomeIdentityMismatch` and changes nothing; `HomeBusy` across processes
  for a created and a reopened home, `opened` once closed; a holder killed by `os._exit` leaves no lock).
  `…/test_memory_store.py`: 12 passed — PASS (D-a-7).
- ruff check, ruff format --check, pyright strict: PASS. `check_doc_headings.py`: 239 sections, none duplicated;
  `check_decision_ids.py`: 140 ids, all distinct — PASS.
- Review: `encoding._ENCODER` is built once at import; no column holds a float (the DDL has no `REAL`); the
  refusal test hashes the file before and after; `seat()`/`view_of()` arrive with `SeatView` at C3 (D-a-6).

**C3 (2026-10-15).** `memory/home/view.py` (new), `memory/home/{home.py, __init__.py}`, `tests/{test_home_store.py,
test_home_gc.py}`; `tests/test_home_open.py` (one line reformatted: C2 committed it unformatted, D-a-8).
- `…/test_home_store.py`: 7 passed in 0.03 s — PASS (a seat's identity kept and another self refused by name with
  the home unchanged; cursor and identity across a reopen; ids ordered up to u64 and written as numbers; `l0_who`
  is the other participants in envelope order, per seat; the meter equals the dump less `meta` after holds,
  private writes, a release and a rollback, two seats in one home; `row_bytes` of `l0`/`l0_who` sums to their
  dump lines, before and after the meter; the file is released on close).
- `…/test_home_gc.py`: 4 passed in 0.08 s — PASS (AC-a-5 over `world(7, 3)` with a seeded release script, the
  test's own two anti-joins after every commit, under `verify="every-commit"`; AC-a-6 in both orders, the record
  surviving the first release and collected inside the second's transaction, the final home dumps equal; a seat
  leaving takes only what no one else holds and leaves the other seat's view unchanged; a peer's record verified:
  a forged re-delivery is `ConflictingFact` and changes nothing).
- ruff, ruff format, pyright strict: PASS.
- Review: every statement in `view.py` that names `fact` or `fact_who` joins `access` on `a.seat = ?`
  (`_L0_SELECT`, `_WHO_SELECT`, `held_digest`, `records_between`); `world.collect` is called only from
  `SeatView.release`, inside the caller's transaction; `MemoryHome.remove_seat` releases inside its own.

**C4 (2026-10-15).** Ported by §5.8: `memory/{ingest, names, retrieve, records, __init__}.py`,
`compress/{episodes, chapters, stable, model_summarizer}.py`, `controller/{seat, triggers, memory_view}.py` (D-a-1);
`memory/store.py` deleted (no alias); `memory/retention/{absorb, bound, forgotten, host, items}.py` import-only
(D-a-9). Tests: `memory_fixtures.py` (`seat_view`, `open_view`, `rows`, `all_rows`), `test_memory_{compress,
render, retrieve, summarizers, isolation}.py`, `test_home_store.py` (P4's ingestion tests moved in, D-a-10),
`capture_golden_renders.py` and `test_home_golden.py` (seat views; `test_sections_name_only_held_facts`),
`golden/SOURCE.md`; `test_memory_store.py` deleted.
- One file at a time (load 8.4–23 from other sessions): `test_home_store` 11 passed; `test_home_gc` 4; `test_home_open`
  8; `test_home_encoding` 4; `test_memory_compress` 9; `test_memory_render` 6; `test_memory_retrieve` 10;
  `test_memory_summarizers` 7; `test_memory_isolation` 3 — all PASS. `test_home_golden`: 3 passed, 2 failed in
  16.9 s — the plain-ingest golden (`test_plain_ingest_renders_the_golden_sections`) and the held-facts test
  **PASS**; `test_retention_renders_the_golden_sections` and `test_the_fgc_cassette_replays_to_the_golden_sections`
  **FAIL** with `no such table: l0_text` in `retention/plan.py` — the retention host is ported at C5 (§9 C4 step 3:
  "the pass points need C5"; D-a-9).
- ruff, ruff format: PASS. pyright: `src/` 0 errors; the errors left are all in C5's test files (§3.3's retention
  and controller tests, which still name `MemoryStore`).
- Review: `grep` of the changed modules for `FROM|INTO|UPDATE` on a private table without `seat` lists only
  statements whose seat predicate is on the next line (read); every `fact` read joins `access` on the seat.

**C5 (2026-10-15).** Ported by §5.8: `memory/retention/{absorb, host, plan, bound, forgotten}.py` (`plan.execute`,
`plan.remove`, `forgotten.normalize`, `host._log`, `host._write_prose`, `host.check_structure` take the
`SeatView`; `plan.delete_records` is gone, a record is removed by `SeatView.release`; `run_pass` opens its
transaction as a retention pass for `verify="pass"`); `items.py` and `assisted.py` needed nothing beyond C4's
import (items reads `row_bytes`, whose cells keep schema 3's names). Performance (bounded, R-a-1): the codecs of a
view table are derived once (`schema.ViewTable.codecs` was a property rebuilt per line) and `encoding.line` skips
the identity codec. Tests: `retention_driver.py`, `retention_oracle.py` (positional parse with its own column list,
keys padded back), `test_{assisted_host, lm_summarise, lm_summarise_replay, retention_host, retention_plan,
retention_policies, retention_real, retention_retrieve, retention_size, retention_store, ac10, controller_parts,
controller_seat}.py`, `test_home_gc.py::test_a_pass_touches_only_its_seat`.
- One file at a time, wall (load average at start): `test_retention_plan` 10 passed 1.1 s; `test_retention_retrieve`
  2, 0.01 s; `test_retention_store` 8, 0.03 s; `test_assisted_host` 9, 3.8 s; `test_controller_parts` 7, 0.08 s;
  `test_controller_seat` 16, 0.22 s; `test_assignment` 9, 0.10 s; `test_retention_host` 14, 26.3 s (12.6);
  `test_lm_summarise` 12, 4.9 s; `test_lm_summarise_replay` 5 + 1 skipped (the recording), 3.1 s;
  `test_retention_size` 15, 273.8 s (9.1); `test_retention_policies` 23, 105.2 s (4.7; 82.9 s user against
  71.5 s user on the base code, +16 %); `test_ac10` 8, 74.4 s (9.3; base 68.0 s, +9 %); `test_retention_real` 2,
  81.7 s (4.3; base 65.7 s, +24 %); `test_home_golden` 5, 19.8 s (2.4) — all **PASS**. Base timings ran the
  base tree (`git archive b37733b9`) against the same binary and venv, one at a time.
- **AC-a-13.** The committed cassette replays with zero `CassetteMiss` (the replay gateway raises on a miss; every
  replay test passed) and its sections equal the C1 golden (`test_the_fgc_cassette_replays_to_the_golden_sections`
  PASS); the dump digest was then re-pinned: old `c3624e035b7a8044a932bc217ba5bc51ee11e2f6732e49a9500015f423a106a4`,
  new `2163d85b6750f4816c6e2d213e694771e4cfec69f67d3a79f3f141389c6df00a`. The cassette file is unchanged
  (`git diff --stat tests/cassettes` empty).
- **AC-a-10.** `test_home_golden.py` reproduces all three arms of the golden on schema 4: every plain-ingest point,
  every pass point (no point of the golden records a bound-forced removal: `forced` is 0 throughout), and the FG-c
  arm.
- Byte figures re-derived, never hand-pinned (C5 step 2): `test_retention_size.BUSY`'s crowd 100 → 300 (D-a-12);
  `test_retention_real`'s keep-all window 30…40 → 75…85 (D-a-12); first forced keep-all pass measured at day 81.
- ruff, ruff format, pyright strict: PASS.
- Review: `git diff b37733b9 -- …/forgetting` is empty; `bound.py`'s diff is the seat predicate in
  `_stable_facts` and the `execute`/`remove`/`normalize` signatures; no `StoreBounds` number, item rule or pass
  order changed; `host.check_structure` reads the seat's keys from `access` only.

**C6 (2026-10-15).** `tests/home_streams.py::interleave` (each step draws one seat with frames left, SplitMix64 of
`(seed, step)`; each seat's order kept), `tests/home_harness.py` (the `none` and `home` arms; every seat snapshot
right after each of its passes — `absorb`'s `run_pass` wrapped in this process — and at the end: the sha256 and
length of its seat-view dump and the sha256 of its 200 renders; in the `home` arm a seat joins at its first frame,
so the internal seat numbers follow the interleaving), and four test files (D-a-13).
- `test_home_transparency.py` (frames 256 and 4 096 × interleavings 1, 2, 3): 6 passed in 61.7 s (load 6.9) — PASS.
- `test_home_transparency_frames.py` (frames 1 on days 0–1, and 7 × 3 interleavings): 6 passed in 42.7 s (load 13.4)
  — PASS (a first run failed on the test's own expectation of ≥ 3 snapshots for frames of 1: two days hold one
  pass and the end, 2 snapshots; the expectation was corrected to 2).
- `test_home_isolation.py` (AC-a-4 × `keep-all`, `decay` threshold 1 000, `importance-recency-cap` every k 0; the
  home dump identical for (frame, interleaving) = (256, 1), (256, 2), (4 096, 3)): 4 passed in 15.8 s — PASS.
- `test_home_transparency_real.py` (the real 12-seat stream, frame 256, interleaving 1; run under
  `cast-slots/with-slot.sh`, which waited 16 min 40 s for a slot): 1 passed in 9.14 s — PASS.
- ruff, ruff format, pyright strict: PASS.
- Review: every assertion compares the `home` arm with the `none` arm computed in the same test (or the home dumps
  of several runs with each other); no value the home reports about itself is asserted.

**C7 (2026-10-15).** `tests/home_kill.py` gains `drive` (the kill world: 4 seats of `world(7, 3)`, frames of 64,
interleaving 5, `absorb` under `consolidate`, into a file home; each seat delivered what lies above its persisted
cursor) and `kill` (`os._exit(137)` at the n-th call of a seam: `record` after `world.insert_or_verify` returns
inside `SeatView.hold`, `release` before `world.collect` inside `SeatView.release`, `commit` before the home's
`_commit`). `tests/test_home_kill.py`: the uninterrupted run once (in the test's process, seams counted through
`MonkeyPatch`), then 50 cases, the n-th call drawn by SplitMix64 of `(28, case)` over that seam's call count; each
case kills a child, opens the killed file and checks I-S28-4 on its committed state with `world.check_holders`,
re-delivers in-process from the persisted cursors, and compares `MemoryHome.dump()` with the uninterrupted run's.
- `uv run --locked pytest …/test_home_kill.py`, under `cast-slots/with-slot.sh`, load average 4.05 at start: **50
  passed in 9.78 s** — PASS (17 at `record`, 17 at `release`, 16 at `commit`).
- ruff, ruff format, pyright strict: PASS.
- Review: `grep` of `src/` for `_exit`, `environ`, `kill` finds nothing; the only production seam is the module
  function `home._commit`, which names no kill point (it is the home's one `COMMIT`).

**C8 (2026-10-15).** `memory/home/schema.py::K_ACCESS_OVERHEAD_BYTES` = **93**, derived in its docstring from the
grammar alone (34 for `fact` + `access` against `l0`; 59 for the `fact_who` line of a record with no counterpart
and one row the view does not show, ids and entities of 20 digits); `tests/test_home_scale.py`;
`tests/test_memory_isolation.py` (`test_only_the_home_writes_the_shared_records_and_the_access_index`,
`test_memory_reads_no_save_and_no_world_log`).
- `test_home_scale.py` under `with-slot.sh` (load 4.5): **4 passed in 25.97 s** — PASS.
  - AC-a-8 synthetic (frames of 7, 494 frame ends): (a) and (b) hold at every frame end; every view ≤ 20 MiB;
    largest average overhead per held record of a one-seat home over its view **40 B** (K = 93).
  - AC-a-8 real (frames of 64, 498 frame ends): (a) and (b) hold; largest average overhead **27 B**.
  - AC-a-1: every access key and every citation end of 200 queries per seat lies in that seat's export: 0
    exceptions.
  - **AC-a-9: home 3 012 914 B against 11 712 772 B for the twelve schema-3 stores (C1): ratio 0.257 ≤ 0.30**
    — 1 506 457 B per world-day; seat-view bytes per held record 150.9 (recorded, not gated); the busiest seat
    perceives 217 324 B of seat view a day, so 20 MiB holds ≈ 96 days of it in full detail (recorded).
  - F-S28a-8's margin in positional bytes: FG-b2's trigger margin is bound/16 = 1.25 MiB; a counterpart's
    `l0_who` line is 15 + id + entity digits (≈ 30 B on social-cafe), so the largest single fact the margin
    absorbs names ≈ 43 000 participants (≈ 3 000 in schema 3's objects). The check under the forced backstop is
    S28-b's (R-S28b-a-1).
- `test_memory_isolation.py`: 5 passed — PASS. ruff, ruff format, pyright strict: PASS.
- Review: AC-a-9 reads `cafe12_schema3` from the golden captured at C1 on the schema-3 code; nothing re-measures it.
  K is asserted only as the bound of (b); the measured overheads (40 B, 27 B) are printed beside it, and the
  design's ≤ 48 B expectation for realistic keys holds.

**C9 (2026-10-15).** AC-a-11's diff scan: `git diff --name-only b37733b9` lists nothing under `kernel/`,
`contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` (0 paths); the changed paths are
`cognition/lm-controller/**`, `docs/DECISIONS.md` and this effort's two plan files. `cognition/lm-controller/README.md`
gains the line §2.1 item 8 asks (memory is a seat view of a home; a pre-schema-4 file is refused and rebuilt).

**§8 acceptance, as measured (local; CI on the exact head is the gate).**

| AC | Result | Evidence |
| --- | --- | --- |
| AC-a-1 | PASS | C8: 0 exceptions over 12 seats' access keys and 200 queries' citations |
| AC-a-2 | PASS | C6: synthetic, frames 1 (2 days), 7, 256, 4 096 × 3 interleavings; real, frame 256 |
| AC-a-3 | PASS | C6: the same runs, 200 sections per seat and point |
| AC-a-4 | PASS | C6: `keep-all`, `decay` 1 000, `importance-recency-cap` all k 0 |
| AC-a-5 | PASS | C3: `test_home_gc.py`, `verify="every-commit"` and the test's own anti-joins |
| AC-a-6 | PASS | C3: both orders, equal final home dumps |
| AC-a-7 | PASS | C7: 50 kills (17/17/16) |
| AC-a-8 | PASS | C8: (a), (b) at every frame end, synthetic and real; views ≤ 20 MiB; overhead 40 B / 27 B against K = 93. See F-S28a-9 |
| AC-a-9 | PASS | C8: ratio 0.257 |
| AC-a-10 | PASS | C5: every golden point, schema 4 |
| AC-a-11 | PASS | C8 scans; C9 diff scan |
| AC-a-12 | PASS on macOS | C2: refusal with bytes unchanged; `HomeBusy` across processes — Linux and Windows in CI |
| AC-a-13 | PASS | C5: zero `CassetteMiss`, sections equal the golden, digest re-pinned |
| AC-a-14 | PASS locally | C5: ported suites green; real-binary files `test_ac10` +9 %, `test_retention_real` +24 % against the base locally; Linux CI is the measure (R-a-1) |

### Mutations

| Id | Planted | Expected red | Observed | Restored |
| --- | --- | --- | --- | --- |
| M-a-2 | `world.collect` deletes without `NOT EXISTS (… access …)` | `test_the_last_holder_collects` | **red**: `test_the_last_holder_collects`, `test_after_every_commit_the_records_are_exactly_the_held_keys`, `test_a_seat_that_leaves_takes_only_what_no_one_else_holds` (raised by `verify` before the test's own assertions) | byte copy, `cmp` clean, 4 passed |
| M-a-3 | `hold` inserts the access row first and calls `insert_or_verify` only when no record exists | `test_a_peers_record_is_verified_not_trusted` | **red**: that test | byte copy, `cmp` clean, 4 passed |
| M-a-4 | `hold` adds the `l0` line but not the `l0_who` lines to the meter | `test_the_meter_is_the_dump` | **red**: that test | byte copy, `cmp` clean, 7 passed |
| M-a-5 | `retrieve._recent`'s L0 query with `? IS NOT NULL` in place of `a.seat = ?` | `test_home_golden.py::test_sections_name_only_held_facts` | **red**: that test | byte copy, `cmp` clean, 1 passed |
| M-a-6 | `SeatView.release` deletes the access rows but does not subtract them from the meter | a size test or `test_the_meter_is_the_dump` | **red**: `test_home_store.py::test_the_meter_is_the_dump`; `test_retention_size.py::test_the_incremental_measure_is_the_dump_without_meta[consolidate]` and `[drop-everything]` | byte copy, `cmp` clean, 11 and 2 passed |
| M-a-7 | `SeatView.release` deletes access rows with `? IS NOT NULL` in place of `seat = ?` | `test_home_gc.py::test_a_pass_touches_only_its_seat` | **red**: that test, and the three other multi-seat GC tests | byte copy, `cmp` clean, 5 passed |
| M-a-8 | the view's `l0_who` keeps the seat's own row (`w.entity <> (? \|\| '#')`) | red by name in C6's harness | **red**: all 6 of `test_home_transparency_frames.py` (the meter, which `hold` keeps without the seat's row, no longer matches the rows `row_bytes` reads at the first pass); also `test_home_store.py::test_the_view_shows_its_held_facts_…` and `::test_the_meter_is_the_dump`, and `test_home_golden.py`'s three golden tests (episode texts name the seat). Note: the harness alone catches it through the meter, not through a `none`/`home` difference — the mutation is symmetric between the arms | byte copy, `cmp` clean; 6, 11 and 5 passed |
| M-a-9 | `MemoryHome.dump()` orders seats by `seats.seat` instead of `key` | the interleaving test | **red**: `test_home_isolation.py::test_the_home_dump_is_the_same_for_every_interleaving_and_frame` | byte copy, `cmp` clean, 1 passed |
| M-a-11 | an `INSERT INTO access` string in `compress/episodes.py` | the writers scan | **red**: `test_memory_isolation.py::test_only_the_home_writes_the_shared_records_and_the_access_index` | byte copy, `cmp` clean, 5 passed |
| M-a-10 | the record committed in its own transaction before the access row (`COMMIT; BEGIN IMMEDIATE` at the end of `world.insert_or_verify`) | AC-a-7 | **red**: every `record` case of `test_home_kill.py` (the killed file holds a committed record no seat holds) | byte copy, `cmp` clean, 17 passed |
| M-a-1 | `_lock` without `BEGIN EXCLUSIVE; COMMIT`, `timeout=5` | `test_a_held_home_is_busy` | First run: **survived** (the held home of the test had just been created, and its creating write takes SQLite's exclusive lock in exclusive locking mode; the original open also wrote through `PRAGMA auto_vacuum`, F-S28a-10). Test strengthened to hold a reopened home too; after the F-S28a-10 fix: **red**, `test_a_held_home_is_busy` (5.56 s: the child opened) | `cp` from byte copy, `cmp` clean, 8 passed |

### Deviations and findings

F-S28a-1 … F-S28a-8 (§3.2), recorded at design time.

**D-a-1 (bounded; R-P6b-a-1).** Previous assumption: P6b (#207) open, adapted by whichever merges second.
Audit evidence: #207 merged at `574e5eeb` before this PR's base. Corrected understanding: this PR is second, so it
adapts `controller/` (R-P6b-a-1). Implementation consequence: `SeatRunner` opens a `MemoryHome` and its one
`SeatView`; `detect`/`memory_view` take a `SeatView`; `_stable_people` gains the seat predicate. The home stays
at the seat's existing path `<store_dir>/<seat>.sqlite`, one seat per home (the `none` arm), because each
`SeatRunner` opens its own home and two runners of one process opening `store/<instance>/memory.sqlite` would
meet `HomeBusy`; one home shared by a process's runners is S28-f's `sharing` wiring (§2.2). Validation
consequence: `test_controller_parts.py`, `test_controller_seat.py`, `test_assignment.py` join the port's test
surface (C4/C5).

**D-a-2 (bounded).** The design's §9.0 and §12 name the worktree `impl-s28-a`; the dispatch assigned
`/Users/yuema137/mineworld-worktrees/impl-s28a`. Every command runs from the dispatch's path.

**D-a-3 (procedural slip, recorded).** At C1 one import line of `tests/home_streams.py` (a file this PR creates)
was edited with `sed -i` against the contract's NEVER list. The edit was the intended one (adding `JsonValue` to
an import) and the file was re-read; every later edit uses the editor or `ruff format`.

**D-a-4 (bounded; C1 step 3, C4 step 3, C5 step 2).** The golden has three arms: `ingest` (plain ingest in day
frames — the "plain-ingest points" C4 reproduces before retention is ported), `absorb` (C1 step 3's pass points)
and `fgc` (the FG-c cassette's stream, C5 step 2's capture, added to C1's script as C5 asks). Each point keeps the
sha256 of its 200 renders. The computation lives in `capture_golden_renders.py` and `test_home_golden.py` imports
it, so the script and the test cannot drift; the JSON is the oracle. `cafe12` (the real 12-seat builder) lives in
`home_streams.py` beside the synthetic world, so the script and `conftest.social_cafe_12` share it. A pass point is
taken after each day frame that ran a pass (each ran exactly one; `passes` is recorded).

**D-a-5 (bounded; §5.4, the grammar this PR fixes).** Two details of the home dump differ from §5.4's letter, both
to keep it the smallest faithful form: (1) a `fact` line writes the id once, `[fact, event_id, at, kind, place,
digest]` — the DDL's `event_key` and `event_id` encode to the same number; (2) `fact_who` is one line per record,
`[fact_who, event_id, role₀, entity₀, role₁, entity₁, …]` in ordinal order, not one line per row. With one line
per row each counterpart would cost the home ≥ 6 bytes more than its `l0_who` line in the view (tag, ordinal,
role), so the per-record overhead of I-a-6 (b) would grow with the participant count and no constant `K` could
bound it; grouped, each counterpart costs the home less than the view, and `K` is a constant for every record
with at most one entity the view does not show (F-S28a-9). Per seat the home writes `meta`, `names`, `access` (in
`l0`'s place), then the private tables (§5.3's order). The concrete grammar is the dated note under `ARC-101`.

**D-a-6 (bounded).** `MemoryHome.seat(identity)` and `view_of(key)` return a `SeatView` and arrive with `view.py`
at C3 instead of C2.

**D-a-7 (bounded).** `test_memory_store.py::test_no_id_is_ever_converted_to_a_number` (AP4-2 (c)) scans `memory/`
for `int(`; the canonical encoding's `int(value)` (`memory/home/encoding.py`, `ARC-101`: ids are numbers in the
dump) is added to its allowed set with that reason, at C2, where the code appears (the file is otherwise ported
at C3).

**D-a-8 (bounded).** C2 committed `tests/test_home_open.py` with one line `ruff format` would rewrap (a trailing
comment); C3 moves the comment above the line. CI's format check would have caught it; no behaviour changed.

**D-a-9 (bounded; C4/C5 boundary).** Deleting `store.py` at C4 (as §9 orders) leaves the retention host, which
C5 ports, without its import. C4 renames the import and annotations only (`MemoryStore` → `SeatView`) in
`memory/retention/{absorb, bound, forgotten, host, items}.py`, so the package imports and `src/` type-checks at
C4; their SQL and signatures are C5's. At the C4 commit the retention tests and the golden's retention arms
fail (recorded above), as §9 C4 step 3 anticipates.

**D-a-10 (bounded).** `test_memory_store.py` is deleted at C4 as §9 says; its tests that need ingestion
(re-delivery, order, the no-number scan, atomicity) move into `test_home_store.py` with the API rename, the
others were ported at C3. `capture_golden_renders.py` builds its FG-c replay gateway itself instead of importing
`test_lm_summarise.py` (a C5 file), and lost its writer and `--cafe` with the schema-3 store (`SOURCE.md` says
so); the schema-3 baseline stays in the JSON.

**D-a-11 (bounded).** The AP4-2 (c) scan's allowed set (D-a-7) loses `int(involved)`: `involved` is derived and
no longer stored (F-S28a-7).

**D-a-12 (bounded; C5 step 2, R-a-7).** The positional seat view is about a third of schema 3's object dump
(Alice on social-cafe: ≈ 195 KB of content a day against ≈ 433 KB), so two fixtures no longer reached the bound
they test, and were lengthened, the bound unchanged (FGb2-1): `test_retention_size.BUSY` keeps 2 800 facts a day
and raises Alice's crowd from 100 to 300 (≈ 9 KB of positional dump a fact, ≈ 25 MB a day; with 100 the busy day
was 9.47 MB, under the 20 MiB bound); `test_retention_real`'s keep-all window moves from days 30…40 to 75…85,
where keep-all now first reaches the 15 MiB target (day 81 measured). Consequence: `test_retention_size.py` takes
273.8 s locally against 97.1 s on the base code, almost all of it in the busier crowd's per-participant work
(`stable` upserts and their meter triggers, `fact_who` rows; the profile of the busy day: 14.6 s, of which
`executemany` 3.8 s self, the meter's `_put` 6.3 s cumulative before the codec fix). It is not a real-binary file
and has no per-file budget of its own in §12; the python CI job ran 12–16 min of its 45 on `main`. Recorded for
R-a-1; not a stop.

**D-a-13 (bounded; R-a-2).** The harness is four files, not one: as one file it ran ≈ 280 s locally, over
§12's 120 s per harness file (its cost is the 200 renders per seat and point, ≈ 90 % of a run: the I-S28-5 runs
without sections take 0.5 s each). `test_home_transparency.py` (frames 256, 4 096), `test_home_transparency_frames.py`
(frames 1, 7), `test_home_isolation.py` (AC-a-4 and the home dump across interleavings, I-S28-5, which §9 C6 asks
of M-a-9's "interleaving test"), `test_home_transparency_real.py` (the real stream, `real_binary`). Shared
machinery is `tests/home_harness.py`, a test-only module beside `home_streams.py`.

**D-a-14 (bounded; CI repair after the first run, R-a-1, R-a-2).** CI run 38819918810 on head `0ffe2f69`: `fast`,
`test`, `test-macos`, `test-windows`, `platforms` ×2, `python (ubuntu-24.04)` (31 min), `python (macos-15)` (29 min)
PASS; **`python (windows-2025)` cancelled at the job's 45-minute limit** inside `test_retention_size.py` — this
PR's cost: on Linux that file took 524 s (D-a-12's crowd of 300) and `test_home_transparency.py` 139 s, over §12's
120 s per harness file. Repairs, no criterion changed: (1) the meter's `UPDATE` trigger calls `put` once instead of
`gone` then `put` (an update keeps the rowid, and `put` replaces the row's size); (2) `SeatView.hold` computes each
`l0_who` line's length from the grammar (`EventId` and `EntityId` are validated ASCII digits, so the line is
`len('["l0_who",') + id digits + entity digits + 5`) instead of encoding hundreds of lines per busy fact — still
held to the dump by `test_the_meter_is_the_dump` (M-a-4 re-run: red) and by every incremental-measure test;
(3) the busy crowd's ids have u64's 20 digits and the crowd is 220, not 300 (≈ 24 MB a day, the busy day still over
the bound); (4) frames of 4 096 move to `test_home_transparency_large.py`. Locally: busy-day profile 14.6 s → 7.8 s;
`test_retention_size.py` 15 passed in 157.8 s (was 273.8 s; base code 97.1 s); `test_home_transparency.py` 3
passed in 29.7 s, `test_home_transparency_large.py` 3 passed in 30.5 s; `test_home_golden`, `test_home_store`,
`test_home_gc`, `test_home_open`, `test_home_kill` (50), `test_home_isolation`, `test_memory_isolation`,
`test_retention_store`, `test_home_scale -k synthetic` PASS.

**D-a-15 (primary ruling 2026-10-16, option 1; CI run 2 on `a94699f5`).** `python (windows-2025)` was cancelled at
the 45-minute limit again (`test_retention_size` ≥ 306 s, `test_home_kill` 177 s, `_frames` 132 s on Windows;
Linux python ≈ 10 → 31 min). The independent review of `a94699f5` found no design defect and caught all five of its
own mutations. Ruling applied, the 45-minute limit untouched:
- a `slow` marker (`pyproject.toml`); the member's `addopts` deselect it (`-m "not live_model and not slow"`),
  `python-smoke`'s expression adds `and not slow`; `test_structural_isolation.py`'s two pins follow;
- the nightly `stability` layer (Linux, macOS, Windows, 90-minute jobs) runs `uv run --locked pytest
  cognition/lm-controller -m slow` (`scripts/ci_layer.py` only; `nightly.yml` unchanged, so nothing overlaps
  `fix/nightly-red`, whose diff touches `nightly.yml`, `ci_godot.py` and a Rust test); the set is 52 cases: 41 of
  the 50 kills, frames 1 (3) and 4 096 (3) of the harness, and the busy-store `size_passes[keep-all]`,
  crossing, crash and `frames_never_change` ×2;
- per PR: 9 kills (3 per seam), transparency frames 7 and 256 (and the real stream), the busy store's overflow,
  `size_passes[consolidate]` (a plain policy) and the assisted size pass. Locally `test_retention_size.py` 10
  passed, 5 deselected in 30.9 s; `test_home_kill.py` 9 in 1.6 s (`-m slow`: 41 in 7.2 s);
  `test_home_transparency_frames.py` 3 in 33.3 s; `test_home_isolation.py` 5 in 14.2 s.
- Review items: `SeatView.hold` asserts the decimal-id precondition of the D-a-14 shortcut (`view._decimal`);
  `test_home_isolation.py::test_a_seat_view_shows_none_of_a_home_mates_rows` fails a seat leak by name (planted:
  `NameBook.load` without its seat predicate → red; restored, `cmp` clean, green).

**CI run 3 (39008602778, head `eca5e568`).** `changes`, `fast` (1 m 50 s), `platforms (windows-2025)` (13 m),
`platforms (macos-26)` (10 m), `python (ubuntu-24.04)` (23 m 10 s; the cognition member 20 m 11 s against 9 m 25 s
on `main` run 38704321403), `python (macos-15)` (21 m 37 s), `test` (44 m), `test-windows` (42 m), `test-macos`
PASS. `python (windows-2025)` (28 m 30 s, under the 45-minute cap): every cognition test PASS; FAIL in
`clients/agent/tests/test_idle.py::test_a_handle_without_calls_gives_its_seat_back` (`JoinRefused:
seat_occupied`) — a member this PR does not touch (`git diff --name-only origin/main...HEAD` lists nothing under
`clients/`), recorded as out of scope (test rules §28). AC-a-14 on Linux CI (per-file sums of the job's log against
`main`'s): `test_ac10.py` 88.2 s against 70.3 s (+25.5 %), `test_retention_real.py` 138.5 s against 94.2 s
(+47 %), `test_retention_policies.py` 212.3 s against 158.5 s (+34 %, not real-binary): over R-a-1's +25 % for the
two real-binary files; R-a-1 makes it a stop only if it breaks a real-binary budget (≤ 90 s per file, §12), which
`test_retention_real.py` already broke on `main` (94.2 s). Recorded; D-a-16 trims the keep-all window.

**D-a-16 (bounded).** `origin/main` merged again at `5e7393f3` (#224's sharded test jobs, `scripts/ci_shard.py`;
`docs/DECISIONS.md` rebuilt as before: main's records, S28-a's two notes, `ARC-109` and `DEP-61` last; 151 ids
distinct). `test_retention_real.py`'s keep-all window narrows from days 75…85 to 78…84 (six passes on a ≈ 15 MiB
store instead of eleven; the first forced pass is still day 81, measured).

**CI run 4 (39053780410, head `a363f100`).** All jobs PASS. `test (2/3)` failed once on a Rust wall-clock tick-budget
assertion (`tools/cli/tests/hosted_town.rs`; nothing under `tools/` changes here) and passed on its re-run.
`python (ubuntu-24.04)` 14 m 30 s (the cognition member 11 m 44 s: 463 passed, 52 `slow` deselected);
`python (windows-2025)` 33 m 16 s; `test-macos (b: core 3/3, python)` 34 m 04 s. AC-a-14 on Linux CI:
`test_retention_real.py` 59.6 s (`main` 94.2 s), `test_ac10.py` 46.5 s (70.3 s): within R-a-1 and §12's 90 s.

**CI run 5 (39073972039, head `5a0515ed`, after merging P6a #236 and S30-b #247).** All jobs PASS.
`python (windows-2025)` was cancelled once at 45 min, hung 26 min in
`test_controller_resilience.py::test_a_the_world_stays_valid_under_thirty_percent_garbage_and_five_percent_failure`
(a real-server test this PR does not change; `main`'s run on `83e011fe` was cancelled the same way, hung in
`test_controller_server.py`); its re-run on the same head passed in 41 m 09 s. `python (ubuntu-24.04)` 23 m 46 s;
`test-macos (b)` 34 m 01 s.

**D-a-17 (bounded; merge of #248 ARC-149 and #244 P7).** `docs/DECISIONS.md` rebuilt as before (153 ids distinct, 240
headings). P7 added `tests/test_milestone_d_100_days.py` and `tests/test_milestone_d_fallback.py`, which open the
deleted `MemoryStore`: both now open the seat's one-seat `MemoryHome` at the same path and its `SeatView` (D-a-1),
closing the home. A mechanical port; no assertion changed.

**MATERIAL STOP (§12: "a needed change outside cognition/lm-controller and docs/DECISIONS.md"), CI run 6
(39098788451, head `8dbb49d1`).** Every Python suite passes (cognition: 469 passed, 52 `slow` deselected, 23 m 00 s
on Linux). The `python` layer then fails at P7's (#244) Rust test, `cargo test -p mineworld-cli --test milestone_d
-- --ignored --exact she_remembers_across_the_two_views`: `tools/cli/tests/milestone_d.rs::store_rows` opens Alice's
memory file with rusqlite and reads `SELECT rowid, event_id FROM l0` → "no such table: l0" (schema 4 has
`access` ⋈ `fact`). The same failure on `test-macos (b: core 3/3, python)`. Proposed fix, prepared as one local
commit and not pushed (D-a-18): `store_rows` reads Alice's cursor from her seat's `meta` row and her held records as
`access ⋈ fact` for seat `alice`, ordered by `access.rowid`; test-only, no assertion changed; `rustfmt --check`
PASS. It is the R-P6b-a-1 situation (a consumer of the store merged after the design: "whichever merges second
adapts"), but in `tools/`, which the contract does not list.

**D-a-18 (authorized by the primary, 2026-10-18: "a bounded adaptation under R-P6b-a-1 … test-only, P7's
assertions unchanged, required by S28-a's own frozen schema 4").** `tools/cli/tests/milestone_d.rs::store_rows`
reads Alice's cursor from her seat's `meta` row and her held records as `access ⋈ fact` for seat `alice`, ordered by
`access.rowid`. **Follow-up FU-S28a-1 (not done here):** P7's "resumed, not rebuilt" check compares (rowid, event
id) across the restart, and a full replay into a new file would recreate the same rows: schema 4 is deterministic by
design (I-S28-5), and its `home_meta` holds only `schema` and `world_instance`, so nothing in the file tells a
resumed home from a rebuilt one. A creation marker (a `home_meta` row written once at creation, never dumped) would
be a schema change, beyond a small test change; the primary's instruction defers it.

**CI run 7 (39103453267, head `31d46cff`, D-a-18 pushed).** `python (ubuntu-24.04)` (26 m 32 s), both macOS shards
(`a` 29 m 14 s; `b`, which holds python, 33 m 06 s), `test` ×3, `test-windows` ×4, `platforms`, `removability`
PASS: D-a-18 cleared the `milestone_d` failure. `python (windows-2025)` failed once in
`test_controller_server.py::test_b_a_failing_script_hands_the_seat_back_and_the_runner_rejoins` (the P6b rejoin flake,
fixed on `main` by #256) and its re-run was cancelled when the primary updated the branch (`46f88c8b`, merging #256
and #206).

**D-a-19 (bounded).** #206 (S26-a) added `test_memory_render.py::…` opening the deleted `MemoryStore`; ported to
`memory_fixtures.seat_view()` (its `_rows` already reads positional lines). No assertion changed; the file's 7 tests
PASS; pyright strict 0 errors.

**D-a-3 (continued).** At C5 one timing command used `uv run python -c ""` (an empty program, to warm the
environment), again against the NEVER list. Nothing was executed or written by it.

**F-S28a-9 (finding for the operator; I-a-6 (b), QS28a-1 × QS28a-3).** I-a-6 (b) bounds a one-seat home's dump by
its seat view's plus `K` per held record. The home also stores every `fact_who` row the view does not show: the
seat's own row and, since QS28a-3, every subject that is not a participant (role 2). With D-a-5's grammar a
counterpart costs the home less than the view, so `K` (C8) covers any record with at most one hidden entity of the
longest id; a record whose envelope names **two or more** subjects that are not participants costs `5 + |entity|`
bytes per further hidden entity beyond `K`. Social-cafe has such facts: `passage-opened` (5 facts in 2 days, no
participant, two place subjects, held by all 12 seats). Their ids are short, so AC-a-8 (b) holds on the AC's
data with a margin (C8 records the largest measured overhead), but no constant `K` bounds every possible envelope.
Proposed amendment, for a ruling: (b) reads "… + `K` × (records it holds) + Σ over hidden `fact_who` entries
beyond the first per record of (5 + |entity|)", which keeps (b) exact and still bounds disk by the envelopes the
seat perceived. Not changed here: (b) is tested as frozen.

**F-S28a-10 (bounded, fixed at C2; I-a-10).** `PRAGMA auto_vacuum = INCREMENTAL` on a database that already has
pages and is already incremental rewrites its header (the change counter; probe: file sha256 `63e093c0…` →
`4cfdc486…`) and, in exclusive locking mode, takes and keeps the write lock. P4 created its stores incremental,
so opening a real schema-3 store would have modified the file it then refused. The home sets `auto_vacuum` only
when `PRAGMA page_count` is 0. The refusal fixture now creates its store incremental, as P4 did; with the old open
the two refusal tests fail on the sha256 (verified), with the fix they pass.
