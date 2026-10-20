# PR S28-e — Memory inspection and seat export: `mineworld-memory inspect` (access, forgotten, bytes, recall --trace, scopes), `export` / `import`, and one step to the cause

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: every question answered with its recommendation)

```text
Design revision:        draft 1 (2026-10-18)
Author:                 S28-c / S28-e PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s28ce, branch docs/pr-designs-s28ce,
                        planning base origin/main @ b85f2804 (#260; S28-a #242 merged at 9884c64c; S28-b's
                        design merged #235 at 450f7985, its implementation NOT merged)
Approved by / evidence: the operator, 2026-10-18, in the primary session: every question answered with its recommendation.
Implementation base:    origin/main at C0, after S28-b has merged; C0 records the commit
Start condition:        step-28 §11.1: S28-b MERGED (schema 5, attributed forgotten ranges); the views of
                        S28-c and S28-d are added as each merges (§2.3, QS28e-4); this design frozen; the
                        primary session's dispatch
Execution contract:     §12 (filled; endpoint authority included)
Implementer:            Haiku-class (step-28 §11.1), the primary reviewing
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0` · **Step:** [`step-28-shared-memory.md`](step-28-shared-memory.md) (STEP DESIGN FROZEN
2026-10-14; rulings §14.1) §§4.4, 5.6 (inspection), 5.5 (errors), 6.6 (export / import), 9.1 (surface 3: the
human-visible memory page is `inspect recall`), 9.2 (C-7), 11 and 11.1 (the S28-e row), 15 (AS28-11, AS28-16) ·
**Parent:** [`overall.md`](overall.md) · **Previous PRs:** [`pr-s28a-memory-home.md`](pr-s28a-memory-home.md)
(merged #242; its R-S28e-a-1), [`pr-s28b-retention-on-seat-views.md`](pr-s28b-retention-on-seat-views.md)
(FROZEN 2026-10-18; its R-S28e-b-1 and the API of its §5.3) · **Coordinates with:**
[`pr-s28c-memory-scopes.md`](pr-s28c-memory-scopes.md) (its R-S28e-c-1; designed in the same session) and
S28-d (the anchored scope's lines) · **Binding parents:** `CLAUDE.md` §§2, 3, 4 (items 1, 3, 6, 7, 8, 9, 10, 11,
12, 13, 16); `docs/CORE_CONCEPTS.md` §§2 (INV-4, INV-13), 5.3; `docs/ARCHITECTURE.md` §9.3;
`docs/MODULE_SPEC.md` §5.3 ("Inspection and degradation"); `docs/MVP.md` §9 `AC-18` ("A diagnostic view traces
any recalled line to its source events and any forgotten item to the policy that removed it");
`docs/DECISIONS.md` `ARC-59`, `ARC-98`, `ARC-101`, `ARC-109`.

**Decision ids.** None new. C1 writes a dated note under `ARC-109`: the seat-export format (QS28e-1 as ruled)
and the read-only open.

No hosted model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Scope (step-28 §11, S28-e row).** "`mineworld-memory inspect` (access, forgotten, bytes, recall --trace,
  scopes) and `export` / `import`; the one-step-to-cause tests (AS28-16)." Implementer Haiku; reviewer the
  primary. Start: "S28-b merged. The views of S28-c and S28-d are added as each merges (the commit plan says
  which)." Parallel with S28-c and S28-d.
- **Inspection (step §5.6, verbatim in its rules).** "The command and the Python API share one implementation
  and produce one typed `InspectReport` (JSON with `--json`). The command only reads: it opens the home
  read-only and never writes, not even the pass log. It runs while the cognition process is stopped. While the
  process runs, the same report is produced by the API, which the process can serve." The five views of the
  step's table (`access [--event]`, `forgotten`, `bytes`, `recall --at T [--with] [--words] [--trace]`,
  `scopes`) and "one step to the cause".
- **Export / import (step §6.6, verbatim).** "`mineworld-memory export HOME --seat alice > alice.seat` writes
  the seat view, the standalone schema-4 seat view. `import` splits it into records (insert-or-verify) and
  private rows. Export then import is byte-identical by I-S28-2 (AS28-11)."
- **AS28-11 (step §15).** "`export` then `import` of every seat into a fresh home gives a home dump equal to the
  original's."
- **AS28-16 (step §15).** "(a) A planted wrong gist in seat B's access row is named by `inspect recall --trace`
  (seat, scope, key, row) in one command. (b) A planted oversized scope row is named by `inspect bytes` (scope,
  table). (c) A planted bound below 20 MiB fails at start with `BoundBelowDefault` naming the seat and the
  source span."
- **C-7 (step §9.2).** "Every perceived id is held, merged into a surviving unit, or forgotten (FGb-1) per seat
  view. A forgotten id is never cited. `inspect access --event` explains every id of a sample of 1 000."
- **R-S28e-a-1 (S28-a §10).** "`inspect access` derives 'how' from `fact_who.role`."
- **R-S28e-b-1 (S28-b §10).** "`inspect … forgotten` and `access --event` read `ForgottenSet.attributed()` and
  `range_of()`; 'forced' is `ForgottenRange.forced()`; the pass log's `prose_contradictions` is shown in the
  last-32-passes table."
- **Integration checkpoint (step-28 §11.1).** "Every PR ends with the transparency harness green (12 seats,
  `none` vs `home`). From S28-b on, it also runs FG-b's real-binary Gate 2 scenario on seat views."
- **R-LOAD-1 (coordinator rule, in force).** §11.

---

## 1. Goal

Make every person's memory **explainable from the outside, in one command**, and **portable between homes**,
without the inspector ever changing what it inspects:

1. `mineworld-memory inspect HOME --seat KEY <view>` answers, for a stopped home: what the person can access and
   why; what was forgotten, by which policy, on which day, and whether it was forced; where the bytes are; the
   exact memory section a consult would receive, every line traced to its facts; and (once S28-c has merged) the
   memory stack the person runs.
2. The same reports come from a Python API over an open home, so a running cognition process can produce them.
3. `mineworld-memory export` writes one seat as a self-contained, deterministic file; `import` adds it to another
   home of the same world instance, so a seat moves between processes and the two homes' dumps prove it moved
   whole (AS28-11).
4. A planted defect is located in one command: a wrong gist names its seat and access row, an oversized row
   names its table and scope (AS28-16 (a), (b)).

Nothing changes in what memory stores, recalls or forgets; no schema, dump, policy or bound changes. Nothing
changes under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/` or `sdk/` (I-S28-10).

---

## 2. Scope

### 2.1 In scope

1. **The read-only open** (`memory/home/home.py`): `MemoryHome.open_read_only(path)`, which reads its world
   instance from the file, writes nothing, and refuses a missing file, a held home and a home needing crash
   recovery by name (§5.2); `MemoryHome.totals()`, the home's byte totals (§5.5).
2. **Explanations** (`memory/explain.py`, new; pure over a `SeatView`): the held counts by "how" and salience,
   the classification of one event id, the units covering an id (§5.3).
3. **The seat export** (`memory/home/export.py`, new): `export_seat(home, key) -> bytes` and `import_seat(home,
   data) -> EntityKey`, in the format of QS28e-1 (§5.7).
4. **The inspector** (`mineworld_cognition/inspect/`, new package, outside `memory/`): the typed `InspectReport`
   models, the request types, `report(home, request, inputs)` (the API), and the command `mineworld-memory`
   (`inspect`, `export`, `import`) (§5.1, §5.4–§5.6, §5.8).
5. **The console script** `mineworld-memory` in `cognition/lm-controller/pyproject.toml` `[project.scripts]`.
6. **Tests** of §7, including AS28-11, AS28-16 (a) and (b), and C-7's sample of 1 000.
7. **Specifications:** `docs/MODULE_SPEC.md` §5.3 "Inspection and degradation" corrected (§3.2); a dated note
   under `ARC-109` (C1).
8. **The `scopes` view and per-scope bytes**, if S28-c has merged first (QS28e-4; C8 is conditional).
9. `cognition/lm-controller/README.md`: one line naming `mineworld-memory` and linking the generated CLI page.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Serving `InspectReport` from a running cognition process (an endpoint, a socket) | a later PR (S28-f or a tools step); this PR provides the API it would call |
| A client page reading `inspect recall` (step §9.1 surface 3, §18) | a presentation step |
| AS28-16 (c), `BoundBelowDefault` at start | S28-c (the error and its start-time check land there; S28-c's AC-c-3) |
| The `anchored` scope's own lines in `recall --trace` and its tables in `bytes` | S28-d, through the views this PR defines (R-S28d-e-1) |
| Per-record merge attribution ("merged into episode *n* on day *d* by policy *p*") | not stored by any schema (F-S28e-3); QS28e-2 |
| A change to what memory stores, recalls or forgets; a schema, dump, policy or bound change | never in this PR (material stop, §12) |
| Any change under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` | never (I-S28-10) |

