# PR S28-f — The controller opens the memory home: `[memory]` in `cognition.toml`, one home per process and world instance, the write lock, pass reports in the decision log, and the cassette criterion AS28-10

## DESIGN FROZEN — 2026-10-19 (operator approved in the primary session: QS28f-1…4 as recommended)

```text
Design revision:        draft 1 (2026-10-19)
Author:                 S28-f / S28-g PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s28fg, branch docs/pr-designs-s28fg,
                        planning base origin/main @ 1308db6d (#329). S28-a (#242), FU-S28a-1 (#262) and
                        P6b (#207) merged; S28-b's and S28-c's designs FROZEN, their implementations NOT merged
Approved by / evidence: the operator, 2026-10-19, in the primary session: QS28f-1…4 as recommended
Implementation base:    origin/main at C0, after S28-c has merged; C0 records the commit
Start condition:        step-28 §11.1: S28-c MERGED (not yet); P6b (#207) merged (yes); this design frozen;
                        the primary session's dispatch
Execution contract:     §12 (filled; endpoint authority included)
Implementer:            Haiku-class (step-28 §11.1), the primary reviewing
Lifecycle:              DESIGN FROZEN 2026-10-19
```

**Effort:** `mvp0` · **Step:** [`step-28-shared-memory.md`](step-28-shared-memory.md) (STEP DESIGN FROZEN
2026-10-14) §§3 (I-S28-2, I-S28-10), 4.5, 5.4 (the `cognition.toml [memory]` rows), 5.5 (`memory.config`),
6.3, 6.5, 6.6, 10, 11 and 11.1 (the S28-f row), 15 (AS28-10, AS28-19 for `SharingModeUnavailable`) ·
**Parent:** [`overall.md`](overall.md) · **Previous PRs:** [`pr-s28a-memory-home.md`](pr-s28a-memory-home.md)
(merged #242; its R-P6b-a-1 and R-S28f-a-1), [`pr-s28c-memory-scopes.md`](pr-s28c-memory-scopes.md) (FROZEN
2026-10-18; its R-S28f-c-1 and QS28c-5), [`pr-s28e-memory-inspect.md`](pr-s28e-memory-inspect.md) (FROZEN; its
R-S28f-e-1), [`pr-s10-p6-lm-controller.md`](pr-s10-p6-lm-controller.md) (P6b, merged #207; its D-P6b-36
follow-up) · **Followed by:** S28-g (scale gates; reads `page_cache_kib` and the home layout) · **Binding
parents:** `CLAUDE.md` §§2, 3, 4 (items 1, 3, 7, 8, 9, 10, 11, 12, 13); `docs/ARCHITECTURE.md` §9.3;
`docs/MODULE_SPEC.md` §5.3 (Placement); `docs/MVP.md` §9 `AC-18`; `docs/DECISIONS.md` `ARC-59`, `ARC-98`,
`ARC-99`, `ARC-109`, `ARC-152`.

**Decision ids.** None new. C1 writes a dated note under `ARC-98` (the placement as built: the layout, the
write lock, and the parameters that were not built) once §14 is ruled.

No hosted model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Scope (step-28 §11, S28-f row).** "P6b wiring: the controller opens the home, `sharing` and `cache` from
  `cognition.toml`, cassette replay criterion AS28-10, the README lines." Implementer Haiku; reviewer the primary.
- **Start (step-28 §11.1).** "S28-c merged. **P6b (#207) merged**." Parallel with S28-d and S28-e.
- **R-P6b-a-1 (S28-a §10, verbatim).** "P6b opens `MemoryHome.open(store/<instance>/memory.sqlite, instance)` and
  `home.seat(identity)` instead of `MemoryStore.open(…/<seat>.sqlite, identity)` (step §10's path); the full
  `sharing` wiring stays S28-f's." S28-a merged first and kept one home per seat at `<store>/<seat>.sqlite`
  (`controller/seat.py:255-256`, "S28-a D-a-1"), so the path move is this PR's.
- **R-S28f-a-1 (S28-a §10, verbatim).** "The record LRU (F-S28a-6)." F-S28a-6: "recommended in S28-f, measured
  first (the SQLite page cache may make it unnecessary — `CLAUDE.md` §4 item 11)."
- **R-S28f-c-1 (S28-c §10, verbatim).** "`verify` checks Σ `account` = `content_size()` per seat (F-S28c-16)."
- **R-S28f-e-1 (S28-e §10, verbatim).** "If the running process serves inspection, it calls `inspect.report` on its
  own open home; the read-only open is for a stopped home."
- **D-P6b-36 follow-up (P6 ledger, verbatim).** "R-P6-FGc-1's second half and §5.12's `PassReport` summaries —
  writing `absorb`'s pass reports (outcome, model and fallback decisions) to the seat's decision log — are not
  written by P6b; S28-f, which rewires the controller's memory home, owns them."
- **QS28c-5 (S28-c, ruled).** "Keep both [`person.sections.L1.lines` and `recall_episodes`], and apply the
  operator's cap after the world's … Retiring `recall_episodes` would change P6's configuration contract, which
  belongs to S28-f's wiring review."
- **The parameters (step §5.4, MODULE_SPEC §5.3 Placement).** `sharing` `none` | `home` (default `home`;
  `service` refused with `SharingModeUnavailable`), `home` (path template, default `store/<instance>/memory.sqlite`),
  `cache_records` (0 … 1 000 000, default 4 096), `page_cache_kib` (256 … 1 048 576, default 8 192), `verify`
  (`off` | `pass` | `every-commit`, default `off`).
- **AS28-10 (step §15, verbatim).** "P6b's recorded cassettes (recorded with v2, or with `sharing: none`) replay
  under `home` with zero `CassetteMiss` and identical decision logs."
- **Rulings not re-asked.** QS28-2 (per process in MVP-0), QS28-8 (refuse and rebuild, no importer), QS28-10
  (placement in `cognition.toml`).
- **Integration checkpoint (step-28 §11.1).** "Every PR ends with the transparency harness green (12 seats,
  `none` vs `home`). From S28-b on, it also runs FG-b's real-binary Gate 2 scenario on seat views."
- **Charter (2026-10-18/19).** Speed first; merges by the coordinator once `fast` is green; no world-specific
  hard-coding; every criterion names a second world or a generated variant.

---

## 1. Goal

Today each `lm` seat opens a memory home of its own, at `<store>/<seat>.sqlite`. The home code from S28-a can
already hold many seats in one file. Nothing in the controller uses that. After this PR:

1. **The seats of one process share one home** for each world instance, at `<store>/<instance>/memory.sqlite`.
   This is `sharing = "home"`, the default. `sharing = "none"` keeps one home per seat, at
   `<store>/<instance>/<seat>.sqlite`. Nothing a seat recalls depends on the choice (I-S28-2).
2. **The operator sets placement in `cognition.toml`** under a new `[memory]` table: `sharing`, `page_cache_kib`
   and `verify`. Each field is typed, bounded, defaulted and generated into `docs/reference/cognition-toml.md`.
   `service` is refused by name.
3. **One home, many seats, one event loop: writes never overlap.** An assisted retention pass awaits a model
   inside its transaction (`retention/host.py:104-110`). A second seat's write in that window would fail or join
   the wrong transaction. A per-home write lock serializes every write call.
4. **Each retention pass is written to the seat's decision log** as one `pass` line (the D-P6b-36 follow-up).
5. **AS28-10 holds and is tested:** a cassette recorded under `none`, or before S28, replays under `home` with
   zero `CassetteMiss` and the same decision log.

