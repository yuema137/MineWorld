# Overall — MVP-0 "Composable Worlds"

**Effort ID:** `mvp0`
**Lifecycle:** `DRAFT — AWAITING OPERATOR AGREEMENT`
**Authority:** this document is the top-level plan for the effort. Step documents refine it;
PR documents refine those. A lower document never silently overrides this one.
**Repository:** `/Users/yuema137/MineWorld`, branch `main`, base commit `75e1d2b`
**Remote:** none configured (see D-9)

**Revision 2026-09-25 (material scope change, operator-instructed).**
`docs/ENGINEERING_RULES.md` was added, promoting an embodied 3D client to a required architecture
target with its own reference demo. Consequences recorded in this document: the 3D reference client
leaves the non-goal list, S2 and S6 gain the spatial contract and the two-scale movement model,
S12 becomes Demo A, a new step S14 delivers Demo B, `AC-13` and `AC-14` join the coverage map, and
risks R-7 and R-8 are added. `AC-1` remains the effort's primary criterion. PR 01, currently
frozen and in execution, is unaffected — see §7.

Binding specifications, which this plan implements and may not contradict:
[`docs/MVP.md`](../../../docs/MVP.md) ·
[`docs/ENGINEERING_RULES.md`](../../../docs/ENGINEERING_RULES.md) ·
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

The effort is complete when all fourteen acceptance criteria `AC-1 … AC-14` in `docs/MVP.md` §9
hold, each demonstrated by an automated, repeatable test rather than by inspection — except
`AC-14`, whose embodiment claim is demonstrated by actually running the 3D client
(`ENGINEERING_RULES.md` §19: rendering work is not validated by inferring it from server tests).