### 2.3 Dependencies and the start condition

- **Required: S28-b merged.** `forgotten`'s attribution columns, `retention_passes.prose_contradictions`,
  `ForgottenSet.attributed()`, `range_of()`, `ForgottenRange.forced()` (S28-b §5.1, §5.3) exist only then.
- **Merged:** S28-a (#242), P6b (#207), P7 (#244).
- **S28-c** (designed in parallel): `inspect … scopes`, per-scope `bytes` and the scope column of `recall --trace`
  belong to whichever of S28-c and S28-e merges second (QS28e-4, R-S28e-c-1). Until S28-c merges, every seat-view
  table is the `person` scope's (the only scope with seat-view tables, step §5.2.1), and a trace line's scope is
  `person`.
- **S28-d** (after S28-c): adds its scope's rows to the views through R-S28d-e-1; not a dependency.
- **Open PR #262** (FU-S28a-1, the home's creation marker) writes a `home_meta` row at the first open of an
  unmarked home. The read-only open must never write it (I-e-1); C0 records whether #262 merged.

---

## 3. Audit (`origin/main @ b85f2804`)

Line numbers are on `b85f2804` (schema 4, S28-a merged, S28-b not merged). C0 re-anchors each on the base; the
S28-b items are read from its frozen design and checked on the merged code at C0.

### 3.1 Source anchors (under `cognition/lm-controller/src/mineworld_cognition/` unless a path says otherwise)

| # | Fact | Where |
| --- | --- | --- |
| E-1 | **There is no read-only open.** `MemoryHome.open(path, world_instance)` connects with `timeout=0`, takes the exclusive lock (`_lock`: `PRAGMA locking_mode = EXCLUSIVE`, `BEGIN EXCLUSIVE; COMMIT`), and **creates** schema 4 in an empty file. It needs the world instance from the caller. | `memory/home/home.py:83-108`, `:381-425` |
| E-2 | `view_of(key)` returns a seat's view without creating it, `SeatNotInHome(seat)` otherwise; `seats()` lists the keys. | `memory/home/home.py:174-188`, `:233-237` |
| E-3 | **The dumps shorten the digest.** `DIGEST_COLUMNS = {"digest"}` → `short_digest` (32 hex) in both `SeatView.dump()` (`l0`) and `MemoryHome.dump()` (`fact`); the table keeps the full 64 hex, and ingest's re-delivery compares the full one. | `memory/home/schema.py:90-101`; `memory/home/encoding.py:36-38`; `memory/home/home.py:53`, `:249-254`; `memory/ingest.py:96-101` |
| E-4 | **A seat view does not carry its records' envelopes.** `l0_who` shows only rows of role 1 or 3 other than the seat itself; `fact_who`'s subject-only rows (role 2) and the seat's own row exist only in the home. | `memory/home/view.py:61-66`; `memory/home/schema.py:30-33` |
| E-5 | `MemoryHome.dump()`: `home_meta`; `fact` (id once, short digest); `fact_who`, one grouped line per record; per seat in key order, `["seat", key, self_entity]`, then the seat's `meta` rows (without identity), `names`, `access`, then the private tables in the view's order. | `memory/home/home.py:239-262`, `:325-375` |
| E-6 | `ForgottenSet.load(view)` gives ranges and the watermark; `contains` answers for explicit ranges only. S28-b adds `attributed()`, `range_of(event)`, `ForgottenRange.forced()`, schema 5's columns. | `memory/retention/forgotten.py:23-51`; S28-b §5.1, §5.3 |
| E-7 | The pass log: `retention_passes` (trigger, day, cause, policy, outcome, kept, merged, dropped, forced, bytes before and after, forgotten ranges, model and fallback decisions; S28-b adds `prose_contradictions`), at most `PASS_LOG_ROWS = 32` per seat. | `memory/home/schema.py:68-73`; `memory/retention/host.py:45`, `:347-379` |
| E-8 | **No record says which pass merged a record into a unit.** A merge deletes the access row (`SeatView.release`) and logs only the pass's counts. | `memory/retention/plan.py:121-150`; `memory/home/view.py:234-252` |
| E-9 | `_represented(view, key)` decides "merged or forgotten" for an unheld id: inside one of the seat's episodes' or chapters' ranges, in its forgotten set, or at or below its watermark. A unit's range stands for the **perceived** ids between its ends, not every world id (records.py's docstring), so an id inside a range may never have been perceived. | `memory/ingest.py:140-153`; `memory/records.py:1-8` |
| E-10 | The section a consult reads is `memory_view(store, query, recall_episodes)` (S28-c: `memory_view(store, stack, query, recall_episodes)`), with `recall_episodes` from `[controller]` or `[seat_overrides.<key>.controller]` of `cognition.toml`. | `controller/memory_view.py:18-29`; `controller/seat.py:368-376`; `config.py:298-345`, `:379-` |
| E-11 | **The isolation scan forbids files in memory.** `memory/`, `compress/`, `forgetting/` may not contain `open(`, `os.`, `Path(`, `subprocess`; only `memory/home/home.py` connects; writes to `fact`, `fact_who`, `access`, `access_text` only in `memory/home/` and `memory/scopes/world.py`; memory imports nothing outside memory, compress, forgetting and three named modules. | `tests/test_memory_isolation.py:20-80` |
| E-12 | The distribution has no `[project.scripts]`; its one command is `python -m mineworld_cognition`. | `cognition/lm-controller/pyproject.toml`; `__main__.py:1-7` |
| E-13 | `mineworld perceived <world> --save DIR --person KEY [--since ID] --json` prints a person's perceived stream. | `docs/reference/cli.md:100-120` |
| E-14 | **The home does not know a seat's bound or stack.** Both come from the decision JSON at seat start (`MemoryPolicyChoice.store_bound_mib`; S28-c's `scopes`). `StoreBounds(mib)` gives bound, target, trigger. | `memory/retention/selection.py:36-46`, `:111-116`; `memory/retention/bound.py` |
| E-15 | The FTS index `access_text` is rebuilt from `access` and never dumped; its `rowid` equals the access row's. | `memory/home/schema.py:74-77`; `memory/home/view.py:198-202` |
| E-16 | The seat's sizes: `size()` = the view dump's length; `content_size()` = less its `meta` lines; `row_bytes(table)` gives each view row with its line bytes. | `memory/home/view.py:255-296` |
| E-17 | The step's `MemoryHome.inspect()` would make the home (in `memory/`, which may import nothing of the controller, E-11) build a report that needs `memory_view` (in `controller/`). | E-10, E-11 |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S28e-1** | **The step's export cannot be imported.** A seat view dump (E-3, E-4) has the short digest and lacks every `fact_who` row it does not show. An import from it cannot rebuild the records, so the imported home's dump differs from the original's (AS28-11 fails by construction), and the first re-delivery of a fact after import raises `ConflictingFact` (128-bit stored against a 256-bit digest). | **§14 QS28e-1** (the export format). |
| **F-S28e-2** | Opening read-only needs SQLite's read-only mode (E-1), and the world instance from the file. A home a killed process left with a hot journal cannot be opened read-only without rolling the journal back, which is a write. | **Bounded:** `open_read_only(path)` connects with the URI `file:<path>?mode=ro` (`uri=True`, `timeout=0`), reads `home_meta`, and never creates, marks or locks for writing. A missing path → `NotAMemoryHome(path)` (no file is created); a held home (the running process's exclusive lock) → `HomeBusy(path)`; a hot journal → `HomeNeedsRecovery(path)`, whose message says: start the cognition process for this home once, or open it with `MemoryHome.open`. |
| **F-S28e-3** | The step's "merged (into episode *n* or chapter *w*, the pass day and policy)" asks for an attribution no schema keeps (E-8). | **§14 QS28e-2.** |
| **F-S28e-4** | "Merged" and "never perceived" are indistinguishable from the home for an unheld id inside a unit's range (E-9). | **§14 QS28e-3.** |
| **F-S28e-5** | `bytes` shows "the bound" and `scopes` shows "the selected scopes", neither of which the home stores (E-14); `recall` must match what a consult receives, which depends on `recall_episodes` (E-10). | **Bounded:** the inspector takes `--decisions FILE` (the JSON `mineworld decisions --json` prints, the file the cognition process reads) and `--config FILE` (`cognition.toml`). Without them it uses the defaults (20 MiB; `[person]`; `recall_episodes = 8`) and every report says which values are defaults. |
| **F-S28e-6** | `MemoryHome.inspect()` (step §5.6, MODULE_SPEC §5.3) would make the home depend on the controller (E-17) and grow it toward a God object (`CLAUDE.md` §4 item 6). | **Bounded:** the API is `mineworld_cognition.inspect.report(home, request, inputs) -> InspectReport`. The home keeps only what is storage (`open_read_only`, `totals`); explanation is `memory/explain.py`. C1 corrects the spec's name. |
| **F-S28e-7** | `recall --trace` "each cited key's access row and record": a range citation of an episode or a chapter can cover hundreds of held keys. | **Bounded:** per range, the trace shows both ends explained (§5.3), the number of held keys inside it, and the first 8 held keys in key order, each with its access row and record; `--trace-all` lifts the cap. |
| **F-S28e-8** | The step's `--with bob` names a seat key; `RecallQuery.counterparts` holds `EntityId`s, and the home knows only the seat's own key → entity. | **Bounded:** `--with` takes an `EntityId` or a name the seat's own name book holds exactly (case-sensitive); anything else is refused by name with the seat's known names listed. |
| **F-S28e-9** | AS28-16 (c) is a start-time refusal, not an inspection; its error lands in S28-c. | **Bounded:** AS28-16 (c) is S28-c's AC-c-3. This PR records it as satisfied there (§8). |
| **F-S28e-10** | MODULE_SPEC §5.3 still names `HomeBusy(path, holder_pid)` (QS28a-4 dropped `holder_pid`). | S28-c's C1 corrects it; if S28-e's C1 runs first, it corrects it. |

