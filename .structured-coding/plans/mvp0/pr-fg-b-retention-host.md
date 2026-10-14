# PR FG-b — The retention host: `ForgettingPolicy`, the forgotten set, bundled no-LLM policies, ledger pruning

## DESIGN FROZEN 2026-10-12 (primary session; operator QFGb-3, QFGb-6)

```text
Design revision:        revision 2 (2026-10-12): revision 1 (PR #180, commit 7e7540a) with §14.1's rulings
                        recorded and §12's contract filled
Approved by / evidence: the operator's rulings on QFGb-3 and QFGb-6 and the primary session's on QFGb-1,
                        QFGb-2, QFGb-4, QFGb-5, QFGb-7 … QFGb-11, of 2026-10-12, relayed by the coordinator
                        (§14.1)
Implementation base:    origin/main at the start of implementation, with DOC-24 merged (C0 records it)
Execution contract:     §12
Lifecycle:              FROZEN. Implementation starts only on the operator's go, in a fresh session.
Dependencies:           DOC-24 merged (ARC-87, CORE_CONCEPTS §5.3) before C0; DL-a for the typed JSON
                        MemoryPolicyChoice it emits (QFGb-1) — FG-b does not wait for DL-a (its tests build
                        the JSON literally); P6 wires the two (R-P6-FG-2)
```

Scope (§2.1), invariants (§2.3), the decisions of §5 as ruled, the acceptance criteria AFGb-1 … AFGb-14 and
the commit plan are frozen. Progress, evidence, findings and bounded corrections stay writable (§15).

*Superseded header:*

```text
Lifecycle:              DRAFT (2026-10-12). Not frozen; nothing here authorizes implementation
                        (CLAUDE.md §3.1). Freeze needs the operator's / primary session's rulings on §14
                        and an approved DESIGN FROZEN header.
Author:                 FG-b/FG-c design agent, 2026-10-12. Worktree
                        /Users/yuema137/mineworld-worktrees/design-fg-bc, branch docs/fg-bc-design, from
                        origin/main @ cda4c13 (#174). One writer.
Planning base:          origin/main @ cda4c13
Placement:              MVP-0, BEFORE P6 (step-24 §8.6, §11.1; operator ruling 2026-10-12)
Sibling design:         pr-fg-c-lm-summarise.md (FG-c, the LLM policy; alongside P6)
```