No retention policy, item rule, bound number, pass order, schema, dump or meter changes. Nothing changes under
`kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/` or `sdk/` (I-S28-10).

---

## 2. Scope

### 2.1 In scope

1. **The `[memory]` table** (`memory/config.py`, new): `MemoryConfig` (`sharing`, `page_cache_kib`, `verify`),
   `SharingModeUnavailable`; `CognitionConfig.memory` (`config.py`); the load-time refusal of `service` (§5.2).
2. **The page cache** (`memory/home/home.py`): `MemoryHome.open(…, page_cache_kib=8192)` sets SQLite's
   `cache_size` (§5.3).
3. **The process's homes** (`controller/homes.py`, new): `SeatHomes`, `HeldSeat`, the layout, the per-home write
   lock, reference-counted close (§5.4).
4. **The seat runner** (`controller/seat.py`): the seat is opened through `SeatHomes` after the welcome; every
   `absorb` runs under the home's write lock; the `verify` scope-account check (R-S28f-c-1) (§5.5).
5. **`serve` and `main`** (`__main__.py`): one `SeatHomes` per process, closed when serving ends; a one-line
   notice for a memory file of the old layout (§5.6).
6. **Pass lines** (`controller/log.py`): `PassLine` (`kind: "pass"`), written for every `PassReport` (§5.7).
7. **Tests** of §7, including AS28-10 (`test_home_cassettes.py`) and the moved paths in the real-binary tests
   (`test_milestone_d_100_days.py`, `tools/cli/tests/milestone_d.rs`).
8. **Reference and docs:** `tools/reference.py` example, `docs/reference/cognition-toml.md` and its schema
   regenerated; `docs/MODULE_SPEC.md` §5.3 Placement corrected to what is built; `cognition/lm-controller/README.md`
   memory and controller lines; a dated note under `ARC-98`.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| The per-home LRU of decoded fact records (`cache_records`, step §4.5; R-S28f-a-1) | **not built** (QS28f-1 recommended); follow-up FU-S28f-1 if S28-g's nightly measurements ask for it |