The worlds are meant to be played from inside, not watched. A world that satisfies every
simulation criterion and cannot be walked through has failed the product north star
(`ENGINEERING_RULES.md` §1).

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
Unreal adapter, photorealistic fidelity, art production
Postgres backend, gateway / multi-worker deployment, Kubernetes
matchmaking, global accounts, server browser
relay / NAT-traversal service
WASM plugin sandbox
Phase 2 world creator GUI, Phase 3 registry, Phase 4 public worlds
```

Each is a later System Pack, client, or phase. None of them may require a kernel change to
become possible later — that is precisely what this effort tests.

Two of those exclusions are now conditional rather than absolute. `travel between cities` stays
out of the MVP as a *system*, but the spatial model designed in S2 and S6 must not preclude it:
walking across a room and travelling between towns are two scales of one model
(`ENGINEERING_RULES.md` §6). Visual fidelity stays out, but the 3D *client* is in — Demo B is
judged on embodiment and interaction correctness, explicitly not on graphics.

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
8. `ENGINEERING_RULES.md` §§1–12 are binding. Two questions gate every spatial or interaction
   contract in this effort **before it merges**: can it support a Minecraft-like embodied 3D
   client without redesigning the kernel (§11), and can 2D and 3D use the capability without
   duplicating game logic (§9, §22)? A "no" returns the contract to design.
9. No mesh, animation, physics, skeleton, camera, scene tree, or navmesh concept enters a generic
   contract (§12). The headless server and the 2D client remain fully valid implementations.
10. `REUSE_POLICY.md` is binding. Every step asks what the ecosystem already provides before
   writing infrastructure, and records the answer in [`docs/DECISIONS.md`](../../../docs/DECISIONS.md)
   whether it adopts or declines. Dependencies selected so far: `DEP-1` purpose-built component
   store, `DEP-2` `rusqlite`, `DEP-3` `tokio`/`axum`, `DEP-4` Godot 4.7, `DEP-5` `serde`,
   `DEP-6` scheduler pending S4.
11. Clients report intent; systems decide. Rejections are semantic —  `Unavailable`, `Busy`,
   `TooFarAway`, `PermissionDenied`, `NoSupportedInteraction` — and are produced by the owning
   system, never by a renderer (§§4, 7–8).

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

### S2 — ActionIntent, Event, Observation and Spatial contracts

*Corresponds to the operator's commit 3, plus `Observation` (D-3) and the spatial contract (D-10).*
**Design:** [`step-02-action-event-spatial-contracts.md`](step-02-action-event-spatial-contracts.md)
— approved 2026-09-25 under the autonomous authorization above.

- **Output:** `ActionIntent`; `ActionResult` with `Accepted` / `Rejected(reason)` /
  `ActionUnavailable`, the reason set covering `Busy`, `TooFarAway`, `PermissionDenied`,
  `NoSupportedInteraction`; the `Event` schema with all ten fields of `CORE_CONCEPTS.md` §11,
  including `CausedBy`, `Visibility`, `Provenance`; the `Observation` envelope; **and the generic
  spatial contract** — a semantic position reference (`Cafe.Counter`), an optional continuous
  local position and orientation that a client may supply and the server treats as authoritative
  state rather than geometry, and the per-action declaration of spatial requirements (same
  `Place`, interaction radius, line of access, target available) that a System states and no
  renderer evaluates.
- **Depends on:** S1.
- **Acceptance checkpoint:** serialization round-trips for every contract type; a table-driven
  test shows an intent naming an unprovided action resolving to `ActionUnavailable` (`INV-10`);
  an event cannot be constructed without `CausedBy` provenance (`INV-15`); an action can declare
  "requires proximity" and "requires no proximity" and both round-trip; a continuous position is
  representable without importing any engine concept.
- **Adversarial criterion:** no code path can turn an `ActionIntent` into state without passing
  through an `Event` (`INV-2`); the spatial types contain nothing a 2D-only client would need and
  a 3D client could not use, and nothing a 3D engine would recognize as its own
  (`ENGINEERING_RULES.md` §§11–12).

### S3 — System interface and registry

*Corresponds to the operator's commit 4.* Split into two PRs — kernel world state with
ownership-gated writes, then the System interface and dispatch over it. **Design:**
[`step-03-kernel-and-systems.md`](step-03-kernel-and-systems.md), PR 03a frozen.

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

*Corresponds to the operator's commit 5.* **Design:**
[`step-04-clock-scheduler-process.md`](step-04-clock-scheduler-process.md), frozen.

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
  occupancy and the `move` action with its `PersonEnteredPlace` event. `MovementSystem` keeps
  `MoveIntent`, authoritative spatial state, and rendered movement distinct, and leaves room for
  a travel `Process` at the larger scale without implementing one (`ENGINEERING_RULES.md` §6).
- **Depends on:** S5.
- **Acceptance checkpoint:** `move` intent → validate → resolve → event → occupancy change,
  persisted and reloaded; disabling `MovementSystem` makes `move` return `ActionUnavailable`
  with no change to any other module (first real evidence for `AC-2`); a movement rejected for
  distance returns `TooFarAway` from the system, decided server-side.

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

### S10 — Cognition layer *(reduced for MVP-0)*

**Scope decision, 2026-09-25.** The finish line is two playable demos sharing one semantic path,
and no part of it requires a language model: the operator's own direction is that LM integration
is not the MVP's architectural centre and must not delay validating the world infrastructure, and
that deterministic controllers are what stable integration testing should use. So MVP-0 delivers
the **controller abstraction with `HumanController` and `RuleController`**, plus perception and
`Observation` production, which the clients genuinely need. `LMController` over Ollama and one
OpenAI-compatible endpoint, subjective memory, and hierarchical biography compression move to
MVP-1.

Consequences recorded honestly rather than quietly: `AC-4` (swapping the model backend changes no
World Pack) and `AC-10` (bounded context after 100 simulated days) are **deferred with S10's LM
half**, since neither can be demonstrated without a model. They remain MVP acceptance criteria;
they are simply not gates on the demo finish line the operator set. Every other criterion stays
in MVP-0.

Original scope, retained for MVP-1:

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

### S12 — Demo A: 2D reference client

- **Output:** the reference top-down 2D client, reading observations and submitting intents. It
  is a permanent integration testbed, not a mock UI to be discarded once 3D exists
  (`ENGINEERING_RULES.md` §2).
- **Depends on:** S11.
- **Acceptance checkpoint:** the client renders the world and drives a human-controlled Person;
  killing the client leaves the simulation running (`AC-3` end to end); no simulation contract
  changed to accommodate the renderer (`INV-5`, `INV-14`); it demonstrates the Demo A list in
  `docs/MVP.md` §7.1 — multiple Persons, places, movement, conversation, relationships, basic
  items, basic group activity, persistence, human and agent controllers, multiplayer.

### S13 — Deployment parity and layered CI

- **Output:** Dockerfile and container run; the four CI layers of
  `ENGINEERING_STANDARDS.md` §16 — fast structural checks, core integration tests, scenario
  tests, long-running stability.
- **Depends on:** S11. Creates the GitHub repository and CI workflows per D-9.
- **Acceptance checkpoint:** the same World Pack runs on the laptop and in the container with no
  semantic difference, demonstrated by identical seeded event sequences (`AC-8`); CI runs the
  layers on the right triggers.

### S14 — Demo B: 3D walking world

- **Output:** the embodied first-person reference client: movement, camera and look controls,
  collision and basic navigation, a walkable environment, an enterable `Place`, physically
  represented NPCs, spatial targeting, and interaction with at least one object. Low fidelity is
  expected and acceptable.
- **Depends on:** S11, and on the spatial contract from S2 having survived the §11 gate.
- **Acceptance checkpoint:** `AC-14` — a player walks through the environment, enters a `Place`,
  approaches an NPC, spatially initiates a conversation, and interacts with an object, with all
  state server-authoritative; and `AC-13` — the `Talk` this produces is the *same*
  `ActionIntent`, resolved by the same system, as the one Demo A produces by clicking. Validated
  by actually running the client (`ENGINEERING_RULES.md` §19).
- **Adversarial criterion:** no business rule exists in either client. Removing the 3D client
  changes no system, and the diff that added it touches no kernel contract. If the two clients
  needed separate rule implementations, the architecture is wrong (§9).

### The `AC-13` harness — proven, 2026-09-25

The mandatory cross-renderer test now has a mechanism rather than an intention, demonstrated on
throwaway clients before any real one exists:

```text
2D client:  click Alice                              ─┐
                                                      ├─► identical ActionIntent ─► one system