### 3.3 What does not exist

No read-only open, no inspector, no report model, no explanation of an id, no byte totals of a home, no export
format or import, no console script, no `HomeNeedsRecovery`, `SeatAlreadyInHome`, `NotASeatExport`.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Decision | Why |
| --- | --- | --- |
| The command line | `argparse` (standard library), as `__main__.py` and `tools/reference.py` | one style in the package; no dependency |
| Read-only access | SQLite's own `mode=ro` URI and its locking | a copy-then-read would read a stale file and double the disk; SQLite already refuses a held file |
| The export file | our positional lines (`memory/home/encoding.py`) with a full-digest codec | the export **is** the home dump's seat section plus its records; a second serializer would be a second canonical form to keep in step (`ARC-101`) |
| `sqlite3` `.backup()` / `VACUUM INTO` as the export | **Declined** (QS28e-1 (c)) | a binary file per seat is not diffable or reviewable, carries every other seat's records unless filtered, and is not a canonical form |
| A TUI or a table library (`rich`, `tabulate`) | **Declined** | the text form is a few aligned columns; `--json` is the machine form; a dependency is a decision (`DEP-*`) for no new capability |

No dependency is added; `uv.lock` does not change (`[project.scripts]` is metadata of the workspace member).

---

## 5. Design

Written with every §14 question answered as recommended; each place a "no" would change is marked **[QS…]**.

### 5.1 Layout and layering

```text
memory/home/home.py         + MemoryHome.open_read_only(path), MemoryHome.totals()         (§5.2, §5.5)
memory/home/errors.py       + HomeNeedsRecovery(path), SeatAlreadyInHome(seat),
                              NotASeatExport(reason), UnsupportedExportFormat(found, wanted)
memory/home/export.py       export_seat(home, key) -> bytes; import_seat(home, data) -> EntityKey   (§5.7)
memory/explain.py           held_counts(view), explain(view, event, perceived), covering(view, key)  (§5.3)
inspect/__init__.py         report(home, request, inputs) -> InspectReport; the request types
inspect/report.py           InspectReport and its parts (typed, strict, frozen)
inspect/views.py            the five views, assembled from memory/explain.py, ForgottenSet, the home
inspect/cli.py              main(argv) -> int: `mineworld-memory inspect | export | import`
inspect/inputs.py           reading --decisions, --config, --perceived (files; this package may open them)
```

- `memory/` keeps its isolation (E-11): `explain.py` and `export.py` take values and a home, never a path; only
  `home.py` connects. `export.py` is a writer of `fact`, `fact_who`, `access`, `access_text` and sits in
  `memory/home/`, where the writers scan allows it.
- `inspect/` is a consumer: it may open files and import `controller.memory_view`, and it **writes nothing to a
  home** except through `import_seat` (scan: no `INSERT`, `UPDATE`, `DELETE`, `CREATE`, `DROP` text in
  `inspect/`).
- `pyproject.toml`: `[project.scripts] mineworld-memory = "mineworld_cognition.inspect.cli:main"`.

### 5.2 Opening for inspection

`MemoryHome.open_read_only(path: Path) -> MemoryHome` (F-S28e-2):

1. `path` absent → `NotAMemoryHome(path)`; nothing is created.
2. `sqlite3.connect(path.as_uri() + "?mode=ro", uri=True, isolation_level=None, timeout=0)`.
3. `SELECT key, value FROM home_meta`: a lock error → `HomeBusy(path)`; SQLite's "hot journal" read-only error →
   `HomeNeedsRecovery(path)`; no `home_meta` → the refusals of `open` (`UnsupportedStoreSchema` for an old
   per-seat store, `NotAMemoryHome`); `schema ≠ SCHEMA_VERSION` → `UnsupportedStoreSchema(found, wanted)`.
4. The returned home has `verify = "off"`, installs no meter triggers (sizes are measured from the dump, in
   full), and every write path (`transaction`, `seat`, `remove_seat`, `reclaim`) raises `ValueError("memory.home:
   the home is open read-only")` before touching SQLite. C2 records the exact SQLite error each platform gives
   for a held home and for a hot journal, and maps only those to `HomeBusy` and `HomeNeedsRecovery`; any other
   error propagates unchanged.

