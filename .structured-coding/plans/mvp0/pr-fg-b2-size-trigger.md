# PR FG-b2 — Per-person memory bound 20 MiB (configurable upward) and a size-triggered retention pass

## DESIGN FROZEN 2026-10-14 (primary; operator ruling '快满了就提前整理')

```text
Design revision:        revision 1 (2026-10-14), written by the FG-b2 designer-implementer session and
                        frozen by the primary session under the operator's two rulings of 2026-10-14 (§1.1)
Approved by / evidence: operator rulings of 2026-10-14, verbatim in §1.1, relayed by the coordinator;
                        the freeze line above is the primary session's, as briefed
Implementation base:    origin/main @ f772a18 (#197)
Execution contract:     §11
Kind:                   a bounded revision of FG-b (pr-fg-b-retention-host.md) §5.6, §5.7, §6 and §9
                        AFGb-4 under the operator's rulings; no new decision id (ARC-87 gains a dated note)
Lifecycle:              FROZEN. Progress, evidence and bounded corrections stay writable (§12).
```

**Effort:** `mvp0`. **Step:** [`../mvp1/step-24-decision-and-memory.md`](../mvp1/step-24-decision-and-memory.md)
(§10.1's per-seat bound). **Parent design revised:** [`pr-fg-b-retention-host.md`](pr-fg-b-retention-host.md)
(merged). **Sibling lanes touched:** FG-c (PR #201, open; `retention/host.py`, `selection.py`), P6b (PR #207,
open; reads `mineworld decisions --json`), DL-a (merged; owns `decisions.yaml`'s reader).

---

## 1. Why

### 1.1 The rulings (operator, 2026-10-14, verbatim)

| Id | Ruling | Meaning taken here |
| --- | --- | --- |
| **R-FGb2-1** | "改成'快满了就提前整理'" | Keep the promise; change when the pass runs: when the store nears its bound, run a pass at once instead of waiting for the next simulated day. |
| **R-FGb2-2** | "一个人物1mb太少了吧，我觉得起码20mb", then "默认 20 MB，世界可以调大" | The per-person bound defaults to 20 MiB; a world may raise it, never lower it. |

R-FGb2-2 **reverses** step-24 §1.2 ruling 2 (4) and FG-b §5.7's sentence "Neither number is a parameter: a
world or a policy cannot loosen them" in one direction only: a world may raise the bound; nothing may set it
below the 20 MiB floor, and no policy can change it. The 6 000-byte per-call memory ceiling (D-P4-9) is not
touched by either ruling.

### 1.2 The finding behind R-FGb2-1

PR #171 (12d, "towns get walls") gives social-cafe bodies and walking; Alice then perceives far more per
day. Her store reached 1 496 077 B just before the day-2 pass against FG-b's `STORE_BOUND` of 1 048 576 B
(`test_retention_real.py:87`, AFGb-4 (b)). FG-b ran one pass per simulated day and assumed one day of
perception fits in the 256 KiB between `TARGET` and the bound (FG-b QFGb-2, risk R-FGb-2, which named
"intra-day passes, a ruling" as the remedy). R-FGb2-1 is that ruling.

With R-FGb2-2 alone, 12d's 1.5 MB per day fits the new 5 MiB daily headroom with three-fold margin; the size
trigger is what makes the promise hold for **any** world, whatever its perception rate.

---

## 2. Scope

### 2.1 In scope

- `memory/retention/bound.py`: `StoreBounds` (the typed, validated bound and the figures derived from it),
  `StoreBoundOutOfRange`; the backstop takes the bounds instead of module constants.
- `memory/retention/selection.py`: `MemoryPolicyChoice.store_bound_mib`; `Retention.bounds`; `resolve`
  validates the bound and refuses by name.
- `memory/retention/absorb.py`: the size trigger, its interaction with the daily pass, its resume rule.
- `memory/retention/host.py`: a pass's `cause` (`day` | `size`); the pass log keyed by trigger.
- `memory/ingest.py`: an optional per-fact stop predicate used only by `absorb` (§4.3).
- `memory/store.py`: schema v3 (the pass log's key, §4.5); `content_size()`, an exact incremental measure of
  the canonical dump without `meta` (§4.2).
- `memory/records.py`: `SCHEMA_VERSION = 3`.
- Rust, `cognition/decision` and `tools/cli`: `store_bound_mib` accepted in `memory:` entries, passed
  through; the JSON form's `format` becomes 2 (§5).
- Tests (§8), the pinned numbers of FG-b's tests re-derived (§7), and the docs: `MODULE_SPEC.md` §4.1
  rule 8, §5.2 rules 7 and 9 and its Selection paragraph, §8.1's JSON form; `ARCHITECTURE.md` §9.3;
  `DECISIONS.md` ARC-87 (a dated note); `cognition/lm-controller/README.md`; FG-b's ledger note.

### 2.2 Not in scope

| Item | Where |
| --- | --- |
| FG-c's assisted branch in a size pass | FG-c (R-FGc-FGb2-1, §6), whichever of FG-b2 / FG-c merges second |
| Closing the open episode at a size pass | not done (§4.7); a later ruling if a world needs it |
| `step-24` §10.1 and `overall.md` | the primary session, after merge |
| The save (`world.sqlite`) and its growth targets | unaffected: the memory store is the controller's, not the save's (§9) |

### 2.3 Invariants

| ID | Invariant |
| --- | --- |
| FGb-2' (replaces FGb-2) | After every pass the canonical dump is ≤ `target` (75 % of the configured bound); at every committed state reachable through `absorb` it is ≤ the configured `bound`, under the one condition of §4.6. |
| FGb2-1 | The bound is ≥ 20 MiB in every `Retention` that can be constructed; only a `memory:` entry raises it; no policy sees or changes it. |
| FGb2-2 | **Size trigger is a function of the facts.** Whether and where a size pass runs depends only on the perceived facts ingested and the store's state, never on frame boundaries or wall time. Frames of 1, of 256 and the whole stream give byte-identical dumps; a crash between a stop and its pass resumes to the same dump. |
| FGb2-3 | **At most one pass per fact.** At a fact where the daily pass is due, the daily pass runs and no size pass does. |
| FGb-1, FGb-3 … FGb-7, I-5, P4-2 | FG-b's, unchanged. |

---

## 3. Audit (`origin/main @ f772a18`)

| Anchor | Finding | Consequence |
| --- | --- | --- |
| `retention/bound.py` `TARGET`, `STORE_BOUND`, `LIMIT = TARGET − 512` | Module constants; `host.run_pass` builds the budget from `TARGET`; `enforce` loops to `LIMIT`. | Replaced by `StoreBounds` carried on `Retention`. |
| `retention/absorb.py` | Splits frames at the first fact of each new day (`fact.at // DAY_S > day`), then `run_pass(…, day × DAY_S)`. A pass due at entry (crash resume) runs first. | The size trigger adds a second, per-fact split point inside `ingest`. |
| `memory/ingest.py` | One transaction per call; `close_weeks` once at the end; `cursor` = the call's `through`, `newest` = the last written key. Every other write (`l0`, `l0_who`, `episodes`, `episode_who`, `stable`, `names`) is immediate. | Mid-call state differs from a split call only by chapters written at call end and by `meta`. §4.2–4.3 remove both differences from the trigger. |
| `memory/store.py` `size()` | Encodes every row of every dumped table: O(store). | Too slow per fact (≈ 20 MiB per check). §4.2 measures incrementally. |
| `memory/store.py` `retention_passes (day INTEGER PRIMARY KEY …)` | One pass per day by construction. | Several passes a day need another key: schema v3 (§4.5). |
| `compress/episodes.py` `_next_number` | The next episode's number is `max(episode) + 1` when none is open. | Why a size pass does not force-close the open episode (§4.7). |
| `cognition/decision/src/read.rs` `RawPolicy`, `RawMemoryAssign` (`deny_unknown_fields`) | A `memory:` entry takes `policy`, `fallback`, `parameters` only. | A world can set the bound only if the Rust reader accepts and passes it (§5). |
| `cognition/decision/src/json.rs` `JSON_FORMAT = 1`; `MODULE_SPEC.md` §8.1 "a change to its shape bumps it" | — | `format` 2. |
| `resolve.rs` `CanonicalMemory` (digest input) | Source positions excluded. | `store_bound_mib` enters the digest only when written, so existing digests are unchanged. |
| FG-c (#201) `host.py` | Adds `_decide`, `_assisted` (the model call happens inside the pass), `calls_per_pass`, and imports `TARGET`. | Expected textual conflict; R-FGc-FGb2-1 (§6). |

Measured on this base before design (FG-b §3.2, §15.2): Alice's 100-day real store under `consolidate` peaks
at 960 176 B before a pass and 333 231 B after; P4 alone (no retention) grows ≈ 433 KB per day; the file on
disk of an unretained store is 3.13 × its dump (40 714 240 B / 12 996 459 B).

---

## 4. Design

### 4.1 The bound and the figures derived from it

```python
MIB = 1_048_576
FLOOR_MIB = 20            # R-FGb2-2: the default and the minimum
CEILING_MIB = 1_024       # an upper sanity bound (1 GiB): `dump()` holds the whole store in memory

@dataclass(frozen=True)
class StoreBounds:
    mib: int                                  # refused outside FLOOR_MIB … CEILING_MIB (StoreBoundOutOfRange)
    bound   = mib × MIB                       # 20 971 520 B at the default
    target  = bound × 3 // 4                  # 15 728 640 B: the dump after every pass
    trigger = bound − bound // 16             # 19 660 800 B: a size pass runs when the content exceeds it
    limit   = target − LOG_RESERVE (512)      # what the backstop drives to, as FG-b D-8

DEFAULT_BOUNDS = StoreBounds(FLOOR_MIB)
```

- **`target` = 75 %**, the ratio FG-b chose (768 KiB of 1 MiB). It leaves a quarter of the bound — 5 MiB at
  the default — for a day of perception before the daily pass: 3.4 × 12d's measured 1.5 MB day and 11 × FG-b's
  433 KB day. A world reaching the size trigger every day is therefore an outlier the trigger handles, not
  the common path, and the daily pass keeps its role.
- **`trigger` = bound − bound / 16** (93.75 %). Two requirements fix it. The gap `trigger − target` (3.75 MiB
  at the default) bounds how often a burst can make the host pass (§4.8): the larger, the cheaper. The
  margin `bound − trigger` (1.25 MiB) must hold the growth of the one fact that crosses the trigger plus
  `meta`, because the store commits that fact before its pass (§4.6). One fact adds an `l0` row (≈ 300 B
  plus its gist, quoted to ≤ 100 characters), one `l0_who` row (≈ 70 B) and at most one `stable` row
  (≈ 180 B) per participant, an episode update and at most one `names` row per entity it names: 1.25 MiB is
  a fact naming ≈ 3 000 participants; `meta` is under 1 KiB. Both ratios scale with a raised bound.
- `StoreBounds` refuses construction outside the range, so FGb2-1 holds by construction: there is no way to
  build a `Retention` with a smaller bound, in tests included. FG-b's tests therefore re-derive their
  figures at 20 MiB scale (§7); none uses a private smaller bound.
- **Configurable upward only.** `MemoryPolicyChoice.store_bound_mib: int | None = None` (strict `int`; `None`
  means `FLOOR_MIB`). `resolve` builds `StoreBounds(store_bound_mib)` and turns a refusal into
  `StoreBoundOutOfRange(given, floor, ceiling, source)`, a `PolicyError`, naming the source span the Rust
  reader reported. It is a field of the **choice**, not of a policy's `Parameters`: it is the host's bound,
  identical in meaning under every policy, and a policy never sees it (FGb-3). The unit is a whole MiB,
  so a world writes `store_bound_mib: 40`, not a byte count.

### 4.2 Measuring the store per fact: `MemoryStore.content_size()`

The trigger is evaluated after every fact, so it needs the dump's size without encoding the dump.

- `content_size()` is the exact byte length of the canonical dump's lines **of every dumped table except
  `meta`**, each with its newline: `size() − meta's lines`. It equals that value at every call (a test
  checks it after every ingest call and pass of a stream, §8 AFGb2-6).
- It is maintained incrementally. On first use on a connection, the store registers two connection
  functions and creates `TEMP` triggers `AFTER INSERT / UPDATE / DELETE` on each metered table that call
  them with the row's `rowid` and, for a write, its new values; the store encodes that row with the dump's
  own encoder and keeps each row's line size by `(table, rowid)` and the running total. Then it measures
  every metered row once. A call is then O(1): the total. Once the meter is in use, `size()` and
  `row_bytes()` read its sizes instead of encoding again (D-FGb2-1).
- `TEMP` objects live in SQLite's per-connection temp schema: nothing enters the file, the schema or the
  dump. The sizes the triggers reported inside a transaction that rolls back are no longer true, so
  `MemoryStore.transaction` drops the cached sizes on rollback; the next call measures in full.
- Why `meta` is excluded: `meta.cursor` is each ingest call's `through` and `meta.newest` is written at the
  end of a call, so mid-call they depend on where frames began. Every other metered write happens per fact
  (with §4.3's `close_weeks`), so the measure after fact *k* is a function of the facts up to *k*. `meta`
  holds eight short rows; its bytes are inside the margin of §4.1.
- **Reuse.** SQLite has no update hook in Python's `sqlite3`; `TEMP` triggers are SQLite's own mechanism for
  observing row changes and need no dependency. Computing line lengths in SQL (`json_object`) was rejected:
  its escaping and key order are not the Python encoder's, so the measure would not be the dump's.

### 4.3 Where a size pass runs

```text
ingest(store, facts, through, *, stop_when: Callable[[], bool] | None = None) -> IngestReport
    after each fact it writes, when stop_when is given:
        if an episode closed with this fact: close_weeks(...)          (chapters per fact, not per call)
        if stop_when(): stop after this fact; cursor and newest become this fact's id; COMMIT
    IngestReport.stopped: EventId | None                               (the fact it stopped after)
```

- `absorb` passes `stop_when = lambda: store.content_size() > bounds.trigger` on every ingest call it makes.
  `ingest` without `stop_when` is unchanged in behaviour (FGb-7).
- Calling `close_weeks` after each closing fact makes chapters a per-fact function too: a week's chapter is
  written by the fact that closes the first episode of a later week, whatever the frame (FG-b's `ingest`
  wrote it at the end of the call, which is the same fact when frames end at day starts, as every FG-b
  test's do).
- When an ingest call stops at fact *k*, `absorb` runs a **size pass** with `now = facts[k].at` (world
  seconds of the fact, the moment the store crossed), then continues with the facts after *k*. A size pass
  is `run_pass(store, retention, now, cause="size")`: the same pass as the daily one (items, policy,
  validation, execution, backstop to `limit`, structural check, log), with `RetentionBudget.now = now`.
- **Interaction with the daily pass (FGb2-3).** Day split points are computed exactly as FG-b does. If an
  ingest call stops at the first fact of a new day — the fact at which the daily pass is due — `absorb` runs
  the daily pass there and no size pass. A size pass never writes `meta.retention_day`, so the daily pass of
  the next day is due exactly when it was in FG-b. Under `consolidate` defaults the daily pass leaves the
  store far under `target` (§3: 333 KB after, at most), so in the reference worlds size passes do not occur.
- **Resume.** On entry, `absorb` first runs a daily pass if one is due (FG-b); otherwise, if
  `content_size() > trigger`, a size pass with `now` = the newest fact's `at`. This is exactly the state an
  uninterrupted call is in after a stop and before its pass, so a crash there resumes to the same dump.
- **Frames do not matter.** Every split point — a day's first fact, a stop — is a fact, decided on a measure
  that is a function of the facts up to it (§4.2). A frame of any size, including one larger than the bound,
  is split by these points into the same ingest calls *in effect* and the same passes.

### 4.4 The trigger and the policies

A policy sees `RetentionBudget(store_bytes = target − overhead, now, calls_left)`. A size pass's `now` is the
crossing fact's time, so recency-based policies age items against the moment of the pass, as a daily pass
ages them against the start of its day. `consolidate` with `l0_hours: 0` then merges every record of a
closed episode older than that second; `keep-all` relies on the backstop, which brings it to `limit`.

### 4.5 Schema v3 and the pass log

```text
retention_passes (trigger TEXT PRIMARY KEY,          the event key of the newest fact at the pass
                  day INTEGER NOT NULL,              now // 86 400
                  cause TEXT NOT NULL CHECK (cause IN ('day', 'size')),
                  policy, outcome, kept, merged, dropped, forced,
                  bytes_before, bytes_after, forgotten_ranges)       as FG-b
                  newest 32 rows by trigger
```

- One pass per fact (FGb2-3), so `trigger` is unique. `PassReport` gains `cause` (default `"day"`, after its
  existing fields, so FG-c's added fields merge beside it).
- `SCHEMA_VERSION = 3`. A v2 store is refused by name (`StoreIdentityMismatch(field="schema")`) and rebuilt
  from a null cursor, as FG-b QFGb-7 ruled for v1; no v2 store exists outside test runs (P6 is not merged).

### 4.6 The guarantee, exactly

- **After every pass**, `len(dump()) ≤ target` (the backstop drives to `limit = target − 512`, D-8).
- **Every committed state** that `absorb` leaves is one of: the end of an ingest call that did not stop
  (every fact's check passed: content ≤ `trigger`, so the dump ≤ `trigger + meta` < `bound`); the end of a
  pass (≤ `target`); or the commit of a stopped ingest call, which holds the content of one fact past
  `trigger`, followed at once by its pass. The last is ≤ `bound` whenever the crossing fact grew the store by
  less than `bound − trigger − meta` (1.25 MiB less `meta` at the default, §4.1).
- **A single fact larger than that margin** is committed over the bound, and its pass, which runs
  immediately, brings the store to `target`; `bytes_before` in the pass log records the excess. Nothing is
  lost and no fact is refused (memory is a derivation, FG-b I-5). A crash in that window leaves the over-bound
  store on disk until the resume's first action, the pass.
- **A single frame larger than the bound** is not a case: frames are split at facts (§4.3).
- **When a pass cannot reach `target`** — the open episode alone (never shown to a pass, §4.7) and the
  unremovable tables exceed it — the pass raises `RetentionBoundUnreachable` and rolls back, as FG-b §5.7.3.
  The stopped ingest is committed below `bound`; the next `absorb` retries the pass first and raises again, so
  the store never grows past that point: the seat's controller stops with a named error, never silently
  over. At the default this needs one open episode of ≈ 15 MiB (≈ 10 × 12d's whole day in one place without a
  30-minute gap); a world that legitimately produces it raises its bound.

### 4.7 Why a size pass does not close the open episode

A pass never shows the open episode (FG-b §5.7.0). Closing it at a size pass would let the pass shed it, but
`Episodes._next_number` numbers the next episode `max(episode) + 1`: a pass that dropped the just-closed newest
episode would hand its number to the next one, and chapter ranges (runs of consecutive episode numbers, FG-b
§5.5) would then span the forgotten episode — a coverage breach (`check_structure` raises). Fixing that needs a
persistent episode counter in P4's ingestion, a wider change than the ruling requires. §4.6 states the cost of
not doing it; it is a recorded limitation (R-FGb2-2).

### 4.8 Cost

- **Per fact:** one encode per row the fact writes (an `l0` row, one `l0_who` and one `stable` row per
  participant, the open episode, rarely `names`, `episode_who`, `chapters`), done by the triggers; the
  check itself reads a counter. Measured in §12.
- **Per size pass:** the pass's cost is FG-b's, O(store) at ≤ `bound`.
- **Per burst:** a burst of perception adding `G` bytes within one simulated day causes at most
  `⌈G / (trigger − target)⌉` size passes (each pass leaves ≤ `target`; the next needs `trigger − target` of
  growth): ⌈G / 3.75 MiB⌉ at the default. 12d's whole day (1.5 MB) causes none.
- **First use per connection:** one full measure, O(store), the cost of one `size()`.

### 4.9 FG-c's assisted passes

FG-c's assisted policy asks its gateway inside a pass, `calls_per_pass` (capped) calls each. Size passes would
multiply a day's calls by the number of size passes. **R-FGc-FGb2-1:** a size pass never calls a model: for an
assisted `Retention`, a size pass decides with the fallback policy alone (FG-c's `_plain(retention.fallback,
…)` route) and records `fallback:size`, so the calls per simulated day and the budget remain FG-c's daily
figures. The rule is implemented by whichever of FG-c and FG-b2 merges second, using `PassReport.cause` /
`run_pass(…, cause=…)` that FG-b2 adds. A summary call's 6 000-byte request ceiling is untouched.
**Implemented here** (FG-c merged first, #201): `host._decide` routes a size pass of an assisted
`Retention` to FG-c's `_fallback_plan`, records `fallback:size` (or the fallback's own refusal) and
`fallback_decisions`; `test_a_size_pass_of_an_assisted_seat_asks_no_model` (§12.3 M-FGb2-9).

---

## 5. Configuration: `decisions.yaml` `memory:` and its JSON form

```yaml
memory:
  default: { policy: consolidate, store_bound_mib: 40 }      # every seat: 40 MiB
  assign:
    - { seat: alice, policy: decay, store_bound_mib: 64 }
```

- The Rust reader accepts an optional `store_bound_mib` (a non-negative integer, else refused with line and
  column as any shape error) in `memory.default` and each `memory.assign` entry, resolves it with its entry
  (seat > class > default — the bound travels with the winning entry, it is not merged across levels) and
  passes it through **without judging it**: the floor and ceiling are the cognition host's, checked by
  `resolve` with the entry's source span, as policy ids and parameters are (QFGb-1).
- `mineworld decisions <world> --json` emits `memory.store_bound_mib` (`null` when not written) after
  `parameters`; `format` becomes **2** (`MODULE_SPEC.md` §8.1: a shape change bumps it).
- The digest's canonical memory encoding includes `store_bound_mib` only when written, so every existing
  list's digest is unchanged.

---

## 6. Requirements on other PRs

- **R-FGc-FGb2-1 (FG-c, #201):** §4.9. Expected conflicts: `host.py` (`run_pass` signature, imports of
  `TARGET`), `selection.py` (`Retention` fields), `absorb.py` (`await run_pass`), `bound.py` users.
  **Done in this PR**: FG-c merged first; the conflicts were resolved and the rule implemented here (§12.5).
- **R-P6b-FGb2-1 (P6b, #207):** read `format` 2 of `mineworld decisions --json`; pass `store_bound_mib` into
  `MemoryPolicyChoice` unchanged; a `StoreBoundOutOfRange` refuses the seat at start like any `PolicyError`.

---

## 7. FG-b's pinned numbers, re-derived

| Where | Was | Now | Derivation |
| --- | --- | --- | --- |
| `test_retention_host.py`, `test_retention_policies.py`, `test_retention_real.py` `TARGET` / `STORE_BOUND` | 786 432 / 1 048 576 | 15 728 640 / 20 971 520 | §4.1 at `FLOOR_MIB`, computed in the test from `20 × 1 048 576` and `× 3 // 4`, not imported |
| AFGb-4 (a) mutation "backstop off: `keep-all` over by day 9" | day 9 of the 120 KB/day stream | the heavy stream (§8.2), first size pass | at 120 KB/day `keep-all` reaches 15 MiB only after ≈ 130 days; the heavy stream crosses `target` within its first day |
| AFGb-4 (a) `keep-all` over the 302-day synthetic stream (`test_retention_host.py`) | `keep-all` and drop-everything | drop-everything only; `keep-all`'s half is AFGb2-1 on the busy stream | measured 150 s locally at 20 MiB (170 passes over a ≈ 15 MiB store: FG-b's per-pass extraction is O(store)), minutes more per CI leg, for a property the busy stream proves in one day (D-FGb2-2) |
| AFGb-4 (b) "before every pass ≤ 1 MiB, a measured property of the reference world" | measured property | guaranteed by the trigger; asserted ≤ `bound` | §4.6 |
| AFGb-4 (c) "`keep-all` forced > 0 by day 30 of the real export" | day 30, over 100 days | first forced pass in (30, 40], over days 0 … 40 | 15 MiB at ≈ 433 KB/day is reached after ≈ 36 days (measured: day 34); days 40 … 99 at ≈ 15 MiB per pass would cost ≈ 100 s more per leg against FG-b's 180 s stop (D-FGb2-2) |
| AFGb-11 (c) "file ≤ 4 MiB after the real 100-day `consolidate`" | 4 MiB | unchanged | `consolidate` never nears the bound (778 240 B measured); the figure is the file, not the bound |
| `MODULE_SPEC.md` §5.2 rule 7, `ARCHITECTURE.md` §9.3, README | 768 KiB / 1 MiB, "not a parameter" | 15 MiB / 20 MiB default, raisable, the size trigger | §4.1, §4.3 |

**Amendment, 2026-10-15 (operator, ruled directly in the primary session; landed in PR 12d #171, step-11
§19.13 TD-D16).** AFGb-4 (c)'s window is now chosen by the store's measured size instead of by fixed days.
`test_retention_real.py` `_keep_all_window` works in three phases:
1. It ingests whole days without retention while `content_size()` < `KEEP_ALL_LEAD` (half the target,
   7.5 MiB).
2. It then runs a keep-all pass every day until `KEEP_ALL_AFTER` (3) days past the first forced pass.
3. It stops at `KEEP_ALL_CAP_DAYS` (100, the fixture's whole history). If the store never reaches the
   target by then, the test fails by name.

The bound assertions are unchanged: after every pass ≤ target, before every pass ≤ bound. A forced pass must
occur. The window's first pass must remove nothing, so removals start at the crossing.

**Why the fixed window broke.** 12d's walls and bodies make social-cafe's history grow faster, about
770 KB a day against about 433 KB on main. The old test ingested days 0 … 29 at once, so its first
keep-all `absorb` already saw a store over the target and cut it before day 30, outside the measured
window. No forced pass then fell in (30, 40]. On 12d's head, the new window opens at day 12 (9 280 924 B,
nothing removed), the first forced pass is day 21, and the run takes 19.0 s locally.

---

## 8. Tests and acceptance

### 8.1 Test ownership

```text
STATIC      ruff, pyright strict (cognition); cargo fmt, clippy -D warnings (decision, cli)
UNIT        StoreBounds figures and refusals; resolve's StoreBoundOutOfRange with the source span;
            content_size() = size() − meta after every call (AFGb2-6); the Rust reader accepts and passes
            store_bound_mib, refuses a string or a negative with line and column; the JSON form (format 2)
INTEGRATION the heavy synthetic stream (§8.2) through absorb: bound and coverage at every pass and after
            every absorb call; determinism across frames of 1, 256 and whole; crash between stop and pass;
            FG-b's own suites with re-derived numbers
GATE 2      FG-b's test_retention_real.py (real binary, real 100-day export), numbers re-derived (§7)
```

### 8.2 The heavy stream (test-only, `retention_stream.heavy`)

A pure SplitMix64 generator like FG-b's: `heavy(seed, days, per_day, crowd)` — Alice among a crowd of
`crowd` distinct people (entity ids `7200000000 + n`) at six places; each fact is a `conversation-started`
or `spoke` with 60 participants drawn from the crowd, and every 40th fact is a notable fact at another place,
so episodes close several times an hour, as a walking person's do. Per fact ≈ 5 KB of dump; the defaults
(≈ 5 000 facts per day) give ≈ 24 MB per simulated day, **16 × 12d's 1.5 MB**, more than the bound itself in
one day. It reproduces 12d's failure mode at the new bound: with only a daily pass, the dump before the day-1
pass would exceed 20 MiB (asserted in the test as the precondition, from a plain `ingest` of day 0).

12d's real history is not used: #171 is unmerged, and at 20 MiB its 1.5 MB day never reaches the trigger, so
it cannot exercise the change; FG-b's 100-day real export stays Gate 2 for the re-derived numbers.

### 8.3 Criteria (fixed before measuring)

| ID | Criterion | Planted mutation that must fail it |
| --- | --- | --- |
| **AFGb2-1** Bound | On the heavy stream under `keep-all` and `consolidate` (`decay` dropped for CI time, D-FGb2-2), fed in frames of 256: after every pass `len(dump()) ≤ target` (15 728 640); after every `absorb` call `len(dump()) ≤ bound` (20 971 520); at least two size passes occur, and on day 1 the dump just before the daily pass is ≤ `bound`. | M-FGb2-1: no size trigger (`stop_when` never fires) → the after-call dump exceeds `bound`. M-FGb2-1b: backstop disabled → `keep-all`'s post-pass dump exceeds `target`. |
| **AFGb2-2** Coverage | AFGb-3's oracle (perceived ids = the generator's) holds after every pass of the heavy stream. | owned by FG-b's M-FGb-3a/3c (the oracle is FG-b's); not re-planted (§12.3) |
| **AFGb2-3** Determinism | (a) Frames of 1, of 256 and the whole heavy stream give byte-identical dumps under `keep-all` and `consolidate`; so does FG-b's stream (AFGb-5, unchanged). (b) The trigger's measure, read after every fact of a 23-day stream (chapters written mid-call), is the same sequence in frames of 1, 7, 256 and whole. | M-FGb2-3: the measure includes `meta` → (b) differs. M-FGb2-3b: `close_weeks` only at the end of a call → (b) differs. ((a) alone would rarely see either: a crossing must fall within a few bytes.) |
| **AFGb2-4** Resume | A run whose first size pass raises after its stop committed (simulated crash), resumed by a new `absorb` of the remaining facts, ends with the same dump as an uninterrupted run. | M-FGb2-4: `absorb` skips the entry size check → dumps differ. |
| **AFGb2-5** One pass per fact; log | The pass log's triggers are distinct; every row's `cause` is `day` or `size`; size passes exist only where the dump exceeded `trigger`; the daily pass still runs. A stream whose crossing fact is moved to be a day's first fact gets exactly one pass there, the daily one, which leaves ≤ `target`. | M-FGb2-5: a size pass also at the day's first fact → two passes for one trigger (IntegrityError). |
| **AFGb2-6** The measure | After every ingest call and every pass of a stream with merges and drops, `content_size()` equals `size()` minus `meta`'s lines; after a rolled-back transaction it is still exact. | M-FGb2-6: the UPDATE trigger omitted → mismatch. M-FGb2-6b: no invalidation on rollback → mismatch. |
| **AFGb2-7** Configuration | `store_bound_mib` 19 and 1 025 are refused by `resolve` as `StoreBoundOutOfRange` naming the value, the floor, the ceiling and the source span; 20 (and absent) give 20 971 520; 40 gives 41 943 040, target 31 457 280, trigger 39 321 600; `StoreBounds(19)` cannot be constructed; a string is refused by the Python model. The Rust reader passes `store_bound_mib: 40` per seat (and into the digest), refuses `big` and `-1` with line and column; `--json` shows `format: 2` (CLI golden). | M-FGb2-7: the floor check removed → 19 accepted. M-FGb2-7r: the Rust reader drops the value → the pass-through and digest tests fail. |
| **AFGb2-8** Unchanged elsewhere | FG-b's suites pass with the re-derived numbers only (each change recorded); `test_ac10.py` unchanged; the memory section ≤ 6 000 B (AFGb-7). | — |

---

## 9. Cost of 20 MiB, stated

- **Dump bound:** 20 MiB per person by default; 12 seats ≤ 240 MiB (≈ 252 MB) of canonical dump at worst.
- **On disk:** the SQLite file holds the rows plus the FTS5 index, indexes and page slack: P4's unretained
  store was 3.13 × its dump (§3), so a store at the bound may approach ≈ 63 MiB and 12 seats ≈ 750 MiB in the
  worst case (a policy that keeps everything up to the bound, e.g. `keep-all`). Under the default
  `consolidate`, the real 100-day store is 778 240 B on disk and 300-day dumps stay under 0.9 MB (FG-b
  §15.2): the bound is a ceiling, not a size. Measured for the heavy stream in §12.
- **Save:** unaffected. The memory store is the controller's file, not part of `world.sqlite`; step-24's save
  growth targets (QS24-4) and SR-b's 300-day save figures do not include it.
- **Model context:** unchanged at ≤ 6 000 B of memory per call (D-P4-9), whatever the store's size;
  retrieval reads bounded row counts (FG-b §5.10).

---

## 10. Commit plan

### C0 — This design (Markdown only)

- [x] Implementation: this document. Evidence: commit 183f428.
- [x] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
  Evidence: "223 numbered sections across 32 documents, none duplicated"; "122 decision ids, all distinct".
- [x] Review: terminology; every rule has a source; no new decision id. Evidence: §1.1 quotes both rulings;
  ARC-87 gains a dated note (C3), no new id.

### C1 — Bounds, the measure, the trigger, schema v3 (Python)

- [x] Implementation: §4.1–4.5 (`bound.py`, `selection.py`, `absorb.py`, `host.py`, `ingest.py`, `store.py`,
  `records.py`). Evidence: commit 2326484; D-FGb2-1 (the meter's mechanism) recorded in §12.4.
- [x] Validation: ruff, pyright strict; AFGb2-1 … AFGb2-7 (Python half), AFGb2-8 targeted files; mutations.
  Evidence: §12.2, §12.3.
- [x] Review: determinism argument (§4.2–4.3) against the code; no wall clock; `ingest` without `stop_when`
  unchanged. Evidence: every metered write is per fact (`_write`, `Episodes.feed/_join/_close`,
  `Stable.feed/met`, `NameBook.learn`), and `close_weeks` runs after each closing fact when `stop_when` is
  given; `meta` is excluded; AFGb2-3 (b) checks the measure's sequence directly. `ingest` without
  `stop_when` takes no new branch (the per-fact block is guarded by `stop_when is not None`);
  `test_ac10.py` 8 passed unchanged. No `time`, `datetime` or `random` import added.

### C2 — The Rust reader and the JSON form

- [x] Implementation: §5 (`read.rs`, `resolve.rs`, `json.rs`, CLI fixture). Evidence: commit 41b00ac.
- [x] Validation: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test -p mineworld-decision` and the
  CLI's `decisions` test. Evidence: fmt clean; clippy (decision, cli, all targets) clean; 17 passed; CLI
  `--test decisions` 7 passed.
- [x] Review: pass-through only; digest unchanged without the key. Evidence: `choice()` copies the value;
  `CanonicalMemory.store_bound_mib` is `skip_serializing_if = "Option::is_none"`; the existing digest
  assertions pass unchanged and a new one shows a written bound changes it. Finding F-FGb2-2 (§12.4).

### C3 — Docs and close-out

- [x] Implementation: §2.1's doc edits; FG-b's ledger note; §12 evidence. Evidence: `MODULE_SPEC.md` §4.1
  rule 8, §5.2 (budget `now`, host rules 1, 2, 5, 7, 8, 9, Selection), §8.1; `ARCHITECTURE.md` §9.3;
  ARC-87 note; README; FG-b §5.7 note.
- [ ] Validation: doc checks; CI green on the exact head. Doc checks clean (§12.2); CI: the PR.
- [x] Review: no stale 768 KiB / 1 MiB claim remains outside history notes. Evidence: `git grep "768 KiB\|786
  432\|1 048 576\|1 MiB" -- docs cognition/lm-controller/README.md` finds only ARC-87's original text, which
  its new note supersedes.

---

## 11. Execution contract

```text
PROJECT / PR:          MineWorld mvp0, FG-b2 — per-person bound 20 MiB (configurable upward), size trigger
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-fg-b2-size-trigger.md
BINDING DOCS:          pr-fg-b-retention-host.md (§§2.3, 5.6, 5.7, 6, 9); step-24 §10.1; ARC-87; CLAUDE.md
WORKTREE / BRANCH:     /Users/yuema137/mineworld-worktrees/impl-fg-b2, mvp0/pr-fg-b2 from origin/main
ALLOWED COMMANDS:      cargo *; git; gh (never merge); uv *; python3 scripts/*; target/*/mineworld *
NEVER:                 python3 -c; sed -i; heredoc writes; reading keys; calling any model
MATERIAL STOPS:        a change to the 6 000-byte ceiling; a floor below 20 MiB; any change outside §2.1
VALIDATION BUDGET:     targeted test files and mutations locally (R-LOAD-1: no hour-long suites); CI is the
                       gate, on the exact head (fast, test, test-macos, test-windows, platforms, python ×3)
ENDPOINT AUTHORITY:    commits, push, PR creation and CI repair: authorized by the brief; merge: never
STOP CONDITION:        READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## 12. Ledger

### 12.1 Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-FGb2-1 | The per-fact measure slows ingestion. | Incremental; measured in §12.2; the first use is one full measure. |
| R-FGb2-2 | An open episode alone larger than `target` stalls the seat (§4.6–4.7). | Named error; ≈ 10 × 12d's day in one episode; a world raises its bound; force-closing is a later ruling. |
| R-FGb2-3 | Merge order with FG-c / P6b. | §6; noted in the PR body. |

### 12.2 Evidence

macOS arm64, Python 3.14.5 (`uv run --locked`), the host shared with other sessions (load 4–20), so wall
times are upper bounds. Targeted files only (R-LOAD-1); CI on the exact head is the gate.

```text
ruff check / ruff format --check / pyright (strict) cognition/lm-controller    clean / clean / 0 errors
pytest test_memory_store, test_retention_store, _plan, _interface, test_memory_compress, _retrieve,
       _isolation, test_retention_retrieve, test_memory_render, _summarizers, test_retention_size
                                                                               92 passed (99.6 s)
pytest test_retention_host, _policies, _size                                   51 passed (217 s)
pytest test_retention_real, test_ac10 (real binary, debug, built in this worktree)
                                                                               10 passed
  consolidate, 98 passes: day 30 (before 608 994, after 188 673), day 99 (800 695, 331 358); largest
    before 960 176, after 334 735; file 765 952 B; frames of 256 = day by day                 40.6 s
  keep-all, days 0 … 40, 39 passes: day 30 after 13 948 340; first forced pass day 34      36.4 s
  (final code, load 2.5; the first run, with §4.2's frozen temp-table meter, took 49.5 s and 55.7 s)
  test_ac10: section bytes {10: 4599, 30: 4990, 60: 4943, 100: 4991}, unchanged
cargo test -p mineworld-decision 17 passed; cargo test -p mineworld-cli --test decisions 7 passed
cargo fmt --all --check, cargo clippy -p mineworld-decision -p mineworld-cli --all-targets -D warnings
                                                                               clean
check_doc_headings / check_decision_ids                                        clean
```

The busy stream (`heavy(7, 2, 2800, 100)`, 5 612 facts, two days), frames of 256 — (cause, dump before,
dump after, forced):

```text
plain ingest of day 0 alone                         dump 23 933 151 B  > bound 20 971 520 (precondition)
keep-all     size (19 663 152 → 1 608 527, 2 292)  day (5 892 824 → 5 893 055, 0)
             size (19 663 279 → 2 877 875, 2 120)  largest dump after an absorb call 19 623 272
consolidate  size (19 663 152 → 1 608 529, 0)      day (5 892 826 → 1 776 221, 0)
             size (19 661 577 → 3 192 021, 0)      largest dump after an absorb call 19 623 272
```

Every `before` exceeds the trigger (19 660 800) by less than one fact (≤ 2 801 B over it) and is under the
bound; every `after` is under the target (15 728 640).

FG-b's 302-day synthetic stream at 20 MiB, dump at days 30 / 100 / 300: drop-everything 9 179 / 9 909 /
9 944; consolidate 282 723 / 517 325 / 530 871; decay 1 600 924 / 1 619 394 / 1 543 513; importance-recency-cap
1 169 396 / 1 309 642 / 1 331 251. `decay` and `importance-recency-cap` keep more than at 1 MiB (FG-b
§15.2: 619 137 and 417 576 at day 100) because their budget is now `target − overhead`: the bound is a
ceiling a policy may use, and both still end far under it.

Cost of the measure: the busy stream's two days (≈ 1.24 M row writes) spend ≈ 5 s in the triggers' encoding
under the profiler; the 100-day real export (124 207 facts) through day-by-day ingest and `absorb` under
`consolidate` takes 13.5 s in all (scratch run, `fgb2-scratch/keepall_real.py`, outside the repository).

### 12.3 Mutations

Each planted in the source, the named test run, seen red, reverted (`git checkout` / restored copy; tree
compared clean after each).

```text
M-FGb2-1   absorb's trigger predicate always false              → test_size_passes…[keep-all] FAIL
           (25 629 272 > 20 971 520 after an absorb call)
M-FGb2-1b  the backstop returns at once                         → test_size_passes…[keep-all] FAIL
           (a pass cannot get under the trigger, the next absorb's entry check passes again at the same
           fact: IntegrityError on the trigger key; 8 min of size passes on every fact first)
M-FGb2-3   meta metered too                                     → test_the_incremental_measure… FAIL and
           test_the_measure_after_each_fact… FAIL (frames of 1)
M-FGb2-3b  no close_weeks after a closing fact                  → test_the_measure_after_each_fact… FAIL
           (frames of 1, index 333: 160 734 != 160 270)
M-FGb2-4   absorb's entry size check removed                    → test_a_crash_between_a_stop… FAIL (dumps
           differ: the pass ran three facts later)
M-FGb2-5   a size pass also at a day's first fact               → test_a_crossing_at_a_days_first_fact… FAIL
           (IntegrityError: one trigger, two passes)
M-FGb2-6   the UPDATE triggers omitted                          → test_the_incremental_measure… FAIL
M-FGb2-6b  no invalidation on rollback                          → test_the_incremental_measure… FAIL (both
           policies, after the rolled-back transaction)
M-FGb2-7   the floor check removed (0 … 1 024)                   → two configuration tests FAIL
M-FGb2-7r  the Rust reader drops store_bound_mib                 → memory_is_resolved_per_seat… and
           the_digest_follows_the_resolution… FAIL
M-FGb2-9   a size pass of an assisted seat takes the assisted branch (after the merge of FG-c)
                                                                 → test_a_size_pass_of_an_assisted_seat… FAIL
           ({'fallback:exhausted'} != {'fallback:size'}: the scripted model was asked)
```

### 12.5 Merge of origin/main (FG-c #201, S27-a #205, S28 docs #209)

FG-c merged before this PR: `run_pass` is async and takes the assisted branch; `Retention` gained `fallback`
and `gateway`; the pass log gained `model_decisions` and `fallback_decisions` (at schema v2). Resolved:
`Retention(policy, fallback, gateway, bounds)`; `resolve` sets the bounds on both branches; `PassReport`
keeps FG-c's fields before `cause` and `trigger`; the v3 pass log holds all fourteen columns; every
`run_pass` call in `absorb` is awaited; the test wrappers of `run_pass` are async. R-FGc-FGb2-1 implemented
(§4.9). S28 assigned ARC-98 … ARC-101 and DEP-58; this PR uses none.

**F-FGb2-4 — FG-c's replay drift guard moves with the schema.** `test_lm_summarise_replay.py`'s `DIGEST`
(the sha256 of the committed cassette's replayed dump) changed: every pass-log row gains `trigger` and
`cause`, and `meta.schema` reads 3. The cassette is not re-recorded and replays without a miss, so no request
moved; `lm-summarise` and `consolidate` do not read `store_bytes`, so the larger budget changes no decision.
The constant is reset with that reason in its docstring.

### 12.4 Deviations and findings

**D-FGb2-1 (C1, bounded) — the meter calls into the store instead of marking rows.** §4.2 as frozen had the
triggers record touched rowids in a `TEMP` table that each call re-read. Measured: ≈ 7 SQL statements per fact
(the real `consolidate` test took 49.5 s against FG-b's 33.4 s; 40.6 s after this change, on a lighter
host), and a trigger's `OR IGNORE` yields to the outer statement's conflict policy (`UNIQUE constraint
failed` on upserts). The triggers now call two connection
functions with the row's values, so a check is O(1) and no statement is added per fact; `size()` and
`row_bytes()` reuse the sizes once the meter runs. Same exactness (AFGb2-6), same mutations red.

**D-FGb2-2 (C1, bounded) — test budget at 20 MiB.** FG-b's per-pass extraction is O(store); at a 15 MiB target
a `keep-all` store makes every pass ≈ 1 s here. `keep-all` therefore left the 302-day synthetic bound test
(its claim moved to AFGb2-1) and runs on the real export through day 40 (AFGb-4 (c) holds by day 34);
AFGb2-1 runs `keep-all` and `consolidate` only. FG-b's §12 stop (`test_retention_real.py` ≤ 180 s per leg)
is kept: 77 s of test calls here (plus the shared 100-day fixture), the CI legs are recorded on the PR.

**D-FGb2-2, continued (after CI run 38696439495 on aa207a6, all jobs green).** The Windows python leg ran
`test_retention_real.py` in 210 s (`consolidate` 120 s, `keep-all` days 0 … 40 90 s), over the 180 s stop.
`keep-all` now ingests days 0 … 29 at once with no pass (a keep-all pass under the target removes nothing)
and passes day by day from day 30 to 40: 10 passes, 9.6 s here against 36.4 s; first forced pass still
day 34, every pass ≤ target, coverage at every pass. The next head's Windows time is recorded on the PR.

**F-FGb2-1 — larger budgets make `decay` and `importance-recency-cap` keep more** (§12.2), and their 302-day
tests take ≈ 2 × longer (35 s and 31 s). Expected under a 20 MiB bound; recorded.

**F-FGb2-2 (C2) — the YAML decoder reads a quoted `"40"` as the integer 40** for an unsigned-integer field
(`serde-saphyr`'s scalar resolution). The reader refuses a non-numeric scalar (`big`) and `-1` with line and
column; the quoted case passes through as 40, which the host then validates. Recorded, not changed: it is the
decoder's behaviour for every integer field of the file, not this field's.

**F-FGb2-3 — a pass log key collision is the symptom of a pass that did not reach its target.** Under
M-FGb2-1b the next `absorb` re-ran a size pass at the same fact. In the real code a pass either ends ≤ target
(< trigger) or raises `RetentionBoundUnreachable`, so the entry check cannot repeat a fact; the unique key
turns any future violation into a loud error rather than a silent second pass.