**Effort:** `mvp0` (filed here under QS24-14). **Step:**
[`../mvp1/step-24-decision-and-memory.md`](../mvp1/step-24-decision-and-memory.md) (approved 2026-10-12;
§§1.2, 2.3–2.5, 3, 8.3, 8.4, 8.6, 10, 11, 14.1). **Parent:** [`overall.md`](overall.md) (S24 entry of
2026-10-12; decision table row "Step-24").
**Predecessors (merged):** P4 [`pr-s10-p4-memory.md`](pr-s10-p4-memory.md) (#143); P5a
[`pr-s10-p5-backends.md`](pr-s10-p5-backends.md) (#120); P3b
[`pr-s10-p3b-perceived.md`](pr-s10-p3b-perceived.md) (#157).
**Prerequisite not yet merged:** DOC-24 (step-24 §11.2), which writes `ARC-87` ("controller-memory
retention: host-computed importance, event-time recency, forgotten set, hard bounds") and the
`CORE_CONCEPTS.md` §5.3 forgotten-set sentence. FG-b cites `ARC-87`; it does not write it.
**Parallel lanes it touches:** DL-a (owns `decisions.yaml`; §6 states the `memory:` shape FG-b assumes and
flags it, QFGb-1); P6 (consumes the host through `absorb`, §11); FG-c (adds the assisted branch to this host).
**Decision ids:** none new. FG-b implements `ARC-87`; adopts no dependency (the policy registry uses the
standard library's `importlib.metadata`), so no `DEP` record is needed (§4).

### Binding requirements this design is written under

| Source | Requirement | Where met |
| --- | --- | --- |
| Operator, 2026-10-12 (step-24 §1.2 ruling 1) | Forgetting is MVP-0; storage, memory and tokens must not explode. P4's QP4-11 is superseded. | the whole PR |
| Operator (step-24 §1.2 ruling 2) | Typed retention interface (items with typed metadata and a budget in; keep / drop / merge out); bundled implementations are ordinary packs, none privileged; selection and parameters per world and per person, typed and validated; hard bounds enforced by the host outside any rule; no-LLM rules are pure functions. | §5.2–§5.9, §6 |
| step-24 §10.1, QS24-4 (accepted) | Per-seat store ≤ 1 MiB (canonical dump); ≤ 6 000 B of memory per model call (P4 D-P4-9, unchanged); forgotten Event IDs recorded and resolvable. | §5.7, §5.8, §5.10 |
| step-24 AC-FG-1, AC-FG-3, AC-FG-4 (no-LLM half) | Bound under every bundled policy and the hostile `keep-all`; coverage at every pass; rebuild determinism. | §9 AFGb-3 … AFGb-6 |
| QS24-10 (accepted) | `keep-all` only behind the host bound; a test fixture. | §5.9 |
| QS24-11 (accepted), `ARC-87` | Recency by event time, never by retrieval. | §5.4 |
| P4 I-5, P4-2, D-P4-8; `ARC-59` | Memory is a derivation of what the seat perceived; the cursor is never ahead of what is durable; determinism is on the canonical dump. | §5.6, §5.11 |
| P3b D-P3b-2, R-P6-P3b-1 | The memory store is the seat's `CursorSource`; the only cursor authority. | §5.11 |
| `CLAUDE.md` §4 rules 3, 7, 10, 11, 12, 16 | Typed boundaries; LM-independent deterministic tests; no premature abstraction; no compatibility debt before a stable contract; reuse before reinvention. | §4, §5, §8 |
| Platforms (operator, 2026-10-08) | macOS, Linux, Windows. | AFGb-12 |
| QS10-19 | Agents never call a hosted model, never use a key, never run or download a model. | FG-b calls no model at all |

---

## 1. Goal, in one paragraph

P4 gave each LM seat a memory store that is a derivation of what the seat perceived, with bounded
retrieval but an unbounded store (no `DELETE` anywhere in `memory/` or `compress/`). FG-b adds the
**retention host**: once per simulated day, at a point fixed by the perceived facts alone, it shows the
store's eligible items — with host-computed importance, event-time recency, citations and byte sizes — to
a pluggable **`ForgettingPolicy`**, validates the policy's keep / merge / drop plan, executes it so that
every perceived fact stays represented exactly once (by a surviving L0 record, episode or chapter, or by
the seat's **forgotten set**), and then enforces the **1 MiB per-seat bound** itself, whatever the policy
did. Four no-LLM policies ship as ordinary entry-point plug-ins — `consolidate` (the default), `decay`,
`importance-recency-cap`, and the hostile fixture `keep-all` — and a user's policy is a Python package
with one entry point. The P5 budget ledger stops growing. Forgetting changes no byte of any save, no
cursor, and no citation's resolvability: a forgotten Event ID still resolves in the save
(`mineworld perceived`). No model is reachable in any FG-b test or code path.

---

## 2. Scope

### 2.1 In scope

- `cognition/lm-controller/src/mineworld_cognition/memory/`: store schema v2 (§5.5); ingestion's
  importance, involvement and forgotten-re-delivery rules (§5.4, §5.11); retrieval made independent of
  dropped rows and bounded in what it reads (§5.10, F-S24-2).
- `memory/retention/` (new): the typed interface, item extraction, the plan validator and executor, the
  forgotten set, the host bound, the pass log, the policy registry, the `memory:` selection model, and the
  `absorb` driver.
- `mineworld_cognition/forgetting/` (new): the bundled policies `consolidate`, `decay`,
  `importance-recency-cap`, `keep-all`, registered as entry points of `mineworld-cognition`.
- `compress/chapters.py`, `compress/stable.py`, `compress/episodes.py`: the columns schema v2 adds; chapter
  citations as runs of surviving episodes.
- `budget.py` `SqliteLedger`: pruning (G9, §5.12).
- `memory/render/movement.py` (new; QFGb-6, accepted by the operator 2026-10-12): `walk-started` /
  `walk-ended` rendered `ambient` unless mine.
- Tests (§8), including a deterministic synthetic perceived stream and a real-binary scenario sharing the
  AC-10 save.
- `docs/MODULE_SPEC.md` §5 (memory policies and their host bounds) and `docs/ARCHITECTURE.md` §9 (controller
  memory retention), the two edits step-24 §13 assigns to FG-b. `cognition/lm-controller/README.md`: a few
  lines.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| `lm-summarise`, `AssistedForgettingPolicy`, the summary gateway | FG-c |
| `decisions.yaml` itself, its Rust reader, class resolution, `--decisions` | DL-a (QFGb-1 states the JSON form FG-b consumes) |
| Calling `absorb` from a live seat, persisting `welcome.resume`, the decision log | P6 (R-P6-FG-1, §11) |
| System-owned forgetting (relationships decay and cap, conversation age-out) | FG-a |
| World-log compaction (SR-b) and cold archive | FG-d, FG-e |
| `ARC-87`, `CORE_CONCEPTS.md` §5.3, the QP4-11 superseded note in `pr-s10-p4-memory.md` | DOC-24 |
| A real 300-day real-binary run in CI | nightly, with FG-d's AC-SAVE-1 layer (QFGb-5) |
| Any Rust, server, protocol, SDK, World Pack or System change | none: a material stop (§13) |

### 2.3 Invariants this PR must hold

`I-n` and `P4-n` are P4's (still binding); `FGb-n` are this PR's.

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-1, I-2, I-3, I-10, I-11 | As P4: no kernel/contract change; nothing depends on `mineworld_cognition`; memory reads only what it is given; no provider concept in memory or its dump; no network but loopback in tests. | AFGb-13; P4's isolation and provider scans, extended to `memory/retention/` and `forgetting/` |
| I-5 | A store rebuilt from the same perceived facts has a byte-identical canonical dump, whatever the frame boundaries — **with retention running**. | AFGb-5 |
| I-12 | Every citation of a surviving unit resolves to a fact the seat perceived. | AFGb-3 |
| I-13 | The memory section is ≤ 6 000 B, independent of age (D-P4-9 unchanged). | AFGb-7 |
| P4-2 | The store's cursor is never ahead of what it durably ingested; retention never writes `cursor` or `newest`. | AFGb-6 |
| FGb-1 | **Coverage.** At every pass, every perceived id is represented by exactly one top-level unit — a surviving chapter's range, a surviving chapterless episode, an L0 record of the open episode — or by the forgotten set; nested units lie inside their container; no surviving unit's citation intersects the forgotten set. | AFGb-3 (oracle from the export) and the host's own structural check |
| FGb-2 | **The bound.** After every pass the canonical dump is ≤ `STORE_BOUND` (1 048 576 B), whatever the policy returned, raised or kept. | AFGb-4 with `keep-all` and hostile plans |
| FGb-3 | **Policies decide, the host writes.** A policy never receives a store, a connection, a clock, a path or a gateway; it receives frozen values and returns decisions. Only the host deletes rows. | AFGb-9 (source scan of `forgetting/`), AFGb-2 |
| FGb-4 | **No policy is privileged.** The bundled policies are found through the same entry-point group as a user's and use only `memory.retention`'s public names. | AFGb-1, AFGb-9 |
| FGb-5 | **The pass is a function of the facts.** A pass runs at a point fixed by the perceived sequence (§5.6) and is a pure function of the store's state there and the policy's parameters. | AFGb-5 |
| FGb-6 | **Truth is never lost.** Retention deletes only rows of the seat's own store; the save is never opened; every forgotten id is a perceived id and resolves through `mineworld perceived`. | AFGb-3 (d) |
| FGb-7 | **Without `absorb`, P4 is unchanged in behaviour.** `ingest` alone never forgets; `test_ac10.py`'s criteria hold unchanged. | AFGb-14 |

---

## 3. Audit (`origin/main @ cda4c13`)

### 3.1 Source anchors

| File / symbol | Finding | Consequence |
| --- | --- | --- |
| `memory/store.py` `_SCHEMA`, `SCHEMA_VERSION = 1` (`records.py`) | Tables `meta`, `l0`, `l0_who`, `l0_text` (FTS5), `episodes`, `episode_who`, `chapters`, `stable`, `names`. No `DELETE` statement in `memory/` or `compress/`. `journal_mode` default; no `auto_vacuum`. | The store only grows (step-24 G5–G7). Deleting rows does not shrink the file without `auto_vacuum` (§5.5). |
| `memory/store.py` `_check_identity` | A different `schema` is refused by name (`StoreIdentityMismatch`). | Schema v2 refuses v1 stores by name; recovery is a rebuild from a null cursor, because memory is a derivation (QFGb-7). |
| `memory/store.py` `dump()`, `_DUMPED` | Canonical JSON Lines of every table but `l0_text`; this is what determinism and the bound are measured on. | FG-b's new tables join `_DUMPED`; the bound is measured on `dump()` (step-24 §10.1). |
| `memory/ingest.py` l. 77–84 | A fact not held and at or below the stored bound raises `OutOfOrder`. | **F-FGb-4:** after retention removes an L0 row, a legitimate re-delivery of that id would raise. §5.11 refines the rule. |
| `memory/retrieve.py` `_stable_text` l. 155, `_at` l. 163 | The L3 line reads `l0.at` of the counterpart's `first_key` and `last_key`, and `fetchone()[0]` crashes when the row is gone. | **F-FGb-2:** retrieval breaks after the first L0 row that an L3 line cites is removed. Schema v2 stores `first_at`, `last_at` on `stable`. |
| `compress/chapters.py` `chapters()` l. 48–52 | A chapter's citation is computed by joining `episodes` on `first_episode` / `last_episode`. | **F-FGb-3:** merging a chapter's episodes into it would break its citation. Schema v2 stores the chapter's ranges. |
| `compress/chapters.py` `close_weeks` | A week's chapter is written when a closed episode exists in a later week; its span is first-to-last episode. | If an earlier episode of that week was dropped (forgotten) before the chapter was written, a single span would overlap the forgotten set. Chapter ranges become runs of consecutive surviving episode numbers (§5.5). |
| `compress/stable.py` `stable_facts()` | Reads every row per retrieval (F-S24-2). | `LIMIT` to the section's line count (§5.10). |
| `memory/retrieve.py` `_matching` / `_episodes_matching` | Lexical matching joins `l0` rows to episodes; an episode whose L0 rows are gone can no longer match. | Retrieval also matches the episode's own text (§5.10); bounded because the store is bounded. |
| `memory/render/__init__.py` `Rendered`, `generic` | The generic renderer is `notable`; renderers return no importance. | Renderers gain a base importance (§5.4); unknown types get the generic one (I-9 holds). |
| `compress/model_summarizer.py` `embellish` | Writes `prose`, `prose_key` of closed summaries; never on the ingest path. | Unchanged. A merged container keeps its prose. |
| `budget.py` `SqliteLedger`, `precheck` | One row per call, never deleted. Windows are **wall** time: calls per wall hour, tokens per wall day (`HOUR_S`, `DAY_S`, "nothing here knows simulated time", P5-3). | G9's pruning is by wall time; step-24's "one simulated day" is corrected (QFGb-9). |
| `gateway.py`, `config.py` | `Router.for_tier("summarize")`; nothing in FG-b uses them. | FG-b calls no model. |
| `cognition/lm-controller/pyproject.toml` | No `[project.entry-points]`; dependencies `mineworld-sdk`, `pydantic`, `httpx2`, `python-dotenv`. No YAML parser in any Python member (`uv.lock`). | Entry points are standard metadata (no dependency). `decisions.yaml` cannot be read in Python without a new dependency (QFGb-1). |
| `tests/test_ac10.py`, `ac10_harness.py` | A module-scoped fixture runs the 100-day social-cafe world; Windows took 233–243 s against a 300 s ceiling (P4 §14.5–14.6). | FG-b's real-binary test shares that save through a session-scoped fixture rather than running the world again (§8.4). |
| `tests/test_memory_isolation.py`, `tests/test_provider_scan.py` | Scope: `memory/`, `compress/`. | Extended to `memory/retention/` and `forgetting/`. |
| `.structured-coding/plans/mvp0/` | No DL-a design exists yet (a `design-dl` worktree holds no document at `cda4c13`). | §6's `memory:` shape is FG-b's assumption, flagged for DL-a (QFGb-1). |

### 3.2 Planning measurement (scratch; sizes the design; not evidence for any criterion)

The step's per-day figure (195 KB per seat, from P4's 100-day store of 2026-10-10) predates S15's walking
(12n-2, `ARC-75`). Re-measured on this base, macOS arm64, with the `mineworld` dev binary built from the
primary checkout, scratch outside the repository:

```text
mineworld run worlds/social-cafe --headless --seed 7 --days 30 --save S   84 639 facts, 5.9 s
mineworld perceived … --person alice --json                               37 441 facts (1 248 / day), 20.0 MB
ingest in frames of 256 into ":memory:" (P4 code as merged)               2.0 s; canonical dump 12 996 459 B

  table         rows     bytes        per row
  l0            37 441   10 276 249   274
  l0_who        37 664    2 553 088    67
  episodes         288      104 183   361
  episode_who    1 139       58 573    51
  stable            11        2 006   182
  names             12        1 067    88
  chapters           4          960   240
  meta               6          333    55

  L0 + who per simulated day: min 392 180, mean 427 644, max 460 369 B
  episodes 9.6 / day, 565 B each with who rows (5 425 B / day); chapters 240 B each
  l0 by class: notable 19 427 rows (5.68 MB), of which mine 7 620 (2.04 MB); ambient 10 394 (2.56 MB)
  l0 by kind (bytes): arrived 3.17 MB · spoke 2.10 MB · walk-ended 1.84 MB · walk-started 1.80 MB ·
                      conversation-started 0.35 MB · group-activity-* 0.62 MB · the rest < 0.2 MB each
```

**F-FGb-1.** A seat's store now grows ≈ **433 KB per simulated day** (2.2 × step-24 §2.3's 195 KB), almost
all of it L0. **F-FGb-5.** `walk-started` and `walk-ended` (13 430 rows, 36 % of L0 bytes) have no renderer,
so the generic renderer files other people's walks as `notable` — they fill L0 sections and would carry
notable importance (QFGb-6). Consequences: (a) step-24's `consolidate` default `l0_days: 2` cannot hold the
bound (2 × 460 KB of L0 alone, plus a day's growth); §5.9 sets detail retention in hours with default 0 and
shows the arithmetic; (b) the bound must be stated at pass points, since one day's growth alone is ≈ 45 % of
1 MiB (QFGb-2).

**Note, 2026-10-12 (primary session's instruction) — step-24's audit figure re-measured.** Step-24 §2.3
computes the P4 store's growth as "≈ 195 KB per seat per simulated day" (from P4's 100-day store of
2026-10-10, before walking). Re-measured above on `origin/main @ cda4c13`: **≈ 433 KB per seat per simulated
day** (12 996 459 B over 30 days; L0 with its participant rows 392–460 KB per day). Step-24's derived
figures scale with it: ≈ 130 MB per seat over 300 days, ≈ 1.4 GB for 11 seats of social-cafe, without
forgetting. Recorded here, not in step-24, by instruction; the step's conclusions (the store explodes first;
FG-b before P6) hold more strongly.

---

## 4. Reuse

Step-24 §7 already decided the methods (Generative Agents' recency × importance, Murre & Dros' forgetting
curves, reservoir / priority sampling) are adopted as methods, not code, and that P4's store stays
(`DEP-37`). FG-b re-examines only what it adds:

| Need | Option | Verdict |
| --- | --- | --- |
| Plug-in discovery | `importlib.metadata` entry points (standard library, PEP 621 `[project.entry-points]`) | **ADOPT** (step-24 §8.3). No loader of our own, no dependency. |
| Plug-in discovery | `pluggy` (pytest's), `stevedore` | **REJECT**: hook-calling frameworks; we need one lookup by id, which entry points already give. |
| Retention scoring | Generative Agents' scores; Ebbinghaus / Murre & Dros curves; A-Res keys | **ADOPT THE METHOD**, integer forms (§5.9); the "random" key of A-Res becomes a sha256 of the seed and the first Event ID, so a policy stays a pure function. |
| Memory frameworks with eviction | Letta / MemGPT tiers, mem0, LangChain summary buffers | **REJECT** (as step-17 §4.3, step-24 §7): none is a derivation of a perceived log with retained ids and byte-exact rebuild. |
| File-size reclamation | SQLite `auto_vacuum = INCREMENTAL` + `PRAGMA incremental_vacuum` | **ADOPT** (SQLite's own mechanism; `VACUUM` rewrites the whole file and is not needed). |
| YAML for `memory:` in Python | PyYAML, ruamel.yaml | **NOT IN FG-b**: a second YAML parser of one file risks dialect differences (PyYAML is YAML 1.1: `on`, `no` become booleans) with the Rust reader DL-a uses (`serde-saphyr`). QFGb-1. |

---

## 5. Design

### 5.1 Package layout (additions to P4's tree)

```text
cognition/lm-controller/
  pyproject.toml                    + [project.entry-points."mineworld.forgetting"] (four lines)
  README.md                         + a short "Forgetting" paragraph
  src/mineworld_cognition/
    memory/
      store.py                      schema v2 (§5.5): new columns and tables; auto_vacuum; _DUMPED
      records.py                    SCHEMA_VERSION = 2; ItemId; Chapter.citation from ranges
      ingest.py                     importance and involved columns; forgotten re-delivery (§5.11);
                                    IngestReport.forgotten_redeliveries
      retrieve.py                   no l0 reads for L3; LIMITs; matching on episode text (§5.10)
      render/__init__.py            Rendered.importance; generic base importance (§5.4)
      render/*.py                   each plug-in sets its base importance
      render/movement.py            (QFGb-6) walk-started / walk-ended
      retention/
        __init__.py                 the public names a policy may import (§5.2)
        policy.py                   MemoryItem, RetentionBudget, Keep / Merge / Drop, ForgettingPolicy
        items.py                    extraction of eligible items; importance at the pass; bytes
        plan.py                     validation (§5.7.1) and execution (§5.7.2)
        forgotten.py                ForgottenSet: ranges, adjacency merge, watermark (§5.8)
        bound.py                    the host's backstop (§5.7.3)
        host.py                     one pass: items → policy → validate → execute → bound → check → log
        absorb.py                   absorb(store, facts, through, retention): split at day starts
        registry.py                 PolicyRegistry over the entry-point group (§5.3)
        selection.py                MemoryPolicyChoice (the `memory:` JSON form, §6) and resolve()
    compress/
      chapters.py                   chapter ranges; first/last keys and times stored; importance
      stable.py                     first_at, last_at; stable_facts(limit)
      episodes.py                   episode importance at close
    forgetting/
      __init__.py
      consolidate.py                Consolidate (default)
      decay.py                      Decay
      importance_recency_cap.py     ImportanceRecencyCap
      keep_all.py                   KeepAll (hostile fixture, QS24-10)
    budget.py                       SqliteLedger prunes at charge (§5.12)
  tests/
    retention_stream.py             a seeded synthetic perceived stream (test-only, §8.3)
    retention_fixtures.py           hand-built stores and hostile policies
    conftest.py                     the session-scoped 100-day social-cafe save (shared with test_ac10)
    test_retention_interface.py     AFGb-1, AFGb-9
    test_retention_plan.py          AFGb-2
    test_retention_host.py          AFGb-3, AFGb-5, AFGb-6 (synthetic)
    test_retention_policies.py      AFGb-8, AFGb-4 (synthetic 300 days)
    test_retention_retrieve.py      AFGb-7
    test_ledger_prune.py            AFGb-10
    test_retention_real.py          AFGb-3, -4, -5 over the real binary (real_binary)
```

No module is expected past ~300 lines; `plan.py` and `host.py` are the largest. `retention/` sits inside
`memory/` because it is the store's owner's code (step-24: "the retention host (in `memory/`)"); the
bundled policies sit **outside** it, in `forgetting/`, so that they are packaged and imported exactly as a
user's would be (FGb-4).

### 5.2 The typed interface (`memory/retention/policy.py`; pyright strict)

```python
ItemId = NewType("ItemId", str)
"""Opaque to policies: "l0:<event key>", "l1:<episode>", "l2:<week>", "l3:<entity id>"."""

ItemLevel = Literal[0, 1, 2, 3]
Importance = Annotated[int, Field(ge=0, le=1000)]

class MemoryItem(CognitionModel):                   # strict, frozen, extra="forbid" (P5a's base)
    item: ItemId
    level: ItemLevel                                 # 0 L0 record, 1 episode, 2 chapter, 3 stable fact
    container: ItemId | None                         # the unit it nests in: L0 → its episode; an episode →
                                                     # its week's chapter if one exists; else None (top-level)
    first_at: int                                    # world seconds of its earliest fact
    last_at: int
    day: Annotated[int, Field(ge=0)]                 # first_at // 86 400
    kind: str                                        # L0: the event type; else "episode", "chapter", "stable"
    counterparts: tuple[EntityIdField, ...]
    place: EntityIdField | None
    mine: bool                                       # L0: I was the actor; L1/L2: any member was
    importance: Importance                           # computed by the HOST (§5.4); never by a policy
    citation: Citation                               # P4's type: the perceived facts it stands for
    text: str                                        # names-only text (P4-5): gist, summary, or L3 line
    bytes: Annotated[int, Field(ge=1)]               # its own rows in the canonical dump (§5.7.3)

class RetentionBudget(CognitionModel):
    store_bytes: Annotated[int, Field(ge=0)]         # bytes the policy's surviving items should fit in
    now: int                                         # world seconds: the start of the triggering day (§5.6)
    calls_left: Annotated[int, Field(ge=0)]          # model calls this pass may make; 0 for every FG-b policy

class Keep(CognitionModel):
    kind: Literal["keep"] = "keep"
    item: ItemId

class Merge(CognitionModel):
    kind: Literal["merge"] = "merge"
    items: Annotated[tuple[ItemId, ...], Field(min_length=1)]
    into: Literal[1, 2]                              # 1: L0 records into their episode; 2: episodes into
                                                     # their chapter (QFGb-8)

class Drop(CognitionModel):
    kind: Literal["drop"] = "drop"
    item: ItemId

RetentionDecision = Annotated[Keep | Merge | Drop, Field(discriminator="kind")]

class ForgettingPolicy[P: CognitionModel](Protocol):
    id: ClassVar[str]                                # ^[a-z][a-z0-9-]{0,47}$, equal to its entry-point name
    Parameters: ClassVar[type[P]]                    # bounded fields, every one with a default
    def plan(self, items: Sequence[MemoryItem], budget: RetentionBudget,
             params: P) -> Sequence[RetentionDecision]: ...
```

- `ItemLevel`, `into` and the decision kinds are `Literal`s; pyright owns their misuse (§7 STATIC).
- `items` is passed as a `tuple` of frozen models: a policy cannot change what the host holds (FGb-3).
- The protocol is generic in its parameters type so that a policy's `plan` sees its own typed model, never
  a `BaseModel` it must cast. `PolicyRegistry` stores policies as `ForgettingPolicy[Any]`-free wrappers
  (`RegisteredPolicy`, §5.3) that bind a policy to its validated parameters, so `Any` never appears.
- `memory/retention/__init__.py` exports exactly: `MemoryItem`, `ItemId`, `ItemLevel`, `RetentionBudget`,
  `Keep`, `Merge`, `Drop`, `RetentionDecision`, `ForgettingPolicy`, `Citation`, `EventRange`, and the
  helpers a policy author needs: `half_life_permille(elapsed_s, half_life_s)`,
  `hyperbolic_permille(elapsed_s, scale_s)`, `tie_key(seed, item)` (§5.9). These are the **only** names a
  bundled policy imports (AFGb-9).

### 5.3 The registry (`memory/retention/registry.py`)

```python
class RegisteredPolicy:                               # a policy bound to validated parameters
    id: str
    def plan(self, items: Sequence[MemoryItem], budget: RetentionBudget) -> Sequence[RetentionDecision]

class PolicyRegistry:
    @staticmethod
    def installed() -> PolicyRegistry                 # entry_points(group="mineworld.forgetting")
    def __init__(self, policies: Mapping[str, type[ForgettingPolicy[Any]]]) -> None   # tests, embedders
    def ids(self) -> tuple[str, ...]                  # sorted
    def bind(self, policy: str, parameters: JsonObject, *,
             source: SourceSpan | None = None) -> RegisteredPolicy
```

- **Discovery.** `installed()` loads every entry point of group `mineworld.forgetting`. A loaded object
  that is not a class satisfying the protocol, whose `id` differs from the entry-point name, or whose
  `Parameters` lacks a default for a field, is refused with `PolicyLoadError(name, distribution, why)`.
  Two distributions publishing one id is `DuplicatePolicy(id, distributions)`. Loading happens once per
  process, at resolution time; a policy package that fails to import names itself in the error.
- **Binding.** `bind` validates `parameters` with `Parameters.model_validate(..., strict=True)`; an unknown
  id is `UnknownPolicy(id, known=ids())`; invalid parameters are `InvalidParameters(policy, errors,
  source)`, where `errors` are Pydantic's locations and messages (never the input, which P5a's
  `_describe` already avoids echoing) and `source` is the file, line and column DL-a reports (§6).
- `JsonObject` is `dict[str, JsonValue]` from the SDK's `wire.ids` — raw JSON only as transport, decoded into
  the policy's own type at the boundary.

### 5.4 Importance, recency and bytes: computed by the host

**Base importance is the renderer's** (structural, per event type, the D-P4-6 plug-in pattern), stored in
`l0.importance` at ingestion. A renderer is host code, not a policy; an unknown event type gets the
generic value, so I-9 holds.

| Renderer | Record | Base importance |
| --- | --- | --- |
| `conversation` `spoke` | addressed to me / by me / overheard | 700 / 600 / 200 |
| `conversation` `conversation-started` | I am a participant / not | 400 / 150 |
| `presence` `arrived`, `person-entered-place` | mine / ambient | 100 / 50 |
| `relationships` `became-acquainted`, `relationship-changed` | concerning me / not | 800 / 300 |
| `naming` `named` | — | 500 |
| `movement` (QFGb-6) `walk-started`, `walk-ended` | mine / ambient | 80 / 30 |
| `generic` (every other type) | notable, I am a participant / notable, not / ambient | 500 / 300 / 50 |

These are literals fixed in this design, never fitted to an outcome (`ARC-23`).

- `l0.involved` (0/1, new): I am among the fact's participants. Schema v1 lacked it (P4 X-6).
- **Episode importance** = the maximum of its records' base importance, written when it closes (one SQL
  `max` over its rows; `L0Record` is unchanged). **Chapter importance** = the maximum over its episodes,
  written when the chapter is written. Both survive the removal of the rows they were computed from.
- **At the pass**, an item's importance = `min(1000, base + bonus)`, with
  `bonus = min(200, 2 × max(stable.exchanges of its counterparts))` read from L3 at that moment (0 with no
  counterparts). This is step-24's "reinforcement" in a form computable from the store at the pass alone;
  step-24's relationship *level* term is not used, because P4 stores level as the pack's own word
  (`stable.level` is free text), which the host cannot rank without knowing the pack (QFGb-10).
- **L3 items**: importance = `min(1000, exchanges + 10 × times_met)`.
- **Recency** is never computed by the host: an item carries `first_at` and `last_at`; a policy computes
  `budget.now − last_at`. Retrieval history never enters an item (`ARC-87`, QS24-11).
- **Bytes**: the UTF-8 length of the item's own canonical-dump lines (its row and its `*_who` or
  `chapter_ranges` rows), each line counted with its newline — exactly what removing it subtracts from
  `dump()`.

### 5.5 Store schema v2

```text
meta            + retention_day   ""  or the day index (decimal) of the last completed pass's `now`
                + forgotten_below ""  or an event key: the watermark (§5.8)
l0              + importance INTEGER NOT NULL, involved INTEGER NOT NULL CHECK (involved IN (0, 1))
episodes        + importance INTEGER NOT NULL      (0 while open)
chapters        + first_key TEXT, last_key TEXT, first_at INTEGER, last_at INTEGER,
                  importance INTEGER               (first_episode / last_episode kept for reading)
chapter_ranges  (week INTEGER, first_key TEXT, last_key TEXT, PRIMARY KEY (week, first_key))
stable          + first_at INTEGER NOT NULL, last_at INTEGER NOT NULL
forgotten       (first_key TEXT PRIMARY KEY, last_key TEXT NOT NULL)
retention_passes (day INTEGER PRIMARY KEY, policy TEXT, outcome TEXT, kept INTEGER, merged INTEGER,
                  dropped INTEGER, forced INTEGER, bytes_before INTEGER, bytes_after INTEGER,
                  forgotten_ranges INTEGER)        newest 32 rows only (bounded; FGb-2)
PRAGMA auto_vacuum = INCREMENTAL                   set before the first table; `PRAGMA incremental_vacuum`
                                                   after each pass's commit
```

- `_DUMPED` adds `chapter_ranges`, `forgotten`, `retention_passes`, in key order. `l0_text` stays undumped.
- **Chapter ranges.** When `close_weeks` writes a chapter, its ranges are the runs of **consecutive episode
  numbers** among the week's surviving episodes: episode numbers are consecutive in perceived order (D-P4-7),
  so `n` and `n + 1` are adjacent in the perceived sequence and a gap in the numbers is a forgotten episode.
  `Chapter.citation()` reads `chapter_ranges`; `chapters()` reads no `episodes` row (F-FGb-3).
- **`SCHEMA_VERSION = 2`.** A v1 file is refused by `_check_identity` with field `schema`, stored `1`,
  given `2`. No migration: before a stable public contract, a wrong layout is replaced cleanly
  (`CLAUDE.md` §4 rule 12), and a store is rebuilt by re-joining from a null cursor (I-5). No v1 store
  exists outside test runs (P6 is not merged).

### 5.6 When a pass runs: `absorb`

```python
@dataclass(frozen=True)
class Retention:
    policy: RegisteredPolicy                       # FG-c adds the assisted alternative

@dataclass(frozen=True)
class AbsorbReport:
    ingest: tuple[IngestReport, ...]               # one per ingest call it made
    passes: tuple[PassReport, ...]                 # one per pass it ran

async def absorb(store: MemoryStore, facts: Sequence[PerceivedEvent], through: EventId,
                 retention: Retention) -> AbsorbReport
```

- **The trigger is a fact.** A pass is due when the newest ingested fact's day (`at // 86 400`) is later
  than `meta.retention_day`. `absorb` therefore splits `facts` at every **first fact of a new day**: it calls
  P4's `ingest(store, head, through=head[-1].id)` for the facts up to and including that fact, runs the
  pass, and continues; the last segment is ingested with the frame's own `through`. Ingesting the
  triggering fact first closes the previous day's last episode (a day change always closes, D-P4-7), so at
  the pass every earlier record is in a closed episode and the open episode holds exactly one record.
- **`now`** = the triggering fact's day × 86 400. A jump over several silent days is one pass (one trigger).
- **A store's first fact** sets `retention_day` to its day in a separate small transaction before the first
  ingest; no pass runs for a day with no earlier day.
- **Crash between the ingest and the pass:** the next `absorb` (or the same call on resume) first checks
  "due" against the stored state, which is exactly the post-trigger state, and runs the pass before
  ingesting anything else. The store's dump is therefore the same whether or not the process died there
  (AFGb-5 (c)).
- **Batch independence:** split points are facts, never frame boundaries, so frames of 1, 256 or the whole
  export produce the same sequence of ingest calls *in effect* and the same passes (I-5 with retention,
  AFGb-5).
- `ingest` alone is unchanged in behaviour: a caller that never calls `absorb` never forgets (FGb-7).
  `absorb` is async because FG-c's assisted policies await a gateway; FG-b's policies are synchronous
  functions it calls directly.

### 5.7 One pass (`host.py`)

```text
pass(store, retention, now)                       one transaction: BEGIN IMMEDIATE … COMMIT
  1 items   = eligible items (§5.7.0), importance at the pass, bytes
  2 budget  = RetentionBudget(store_bytes = TARGET − overhead, now, calls_left = 0)
  3 plan    = policy.plan(items, budget)          any Exception → rejected("raised: <type>")
  4 check   = validate(plan, items)               → ok | rejected(reason)          (§5.7.1)
  5 if rejected: plan = Consolidate(defaults).plan(items, budget); outcome "fallback:<reason>"
  6 execute(plan)                                                                (§5.7.2)
  7 bound   : while dump > TARGET, the backstop                                  (§5.7.3)
  8 forgotten set: adjacency merge; watermark if > 256 ranges                     (§5.8)
  9 structural check (FGb-1's store-side half); any breach → raise, roll back    (a bug, never a fallback)
 10 retention_passes row; trim to 32; meta.retention_day = now // 86 400
COMMIT; PRAGMA incremental_vacuum
```

`TARGET = 786 432` B (768 KiB) and `STORE_BOUND = 1 048 576` B are module constants of the host, not
parameters: a world or a policy cannot loosen them (step-24 §1.2 ruling 2 (4)). The gap between them is the
room for one simulated day of ingestion before the next pass (QFGb-2). `overhead` = the dump bytes of
`meta`, `names`, `forgotten`, `retention_passes` and the open episode with its record.

#### 5.7.0 Eligible items

Every L0 record inside a **closed** episode; every closed episode; every chapter; every L3 row. The open
episode and its one record are never shown and never removed. `container` is set as in §5.2.

#### 5.7.1 Validation (the plan is refused whole, by a typed reason)

| Rule | Rejection reason |
| --- | --- |
| Every named item is one of `items` | `unknown-item` |
| Each item is named by at most one decision | `duplicate` |
| `Merge(into=1)`: every item is level 0 and all share one container, which is not removed by the plan | `merge-mismatch` |
| `Merge(into=2)`: every item is level 1, all share one chapter (`container` set), which is not removed | `merge-mismatch` |
| `Drop(x)`: `x` is top-level (`container is None`) | `drop-nested` |
| An item nested in a merged or dropped container is not explicitly `Keep`-ed | `keep-under-removed` |
| L3 items are only `Keep` or `Drop` | `merge-mismatch` |
| The result is a sequence of `Keep` / `Merge` / `Drop` instances (a policy written without pyright) | `malformed` |

An item no decision names is **kept**, unless its container is merged or dropped, in which case it follows
the container. A refused plan is never partly applied; the pass uses `consolidate` with its defaults
(the host's fallback for a plain policy) and records `fallback:<reason>`.

#### 5.7.2 Execution

- `Merge(into=1)`: delete the L0 rows and their `l0_who` and `l0_text` rows. Their ids stay represented by
  the episode, whose citation is unchanged.
- `Merge(into=2)`: delete the episode rows and `episode_who` rows, and every L0 row still nested in them.
  Their ids stay represented by the chapter's ranges.
- `Drop(x)` (top-level): delete `x` and everything nested; add `x`'s citation ranges to the forgotten set.
  An L3 `Drop` deletes the `stable` row only: a stable fact cites, it does not cover, so nothing is added to
  the forgotten set (the counterpart is "met again" from scratch if perceived later).
- A merged container keeps its `text`, `prose` and `prose_key`. Its citation already is the union of its
  members' (step-24 §8.3 point 3): a container's range is exactly the run of its members.

#### 5.7.3 The host bound (the backstop)

After execution, while `len(dump()) > TARGET`, the host applies, in order, each step until under:

1. `consolidate` with its defaults over the current items;
2. merge every remaining L0 record of a closed episode into its episode, oldest episode first;
3. merge the episodes of weeks that have a chapter into it, oldest week first;
4. drop chapters, oldest first (forgotten);
5. drop chapterless closed episodes, oldest first (forgotten);
6. drop L3 rows, fewest `times_met` first (ties: oldest `first_key`, then entity id).

Sizes are accounted per item from `bytes` and confirmed by one `dump()` at the end of each step. Every
removal the backstop makes is counted in `retention_passes.forced`. If the store is still over `TARGET`
after step 6 — only `meta`, `names`, the forgotten set, the pass log and one record remain, which no
reference world approaches — the pass raises `RetentionBoundUnreachable(bytes, floor)` and rolls back; the
seat's controller sees the exception (it is a bug or a pathological world, never silently over).

### 5.8 The forgotten set (`forgotten.py`)

- Sorted, disjoint ranges of event keys, each over the seat's perceived sequence (as `Citation`).
- **Adjacency merge:** two ranges with no surviving unit (L0 record, episode, chapter range) between them are
  merged — every perceived id between them is, by FGb-1, forgotten already.
- **Watermark:** while there are more than 256 ranges, the oldest is removed and `forgotten_below` becomes
  its `last_key` (if larger). Meaning: "every perceived id at or below the watermark that no surviving unit
  represents is forgotten." Bounded by construction (step-24 §8.3 point 5).
- API: `ForgottenSet.load(store)`; `contains(event: EventId) -> bool`; `ranges() -> tuple[EventRange, ...]`;
  `watermark() -> EventId | None`. This is how tools say "Alice has forgotten 12 Oct 08:00–14:00"
  (`CORE_CONCEPTS.md` §5.3 after DOC-24). The facts themselves stay in the save and resolve through
  `mineworld perceived` (FGb-6).

### 5.9 The bundled policies (`forgetting/`, entry points; none privileged)

Each is a class with `id`, a `Parameters` model (strict, bounded, every field defaulted) and a pure
`plan`. They import only `mineworld_cognition.memory.retention` and the standard library's `hashlib`
(AFGb-9).

| Policy | Parameters (bound; default) | Rule |
| --- | --- | --- |
| `consolidate` (default) | `l0_hours` 0 … 720 (**0**); `l1_weeks` 1 … 104 (8); `l2_years` 1 … 50 (5) | L0 records whose `last_at < now − l0_hours·3 600` → `Merge(into=1)` per episode. Episodes with a chapter and `last_at < now − l1_weeks·604 800` → `Merge(into=2)` per chapter. Chapters with `last_at < now − l2_years·31 536 000` → `Drop`, **except** a chapter whose ranges contain an L3 row's first, last or level fact (the stable facts' establishing moments). L3 rows: kept. |
| `decay` | `curve` `half-life` \| `hyperbolic` (`half-life`); `half_life_hours` 1 … 87 600 (168); `scale_hours` 1 … 87 600 (72); `threshold` 0 … 1 000 (100) | Retention `r` (‰) of `now − last_at`: `half-life` — `1000 >> k` for `k` whole half-lives, linear between (integer); `hyperbolic` — `1000·s // (s + Δ)` (a power law of exponent 1, the hyperbolic family Murre & Dros compare; their fitted parameters are not used). An L0, L1 or L2 item survives while `importance·r // 1000 ≥ threshold`. A non-survivor nested in a surviving container is merged into it; a top-level non-survivor is dropped. If the survivors' bytes exceed `budget.store_bytes`, `consolidate`'s defaults are applied to the survivors. L3: kept. |
| `importance-recency-cap` | `w_importance` 0 … 1 000 (700); `w_recency` 0 … 1 000 (300); `half_life_hours` (168); `k_l0` 0 … 100 000 (2 000); `k_l1` (400); `k_l2` (260); `k_l3` 0 … 10 000 (150); `seed` 0 … 2³²−1 (0) | `score = (w_importance·importance + w_recency·r) // 1000`. Top-down and greedy by score (ties: `tie_key(seed, item)`, the sha256 of `seed` and the item's first Event ID as a big-endian integer): choose top-level units (chapters, chapterless episodes) up to `k` and `store_bytes`; then, inside chosen chapters, episodes; then, inside chosen episodes, L0 records; then L3 rows. Unchosen nested items are merged into their container; unchosen top-level items are dropped. |
| `keep-all` | none | Returns no decisions (everything kept). Only the host bound limits it (QS24-10): the hostile fixture of AFGb-4. |

**Arithmetic of the `consolidate` default** (from §3.2's measurement; to be measured in C7): after the pass,
L0 holds only the open episode's record; L1 holds ≤ 8 weeks × 9.6 episodes/day × 565 B ≈ 304 KB; L2 holds
≤ 5 × 52 chapters × 240 B ≈ 62 KB, plus chapters kept for L3; L3 and names ≈ 3 KB. Post-pass ≈ 0.37 MB,
under `TARGET`; with a day's growth (max 460 KB) the store peaks at ≈ 0.83 MB before the next pass, under
`STORE_BOUND`. `l0_hours: 24` (step-24's two days were `l0_days: 2`) would peak at ≈ 1.29 MB: the backstop
would hold the post-pass bound, but the pre-pass store would exceed 1 MiB (QFGb-3).

### 5.10 Retrieval after forgetting (`retrieve.py`, `chapters.py`, `stable.py`)

- L3 lines read `stable.first_at` / `last_at`; no `l0` read (F-FGb-2). `stable_facts(store, limit=16)` and
  `chapters(store, limit=4)` read at most the section's line count (F-S24-2).
- Lexical matching unions two candidate sets, ranked by the unchanged D-P4-11 rule: episodes holding a
  matching **notable L0 record** (FTS5 or `LIKE`, as today), and episodes whose own **`text` or `prose`**
  holds a term (`LIKE` with `ESCAPE`, terms from `fts.query_terms`, P4-4 unchanged). The second scan is over
  at most the store's episodes, bounded by FGb-2.
- Sections, budgets, ordering and the 6 000-byte ceiling are D-P4-9 / D-P4-11 unchanged.
- **L3 citations may point into the forgotten set** (QFGb-4): a stable fact ("met Bob on day 1") outlives
  the detail of the day it was established; its establishing ids still resolve in the save. Every L0, L1
  and L2 line's citation is disjoint from the forgotten set (FGb-1), so P6's `recalls` validation, which is
  against `MemorySection.citations()` (P4 R-P6-2), can never accept an id whose detail is gone, except as
  an L3 line's provenance.

### 5.11 The cursor and re-delivery

- Retention never writes `meta.cursor` or `meta.newest` (AFGb-6 compares them before and after every pass).
  The store stays the seat's `CursorSource` (P3b D-P3b-2): a rejoin presents `store.cursor()`, which is
  never ahead of what was durably ingested (P4-2), whether or not those facts were later forgotten.
- `ingest`'s re-delivery rule is refined (F-FGb-4): within one call, ids must ascend strictly, else
  `OutOfOrder` (unchanged, D-P4-5). A fact whose id is held is compared by digest (unchanged). A fact **not
  held**, at or below the **cursor stored when the call began**, is a re-delivery of a fact the store has
  since merged or forgotten: it is skipped and counted (`IngestReport.forgotten_redeliveries`); its digest
  can no longer be compared (a recorded limitation, R-FGb-5). Anything else below the stored bound is
  `OutOfOrder` as before.

### 5.12 Budget ledger pruning (G9)

`SqliteLedger.charge` deletes, after its insert and in the same commit, every row with
`at < charged_at − 2 × DAY_S`. Windows read at most `DAY_S` back (`precheck`), so no window's sum changes;
the factor 2 tolerates a wall clock stepping back by up to a day. The ledger is wall-time bookkeeping
(P5-3), outside memory: pruning there needs no pass, no `absorb` and no coupling between `memory/` and
`budget.py` (QFGb-9 corrects step-24's "simulated day").

---

## 6. Selection: `decisions.yaml` `memory:` (coordinate with DL-a — assumption, flagged)

**What FG-b assumes DL-a defines** (step-24 §4.2, extended by class assignment like `assign:` of models):

```yaml
# decisions.yaml (World Pack; read by hosts; never seeded — ARC-84)
memory:
  default: { policy: consolidate }                       # absent → { policy: consolidate }
  assign:                                                # seat > class > default; one entry per seat or
    - { class: staff, policy: decay, parameters: { half_life_hours: 336 } }    # class per level
    - { seat: alice, policy: lm-summarise, fallback: consolidate,
        parameters: { calls_per_sim_day: 4 } }           # FG-c
```

**What crosses into Python** (QFGb-1, recommended (a)): DL-a's Rust reader resolves seat > class > default
(classes are `ARC-64`'s, which only the Rust side reads) and emits, per seat, one JSON object; FG-b owns its
Python model and its semantic validation:

```python
class SourceSpan(CognitionModel):
    file: str
    line: Annotated[int, Field(ge=1)]
    column: Annotated[int, Field(ge=1)]

class MemoryPolicyChoice(CognitionModel):
    policy: PolicyId                                     # ^[a-z][a-z0-9-]{0,47}$
    fallback: PolicyId | None = None                     # FG-c: only an assisted policy takes one
    parameters: JsonObject = {}
    source: SourceSpan | None = None                     # where the winning entry was written

DEFAULT_CHOICE = MemoryPolicyChoice(policy="consolidate")

def resolve(choice: MemoryPolicyChoice, registry: PolicyRegistry) -> Retention
```

- `resolve` refuses by name: an unknown policy (listing the installed ids), invalid parameters (Pydantic
  location and message, plus `source`), a `fallback` on a plain policy (`FallbackNotAllowed`). FG-c adds the
  rules for assisted policies.
- **Rust-side obligations FG-b asks of DL-a** (to be confirmed in DL-a's design): accept `memory:` with the
  shape above; refuse unknown keys, a non-seat person and an unknown class with line and column (as for
  `assign:`); treat `policy`, `fallback` and `parameters` as data it does not interpret (policy ids are
  Python entry points the Rust side cannot see); emit `{"seat": …, "memory": MemoryPolicyChoice}` per seat in
  a JSON form a cognition process can read (for example `mineworld decisions <world> [--decisions FILE]
  --json`). An unknown policy id therefore fails when the cognition process starts, naming the file, line
  and column — not at `run`'s load, a difference DL-a records.
- Until DL-a lands, FG-b's tests construct `MemoryPolicyChoice` JSON literally; nothing in FG-b reads YAML.
  P6 wires DL-a's output to `resolve` (R-P6-FG-2).

---

## 7. Test ownership (test rules §26)

```text
STATIC     ruff; pyright strict over memory/retention, forgetting, the schema changes: ItemId never mixed
           with EventId/EntityId; decision kinds and levels are Literals; protocols generic in parameters;
           no Any. No unit test re-proves these.
UNIT       registry discovery, refusals and binding; every validation rule (one hostile plan each);
           execution of each decision kind on hand-built stores; forgotten-set adjacency and watermark;
           importance literals per renderer; bundled policies' decisions on hand-built item sets against
           literals (ages at and across thresholds; half-life and hyperbolic arithmetic; tie keys whose
           order was computed with `shasum -a 256` before the test existed); ledger pruning against a
           MemoryLedger oracle; schema v2 refusal of v1; re-delivery of a forgotten fact.
INTEGRATION (deterministic, no binary)
           a seeded synthetic stream of 300 simulated days (§8.3) through `absorb` under every bundled
           policy and keep-all: bound at days 30, 100, 300; coverage at every pass; rebuild in frames of
           1, 256 and whole → identical dumps; crash between ingest and pass; retrieval after forgetting.
GATE 1     NOT REQUIRED: FG-b has no LLM-facing behaviour.
GATE 2     test_retention_real.py (real_binary): the 100-day social-cafe save shared with test_ac10.py,
           Alice's real export, absorb under `consolidate` and `keep-all`: bound at days 30 and 100,
           coverage and resolvability against the export, rebuild determinism. In CI's python job on three
           platforms. Plus one local 300-day real run, recorded (QFGb-5).
CI IMPACT  cognition pytest gains these modules (synthetic ≈ seconds per policy; real ≈ 30–90 s per leg,
           measured in C7); python-smoke deselects real_binary as today; Rust layers unchanged.
```

---

## 8. Test fixtures

### 8.1 Hostile policies (`retention_fixtures.py`)

`KeepAll` (bundled), and test-only policies: one per row of §5.7.1; one that raises `RuntimeError`; one
that returns a `dict`; one that returns `Drop` for every item (the most aggressive legal plan); one that
keeps everything but tries to mutate an item (fails: frozen model) — each registered through
`PolicyRegistry({...})`, which is how a user's policy is exercised without installing a distribution.

### 8.2 The bound oracle

`len(store.dump())` after every pass, read by the test; never a value the host reports about itself.

### 8.3 The synthetic stream (`retention_stream.py`, test-only)

A pure generator `stream(seed, days, per_day) -> Iterator[PerceivedEvent]` over a fixed cast of 12
entities and 6 places: `spoke` (utterances of 1–480 bytes, multibyte included, addressed to me, by me or
overheard), `arrived`, `person-entered-place`, `conversation-started`, `named` (day 0), `became-acquainted`,
`relationship-changed`, and one unknown type `fished` (I-9), with ids strictly ascending and gaps between
them (ids are not contiguous in a real world), from a SplitMix64 counter over `(seed, index)` — no RNG state.
Defaults: 200 facts per day of 300 days, ≈ 120 KB of dump per day, which exceeds `STORE_BOUND` within 9 days
without retention. Its oracles (perceived set = the generated ids) are the generator's output, not the
host's.

### 8.4 The real-binary fixture

`tests/conftest.py` gains one session-scoped fixture `social_cafe_100` returning the save directory, Alice's
export lines, her entity id and the instance (the current module-scoped body of `test_ac10.py`'s fixture,
moved, not changed). `test_ac10.py` and `test_retention_real.py` both use it, so the 100-day world runs once
per session. `test_retention_real.py` ingests the export through `absorb` in frames of 256 under
`consolidate` and `keep-all`, measuring at the passes of days 30 and 100.

---

## 9. Acceptance and adversarial criteria (fixed now, before anything is measured)

Bounds are literals from step-24 or this design, never from the implementation (`ARC-23`). Each criterion
names the mutation that must turn it red; mutations are planted, seen red, reverted and recorded in §15.
Oracles never call the code under test.

| ID | Criterion | Mutation that must fail it |
| --- | --- | --- |
| **AFGb-1** Interface and registry | (a) `PolicyRegistry.installed().ids()` equals `("consolidate", "decay", "importance-recency-cap", "keep-all")` (FG-b's tree; FG-c adds its own). (b) A test-defined policy given through `PolicyRegistry({...})` is bound and run by `absorb` exactly as a bundled one. (c) An unknown id is refused naming the id and every installed id; a policy class whose `id` differs from its entry name, two policies with one id, and a `Parameters` field without default are each refused by name. (d) Invalid parameters (`l0_hours: 721`, an unknown key, a string for an integer) are refused naming the policy, the field and the `source` span given. | (a) Register `consolidate` by direct import in the host instead of the entry point: a renamed entry point still passes — the test reads the distribution's entry points too. (c) Let `bind` ignore unknown keys: refused case passes silently. |
| **AFGb-2** Validation | Each hostile plan of §8.1 is refused with its typed reason; the pass applies `consolidate`'s defaults and records `fallback:<reason>` in `retention_passes`; the resulting dump equals that of a pass whose policy is `consolidate` itself, except for that row's `policy` and `outcome` fields. | Accept a `Drop` of a nested item: the forgotten set overlaps a surviving chapter and AFGb-3 (c) reddens; the `drop-nested` case passes. |
| **AFGb-3** Coverage (AC-FG-3) | After every pass, on the synthetic stream (every policy) and on Alice's real export (`consolidate`, `keep-all`): (a) the perceived ids (the generator's / the export's) up to the newest ingested equal, exactly and disjointly, the union of top-level units (chapter ranges, chapterless episodes' ranges, the open episode's record) and the forgotten set (explicit ranges, plus ids at or below the watermark that no unit represents); (b) every surviving L0 record lies in a surviving episode or is the open record, and every surviving episode of a week with a chapter lies in that chapter's ranges; (c) no L0, L1 or L2 citation intersects a forgotten range; (d) every forgotten id is in the export (resolvable through `mineworld perceived`). | `Drop` that forgets to add its ranges: (a) misses ids. A chapter written as one span over a dropped episode: (c) fails. |
| **AFGb-4** Bound (AC-FG-1) | (a) After every pass, `len(dump()) ≤ 1 048 576`, under every bundled policy, `keep-all`, and the "drop everything" fixture, at the passes of days 30, 100 and 300 of the synthetic stream and of days 30 and 100 of Alice's real export; also ≤ `TARGET` (786 432). (b) Under `consolidate` defaults on Alice's real export, the dump just **before** every pass is ≤ 1 048 576 too (a measured property of the reference world, QFGb-2). (c) Under `keep-all`, `retention_passes.forced` > 0 on the real export by day 30 (the host, not the policy, held the bound). | Disable the backstop (step 7 of §5.7): `keep-all` exceeds the bound by day 9 of the synthetic stream. |
| **AFGb-5** Determinism (AC-FG-4, no-LLM half) | (a) For every bundled policy, three stores built from one stream in frames of 1, of 256 and whole give byte-identical dumps (synthetic: all policies; real: `consolidate`). (b) Rebuilt from a null cursor after deletion, the store's dump is identical. (c) Crash simulation: `ingest` up to a triggering fact, close, reopen, `absorb` the rest → dump identical to an uninterrupted `absorb`. (d) Calling a bundled policy's `plan` twice on the same items gives equal decisions. | Recency from a wall clock in `decay`: (a) and (d) differ. Split at frame ends instead of day starts: (a) differs. |
| **AFGb-6** Cursor | (a) `meta.cursor` and `meta.newest` are equal before and after every pass. (b) Re-delivering, in a later call, a fact the store has merged or forgotten (its id at or below the stored cursor) is a no-op counted in `forgotten_redeliveries`, dump unchanged. (c) `[5, 3]` in one call still raises `OutOfOrder`; a held id with different content still raises `ConflictingFact`. | Let the pass rewrite `newest` to the newest surviving L0 key: (a) fails. Raise `OutOfOrder` on a forgotten re-delivery (today's code): (b) fails. |
| **AFGb-7** Retrieval after forgetting | On the synthetic stream at day 300 under every policy: (a) the rendered section is ≤ 6 000 B and every section within its D-P4-9 budget; (b) no L0/L1/L2 line's citation intersects the forgotten set; (c) an L3 line renders with every one of its establishing L0 rows gone; (d) a word that occurs only in a merged episode's `text` (its L0 rows gone) is matched and that episode returned; (e) `stable.py` and `chapters.py` apply a `LIMIT` (source check: no unbounded `SELECT` of those tables in `retrieve`'s path). | Restore `_at`'s `l0` read: (c) raises `TypeError`. |
| **AFGb-8** Bundled semantics | On hand-built item sets with literal expectations: `consolidate` at `last_at = now − l0_hours·3 600` exactly (kept) and one second older (merged), likewise for weeks and years, and the L3 exemption; `decay` half-life: Δ = one half-life gives `r = 500` ‰, so an item of importance 600 has survival value 600 × 500 // 1000 = 300 — kept at `threshold` 300, merged at 301; hyperbolic with `s = Δ` → 500 ‰; `importance-recency-cap` tie order equal to the order of sha256 digests computed with `shasum -a 256` and written into the test before the code existed; `k_l1 = 0` merges every episode into its chapter. | `>` for `≥` at a threshold: the boundary literal fails. |
| **AFGb-9** Purity and no privilege | (a) Source scan of `forgetting/`: imports only `mineworld_cognition.memory.retention` (public names) and `hashlib`; no `time`, `datetime`, `random`, `os`, `sqlite3`, `pathlib`, `gateway`, `config`, `record`, `budget`. (b) `memory/retention/__init__.py`'s `__all__` is exactly §5.2's list. (c) `plan` receives a `tuple` of frozen items: an item mutation raises. | `forgetting/decay.py` imports `time`: (a) fails. |
| **AFGb-10** Ledger | After charges at wall seconds 0, 1 h, 1 d, 2 d + 1, 3 d: rows older than `latest − 2 d` are gone (row count literal), and every `window(seat, now, s)` for `s ∈ {HOUR_S, DAY_S}` and every charged `now` equals a `MemoryLedger` fed the same charges. | Prune at `latest − DAY_S/2`: a day window loses tokens. |
| **AFGb-11** Schema v2 and disk | (a) A store file created with schema 1 is refused with `StoreIdentityMismatch(field="schema", stored="1", given="2")`, file unchanged. (b) A new store reports `PRAGMA auto_vacuum` = 2 (incremental). (c) After the real 100-day `absorb` under `consolidate`, the file on disk is ≤ 4 MiB (recorded; FTS5 index and page slack included). | Omit the `auto_vacuum` pragma: (b) fails; (c) shows the high-water size. |
| **AFGb-12** Platforms | The cognition suite, AFGb-* included, passes on `ubuntu-24.04`, `windows-2025`, `macos-15`. `test_retention_real.py` ≤ 180 s per leg (measured; over is a stop, §13). | — |
| **AFGb-13** Scope | `git diff --stat <base>..HEAD` touches only `cognition/lm-controller/**`, `docs/MODULE_SPEC.md` (§5), `docs/ARCHITECTURE.md` (§9), `.structured-coding/**`, and `uv.lock` only if the entry-point metadata requires it. No `*.rs`, nothing under `sdk/python/`, `worlds/`, `systems/`, `.github/`, `scripts/`. | — (a diff gate) |
| **AFGb-14** P4 unchanged without `absorb` | `test_ac10.py`'s eight criteria pass unchanged (with the fixture moved to `conftest.py`); P4's unit suites pass with only the edits schema v2 requires (recorded per file). | Run a pass inside `ingest`: AP4-10 (c) and (g) change. |

---

## 10. Commit plan

Each commit tracks **implementation**, **validation** and **review** separately. `[x]` needs the work and
its evidence; an inapplicable item is `N/A` with the audited reason. Targeted validation per commit; no
local full suite (test rules §8); the PR's CI is the one full run. Commands assume the worktree root.

### C0 — Freeze and contract (Markdown only)

- **Goal.** Start the implementation context against the frozen design.
- **Scope.** This document (§15 opened), `handoff-fg-b.md`.
- [x] Implementation: verify the `DESIGN FROZEN` header, §12's authority lines and their sources, that
  DOC-24 is merged (`ARC-87` present in `docs/DECISIONS.md`); record the base commit; initialize the handoff.
  Evidence: §15.1 (base 6306977; ARC-87 at line 7799); `handoff-fg-b.md` created.
- [x] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
  Evidence: "223 numbered sections across 31 documents, none duplicated"; "122 decision ids, all distinct".
- [x] Review: no authority line widened; every one has a source. Evidence: §12 unchanged; the handoff's
  endpoint line cites §12 and the go of 2026-10-13.
- **Commit boundary.** Documentation only.

### C1 — Specification edits

- **Goal.** The spec carries the retention host before the code (`CLAUDE.md` §2.2).
- **Scope.** `docs/MODULE_SPEC.md` §5: memory policies are part of a Controller Pack, discovered through the
  `mineworld.forgetting` entry-point group, bound by the host's bounds (1 MiB store, 6 000 B per call),
  selected through `decisions.yaml` `memory:`. `docs/ARCHITECTURE.md` §9: controller memory retention, pass
  points, coverage, the forgotten set, the bound. Both cite `ARC-87`.
- [x] Implementation: the two sections. Evidence: DOC-24 had already created `MODULE_SPEC.md` §5.2 and
  `ARCHITECTURE.md` §9.3 from step-24's sketch; both rewritten in place to the frozen design (§15.4 F-FGb-6).
- [x] Validation: both doc checks. Evidence: "223 numbered sections …, none duplicated"; "122 decision ids,
  all distinct".
- [x] Review: terminology matches `CORE_CONCEPTS.md` (Controller Pack, Observation, Event); no synonym for a
  defined term; nothing contradicts step-24 §8.3 except the recorded corrections (§14). Evidence: the
  corrections carried are QFGb-2 (post-pass target and hard bound), QFGb-3 (`l0_hours`, default 0),
  QFGb-8 (`into` ∈ {1, 2}, top-level `Drop`), QFGb-9 (ledger prunes by wall time), QFGb-10 (no level
  term); "memory policy", "Controller Pack", "seat" as `MODULE_SPEC` §5 already uses them.
- **Failure cases.** DOC-24 not merged: stop (C0 checks it).

### C2 — Schema v2, importance at ingestion, retrieval without dropped rows

- **Goal.** The store can lose rows without breaking ingestion, citations or retrieval.
- **Scope.** `records.py` (`SCHEMA_VERSION`, `ItemId`), `store.py` (§5.5), `ingest.py` (§5.4, §5.11),
  `render/__init__.py` and each plug-in (base importance; `involved`), `render/movement.py` (QFGb-6),
  `compress/episodes.py`, `chapters.py`, `stable.py`, `retrieve.py` (§5.10); P4 tests touched only where
  schema v2 or the refined re-delivery rule requires (each recorded).
- [ ] Implementation: as scoped. `chapters()` reads `chapter_ranges`; L3 lines read `first_at`/`last_at`.
- [ ] Validation: AFGb-6 (b), (c); AFGb-7 (c), (d) on hand-built stores with rows deleted by the test;
  AFGb-11 (a), (b); importance literals per renderer; P4's unit suites; `test_ac10.py` locally (AFGb-14).
- [ ] Review: no `int(` of an id (P4 AP4-2 (c) scan); every chapter range is a run of consecutive episode
  numbers; no read of `l0` remains in L3 rendering; the re-delivery rule cannot skip an id above the
  call-start cursor.
- **Failure cases.** A P4 test whose expectation encodes v1 layout: updated with the reason recorded, never
  weakened. `test_ac10.py` red: stop and audit (FGb-7).

### C3 — The interface, the registry and selection

- **Goal.** A typed, discoverable, refusable policy surface.
- **Scope.** `retention/policy.py`, `registry.py`, `selection.py`, `retention/__init__.py`; `pyproject.toml`
  entry-point table (bundled ids pointing at stub-free classes landing in C5 — the table lands in C5 if a
  dangling entry would break discovery; the implementing session decides and records); tests AFGb-1 (b)–(d).
- [ ] Implementation: §5.2, §5.3, §6.
- [ ] Validation: AFGb-1 (b)–(d); pyright strict over the generic protocol.
- [ ] Review: no `Any`; parameters are decoded once, at `bind`; errors never echo input values.

### C4 — The host: items, validation, execution, forgotten set, bound, `absorb`

- **Goal.** One pass, at a point fixed by the facts, that cannot exceed the bound or lose coverage.
- **Scope.** `retention/items.py`, `plan.py`, `forgotten.py`, `bound.py`, `host.py`, `absorb.py`;
  `tests/retention_stream.py`, `retention_fixtures.py`, `test_retention_plan.py`, `test_retention_host.py`.
  Policies used here: `KeepAll` and test-only ones (C5 brings the rest).
- [ ] Implementation: §5.6–§5.8.
- [ ] Validation: AFGb-2; AFGb-3 (a)–(c) on the synthetic stream; AFGb-4 (a) with `keep-all` and
  "drop everything"; AFGb-5 (a)–(c) with `keep-all`; AFGb-6 (a); mutations M-FGb-2 … -6.
- [ ] Review: one transaction per pass; no partial plan is ever applied; the structural check raises,
  never falls back; split points are facts; a pass never reads a clock.

### C5 — The bundled policies

- **Goal.** Three useful no-LLM policies and the hostile fixture, as ordinary plug-ins.
- **Scope.** `forgetting/*.py`; `pyproject.toml` entry points; `test_retention_policies.py`,
  `test_retention_interface.py` (AFGb-1 (a), AFGb-9).
- [ ] Implementation: §5.9.
- [ ] Validation: AFGb-8 literals (the `shasum -a 256` values recorded in §15 before the test is written);
  AFGb-1 (a); AFGb-9; AFGb-4 (a) and AFGb-5 (a), (d) for every policy over the 300-day synthetic stream;
  AFGb-7 (a), (b), (e); wall time per policy recorded.
- [ ] Review: each policy imports only public names; every threshold comparison matches the table's `<`
  / `≥`; no float.

### C6 — Ledger pruning

- **Goal.** The P5 ledger stops growing (G9).
- **Scope.** `budget.py` `SqliteLedger.charge`; `tests/test_ledger_prune.py`.
- [ ] Implementation: §5.12.
- [ ] Validation: AFGb-10; P5's `test_budget.py` unchanged and green.
- [ ] Review: the delete is in the same commit as the insert; `MemoryLedger` unchanged.

### C7 — Real lifecycle over the binary (Gate 2)

- **Goal.** The bound, coverage and determinism on a real 100-day history, on three platforms.
- **Scope.** `tests/conftest.py` (the fixture moved from `test_ac10.py`), `test_ac10.py` (uses it),
  `test_retention_real.py`.
- [ ] Implementation: §8.4.
- [ ] Validation:
  - Gate 2 specification written in §15 before the run (claim, owner, command, PASS evidence, mutations);
  - `cargo build -p mineworld-cli`; `uv run --locked pytest cognition/lm-controller -k "ac10 or retention_real"`;
    AFGb-3, AFGb-4 (a)–(c), AFGb-5 (a) (real half), AFGb-11 (c); wall time;
  - one local 300-day real run under `consolidate`: dumps at days 30, 100, 300 recorded (QFGb-5);
  - the PR's CI: per-leg wall time of `test_retention_real.py` and `test_ac10.py`.
- [ ] Review: no oracle calls `mineworld_cognition`; the save is read only through the CLI; the moved fixture
  is byte-for-byte the old body.
- **Failure cases.** Windows over 180 s for `test_retention_real.py`, or `test_ac10.py` over its 300 s
  ceiling because of the shared fixture: stop with the measurement (§13).

### C8 — Close-out

- **Goal.** Review readiness.
- **Scope.** README paragraph; this ledger; `handoff-fg-b.md`; parent synchronization marked pending.
- [ ] Implementation: ledger complete, deviations recorded.
- [ ] Validation: doc checks; the final head's CI green on every job (recorded in the handoff and PR).
- [ ] Review: AFGb-13's diff gate; every `[x]` carries evidence.

---

## 11. Requirements this PR places on other PRs

- **R-P6-FG-1 (on P6).** Replace `ingest(store, batch.events, batch.through)` with
  `await absorb(store, batch.events, batch.through, retention)` for every batch before taking the next
  (P3b's R-P6-P3b-1 otherwise unchanged); report each `PassReport` in the decision log; never run a pass
  elsewhere.
- **R-P6-FG-2 (on P6).** Obtain each LM seat's `MemoryPolicyChoice` from DL-a's resolved decision list
  (default `DEFAULT_CHOICE` when the world has none), `resolve` it at seat start, and refuse to start the
  seat on a resolution error (naming the file, line and column).
- **R-DLa-FG-1 (on DL-a).** §6's Rust-side obligations: accept and resolve `memory:`, refuse structural
  errors with line and column, emit the per-seat JSON form. Confirm or amend §6's shape in DL-a's design.
- **R-P7-FG-1 (on P7).** Milestone D's 100-day assertion (step-24 §8.6) runs Alice's store through
  `absorb` with the world's resolved policy.
- **R-FGc-FG-1 (on FG-c).** FG-c adds the assisted branch to `host.py`, the assisted kind to the registry
  and the fallback rule to `selection.py`, and touches nothing else of this host's contract.

## 12. Execution contract (filled at freeze, 2026-10-12)

Source of every line marked "freeze": the primary session's freeze of 2026-10-12, relayed by the
coordinator: "record rulings, headers DESIGN FROZEN …, fill execution contracts, note 'implementation starts
only on the operator's go'". Lines marked "S10 convention" follow P4 §12 and were not separately ruled; the
freeze adopts them. Endpoints the freeze does not name individually take the working rules' shipped
defaults (§§14, 21), which it does not narrow; they apply once the operator's go is given.

```text
PROJECT / PR:            MineWorld mvp0, step-24 PR FG-b — the retention host
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-fg-b-retention-host.md
RELATED / BINDING DOCS:  mvp1/step-24-decision-and-memory.md (§§1.2, 2.3–2.5, 3, 7, 8.3, 8.4, 8.6, 10, 11,
                         14.1); pr-s10-p4-memory.md; pr-s10-p5-backends.md; pr-s10-p3b-perceived.md;
                         pr-fg-c-lm-summarise.md (R-FGc-FG-1); overall.md; docs/DECISIONS.md ARC-59, ARC-84,
                         ARC-87, DEP-37; docs/ENGINEERING_STANDARDS.md; docs/REUSE_POLICY.md; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-fg-b, its own, held by one session
                                                                 (S10 convention, adopted at freeze)
BRANCH:                  mvp0/pr-fg-b, created from main                         (operator brief, 2026-10-12)
IMPLEMENTATION BASE:     origin/main at the start of implementation, with DOC-24 merged; C0 records it
APPROVED SCOPE:          §2.1, as frozen
FROZEN INVARIANTS:       §2.3; §5 decisions as ruled in §14; STORE_BOUND 1 048 576 and the 6 000-byte
                         section ceiling (never loosened); every platform
SEQUENCE:                C0 … C8
ALLOWED COMMANDS:        cargo *; git; gh (never merge); uv *; python3 scripts/*; target/*/mineworld *;
                         mkdir -p; sed -n; shasum -a 256         (S10 convention, adopted at freeze)
NEVER:                   python3 -c; sed -i; awk; xargs; curl; heredoc writes; reading any key file or
                         secrets.env; using any API key; calling any hosted API; running, pulling or
                         downloading any model                                   (operator brief; QS10-19)
MATERIAL STOPS:          any Rust, sdk/python, world, system, scripts/ or .github change; any new dependency;
                         a change to STORE_BOUND, TARGET or D-P4-9 after AFGb-4 has run once; DOC-24 not
                         merged; test_retention_real.py over 180 s or test_ac10.py over 300 s on any leg;
                         a P4 frozen criterion that cannot hold
PLATFORMS:               Linux, macOS, Windows (AFGb-12)
VALIDATION BUDGET:       unit, static, local integration (incl. one 300-day real run): unrestricted. Model
                         calls: none. CI: the PR's runs; no manual dispatch
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-fg-b.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized ONLY after the operator's go, in a
                         fresh session                           (freeze, primary session 2026-10-12)
                         semantic commits: authorized after the go (freeze; working rules §14)
                         branch push: authorized after the go    (freeze; working rules §21)
                         PR creation / update: authorized after the go (freeze; working rules §21)
                         CI repair to review readiness: authorized after the go (freeze; rules §21)
                         merge: never by the implementation session; explicit operator authorization
                         only                                    (freeze: "Do not merge"; rules §22)
POST-MERGE SYNC OWNER:   the primary session owns step-24 §11 and overall.md; the implementation session
                         owns this document's ledger, evidence, deviations and remaining issues
                                                                 (S10 convention, adopted at freeze)
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## 13. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-FGb-1 | Forgetting drops the lines Milestone D cites (R-S24-4). | Lines addressed to me carry importance 700; `consolidate` keeps episodes 8 weeks and their quoted gist; lexical matching reaches episode text (§5.10); P7's 100-day assertion is the adversarial check (R-P7-FG-1). |
| R-FGb-2 | A day's growth in a busier world exceeds the 256 KiB headroom, so the pre-pass store exceeds 1 MiB. | The hard bound is post-pass (QFGb-2); AFGb-4 (b) measures the reference world; a busier world shows it in `retention_passes.bytes_before`, and the remedy is a tighter host target or intra-day passes, a ruling. |
| R-FGb-3 | The real-binary test pushes the Windows python job over its ceilings (P4 §14.6: `test_ac10.py` at 233–243 s of 300). | One shared 100-day run (§8.4); two policies on the real export, all four on the synthetic stream; a stop at 180 s with P4 §14.5's option (c) (release build) as the lever, a ruling. |
| R-FGb-4 | A user policy loops forever or is slow. | The host bounds bytes, not CPU time; a seat whose pass never returns stalls its own controller only (the world is unaffected, `ARC-25`). Recorded; a time limit would need a subprocess per pass, not justified now. |
| R-FGb-5 | A forgotten fact re-delivered with different content is not detected. | Recorded limitation (§5.11); a re-delivery below the cursor happens only on a reconnect whose `since` the store chose, so the server already holds those ids as delivered. |
| R-FGb-6 | DL-a chooses another shape for `memory:`. | FG-b depends only on the per-seat JSON form (§6); the YAML shape is DL-a's. QFGb-1. |
| R-FGb-7 | Removing the movement renderer's notable walks changes AC-10's numbers. | AC-10 has no literal counts (P4 R-P4-3); bounds and oracles are computed from the export. |

## 14. Operator questions (each with a recommendation)

### 14.1 Rulings, 2026-10-12 (relayed by the coordinator)

| Id | Ruling |
| --- | --- |
| **QFGb-3** | **Operator: accepted.** `consolidate` keeps exact detail by the hour, `l0_hours` default 0 (§5.9). |
| **QFGb-6** | **Operator: accepted.** `memory/render/movement.py` is in scope: other people's walks are background (`ambient`) (§2.1, §5.4, C2). |
| **QFGb-1** | **Primary: (a).** DL-a's Rust reader emits typed JSON `MemoryPolicyChoice` per seat (DL-a's design is being updated to say so). FG-b validates the policy name — and its parameters — when the cognition process starts (`resolve`, §6). FG-b's tests build the JSON literally; FG-b does not wait for DL-a. |
| QFGb-2, QFGb-4, QFGb-5, QFGb-7 … QFGb-11 | **Primary: accepted as recommended.** |
| Re-measurement | **Primary:** record the 433 KB per seat per day as a dated note in this document (§3.2), not in step-24. Done. |
| Freeze | **Primary session, 2026-10-12:** FG-b DESIGN FROZEN; implementation starts only on the operator's go, in a fresh session; do not merge (§12). |

### 14.2 The questions as asked (kept for review)

**[operator]** marks a scope, cost or product decision; **[primary]** one the primary session decides.

| Id | Question | Recommendation |
| --- | --- | --- |
| **QFGb-1 [primary, with DL-a]** | How does the cognition process (Python) read `decisions.yaml` `memory:`? (a) DL-a's Rust reader resolves seat > class > default and emits per-seat JSON (`MemoryPolicyChoice`, §6) with source spans; (b) add a Python YAML parser (`DEP-49`) and read the file twice; (c) move memory selection to the operator's `cognition.toml`. | **(a).** One YAML parser for one file (PyYAML's YAML 1.1 reads `no` as `false`, `serde-saphyr` does not); classes resolve where they are defined; FG-b adds no dependency. Cost: an unknown policy id is refused when the cognition process starts, not at `run`'s load. |
| **QFGb-2 [operator]** | The 1 MiB bound is enforced **after every retention pass** (once per simulated day, at the first perceived fact of a new day); between passes the store holds at most one day of new facts on top of a 768 KiB post-pass target. Accept this reading of "≤ 1 MiB at any age"? | **Yes.** Measured: one day of Alice's perception is up to 460 KB (§3.2); a per-fact trigger would need per-row byte accounting on the ingest path for a property the reference world already meets (AFGb-4 (b)). |
| **QFGb-3 [operator]** | `consolidate`'s detail retention in **hours**, default **0** (L0 detail kept only for the current day; yesterday lives on as episodes with one quoted gist each), instead of step-24's `l0_days: 2`? | **Yes.** Measured growth is 2.2 × step-24's estimate (F-FGb-1); `l0_days: 2` would rely on the backstop every day. `l0_hours` up to 720 stays available per world or seat. |
| **QFGb-4 [primary]** | May an L3 stable fact keep citing its establishing facts after their detail is forgotten (the facts resolve in the save; P6's `recalls` would accept them only as that L3 line's provenance)? | **Yes.** "I know Bob; we met on day 1" outlives the details of day 1. The alternative — rewriting L3 provenance on every drop — would make L3 cite something that did not establish it. Step-24 §8.6's "never citable" then reads "never citable as remembered detail". |
| **QFGb-5 [operator]** | AC-FG-1's evidence: in PR CI on three platforms, the 300-day **synthetic** stream for every policy plus the **real** 100-day save (shared with AC-10) for `consolidate` and `keep-all`; one real 300-day run locally, recorded; a nightly real 300-day check added with FG-d's AC-SAVE-1 layer. | **Yes.** A real 300-day run in the Windows python job would take ≈ 10 min (P4 measured 157–215 s for 100 days there). |
| **QFGb-6 [primary — scope]** | Add `memory/render/movement.py` to FG-b: `walk-started` / `walk-ended` rendered `ambient` unless mine (today generic-`notable`, 36 % of L0 bytes)? | **Yes**, one small renderer in C2. Otherwise other people's strides fill the L0 section and carry notable importance into every policy. Alternative: a separate PR before P6. |
| **QFGb-7 [primary]** | Schema v2 refuses v1 stores by name, with no migration (rebuild from a null cursor)? | **Yes** (`CLAUDE.md` §4 rule 12; no v1 store exists outside tests). |
| **QFGb-8 [primary]** | Narrow step-24's sketch: `Merge.into ∈ {1, 2}` (no "merge into L3"), and `Drop` only of top-level units (a nested item is shed by `Merge` into its container)? | **Yes.** Each perceived id then has exactly one top-level representative, which is what makes coverage checkable; an L3 row is per counterpart and has no members to merge. |
| **QFGb-9 [primary]** | Ledger pruning by **wall** time inside `SqliteLedger.charge`, keeping two wall days, correcting step-24's "one simulated day" at "the same pass"? | **Yes.** P5's windows are wall time (P5-3); pruning there needs no coupling between memory and budgets. |
| **QFGb-10 [primary]** | Importance = the renderer's base value + a counterpart bonus from L3 `exchanges`; step-24's "relationship level" term dropped because P4 stores the level as the pack's own word? | **Yes.** P6 may add a level term once it has disclosed numeric relationship values in observations. |
| **QFGb-11 [primary]** | P6 calls `absorb` (not `ingest`) for every perceived batch (R-P6-FG-1)? | **Yes**; `ingest` stays as P4 froze it, for tools and the AC-10 scenario. |

---

## 15. Ledger (live during implementation)

Opened at C0, 2026-10-13, by the implementation session (worktree
`/Users/yuema137/mineworld-worktrees/impl-fg-b`, branch `mvp0/pr-fg-b`). The operator's go was given on
2026-10-13 and relayed in this session's brief.

### 15.1 Base and authority (C0)

```text
Implementation base:   origin/main @ 6306977 (Merge #177, docs/dl-design)
DOC-24:                merged (#175): docs/DECISIONS.md holds "ARC-87 — Controller-memory retention …"
                       (line 7799 at the base)
DESIGN FROZEN header:  present (revision 2, 2026-10-12); §12's endpoint lines each carry a source
                       ("freeze" or "S10 convention, adopted at freeze"); none widened here
Handoff:               handoff-fg-b.md
Brief additions:       the sdk/python and cognition Python suites and any affected Rust tests run locally
                       before pushing (the operator's brief of 2026-10-13; this narrows test rules §8's
                       "no local full suite" default for this PR, with that brief as its source)
```

### 15.2 Per-commit evidence

(Filled as each commit lands.)

### 15.3 Mutations

(M-FGb-n, one per AFGb row's mutation; planted, seen red, reverted.)

### 15.4 Deviations and discoveries

**F-FGb-6 (C1) — the specification sections already existed, written from step-24's sketch.**

```text
Previous assumption:  C1 adds "MODULE_SPEC §5" and "ARCHITECTURE §9" text for memory retention.
Audit evidence:       at the base, DOC-24 (#175) had written MODULE_SPEC.md §5.2 "Memory policies and host
                      bounds" and ARCHITECTURE.md §9.3 "Controller memory: retention and bounds" from
                      step-24 §8.3. They disagree with the frozen design in six places: `l0_days` 1 … 30
                      (2) vs `l0_hours` 0 … 720 (0) (QFGb-3); a merge of items "sharing a counterpart set
                      or a week" vs one container (QFGb-8); a merge "building a summary" vs a container
                      that already exists; the backstop "drops the oldest L0 then L1 rows" vs §5.7.3's six
                      steps; the bound "1 MiB after every pass" vs TARGET 768 KiB post-pass and 1 MiB hard
                      (QFGb-2); ledger pruning "in the same pass" vs by wall time at charge (QFGb-9); and
                      importance with a relationship-level term (QFGb-10).
Corrected:            both sections rewritten in place to the frozen design; the `lm-summarise` row and
                      the model-calls paragraph (FG-c's) left as DOC-24 wrote them.
Remaining:            docs/DECISIONS.md ARC-87's "Choice" paragraph keeps step-24's sketch (level term,
                      "dropping the oldest L0 and L1 rows", "bound-enforced"). ARC-87 is DOC-24's record
                      and outside AFGb-13's file list, so FG-b does not edit it; reported to the primary
                      session to amend with a pointer to this design's §14.1 rulings.
```

### 15.5 Gate 2 specification and result

(Written before the run, C7.)

### 15.6 AFGb-8 `shasum -a 256` literals

(Recorded before the test is written, C5.)

### 15.7 CI runs per head

(Filled at C8.)
