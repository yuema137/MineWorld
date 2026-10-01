# Step 06 / PR 07 — Persistence and event sourcing (S5)

**Role:** combined step and PR document. S5 needs one PR.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 S5 · **Lifecycle:** `DRAFT — AWAITING FREEZE`
**Base:** `main @ 5f02332` (S4 merged as `1241cab`; `5f02332` is the docs-only post-merge update)
**Branch / worktree:** `mvp0/pr-07-persistence` in `/Users/yuema137/mineworld-worktrees/s5-persistence`
**Depends on:** S4's `ScheduleSnapshot` / `World::restore_schedule`, S3's single-writer stores, S5V's
server and `AC-15` acceptance test, `ARC-15` genesis

Binding: [`docs/CORE_CONCEPTS.md`](../../../docs/CORE_CONCEPTS.md) §§2, 3, 11 ·
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) §§7, 8, 14 ·
[`docs/MVP.md`](../../../docs/MVP.md) §9 `AC-6`, `AC-9`, `AC-12`, `AC-15`, §9.1 ·
[`docs/DECISIONS.md`](../../../docs/DECISIONS.md) `DEP-1`, `DEP-2`, `DEP-5`, `DEP-6`, `ARC-15`, `ARC-23` ·
[`docs/NETWORKING.md`](../../../docs/NETWORKING.md) §10 · [`server/PROTOCOL.md`](../../../server/PROTOCOL.md) §5 ·
[`docs/MODULE_SPEC.md`](../../../docs/MODULE_SPEC.md) §9 rule 2

`DESIGN FROZEN` is **not** recorded. This document is a draft for the primary session's review;
§10 lists what that review has to decide.

## Why this is PR 07, and why the file is `step-06`

The overall plan calls this step S5. PR numbers 01–06 are taken (PR 06 was S4), so it ships as **PR
07**. The step-document number follows the existing file sequence (`step-05-vertical-slice.md` is
S5V), so this is `step-06-persistence.md`. Neither number changes what the step is.

---

# 1. Goal

A world survives the death of the process hosting it. `mineworld server worlds/social-cafe --save
DIR`, killed with `SIGKILL` mid-conversation and started again with the same command, is **the same
world**: the same instance, the same `EntityId`s, the same event history continuing at the next
identity, Alice still remembering what she was told — and a client is told which persisted revision
of that world each observation reflects, which closes the fourth evidence line of `AC-15` (`MVP.md`
§9.1).

## 1.1 Scope

```text
kernel/src/snapshot.rs            WorldSnapshot: the whole of a world's state as data; World::snapshot,
                                  World::restore (assembly-time, validated, refuses rather than guesses)
kernel/src/components.rs          a table can encode its rows as ComponentRecords and decode a fresh
                                  table from them, by the component type it was declared with
kernel/src/{world,error,lib}.rs   wiring, refusals, exports
persistence/  (new crate)         mineworld-persistence: WorldInput, WorldRevision, the save format,
                                  PersistenceBackend, SqliteBackend, PersistentWorld (create / resume /
                                  verify), replay with byte-for-byte divergence refusal
worldpack/src/load.rs             WorldPack::compose (install only) and WorldPack::assemble (install +
                                  entities, genesis facts returned unapplied); load = assemble + genesis
server/src/{host,runtime,protocol}.rs   a hosted world may be persisted: every revision committed before
                                  it is answered or observed; `revision` in WorldSummary and in the
                                  observation frame; instance, action ids and the host's pacing resume
tools/cli/src/main.rs             `mineworld server <world> --save DIR`; `mineworld replay <world> --save DIR`
tests                             kernel snapshot/restore; persistence in-process; process-kill
                                  checkpoint (persistence/tests); server restart and AC-15 (tools/cli/tests)
docs                              DECISIONS (ARC-25, DEP-2/DEP-5 notes), ARCHITECTURE §§7, 8, 14,
                                  server/PROTOCOL.md §5, READMEs, MVP_STATUS
Cargo.toml                        workspace member `persistence`, dependency `rusqlite` (bundled)
```

No System Pack changes. That is a property this step must keep, not an accident: persistence that
needed `systems/conversation` or `systems/presence` to change would fail the change-amplification test
(`ENGINEERING_STANDARDS.md` §8, `CLAUDE.md` §4.5).

## 1.2 Non-goals

```text
save migration (a system or component schema changing under an existing save)   refused, not migrated;
                                  the migration schema of CORE_CONCEPTS §13 is a later step (§10 Q7)
Postgres / cloud backend          ARCHITECTURE §8 "later"; same trait, no second implementation now
runtime enable/disable, runtime entity creation or destruction as journaled inputs
                                  no caller exists (audited §8.1); PersistentWorld does not expose them
cognition_cache/                  ARCHITECTURE §8 — S10 / MVP-1
headless `mineworld run`          S7; this step adds only `server --save` and `replay`
log compaction, snapshot pruning  the log is append-only and kept whole; size is not yet a problem
host-only state across a restart  subscribers, observation seq, dropped/faults counters: not world state
removing `deferrals_unscheduled`  left to the next protocol revision, as S4 recorded (§10 Q3)
```

## 1.3 Frozen invariants (proposed)

- **I-1 `INV-11`.** The event log is append-only and is the single source of truth for history. No
  fact is ever rewritten, deleted, re-numbered or "repaired" in a save.