`inspect` uses `open_read_only`; the API takes any open home (a running process's own). I-e-1: the file's bytes,
its `-journal` absence and its modification time are unchanged by any inspection.

### 5.3 Explaining access (`memory/explain.py`, pure over a `SeatView`)

- `held_counts(view) -> HeldCounts`: the seat's held records counted by **how** (from `me`'s `fact_who` row:
  role 2 or 3 → `subject`, role 1 → `participant`, none → `witness`; R-S28e-a-1) × salience; with the cursor,
  the newest key and the watermark.
- `covering(view, key) -> tuple[Unit, ...]`: the seat's surviving units whose ranges hold the key: open or closed
  episode *n* (day, place, record count), chapter *w* (its range), with this seat's predicate in every statement.
- `explain(view, event, perceived: PerceivedSet | None) -> Explanation`, exactly one of:

| Class | When | Carries |
| --- | --- | --- |
| `held` | the seat holds the key | the record (time, kind, place, every `fact_who` row with its role), the access row (gist, salience, mine, importance), how, the covering units, the number of seats of this home holding the record (diagnostic only) |
| `merged` | not held; inside a covering unit; and, when `perceived` is given, on the seat's perceived stream **[QS28e-3]** | the covering units; **no pass attribution [QS28e-2]** |
| `forgotten` | in an explicit forgotten range | `ForgottenRange` (first and last day and policy, passes, forced) (R-S28e-b-1) |
| `forgotten-below-watermark` | at or below the watermark, not held, not covered | the watermark ("no attribution is kept below it": FG-b's 256-range cap, S28-b §5.2) |
| `not-yet-delivered` | above the seat's cursor | the cursor |
| `never-perceived` | none of the above; and, when `perceived` is given, not on the stream | — |
| `covered-not-perceived` | inside a covering unit, `perceived` given, and not on the stream | the units (a range stands for perceived ids only, E-9) |
| `incoherent` | `perceived` given, the id on the stream, and none of `held`, `merged`, `forgotten*` holds | a defect of memory (FGb-1 broken); the report says so and the command exits 3 |

Without `perceived`, an unheld id inside a unit is `merged` with the note "if this seat perceived it", and
`never-perceived` carries "pass `--perceived` to confirm". Every explanation carries the command that shows the
fact itself: `mineworld perceived <world> --save <save> --person <seat> --since <id − 1> --json` (world and save
are the user's; the home does not know them).

### 5.4 The views and their reports (`inspect/`)

Every report is a strict, frozen Pydantic model (`InspectReport` = a discriminated union by `view`), printed as
aligned text, or as JSON with `--json` (`model_dump_json`, keys in model order, UTF-8, `\n` line ends on every
platform). A report is a function of the home's content and the inputs only: no clock, no path except those given.

| View | Request | Report |
| --- | --- | --- |
| `access` | `--seat KEY [--event ID] [--perceived FILE]` | without `--event`: `held_counts`; with it: `explain` (§5.3) |
| `forgotten` | `--seat KEY` | `ForgottenSet.attributed()` in key order (first and last id, first and last day and policy, passes, forced), the watermark, and the seat's pass log (≤ 32 rows, every column, `prose_contradictions` included), newest first |
| `bytes` | `[--seat KEY] [--decisions FILE]` | §5.5 |
| `recall` | `--seat KEY --at T [--with E…] [--words TEXT] [--trace [--trace-all]] [--decisions FILE] [--config FILE]` | §5.6 |
| `scopes` | `--seat KEY [--decisions FILE]` | **only once S28-c has merged [QS28e-4]**: the installed scopes (id, distribution), the seat's stack in order with each parameter and the source position that set it, and the scopes whose tables the home holds that the stack does not name (from S28-d on) |

Exit codes: 0 the report; 2 a refusal (the named error's message on stderr, beginning with its module's tag); 3
an `incoherent` explanation (the report is still printed).

### 5.5 Bytes (`bytes`)

- **Seat** (`--seat`): `size()`, `content_size()`, each view table's bytes and row count (from `row_bytes`), its
  scope (`person` for every table until S28-c's `account` says otherwise **[QS28e-4]**), the five largest rows
  with table, key and bytes (what names a planted oversized row, AS28-16 (b)), the bound, target and trigger of
  `StoreBounds(store_bound_mib)` from `--decisions` (else the 20 MiB default, labelled) and the fill as
  per-mille of the bound; the **physical share** of the seat's held records, Σ over held keys of (the record's
  home-dump bytes ÷ the number of seats holding it), labelled "diagnostic, never charged" (QS28-4).
- **Home** (always): `MemoryHome.totals()` — the `fact` and `fact_who` bytes of `MemoryHome.dump()`, each seat's
  section bytes in key order, the dump's total, the file's bytes (`page_count × page_size`) and its free bytes
  (`freelist_count × page_size`), all read through SQL in `home.py` (no `os` call, E-11).

### 5.6 Recall and the trace (`recall`)

- The query is `RecallQuery(at=T, counterparts=…, words=…)`; `--with` resolves as F-S28e-8.
- The section is `controller.memory_view(view, stack, query, recall_episodes)` — **the function a consult
  calls** — with the stack from `--decisions` (S28-c merged) or `[person]`, and `recall_episodes` from
  `--config` (the seat's override, else `[controller]`, else 8). Before S28-c merges, `memory_view(view, query,
  recall_episodes)`. The report prints `MemorySection.render()` byte for byte.
- `--trace`: after each line, its section key, its scope (`L*` → `person`; another key → that scope), its
  citation token, and per range (F-S28e-7) the two ends explained (§5.3, without `perceived` unless given), the
  number of held keys inside, and up to 8 of them with their access rows and records. Every statement filters by
  this seat.

A wrong gist planted in seat B's access row (AS28-16 (a)) appears in B's trace on the line it produced, under
`held`, with B's access row showing the gist and the shared record showing its envelope; seat A's trace for the
same key shows A's own gist. The cause — B's access row, not the record — is visible in one command.

### 5.7 The seat export (`memory/home/export.py`) [QS28e-1]

**Format 1**, UTF-8, one JSON array per line, the encoder of `encoding.py`, `\n` after every line:

```text
["mineworld-seat-export", 1, "<schema>", "<world_instance>", "<seat key>", "<self entity>"]
["fact", <id>, <at>, "<kind>", "<place>"|null, "<digest, all 64 hex>"]        every held record, key order
["fact_who", <id>, <role>, "<entity>", <role>, "<entity>", …]                  one per held record, ordinals
<the seat's section of MemoryHome.dump(), without its ["seat", …] line>:
  its meta rows (no identity rows), names, access, then the private tables in the view's order
```

- `export_seat(home, key)`: `view_of(key)` (`SeatNotInHome` otherwise); the lines above, from one read
  transaction. Deterministic: a function of the seat's rows and its held records.
- `import_seat(home, data)`: parse every line first (anything malformed → `NotASeatExport(reason)`, naming the
  line number, never its content); header format ≠ 1 → `UnsupportedExportFormat(found, 1)`; schema ≠ the home's →
  `UnsupportedStoreSchema(found, wanted)`; world instance ≠ the home's → `HomeIdentityMismatch("world_instance",
  …)`; the seat key already in the home → `SeatAlreadyInHome(seat)`. Then **one transaction**: each record by
  `world.insert_or_verify` with its full digest (`ConflictingFact` if a held record differs), the `seats` row,
  the access rows in key order with their `access_text` entries, the private rows, the `meta` rows; under
  `verify`, `check()` before the commit. Any error rolls everything back.
- The export reads a home read-only; `import` opens the target with `MemoryHome.open` (it writes).

AS28-11 then holds by construction and is tested: exporting every seat of a home and importing them, in any
order, into a fresh home of the same world instance gives the same `MemoryHome.dump()`, and every
`SeatView.dump()` is the original's (I-S28-2).

### 5.8 The command line (`inspect/cli.py`)

```text
mineworld-memory inspect HOME --seat KEY access [--event ID] [--perceived FILE] [--json]
mineworld-memory inspect HOME --seat KEY forgotten [--json]
mineworld-memory inspect HOME [--seat KEY] bytes [--decisions FILE] [--json]
mineworld-memory inspect HOME --seat KEY recall --at T [--with E]… [--words TEXT] [--trace [--trace-all]]
                         [--decisions FILE] [--config FILE] [--json]
mineworld-memory inspect HOME --seat KEY scopes [--decisions FILE] [--json]        (S28-c merged)
mineworld-memory export HOME --seat KEY [--out FILE]                               (stdout by default)
mineworld-memory import HOME --world-instance ID FILE                              (creates HOME if absent)
```

`--perceived FILE` is `mineworld perceived … --json`'s output for that seat (E-13), read by `inputs.py` into a
`PerceivedSet` of event ids; a file whose person is not the seat is refused by name. `--help` on every command.
The generated CLI reference (`docs/reference/cli.md`) is the Rust binary's and does not list this script; the
package README names it and its `--help` (C9).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-e-1 | **Inspection writes nothing.** Every `inspect` view and `export` leave the home file's bytes unchanged and create no file; a path that is not a home is never created | AC-e-1 |
| I-e-2 (I-S28-1) | No report shows another seat's access row, unit, name or pass; the only cross-seat figures are the record's holder count and the physical share, both labelled diagnostic | AC-e-2, AC-e-7 |
| I-e-3 | `recall` prints exactly the section a consult with the same inputs receives (it calls the consult's function) | AC-e-6 |
| I-e-4 | Every explanation's class agrees with an oracle computed from the seat's perceived stream and its dump (FGb-1, C-7) | AC-e-3 |
| I-e-5 (I-S28-2) | Export then import reproduces every seat view and the home dump, byte for byte | AC-e-8 |
| I-e-6 | A report is a function of the home's content and the inputs: two runs give byte-identical JSON on every platform | AC-e-10 |
| I-e-7 | Memory's storage, recall and retention are unchanged: no schema, dump, policy, bound or pass change | the ported suites; the diff scan |
| I-e-8 (I-S28-10) | No path under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` changes | AC-e-12 |

---

## 7. Test ownership (test rules §26)

| Layer | Owns | Where |
| --- | --- | --- |
| Static | ruff, ruff format, pyright strict over `cognition/lm-controller` | CI `fast`, `python` |
| Unit | the read-only open and its refusals (`test_home_read_only.py`); the explanation classes and held counts against an oracle built from the synthetic world's facts (`test_memory_explain.py`); the export grammar and every import refusal (`test_home_export.py`); the report models' JSON determinism (`test_inspect_reports.py`) | `cognition/lm-controller/tests/` |
| Integration | the command through its console script in a subprocess (`sys.executable -m mineworld_cognition.inspect.cli` and the installed `mineworld-memory`), every view on a home built from `home_streams.world` (`test_inspect_cli.py`); AS28-11 on the 12-seat synthetic home (`test_home_export.py`); AS28-16 (a), (b) (`test_inspect_one_step.py`) | same |
| Golden | `recall` against the frozen golden renders (`tests/golden/s28a_renders.json`): the sha256 of 200 `inspect recall` sections per seat and point equals the golden's | `test_inspect_cli.py` |
| Real binary | C-7's sample of 1 000 ids explained with `--perceived` from `mineworld perceived` on `social_cafe_12` (2 days) (`test_inspect_real.py`, `real_binary`) | same |
| Scans | `test_memory_isolation.py` already covers `explain.py` and `export.py`; new: `inspect/` contains no SQL write text, and only `export.py` among new modules writes the shared tables | same |
| Harness | S28-a's per-PR transparency set and S28-b's shared-home Gate 2, re-run unchanged at C9 | same |
| Gate 1 (real LLM) | **NOT REQUIRED**: no LLM-facing semantics change | — |
| Gate 2 (real lifecycle) | the real-binary C-7 test above; the checkpoint re-run | — |

Oracles are the perceived export, the dumps parsed by the test, and the frozen golden; no expected value is
produced by the inspector itself (test rules §25).

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-e-1 | **Read-only.** For each view and `export`, on a 12-seat file home: the file's sha256 and size are unchanged and no `-journal` file appears; `inspect` of a missing path raises `NotAMemoryHome` and creates nothing; of a home held by another process (the `home_kill.py busy` child) raises `HomeBusy`; of a home left by a SIGKILL inside a transaction (`home_kill.py`'s record seam) raises `HomeNeedsRecovery`, file unchanged | step §5.6 |
| AC-e-2 | **Access counts.** For every seat of the synthetic world (seed 7, 10 days, 12 seats, `consolidate`), `held_counts` equals a recount from the seat's stream and the role rule computed in the test (subject, participant, witness × notable, ambient) | R-S28e-a-1 |
| AC-e-3 | **Every id explained (C-7).** On `social_cafe_12` after 2 days, for each of the 12 seats, 1 000 ids drawn by SplitMix64 from that seat's perceived ids and 200 from ids it never perceived: each is classed exactly as the oracle classes it (held from the dump's `l0`, covered from its episodes and chapter ranges, forgotten from its `forgotten` lines and watermark, perceived from the export); zero `incoherent` | C-7, AS28-18's C-7 |
| AC-e-4 | **Forgotten.** The `forgotten` report equals the seat dump's `forgotten` lines (every attribution column) and its `retention_passes` lines, parsed by the test; `forced` is true exactly for ranges whose first or last policy starts with `host:` | R-S28e-b-1 |
| AC-e-5 | **Bytes.** Per table, the report's bytes equal the sum of that table's line lengths in `SeatView.dump()` (newlines included), their sum equals `size()` and, less `meta`, `content_size()`; the home totals equal the corresponding segments of `MemoryHome.dump()`; file bytes equal `page_count × page_size` | step §5.6 |
| AC-e-6 | **Recall is the consult's.** For every (seat, point) of the golden's `absorb` arm, the sha256 of the 200 rendered `inspect recall` sections equals the golden's; with `--config` setting `recall_episodes = 2`, every report has ≤ 2 L1 lines | I-e-3, C-2 |
| AC-e-7 | **One step to the cause (AS28-16 (a)).** A test writes a wrong gist into seat B's access row for a record A also holds. One `inspect recall --trace --json` for B names, on the line showing that gist: seat B, scope `person`, the key, B's access row with the gist; A's trace for the same key shows A's own gist, and the record's envelope is the same in both | AS28-16 (a) |
| AC-e-8 | **Export and import (AS28-11).** Exporting all 12 seats of the synthetic home and importing them into a fresh home, in key order and in a seeded shuffled order, gives a `MemoryHome.dump()` equal to the original's and every `SeatView.dump()` equal; after import, re-delivering each seat's last 50 facts is a no-op (no `ConflictingFact`); a seat imported into a home that holds other seats leaves their views unchanged; each refusal of §5.7 raises its named error, and a failed import leaves the target's dump unchanged | AS28-11 |
| AC-e-9 | **AS28-16 (b).** A test inserts a 64 KiB name into seat B's `names` table. `inspect bytes --seat B` lists it first among the five largest rows, naming table `names`, its scope (`person`) and its entity | AS28-16 (b) |
| AC-e-10 | **Determinism.** Every view's `--json` output is byte-identical across two runs, and on Linux, macOS and Windows CI for the same synthetic home (the test pins nothing: it compares two runs and checks `\n` line ends) | I-e-6 |
| AC-e-11 | **Scans.** `memory/explain.py` and `memory/home/export.py` pass the isolation scan; `inspect/` contains no `INSERT`, `UPDATE`, `DELETE`, `CREATE` or `DROP`; the writers scan admits `export.py` only | E-11 |
| AC-e-12 | **Checkpoint and diff scan.** S28-a's per-PR transparency set and S28-b's shared-home Gate 2 pass on the final head; nothing under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` changes | step §11.1; I-S28-10 |
| AC-e-13 | **Scopes (conditional, C8).** If S28-c has merged: `inspect scopes` lists `person` and `world` with distribution `mineworld-cognition`, the seat's stack with each parameter's source line and column from `--decisions`; `bytes` attributes every view table by `account` and the per-scope sums equal `content_size()` | QS28e-4 |
| AS28-16 (c) | Satisfied by S28-c's AC-c-3 (F-S28e-9) | — |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason. Targeted validation
per commit; the PR's CI is the one full run.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s28e`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full,
   before editing it.