3D client:  walk near Alice → look at her → press E  ─┘
```

Both clients submitted `{action_type: "talk", actor: "person:player", target: "person:alice"}`.
The server recorded each verbatim, tagged only by which client sent it, and compared them: byte
identical. The outcome — accept, or `TooFarAway`, or `Unavailable` — was decided by the server
alone; neither client evaluated distance, availability or permission.

The harness for S12/S14 follows from that: the server records submitted intents with a client
tag, and the acceptance test asserts equality of the two recordings. What must differ between
renderers is only acquisition — a click versus a camera ray. What must not differ is everything
after it.

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
| `AC-13` 2D / 3D semantic parity | S14, against the client delivered in S12 |
| `AC-14` embodiment | S14 |
| Commit sequence 2–8 (`docs/MVP.md` §12) | S1 → S7, in order |
| Sample worlds as integration fixtures (§21) | S8, S9, maintained thereafter |
| Layered CI (§16) | S13, with fast checks introduced in S1 |
| Playable-world north star (`ENGINEERING_RULES.md` §1) | S12 and S14; gated continuously from S2 onward by the §11 review question |

---

# 5. Decisions

Resolved by the operator on 2026-09-25, in the planning session that produced this document.

| ID | Decision | Resolution |
| --- | --- | --- |
| **D-4** | Contract representation. | **Rust types first.** `mineworld-contracts` holds the single source of truth as Rust types. Protobuf is introduced at the first real cross-language boundary (S10 Python cognition, S11/S12 Godot) and is mirrored from those types. `ARCHITECTURE.md` §13 is amended to record this route in the PR that first writes contracts. |
| **D-5** | Component storage. | **Purpose-built typed store**, now recorded at project level as [`DEP-1`](../../../docs/DECISIONS.md) with the full comparison the reuse policy requires: ECS handles are mostly unstable across save/load, archetype storage optimizes an iteration profile MineWorld does not have, and `&mut World` access would reduce `INV-7` to a convention. |
| **D-7** | Rust layout. | **One Cargo workspace at the repository root**, crates `mineworld-contracts` (`contracts/`), `mineworld-kernel` (`kernel/`), `mineworld-system-*` (`systems/<name>/`), `mineworld-server` (`server/`), `mineworld-cli` (`tools/cli/`). Crate boundaries make the one-way dependency rule compiler-checked rather than review-checked. |
| **D-9** | Publication endpoint. | **Local-only until S13.** Each step commits on a local branch; no push, no remote, no PR, no remote CI. Every execution contract in this effort records a local-only endpoint. The GitHub repository and CI workflows are created in S13. |
| **D-3** | Whether S2 includes the `Observation` contract. | **Included in S2.** `RuleController` in S7 needs observations, and defining the controller-facing triple (`ActionIntent`, `Event`, `Observation`) together prevents `INV-13` from being retrofitted. Recorded as a planning decision; raise it when agreeing to this document if you disagree. |
| **D-1** | License. | **MIT stays.** Operator decision 2026-09-25. Revisit only before publication in S13 if the patent-grant argument becomes material. |
| **D-2** | `ENGINEERING_STANDARDS.md` §16 wording. | **Fixed** to "The principle does not change:" — operator approved 2026-09-25. |
| **D-6** | Time model: fixed semantic tick versus a discrete-event queue. | **Discrete-event queue**, keyed by `(WorldTime, sequence)` with a monotonic sequence as the tie-break, at one-simulated-second granularity. Idle time is skipped rather than ticked, which is what makes `AC-11`'s hundreds of simulated days cheap. Recorded at project level as [`DEP-6`](../../../docs/DECISIONS.md) with the comparison against existing crates. |
| **D-11** | How the reference clients are built and how their demos are verified, given that "actually run the renderer" must be mechanically possible. | **Godot 4.7.2 for both clients, as the frozen spec already names.** Installed and verified on 2026-09-25 to run headless and execute GDScript with stdout. Demo evidence comes from a scripted run that drives input and writes PNG frames, which are then inspected — not from compilation or from server tests. No deviation from `ARCHITECTURE.md` §13 is needed, and therefore no engine concept needs to enter a contract to make verification possible. **Proven end to end on 2026-09-25**, not assumed: a windowed run rendered through Metal on the host GPU, `get_viewport().get_texture().get_image().save_png()` returned `0`, and the resulting 640×360 PNG was read back and visually confirmed. The recipe is `godot --path <project>`, a `_process` counter that captures after a few frames and calls `get_tree().quit(0)`; `--headless` runs scripts and prints but renders nothing, so it is for logic checks only. |
| **D-10** | Whether an embodied 3D client belongs in this effort or a later phase. | **In this effort, as S14.** Operator instruction 2026-09-25 adding `docs/ENGINEERING_RULES.md` §§2–3, §10: 3D is a required architecture target, not a cosmetic renderer added later. This materially expands the effort; `AC-1` is unaffected. |

## Still open

| ID | Decision | Blocks | Recommendation |
| --- | --- | --- | --- |


# 6. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-1** | Compile-time single-writer enforcement may make the system API awkward or impossible in safe Rust without excessive generics. | S3 explores it with a real stub system before committing; if compile-time enforcement costs too much clarity, fall back to runtime enforcement plus a review convention, and record the decision and its cost in the S3 design. Never fall back silently. |
| **R-2** | Event sourcing and component storage are easy to couple accidentally, producing state that replay cannot reconstruct. | S5's adversarial criterion tests reconstruction from an empty store; S1 keeps component data serializable by construction. |
| **R-3** | YAML World Pack schemas drift from the Rust contracts. | S7 generates or derives validation from the contract types rather than hand-writing a parallel schema; `tools/world-validator` is the single validation path. |
| **R-4** | Kernel absorbs domain semantics under time pressure — the failure mode that ends the project's premise. | `INV-12` is an acceptance criterion of every kernel step; S6 and S8 exist partly to keep domain logic outside the kernel from the first real system onward. |
| **R-5** | Effort drifts toward making an appealing demo rather than proving composability. | `AC-1` is mechanically tested in S9, and no model is attached before S7 closes. |
| **R-6** | Determinism erodes as systems are added (iteration order, hash maps, floating point). | S4 fixes ordering rules; every subsequent step's checkpoint includes a seeded-repeat assertion. |
| **R-7** | Spatial and interaction contracts get designed against the easy 2D client, accumulating tile-only movement, click-only interaction, instant movement, single-room places, no orientation and no local geometry — each quietly foreclosing Demo B. | The §11 gate question is part of the acceptance checkpoint of S2 and S6, not a late review. S14 is planned inside this effort precisely so the assumption is tested while changing the contract is still cheap. |
| **R-9** | A client-side encoding defect corrupts world data in a way no Rust test can see. Already demonstrated once: Godot parses JSON numbers as doubles, so 64-bit ids round above 2^53 (S2 design, DD-15). | S11 owns a JSON wire encoding that renders 64-bit ids as decimal strings, and its acceptance requires a round-trip proven through an actual Godot client, not a Rust-only test. The general rule this instance establishes: every cross-language boundary is verified from the far side. |
| **R-8** | The opposite failure: building Demo B pulls engine concepts (mesh, navmesh, camera, scene tree) into generic contracts, so the headless server and 2D client stop being first-class. | S14's adversarial criterion is that removing the 3D client changes no system and its diff touches no kernel contract; `ENGINEERING_RULES.md` §12 is a frozen invariant of that step. |

---

# 7. Current position

```text
Completed:  commit 1 — specifications and process (75e1d2b)
            commit 2 — planning documents (39abfb3)
            commit 3 — playable 2D/3D requirements + governance (10dc9ef, 30630b8)
