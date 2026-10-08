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

The effort is complete when all fifteen acceptance criteria `AC-1 … AC-15` in `docs/MVP.md` §9
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

ARC-35 makes this measurable. The operator approved it on 2026-10-07.

- **The transformation** is the merges of PR 11d and PR 11e. Those merges may touch only
  `systems/**` and `worlds/**`, plus `Cargo.lock` path packages under `systems/` and Markdown
  documentation.
- **The precursors** are 11a–11c. They add framework capability before the transformation, and a
  scan proven to bite holds them to naming no market concept.
- **Installing a pack is static.** It means a pack directory, two lines in `systems/installed`, and
  a rebuild (ARC-33). Installing without a rebuild is ARC-8 and is outside MVP-0.

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
   `DEP-6` purpose-built discrete-event queue (implemented in S4).
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
[`step-04-clock-scheduler-process.md`](step-04-clock-scheduler-process.md), frozen; implemented as
PR 06 (GitHub #20), ready for operator review — see §7.

- **Output:** `WorldTime`; the discrete-event / semantic-tick scheduler; process lifecycle with
  interruption *requests* routed to the owning system; event delivery to subscribers.
- **Depends on:** S3. Needs D-6.
- **Acceptance checkpoint:** a seeded run produces an identical event sequence on repeat
  (`AC-12` in the small); a process receiving an interruption request ends only through its
  owner's decision (`CORE_CONCEPTS.md` §10); event ordering under simultaneous timestamps is
  deterministic and documented.

### S5 — Persistence and event sourcing

*Corresponds to the operator's commit 6.*

*Design:* [`step-06-persistence.md`](step-06-persistence.md), frozen 2026-09-30, PR 07.

- **Output:** `PersistenceBackend` trait and `SqliteBackend`; append-only semantic event log;
  the input journal; periodic snapshots; `snapshot + journal tail` reconstruction verified against
  the event log; save manifest.
- **Depends on:** S4.
- **Acceptance checkpoint:** run N ticks → persist → kill the process → restart → state and
  event-log head are identical (`AC-6`); re-executing the journal from genesis reproduces the event
  log byte for byte and the same state; a snapshot plus its journal tail reproduces the same state as
  full re-execution.
- **Adversarial criterion:** no state exists that re-executing the journal cannot reconstruct, and
  no reconstruction is accepted whose re-executed facts differ from the logged ones. State that is
  only reachable by having been in memory is a defect, detected as a replay divergence.
- **Reworded 2026-09-30 (`ARC-25`).** This entry read "replaying the event log from empty
  reproduces the same state". The S5 audit showed that literal fact replay is incompatible with the
  merged System contract — `react`, `wake` and `interrupt` write state *and* return further
  emissions, `resolve` may write, and process wakes are driven by time rather than by facts — so
  reconstruction re-executes the recorded inputs and the event log is the byte-for-byte check on it.

### S6 — First real systems: time, places, movement

*Corresponds to the operator's commit 7.*

- **Output:** the smallest system set that makes a world do something observable: place
  occupancy and the `move` action. `MovementSystem` decides whether a step is legal and states
  presence's `Arrived` fact; **presence** owns location and states `PersonEnteredPlace`, because
  occupancy is presence's state (`ARC-26`; reworded after step-07 Q5). `MoveIntent`,
  authoritative spatial state, and rendered movement stay distinct, with room for a travel
  `Process` at the larger scale without implementing one (`ENGINEERING_RULES.md` §6). The old
  `arrive` action is retired, since it bypassed every distance rule.
- **Depends on:** S5.
- **Acceptance checkpoint:** `move` intent → validate → resolve → event → occupancy change,
  persisted and reloaded; disabling `MovementSystem` makes `move` return `ActionUnavailable`
  with no change to any other module (first real evidence for `AC-2`); a movement rejected for
  distance returns `TooFarAway` from the system, decided server-side.

### Artefact note, from S7 onward

**Artefact note (`ARC-6`).** From S7 onward every step produces something a person can *run*, not
a library another step will use. S7 is where `mineworld` becomes a real command:
`create`, `validate`, `run`, `inspect`. S11 makes `mineworld server <world>` the hosted runtime;
S12 and S14 produce `mineworld-2d` and `mineworld-3d` as programs, however they end up packaged.

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
- **Depends on:** S11. The repository now exists (`D-12`), so this step keeps the CI workflow files and the container work only.
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

### S15 — Bodies and physical interaction *(inserted 2026-10-07, operator requirement)*

**Design:** [`step-11-bodies.md`](step-11-bodies.md). People never pass through each other or through
walls; walking nudges people and pushes loose objects aside; people can kick, throw and shove. Presence
resolves every arrival before recording it, through an `ArrivalResolver` seam carried by `installed!`
(`ARC-39`, a framework precursor proven with a synthetic resolver); a `bodies` System Pack on Rapier
(`DEP-13`) is the first resolver. The 3D client collides with people and reconciles on difference (Jolt,
`DEP-14`). No kernel or contract change; the log holds only true arrivals.

- **Depends on:** S9 complete (11f merged). **Feeds:** S14 (`AC-14` collision and object interaction),
  S12 (the same three actions).
- **Acceptance checkpoint:** with no resolver installed, existing worlds' facts are byte-identical; after
  every request of a seeded 30-day run with bodies, no two people in one place are closer than 595 mm,
  every nudge is at most 310 mm and every chain at most two generations; a person walking into a box
  moves it; kick, throw and shove are resolved server-side and replay byte for byte after SIGKILL, on
  arm64 and on x86_64; removing `bodies` makes the three actions unavailable and changes nothing else.

Applied in the S9 closeout from `step-11-bodies.md` §12.1. The step design is `DESIGN FROZEN
(2026-10-07)`, revision 1, with PRs 12a–12e; S15 precedes S14 (step-11 §12).

### The `AC-13` harness — proven, 2026-09-25

The mandatory cross-renderer test now has a mechanism rather than an intention, demonstrated on
throwaway clients before any real one exists:

```text
2D client:  click Alice                              ─┐
                                                      ├─► identical ActionIntent ─► one system
3D client:  walk near Alice → look at her → press E  ─┘
```

Both clients submitted `{action_type: "talk", actor: "person:player", target: "person:alice"}`.
The server recorded each verbatim, tagged only by which client sent it, and compared them:
identical.

**Corrected 2026-09-26.** That proof used a stripped-down intent, so "byte identical" held for it
and does not hold for the real contract — `docs/MVP.md` §9 now states the criterion as an
identical *semantic core*. PR 04 narrowed the gap further: with `ActionRequest`, a client no
longer invents an `action_id` or an `issued_at`, so of the three fields that legitimately
differed, only `actor_location` remains — and that one differs by design, because a 2D client
that models no position sends none. The outcome — accept, or `TooFarAway`, or `Unavailable` — was decided by the server
alone; neither client evaluated distance, availability or permission.

The harness for S12/S14 follows from that: the server records submitted intents with a client
tag, and the acceptance test asserts equality of the two recordings. What must differ between
renderers is only acquisition — a click versus a camera ray. What must not differ is everything
after it.

### Critical-path reorder, 2026-09-26 — a vertical slice before more layers

The steps as numbered build the runtime bottom-up and reach three live windows only at S14. That
ordering delays the project's most informative moment for no dependency reason. `AC-15`'s minimal
form does not need the scheduler, durable persistence, the economy or any LM: it needs dispatch
(done), a thin conversation system, enough perception to see who is present, a server, and two
clients.

So after PR 03b merges, the next effort is a **vertical slice** — the thinnest possible everything
— aimed squarely at:

```text
Terminal 1:  mineworld server worlds/social-cafe
Terminal 2:  mineworld-2d
Terminal 3:  mineworld-3d
```

alive at once, with one Alice between them. S4 (scheduler), S5 (durable persistence), S9 (Market
Town) and the rest then thicken a system that already runs end to end, rather than being
prerequisites to seeing it run at all. The composition axis (`AC-1`) is unaffected and follows the
slice.

This is a sequencing change, not a scope change: every acceptance criterion stands.

### S5V — the vertical slice *(inserted 2026-09-26, ahead of S4)*

**Design:** [`step-05-vertical-slice.md`](step-05-vertical-slice.md), frozen. Four PRs taking the
project to three live windows and one Alice, since `AC-15`'s minimal form needs dispatch, a thin
conversation system, perception, a server and two clients — and nothing from S4, S5 or S10.

### Cross-cutting, delivered with the step that first needs them

```text
tools/world-validator   with S7 — delivered as `mineworld validate` (S5V/05c); no separate tool
tools/replay            with S5 — delivered as `mineworld replay` (S5)
tools/inspector         with S8 — delivered early as `mineworld inspect` (S7, with the AC-9 check)
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
| `AC-15` one world, many windows | S14, against S11's server with S12's client also connected |
| Commit sequence 2–8 (`docs/MVP.md` §12) | S1 → S7, in order |
| Sample worlds as integration fixtures (§21) | S8, S9, maintained thereafter |
| Layered CI (§16) | S13, with fast checks introduced in S1 |
| Playable-world north star (`ENGINEERING_RULES.md` §1) | S12 and S14; gated continuously from S2 onward by the §11 review question |
| Operator requirement 2026-10-07: bodies do not interpenetrate; nudge, push, kick, throw | S15 |

---

# 5. Decisions

Resolved by the operator on 2026-09-25, in the planning session that produced this document.

| ID | Decision | Resolution |
| --- | --- | --- |
| **D-4** | Contract representation. | **Rust types first.** `mineworld-contracts` holds the single source of truth as Rust types. Protobuf is introduced at the first real cross-language boundary (S10 Python cognition, S11/S12 Godot) and is mirrored from those types. `ARCHITECTURE.md` §13 is amended to record this route in the PR that first writes contracts. |
| **D-5** | Component storage. | **Purpose-built typed store**, now recorded at project level as [`DEP-1`](../../../docs/DECISIONS.md) with the full comparison the reuse policy requires: ECS handles are mostly unstable across save/load, archetype storage optimizes an iteration profile MineWorld does not have, and `&mut World` access would reduce `INV-7` to a convention. |
| **D-7** | Rust layout. | **One Cargo workspace at the repository root**, crates `mineworld-contracts` (`contracts/`), `mineworld-kernel` (`kernel/`), `mineworld-system-*` (`systems/<name>/`), `mineworld-server` (`server/`), `mineworld-cli` (`tools/cli/`). Crate boundaries make the one-way dependency rule compiler-checked rather than review-checked. |
| **D-9** | Publication endpoint. | ~~Local-only until S13.~~ **Superseded 2026-09-26 by `D-12`.** |
| **D-12** | Publication endpoint, revised. | **Remote from now on.** `git@github.com:yuema137/MineWorld.git` exists; `main` and the in-flight branch are pushed. The operator is enabling branch protection on `main`, so from here every change reaches `main` through a pull request. Execution contracts change accordingly: **branch push is authorized**, **PR creation is authorized**, and **merge still requires explicit operator authorization** — protection makes that a mechanism rather than a promise. Remote CI becomes possible earlier than S13 planned, so S13 keeps only the workflow files and container work. |
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

**Last updated 2026-10-07** (S15: 12a merged as `03f1d7c`; next is 12b. Before that: the S9
closeout, 11f merged as `fea2516`, S9 complete, S15 placed).
**Restated 2026-09-29.** This section had not been updated since PR 01 — it still read
"In flight: PR 01" with seventeen PRs merged — because the post-merge obligation to update
PR → step → overall was skipped after nearly every merge. A plan that cannot answer "where are
we" has stopped being the authority, so the obligation is restated below and is not optional.

```text
Done (main @ 41d4ab1, 311 tests):
  S1   Entity / Component contracts                          PR 01
  S2   ActionIntent / Event / Observation / spatial          PR 02
  S3   System interface, registry, dispatch                  PR 03a, 03b
  --   contract fixes from the renderer-integration spike    GitHub PR #2 ("PR 04")
  S5V  vertical slice                                        05a 05b 05c 05d (GitHub #3 #4 #5 #7)
       -> Milestone A complete: AC-15 holds, one Alice across two clients and an agent
  S4   world clock, scheduler, Process                       PR 06 (GitHub #20), merged 1241cab
       Reviewed independently before merge: all gates re-run (294 passed, 0 failed); two
       mutations the implementing session could not run were run in review — disabling the
       clock's backwards check fails 2 tests, reversing same-instant queue order fails 5 — and
       both were reverted. F-6 (ProcessTypeId added to contracts) accepted as bounded: purely
       additive, follows the EventTypeId pattern, backs the `type` field CORE_CONCEPTS §10
       already specifies. 300 simulated days replay byte for byte from one seed.
  S5   persistence and event sourcing                        PR 07 (GitHub #22), merged 41d4ab1
       ARC-25: state = newest snapshot + journal of inputs re-executed, verified byte for
       byte against the append-only fact log (literal fact replay is incompatible with the
       merged System contract — react/wake/interrupt both mutate and emit). Reviewed
       independently before merge: gates re-run (311 passed, 0 failed); the process-kill
       checkpoint IC-1 re-run (SIGKILL early/middle/late in two worlds, every resume
       byte-identical to an uninterrupted run); one mutation run in review — restarting the
       server's ActionId allocator at 1, i.e. reinstating the restart defect found at
       freeze — fails both restart tests, then reverted. AC-15's fourth evidence line,
       same persisted revision, now holds.
       Open, outside S5 (F-13): a restarted `--agent` rule controller re-answers its last
       line, because what it has answered lives in controller memory, not world state.
       Belongs with S10 (cognition); recorded so Milestone D does not rediscover it.
  S6   places and movement                                   PR 08 (GitHub #25), merged 6f61582
       ARC-26: movement decides whether a step is legal and states presence's `Arrived`;
       presence alone reduces it and keeps the power to refuse an invalid one. A system may
       emit another's vocabulary only by declaring it and depending on its owner, enforced by
       the kernel at install. `arrive` retired; social-cafe gains the street outside the café,
       joined by a passage, so `PersonEnteredPlace` fires in a real world. Client reporting
       rule: report before travelling MAX_STRIDE (2 m). Reviewed independently before merge:
       gates re-run (330 passed, 0 failed); one mutation run in review — disabling the ARC-26
       install check fails its targeted test, then reverted. The implementing session's own
       mutations showed the AC-2 removability test and the reporting-rule test both bite.
  S7   headless demo: run, create, inspect                   PR 09 (GitHub #27), merged 4f4cb1d
       The audit's key finding: a headless social-cafe did nothing (no system acts alone and
       RuleController only answers), so a 300-day run would have been five reproducible,
       meaningless facts. A seeded PacedRuleController takes initiative; an activity
       precondition (every seat moved and talked in every 30-day bucket) is checked before
       determinism. `mineworld run|create|inspect`; argument parsing on clap (DEP-11, the
       trigger recorded in main.rs having been met). Movement discloses a place's passages,
       verified through the real Godot module. F-13 closed for `run` by a stateless pace
       window; still open for `--agent` (S10). Reviewed independently before merge: gates re-run
       (350 passed, 0 failed), and the deliverable run by hand — two `run --seed 7 --days 60`
       saves dump to byte-identical 55 MB fact tables, seed 8 differs, every seat moved ~2,000
       and talked ~1,400 times per 30 days, and `inspect` resolved all 28,170 causes.

Next, framework (critical path to Milestone B):
  S8   Social Café system set — three PRs (step-09, frozen 2026-10-07). Milestone B closes at 10b.
       10a  the town                              PR 10a (GitHub #29), merged 2f24eef
            The café re-authored to the 3D slice's layout, so a person at the counter can talk to
            Alice (and is refused from the door) — the world-data follow-up is closed. The MVP
            town: six places on one street, twelve people, eleven seats. Also fixed a latent S7
            bug: the paced controller's strides came out a fraction of a millimetre over 2 m
            through a rounded-down square root, and S7's own test measured with the same
            rounding, so it passed. Reviewed before merge: gates re-run (353/0), and a 30-day
            run checked by hand — 16,595 moves accepted, none refused, after confirming the run
            summary does print rejected outcomes when they occur.
       10b  social life                             PR 10b (GitHub #31), merged 85451c7
            `relationships` and `group-activity` System Packs, `mineworld biography`, and a
            controller that invites, answers, joins and leaves. Milestone B DEMONSTRATED,
            awaiting the operator: `milestone_b.rs` locates Alice and Bob becoming acquainted,
            sharing an activity and crossing a relationship level, SIGKILLs the run after that
            history, resumes it byte-identical, then hosts the save, SIGKILLs and restarts the
            server, and reads both relationship values and both biographies unchanged. Reviewed
            before merge: gates re-run (381/0); a 30-day run and Alice's biography read by hand;
            the Milestone B test confirmed to have run and passed in the review's own run.
            Dev profile now opt-level 1 (ARC-30): a 300-day run 64.7 s → 10.6 s, output identical.
            **Known gap:** relationships never decay, so the social graph saturates — level
            crossings are 313 in days 1–30, 39 in 31–60, and 0 in every later bucket, while group
            activities continue at ~640 per bucket. A living-world deficiency for a later step.
       10c  names and routines, on a generic content seam   PR 10c (GitHub #33), merged 9ab5e62
            The `authoring/` crate (`AuthoredSection`, ARC-31): a pack owns its section of a
            person file and seeds its own facts. `naming` owns `display-name`; it is public, and
            every name differs from its key. Replies say "Earlier, Vera Lindgren said …" or
            "someone else", and never an id. The biography and the Godot module show names.
            `schedule` (ARC-32) gives each person an agenda. It never moves anyone itself, and
            the controller follows it: seats reach 96.7–100 % of agenda time against 39–51 %
            with following off. I-1 held: kernel, contracts, persistence and server unchanged.
            Freeze condition met: the restart and AC-15 tests assert the routine-free
            00:00–05:00 window themselves and fail by name when a boundary moves into it.
            Reviewed before merge: gates re-run (419/0; 407 tests + 12 doctests, which explains
            10b's 369 vs 381). Mutation check: with name disclosure turned off, three tests fail
            (AC-15, naming's AC-2, disclosure).
            **Known gap (L-12):** people walk ~8 m per in-world hour at the headless pace, so a
            journey takes a median of 2 h. The routines were authored with parts of the day of
            at least 4 h so that they can be followed. This is a living-world deficiency for a
            later step, alongside relationship saturation.
       S8 COMPLETE. Milestone B is demonstrated and awaits the operator's test (on 266daf7;
       10c does not change its claims).
       F-1 and F-3 (AC-1: independently installable packs; a controller limited to actions it
       knows by name) are carried to S9 as material findings (step-09 §10.1).

  S9   Market Town and AC-1 — six PRs (step-10, frozen 2026-10-07; operator approved ARC-35)
       11a  installable System Packs                PR 11a (GitHub #36), merged c472636
            `sdk/rust` (`SystemPack`, `installed!`), `systems/installed` as the build's only list of
            packs, `worldpack` naming only presence and movement, the members glob `systems/*`
            (ARC-33, DEP-12). The I-2 vocabulary scan is `tests/acceptance` (ARC-35 point 7).
            A canary install touched only `systems/**`, `worlds/**` and one `Cargo.lock` path
            package. Reviewed before merge:
            - gates re-run (428/0);
            - my own plant ("wages" in sdk) failed the scan by file and line;
            - kernel/contracts/persistence/server diff empty.
            The social-cafe 300-day output is unchanged (E-0 sha reproduced by the
            implementer). The PR must be merged with a merge commit, which the scan's range uses.
       11b  items and organizations as content kinds  PR 11b (GitHub #39), merged ae1a315
            `world.yaml`'s `items:` and `organizations:` name files under `items/` and
            `organizations/` with tags, a note and sections (ARC-36: an authored Item is a kind,
            and holdings are counts). Their ids come after people's, so no existing id or genesis
            fact moves. The I-2 scan admits words, not lines. Reviewed before merge:
            - gates re-run (441/0);
            - my own plant (`// an item shop` in `worldpack/src/read.rs`) was refused naming `shop`;
            - the diff touched no contracts, kernel, persistence, server, clients or systems.
       11c  complete affordances (F-3)              PR 11c (GitHub #40), merged c5dc51c
            An affordance may carry the exact request its pack would accept (`Affordance<P>.payload`,
            `Offer::complete`, ARC-34). The paced controller attempts an available one at a fixed
            rate of 20 in 100, frozen before the market exists (I-9). CP-3 is shown with a
            test-only pack the controller was never compiled against. Rebased onto ae1a315 and
            merged second. Reviewed before merge:
            - gates re-run on the rebased head (456/0);
            - my own mutation (`ATTEMPTS_OFFERED = 0`) failed the CP-3 ring test and the
              byte-identity test;
            - the contract change is additive;
            - the worst-case `chimes` measurement passed at 20.
       The precursors are complete. On c5dc51c the I-2 scan reads all three as merged, and the
       social-cafe 300-day sha is still E-0 (step-10 §9 E-3).
       11d  owning and giving things                PR 11d (GitHub #43), merged 70e532f
            The first half of the measured AC-1 transformation. `item` declares kinds, `inventory`
            alone writes what people and organizations hold and refuses any fact it may not take, a
            person carries at most six (ARC-37: without it the unseated Otto absorbs every item), and
            `item-transfer` offers a complete `give` per kind held, which the unchanged paced
            controller attempts. `worlds/market-town` is Social Café plus the three packs and their
            content. Reviewed before merge:
            - the change set, checked on the actual merge diff `70e532f^1..70e532f`: 0 paths outside
              `systems/`, `worlds/`, `Cargo.lock` and Markdown;
            - gates re-run (479/0);
            - my own 300-day market-town run, seed 7: 0 faults, 15.2 s, every seat moved and talked in
              every bucket;
            - my own 30-day save holds `items-transferred` facts, each caused by an action, 37,090
              facts in all — the implementer's count;
            - my own mutation, `PERSON_CAPACITY` 6→7, failed two capacity tests.
            F-37 (authoring's `Seeding` doc said sections see earlier genesis facts; none are applied
            before every section is seeded) is fixed by a docs-only PR outside the AC-1 range.
       11e  work, money, shops and consumption      PR 11e (GitHub #46), merged 2dddda8
            The second and last half of the measured AC-1 transformation; the market is six packs
            (ARC-35 note, operator QS-35). `economy` owns wallets and shops and is the only mover of
            money; `employment` turns attendance at a shift into `wage-due` and production, never
            touching a wallet; `consumption` offers `eat` and `drink`; `inventory` gained
            `items-produced` and `items-consumed` and is still the only writer of holdings (ARC-38).
            The unchanged paced controller buys, eats and drinks; two people work by following the
            routines they already had. Reviewed before merge:
            - the change set, checked on the actual merge diff `2dddda8^1..2dddda8`: 0 paths outside
              `systems/`, `worlds/`, `Cargo.lock` and Markdown;
            - gates re-run (510/0);
            - my own 300-day market-town run, seed 7, with `--save`: 0 faults, 372,755 facts (the
              implementer's count), 33.7 s; `inspect` shows 2,717 `items-consumed` and 3,300
              `money-transferred`, and no `wage-unpaid`;
            - my own mutation (the payer is not debited, which breaks money conservation) failed two
              economy tests, the buy test among them.
            **Known gap (L-13):** a bounded-horizon economy, not a closed one. Ten of twelve people
            have no income and live on endowments sized for the measured 300 days; past that they run
            out. Closing it (more jobs, or income without one) is a later step's work.
            F-47 (a pack owns one section, so a shop is authored on its operator) and F-48 (an offer
            carries one unavailability reason) are framework limits worked within; both are carried
            to the S9 closeout.
            The measured transformation is complete: 11d `70e532f` and 11e `2dddda8`.
       11f  the proof                               PR 11f (GitHub #51), merged fea2516
            Outside the AC-1 range. `tests/acceptance/tests/ac1_composability.rs` runs ARC-35's
            three checks: the two transformation merges change only `systems/`, `worlds/`,
            `Cargo.lock` and Markdown; only `systems/` depends on a market pack (normal and build
            edges, QS-54); Market Town's files are Social Café's plus sections the six packs own.
            Check 1 fails, never skips, on a shallow clone. `market_town.rs` makes CP-4 a committed
            300-day test, `market_composition.rs` is AC-2 at world level for the six market packs,
            and `milestone_c.rs` drives Milestone C through the real server with a real SIGKILL.
            No pack, world or framework file changed. Reviewed before merge:
            - gates re-run on the PR head (526/0);
            - the diff touched no pack, world or framework file; the only change under `worlds/`
              is market-town's README;
            - my own mutation, a comment naming `mineworld_inventory` planted in
              `server/src/lib.rs`, failed check 2, naming `server/src/lib.rs:84`; reverted;
            - after the merge, `ac1_composability` passes on main itself, 13/13, on a non-shallow
              clone.
            DP-5 is ruled a bounded deviation, not a material stop. Milestone C's claim, "works and
            earns", is located in the save: Alice arrives during the shift, then `shift-ended` with
            time worked, `wage-due`, and the payment. "Present at shift start" was a wrong planning
            assumption. Content finding, not fixed here (I-9): Alice's routine and her shift both
            start at 05:30, so she is always late; a content issue for a later step.
       S9 COMPLETE (2026-10-07).
            AC-1       demonstrated as ARC-35 measures it, within ARC-33's static-linking boundary
                       (installing = a directory, two lines in systems/installed, a rebuild; without
                       a rebuild is ARC-8, outside MVP-0) — ACCEPTED by the operator 2026-10-07 (QS-65),
                       who ran the checklist on main: all passing
            AC-2       confirmed for the six market packs at world level, beside S6's movement and
                       S8's social packs
            CP-4       the market lives 300 days, as a committed test (market_town.rs)
            Milest. C  demonstrated, awaiting the operator's review
          Known limits carried forward (step-10 §4.6.6):
            F-47       a System Pack owns one section, so a shop is authored on its operator (ARC-38);
                       lifting it is framework work for a later step
            F-48       an offer carries one unavailability reason: out of stock, cannot pay and
                       cannot carry all read TargetUnavailable
            F-41       item kinds and organizations have no names; a listing shows ids (S12)
            L-12       people walk ~8 m per in-world hour headless; routines are authored in ≥ 4 h parts
            L-13       a bounded-horizon economy: ten of twelve people live on endowments sized for
                       300 days
            Rel. sat.  relationships never decay; the social graph saturates after ~60 days
            QS-10      `sleep` and needs are still open (MVP §5); `eat` and `drink` closed by
                       consumption
            DP-5       shift-started.present is false whenever routine and shift begin at the same
                       minute — Market Town's content (Alice, 05:30), for a later step
            F-58       persistence's kill_and_resume test links the market packs through worldpack
                       and the installed set; check 2 reads normal and build edges (QS-54)
            S13        CI must check out with `fetch-depth: 0`, or the AC-1 test fails (by design,
                       never a skip)

  S15  Bodies and physical interaction — five PRs (step-11, DESIGN FROZEN 2026-10-07, revision 1;
       operator decisions QB-1 resolve before recording, QB-10 nudging, QB-2 withdrawn, QB-15 F1
       registration at start-up; QB-16 decided and tested in 12b; QB-17 deferred; the remaining
       QBs as recommended). Placed in §3 by step-11-bodies §12.1:
       12a  the arrival-resolver seam               PR 12a (GitHub #56), merged 03f1d7c
            A framework precursor that names no physics (ARC-39). presence asks every registered
            `ArrivalResolver` what an arrival achieves before it records it, and records only that:
            the walker's `arrived`, one `arrived` per person displaced, and `stopped-short`. movement
            states through `arrivals()` and names no resolver. The installed set gained an optional
            `resolution:` line, still `[]`, which `worldpack::compose` registers once per process.
            presence is now v3, so a save from before 12a is refused by name. Proven with synthetic
            resolvers in `tests/acceptance` only. Reviewed before merge:
            - gates re-run on the PR head (542/0), `resolver-yard`, `cafe` and `clock` PASS;
            - the diff touched no kernel, contracts, server, persistence, cognition or worlds path;
            - my own mutation, `stopped-short` suppressed while the shortened arrival was kept,
              failed four acceptance tests (RS-3, RS-4, RS-5, RS-6); reverted;
            - after the merge, `ac1_composability` 13/13 and `precursor_vocabulary` 4/4 on main.
            Both 300-day seed-7 digests are unchanged (I-7). DR-3 accepted: one 988-line test support
            file. DR-4 accepted: three pre-existing words are allow-listed in `seam_vocabulary.rs`.
            Follow-up FU-12a-1: the next PR allowed to edit movement's `action.rs` and worldpack's
            `read.rs` rewords those comments and removes the entries.
       12b  people: walls and nudging (DEP-13, `systems/bodies` on Rapier) — being detailed
            (step-11 §17)
       12c  objects: push, kick, throw; shove
       12d  the town gets bodies (digests re-baselined here, and only here)
       12e  the 3D client (Jolt, DEP-14), after the visual slice (now on main as #50)
       Each PR is detailed to the commit and frozen in turn. 12a merged; 12b–12e not frozen.

Next, framework:  S15 12b, people: walls and nudging — detailed to the commit in step-11 §17, then
                  frozen by the operator and implemented in a fresh session. S15 precedes S14
                  (step-11 §12).
Next, operator:   AC-1, Milestone B and Milestone C ACCEPTED 2026-10-07 (the operator ran
                  milestone_b, ac1_composability, milestone_c, market_town and a 30-day run on
                  main, all passing). VIS-3D-GODOT-2 accepted after the operator played it. Nothing pending with the operator.

Remaining:  S15 (12b–12e), S10 (reduced), S11 (authentication, admin frames, deltas), S12, S13,
            S14; Milestones D and E

Visual track (parallel, never blocking the above; ARC-20):
  VIS-2D-1         town accepted as default style (ARC-14); milestone not yet packaged
  VIS-3D-GODOT-1   route D+ ACCEPTED by the operator as the interim standard, 2026-10-07 —
                   not a final acceptance; refinement continues later. Fixed afterwards: the
                   hand and pack clipping, and the head sway while running. Before it, the
                   CharMorph candidate FAILED (2026-10-06) and is kept as pipeline validation
                   only (VISUAL_FIDELITY §9.2).
  VIS-3D-GODOT-2   ACCEPTED by the operator, 2026-10-07, after playing the combined session
                   (standalone and connected, with the character): every point passed.
                   Both are on main as of #50 (squash, 690c9d0).
  VIS-3D-UE5-1     parked (ARC-21)

Toolchain:  rust 1.97.1, Godot 4.7.2, Blender 5.2.2, Python 3.14.7
```

## Why the framework track stopped, recorded so it does not recur

Between 05d's merge and 2026-09-29 no framework work was scheduled. Every PR in that window
(GitHub #6, #8–#18) was visual or process documentation, although the operator's direction was
explicit that Milestones A–E continue and that the visual track never blocks them. Nothing was
waiting on the operator: S4 was frozen and ready. The cause was the primary session directing
every agent at the visual track.

**Standing rule from here:** while any framework step is unblocked, at least one agent is
working on it. The visual track runs beside it, never instead of it.

**Post-merge rule, restated:** after every merge, update the PR document, its step, and this
section before starting the next PR on that track.

## Governance and merge order (operator, 2026-09-25)

Project-level rules and revisions to frozen specifications merge into `main` **independently of
implementation work**, and as soon as they are agreed:

Since 2026-09-26 this happens through pull requests against a protected `main` (`D-12`); the
shape below is unchanged, only the mechanism that enforces it.

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
- Remote PR creation is now part of the flow (`D-12`), and `main` is protected, so integration
  happens by opening a PR rather than by a local merge.
- Reviewing and approving a design remains this session's authority; **merging a PR into a
  protected `main` remains the operator's**. Every other authority boundary
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
