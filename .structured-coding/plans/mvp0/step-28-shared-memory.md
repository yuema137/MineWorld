# Step 28 — S28: shared, access-controlled, modular memory

**Lifecycle:** `STEP DESIGN — DRAFT (2026-10-14), rulings recorded`. Step-level design only. It is not
frozen, and nothing here authorizes implementation or a merge (`CLAUDE.md` §3.1).
- The operator ruled on every question of §14 on 2026-10-14. The rulings are recorded verbatim in §14.1, and
  the proposals they settle are now written as rulings throughout.
- The `STEP DESIGN FROZEN` header is written by the primary session itself, not by this session.
- Each PR in §11 then gets its own PR design, detailed to the commit and frozen on its own.
**Author:** the S28 planning session (Opus), 2026-10-14. Worktree
`/Users/yuema137/mineworld-worktrees/design-s28`, branch `docs/s28-shared-memory`, from
`origin/main @ f772a18` (#197). One writer.
**Effort:** `mvp0`. Per QS28-1, S28-0 and S28-a … S28-f land in MVP-0, S28-g is MVP-0's nightly gate, and
S28-h plus the `organization` example are MVP-1. **Parent:** [`overall.md`](overall.md).
**Siblings it builds on or coordinates with:**
[`step-17-cognition.md`](step-17-cognition.md) (S10: P4 memory, `AC-10`, the 6 000-byte context),
[`../mvp1/step-24-decision-and-memory.md`](../mvp1/step-24-decision-and-memory.md) (forgetting layers, §8.3,
§10), [`pr-s10-p4-memory.md`](pr-s10-p4-memory.md) (the store, `ARC-59`, `DEP-37`),
[`pr-fg-b-retention-host.md`](pr-fg-b-retention-host.md) (retention host, schema v2),
FG-b2 (in progress, no PR yet: the 20 MiB default bound, raise-only, and a size-triggered pass),
[`pr-fg-c-lm-summarise.md`](pr-fg-c-lm-summarise.md) (#201, merged 2026-10-14), [`pr-fg-a-relationship-forgetting.md`](pr-fg-a-relationship-forgetting.md)
(System-owned forgetting, untouched here), [`pr-sr-b-log-compression.md`](pr-sr-b-log-compression.md) (#203, the
sealed world log), [`pr-s10-p6-lm-controller.md`](pr-s10-p6-lm-controller.md) (P6b, #207),
[`step-25-creator-docs.md`](step-25-creator-docs.md) (the generated reference, 25a #200 / 25b).
**Binding parents:** `CLAUDE.md` §§1, 2, 3, 4 (rules 1, 2, 3, 5, 7, 8, 9, 10, 11, 12, 13, 16); `docs/CORE_CONCEPTS.md`
§§2 (INV-4, INV-6, INV-11, INV-13), 5.1–5.3, 11, 14, 15; `docs/ARCHITECTURE.md` §9.3; `docs/MODULE_SPEC.md` §5.2;
`server/PROTOCOL.md` §§5.2, 5.8; `docs/DECISIONS.md` `ARC-25`, `ARC-27`, `ARC-43`, `ARC-57`, `ARC-58`, `ARC-59`,
`ARC-86`, `ARC-87`, `ARC-89`, `DEP-37`.
**Decision ids:** assigned by the primary session on 2026-10-14. They replace the draft placeholders:

| Draft placeholder | Assigned id |
| --- | --- |
| `ARC-S28-a` | `ARC-98` |
| `ARC-S28-b` | `ARC-99` |
| `ARC-S28-c` | `ARC-100` |
| `ARC-S28-d` | `ARC-101` |
| `DEP-S28-a` | `DEP-58` |

The records are in §12, and S28-0 writes them into `docs/DECISIONS.md`. The next free ids are `ARC-102` and
`DEP-59`.
**Base:** `origin/main` merged in at `85a91d0` (#201 FG-c, #202 kill/resume fix).
**PR labels:** `S28-0`, `S28-a` … `S28-h` (§11); start conditions and implementers in §11.1.
**Scope of this file.** It is the only file this session writes. Edits to the specifications, `docs/DECISIONS.md`
and `overall.md` are listed in §13 for the primary session to apply.

No hosted model was called while writing this design, and nothing it specifies for CI calls one. Every
statement about an external project carries the date it was considered (2026-10-14). A licence or claim not
confirmed from a primary source in this session is marked **verify**; the PR that first relies on it confirms it
before its own freeze.

---

## 0. The answer in brief

1. **Today each seat stores its own copy of every fact it perceived.** In the 12-person `social-cafe`,
   30 days at seed 7, 86.7 % of the 84 639 distinct facts are perceived by more than one person. The 12
   seats hold 449 726 copies, 5.3 per fact. The perceived envelope is byte-identical across seats for every
   id (0 conflicts, measured). Before any forgetting, the P4/FG-b v2 stores grow by **465 695 B per
   person-day** and **5.59 MB per world-day** (§2).
2. **The model** (§4). A **memory home** is one SQLite database per world instance and controller process.
   It holds three things:
   - **Shared fact records**, stored once and keyed by `EventId`. A record keeps only the perspective-free
     envelope that memory uses (time, kind, place, participants, digest).
   - **Each person's access index**: one row per fact the person perceived, carrying that person's own
     rendering of it (gist, salience, importance, whether it was theirs).
   - **Each person's private layer**: episodes, chapters, stable facts, names, the forgotten set, the pass
     log, summaries, and any future inner notes.

   The context builder loads shared records only for the lines it selects. It loads them through a
   `SeatView`, and a `SeatView` can reach only the keys in that seat's access index.
3. **Perception is the only access right** (§3, I-S28-1). A fact record exists only while at least one
   person in the home holds an access row for it. Garbage collection runs in the same transaction that
   releases the last access row, and it is a pure function of the access rows (§6.4).
4. **Sharing is invisible.** For every person, the canonical **seat view** (private layer plus held shared
   records, projected) is byte-identical to the store that person would have alone. So is every rendered
   memory section (I-S28-2). Sharing is therefore a deployment knob, not a world rule (§9).
5. **A person's size is their seat view**, with every shared record they hold counted in full (§7). A
   person's bound, forgetting and recall never depend on what anyone else perceives, keeps or forgets.
   Physical bytes are always ≤ the sum of the seat views, and that sum is ≤ N × bound. The 20 MiB promise
   keeps its exact meaning: what one person may remember. Sharing saves disk for the world. It does not
   take memory away from anyone.
6. **Memory scopes are plug-ins behind one typed interface** (§5). The scopes are:
   - `person`, private memory;
   - `world`, the shared fact layer;
   - `place` / `item`, entity-anchored memory fed from perception, never from the world log;
   - any scope a user writes, for example `organization`.

   Each scope owns its tables, parameters, recall section, named errors, inspection view and isolated
   tests. A scope can index and recall; it can never grant access.
7. **Efficiency** (§8). Shared records and derivable counterparts cut world bytes by 41 % at today's
   canonical encoding. The positional canonical encoding (QS28-5, ruled: adopted in S28-a) cuts them by 77 %, and lets 20 MiB hold
   ≈ 106 days of full detail instead of ≈ 45.
8. **There is no kernel, contract, server or protocol change in MVP-0.** Sharing across processes and
   machines is MVP-1. Its recommended path is the server serving already-perceived facts by id under the
   audience rule it already applies (§6.6). No new knowledge crosses that boundary.
9. **Migration.** Schema v2 is refused by name and the store is rebuilt from the perceived stream, as
   QFGb-7 did. There is no importer and no compatibility layer (§10).
10. **Debuggability is a deliverable.** `mineworld-memory inspect` shows four things: what a person can
    access and why; what was forgotten, and by which policy and pass; the bytes per scope; and every recalled
    line traced back to its facts (§5.6). Every error names its module (§5.5).

---

## 1. The requirement

### 1.1 The operator's words (2026-10-14), verbatim

> 我觉得我们需要做到分散化memory，比如一些共有的事件，可以存到一起，人物可以有access权限，需要看的时候动态加载就可以了。只要我们给用户呈现的是连续的，后台怎么分散都没关系的

> 一个人物1mb太少了吧，我觉得起码20mb

> 默认 20 MB，世界可以调大

> 人物的记忆和小镇共有记忆，跟地点/物品有关的信息等等，模块化分散记忆，这个你需要用良好的工程模块化 做的robust，可调参，这样用户才能根据他们自己的需求设置。我们要做的是infra，提供足够灵活的实现，尽量最大限度地做到高效地存储各种记忆信息，让用户能产生虚拟世界连贯 persistent演化的感觉

> 保持代码整洁干净模块化，可以调试。一个好的系统不仅仅是一个能达到目的的系统，而是一个有了问题知道从哪里改进，维护起来方便的系统

### 1.2 The requirements, numbered (each traced to where it is met)

| ID | Requirement (English; the quotes above govern) | Met in |
| --- | --- | --- |
| R-1 | Shared events are stored once, in one place. | §4.2, §6 |
| R-2 | A person's access to a shared event is a right. An event is loaded dynamically when needed. | §4.3, §4.5, I-S28-1 |
| R-3 | What the user sees stays continuous, however the backend splits storage. | I-S28-2, §9 |
| R-4 | Per-person bound: 20 MiB by default, raise-only (FG-b2). S28 defines what "per-person size" means once storage is shared. | §7 |
| R-5 | Memory scopes, each modular: person-private; shared town or world; entity-anchored (Place, Item); user-defined (for example Organization). | §5 |
| R-6 | A scope is a plug-in that a world installs and configures, registered the way FG-b policies are. | §5.2, §5.3 |
| R-7 | Every bound, ratio, policy, sharing scope, load or cache policy, and index granularity is a typed, validated parameter with a realistic default. Invalid settings fail with named errors. Parameters appear in the generated reference. | §5.4 |
| R-8 | Robust: byte-exact replay, kill/resume, deterministic GC; bounded memory and disk at any world size, with explicit cost formulas per scope; graceful degradation when a scope is uninstalled. Tests at 12 and 100 people over 300 days. | §6, §7.3, §5.7, §15 |
| R-9 | Efficient storage is a primary goal: measured duplication, content addressing, deduplication, reference indices, compaction, hot and cold tiers, bytes per person-day and per world-day before and after. | §2, §8 |
| R-10 | Continuity is the product goal. Continuity criteria are checkable, for LLM and non-LLM controllers alike. | §9 |
| R-11 | Infra, not a demo: a user composes a memory stack per world without editing cognition internals. | §5.3, §5.4 |
| R-12 | Clean, modular, debuggable. Each scope and policy has one owner, named errors, an inspection surface and isolated tests. A wrong recall or a size overrun is located in one step. | §5.5, §5.6, §5.8 |

### 1.3 Binding constraints restated

- **Single ownership** (`CLAUDE.md` §4.1). The world log is the kernel's and is the world's truth (INV-11).
  Controller memory is cognition's (`ARC-59`). A memory home is owned by exactly one cognition process.
- **Kernel ignorance** (§4.2). The kernel does not learn what memory, a scope, or sharing is.
- **No leakage** (§4.3). No provider concept enters memory or its dump (P4 I-10). No renderer concept enters
  memory.
- **INV-13.** A controller receives Observations and its perceived stream, never world state. Storing a fact
  once must not let a person read a fact they did not perceive.
- **INV-4.** Memory, biography and world truth are three layers. Shared storage of perceived facts is still
  subjective memory: it holds the perceived envelope, not the world's state.
- **`ARC-87` and FG-b's FGb-1 … FGb-7** (coverage, bound, "policies decide, the host writes", no privilege,
  "the pass is a function of the facts", truth never lost, `ingest` never forgets) hold for every seat view.
- **QS10-19.** Agents never call a hosted model. S28 calls none.

---

## 2. Facts: today's duplication, measured

### 2.1 Method (scratch; sizes the design; not evidence for any criterion)

The measurement ran on macOS arm64, outside the repository in `/Users/yuema137/mineworld-worktrees/scratch-s28`.
It used the `mineworld` dev binary from the primary checkout (built 2026-10-12) and the World Pack at
`origin/main @ f772a18`.

```text
mineworld run worlds/social-cafe --headless --seed 7 --days 30 --save save30
    84 639 facts, 8.7 s (identical count to FG-b §3.2's run: same world, same seed)
mineworld perceived worlds/social-cafe --save save30 --person <key> --json        for each of the 12 people
    alice 37 441 · bob 38 527 · carol 38 198 · dev 34 878 · erin 39 988 · felix 39 663 · grace 38 596
    hana 37 092 · ivan 40 413 · otto 30 294 · visitor 37 618 · wanderer 37 018   → 449 726 perceived copies
```

The facts were then ingested with the merged P4/FG-b code (`mineworld_cognition.memory.ingest`, frames of 256,
`":memory:"`, no retention pass), one store per person. The results were measured on the canonical dump
(`MemoryStore.row_bytes`). A prototype layout was measured by writing each row in the canonical form the dump
uses: sorted-key JSON objects, plus `"table"`, plus a newline. Two Python scripts did this (`tools/measure.py`
and `tools/proto.py` in the scratch directory). They are not committed. S28-a re-measures with the real
implementation (AS28-12).

### 2.2 Findings

| # | Finding | Figure |
| --- | --- | --- |
| F-S28-1 | Distinct facts perceived by at least one person / perceived copies | 84 639 / 449 726 (5.31 copies per fact; ρ = distinct ÷ copies = 0.188) |
| F-S28-2 | Facts perceived by more than one person | 73 346 (86.7 %). Histogram (number of persons → facts): 1→11 293, 2→24 470, 3→10 671, 4→6 075, 5→3 496, 6→1 325, 7→1 355, 8→434, 9→749, 10→3 187, 11→2 826, 12→18 758 |
| F-S28-3 | Same id, different bytes across persons (wire form) | **0**. `PerceivedEvent` is the contract's envelope, unredacted per observer (`PROTOCOL.md` §5.2), so a fact's content is a function of its id |
| F-S28-4 | Wire bytes: per-person copies vs stored once | 240 352 404 B vs 44 496 009 B (5.4×) |
| F-S28-5 | v2 canonical dump, 12 stores, 30 days, before retention | 167 650 141 B, of which `l0` 134 544 623, `l0_who` 31 093 307, everything else 2 012 211 |
| F-S28-6 | `l0` columns that do not depend on the person (`event_key`, `event_id`, `at`, `kind`, `place`, `digest`) | 78 666 500 B of the 134.5 MB (58 %) |
| F-S28-7 | `l0_who` (a record's counterparts) | Derivable exactly from the shared participants and the person's own id: `counterparts(fact, me)` = the participants, deduplicated, minus `me` (`render/__init__.py`). 31.1 MB (19 %) need not be private |
| F-S28-8 | Gists: all rows / distinct (id, gist) pairs | 18 227 879 B / 6 714 659 B. 281 285 of 449 726 rows have a gist equal to another person's for the same id. The mean gist is 40.5 B |
| F-S28-9 | Canonical-encoding syntax (key names, quotes, `"table"`) | ≈ 123 B of every ≈ 299-B `l0` line, so ≈ 41 % of `l0` bytes are syntax |
| F-S28-10 | The store does not keep payloads | Only the digest and the rendered gist are kept. The full fact is "one citation away" in the save (`mineworld perceived`). The world log is the only place that stores the whole fact (SR-b compresses it) |

### 2.3 What these findings decide

- **The gist stays private** (D-S28-3). A gist is the person's subjective rendering (`render(fact, me, names)`).
  It depends on the person (`I said to Bob` vs `Alice said to me`) and on that person's name book *at
  ingestion*. P6 teaches names from observations with no source fact (`names.py` `learn(source=None)`), so
  re-rendering a gist at retrieval from the shared fact would change old lines when a name is learned later.
  That breaks I-S28-2 and INV-4's "what Alice believes".

  Deduplicating gists by content hash is **declined**. The mean gist is 40.5 B, and a canonical reference to a
  hash costs ≥ 22 B plus its key, so it would save under 4 % of world bytes in exchange for a third indirection.
- **The shared record is the envelope index memory already keeps.** It holds `event_id`, `at`, `kind`, `place`,
  `digest` and the ordered participants. The payload is not copied into memory: that would duplicate the world
  log, which SR-b already stores once (F-S28-10).
- **Counterparts and involvement become derived** (F-S28-7). The access row keeps only what depends on the
  person.

Today's figures after forgetting (FG-b defaults, `consolidate` with `l0_hours: 0`) are much smaller. L0 lives
for about one day, and episodes and chapters are private by nature. **Sharing pays in proportion to how long
detail is kept.** FG-b2's 20 MiB, size-triggered regime keeps detail for weeks, which is why S28 matters now
(§8.3).

---

## 3. Invariants

`I-n`, `P4-n` and `FGb-n` remain binding as P4 and FG-b state them, for every **seat view** (§4.4). This
step adds:

| ID | Invariant | Checked by |
| --- | --- | --- |
| **I-S28-1 Perception is the access right.** | A person can read a shared record only if their access index holds its key. A key enters that index only by `ingest` of a fact on that person's perceived stream. No scope, policy, query, cache, inspection call or service can widen access. Forgetting removes the access row, or demotes it into a private summary; it never redirects to a peer's copy. | AS28-1, AS28-2 |
| **I-S28-2 Sharing is transparent.** | For every person, at every pass point and at the end of any run, the canonical seat view and every rendered memory section (and so P6b's provider-neutral request) are byte-identical whether the person's home shares with 0 or 99 other persons. | AS28-3, AS28-4, AS28-10 |
| **I-S28-3 Isolation of persons.** | Nothing another person perceives, keeps, forgets or summarises changes one byte of my seat view, my size, my pass decisions or my recall. (This is I-S28-2 restated for retention.) | AS28-5 |
| **I-S28-4 A record lives exactly while it is held.** | After every committed transaction, the shared fact table holds exactly the keys referenced by at least one access row in the home. No orphan is committed, and no held key lacks its record. | AS28-6, AS28-7 |
| **I-S28-5 The home is a function of its seats.** | The canonical home dump depends only on each seat's own perceived prefix and configuration, never on the interleaving of seats, frame sizes or crash points. | AS28-6, AS28-8 |
| **I-S28-6 Physical ≤ logical.** | The home's canonical bytes are ≤ the sum of its seat views, and therefore ≤ (number of seats) × (largest bound). | AS28-9 |
| **I-S28-7 Truth is not memory.** | No scope reads the world log, a save, or another seat's private layer. Entity-anchored memory is derived from the anchoring person's perception (§5.2.3). | AS28-13 (source scan), AS28-1 |
| **I-S28-8 Scopes cannot write truth or other scopes.** | A scope receives frozen values and a `SeatView`, and returns typed rows and section lines. Only the home writes, and only into the scope's own namespaced tables. | AS28-14 |
| **I-S28-9 Graceful degradation.** | A world whose stack omits an optional scope, or a home whose scope package is uninstalled, still opens. Memory works, the world stays valid, the scope's tables are reported and ignored, and reinstalling rebuilds the scope deterministically from held facts. | AS28-15 |
| **I-S28-10 No kernel, contract, server or protocol change in MVP-0.** | — | AS28-13 (diff scan) |

---

## 4. The model

### 4.1 Vocabulary (cognition-internal; no new ontology term)

These are names of cognition modules and types. None is a new term of `CORE_CONCEPTS.md`. "Memory" keeps that
document's meaning: a Person's subjective memory, controller-owned (§5.3). `Event`, `Observation`, `Place`,
`Item`, `Organization` and `Person` keep their meanings.

| Name | Meaning |
| --- | --- |
| **memory home** | One SQLite database holding the memory of every seat one cognition process hosts in one world instance. It is the only writer of its tables. |
| **fact record** | The shared, perspective-free index of one perceived fact, keyed by `EventKey`. It is a cognition-side copy of envelope fields, not world state. |
| **access row** | One person's holding of one fact record, carrying that person's rendering of it. This is P4's L0 record minus the shared columns. |
| **private layer** | A person's own rows: access rows, episodes, chapters, stable facts, names, forgotten set, pass log, summaries' prose, and scope-private rows. |
| **seat view** | The projection of a home onto one person: their private layer joined with the fact records they hold. It is canonically dumpable, and its dump is the standalone v3 store. |
| **memory scope** | A plug-in that adds indices, private rows and a recall section over a seat view (§5). |
| **memory stack** | The scopes, policy and parameters a world selects per person (§5.3). |

### 4.2 Fact records (the `world` scope's table)

```text
fact (event_key PRIMARY KEY, event_id, at, kind, place NULL, digest)
fact_who (event_key, ordinal, entity, PRIMARY KEY (event_key, ordinal))      participants, in envelope order
```

- **Keyed by `EventKey`, verified by digest.** The key is the stable `EventId` (the world's own identity of
  the fact), and the sha256 digest of the canonical `PerceivedEvent` (P4 `ingest.digest`) is its content check.
  This is content addressing by the world's id, not by hash. The id is already unique and stable (INV-11), and a
  hash key would add a second identity to keep in step.
- **Insert-or-verify.** The first holder inserts the record. Every later holder compares digests: equal is a
  no-op, different is `ConflictingFact` (P4's error, raised from `memory.facts`). F-S28-3 says a world never
  produces a difference, so a difference is a defect worth stopping on.
- **Immutable.** A record is never updated, only inserted and garbage-collected.

### 4.3 The access index (the `person` scope's base table)

```text
access (seat, event_key, gist, salience, mine, importance, PRIMARY KEY (seat, event_key))
```

- **"Which, when, how", stated deterministically.**
  - Which: the key.
  - When: the fact's own world time `at`, read from the record. **Never the delivery time.** Frame `through`
    and wall time depend on batching, and storing them would break I-5 and I-S28-5.
  - How: derived from the record and `me`: `subject` if `me ∈ subjects`, `participant` if `me ∈ participants`,
    otherwise `witness`. P4's `involved` becomes this derivation. `salience` and `mine` stay stored, because
    the renderer decided them from the payload, which memory does not keep.
- **The FTS index** over notable gists becomes one FTS5 table with an `UNINDEXED seat` column. It is an index,
  rebuilt from `access`, and never dumped (as P4's `l0_text`).

### 4.4 The private layer and the seat view

P4/FG-b's other tables move into the home unchanged in meaning, each gaining a leading `seat` column:
`episodes`, `episode_who`, `chapters`, `chapter_ranges`, `stable`, `names`, `forgotten`, `retention_passes`,
`meta` (per-seat cursor, newest, retention day, watermark). FG-c's prose columns move with the units they
describe.

The **seat view** of seat *s* is defined by a projection:

```text
view(s).l0      = access(s, k, …) ⋈ fact(k, …)                    → P4's l0 row, columns as v2 named them
view(s).l0_who  = fact_who(k, i, e) for k held by s, e ≠ me(s)    → counterparts, envelope order
view(s).<t>     = <t> rows with seat = s, seat column removed     for every private table t
```

`SeatView.dump()` writes these tables, in v3 canonical form (§8.2), in the order P4's `_DUMPED` fixes. It
**is** the standalone v3 store's dump. This is what I-S28-2 compares, what the bound measures (§7), and what
`export` writes (§6.6).

### 4.5 Dynamic loading: the context builder

Retrieval (`retrieve.py`) today reads `l0` and joins `l0_who` with bounded queries (F-S24-2). S28 replaces each
of those reads with a `SeatView` method that joins `access` to `fact` for that seat, with the same `ORDER BY`
and `LIMIT`. Only the records behind the lines a section selects are read. Concretely:

- the L0 section reads at most 48 records, from 86 400 s back (D-P4-11);
- L1 matching reads the episode text and the matched access rows;
- nothing is loaded "in case".

A per-home LRU of decoded fact records (`cache_records`, §5.4) serves repeated loads within one consult. A
cache can only hold records the home holds, and every read goes through the seat's access join first, so a
cache cannot widen access (I-S28-1).

---

## 5. Memory scopes: one interface, plug-ins, parameters, debugging

### 5.1 Why scopes are indices plus private rows, never access grants

The operator asks for person, town, place/item and user-defined memory. Each is a different way of
*organizing and recalling* what a person perceived. None of them is a different source of knowledge. A
"town memory" a person could read without having perceived it would be INV-13's omniscience by another name.
So the interface fixes the split:

- **Access is the home's**, and it comes only from perception (I-S28-1).
- **Organization is the scope's**: indices over fact records, private derived rows, and a recall section that
  reads only through the `SeatView`.
- **Information that should reach someone who did not witness it reaches them as a world fact.** A System
  decides that someone was told, briefed, read a notice or heard a rumour. Perception then delivers that fact
  (CORE_CONCEPTS §15's `RumorSystem`, `NewsSystem`). That keeps the transfer auditable, replayable, and valid
  with every model removed (QS28-7).

### 5.2 The scopes

Each scope is one Python package module with one owner. "Owner" names the module that holds the code, the
tables, the errors and the tests.

#### 5.2.1 `person` — person-private memory (required)

- **Owner:** `mineworld_cognition.memory.scopes.person` (today's `memory/` ingest, render, compress and
  retrieve, re-homed).
- **Tables:** `access`, `episodes`, `episode_who`, `chapters`, `chapter_ranges`, `stable`, `names`,
  `forgotten`, `retention_passes`, `meta`.
- **Recall:** the four P4 sections L3, L2, L1 and L0, unchanged in content and order.
- **Not removable.** A person with no memory is a valid MineWorld person, but such a seat simply runs without
  a home. `person` is the base of every stack that has one.

#### 5.2.2 `world` — shared fact records

- **Owner:** `mineworld_cognition.memory.scopes.world` (`fact`, `fact_who`, insert-or-verify, GC).
- **Placement parameter `sharing`:**
  - `none`: each seat gets its own home, and the result is identical to a standalone v3 store;
  - `home` (default): all seats of the process share one home;
  - `service`: MVP-1, §6.6.
- **Uninstalling it is not possible.** Envelope fields must live somewhere. "Turning the town memory off" is
  `sharing: none`, which I-S28-2 proves changes nothing a person sees.

#### 5.2.3 `place` and `item` — entity-anchored memory (bundled, optional)

- **Owner:** `mineworld_cognition.memory.scopes.anchored`. One implementation, configured per anchor kind; the
  bundled entry points `place` and `item` are two configurations of it.
- **Fed from perception, not from the world log** (D-S28-5, QS28-6). A person's knowledge of a place is what
  they perceived happening there. Reading the place's objective history from the log would give a controller
  world truth: INV-13 and INV-4 forbid it, and CORE_CONCEPTS §5.3 calls it the damaging conflation. The
  objective history *is* derivable from the log, like a Person's biography (§5.2 there). It is the right
  **oracle** for continuity tests and tools (§9.2 C-3), and it is never an input to memory.
- **Index:** `anchor (event_key, entity, role)`. Rows are derived purely from the shared record's envelope:
  `place` gives role `at`, `subjects` give `subject`, `participants` give `participant`. The parameter
  `roles` filters them. One row per (fact, anchor, role). The index is shared, keyed by `event_key`, and is
  garbage-collected with its fact (§6.4). Its granularity parameter is `event` (default) or `episode`. In
  `episode` mode the index is per-seat, `(seat, episode, entity)` rows that are smaller and coarser, and recall
  cites the episode.
- **Private rows:** `anchor_known (seat, entity, first_key, last_key, first_at, last_at, visits)`, a stable
  fact per anchor, like L3's per counterpart. It is capped at `anchors_max` and evicted fewest visits first,
  with ties broken by entity id string. A forced host step drops these rows, oldest `last_at` first (§7.2).
- **Recall:** an optional section `About <place name>:` for the anchors in the query, meaning the
  observation's place and any item it lists. It shows up to `section_lines` lines within `section_bytes`.
  It holds one `anchor_known` line plus the newest held notable gists anchored there that no other section
  shows. Every line carries its citation.
- **Payload-level anchors** (an Item named only inside a payload) need a renderer to declare them. That is
  S28-d's bounded extension of `Rendered` with `anchors: tuple[EntityId, ...]`, stored on the access row (it
  is the person's reading of the payload), so reinstalling the scope can rebuild from held rows (I-S28-9).

#### 5.2.4 User-defined scopes (for example `organization`)

A Python package with one entry point in the group `mineworld.memory_scopes`, implementing §5.3's interface.
The MVP-1 example is `organization`: an anchor scope over the entities an Organization System names in its
facts' envelopes, recalling "what I know of the Lakeside Cafe staff". It is implemented in the same way as
5.2.3, and it is outside the cognition package in order to prove R-11.

#### 5.2.5 Inner notes (not on main; position stated now)

No "inner note" or reflection record exists on `main` (audited: `memory/`, P6, step-17, step-24). If P6b or a
later PR adds model-written notes, they belong to the `person` scope's private layer. No scope may index them
as shared, and they are never written into a fact record. A belief reaches another person only by being said,
which is a world fact (QS28-3).

### 5.3 The typed interface (`mineworld_cognition.memory.scopes`, pyright strict)

The interface follows FG-b's pattern (`ARC-85`: interface, carrier, selection, parameters, host).

```python
class MemoryScope(Protocol):
    id: ClassVar[str]                      # = entry-point name, ^[a-z][a-z0-9-]{0,47}$
    Parameters: ClassVar[type[ScopeParameters]]   # strict, frozen, every field bounded and defaulted
    tables: ClassVar[tuple[TableSpec, ...]]       # namespaced `<id>_*`; keyed by event_key and/or seat

    def index(self, fact: FactRecord, reading: Reading, params: Parameters) -> tuple[ScopeRow, ...]: ...
        # pure: rows to add when a seat first holds `fact`; `reading` is the seat's rendering (gist,
        # salience, mine, anchors) — no store, no connection, no clock, no path, no gateway (FGb-3)

    def recall(self, view: SeatView, query: RecallQuery, budget: SectionBudget,
               params: Parameters) -> tuple[SectionLine, ...]: ...
        # reads only through `view`, whose every method is filtered by the seat's access index;
        # every line carries a Citation of held keys; the host checks both

    def account(self, view: SeatView) -> ScopeBytes: ...     # the scope's share of the seat view, by table
```

- **The host does the writing.** `MemoryHome` creates and drops tables, inserts the returned rows, cascades
  deletion by `event_key` when a fact is collected, and cascades by `seat` when a seat leaves the home. A scope
  never deletes.
- **The host checks every line.** A citation outside the seat's held keys is `AccessDenied` (a defect, named
  with the scope, the seat and the key, and never shown to a model). A section over its budget is
  `ScopeSectionOverBudget`.
- **Registry:** `ScopeRegistry` over `importlib.metadata` entry points, group `mineworld.memory_scopes`. It
  mirrors FG-b's `PolicyRegistry`, with the same load checks and the same refusal of duplicates, unknown ids
  and invalid parameters (§5.5). The bundled `person`, `world`, `place` and `item` are ordinary entry points
  and none is privileged (FGb-4's rule).
- **Selection:** the memory stack is per seat, seat > class > default, in `decisions.yaml` `memory:` (MODULE_SPEC
  §5.2, already the home of `policy` and FG-b2's bound). The Rust reader (DL-a's) passes `scopes` through as
  typed JSON, `MemoryScopeChoice { id, parameters, span }`, exactly as it passes `MemoryPolicyChoice` today.
  Placement and caching are operator settings in `cognition.toml` `[memory]`, because I-S28-2 makes them
  invisible to the world.

```yaml
# decisions.yaml (World Pack) — sketch; S28-c fixes the exact form with DL-a's reader
memory:
  default:
    policy: consolidate
    bound_mib: 20                 # FG-b2: >= 20, raise-only
    scopes:
      - id: person
        parameters: { sections: { L3: { lines: 16, bytes: 1600 }, L2: { lines: 4, bytes: 800 },
                                  L1: { lines: 8, bytes: 1600 }, L0: { lines: 48, bytes: 2000 } } }
      - id: place
        parameters: { section_lines: 4, section_bytes: 0, anchors_max: 256 }   # 0 B: indexed, not recalled
```

### 5.4 Parameters (typed, validated, defaulted, generated into the reference)

Every parameter model is a strict Pydantic model. Its JSON Schema feeds S25 25b's generated pages
`reference/memory-scopes.md` and `reference/cognition-toml.md`. A generated page is never edited by hand. The
defaults below are chosen so that **the default stack reproduces today's context byte for byte** (§9.2 C-1).

| Where | Parameter | Type and range | Default | Why this default |
| --- | --- | --- | --- | --- |
| `cognition.toml [memory]` | `sharing` | `none` \| `home` (`service` refused before MVP-1 with `SharingModeUnavailable`) | `home` | Transparent by I-S28-2; saves 41–77 % (§8) |
| | `home` | path template | `store/<instance>/memory.sqlite` | Beside P6's `store/<instance>/` files |
| | `cache_records` | 0 … 1 000 000 | 4 096 | ≥ one consult's loads (≤ 48 + 8 + 16 + 4 + anchors) for 40 seats |
| | `page_cache_kib` | 256 … 1 048 576 | 8 192 | SQLite `cache_size`, 8 MiB, enough for a 12-seat home's hot day |
| | `verify` | `off` \| `pass` \| `every-commit` | `off` | Debug: recompute I-S28-4 and sizes at each pass or commit, and raise on a mismatch |
| `decisions.yaml memory:` (per seat) | `policy`, `parameters` | FG-b, unchanged | `consolidate` | FG-b |
| | `bound_mib` | 20 … 1 024, integer (FG-b2) | 20 | Operator ruling, 2026-10-14 |
| | `scopes` | list of `{ id, parameters }`; `person` required first; ids unique | `[person]` | Today's behaviour |
| `person` | `sections.<L>.lines` / `.bytes` | L3 0…16 / ≤1600; L2 0…4 / ≤800; L1 0…8 / ≤1600; L0 0…48 / ≤2000 | P4's `BUDGETS` | Unchanged; they can only shrink. The sum over every scope's sections must be ≤ 6 000 B (D-P4-9) |
| | `l0_window_s` | 3 600 … 86 400 | 86 400 | D-P4-11 |
| `place` / `item` | `roles` | subset of `at`, `subject`, `participant` | place: `[at]`; item: `[subject, participant]` | A place is where things happen; an item is what was acted on |
| | `granularity` | `event` \| `episode` | `event` | Exact citations; `episode` is ≈ 1/130 the rows (9.6 episodes vs 1 248 facts per person-day, FG-b §3.2) |
| | `anchors_max` | 1 … 10 000 | 256 | Social-cafe has fewer than 40 places and items. 256 × ≈ 180 B ≈ 45 KiB, 0.2 % of 20 MiB |
| | `section_lines` / `section_bytes` | 0 … 16 / 0 … 1 600 | 4 / 0 | Off in recall by default (indexed only); a world turns it on and must take the bytes from another section |

Non-parameters, with the reason each is fixed:

- **GC timing.** GC runs in the transaction that releases the last access row. A deferred GC would make the
  dump depend on when it ran.
- **The context ceiling of 6 000 B** (D-P4-9, ARCHITECTURE §9.3).
- **The access rule** (I-S28-1).

### 5.5 Named errors (each in the module it names; the message starts with that module's tag)

| Module | Error | When |
| --- | --- | --- |
| `memory.home` | `NotAMemoryHome`, `HomeIdentityMismatch(field, stored, given)`, `HomeBusy(path, holder_pid)`, `SeatNotInHome(seat)`, `UnsupportedStoreSchema(found, wanted)` | Opening the wrong file; a second process opening a held home (an OS lock on all three platforms); a v2 store (§10) |
| `memory.scopes` | `ScopeLoadError`, `DuplicateScope`, `UnknownScope(id, installed)`, `ScopeParameterError(scope, field, span)` (never echoing the value, as FG-b), `PersonScopeRequired`, `ScopeSectionOverBudget(scope, section, bytes, limit)`, `TotalSectionsOverCeiling(total)` | Start-up selection and per-call checks |
| `memory.scopes` (raised by the host *about* a scope) | `AccessDenied(scope, seat, event)` | A scope's line cited a key the seat does not hold. This is a defect, and the consult falls back as P6 does for an internal error |
| `memory.scopes.world` | `ConflictingFact(event)` (P4's, moved), `FactRecordMissing(seat, event)`, `HolderInvariantBroken(keys)` | Digest mismatch; a held key without its record; `verify` found an orphan or a dangling key |
| `memory.retention` | FG-b's `RetentionBoundUnreachable`, plus `BoundBelowDefault(seat, given_mib)` | The bound cannot be met; a bound under 20 MiB was configured |
| `memory.config` | `SharingModeUnavailable(mode)` | `service` before MVP-1 |

### 5.6 Inspection: `mineworld-memory inspect` (console script of `mineworld-cognition`) and `MemoryHome.inspect()`

The command and the Python API share one implementation and produce one typed `InspectReport` (JSON with
`--json`). The command only reads: it opens the home read-only and never writes, not even the pass log. It
runs while the cognition process is stopped. While the process runs, the same report is produced by the API,
which the process can serve.

| View | Shows | Answers |
| --- | --- | --- |
| `inspect HOME --seat alice access [--event 1890]` | The held count by how (subject, participant, witness) and by salience. For one id: **held** (with its record, its access row, and how), **merged** (into episode *n* or chapter *w*, the pass day and policy), **forgotten** (the range, the pass day and policy that added it), **never perceived** (not on the stream, so no access). Every answer includes the `mineworld perceived … --since <id−1>` command that shows the fact itself | "What can Alice access, and why?" |
| `… forgotten` | The forgotten ranges with their attribution, the watermark, and the last 32 passes (policy, outcome, kept, merged, dropped, forced by host step, bytes before and after) | "What was forgotten, by which policy, and was it forced?" |
| `… bytes` | The seat view's bytes by scope and table; the bound; the physical share of each held record (`bytes ÷ holders`, a diagnostic only, §7.1); home totals: fact records, every seat's private bytes, the file size, the freelist | "Where are the bytes, and which scope overran?" |
| `… recall --at T [--with bob] [--words "…"] [--trace]` | The exact memory section a consult would receive. With `--trace`, each line is followed by its section, its scope, its citation, and each cited key's access row and record | "Why did Alice say that?" — every recalled line back to its source facts |
| `… scopes` | The installed and selected scopes, their parameters with the source span that set each, and uninstalled scopes whose tables remain | "What memory stack is this person running?" |

**Forgotten-set attribution** is a bounded schema addition. Each forgotten range carries
`(first_day, first_policy, last_day, last_policy, passes)`, updated when ranges merge. Full per-id attribution
would grow without bound, so the report says "added by `consolidate` on day 12, extended by 3 passes, last on
day 40".

**One step to the cause:**

- A wrong recall: `recall --trace` names the scope, the seat, the access row and the record. A wrong gist is a
  `person` renderer defect. A wrong place is the record, so compare against `mineworld perceived`. A line
  citing an unheld key cannot reach a model (`AccessDenied`).
- A size overrun: `bytes` names the scope and the table, and `forgotten` shows which forced host step ran.

### 5.7 Graceful degradation (I-S28-9)

- **A scope removed from a stack:** its section is gone, and its private rows for that seat are dropped by the
  home at the next open, recorded in the pass log as `scope-removed:<id>`. Shared index rows are dropped when no
  remaining seat selects the scope. Memory and the world are unaffected.
- **A scope package uninstalled while a stack still names it:** `UnknownScope` at start, naming the installed
  ids. The operator then either removes the scope from the stack or reinstalls it. The home is never opened
  with a scope it cannot run, so no table is silently stale.
- **Reinstalling a scope:** the home rebuilds its rows from the held records and access rows, in key order.
  `index` is pure, so the rebuilt rows are byte-identical to never having removed it (AS28-15).
- **Without any home** (`cognition.toml` absent, or a non-cognition controller): the world runs exactly as
  today. Memory is a controller's, and the world never reads it.

### 5.8 Isolated testing, per scope and policy

Each scope gets the following, without a home or a binary:

- a **property test** of `index` (pure: same fact and reading give the same rows; no row cites another key);
- a **recall test** against a `SeatView` fake that holds a fixed key set and raises on any other key;
- a **budget test** (every line ≤ 160 B; the section ≤ its bytes).

The home's own tests use a fixture scope that is hostile: it cites unheld keys, returns oversized sections and
raises. FG-b's hostile policies keep their tests and now run against seat views. Every scope and policy has an
isolation and provider scan (P4 I-3, I-10), extended to `memory/scopes/`.

---

## 6. Ownership, placement, transactions, GC and the boundary

### 6.1 Ownership

| State | Owner | Notes |
| --- | --- | --- |
| The world log (facts, journal, sealed blocks) | Kernel / persistence (`ARC-89`) | Truth. Never read by memory. |
| Which facts a person perceives | The world's perception systems through the server (`ARC-43`, `PROTOCOL.md` §5.8) | The access right's only source |
| A memory home and everything in it | The one cognition process that opened it (`HomeBusy` otherwise) | Controller-owned memory (`ARC-59`) |
| Fact records | `memory.scopes.world`, written only by the home | A cache of envelope fields, not truth |
| Access rows and private layer | `memory.scopes.person`, written by `ingest` and the retention host | |
| Scope rows | The scope's module, written by the home | |
| Bounds | The retention host (FG-b, FG-b2) | Measured on the seat view |

### 6.2 Why the shared layer cannot be the world log, and why there is no kernel change

The controller is a client (INV-6, INV-9, INV-13). It sees only its perceived stream, so it cannot read the
log. Even if it could, the log is the world's truth, and the memory of one person is a subjective layer
(INV-4). The shared layer therefore has to be a **cognition-side store of perceived envelopes**, and it can be
one with no help from the kernel:

- facts are identical across observers (F-S28-3);
- the perceived stream is exactly-once and ordered (`PROTOCOL.md` §5.8);
- the digest already detects disagreement.

A kernel change would have to teach the kernel what "memory" or a "holder" is, which is exactly what kernel
ignorance forbids (INV-12, `CLAUDE.md` §4.2). **None is proposed.**

### 6.3 One transaction per ingest call

`ingest(home, seat, facts, through)` runs one `BEGIN IMMEDIATE … COMMIT` on the home that covers:

- the fact inserts or verifies;
- the access rows;
- the scope rows;
- the seat's episodes, chapters, stable facts and names;
- the seat's cursor.

This is P4-2 over one file, and that is the reason for one file per home rather than a file per seat with
`ATTACH`:

- SQLite commits across attached files atomically only in rollback-journal mode, through a super-journal, and
  not in WAL mode (sqlite.org "Atomic Commit", **verify** the exact current wording). That is a mode
  restriction we would have to defend forever.
- A per-seat file is not self-contained anyway once records are shared.
- The **seat view export** gives the per-seat portability back (§6.6).

A crash anywhere rolls back everything the call wrote, and the seat's re-delivery from its cursor repeats it
(insert-or-verify makes repetition a no-op). One cognition process hosts its seats on one asyncio loop, so
ingest calls are serialized by the process. The home is single-writer by construction, and `HomeBusy` stops a
second process.

### 6.4 Garbage collection (deterministic, transactional, exact)

There is no stored reference count. **The holders of a record are the access rows that cite it.** This is one
relation, so a count cannot drift from its rows.

1. A retention pass (or a seat leaving the home) deletes access rows. That covers FG-b's `Merge(into=1)`, the
   records of merged episodes, and `Drop`. It collects the set *K* of released keys.
2. In the same transaction it deletes `fact`, `fact_who` and every shared scope row for the keys of *K* that no
   longer have any access row:

   ```sql
   DELETE … WHERE event_key IN K AND NOT EXISTS (SELECT 1 FROM access WHERE event_key = k)
   ```

   An index on `access(event_key)` serves the check.
3. After commit, `PRAGMA incremental_vacuum` returns freed pages (FG-b §5.5).

Every step is a function of the access rows, which are a function of each seat's perceived prefix and
parameters (FGb-5). The home dump is therefore a function of the seats (I-S28-5), whatever their interleaving.
`verify = pass` recomputes I-S28-4 with one anti-join in each direction.

### 6.5 Determinism, replay and kill/resume (`ARC-25`, `ARC-27`)

- **Byte identity is defined on canonical dumps**, never on file bytes (D-P4-8): `SeatView.dump()` per seat,
  and `MemoryHome.dump()` for the whole home (fact records, then each seat's private tables in seat order, then
  shared scope rows).
- **Frame sizes:** ingest is already frame-independent (I-5, FGb-5). Pass points are fixed by the perceived
  sequence of each seat alone. Nothing in the home reads another seat's position.
- **Crash points:** the only durable states are the commits of §6.3 and §6.4, and every commit preserves every
  invariant. A kill between commits loses nothing that is not re-delivered.
- **Replay of LLM-assisted forgetting** (FG-c): prose is private to the seat, and cassettes are keyed by the
  provider-neutral request. Under I-S28-2 that request is identical with or without sharing, so recordings made
  before S28 or under `sharing: none` replay with zero `CassetteMiss` (AS28-10).
- **Rebuild:** a home rebuilt from null cursors for its seats, in any order and interleaving, has the same dump.

### 6.6 Beyond one process: what crosses the boundary

- **MVP-0:** one home per cognition process. Seats in different processes have different homes, each with its
  own fact records. That duplication is bounded by the number of processes, not of persons.
- **Moving a seat between processes:** `mineworld-memory export HOME --seat alice > alice.v3` writes the seat
  view, the standalone v3 store. `import` splits it into records (insert-or-verify) and private rows. Export
  then import is byte-identical by I-S28-2 (AS28-11). Rebuild from the perceived stream also works.
- **MVP-1 (QS28-2, ruled: path (i))**. The two paths considered:
  - **(i) The server as the cold tier (ruled).** A protocol addition (not kernel): a seat may ask
    `perceived_by_id { ids }` and receives the facts **it already perceived**. The server checks each id with
    the audience rule it already applies to `perceived` (`ARC-43`). What crosses is ids out and the seat's own
    perceived envelopes back, nothing new. The world log is already the one place every fact is stored, once
    and compressed (SR-b). A home may then evict cold fact records (older than `hot_days`) and load them on
    demand. The access right stays verifiable by the only party that knows it.
  - **(ii) A cognition shared-store service** (one operator's processes, loopback or LAN). What crosses is
    `put(seat, fact)`, `get(seat, keys)` and `release(seat, keys)`. The service can verify a digest but cannot
    verify that a seat really perceived the fact, so it is a trust domain of one operator. Declined as the
    default, and kept as an option for an air-gapped cluster.

---

## 7. Forgetting with sharing, and what "per-person size" means

### 7.1 The rule (D-S28-4)

> **A person's size is the canonical byte length of their seat view**: their private layer plus every fact
> record they hold, each record counted **in full**, however many other persons hold it. The bound
> (`bound_mib`, 20 MiB default, raise-only) applies to that size. The physical share of a shared record
> (`bytes ÷ holders`) is reported by `inspect bytes` as a diagnostic and is never charged.

Why "in full" and not a fair share:

- A fair share changes when a peer gains or loses access. Alice's size would drop when Bob perceives the same
  scene and rise when Bob forgets it, which could trigger Alice's size pass and change what Alice remembers
  because of Bob. That violates I-S28-3, makes Alice's memory a function of the town rather than of her
  perception (`ARC-59`), and breaks I-S28-2 (a shared Alice would differ from a standalone Alice).
- "In full" keeps the operator's 20 MiB promise literally: **every person may remember up to 20 MiB, measured
  as if alone, and no one else's memory ever reduces it.** With the encoding of QS28-5 that is ≈ 106 days of
  undigested detail rather than ≈ 45 (§8.3).
- Sharing still pays: the world's physical bytes are ≤ the sum of persons' sizes (I-S28-6). Measured, they are
  59 % of it at today's encoding and 23 % with QS28-5 (§8).

### 7.2 FG-b, FG-b2 and FG-c on a seat view

- **Items:** the retention host builds `MemoryItem`s from the seat view exactly as it does from a v2 store
  today. The level, container, times, kind, counterparts, place, importance, citation, text and **size**
  (now the bytes the item contributes to the seat view, shared columns included) are the same. Policies are
  unchanged; they cannot tell.
- **Execution:**
  - `Merge(into=1)` and `Merge(into=2)` delete the seat's access rows (and episodes);
  - `Drop` deletes them and adds ranges to the seat's forgotten set;
  - the GC of §6.4 collects released records whose last holder left, in the same transaction.

  A peer's rows are never touched.
- **FG-b's host-bound steps** keep their order, plus one step before "drop stable facts": drop scope-private
  rows, oldest `last_at` first (§5.2.3). The bound's target and hard margin are scaled from FG-b's 768 KiB and
  1 MiB ratio (0.75), as FG-b2 states it; S28 inherits FG-b2's numbers and changes only what they measure.
- **FG-b2's size trigger** fires on the seat view's size. The home keeps a per-seat running total, updated
  in-transaction, with each scope's `account` as the authority. Under `verify` the total is recomputed from
  `SeatView.dump()` and must be equal (`HolderInvariantBroken` names the seat otherwise).
- **FG-c summaries** are written to the seat's private units, with requests built from the seat view (names
  only, ≤ 6 000 B). Nothing in a request depends on sharing. The summaries' non-contradiction checks are §9.2
  C-4.
- **The forgotten set** stays per seat. A forgotten id is a perceived id that the seat no longer holds. It still
  resolves in the save, and another person's copy is irrelevant to it (I-S28-1).

### 7.3 Cost formulas (bounded at any world size)

For a home with *N* seats, bound *B* bytes, and ρ = distinct held facts ÷ held copies:

```text
seat view, each seat:          size_s ≤ B                                              (host bound)
home, logical:                 Σ_s size_s ≤ N·B
home, physical (canonical):    P = Σ_s private_s + Σ_{k held} record_k + shared_scope_rows
                               ≤ Σ_s size_s ≤ N·B                                      (I-S28-6)
expected:                      P ≈ Σ_s private_s + ρ·Σ_s shared_part_s
file on disk:                  ≤ P·(1 + ε) + freelist, freelist returned after each pass (incremental vacuum)
process memory:                page cache (page_cache_kib) + cache_records × ≈ 300 B + one consult's rows
place/item shared index:       ≈ (distinct held facts) × (anchors per fact) × ≈ 40 B (canonical objects)
place/item private:            ≤ N × anchors_max × ≈ 180 B
```

Worked bounds:

- 12 seats at 20 MiB is ≤ 240 MiB logical; expected physical is ≈ 142 MiB (today's encoding) or ≈ 131 MiB
  holding 2.4× the days (QS28-5).
- 100 seats is ≤ 2 000 MiB logical. Larger towns share more (more persons per public place), so ρ falls, but
  that is to be measured (no 100-person world exists, F-S28-11).

---

## 8. Efficiency: what is stored, how, and the numbers

### 8.1 Mechanisms, each with its verdict

| Mechanism | Verdict |
| --- | --- |
| Content addressing | **Adopt, by world id.** The key is `EventId` and the check is the sha256 digest (§4.2). Hash-keyed gist dedup is declined (§2.3) |
| Deduplication | **Adopt** for envelope fields and participants (F-S28-6, F-S28-7) |
| Reference index | **Adopt**: `access(event_key)`, the holder relation, with no separate refcount (§6.4) |
| Compaction | Retention's merges (FG-b) plus GC plus incremental vacuum. A size-triggered pass (FG-b2) on the seat view |
| Tiered loading | **Hot:** the SQLite page cache plus the record LRU, with bounded joins at recall (§4.5). **Cold (MVP-1):** fact records older than `hot_days` evicted and fetched by id from the server under the audience rule (§6.6 (i)) |
| Canonical encoding | **Positional rows (QS28-5, ruled 2026-10-14: adopted in S28-a):** `[table, c1, c2, …]` in schema order, ids unpadded, digest 128-bit in the record (the full 256-bit digest stays the re-delivery check in the transaction, where it is not dumped). Defined once in `memory.home`, used for the bound and for determinism |

### 8.2 Measured bytes, before forgetting (12 persons, 30 days; §2.1 method)

| Layout | Private per person-day | Shared per world-day | **World-day total** | vs v2 |
| --- | --- | --- | --- | --- |
| v2 today (P4/FG-b, per-seat copies) | 465 695 B (all) | — | **5 588 338 B** | 1.00 |
| v3, shared records, canonical objects | 210 776 B | 789 849 B | **3 319 161 B** | 0.59 |
| v3, shared records, positional (QS28-5) | 86 927 B | 248 385 B | **1 291 514 B** | 0.23 |

The positional row's shared column uses a 128-bit digest. With the full 256-bit digest add ≈ 90 KB per
world-day (+7 %).

### 8.3 What 20 MiB holds, and steady state

Seat-view bytes per person-day (what is charged):

- today, ≈ 465 695 B, so 20 MiB holds ≈ **45 days** of undigested detail;
- positional (estimate: private 86 927 + 1 249 held facts × ≈ 88 B ≈ 196 800 B), so ≈ **106 days**.

Under FG-b's current default (`l0_hours: 0`) detail is merged daily, so the hot day is all that sharing
deduplicates. Under FG-b2's size-triggered regime detail is kept until the bound, and the physical world saving
is the table's 41 % or 77 %. AS28-12 fixes the targets before measuring.

### 8.4 Finding

**F-S28-11.** No World Pack on `main` has more than 12 people (`bodies-yard`, `market-town`, `social-cafe`: 12;
`lakeside`: 10). The 100-person gate needs a generated fixture world (QS28-9).

---

## 9. Continuity: what "presented continuously" means, checkably

### 9.1 The surfaces a person's continuity is seen on

1. The **memory section** of an LLM controller's context: P6b's request, whose provider-neutral form keys the
   cassettes.
2. The **memory a non-LLM controller reads**. Scripted or recorded cognition in tests, and any Python
   controller using `retrieve`, read the same `MemorySection`.
3. The **human-visible memory page**. None exists on `main` (audited: `clients/`, `docs/guide/`; the 2D HUD
   keeps only display names). S28 defines it as `inspect recall`, the exact section a consult would receive, so
   a human sees what the model sees. A client page later reads the same report through the cognition process.

### 9.2 Criteria (each a test in §15)

| ID | Criterion | Applies to |
| --- | --- | --- |
| **C-1 Sharing is invisible** | The surface (1) request bytes, the surface (2) sections and the surface (3) report are identical with `sharing: none` and `home`, for every seat, for 200 seeded queries per seat at every pass point | LLM and non-LLM |
| **C-2 Migration keeps the context** | The rendered sections of the default stack equal the v2 golden renders captured at S28-a's C0 from the same perceived streams, for every query, at every pass point at which v2 performed no bound-forced removal | LLM and non-LLM |
| **C-3 Place recall agrees with the place and the log** | Every line of a `place` section cites only held facts whose envelope anchors that place in a selected role. Its `visits`, `first_at` and `last_at` equal a recount over the person's `mineworld perceived` export filtered by that place, up to the forgotten set. Every cited fact appears in the objective place history derived from the log by the test oracle (a test-side filter over `mineworld perceived` and the save, never memory's input) | Both |
| **C-4 Summaries never contradict retained detail** | For every episode and chapter: (a) its structural fields (day, place, counterparts, record count) equal a recomputation from its cited facts when it was written; (b) no retained L0 record lies inside a merged container's ranges (FGb-1); (c) a retained record and the unit covering it agree on day and place. For FG-c prose (d): every person name in the prose is a known name of a counterpart of the group, and every place name is the group's place. Otherwise the host takes the fallback for that group and records `prose-contradiction` (FU-S28-1, applied by S28-b in FG-c's merged code, §11.1) | Both; (d) only LLM |
| **C-5 One fact, one truth** | Two persons who recall the same fact see the same time, place and participants in their lines' citations. This holds trivially through one record, and is checked against digests | Both |
| **C-6 Persistence** | After a kill at any of 50 seeded points and resume, every seat view and section equals the uninterrupted run's | Both |
| **C-7 Forgetting is coherent** | Every perceived id is held, merged into a surviving unit, or forgotten (FGb-1) per seat view. A forgotten id is never cited. `inspect access --event` explains every id of a sample of 1 000 | Both |
| **C-8 Life accumulates** | Over 300 days on the default stack, from day 8 on every seat's L2 section is non-empty at each day's first consult, and L3 lists the seat's most-met counterparts. The person "knows their town" at any age, within 6 000 B | Both |

---

## 10. Migration from P4/FG-b schema v2

- **Clean change, no compatibility debt** (`CLAUDE.md` §4.12; precedent QFGb-7). The home is schema v3. A v2
  per-seat file is refused by name with `UnsupportedStoreSchema(found=2, wanted=3)`, and the message gives the
  rebuild instruction. Memory is a derivation, so rebuilding from a null cursor (`perceived: { since: null }`)
  reproduces it, and FG-c prose is reproduced with its recordings.
- **No importer.** An importer would be code kept for a format that has no stable contract.
- **P6's paths:** `store/<instance>/<seat>.sqlite` becomes one `store/<instance>/memory.sqlite`. P6's
  `<seat>.resume` and `<seat>.decisions.jsonl` are unchanged (they are not memory). S28-f wires it.
- **Golden renders** (C-2) are captured from v2 code at S28-a's C0 and committed as test fixtures, so the
  oracle survives v2's deletion.

---

## 11. PR split

Each PR gets its own frozen design. "Haiku" means the PR is suitable for a Haiku implementer once its design
is frozen at commit level, with the primary session reviewing (memory: *design on Opus, implement on Haiku*).
"Opus" means the invariants are too easy to break subtly and the implementer should be Opus.

| PR | Scope | Depends on | Implementer | MVP (QS28-1) |
| --- | --- | --- | --- | --- |
| **S28-0** | Spec edits (§13): CORE_CONCEPTS §5.3 (shared storage is not shared knowledge), ARCHITECTURE §9.3 (homes, seat views, the size rule), MODULE_SPEC §5.2 and a new §5.3 (memory scopes), DECISIONS `ARC-98` … `ARC-101`, `DEP-58`. Docs only | this step frozen | Opus / primary | 0 |
| **S28-a** | `memory.home` and `memory.scopes.world`: schema v3, the fact, fact_who and access tables, insert-or-verify, `SeatView` and its dump, positional canonical form (QS28-5, ruled), one transaction per ingest, GC, `verify`, `HomeBusy`, v2 refusal. The transparency harness (12 seats, `none` vs `home`, frames, interleavings, kills). Golden v2 renders | FG-b2 merged | **Opus** | 0 |
| **S28-b** | The retention host on seat views: items, execution with GC, FG-b2's size trigger on the seat view, FG-c prose in the home, forgotten-range attribution, C-4 (a)–(c). **Plus the FG-c follow-up FU-S28-1** (C-4 (d), below) in FG-c's merged code | S28-a; FG-c (#201, merged) | **Opus** | 0 |
| **S28-c** | `MemoryScope` interface, `ScopeRegistry` (entry points), `person` and `world` as scopes, selection through `decisions.yaml` `memory.scopes` (a small Rust reader addition beside DL-a's `MemoryPolicyChoice`), parameter models, named errors, section ceiling checks, the generated reference pages (25b) | S28-b; 25a (#200) | Haiku | 0 |
| **S28-d** | The `anchored` scope with `place` and `item` entry points; `Rendered.anchors`; the recall section; C-3 with its test oracle | S28-c | Haiku | 0 |
| **S28-e** | `mineworld-memory inspect` (access, forgotten, bytes, recall --trace, scopes) and `export` / `import`; the one-step-to-cause tests (AS28-16) | S28-b (views grow with S28-c and S28-d) | Haiku | 0 |
| **S28-f** | P6b wiring: the controller opens the home, `sharing` and `cache` from `cognition.toml`, cassette replay criterion AS28-10, the README lines | S28-c; P6b (#207) merged | Haiku | 0 |
| **S28-g** | Scale gates: 12 × 300 days in nightly CI, and 100 × 300 days on a generated fixture World Pack (`tests/fixtures/worlds/town-100`, generated by a committed tool) in nightly CI on all three platforms; cost-formula checks (AS28-17) | S28-d, S28-f | Haiku | 0 (nightly) |
| **S28-h** | The cold tier through the server (`perceived_by_id`, §6.6 (i)): a protocol addition, a server handler, and the SDK. `organization` as an external example scope package | S28-g; QS28-2 ruled path (i) | Opus | **1** |

Order: S28-0 → S28-a → S28-b → (S28-c → S28-d) ∥ S28-e → S28-f → S28-g; S28-h in MVP-1. Parallel lanes: S28-d
and S28-e in separate worktrees after S28-c merges; S28-e may begin on S28-b's views.

### 11.1 Start conditions and implementers (QS28-1 ruled; filled 2026-10-14)

Common to every PR:

- The step is frozen by the primary session.
- The PR's own design is frozen with a filled execution contract.
- The primary session dispatches the PR to a **fresh** implementation session in its **own worktree**
  (`/Users/yuema137/mineworld-worktrees/impl-s28-<x>`), one writer, from `origin/main` at dispatch.
- CI on the exact head is the gate. No local full-workspace runs while other agents run (R-LOAD-1).
- No merge without the operator's authorization, except that S28-0 is a design-document-only PR, which the
  primary session may merge after review under the standing ruling.

| PR | Start condition (all must hold) | PR design by | Implementer | Reviewer | MVP | Parallel with |
| --- | --- | --- | --- | --- | --- | --- |
| S28-0 | Step frozen | — (step doc is its design) | Opus (primary or a docs agent) | primary | 0 | S28-a's PR design |
| S28-a | S28-0 merged. **FG-b2 merged** (bound, size trigger). Golden v2 renders captured at C0 from that `main` | Opus | **Opus** | primary + an Opus logic review | 0 | — |
| S28-b | S28-a merged. FG-c already merged (#201, `85a91d0`) | Opus | **Opus** | primary + an Opus logic review | 0 | S28-c's PR design |
| S28-c | S28-b merged. S25 25a (#200) merged (generated-reference machinery). 25b's page list known (coordinate with S25) | Opus | Haiku | primary | 0 | S28-e |
| S28-d | S28-c merged | Opus | Haiku | primary | 0 | S28-e |
| S28-e | S28-b merged. The views of S28-c and S28-d are added as each merges (the commit plan says which) | Opus | Haiku | primary | 0 | S28-c, S28-d |
| S28-f | S28-c merged. **P6b (#207) merged** | Opus | Haiku | primary | 0 | S28-d, S28-e |
| S28-g | S28-d and S28-f merged. The nightly workflow exists (S14) | Opus | Haiku | primary | 0 (nightly gate) | — |
| S28-h | S28-g green on nightly. MVP-1 opened by the operator | Opus | Opus | primary | 1 | `organization` example |

**FU-S28-1 — the FG-c follow-up that S28-b applies in FG-c's code (C-4 (d)).** FG-c merged with `valid_prose`
(`compress/model_summarizer.py`) checking only length, digits and the delimiter, and `lm_summarise.py`
accepting any prose that passes it (audited at `85a91d0`). S28-b adds the name check:

- prose for a group may name only known names of that group's counterparts, and the group's place;
- any other known person or place name makes the answer invalid for that group, so the group keeps the
  fallback's decisions;
- the host records the reason as `prose-contradiction`.

Recorded answers that break the rule now take the fallback on replay. S28-b states which cassettes change and
re-records none silently. FG-c's design document gets a pointer to this follow-up, applied by S28-b's C1.

**Integration checkpoint per PR.** Every PR ends with the transparency harness green (12 seats, `none` vs
`home`). From S28-b on, it also runs FG-b's real-binary Gate 2 scenario on seat views.

---

## 12. Decision records (ids assigned 2026-10-14; written into `docs/DECISIONS.md` by S28-0)

- **`ARC-98` — Shared controller memory is a cognition-side home of perceived envelopes, and perception is
  the only access right.** Fact records are stored once per home, keyed by `EventId` and verified by digest.
  Access rows are per person. GC is the anti-join in the releasing transaction. There is no kernel, server or
  protocol change in MVP-0.
- **`ARC-99` — A person's size is the seat view, counting shared records in full.** Sharing is transparent
  (I-S28-2, I-S28-3) and is a deployment setting, not a world rule.
- **`ARC-100` — Memory scopes:** one typed interface, entry-point registration, host-written tables,
  `SeatView`-only reads, no access grants. Entity-anchored memory is fed from perception, not the log.
- **`ARC-101` — Canonical v3 encoding** (QS28-5, ruled 2026-10-14): positional rows, which the bound and
  determinism are defined on.
- **`DEP-58` — Reuse verdicts** (§16): build on SQLite and P4/FG-b; adopt the methods of content addressing
  and tiered memory; decline Letta, Zep, Graphiti and mem0 as runtime dependencies.

## 13. Edits to other documents (S28-0 applies the specification edits; the primary session applies `overall.md`)

- `CORE_CONCEPTS.md` §5.3: add "Storing a perceived fact once for several persons is a storage detail. Each
  person still remembers only what they perceived, in their own rendering. Shared storage is not shared
  knowledge."
- `ARCHITECTURE.md` §9.3: add homes, seat views, the size rule, GC, the transparency invariant, and the bound
  measured on the seat view (with FG-b2's numbers).
- `MODULE_SPEC.md` §5.2: items' sizes are seat-view sizes. Add a new **§5.3 Memory scopes**: the interface,
  registry, selection, parameters and errors.
- `overall.md`: the S28 entry, PR list, and ids after assignment. The progress page is updated per the
  primary's practice.
- `pr-fg-c-lm-summarise.md`: a pointer to FU-S28-1 (§11.1), the prose-names check that S28-b applies in FG-c's
  code (C-4 (d)).

---

## 14. Operator questions and rulings

### 14.1 Rulings, 2026-10-14 (operator, in the primary session; relayed by the coordinator)

Verbatim:

> **QS28-1:** "MVP-0 做主体，跨进程共享放 MVP-1"
>
> **QS28-5:** "换"
>
> **QS28-2, 3, 4, 6, 7, 8, 9, 10:** "全部按推荐"

How each ruling reads, and where it is applied:

| ID | Ruling | Applied in |
| --- | --- | --- |
| QS28-1 | The main body is MVP-0: S28-0 and S28-a … S28-f, with S28-g as MVP-0's nightly gate. Cross-process sharing (S28-h) and the `organization` example are MVP-1 | header, §11, §11.1 |
| QS28-2 | Share per controller process in MVP-0. Per world in MVP-1 through the server as the cold tier (§6.6 (i)). A cognition-side shared service is not the default | §5.4 (`service` refused before MVP-1), §6.6, S28-h |
| QS28-3 | Inner notes are always private. No scope may share them | §5.2.5 |
| QS28-4 | Per-person size counts the seat view, with shared records in full | §7.1, `ARC-99` |
| QS28-5 | Adopt the positional canonical encoding, in S28-a | §8.1, S28-a, `ARC-101`, AS28-12 |
| QS28-6 | Entity-anchored memory is fed from perception only. The log-derived place history is a test and tool oracle only | §5.2.3, C-3 |
| QS28-7 | A scope never grants access beyond perception. Knowledge reaches others as world facts | §5.1, §5.2.4, `ARC-100` |
| QS28-8 | Refuse v2 and rebuild from the perceived stream. There is no importer | §10 |
| QS28-9 | The 100-person world is a generated fixture under `tests/fixtures/`, used by the nightly gate only | S28-g |
| QS28-10 | The memory stack lives in `decisions.yaml` `memory:` per seat; placement and cache live in `cognition.toml` | §5.3, §5.4 |

The decision ids were assigned the same day: `ARC-98` … `ARC-101` and `DEP-58` (header). No question remains
open.

### 14.2 The questions as asked (kept for review)

| ID | Question | Recommendation (accepted) |
| --- | --- | --- |
| **QS28-1** | **MVP-0 or MVP-1?** Which S28 PRs land before the MVP-0 release? | **MVP-0: S28-0, a, b, c, d, e, f, with g as a nightly gate. MVP-1: S28-h (cross-process sharing and the cold tier) and the `organization` example.** FG-b2's 20 MiB makes duplication 20× larger in absolute bytes now, and the scope interface is cheapest to establish before users write stacks. |
| **QS28-2** | **What scope of sharing?** Per controller process (one home per process and world instance) or per world (every seat of a world, across processes and machines)? | **Per process in MVP-0.** It is atomic in one SQLite file, needs no protocol change, and is proven transparent. **Per world in MVP-1 through the server as the cold tier** (§6.6 (i)): the server is the only party that can verify perception, and the log already stores each fact once. Decline a cognition-side shared service as the default (it cannot verify access). |
| **QS28-3** | **Do inner notes stay private?** Do model-written notes or reflections (none exist yet) stay private for ever, or may a scope share them? | **Always private.** A belief reaches another person only by being said or written in the world, which makes it a fact a System decides and perception delivers. Sharing notes directly would let models exchange knowledge outside the world, breaking INV-13 and "valid with the models unplugged". |
| **QS28-4** | **How is per-person size counted?** In full (the seat view, as if alone), or as a fair share of shared records? | **In full.** A fair share couples one person's memory to everyone else's forgetting (I-S28-3), breaks transparency, and makes the 20 MiB promise mean "it depends on the town". In full, every person keeps the promise exactly, and sharing saves the world's disk (59 % or 23 % of logical, measured). |
| **QS28-5** | **Adopt a positional canonical encoding in v3?** This changes what 20 MiB buys: ≈ 106 days of full detail instead of ≈ 45, and world bytes −77 % instead of −41 %. | **Yes, in S28-a.** It is one schema break instead of two. It is the same determinism contract on a leaner form, and the generated reference documents it. |
| **QS28-6** | **Where is entity-anchored memory fed from?** Perception, or a world-log read surface? | **Perception only** for memory. The log-derived place history exists only as a test and tool oracle (C-3), never as an input to a controller (INV-13, INV-4). |
| **QS28-7** | **May a user scope grant access beyond perception?** For example, an Organization's archive readable by every member. | **No.** A scope can organize and recall, but never grant. An organization's knowledge reaches a member as a world fact (a System's `briefed` or `notice-read` event), which perception delivers and the log keeps. |
| **QS28-8** | **How do v2 stores migrate?** Refuse v2 and rebuild from the perceived stream, or write an importer? | **Refuse and rebuild** (precedent QFGb-7). There is no stable contract yet, and an importer is compatibility debt. |
| **QS28-9** | **What 100-person fixture?** A generated 100-person fixture World Pack under `tests/fixtures/`, or a 100-person reference world? | **A generated fixture**, built by a committed tool from `social-cafe`'s packs, used only by the nightly gate. A 100-person reference world is a product decision, separate from S28. |
| **QS28-10** | **Where do settings live?** Memory stack (scopes, policy, bound) in `decisions.yaml` `memory:` per seat, and placement and cache in `cognition.toml`? | **Yes.** A stack changes what a person recalls, which is the world author's call. Placement provably does not, which is the operator's call. |

---

## 15. Acceptance and adversarial criteria (fixed now, before anything is measured; `ARC-23`)

| ID | Criterion |
| --- | --- |
| **AS28-1 Access oracle** | 12 seats, 30 days. Every key in every seat's access index, and every citation in every section of 200 seeded queries per seat, is in that seat's `mineworld perceived --json` export. 0 exceptions. |
| **AS28-2 Hostile scope** | A fixture scope that cites unheld keys, a peer's keys, forgotten keys and never-perceived keys gets `AccessDenied` for each. No such line reaches a section, and the hostile scope's tests never hold a connection. |
| **AS28-3 Transparency** | For 12 seats, `SeatView.dump()` under `sharing: home` equals the standalone dump under `none`, byte for byte. This holds at every pass point and at the end, with frames of 1, 7, 256 and 4 096 and three seeded seat interleavings. |
| **AS28-4 Section identity** | For the same runs, 200 seeded `RecallQuery`s per seat render byte-identical sections under `none` and `home`. |
| **AS28-5 Isolation** | Seat A's dump and sections are identical when its home-mates run `keep-all`, `decay` with threshold 1 000, or `importance-recency-cap` with every k at 0, compared with A alone. |
| **AS28-6 GC exactness** | After every commit, under `verify = every-commit`, the fact keys equal the union of held keys (both anti-joins empty) and no scope row's key is unheld. |
| **AS28-7 Last holder** | A fact held by exactly 2 seats survives the first seat's `Drop` and is collected by the second's, in that transaction. Reversing the order gives the same final dump. |
| **AS28-8 Kill/resume** | 50 seeded SIGKILLs, including inside ingest after the fact inserts and before commit, and inside a pass after the access deletes and before GC. After resume and re-delivery, the home dump equals the uninterrupted run's. |
| **AS28-9 Physical ≤ logical** | At every pass point, `MemoryHome.dump()` length ≤ Σ seat-view dump lengths, and every seat view ≤ its bound. With `keep-all` everywhere, the bound holds and physical ≤ N × bound. |
| **AS28-10 Cassettes** | P6b's recorded cassettes (recorded with v2, or with `sharing: none`) replay under `home` with zero `CassetteMiss` and identical decision logs. |
| **AS28-11 Export / import** | `export` then `import` of every seat into a fresh home gives a home dump equal to the original's. |
| **AS28-12 Efficiency targets** | On social-cafe, seed 7, 30 days, before retention, the home's world-day bytes in the v3 positional encoding (QS28-5, ruled) are ≤ 0.30 × v2's; the canonical-objects layout is not built. Steady state at 300 days (nightly): physical ≤ 0.70 × Σ logical. |
| **AS28-13 No truth, no kernel** | Source scan: no import of `persistence`, no save path and no `sqlite3.connect` outside `memory.home` in `memory/` or `forgetting/`. Diff scan: no change under `kernel/`, `contracts/`, `server/` or `server/PROTOCOL.md` in S28-a … S28-g. |
| **AS28-14 Scopes write nothing** | Scope modules receive no connection, path, clock or gateway (signature scan). Only the home executes SQL writes (grep for `INSERT` / `DELETE` outside `memory.home` and the person scope's ingest). |
| **AS28-15 Degradation** | Removing `place` from a stack, uninstalling its package (`UnknownScope` named with the installed ids), and reinstalling it all behave as §5.7 states. A rebuilt index is identical to an uninterrupted one. The world's save and `mineworld replay` are unaffected throughout. |
| **AS28-16 One step to the cause** | (a) A planted wrong gist in seat B's access row is named by `inspect recall --trace` (seat, scope, key, row) in one command. (b) A planted oversized scope row is named by `inspect bytes` (scope, table). (c) A planted bound below 20 MiB fails at start with `BoundBelowDefault` naming the seat and the source span. |
| **AS28-17 Scale** | Nightly, on all three platforms: 12 × 300 days, and the generated 100-person world × 300 days. Every seat view ≤ bound at every pass point. Home file ≤ physical × 1.25 + 8 MiB. Process RSS ≤ `page_cache_kib` + `cache_records` × 512 B + 256 MiB. Rebuild determinism on a sampled 10 seats. |
| **AS28-18 Continuity C-1 … C-8** | Each criterion of §9.2 as its own test, on social-cafe, 30 days (CI) and 300 days (nightly). |
| **AS28-19 Configuration errors** | Each named error of §5.5 is raised by a fixture with its exact type and fields, and never echoes a parameter's value. |

---

## 16. Reuse, in both directions (considered 2026-10-14)

| Candidate | What it is | Licence | Verdict |
| --- | --- | --- | --- |
| SQLite (`sqlite3`, stdlib) | The store already in use (`DEP-37`) | Public domain | **Keep.** One file per home, FTS5, incremental vacuum |
| Content-addressed stores (Git objects, IPFS / `multiformats`, Perkeep, restic, borg) | Hash-keyed immutable blobs, dedup | Git GPL-2.0; IPFS implementations MIT / Apache-2.0; restic BSD-2; borg BSD-3 (**verify** each) | **Adopt the method, decline the code.** Our objects already have a stable world id (`EventId`). Hash-keying would add a second identity, and the blobs are ≈ 100 B rows in an existing SQLite file |
| Ref-counted blob stores (`diskcache`, LMDB-backed stores, CAS caches) | Key-value with eviction or refcounts | `diskcache` Apache-2.0; LMDB OpenLDAP (**verify**) | **Decline.** A stored refcount can drift from its references. The anti-join over `access` is the refcount, exact by construction, in the same transaction |
| Generative Agents memory stream (Park et al., 2023) | Per-agent stream of observations; recency, importance and relevance retrieval; reflection | Code Apache-2.0 (**verify**) | **Reference only**, as step-17 and FG-b already decided. Each agent's stream is its own. Its "shared" world state is the environment, not memory, which matches our split (log ≠ memory) |
| MemGPT / Letta tiered memory | Core, recall and archival tiers, model-managed paging; Letta supports shared memory blocks across agents (**verify**) | Apache-2.0 (**verify** the current Letta licence) | **Adopt the method, decline the dependency.** Hot and cold tiers and "load when needed" are the method (§4.5, §6.6). Its shared blocks are agent-writable shared text, which is a knowledge channel outside the world (QS28-3, QS28-7). Model-managed paging is not a deterministic derivation (`ARC-59`) |
| Zep / Graphiti | A temporal knowledge graph of episodes, entities and facts, shared across sessions | Graphiti Apache-2.0; Zep Community Edition Apache-2.0, since deprecated in favour of Zep Cloud (**verify**); needs Neo4j (GPL-3.0 community) or FalkorDB | **Decline.** Model-extracted triples are not a deterministic derivation. It needs a graph server, and its sharing is by tenant, not by perception. Our `place` and `item` anchors are a much smaller, exact version of its entity index |
| mem0 | An LLM-extracted memory layer with user, agent and session scopes | Apache-2.0 (**verify**) | **Decline.** Its scopes are application tenancy, and its writes are model-decided. The idea of scopes as first-class configuration is adopted (§5) |
| Forcing an existing wheel | — | — | Not done: no framework's execution model owns our transactions, access rule or determinism (`REUSE_POLICY`) |

---

## 17. Risks

| Risk | Mitigation |
| --- | --- |
| The one-file home becomes a contention point for many seats | One asyncio loop already serializes ingest. Transactions are short (one frame). Measure at 100 seats (AS28-17). If needed, shard homes by seat group: transparency holds for any grouping |
| A subtle dependency of a seat view on peers (via ordering, rowid, FTS rowids) | Dumps never include rowids. Ordering is by keys. The harness compares against `none` at every pass point, with interleavings (AS28-3) |
| The size counter drifts from the dump | `verify` modes, plus AS28-9 in CI with `verify = pass` |
| FG-b2 or FG-c change the v2 schema again before S28-a | S28-a starts after both merge. C0 re-audits and refreshes the golden renders |
| The positional encoding makes dumps harder to read for a human | `inspect` reads them. The dump is a determinism artefact, not a UI |
| Windows file locking for `HomeBusy` | Use SQLite's own locking plus an `msvcrt` / `fcntl` lock file. Tested on all three (AS28-17) |
| The 100-person fixture and 300-day runs are too heavy for CI | Nightly only (R-LOAD-1: CI is the gate; no local full runs) |
| Users expect "town memory" to tell persons things they did not see | Documented in the guide and specs: scopes organize, Systems disclose (QS28-7) |

---

## 18. What remains after this step

- Cross-process and cross-machine sharing, and the cold tier (S28-h, MVP-1).
- An `organization` example scope as an external package (MVP-1).
- Embeddings or semantic recall as a scope (not proposed: it would need a model or a vector library, and a
  `DEP` record of its own).
- A client-side memory page reading `inspect recall` (a presentation step's choice).
