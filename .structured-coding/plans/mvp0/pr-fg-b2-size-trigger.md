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
- It is maintained incrementally. On first use on a connection, the store creates a `TEMP` table
  `meter_dirty (tbl, rid)` and `TEMP` triggers `AFTER INSERT / UPDATE / DELETE` on each metered table that
  record the touched `rowid` (old and new for an update). Then it measures every metered row once, keeping
  each row's line size by `(table, rowid)`. Each later call re-encodes only the dirty rows (absent → its size
  is removed), clears the dirty set and returns the running total. Encoding is the dump's own encoder and
  cell decoding, so a line has the same bytes as in `dump()`.
- `TEMP` objects live in SQLite's per-connection temp schema: nothing enters the file, the schema or the
  dump. A transaction that rolls back also rolls back the dirty rows, so `MemoryStore.transaction` drops the
  cached sizes on rollback; the next call measures in full.
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

- **Per fact:** one `content_size()` refresh: the rows the fact touched (an `l0` row, one `l0_who` and one
  `stable` row per participant, the open episode, rarely `names`, `episode_who`, `chapters`), each one indexed
  `SELECT` by rowid and one encode. Measured in §12.
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
- **R-P6b-FGb2-1 (P6b, #207):** read `format` 2 of `mineworld decisions --json`; pass `store_bound_mib` into
  `MemoryPolicyChoice` unchanged; a `StoreBoundOutOfRange` refuses the seat at start like any `PolicyError`.

---

## 7. FG-b's pinned numbers, re-derived

| Where | Was | Now | Derivation |
| --- | --- | --- | --- |
| `test_retention_host.py`, `test_retention_policies.py`, `test_retention_real.py` `TARGET` / `STORE_BOUND` | 786 432 / 1 048 576 | 15 728 640 / 20 971 520 | §4.1 at `FLOOR_MIB`, computed in the test from `20 × 1 048 576` and `× 3 // 4`, not imported |
| AFGb-4 (a) mutation "backstop off: `keep-all` over by day 9" | day 9 of the 120 KB/day stream | the heavy stream (§8.2), first size pass | at 120 KB/day `keep-all` reaches 15 MiB only after ≈ 130 days; the heavy stream crosses `target` within its first day |
| AFGb-4 (b) "before every pass ≤ 1 MiB, a measured property of the reference world" | measured property | guaranteed by the trigger; asserted ≤ `bound` | §4.6 |
| AFGb-4 (c) "`keep-all` forced > 0 by day 30 of the real export" | day 30 | by day 100 | 15 MiB at ≈ 433 KB/day is reached after ≈ 36 days |
| AFGb-11 (c) "file ≤ 4 MiB after the real 100-day `consolidate`" | 4 MiB | unchanged | `consolidate` never nears the bound (778 240 B measured); the figure is the file, not the bound |
| `MODULE_SPEC.md` §5.2 rule 7, `ARCHITECTURE.md` §9.3, README | 768 KiB / 1 MiB, "not a parameter" | 15 MiB / 20 MiB default, raisable, the size trigger | §4.1, §4.3 |

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
| **AFGb2-1** Bound | On the heavy stream under `keep-all`, `consolidate` and `decay`, fed in frames of 256: after every pass `len(dump()) ≤ target` (15 728 640); after every `absorb` call `len(dump()) ≤ bound` (20 971 520); at least two size passes occur, and on day 1 the dump just before the daily pass is ≤ `bound`. | M-FGb2-1: no size trigger (`stop_when` never fires) → the after-call dump exceeds `bound`. M-FGb2-1b: backstop disabled → `keep-all`'s post-pass dump exceeds `target`. |
| **AFGb2-2** Coverage | AFGb-3's oracle (perceived ids = the generator's) holds after every pass of the heavy stream. | M-FGb2-2: a size pass shows the open episode's records as closed (eligible) → the structural check or the oracle fails. |
| **AFGb2-3** Determinism | Frames of 1, of 256 and the whole heavy stream give byte-identical dumps under `keep-all` and `consolidate`; so does the 300-day FG-b stream (AFGb-5, unchanged). | M-FGb2-3: the trigger measures the whole dump including `meta` → frames of 1 differ (or, if the run happens not to differ, the static AFGb2-6 check owns it; recorded). M-FGb2-3b: `close_weeks` only at the end of a call. |
| **AFGb2-4** Resume | A run whose first size pass raises after its stop committed (simulated crash), resumed by a new `absorb` of the remaining facts, ends with the same dump as an uninterrupted run. | M-FGb2-4: `absorb` skips the entry size check → dumps differ. |
| **AFGb2-5** One pass per fact; log | The pass log's triggers are distinct; no fact has a `day` and a `size` pass; every pass log row's `cause` is `day` or `size`; size passes exist only where the content exceeded `trigger`; the daily pass still runs on every day with facts. | M-FGb2-5: a size pass also at the day's first fact → two passes for one trigger (IntegrityError) or a duplicate. |
| **AFGb2-6** The measure | After every ingest call and every pass of a stream with merges and drops, `content_size()` equals `size()` minus `meta`'s lines; after a rolled-back transaction it is still exact. | M-FGb2-6: the UPDATE trigger omitted → mismatch. M-FGb2-6b: no invalidation on rollback → mismatch. |
| **AFGb2-7** Configuration | `store_bound_mib` 19 and 1 025 are refused by `resolve` as `StoreBoundOutOfRange` naming the value, the floor, the ceiling and the source span; 20 (and absent) give 20 971 520; 40 gives 41 943 040, target 31 457 280, trigger 39 321 600; `StoreBounds(19)` cannot be constructed. The Rust reader passes `store_bound_mib: 40` per seat, refuses `"40"` and `-1` with line and column; `--json` shows `format: 2`. | M-FGb2-7: the floor check removed → 19 accepted. |
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

- [x] Implementation: this document. Evidence: the file.
- [ ] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
- [ ] Review: terminology; every rule has a source; no new decision id.

### C1 — Bounds, the measure, the trigger, schema v3 (Python)

- [ ] Implementation: §4.1–4.5 (`bound.py`, `selection.py`, `absorb.py`, `host.py`, `ingest.py`, `store.py`,
  `records.py`).
- [ ] Validation: ruff, pyright strict; AFGb2-1 … AFGb2-7 (Python half), AFGb2-8 targeted files; mutations.
- [ ] Review: determinism argument (§4.2–4.3) against the code; no wall clock; `ingest` without `stop_when`
  unchanged.

### C2 — The Rust reader and the JSON form

- [ ] Implementation: §5 (`read.rs`, `resolve.rs`, `json.rs`, CLI fixture).
- [ ] Validation: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test -p mineworld-decision` and the
  CLI's `decisions` test.
- [ ] Review: pass-through only; digest unchanged without the key.

### C3 — Docs and close-out

- [ ] Implementation: §2.1's doc edits; FG-b's ledger note; §12 evidence.
- [ ] Validation: doc checks; CI green on the exact head.
- [ ] Review: no stale 768 KiB / 1 MiB claim remains outside history notes.

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

(filled during implementation)