In flight:  PR 01 — S1, Entity and Component contracts (frozen, executing)
Designed:   PR 02 — S2, action/event/observation/spatial contracts (approved, blocked on S1)
Remaining:  S3 … S14
Toolchain:  rust 1.97.1, Godot 4.7.2, Docker 29.7.2, Python 3.14.7, Node 26.4.0
```

**Effect of the 2026-09-25 scope change on PR 01: none.** S1 delivers identity, the entity
record, the component model and the relation model. It defines no spatial type, no movement, and
no action, so it cannot encode a 2D-only assumption; the §11 gate first applies to S2, which owns
the spatial contract. PR 01 therefore stays frozen as approved rather than being reopened.

PR 01 covers S1 only: the contract layer for identity, components, and relations, plus the
Cargo workspace it lives in and the fast structural checks that run on it. It does not
implement dispatch, scheduling, persistence, or any system.

D-4, D-5, D-7 and D-9 are resolved (§5), so PR 01 can be designed in detail. Its execution
contract records a local-only endpoint.

## Governance and merge order (operator, 2026-09-25)

Project-level rules and revisions to frozen specifications merge into `main` **independently of
implementation work**, and as soon as they are agreed:

```text
docs/engineering-rules ──ff──► main          (10dc9ef, merged 2026-09-25)
                                  │
                                  │  authoritative specs
                                  │
