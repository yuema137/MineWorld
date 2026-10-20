# PR S28-c — Memory scopes: the `MemoryScope` interface, `ScopeRegistry`, `person` and `world` as scopes, the memory stack in `decisions.yaml`, parameters, named errors, the section ceiling and the generated reference

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: every question answered with its recommendation)

```text
Design revision:        draft 1 (2026-10-18)
Author:                 S28-c / S28-e PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s28ce, branch docs/pr-designs-s28ce,
                        planning base origin/main @ b85f2804 (#260; S28-a #242 merged at 9884c64c; S28-b's
                        design merged #235 at 450f7985, its implementation NOT merged)
Approved by / evidence: the operator, 2026-10-18, in the primary session: every question answered with its recommendation.
Implementation base:    origin/main at C0, after S28-b has merged; C0 records the commit
Start condition:        step-28 §11.1: S28-b MERGED (not yet: its implementation runs in
                        /Users/yuema137/mineworld-worktrees/impl-s28b); S25 25a (#200) merged (yes); 25b's
                        page list known (yes: tools/reference.py writes cognition-toml.md); this design
                        frozen; the primary session's dispatch
Execution contract:     §12 (filled; endpoint authority included)
Implementer:            Haiku-class (step-28 §11.1), the primary reviewing
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0` · **Step:** [`step-28-shared-memory.md`](step-28-shared-memory.md) (STEP DESIGN FROZEN
2026-10-14; rulings §14.1) §§3 (I-S28-1, I-S28-7, I-S28-8, I-S28-10), 5.1–5.5, 5.8, 11 and 11.1 (the S28-c
row), 15 (AS28-2, AS28-13, AS28-14, AS28-19) · **Parent:** [`overall.md`](overall.md) · **Previous PRs:**
[`pr-s28a-memory-home.md`](pr-s28a-memory-home.md) (merged #242; its R-S28c-a-1),
[`pr-s28b-retention-on-seat-views.md`](pr-s28b-retention-on-seat-views.md) (FROZEN 2026-10-18; its R-S28c-b-1,
and `BoundBelowDefault` deferred to this PR, its §2.2) · **Followed by:** S28-d (the anchored scope), S28-e
(inspect reads the stack, §10), S28-f (P6b wiring of `sharing`) · **Coordinates with:**
[`pr-s28e-memory-inspect.md`](pr-s28e-memory-inspect.md) (designed in the same session; the `scopes` view,
QS28c-7) · **Binding parents:** `CLAUDE.md` §§2, 3, 4 (items 1, 3, 5, 7, 8, 11, 12, 13, 16);
`docs/CORE_CONCEPTS.md` §§2 (INV-4, INV-13), 5.3; `docs/ARCHITECTURE.md` §9.3; `docs/MODULE_SPEC.md` §§4.1,
5.2, 5.3; `docs/MVP.md` §9 `AC-18`; `docs/DECISIONS.md` `ARC-59`, `ARC-85`, `ARC-87`, `ARC-98` … `ARC-101`,
`ARC-109`.

**Decision ids.** None new. C1 writes a dated note under `ARC-100` (the interface as built: base scopes, the
citable set, the per-scope sections, the consult's behaviour on a defect) once §14 is ruled.

No hosted model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Scope (step-28 §11, S28-c row).** "`MemoryScope` interface, `ScopeRegistry` (entry points), `person` and
  `world` as scopes, selection through `decisions.yaml` `memory.scopes` (a small Rust reader addition beside
  DL-a's `MemoryPolicyChoice`), parameter models, named errors, section ceiling checks, the generated reference
  pages (25b)." Implementer Haiku; reviewer the primary.
- **Start (step-28 §11.1).** "S28-b merged. S25 25a (#200) merged (generated-reference machinery). 25b's page
  list known (coordinate with S25)." Parallel with S28-e.
- **R-S28c-a-1 (S28-a §10, verbatim).** "`memory/scopes/world.py`'s functions become the `world` scope; the
  `person` scope re-homes `ingest`/`compress`/`retrieve`; `fact_who.role` (if ruled) is what anchor roles read."
- **R-S28c-b-1 (S28-b §10, verbatim).** "The scope-private backstop step (step §7.2) signs its drops
  `host:scope-rows` if it ever adds a forgotten range (it should not: scope rows cite, they do not cover)."
- **Deferred to this PR by S28-b (§2.2, verbatim).** "`BoundBelowDefault(seat, given_mib)` (step §5.5; AS28-16
  (c)) — S28-c (selection and named errors). FG-b2's `StoreBoundOutOfRange` stands until then."
- **The interface, the registry and the selection** (step §5.3, MODULE_SPEC §5.3), the **parameters** (step
  §5.4), the **errors** (step §5.5), the **isolated tests** (step §5.8) as written there; §3.2 lists every place
  where the audited code makes a sentence of those sections inexact.
- **Rulings not re-asked.** QS28-7 (a scope never grants access), QS28-10 (the stack in `decisions.yaml`,
  placement in `cognition.toml`), QS28a-3 (`fact_who.role`).
- **Integration checkpoint (step-28 §11.1).** "Every PR ends with the transparency harness green (12 seats,
  `none` vs `home`). From S28-b on, it also runs FG-b's real-binary Gate 2 scenario on seat views."
- **R-LOAD-1 (coordinator rule, in force).** §11.

---

## 1. Goal

Turn memory's recall into a **stack of scopes behind one typed interface**, chosen per person by the World Pack,
without changing a byte of what any person recalls under the default stack:

1. `MemoryScope` is a typed protocol; `ScopeRegistry` finds scopes through the entry-point group
   `mineworld.memory_scopes`, with FG-b's load checks and refusals; `person` and `world` are its first two
   entries.
2. A seat's **memory stack** is written in `decisions.yaml` `memory:` (`scopes:` beside `policy` and
   `store_bound_mib`), passed through by the Rust reader with its source position, and resolved at seat start
   into bound scopes, or refused by name with the file, line and column.
3. The `person` scope's four sections take their budgets and the L0 window from its parameters; the defaults are
   P4's, so the default stack renders exactly today's section.
4. The **host** runs every scope's recall, checks every line's citation against what the seat may cite and every
   section against its budget, and assembles the one `MemorySection` a consult reads; a hostile scope can neither
   leak an unheld fact nor exceed the 6 000-byte ceiling.
5. The parameters of every bundled scope are **generated** into `docs/reference/memory-scopes.md`, and the
   decision list's reference gains `scopes`.

No policy, item rule, bound number, pass order, schema or dump changes. Nothing changes under `kernel/`,
`contracts/`, `server/`, `persistence/`, `systems/`, `worlds/` or `sdk/` (I-S28-10).

---

## 2. Scope

### 2.1 In scope

1. **The interface** (`memory/scopes/interface.py`, new): `MemoryScope`, `ScopeParameters`, `SectionSpec`,
   `SectionBudget`, `ScopeView`, `ScopeBytes`, `TableSpec`, `FactRecord`, `Reading`, `ScopeRow` (§5.2), and the
   scope author's surface re-exported by `memory/scopes/__init__.py`.
2. **The registry** (`memory/scopes/registry.py`, new): `ScopeRegistry` over `importlib.metadata`, its load
   checks, `bind` (§5.3).
3. **The bundled scopes:** `memory/scopes/person.py` (new: `PersonScope`, `PersonParameters`) and `WorldScope`
   added to `memory/scopes/world.py`; the entry points `person` and `world` in `pyproject.toml` (§5.4).
4. **Selection** (`memory/scopes/stack.py`, new): `MemoryScopeChoice`, `MemoryStack`, `resolve_stack`, the
   ceiling check; `MemoryPolicyChoice.scopes`; `BoundBelowDefault` in `memory/retention/selection.py`
   (§5.5).
5. **The recall host** (`memory/scopes/host.py`, new): runs the stack, checks lines, assembles the section; the
   widened `SectionLine.level` and `MemorySection.sections` (`memory/retrieve.py`); `retrieve` parameterized by
   the person scope's budgets and window; `SeatView.citable` (`memory/home/view.py`) (§5.6, §5.7).
6. **The controller's use** (`controller/seat.py`, `controller/memory_view.py`, `controller/log.py`): the seat
   resolves its stack at start beside its retention, `memory_view` recalls through the stack, a scope defect is
   withheld and logged (§5.8).
7. **The Rust reader** (`cognition/decision/src/{read.rs, resolve.rs, json.rs, schema.rs, read_tests.rs}`):
   `scopes` in `memory.default` and `memory.assign` entries, passed through with positions; decision JSON
   format 3 (§5.9).
8. **The reference:** `cognition/lm-controller/tools/reference.py` writes `memory-scopes.md` and
   `schema/memory-scopes.schema.json`; `tools/cli/src/reference/pages.rs` lists them; `docs/reference/` is
   regenerated (`decisions.md`, `decisions.schema.json`, the index, the two new files) (§5.10).
9. **Specifications:** `docs/MODULE_SPEC.md` §5.3 corrected to the code and the rulings (§3.2, §14); a dated
   note under `ARC-100` (C1).
