# Step 07 / PR 08 — First real systems: places and movement (S6)

**Role:** combined step and PR document. S6 needs one PR.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 S6, §7 · **Lifecycle:** `DESIGN FROZEN`
(2026-09-30, primary session; review and answers recorded in §10.1)
**Base:** `main @ a594164` (S5 merged as `41d4ab1`; `a594164` is the docs-only post-merge update)
**Branch / worktree:** `mvp0/pr-08-movement` in `/Users/yuema137/mineworld-worktrees/s6-movement`
(held by this session only; `vis-character` and `vis-environment` belong to other agents)
**Depends on:** S2's spatial contract (`Location`, `SpatialRequirement::evaluate`), S3's single-writer
registry and `disable`, S4's clock, S5's `PersistentWorld` and `ARC-25`, S5V's `PresenceSystem`,
`ConversationSystem`, server, Godot protocol demo and the `AC-13` / `AC-15` tests, `ARC-15` genesis

Binding: [`CLAUDE.md`](../../../CLAUDE.md) §4 rules 1, 4, 5, 14, 15 ·
[`docs/ENGINEERING_RULES.md`](../../../docs/ENGINEERING_RULES.md) §§4–9, 11–12, 19 ·
[`docs/CORE_CONCEPTS.md`](../../../docs/CORE_CONCEPTS.md) §§2, 6, 10, 11, 12, 13 ·
[`docs/MVP.md`](../../../docs/MVP.md) §§4–5, §9 `AC-2`, `AC-6`, `AC-12`, `AC-13`, `AC-15` ·
[`docs/NETWORKING.md`](../../../docs/NETWORKING.md) §4 ·
[`docs/DECISIONS.md`](../../../docs/DECISIONS.md) `ARC-15`, `ARC-23`, `ARC-25` ·
[`step-04-clock-scheduler-process.md`](step-04-clock-scheduler-process.md) ·
[`step-06-persistence.md`](step-06-persistence.md)

**`DESIGN FROZEN`** at `4d6e42e` by the primary session (§10.1). The answers changed the plan in four
places — Q1's two clarifications, Q3 as a hard condition, Q4's client reporting rule, Q6's street —
and §4 was extended for them before C1 (the "Frozen-answer amendments" block at the head of §4).
Implementation context: this session, Phase 2 of the brief.

## Why this is PR 08, and why the file is `step-07`

The overall plan calls this step S6. PR numbers 01–07 are taken (PR 07 was S5), so it ships as **PR
08**. The step-document number follows the file sequence (`step-06-persistence.md` was S5), so this is
`step-07-movement.md`. Neither number changes what the step is.

---

# 1. Goal

A person in a MineWorld world **moves** — a client or a controller asks to go somewhere, and the
server decides. `move` is validated against authoritative spatial state, refused `TooFarAway` by the
movement system when the destination cannot be reached from where the person is, resolved into the
fact that changes where they are, reduced into occupancy, journaled, and rebuilt identically after a
restart. And the capability is a module: a world without the movement system answers `move` with
`ActionUnavailable` and changes in no other way — the first real evidence for `AC-2`.

The S6 checkpoint from `overall.md` §3 is the floor of this design:

```text
CP-1  move intent → validate → resolve → event → occupancy change, persisted and reloaded
CP-2  disabling the movement system makes `move` return ActionUnavailable with no change to any
      other module                                                        (first real AC-2 evidence)
CP-3  a movement refused for distance returns TooFarAway from the system, decided server-side
CP-4  MoveIntent, authoritative spatial state and rendered movement stay distinct, with room left
      for a travel Process that is not implemented                       (ENGINEERING_RULES.md §6)
```

## 1.1 Scope

```text
systems/movement/  (new crate)    mineworld-movement: MovementSystem; the `move` action; Passages
                                  (which places adjoin, and where their doorways are); the
                                  PassageOpened genesis fact; offers for perception; README
systems/presence/                 `arrive` retired (Q2); PersonEnteredPlace emitted when an arrival
                                  changes a person's place (Q5); version 2; tests re-based on a
                                  test-only relocating system; structural scan extended; README
kernel/src/{system,registry,error}.rs
                                  a system may emit an event type owned by another system only if it
                                  declares a dependency on that owner (Q1, Q3)
contracts/src/event.rs            Event::OWNER documented as the vocabulary owner, not the only
                                  possible emitter (doc only; Q1)
persistence/src/format.rs         SAVE_FORMAT 1 → 2, because SystemDeclaration's shape changes (Q3)
worldpack/src/catalog.rs          Capability::Movement
worlds/social-cafe/world.yaml     systems: presence, movement, conversation
tools/cli/tests/                  a `walk` helper replaces `arrive`; AC-15 and restart tests walk
persistence/tests/kill_and_resume.rs   the cafe scenario requests `move` instead of `arrive`
systems/conversation/tests/       setup by genesis; "walk closer, then talk" through MovementSystem
clients/protocol/                 the Godot demo walks with `move` in strides; evidence re-recorded
                                  by a real Godot run (ENGINEERING_RULES.md §19, R-9)
docs                              DECISIONS ARC-26; CORE_CONCEPTS §§11, 13; MVP_STATUS; READMEs;
                                  clients/protocol/ADOPTION.md; systems/README.md
Cargo.toml                        workspace member and dependency `systems/movement`
```

## 1.2 Non-goals

```text
a travel Process / TravelSystem          room is left (§2.4); none is built (ENGINEERING_RULES §6)
a speed model or a request-rate bound    the stride bounds one request, not requests per second (§2.5)
line of access / walls inside a place    DD-7: no layer owns geometry yet; not evaluated (§2.5)
doors that open and close, capacity      a later DoorSystem / InteriorSystem reads or replaces Passages
a World Pack `passages:` field           deferred unless Q6 says otherwise; S6 seeds passages through
                                         genesis in tests and the persistence checkpoint
a second place in worlds/social-cafe     same — Q6
pathfinding / navigation                 a client's acquisition problem; the server validates strides
runtime enable/disable as a journaled input   S5 non-goal, unchanged; CP-2 is shown in memory
save migration                           L-1 of S5: old saves are refused by name, not migrated (Q9)
the 3D spike client (clients/3d-spike)   visual track; not connected to the server; untouched
```

## 1.3 Frozen invariants (proposed)

- **I-1 One spatial truth.** `Presence` (owned by `PresenceSystem`) remains the only authoritative
  record of where a person is, and it changes only by `PresenceSystem` reducing an `Arrived` fact.
  No second component, cache or system holds a person's location.
- **I-2 One movement path.** After this PR there is exactly one runtime action that changes where a
  person is — `move` — and it is provided by `MovementSystem`. `arrive` no longer exists as an action
  (subject to Q2). World assembly places people by genesis (`ARC-15`), as today.
- **I-3 The server decides.** Whether a move is possible is decided only in
  `MovementSystem::validate`, against `Presence` and `Passages`, through
  `SpatialRequirement::evaluate`. No client evaluates it; a client's `actor_location` is still a report
  the server ignores (`ENGINEERING_RULES.md` §8, `INV-9`).
- **I-4 Dependency direction.** `movement → presence → kernel/contracts`, one way. `presence` and
  `conversation` sources name nothing of `movement` (structurally checked). Disabling `movement`
  requires no edit anywhere and leaves every other system's behaviour unchanged (`AC-2`).
- **I-5 Fixed point.** No `f32`/`f64` in any new or changed crate; distances are compared exactly
  (`CORE_CONCEPTS.md` §6.2 property 1).
- **I-6 No engine concept** in `move`, `Passages` or any fact: no mesh, navmesh, collider, camera,
  scene node, animation or physics (`ENGINEERING_RULES.md` §12).
- **I-7 One pipeline, persisted.** Every move is a journaled `dispatch`; a resumed world re-executes
  it and must reproduce its facts byte for byte (`ARC-25`). Nothing about movement lives in memory
  only.
- **I-8** The 311 pre-existing tests stay green in substance — `AC-15`, `AC-13`, the restart tests and
  the process-kill checkpoint included. Tests whose *request* was `arrive` are migrated to `move`
  with the same claim, not weakened; each migration is listed in the commit that makes it.

---

# 2. The question the brief asked first: where does movement belong, and what happens to `arrive`

## 2.1 What `arrive` is today, from source

`systems/presence/src/action.rs`: `Arrive { location: Location }`, `ACTION_TYPE "arrive"`, owner
`presence`, declared requirement `SpatialRequirement::NONE`. `PresenceSystem::validate` checks only
identity (actor is a living person, destination is a place); `resolve` emits `arrival(person,
location)` → `Arrived`; `react` reduces `Arrived` into `Presence` and the `present-in` edge. Its own
doc says what it is not:

> *"It is not movement. Movement is a `MoveIntent`, a travel `Process` that takes simulated time, and
> authoritative state that changes as it runs … `arrive` records an arrival that something else
> decided; a world that installs a movement system lets that system drive it, and this pack keeps
> owning the state either way."* (`action.rs:17-22`)

and the README's reason for having it at all — *"Without an action, though, location could not exist
at all"* — predates `ARC-15`: genesis now places every authored person (`worldpack/src/catalog.rs`
`located` → `arrival`), so that reason no longer holds.

**Every caller, audited** (`rg` over the tree, `main @ a594164`):

```text
systems/presence/tests/presence.rs              fixture moves people with `arrive` (13 tests)
systems/conversation/tests/conversation_and_presence.rs
                                                setup and "walk closer" with `Arrive`; asserts
                                                offered == ["arrive","talk"] and ["arrive"]
tools/cli/tests/support/mod.rs                  `arrive(actor, place, x, y)` wire helper
tools/cli/tests/ac15_one_alice.rs               6 call sites: visitor (4600,200)→(1200,1000),
                                                wanderer (4600,4400)→(2400,2400)
tools/cli/tests/restart.rs                      1 call site
tools/cli/tests/ac13_semantic_parity.rs         replays clients/protocol/evidence/request-{2d,3d}.json,
                                                whose first frames are `arrive` recorded from Godot
persistence/tests/kill_and_resume.rs            the cafe scenario: every third request an `arrive`
                                                to a random point in a 6 m square
clients/protocol/demo/demo.gd                   `_walk_beside` submits `arrive`; VERB "Walk to"
clients/protocol/evidence/*                     recorded frames and transcripts naming `arrive`
spike/unreal/, docs/references/UNREAL_ADAPTER_SPIKE.md
                                                historical spike evidence; records, not callers
server/, cognition/, worldpack/src              no reference: the server routes any action type
```