| A path template for the home (`home`, step §5.4) | **not built** (QS28f-2 recommended): the layout is fixed (§5.4) |
| Serving `InspectReport` from the running process (R-S28f-e-1) | **not done**: no endpoint exists and none is added; R-S28f-e-1 is conditional ("if"); recorded as not applicable |
| Retiring `[controller] recall_episodes` (QS28c-5) | **not done** (QS28f-3 recommended: keep it) |
| Cross-process sharing, `service`, the cold tier | S28-h (MVP-1) |
| The scale gates and `dump_lines()` | S28-g |
| A migration of `<store>/<seat>.sqlite` files into the new layout | never (QS28-8: memory is rebuilt from the perceived stream) |
| Any change to a policy, an item rule, a bound number, the pass order, schema, a dump or the meter | never in this PR (material stop, §12) |
| Any change under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` | never (I-S28-10) |

### 2.3 Dependencies and the start condition

- **Required: S28-c merged.** This PR calls S28-c's `memory_view(store, stack, query, recall_episodes) -> Recalled`
  and its `resolve(…, seat=…)`, and R-S28f-c-1 needs `PersonScope.account`. C0 re-anchors every line of §3 on
  the merged code and records the names S28-c actually shipped.
- **Merged:** S28-a (#242), FU-S28a-1 (#262, the creation marker), P6b (#207), P7 (#244, the milestone D tests),
  25b (`tools/reference.py`).
- **Not required:** S28-b's merge is implied by S28-c's start condition. S28-d and S28-e may merge before or
  after this PR; neither touches `controller/seat.py`'s store opening (C0 checks).

---

## 3. Audit (`origin/main @ 1308db6d`)

Line numbers are on `1308db6d`. C0 re-anchors each on the base.

### 3.1 Source anchors (under `cognition/lm-controller/src/mineworld_cognition/` unless a path says otherwise)

| # | Fact | Where |
| --- | --- | --- |
| A-1 | `SeatRunner.__init__` sets `self._store_path = store_dir / f"{key}.sqlite"`, `_resume_path`, and the decision log `store_dir / f"{key}.decisions.jsonl"`. | `controller/seat.py:179-182` |
| A-2 | `_open_store(seat)` is synchronous. On first use it builds `StoreIdentity(world_instance=seat.instance, seat=key, self_entity=seat.observer)`, creates `store_dir`, opens `MemoryHome.open(self._store_path, seat.instance)` and takes `home.seat(identity)`. Its docstring: "One home shared by a process's seats is S28-f's `sharing` wiring." It is called from `_join_and_serve` (after the seat check), `_serve`, and `_consult_once`. | `controller/seat.py:254-266`, `:229`, `:286`, `:503` |
| A-3 | The world instance is known only from a welcome; a process's first join presents no cursor (D-P6b-15). | `controller/seat.py:117-126` |
| A-4 | `_ingest` calls `await absorb(store, batch.events, batch.through, self._retention)` and discards the `AbsorbReport`. | `controller/seat.py:319-331` |
| A-5 | `_close` writes the counters line, then closes the home. | `controller/seat.py:644-653` |
| A-6 | `serve()` builds one `SeatRunner` per seat on one event loop (`asyncio.TaskGroup`). | `__main__.py:80-110` |
| A-7 | **A pass awaits inside its transaction.** `run_pass` opens `store.transaction(retention_pass=True)` and then awaits `_decide`, which for an assisted policy awaits the `summarize` gateway; "every await comes before the first write". `absorb` is async for this reason. | `memory/retention/host.py:99-110`; `memory/retention/absorb.py:16-18` |
| A-8 | **A transaction is the home's, not the seat's.** `MemoryHome.transaction()` runs `BEGIN IMMEDIATE … COMMIT` on the one connection. `MemoryHome.seat()` creates a seat row inside the open transaction when `db.in_transaction` is true. | `memory/home/home.py:121-135`, `:155-178` |
| A-9 | `MemoryHome.open(path, world_instance, *, verify="off")`; `_lock` sets `locking_mode = EXCLUSIVE` and `auto_vacuum`; no `cache_size` is set anywhere in `memory/`. | `memory/home/home.py:88-113`, `:391-412` |
| A-10 | `Verify = Literal["off", "pass", "every-commit"]`; `check()` recomputes I-S28-4 and every measured seat's meter. | `memory/home/home.py:51-53`, `:141-152` |
| A-11 | `PassReport` fields: `day`, `policy`, `outcome`, `kept`, `merged`, `dropped`, `forced`, `bytes_before`, `bytes_after`, `forgotten_ranges`, `model_decisions`, `fallback_decisions`, `cause`, `trigger`. `AbsorbReport.passes` holds one per pass run. | `memory/retention/host.py:57-80`; `memory/retention/absorb.py:37-42` |
| A-12 | `DecisionLog` writes `decision` and `counters` lines; `read_decisions` keeps `kind == "decision"` lines only; `__main__._status` reads the last `counters` line. | `controller/log.py:145-176`; `__main__.py:140-152` |
| A-13 | `CognitionConfig` is strict, frozen, `extra="forbid"`: an unknown `[memory]` table is refused today. `load()` resolves `store` against the file and refuses a store inside a world. | `config.py:454-560`, `:597-628` |
| A-14 | `tools/reference.py` writes `cognition-toml.md` from `CognitionConfig`, refusing an undescribed field and any example the loader refuses. | `cognition/lm-controller/tools/reference.py:32-130`, `:330-372` |
| A-15 | The isolation scan allows `memory/` to import only `memory`, `compress`, `forgetting` and three named modules, and forbids `open(`, `os.`, `Path(`, `subprocess`. | `tests/test_memory_isolation.py:20-66` |
| A-16 | **Readers of the per-seat path.** `test_milestone_d_100_days.py:182` opens `store_dir / "alice.sqlite"` after a live run; `tools/cli/tests/milestone_d.rs:432-434` (`store_rows`) opens `store.join("alice.sqlite")`. Every other test opens a home at a `tmp_path` of its own. No guide page names the file (`git grep` over `docs/guide`, `docs/reference`). | as named |
| A-17 | **Cassettes.** `tests/cassettes/lm-summarise.jsonl` (FG-c) replays one seat's 12-day stream to the dump digest `DIGEST`; `tests/cassettes/milestone_d.jsonl` replays from the logged inputs (`replay_log`), not through a home; `tests/cassettes/example.jsonl` is a recorder fixture. | `tests/test_lm_summarise_replay.py:34-110`; `tests/test_milestone_d_replay.py:1-40`; `tests/test_record_replay.py:186-200` |
| A-18 | The scripted fake server (`scripted_seat.py`) drives a `SeatRunner` with scripted links and a fake clock; its welcome lists the seats `alice` and `wanderer`, and takes an `observer`. `RecordingBackend` wraps any `ModelBackend`, a scripted one included. | `tests/scripted_seat.py:18-60`; `tests/test_controller_seat.py:118-138`; `record.py:256-323` |
| A-19 | `SeatRunner(` is constructed in `__main__.serve` and at three test sites. | `tests/test_controller_seat.py:127`, `:551`, `:608` |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S28f-1** | Sharing one connection across seats breaks under an assisted pass (A-7, A-8). While seat A awaits its model inside `BEGIN IMMEDIATE`, seat B's ingest calls `BEGIN IMMEDIATE` again (SQLite: "cannot start a transaction within a transaction"), and B's first `home.seat()` would write its seat row into A's transaction. | **Bounded:** one `asyncio.Lock` per home (`SeatHomes`). Every call that can write (`absorb`, the first `home.seat`) runs under it. Reads (`memory_view`, `detect`, `NameBook.load`) need no lock: no write is pending while a pass awaits (A-7), so a read sees committed rows only. |
| **F-S28f-2** | The step's default path `store/<instance>/memory.sqlite` needs the instance, which only a welcome gives (A-3). | **Bounded:** the home is opened when the first seat of that instance is welcomed; later seats of the instance reuse it. |
| **F-S28f-3** | The step names `home` as a "path template". Nothing in the code or the specs says which placeholders it has, how `none` names per-seat files, or what stops a template from escaping the store directory. | **§14 QS28f-2.** |
| **F-S28f-4** | No record LRU exists, and nothing has measured a need for one (R-S28f-a-1, F-S28a-6). | **§14 QS28f-1.** |
| **F-S28f-5** | R-S28f-c-1's check needs each scope's `account`; the home does not know the stack (S28-c F-S28c-16). | **Bounded:** the seat runner checks it after every `absorb` that ran a pass, when `verify` is not `off`, and raises `HolderInvariantBroken(seat=…)` on a mismatch (§5.5). |
| **F-S28f-6** | An operator who ran S28-a's controller has `<store>/<seat>.sqlite` files. The new layout never reads them, and a first run rebuilds memory from the perceived stream anyway (A-3). | **Bounded:** `main` prints one notice per such file and leaves it in place (QS28-8: no importer; the operator's file is not deleted). |
| **F-S28f-7** | `SharingModeUnavailable` belongs to `memory.config` (step §5.5). A Pydantic validator that raises it would be wrapped in a `ValidationError`, losing its type. | **Bounded:** `MemoryConfig.sharing` accepts `service` as a value; `config.load()` then raises `ConfigError` **from** `SharingModeUnavailable("service")`, so the type and field survive as `__cause__` (AS28-19). |
| **F-S28f-8** | A `PassReport` carries no seat; the decision log is already per seat. | **Bounded:** the `pass` line carries `seat` (as `decision` lines do) and every `PassReport` field. |

### 3.3 What does not exist

No `[memory]` table, no `SharingModeUnavailable`, no `cache_size` pragma, no process-level holder of homes, no
write lock, no `pass` line in the decision log, no test that replays a cassette under a shared home.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Decision | Why |
| --- | --- | --- |
| Serializing writes on one connection from many tasks | `asyncio.Lock` (stdlib) | one loop, one connection; a thread lock or a second connection would add nothing |
| The page cache | SQLite's own `PRAGMA cache_size` (negative value = KiB) | the documented knob; no wrapper |
| A record LRU | `functools.lru_cache` would be the tool if one is built | not built (QS28f-1) |
| A connection pool or an ORM | **Declined** | one writer by design (`ARC-98`); a pool contradicts `HomeBusy` |

No dependency is added; `uv.lock` and `Cargo.lock` do not change.

---

## 5. Design

Written with every §14 question answered as recommended; each place a "no" would change is marked **[QS…]**.

### 5.1 Module layout

```text
memory/config.py            MemoryConfig, Sharing, SharingModeUnavailable, PAGE_CACHE_KIB      (§5.2)
memory/home/home.py         MemoryHome.open(…, page_cache_kib=PAGE_CACHE_KIB)                  (§5.3)
config.py                   CognitionConfig.memory; load() refuses `service`                   (§5.2)
controller/homes.py         SeatHomes, HeldSeat, home_path()                                   (§5.4)
controller/seat.py          the seat opened through SeatHomes; absorb under the lock;
                            the verify account check; pass lines                               (§5.5, §5.7)
controller/log.py           PassLine (kind "pass"), DecisionLog.passes(seat, reports)          (§5.7)
__main__.py                 serve(…) builds and closes one SeatHomes; the old-layout notice    (§5.6)
```

### 5.2 The `[memory]` table (`memory/config.py`, pyright strict)

```python
Sharing = Literal["none", "home", "service"]
PAGE_CACHE_KIB = 8192

class MemoryConfig(CognitionModel):
    """The `[memory]` table: where seats' memory is stored. Placement never changes what a seat recalls."""
    sharing: Sharing = Field(default="home", description=…)
    page_cache_kib: Annotated[int, Field(ge=256, le=1_048_576)] = Field(default=PAGE_CACHE_KIB, description=…)
    verify: Literal["off", "pass", "every-commit"] = Field(default="off", description=…)

class SharingModeUnavailable(ValueError):
    """memory.config: sharing = "service" is MVP-1's cross-process sharing (step-28 §6.6); refused before it."""
    def __init__(self, mode: str) -> None: ...      # message begins "memory.config:"; field `mode`
```

- Descriptions (plain, one sentence each, the reference refuses an empty one):
  - `sharing`: "How the seats of this process store memory. `home`: one file per world instance for all seats,
    storing each perceived fact once. `none`: one file per seat. `service` is not available yet. Either choice
    gives every seat exactly the same memory."
  - `page_cache_kib`: "The memory file's page cache in KiB, from 256 to 1 048 576 (SQLite `cache_size`). More
    makes recall faster and uses more RAM."
  - `verify`: "Debug checks on the memory file. `off`: none. `pass`: before each forgetting pass commits.
    `every-commit`: before every commit. A failed check stops the seat."
- `CognitionConfig.memory: MemoryConfig = Field(default_factory=MemoryConfig, description="The `[memory]` table:
  where seats' memory is stored.")`.
- `config.load()`: after validation, `if config.memory.sharing == "service": raise ConfigError(f"{path}:
  {error}") from SharingModeUnavailable("service")` (F-S28f-7). `main` prints it and exits 2, as every
  configuration error.
- **[QS28f-1]** No `cache_records` field. **[QS28f-2]** No `home` field.
- `memory/config.py` imports only `pydantic` and `mineworld_cognition.backend.model` (`CognitionModel`), so the
  isolation scan (A-15) stays green unchanged.

### 5.3 The page cache (`memory/home/home.py`)

`MemoryHome.open(path, world_instance, *, verify="off", page_cache_kib=PAGE_CACHE_KIB)`. After `_lock`, it runs
`PRAGMA cache_size = -{page_cache_kib}` (SQLite: a negative value is KiB). The value is validated by
`MemoryConfig`; `open` refuses a value outside 256 … 1 048 576 with `ValueError("memory.home: page_cache_kib …")`
for direct callers. Nothing else in `open` changes; no dump changes.

### 5.4 The process's homes (`controller/homes.py`)

```python
@dataclass(frozen=True)
class HeldSeat:
    view: SeatView
    lock: asyncio.Lock          # the home's write lock, shared by every seat of the home

def home_path(store: Path, instance: WorldInstanceId, seat: EntityKey, sharing: Sharing) -> Path:
    """`<store>/<instance>/memory.sqlite` under `home`; `<store>/<instance>/<seat>.sqlite` under `none`."""

class SeatHomes:
    """The memory homes of one cognition process (step-28 §6.6, MVP-0): one per world instance under
    `sharing = "home"`, one per seat under `"none"`. The only opener and closer of homes in the controller."""
    def __init__(self, store: Path, memory: MemoryConfig) -> None: ...
    async def hold(self, identity: StoreIdentity) -> HeldSeat: ...
    def release(self, seat: EntityKey) -> None: ...
    def close(self) -> None: ...
```

- **Layout [QS28f-2].** Fixed: `<store>/<instance>/memory.sqlite` (home) or `<store>/<instance>/<seat>.sqlite`
  (none). `<instance>` is the welcome's 32-hex instance and `<seat>` a seat key: both are validated identifiers,
  so neither can name another directory. Resume files and decision logs stay at `<store>/<seat>.resume` and
  `<store>/<seat>.decisions.jsonl` (step §10: "they are not memory").
- **`hold(identity)`**:
  1. path = `home_path(...)`; key the open homes by path;
  2. if no home is open at that path, `mkdir(parents=True, exist_ok=True)` its directory and `MemoryHome.open(path,
     identity.world_instance, verify=memory.verify, page_cache_kib=memory.page_cache_kib)`, with a new
     `asyncio.Lock`; a `HomeBusy`, `HomeIdentityMismatch`, `UnsupportedStoreSchema` or `NotAMemoryHome`
     propagates unchanged (the seat stops, as today);
  3. `async with lock: view = home.seat(identity)` (F-S28f-1: a seat row is never written into another seat's
     open transaction);
  4. count the seat as a holder of that home; return `HeldSeat(view, lock)`.
- **`release(seat)`** drops the seat from its home's holders; when none remain, the home is closed and forgotten.
  A seat released twice is a no-op. **`close()`** closes every home still open (called once by `serve`).
- A seat key held twice at one time is a defect: `hold` raises `ValueError("controller.homes: seat <k> is
  already held")`.

### 5.5 The seat runner (`controller/seat.py`)

- `SeatRunner.__init__(…, store_dir: Path, homes: SeatHomes, …)`: the new required keyword `homes`.
  `_store_path` and `self._home` are removed.
- `_open_store` becomes `async def _hold_store(self, seat) -> SeatView`: on first use it calls `await
  self._homes.hold(identity)`, keeps `self._held`, sets `self._store` and `self._cursor.store`. It is awaited once
  in `_join_and_serve`, at today's line 229. `_serve` and `_consult_once` read `self._store` through a small sync
  accessor `_view()` that asserts it is set (it always is after `_join_and_serve`).
- `_ingest`: `async with self._held.lock: report = await absorb(store, …)`; then, still under the lock, (a)
  `self._log.passes(self._key, report.passes)` (§5.7) and (b) the account check below. The trigger detection that
  follows stays outside the lock.
- **The account check (R-S28f-c-1).** When `config.memory.verify != "off"` and `report.passes` is not empty:
  `Σ bytes over self._stack's scopes' account(view)` must equal `view.content_size()`; otherwise raise
  `HolderInvariantBroken(seat=self._key)` (the home's error, `memory/home/errors.py`). The stack is S28-c's
  `self._stack`; the exact call is re-anchored at C0.
- `_close`: writes the counters line, then `self._homes.release(self._key)` (never `home.close()` itself).

### 5.6 `serve` and `main` (`__main__.py`)

- `serve(config, seats, *, router, invite, url, store_dir)` builds `homes = SeatHomes(store_dir, config.memory)`,
  passes it to every runner, and calls `homes.close()` in a `finally` after the task group ends.
- `main`, before serving: for each selected seat, if `store_dir / f"{seat.key}.sqlite"` exists, print
  `[cognition] {seat}: {path} is a memory file of the earlier layout; it is no longer read and may be deleted
  (memory is rebuilt from the perceived stream)` to stderr. Nothing is moved or deleted (F-S28f-6).

### 5.7 Pass lines (`controller/log.py`)

```python
class PassLine(CognitionModel):
    kind: Literal["pass"] = "pass"
    seat: str
    day: int; policy: str; outcome: str; cause: str; trigger: str
    kept: int; merged: int; dropped: int; forced: int
    bytes_before: int; bytes_after: int; forgotten_ranges: int
    model_decisions: int; fallback_decisions: int
```

- `DecisionLog.passes(seat, reports)` writes one line per `PassReport`, in order, with the same rotation rule.
- `LogLine` gains `PassLine` in its discriminated union. `read_decisions` still returns `decision` lines only;
  `_status` still reads the last `counters` line. No line holds a gist, a name or a provider concept (I-P6-8).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-f-1 (I-S28-2) | Every seat's memory sections, and so every `CompletionRequest` and decision record, are byte-identical under `sharing = "home"` and `"none"` | AC-f-3, AC-f-4 |
| I-f-2 | No write call of one seat starts while another seat's transaction on the same home is open | AC-f-5 |
| I-f-3 | The controller opens and closes homes only through `SeatHomes`; one path is open at most once per process | AC-f-2, AC-f-6 (scan) |
| I-f-4 | Resume files and decision logs keep their paths; only memory files move | AC-f-2 |
| I-f-5 | The committed cassettes and goldens are unchanged, and each replays with zero `CassetteMiss` | AC-f-3, AC-f-8 |
| I-f-6 | Retention, every schema, every dump and the meter are unchanged | the S28-a/S28-b/S28-c suites; diff scan |
| I-f-7 (I-S28-10) | No path under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` changes | AC-f-9 |

---

## 7. Test ownership (test rules §26)

| Layer | Owns | Where |
| --- | --- | --- |
| Static | ruff, ruff format, pyright strict; `cargo fmt`, `cargo clippy -D warnings` on `mineworld-cli`'s tests | CI `fast` |
| Unit | `MemoryConfig` bounds and defaults through `load()`; `SharingModeUnavailable` as `__cause__`; `home_path`; `SeatHomes` reuse, release, close, double hold; `page_cache_kib` reaches `PRAGMA cache_size` | `tests/test_controller_homes.py` (new) |
| Unit (concurrency) | F-S28f-1: with a gateway that blocks inside an assisted pass of seat A, seat B's `absorb` waits for the lock and then succeeds; without the lock (mutation) B fails | same |
| Integration (scripted server) | two runners (`alice`, `wanderer`) in one process share one file under `home`, two files under `none`; the same decision logs both ways; `pass` lines present | `tests/test_home_cassettes.py` (new) |
| Replay (AS28-10) | (a) the committed `lm-summarise.jsonl` replays to `DIGEST` for its seat inside a shared home with peers; (b) a cassette recorded by the two-runner run under `none` replays under `home` with zero misses and equal logs | same |
| Real binary | the moved paths: `test_milestone_d_100_days.py`; `tools/cli/tests/milestone_d.rs` (the kill/resume reads the shared home) | existing files |
| Reference | `test_reference.py` drift and example checks | existing |
| Harness | S28-a's per-PR transparency set and S28-b's shared-home Gate 2, re-run unchanged at C7 | existing |
| Gate 1 (real LLM) | **NOT REQUIRED**: no LLM-facing byte changes (I-f-1, proven offline) | — |
| Gate 2 (real lifecycle) | the milestone D real-binary test (a real server, a real controller process, SIGKILL, resume) on the new layout | `tools/cli/tests/milestone_d.rs` |

No expected value comes from the code under test: the transparency claims compare two runs of the shipped code
that differ only in `sharing`, and the replay claim uses the frozen `DIGEST` and the committed cassette.

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-f-1 | **Configuration.** A file without `[memory]` loads with `sharing == "home"`, `page_cache_kib == 8192`, `verify == "off"`. `page_cache_kib = 255` and `1048577` are refused naming `memory.page_cache_kib`. `sharing = "service"` raises `ConfigError` whose `__cause__` is `SharingModeUnavailable` with `mode == "service"` and a message beginning `memory.config:`. An unknown key in `[memory]` is refused by name | §5.4, AS28-19 |
| AC-f-2 | **Layout.** Two runners of one process (scripted server, one instance) under `home` leave exactly one memory file, `store/<instance>/memory.sqlite`, whose `seats` table holds both keys; under `none` they leave `store/<instance>/alice.sqlite` and `store/<instance>/wanderer.sqlite`; both ways `store/<seat>.decisions.jsonl` exist and no `store/<seat>.sqlite` is created | §5.4, §10 |
| AC-f-3 | **AS28-10 (a).** The committed `tests/cassettes/lm-summarise.jsonl` replays with zero `CassetteMiss` for its seat inside a home that also holds two peer seats ingesting overlapping facts, interleaved frame by frame; the seat's `SeatView.dump()` sha256 equals `test_lm_summarise_replay.DIGEST` | AS28-10 |
| AC-f-4 | **AS28-10 (b).** The two-runner scripted run, recorded through `RecordingBackend` under `none`, replays through `ReplayBackend` under `home` with zero `CassetteMiss`; each seat's `decision` lines are equal (`model_dump_json`) between the two runs; the `pass` lines are equal too | AS28-10, I-S28-2 |
| AC-f-5 | **The write lock.** In one home, seat A's assisted pass is held at its model call (a gateway that waits on an `asyncio.Event`); seat B's `absorb` started meanwhile completes only after A's pass commits, and both seats' dumps equal the same calls run one after the other. Mutation **M-f-1** (no lock) → B raises `sqlite3.OperationalError` or the dumps differ | F-S28f-1 |
| AC-f-6 | **One opener.** No `MemoryHome.open(` under `controller/` outside `homes.py` (source scan); holding one seat twice raises; the home closes when its last seat is released and not before | §5.4 |
| AC-f-7 | **Pass lines.** Every `PassReport` an `absorb` returned in the scripted run appears once, in order, as a `pass` line of that seat's log with every field equal; `read_decisions` and `_status` are unchanged on a log holding them | §5.7 |
| AC-f-8 | **Real binary, moved path.** `test_milestone_d_100_days.py` and `milestone_d.rs` pass reading `store/<instance>/memory.sqlite`; `milestone_d.rs`'s creation-marker and cursor checks across the SIGKILL hold unchanged; `test_milestone_d_replay.py` passes on the unchanged fixture | §10, I-f-5 |
| AC-f-9 | **Diff scan.** `git diff --name-only origin/main...HEAD` lists nothing under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/`, `tests/cassettes/` or `tests/golden/` | I-S28-10, I-f-5 |
| AC-f-10 | **Checkpoint.** S28-a's per-PR transparency files and S28-b's shared-home Gate 2 pass on the final head | step §11.1 |
| AC-f-11 | **Second world.** AC-f-4's scenario uses the scripted server's stream, not the default town; AC-f-8's real-binary run uses `social-cafe`. The layout code names no world, seat or place (source scan of `homes.py`) | charter |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason. Targeted validation
per commit; the PR's CI is the one full run.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s28f`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full,
   before editing it.
2. Make the numbered steps, in order. Do not do work that belongs to a later commit.
3. Run the commit's validation commands. Record for each: the command, the test count, the wall time, and PASS /
   FAIL / INCONCLUSIVE (a test not run is not a pass).
4. For each mutation the commit lists: copy the file (`cp f f.orig`), plant the change, run the named test,
   record that it turned red (the failing test's name), restore with `mv f.orig f`, check `git diff --stat` is
   clean for that file, run the test again green.
5. Static: `uv run --locked ruff check cognition/lm-controller`; `uv run --locked ruff format --check
   cognition/lm-controller`; `uv run --locked pyright cognition/lm-controller`. Python tests one file at a time:
   `uv run --locked pytest cognition/lm-controller/tests/<file>`. Rust (C6 only): `cargo fmt -p mineworld-cli
   --check`, `cargo clippy -p mineworld-cli --tests -- -D warnings`. **R-LOAD-1:** never the whole member suite,
   never `cargo test --workspace` locally. Markdown: `python3 scripts/check_doc_headings.py`; `python3
   scripts/check_decision_ids.py`.
6. Review: answer the commit's review question in one or two sentences, in §15.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files**; otherwise
   revert it, or stop and record why.
8. Update §15 (ledger row, evidence, findings) in the same commit.
9. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` with the message given plus a blank
   line and `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>` (pass it as a second `-m`). No
   heredoc, no `sed -i`, no `python3 -c`.
10. A material stop (§12) is reported, not improvised around.

#### C0 — Base and re-audit (Markdown only)

- **Goal:** fix the base and re-anchor §3 on code that has S28-c in it.
- **Files:** this file (§15).
- Steps:
  1. Record the base SHA. Confirm S28-c merged: `git log --oneline origin/main | grep -i "s28-c\|s28c"` shows its
     merge, and `memory/scopes/host.py` exists. Not merged → material stop.
  2. Re-anchor A-1 … A-19. A moved line is bounded; a changed statement is a stop. Record S28-c's shipped names
     for `memory_view`, `Recalled`, `resolve(…, seat=…)`, `resolve_stack`, `SeatRunner._stack` and the
     `account` call (§5.5).
  3. Record whether S28-d and S28-e have merged, and whether either touched `controller/seat.py`'s store
     opening (`git log -p origin/main -- cognition/lm-controller/src/mineworld_cognition/controller/seat.py`).
- Validation: the two doc scripts.
- [ ] Implementation. [ ] Validation. [ ] Review: the base satisfies §2.3.
- **Commit message:** `docs(plan): S28-f — open the ledger at the base`

#### C1 — The specification (Markdown only)

- **Goal:** the spec says what this PR builds before it is built (`CLAUDE.md` §2.2).
- **Files:** `docs/MODULE_SPEC.md` (§5.3, the **Placement** bullet), `docs/decisions/ARC-98.md` (a dated note),
  `docs/decisions/INDEX.md` (regenerated by `python3 scripts/check_decision_ids.py --write-index` only if the
  script reports it stale).
- Steps:
  1. Placement bullet: `sharing` (`none` | `home`; `service` refused), `page_cache_kib`, `verify`; the fixed
     layout and why there is no template [QS28f-2]; no `cache_records` [QS28f-1]; the write lock (one sentence);
     `recall_episodes` kept [QS28f-3].
  2. `ARC-98` dated note (2026-10-xx, S28-f): the same four points in four sentences.
- Validation: the doc scripts.
- [ ] Implementation. [ ] Validation. [ ] Review: nothing in the bullet promises a parameter that is not built.
- **Commit message:** `docs(spec): memory placement as built — the layout, the write lock (S28-f)`

#### C2 — `[memory]` and the page cache

- **Goal:** the operator can set placement, and a wrong value fails by name.
- **Files:** `memory/config.py` (new), `config.py`, `memory/home/home.py`, `tests/test_controller_homes.py` (new:
  the config and pragma cases only).
- Steps:
  1. Write `memory/config.py` as §5.2.
  2. `config.py`: import `MemoryConfig`, `SharingModeUnavailable`; add the `memory` field; in `load()`, after the
     paths are resolved, refuse `service` as §5.2.
  3. `home.py`: the `page_cache_kib` keyword and pragma, §5.3.
  4. Tests: AC-f-1 cases; `MemoryHome.open(tmp/"m.sqlite", INSTANCE, page_cache_kib=1024)` then
     `PRAGMA cache_size` reads `-1024`.
- Validation: `test_controller_homes.py`; `test_controller_config.py`; `test_home_open.py`;
  `test_memory_isolation.py`. Mutation **M-f-2** (`load()` without the `service` check) → the AC-f-1 case red.
- [ ] Implementation. [ ] Validation. [ ] Review: no message echoes a value other than the mode name.
- **Commit message:** `feat(cognition): the [memory] table and the home's page cache (S28-f)`

#### C3 — `SeatHomes` and the write lock

- **Goal:** one place opens, shares and closes homes, and writes never overlap.
- **Files:** `controller/homes.py` (new), `tests/test_controller_homes.py`.
- Steps:
  1. Write `homes.py` as §5.4.
  2. Tests: `home_path` both ways; two `hold`s of one instance under `home` return views of one `MemoryHome`
     and one lock; under `none`, two homes; `release` closes on the last holder; a second `hold` of a held seat
     raises; `close()` closes all.
  3. AC-f-5: two seats of one home; seat A's retention is an assisted `lm-summarise` whose gateway awaits an
     `asyncio.Event` (build it as `test_lm_summarise.py`'s `gateway(...)` over a backend whose `complete` awaits
     the event); start A's `absorb` under `held.lock` so it parks at the model call; start B's `absorb` under
     its lock; set the event; both finish; compare each seat's dump with the same two calls run one after the
     other in a fresh home.
- Validation: `test_controller_homes.py`. Mutation **M-f-1** (`hold` returns a fresh `asyncio.Lock()` per seat)
  → AC-f-5 red.
- [ ] Implementation. [ ] Validation. [ ] Review: no path is opened twice; the first `home.seat` is under the lock.
- **Commit message:** `feat(cognition): one memory home per process and instance, with a write lock (S28-f)`

#### C4 — The seat runner and `serve`

- **Goal:** the shipped controller uses `SeatHomes`.
- **Files:** `controller/seat.py`, `__main__.py`, `tests/test_controller_seat.py`, `tests/test_controller_server.py`
  (only if a helper builds runners directly; C0 records), `tests/test_milestone_d_100_days.py`.
- Steps:
  1. `seat.py` as §5.5 except the pass lines (C5) and the account check (C5).
  2. `__main__.py` as §5.6.
  3. Update the three `SeatRunner(` test sites (A-19) to pass `homes=SeatHomes(tmp_path / "store",
     MemoryConfig())`, and close it at the test's end.
  4. `test_milestone_d_100_days.py:182`: open `store_dir / instance / "memory.sqlite"` (the instance is already
     parsed there).
- Validation: `test_controller_seat.py`; `test_controller_breaker.py`; `test_controller_resilience.py`;
  `test_controller_server.py` (real binary, through `cast-slots/with-slot.sh`); `test_milestone_d_100_days.py`
  (through the slot).
- [ ] Implementation. [ ] Validation. [ ] Review: `_ingest` holds the lock across `absorb` and releases it
  before trigger detection; `_close` releases, never closes.
- **Commit message:** `feat(cognition): seats open their memory through the process's homes (S28-f)`

#### C5 — Pass lines and the account check

- **Goal:** every pass is visible in the seat's log; `verify` checks the scopes' accounts.
- **Files:** `controller/log.py`, `controller/seat.py`, `tests/test_controller_log.py`, `tests/test_controller_seat.py`.
- Steps: §5.7; §5.5's account check; tests: AC-f-7 on a scripted run that crosses one simulated day; a log with
  `pass` lines through `read_decisions` and `_status`; with `verify = "pass"` and a planted scope whose `account`
  under-reports by one byte (a fixture scope registered as S28-c's tests register theirs) the seat raises
  `HolderInvariantBroken` naming the seat.
- Validation: the two files. Mutation **M-f-3** (`passes` not called) → AC-f-7 red; **M-f-4** (the check compares
  with `size()` instead of `content_size()`) → the account test red.
- [ ] Implementation. [ ] Validation. [ ] Review: no `pass` line carries text from memory.
- **Commit message:** `feat(cognition): retention passes in the decision log; verify checks scope accounts (S28-f)`

#### C6 — AS28-10 and the real-binary path

- **Goal:** the cassette criterion holds, and the milestone D test reads the new layout.
- **Files:** `tests/test_home_cassettes.py` (new), `tools/cli/tests/milestone_d.rs`.
- Steps:
  1. AC-f-3: reuse `test_lm_summarise_replay`'s `_facts()`, `retention(...)` and `gateway(...)` over
     `ReplayBackend(Cassette.load(CASSETTE))` for the seat `alice`; two peer seats `bob` and `carol` in the same
     home (`MemoryHome.open(":memory:", …)`) ingest every second and every third fact of the same stream with
     `consolidate`; feed the three seats frame by frame (frames of 256, seat order alice, bob, carol); assert
     zero misses (a miss raises) and the sha256 of alice's `SeatView.dump()` equals `DIGEST`.
  2. AC-f-2 and AC-f-4: two runners, `alice` (observer `101`) and `wanderer` (observer `102`), on the scripted
     server, both linked to the same perceived frames: three frames, each with one `spoke` fact addressed to
     both, the third frame on the next simulated day (so a pass runs). Run once under `none` with
     `RecordingBackend(ScriptedAnswers(say_once), cassette)`, await each consult (`_logged`), stop; run again in a
     fresh store under `home` with `ReplayBackend(Cassette.load(cassette))`; compare logs as AC-f-4 says, and the
     files as AC-f-2 says.
  3. `milestone_d.rs` `store_rows`: find the single directory under `store/` that holds `memory.sqlite` and
     open that file; everything else unchanged.
- Validation: `test_home_cassettes.py`; `cargo test -p mineworld-cli --test milestone_d` through the slot.
  Mutation **M-f-5** (`home_path` ignores `sharing`, so `none` shares) → AC-f-2 red; **M-f-6** (a peer seat's
  ingest writes into alice's seat id) → AC-f-3 red.
- [ ] Implementation. [ ] Validation. [ ] Review: no cassette, golden or digest was edited.
- **Commit message:** `test(cognition): cassettes replay under a shared home (AS28-10, S28-f)`

#### C7 — Reference, README, checkpoint, close-out

- **Files:** `cognition/lm-controller/tools/reference.py` (one example: "Keep one memory file per seat"),
  `docs/reference/cognition-toml.md`, `docs/reference/schema/cognition-toml.schema.json` (regenerated),
  `cognition/lm-controller/README.md`, this file (§15).
- Steps:
  1. Add the example `[memory]\nsharing = "none"` to a minimal replay config; regenerate with `uv run --locked
     python cognition/lm-controller/tools/reference.py --out docs/reference`.
  2. README **Memory** paragraph: replace "one SQLite file at a path you choose" with "one SQLite file per world
     instance under the store directory (`<store>/<instance>/memory.sqlite`), shared by every seat of the
     process; `[memory] sharing = "none"` keeps one file per seat and changes nothing a seat remembers". One new
     controller bullet: "Each retention pass is one `pass` line in the seat's decision log."
  3. Run S28-a's per-PR transparency files and S28-b's shared-home Gate 2 file, one at a time, through the slot
     (AC-f-10). AC-f-9's diff scan.
  4. Push `mvp0/pr-s28f`; `gh pr create`; report MERGE-READY with the head SHA once `fast` is green.
- Validation: `test_reference.py`; the harness files; the doc scripts.
- [ ] Implementation. [ ] Validation: `fast` green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(reference): the [memory] table; README memory lines (S28-f)`

---

## 10. Requirements this PR places on other PRs, and parent edits (owner: the primary session)

- **R-S28g-f-1 (on S28-g).** The scale harness opens its homes with `page_cache_kib` from `MemoryConfig()` and
  uses AS28-17's RSS formula with `cache_records = 0` if QS28f-1 is ruled as recommended.
- **FU-S28f-1 (follow-up, unassigned).** Build the record LRU only if S28-g's nightly shows recall time or RSS
  needs it; then add `cache_records` with its measurement.

| Document | Edit |
| --- | --- |
| `step-28-shared-memory.md` §5.4 | `cache_records` and `home` not built (QS28f-1, QS28f-2 as ruled) |
| `step-28-shared-memory.md` §10 | the paths: `<store>/<instance>/memory.sqlite` or `<store>/<instance>/<seat>.sqlite` |
| `step-28-shared-memory.md` §15 AS28-17 | the RSS term `cache_records × 512 B` reads 0 while no LRU exists |
| `pr-s10-p6-lm-controller.md` | D-P6b-36's follow-up: done by S28-f (`pass` lines) |
| `overall.md` | S28-f frozen / merged |

---

## 11. R-LOAD-1

Locally: only the commit's test files, **one at a time**, plus ruff, ruff format, pyright, and the doc scripts.
Real-binary files (`test_controller_server.py`, `test_milestone_d_100_days.py`, `milestone_d.rs`) and the C7
harness go through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`, one at a time, with the load
average recorded beside the wall time. Never `cargo test --workspace`, never the whole `cognition/lm-controller`
suite. The PR's CI on the exact head is the gate.

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S28-f — the controller opens the memory home: [memory], one home per
                         process and instance, the write lock, pass lines, AS28-10
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s28f-home-wiring.md
RELATED / BINDING DOCS:  step-28-shared-memory.md (§§3, 5.4, 5.5, 6, 10, 11, 14.1, 15); pr-s28a-memory-home.md
                         (§10); pr-s28c-memory-scopes.md (§10, QS28c-5); pr-s28e-memory-inspect.md (§10);
                         pr-s10-p6-lm-controller.md (D-P6b-36); docs/MODULE_SPEC.md §5.3; docs/ARCHITECTURE.md
                         §9.3; docs/MVP.md §9 AC-18; .structured-coding/standards.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s28f, its own, held by one session
BRANCH:                  mvp0/pr-s28f, created from origin/main
START CONDITION:         S28-c merged AND this design frozen, then the primary session's dispatch
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–8, as §14's rulings shape them        (unresolved until §14 is ruled)
FROZEN INVARIANTS:       step-28 I-S28-2, I-S28-10; §6 I-f-1 … I-f-7; S28-a's schema and dump, S28-b's
                         retention, S28-c's interface unchanged; §8 as written
SEQUENCE:                C0 … C7 (§9)
ALLOWED COMMANDS:        uv run --locked *; uv sync --locked; cargo build -p mineworld-cli; cargo test -p
                         mineworld-cli --test milestone_d (through the slot); cargo fmt / clippy -p mineworld-cli;
                         target/*/mineworld *; git; gh (never merge); python3 scripts/*; mkdir -p; cp; mv; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or the
                         whole member suite locally (R-LOAD-1); reading any key or .env; calling any model; any
                         record mode against a real backend; editing a golden, a cassette or a pinned digest;
                         edits outside APPROVED SCOPE
MATERIAL STOPS:          S28-c not merged; a transparency or replay test (AC-f-3, AC-f-4, AC-f-10) failing for a
                         reason other than a defect of this PR; a needed change to a policy, an item rule, a
                         bound number, the pass order, a schema, a dump or the meter; a needed change under
                         kernel/, contracts/, server/, persistence/, systems/, worlds/, sdk/; a new dependency
                         or a lock change
PLATFORMS:               Linux, macOS, Windows (the python jobs on all three; Windows: the home's directory is
                         created with Path.mkdir, no shell)
VALIDATION BUDGET:       unit files unrestricted locally one at a time; real-binary files within their current
                         budgets + 25 %; CI: the PR's runs. Gate 1: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds and the
                                                         design is frozen
                         semantic commits: authorized
                         branch push: authorized
                         PR creation / update: authorized
                         CI repair to `fast` green: authorized
                         endpoint: the PR MERGE-READY (head SHA reported to the coordinator)
                         merge: NOT authorized for the implementation session; the coordinator merges
                         (charter, 2026-10-18)
POST-MERGE SYNC OWNER:   primary: step-28, overall.md; implementation session: this ledger
STOP CONDITION:          S28-f MERGE-READY — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-f-1 | The lock makes one seat's long assisted pass delay every other seat's ingest (a model call can take up to `call_timeout_s`) | the delay is bounded by the summarize tier's timeout; triggers are detected after ingest, so a delayed seat only answers later; S28-g measures ingest wall time at 100 seats; a per-seat home (`none`) removes the coupling if an operator needs it |
| R-f-2 | A reader runs while another seat's pass is between its await and its commit | there is no await between a pass's first write and its commit (A-7, `host.py` docstring); C3's test exercises the window; a future await after a write would be a defect of that change, caught by AC-f-5 |
| R-f-3 | Windows: a home file still open when a test deletes `tmp_path` | `SeatHomes.close()` in `serve`'s `finally` and in every test; `release` closes on the last holder |
| R-f-4 | A Haiku implementer re-records a cassette or edits `DIGEST` to turn AC-f-3 green | NEVER list; AC-f-9 scans `tests/cassettes/` and `tests/golden/`; the primary reviews |
| R-f-5 | The old `<store>/<seat>.sqlite` files confuse an operator | the one-line notice (§5.6) names each and says it may be deleted |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **QS28f-1 [operator — the record LRU]** | The step lists `cache_records`, a per-home LRU of decoded fact records (step §4.5). Nothing has measured a need: SQLite's page cache already keeps the pages of recently read records, and one consult reads at most about 80 records. Build the LRU now, or wait for S28-g's numbers? | **Wait.** Ship `page_cache_kib` only. S28-g measures recall wall time and RSS at 12 and 100 seats each night; if either shows a need, a follow-up (FU-S28f-1) adds the LRU with its measurement. Building it now is an optimisation with no measured problem (`CLAUDE.md` §4 item 11), and it adds a cache that a test must prove can never widen access. |
| **QS28f-2 [operator — the home's path]** | The step lists `home`, "a path template" (default `store/<instance>/memory.sqlite`). It does not say which placeholders exist, how `none` names one file per seat, or what stops a template from pointing outside the store. Keep a template, or fix the layout? | **Fix the layout:** `<store>/<instance>/memory.sqlite` under `home`, `<store>/<instance>/<seat>.sqlite` under `none`. The operator already chooses the place with `store`. A fixed layout is what `inspect` and the tests can find without reading the configuration, and it has no escaping or collision cases to validate. A template can be added later if an operator asks for one. |
| **QS28f-3 [operator — `recall_episodes`]** | S28-c left this to S28-f: `person.sections.L1.lines` (the world's) and `[controller] recall_episodes` (the operator's) both bound the episode lines. Retire the operator's knob now? | **Keep both,** as S28-c applies them (the seat shows the smaller). Retiring it changes P6's configuration contract and every operator file that sets it, for no change a person can see. |
| **QS28f-4 [primary — pass lines]** | P6b's ledger (D-P6b-36) gave S28-f the job of writing each retention pass to the seat's decision log. The step's S28-f row does not list it. Do it here? | **Yes, here.** It is about 30 lines, it touches the same `_ingest` call this PR rewires, and it makes "which pass forgot this, and was the model used" answerable from the log of a running seat. |

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 base | `7c7edf8c` | [x] | [x] doc scripts PASS | [x] base recorded (D-f-0) |
| C1 spec | `f0d57c9a` | [x] | [x] doc scripts PASS | [x] no unbuilt parameter promised |
| C2 `[memory]`, page cache | `691e0f2c` | [x] | [x] config 17, home_open 11, isolation 6 PASS | [x] messages name the mode only |
| C3 `SeatHomes`, lock | `ce4ccd8e` | [x] | [x] `test_controller_homes.py` 15 PASS (AC-f-1, -5, -6) | [x] first `home.seat` under the lock |
| C4+C5 runner, serve, pass lines, account check | `335bd98c` | [x] | [x] seat 20, scopes 2, log 6, breaker 6 PASS; `test_milestone_d_100_days.py`, `test_controller_server.py`, `test_controller_resilience.py` left to CI (INCONCLUSIVE) | [x] lock held across `absorb`, released before detection; `_close` releases |
| C6 AS28-10 | `738b7e62` | [x] | [x] `test_home_cassettes.py` 2 PASS (AC-f-2, -3, -4, -7); `cargo fmt` PASS; `milestone_d.rs` and clippy left to CI (INCONCLUSIVE) | [x] no cassette, golden or digest edited |
| C7 reference, README | this commit | [x] | [x] `test_reference.py` 9, structural/provider scans, `test_home_isolation.py` 5, `test_home_transparency.py` 3 PASS; S28-b Gate 2 and the other transparency files left to CI | [x] deviations recorded |

### Evidence

- **C0 base (2026-10-19).** Branch `mvp0/pr-s28f` created from `origin/mvp0/pr-s28c` @ `42de07de` (PR #353, not
  yet merged; D-f-0). `origin/main` @ `de6f54f3` holds S28-b (#340) and S28-e (#351); S28-d is not merged. S28-e
  did not touch `controller/seat.py`'s store opening. `memory/scopes/host.py` exists on the base.
- **Re-anchored on the base.** A-1 `seat.py:188-191`; A-2 `_open_store` `seat.py:268-279`, called at `:243`,
  `:300`, `:517`; A-4 `_ingest` `seat.py:333-360`; A-5 `_close` `seat.py:671-679`; A-6 `__main__.py:77-106`; A-7,
  A-8, A-9 (`home.py:88-113`, `_lock` `:391-412`), A-11, A-12 (`log.py`, now also `scope-defect` lines, S28-c),
  A-13 unchanged in statement. A-19: `SeatRunner(` is built at `test_controller_seat.py:127`, `:555`, `:616` and,
  since S28-c, `test_controller_scopes.py:66`; `serve(` callers (`test_controller_resilience.py`,
  `test_controller_server.py`, `test_milestone_d_100_days.py`) keep their signature.
- **S28-c's shipped names.** `controller.memory_view.recall_view(store, stack, query, recall_episodes)`;
  `memory.scopes.host.recall`/`Recalled`; `memory.scopes.stack.resolve_stack(choice, registry, seat=)` →
  `MemoryStack(seat, scopes: tuple[BoundScope, ...])`; `SeatRunner._stack`; `MemoryScope.account(view) ->
  ScopeBytes` (`.total()`), read through `BaseView` for a base scope and `GuardedView` otherwise (as
  `host._checked` does).

### Mutations

M-f-1 … M-f-6: deferred to milestone (operator SPEED FIRST). The lock's reason is shown by a standing test
instead (`test_without_the_lock_a_second_seats_write_fails_inside_the_open_pass`: SQLite's nested-transaction
error), and the account check by a planted one-byte under-report (`test_controller_seat.py`).

### Deviations and findings

F-S28f-1 … F-S28f-8 (§3.2), recorded at design time.

- **D-f-0 (bounded, coordinator dispatch 2026-10-19).** The start condition says S28-c merged; it is not. The
  branch is stacked on `mvp0/pr-s28c` (#353, head `42de07de`) and merges `origin/main` once #353 lands.
- **D-f-S (operator SPEED FIRST, charter 2026-10-18).** Mutations M-f-1 … M-f-6 and the independent review are
  deferred to the milestone; real-binary files (`test_controller_server.py`, `test_milestone_d_100_days.py`,
  `milestone_d.rs`) and the C7 harness are left to CI (INCONCLUSIVE locally).
- **D-f-1 (bounded).** C4 and C5 are one commit: both edit `_ingest`'s one new `_absorb` helper.
- **D-f-2 (bounded).** `PassLine` also carries `prose_contradictions`, a `PassReport` field S28-b added after the
  design's §5.7 list was written (F-S28f-8: "every `PassReport` field").
- **D-f-3 (bounded).** The account sum is `memory.scopes.host.accounted(view, stack)`, a new function beside
  `recall` that reads each scope through the same `BaseView`/`GuardedView` recall uses; no S28-c signature changes.
- **D-f-4 (bounded).** The verify test plants the under-report by patching `accounted` in the seat runner, not
  through a fixture scope; a healthy run under `verify = "pass"` checks the real sum.
- **D-f-5 (bounded).** `test_controller_seat.py`'s secret scan walks the store recursively (the home now lives
  in `<store>/<instance>/`), and asserts the home file is among those scanned.
- **Lock choice.** `asyncio.Lock` (stdlib), as §4 records: one loop, one connection; SQLite's own
  `locking_mode = EXCLUSIVE` still guards the file across processes (`HomeBusy`).