2. Make the numbered steps, in order. Do not do work that belongs to a later commit.
3. Run the commit's validation commands. Record for each: the command, the test count, the wall time, and PASS /
   FAIL / INCONCLUSIVE (a test not run is not a pass).
4. For each mutation the commit lists: plant it, run the named test, record that it turned red (with the failing
   test's name), restore it from a byte copy (`cmp` clean), run the test again green.
5. Static: `uv run --locked ruff check cognition/lm-controller`; `uv run --locked ruff format --check
   cognition/lm-controller`; `uv run --locked pyright`. Tests one file at a time: `uv run --locked pytest
   cognition/lm-controller/tests/<file>`. **R-LOAD-1:** never the whole member suite, never `cargo test
   --workspace` locally. Markdown: `python3 scripts/check_doc_headings.py`; `python3
   scripts/check_decision_ids.py`.
6. Review: answer the commit's review question in §15 in one or two sentences.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files**; otherwise
   revert it, or stop and record why.
8. Update §15 (ledger row, evidence, findings) and the handoff in the same commit.
9. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` with the message given plus the
   session's attribution line. No heredoc, no `sed -i`, no `python3 -c`.
10. A material stop (§12) is reported, not improvised around.

#### C0 — Base and re-audit (Markdown only)

- **Goal:** fix the base and the state of the PRs this one coordinates with.
- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s28e.md` (new).
- Steps:
  1. Record the base; confirm S28-b merged (`SCHEMA_VERSION = 5`, `ForgottenSet.attributed` exists). Not merged →
     material stop.
  2. Re-anchor E-1 … E-17 on the base; a moved line is bounded, a changed statement is a stop.
  3. Record whether S28-c, S28-d and #262 have merged; that decides C8 (QS28e-4) and §5.2's marker rule.
  4. Write the handoff.
- Validation: the two doc scripts.
- [ ] Implementation. [ ] Validation. [ ] Review: the base satisfies §2.3.
- **Commit message:** `docs(plan): S28-e — open the ledger at the base`

#### C1 — The specifications (Markdown only)

- **Goal:** the specification describes the inspector and the export before they are built (`CLAUDE.md` §2.2).
- **Files:** `docs/MODULE_SPEC.md` (§5.3 "Inspection and degradation"), `docs/DECISIONS.md` (a dated note under
  `ARC-109`).
- Steps: the API is `inspect.report` (F-S28e-6); the read-only open and its three refusals (F-S28e-2); the export
  format (QS28e-1 as ruled); the explanation classes (QS28e-2, QS28e-3 as ruled); `--decisions`, `--config`,
  `--perceived` (F-S28e-5); `HomeBusy(path)` if S28-c's C1 has not corrected it (F-S28e-10).
- Validation: the doc scripts.
- [ ] Implementation. [ ] Validation. [ ] Review: the spec states every rule a tool author needs; nothing
  contradicts §5.
- **Commit message:** `docs(spec): memory inspection and the seat export (S28-e)`

#### C2 — The read-only open and the home's totals

- **Goal:** a home can be read without being changed.
- **Files:** `memory/home/{home.py, errors.py, __init__.py}`, `tests/test_home_read_only.py`.
- Steps: §5.2; `totals()` (§5.5); the four new errors of §5.1 (two used here, two by C6).
- Validation: `test_home_read_only.py` (AC-e-1's open cases; the busy and hot-journal homes come from
  `tests/home_kill.py`'s `busy` and `kill` children). Mutation **M-e-1** (`open_read_only` falls back to `open`
  on a missing path) → `test_a_missing_path_is_not_created` red; **M-e-2** (no `mode=ro`) →
  `test_a_hot_journal_needs_recovery_and_is_left_alone` red (the journal is rolled back: the file changes).
- [ ] Implementation. [ ] Validation. [ ] Review: no write path of the home is reachable from a read-only home.
- **Commit message:** `feat(cognition): open a memory home read-only, with its byte totals (S28-e)`

#### C3 — Explanations

- **Goal:** every id has one class, decided from the seat's own rows (and its perceived stream when given).
- **Files:** `memory/explain.py`, `memory/__init__.py` (exports), `tests/test_memory_explain.py`.
- Steps: §5.3.
- Validation: `test_memory_explain.py`: AC-e-2 on the synthetic world; the classes on hand-built seats (one per
  class, `incoherent` built by deleting a unit in the test). Mutation **M-e-3** (`covering` without `seat = ?`) →
  a two-seat test red (a home-mate's episode covers the id); **M-e-4** (`merged` ignores `perceived`) →
  `covered-not-perceived` test red.
- [ ] Implementation. [ ] Validation. [ ] Review: every statement names this seat; the classes are exhaustive and
  disjoint (one `match` with no default).
- **Commit message:** `feat(cognition): explain why a seat can or cannot recall an event (S28-e)`

#### C4 — The inspector: access, forgotten, bytes; the console script

- **Goal:** three views from the command line and the API.
- **Files:** `inspect/{__init__.py, report.py, views.py, cli.py, inputs.py}`, `cognition/lm-controller/pyproject.toml`
  (`[project.scripts]`), `tests/{test_inspect_reports.py, test_inspect_cli.py}`.
- Steps: §5.1, §5.4 (access, forgotten, bytes), §5.5, §5.8 (those commands).
- Validation: `uv sync --locked` (the script installs); the two files (AC-e-4, AC-e-5, AC-e-10 for these views,
  AC-e-1 for these views). Mutation **M-e-5** (`bytes` counts `meta` in `content_size`) → AC-e-5 red; **M-e-6**
  (`forgotten` drops `prose_contradictions`) → AC-e-4 red.
- [ ] Implementation. [ ] Validation. [ ] Review: `inspect/` opens only the files it is given and writes no SQL.
- **Commit message:** `feat(cognition): mineworld-memory inspect — access, forgotten, bytes (S28-e)`

#### C5 — Recall and the trace

- **Goal:** a human sees exactly what the model sees, and why.
- **Files:** `inspect/{views.py, report.py, cli.py, inputs.py}`, `tests/{test_inspect_cli.py, test_inspect_one_step.py}`.
- Steps: §5.6; `--with` resolution (F-S28e-8); the stack and `recall_episodes` inputs.
- Validation: AC-e-6 (the golden), AC-e-7 (AS28-16 (a)). Mutation **M-e-7** (the trace lists held keys in a range
  without `seat = ?`) → AC-e-7 red (A's gist appears in B's trace); **M-e-8** (`recall` calls `retrieve` instead
  of `memory_view`) → the `recall_episodes = 2` case red.
- [ ] Implementation. [ ] Validation. [ ] Review: the report renders `MemorySection.render()` unaltered.
- **Commit message:** `feat(cognition): inspect recall --trace, every line back to its facts (S28-e)`

#### C6 — Export and import

- **Goal:** a seat moves between homes whole.
- **Files:** `memory/home/{export.py, __init__.py}`, `inspect/cli.py`, `tests/{test_home_export.py,
  test_memory_isolation.py}` (the writers scan admits `export.py`).
- Steps: §5.7; the `export` and `import` commands.
- Validation: AC-e-8. Mutation **M-e-9** (the export writes the short digest) → the re-delivery case red
  (`ConflictingFact`); **M-e-10** (`import_seat` inserts a record without `insert_or_verify`) → the conflicting-
  record case red; **M-e-11** (the export omits role-2 `fact_who` rows) → the home-dump equality red.
- [ ] Implementation. [ ] Validation. [ ] Review: one transaction per import; every refusal before the first write.
- **Commit message:** `feat(cognition): export and import one seat of a memory home (S28-e)`

#### C7 — One step to the cause, on real data

- **Goal:** AS28-16 (b) and C-7 on the real 12-seat stream.
- **Files:** `tests/{test_inspect_one_step.py, test_inspect_real.py}`.
- Steps: AC-e-9; AC-e-3 through the console script with `--perceived` files written by `mineworld perceived`.
- Validation: the two files (record the real-binary wall time). Mutation **M-e-12** (the largest-rows list sorts
  ascending) → AC-e-9 red.
- [ ] Implementation. [ ] Validation. [ ] Review: the oracle of AC-e-3 is the export and the dump, computed in
  the test.
- **Commit message:** `test(cognition): one step to the cause, and every id explained on social-cafe (S28-e)`

#### C8 — Scopes and per-scope bytes (conditional, QS28e-4)

- **Goal:** the memory stack is visible, when it exists.
- **Condition:** S28-c merged at C0 (or merged since; then merge `origin/main` first, recorded). Otherwise
  **N/A**, with the reason "S28-c not merged; R-S28e-c-1 passes to S28-c", and C9 follows.
- **Files:** `inspect/{views.py, report.py, cli.py}`, `tests/test_inspect_cli.py`.
- Steps: the `scopes` view (§5.4); `bytes` per scope by `account`; the trace's scope from `SectionLine.level`.
- Validation: AC-e-13. Mutation **M-e-13** (a parameter's source dropped) → the source test red.
- [ ] Implementation. [ ] Validation. [ ] Review: nothing reads a stack except through `resolve_stack`.
- **Commit message:** `feat(cognition): inspect scopes and per-scope bytes (S28-e)`

#### C9 — Scans, checkpoint, README, close-out

- **Files:** this file (§15), `handoff-s28e.md`, `cognition/lm-controller/README.md` (one line),
  `tests/test_memory_isolation.py` (the `inspect/` write scan).
- Steps: AC-e-11; the per-PR transparency set and S28-b's shared-home Gate 2 through
  `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`, one at a time; AC-e-12's diff scan; push
  `mvp0/pr-s28e`; `gh pr create`; CI on the exact head (Linux, macOS, Windows); repair to green.
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S28-e close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs, and parent edits (owner: the primary session)

- **R-S28c-e-1 (on S28-c, if it merges second; QS28e-4).** S28-c adds the `scopes` view and per-scope bytes to
  `inspect/` as C8 of this design specifies.
- **R-S28d-e-1 (on S28-d).** The anchored scope's tables appear in `bytes` through `account`, its lines in `recall
  --trace` through `SectionLine.level`, and its tables kept for an uninstalled scope in `scopes`; S28-d adds no
  view of its own unless its design shows a question these cannot answer. `export` / `import` carry its
  per-seat rows (and rebuild its shared index rows from the imported records by `index`).
- **R-S28f-e-1 (on S28-f).** If the running process serves inspection, it calls `inspect.report` on its own open
  home; the read-only open is for a stopped home.

| Document | Edit |
| --- | --- |
| `step-28-shared-memory.md` §5.6 | the API name (F-S28e-6); `--decisions`, `--config`, `--perceived` (F-S28e-5); the explanation classes (QS28e-2, QS28e-3 as ruled); the trace cap (F-S28e-7) |
| `step-28-shared-memory.md` §6.6, §15 AS28-11 | the export format (QS28e-1 as ruled) |
| `step-28-shared-memory.md` §11 | the `scopes` view's owner (QS28e-4); AS28-16 (c) in S28-c (F-S28e-9) |
| `overall.md` | S28-e frozen / merged |

---

## 11. R-LOAD-1

Locally: only the commit's test files, **one at a time**, plus ruff, ruff format, pyright and the doc scripts.
Never `cargo test --workspace`, never the whole `cognition/lm-controller` suite. The real-binary file
(`test_inspect_real.py`) and, at C9, the harness and Gate 2 files go through
`/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`, one at a time, with the load average recorded
beside the wall time. The PR's CI on the exact head is the gate and the one full run.

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S28-e — memory inspection (access, forgotten, bytes, recall --trace,
                         scopes) and the seat export / import
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s28e-memory-inspect.md
RELATED / BINDING DOCS:  step-28-shared-memory.md (§§5.5, 5.6, 6.6, 9, 11, 14.1, 15); pr-s28a-memory-home.md
                         (§§5, 10); pr-s28b-retention-on-seat-views.md (§§5.1, 5.3, 10); pr-s28c-memory-scopes.md
                         (§10); docs/MODULE_SPEC.md §5.3; docs/ARCHITECTURE.md §9.3; docs/MVP.md §9 AC-18;
                         docs/DECISIONS.md ARC-98, ARC-101, ARC-109; .structured-coding/standards.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s28e, its own, held by one session
BRANCH:                  mvp0/pr-s28e, created from origin/main
START CONDITION:         S28-b merged AND this design frozen, then the primary session's dispatch  (step §11.1)
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–9, as §14's rulings shape them                     (unresolved until §14
                                                                                          is ruled)
FROZEN INVARIANTS:       step-28 I-S28-1, I-S28-2, I-S28-7, I-S28-10; §6 I-e-1 … I-e-8; schema 5, every dump,
                         FGb-1 … FGb-7 and FG-b2's numbers unchanged; §8 as written
SEQUENCE:                C0 … C9 (§9), C8 conditional
ALLOWED COMMANDS:        uv run --locked *; uv sync --locked; cargo build -p mineworld-cli (the binary for
                         social_cafe_12); target/*/mineworld *; git; gh (never merge); python3 scripts/*;
                         mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or the
                         whole member suite locally (R-LOAD-1); reading any key or .env; calling any model; any
                         record mode for cassettes; editing a golden or a cassette; edits outside APPROVED SCOPE
MATERIAL STOPS:          S28-b not merged; a needed change to a schema, a dump, a policy, an item rule, a bound
                         number or the pass order; a needed change under kernel/, contracts/, server/,
                         persistence/, systems/, worlds/, sdk/; a new dependency or a lock change; AC-e-3 finding
                         an `incoherent` id (a memory defect: report it, do not fix it here); AC-e-8 failing for
                         a reason the export format cannot fix
PLATFORMS:               Linux, macOS, Windows (python job on all three; the read-only and busy cases on each)
VALIDATION BUDGET:       unit files unrestricted locally one at a time; test_inspect_real.py ≤ 90 s on Linux CI;
                         the C9 harness and Gate 2 within S28-a's and S28-b's own budgets; CI: the PR's runs.
                         Gate 1: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s28e.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds and the
                                                         design is frozen          (source: unresolved until
                                                         the primary's dispatch; step §11.1's rule)
                         semantic commits: authorized                     (working rules §14)
                         branch push: authorized                          (working rules §21)
                         PR creation / update: authorized                 (working rules §21)
                         CI repair to review readiness: authorized        (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized — never by the implementation session; only with the operator's
                         explicit authorization after the primary's review (working rules §22)
POST-MERGE SYNC OWNER:   primary: step-28, overall.md; implementation session: this ledger
STOP CONDITION:          S28-e READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-e-1 | SQLite's read-only mode behaves differently on Windows (file sharing, the journal) | AC-e-1 runs on all three CI platforms; `HomeNeedsRecovery` is named, so a platform difference is a refusal, never a silent write |
| R-e-2 | The export becomes a second canonical form that drifts from the home dump | the export's seat section **is** `MemoryHome.dump()`'s seat section, produced by the same function (`_seat_lines`); AC-e-8 compares dumps |
| R-e-3 | An inspector run on a live home's file during play | the running process holds the exclusive lock: `HomeBusy`, by design; the API serves the live case (R-S28f-e-1) |
| R-e-4 | `--perceived` files of 40 000 ids per seat make the real test slow | the set is read once per seat; AC-e-3 samples 1 200 ids per seat; ≤ 90 s budget |
| R-e-5 | A Haiku implementer "fixes" an `incoherent` result by changing memory | the contract makes it a material stop; the report is the deliverable |
| R-e-6 | S28-c changes `memory_view`'s signature after this PR's C5 | QS28e-4's "second merges adapts" rule; C0 records which merged |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

Ruled by the operator on 2026-10-18 in the primary session: every question answered with its recommendation.

| Id | Question | Recommendation |
| --- | --- | --- |
| **QS28e-1 [operator — the export format; step §6.6, AS28-11]** | The step says `export` writes the seat view and `import` splits it into records and private rows. A seat view dump cannot be imported: it carries only a 128-bit digest and only the `fact_who` rows the seat sees (F-S28e-1). The imported home would differ from the original, and the first re-delivery after import would raise `ConflictingFact`. What should an export be? (a) A **seat export**: a header line, then the seat's held records with their full digests and all their `fact_who` rows, then the seat's section of the home dump. Text, positional, deterministic. (b) The seat view plus a sidecar of digests and roles. (c) A binary one-seat SQLite file (`VACUUM INTO` a filtered copy). | **(a).** It is one file, readable and diffable. It reuses the home dump's own lines, so it cannot drift from the canonical form. It makes AS28-11 hold by construction, and the test proves it. (b) is two files that must agree. (c) is not reviewable, not canonical, and needs per-seat filtering of a shared database anyway. The step's "export is byte-identical by I-S28-2" still holds as a test: each imported seat's view equals the original's. |
| **QS28e-2 [operator — "merged … the pass day and policy"]** | No schema records which pass merged a record into a unit (F-S28e-3). Should inspect report the covering unit without a pass, or should memory start recording merge attribution? | **Report the covering unit (episode *n* with its day and place, or chapter *w*), with no pass.** Adding attribution would mean a column per unit and a schema bump (6), touching retention, which S28-b's frozen scope excludes. Forgotten ranges already carry full attribution (S28-b), and those are what `AC-18` asks for: "any forgotten item to the policy that removed it". If a later need appears, it is its own PR. |
| **QS28e-3 [operator — telling "merged" from "never perceived"]** | An unheld id inside one of a seat's unit ranges may have been merged, or may never have been perceived, since a range stands for perceived ids only (F-S28e-4). The home cannot tell which. Add an optional `--perceived FILE` (the seat's `mineworld perceived --json` export) to decide exactly? | **Yes.** Without the file the answer is "merged, if this seat perceived it". With it the answer is exact, and an id that is on the stream but neither held, merged nor forgotten is reported as `incoherent` (a memory defect, exit 3). The file is the seat's own perceived stream, so the inspector learns nothing the person did not perceive. Memory itself never reads it (I-S28-7). C-7's sample of 1 000 uses it. |
| **QS28e-4 [primary — who adds the `scopes` view]** | The `scopes` view and per-scope bytes need S28-c, which is designed in parallel and may merge after this PR. | **Whichever of S28-c and S28-e merges second adds them** (C8 here, R-S28c-e-1 there; the R-P6b-a-1 precedent). Until then every seat-view table is `person`'s and every trace line's scope is `person`, which is exact before S28-d. |

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | (this commit) | [x] base and re-anchor below | [x] doc scripts PASS | [x] the base satisfies §2.3 except D-e-0 |
| C1 | `2f101608` | [x] MODULE_SPEC §5.3, errors row (F-S28e-10), ARC-109 note | [x] doc scripts PASS | [x] nothing contradicts §5 |
| C2 | (this commit) | [x] `open_read_only`, `totals`, `writable`, four errors | [x] `test_home_read_only.py` 8 PASS (1.7 s); `test_home_open.py` 11 PASS; ruff, format, pyright PASS | [x] every write path (`transaction`, `seat`, `remove_seat`, `reclaim`) calls `writable()` first; `measure` keeps no meter |
| C3 … C9 | — | [ ] | [ ] | [ ] |

### Evidence

**C0 (2026-10-19).** Base: `mvp0/pr-s28b` @ `2c13d9c9` (S28-b's implementation commits `70a7c0af`, `e97762fd`
with `origin/main` @ `5b576413` merged in; S28-b not yet merged, D-e-0). `SCHEMA_VERSION = 5`
(`memory/home/schema.py:23`); `ForgottenSet.attributed()` / `range_of()` and `ForgottenRange.forced()` exist
(`memory/retention/forgotten.py:81-115`); `retention_passes.prose_contradictions` exists (`schema.py:77`).
Re-anchored (statements unchanged, lines moved, bounded): E-1 `home.py:87-113`, `_lock` `:391-412`; E-2
`:180-194`, `:239-243`; E-3 `schema.py:104-115`, `encoding.py:36-38`; E-5 `home.py:245-272`, `_seat_lines`
`:352-371`; E-6 `forgotten.py:89-124`; E-9 `ingest.py:140-153`; E-10 `controller/memory_view.py:18-30`,
`controller/seat.py:511-521`, `config.py:445-451` (`controller_for`); E-11 `tests/test_memory_isolation.py:36-90`;
E-12 no `[project.scripts]`; E-14 `selection.py:36-46`, `bound.py:60-90`; E-16 `view.py:255-295`.
S28-c: not merged and not on this base → C8 **N/A** (QS28e-4; R-S28e-c-1 passes to S28-c). S28-d: not merged.
#262 (FU-S28a-1 creation marker): merged (`home.py:449-467`, `_marked`); the read-only open never calls it.

### Mutations

Deferred to milestone (operator SPEED FIRST 2026-10-18): M-e-1 … M-e-13 are not planted per commit.

### Deviations and findings

F-S28e-1 … F-S28e-10 (§3.2), recorded at design time.

- **D-e-0 (bounded, coordinator dispatch 2026-10-19).** The branch is stacked on `mvp0/pr-s28b` (implemented, not
  merged) instead of `origin/main` after S28-b's merge, for speed. Everything this PR needs from S28-b is on the
  base. If S28-b changes before it merges, this branch rebases onto it.
- **D-e-SPEED (operator SPEED FIRST 2026-10-18).** No mutation testing, no independent reviewer, no full suites:
  only ruff, ruff format, pyright and each new or directly affected test file, one at a time. The C9 harness and
  Gate 2 re-runs and CI on the exact head are left to the coordinator's milestone run; this session pushes and
  opens no PR.
- **D-e-2 (bounded, C2).** A kill at a seam (`home_kill.py kill`) leaves **no hot journal**: SQLite keeps the
  uncommitted pages in the dead process's cache, and in exclusive locking mode a committed transaction's journal
  stays on disk with a zeroed header. Such a home reads read-only as its last committed state (tested). The hot
  journal of AC-e-1 comes from a new `home_kill.py hot` child, which dies inside a transaction that has spilled
  pages into the file (a one-page cache). On macOS SQLite reports it as a `SQLITE_READONLY` code (mapped to
  `HomeNeedsRecovery`); a held home as `SQLITE_BUSY` (`HomeBusy`). Linux and Windows are left to CI.
- **D-e-3 (bounded, C2).** `NotAMemoryHome(path, missing=True)` says "does not exist" for a read-only open of a
  missing path; the default message is unchanged.
