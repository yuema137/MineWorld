# PR FU-S28a-1 — The memory home's creation marker: a `home_meta` row written once, never dumped

## DESIGN FROZEN — operator ruling FU-S28a-1, 2026-10-18

```text
Design revision:        revision 1 (2026-10-18), FROZEN 2026-10-18
Approved by / evidence: the operator, in the primary session, 2026-10-18: "do it" on FU-S28a-1 (S28-a ledger,
                        pr-s28a-memory-home.md, D-a-18 paragraph). The ruling approves the change to S28-a's
                        frozen schema 4; the details below are the implementer's, as the ruling delegates
Implementation base:    origin/main @ 96b41db0 (S28-a #242 and P7 #244 merged)
Branch / worktree:      fix/memory-home-marker, /Users/yuema137/mineworld-worktrees/memory-home-marker
Decision id:            ARC-152 (the primary's block starts at ARC-152; checked free with
                        scripts/check_decision_ids.py)
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0` · **Revises:** [`pr-s28a-memory-home.md`](pr-s28a-memory-home.md) §§5.2, 5.4, 5.5 (schema 4's
`home_meta`, `MemoryHome.dump()`, `MemoryHome.open`) and the test of
[`pr-s10-p7-milestone-d.md`](pr-s10-p7-milestone-d.md) AP7-2 · **Binding parents:** `CLAUDE.md` §§2, 3, 4
(items 1, 8, 9, 10, 12); `ARC-101`, `ARC-109`; step-28 I-S28-5.

## 1. Problem

P7's Milestone D test (`tools/cli/tests/milestone_d.rs`) asserts AP7-2 — after a SIGKILL and a restart "Alice's
store file is the same store" — by comparing Alice's cursor and her held records as (`access.rowid`,
`fact.event_id`) before and after. A full replay of the perceived stream into a new file recreates exactly those
rows and that cursor: schema 4 is deterministic by design (I-S28-5), and `home_meta` holds only `schema` and
`world_instance`, both functions of the world. Nothing in the file tells a resumed home from a rebuilt one, so the
check cannot fail on the case it names. Today only the separate "Alice said exactly two lines" assertion catches a
wiped store, and only indirectly.

## 2. Change

### 2.1 Schema: a compatible `home_meta` row, schema stays 4

`home_meta` gains the row `created`, whose value is 32 lowercase hexadecimal characters: 128 bits from the
operating system's CSPRNG (`secrets.token_hex(16)`), drawn once, in the transaction that creates the home.

**Why schema 4, not 5.** S28-a's versioning rule (`schema.SCHEMA_VERSION`; QS28-8; `ARC-109`) is that a version
names a layout, and a store of another layout is refused and rebuilt from the perceived stream. This change adds
no table, no column and no constraint: `home_meta` is a key–value table, and every reader of schema 4 reads its
rows by key. A seat view's dump and the home's dump are unchanged byte for byte (§2.3), so every pinned digest
(`test_lm_summarise_replay.py`), golden render and AC of S28-a holds without a re-pin. Bumping to 5 would refuse
every home written today and re-pin the replay digest for a row no dump contains: version churn with no layout
behind it.

**Existing homes (the migration).** Schema-4 homes without the row exist only since #242 merged (2026-10-18). At
open, a schema-4 home whose identity checks pass and which has no `created` row receives one, in one transaction,
before `open` returns. This is the row's one write for that file; from then on it is never written again. No
other row is touched, and a refused file (schema 2 or 3, another world instance, not a home) is still never
modified (I-a-10): the check runs first.

### 2.2 The value is never rewritten

Only `_create` and the one-time fill of §2.1 insert the row; nothing updates or deletes it. `MemoryHome.created`
(a read-only `str` attribute) reports it, for tests and for S28-e's inspect.

### 2.3 Determinism: the marker reaches no dump, no digest and no world state

- `MemoryHome.dump()` writes the `home_meta` rows named by `schema.DUMPED_HOME_META = ("schema",
  "world_instance")`, in key order, and never `created`. I-S28-5 (the dump is a function of each seat's perceived
  prefix and configuration) therefore still holds: two homes fed the same seats dump identically although their
  markers differ.
- `SeatView.dump()` reads no `home_meta` row by query; its identity lines are `world_instance` and `schema`, as
  before (`ARC-101`). Nothing the seat renders, decides, sends or logs reads the marker.
- The home lives in cognition's own directory, never in a world's save (`ARC-98`); the marker never enters world
  state, the event log, a snapshot or any kernel digest.
- The nonce is drawn from `secrets`, not from any seeded stream, so no seeded draw in cognition shifts.

### 2.4 The test change (P7, AP7-2)

`store_rows` also reads `home_meta.created`. The test asserts that the marker after the restart equals the marker
before it ("the same home, not a new one"), and keeps every existing assertion: the cursor did not go back, every
pre-kill row is the same row after, and Alice said exactly two lines.

Python (`test_home_open.py`): a new home has a 32-hex marker; reopening keeps it; two homes fed identically have
different markers and identical dumps, and neither dump names `created`; a schema-4 home without the row receives
one at its first open, and keeps it at the next.

## 3. Acceptance

| Id | Criterion | Evidence |
| --- | --- | --- |
| AC-m-1 | A created home has a `created` row; reopening keeps it unchanged | `test_home_open.py` |
| AC-m-2 | `MemoryHome.dump()` and `SeatView.dump()` never contain the marker; S28-a's pinned digests and golden renders unchanged | `test_home_open.py`; the unchanged pins pass in CI |
| AC-m-3 | An unmarked schema-4 home is marked once at open; refused stores are still byte-identical | `test_home_open.py` (the existing refusal tests unchanged) |
| AC-m-4 | Milestone D compares the marker across the restart, existing assertions kept | `milestone_d.rs` PASS in CI's `python` layer |
| AC-m-5 | **Mutation:** wiping Alice's home before the restart fails the marker assertion by itself, with "two lines" disabled on a scratch copy | ledger §5 |

## 4. Commit plan

| # | Commit | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C1 | design (this file) | [x] | N/A (document) | [x] checked against S28-a §5.5 and `ARC-101` |
| C2 | `memory.home`: the marker; tests; P7's check; specs, `ARC-152` | [ ] | [ ] | [ ] |

## 5. Ledger

(Evidence, deviations and the mutation are recorded here as the work proceeds.)
