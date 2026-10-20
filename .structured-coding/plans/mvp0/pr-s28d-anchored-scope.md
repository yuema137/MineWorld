# PR S28-d — The anchored scope: `place` and `item` memory, payload anchors, the scope-table host, the recall section and C-3

## DRAFT — not frozen (awaiting the operator's rulings on §14)

```text
Design revision:        draft 1 (2026-10-19)
Author:                 S28-d PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s28d, branch docs/pr-design-s28d,
                        planning base origin/main @ 1308db6d (#329). On that base: S28-a merged (#242), FU-S28a-1
                        merged (#262), the S28-b, S28-c and S28-e designs frozen and merged as documents; the
                        S28-b and S28-c implementations NOT merged (both in progress in their own worktrees)
Approved by / evidence: none yet (§14 open)
Implementation base:    origin/main at C0, after S28-c has merged; C0 records the commit
Start condition:        step-28 §11.1: S28-c MERGED; this design frozen; the coordinator's dispatch
Execution contract:     §12 (filled; endpoint authority included)
Implementer:            Haiku-class (step-28 §11.1), the coordinator reviewing
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-28-shared-memory.md`](step-28-shared-memory.md) (STEP DESIGN FROZEN
2026-10-14; rulings §14.1) §§3 (I-S28-1, I-S28-2, I-S28-7, I-S28-8, I-S28-9, I-S28-10), 5.2.3, 5.3, 5.4 (the
`place` / `item` rows), 5.7, 5.8, 7.2 (the scope-private backstop step), 7.3, 9.2 (C-3), 11 and 11.1 (the S28-d
row), 15 (AS28-1, AS28-2, AS28-14, AS28-15, AS28-18 for C-3) · **Parent:** [`overall.md`](overall.md) ·
**Previous PRs:** [`pr-s28a-memory-home.md`](pr-s28a-memory-home.md) (merged #242; `fact_who.role`, QS28a-3),
[`pr-s28b-retention-on-seat-views.md`](pr-s28b-retention-on-seat-views.md) (FROZEN; schema 5; R-S28c-b-1),
[`pr-s28c-memory-scopes.md`](pr-s28c-memory-scopes.md) (FROZEN 2026-10-18; its R-S28d-c-1 is most of this PR's
host work) · **Coordinates with:** [`pr-s28e-memory-inspect.md`](pr-s28e-memory-inspect.md) (its R-S28d-e-1) ·
**Followed by:** S28-f (P6b wiring), S28-g (scale gates; the anchored cost formula) · **Binding parents:**
`CLAUDE.md` §§2, 3, 4 (items 1, 2, 3, 5, 7, 8, 9, 10, 11, 12, 13, 16); `docs/CORE_CONCEPTS.md` §§2 (INV-4,
INV-13), 5.2, 5.3; `docs/ARCHITECTURE.md` §9.3; `docs/MODULE_SPEC.md` §5.3; `docs/DECISIONS.md` (per-file records)
`ARC-59`, `ARC-87`, `ARC-98` … `ARC-101`, `ARC-109`; the lane-lead charter (speed first; no world-specific
hard-coding; tools first).

**Decision ids.** None new. C1 writes a dated note under `ARC-100` (anchors, the scope-table host, the merge
rules, eviction) and under `ARC-109` (schema 6), once §14 is ruled.

No hosted model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Scope (step-28 §11, S28-d row).** "The `anchored` scope with `place` and `item` entry points;
  `Rendered.anchors`; the recall section; C-3 with its test oracle." Implementer Haiku; depends on S28-c.
- **R-S28d-c-1 (S28-c §10, verbatim).** "S28-d implements the scope-table host (QS28c-4 as ruled): `TableSpec`
  DDL in the home, `index` called by `SeatView.hold` for each selected optional scope, cascades by `event_key`
  (in `world.collect`'s transaction) and by `seat`, a home table of installed scope tables (schema 6),
  `scope-removed:<id>`, rebuild on reinstall, AS28-14's runtime half, AS28-15; `GuardedView` gains the scope's
  own-row reads it needs. The interface's members do not change."
- **R-S28c-b-1 (S28-b §10, verbatim), now S28-d's (S28-c §2.2).** "The scope-private backstop step (step §7.2)
  signs its drops `host:scope-rows` if it ever adds a forgotten range (it should not: scope rows cite, they do
  not cover)."
- **R-S28d-e-1 (S28-e §10, verbatim).** "The anchored scope's tables appear in `bytes` through `account`, its
  lines in `recall --trace` through `SectionLine.level`, and its tables kept for an uninstalled scope in
  `scopes`; S28-d adds no view of its own unless its design shows a question these cannot answer. `export` /
  `import` carry its per-seat rows (and rebuild its shared index rows from the imported records by `index`)."
- **The anchored scope as the step writes it** (§5.2.3): one implementation, two entry points; fed from
  perception, never the log; an `anchor (event_key, entity, role)` index; private `anchor_known` rows capped at
  `anchors_max`; an optional recall section; payload-level anchors through `Rendered.anchors`. §3.2 lists every
  sentence of it that the audited code makes inexact.
- **C-3 (step §9.2, verbatim).** "Every line of a `place` section cites only held facts whose envelope anchors
  that place in a selected role. Its `visits`, `first_at` and `last_at` equal a recount over the person's
  `mineworld perceived` export filtered by that place, up to the forgotten set. Every cited fact appears in the
  objective place history derived from the log by the test oracle (a test-side filter over `mineworld perceived`
  and the save, never memory's input)."
- **Rulings not re-asked.** QS28-6 (entity-anchored memory is fed from perception only; the log-derived history
  is a test oracle only), QS28-7 (a scope never grants access), QS28-10 (the stack in `decisions.yaml`), QS28c-1
  (base scopes; optional scopes read a `GuardedView`), QS28c-2 (an optional scope cites held keys only, at both
  ends), QS28c-3 (a defect at consult time withholds that scope), QS28c-4 (the table host lands here).
- **Lane-lead charter (binding).** Speed first: merge on a green `fast`; heavy tests at the milestone. No
  world-specific hard-coding: every mechanism is data-driven, and every acceptance criterion names a second world
  or a generated variant. Tools first: reuse what exists before writing new machinery.
- **R-LOAD-1 (coordinator rule, in force).** §11.

---

## 1. Goal

Let a person remember **places and things**, not only people, from what they perceived, with the default stack
unchanged in what it recalls:

1. Every perceived fact's **payload anchors** — the places, items and organizations its payload names by the
   contract's typed reference `{"entity": …, "entity_type": …}` — are read once at ingest by one generic scan,
   carried as `Rendered.anchors`, and kept beside the seat's access row. No pack's event type is named in this
   code.
2. The **scope-table host** of S28-c's interface exists: an optional scope declares per-seat tables, the home
   creates them, writes the rows `index` returns, merges them by declared rules, caps them, dumps and meters them
   as part of the seat view, drops them when a stack stops naming the scope, and rebuilds them when it names it
   again.
3. The bundled scopes **`place`** and **`item`** are two configurations of one `anchored` implementation. Each
   keeps one "known" row per anchor (like L3 does per person) and offers an optional recall section about the
   anchors the person is with now: the observation's place, and the items and places it lists.
4. **C-3** holds and is tested with an oracle that never runs memory code: on a synthetic stream shaped like
   `social-cafe`, one shaped like `lakeside` (whose `fished` payload names the rod as an item), and a seeded
   generated variant with random ids, kinds, names and payload shapes.

Nothing changes under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/` or `sdk/`
(I-S28-10). No World Pack turns the new section on: its default is off (`section_bytes: 0`).

---

## 2. Scope

### 2.1 In scope

1. **Payload anchors** (`memory/anchors.py`, new): `Anchor`, `AnchorKind`, `payload_anchors(fact)`;
   `Rendered.anchors` filled by `render()` for every renderer (§5.2).
2. **Schema 6** (`memory/home/schema.py`): the per-seat table `access_anchor` and its view table; the home tables
   `scope_tables` and `scope_seats`; one index. `SeatView.hold` writes anchors, `release` deletes them; schema 5 is
   refused by name (§5.3).
3. **The scope-table host** (`memory/home/scope_tables.py`, new; `memory/home/home.py`, `memory/home/view.py`):
   DDL from `TableSpec`, the registry of installed scope tables, `MemoryHome.bind_scopes`, `index` at `hold`, the
   merge rules, the cap, the seat cascade, dump and meter, removal and rebuild (§5.4).
4. **The interface's carriers** (`memory/scopes/interface.py`, `memory/scopes/registry.py`): `Reading.anchors`;
   `TableSpec.columns` as typed `Column`s with merge rules, `TableSpec.cap` and `TableSpec.evict`; three
   `ScopeView` reads (`anchored`, `own_rows`, `shown`), implemented once in `memory/scopes/reads.py` (§5.5).
5. **The anchored scope** (`memory/scopes/anchored.py`, new): `AnchoredParameters`, `AnchoredScope`,
   `PlaceScope`, `ItemScope`; entry points `place` and `item` (§5.6).
6. **Query anchors** (`memory/retrieve.py` `RecallQuery.anchors`; `controller/situation.py` `anchors_of`;
   `controller/seat.py`) (§5.7).
7. **Retention** (`memory/retention/items.py`, `memory/retention/bound.py`): anchor bytes belong to their record;
   scope rows are overhead; the backstop step "drop scope rows" before "drop stable facts" (§5.8).
8. **The controller binds the stack** at seat open and logs a `scope-change` line (`controller/seat.py`,
   `controller/log.py`) (§5.9).
9. **C-3 and its oracle** (`tests/anchor_oracle.py`, `tests/anchor_streams.py`, new; test-only, importing no code
   under test) (§6).
10. **The reference:** `docs/reference/memory-scopes.md` and its schema regenerated (S28-c's generator picks up the
    two new entry points by itself).
11. **Specifications:** `docs/MODULE_SPEC.md` §5.3 corrected to the code and the rulings; dated notes under
    `ARC-100` and `ARC-109` (C1).
12. **R-S28d-e-1**, only if S28-e has merged at C0 (§10, QS-precedent R-P6b-a-1): `export` / `import` carry
    `access_anchor` and the scope tables; `recall --trace` names `place` / `item` lines by `SectionLine.level`.
13. **Tests** of §7.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| A shared, event-keyed scope index table, its cascade in `world.collect`, and the step's `granularity: episode` mode | **Not built** (QS28d-3, F-S28d-4): the envelope is already indexed by `fact` / `fact_who`, and an SQLite index costs no dumped byte. The first scope that needs an event-keyed table adds its hosting |
| The roles `subject` and `participant` for anchors | deferred to the first scope whose anchors are untyped envelope entities, for example `organization` (MVP-1) (QS28d-1, F-S28d-2) |
| `cognition.toml [memory]`, `sharing`, `verify` (and `verify`'s Σ `account` = `content_size()` check, R-S28f-c-1) | S28-f |
| The 12 × 300-day and 100-person gates, and the anchored cost formula's measurement | S28-g |
| A World Pack that turns a place or item section on | never in this PR (I-S28-10); a world author's choice |
| `organization` as an external example scope | S28-h / MVP-1 |
| Any change to a retention policy, an item rule, a bound number or the pass order other than §5.8's one step | never (material stop, §12) |
| Any change under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` | never (I-S28-10) |

### 2.3 Dependencies and the start condition

- **Required: S28-c merged.** This PR edits the files S28-c creates (`memory/scopes/{interface, registry, host,
  person}.py`, `controller/memory_view.py`'s new signature). C0 re-anchors §3 on the merged code; a moved line is
  bounded, a changed statement that breaks §5 is a material stop.
- **S28-c's frozen interface is the contract designed against.** Every name in §5 that S28-c defines
  (`MemoryScope`, `ScopeParameters`, `SectionSpec`, `SectionBudget`, `ScopeView`, `GuardedView`, `FactRecord`,
  `Reading`, `TableSpec`, `ScopeRow`, `BoundScope`, `MemoryStack`, `resolve_stack`, `host.recall`, `Recalled`,
  `ScopeDefect`, `SectionKey`) is used as `pr-s28c-memory-scopes.md` §5 states it.
- **Merged:** S28-a (#242), FU-S28a-1 (#262), S28-b (required by S28-c), 25a / 25b (the reference generator).
- **Not required:** S28-e. C0 records whether it has merged; that decides §2.1 item 12 (§10).

---

## 3. Audit (`origin/main @ 1308db6d`)

Line numbers are on `1308db6d` (schema 4; S28-b and S28-c not merged). C0 re-anchors each on the base.

### 3.1 Source anchors (under `cognition/lm-controller/src/mineworld_cognition/` unless a path says otherwise)

| # | Fact | Where |
| --- | --- | --- |
| D-1 | `Rendered` is a frozen dataclass: `gist`, `counterparts`, `salience`, `mine`, `importance`, `level`, `names`, `fallback`. `render()` picks a renderer by event type, else `generic`. A renderer is pure in `(fact, me, names)`. | `memory/render/__init__.py:37-50`, `:144-145` |
| D-2 | **Typed references are the contract's.** A typed id (`PlaceId`, `ItemId`, `PersonId`, `OrganizationId`) serializes as `{"entity": "<id>", "entity_type": "<kind>"}` (`TypedEntityRef`); `EntityType` is `person`, `place`, `item`, `organization`. | `contracts/src/ids.rs:850-900`; `sdk/python/src/mineworld_sdk/wire/contract.py:43`, `:60-64` |
| D-3 | **Items appear in payloads, not in the envelope.** Inventory's `items-transferred` puts the item in the payload (`item: ItemId`) and only the two persons in `participants` and `subjects`. Lakeside's `fished` payload names the rod as `{"entity": …, "entity_type": "item"}` (the render test's fixture). The movement pack's `walk-started` nests a place reference three levels deep (`destination.place.place`). | `systems/inventory/src/admit.rs:104-111`; `cognition/lm-controller/tests/test_memory_render.py:87`; `tests/memory_fixtures.py:220-230` |
| D-4 | `PerceivedEvent` = `id`, `at`, `event_type`, `subjects`, `participants`, `place: PlaceId | None`, …, `payload`. Subjects and participants are bare `EntityId`s: the envelope carries no kind for them. | `sdk/python/src/mineworld_sdk/wire/contract.py:303-319` |
| D-5 | `ingest` renders each new fact, teaches its names, then calls `store.hold(fact, fingerprint, rendered)` inside one transaction. | `memory/ingest.py:108-114` |
| D-6 | `SeatView.hold` inserts or verifies the record (`world.insert_or_verify`), inserts the access row and its FTS row, and feeds the meter the `l0` and `l0_who` line sizes. `release` deletes FTS rows and access rows, updates the meter and calls `world.collect`. | `memory/home/view.py:184-232`, `:234-251` |
| D-7 | `SCHEMA_VERSION = 4`. `PRIVATE` lists the private view tables in dump order; `SEAT_TABLES = (META, NAMES, *PRIVATE)`; `VIEW_TABLES = (META, NAMES, L0, L0_WHO, *PRIVATE)`; `KEY_COLUMNS` (encoded as numbers) holds `event_key`, `first_key`, `last_key`, `level_key`, `source_key`, `trigger`. A `ViewTable`'s name is both its SQL table and its dump tag. | `memory/home/schema.py:23`, `:93-95`, `:116-128`, `:140-224` |
| D-8 | **Adding a per-seat table to `PRIVATE` is enough** for: the home dump (`_seat_lines` walks `PRIVATE`), the seat view dump (`VIEW_TABLES`), the meter (`METERED = (NAMES, *PRIVATE)`, `TEMP` triggers per table), and `remove_seat` (walks `SEAT_TABLES`). | `memory/home/home.py:196-212`, `:311-333`, `:352-372`; `memory/home/view.py:54-55` |
| D-9 | `UnsupportedStoreSchema` refuses any other schema at open; the creation marker is never dumped. | `memory/home/home.py:88-106`, `:415-420`; `schema.py:79-85` |
| D-10 | **Retention items read every view table once**, so an extraction's `total` equals the dump. A record's bytes are its `l0` line plus its `l0_who` lines (`_owned(self.rows("l0_who"), "event_key")`); an open episode's records are overhead; `OVERHEAD_TABLES = ("meta", "names", "forgotten", "retention_passes")`. | `memory/retention/items.py:25`, `:105-121`, `:174-213` |
| D-11 | The host bound's steps, in order: `consolidate`'s defaults, closed records, chaptered episodes, chapters, chapterless episodes, stable facts (fewest meetings first). `_shed` removes items until the accounted size is under the limit and confirms with `size()`. | `memory/retention/bound.py:121-146`, `:149-170`, `:197-202` |
| D-12 | Retrieval on `main` already has **five** person sections: `with` (6 lines, 1 000 B; P6c), `L3`, `L2`, `L1`, `L0` (48 lines, 1 000 B), summing to 6 000 B. `RecallQuery` = `at`, `counterparts`, `words`, `speaker`, `heard`. `cut` shortens a line to 160 B with its citation. | `memory/retrieve.py:50-71`, `:78-88`, `:125-133` |
| D-13 | The consult builds `RecallQuery` from the newest observation; `Situation.of` reads `observation.self_location.place` and lists persons by `entity_type == "person"`; `Observation.entities` are in ascending id order, each with `entity_type`. | `controller/seat.py:503-521`; `controller/situation.py:100-106`, `:323-327`; `sdk/.../contract.py:195-202`, `:332-345` |
| D-14 | `_open_store` opens the seat's home and its view once per seat runner. | `controller/seat.py:254-265` |
| D-15 | The writers scan: `INSERT INTO` / `DELETE FROM` / `UPDATE` of `fact_who`, `fact`, `access_text`, `access` only in `memory/home/` and `memory/scopes/world.py`. `\baccess\b` does not match `access_anchor` (the `_` is a word character). | `tests/test_memory_isolation.py:74-90` |
| D-16 | `world.who(fact)` gives `fact_who` rows with role 1 (participant), 2 (subject only) or 3 (both). | `memory/scopes/world.py:24-50` |
| D-17 | `ARC-100`'s "Revisit" clause: "when a scope needs a write that the home cannot express". | `docs/decisions/ARC-100.md` (last line) |
| D-18 | S28-c's frozen design (read, not code): `TableSpec(name, columns: tuple[str, ...], keyed_by: Literal["event", "seat", "seat-event"])`; `ScopeRow(table, cells)`; `Reading(gist, salience, mine, importance)`; `ScopeView` = `seat`, `me`, `holds`, `record`, `held_between`, `name`; an optional scope gets a `GuardedView` with no connection; the citable set of an optional scope is held keys only; `SectionSpec(key, header, budget)`; the stack is person first; Σ section bytes ≤ 6 000. | `pr-s28c-memory-scopes.md` §§5.2, 5.5, 5.7 |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S28d-1** | The step's payload anchors need "a renderer to declare them" (§5.2.3). The contract already types every entity reference in a payload (D-2), and the audited packs name items only that way (D-3). One generic scan finds them for every pack, including packs this code has never seen. A per-renderer declaration would need code per event type, which is the world-specific coding the charter forbids. | **Bounded:** `render()` fills `Rendered.anchors` from `payload_anchors(fact)` for every renderer; no renderer is edited (§5.2). |
| **F-S28d-2** | The step gives `item` the roles `subject` and `participant`. The envelope carries no kind for them (D-4), and in every audited pack those entities are persons (D-3). An `item` scope reading them would anchor persons as items. | **§14 QS28d-1.** |
| **F-S28d-3** | The step stores payload anchors "on the access row". A column on `access` would add bytes to every `l0` line of every seat, even for facts with no anchor, and change every dump. | **§14 QS28d-2** (a side table, `access_anchor`, with rows only where a payload names an anchor). |
| **F-S28d-4** | The step's shared `anchor (event_key, entity, role)` index duplicates what the home already stores: role `at` is `fact.place`, and the payload anchors are `access_anchor`. An SQLite index over those columns serves recall and costs no dumped byte. `granularity: episode` existed only to make that table smaller. | **§14 QS28d-3.** |
| **F-S28d-5** | `anchor_known` must count across facts (one row per anchor, `visits` growing). S28-c's `ScopeRow` can only be inserted; the host has no rule to merge a second fact's row into the first. `ARC-100` foresaw this (D-17). | **§14 QS28d-3** (typed merge rules on `TableSpec` columns). |
| **F-S28d-6** | The step evicts `anchor_known` "fewest visits first" at the cap. At the cap, every new anchor arrives with `visits = 1` and is evicted at once, so a person could never learn a new place again. | **§14 QS28d-4.** |
| **F-S28d-7** | A "known" row's `first_key` / `last_key` are released by retention long before the row goes. An optional scope may cite held keys only (QS28c-2), so the row's own keys cannot be its citation. | **Bounded:** the known line cites the newest held fact anchored to the entity; with no such fact, the line is not shown. This is exactly C-3's rule "cites only held facts". |
| **F-S28d-8** | AS28-15 says a rebuilt scope "is identical to an uninterrupted one". A known row counts facts that retention later released. A rebuild can read only held records, so after any release the rebuilt counts are smaller. | **§14 QS28d-5.** |
| **F-S28d-9** | C-3 says the counts match the export "up to the forgotten set". A known row is written at `hold` and never decremented, so its counts match a recount over the **whole** export, with no subtraction. That is stronger and exact. | **Bounded:** C-3's oracle recounts the whole export (§6.2). The step's wording is corrected in C1. |
| **F-S28d-10** | The step's section header is `About <place name>:`. S28-c's `SectionSpec.header` is one static string per section. | **Bounded:** static headers (`Places I know:` and `Things I know:`); each line starts with the anchor's name (§5.6.4). |
| **F-S28d-11** | The step records a removed scope "in the pass log as `scope-removed:<id>`". The pass log (`retention_passes`) is keyed by a trigger fact and has a fixed `cause` check; a stack change at seat open has no trigger fact. | **Bounded:** `bind_scopes` returns a `ScopeSync`, and the controller writes one `scope-change` line to the decision log (§5.9). No retention table changes. |
| **F-S28d-12** | The step's recall shows gists "that no other section shows". A scope sees only its view, the query, its budgets and its parameters. | **Bounded:** `ScopeView.shown(event)` (§5.5). The host builds it from the lines already admitted. For a base scope's `SeatView` it is always `False`: base scopes run first, when no section exists yet. |
| **F-S28d-13** | **S28-c's frozen audit predates P6c's `with` section** (D-12: five person sections, `L0` at 1 000 B). Its `PersonSections` lists four. | Recorded for the coordinator, to relay to S28-c's implementer; S28-d uses the person sections as S28-c ships them. Not this PR's to fix. |
| **F-S28d-14** | The writers scan does not see `access_anchor` writes (D-15). | **Bounded:** C3 adds `access_anchor` to the scan's table list; writers stay `memory/home/`. |
| **F-S28d-15** | A payload can name the same entity twice, or many entities. Bytes must stay bounded per fact. | **Bounded:** anchors are deduplicated by entity, in scan order, and capped at `ANCHORS_PER_FACT = 16` (§5.2). |

### 3.3 What does not exist

No payload-anchor scan, no `Rendered.anchors`, no `access_anchor`, no scope tables or their registry, no
`bind_scopes`, no merge rule, no `place` or `item` entry point, no `RecallQuery.anchors`, no backstop step for scope
rows, no C-3 oracle.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Decision | Why |
| --- | --- | --- |
| Knowing which payload values are places or items | **The contract's typed reference** (`TypedEntityRef`, D-2) | The world already discloses each reference's kind. No per-pack table, no guess from names |
| Finding held facts at a place or about an item | **SQLite indices**: a new `fact_by_place` on `fact (place)` and `access_anchor_by_entity` on `access_anchor (seat, entity)` | An index is maintained by SQLite, never dumped, never charged; a second table would duplicate the envelope (F-S28d-4) |
| Merging a second sighting into a known row | **SQLite upsert** (`INSERT … ON CONFLICT … DO UPDATE`) generated from typed column rules | The database does the merge in the same statement; the rules are four words, not code a scope runs |
| The stable-fact shape (first, last, count) | **L3's `stable` table** as the model (`compress/stable.py`) | Same idea for places and things; no new concept |
| A knowledge-graph or memory library (Graphiti, mem0, Letta) | **Declined** (`DEP-58`, unchanged) | Model-extracted entities are not deterministic; ours are exact from the contract |
| A rule engine for merge rules | **Declined** | Four fixed roles cover first / last / count; a general engine would be an abstraction with one user |

No dependency is added; `uv.lock` and `Cargo.lock` do not change.

---

## 5. Design

Written with every §14 question answered as recommended; each place a "no" would change is marked **[QS…]**.

### 5.1 Module layout

```text
memory/anchors.py             Anchor, AnchorKind, ANCHORS_PER_FACT, payload_anchors(fact)        (§5.2)
memory/render/__init__.py     Rendered.anchors; render() fills it                                 (§5.2)
memory/home/schema.py         schema 6: ACCESS_ANCHOR view table, SCOPE_DDL, SCHEMA_VERSION = 6   (§5.3)
memory/home/view.py           hold writes anchors and calls the host; release deletes anchors      (§5.3, §5.4)
memory/home/scope_tables.py   the scope-table host: DDL, registry, upsert, cap, sync, rebuild     (§5.4)
memory/home/home.py           bind_scopes; dump, meter and remove_seat include scope tables       (§5.4)
memory/scopes/interface.py    Reading.anchors; Column, TableSpec.cap / .evict; ScopeView reads    (§5.5)
memory/scopes/reads.py        the three reads, written once over (connection, seat)                (§5.5)
memory/scopes/registry.py     load checks of the new TableSpec fields                             (§5.5)
memory/scopes/host.py         GuardedView implements the reads; `shown` from admitted lines        (§5.5)
memory/scopes/anchored.py     AnchoredParameters, AnchoredScope, PlaceScope, ItemScope            (§5.6)
memory/retrieve.py            RecallQuery.anchors                                                 (§5.7)
controller/situation.py       anchors_of(observation)                                             (§5.7)
memory/retention/items.py     anchor bytes to their record; scope tables as overhead              (§5.8)
memory/retention/bound.py     the step _scope_rows before _stable_facts                           (§5.8)
controller/seat.py, log.py    bind the stack at open; the `scope-change` line                     (§5.9)
```

### 5.2 Payload anchors (`memory/anchors.py`, pure)

```python
AnchorKind = Literal["place", "item", "organization"]
ANCHOR_KINDS: tuple[AnchorKind, ...] = ("place", "item", "organization")
ANCHORS_PER_FACT = 16
"""The most anchors kept per fact: a payload naming more is cut, in scan order (F-S28d-15)."""

@dataclass(frozen=True)
class Anchor:
    entity: EntityId
    kind: AnchorKind

def payload_anchors(fact: PerceivedEvent) -> tuple[Anchor, ...]:
    """Every typed reference in the fact's payload whose kind is a place, an item or an organization."""
```

The scan, exactly:

1. Walk `fact.payload.payload` depth first. In a JSON object, visit the keys in sorted order. In a list, visit the
   elements in order.
2. A JSON object is a **typed reference** when it has exactly the keys `entity` and `entity_type`, `entity` is a
   string of ASCII decimal digits with no leading zero, and `entity_type` is a string. A typed reference is not
   walked further.
3. Keep the reference when `entity_type` is in `ANCHOR_KINDS`. A `person` reference is not an anchor (persons
   are L3's). Any other `entity_type` string is ignored.
4. Keep the first occurrence of each entity. Stop after `ANCHORS_PER_FACT`.
5. A `null`, a number, a string or a payload that is not JSON gives `()`. The scan never raises.

Example: `walk-started {"person": {"entity": "11", "entity_type": "person"}, "destination": {"place": {"place":
{"entity": "40", "entity_type": "place"}}}}` gives `(Anchor("40", "place"),)`. Lakeside's `fished {"catch":
"trout", "rod": {"entity": "123", "entity_type": "item"}}` gives `(Anchor("123", "item"),)`.

`Rendered` gains `anchors: tuple[Anchor, ...] = ()`. `render(fact, me, names)` becomes
`dataclasses.replace(RENDERERS.get(…)(fact, me, names), anchors=payload_anchors(fact))`. No renderer changes, so
every gist is unchanged. The anchors are a function of the fact alone, so they are the same for every seat.

### 5.3 Schema 6 (`memory/home/schema.py`)

```sql
CREATE TABLE access_anchor (seat INTEGER NOT NULL, event_key TEXT NOT NULL, entity TEXT NOT NULL,
                            kind TEXT NOT NULL CHECK (kind IN ('place', 'item', 'organization')),
                            PRIMARY KEY (seat, event_key, entity))
CREATE INDEX access_anchor_by_entity ON access_anchor (seat, entity)
CREATE INDEX fact_by_place ON fact (place)
CREATE TABLE scope_tables (name TEXT PRIMARY KEY, scope TEXT NOT NULL, ddl TEXT NOT NULL)
CREATE TABLE scope_seats (seat INTEGER NOT NULL, scope TEXT NOT NULL, parameters TEXT NOT NULL,
                          PRIMARY KEY (seat, scope))
SCHEMA_VERSION = 6
```

- **`access_anchor` is a private view table**: `ViewTable("access_anchor", ("event_key", "entity", "kind"),
  "event_key, entity")`, placed **first** in `PRIVATE` (so it follows `l0_who` in the dump: a record's anchors right
  after its participants). By D-8 this alone puts it in both dumps, the meter and `remove_seat`. Its line is
  `["access_anchor",<id>,"<entity>","<kind>"]`.
- **`hold`** inserts one `access_anchor` row per `rendered.anchors` entry, after the access row, in the same
  transaction. **`release`** deletes the seat's `access_anchor` rows of the released keys before the access rows.
- **Not dumped:** `scope_tables` and `scope_seats` are the host's bookkeeping, like `seats.seat` (§5.4). They depend
  only on each seat's configuration, so I-S28-5 holds.
- `MemoryHome.open` refuses schemas 4 and 5 with `UnsupportedStoreSchema(found, wanted=6)`; 2 and 3 keep their
  refusal with `wanted=6`. The rebuild instruction is S28-a's.
- Bytes: one line of ≈ 45 B per anchor per holder. In `social-cafe` only `walk-started` names a place in its payload
  (D-3), so the seat view grows by well under 1 %. Every byte is charged to the person (D-S28-4), because it is
  part of what they remember.

### 5.4 The scope-table host (`memory/home/scope_tables.py`)

**What a scope declares** (§5.5): per-seat tables only (`keyed_by = "seat"`). Each column has a merge role. The
host turns a `TableSpec` into DDL:

```sql
CREATE TABLE <name> (seat INTEGER NOT NULL, <col> <type> [NOT NULL], …, PRIMARY KEY (seat, <key columns>))
```

`<type>` is `TEXT` for a `text` or `key` column and `INTEGER` for an `int` column.

**The registry of installed scope tables.** On first use of a scope in a home, the host creates each table and
writes `(name, scope, ddl)` into `scope_tables`. On a later use it compares the stored `ddl` with the one it would
write. A difference is `ScopeTableMismatch(scope, table)` (the scope's package changed its tables): the operator
removes the scope from every stack, which drops its rows, then reinstalls. A table name already registered to
another scope is `ScopeTableMismatch` too.

**`MemoryHome.bind_scopes(view, stack) -> ScopeSync`**, called once per seat after `seat(identity)`, in one
transaction:

1. `selected` = the stack's optional scopes (every bound scope after the base scopes), each with the canonical JSON
   of its decoded parameters (sorted keys, no spaces).
2. `stored` = this seat's `scope_seats` rows.
3. **Removed:** a stored scope not selected, or selected with different parameters → delete this seat's rows from
   each of its registered tables, then its `scope_seats` row.
4. **Added:** a selected scope not stored (including one just removed in step 3) → create its tables (above), then
   **rebuild**: for every access row of this seat in key order, build the `FactRecord` and the `Reading` (anchors
   read from `access_anchor`), call `index`, and write the rows exactly as `hold` would (below). Then insert the
   `scope_seats` row with the parameters.
5. Install the meter's `TEMP` triggers for every table of the selected scopes (the same body as D-8's), and drop
   the seat's meter so the next `content_size()` measures in full.
6. Return `ScopeSync(seat, added: tuple[str, ...], removed: tuple[str, ...], rebuilt_rows: int)`.

With an empty `selected` and an empty `stored` the call writes nothing. A home that no seat binds behaves exactly
as before.

**At `hold`** (after the access and anchor rows), for each bound optional scope of the view, in stack order:

1. `rows = scope.index(record, reading, params)`.
2. Check each row before writing; a failure is `ScopeRowRefused(scope, table, why)` and rolls the ingest
   transaction back (fail closed, like `ConflictingFact`):
   - the table is one of the scope's own `TableSpec`s;
   - the cells match the columns in number and in type (`text` / `key` → `str`, `int` → `int`, never `bool`);
   - every `key` column's value is the event key of the fact being held (a row can only cite what the seat holds
     now; AS28-14's runtime half).
3. Write each row as one upsert: `INSERT … ON CONFLICT (seat, <key columns>) DO UPDATE SET <col> = <rule>`, with the
   rule per merge role: `key` and `first` keep the stored value; `last` takes the new value; `count` adds the new
   value to the stored one.
4. **The cap** [QS28d-4]: when the scope declares `cap`, and this seat now has more rows in the table than the
   parameter named by `cap`, delete the surplus rows first in `evict` order (ascending). One `DELETE … WHERE seat
   = ? AND rowid IN (SELECT rowid … ORDER BY <evict> LIMIT ?)`.

**Seat cascade.** `remove_seat` also deletes the seat's rows of every registered scope table and its `scope_seats`
rows. **No event cascade:** scope rows are per-seat summaries; retention never deletes them when it releases a
record (F-S28d-7). Only the cap, the backstop (§5.8), removal and `remove_seat` delete them.

**Dumps and meter.** The seat view's dump ends with the scope tables this seat has rows in, in table-name order,
each row `["<table>", <cells…>]` in key order, with `key` columns encoded as numbers (as `KEY_COLUMNS` are). The
home dump writes the same lines in each seat's section, after its private tables. `row_bytes(table)` and
`measured()` cover them. So a scope row costs the same bytes in the home as in its seat view, and I-S28-6
(physical ≤ logical) holds row for row.

### 5.5 The interface's carriers (`memory/scopes/interface.py`) [QS28d-3]

S28-c's `MemoryScope` members do not change. Its carriers change as follows.

```python
class Reading(CognitionModel):               # S28-c's, one field added
    gist: str; salience: Salience; mine: bool; importance: Importance
    anchors: tuple[Anchor, ...] = ()         # the fact's payload anchors (§5.2)

MergeRole = Literal["key", "first", "last", "count"]
ColumnType = Literal["text", "key", "int"]

@dataclass(frozen=True)
class Column:
    name: str                                # ^[a-z][a-z0-9_]{0,47}$, not "seat"
    type: ColumnType                         # "key": an EventKey, which must be the held fact's (§5.4)
    merge: MergeRole

@dataclass(frozen=True)
class TableSpec:                             # S28-c's, columns typed; two fields added
    name: str                                # "<scope id, each - written _>_<name>": a bare SQL name
    columns: tuple[Column, ...]
    keyed_by: Literal["seat"]                # event-keyed tables are not hosted yet (F-S28d-4)
    cap: str | None = None                   # the name of an int field of the scope's Parameters
    evict: tuple[str, ...] = ()              # columns, ascending: the cap's and the backstop's order
```

New load checks in `ScopeRegistry` (each a `ScopeLoadError(name, distribution, why)`): at least one `key` column;
every `key`-typed column has merge role `first` or `last` or `key`; `count` only on `int` columns; column names
unique and never `seat`; `cap`, when set, names an `int` field of `Parameters` with `ge=1`; every `evict` column
exists, and `evict` is non-empty when `cap` is set; `keyed_by` is `"seat"`.

`ScopeView` gains three reads. Each is written once in `memory/scopes/reads.py` as a function of `(connection,
seat, …)`, and both `GuardedView` and `SeatView` call it:

```python
def anchored(self, entity: EntityId, *, at: bool, named: frozenset[AnchorKind], until: int,
             limit: int, notable: bool) -> tuple[EventId, ...]: ...
    # held facts of this seat anchored to `entity`, `f.at <= until`, newest key first, at most `limit`:
    # (at and f.place = entity) or (an access_anchor row of this seat, this key, this entity, kind in named);
    # notable=True keeps only `salience = 'notable'`
def own_rows(self, table: str, where: tuple[tuple[str, str | int], ...]) -> tuple[ScopeRow, ...]: ...
    # this seat's rows of one of the calling scope's own tables whose named key columns equal the given
    # values (each column load-checked to be a `key`-role column of that table), in key order;
    # another scope's table, or a column that is not a key column → AccessDenied(scope, seat, table)
def shown(self, event: EventId) -> bool: ...
    # whether a line admitted earlier in this recall cites a range that contains `event`
```

For `SeatView`, `own_rows` raises `AccessDenied` (base scopes have no scope tables) and `shown` returns `False` (a
base scope runs before any section exists, F-S28d-12). The host builds each `GuardedView` after the previous scopes'
lines are admitted, so `shown` sees them.

### 5.6 The anchored scope (`memory/scopes/anchored.py`)

#### 5.6.1 Parameters

```python
AnchorRole = Literal["at", "named"]

class AnchoredParameters(ScopeParameters):
    roles: tuple[AnchorRole, ...]            # non-empty, unique
    kinds: tuple[AnchorKind, ...]            # non-empty, unique
    anchors_max: Annotated[int, Field(ge=1, le=10_000)] = 256
    section_lines: Annotated[int, Field(ge=0, le=16)] = 4
    section_bytes: Annotated[int, Field(ge=0, le=1_600)] = 0
    # validator: "at" in roles requires "place" in kinds (`at` anchors only places)
```

| Scope | `roles` | `kinds` | Why |
| --- | --- | --- | --- |
| `place` | `("at",)` | `("place",)` | A place is where things happen: the envelope's own `place` [QS28d-1] |
| `item` | `("named",)` | `("item",)` | An item is what a payload names as an item (D-3) [QS28d-1] |

A world may widen either, for example `place` with `roles: [at, named]` to count a walk's destination. Every field
carries a `description` (the reference generator refuses one without).

#### 5.6.2 Classes and entry points

`AnchoredScope` holds the shared code. `PlaceScope(AnchoredScope)` sets `id = "place"` and `Parameters =
PlaceParameters` (the `place` defaults); `ItemScope` sets `id = "item"` and `Parameters = ItemParameters`. Each
declares one table:

```python
TableSpec(name=f"{id}_known", keyed_by="seat", cap="anchors_max", evict=("last_at", "entity"),
          columns=(Column("entity", "text", "key"), Column("first_key", "key", "first"),
                   Column("last_key", "key", "last"), Column("first_at", "int", "first"),
                   Column("last_at", "int", "last"), Column("visits", "int", "count")))
```

`sections(params)` returns one `SectionSpec(key=id, header=HEADER, budget=SectionBudget(lines=section_lines,
bytes=section_bytes))`, with `HEADER` = `Places I know:` for `place` and `Things I know:` for `item`.

```toml
[project.entry-points."mineworld.memory_scopes"]
place = "mineworld_cognition.memory.scopes.anchored:PlaceScope"
item = "mineworld_cognition.memory.scopes.anchored:ItemScope"
```

#### 5.6.3 `index` (pure)

The anchors of a fact for this scope, in this order, deduplicated by entity, never the seat's own entity:

1. `fact.place`, when `"at"` is in `roles` (its kind is `place`);
2. each `reading.anchors` entry whose kind is in `kinds`, when `"named"` is in `roles`.

One row per anchor: `(entity, key, key, fact.at, fact.at, 1)`, where `key = event_key(fact.event)`. The upsert then
keeps the first key and time, takes the newest, and adds one visit. **`visits` counts the perceived facts anchored
to the entity**, from the first the seat held (F-S28d-9).

#### 5.6.4 `recall`

Given the budget `b = budgets[id]`; if `b.lines == 0` or `b.bytes == 0`, return `()` (the default).

1. `targets` = `query.anchors` whose kind is in `kinds`, deduplicated by entity, in query order.
2. **Known lines**, one per target, in target order. Read the row with `own_rows(f"{id}_known", (("entity", entity),))`; read the
   citation with `anchored(entity, …, until=query.at, limit=1, notable=False)`. Skip the target when either is
   empty. The text is `f"{name}: {visits} moments I remember, Day {day(first_at)} to Day {day(last_at)}"`.
3. **Gist lines.** For each target, `anchored(entity, …, until=query.at, limit=b.lines, notable=True)`. Merge
   the lists, newest key first, dropping a fact already cited by this section or `shown`. The text is
   `f"{name}, Day {day(at)} {clock(at)}: {gist}"`, from `record(event)`.
4. Offer every line with P4's rules (`cut` to 160 B; stop at the first line over the line count or the bytes,
   header and newlines counted). Each line's citation is `Citation.of(event)`.

`name` is `view.name(entity)`, else the generic label for the kind: `somewhere` (place), `something` (item),
`some group` (organization). A line never shows an id (I-10). `day` and `clock` are `compress.summarize`'s.

The host then applies S28-c's four checks: every citation is a held key (true by construction), and every budget
holds. A hostile or buggy scope is withheld as QS28c-3 rules.

`account(view)` returns `((f"{id}_known", bytes),)` from `row_bytes`.

### 5.7 Query anchors

- `RecallQuery` gains `anchors: tuple[Anchor, ...] = ()`, "the places, items and organizations the person is with
  now". The person scope never reads it, so I-d-1 holds.
- `controller/situation.py` gains `anchors_of(observation) -> tuple[Anchor, ...]`: the observer's place first
  (`self_location.place`, kind `place`), then each `observation.entities` entry whose `entity_type` is `place`,
  `item` or `organization`, in the observation's (ascending id) order, without repeats, at most `QUERY_ANCHORS =
  16`. All of it is what the world disclosed to this person now.
- `_consult_once` passes `anchors=anchors_of(observation)`.

### 5.8 Retention

- **Items** (`items.py`): `records()` adds a record's `access_anchor` line bytes to its item, exactly as it adds
  its `l0_who` lines (`_owned(self.rows("access_anchor"), "event_key")`). An open episode's anchors are overhead,
  with its record. Every scope table's rows are **overhead**: no policy sees a scope row (FGb-4: policies decide
  about memory items, and a scope row is not one). `extract` reads each scope table once, so `total` still equals
  the dump.
- **The backstop step** (`bound.py`, R-S28c-b-1): `_scope_rows`, between `_chapterless_episodes` and
  `_stable_facts`. For each scope table this seat has rows in, in table-name order, delete rows in that table's
  `evict` order, one at a time, re-measuring with `content_size()`, until the seat is under the limit or the table
  is empty. Each deleted row counts as forced. It adds **no** forgotten range: a scope row cites, it does not cover.
  The step reads the tables from `scope_tables`; it knows no scope by name.

### 5.9 The controller

- `_open_store` calls `self._home.bind_scopes(self._store, self._stack)` once, after `seat(identity)`.
- When the returned `ScopeSync` adds or removes anything, `DecisionLog` writes one line before the first consult:
  `{"kind":"scope-change","seat":…,"added":[…],"removed":[…],"rebuilt_rows":n}`. Each removal is the step's
  `scope-removed:<id>` (F-S28d-11).
- `_consult_once` passes the query anchors (§5.7).

### 5.10 Named errors

In `memory.home` (message begins `memory.home:`): `ScopeTableMismatch(scope, table)`, `ScopeRowRefused(scope,
table, why)`, and `UnsupportedStoreSchema` with `wanted=6`. In `memory.scopes`: the new `ScopeLoadError` reasons
(§5.5) and `AccessDenied(scope, seat, table)` for a foreign table in `own_rows`. None echoes a cell value or a
parameter value. Each subclasses `ValueError`.

---

## 6. C-3 and its oracle

### 6.1 The worlds the tests run on

| Arm | What it is | What it proves |
| --- | --- | --- |
| `cafe-like` | `home_streams.world(seed=7, days=7)` (S28-b's synthetic multi-seat stream), plus `walk-started` facts with nested place references | Places only, the default world's shape |
| `lakeside-like` | a synthetic stream in `tests/anchor_streams.py` shaped like `lakeside`: arrivals at a pier, a bakery and a boathouse; `fished` with a typed rod; `items-transferred` with a typed item | Items from payloads, the **second world** |
| `generated` | `anchor_streams.generated(seed)`: seeded random place, item and organization ids (8–19 digits), random names, references nested 0–4 levels inside random keys, person references mixed in, duplicates, payloads with more than 16 anchors | Nothing depends on an id, a name, an event type or a payload shape |
| `lakeside-real` (milestone) | `mineworld run worlds/lakeside --headless --seed 7 --days 3 --save S`, then `mineworld perceived worlds/lakeside --save S --person <key> --json` for every person | The real pack's payloads, end to end |

### 6.2 The oracle (`tests/anchor_oracle.py`, test-only)

It imports **no** module of `mineworld_cognition`. It reads the person's perceived export (JSON lines in the wire
form) and re-implements, by itself, the scan of §5.2 (its own code, from §5.2's text).

- `history(export, entity, roles, kinds)` → the facts anchored to `entity`, in id order.
- `known(export, entity, roles, kinds)` → `(visits, first_at, last_at)` over the **whole** export (F-S28d-9).
- `objective(exports, place)` → the union of every person's facts whose envelope `place` is `place`: the log's
  facts at that place as perception delivered them (`PROTOCOL.md` §5.8). It is the test oracle C-3 names, never an
  input to memory.

### 6.3 What C-3 checks, on every arm

For each seat, at the end and at every retention pass point, for every query of a seeded set of 50 per seat whose
`anchors` are drawn from the anchors in that seat's export:

1. every line of a `place` or `item` section cites one fact; that fact is held by the seat, it is in the seat's
   export, and `history(export, target, roles, kinds)` contains it for the line's target;
2. every known line's `visits`, `first_at` and `last_at` equal `known(…)` for its target (the arm sets
   `anchors_max` above the number of anchors, so no row was evicted);
3. every cited fact of a `place` line is in `objective(all exports, target)`;
4. no line cites a fact also cited by an earlier section of the same render.

---

## 7. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-d-1 | **Default identity of recall.** With the default stack, every rendered section equals S28-c's golden renders, for every query | AC-d-1 |
| I-d-2 (I-S28-1) | No anchored line cites a key the seat does not hold; a scope row can only be written with the key of the fact being held | AC-d-5, AC-d-6 |
| I-d-3 (I-S28-2) | Sharing stays invisible with `place` and `item` in the stack: seat-view dumps and sections are byte-identical under `none` and `home` | AC-d-8 |
| I-d-4 (I-S28-6) | Every scope row and anchor row costs the same bytes in the home dump as in its seat view | AC-d-3 |
| I-d-5 (I-S28-7, QS28-6) | Anchored memory reads only the seat's held records and its own rows; the oracle's log-derived history is never an input | AC-d-11 (scan), §6.2 |
| I-d-6 (I-S28-8) | Only the home writes scope tables; a scope returns rows | AC-d-11 |
| I-d-7 (I-S28-9) | A stack without `place` / `item` opens and plays; removing them drops their rows; adding them rebuilds from held records | AC-d-9 |
| I-d-8 | Anchors come from the contract's typed references only: no event type, entity id, name or world is named in `memory/` or `controller/` code | AC-d-11 (scan), the `generated` arm |
| I-d-9 (I-S28-10) | No path under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` changes | AC-d-12 |

---

## 8. Test ownership and acceptance

### 8.1 Ownership (test rules §26)

| Layer | Owns | Where |
| --- | --- | --- |
| Static | ruff, ruff format, pyright strict (`PlaceScope` and `ItemScope` conform to `MemoryScope`; `GuardedView` and `SeatView` to `ScopeView`) | CI `fast` |
| Unit | the scan (`test_memory_anchors.py`); schema 6 rows, meter, release (`test_home_anchors.py`); the host's DDL, upsert, cap, sync, rebuild, refusals (`test_scope_tables.py`); the scope's `index`, `recall`, budgets against a fake view (`test_anchored_scope.py`); the backstop step (`test_retention_scope_rows.py`); `anchors_of` (`test_controller_anchors.py`) | `cognition/lm-controller/tests/` |
| Scenario (unit-speed) | C-3 on the three synthetic arms; the mini transparency set; degradation (`test_anchored_c3.py`) | same |
| Golden | I-d-1: S28-c's golden test unchanged | same |
| Milestone (`slow` / `real_binary`) | C-3 on `lakeside-real`; S28-a's full transparency harness with `[person, place, item]` stacks; S28-b's shared-home Gate 2 | same; run at the milestone |
| Gate 1 (real LLM) | **NOT REQUIRED**: the default stack's request bytes are unchanged (I-d-1), and the new section is off by default | — |

Oracles: the golden renders, the independent anchor oracle (§6.2), and dumps compared across arms. No expected
value comes from the code under test.

### 8.2 Acceptance criteria (fixed before measuring, `ARC-23`)

Every criterion names a second world or the generated variant (charter).

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-d-1 | **Default identity.** S28-c's golden test passes unchanged; on the `cafe-like` and `lakeside-like` arms, the rendered sections with `[person]` equal those of the S28-c base code (captured at C0 into a scratch file, never committed as a golden) | I-d-1 |
| AC-d-2 | **The scan.** On fixed payloads (the walk's nested place, the rod, `items-transferred`) and on 500 `generated` payloads, `payload_anchors` equals the oracle's scan; persons are never anchors; 17 distinct anchors give the first 16; a non-JSON or `null` payload gives `()` | §5.2 |
| AC-d-3 | **Schema 6.** On the `lakeside-like` arm: each anchor is one `access_anchor` line in the seat view and in the home dump with equal bytes; `release` removes them; `content_size()` equals the dump less `meta` after every ingest and pass; the item bytes `extract` reports sum to the dump; a schema-5 home is refused with `UnsupportedStoreSchema(found=5, wanted=6)` | §5.3, §5.8 |
| AC-d-4 | **The host.** With a fixture scope on the `generated` arm: tables are created once and registered; a changed `TableSpec` gives `ScopeTableMismatch`; `first`, `last` and `count` merge as §5.4 states (oracle: a fold over the facts in the test); the cap evicts in `evict` order; `remove_seat` leaves no row of the seat | §5.4 |
| AC-d-5 | **Refusals (AS28-14 runtime half).** A fixture scope whose `index` returns a row for another scope's table, the wrong number of cells, a `bool` for an `int`, or a `key` cell other than the held fact's key gets `ScopeRowRefused` naming the scope and table, and the ingest rolls back (the dump equals the dump before the call) | AS28-14 |
| AC-d-6 | **Load checks.** Fixture scopes with no `key` column, `count` on a text column, a column named `seat`, a `cap` naming a missing or non-int field, an `evict` column that does not exist, or `keyed_by="event"` are each refused by `ScopeLoadError` | §5.5 |
| AC-d-7 | **C-3.** §6.3's four checks pass on `cafe-like`, `lakeside-like` and `generated` (per PR, unit speed), and on `lakeside-real` (milestone) | C-3, AS28-18 |
| AC-d-8 | **Transparency (I-S28-2).** On the `lakeside-like` arm with 3 seats and stacks `[person, place, item]` (sections on), every seat-view dump and the sections of 50 seeded queries per seat are byte-identical under `none` and `home`, at every pass point; the full 12-seat harness runs at the milestone | AS28-3, AS28-4 |
| AC-d-9 | **Degradation (AS28-15).** On the `generated` arm: removing `place` from a stack drops its rows and logs `scope-change` with `removed: [place]`; the world's save is untouched; re-adding it rebuilds rows equal to an uninterrupted run's under `keep-all`, and equal to the oracle's recount over held records under `consolidate` [QS28d-5]; a stack naming an uninstalled scope still fails with `UnknownScope` (S28-c) | AS28-15 |
| AC-d-10 | **The backstop.** On the `generated` arm with a bound the scope rows push over, `_scope_rows` runs after chapterless episodes and before stable facts, drops rows in `evict` order, adds no forgotten range, and counts each row as forced | §5.8 |
| AC-d-11 | **Scans.** `memory/scopes/anchored.py` and `memory/anchors.py` pass the isolation scan and name no event type, entity id or world; the writers scan, extended to `access_anchor` and every `*_known` table, finds writes only in `memory/home/` | AS28-13, AS28-14 |
| AC-d-12 | **Diff scan.** `git diff --name-only origin/main...HEAD` lists nothing under `kernel/`, `contracts/`, `server/`, `persistence/`, `systems/`, `worlds/`, `sdk/` | I-S28-10 |
| AC-d-13 | **Reference.** `memory-scopes.md` lists `place` and `item` with every parameter described; the drift test passes | §2.1 item 10 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s28d`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full,
   before editing it.
2. Make the numbered steps, in order. Do not do work that belongs to a later commit.
3. Run the commit's validation commands. Record for each: the command, the test count, the wall time, and PASS /
   FAIL / INCONCLUSIVE (a test not run is not a pass).
4. For each mutation the commit lists: plant it, run the named test, record that it turned red (with the failing
   test's name), restore it from a byte copy (`cmp` clean), run the test again green.
5. Static: `uv run --locked ruff check cognition/lm-controller`; `uv run --locked ruff format --check
   cognition/lm-controller`; `uv run --locked pyright`. Tests one file at a time: `uv run --locked pytest
   cognition/lm-controller/tests/<file>`. **R-LOAD-1:** never the whole member suite. Markdown: `python3
   scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
6. Review: answer the commit's review question in one or two sentences, in §15.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files**; otherwise
   revert it, or stop and record why.
8. Update §15 (ledger row, evidence, findings) in the same commit.
9. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` with the message given plus the
   session's attribution line. No heredoc, no `sed -i`, no `python3 -c`.
10. A material stop (§12) is reported, not improvised around.

#### C0 — Base, re-audit and frozen evidence (Markdown only)

- **Goal:** fix the base and capture what later commits compare against, before any code changes.
- **Files:** this file (§15).
- Steps:
  1. Record the base. Confirm S28-c merged: `memory/scopes/interface.py` exists and `ScopeRegistry.installed().ids()`
     is `("person", "world")`. Not merged → material stop.
  2. Re-anchor D-1 … D-18 on the base; record every S28-c name of §2.3 as it was merged (file and line). A name
     missing or with a different shape is a stop **only** if §5 cannot be written on it; otherwise record the
     mapping.
  3. Record whether S28-e merged (decides C8 item 4).
  4. List every test that pins the schema number or a whole-dump digest (`grep -rn "SCHEMA_VERSION\|\"schema\""
     cognition/lm-controller/tests`, and the digests S28-a and S28-b pinned). C3 updates exactly these, with the
     reason "schema 6". A pin of a **render** or a **request** is never changed (material stop).
  5. Capture AC-d-1's base renders for the `cafe-like` and `lakeside-like` arms into
     `/Users/yuema137/mineworld-worktrees/scratch-s28d/base-renders.json` (scratch, never committed), with a small
     script kept in the same scratch directory.
- Validation: the two doc scripts.
- [ ] Implementation. [ ] Validation. [ ] Review: the base satisfies §2.3; the pins are listed before C3.
- **Commit message:** `docs(plan): S28-d — open the ledger at the base`

#### C1 — The specifications (Markdown only)

- **Goal:** the specification says what this PR builds before it is built (`CLAUDE.md` §2.2).
- **Files:** `docs/MODULE_SPEC.md` (§5.3), `docs/decisions/ARC-100.md`, `docs/decisions/ARC-109.md`.
- Steps:
  1. §5.3's `place` / `item` row and parameter rows: roles `at` / `named` and `kinds` (QS28d-1); payload anchors
     and `access_anchor` (QS28d-2); no shared index, no `granularity` (QS28d-3); the merge roles, `cap`, `evict`
     (QS28d-3, QS28d-4); the known line's citation (F-S28d-7); `visits` as counted (F-S28d-9); the static headers
     (F-S28d-10); `scope-change` (F-S28d-11); rebuild exactness (QS28d-5).
  2. `ARC-100` dated note (2026-10-xx): the same rulings in five sentences. `ARC-109` dated note: schema 6
     (`access_anchor`, `scope_tables`, `scope_seats`).
- Validation: the doc scripts.
- [ ] Implementation. [ ] Validation. [ ] Review: a scope author can write an anchored scope from §5.3 alone.
- **Commit message:** `docs(spec): the anchored scope as built — payload anchors, scope tables, merge rules (S28-d)`

#### C2 — Payload anchors

- **Goal:** every rendered fact carries its anchors, found by one generic scan.
- **Files:** `memory/anchors.py` (new), `memory/render/__init__.py`, `tests/test_memory_anchors.py` (new),
  `tests/anchor_oracle.py` (new; the scan half only), `tests/anchor_streams.py` (new; `generated` payloads only).
- Steps: §5.2 exactly. `anchor_oracle.py` writes its own scan from §5.2's numbered rules; it imports nothing from
  `mineworld_cognition`.
- Validation: `test_memory_anchors.py` (AC-d-2); `test_memory_render.py` unchanged and green (no gist changed).
  Mutations **M-d-1** (dict keys visited unsorted) → the order test red; **M-d-2** (person references kept) → the
  persons test red; **M-d-3** (the cap at 17) → the cap test red.
- [ ] Implementation. [ ] Validation. [ ] Review: no event type is named; the scan never raises.
- **Commit message:** `feat(cognition): payload anchors from the contract's typed references (S28-d)`

#### C3 — Schema 6: `access_anchor`

- **Goal:** a seat remembers each fact's anchors, charged to it, released with the fact.
- **Files:** `memory/home/schema.py`, `memory/home/view.py`, `memory/home/home.py` (the refusal message only),
  `memory/retention/items.py` (anchor bytes), `tests/test_home_anchors.py` (new), `tests/test_memory_isolation.py`
  (`access_anchor` in `WRITES`), and the pin files C0 listed.
- Steps:
  1. §5.3's DDL for `access_anchor`, its index and `fact_by_place`; `SCHEMA_VERSION = 6`; the `ViewTable` first
     in `PRIVATE`.
  2. `hold` inserts the anchor rows; `release` deletes them first.
  3. `items.py` `records()`: add `_owned(self.rows("access_anchor"), "event_key")` sizes to each record, as
     `l0_who`.
  4. Update the pins C0 listed, each with the reason in §15.
- Validation: `test_home_anchors.py` (AC-d-3); `test_home_store.py`, `test_home_encoding.py`, `test_home_gc.py`,
  `test_retention_size.py`, `test_memory_isolation.py`, `test_home_golden.py`, one at a time. Mutations **M-d-4**
  (`release` leaves anchor rows) → the release test red; **M-d-5** (`records()` ignores anchor bytes) → the
  `total == dump` test red.
- [ ] Implementation. [ ] Validation. [ ] Review: no gist or render changed; only listed pins moved.
- **Commit message:** `feat(memory): schema 6 — each held fact's anchors beside its access row (S28-d)`

#### C4 — The scope-table host: tables, upsert, cap, dump, meter

- **Goal:** an optional scope's rows are written, merged, capped, dumped and metered by the home.
- **Files:** `memory/scopes/interface.py` (`Reading.anchors`, `Column`, `TableSpec`), `memory/scopes/registry.py`
  (load checks), `memory/home/scope_tables.py` (new), `memory/home/schema.py` (`scope_tables`, `scope_seats` DDL),
  `memory/home/view.py` (`hold` calls the host; dump, `row_bytes`, `measured` include scope tables),
  `memory/home/home.py` (home dump, triggers, `remove_seat`), `tests/scope_fixtures.py` (a tally fixture scope and
  hostile ones), `tests/test_scope_tables.py` (new), `tests/test_scope_registry.py`.
- Steps: §5.4 (all but `bind_scopes`' rebuild, which is C5), §5.5's carriers and load checks, §5.10's errors.
  `hold` reads the view's bound optional scopes from `view.scopes` (empty until C5 binds them; tests set it through
  `bind_scopes` without rebuild rows).
- Validation: `test_scope_tables.py` (AC-d-4, AC-d-5), `test_scope_registry.py` (AC-d-6), `test_home_store.py`.
  Mutations **M-d-6** (`count` replaces instead of adding) → the merge test red; **M-d-7** (the cap evicts
  descending) → the cap test red; **M-d-8** (the `key`-cell check skipped) → AC-d-5 red; **M-d-9** (scope tables
  left out of `measured()`) → the meter test red.
- [ ] Implementation. [ ] Validation. [ ] Review: no SQL in `scope_tables.py` lacks `seat = ?`; the DDL is built
  only from load-checked names.
- **Commit message:** `feat(memory): the scope-table host — declared tables, merge rules, the cap (S28-d)`

#### C5 — Binding, removal and rebuild

- **Goal:** a seat's stack decides which scope rows it has, at every open.
- **Files:** `memory/home/scope_tables.py`, `memory/home/home.py` (`bind_scopes`, `ScopeSync`),
  `tests/test_scope_tables.py`.
- Steps: §5.4's `bind_scopes` steps 1–6.
- Validation: `test_scope_tables.py` (binding, parameter change, rebuild equal to an uninterrupted fold under
  keep-all). Mutation **M-d-10** (rebuild in descending key order) → the rebuild test red (`first_*` differ).
- [ ] Implementation. [ ] Validation. [ ] Review: removal and rebuild run in one transaction; a home no seat binds
  is unchanged.
- **Commit message:** `feat(memory): seats bind their scopes; removal drops rows, re-adding rebuilds them (S28-d)`

#### C6 — The anchored scope and its reads

- **Goal:** `place` and `item` exist and recall.
- **Files:** `memory/scopes/anchored.py` (new), `memory/scopes/reads.py` (new), `memory/scopes/interface.py`
  (`ScopeView` reads), `memory/scopes/host.py` (`GuardedView` reads, `shown`), `memory/home/view.py` (`SeatView`
  reads), `memory/retrieve.py` (`RecallQuery.anchors`), `cognition/lm-controller/pyproject.toml` (two entry
  points), `tests/test_anchored_scope.py` (new), `tests/test_scope_registry.py` (installed ids).
- Steps: §5.5's reads, §5.6, §5.7's `RecallQuery` field.
- Validation: `test_anchored_scope.py` (index property: same fact and reading give the same rows, every row's keys
  are the fact's; recall against a fake view that holds a fixed key set and raises on any other; every line ≤ 160 B
  and the section within its bytes; names never ids); `test_scope_registry.py` (`ids() == ("item", "person",
  "place", "world")` in the registry's order); `test_scope_host.py` (S28-c's, unchanged, green). Mutations
  **M-d-11** (`anchored` without `seat = ?`) → the fake-view test red; **M-d-12** (`shown` ignored) → the
  no-repeat test red.
- [ ] Implementation. [ ] Validation. [ ] Review: no statement reads `fact` without the seat's access join.
- **Commit message:** `feat(cognition): the place and item scopes — what I know of places and things (S28-d)`

#### C7 — Retention and the controller

- **Goal:** scope rows obey the bound; a seat plays with its anchors.
- **Files:** `memory/retention/items.py` (scope tables as overhead), `memory/retention/bound.py` (`_scope_rows`),
  `controller/situation.py` (`anchors_of`), `controller/seat.py`, `controller/log.py`,
  `tests/test_retention_scope_rows.py` (new), `tests/test_controller_anchors.py` (new).
- Steps: §5.8, §5.9, §5.7's `anchors_of`.
- Validation: `test_retention_scope_rows.py` (AC-d-10); `test_controller_anchors.py` (`anchors_of` on a fixture
  observation; a `SeatRunner` with the scripted backend logs one `scope-change` line when the stack adds `place`);
  `test_controller_seat.py` and `test_milestone_d_replay.py`, one at a time. Mutation **M-d-13** (`_scope_rows`
  after `_stable_facts`) → the order test red.
- [ ] Implementation. [ ] Validation. [ ] Review: the pass order is otherwise unchanged; no forgotten range is
  added by the new step.
- **Commit message:** `feat(cognition): scope rows under the bound; consults recall the place and things here (S28-d)`

#### C8 — C-3, transparency, degradation and the reference

- **Goal:** the step's criteria on real code, on two worlds and a generated one.
- **Files:** `tests/anchor_oracle.py` (complete), `tests/anchor_streams.py` (complete), `tests/test_anchored_c3.py`
  (new), `docs/reference/{memory-scopes.md, schema/memory-scopes.schema.json}` (regenerated), and, only if S28-e
  merged (C0 step 3), the files of item 4.
- Steps:
  1. §6 in full. The `lakeside-real` case is marked `real_binary` and `slow` (milestone).
  2. AC-d-8's mini transparency set and AC-d-9's degradation cases in `test_anchored_c3.py`; the 12-seat harness
     case marked `slow`.
  3. Regenerate the reference: `uv run --locked python cognition/lm-controller/tools/reference.py --out
     docs/reference`.
  4. If S28-e merged: R-S28d-e-1 (export / import carry `access_anchor` and scope rows; import calls
     `bind_scopes`; `recall --trace` names a line's scope from `SectionLine.level`).
- Validation: `test_anchored_c3.py` without `slow` (AC-d-7, AC-d-8, AC-d-9 at unit speed); `test_reference.py`
  (AC-d-13); `test_memory_isolation.py` (AC-d-11). Mutation **M-d-14** (`index` counts the seat's own entity) → the
  `generated` C-3 case red.
- [ ] Implementation. [ ] Validation. [ ] Review: the oracle imports no code under test; every arm ran.
- **Commit message:** `test(cognition): C-3 on two worlds and a generated one; the reference (S28-d)`

#### C9 — Close-out

- **Files:** this file (§15), `cognition/lm-controller/README.md` (one line: a world can let people remember places
  and things; link to the generated page).
- Steps: AC-d-12's diff scan; the README line; push `mvp0/pr-s28d`; `gh pr create`; CI `fast` on the exact head;
  repair to green.
- [ ] Implementation. [ ] Validation: `fast` green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S28-d close-out`
- **Then:** report MERGE-READY (PR number, head SHA) to the coordinator. The implementation session never merges.

---

## 10. Requirements this PR places on other PRs, and parent edits (owner: the coordinator)

- **R-S28e-d-1 (on S28-e, only if S28-e merges after this PR).** S28-e's `export` / `import` carry
  `access_anchor` and every scope table; `import` calls `bind_scopes` with the imported seat's stack; `recall
  --trace` maps `place` / `item` lines to their scope by `SectionLine.level`. If S28-e merged first, this PR's C8
  item 4 does it instead (the R-P6b-a-1 precedent).
- **R-S28f-d-1 (on S28-f).** The process that opens the shared home calls `bind_scopes` for each seat with that
  seat's stack; `verify` checks Σ `account` = `content_size()` with scope tables included.
- **R-S28g-d-1 (on S28-g).** The anchored cost formula, measured: per seat ≤ `anchors_max` × ≈ 70 B per anchored
  scope, plus ≈ 45 B per held payload anchor.

| Document | Edit |
| --- | --- |
| `step-28-shared-memory.md` §5.2.3 | anchors from typed references; roles `at` / `named`; `access_anchor`; no shared index; the known line's citation; eviction order (QS28d-1 … QS28d-4 as ruled) |
| `step-28-shared-memory.md` §5.4 | the `place` / `item` rows: `roles`, `kinds`; no `granularity` |
| `step-28-shared-memory.md` §7.3 | the anchored cost lines (no shared index term) |
| `step-28-shared-memory.md` §9.2 C-3, §15 AS28-15 | "a recount over the whole export" (F-S28d-9); rebuild exactness (QS28d-5) |
| `pr-s28c-memory-scopes.md` (implementer note) | F-S28d-13: the person scope has five sections on `main` (`with` included) |
| `overall.md` | S28-d frozen / merged |

---

## 11. R-LOAD-1 and speed first

Locally: only the commit's test files, **one at a time**, plus ruff, ruff format, pyright and the doc scripts.
Never the whole `cognition/lm-controller` suite. A PR's CI runs `fast` only (ARC-154): format, compile, the
static checks. The `slow` and `real_binary` cases of C8 (the 12-seat harness, `lakeside-real`, S28-b's Gate 2)
run at the milestone, and failures found there are fixed in that batch.

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S28-d — the anchored scope (place, item), payload anchors, the scope-table
                         host, the recall section, C-3
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s28d-anchored-scope.md
RELATED / BINDING DOCS:  step-28-shared-memory.md (§§3, 5.2.3, 5.3, 5.4, 5.7, 5.8, 7.2, 9.2, 11, 15);
                         pr-s28c-memory-scopes.md (§§5, 10); pr-s28b-retention-on-seat-views.md (§10);
                         pr-s28e-memory-inspect.md (§10); docs/MODULE_SPEC.md §5.3; docs/ARCHITECTURE.md §9.3;
                         docs/decisions/ARC-100.md, ARC-109.md; .structured-coding/standards.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md;
                         /Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s28d, its own, held by one session
BRANCH:                  mvp0/pr-s28d, created from origin/main
START CONDITION:         S28-c merged AND this design frozen, then the coordinator's dispatch
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–13, as §14's rulings shape them              (unresolved until §14 is ruled)
FROZEN INVARIANTS:       step-28 I-S28-1, I-S28-2, I-S28-6 … I-S28-10; §7 I-d-1 … I-d-9; S28-c's I-c-1 … I-c-9;
                         FGb-1 … FGb-7 and FG-b2's numbers; §8.2 as written
SEQUENCE:                C0 … C9 (§9)
ALLOWED COMMANDS:        uv run --locked *; uv sync --locked; cargo build -p mineworld-cli; target/*/mineworld *;
                         git; gh (never merge); python3 scripts/*; python3 <a script file in the scratch dir>;
                         mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; the whole member suite locally
                         (R-LOAD-1); reading any key or .env; calling any model; any cassette record mode; editing
                         a golden render or a cassette; edits outside APPROVED SCOPE; naming an event type, an
                         entity id or a world in memory/ or controller/ code
MATERIAL STOPS:          S28-c not merged; AC-d-1 failing for a reason other than a defect of this PR (the default
                         stack would change what a person recalls); a needed change to a retention policy, an item
                         rule, a bound number or the pass order beyond §5.8's one step; a needed change to a
                         MemoryScope member; a needed change under kernel/, contracts/, server/, persistence/,
                         systems/, worlds/, sdk/; a new dependency or a lock change; a render or request pin moving
PLATFORMS:               Linux, macOS, Windows (the milestone run covers all three)
VALIDATION BUDGET:       unit files unrestricted locally, one at a time; per PR, CI `fast` only; the `slow` and
                         `real_binary` cases at the milestone. Gate 1: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds and the
                                                         design is frozen
                         semantic commits: authorized                     (working rules §14)
                         branch push: authorized                          (working rules §21)
                         PR creation / update: authorized                 (working rules §21)
                         CI repair to `fast` green: authorized            (working rules §21)
                         endpoint: the PR reported MERGE-READY (number and head SHA)
                         merge: by the coordinator only (charter: speed first; the permission system refuses
                         merges by sub-agents); never by the implementation session
POST-MERGE SYNC OWNER:   coordinator: step-28, overall.md; implementation session: this ledger
STOP CONDITION:          S28-d MERGE-READY — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-d-1 | Schema 6 changes every dump (the `schema` value, and anchor lines), so a pinned digest moves and a Haiku implementer "fixes" a render pin too | C0 lists the pins; only dump and schema pins move, each with a reason; a render or request pin moving is a material stop; the coordinator reviews `tests/golden/` and `tests/cassettes/` (must be untouched) |
| R-d-2 | Anchor bytes move a size-triggered pass earlier, so a golden point changes | golden runs stay far below 20 MiB; if one moves, it is a material stop with evidence, not a re-capture |
| R-d-3 | The upsert SQL is built from names, so a bad name injects SQL | names are load-checked against `^[a-z][a-z0-9_]{0,47}$` before any DDL; no value is ever interpolated, every value is a parameter |
| R-d-4 | A shop lists many items, so `QUERY_ANCHORS = 16` hides some | the section shows at most 16 lines anyway; the order is the observation's (ascending id), deterministic; a smarter choice (held items first) is a later scope parameter |
| R-d-5 | S28-c's merged code differs from its frozen design (for example the `with` section, F-S28d-13) | C0 maps every name of §2.3; a difference that breaks §5 is a stop, any other is recorded |
| R-d-6 | Rebuild over many held records slows a seat's first open | one pass over the seat's access rows in key order, ≈ 1 250 per person-day kept; measured at the milestone (S28-g) |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **QS28d-1 [operator — where anchors come from]** | The step anchors items by the envelope's `subject` / `participant`. The envelope gives those entities no kind, and in every audited pack they are persons; items appear in payloads as typed references `{entity, entity_type: "item"}` (D-3, F-S28d-2). Which sources? (a) `at` (the envelope's place) and `named` (the payload's typed references, kinds `place`, `item`, `organization`); `subject` / `participant` deferred to the first scope that needs untyped entities (`organization`, MVP-1). (b) Keep the step's roles and accept that `item` also anchors persons. | **(a).** Typed references are the contract's own statement of what each id is, the same for every pack, so no world-specific code is needed. `item` defaults to `roles: [named], kinds: [item]`; `place` to `roles: [at]`. |
| **QS28d-2 [operator — where payload anchors are kept]** | The payload is not stored in memory, so its anchors must be kept at ingest, or a re-added scope cannot rebuild (I-S28-9). Where? (a) A per-seat side table `access_anchor (seat, event_key, entity, kind)`, written for every held fact that names an anchor, whatever the stack, charged to the person, deleted with the access row. (b) A column on `access` (the step's words). (c) Only while an anchored scope is selected. | **(a).** Rows exist only where a payload names a place or a thing (well under 1 % of a seat view in `social-cafe`). (b) adds bytes to every record of every seat and changes every dump. (c) makes a re-added scope unable to rebuild. |
| **QS28d-3 [operator — the scope-table host's shape]** | S28-c's host plan includes a shared event-keyed index and its cascade. The envelope is already indexed (`fact.place`), and payload anchors are in `access_anchor`, so a shared table would only duplicate them (F-S28d-4). The known row must also count across facts, which S28-c's insert-only rows cannot express (F-S28d-5). Accept: (a) per-seat scope tables only (event-keyed refused at load until a scope needs them), SQLite indices instead of a shared index, no `granularity` parameter, and typed column merge rules on `TableSpec` (`key`, `first`, `last`, `count`) plus `cap` and `evict`? This changes S28-c's `TableSpec` carrier; no `MemoryScope` member changes. | **Yes, (a).** Less code, no duplicated bytes, no GC for scope rows, and the merge is one SQLite upsert. `ARC-100` already foresaw "a write the home cannot express"; the four roles answer it without letting a scope run code inside a write. |
| **QS28d-4 [operator — which known row goes at the cap]** | The step evicts "fewest visits first". At the cap a new anchor always has 1 visit, so it would be evicted at once and the person could never learn a new place (F-S28d-6). Evict instead the anchor seen **least recently** (`last_at`, then entity id), for the cap and for the bound's backstop? | **Yes.** A person forgets the places they have not been to for longest, keeps learning new ones, and one order serves both the cap and the bound. The default cap (256) is far above any shipped world's places and items, so it rarely acts. |
| **QS28d-5 [operator — what "rebuilt is identical" means]** | A known row counts facts that retention later merges away. When a scope is re-added, the home can rebuild only from the facts the seat still holds, so its counts can be smaller than an uninterrupted run's (F-S28d-8). Accept: a rebuild equals an uninterrupted run while nothing was released (tested with `keep-all`), and otherwise equals a recount over the held facts (tested against the oracle)? | **Yes.** It is exact and deterministic, and honest about what was forgotten. The alternative, keeping a removed scope's rows "in case", would charge a person for memory their stack no longer uses. |

Not asked (bounded, recorded in §3.2): the generic scan (F-S28d-1), the known line's citation (F-S28d-7), C-3's
"whole export" recount (F-S28d-9), static headers (F-S28d-10), the `scope-change` log line (F-S28d-11), `shown`
(F-S28d-12), the anchor cap per fact (F-S28d-15). For the coordinator: F-S28d-13 (S28-c's audit predates the
`with` section) should be relayed to S28-c's implementer now.

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C9 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet; C0 records the base, the re-anchored lines, S28-c's names as merged, the state of S28-e, and the pins)

### Mutations

(none yet; M-d-1 … M-d-14 as §9 lists them)

### Deviations and findings

F-S28d-1 … F-S28d-15 (§3.2), recorded at design time.