## 2.2 Why `arrive` and a new `move` cannot both stay

`arrive` requires nothing of space and checks only identity. Any client may submit it with any
destination. If `MovementSystem` adds `move` with a distance rule beside it, the rule is decorative:
the same client gets the refused position one request later through `arrive`. CP-3 — *a movement
refused for distance returns `TooFarAway` from the system* — would be true of `move` and false of the
world. That is the "second movement path" the brief forbids, and it is also an `INV-9` problem: the
server would be authoritative over a rule it lets anyone bypass.

Nor can `PresenceSystem` gate `arrive` on whether a movement system is installed: it would have to
name another pack's action, which is exactly what its structural test forbids (`presence.rs:604`) and
what `ENGINEERING_STANDARDS.md` §8 calls change amplification.

## 2.3 The options, judged against the kernel as merged

```text
(a) movement inside PresenceSystem       one system provides `move`, so disabling movement disables
                                         presence — and the registry refuses that while conversation
                                         depends on presence (registry.rs:332). CP-2 is unreachable.
(b) MovementSystem owns location         presence and conversation would depend on movement; removing
                                         movement removes location and perception. Inverts I-4.
(c) MovementSystem emits its own `Moved`;   presence would import movement's event type while
    PresenceSystem subscribes            movement imports presence's `Presence` to validate — a Cargo
                                         cycle, and presence naming another pack (fails its scan).
(d) MovementSystem keeps its own copy    two truths about one person's position; violates I-1.
    of positions
(e) MovementSystem decides; it states    movement → presence, one way. Presence keeps the only
    presence's own `Arrived` fact,       state, reduces the fact exactly as it reduces a genesis
    built by presence's `arrival()`;     arrival, and never learns movement exists. Disabling
    PresenceSystem reduces it            movement removes `move` and nothing else.
```