PR 01 branch ─────────────────────┤  frozen scope, untouched mid-flight
  implementation                  │
                                  ▼
                            review PR 01 against the latest main
                                  │
                            approve implementation
                                  │
                            sync with main
                                  │
                            final checks → merge PR 01
```

Two reasons this is not deferred until PR 01 merges. A governance decision must own its own
commit, so that "why did 3D move from non-goal to required architecture target?" is answerable
from history without reading an implementation diff. And any session that starts other work
before PR 01 is reviewed must read the *current* rules from `main` — leaving them on an
unmerged branch reproduces exactly the risk D-10 was recorded to remove.

**PR 01 does not synchronize with `main` mid-flight.** Its scope is frozen and unaffected by
D-10 (§7), so pulling the docs commit into its worktree would add interference for no change in
what it must build. It synchronizes after implementation is approved.

## Autonomous execution authorization (operator, 2026-09-25)

The operator authorized the primary session to act as reviewer and approver for the design
documents, PR plans, architecture reviews, implementation reviews and local integration decisions
required to complete MVP-0, and to continue through step boundaries without pausing for routine
approval.

What this changes:

```text
design → self-review against the frozen specs → record the review → approve
       → implement → validate → review → semantic commit → local integration → next step
```

- The `DESIGN FROZEN` gate stays, but the approval recorded in it may be this session's, and each
  design document carries the review that justified it (see step-02 §8 for the shape).
- Remote PR creation, remote review and remote merge are skipped: no remote exists (D-9).
- Local integration into `main` is authorized for this effort. Every other authority boundary
  stands: frozen invariants still require an explicit revision with evidence, and an architecture
  review trigger still stops feature work.

What this does not change: the obligation to record decisions, evidence, deviations and known
limitations. Self-approval without a recorded review is not approval.

## Review criteria carried into PR 01's review (operator, 2026-09-25)

D-10 does not invalidate PR 01, but the review must confirm that rather than assume it. The
governing question:

> **Does anything in PR 01 accidentally introduce assumptions that constrain the S2 spatial
> contracts?**

Specifically:

```text
1  Entity carries no x/y, tile coordinate, or any other positional field
2  Person inherits no movement or render concept
3  Relation assumes nothing about endpoints sharing a 2D map or any spatial frame
4  the component abstraction is bound to no renderer
5  the id and component model leave normal room for a future spatial component
```

If all five hold, PR 01 needs no rework for D-10. A violation of any of them is a material
finding against the S2 gate in §2, not a style note.

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