- **I-2 Identity is stable.** Every `EntityId`, `EventId`, `ProcessId`, schedule `Sequence` and
  `ActionId` a world issued before a restart is the same after it, and no identity is issued twice
  across a restart (`DEP-1`'s decisive reason, `CORE_CONCEPTS.md` §3).
- **I-3 One pipeline.** State is reconstructed only by running the kernel's own pipeline — `restore`
  of a snapshot the kernel itself produced, then `genesis` / `dispatch` / `advance_to` of recorded
  inputs. There is no second reducer, no "replay mode", and no path that writes a component outside
  its owning system (`INV-7`, `kernel/src/dispatch.rs` "three entry points share it").
- **I-4 Divergence is a refusal.** A replayed input whose answer or facts differ by a single byte from
  what the save recorded refuses the load, naming the revision. Never tolerated, never repaired.
- **I-5 Versions are refused, never guessed.** A save, a system, a component schema or an event
  schema newer *or* older than the running code is refused with a named error, matching the existing
  `ComponentSchemaTooNew` / `ComponentSchemaOutdated` pair. No silent decode across versions.
- **I-6 Told means persisted.** A revision a client is told about — in a result, a welcome, a status
  answer or an observation — is durable before it is told.
- **I-7 `INV-12` / `DEP-2`.** The kernel learns no domain concept and no SQL; no SQLite type appears in
  the kernel, any system or the server.
- **I-8 `INV-14`.** Simulation semantics do not depend on whether a world is persisted: a persisted and
  an ephemeral world given the same inputs record identical facts.
- **I-9** The 294 pre-existing tests stay green, `AC-15` in particular.

---

# 2. The answer the brief asked for: what is the authority, and what a restart rebuilds from

## 2.1 What the code allows, audited

`ARCHITECTURE.md` §7 states reconstruction as `snapshot_day_50 + events_after_day_50`. Read literally —
*feed the logged facts back through the reducers* — that is not possible with the kernel as built, and
the audit (§8.2 F-1…F-4) says why in four independent ways:

1. **`resolve` holds a writable view.** `System::resolve` is handed `&mut WorldView<Self>`
   (`kernel/src/system.rs:523`). Both shipped systems choose to write nothing there, but the kernel does
   not require it, so a write made while resolving is caused by an `ActionIntent` and recorded in no
   fact.
2. **Process lifecycle is not evented.** `start_process`, `end_process`, `suspend_process`,
   `reschedule_process` and `set_process_state` are writes a system makes in `resolve`, `react`, `wake`
   or `interrupt` (`kernel/src/view.rs`), with or without an accompanying fact. S4's long run ends
   thousands of processes; the facts describe them, they do not *are* them.
3. **Time is an input, not a fact.** A `Wake` fires because the clock reached a process's expected end
   (`kernel/src/advance.rs`), and a deferred fact fires because the clock reached its instant. Nothing in
   the log says "the world advanced to t".
4. **`react` both reduces and emits.** Feeding a logged fact to `react` would emit its consequences a
   second time — consequences that are themselves already in the log. Suppressing them would be a second
   reduction mode, which is exactly what `dispatch.rs` was written to prevent ("two recording paths that
   could drift would be two accounts of one world's history").

## 2.2 Decision (proposed `ARC-25`)

A persisted world is three append-only things in one SQLite file, committed together, one revision at
a time:

```text
facts        every EventEnvelope, in EventId order            HISTORY — the event log (INV-11)
journal      every input that moved the world, in order       what the world was ASKED, and when
snapshots    the whole of a world's state at some revisions   a checkpoint, never an authority alone
```

and the two questions have two different answers, stated precisely:

```text
What happened?               the fact log. Authoritative, append-only, never derived from anything.
                             Biographies, histories and projections are derived from it (INV-4, INV-11).

What is the world's state?   the deterministic pipeline applied to the journal, starting from genesis.
                             A snapshot at revision R is a cache of that function at R. A restart loads
                             the newest snapshot at or below the head and re-executes the journal tail
                             through the same pipeline; every re-executed input must regenerate its
                             logged answer and its logged facts byte for byte, or the load is refused.
```

So the event log is not demoted: it is the *check* on every reconstruction. Replay never trusts the
journal alone — a journal that would produce different facts than the log recorded is rejected — and
it never trusts a snapshot alone: `verify` (and `mineworld replay`) re-executes the whole journal from
genesis and requires every stored snapshot to equal, byte for byte, the state the history produces at
its revision. "Snapshot plus tail equals full replay" (overall S5 checkpoint) is that comparison.

**What is replayed:** the journal — genesis, `dispatch(intent, at)`, `advance_to(until)` — through the
kernel's one pipeline. **What is compared:** each regenerated `ActionResult` and each regenerated fact
against the logged bytes; each regenerated state against each stored snapshot. **What is snapshotted:**
`WorldSnapshot` (§3.1), at genesis, every `snapshot_interval` revisions (default 64), and on clean
shutdown. **What a restart rebuilds from:** the newest snapshot ≤ head, plus the journal after it,
verified against the facts after it.

**Why this and not pure fact replay.** Making §7 literally true would require changing the public
System contract: `resolve`, `wake` and `interrupt` read-only; process lifecycle expressed as facts that
the owner's `react` applies; time advancement recorded as a fact; `react` split into an apply half and
an emit half. That is a material redesign of S3 and S4 contracts that both merged with evidence, and it
would add a second way to reduce. The journal costs one table and keeps one pipeline. It is also the
form recorded cognition already needs (`ARCHITECTURE.md` §7: "recorded LM outputs are replayable"): an
LM controller's decision reaches a world only as an `ActionIntent`, so journaling intents is what makes
an LM-driven run replayable without the model.

**Why it is still event sourcing in the sense the project means.** Every state change remains caused
by an `ActionIntent`, a `Process` or an `Event` (`INV-15`) — the journal records exactly the first and
the clock that drives the second — and the fact log remains the one historical truth. What changes is
one sentence of `ARCHITECTURE.md` §7, which becomes precise rather than aspirational (C1 below). This is
the decision §10 Q1 asks the operator to accept.

**What makes state that only existed in memory a detected defect** (overall S5 adversarial criterion).
A system holding state outside the stores (forbidden by `INV-7`, possible through interior mutability,
a global, or a wall clock) makes re-execution differ from the original run, and the first differing
fact refuses the load with `ReplayDiverged`. C3 pins this with a deliberately misbehaving system: the
instrument is shown to see the defect it exists to catch (`ARC-23`).

## 2.3 What a revision is — the fourth `AC-15` line

```text
WorldRevision(u64)   the position of an input in a world's journal. Revision 1 is genesis; each
                     later journaled input is the next revision. Monotonic, never reused, durable
                     before anyone is told it (I-6).
```

Journaled, each as one revision:

```text
genesis                       always — the world beginning (revision 1)
dispatch(intent, at)          always, whatever the answer: Accepted, Rejected, Unavailable, or a
                              KernelError (a system fault). Every ActionId a server hands out is
                              therefore in the journal, which is what keeps ActionIds unique across a
                              restart (I-2); and a dispatch moves the clock even when refused.
advance_to(until)             only when it fired at least one instant (Advanced::instants() > 0, which
                              counts skipped-only instants too), or faulted
```

Not journaled: an `advance_to` that fired nothing. The server advances at 10 Hz to a one-second clock,
so journaling idle advances would write a revision per second of an empty world. Skipping them is
replay-safe (audited §8.2 F-6): every later journaled input carries its own instant, and the kernel's
checks pass identically because nothing was due. The single consequence — a restored world's clock is
the instant of its last revision, not the last idle second before the crash — is §10 Q4.

A client is told the revision in three places (§10 Q3 for the wire question):

```text
WorldSummary.revision            welcome and GET /status: the world's persisted head, or null when the
                                 world is not persisted
observation frame `revision`     the persisted revision of the state that observation was computed from
result frame                     unchanged; the revision is observable on the next observation
```

`AC-15`'s fourth line then reads, mechanically: both windows' observations carry the same revision
number from one monotonic sequence, `GET /status` reports it, and after the server is stopped the save
file's journal head is that revision (`tools/cli/tests/ac15_one_alice.rs`, C5).

## 2.4 Identity across save and restore

```text
EntityId     restored from EntityRegistrySnapshot (existing refusals: counter too low, id mismatch,
             reuse, repeated key) — never re-allocated. A resumed world creates no entity.
EventId      ScheduleSnapshot.next_event (S4), continued; the first fact after a restart is head + 1
ProcessId    ScheduleSnapshot.next_process (S4)
Sequence     ScheduleSnapshot.next_sequence (S4)
ActionId     the server resumes its allocator at (the highest ActionId in the journal) + 1. Every
             allocated id that reached dispatch is journaled (§2.3), so none is reused.
instance     WorldInstanceId is allocated when a persisted world is CREATED, stored in the manifest,
             and reported unchanged by every later process that resumes it (§10 Q2)
```

Replay from genesis does not re-allocate entities either: the genesis revision records the assembled
world's `WorldSnapshot` *before* genesis (entities created, no state), so replay restores identity and
re-executes only `genesis(at, facts)`. The kernel never has to predict or re-derive an id.

## 2.5 Schema and version refusals

```text
save format      SaveFormatVersion in the manifest; open refuses SaveFormatTooNew / SaveFormatOutdated.
                 One number covers every row encoding (journal, facts, snapshots, manifest).
composition      the manifest records each installed system in registration order: id, SystemVersion,
                 enabled, owned ComponentDeclarations, declared relation types. Resume compares with
                 the world the host composed: a different system set or order → CompositionDiffers
                 naming the first difference; same system at a higher code version →
                 PersistedSystemOutdated; at a lower code version → PersistedSystemTooNew.
                 Registration order is compared because it is the reduction order (BD-4): the same
                 systems in another order are a different world.
components       each snapshot record carries its ComponentSchemaVersion; decoding goes through
                 ComponentRecord::payload_for::<C>(), so ComponentSchemaTooNew / ComponentSchemaOutdated
                 are the existing contract errors, surfaced as KernelError::Contract
events           facts are stored verbatim and never decoded by persistence. An EventSchemaVersion
                 change without a SystemVersion change is still caught: the regenerated fact's bytes
                 differ from the logged ones → ReplayDiverged (I-4)
```

No migration exists. `MODULE_SPEC.md` §9 rule 2 says a pack that changes an owned component's schema
ships a migration; no pack has done so, and the migration schema of `CORE_CONCEPTS.md` §13 is a later
step. Until then the refusal is the correct behaviour (§10 Q7).

## 2.6 Durability and failure

- One SQLite transaction per revision: journal row, its facts, an optional snapshot, the head. A
  process killed mid-transaction leaves the previous revision as head; nobody was told the unfinished
  one, because the reply and the observation sweep happen after commit (I-6).
- SQLite in WAL mode. `synchronous = FULL` for a hosted world, so a revision a client was told survives
  power loss as well as a process kill; `Durability::ProcessCrash` (`synchronous = NORMAL`, durable
  against an application crash per SQLite's WAL documentation) is available to headless bulk runs and
  is what the long process-kill scenario uses (§10 Q5).
- The in-memory world is applied first and committed second (the facts come from applying). If a
  commit fails, the in-memory world is ahead of its save: `PersistentWorld` poisons itself and refuses
  every further input (`WorldAheadOfSave`), and the server reports the world stopped. Continuing would
  tell clients about revisions that do not exist on disk.
- A `KernelError` out of dispatch or advance is journaled with its error text and replayed: the kernel
  is deterministic, so replay must reproduce the same error, and the partial writes the kernel
  documents (`dispatch.rs` "where the refusal promise narrows") are reproduced with it. The server's
  existing keep-serving behaviour is unchanged (§10 Q6).

---

# 3. Design decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| **PD-1** | State authority = newest snapshot + journal tail, re-executed and verified byte-for-byte against the fact log; history authority = the fact log (§2.2). | §2.1: pure fact replay contradicts the System contract as merged. One pipeline, one log, and every reconstruction checked. |
| **PD-2** | `WorldSnapshot` is a kernel type: composition, `EntityRegistrySnapshot`, every component as a `ComponentRecord`, `RelationStoreSnapshot`, `ScheduleSnapshot`. `World::snapshot()` / `World::restore(snapshot)`. | Only the kernel can see the typed rows behind `ComponentStore`'s erasure (`DEP-1`), and only the kernel can validate a restore against its own invariants. Reuses the three snapshot types that already exist. |
| **PD-3** | Component rows are encoded as JSON (`serde_json`) by the kernel, one `ComponentRecord` per row, payload bytes; `serde_json` becomes a kernel runtime dependency. | The kernel holds the typed value; something must encode it. `DEP-5` already selected `serde_json`; `ComponentRecord`'s erasure boundary was built for exactly this. JSON from `BTreeMap`-ordered values is canonical, and `clippy.toml` bans `HashMap` workspace-wide, so equal state gives equal bytes. Recorded as a `DEP-5` note. |
| **PD-4** | `World::restore` is assembly: refused once the world has run, refused into a world that already has entities, and validates everything — composition, every record's table, owner, schema version, entity existence, decodability, relation declarations and endpoints, the schedule (S4's checks) — before replacing anything. | Kernel rule 4: a refusal changes nothing. A save that could not have come from this world is refused at load, not discovered later. |
| **PD-5** | New crate `mineworld-persistence` at `persistence/`: `WorldInput`, `WorldRevision`, `PersistenceBackend` (trait), `SqliteBackend`, `PersistentWorld`. The kernel stays free of the journal and of SQL. | `DEP-2`: no SQL type in the kernel or a system. `ARCHITECTURE.md` §14 lists persistence under `kernel/`; the crate is the kernel *layer*, one-way dependent on the kernel crate, and §14 is amended to say so. |
| **PD-6** | `PersistenceBackend` stays a trait with one implementation. No in-memory backend: tests use real SQLite files. | `DEP-2` decided the isolating trait; the abstraction rule (`ENGINEERING_STANDARDS.md` §28) argues against a second, test-only implementation, and a real file is the realistic path (§4.9). |
| **PD-7** | One file, `DIR/world.sqlite`, holding `manifest`, `journal`, `facts`, `snapshots` tables. No separate `manifest.json`. | Two files cannot be committed atomically; a manifest outside the transaction is a second copy of a fact that can disagree. `ARCHITECTURE.md` §8's layout is "conceptual" and is amended. |
| **PD-8** | `PersistentWorld` owns the `World` and exposes only journaled operations (`dispatch`, `advance_to`) plus reads. No `&mut World` escapes. | A mutation that bypassed the journal would be state replay cannot reproduce. Making it unreachable is the structural form of the adversarial criterion. |
| **PD-9** | Genesis is revision 1, recording the pre-genesis `WorldSnapshot`, the instant and the genesis `Emission`s. `WorldPack` gains `compose()` (install only, for resume) and `assemble(at)` (install + entities, facts returned unapplied); `load` is `assemble` + `genesis`, unchanged for its callers. | `ARC-15` made genesis a recorded fact; this records the input that produced it, so replay re-runs genesis instead of trusting its result. Entities are authored, not computed, so restoring them is honest. |
| **PD-10** | The server holds either an ephemeral `World` or a `PersistentWorld` behind one internal type; for a persisted world, every input is committed before its reply and before the sweep. `HostClock` resumes from the restored world's `now`; world time does not pass while no process hosts the world. | I-6. `NETWORKING.md` §10 says the world continues while nobody is *connected*; while nobody is *hosting* it, there is no clock to continue on. Pausing is the only deterministic answer (§10 Q4). |
| **PD-11** | `revision` is added to `WorldSummary` and to the observation frame, additively, within protocol revision 1. | Smallest change; a revision-1 client ignores an unknown field correctly. The alternative — bump to 2 and remove `deferrals_unscheduled` together — touches the Godot client (§10 Q3). |
| **PD-12** | `mineworld replay <world> --save DIR`: composes the world from the pack, re-executes the whole journal from genesis, compares every fact and every stored snapshot, prints the evidence, exits non-zero on any divergence. Delivered as a subcommand, not a `tools/replay` crate. | `overall.md` §3 puts `tools/replay` with S5; `ARC-6` makes `mineworld` the one binary. It is also the operator-runnable form of the `AC-6` claim. |
| **PD-13** | On resume, the server pre-fills its perception window with the last `recent_events` facts from the log. | Otherwise a restart silently empties what perception can look back over — a visible difference between a restarted world and one that never stopped. |

---

# 4. Commit plan

Each commit tracks implementation, validation and review separately. Evidence goes to §9.

## C0 — Design (this document) — docs only
- [x] Implementation: audit (§8), design (§§1–7), execution contract (§11).
- [x] Validation: `python3 scripts/check_decision_ids.py` → 33 ids, all distinct;
  `python3 scripts/check_doc_headings.py` → 134 numbered sections across 21 documents, none duplicated.
- [ ] Review: primary-session review against the frozen specifications; `DESIGN FROZEN` recorded or
  revisions requested. **Implementation does not start before this.**

## C1 — Specification amendments, before code
**Goal:** the decisions this step makes exist in the specifications before any code depends on them
(`CLAUDE.md` §2.2; §2.1 rule 4).
- [ ] Implementation:
  - `docs/DECISIONS.md`: new `ARC-25` (§2.2: state authority vs history authority, journal, revision,
    divergence refusal, what is and is not journaled); `DEP-2` implementation note (crate placement,
    one file, WAL, `synchronous`, no `spawn_blocking` because the world already lives on its own
    blocking thread); `DEP-5` note (`serde_json` as a kernel runtime dependency, PD-3).
  - `docs/ARCHITECTURE.md` §7 (reconstruction stated as in §2.2), §8 (save layout PD-7), §14
    (`persistence/`).
  - `server/PROTOCOL.md` §5: `instance` semantics for persisted worlds; `revision` in `world` and in
    `observation`; §9 unchanged except noting `revision` is additive.
- [ ] Validation: both doc scripts; every new section heading unique; `ARC-25` id distinct.
- [ ] Review: each amended paragraph agrees with §2 here and with `INV-11`, `INV-14`, `INV-15`; no term
  from `CORE_CONCEPTS.md` used with a new meaning ("journal" and "revision" are defined in `ARC-25`, not
  synonyms of Event Log).

## C2 — Kernel: a world's whole state as data
**Goal:** `World::snapshot` and `World::restore` cover everything a world holds. **Depends on:** C1.
- [ ] Implementation:
  - `kernel/src/components.rs`: `ComponentRows` gains `encode(&self, &ComponentDeclaration) ->
    Result<Vec<ComponentRecord>, KernelError>` and `decoded(&self, Vec<ComponentRecord>) ->
    Result<Box<dyn ComponentRows>, KernelError>` (a *new* table of the same type — decode never touches
    the live one); `ComponentStore::{records, restored}`.
  - `kernel/src/snapshot.rs` (new): `WorldSnapshot { composition: Vec<InstalledSystemRecord>, entities:
    EntityRegistrySnapshot, components: Vec<ComponentRecord>, relations: RelationStoreSnapshot, time:
    ScheduleSnapshot }`, `InstalledSystemRecord { system, version, enabled, owns, relation_types }`;
    `World::snapshot()`, `World::restore(WorldSnapshot)`; `World::composition()` for the persistence
    manifest.
  - `kernel/src/error.rs`: `RestoreIntoPopulatedWorld`, `RestoredCompositionDiffers`,
    `PersistedComponentTypeNotInstalled`, `PersistedComponentForUnknownEntity`,
    `PersistedComponentRepeated`, `PersistedComponentUndecodable`, `ComponentNotEncodable`,
    `PersistedRelationForUnknownEntity`, `RestoredRelationDeclarationsDiffer` (exact names settled in
    implementation; each names what it found).
  - `kernel/Cargo.toml`: `serde_json` to `[dependencies]`.
  - `kernel/README.md`, `kernel/src/lib.rs` module table.
- [ ] Validation (`kernel/tests/snapshot.rs`):
  - a world with components of two systems, relations, a pending deferral and a running process:
    snapshot → JSON → restore into a freshly composed world → snapshot again: identical bytes; then the
    same requests dispatched to both worlds produce identical facts (continuation, not just equality);
  - identity: the restored world's next entity-free operations allocate `EventId`/`ProcessId`/
    `Sequence` exactly where the original would (asserted against the original world's continuation);
  - refusals, each changing nothing (world snapshot before == after): restore after the world ran; into
    a world with entities; composition with a missing system, an extra system, two systems swapped;
    a record for an uninstalled component type; for an entity not in the registry; a repeated record;
    an undecodable payload; a record one schema version newer → `Contract(ComponentSchemaTooNew)`;
    one older → `Contract(ComponentSchemaOutdated)`; a relation endpoint not in the registry;
  - `ARC-23`: the round-trip test asserts the snapshot holds the counts the scenario implies (components
    per type, edges, entries, processes) before comparing bytes, so "equal" is not two empty worlds.
- [ ] Review: no component written outside its owner (restore swaps whole stores built from records the
  owner's own type decodes — no `WriteToken` is needed or minted); no `HashMap`; no wall clock; restore
  validates before mutating; `INV-12` (no domain term in the new code).

## C3 — `mineworld-persistence`: journal, save format, SQLite, resume, verify
**Goal:** a world can be created into a save, driven, reopened, and verified against its own history.
**Depends on:** C2.
- [ ] Implementation:
  - `Cargo.toml`: member `persistence`; `rusqlite = { version = <current>, features = ["bundled"] }`.
  - `persistence/src/input.rs`: `WorldInput::{Genesis { before: WorldSnapshot, at, facts: Vec<Emission> },
    Dispatch { intent: ActionIntent, at }, Advance { until }}`; `Outcome::{Dispatched(ActionResult),
    Advanced { instants, skipped }, Began, Fault(String)}`; `WorldRevision`.
  - `persistence/src/format.rs`: `SaveFormatVersion`, `Manifest { format, instance: u128, pack: String,
    composition }`, row encodings (serde_json).
  - `persistence/src/backend.rs`: `PersistenceBackend` — `create(manifest, genesis revision)`,
    `manifest()`, `head()`, `commit(Revision)`, `journal(range)`, `facts(range)`, `latest_snapshot(at_or_below)`,
    `snapshots()`; `Revision { revision, input, outcome, facts, snapshot: Option<_> }`.
  - `persistence/src/sqlite.rs`: `SqliteBackend::{create, open}` — schema, WAL, `Durability`, one
    transaction per commit, refusal to create over an existing save and to open a missing one.
  - `persistence/src/world.rs`: `PersistentWorld::{create, resume, dispatch, advance_to, checkpoint,
    world (read-only), revision, instance, highest_action_id, recent_facts}`; snapshot cadence; poison on
    commit failure.
  - `persistence/src/replay.rs`: apply one `WorldInput` to a `World` through the public kernel API;
    compare outcome and fact bytes; `verify(composed, backend) -> Verified { revisions, facts, snapshots
    compared }`.
  - `persistence/src/error.rs`: `PersistError` — `SaveFormatTooNew`, `SaveFormatOutdated`,
    `CompositionDiffers`, `PersistedSystemTooNew`, `PersistedSystemOutdated`, `ReplayDiverged { revision,
    detail }`, `SnapshotDisagreesWithHistory { revision }`, `WorldAheadOfSave`, `SaveExists`, `NoSave`,
    `Storage`, `Kernel(#[from] KernelError)`.
  - `persistence/README.md`.
- [ ] Validation (`persistence/tests/`, real files in a per-test directory under `std::env::temp_dir()`):
  - create → 200 dispatches and advances over a test world with components, relations, deferrals and
    processes → drop → `resume` → identical `WorldSnapshot` bytes and identical continuation facts
    against an uninterrupted in-memory twin;
  - `verify` from genesis: every fact compared (count asserted against the scenario's own arithmetic, not
    read from the code), every stored snapshot compared; snapshot at R plus tail == full replay at head;
  - resume uses the newest snapshot and a non-empty tail — asserted by location (which snapshot
    revision, how many inputs replayed), not inferred (`ARC-23`);
  - an idle `advance_to` creates no revision; a rejected dispatch does; a fault is journaled and
    replays to the same error;
  - adversarial: a system that keeps a counter outside its components (interior mutability, the
    defect `INV-7` forbids) → `verify` / resume refuses with `ReplayDiverged` at the first revision it
    changes, naming it; a fact row's bytes altered in the file (test-side `rusqlite` dev-dependency) →
    `ReplayDiverged`; a snapshot row altered → `SnapshotDisagreesWithHistory`;
  - refusals: format version ±1; a system version ±1; a missing system; swapped registration order;
    create over an existing save; open of a missing one;
  - durability ordering: after `dispatch` returns, a second connection opened on the file sees the
    revision (committed, not buffered).
- [ ] Review: no SQL type crosses `persistence/src/sqlite.rs`; `PersistentWorld` hands out no
  `&mut World`; every refusal changes nothing in the file; facts are never decoded; rows are compared as
  bytes, not re-serialized values.

## C4 — Pack composition split and the process-kill checkpoint
**Goal:** prove continuity across a real process death against a real SQLite file. **Depends on:** C3.
- [ ] Implementation:
  - `worldpack/src/load.rs`: `WorldPack::compose() -> Composed { world, providers }` (install only),
    `WorldPack::assemble(at) -> Assembled { world, ids, at, facts: Vec<Emission>, providers }`; `load`
    = `assemble` then `World::genesis`, behaviour and errors unchanged.
  - `persistence/tests/kill_and_resume.rs`: the parent re-executes its own test binary as a child
    (`std::env::current_exe`, an environment variable selecting the role); children print `revision N`
    per commit on stdout. Two scenarios:
    - **(a) social-cafe:** `WorldPack::read("worlds/social-cafe")` → `assemble` →
      `PersistentWorld::create` → a fixed script of ~400 `arrive` / `talk` requests among the pack's
      people at strictly increasing instants (some deliberately rejected, e.g. out of reach);
    - **(b) clock-heavy:** a test-local routine/pager world (processes with expected ends, refused and
      accepted interruptions, deferred facts) advanced over 30 simulated days, `Durability::ProcessCrash`.
  - Per scenario, for kill points early / middle / late: child A runs until it has printed revision K,
    then the parent `SIGKILL`s it (`Child::kill`) while it is still writing; child B resumes the same
    file and finishes the script from the first step after the restored world's `now`; child C runs the
    whole script uninterrupted into a second file.
- [ ] Validation: per run, the facts table, the journal and the final head snapshot of B's file equal
  C's **byte for byte**; `verify` passes on B's file; B reports the head H it resumed at (H ≥ K), the
  snapshot revision it loaded and the tail length it replayed — each asserted non-trivial (tail > 0 for
  at least one kill point per scenario); the scenario's facts are counted against bounds derived from
  the script (`ARC-23`). Existing `worldpack/tests` pass unchanged (the split changes nothing they see).
- [ ] Review: the kill is a real `SIGKILL` of a real process, not a dropped value; nothing in the child
  flushes or checkpoints on a signal; the comparison is between files written by different processes.

## C5 — Server and CLI: `--save`, the revision on the wire, a real restart
**Goal:** `AC-6` through the command an operator types, and `AC-15`'s fourth line. **Depends on:** C4.
- [ ] Implementation:
  - `server/Cargo.toml`: depends on `mineworld-persistence`.
  - `server/src/host.rs`: `HostedWorld::persisted(PersistentWorld)`; `WorldHost::shutdown` checkpoints a
    persisted world.
  - `server/src/runtime.rs`: an internal `Hosted` type over ephemeral and persisted worlds;
    `submit`/`advance` go through it and commit before answering or sweeping; instance taken from the
    save; `ActionIds` resume at `highest_action_id + 1`; `HostClock` epoch = restored `now`; perception
    window pre-filled (PD-13); `WorldAheadOfSave` stops the world (every later submit refused
    `world_stopped`).
  - `server/src/protocol.rs`: `WorldSummary.revision: Option<WorldRevision>`; `ServerFrame::Observation
    { seq, revision, observation }`; `WorldInstanceId::allocate` public for the composition root.
  - `server/src/session.rs`: the observation channel carries the revision with the observation.
  - `tools/cli/src/main.rs`: `server <world> --save DIR` (create when `DIR/world.sqlite` is absent,
    resume when present — printing which, with the snapshot revision and tail replayed);
    `replay <world> --save DIR` (PD-12). `--help` text.
- [ ] Validation:
  - `tools/cli/tests/restart.rs` (real binary, real socket, real kill): `server social-cafe --agent alice
    --save DIR`; the visitor arrives next to Alice and talks; Alice's controller replies; record the
    instance, Alice's `EntityId`, the highest `EventId` seen, the action ids, `GET /status` revision R;
    `SIGKILL`; start the same command again; reconnect: same instance, `status.revision == R`, the barista
    is the same `EntityId`, the visitor's own observation still places it next to Alice and its own
    history still holds Alice's reply; a new `talk` within `CONVERSATION_GAP` records **one** fact, not two
    — the conversation continued, so Alice's `ConversationHistory` survived; its event id is the pre-kill
    highest + 1 and its action id is above every pre-kill action id;
  - then `mineworld replay social-cafe --save DIR` exits 0 and reports the revisions and facts it
    compared;
  - `tools/cli/tests/ac15_one_alice.rs`: runs with `--save`; both welcomes carry a revision; the
    observations both windows receive after the exchange carry the same revision, equal to
    `GET /status`; after the server stops, the save's journal head equals it. The module comment that
    records the fourth line as missing is replaced by the evidence;
  - existing `server/tests` (ephemeral worlds) pass unchanged: `revision` is `null` there.
- [ ] Review: an observation is never sent with a revision that is not committed; no persistence call
  happens on an async task (only the world thread touches the save); the server crate names no SQLite
  type; `INV-13` unchanged (the revision is a number about the world, not a view of state).

## C6 — Documentation and ledger close
- [ ] Implementation: `persistence/README.md`, `server/README.md`, `tools/cli` usage, `docs/MVP_STATUS.md`
  (`AC-6`, `AC-15` line four), `kernel/README.md` "still to come"; this document's §9 ledger and closeout.
- [ ] Validation: the terminal gates of §6 on the final executable head.
- [ ] Review: every claim in the docs points at the test that demonstrates it.

---

# 5. Integration checkpoint

The real path `CLAUDE.md` §4.9 names, twice — once deterministic, once through the operator's command:

```text
IC-1  persistence/tests/kill_and_resume.rs      create world (from the real pack) → act → persist →
      SIGKILL the process → a new process resumes the same SQLite file → continue → facts, journal and
      state byte-identical to an uninterrupted run; verify() from genesis passes
IC-2  tools/cli/tests/restart.rs                mineworld server --save → clients act → SIGKILL →
      mineworld server --save → same instance, same Alice, same revision, events continue at head + 1,
      Alice still remembers; mineworld replay verifies the save
```

Why two. Byte-identical continuation needs a deterministic driver: the server paces world time from the
wall clock, so two server runs never stamp the same instants and cannot be compared byte for byte. IC-1
supplies the determinism; IC-2 supplies the real command, the real socket and the real host loop. S4
proved save→restore continuity in memory; both of these cross a real process boundary.

**Adversarial criteria.** A memory-only system state is detected (C3). A tampered fact or snapshot is
detected (C3). An observation never carries an uncommitted revision (C5 review). The kill lands while
the child is writing, not at a convenient boundary (C4). No System Pack changed (diff check at C6).

# 6. Test ownership and verification

```text
static        fmt, clippy -D warnings, types: unchanged ownership
unit          kernel snapshot/restore refusals and round trip; persistence refusals, journaling rules,
              divergence detection (C2, C3)
real lifecycle  the process-kill checkpoint (real child processes, real SIGKILL, real fsync, real file);
              the server restart (real binary, real sockets, real SIGKILL) (C4, C5)
real model    NOT REQUIRED — no LM-facing semantics change
CI            N/A — no workflow in the repository (S13); the local gates below are the terminal evidence,
              run once on the final executable head
```

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --no-fail-fast
python3 scripts/check_decision_ids.py
python3 scripts/check_doc_headings.py
```

Budget: the process-kill tests are bounded to well under a minute of wall time in total (scenario (b)
uses `Durability::ProcessCrash` to keep fsync cost out of a 30-day run).

# 7. Self-review against the frozen specifications

```text
CHECKED  INV-11  the fact log is append-only and never rewritten; every reconstruction is checked
                 against it (I-1, I-4)
CHECKED  INV-7   restore swaps whole stores decoded by the owning component types; no WriteToken is
                 minted, no system's component is written by another; PersistentWorld exposes no &mut
                 World (PD-8)
CHECKED  INV-12  new kernel code knows snapshots, records and versions; no domain term
CHECKED  INV-14  the backend stores bytes and is never consulted for semantics; persisted and ephemeral
                 worlds record identical facts (I-8)
CHECKED  INV-15  every journaled input is an ActionIntent, the clock that drives Processes, or genesis
CHECKED  AC-6    IC-2 restarts the real server with SIGKILL and finds the same world
CHECKED  AC-9    unchanged causation; journal adds the missing link from fact to request bytes
CHECKED  AC-12   replay is AC-12 applied to a save; divergence is refused
CHECKED  AC-15   §9.1 line four gets a referent: WorldRevision, on the wire and in the file
CHECKED  DEP-1   identity restored, never re-allocated (§2.4)
CHECKED  DEP-2   rusqlite bundled behind PersistenceBackend; no SQL type in kernel, systems or server;
                 backend choice not reopened
CHECKED  DEP-6   the schedule's canonical form (BTreeMap) is what is saved
CHECKED  ARC-15  genesis is revision 1 and is re-executed, not trusted
CHECKED  ARC-23  every byte-equality is preceded by a located count; the kill point, snapshot revision
                 and tail length are reported and asserted
CHECKED  REUSE   SQLite reused, no file format of our own; one trait, one implementation
CHECKED  §8      change amplification: no System Pack, controller or client changes
CHECKED  ENGINEERING_RULES §§11–12  no spatial contract touched; 2D and 3D clients unaffected (additive
                 field)
FLAGGED  ARCHITECTURE §7 is reinterpreted (PD-1) — material, §10 Q1
FLAGGED  PROTOCOL.md §5 instance semantics change for persisted worlds — §10 Q2
FLAGGED  kernel/src/error.rs (639 lines) will pass ~750 with C2's refusals: under the 800 warning;
         reviewed in C2 rather than split pre-emptively
```

---

# 8. Source audit (`main @ 5f02332`, 2026-09-30)

## 8.1 What was inspected

```text
kernel/src/{lib,world,dispatch,system,components,entities,relations,schedule,advance}.rs
kernel/tests/long_run.rs (save/restore pattern), kernel/Cargo.toml
contracts/src/{component,event}.rs        ComponentRecord / EventRecord version refusals
systems/{conversation,presence}/src/system.rs   what resolve and react write
worldpack/src/load.rs                     install → create → genesis in one call
server/src/{runtime,host,protocol}.rs, server/PROTOCOL.md, server/Cargo.toml
tools/cli/src/main.rs, tools/cli/tests/{ac15_one_alice.rs, support/mod.rs}
clients/protocol/mineworld/world_client.gd   PROTOCOL = 1 check
docs: MVP §9/§9.1, ARCHITECTURE §§2,7,8,14, CORE_CONCEPTS §§2,3,11,13,16, NETWORKING §10,
      MODULE_SPEC §9, DECISIONS DEP-1/2/5/6, ARC-15, ARC-23; step-04 §§8–11
callers of create_entity / create_authored_entity / enable / disable / destroy_entity outside tests:
      worldpack/src/load.rs only (assembly)
baseline  cargo test --workspace --no-fail-fast → 294 passed, 0 failed (main @ 5f02332)
```

## 8.2 Findings

**F-1 — `resolve` may write (bounded; decisive for PD-1).** `System::resolve` takes `&mut
WorldView<Self>`. `ConversationSystem::resolve` and `PresenceSystem::resolve` document "writes
nothing", by choice. Pure fact replay would silently lose any write a future system makes there.

**F-2 — processes change without facts (bounded; decisive for PD-1).** `WorldView` process writes
happen in any hook; `fire_wake` is driven by the clock. The S4 long run's state lives largely in
processes.

**F-3 — no event log exists yet (bounded).** `Dispatched::events`, `Advanced::events` and
`World::genesis` *return* facts; the server keeps a 64-fact window (`HostConfig::recent_events`). S5
writes the first durable log; nothing has to be migrated.

**F-4 — component state cannot leave the store today (bounded).** `ComponentStore` erases each table
behind `ComponentRows { len, as_any, as_any_mut }`; nothing can encode a row without naming its type.
PD-2/PD-3 add the encode/decode methods to that trait, where the type is still known.

**F-5 — three snapshot types already exist (bounded).** `EntityRegistrySnapshot`,
`RelationStoreSnapshot` (both with validated `TryFrom`) and S4's `ScheduleSnapshot`. `WorldSnapshot`
composes them rather than inventing parallel shapes.

**F-6 — idle advances (bounded).** `World::advance_to` with nothing due only moves the clock;
`check_dispatch_instant` refuses only a backwards instant or due work. Replaying the journal without
idle advances therefore passes the same checks and fires the same entries. The restored `now` is the
last revision's instant (§10 Q4).

**F-7 — the server's allocator and clock restart from zero (bounded; a defect for a resumed world).**
`ActionIds::new()` starts at 1 and `HostClock::new(config.epoch)` at the epoch on every start. A resumed
world would reuse `ActionId`s already named in `Causation::Action` and be refused
`ClockWouldMoveBackwards` on its first request. PD-10 and §2.4 resolve both.

**F-8 — `instance` is per process (needs a decision).** `WorldInstanceId::allocate` mixes the process
id and wall time, and `PROTOCOL.md` §5 says two worlds loaded from one pack are two instances. With a
save, a restarted process hosts the *same* world (`AC-6`). §10 Q2.

**F-9 — the pack loader does assembly and genesis in one call (bounded).** `WorldPack::load` installs,
creates and genesis-es; resume needs composition alone and creation needs the genesis facts unapplied.
PD-9 splits it without changing `load`'s callers.

**F-10 — the Godot client checks `protocol == 1` (bounded; drives PD-11).**
`clients/protocol/mineworld/world_client.gd:258` refuses any other number. An additive field needs no
client change; a version bump does.

**F-11 — dependency fetch (risk).** `rusqlite`/`libsqlite3-sys` are not in the local cargo cache; C3
needs network access to crates.io and a C compiler for the `bundled` feature (present: Apple clang).

## 8.3 Material findings

**PD-1 (F-1, F-2).** The literal reading of `ARCHITECTURE.md` §7 and of `overall.md` S5's "replaying
the event log from empty reproduces the same state" is not achievable without redesigning the merged
System contract. This design proposes the journal instead and amends §7. That is a change of a frozen
specification's wording, so it is raised (§10 Q1) rather than assumed.

---

# 9. Ledger and evidence

**E-0 (C0).** Baseline on `main @ 5f02332`: `cargo test --workspace --no-fail-fast` → 294 passed,
0 failed. Design drafted; both doc checks PASS (33 ids distinct; 134 sections, none duplicated).

---

# 10. Questions for the primary session / operator

```text
Q1 (material)  Accept PD-1 / ARC-25: state = newest snapshot + journal tail re-executed through the one
               pipeline and verified byte-for-byte against the fact log; ARCHITECTURE §7 and overall
               S5's "replay the event log" reworded accordingly. Alternative: redesign the System
               contract for pure fact replay (resolve/wake/interrupt read-only, processes evented,
               time as facts) — a material S3/S4 contract change, not recommended.
Q2             A persisted world keeps its WorldInstanceId across restarts (allocated at creation,
               stored in the manifest). PROTOCOL.md §5 amended: "two worlds created from one pack are
               two instances; one world resumed from its save is one instance." Recommended — AC-6 says
               a restart is the same world.
Q3             `revision` additive within protocol revision 1 (recommended; no client change), or bump to
               2 and remove deferrals_unscheduled in the same change (touches the Godot client).
Q4             World time pauses while no process hosts the world, and a restarted world resumes at the
               instant of its last revision (idle seconds after it are not persisted). Recommended.
Q5             Hosted worlds commit with synchronous=FULL (an fsync per revision; "persisted" survives
               power loss). Headless/bulk callers may choose ProcessCrash durability. Recommended.
Q6             Faulted dispatches/advances are journaled and replayed; the server keeps serving after a
               fault as today. A commit failure, by contrast, stops the world.
Q7             No migration in S5: any system/component/save-format version difference is refused by
               name. Migration (CORE_CONCEPTS §13 "Migration Schema", MODULE_SPEC §9 rule 2) is a later
               step when the first pack needs it.
```

---

# 11. Execution contract (to be confirmed at freeze)

```text
PROJECT / PR        MVP-0 · Step 06 / PR 07 — Persistence and event sourcing (S5)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-06-persistence.md (this file)
RELATED / BINDING   overall.md §§2, 3 (S5), 7; CORE_CONCEPTS §§2, 3, 11, 13; ARCHITECTURE §§7, 8, 14;
                    MVP §9, §9.1; NETWORKING §10; DECISIONS DEP-1, DEP-2, DEP-5, DEP-6, ARC-15, ARC-23
                    (+ ARC-25 once C1 lands); ENGINEERING_RULES; ENGINEERING_STANDARDS; step-04 §§8–11
IMPLEMENTATION BASE main @ 5f02332; branch mvp0/pr-07-persistence; worktree
                    /Users/yuema137/mineworld-worktrees/s5-persistence (held by this session only)
APPROVED SCOPE      §1.1, as answered in §10
FROZEN INVARIANTS   §1.3 I-1 … I-9
SEQUENCE            C0 → C1 → C2 → C3 → C4 → C5 → C6, each committed and pushed when coherent
VALIDATION BUDGET   unit/integration/static: unrestricted; real-model: NOT REQUIRED; process-kill and
                    restart tests bounded to well under a minute of wall time in total
LIVE DOCUMENTATION  this file (§4 checkboxes, §9 ledger)
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for PR 07 at C1
ENDPOINT AUTHORITY
  implementation + local validation   authorized after DESIGN FROZEN — operator autonomous
                                      authorization (overall §7, 2026-09-25); primary session's brief
                                      of 2026-09-30 ("Phase 2 — after the freeze")
  semantic commits                    authorized — same sources; "commit and push after every
                                      coherent step" (brief)
  branch push                         authorized — D-12; brief
  PR creation / update                authorized — D-12; brief ("open a PR with gh pr create")
  CI repair                           N/A — no CI workflow exists in the repository (S13)
  merge                               explicit operator authorization only; the brief repeats
                                      "do not merge"
POST-MERGE SYNC     the planning session owns step/overall updates; this session owns this document
NORMAL STOP         PR 07 READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       any change to §1.3, to an existing public contract's shape beyond §1.1, to
                    ownership, or to scope — stop and report with evidence
```