**Proposed: (e), with `arrive` retired.** It is the only option that satisfies CP-2, I-1 and I-4 at
once, and it is the shape `action.rs` already anticipated (*"a world that installs a movement system
lets that system drive it, and this pack keeps owning the state"*).

It is not free, and the cost is why §2.7 is material. `contracts/src/event.rs` documents
`Event::OWNER` as *"the system that emits it"* and says *"a system cannot emit another system's events
by accident"*. The kernel does **not** enforce single emission — `Dispatcher::record`
(`kernel/src/dispatch.rs:672-705`) checks only that the *running* system declared the type, and
`Emission::owner` is consulted only by genesis — so (e) works today without a kernel change. But it
works by reading a contract against its own text, and a reading of that kind is a decision to record,
not a loophole to use (`CLAUDE.md` §2.1 rule 4). §2.7 states it.

## 2.4 Split by responsibility and by scale

```text
                    PresenceSystem  (owns where people are)          MovementSystem  (decides walking)
state               Presence, present-in                             Passages (on places)
facts it owns       Arrived, PersonEnteredPlace                      PassageOpened
facts it emits      Arrived only as genesis (attributed to its       Arrived (presence's, via arrival()),
                    owner); PersonEnteredPlace while reducing        PassageOpened (genesis only)
actions             none (arrive retired)                            move
depends on          —                                                presence
perception          observe(), as today                              offers `move`
```

The four things `ENGINEERING_RULES.md` §6 keeps distinct, mapped onto code:

```text
MoveIntent          an ActionIntent whose action is `Move { to: Location }` — a request, refusable.
                    Not a new contract type: "MoveIntent" is the spec's name for an ActionIntent
                    carrying `move`, exactly as "InteractIntent" in §8 is (no synonym introduced).
Travel Process      NOT built. Room left: a future TravelSystem (its own pack, depending on presence)
                    starts a Process, and at its wake states presence's `Arrived` through the same
                    arrival() under the same rule as movement (ARC-26). `move` refuses a destination
                    in a non-adjoining place with TooFarAway, which is precisely the request a travel
                    action would accept. Nothing in this PR has to change for that.
Spatial state       Presence (+ present-in), owned by presence, written only by reducing Arrived.
Rendered movement   a client's interpolation between positions it was shown; in no contract. The
                    Godot demo animates its own walk and submits strides; the server never sees
                    frames, velocities or animation.
```

Two scales of one model: a **stride** inside a place and **through a passage** to an adjoining place
is local movement, decided per request by `move`; **travel** between places that do not adjoin takes
simulated time and is a Process. Both end in the same fact, `Arrived`, reduced by the same owner — so
"walking across a room and travelling between towns are different scales of the same spatial model"
holds in the log, not only in the prose.

## 2.5 What `move` checks, and why `TooFarAway` is the system's answer

`Move { to: Location }`. `MovementSystem::validate`, in order, writing nothing (`BD-6`):

```text
1  payload readable as Move                          else System(malformed-payload)
2  actor exists, is a Person, not Destroyed          else NoSupportedInteraction / PreconditionFailed
3  to.place() exists and is a Place                  else PreconditionFailed
4  actor has a Presence (somewhere to move FROM)     else PreconditionFailed — placement is genesis
5a same place:     stride_requirement().evaluate(&from, Some(&to), true)
                     = SpatialRequirement::same_place().within(MAX_STRIDE)
                   → Ok, or TooFarAway when |to − from| > MAX_STRIDE
5b another place:  the passage from from.place() to to.place() in from.place()'s Passages
                     none                            → TooFarAway (not adjoining: that is travel)
                     doorway `here` known            → evaluate(from vs doorway-here)   TooFarAway?
                     doorway `there` known           → evaluate(to vs doorway-there)    TooFarAway?
```

There is **no distance arithmetic in this crate**: every comparison is the one contract evaluator,
`SpatialRequirement::evaluate`, with the destination (or a doorway) in the target position — the
same discipline `ConversationSystem::validate` already follows (`systems/conversation/src/system.rs`
"there is no distance arithmetic in this crate"). The degeneracies are therefore the contract's, not
new ones: in a world that models no continuous position, a stride degenerates to *same place*
(`CORE_CONCEPTS.md` §6.3), so a headless or semantic 2D world moves freely within a place and through
passages, and is refused only for a non-adjoining place.

**`MAX_STRIDE = 2 000 mm`** (Q4). What it is: the furthest one request may carry a person from their
authoritative position. A walking or running embodied client reporting at 3 Hz or more never exceeds
it; crossing the café's 4.6 m takes at least three requests; a client that reports a 5 m jump is
refused. What it is **not**: a speed limit. It bounds one request, not requests per second — a
speed model needs per-person time accounting at a finer grain than `WorldTime`'s one second, and is
recorded as a limitation (L-1), not approximated. Walls inside a place are not evaluated either
(`DD-7`: line of access needs geometry no layer owns), so a stride may pass through a table (L-2).

**Rejection answers on a refused move are TooFarAway, PreconditionFailed, NoSupportedInteraction or
the pack's own `malformed-payload`** — all from `MovementSystem`, all server-side, none computed by a
client. A client that receives `TooFarAway` snaps its predicted position back to the authoritative
one it is shown in the next observation; that reconciliation is presentation.

`resolve` emits exactly one fact: `arrival(person, to)` — presence's own constructor, so there is no
second codec for presence's payload (`event.rs` on `arrival`: *"a loader that built these bytes would
be a second implementation of this pack's codec"*). `MovementSystem` writes nothing in `resolve`.

## 2.6 Entering a place: Passages

Occupancy (CP-1: *"occupancy change"*) is the `present-in` edge, and it changes only when a person
changes place. A world in which nobody can change place has no occupancy change to show. So `move`
needs to know which places adjoin — without that, crossing a threshold is either impossible or a
teleport to any place by name, and §6 forbids both.

```text
Passages        component on a Place, owned by MovementSystem
                  leads_to: one Passage per adjoining place, ordered by PlaceId
Passage           to: PlaceId
                  here:  Option<LocalPosition>   the doorway, in this place's frame
                  there: Option<LocalPosition>   the same doorway, in the other place's frame
PassageOpened   fact owned and emitted by MovementSystem, stated at genesis through
                passage(a, a_at, b, b_at); reduced by MovementSystem into BOTH places' Passages
```

A doorway position on each side is what makes entering a place a walk through a door rather than a
jump: the person must be within a stride of the doorway on this side and land within a stride of it
on the other. Both positions are optional for the reason `Location`'s refinement is: a semantic world
says only *the café opens onto the street*.

Ownership is MovementSystem's because the passage is a fact about where one can *walk*; a later
`DoorSystem` that opens and closes doors, or an `InteriorSystem` that owns topology, either reads it
or replaces it with its own (Q6). Passages are stated only at genesis in S6; no runtime action opens
one.

`PersonEnteredPlace { person, place, from: PlaceId }` (Q5) is emitted by **PresenceSystem** while
reducing an `Arrived` that changes a person's place from a known previous place: occupancy is
presence's state, so the occupancy-change fact is presence's, whoever caused the arrival — movement
now, travel later. It is not emitted for a first placement (genesis) or a move within a place, so
every existing fact sequence in the repository is unchanged by it (one-place worlds only).
Visibility `Place(place)`, subject and participant `person`, at `place`.

## 2.7 Material findings, raised rather than routed around

**MF-1 — an event type spoken by a system other than its owner (Q1, Q3).** Option (e) has
`MovementSystem` state `Arrived`, whose `Event::OWNER` is `presence`. The kernel permits it; the
contract text says otherwise. Proposed resolution, recorded as **`ARC-26`** before any code:

```text
Event::OWNER is the event type's VOCABULARY OWNER: the system that defines its schema and its
public constructor and, for a fact describing state, reduces it into the state it owns.
Another system may state a fact of that type only if
  1  it declares .emitting::<E>()                         (already enforced, dispatch.rs:680)
  2  it declares a dependency on E::OWNER                 (NEW — enforced at install, Q3)
  3  it builds the payload through the owner's public constructor (convention + review; there is no
     way for the kernel to see a codec)
Provenance names the system that stated the fact; Event::OWNER names whose vocabulary it is. They
differ exactly when another system decided what the owner records.
```

This changes no data flow and no existing behaviour; it changes what one contract *means*, and
`CLAUDE.md` §3.1 makes that the operator's / primary session's call. The rejected alternatives are
§2.3 (a)–(d).

**MF-2 — retiring `arrive`, a public action on the wire (Q2).** §2.2: keeping it makes CP-3 hollow.
Retiring it changes the client protocol's vocabulary (not its frame shape): the Godot demo, its
recorded `AC-13` evidence, and five Rust test files submit it today (§2.1). The migration is planned
commit by commit (§4) so that every commit stays green and the frozen Godot evidence is re-recorded
from a real Godot run rather than edited by hand (`clients/protocol/evidence/README.md`: *"a change to
them is a claim that a client now sends something else, and it belongs with the run that produced
it"*). The fallback if Q2 is refused — keep `arrive` and document it as an unrestricted placement
bypass — is listed so the decision is a choice, not recommended.

## 2.8 The two gate questions (`ENGINEERING_RULES.md` §§11–12, `overall.md` §2 item 8)

**§11 — Can a Minecraft-like embodied 3D client use this without redesigning the kernel? Yes.** The
client walks its own body with local physics and collision (`NETWORKING.md` §4: *"Local physics …
never authoritative"*) and reports where it got to as `move` strides of at most 2 m, with `facing` —
orientation travels in `Location` already. The server answers each; the client reconciles on
`TooFarAway` to the position it is shown. Entering a place is walking through a doorway, which the
passage model checks on both sides — no single-room assumption, no tile grid, no click-only
interaction, no instant cross-room jump. What the 3D client cannot yet get from the server is
collision against walls (L-2) and a speed limit (L-1); both are additions to `MovementSystem`'s
validation (a geometry provider, a gait component), not kernel changes. The kernel change this PR
does make (Q3) is generic — it names no spatial concept.

The one place the answer is weaker than it looks: event volume. A 3D client reporting at 5 Hz writes
5 `Arrived` facts per second per walking player into the journal and the log. That is correct and
replayable; whether it is affordable at Demo B scale is measured there, and the remedy (report on
stop and every N strides, or a coalescing movement Process) changes no contract here (R-1 below).

**§12 — Does anything engine-shaped enter a generic contract? No.** `Move`, `Passage`,
`PassageOpened` and `PersonEnteredPlace` carry `PlaceId`, `Location`, `LocalPosition` and
`Millimetres` only. No mesh, navmesh, collider, camera, scene node, animation, physics. The headless
world and a 2D client are complete implementations: a semantic world moves through places with no
positions at all (§2.5 degeneracy).

**§9 / §22 — Can 2D and 3D share it without duplicating game logic? Yes.** Both clients submit the
same `move` action with the same payload shape; the only difference is acquisition — the 2D demo
interpolates a click into strides, the 3D client reports where its body walked. Neither client
decides acceptance: a client that chops a path into strides is choosing efficient requests, and a
client that does not is refused by the server, not by itself. The one shared number a client benefits
from knowing, `MAX_STRIDE`, is published in the movement contract and `ADOPTION.md`, and is a request
size, not a rule a client enforces (Q4 asks whether to also carry it in the affordance).

---

# 3. Design decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| **MD-1** | New System Pack `mineworld-movement` at `systems/movement/`, `MovementSystem` (`SystemId "movement"`), depending on `presence`. | §2.3 (e). A separate system is the only way CP-2 holds while conversation depends on presence. |
| **MD-2** | `move` is the one runtime action that relocates a person; `arrive` is retired from `PresenceSystem` (Q2). | §2.2. A second, unrestricted path makes the server's distance decision bypassable. |
| **MD-3** | `MovementSystem` emits presence's `Arrived` through `mineworld_presence::arrival`; `PresenceSystem` stays the only writer of `Presence` and `present-in`. | I-1, I-4. Recorded as `ARC-26` (Q1). |
| **MD-4** | Kernel: `SystemDeclaration::emitting::<E>()` records `E::OWNER`; installation refuses a declared emission whose owner is neither the system itself nor one of its declared dependencies (`KernelError::EmittedEventOwnerNotADependency`). `SAVE_FORMAT` 1 → 2 because the declaration is part of a save's composition. | `overall.md` §2 item 1 prefers a runtime check to a convention. Without it, a system could speak a vocabulary whose owner is absent and nothing would reduce it (Q3). |
| **MD-5** | Every distance in movement is decided by `SpatialRequirement::evaluate` with the destination or a doorway as the target; `MAX_STRIDE = 2 000 mm`, a constant in S6. | One evaluator for every system (`CORE_CONCEPTS.md` §6.3); value and configurability are Q4. |
| **MD-6** | `Passages` on places, owned by movement, seeded by the `PassageOpened` genesis fact; a cross-place move goes through a passage, checked on both sides of the doorway. | §2.6. Without adjacency there is no occupancy change, or there is teleportation. |
| **MD-7** | `PersonEnteredPlace` is presence's fact, emitted while reducing an `Arrived` that changes a known place. | Occupancy is presence's state; one owner states its changes for every cause (Q5). Deviation from `overall.md` S6's wording, which attached it to `MovementSystem`. |
| **MD-8** | `PresenceSystem::VERSION` 1 → 2 (no longer provides `arrive`; emits `PersonEnteredPlace`). `MovementSystem::VERSION` 1. | A declaration change is a version change; S5 refuses mismatched saves by name. |
| **MD-9** | `move` is offered once per observer with no target, with requirement `SpatialRequirement::NONE` documented as "no requirement on a *target*"; the stride is published as a contract constant (Q4). | `verdict` (`observe.rs`) evaluates an offer's requirement against a target; a targetless offer with a target requirement would always show `PreconditionFailed`. |
| **MD-10** | `worlds/social-cafe` installs `presence, movement, conversation`; nothing else in the pack changes (no second place unless Q6). | Players must still be able to walk to Alice. Registration order is reduction order; movement after presence. |
| **MD-11** | The Godot demo walks in strides of at most `MAX_STRIDE` and its `AC-13` evidence is re-recorded by `clients/protocol/run.sh evidence` against the new server. | §19: client work is validated by running the client; frozen evidence is regenerated, never edited. |
| **MD-12** | CP-1's persisted path is proven with `PersistentWorld` + `SqliteBackend` on a two-place world built by genesis in `systems/movement/tests`; the real-pack process-kill checkpoint keeps running with `move` in place of `arrive`. | S5 already proved restart mechanics; this proves *movement* survives them, including a passage crossing and a refused stride. |

---

# 4. Commit plan

Each commit tracks implementation, validation and review separately; evidence goes to §9. Every commit
compiles and passes the full workspace suite on its own — the order below exists so that `arrive`
is retired only after every caller has moved off it.

### Frozen-answer amendments (recorded at the start of execution, from §10.1)

The freeze answered four questions in ways the drafted commits did not yet carry. Each is placed
below, and none widens scope beyond what §10.1 instructs:

```text
Q1.1  OWNER = the vocabulary owner and the ONLY reducer of the fact into owned state
                                                       → ARC-26 text (C1); Event::OWNER doc (C2)
Q1.2  the owner still decides: presence can refuse an Arrived that would make its state invalid
                                                       → new commit C2b: mineworld_presence::admit,
                                                         arrival() checked against the world, and a
                                                         reduction that refuses rather than writes;
                                                         kernel KernelError::FactRefusedByOwner (C2)
Q3    kernel enforcement is a hard condition           → C2 is no longer conditional
Q4    client reporting rule: report before travelling MAX_STRIDE since the last accepted position
                                                       → PROTOCOL.md + ADOPTION.md (C1); a test of a
                                                         rule-following and a rule-ignoring jogging
                                                         client (C3); the Godot demo follows it (C5)
Q6    a street outside the café, joined by a doorway   → new commit C4b: the World Pack `passages`
                                                         field + MODULE_SPEC §4.1 + PACKAGE_FORMAT §8
                                                         in one commit; the street itself lands in
                                                         worlds/social-cafe with C5 (see below)
```

**Why the street lands with C5 and not C4b (bounded sequencing decision).** A new place shifts every
person's `EntityId` — places are created before people (`worldpack/src/load.rs` `assemble`) — so the
visitor seat becomes entity 5, not 4. `tools/cli/tests/ac13_semantic_parity.rs` replays frozen Godot
frames naming actor `"4"` and asserts it. Adding the street in C4b would leave a red commit until C5
re-recorded the frames, and re-recording in C5 against a one-place world would have to be redone
after. So C4b delivers the format and the loader (proven on a test pack), and C5 adds the street and
records the evidence against the world as it will be. **Field-spec location:** the fields a World Pack
may use are specified in `docs/MODULE_SPEC.md` §4.1 (`worldpack/src/format.rs` cites it), so C4b
updates §4.1 as the authority and `docs/PACKAGE_FORMAT.md` §8 as the brief instructs.

## C0 — Design (this document) — docs only

- [x] Implementation: audit (§8), design (§§1–7), proposed execution contract (§11).
- [x] Validation: baseline `cargo test --workspace --no-fail-fast` on `main @ a594164` → 311 passed,
  0 failed, `kill_and_resume` cafe PASS, clock PASS (E-0). `check_decision_ids.py` → 34 ids distinct;
  `check_doc_headings.py` → 134 sections, none duplicated (E-0).
- [x] Review: primary-session review recorded in §10.1; `DESIGN FROZEN` at `4d6e42e`.

## C1 — Specification amendments, before code

**Goal.** The decisions this step makes exist in the specifications before code depends on them
(`CLAUDE.md` §2.2, §2.1 rule 4). **Depends on:** C0 frozen.

- [ ] Implementation:
  - `docs/DECISIONS.md`: new **`ARC-26`** — movement and presence split; vocabulary owner vs emitter
    (§2.7 MF-1); `arrive` retired (MF-2); `PersonEnteredPlace` owned by presence; passages; the stride
    and its two stated non-properties (speed, walls); alternatives (a)–(d) rejected with reasons.
  - `docs/CORE_CONCEPTS.md` §11 (an Event type has one vocabulary owner; another system may state it
    under `ARC-26`'s three conditions; provenance names who stated it) and §13 (emitted events may
    include a dependency's vocabulary).
  - `clients/protocol/ADOPTION.md`: "moving is submitting `move`, in strides of at most 2 m; a refused
    stride is answered `too_far_away`; reconcile to the position you are shown".
  - **Amended at freeze (Q4):** the client reporting rule — *report before travelling `MAX_STRIDE`
    since the last accepted position* — in `server/PROTOCOL.md` and `clients/protocol/ADOPTION.md`,
    with the jog-speed example that motivates it.
  - Handoff reinitialized for PR 08.
- [x] Implementation, as planned: `ARC-26` (with §10.1's two clarifications, the reporting rule and the
  limitations); `CORE_CONCEPTS.md` §11 (an EventType's owner; the three conditions; the owner still
  decides) and §13 (when the requester states the owner's vocabulary); `server/PROTOCOL.md` new §6.2
  (the `move` frame, what the server decides, the reporting rule); `clients/protocol/ADOPTION.md` new
  §4.1; handoff reinitialized for PR 08. `ARC-26` was checked free on every remote branch before use.
- [x] Validation: `check_decision_ids.py` → 35 decision ids, all distinct; `check_doc_headings.py` →
  134 numbered sections across 21 documents, none duplicated. PASS.
- [x] Review: the term used is the EventType's **owner**, already the contract's word (`Event::OWNER`),
  now defined as vocabulary + sole reducer rather than coined anew — no synonym introduced. "MoveIntent"
  is not introduced as a type anywhere; the documents say `move` request / action. The rule lives in
  `ARC-26`, `CORE_CONCEPTS` and `PROTOCOL.md`; `ADOPTION.md` restates it for client authors and links
  `PROTOCOL.md` §6.2. Bounded note: `PROTOCOL.md` describes `move` before C3 implements it, by design
  (specification leads code, `CLAUDE.md` §2.2).
- **Commit boundary:** docs only. Note: the parent `overall.md` S6 text (which names
  `PersonEnteredPlace` under `MovementSystem`) is the planning session's to reword; this commit does not
  edit it unless the freeze instructs so (as S5's freeze did).

## C2 — Kernel: a foreign emission requires a dependency on its owner

**Goal.** MF-1's rule is a check, not a convention (MD-4). **Depends on:** C1. Applies only if Q3 is
accepted; otherwise this commit becomes a review convention in `.structured-coding/standards.md` and
`ARC-26`, and is marked `N/A` with that reason.

- [ ] Implementation:
  - `kernel/src/system.rs`: `SystemDeclaration` records the owner of each emitted event type
    (`emitting::<E>()` reads `E::OWNER`); `emits()` keeps returning event types; a new accessor for the
    owners. Exact field shape settled in implementation and recorded.
  - `kernel/src/registry.rs` `check_installable`: a fifth refusal — an emitted type whose owner is
    neither the declaring system nor in its `depends_on`. Same refusal on `enable` is unnecessary
    (dependencies are already re-checked there); stated in the doc comment.
  - `kernel/src/error.rs`: `EmittedEventOwnerNotADependency { system, event_type, owner }`.
  - `contracts/src/event.rs`: `Event::OWNER` doc — vocabulary owner, `ARC-26` (doc only).
  - `persistence/src/format.rs`: `SAVE_FORMAT` 2, with the reason.
- [ ] Validation (`kernel/tests/composition.rs` or `two_systems.rs`, chosen by where the existing
  install refusals are tested):
  - a system emitting a type owned by a system it does not depend on is refused at install, naming
    all three; the world is unchanged (snapshot bytes before == after);
  - the same system declaring the dependency installs, and its emission is recorded with
    `Provenance::emitted_by` = the emitter while the fact's type is the owner's — the property the
    whole of MF-1 rests on, asserted on a recorded envelope;
  - an own-vocabulary emission is unaffected (every existing test passes);
  - a save written with format 1 is refused by name (`persistence` refusal test extended).
- [ ] Review: no domain term in the kernel change (`INV-12`); declaration remains a value derived from
  types (no string-typed owner a system could misstate); `SAVE_FORMAT` bump is the only persistence
  change.

**C2 as built.**
- [x] Implementation. Field shape: `SystemDeclaration.emits_owned_by_others: Vec<(EventTypeId,
  SystemId)>`, filled by `emitting::<E>()` only when `E::OWNER` differs from the declaring system, with
  accessor `emits_owned_by_others()`; `emits()` unchanged. `SystemRegistry::check_emitted_vocabularies`
  runs after the dependency check. Also in this commit, ahead of C2b which uses it:
  `KernelError::FactRefusedByOwner { system, event_type, reason: Rejection }`. Tests in a new file,
  `kernel/tests/borrowed_vocabulary.rs` (neither `composition.rs` nor `two_systems.rs` had an
  emitter/owner pair to reuse). Process deviation: the six source files were committed by the primary
  session as `wip` `7eb7cd0` after this session stalled mid-commit; that commit is squashed into this
  semantic one after verification.
- [x] Validation. `7eb7cd0` as recovered (untested when committed): `cargo test --workspace
  --no-fail-fast` rc 0, **311 passed, 0 failed**, `kill_and_resume` cafe PASS, clock PASS — so the
  `SAVE_FORMAT` bump and the declaration change break no existing test. Then
  `kernel/tests/borrowed_vocabulary.rs`: 2 passed — refusal names `undeclared-clerk` / `counted` /
  `ledger` and the world's snapshot bytes are unchanged; the accepted case records exactly one fact
  (counted before reading it) whose type is `counted` and whose provenance is `clerk`, and the tally is
  written by `ledger`. `persistence/tests/save.rs` format refusal now asserts format 3 → `TooNew
  {3, 2}` and S5's format 1 → `Outdated {1, 2}`. fmt, clippy `-D warnings` clean.
  **Mutation evidence NOT obtained:** a run of the refusal test with the registry check disabled was
  attempted and refused by the session's permission classifier; the mutation was reverted at once
  (`git diff` confirms). The refusal test asserts the exact error value, so it cannot pass without
  the check returning that error; recorded as reasoning, not as a mutation result.
  **Terminal gates on the committed C2 head `47874ca`** (resumed session, 2026-10-01, clean tree):
  `cargo test --workspace --no-fail-fast` rc 0, 51 harness results summing to **313 passed, 0
  failed** (311 + the 2 in `borrowed_vocabulary.rs`); `cargo test -p mineworld-persistence --test
  kill_and_resume` → `[cafe] PASS in 0.3 s`, `[clock] PASS in 0.4 s`; `check_decision_ids.py` → 35
  ids distinct; `check_doc_headings.py` → 134 sections, none duplicated. PASS (E-2).
- [x] Review: the kernel change names no domain concept; the owner is read off `E::OWNER`, never passed
  as a value; `enable` needs no repeat (a declaration is fixed at install, and `enable` re-checks
  dependencies). `SAVE_FORMAT` is the only persistence change.

## C2b — Presence: the owner still decides (Q1.2)

**Goal.** Presence can refuse an `Arrived` that would make its state invalid, at the constructor and
at reduction, so that "movement states the fact, presence reduces it" never lets another system
choose an invalid value for presence's state. **Depends on:** C2.

- [ ] Implementation:
  - `systems/presence/src/event.rs`: `pub fn admit(world: &WorldRead, person: PersonId, location:
    Location) -> Result<(), Rejection>` — the person exists, is a Person and is not Destroyed; the
    place exists and is a Place. `arrival(world, person, location) -> Result<Emission, Rejection>`
    calls it before building the fact; there is no unchecked public constructor.
  - `systems/presence/src/system.rs` `react`: re-checks `admit` against the world as it is when the
    fact is reduced and, on refusal, writes nothing and returns
    `KernelError::FactRefusedByOwner { system, event_type, reason }` (C2) — the emitter broke the
    contract, and the world says so instead of taking the value. `validate` uses `admit` too.
  - `worldpack/src/{catalog,load}.rs`: `located` and `initial_facts` read the assembled world, so a
    genesis placement goes through the same check.
- [ ] Validation (`systems/presence/tests/presence.rs`):
  - the constructor refuses a destroyed person and a place that is not a place
    (`PreconditionFailed`), and accepts the living case;
  - a test-only system that depends on presence and states an `Arrived` built **without** the
    constructor (raw `Emission::new::<Arrived>`) for a destroyed person → dispatch returns
    `Err(FactRefusedByOwner { system: presence, .. })` and `Presence` is unchanged (snapshot bytes);
    the same system stating a valid one is reduced — the positive control, so the refusal is shown to
    be the check and not a broken path.
- [ ] Review: presence is still the only writer of `Presence`; the reduction check reads only.

**C2b as built.**
- [x] Implementation. `systems/presence/src/event.rs`: `admit(&WorldRead, PersonId, Location) ->
  Result<(), Rejection>` (person and place each exist, are of their type and are not `Destroyed`);
  `arrival(&WorldRead, PersonId, Location) -> Result<Emission, Rejection>` calls it first. Both
  re-exported from `lib.rs`. `system.rs`: `validate` keeps its first three answers (malformed payload,
  missing actor → `PreconditionFailed`, non-person → `NoSupportedInteraction`) and then delegates to
  `admit`; `resolve` uses the checked `arrival`; `react` asks `admit` before writing and maps a
  refusal through a private `refused()` to `KernelError::FactRefusedByOwner { presence, arrived, .. }`.
  `worldpack/src/catalog.rs` `located(&WorldRead, ..) -> Result<Emission, KernelError>` (same error
  value, surfacing as `PackError::Composition`); `load.rs` `initial_facts` takes `world.read()` of the
  assembled world. **Bounded deviation:** `admit` also refuses a *destroyed place*, which the old
  `validate` did not check — §10.1 Q1.2 names "a dead entity" without restricting it to the person,
  and a presence in a destroyed place is the same invalid state. No existing test relied on it.
  `Arrived::new` stays public (the payload type is the vocabulary); the *fact* has no unchecked public
  constructor in presence, which is what the reduction check backs up.
- [x] Validation. `systems/presence/tests/presence.rs`, 2 new tests (13 total): the constructor refuses
  a person entity named as a place, an unallocated place id and a destroyed person (each
  `PreconditionFailed`) and builds the living case; a test-only `Mover` (depends on presence, states
  `arrived`) is reduced when it states a valid arrival raw and via `arrival()` (positive controls),
  and stating one raw for a destroyed person returns `Err(FactRefusedByOwner { presence, arrived,
  PreconditionFailed })`, bob has no `Presence`, and the snapshot's component rows and edges are
  byte-identical (the whole snapshot is not compared: recording advances the event counter before
  reduction refuses, which is the kernel's existing behaviour for any reduction error).
  **Mutation obtained:** with the `admit` line in `react` commented out, the refusal test FAILS
  (`dispatch returned Ok(Accepted { events: [EventId(3)] })`); restored, PASS. Workspace: rc 0, 51
  harness results, **315 passed, 0 failed**; `kill_and_resume` cafe PASS 0.3 s, clock PASS 0.4 s;
  clippy `-D warnings` clean; fmt clean.
- [x] Review: `react` reads through `world.read()` before any write and returns before the first
  write on refusal; presence remains the only writer of `Presence`/`present-in`. The kernel learns
  nothing new (the error variant landed in C2). Genesis goes through the same check, so an authored
  placement and a decided one cannot differ in what presence admits.

## C3 — `mineworld-movement`: the system, its facts, and CP-1/CP-2/CP-3 in-process and persisted

**Goal.** Movement exists and is proven against the floor, while `arrive` still exists so nothing
else changes yet. **Depends on:** C2.

- [ ] Implementation:
  - `Cargo.toml`: member `systems/movement`, workspace dependency `mineworld-movement`.
  - `systems/movement/Cargo.toml`: deps contracts, kernel, presence, serde, serde_json; dev-deps
    persistence, conversation (for CP-2's "other systems unchanged").
  - `src/lib.rs` (module table, the §2.4 split), `src/action.rs` (`Move { to }`, `MAX_STRIDE`,
    `stride_requirement()`), `src/component.rs` (`Passages`, `Passage`), `src/event.rs`
    (`PassageOpened`, `passage(..)` genesis constructor), `src/system.rs` (`MovementSystem`:
    declaration `depending_on([presence])`, `owning::<Passages>()`, `providing::<Move>()`,
    `emitting::<Arrived>()`, `emitting::<PassageOpened>()`, `subscribing_to::<PassageOpened>()`;
    `validate` per §2.5; `resolve` → `arrival`; `react` reduces `PassageOpened` into both places;
    `PerceptionProvider::offers`), `src/codec.rs` (this pack's own, per the presence pattern),
    `README.md`.
- [ ] Validation — `systems/movement/tests/movement.rs` (integration, real presence + conversation):
  - CP-1 in memory: a stride inside the café → accepted, one `Arrived` with provenance `movement`,
    `Presence` updated, the one `present-in` edge kept; through a passage café→street → accepted,
    `present-in` edge moved (located: the edge set before and after is printed and asserted, `ARC-23`);
  - CP-3 boundary with **literal** distances (`ARC-23` rule 2: the bound is never derived from the
    constant under test): 2 000 mm accepted, 2 001 mm → `Rejected(TooFarAway)` and no fact; a place
    with no passage → `TooFarAway`; through a passage from 2 001 mm off the doorway → `TooFarAway`;
    landing 2 001 mm off the far doorway → `TooFarAway`; each refusal changes nothing (world snapshot
    bytes equal);
  - negatives: no `Presence` → `PreconditionFailed`; destination not a place → `PreconditionFailed`;
    a place as actor → `NoSupportedInteraction`; unreadable payload → `System(malformed-payload)`;
  - semantic world (no local positions): within place accepted, adjoining place accepted,
    non-adjoining → `TooFarAway` — the degeneracy of §2.5 stated as behaviour;
  - CP-2: with movement disabled, `move` → `Unavailable` with no events; `talk` still answered exactly
    as in the same world with movement never installed (same results, same facts byte for byte);
    `Presence` unchanged; the `move` affordance disappears from `observe` and nothing else in the
    observation changes (compared against the never-installed world);
  - structural: no `f32`/`f64` in `systems/movement/src`; presence's and conversation's sources name
    no `movement`/`passage` vocabulary (a scan over their `src/`, owned here because the claim is
    about this pack's isolation).
  - **Q4 reporting rule (amended at freeze):** a model client jogging at a literal 2 600 mm/s along a
    straight line for a literal 20 s, sampled every 100 ms. The rule-following client reports whenever
    its next sample would put it more than 2 000 mm from its last *accepted* position → every report
    accepted, final `Presence` = its final reported position. The rule-ignoring client reports once
    per second (2 600 mm) → its first report refused `TooFarAway`, and with no accepted position to
    advance from, every later report too; `Presence` stays at the start. Both counts printed and
    asserted (`ARC-23`).
  - **`ARC-23` counterfactual for CP-2 (amended at freeze):** the removability test is run once as
    written and once against a deliberately broken world in which movement is "disabled" by a
    mechanism that leaves it reachable (the test's own negative control: the same assertions applied
    to a world where movement stays enabled must FAIL). The test owns both halves, so it cannot pass
    whether or not movement is unreachable.
- [ ] Validation — `systems/movement/tests/persisted.rs` (real lifecycle, real SQLite file):
  `PersistentWorld::create` a two-place world (genesis: places, people, one passage) → a stride, a
  refused stride, a passage crossing → drop → `SqliteBackend::open` + `PersistentWorld::resume` into a
  freshly composed world → `Presence`, `present-in` and `Passages` equal; `Resumed` reports the tail
  re-executed (located, not assumed); `verify` from genesis passes; one further move continues at the
  next `EventId`.
- [ ] Review: movement writes only `Passages`; `resolve` writes nothing; no distance arithmetic outside
  `SpatialRequirement::evaluate`; the payload of `Arrived` is built only by `arrival()`; README is short
  and links to the crate docs.

**C3 as built** — split into two commits for survivability: `41bb073` (the crate) and the tests commit.
- [x] Implementation. `systems/movement/src/{lib,action,component,event,codec,system}.rs`, `README.md`;
  workspace member + dependency. `Move { to }`, `MAX_STRIDE = 2 000 mm`, `stride_requirement()`,
  `move_offer_requirement() = NONE` (MD-9); `Passages { leads_to: Vec<Passage> }` kept sorted by
  `PlaceId` (`open` replaces a passage to the same place); `PassageOpened { a, a_at, b, b_at }` and the
  genesis constructor `passage(..)` (visibility `Public`, subjects both places, at `a`). `validate`
  follows §2.5 with one refinement: steps 2–3 call presence's `admit` (C2b), so a destroyed actor or a
  destroyed/non-place destination is `PreconditionFailed` from the owner's single check. `resolve`
  states `arrival(&read, person, to)`; a refusal there (validate and resolve disagreeing) is
  `FactRefusedByOwner { presence, arrived }`. `react` reduces `PassageOpened` into both places and,
  **owner decides** as in C2b, refuses a self-passage or a non-place end with `FactRefusedByOwner {
  movement, passage-opened }`. `offers`: `move` once, targetless, to a living person.
- [x] Validation. `systems/movement/tests/movement.rs` (7) + `persisted.rs` (1), fixture in
  `tests/support/mod.rs` (café, street joined at café (4600, 2000) ↔ street (0, 2000), attic with no
  passage):
  - CP-1: two strides and a crossing, each one `arrived` with provenance `movement`; `present-in`
    printed before `[EntityId(1)]` and after `[EntityId(2)]`.
  - CP-3: 6 literal cases, 4 refused `TooFarAway` (2 001 mm stride; 2 001 mm off the near doorway;
    landing 2 001 mm off the far doorway; the attic), 2 accepted at exactly 2 000 mm (stride; both
    doorway sides); each refusal leaves presence and the world's state bytes unchanged.
  - negatives: unplaced person, non-place destination → `PreconditionFailed`; a place as actor →
    `NoSupportedInteraction`; unreadable payload → `System(malformed-payload)`.
  - semantic world: within place and through passage accepted, attic `TooFarAway`.
  - **CP-2 with its negative control (`ARC-23`)**: `removability_violations` states the AC-2 claims once
    (move `Unavailable`, no fact, not offered; entities, Presence rows, edges, every fact, every answer
    and both observations equal to the never-installed world after the same script). Disabled → **0
    violations**. Enabled (movement reachable) → **7 violations** (`move was answered Accepted`, `move
    recorded 1 fact`, `move is still offered`, `presence/facts/results/observations differ`), and the
    test asserts four of them are present. So the test cannot pass whether or not movement is
    unreachable.
  - **Q4**: jog 2 600 mm/s × 20 s at 100 ms (literals). Rule-following client: **29 accepted, 0
    refused**, final Presence (52 000, 0). Rule-ignoring client (once a second): **0 accepted, 20
    refused**, Presence stays at the start.
  - structural: no `f32`/`f64` in movement's 6 sources; presence (8 files) and conversation (8 files)
    name none of `mineworld_movement`, `MovementSystem`, `Passage`, `passage`, `MAX_STRIDE`, `stride`.
    **Bounded deviation:** the scan is over identifiers, case-sensitive, not the lowercase word
    "movement" — presence's `action.rs` (deleted in C6) and conversation's `validate` doc say "a
    movement system" in general prose, which is not a dependency. C6 extends presence's own scan.
  - IC-1 (`persisted.rs`): real SQLite file; genesis 3 facts; walk answers `Accepted, TooFarAway,
    Accepted, Accepted`; head revision 5, 6 facts in the log; resume → snapshot revision 1, **replayed
    4, facts 3**; component rows + edges byte-identical; `verify` 5 revisions; the next move's fact is
    `EventId(7)`.
  - **Finding (test seam):** comparing whole snapshot bytes around a *first* refused request fails,
    because `WorldSnapshot.ran` flips on any dispatch, whatever the answer. The fixture compares every
    snapshot field except `ran` and says why; no production change.
  - **Mutation obtained:** `MAX_STRIDE` 2 000 → 2 001 turns the boundary test red (6 passed, 1 failed);
    restored.
  - Workspace: rc 0, 55 harness results, **323 passed, 0 failed**; `kill_and_resume` cafe/clock PASS;
    clippy `-D warnings` and fmt clean.
- [x] Review: movement writes only `Passages` (only `react` inserts); `resolve` writes nothing;
  `reachable` is the only spatial decision and it calls `SpatialRequirement::evaluate` three ways
  (destination, near doorway, far doorway) — no arithmetic; `arrived` is built only by presence's
  `arrival`. README is 25 lines and links the crate docs, `ARC-26` and `PROTOCOL.md` §6.2.

## C4 — The pack, the server and the Rust clients walk with `move`

**Goal.** The real world installs movement and every Rust caller that relocates a person does it with
`move`. `arrive` still exists (unused by these callers). **Depends on:** C3.

- [ ] Implementation:
  - `worldpack/src/catalog.rs`: `Capability::Movement` (id, install, provider); `AVAILABLE` gains it.
  - `worlds/social-cafe/world.yaml`: `systems: [presence, movement, conversation]`, comment updated.
  - `tools/cli/tests/support/mod.rs`: `walk(actor, place, from, to) -> Vec<Value>` — straight-line
    strides of at most 2 000 mm in integer arithmetic; `arrive` helper removed.
  - `tools/cli/tests/ac15_one_alice.rs`, `restart.rs`: `submit_accepted` for each stride; positions and
    claims unchanged.
  - `persistence/tests/kill_and_resume.rs`: the cafe scenario requests `move` (generator bounded so
    that both accepted and `TooFarAway` outcomes occur; both are journaled and replayed).
- [ ] Validation:
  - `AC-15` (6 tests) and the restart tests (2) pass unchanged in claim; the number of strides each walk
    takes is printed and asserted from the literal start/end positions (visitor 3 493 mm → 2 strides;
    wanderer 2 970 mm → 2 strides);
  - `kill_and_resume` cafe and clock PASS; the scenario's located counts of accepted moves and
    `TooFarAway` refusals are printed and both are non-zero (`ARC-23` rule 1);
  - `worldpack` tests: social-cafe composes with movement; existing genesis fact sequence unchanged.
- [ ] Review: no claim weakened in a migrated test (diff read line by line); the walk helper computes
  strides from the literal positions and the documented 2 000 mm, not from production code.

**C4 as built.**
- [x] Implementation. `worldpack/Cargo.toml` depends on `mineworld-movement`; `catalog.rs`
  `Capability::Movement` (id, install, provider), `AVAILABLE` of 3 in pack order. `world.yaml`
  `systems: [presence, movement, conversation]`. `tools/cli/tests/support/mod.rs`: `arrive` replaced by
  `stride(..)` (one `move` frame), `STRIDE_MM = 2_000` (a literal, `ARC-23` rule 2), `walk(actor, place,
  from, to)` (fewest equal integer strides, `n` grown until each stride's squared length fits) and
  `Client::walk_accepted`. `ac15_one_alice.rs` (5 walks) and `restart.rs` (1) walk from the pack's
  literal seats `(4600, 200)` / `(4600, 4400)`. `kill_and_resume.rs`: every third cafe request is a
  `move` to a point in the 2.4 m square around the actor's seat (`CAFE_SEATS`, literals); the child
  prints each move's answer and the parent counts them.
- [x] Validation. AC-15 6/6 PASS, printing `the 2D window walked to Alice in 2 strides` / `the 3D
  window … 2 strides` and asserting 2 each (3 493 mm and 2 970 mm). Restart 2/2 PASS. `kill_and_resume`:
  `[cafe] control: 100 moves, 94 accepted, 6 refused too-far-away` (asserted: sum = 100, both > 0;
  the fact floor is now the located accepted count instead of "every arrive is accepted"); cafe PASS
  0.3 s, clock PASS 0.4 s, byte-identical survivor saves at 3 kill points. Workspace rc 0, **323
  passed, 0 failed**; clippy `-D warnings` clean.
  **Expected-value updates (composition grew, claims unchanged):** `commands.rs` report lists
  `presence, movement, conversation`; `server_command.rs` systems list; `worldpack/tests/social_cafe.rs`
  `pack.systems()`; `restart.rs` revision counts 3 → 4 before the kill ("genesis, the two strides and
  the talk"), 4 → 5 after it, replay "5 revision(s) … head revision 5" — each is the same claim counted
  over one more request, because a walk to Alice is now two requests.
- [x] Review: diffs read line by line — no assertion removed or loosened; every changed number is a
  located count of requests (`ARC-23` rule 1) and the stride bound in the helper is a literal. The
  genesis fact count reported by `validate` is unchanged (`4 genesis fact(s)`): installing movement adds
  no fact to a pack without passages.

## C4b — The World Pack `passages` field (Q6)

**Goal.** A World Pack can state that two of its places open onto each other and where the doorway
is, as a genesis fact movement reduces. **Depends on:** C4. Format, loader and specification in one
commit (§10.1 Q6).

- [ ] Implementation:
  - `worldpack/src/format.rs`: `AuthoredPlace.passages: Vec<AuthoredPassage { to: EntityKey, here:
    Option<AuthoredPosition>, there: Option<AuthoredPosition> }>` (`deny_unknown_fields`).
  - `worldpack/src/read.rs` / `error.rs`: refused by name — a passage to an undeclared place, to
    itself, a pair stated twice (from both files or twice in one), and passages in a pack that does
    not enable `movement` (rule 4 of §4.1, as for `location` and `presence`).
  - `worldpack/src/{catalog,load}.rs`: `PASSAGE_OWNER = Movement`; each passage becomes
    `mineworld_movement::passage(..)`, stated after the places' and before the people's facts.
  - `docs/MODULE_SPEC.md` §4.1 (the field, rule 4 extended) and `docs/PACKAGE_FORMAT.md` §8.
- [ ] Validation (`worldpack/tests/`): a test pack with two places and a doorway loads and movement's
  `Passages` holds the doorway on both sides; each refusal above names its file and field.
- [ ] Review: a World Pack still states no rule — the doorway is a fact movement owns, not a
  movement policy; social-cafe unchanged in this commit (see the amendments block).

## C5 — The Godot demo walks with `move`; `AC-13` evidence re-recorded

**Amended at freeze (Q6, Q4).** This commit also adds `places/street.yaml` and the café's doorway to
`worlds/social-cafe` (ids shift: re-checked in every test that names one), and the demo's stride
loop follows the Q4 reporting rule.

**Goal.** The reference client that exists submits `move`, and the frozen evidence `AC-13` reads comes
from a real run. **Depends on:** C4 (the server must provide `move`).

- [ ] Implementation:
  - `clients/protocol/demo/demo.gd`: `_walk_beside` submits strides of at most 2.0 m toward the
    destination (converted once at the boundary by `MineWorldSpace`, integers on the wire); VERB
    `"move": "Walk to"`.
  - `clients/protocol/run.sh evidence` and `run.sh` (windowed) re-run; `evidence/` regenerated.
  - `clients/protocol/README.md` / `evidence/README.md` if a sentence names `arrive`.
- [ ] Validation:
  - real Godot 4.7.2, headless, both flavours, against `mineworld server worlds/social-cafe --agent
    alice`: the transcripts show each stride `accepted` and the `talk` accepted after the walk; the
    windowed run's screenshot is inspected (§19);
  - `tools/cli/tests/ac13_semantic_parity.rs` passes against the regenerated `request-{2d,3d}.json`:
    identical semantic cores, both answered `Accepted` by the same system.
- [ ] Review: the client decides nothing — it submits and renders the answer; `MAX_STRIDE` appears in
  the client only as a request size; GDScript sends integers (F9).
- **Dependency on permissions:** `run.sh` is a shell script outside the allowed command list of this
  session (Q8).

## C6 — `arrive` retired; `PersonEnteredPlace`

**Goal.** One movement path (I-2) and an occupancy-change fact (MD-7). **Depends on:** C5 (no caller
left).

- [ ] Implementation:
  - `systems/presence/src/action.rs` deleted; `lib.rs` exports; `system.rs`: declaration without
    `providing::<Arrive>()`, `emitting::<PersonEnteredPlace>()` added, `validate`/`resolve` removed
    (defaults: a system that provides no action is never asked), `offers` returns nothing, `react`
    emits `PersonEnteredPlace` on a change of known place, `VERSION` 2; `MALFORMED_PAYLOAD` and
    `codec::action_payload` removed if unused.
  - `systems/presence/src/event.rs`: `PersonEnteredPlace`.
  - `systems/presence/tests/presence.rs`: fixture places by genesis and relocates through a test-only
    system that provides a test action and states `Arrived` via `arrival()` while depending on
    presence — the `ARC-26` pattern exercised from presence's side; tests that asserted `arrive`'s
    affordance are replaced by "presence offers nothing"; structural scan gains `movement`, `passage`,
    `stride`.
  - `systems/conversation/tests/conversation_and_presence.rs`: setup by genesis, "walk closer" through
    `MovementSystem` (dev-dependency); offered lists become `["move","talk"]` / `["move"]` where movement
    is installed.
  - `systems/presence/README.md`, crate docs: the `arrive` section replaced by "presence owns where
    people are; who may move them is another system's decision (`ARC-26`)".
- [ ] Validation:
  - a cross-place move emits `Arrived` then `PersonEnteredPlace` (caused by the `Arrived`, provenance
    `presence`, `from` the old place), a within-place move and a genesis placement emit none — every
    existing fact-sequence assertion in the repository passes unchanged;
  - `arrive` submitted to the social-cafe server → `Unavailable` (a cli test, one assertion added to an
    existing scenario);
  - **amended at freeze (Q6):** in the real `worlds/social-cafe`, the visitor walks to the café
    doorway and out into the street with `move` strides; `PersonEnteredPlace { from: cafe, place:
    street }` is recorded and the `present-in` edge moves — the headline fact of S6 in a world the
    repository ships, not only in a fixture;
  - full workspace suite green; `kill_and_resume` PASS.
- [ ] Review: `rg -w arrive` over non-historical sources is empty except the retirement note;
  presence no longer reads any action payload; no other pack's vocabulary in presence.

## C7 — Documentation and ledger close

- [ ] Implementation: `systems/README.md`, `docs/MVP_STATUS.md`, `worlds/social-cafe/README.md`,
  `tools/cli/README.md` if affected, `docs/ARCHITECTURE.md` §14 if its systems list needs `movement`;
  this document's §9 and §12; handoff.
- [ ] Validation: terminal gates (§6) on the final executable head; doc checks.
- [ ] Review: change-amplification diff check — `git diff a594164 -- kernel contracts` contains only C2;
  `server/` unchanged; `cognition/` unchanged.

---

# 5. Integration checkpoint

```text
IC-1  systems/movement/tests/persisted.rs   create a two-place world into a real SQLite save → stride
      → refused stride → passage crossing (occupancy moves) → drop the world → resume from the file
      → identical Presence / present-in / Passages; verify() from genesis; continue at the next id
IC-2  tools/cli/tests/ac15_one_alice.rs     the real binary, real sockets, two clients walking to the
      same Alice with `move` strides and talking; AC-15 unchanged in claim
IC-3  clients/protocol/run.sh evidence      the real Godot client walks with `move` and AC-13 compares
      what it actually sent (C5)
IC-4  persistence/tests/kill_and_resume.rs  SIGKILL mid-run with moves (accepted and refused) in the
      journal; byte-identical resume
```

**Adversarial criteria.** A stride one millimetre over the bound is refused, and the refusal changes
nothing (C3). A client cannot relocate a person by any action other than `move` once C6 lands (C6
cli assertion: `arrive` → `Unavailable`). Disabling movement leaves the rest of a world byte-identical
to one that never had it (C3). Presence names nothing of movement (structural scan). A system that
speaks another's vocabulary without depending on it cannot be installed (C2).

# 6. Test ownership and verification

```text
static          fmt, clippy -D warnings; the type system owns "Move carries a Location" and
                "Passages is written only by MovementSystem" (owned_component!, WriteToken)
unit            none new as such: the validation rules are integration-tested through dispatch
                against real systems, because their meaning is the answer dispatch gives
integration     systems/movement/tests/movement.rs (CP-1 in memory, CP-2, CP-3, negatives, degeneracy,
                structural); kernel install refusal (C2); presence and conversation migrations (C6)
real lifecycle  IC-1 real SQLite file; IC-2 real server binary and sockets; IC-4 real SIGKILL;
                IC-3 real Godot 4.7.2 run (C5)
real model      NOT REQUIRED — no LM is involved (CLAUDE.md §5)
CI              N/A — no workflow in the repository (S13); the local gates are the terminal evidence,
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

Budget: the full suite runs in about 20 s of wall time today (E-0); this PR adds seconds. The Godot
evidence run is bounded by `run.sh`'s `--quit-after 1200` frames per flavour.

# 7. Self-review against the frozen specifications

```text
CHECKED  INV-7   presence alone writes Presence/present-in; movement alone writes Passages; the
                 cross-system effect travels as a fact the owner reduces (I-1)
CHECKED  INV-9   the stride and passage rules are evaluated only in MovementSystem::validate; the
                 client's actor_location stays a report (I-3)
CHECKED  INV-10  disabling movement → move Unavailable (CP-2)
CHECKED  INV-12  the kernel change names no spatial or domain concept (C2)
CHECKED  INV-14  no renderer, transport or backend concept in any new contract
CHECKED  INV-15  every relocation is caused by a move ActionIntent or by genesis
CHECKED  ENGINEERING_RULES §6   MoveIntent / travel Process / spatial state / rendered movement are
                 four things (§2.4); no teleportation by place name (passages), no renderer-only motion
CHECKED  ENGINEERING_RULES §7–8  move declares what it needs (stride, passage); the answer is the
                 system's; TooFarAway is server-side (CP-3)
CHECKED  ENGINEERING_RULES §11–12, §9  answered in §2.8 — yes, no, yes
CHECKED  ARC-15  people and passages are seeded by genesis facts their owners reduce
CHECKED  ARC-23  every bound in a test is a literal from the requirement; every equality is preceded by
                 a located count
CHECKED  ARC-25  every move is a journaled dispatch; IC-1 and IC-4 re-execute it byte for byte
CHECKED  REUSE   no infrastructure built: the system uses the contract evaluator, the kernel's
                 registry and S5's persistence. No dependency added. Pathfinding/navmesh libraries
                 were considered and are a client's concern, not the server's (§1.2)
CHECKED  §8 change amplification  a new capability is one crate + one catalog line + one pack line;
                 presence changes only to lose `arrive` and gain its own occupancy fact
FLAGGED  MF-1  Event::OWNER reinterpreted as vocabulary owner — material, Q1
FLAGGED  MF-2  `arrive` retired from the wire — material, Q2
FLAGGED  kernel change in a "first real systems" step (C2) — generic and small, but a kernel change; Q3
FLAGGED  deviation from overall S6 wording: PersonEnteredPlace owned by presence, not movement — Q5
```

---

# 8. Source audit (`main @ a594164`, 2026-09-30)

## 8.1 What was inspected

```text
systems/presence/src/{lib,action,component,event,system,codec,interaction,observe}.rs, README.md,
      tests/presence.rs (fixture, disable test, structural scan)
systems/conversation/src/system.rs (validate: evaluator-only, ignores actor_location),
      tests/conversation_and_presence.rs (Arrive call sites, offered lists)
kernel/src/system.rs (SystemDeclaration, Emission::owner, System trait), dispatch.rs (record: emits
      check only; genesis: owner from Event::OWNER; reduce order), registry.rs (check_installable,
      disable refused while a dependent is enabled), view.rs, process.rs (ProcessStart for a future
      travel Process)
contracts/src/spatial.rs (Location, LocalPosition::within is private, SpatialRequirement::evaluate
      order and degeneracies), event.rs (Event::OWNER doc), observation.rs (Affordance)
worldpack/src/catalog.rs (Capability, located → arrival), worlds/social-cafe/{world.yaml, places/,
      people/}
persistence/src/{lib,world,format}.rs (PersistentWorld::create/resume, SAVE_FORMAT 1)
persistence/tests/kill_and_resume.rs (cafe generator), tools/cli/tests/{support/mod.rs, ac15_one_alice.rs,
      restart.rs, ac13_semantic_parity.rs}, clients/protocol/{demo/demo.gd, run.sh, evidence/README.md,
      evidence/request-2d.json}
server/src, server/PROTOCOL.md (no action vocabulary; routes any action type)
docs: CORE_CONCEPTS §§2, 6, 10–13, 15; ENGINEERING_RULES §§4–12; MVP §§4–5, 9, 12; NETWORKING §4;
      DECISIONS ARC-15, ARC-23, ARC-25; references/UNREAL_ADAPTER_SPIKE.md §"Moving is submitting arrive"
baseline  cargo test --workspace --no-fail-fast → 311 passed, 0 failed; kill_and_resume cafe PASS,
      clock PASS (E-0)
```

## 8.2 Findings

**F-1 — `arrive` is an unrestricted relocation available to every client (material; MF-2).**
`arrive_requirement()` is `NONE` and `validate` checks identity only (§2.1). Any distance rule added
beside it is bypassable.

**F-2 — `arrive`'s reason to exist was superseded by `ARC-15` (bounded).** Initial placement is a
genesis fact (`catalog.rs::located`); no runtime caller needs `arrive` for placement.

**F-3 — the kernel permits a system to emit another system's event type; the contract text does not
(material; MF-1).** `Dispatcher::record` checks the running system's own declaration only;
`Event::OWNER` is read only by genesis. `contracts/src/event.rs:140-166` documents `OWNER` as the
emitter.

**F-4 — disabling a dependency is refused (bounded; decides §2.3 (a)).** `registry.rs:332`: a system
cannot be disabled while an enabled system depends on it. Movement inside presence could never be
disabled in a world with conversation.

**F-5 — `LocalPosition::within` is private (bounded; drives MD-5).** Movement must not reimplement
distance; `SpatialRequirement::same_place().within(r).evaluate(from, Some(to), true)` gives the exact
comparison through the public evaluator, with the contract's degeneracies, and no contract change.

**F-6 — a targetless offer is evaluated against its requirement with no target (bounded; drives
MD-9).** `observe.rs::verdict`: a `same_place().within(..)` requirement on a targetless offer would
always render `PreconditionFailed`.

**F-7 — `AC-13` reads frozen Godot evidence that submits `arrive` (material consequence of MF-2).**
`ac13_semantic_parity.rs` replays `request-{2d,3d}.json`; after retirement those frames answer
`Unavailable` and the test's `Accepted` assertion fails. The evidence must be regenerated by a real
Godot run (C5) before `arrive` is retired (C6).

**F-8 — world-pack positions put both seats more than one stride from Alice (bounded).** Visitor
(4600,200) → `NEXT_TO_ALICE` (1200,1000) is 3 493 mm; wanderer (4600,4400) → (2400,2400) is 2 970 mm.
Each becomes two strides. The test claims do not change.

**F-9 — the declaration is part of a save (bounded; drives the `SAVE_FORMAT` bump).** S5's
`InstalledSystemRecord` stores the whole `SystemDeclaration`; adding a field changes the encoding, so
an old save must be refused by format version rather than fail to decode.

**F-10 — saves of the social-cafe world written before this PR will be refused (bounded; Q9).** The
composition gains `movement` and presence's version changes; S5 refuses both by name (no migration,
S5 L-1).

**F-11 — the 3D spike client submits nothing (bounded).** `clients/3d-spike` has no `submit` call; only
`clients/protocol/demo` is a server-connected client. The visual agents' worktrees are not touched.

## 8.3 Material findings

MF-1 (F-3) and MF-2 (F-1, F-7) — §2.7, Q1 and Q2. Both change what an existing public contract means
or contains, so neither is assumed; implementation waits on the answers.

---

# 9. Ledger and evidence

**E-0 (C0).** Baseline on `main @ a594164`, this worktree, clean tree:
`cargo test --workspace --no-fail-fast` → rc 0; 50 harness results summing to **311 passed, 0 failed**;
`kill_and_resume` program: `[cafe] PASS in 0.3 s`, `[clock] PASS in 0.4 s`.
`python3 scripts/check_decision_ids.py` → 34 decision ids, all distinct;
`python3 scripts/check_doc_headings.py` → 134 numbered sections across 21 documents, none duplicated.
PASS.

**E-2 (C2, `47874ca`).** Full workspace suite rc 0, 51 harness results, **313 passed, 0 failed**;
`kill_and_resume` cafe PASS 0.3 s, clock PASS 0.4 s; decision ids 35 distinct; doc headings 134, none
duplicated. fmt and clippy `-D warnings` clean (before the stall, same head). PASS.

## 9.1 Limitations and follow-ups (expected)

```text
L-1  No speed model: MAX_STRIDE bounds one request, not requests per second. A client spamming strides
     moves arbitrarily fast. Needs per-person time accounting finer than WorldTime's one second.
L-2  No walls: line of access inside a place is not evaluated (DD-7); a stride can cross furniture.
L-3  Passages are genesis-only and always open; doors are a later system.
L-4  The stride is a constant, not world configuration, until S7's configuration schema.
L-5  Event volume of continuous 3D movement is unmeasured until Demo B (R-1).
L-6  Old saves of social-cafe are refused, not migrated (S5 L-1).
```

**Risk R-1.** A 3D client at 5 Hz writes 5 facts per second per walking player. Correct and
replayable; cost unmeasured. Remedy, if needed, is a client reporting policy or a coalescing movement
Process — no contract here changes.

---

# 10. Questions for the primary session / operator

```text
Q1 (material)  Accept MF-1 / ARC-26: Event::OWNER is the vocabulary owner; another system may state a
               fact of that type if it declares the emission, depends on the owner, and builds the
               payload with the owner's constructor. MovementSystem states presence's Arrived.
               Alternatives (a)–(d) of §2.3 rejected. Recommended.
Q2 (material)  Retire `arrive` (C6), after every caller moves to `move` (C4, C5) and the AC-13 Godot
               evidence is re-recorded from a real run. Fallback: keep `arrive` as a documented
               unrestricted bypass — not recommended, it makes CP-3 hollow. Recommended: retire.
Q3             Enforce Q1's dependency condition in the kernel at install (C2; SystemDeclaration records
               emitted owners; SAVE_FORMAT 1 → 2), or leave it a review convention. Recommended: enforce
               — overall §2 item 1 prefers a check to a convention, and it is ~40 lines with no domain
               term.
Q4             MAX_STRIDE = 2 000 mm, a constant in S6 (configuration with S7); published in the
               movement contract and ADOPTION.md; `move` offered with requirement NONE documented as
               "nothing required of a target". Alternative: carry the stride in the affordance, which
               needs a targetless-requirement semantics in contracts — not recommended in S6.
Q5             PersonEnteredPlace owned and emitted by PresenceSystem on a change of known place (not on
               genesis, not within a place). Deviates from overall S6's wording ("MovementSystem …
               PersonEnteredPlace"); the planning session rewords S6. Recommended.
Q6             Passages owned by MovementSystem, genesis-only, World Pack field deferred, social-cafe
               stays one place in S6. Alternative: add `passages:` to the pack format (MODULE_SPEC §4.1,
               worldpack loader) and a `street` outside the café now, so the 2D/3D demos can walk out —
               cheap, but widens S6 into the pack format. Recommended: defer to S7 or Demo B.
Q7             Accept L-1 (no speed model) and L-2 (no walls) as stated limitations, not S6 work.
Q8             C5 needs `clients/protocol/run.sh evidence` (Godot headless) and `run.sh` (windowed
               screenshot). `bash <script>` is outside this session's allowed command list. Either the
               primary session grants it for C5, or runs C5's evidence step itself; the commit order
               (C5 before C6) does not change.
Q9             Existing social-cafe saves are refused after this PR (composition + presence version +
               SAVE_FORMAT). Accept; migration stays a later step (S5 Q7).
```

## 10.1 Answers — primary session review, 2026-09-30

Decided under the operator's autonomous authorization (overall §7) and their standing rule that
objective architectural correctness belongs to the agent. Q1 is reported to the operator by name
because it touches `CLAUDE.md` §4 rule 1; they may overrule it. Two answers depart from the draft's
recommendation: **Q4** gains a requirement and **Q6** is answered the other way.

**Q1 — ACCEPTED, with Q3 as a hard condition and two clarifications.** Verified in source before
answering. `contracts/src/event.rs` says `OWNER` is "the system that emits it", while
`kernel/src/dispatch.rs` `record` checks only that the *emitter's own declaration* lists the type and
never that `OWNER == emitter`. The specification and the kernel **already disagree**, which is a
`CLAUDE.md` §2.1(4) defect in its own right; this decision resolves it rather than introducing it.

Why this does not weaken single ownership (`CLAUDE.md` §4 rule 1). That rule is about **mutable
state**: one owning system writes it, and others ask by emitting a fact. Under this design
`Presence` is still written by exactly one thing — presence's own reduction of `Arrived`. Movement
writes no presence state; it states a fact in presence's vocabulary, built through presence's
constructor. The canonical alternative, movement emitting its own `MoveAccepted` for presence to
react to, would make presence depend on movement while movement depends on presence to read where
people are: a cycle, as §2.3 shows. Exposing the owner's vocabulary to declared dependents is the
smallest acyclic form of the same rule.

The two clarifications, which become part of `ARC-26`:

1. `OWNER` means the system that **defines the fact's vocabulary and is the only system that reduces
   it into owned state** — not merely the only system allowed to emit it.
2. **The owner still decides.** Presence's constructor and reduction keep the power to refuse an
   `Arrived` that would make presence's state invalid (an unknown place, a dead entity). Movement
   decides whether a *move* is legal; presence decides whether its *state* may take the value.
   Implement it so that refusal path exists and is tested.

**Q2 — ACCEPTED.** Retire `arrive`. A distance rule that one `arrive` bypasses is not a rule, and
`TooFarAway` would hold for the action but not for the world. C4 → C5 → C6 as drafted. The `AC-13`
recordings are re-recorded from a real Godot run, never edited by hand.

**Q3 — ACCEPTED.** Enforce "emitting another system's vocabulary requires declaring it and depending
on its owner" in the kernel at install time. A rule this close to single ownership must not be a
review convention. `SAVE_FORMAT` 1 → 2 is acceptable; no saves exist outside tests.

**Q4 — ACCEPTED, with a requirement the draft lacks.** `MAX_STRIDE` = 2000 mm as a constant now and
world configuration in S7. **But a per-request stride limit is only correct if clients report often
enough.** A 3D client jogging at about 2.6 m/s that reports every second would be refused for moving
legally. The design must state the client reporting rule — report before travelling `MAX_STRIDE`
since the last accepted position — and put it in `PROTOCOL.md` and `ADOPTION.md`. A test must show a
client following that rule at jog speed is never refused, and one that ignores it is.

**Q5 — ACCEPTED.** `PersonEnteredPlace` is owned by presence, since occupancy is presence's state.
The planning session rewords overall S6.

**Q6 — ANSWERED THE OTHER WAY: add the street outside the café to social-cafe now.** With one place,
`PersonEnteredPlace` never fires in any real world. The headline event of S6 would then exist only in
test fixtures, against `CLAUDE.md` §4 rule 9 ("sample worlds are integration fixtures… major changes
are evaluated by running them"). Two places joined by a doorway is also exactly what the 2D town and
the 3D environment slice already depict, so the world pack catches up with its clients rather than
running ahead of them. Add the doorway field to the World Pack format and record it in
`docs/PACKAGE_FORMAT.md` in the same commit — that document is the specification and must not lag
the schema (`CLAUDE.md` §2.1(4)).

**Q7 — ACCEPTED**, together with the Q4 reporting rule. No speed limit and no walls within a place
are recorded limitations, not hidden ones.

**Q8 — RESOLVED.** `clients/protocol/run.sh`, bare and through `bash`, is added to the permission
allow-list. It is a project launcher, within the tier the operator approved. Run C5's evidence step
yourself.

**Q9 — ACCEPTED.** Pre-PR saves are refused by name; migration stays a later step.

---

# 11. Execution contract (confirmed at freeze, 2026-09-30)

```text
PROJECT / PR        MVP-0 · Step 07 / PR 08 — First real systems: places and movement (S6)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-07-movement.md (this file)
RELATED / BINDING   overall.md §§2, 3 (S6), 7; CORE_CONCEPTS §§2, 6, 10–13, 15; ENGINEERING_RULES
                    §§4–12, 19; MVP §9 AC-2, AC-6, AC-12, AC-13, AC-15; NETWORKING §4; DECISIONS
                    ARC-15, ARC-23, ARC-25 (+ ARC-26 once C1 lands); step-04; step-06
IMPLEMENTATION BASE main @ a594164; branch mvp0/pr-08-movement; worktree
                    /Users/yuema137/mineworld-worktrees/s6-movement (held by this session only)
APPROVED SCOPE      §1.1, as answered in §10
FROZEN INVARIANTS   §1.3 I-1 … I-8
SEQUENCE            C0 → C1 → C2 → C3 → C4 → C5 → C6 → C7, each committed and pushed when coherent
VALIDATION BUDGET   unit/integration/static: unrestricted; real-model: NOT REQUIRED; Godot evidence run
                    bounded by run.sh's frame limits; total well under the one-hour envelope
LIVE DOCUMENTATION  this file (§4 checkboxes, §9 ledger)
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for PR 08 at C1
ENDPOINT AUTHORITY
  implementation + local validation   authorized after DESIGN FROZEN — source: the brief ("Phase 2 —
                                      after you are told the design is frozen") and overall §7
                                      autonomous authorization (2026-09-25)
  semantic commits                    authorized — source: the brief ("Commit and push after every
                                      coherent step")
  branch push                         authorized — source: D-12; the brief
  PR creation / update                authorized — source: the brief ("Open a PR with gh pr create")
  CI repair                           N/A — no CI workflow in the repository (S13)
  merge                               explicit operator authorization only; the brief: "do not merge"
POST-MERGE SYNC     the planning session owns step/overall updates; this session owns this document
NORMAL STOP         PR 08 READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       any change to §1.3, to an existing public contract's shape beyond §1.1 as answered,
                    to ownership, or to scope — stop and report with evidence
```
