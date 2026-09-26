# Overall — MVP-0 "Composable Worlds"

**Effort ID:** `mvp0`
**Lifecycle:** `DRAFT — AWAITING OPERATOR AGREEMENT`
**Authority:** this document is the top-level plan for the effort. Step documents refine it;
PR documents refine those. A lower document never silently overrides this one.
**Repository:** `/Users/yuema137/MineWorld`, branch `main`, base commit `75e1d2b`
**Remote:** none configured (see D-9)

Binding specifications, which this plan implements and may not contradict:
[`docs/MVP.md`](../../../docs/MVP.md) ·
[`docs/CORE_CONCEPTS.md`](../../../docs/CORE_CONCEPTS.md) ·
[`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) ·
[`docs/MODULE_SPEC.md`](../../../docs/MODULE_SPEC.md) ·
[`docs/NETWORKING.md`](../../../docs/NETWORKING.md) ·
[`docs/ENGINEERING_STANDARDS.md`](../../../docs/ENGINEERING_STANDARDS.md) ·
[`CLAUDE.md`](../../../CLAUDE.md)

---

# 1. The effort and its observable outcome

Deliver MVP-0 as specified in `docs/MVP.md`: a headless-first, server-authoritative kernel
plus enough independently installable systems to prove the project's frozen top-level
criterion.

> **Materially different games can be constructed by composing the same core entities with
> different independently installable interaction systems, without modifying the kernel.**

The effort is complete when all twelve acceptance criteria `AC-1 … AC-12` in `docs/MVP.md` §9
hold, each demonstrated by an automated, repeatable test rather than by inspection.

The single decisive outcome, restated as an executable claim:

```text
worlds/social-cafe   (movement, conversation, relationships, group_activity)
        + install inventory, item_transfer, economy, employment
        + change world configuration
        = worlds/market-town

with a diff that touches only  systems/  and  worlds/
and no change to  kernel/, contracts/, controllers, or the renderer
```

## Non-goals for this effort

Out of scope, and not to be smuggled in:

```text
romance, children, health, crime, combat, vehicles
property market, education, construction, business ownership
complex interiors, travel between cities
3D renderer, Unreal adapter
Postgres backend, gateway / multi-worker deployment, Kubernetes
matchmaking, global accounts, server browser
relay / NAT-traversal service
WASM plugin sandbox
Phase 2 world creator GUI, Phase 3 registry, Phase 4 public worlds
```

Each is a later System Pack, client, or phase. None of them may require a kernel change to
become possible later — that is precisely what this effort tests.

---

# 2. Binding constraints carried from the specifications

Restated here because every step is judged against them:

1. Invariants `INV-1 … INV-15` (`docs/CORE_CONCEPTS.md` §2) are structural requirements, not
   review preferences. Where a compile-time expression of an invariant is available, prefer it
   to a runtime check, and prefer a runtime check to a convention.
2. The kernel contains no domain semantics (`INV-12`). Steps S1–S7 must not learn what a job,
   money, hunger, or sleep is.
3. Single-writer rule (`INV-7`): a system writes only components it owns; cross-domain effects
   travel as events.
4. Headless, deterministic, model-free core (`ENGINEERING_STANDARDS.md` §§22–24). **No language
   model is attached before S7 completes.**
5. Integration-first validation (§§17–20): a step is validated by a realistic path
   (create world → load system → act → emit → persist → restart → verify), not by constructors.
6. Rust by default for kernel, systems, persistence, networking, server (§3).
7. Change-amplification test (§8): if a step forces edits across unrelated modules, stop and
   raise an architecture question instead of pushing through.

---

# 3. High-level steps

Each step is one or more PRs. Steps S1–S7 are the pre-model foundation and run strictly in
order. Later steps are stated at broad scope deliberately; they are detailed only when the
merged code that precedes them exists (`CLAUDE.md` §3, "detail one step ahead").

### S1 — Entity and Component contracts

*Corresponds to the operator's commit 2.*

- **Output:** the contract layer for identity and state: `EntityId`, `EntityType`, `Tags`,
  `LifecycleState`, `Metadata`; distinct newtypes for `PersonId`, `PlaceId`, `ItemId`,
  `OrganizationId`, `EventId`, `ActionId`, `ProcessId`, `SystemId`, `WorldTime`; component
  declaration with its owning system, versioned schema, and typed data; the typed `Relation`
  edge.
- **Depends on:** nothing merged; D-4, D-5 and D-7 are resolved (§5).
- **Acceptance checkpoint:** a test constructs all four entity types, attaches a component
  declared by a stub system, and reads it back; passing a `PlaceId` where a `PersonId` is
  required fails to compile (demonstrated by a compile-fail test); `cargo fmt`, `cargo check`,
  `cargo clippy -D warnings` clean on the new workspace.
- **Adversarial criterion:** an attempt to represent a component with an untyped map, or an
  entity whose semantics live outside a component, must be visibly impossible in the API.

### S2 — ActionIntent, Event, and Observation contracts

*Corresponds to the operator's commit 3, plus `Observation` (D-3).*

- **Output:** `ActionIntent`; `ActionResult` with `Accepted` / `Rejected(reason)` /
  `ActionUnavailable`; the `Event` schema with all ten fields of `CORE_CONCEPTS.md` §11,
  including `CausedBy`, `Visibility`, `Provenance`; the `Observation` envelope.
- **Depends on:** S1.
- **Acceptance checkpoint:** serialization round-trips for every contract type; a table-driven
  test shows an intent naming an unprovided action resolving to `ActionUnavailable` (`INV-10`);
  an event cannot be constructed without `CausedBy` provenance (`INV-15`).
- **Adversarial criterion:** no code path can turn an `ActionIntent` into state without passing
  through an `Event` (`INV-2`).

### S3 — System interface and registry

*Corresponds to the operator's commit 4.*

- **Output:** the `System` declaration of `CORE_CONCEPTS.md` §13 (id, version, dependencies,
  owned components, provided actions, emitted and subscribed events, configuration schema,
  migration schema); the registry that resolves dependencies and enables/disables systems; the
  mechanism that enforces single-writer component access.
- **Depends on:** S1, S2.
- **Acceptance checkpoint:** two stub systems are registered; a system writing a component it
  does not own is rejected — at compile time if the chosen design permits, otherwise at runtime
  with a named error and no partial mutation; the registry refuses to enable a system whose
  declared dependency is absent, naming it; disabling a system removes its provided actions from
  dispatch.
- **Adversarial criterion:** the enforcement must not be bypassable by an ordinary API user who
  is not deliberately subverting it. If the chosen design leaves a bypass, record it and raise
  it — this is the step where the composability claim is either made real or lost.

### S4 — World clock, scheduler, and Process

*Corresponds to the operator's commit 5.*

- **Output:** `WorldTime`; the discrete-event / semantic-tick scheduler; process lifecycle with
  interruption *requests* routed to the owning system; event delivery to subscribers.
- **Depends on:** S3. Needs D-6.
- **Acceptance checkpoint:** a seeded run produces an identical event sequence on repeat
  (`AC-12` in the small); a process receiving an interruption request ends only through its
  owner's decision (`CORE_CONCEPTS.md` §10); event ordering under simultaneous timestamps is
  deterministic and documented.

### S5 — Persistence and event sourcing

*Corresponds to the operator's commit 6.*

- **Output:** `PersistenceBackend` trait and `SQLiteBackend`; append-only semantic event log;
  periodic snapshots; `snapshot + events` reconstruction; save manifest.
- **Depends on:** S4.
- **Acceptance checkpoint:** run N ticks → persist → restart process → state and event-log head
  are identical (`AC-6`); replaying the event log from empty reproduces the same state; a
  snapshot plus its tail reproduces the same state as full replay.
- **Adversarial criterion:** no component state exists that the event log cannot reconstruct.
  State that is only reachable by having been in memory is a defect.

### S6 — First real systems: time, places, movement

*Corresponds to the operator's commit 7.*

- **Output:** the smallest system set that makes a world do something observable: place
  occupancy and the `move` action with its `PersonEnteredPlace` event.
- **Depends on:** S5.
- **Acceptance checkpoint:** `move` intent → validate → resolve → event → occupancy change,
  persisted and reloaded; disabling `MovementSystem` makes `move` return `ActionUnavailable`
  with no change to any other module (first real evidence for `AC-2`).

### S7 — Headless demo: World Pack loading and rule controller

*Corresponds to the operator's commit 8. This step closes the pre-model foundation.*

- **Output:** World Pack loader with schema validation (`world.yaml`, `people/`, `places/`);
  `RuleController`; `mineworld run <world> --headless --seed <n> --days <n>`.
- **Depends on:** S6.
- **Acceptance checkpoint:** a seeded rule-based world runs hundreds of simulated days with no
  renderer and no model (`AC-11`); the same seed reproduces the run exactly (`AC-12`); a
  mid-run restart continues correctly (`AC-6` end to end); every state change in a sampled
  window traces to an intent, a process, or a system event (`AC-9`).

### S8 — Social Café system set

- **Output:** `conversation`, `relationships`, `schedule`, `group_activity` System Packs;
  `worlds/social-cafe` World Pack with the MVP population of `docs/MVP.md` §3.
- **Depends on:** S7.
- **Acceptance checkpoint:** a headless seeded run produces conversations, relationship-value
  changes, and group activities; an objective biography for a Person is derived from the event
  log and matches the events that produced it.

### S9 — Market Town and the AC-1 composability proof

- **Output:** `item`, `inventory`, `item_transfer`, `economy`, `employment` System Packs;
  `worlds/market-town`; the automated composability test.
- **Depends on:** S8.
- **Acceptance checkpoint (the primary criterion of the whole effort):** `market-town` is
  produced from `social-cafe` by installing systems and changing configuration, and the test
  asserts mechanically that the change set touches only `systems/` and `worlds/` — no
  `kernel/`, no `contracts/`, no controller, no renderer (`AC-1`).

### S10 — Cognition layer

- **Output:** perception producing `Observation`s; subjective memory; hierarchical biography
  compression L0–L3 with retained Event IDs; `LMController` over Ollama and one
  OpenAI-compatible endpoint; asynchronous cognition workers with mandatory revalidation;
  cognition budgets; recorded and replayable model outputs.
- **Depends on:** S7 (technically), S8 (for anything worth thinking about).
- **Acceptance checkpoint:** swapping the model backend changes no World Pack (`AC-4`); after
  100 simulated days a character's context stays bounded while event provenance is retained
  (`AC-10`); the full core test suite still passes with no model reachable (§24).

### S11 — Server and networking

- **Output:** the server process: HTTP control plane, WebSocket observation streams and intent
  submission, invite-token plus nickname authentication, multi-client sessions, admin client
  surface.
- **Depends on:** S7; meaningful with S8.
- **Acceptance checkpoint:** simulation continues while clients disconnect and reconnect
  (`AC-3`); a human takes over an existing NPC with biography, relationships, inventory and
  employment intact (`AC-5`); several clients inhabit one world and interact with the same NPCs
  (`AC-7`); a message asserting state rather than requesting an action is rejected (`INV-9`).

### S12 — Godot 2D client

- **Output:** the reference top-down 2D client, reading observations and submitting intents.
- **Depends on:** S11.
- **Acceptance checkpoint:** the client renders the world and drives a human-controlled Person;
  killing the client leaves the simulation running (`AC-3` end to end); no simulation contract
  changed to accommodate the renderer (`INV-5`, `INV-14`).

### S13 — Deployment parity and layered CI

- **Output:** Dockerfile and container run; the four CI layers of
  `ENGINEERING_STANDARDS.md` §16 — fast structural checks, core integration tests, scenario
  tests, long-running stability.
- **Depends on:** S11. Creates the GitHub repository and CI workflows per D-9.
- **Acceptance checkpoint:** the same World Pack runs on the laptop and in the container with no
  semantic difference, demonstrated by identical seeded event sequences (`AC-8`); CI runs the
  layers on the right triggers.

### Cross-cutting, delivered with the step that first needs them

```text
tools/world-validator   with S7
tools/replay            with S5
tools/inspector         with S8
tools/benchmark         with S13
sdk/rust, sdk/python    with S10 (Python) and S9 (Rust pack authoring)
```

---

# 4. Requirement coverage

Every operator-required outcome has a home. Nothing leaves this effort except by an explicit
scope decision.

| Requirement | Step |
| --- | --- |
| `AC-1` composability (primary) | S9 |
| `AC-2` modularity | S6 first, confirmed in S9 |
| `AC-3` renderer independence | S11, end-to-end in S12 |
| `AC-4` cognition independence | S10 |
| `AC-5` controller independence | S11, with S10 |
| `AC-6` persistence | S5, end-to-end in S7 |
| `AC-7` networking | S11 |
| `AC-8` local/cloud parity | S13 |
| `AC-9` causality | S2 and S4, verified in S7 |
| `AC-10` bounded cognition context | S10 |
| `AC-11` headless stability | S7 |
| `AC-12` determinism | S4, verified in S7 |
| Commit sequence 2–8 (`docs/MVP.md` §12) | S1 → S7, in order |
| Sample worlds as integration fixtures (§21) | S8, S9, maintained thereafter |
| Layered CI (§16) | S13, with fast checks introduced in S1 |

---

# 5. Decisions

Resolved by the operator on 2026-09-25, in the planning session that produced this document.

| ID | Decision | Resolution |
| --- | --- | --- |
| **D-4** | Contract representation. | **Rust types first.** `mineworld-contracts` holds the single source of truth as Rust types. Protobuf is introduced at the first real cross-language boundary (S10 Python cognition, S11/S12 Godot) and is mirrored from those types. `ARCHITECTURE.md` §13 is amended to record this route in the PR that first writes contracts. |
| **D-5** | Component storage. | **Purpose-built typed store.** Keyed by `EntityId`, one component table per owning system. No general-purpose ECS: the requirements are single-writer enforcement, event-sourced reconstruction, schema migration and persistence, not frame-loop iteration speed. |
| **D-7** | Rust layout. | **One Cargo workspace at the repository root**, crates `mineworld-contracts` (`contracts/`), `mineworld-kernel` (`kernel/`), `mineworld-system-*` (`systems/<name>/`), `mineworld-server` (`server/`), `mineworld-cli` (`tools/cli/`). Crate boundaries make the one-way dependency rule compiler-checked rather than review-checked. |
| **D-9** | Publication endpoint. | **Local-only until S13.** Each step commits on a local branch; no push, no remote, no PR, no remote CI. Every execution contract in this effort records a local-only endpoint. The GitHub repository and CI workflows are created in S13. |
| **D-3** | Whether S2 includes the `Observation` contract. | **Included in S2.** `RuleController` in S7 needs observations, and defining the controller-facing triple (`ActionIntent`, `Event`, `Observation`) together prevents `INV-13` from being retrofitted. Recorded as a planning decision; raise it when agreeing to this document if you disagree. |

## Still open

| ID | Decision | Blocks | Recommendation |
| --- | --- | --- | --- |
| **D-6** | Time model: fixed semantic tick versus a discrete-event queue. | S4 | Discrete-event queue at a declared semantic granularity (e.g. one simulated minute), with a documented deterministic tie-break. Decide when S4 is designed. |
| **D-1** | License: MIT (currently in `LICENSE`) versus Apache-2.0 for its patent grant. | nothing | Operator decision; zero cost to change now, higher after publication in S13. |
| **D-2** | `ENGINEERING_STANDARDS.md` §16 ends "The principle does not:", preserved verbatim from the source. As an agent-facing specification it should read "The principle does not change:". | nothing | Fix in the next docs-touching PR if the operator agrees. |

# 6. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-1** | Compile-time single-writer enforcement may make the system API awkward or impossible in safe Rust without excessive generics. | S3 explores it with a real stub system before committing; if compile-time enforcement costs too much clarity, fall back to runtime enforcement plus a review convention, and record the decision and its cost in the S3 design. Never fall back silently. |
| **R-2** | Event sourcing and component storage are easy to couple accidentally, producing state that replay cannot reconstruct. | S5's adversarial criterion tests reconstruction from an empty store; S1 keeps component data serializable by construction. |
| **R-3** | YAML World Pack schemas drift from the Rust contracts. | S7 generates or derives validation from the contract types rather than hand-writing a parallel schema; `tools/world-validator` is the single validation path. |
| **R-4** | Kernel absorbs domain semantics under time pressure — the failure mode that ends the project's premise. | `INV-12` is an acceptance criterion of every kernel step; S6 and S8 exist partly to keep domain logic outside the kernel from the first real system onward. |
| **R-5** | Effort drifts toward making an appealing demo rather than proving composability. | `AC-1` is mechanically tested in S9, and no model is attached before S7 closes. |
| **R-6** | Determinism erodes as systems are added (iteration order, hash maps, floating point). | S4 fixes ordering rules; every subsequent step's checkpoint includes a seeded-repeat assertion. |

---

# 7. Current position

```text
Completed:  commit 1 — specifications and process (75e1d2b)
Next PR:    PR 01 — S1, Entity and Component contracts
Remaining:  S2 … S13
```

PR 01 covers S1 only: the contract layer for identity, components, and relations, plus the
Cargo workspace it lives in and the fast structural checks that run on it. It does not
implement dispatch, scheduling, persistence, or any system.

D-4, D-5, D-7 and D-9 are resolved (§5), so PR 01 can be designed in detail. Its execution
contract records a local-only endpoint.

## Process facts for this effort

- Planning documents live in `.structured-coding/plans/mvp0/` and **are tracked in Git**, a
  deliberate deviation from the skill default, recorded in `CLAUDE.md` §3.2.
- Project standards are declared in `.structured-coding/standards.md` with `review.trigger`
  at `commit`. The helper reports; it never blocks. `cargo` checks require one
  `standards.py approve` before they run, and report `INCONCLUSIVE` until a workspace exists.
- No host hooks are registered. The `continuity`, `checkpoints`, and `standards` presets are
  off; their obligations are met procedurally.
- Synchronization owner for parent-document updates after a merge: the planning session, per
  the skill default. The implementation session owns updating its own PR document.