10. **Tests** of §7.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| The host's scope-table machinery: creating `<id>_*` tables from `TableSpec`, inserting `index` rows at `hold`, cascading by `event_key` and `seat`, `scope-removed:<id>`, rebuild on reinstall, the home's record of installed scope tables (a schema change), AS28-14's runtime half and AS28-15 | **S28-d** (QS28c-4 recommended); S28-c defines `index`, `tables` and their load checks only |
| The `anchored` scope, `place` and `item`, `Rendered.anchors`, C-3 | S28-d |
| The backstop step "drop scope-private rows, oldest `last_at` first" (step §7.2; R-S28c-b-1) | S28-d: no scope-private row exists before it |
| `mineworld-memory inspect … scopes` and per-scope bytes in `inspect … bytes` | whichever of S28-c and S28-e merges second (QS28c-7) |
| `cognition.toml [memory]` (`sharing`, `home`, `cache_records`, `page_cache_kib`, `verify`), `SharingModeUnavailable` | S28-f |
| A generated page for the **retention policies'** parameters (none exists: F-S28c-14) | follow-up FU-S28c-1, recorded for the primary |
| Moving `ingest`, `compress/` or `retrieve` into `memory/scopes/person/` | not done: `person.py` owns the scope and names its implementation modules (F-S28c-6) |
| Any change to a policy, an item rule, a bound number, the pass order, schema 5, a dump or the meter | never in this PR (material stop, §12) |
| Any change under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` | never (I-S28-10) |

### 2.3 Dependencies and the start condition

- **Required: S28-b merged.** Its schema 5 and its host changes touch `memory/home/schema.py`,
  `memory/retention/{host, plan, bound, forgotten}.py`; this PR edits `memory/retention/selection.py` and
  `memory/home/view.py`. C0 re-anchors every line of §3 on the merged code.
- **Merged:** S28-a (#242), 25a (#200), 25b (`tools/reference.py`, `docs/reference/cognition-toml.md`), DL-a,
  FG-b2 (#213), P6b (#207), P7 (#244, the decision-log fixture C-7 names).
- **Not required:** S28-e. QS28c-7 settles who adds the `scopes` view.
- **Open PR that may touch the same file:** #262 (FU-S28a-1, the home's creation marker) edits
  `memory/home/home.py` and `schema.py`, not `view.py`; C0 records whether it merged.

---

## 3. Audit (`origin/main @ b85f2804`)

Line numbers are on `b85f2804` (schema 4, S28-a merged, S28-b not merged). C0 re-anchors each on the base.

### 3.1 Source anchors (under `cognition/lm-controller/src/mineworld_cognition/` unless a path says otherwise)

| # | Fact | Where |
| --- | --- | --- |
| C-1 | **The pattern to mirror.** `PolicyRegistry` reads one entry-point group, loads each point, refuses a name that is not an id, a non-class, an `id` other than the entry name, a `Parameters` that is not a Pydantic model or has a field without a default, a missing `plan`, an object that cannot be built without arguments; two distributions publishing one id → `DuplicatePolicy`; an import failure → `PolicyLoadError`. `bind` refuses unknown keys and invalid values as `InvalidParameters(policy, errors, source)`, each error a location and a message, never the value (`include_input=False`). | `memory/retention/registry.py:39-47` (`SourceSpan`), `:50-98` (errors), `:162-216`, `:224-247` |
| C-2 | `MemoryPolicyChoice` = `policy`, `fallback`, `parameters`, `store_bound_mib`, `source`; `resolve` binds the policy and the bound; `_bounds` raises `StoreBoundOutOfRange(mib, source)` for a bound outside 20 … 1 024. `resolve` does not know the seat. | `memory/retention/selection.py:36-46`, `:90-116` |
| C-3 | Retrieval: `LINE_BYTES = 160`, `SECTION_BYTES = 6000`; `BUDGETS` fixes the four sections' headers, lines and bytes (`L3` 16 / 1 600, `L2` 4 / 800, `L1` 8 / 1 600, `L0` 48 / 2 000, a section's bytes counting its header and newlines); `SectionLine` is `level: Level`, `text`, `citation`; `MemorySection.render` walks `BUDGETS` in order; `retrieve` fills L3, L2, L1, L0; `_recent` reads `f.at > query.at - DAY_S`. | `memory/retrieve.py:45-63`, `:75-99`, `:113-143`, `:248-264` |
| C-4 | `Level = Literal["L3", "L2", "L1", "L0"]`; retention items use their own integer `ItemLevel`. | `memory/records.py:37`; `memory/retention/policy.py:22` |
| C-5 | `memory_view` keeps every line but the L1 lines beyond `recall_episodes` (`line.level == "L1"`); `[controller] recall_episodes` 0 … 8, default 8. | `controller/memory_view.py:18-29`; `config.py:318-321` |
| C-6 | `SeatRunner.__init__` resolves the seat's retention once (`resolve(seat.memory or DEFAULT_CHOICE, PolicyRegistry.installed(), …)`); a consult calls `memory_view(store, RecallQuery(...), recall_episodes)` with no exception handling. | `controller/seat.py:147-151`, `:368-376` |
| C-7 | **The decision log stores the section.** `DecisionInput.memory: MemorySection`; every model is strict, frozen, `extra="forbid"`; the committed fixture `tests/fixtures/milestone_d.decisions.jsonl` (P7) carries `{"level":"L3",…}` lines and is replayed offline by `test_milestone_d_replay.py`. | `controller/log.py:56-63`; `backend/model.py:33-36`; `tests/test_milestone_d_replay.py:36` |
| C-8 | `validate` reads `memory.lines` citation tokens and `memory.citations()`. | `controller/validate.py:93-94` |
| C-9 | The Python reader of the decision JSON checks `format == 2`; `ResolvedSeat.memory: MemoryPolicyChoice | None`; fixture `tests/fixtures/decisions_format2.json`. | `controller/assignment.py:26`, `:39-53`, `:79-81` |
| C-10 | **The Rust reader.** `JSON_FORMAT = 2`; `MemoryPolicyChoiceJson { policy, fallback, parameters, store_bound_mib, source }`, `deny_unknown_fields`; `RawPolicy` and `RawMemoryAssign` (`deny_unknown_fields`) take `policy`, `fallback`, `parameters`, `store_bound_mib`; `choice()` checks the policy id's spelling and passes the rest through; `memory_of` takes the **whole** winning entry, seat > class > default; `digest()` serializes each seat's memory choice (without its position) only when the file has `memory:`; `memory_fields` is the JSON Schema of an entry. | `cognition/decision/src/json.rs:14`, `:57-71`; `read.rs:81-95`, `:235-262`, `:704-777`; `resolve.rs:229-236`, `:267-290`; `schema.rs:196-237` |
| C-11 | **The reference.** `tools/reference.py` (the cognition half, 25b) writes `cognition-toml.md` and its schema from `CognitionConfig`, refusing an undescribed field and an example the loader refuses; `pages.rs` `COGNITION` lists those two files for the index and leaves them to the Python half; `test_reference.py` is the drift test; `tools/cli/tests/reference.rs:260-275` checks the index links the cognition files. No page documents a retention policy's parameters. | `cognition/lm-controller/tools/reference.py:1-32`, `:330-372`; `tools/cli/src/reference/pages.rs:17-29`; `cognition/lm-controller/tests/test_reference.py` |
| C-12 | **The specification disagrees with the code in three places.** MODULE_SPEC §5.3 writes the bound key `bound_mib` (the shipped key is `store_bound_mib`, C-10), the choice's position field `span` (the shipped models call it `source`, C-2), and `HomeBusy(path, holder_pid)` (QS28a-4 dropped `holder_pid`). | `docs/MODULE_SPEC.md:2057-2069`, `:2105` |
| C-13 | The isolation scan covers `memory/`, `compress/`, `forgetting/`: no `open(`, `os.`, `Path(`, `subprocess`; `sqlite3.connect` only in `memory/home/home.py`; imports only within those packages plus three; the writers scan allows writes to `fact`, `fact_who`, `access`, `access_text` only in `memory/home/` and `memory/scopes/world.py`. | `tests/test_memory_isolation.py:20-80` |
| C-14 | `memory/scopes/__init__.py` is one docstring line; `memory/scopes/world.py` holds `ConflictingFact`, `who`, `insert_or_verify`, `collect`, `check_holders`. | `memory/scopes/__init__.py:1`; `memory/scopes/world.py:24-95` |
| C-15 | The distribution publishes the groups `mineworld.forgetting`, `mineworld.retry`, `mineworld.breaker`; no `mineworld.memory_scopes`. | `cognition/lm-controller/pyproject.toml` |
| C-16 | **No "internal error" fallback exists in P6.** A consult's failure classes are `transport`, `budget`, `content`, `semantic`, `frame`, `seat`; an exception from `memory_view` propagates out of `_consult_once`. | `controller/failures.py:21`; `controller/seat.py:353-376` |
| C-17 | `SeatView` exposes `connection` (the home's raw `sqlite3.Connection`); `retrieve`, `stable_facts`, `chapters`, `load_episodes` query through it with the seat predicate. | `memory/home/view.py:96-120`; `memory/retrieve.py:135-264` |
| C-18 | The size trigger's authority is the seat's meter (`content_size()`), kept in-transaction (S28-a §5.7); nothing else measures a seat. | `memory/home/view.py:255-296`, `:350-368` |
| C-19 | A stable fact's citation is `first`, `last` and `level_event`, whose L0 rows may be gone; an episode's is `first … last`; a chapter's is its stored ranges. | `memory/records.py:115-175` |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S28c-1** | The spec's `bound_mib` and `span` (C-12) are not the shipped names `store_bound_mib` and `source` (C-2, C-10). | **Bounded: the code wins** (`CLAUDE.md` §2.1 rule 4). The stack is `scopes:` beside `store_bound_mib`; each scope choice carries `source`. C1 corrects MODULE_SPEC §5.3, and the stale `HomeBusy(path, holder_pid)` with it. |
| **F-S28c-2** | `SectionLine.level` is the `Level` literal (C-3, C-4), and the decision log stores `MemorySection` in a strict model whose committed fixture must keep parsing (C-7). A new required field, or a renamed one, breaks the fixture. | **Bounded:** `level` keeps its name and its values for the person scope's lines, and is widened to `SectionKey` (`L0` … `L3`, or a scope id) (§5.6). `MemorySection` gains `sections`, defaulted to `()`, naming the non-person sections in render order. An old log parses unchanged; the default stack's rendering is byte-identical. |
| **F-S28c-3** | "A citation outside the seat's held keys is `AccessDenied`" (step §5.3) cannot be applied literally: the person scope's lines cite **ranges** (C-19). An episode's range covers records merged into it, and a stable fact cites records long released. Every surviving L1, L2 and L3 line would be refused. | **§14 QS28c-2** (the citable set). |
| **F-S28c-4** | "The consult falls back as P6 does for an internal error" (step §5.5) names a behaviour P6 does not have (C-16). | **§14 QS28c-3.** |
| **F-S28c-5** | "`recall` reads only through `SeatView`, whose every method is filtered by the seat's access index" (step §5.3) is not what `SeatView` is: it exposes the raw connection (C-17), and the person scope's retrieval is six seat-filtered SQL queries on it. The step also says "none is privileged". Both cannot hold for `person` without rewriting retrieval. | **§14 QS28c-1** (base scopes and the `ScopeView`). |
| **F-S28c-6** | "The `person` scope re-homes `ingest`/`compress`/`retrieve`" (R-S28c-a-1) read as moving files would touch about 20 modules and every test import for no behaviour. | **Bounded:** `memory/scopes/person.py` is the scope's owner module: it holds the class, its parameters and its recall entry, and its docstring names `ingest`, `compress/` and `retrieve` as the scope's implementation. Nothing moves. MODULE_SPEC's owner column stays true. |
| **F-S28c-7** | The step's `recall(view, query, budget: SectionBudget, params)` gives one budget; the person scope has four sections, and the host must know every scope's section budgets before a seat plays to check the 6 000-byte ceiling. | **Bounded:** a scope declares `sections(params) -> tuple[SectionSpec, ...]` (key, header, budget), and `recall` receives the budgets as a mapping by section key (§5.2). |
| **F-S28c-8** | `BoundBelowDefault(seat, given_mib)` (step §5.5) needs the seat; `resolve` does not have it (C-2). Above 1 024 is not "below default". | **Bounded:** `resolve` takes the seat key. Below 20 → `BoundBelowDefault(seat, given_mib, source)`; above 1 024 → `StoreBoundOutOfRange` as today. |
| **F-S28c-9** | The step's `DuplicateScope` is about two distributions (C-1's `DuplicatePolicy`); a stack listing one id twice has no named error. | **Bounded:** `ScopeListedTwice(scope, source)`. |
| **F-S28c-10** | A stack is "`person` required first" (step §5.4) and `world` "uninstalling it is not possible" (§5.2.2), but nothing says whether a world may list `world`. | Folded into **QS28c-1**. |
| **F-S28c-11** | Adding `scopes` to the canonical form changes the digest of every decision file with `memory:` (C-10), which `run` prints and users compare by eye. | **Bounded:** the canonical memory entry serializes `scopes` only when the entry writes it, so a file without `scopes` keeps its digest (C0 records the digests of every World Pack's decision file; C2 checks them unchanged). |
| **F-S28c-12** | The decision JSON gains a key, so its `format` goes from 2 to 3 (C-9, C-10). | **Bounded:** format 3, refused by name at 2 (no compatibility layer, `CLAUDE.md` §4 item 12); `decisions_format2.json` becomes `decisions_format3.json`. |
| **F-S28c-13** | The winning `memory:` entry is whole (C-10): a seat entry without `scopes` gets the default stack `[person]`, even when `memory.default` lists more. | **Bounded, and stated in the reference:** this is how `policy`, `parameters` and `store_bound_mib` already behave. A world that wants a scope for every seat writes it in each entry that overrides the default. |
| **F-S28c-14** | No generated page documents a retention policy's parameters (C-11); the step asks only for `memory-scopes.md`. | Recorded as **FU-S28c-1** for the primary. Not done here. |
| **F-S28c-15** | `person.sections.L1.lines` (0 … 8, a world's) and `[controller] recall_episodes` (0 … 8, the operator's, C-5) bound the same lines. | **§14 QS28c-5.** |
| **F-S28c-16** | The step says the per-seat running total uses "each scope's `account` as the authority" (§7.2). S28-a made the meter the authority (C-18). | **Bounded:** the meter stays the authority for the bound; `account` is the per-scope breakdown and must sum to it (checked by a test, and by `verify` when S28-f wires it). |
| **F-S28c-17** | The host machinery for scope tables (`tables`, `index`, step §5.3 "the host does the writing") has no user in this PR: `person` and `world` own the home's base tables. | **§14 QS28c-4.** |

### 3.3 What does not exist

No `MemoryScope`, no scope registry or entry-point group, no stack in `decisions.yaml` or the decision JSON, no
per-scope parameters, no host check of a section's citations, no `AccessDenied`, `ScopeSectionOverBudget`,
`TotalSectionsOverCeiling`, `PersonScopeRequired`, `BoundBelowDefault`, no `memory-scopes.md`.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Decision | Why |
| --- | --- | --- |
| Discovery of installed scopes | `importlib.metadata` entry points (the standard), exactly as `PolicyRegistry` | `ARC-85`; one pattern for every plug-in group of the package |
| Parameter models and their JSON Schema | Pydantic (`DEP-24`), strict models; the schema from `model_json_schema()` | the reference generator already reads Pydantic schemas (25b) |
| A plug-in framework (`pluggy`, `stevedore`) | **Declined** | the registry is ≈ 150 lines mirroring C-1; a framework adds hook-call semantics we do not use and a second plug-in style beside `mineworld.forgetting` |
| Restricting what a scope's code can read (SQLite's authorizer, a sandbox) | **Declined** for MVP-0 (QS28c-1's alternative (c)) | an authorizer can refuse writes and whole tables, not rows of another seat; the typed `ScopeView` plus the host's citation check is exact and testable |
| The YAML side | the existing `serde` reader (`Spanned`, `deny_unknown_fields`) | DL-a's; positions come for free |

No dependency is added; `uv.lock` and `Cargo.lock` do not change.

---

## 5. Design

Written with every §14 question answered as recommended; each place a "no" would change is marked **[QS…]**.

### 5.1 Module layout

```text
memory/scopes/__init__.py   the scope author's surface: MemoryScope, ScopeParameters, SectionSpec,
                            SectionBudget, ScopeView, ScopeBytes, TableSpec, FactRecord, Reading, ScopeRow,
                            RecallQuery, SectionLine, Citation, EventRange (re-exported, as retention/__init__)
memory/scopes/interface.py  the protocol and its carriers                                       (§5.2)
memory/scopes/errors.py     the named errors of `memory.scopes`                                 (§5.11)
memory/scopes/registry.py   ScopeRegistry, BoundScope                                           (§5.3)
memory/scopes/stack.py      MemoryScopeChoice, MemoryStack, resolve_stack                       (§5.5)
memory/scopes/host.py       recall(view, stack, query, *, on_defect) -> Recalled                (§5.7)
memory/scopes/person.py     PersonScope, PersonParameters                                       (§5.4)
memory/scopes/world.py      + WorldScope (empty parameters, no section)                         (§5.4)
memory/retrieve.py          SectionKey, SectionLine.level widened, MemorySection.sections,
                            retrieve(view, query, budgets=PERSON_DEFAULTS, window_s=DAY_S)      (§5.6)
memory/home/view.py         + SeatView.citable(endpoints) -> frozenset[EventKey]                (§5.7)
memory/retention/selection.py  MemoryPolicyChoice.scopes; resolve(choice, registry, gateway, *, seat)
controller/memory_view.py   memory_view(store, stack, query, recall_episodes) -> Recalled
controller/seat.py          resolves the stack beside the retention; logs scope defects
controller/log.py           + ScopeDefectRecord (kind "scope-defect")
```

`memory/scopes/` is under the isolation scan (C-13): no `open(`, `os.`, `Path(`, `subprocess`, no connection.

### 5.2 The interface (`memory/scopes/interface.py`, pyright strict)

```python
ScopeId = Annotated[str, Field(pattern=r"^[a-z][a-z0-9-]{0,47}$")]
SectionKey = Annotated[str, Field(pattern=r"^(L[0-3]|[a-z][a-z0-9-]{0,47})$")]
"""A section's key: `L3` … `L0` for the person scope's four, a scope's id for any other scope's one section."""

class ScopeParameters(CognitionModel):
    """The base of every scope's `Parameters`: strict, frozen, unknown keys refused; every field defaulted."""

class SectionBudget(CognitionModel):
    lines: Annotated[int, Field(ge=0, le=48)]
    bytes: Annotated[int, Field(ge=0, le=6000)]
    """The section's bytes, its header and every newline included (P4's rule, QP4-7)."""

class SectionSpec(CognitionModel):
    key: SectionKey
    header: Annotated[str, Field(min_length=1, max_length=60)]   # one line, ends with ":"
    budget: SectionBudget

class FactRecord(CognitionModel):            # a held record's envelope, as the world scope stores it
    event: EventIdField; at: int; kind: EventTypeIdField; place: EntityIdField | None
    who: tuple[tuple[EntityIdField, Literal[1, 2, 3]], ...]     # fact_who rows, ordinal order (QS28a-3)

class Reading(CognitionModel):               # the seat's own rendering of a held record
    gist: str; salience: Salience; mine: bool; importance: Importance

class ScopeBytes(CognitionModel):
    tables: tuple[tuple[str, NonNegative], ...]                 # (view table, bytes), in view order

@dataclass(frozen=True)
class TableSpec:                             # declared now, hosted by S28-d [QS28c-4]
    name: str                                # "<scope id>_<name>"
    columns: tuple[str, ...]
    keyed_by: Literal["event", "seat", "seat-event"]

class ScopeRow(CognitionModel):              # one row `index` returns, hosted by S28-d [QS28c-4]
    table: str; cells: tuple[str | int | None, ...]

class ScopeView(Protocol):
    """What a scope reads: one seat's held facts, nothing else. Every method refuses a key the seat does
    not hold with `AccessDenied` (raised by the view, naming the scope, the seat and the key)."""
    @property
    def seat(self) -> EntityKey: ...
    @property
    def me(self) -> EntityId: ...
    def holds(self, event: EventId) -> bool: ...
    def record(self, event: EventId) -> tuple[FactRecord, Reading]: ...
    def held_between(self, first_at: int, last_at: int, limit: int) -> tuple[EventId, ...]: ...  # newest first
    def name(self, entity: EntityId) -> str | None: ...           # the seat's own name book

class MemoryScope[P: ScopeParameters](Protocol):
    id: ClassVar[str]
    Parameters: type[P]
    tables: ClassVar[tuple[TableSpec, ...]]
    def sections(self, params: P) -> tuple[SectionSpec, ...]: ...
    def index(self, fact: FactRecord, reading: Reading, params: P) -> tuple[ScopeRow, ...]: ...
    def recall(self, view: ScopeView, query: RecallQuery,
               budgets: Mapping[str, SectionBudget], params: P) -> tuple[SectionLine, ...]: ...
    def account(self, view: ScopeView) -> ScopeBytes: ...
```

- No signature names a `sqlite3.Connection`, a path, a clock or a gateway (AS28-14's signature half; the scan of
  §7 checks the annotations of `interface.py`).
- **[QS28c-1] Base scopes.** `person` and `world` are the home's base scopes. They own the home's base tables
  and are implemented by the memory package itself. The host passes them the seat's `SeatView`, which
  implements `ScopeView` structurally and also carries the connection their existing code reads through (C-17).
  Every other scope is passed a `GuardedView` (`host.py`), which implements `ScopeView` and nothing else: it has
  no `connection` attribute, and its methods run the seat's access join themselves. The privilege is explicit,
  named once (`registry.BASE_SCOPES = ("person", "world")`), granted only to the entry points of the
  `mineworld-cognition` distribution, and covered by the same evidence as today: the seat-predicate scans,
  S28-a's golden and transparency harness, and M-a-5.
- `index` and `tables` are part of the protocol and are load-checked now (§5.3), so S28-d adds no member. The
  host does not call `index` in this PR **[QS28c-4]**.

### 5.3 The registry (`memory/scopes/registry.py`)

`GROUP = "mineworld.memory_scopes"`. `ScopeRegistry.installed()` / `from_entry_points(points)` /
`ScopeRegistry(mapping)` and `ids()` mirror C-1 line for line. The load checks, each a `ScopeLoadError(name,
distribution, why)`:

1. the entry name matches `ScopeId`;
2. the object is a class, and its `id` equals the entry name;
3. `Parameters` subclasses `ScopeParameters`, and every field has a default;
4. `tables` is a tuple of `TableSpec`, each named `<id>_<…>`, the names unique; a base scope declares `()`;
5. `sections`, `index`, `recall` and `account` are callable;
6. the class builds without arguments;
7. `person` or `world` from any distribution other than `mineworld-cognition` → refused, "the base scopes are
   the memory package's own" **[QS28c-1]**.

Two distributions publishing one id → `DuplicateScope(scope, distributions)`. `bind(scope, parameters, *,
source)` → `BoundScope` (the instance, its decoded `Parameters`, its `sections(params)`), refusing an unknown id
with `UnknownScope(scope, installed)` and unknown keys or invalid values with `ScopeParameterError(scope, errors,
source)`, each error a location and a message, never the value (C-1's `include_input=False`).

### 5.4 The bundled scopes

**`person`** (`memory/scopes/person.py`):

```python
class PersonSection(CognitionModel):          # one of the four, its maximum the P4 default
    lines: int; bytes: int                     # validated per section below

class PersonSections(CognitionModel):
    L3: PersonSection = (16, 1600); L2: PersonSection = (4, 800)
    L1: PersonSection = (8, 1600);  L0: PersonSection = (48, 2000)     # P4's BUDGETS (C-3)
    # a model validator refuses lines above the default or bytes above the default, per section:
    # "they can only shrink" (step §5.4)

class PersonParameters(ScopeParameters):
    sections: PersonSections = PersonSections()
    l0_window_s: Annotated[int, Field(ge=3600, le=86400)] = 86400      # D-P4-11
```

Every field carries a `description` (the reference refuses one without, C-11). `sections()` returns the four
`SectionSpec`s with P4's headers (`People I know:`, `Recent weeks:`, `Episodes I remember:`, `The last day:`).
`recall` calls `retrieve(view, query, budgets, window_s)` (§5.6). `index` returns `()`. `account` returns the
seat view's tables except `meta`, each with its bytes from `row_bytes`, in view order, so that the sum equals
`content_size()` (F-S28c-16).

**`world`** (`memory/scopes/world.py`, beside S28-a's functions): empty `Parameters`, `sections()` = `()`,
`recall` = `()`, `index` = `()`, `account` = `()` in a seat view (its bytes are the home's `fact` and `fact_who`
lines, which S28-e reports as home totals). It is never written in a stack **[QS28c-1]**: placement is
`cognition.toml`'s (`sharing`, S28-f).

`pyproject.toml`:

```toml
[project.entry-points."mineworld.memory_scopes"]
person = "mineworld_cognition.memory.scopes.person:PersonScope"
world = "mineworld_cognition.memory.scopes.world:WorldScope"
```

### 5.5 Selection

**YAML** (World Pack, `decisions.yaml`):

```yaml
memory:
  default:
    policy: consolidate
    store_bound_mib: 40
    scopes:
      - id: person
        parameters: { sections: { L0: { lines: 24, bytes: 1200 } }, l0_window_s: 43200 }
  assign:
    - { seat: bob, policy: decay, scopes: [ { id: person } ] }
```

**JSON** (format 3), per seat `memory`:

```json
{"policy":"consolidate","fallback":null,"parameters":{},"store_bound_mib":40,
 "scopes":[{"id":"person","parameters":{"sections":{"L0":{"lines":24,"bytes":1200}},"l0_window_s":43200},
            "source":{"file":"decisions.yaml","line":5,"column":9}}],
 "source":{"file":"decisions.yaml","line":2,"column":3}}
```

`scopes` is `null` when the entry writes none. **Python:**

```python
class MemoryScopeChoice(CognitionModel):
    id: ScopeId
    parameters: JsonObject = Field(default_factory=dict)
    source: SourceSpan | None = None

class MemoryPolicyChoice(CognitionModel):      # C-2, one field added
    …
    scopes: tuple[MemoryScopeChoice, ...] | None = None
    """The memory stack as written; `None` is `[person]` with its defaults."""

@dataclass(frozen=True)
class MemoryStack:
    scopes: tuple[BoundScope, ...]             # person first, then the world's order

def resolve_stack(choice: MemoryPolicyChoice, registry: ScopeRegistry, *, seat: EntityKey) -> MemoryStack
```

`resolve_stack` refuses, in this order, each by name with the entry's source:

1. `ScopeNotSelectable("world", source)` for a `world` entry **[QS28c-1]**;
2. `PersonScopeRequired(source)` when the list is empty or its first id is not `person`;
3. `ScopeListedTwice(scope, source)` (F-S28c-9);
4. `UnknownScope(scope, installed)`;
5. `ScopeParameterError(scope, errors, source)`;
6. `TotalSectionsOverCeiling(total, source)` when Σ over every bound scope's `sections()` budgets' bytes >
   `SECTION_BYTES` (6 000). The default `person` alone is exactly 6 000, so a scope with a section must take its
   bytes from `person`'s.

`resolve` (retention, C-2) gains the keyword `seat` and raises `BoundBelowDefault(seat, given_mib, source)` for a
bound under 20 MiB (F-S28c-8). `SeatRunner` calls both at construction, so every refusal stops the seat before it
plays, as today's policy refusals do.

### 5.6 Sections and lines (`memory/retrieve.py`)

- `SectionLine.level: SectionKey` (F-S28c-2). The person scope's lines keep `L3` … `L0`; another scope's lines
  carry the scope's id.
- `MemorySection` gains `sections: tuple[SectionHeading, ...] = ()`, `SectionHeading(key, header)`, listing the
  **non-person** sections in render order. `render()` writes the person scope's four in P4's order (L3, L2, L1,
  L0) with P4's headers, then each heading of `sections` in order, each non-empty section as `header + "\n" +
  lines`. With the default stack `sections == ()` and the output is today's, byte for byte.
- `retrieve(store, query, budgets=PERSON_DEFAULTS, window_s=DAY_S)`: `_Section` takes its `Budget` from
  `budgets`, `_recent` reads `f.at > query.at - window_s`. With the defaults every statement is today's.
- `citations()` is unchanged (all lines, all sections).

### 5.7 The recall host (`memory/scopes/host.py`)

```python
@dataclass(frozen=True)
class ScopeDefect:                     # what `withhold` records instead of raising
    scope: str; seat: EntityKey; error: str        # the error's class name and message (no gist, no value)

@dataclass(frozen=True)
class Recalled:
    section: MemorySection
    defects: tuple[ScopeDefect, ...]

def recall(view: SeatView, stack: MemoryStack, query: RecallQuery, *,
           on_defect: Literal["raise", "withhold"]) -> Recalled
```

For each bound scope in stack order: build its view (`SeatView` for a base scope, `GuardedView` otherwise), call
`recall`, then check every returned line **before** it can join the section:

1. **Section.** `line.level` is one of the scope's `SectionSpec` keys → otherwise `ScopeSectionOverBudget(scope,
   section, what="section", …)`.
2. **Line.** `len(line.render().encode()) ≤ LINE_BYTES` (160) → otherwise `ScopeSectionOverBudget(…,
   what="line", bytes, limit=160)`.
3. **Section budget.** The section's lines ≤ its `lines`, and header + newline + Σ (line + newline) ≤ its `bytes`
   → otherwise `ScopeSectionOverBudget(…, what="lines" | "bytes", …)`.
4. **Citation [QS28c-2].** Every range end of every citation is in the seat's **citable set** →
   otherwise `AccessDenied(scope, seat, event)` naming the first offending end. The citable set
   (`SeatView.citable(ends)`, one query per check, seat predicate first) is:
   - for a base scope: the keys of the seat's `access` rows, and the range ends its own private units cite
     (episodes' `first_key`/`last_key`, `chapter_ranges`, stable facts' `first_key`, `last_key`, `level_key`);
   - for any other scope: the keys of the seat's `access` rows only, and a range must have held keys at both ends.
5. A scope's `recall` raising → the exception is wrapped as `ScopeRecallFailed(scope, seat, error class)`.

`on_defect="raise"` raises the first error. `"withhold"` drops **that scope's whole contribution** for this call
and records a `ScopeDefect`; the other scopes' sections are unchanged **[QS28c-3]**. Every check is a function of
the view and the lines, so a replay withholds the same scope.

### 5.8 The controller

- `SeatRunner.__init__` resolves `self._stack = resolve_stack(seat.memory or DEFAULT_CHOICE,
  ScopeRegistry.installed(), seat=seat.key)` beside `resolve(…, seat=seat.key)` (C-6).
- `memory_view(store, stack, query, recall_episodes) -> Recalled` calls `host.recall(…, on_defect="withhold")`,
  then applies `recall_episodes` to the `L1` lines exactly as today **[QS28c-5]**.
- `_consult_once` uses `recalled.section`; for each defect, `DecisionLog` writes one
  `{"kind":"scope-defect","seat":…,"consult":n,"scope":…,"error":…}` line before the consult's record. No model
  ever sees a withheld line (I-c-2). The `counters` line gains `scope_defects`.

### 5.9 The Rust reader (`cognition/decision`)

- `RawScope { id: Spanned<String>, parameters: Option<JsonObject> }`, `deny_unknown_fields`; `RawPolicy` and
  `RawMemoryAssign` gain `scopes: Option<Vec<Spanned<RawScope>>>`.
- `MemoryChoice.scopes: Option<Vec<ScopeChoice>>`, `ScopeChoice { id: ScopeId, parameters: Map<String, Value>,
  at: Position }`; `ScopeId` a newtype with `PolicyId`'s pattern (C-10), its spelling refused at the id's own
  position as `memory.<entry>.scopes[<n>].id: …`. Nothing else is judged in Rust (QFGb-1: "passed through, never
  interpreted"): existence, order, duplicates and parameters are the cognition host's.
- `JSON_FORMAT = 3`; `MemoryPolicyChoiceJson.scopes: Option<Vec<MemoryScopeChoiceJson { id, parameters,
  source }>>`, serialized `null` when absent.
- `digest()`: `CanonicalMemory` gains `scopes` with `skip_serializing_if = "Option::is_none"` (F-S28c-11).
- `schema.rs` `memory_fields`: `scopes`, an array of `{ id (pattern), parameters (object) }`, each described.
- `read_tests.rs`: a stack passes through with positions; a misspelled id, a non-list, an entry without `id`, an
  unknown key each refused with line and column; the digest of a file without `scopes` equals C0's recorded value.

### 5.10 The generated reference

- `tools/reference.py` gains a second generator: for each scope of `ScopeRegistry.installed()` whose
  distribution is `mineworld-cognition`, in id order, a section with its parameters' table (the same
  `field_table`), its sections (key, header, default budget), and the ceiling rule; `schema/memory-scopes.schema.json`
  holds `{scope id: Parameters.model_json_schema()}`. An undescribed field stops generation, naming
  `Model.field`; the page's example stack is run through `resolve_stack` and a refused example stops generation.
- `pages.rs`: `COGNITION` lists four files; the index line for `memory-scopes.md` says "the memory scopes each
  seat's stack may name, and their parameters (generated from the cognition package's models)".
- Regeneration: `uv run --locked python cognition/lm-controller/tools/reference.py --out docs/reference` and
  `target/debug/mineworld reference --out docs/reference`; both drift tests compare the checked-in files.

### 5.11 Named errors (`memory/scopes/errors.py`; each message begins `memory.scopes:`; no value is echoed)

`ScopeLoadError(name, distribution, why)`, `DuplicateScope(scope, distributions)`, `UnknownScope(scope,
installed)`, `ScopeParameterError(scope, errors, source)`, `PersonScopeRequired(source)`, `ScopeListedTwice(scope,
source)`, `ScopeNotSelectable(scope, source)`, `TotalSectionsOverCeiling(total, source)`,
`ScopeSectionOverBudget(scope, seat, section, what, bytes, limit)`, `AccessDenied(scope, seat, event)`,
`ScopeRecallFailed(scope, seat, error)`; in `memory/retention/selection.py`: `BoundBelowDefault(seat, given_mib,
source)`, message beginning `memory.retention:`. Every one subclasses `ValueError`, as the home's do.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-c-1 | **Default identity.** With no `scopes` written, and with `[person]` written with default parameters, every rendered section is byte-identical to the base code's (C-2 golden renders, `tests/golden/s28a_renders.json`, all arms) | AC-c-1 |
| I-c-2 (I-S28-1) | No line reaches a `MemorySection` unless every range end it cites is in the seat's citable set; a withheld line is never shown to a model | AC-c-4, AC-c-9 |
| I-c-3 | Every valid stack's rendered section is ≤ 6 000 B; a stack declaring more is refused before the seat plays | AC-c-3, AC-c-5 |
| I-c-4 (I-S28-8) | No scope signature names a connection, path, clock or gateway; an optional scope's view has no connection | AC-c-10 |
| I-c-5 | Selection is a pure function of the decision JSON and the installed registry; every refusal names the scope and the source position and never echoes a value | AC-c-3 |
| I-c-6 | The committed decision-log fixture parses and replays unchanged (AP7-4); a decision file without `scopes` keeps its digest | AC-c-7, AC-c-11 |
| I-c-7 (I-S28-2) | The transparency harness stays green: sharing is invisible under every stack this PR can build | AC-c-11 |
| I-c-8 | Retention, schema 5, every dump and the meter are unchanged (no policy, item rule, bound number or pass order) | the S28-a/S28-b suites; diff scan |
| I-c-9 (I-S28-10) | No path under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` changes | AC-c-12 |

---

## 7. Test ownership (test rules §26)

| Layer | Owns | Where |
| --- | --- | --- |
| Static | ruff, ruff format, pyright strict (the protocol's conformance by `PersonScope`, `WorldScope` and the fixture scopes); `cargo fmt`, `cargo clippy -D warnings` on `mineworld-decision` | CI `fast`, `python`, `test` |
| Unit (Python) | registry load checks and binding (`test_scope_registry.py`); selection and every named error (`test_scope_selection.py`); the host's four checks against a hostile fixture scope and the citable set (`test_scope_host.py`); the person scope's parameters against an oracle computed from the facts (`test_person_scope.py`); `MemorySection` render order and the old log's parse (`test_scope_section.py`) | `cognition/lm-controller/tests/` |
| Unit (Rust) | the reader's pass-through, positions, refusals, format 3, the digest (`read_tests.rs`) | `cognition/decision/src/` |
| Golden | I-c-1: `test_home_golden.py` unchanged, plus the explicit-`[person]` arm | same |
| Integration | the consult withholds a hostile scope and logs it, with the scripted backend (`test_controller_scopes.py`); one real-binary case: `mineworld decisions <fixture world> --json` with a stack, read by `read_assignment`, resolved, a planted error's line and column (`test_scope_selection.py::…_real`, `real_binary`) | same |
| Reference | `test_reference.py` extended (drift, undescribed field, refused example); `tools/cli/tests/reference.rs` (the index links four cognition files) | same; `tools/cli/tests/` |
| Scans | `test_memory_isolation.py` already covers `memory/scopes/`; new: the signature scan of `interface.py`; the writers scan unchanged | same |
| Harness | S28-a's transparency harness (per-PR set) re-run unchanged at C8 | same |
| Gate 1 (real LLM) | **NOT REQUIRED**: no LLM-facing semantics change; the request bytes are unchanged under the default stack, which the golden and the consult test prove offline | — |
| Gate 2 (real lifecycle) | S28-b's shared-home real-binary scenario re-run at C8 as the step's checkpoint; no new Gate 2 claim | — |

The oracles are P4's frozen golden renders, sections recomputed in the test from the facts (never by
`retrieve`), and the dump; no expected value comes from the code under test (test rules §25).

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-c-1 | **Default identity.** Every point of `tests/golden/s28a_renders.json` (the `ingest`, `absorb` and `fgc` arms) reproduces under (a) no `scopes` and (b) an explicit `[{id: person}]`, for every query | I-c-1, C-2 |
| AC-c-2 | **Registry.** `ScopeRegistry.installed().ids() == ("person", "world")` from distribution `mineworld-cognition`; fixture entry points that are not a class, whose `id` differs, whose `Parameters` has a field without default or is not a `ScopeParameters`, whose table is not `<id>_…`, that lack `recall`, or that fail to import are each refused by `ScopeLoadError` naming the distribution; two distributions → `DuplicateScope`; a foreign `person` → refused [QS28c-1] | §5.3 |
| AC-c-3 | **Selection errors (AS28-19's S28-c half).** Each of `ScopeNotSelectable`, `PersonScopeRequired` (empty list; `place`-like fixture first), `ScopeListedTwice`, `UnknownScope` (its message lists every installed id), `ScopeParameterError` (field path and source; a planted value `"s3cr3t-0042"` never appears in the message), `TotalSectionsOverCeiling` (person at default + a fixture scope with a 1-byte section → total 6 001), `BoundBelowDefault` (19 MiB; seat and source named) is raised with its exact type and fields; one case end-to-end through the real `mineworld decisions --json` with the line and column of the YAML entry | AS28-19, AS28-16 (c) |
| AC-c-4 | **Hostile scope (AS28-2).** In a two-seat home, a fixture scope whose lines cite (i) a key the seat never held, (ii) a key only the home-mate holds, (iii) a key the seat forgot (in its forgotten set), (iv) an id never perceived by anyone, (v) a range whose first end is held and whose last end is only the end of one of the seat's episodes (citable for a base scope, not for an optional one) — each raises `AccessDenied(scope, seat, event)` under `raise`, and under `withhold` none of its lines is in the section and the person scope's section is byte-identical to the stack without it; its `GuardedView` has no `connection`, and its `record` of an unheld key raises `AccessDenied` | AS28-2, I-S28-1 |
| AC-c-5 | **Budgets.** A fixture scope returning a 161-byte line, one line more than its `lines`, or a section one byte over its `bytes`, or a line keyed to a section it did not declare, raises `ScopeSectionOverBudget` with the matching `what`; nothing of it reaches the section | step §5.8 |
| AC-c-6 | **Person parameters.** With `L0: {lines: 5, bytes: 2000}` the L0 section has ≤ 5 lines and every other section equals the default's; with `l0_window_s: 3600` every L0 line's fact is within the hour before the query (oracle: the facts' `at`, from the stream); `L3: {lines: 17}` and `L2: {bytes: 801}` are refused by `ScopeParameterError` | §5.4 |
| AC-c-7 | **The reader.** A stack round-trips YAML → `mineworld decisions --json` (format 3) → `read_assignment` → `resolve_stack` with every position; format 2 is refused by name; the digest of each World Pack's decision file, recorded at C0, is unchanged | §5.9, F-S28c-11 |
| AC-c-8 | **Reference.** `memory-scopes.md` and its schema match the generator (drift test); a `PersonParameters` field stripped of its description stops generation naming it; the page's example resolves; `decisions.md` documents `scopes`; the reference index links the four cognition files | §5.10 |
| AC-c-9 | **The consult.** A `SeatRunner` (scripted backend, the controller tests' fake server) whose stack holds a hostile scope after `person` produces the same `CompletionRequest` bytes as one with `[person]`, logs one `scope-defect` line per consult naming the scope, and raises nothing | §5.8 |
| AC-c-10 | **Scans.** `memory/scopes/` passes the isolation scan; no annotation in `interface.py` names `Connection`, `Path`, `Clock`, `ModelGateway` or `SeatView`; the writers scan is unchanged and green | AS28-13, AS28-14 |
| AC-c-11 | **Checkpoint.** S28-a's per-PR transparency set and S28-b's shared-home Gate 2 pass on the final head; `test_milestone_d_replay.py` passes on the unchanged fixture | step §11.1 |
| AC-c-12 | **Diff scan.** `git diff --name-only origin/main...HEAD` lists nothing under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` | I-S28-10 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason. Targeted validation
per commit; the PR's CI is the one full run.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s28c`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full,
   before editing it.
2. Make the numbered steps, in order. Do not do work that belongs to a later commit.
3. Run the commit's validation commands. Record for each: the command, the test count, the wall time, and PASS /
   FAIL / INCONCLUSIVE (a test not run is not a pass).
4. For each mutation the commit lists: plant it, run the named test, record that it turned red (with the failing
   test's name), restore it from a byte copy (`cmp` clean), run the test again green.
5. Static: `uv run --locked ruff check cognition/lm-controller`; `uv run --locked ruff format --check
   cognition/lm-controller`; `uv run --locked pyright`; for Rust commits `cargo fmt -p mineworld-decision --check`
   and `cargo clippy -p mineworld-decision --all-targets -- -D warnings`. Tests one file at a time: `uv run
   --locked pytest cognition/lm-controller/tests/<file>`; Rust: `cargo test -p mineworld-decision`. **R-LOAD-1:**
   never the whole member suite, never `cargo test --workspace` locally. Markdown: `python3
   scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
6. Review: answer the commit's review question in §15 in one or two sentences.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files**; otherwise
   revert it, or stop and record why.
8. Update §15 (ledger row, evidence, findings) and the handoff in the same commit.
9. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` with the message given plus the
   session's attribution line. No heredoc, no `sed -i`, no `python3 -c`.
10. A material stop (§12) is reported, not improvised around.

#### C0 — Base, re-audit and frozen evidence (Markdown only)

- **Goal:** fix the base and capture the evidence later commits compare against, before any code changes.
- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s28c.md` (new).
- Steps:
  1. Record the base; confirm S28-b merged (`git log` shows its merge; `SCHEMA_VERSION = 5`). Not merged →
     material stop.
  2. Re-anchor C-1 … C-19 on the base; a moved line is bounded, a changed statement is a stop.
  3. Record whether #262 and S28-e have merged (QS28c-7).
  4. **Frozen evidence:** build `target/debug/mineworld` (`cargo build -p mineworld-cli`) and record, for every
     `worlds/*/decisions.yaml` and every decision file under `cognition/lm-controller/tests/fixtures/`, the
     `digest` of `mineworld decisions <world> --json` (format 2) in §15.
  5. Write the handoff.
- Validation: the two doc scripts.
- [ ] Implementation. [ ] Validation. [ ] Review: the base satisfies §2.3; the digests are recorded before C2.
- **Commit message:** `docs(plan): S28-c — open the ledger at the base`

#### C1 — The specifications (Markdown only)

- **Goal:** the specification says what this PR builds before it is built (`CLAUDE.md` §2.2).
- **Files:** `docs/MODULE_SPEC.md` (§5.3), `docs/DECISIONS.md` (a dated note under `ARC-100`).
- Steps:
  1. §5.3: `store_bound_mib` and `source` (F-S28c-1); `HomeBusy(path)`; the base scopes and `ScopeView` (QS28c-1
     as ruled); the citable set (QS28c-2); `sections()` and per-section budgets (F-S28c-7); the consult's
     behaviour on a defect (QS28c-3); the table host's owner (QS28c-4); `recall_episodes` (QS28c-5); the errors of
     §5.11; the whole-entry rule (F-S28c-13).
  2. `ARC-100` dated note: the same four rulings in four sentences.
- Validation: the doc scripts.
- [ ] Implementation. [ ] Validation. [ ] Review: every rule of §5 that a reader must know to write a scope is in
  §5.3, and nothing in §5.3 contradicts §5.
- **Commit message:** `docs(spec): memory scopes as built — base scopes, the citable set, sections (S28-c)`

#### C2 — The Rust reader: `scopes`, positions, format 3

- **Goal:** a World Pack can write a stack, and the cognition process receives it with positions.
- **Files:** `cognition/decision/src/{read.rs, resolve.rs, json.rs, schema.rs, read_tests.rs, lib.rs}`,
  `docs/reference/{decisions.md, schema/decisions.schema.json}` (regenerated).
- Steps: §5.9, items 1–5; regenerate with `target/debug/mineworld reference --out docs/reference`.
- Validation: `cargo test -p mineworld-decision`; `cargo test -p mineworld-cli --test reference`; the digests of C0
  recomputed equal. Mutation **M-c-1** (`json()` drops `scopes`) → the pass-through test red; **M-c-2** (`scopes`
  serialized in the canonical form when absent) → the digest test red.
- [ ] Implementation. [ ] Validation. [ ] Review: Rust judges spelling only; no existence or order rule leaked in.
- **Commit message:** `feat(decision): memory stacks in decisions.yaml, passed through as format 3 (S28-c)`

#### C3 — The interface, the registry, `person` and `world`

- **Goal:** scopes exist as typed plug-ins, found and checked like policies.
- **Files:** `memory/scopes/{__init__.py, interface.py, errors.py, registry.py, person.py, world.py}`,
  `cognition/lm-controller/pyproject.toml`, `tests/{test_scope_registry.py, scope_fixtures.py}`.
- Steps: §5.2, §5.3, §5.4, §5.11 (the load and bind errors); `scope_fixtures.py` holds the malformed fixture
  classes and a `fake_points(...)` builder of `EntryPoint`s (as `test_retention_interface.py` builds them).
- Validation: `test_scope_registry.py` (AC-c-2); `uv sync --locked` succeeds (entry points need no lock change).
  Mutation **M-c-3** (the table-namespace check removed) → its test red; **M-c-4** (`include_input=False`
  dropped) → the "never echoes" test red.
- [ ] Implementation. [ ] Validation. [ ] Review: the registry is C-1's shape; the base-scope rule is in one place.
- **Commit message:** `feat(cognition): the MemoryScope interface and ScopeRegistry, with person and world (S28-c)`

#### C4 — Selection: the stack, the ceiling, `BoundBelowDefault`, format 3

- **Goal:** a seat's stack is resolved at start or refused by name.
- **Files:** `memory/scopes/stack.py`, `memory/retention/selection.py`, `controller/assignment.py`,
  `tests/{test_scope_selection.py, fixtures/decisions_format3.json (renamed from decisions_format2.json),
  test_assignment.py, test_retention_interface.py}`.
- Steps: §5.5; `FORMAT = 3`; `resolve(…, seat=…)`; every caller of `resolve` passes the seat (grep `resolve(` in
  `src/` and `tests/`).
- Validation: `test_scope_selection.py` (AC-c-3 including the `real_binary` case), `test_assignment.py`,
  `test_retention_interface.py`. Mutation **M-c-5** (ceiling `>=` for `>`) → the 6 000-exactly test red;
  **M-c-6** (the person-first check skipped) → `PersonScopeRequired` test red.
- [ ] Implementation. [ ] Validation. [ ] Review: refusal order matches §5.5; no message carries a value.
- **Commit message:** `feat(cognition): the memory stack is resolved per seat, with the 6 000-byte ceiling (S28-c)`

#### C5 — The recall host, the person scope's parameters, the section shape

- **Goal:** recall runs through the stack and every line is checked.
- **Files:** `memory/scopes/host.py`, `memory/retrieve.py`, `memory/home/view.py` (`citable`),
  `memory/scopes/person.py`, `tests/{test_scope_host.py, test_person_scope.py, test_scope_section.py,
  test_home_golden.py}`.
- Steps: §5.6, §5.7; `GuardedView` in `host.py`; `test_home_golden.py` gains the explicit-`[person]` arm.
- Validation: the four files (AC-c-1, AC-c-4, AC-c-5, AC-c-6); `test_scope_section.py` parses the first 20 lines of
  `milestone_d.decisions.jsonl` with `DecisionRecord`. Mutations **M-c-7** (the citation check skipped) →
  AC-c-4 red; **M-c-8** (`citable` without `seat = ?`) → AC-c-4 (ii) red; **M-c-9** (`retrieve` ignores `budgets`)
  → AC-c-6 red; **M-c-10** (render puts `sections` before L3) → the order test red.
- [ ] Implementation. [ ] Validation. [ ] Review: no statement in `host.py` or `GuardedView` reads `fact` without
  the seat's access join; the default path is statement-for-statement today's.
- **Commit message:** `feat(cognition): recall through the scope stack, every line checked (S28-c)`

#### C6 — The controller recalls through the stack

- **Goal:** a seat plays with its stack; a defect never reaches a model.
- **Files:** `controller/{seat.py, memory_view.py, log.py}`, `tests/{test_controller_scopes.py,
  test_controller_parts.py}`.
- Steps: §5.8.
- Validation: the two files (AC-c-9); `test_controller_seat.py` and `test_milestone_d_replay.py` one at a time.
  Mutation **M-c-11** (`withhold` drops every scope's section) → AC-c-9 red (the request differs).
- [ ] Implementation. [ ] Validation. [ ] Review: `recall_episodes` still applies only to `L1`; the counters line
  stays parseable by `_status`.
- **Commit message:** `feat(cognition): seats recall through their memory stack and log scope defects (S28-c)`

#### C7 — The generated reference

- **Goal:** every bundled scope's parameters are documented from their models.
- **Files:** `cognition/lm-controller/tools/reference.py`, `cognition/lm-controller/tests/test_reference.py`,
  `tools/cli/src/reference/pages.rs`, `tools/cli/tests/reference.rs`, `docs/reference/{memory-scopes.md,
  schema/memory-scopes.schema.json, README.md}`.
- Steps: §5.10; regenerate both halves.
- Validation: `test_reference.py`, `cargo test -p mineworld-cli --test reference` (AC-c-8). Mutation **M-c-12** (one
  description removed) → the generation-stops test red.
- [ ] Implementation. [ ] Validation. [ ] Review: the page reads for a world author: the ceiling rule and the
  whole-entry rule are stated where a reader looks.
- **Commit message:** `docs(reference): generated memory-scopes page (S28-c)`

#### C8 — Scans and the integration checkpoint

- **Goal:** the step's checkpoint on the final code.
- **Files:** `tests/test_memory_isolation.py` (the signature scan).
- Steps: AC-c-10's signature scan; run S28-a's per-PR transparency files and S28-b's shared-home Gate 2 file, one
  at a time, through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.
- Validation: AC-c-10, AC-c-11. Mutation **M-c-13** (`sqlite3.Connection` in `ScopeView.record`'s signature) →
  the scan red.
- [ ] Implementation. [ ] Validation. [ ] Review: the harness ran unchanged; no file of it was edited.
- **Commit message:** `test(cognition): scope signatures carry no connection; checkpoint (S28-c)`

#### C9 — Close-out

- **Files:** this file (§15), `handoff-s28c.md`, `cognition/lm-controller/README.md` (one line: a world chooses a
  memory stack; link to the generated page).
- Steps: AC-c-12's diff scan; the README line; push `mvp0/pr-s28c`; `gh pr create`; CI on the exact head (Linux,
  macOS, Windows); repair to green.
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S28-c close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs, and parent edits (owner: the primary session)

- **R-S28d-c-1 (on S28-d).** S28-d implements the scope-table host (QS28c-4 as ruled): `TableSpec` DDL in the home,
  `index` called by `SeatView.hold` for each selected optional scope, cascades by `event_key` (in `world.collect`'s
  transaction) and by `seat`, a home table of installed scope tables (schema 6), `scope-removed:<id>`, rebuild on
  reinstall, AS28-14's runtime half, AS28-15; `GuardedView` gains the scope's own-row reads it needs. The
  interface's members do not change.
- **R-S28e-c-1 (on S28-e, or on this PR, whichever merges second; QS28c-7).** `inspect … scopes` reads
  `resolve_stack` from the decision JSON and `ScopeRegistry.installed()`; `inspect … bytes` sums `account` per scope;
  `recall --trace` names a line's scope from `SectionLine.level` (`L*` → `person`).
- **R-S28f-c-1 (on S28-f).** `verify` checks Σ `account` = `content_size()` per seat (F-S28c-16).

| Document | Edit |
| --- | --- |
| `step-28-shared-memory.md` §5.3 | `sections()`; the base scopes and `ScopeView` (QS28c-1); `store_bound_mib`, `source` (F-S28c-1) |
| `step-28-shared-memory.md` §5.5 | `ScopeListedTwice`, `ScopeNotSelectable`, `ScopeRecallFailed`; `BoundBelowDefault(seat, given_mib, source)` |
| `step-28-shared-memory.md` §5.3, §5.5 | `AccessDenied` against the citable set (QS28c-2); "withhold and log" (QS28c-3) |
| `step-28-shared-memory.md` §11 | the table host moves to S28-d (QS28c-4); the `scopes` view's owner (QS28c-7) |
| `overall.md` | S28-c frozen / merged |

---

## 11. R-LOAD-1

Locally: only the commit's test files, **one at a time**, plus ruff, ruff format, pyright, `cargo fmt`, `cargo
clippy` and `cargo test` of `mineworld-decision` alone, and the doc scripts. Never `cargo test --workspace`, never
the whole `cognition/lm-controller` suite. The harness and Gate 2 files at C8 go through
`/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`, one at a time, with the load average recorded beside
the wall time. The PR's CI on the exact head is the gate and the one full run.

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S28-c — memory scopes: interface, registry, person and world, the stack
                         in decisions.yaml, parameters, errors, the ceiling, the generated reference
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s28c-memory-scopes.md
RELATED / BINDING DOCS:  step-28-shared-memory.md (§§3, 5, 11, 14.1, 15); pr-s28a-memory-home.md (§10);
                         pr-s28b-retention-on-seat-views.md (§§2.2, 10); pr-fg-b-retention-host.md (the registry);
                         docs/MODULE_SPEC.md §§4.1, 5.2, 5.3; docs/ARCHITECTURE.md §9.3; docs/MVP.md §9 AC-18;
                         docs/DECISIONS.md ARC-85, ARC-98…ARC-101, ARC-109; .structured-coding/standards.md;
                         CLAUDE.md; docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s28c, its own, held by one session
BRANCH:                  mvp0/pr-s28c, created from origin/main
START CONDITION:         S28-b merged AND this design frozen, then the primary session's dispatch  (step §11.1)
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–10, as §14's rulings shape them                     (unresolved until §14
                                                                                          is ruled)
FROZEN INVARIANTS:       step-28 I-S28-1, I-S28-2, I-S28-7, I-S28-8, I-S28-10; §6 I-c-1 … I-c-9; FGb-1 … FGb-7,
                         FG-b2's numbers and S28-b's schema 5 unchanged; §8 as written
SEQUENCE:                C0 … C9 (§9)
ALLOWED COMMANDS:        uv run --locked *; uv sync --locked; cargo build -p mineworld-cli; cargo test -p
                         mineworld-decision; cargo test -p mineworld-cli --test reference; cargo fmt / clippy
                         -p <those crates>; target/*/mineworld *; git; gh (never merge); python3 scripts/*;
                         mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or the
                         whole member suite locally (R-LOAD-1); reading any key or .env; calling any model; any
                         record mode for cassettes; editing a golden or a cassette; edits outside APPROVED SCOPE
MATERIAL STOPS:          S28-b not merged; AC-c-1 failing for a reason other than a defect of this PR (the
                         default stack would change what a person recalls); a needed change to a policy, an item
                         rule, a bound number, the pass order, schema 5, a dump or the meter; a needed change
                         under kernel/, contracts/, server/, persistence/, systems/, worlds/, sdk/; a new
                         dependency or a lock change; the milestone D fixture failing to parse; a digest of C0
                         changing
PLATFORMS:               Linux, macOS, Windows (python and Rust jobs on all three)
VALIDATION BUDGET:       unit files unrestricted locally one at a time; the real-binary selection case ≤ 30 s on
                         Linux CI; the C8 harness and Gate 2 within S28-a's and S28-b's own budgets; CI: the PR's
                         runs. Gate 1: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s28c.md
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
STOP CONDITION:          S28-c READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-c-1 | The widened `level` lets a non-person line be mistaken for an L1 line by `memory_view` | a non-person key is a scope id, never `L1`; `SectionKey`'s pattern makes `L0` … `L3` reserved; a test keys a fixture scope `l1` (lower case is an id) and checks it is not trimmed |
| R-c-2 | The citable-set query per check slows a consult | one query per scope per consult over the line's range ends (≤ 76 lines, ≤ 3 ends each), served by `access`'s and the units' primary keys; AC-c-11 runs the per-PR harness within its budget |
| R-c-3 | A Haiku implementer edits the golden or a cassette to turn a test green | the contract's NEVER list; AC-c-1 is a material stop; the primary reviews the diff of `tests/golden/` and `tests/cassettes/` (must be empty) |
| R-c-4 | Format 3 breaks an operator's saved decision JSON | the JSON is regenerated by `mineworld decisions --json` at every start (it is not a user file); format 2 is refused by name with the command that regenerates it |
| R-c-5 | The base-scope privilege grows over time | it is one tuple and one load check; any third base scope is a change to this design and to MODULE_SPEC §5.3 |
| R-c-6 | S28-b changes `selection.py` or `view.py` in a way this design did not see | C0 re-anchors; S28-b's frozen scope does not list `selection.py` |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

Ruled by the operator on 2026-10-18 in the primary session: every question answered with its recommendation.

| Id | Question | Recommendation |
| --- | --- | --- |
| **QS28c-1 [operator — "none is privileged" and "reads only through the view"]** | The person scope's recall is six seat-filtered SQL queries on the home's connection (F-S28c-5). The step wants every scope to read only through an access-filtered view, and no bundled scope to be privileged. Both cannot hold for `person` without rewriting retrieval. Which way? (a) `person` and `world` are **base scopes**: required, implemented by the memory package, given the `SeatView`; every other scope gets a `GuardedView` with no connection. `world` is never written in a stack (`ScopeNotSelectable`). (b) Rewrite retrieval onto typed `ScopeView` methods, so every scope, `person` included, reads the same narrow view. (c) Keep the connection and guard it with SQLite's authorizer. | **(a).** `person` and `world` already differ from every other scope: one is required, the other cannot be uninstalled. Their code is covered by the evidence that already proves seat isolation (the seat-predicate scans, the golden renders, the transparency harness, M-a-5). The privilege is one named tuple and one load check. (b) rewrites the code that I-c-1 must keep byte-identical, for no change a person can see. (c) can refuse writes and whole tables, but not another seat's rows. |
| **QS28c-2 [operator — what `AccessDenied` checks]** | Lines cite ranges, not keys (F-S28c-3). An episode's range covers records merged into it; a stable fact cites records long released. A literal "held keys" check would refuse every surviving L1, L2 and L3 line. What may a line cite? | **The citable set:** for a base scope, the seat's held keys plus the range ends its own surviving units cite (episodes, chapter ranges, stable facts); for any other scope, held keys only, at both ends of every range. This still catches a line that cites another seat's units or a fact the seat never held (M-c-8). It cannot widen access: a unit exists only from facts the seat perceived. |
| **QS28c-3 [operator — a defect at consult time]** | P6 has no "internal error" fallback (F-S28c-4). When a scope's line fails a check during a real consult, what happens? (a) Raise: the seat stops. (b) Withhold that scope's whole section for this consult, log a `scope-defect` line, and keep playing. (c) Drop all memory for the consult. | **(b).** The model never sees the line (I-S28-1 holds), the person keeps playing with the other sections, and the log names the scope, the seat and the key, so the defect is found in one step. Tests and the host's own checks use `raise`. Withholding is deterministic, so a replay withholds the same scope. |
| **QS28c-4 [operator — scope tables in S28-c or S28-d]** | The step gives S28-c the host machinery for scope tables: creating `<id>_*` tables, inserting `index` rows, cascades, `scope-removed`, rebuild on reinstall (F-S28c-17). No scope in S28-c has a table: `person` and `world` own the base tables. Build it now, tested only with fixture scopes, or with its first user? | **Move it to S28-d.** S28-c defines `index` and `tables` in the protocol and checks them at load, so S28-d adds no member to the interface. S28-d builds the host with its first real user (`place`/`item`) and with AS28-15, and takes the one schema bump the host needs (a table of installed scope tables). This avoids an abstraction with no user (`CLAUDE.md` §4 item 11), and it makes S28-c a clean Haiku-sized PR. |
| **QS28c-5 [operator — two knobs for one section]** | `person.sections.L1.lines` (a world's) and `[controller] recall_episodes` (the operator's, P6 PA-11) both bound the L1 lines (F-S28c-15). Keep both, or retire one? | **Keep both, and apply the operator's cap after the world's:** the seat shows `min(world lines, recall_episodes)` episode lines. That is today's behaviour exactly. Both references state the rule. Retiring `recall_episodes` would change P6's configuration contract, which belongs to S28-f's wiring review, not to this PR. |
| **QS28c-6 [primary — an error the step does not list]** | A stack listing the same id twice has no named error (F-S28c-9). The step's `DuplicateScope` means two distributions. | **Add `ScopeListedTwice(scope, source)`.** It follows the rule that every refusal has its own name, and it never echoes a value. |
| **QS28c-7 [primary — who adds the `scopes` view]** | S28-e (designed in parallel, `pr-s28e-memory-inspect.md`) may merge before or after this PR. `inspect … scopes` and per-scope bytes need this PR's registry and stack. | **Whichever of S28-c and S28-e merges second adds them** (R-S28e-c-1; the R-P6b-a-1 precedent). The second PR's C0 records which case holds. |

---

## 15. Ledger (live during implementation)

Execution: impl session (Opus), worktree `/Users/yuema137/mineworld-worktrees/impl-s28c`, branch `mvp0/pr-s28c`.
**Operator SPEED FIRST (2026-10-18)** governs validation: quick local checks only (ruff, ruff format, pyright, the
test files written or directly broken, one at a time; `cargo test`/`clippy`/`fmt` of the one crate touched). Every
mutation (M-c-1 … M-c-13), the independent review and the C8 harness / Gate 2 re-runs are **deferred to milestone
(operator SPEED FIRST 2026-10-18)**; CI on the exact head and the milestone runs are the gate.

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 base, re-audit, digests | (this commit) | [x] | [x] doc scripts | [x] base recorded; digests captured before C2 |

### Evidence

- **Base (C0).** `mvp0/pr-s28c` at `2c13d9c9` = S28-b C0–C3 (`65936053`, `f9e20e81`, `70a7c0af`, `e97762fd`) plus a
  merge of `origin/main` (`5b576413`). `SCHEMA_VERSION = 5` (`memory/home/schema.py:23`). S28-b is **not merged**:
  see D-c-0. #262 (home marker) **merged** (`73e54621`). S28-e is **not merged** (branch `mvp0/pr-s28e` exists
  locally): under QS28c-7 the `scopes` view and per-scope bytes are S28-e's if it merges second.
- **Re-anchor.** C-1 (`retention/registry.py:39-47`, `:50-98`, `:162-216`), C-2 (`retention/selection.py:36-46`,
  `:90-116`), C-5, C-6 (`controller/seat.py:169-173`, `:511-521`), C-7, C-9, C-10, C-11, C-13, C-14, C-15: lines
  moved, statements unchanged. **C-3 changed** (D-c-1).
- **Frozen digests (C0, `target/debug/mineworld decisions … --json`, format 2).** No World Pack ships a
  `decisions.yaml`; each digest is the pack's default resolution:
  `worlds/bodies-yard` `db30c571c952c581`; `worlds/market-town` `26c292f374f03fcb`; `worlds/social-cafe`
  `26c292f374f03fcb`; `worlds/lakeside` not resolvable without its `requires:` pack (no `--packs` dir in the
  tree; it has no decision file). On `worlds/social-cafe`: `examples/alice-lm.decisions.yaml` `fdff93fd32455f5e`;
  `tools/cli/tests/fixtures/decisions/rule-at-boundary.yaml` `7d25db54216ac73e`;
  `rule-at-boundary-strict-boss.yaml` `ac4e8532e7e4048e`; a list with a `memory:` section and no `scopes`
  (default `{policy: consolidate, store_bound_mib: 40}`, `bob: {policy: decay, parameters: {half_life_hours: 336}}`,
  alice `lm`) `112c208bf894f528`. (`memory.yaml` needs the decisions.rs temp world with classes; its golden
  substitutes the digest.)

### Mutations

Deferred to milestone (operator SPEED FIRST 2026-10-18): M-c-1 … M-c-13.

### Deviations and findings

F-S28c-1 … F-S28c-17 (§3.2), recorded at design time.

- **D-c-0 (bounded; lead's dispatch).** Previous assumption: base `origin/main` after S28-b merged (§2.3, §12).
  Evidence: S28-b's C0–C3 are implemented, not merged; its C4–C7 (tests only) continue in another worktree.
  Corrected: this branch is stacked on `mvp0/pr-s28b` @ `2c13d9c9` for speed. Consequence: the lead rebases or
  merges after S28-b lands; this session does not merge main unless a conflict forces it.
- **D-c-1 (bounded).** Previous assumption (C-3): four person sections, L0 48 / 2 000. Evidence:
  `memory/retrieve.py` `BUDGETS` now has five: `with` (6 / 1 000, header `What {name} and I said to each other
  before:`, pr-p6c-fgd), L3 16 / 1 600, L2 4 / 800, L1 8 / 1 600, L0 48 / 1 000; sum still 6 000. Corrected: the
  person scope declares **five** sections, its defaults are today's `BUDGETS`; `with` is a reserved person section
  key beside `L0` … `L3` (a scope whose id is `with` is refused at load); the person scope's sections render in
  today's order (`with`, L3, L2, L1, L0). Consequence: `PersonSections` gains `with`; I-c-1 unchanged.
