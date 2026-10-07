# Step 11 — Bodies and physical interaction (S15)

**Role:** step document for a new step, proposed as **S15 — Bodies and physical interaction**. It
records the operator's requirement, the audit, the answers to the planning questions, the Rapier
determinism prototype, the proposed ownership, facts and contracts, a PR split, risks and open
questions. It holds no frozen PR design yet: the first PR is detailed only after the operator decides
the material questions in §13 (`CLAUDE.md` §3, "detail one step ahead").
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S11, S12, S14), §7.
**Lifecycle:** `DRAFT — awaiting operator review`. Nothing in this document is frozen, and nothing in
it authorizes implementation.
**Branch:** `design/physics` from `main @ 7f6960e`. Worktree `/Users/yuema137/mineworld-worktrees/physics`.
**Prototype:** `/Users/yuema137/mineworld-demos/physics-spike/`, a standalone Cargo project outside the
repository, never committed to MineWorld (§9).

## Why the file is `step-11`, and the step is S15

Step files are numbered in the order they were written, not by step: `step-05` is S5V and `step-06`
is S5. The next free file number is 11. The step itself is new, so it takes the next free step
number, **S15**, after the fourteen steps of `overall.md` §3. Its number says nothing about its
order; §12 places it in the sequence.

---

# 1. The requirement

## 1.1 Verbatim (operator, 2026-10-07)

> 我发现人在跑步 走路的时候，也会跟其他人穿模。这个是不应该发生的。我们有什么办法可以更加统一地避免穿模呢？是不是要加入一些物理引擎啥的，我不要求很真实的物理模拟，但是人跟其他物体 人 需要相互作用，会把别的东西挤开，踢走，扔掉之类的。

English rendering, which the requirement IDs below cite:

```text
R-1  People must not pass through each other while walking or running.
R-2  Avoiding interpenetration is uniform, not patched case by case.
R-3  Realistic physics is not required.
R-4  People interact with objects and with other people: push things aside, kick things away,
     throw things.
```

## 1.2 The direction the operator approved (2026-10-07)

```text
D-1  Authoritative physics on the server, as a System Pack, not in the kernel. Candidate: Rapier.
D-2  Client-side collision and prediction in Godot, candidate Jolt. Presentation only; the
     server's result is authoritative.
D-3  Verify the licences and versions of Rapier and Jolt before anything else.
D-4  Define the boundary with the existing `movement` System Pack.
D-5  Build a determinism prototype.
```

## 1.3 The answer in brief

```text
Server   a new System Pack, `bodies`, resolves every arrival of a person against the bodies,
         walls and loose objects of that place, with Rapier queries and a few fixed sub-steps,
         rebuilt from integer state for each resolution and quantized back to whole millimetres.
         It never decides whether a move is allowed: `movement` still does. It decides what a
         body does when it gets there: stops at a person or a wall, pushes a box aside.
         It provides `kick`, `throw` and `shove`, each with a declared SpatialRequirement.
Presence `arrived` gains `from`, filled by presence's own constructor, so that whoever reacts to
         an arrival can see the whole stride. The one edit to an existing pack.
Kernel   unchanged. Contracts unchanged.
Clients  Godot keeps its local CharacterBody3D, now with Jolt selected explicitly, and gives every
         other person and every loose object a collider at its observed position. It reconciles
         to the authoritative position whenever the two differ by more than a stated tolerance,
         not only after a refusal. Kick, throw and shove are intents with a target.
```

---

# 2. Audit — what exists, from source (`main @ 7f6960e`)

Every claim below was read in this session from the file named.

## 2.1 Why people pass through each other today

There are two causes, one in each layer, and both have to be fixed for R-1.

**The 3D client gives other people no collider.** `vis-environment` `clients/3d-spike/scripts/player.gd`
(read-only audit): the walker is a `CharacterBody3D` with `collision_layer = 0` and
`collision_mask = Build.LAYER_WORLD`, and `LAYER_WORLD` (`scripts/build.gd`, `= 1`) is set only on
`StaticBody3D` nodes: walls, floors and ramps. Other people are drawn by `slice_link.gd` `_figure()`
as `NPC.make(...)`, and `npc.gd` `extends Node3D`. There is no body and no collision shape. So the
player's local physics walks straight through every drawn person, at any speed.

**The server has no body model.** `systems/movement/src/system.rs` `reachable()` checks only that
the stride is at most `MAX_STRIDE` (2 000 mm), via `SpatialRequirement::evaluate`, and that a passage
exists when the place changes. Nothing compares the destination with where anybody else is. The
pack's own documentation states the limit: "Nor does it see walls: a stride may pass through a table,
because line of access needs geometry no layer owns yet (`DD-7`, L-2)" (`systems/movement/src/action.rs`,
`MAX_STRIDE`). So two people may be told they stand at the same millimetre, and the headless
controller's `approach` and `wander` (`cognition/rule-controller/src/paced.rs`) propose such strides
freely. The only thing that keeps a paced person from walking into another is `APPROACH_STOPS_AT`,
a constant in a controller. That is not a world rule.

Fixing only the client would hide the symptom for one player and leave the world state wrong. Fixing
only the server would leave the 3D player walking through people until the next observation snapped
them back. R-2 ("uniform") asks for one rule, held by the world, that both clients display.

## 2.2 Movement and presence

```text
presence (systems/presence)
  owns       Presence { location: Location }       per person; where they are
             present-in                             person → place edge
  facts      arrived { person, location }          its vocabulary; it alone reduces it
             person-entered-place                   stated while reducing an arrival that
                                                    changes place
  provides   no action (ARC-26)
  checks     admit(world, person, location)         living person, living place: nothing spatial

movement (systems/movement)
  depends    presence
  owns       Passages                               per place: doorways, both sides
  provides   move { to: Location }                  decided in validate:
                                                      same place: |to − from| ≤ MAX_STRIDE
                                                      other place: through a passage, within a
                                                      stride of the doorway on both sides
                                                      else TooFarAway
  states     presence's arrived, through arrival()  ARC-26: depend on the owner, use its
                                                    constructor; the owner may still refuse
```

**Numeric types.** `contracts/src/spatial.rs`: `Millimetres(i32)`, `Millidegrees(i32)`,
`LocalPosition { x, y, z }` in millimetres, `Location { place, local?, facing? }`. Distances are
compared on squared `i128`. `docs/CORE_CONCEPTS.md` §6.2 makes it a non-negotiable property:
"Fixed-point, never floating-point… Positions reach the Event Log, the log is replayed, and
floating-point arithmetic is not reproducible across platforms." No float exists in any persisted
spatial value.

**What the arrival carries.** `systems/presence/src/event.rs`: `Arrived { person, location }`. There
is no `from`. By the time a subscriber that depends on presence reacts, presence has already reduced
the fact (reactions run in registration order, `kernel/src/system.rs` on `react`, and a dependent is
registered after its dependency), so the previous position is gone from the world. This matters for
§4: a system that wants to know *what a stride passed through* cannot learn it from the fact today.

## 2.3 Kernel: the System contract and the clock

- `kernel/src/system.rs`: `System` methods take `&self`. A system owns no fields: "a system that kept
  simulation state in its own fields would be state no event log could reconstruct and no snapshot
  could carry." **A Rapier world cannot live in a system between calls.** It is either rebuilt for
  each resolution or stored as a component.
- Hooks: `validate` (read-only), `resolve` (writes owned components, returns emissions), `react`
  (reduces subscribed facts, may emit), `wake` (process end), `interrupt`. No per-tick hook exists.
- `contracts/src/time.rs`: `WorldTime(i64)`, whole seconds. `DEP-6`: a discrete-event queue keyed by
  `(WorldTime, sequence)` that **skips idle time**. That skip is what makes `AC-11`'s 300 days cheap.
  A stepped physics world cannot skip idle time.
- `kernel/src/dispatch.rs`: `CASCADE_DEPTH_LIMIT = 16`. A reaction that emits a fact which is delivered
  again terminates only if it stops emitting. The design in §4 relies on that and states why it stops.
- `clippy.toml` bans `HashMap` and `HashSet` across the workspace, because iteration order reaches
  serialized state.
- Registry: an action type has exactly one provider (`kernel/src/error.rs`, "action type '…' is already
  provided by '…'"). A second pack cannot provide `move`.

## 2.4 Persistence

`ARC-25` (`docs/DECISIONS.md`): a world's state is its journal re-executed. The journal holds genesis,
every `dispatch(intent, at)` and every `advance_to` that fired something. A restart loads the newest
snapshot and re-executes the journal after it, and **every re-executed input must regenerate its
logged facts byte for byte**, or the load is refused (`ReplayDiverged`). A snapshot encodes every
component row as canonical JSON (`DEP-5` note). Consequence for physics: every float computation a
resolution performs must give the same bits on re-execution, on any machine the save may be resumed
on (`AC-8`: laptop and container), and every Rapier upgrade that changes a result must be refused by
name rather than discovered as a divergence.

## 2.5 Server and protocol

`server/PROTOCOL.md` §6.2: a client moves only by `move`, and must report before travelling
`MAX_STRIDE` since the last accepted position. §8: observations at 10 Hz by default
(`server/src/host.rs`, `observation_interval: 100 ms`); "world clock whole simulated seconds; several
frames share one instant". `NETWORKING.md` §4: "Local physics: renderer-dependent, never
authoritative"; §3: "There is no requirement for 60 Hz authoritative replication."

The 3D slice reports every 0.5 m (`slice_link.gd` `REPORT_DIST`, a quarter of the bound) and
reconciles **only after a refused move** (`_on_resolved`: `if what == MOVE_ACTION and kind !=
"accepted": _reconcile = true`). An accepted move whose resulting position differs from what was
reported is never corrected on the client today. The world has had no way to produce one until now.

## 2.6 The headless controller

`cognition/rule-controller/src/paced.rs`: `approach` takes a stride toward a person, stopping
`APPROACH_STOPS_AT` short; `wander` takes a seeded offset of up to 1 400 mm on each axis; `walk`
submits `Move` if the observation offers it. Places have no extent (`ARC-27`, accepted limitations),
so wandering is bounded by nothing in the world.

## 2.7 What the specifications already anticipate

- `CORE_CONCEPTS.md` §6.3: "Line of access is declared and not yet evaluated. Deciding whether a wall
  stands between two positions needs world geometry that no layer owns yet. The declaration is carried
  so that a geometry provider can answer it later."
- `step-07-movement.md` §2.8: "What the 3D client cannot yet get from the server is collision against
  walls (L-2) and a speed limit (L-1); both are additions to `MovementSystem`'s validation (a geometry
  provider, a gait component), not kernel changes."
- `ENGINEERING_RULES.md` §12: no "physics engine" in generic entity contracts. §5: "the kernel must not
  depend on … colliders".
- `DEP-4`: Godot's "physics and collision, the character controller (`CharacterBody3D`)" are reused,
  not built.
- `REUSE_POLICY.md` §4 lists "physics engines" as commodity infrastructure.

So this step is where a layer comes to own geometry, as S6 anticipated. It has to do so without that
layer being the kernel and without a physics concept entering a contract.

---

# 3. Question 1 — Licences, versions and the dependency decision

## 3.1 Rapier (server)

All read on 2026-10-07.

| Fact | Value | Source |
| --- | --- | --- |
| `rapier2d`, `rapier3d`, `rapier2d-f64` | **0.36.0**, published 2026-09-25 | <https://crates.io/api/v1/crates/rapier3d> (and `/rapier2d`, `/rapier2d-f64`) |
| Licence | **Apache-2.0** ("Copyright 2020 Sébastien Crozet") | <https://raw.githubusercontent.com/dimforge/rapier/master/LICENSE>; crates.io `license` field |
| MSRV | `rust_version = 1.86`, edition 2024. Ours is **1.97.1**: compatible. | crates.io JSON for 0.36.0 |
| Cadence | 0.31.0 2025-11-21 · 0.32.0 2026-01-09 · 0.33.0 2026-06-05 · 0.34.0 2026-07-04 · 0.35.0 2026-08-08 (0.35.1–0.35.3 to 2026-08-28) · 0.36.0 2026-09-25. Roughly one **breaking** minor a month since June 2026. Every release published by `sebcrozet`. | crates.io JSON |
| `parry3d` / `parry2d` (its geometry layer) | **0.31.1**, 2026-09-18; 0.31.0 was yanked the same day; Apache-2.0; no MSRV declared | <https://crates.io/api/v1/crates/parry3d> |
| Maintenance | Active. The 0.35 line moved the math from `nalgebra` to `glam` (`glamx`); `simd-stable` / `simd-nightly` were removed and replaced by `simd8`; `PhysicsWorld` bundles the state. | crates.io feature lists; `rapier3d-0.36.0/src/pipeline/physics_world.rs` |
| Features of 0.36.0 | `enhanced-determinism = ["simba/libm_force", "parry/enhanced-determinism"]`, `serde-serialize`, `parallel`, `simd8`, `f32` (default), `dim3`; f64 is a separate crate. | crates.io JSON |

**Determinism, quoted** (<https://rapier.rs/docs/user_guides/rust/determinism>, Rust guide for 0.36,
read 2026-10-07):

- Default: "By default, Rapier is **locally deterministic**": the same initial conditions "with the same
  machine, using the same version of Rapier, and the same version of the Rust compiler"; "doing this on
  two different computers may result in completely different results."
- The same initial conditions include insertion order: bodies, colliders and joints "added/removed to
  sets (rigid-body sets, etc.) in the exact same order."
- Cross-platform: "The `enhanced-determinism` feature of Rapier is enabled"; the platform must "strictly
  comply to the IEEE 754-2008 floating-points standard", which "include most modern mainstream
  processors as well as WASM targets"; initial values computed with `sin`, `cos`, `sqrt` and similar must
  use nalgebra's `ComplexField`/`RealField`, e.g. `ComplexField::sin(0.4)`, not `0.4.sin()`.
- It conflicts only with `simd8`: "cannot be enabled at the same time as the `simd8` feature, which changes
  the SIMD lane width and is therefore its own determinism domain." `parallel` is allowed: "the results of
  the parallel solver are identical to the results of the sequential one", independent of the rayon pool
  size.
- The page does **not** require a fixed timestep, and does not require the same compiler for the
  cross-platform case. It does not say what happens across Rapier versions; we assume nothing and treat a
  Rapier upgrade as a change of results (§6.3).

**Snapshots, quoted** (<https://rapier.rs/docs/user_guides/rust/serialization>): with
`serde-serialize`, the complete state "is serialized by serializing its physics world as a whole", for
example with `bincode`. With `enhanced-determinism` on a compliant platform, two machines give "the
exact same byte vectors if the physics state is serialized on both machines after the same number of
timesteps." The source confirms what a snapshot omits: `physics_pipeline`, `collision_pipeline` and
`ccd_solver` are `#[serde(skip)]`, "workspace only: not part of a snapshot"
(`rapier3d-0.36.0/src/pipeline/physics_world.rs`).

**Character controller** (<https://rapier.rs/docs/user_guides/rust/character_controller>, and
`src/control/character_controller.rs`): `KinematicCharacterController::move_shape` performs
move-and-slide by shape casts; it "**does not** support rotational movement"; it keeps a small `offset`
gap; it does not push dynamic bodies by itself ("the character controller's offset prevents actual
contacts"), for which `solve_character_collision_impulses` exists; the character must exclude its own
collider through the query filter.

## 3.2 Jolt (client)

All read on 2026-10-07.

| Fact | Value | Source |
| --- | --- | --- |
| Built into Godot | **4.4**, as an experimental alternative: "Until then, you have to enable this alternative to Godot Physics in the project settings." | <https://godotengine.org/releases/4.4/>; PR <https://github.com/godotengine/godot/pull/99895> |
| Default | **4.6**: "remove the experimental label and make Jolt the default physics engine for all new 3D projects. Existing projects aren't affected." | <https://godotengine.org/releases/4.6/>; PR <https://github.com/godotengine/godot/pull/105737> |
| Godot 4.7.2 | Latest stable, 2026-08-18. The 4.7 documentation still describes Jolt as built in and the default for new projects. **4.7.2 has it.** | <https://github.com/godotengine/godot/releases>; <https://docs.godotengine.org/en/stable/tutorials/physics/using_jolt_physics.html> |
| Enabling | "set Project Settings > Physics > 3D > Physics Engine to `Jolt Physics`", then Save & Restart. In `project.godot`: `[physics] 3d/physics_engine="Jolt Physics"`. | the 4.7 docs page above |
| Our projects | `clients/3d-spike/project.godot` and the slice's (`vis-environment`) set **no** physics engine key. Both were created before 4.6, so by the 4.6 note they keep Godot Physics. Which engine actually runs is to be confirmed by a run in the client PR (§11, PR 12d) and then set explicitly. | the files |
| Licence | Jolt Physics: **MIT** ("Copyright 2021 Jorrit Rouwe"). The former `godot-jolt` extension: MIT, in maintenance mode, supports Godot 4.3–4.6 only; the built-in module supersedes it. | <https://raw.githubusercontent.com/jrouwe/JoltPhysics/master/LICENSE>; <https://github.com/godot-jolt/godot-jolt> |
| 2D | Jolt in Godot is **3D only**. The 2D client keeps Godot's 2D physics and needs none for this step (§7.3). | 4.7 docs |
| Determinism | Jolt itself can be cross-platform deterministic (`JPH_CROSS_PLATFORM_DETERMINISTIC`, about 8 % cost), but Godot states: "Physics in Godot, regardless of physics engine, is **not** deterministic." The godot-jolt README: "Godot Jolt is not able to make such guarantees." **Irrelevant here, by design:** client physics is presentation (`NETWORKING.md` §4). | <https://raw.githubusercontent.com/jrouwe/JoltPhysics/master/Build/README.md>; <https://docs.godotengine.org/en/stable/tutorials/physics/physics_introduction.html> |

Known differences of Jolt from Godot Physics that touch this design (4.7 docs page): position-only
Baumgarte stabilization ("cannot overshoot but it may take longer to resolve the penetration");
kinematic-frozen bodies do not report contacts with static or kinematic bodies unless
`Generate All Kinematic Contacts` is set; contact impulses are estimates.

## 3.3 Alternatives, in both directions (`REUSE_POLICY.md` §§11–12, §17)

| Option | Verdict | Why |
| --- | --- | --- |
| **Rapier** (`rapier3d`, `enhanced-determinism`) | **Keep, proposed DEP-13.** | Pure Rust, Apache-2.0, active, a cross-platform determinism mode documented by its authors, a kinematic character controller that is exactly "a person walks and stops at things", dynamic bodies for kicked and thrown objects, serde snapshots if ever needed. Proven against our criteria in §9. |
| **`parry` alone** (geometry and queries, no dynamics) | **Rejected as the whole answer; it is inside Rapier anyway.** | Shape casts answer "how far can this capsule go" (R-1), but kicking and throwing (R-4) need integration, contacts and friction — the dynamics we would then write ourselves on top of parry. Rapier *is* parry plus that. Using only Rapier's query half for strides (mode `ql`, §9) is the same economy without a second dependency. |
| **Our own minimal circle separation and push code** | **Rejected**, recorded as `REUSE_POLICY.md` §12 requires. | For R-1 alone, circles on a plane plus axis-aligned walls are perhaps 200 lines. R-4 is not: a kicked box sliding into a wall and a person, a thrown object landing on a table, stacking — a sliding, colliding, resting integrator with friction is a physics engine, "commodity infrastructure" (`REUSE_POLICY.md` §4). The custom code would also lack the documented cross-platform determinism we get from `enhanced-determinism` and would have to prove its own. Valid reasons from §12 that apply: "missing required semantics" (dynamics) and "dependency larger than the problem" does **not** apply, because the problem includes dynamics. |
| **Jolt on the server, via Rust bindings** (`joltc-sys` / `rolt`) | **Rejected.** | Newest `0.3.1+Jolt-5.0.0`, published 2024-05-19, no release since; the README says "early work in progress. Watch for exposed nails" and safety is "best-effort"; the crates expose no way to set `JPH_CROSS_PLATFORM_DETERMINISTIC`. C++ through FFI adds a toolchain and `unsafe` to the server (`ENGINEERING_STANDARDS.md` §3). "Unmaintained project" and "inability to isolate it cleanly" (§12). The only argument for it — the same engine on client and server — does not apply, because the client is not authoritative and does not need bit-equality with the server. |
| **Avian** (`avianphysics/avian`, MIT/Apache-2.0) | **Rejected.** | "an ECS-driven 2D and 3D physics engine for the Bevy game engine… Made with Bevy, for Bevy… it shouldn't need to maintain a separate physics world." Adopting it means adopting Bevy's ECS and scheduler — the framework lock-in `REUSE_POLICY.md` §3 forbids and the opposite of `DEP-1`. Its collision detection is parry, which we get through Rapier. |
| **Rapier in the Godot client too** (`appsinacup/godot-rapier-physics`, MIT, "Locally Deterministic", "Cross Platform Deterministic", `enhanced-determinism` on) | **Not now; recorded as the route if client prediction ever must match the server bit for bit.** | Tempting symmetry, but the client does not need it: the server's answer always wins (§7). Its README says "the 3D part is still missing some things". Jolt is built into 4.7.2 and is the engine's default. Revisit only if reconciliation corrections become visible enough to matter (R-B4). |
| **Jolt in Godot** (built in) | **Keep, proposed DEP-14.** | Built into the engine we already use (`DEP-4`), MIT, the default for new projects since 4.6, and `CharacterBody3D` already does move-and-slide on it. Nothing new to install. |
| **`rapier2d`** instead of `rapier3d` on the server | **Open, QB-4.** | Everything on the server is on a floor today (the slice never reports height, `REPORT_HEIGHT = false`), so a 2D world of circles and boxes would answer R-1 and pushing, and costs less. A thrown object landing *on* a table, and later stairs and ramps, need the third dimension. Recommendation: `rapier3d` with z up, people as capsules held at floor height. |

The DEP records themselves are proposed in §15.

---

# 4. Question 2 — Ownership and the boundary with `movement`

## 4.1 The four things that stay distinct (`CLAUDE.md` §4 rule 14)

```text
MoveIntent          `move { to }`                           movement validates it: may this
                                                            person ask to go there? (stride,
                                                            passage)
Travel Process      not built; ends in presence's arrived   unchanged by this step
Spatial state       Presence (presence); where a loose      one writer each
                    object lies (bodies)
Rendered movement   the client's CharacterBody3D / sprite   never authoritative
```

This step adds a fifth thing between the first and the third, and names it so it is not confused with
either: **bodily resolution** — what a body does when it gets where it was allowed to go. It is not an
intent (nobody asks for it), not state (it writes no position of a person), and not rendering.

## 4.2 Options considered

```text
(A) movement consults bodies       movement → bodies → presence. movement's validate calls a
                                   bodies function (like presence's admit) and its resolve
                                   states the clipped arrival.
(B) a kernel geometry seam         the kernel lets one installed system be the world's geometry
                                   provider; WorldRead exposes "sweep this entity from here to
                                   there" and line of access; movement calls it.
(C) bodies resolves every arrival  bodies depends on presence, subscribes to presence's arrived,
                                   and corrects it: a stride that ran into a person or a wall
                                   is stated again, stopped at the contact; a box in the way is
                                   pushed. movement is untouched.
(D) bodies provides its own move   refused by the registry (one provider per action), and two
                                   ways to move would be ARC-26's "distance rule beside an
                                   unrestricted relocation", which is not a rule.
(E) a persistent physics world     one Rapier world per place lives as a component, stepped
                                   by a process every simulated second or faster.
```

| | A | B | C | E |
| --- | --- | --- | --- | --- |
| Kernel or contract change | no | **yes** (operator-material) | no | no |
| Edit to an existing pack | movement (dependency + calls) | movement (calls) | presence: `arrived` gains `from` | none |
| `bodies` removable (`AC-2`) | **no**: movement depends on it, so every world with walking must install it | yes | yes | yes |
| Uniform for every mover (R-2) | only movers that call it; a future travel pack or `shove` must remember | only callers of the seam | **yes**: every `arrived`, whoever stated it, passes through bodies | yes |
| Can refuse a blocked stride | yes | yes | no: corrects after acceptance | no |
| Log contains a transient overlap | no | no | **yes**: `arrived X` then `arrived X′` in the same instant, caused by the first | yes, continuously |
| Idle cost | none | none | none | stepping every place forever; ruled out by PC-d (§9.4) |

## 4.3 Recommendation: (C), with the transient stated rather than hidden

C is the only option that makes non-interpenetration a property of the **world's state** rather than of
each mover's validation, which is what R-2 asks for: whatever states an arrival — `move`, a future travel
process, `shove`, a genesis placement — bodies resolves it, and no pack can forget to ask. It needs no
kernel change, keeps `movement` untouched and `bodies` removable, and follows `ARC-26` exactly: bodies
depends on presence, states presence's `arrived` through presence's `arrival()`, and presence still
decides what its state may hold.

Its cost is the transient. In one instant the log reads:

```text
#n    arrived        Alice  cafe (3 200, 4 000)     stated by movement, caused by Alice's move
#n+1  stride-blocked Alice  wanted (3 200, 4 000)   stated by bodies, caused by #n
                            reached (2 710, 4 000)
                            by Bob
#n+2  arrived        Alice  cafe (2 710, 4 000)     stated by bodies, caused by #n
```

Every fact is true as a statement of what was decided: movement allowed the stride, the body stopped at
Bob. No observation ever shows the intermediate *state*: one thread owns the world
(`server/src/host.rs`), `world.dispatch` runs the whole cascade synchronously, and the server sweeps
observations only after it returns (`server/src/runtime.rs`, `self.sweep()` after `dispatch`). An
observer entitled to the facts does receive all three in that observation's events, in order, and a
client that animates facts must read the last `arrived` as the outcome. A biography that lists arrivals
sees two; that is the honest record of a stride that was cut short. This is recorded as proposed `ARC-39`
(§15) and as QB-1, because it is the decision the operator must accept or refuse.

**Why the reaction terminates.** The corrective arrival is computed as a position at which the body
overlaps nothing it may not overlap, and it lies on the stride's own path, so it is within
`MAX_STRIDE` of the start. When bodies receives that arrival in turn, the stride from `X` back to `X′`
meets nothing, the resolution equals the arrival, and bodies emits nothing. Depth 3 of 16.

**What (C) needs from presence.** A subscriber cannot see a stride without its start (§2.2). So
`Arrived` gains `from: Option<Location>`, filled by presence's own constructor `arrival()` from the
`Presence` it already reads, and `None` at a first placement. No stating system changes its call.
Presence's `VERSION` rises (3), `Arrived`'s schema version rises (2), and saves from before are refused
by name (`ARC-25`, "versions are refused, never guessed"). Every existing world's fact bytes change,
because every `arrived` now carries `from`: the social-cafe and market-town seed-7 digests are
re-baselined in the PR that makes the change, with a statement that nothing else in them changed
(QB-2, operator-material).

**Fallbacks, if the operator refuses C.** A is the next best: the same `bodies` pack, called by
movement; it gives up `AC-2` removability of bodies and uniformity for future movers. B is the cleanest
in the long run — line of access, which `CORE_CONCEPTS.md` §6.3 has been waiting for, would come with it
for every pack — but it is a kernel change and is not needed for R-1 to R-4. Recommended as a later step
when line of access is needed (QB-9).

## 4.4 Ownership

| State | Owner | Written only while | Notes |
| --- | --- | --- | --- |
| Where a person is (`Presence`, `present-in`) | **presence** | reducing `arrived` | Unchanged. bodies states `arrived` for a blocked stride or a shoved person; presence reduces it and may refuse it. |
| A person's body: shape (capsule radius and height) | **bodies** | reducing `body-formed` (genesis) | Every person in a world with bodies has one: an authored `body:` section or the default human capsule, r 300 mm, height 1 720 mm (the 3D client's own, `player.gd`). |
| A place's fixed geometry: walls, counters, tables as boxes in the place's frame | **bodies** | reducing `place-shaped` (genesis) | Authored as the place's `body:` section (`ARC-31`). A doorway is a gap. Static for MVP; doors that open are a later pack. |
| A loose object's body and where it lies (place and local position) | **bodies** | reducing `object-placed` (genesis) and `object-moved` | See QB-3: an Item file is a kind (`ARC-36`); a loose object is one Item entity whose `body:` section places exactly one physical instance. |
| Velocity, contacts, sleep state | **nobody** | — | Never persisted. Everything is at rest between resolutions (§6.1). |
| Passages | movement | unchanged | bodies does not read them; a doorway must simply be a gap in the place's geometry, which the world validator checks (§10.4). |

**Single ownership holds.** No component has two writers. bodies never writes `Presence`; it states a
fact presence reduces. Movement never writes anything new.

## 4.5 Facts (`ARC-26` for cross-vocabulary emission)

```text
bodies' own vocabulary (owner and only reducer: bodies)
  body-formed      genesis     a person or an object has this shape
  place-shaped     genesis     a place has this fixed geometry
  object-placed    genesis     a loose object lies here
  stride-blocked   reaction    a body stopped short of where it was going
                               { person, wanted: Location, reached: Location,
                                 by: Option<EntityId> }    by = the person or object first hit;
                                                           None for fixed geometry
  object-moved     reaction    a loose object moved      { object, from, to, how, by: PersonId,
                   or action                               path: Vec<LocalPosition> }
                               how ∈ { pushed, kicked, thrown }
  person-shoved    action      { by, person, from, to }

stated by bodies in presence's vocabulary (owner: presence; ARC-26)
  arrived          the corrected position after stride-blocked, and the shoved person's new
                   position after person-shoved. Built by presence's arrival(); presence
                   may refuse.
```

`path` is a short list of quantized keyframes (at most one per 0.1 s of simulated motion, at most 40)
for a client to animate an object's flight. It is presentation data stated by the owner because only the
owner knows it; the client may ignore it and draw the end position.

Bodies subscribes to: `arrived` (presence's) and its own genesis facts.

---

# 5. Question 3 — Semantic space and continuous space

- **A Place is semantic; bodies are continuous inside one.** A resolution is always **per place**: the
  Rapier world built for it contains the place's fixed geometry, every person whose `Presence` is in that
  place with a local position, and every loose object lying there. Nothing from another place is in it.
  This follows `CORE_CONCEPTS.md` §6.2: two positions are comparable only within one place's frame.
- **A world without positions loses nothing.** An arrival with `local: None` is not resolved (there is no
  body to place), so a semantic or 2D-without-positions world behaves exactly as now. This is the same
  degeneracy `SpatialRequirement::evaluate` already has.
- **Passages.** A stride that changes place starts in one frame and ends in another. Bodies does not sweep
  across frames: for an arrival in a new place it resolves the destination only — if the body overlaps
  something there, it is moved to the nearest free position, starting from the arrival point (a
  zero-length character-controller move depenetrates, `move_shape`'s step 1). The doorway on the far side
  must be clear in the place's authored geometry; the world validator checks that every passage's `there`
  point is free for the default human capsule (§10.4). A crowd in a doorway is resolved by the same
  depenetration: the arriving person is placed beside the crowd, not inside it.
- **Travel between towns** stays a Process that ends in `arrived` (`ARC-26`). Bodies resolves its end like
  any other arrival. Nothing here prevents it.
- **Height.** z is the floor for people (positions keep `z = 0`; the capsule is held at floor height);
  objects may come to rest above the floor (on a counter), so an object's `z` is meaningful. The 3D client
  already never reports height (`REPORT_HEIGHT = false`), and a jump stays rendered movement.

---

# 6. Question 4 — Determinism and persistence

## 6.1 Recommended model: integer state, resolution rebuilt each time (mode Q / `ql`)

```text
between resolutions   integers only: Presence (presence), body shapes, place geometry, where each
                      loose object lies (bodies). All at rest. No velocity, no contact cache, no
                      Rapier state anywhere — in no component, no field, no global.
a resolution          build a Rapier world for one place from those integers, inserting in a fixed
                      canonical order (geometry in authored order; people by EntityId; objects by
                      EntityId) → run the character controller and, only if something dynamic is
                      involved, at most a fixed number of fixed-dt sub-steps → quantize every
                      position to whole millimetres → state facts
```

Why this and not a persistent world (mode P):

1. **It is what a System can be.** Systems keep no fields (§2.3). A persistent Rapier world would have to
   be a component: an opaque serialized blob (against `CORE_CONCEPTS.md` §3.1, "typed fields, never an
   untyped blob"), re-encoded to JSON on every snapshot, and changing whenever Rapier's internal layout
   changes.
2. **It costs nothing when nothing happens.** Mode P must be stepped through idle time; PC-d measured it
   at about 190 µs per step for one café, which is about **82 hours** of wall time per café per 300
   simulated days at 60 Hz (§9.4). The discrete-event clock (`DEP-6`) exists to avoid exactly that.
3. **Persistence needs nothing new.** The state is a few typed integer components, snapshotted as every
   other component is. PC-b(Q) resumes from a 127-byte snapshot byte for byte.
4. **It keeps `CORE_CONCEPTS.md` §6.2.** Floats exist only inside one resolution; everything persisted is
   `i32` millimetres.

What it gives up: momentum between resolutions. A kicked ball does not keep rolling into the next
second; its whole motion is resolved at the instant of the kick, and its path is stated for clients to
animate (§4.5). That is the "not realistic" the operator allowed (R-3). A deferred landing fact, so the
ball arrives when its flight would end, is QB-6.

## 6.2 Floats, fixed point, or quantized floats

**Quantized floats.** Inside a resolution, Rapier's `f32` with `enhanced-determinism`; at its boundary,
round to whole millimetres. Fixed-point physics was considered and rejected: no maintained fixed-point
Rust physics engine exists, and writing one is the custom engine §3.3 rejects. Quantization alone does
not make float determinism unnecessary: a result one ulp apart can round to a different millimetre. It
does limit how far a difference can travel, and §9.4 found that quantized state pinned against geometry
**forgets** small differences (PC-c′, PC-c″) — which is why the world's replay check must compare facts
(as `ARC-25` already does), not only a final state.

## 6.3 Conditions bodies must hold, from the Rapier documentation and the prototype

```text
DC-1  `rapier3d` pinned to an exact version (`=0.36.0`) with `enhanced-determinism`; never `simd8`.
      `parallel` is not used (not needed at this scale; allowed by the docs).
DC-2  Insertion order canonical and documented. PC-c1 shows it matters in both modes.
DC-3  No `sin`, `cos`, `sqrt` or similar from `std` on any value that reaches Rapier: the Rapier page
      names these and asks for nalgebra's `ComplexField`/`RealField` instead. Directions (a kick away
      from the kicker, a throw toward a point) are normalized through those, or computed in integers
      before conversion. (The prototype used literal unit vectors and so did not exercise this.)
DC-4  Fixed dt (1/60 s) and fixed bounds on sub-steps per resolution; the settle loop stops at a
      bound or at rest, both pure functions of the state.
DC-5  A Rapier upgrade is a change of results: it bumps bodies' VERSION, so an old save is refused by
      name instead of diverging on replay.
DC-6  Every quantization is `round()` of `metres × 1000` to `i32`, in one function.
DC-7  Workaround for the defect found in §9.4 F-P1, with a regression test in the pack.
```

## 6.4 Headless cost

Movement volume, measured on main: a 30-day social-cafe run accepted 16 595 moves (`overall.md` §7,
10a), so about **166 000 arrivals per 300 days**. Every one would pass through bodies.

```text
mode Q    160.9 µs per request (§9.4)  →  ≈ 26.7 s per 300-day run
mode ql    58.6 µs per request (§9.4)  →  ≈  9.7 s per 300-day run
```

Against current 300-day runs of 15.2 s (market-town, 11d) to 33.7 s (11e with `--save`), `ql` adds
roughly 30–65 %. Two further reductions are designed in, not yet measured: skipping Rapier altogether
when no other body's box meets the stride's box (an integer test), which is most strides in an open
street; and building the place's fixed geometry once per resolution from a pre-sorted list. The bound
to hold in PR 12b's acceptance is stated there (§11, QB-11), not derived from what the code happens to cost.

Only a world that installs bodies pays. `social-cafe` and `market-town` do not install it in this step
until PR 12c (§11), so until then their run times are unchanged except for the `from` field.

---

# 7. Question 5 — The server decides; the client predicts

## 7.1 What Godot/Jolt does locally (the 3D client)

1. **Select Jolt explicitly**: `[physics] 3d/physics_engine="Jolt Physics"` in `project.godot`, since the
   project predates 4.6's default (§3.2).
2. **Collide with people.** Every drawn person gets an `AnimatableBody3D` with the same capsule the server
   uses (r 0.30 m, 1.72 m), on a new layer `LAYER_BODIES`, moved to its observed position. The player's
   `collision_mask` gains `LAYER_BODIES`. This alone removes the interpenetration the operator saw.
3. **Loose objects.** Drawn at their observed position; collision layer `LAYER_OBJECTS`, which the player
   does **not** mask in the first version: walking into a box lets the player's body enter it briefly
   until the server's `object-moved` arrives at the next observation (≤ 100 ms). Local push prediction
   with a Jolt `RigidBody3D` is QB-7.
4. **Reconcile on difference, not only on refusal.** When an observation places the observer more than
   **150 mm** from where the client last reported it (more than the server's quantization and the
   controller's 10 mm gap, less than a visible jump), the client moves its body to the authoritative
   position. That covers a stride stopped short by the server, a shove, and a refusal, with one rule.
   It replaces the refusal-only reconciliation in `slice_link.gd` (`ADOPTION.md` gains the rule).
5. **Animate objects.** An `object-moved` with a `path` is drawn along it over its stated duration.

Jolt's lack of determinism in Godot does not matter: nothing the client simulates is authoritative.

## 7.2 How a client asks for push, kick, throw (intents only, rule 15)

```text
push    nothing to ask: walking into an object is a move; the server's resolution pushes it
kick    { action: kick,  target: <object> }                          payload {}
throw   { action: throw, target: <object>, payload { toward: LocalPosition } }
shove   { action: shove, target: <person> }                          payload {}
```

The 3D client acquires the target by its existing camera ray (as `talk` does) and the throw point from
the ray's floor hit; the 2D client by clicking the object, and for a throw a second click on the floor.
Both send the same request. `kick` and `shove` are offered as complete affordances (`ARC-34`), so the
paced controller can attempt them without knowing them; `throw` carries a free-form position, so it is
not complete and is attempted only by a requester that knows it by name.

## 7.3 The 2D client, and the two gating questions

The 2D client (S12) needs no physics. It draws people and objects where observations put them, sends
`move` strides from clicks (as the S6 design describes), and sends `kick`, `throw` and `shove` as above.
Blocking and pushing are decided once, on the server, and the 2D client shows the result.

- **§11 — Can this support a Minecraft-like embodied 3D client without redesigning the kernel? Yes.**
  The kernel is untouched. The 3D client keeps local collision and prediction (now against people too),
  the server's resolution is the authority, and the reconciliation rule is one number. Height, stairs
  and objects at rest on furniture are representable because the pack works in three dimensions (QB-4).
- **§§9, 22 — Can 2D and 3D use the capability without duplicating game logic? Yes.** No rule about
  blocking, pushing, reach or force exists in either client. The 3D client's local collision is
  prediction of a result the server states; removing it changes no outcome, only how soon the player
  sees it. `kick`, `throw` and `shove` are the same `ActionIntent`s from both, differing only in
  acquisition (`AC-13`).
- **§12 — Does a physics concept enter a generic contract? No.** Rapier types live in `systems/bodies`.
  The facts carry `PersonId`, `ItemId`, `Location`, `LocalPosition` and `Millimetres`.

---

# 8. Questions 6 and 7 — Interactions in scope, and change amplification

## 8.1 Interactions in scope, deliberately simple (R-3)

| Interaction | How | Declared requirement (`SpatialRequirement`) | Refusals |
| --- | --- | --- | --- |
| People never interpenetrate | Every arrival resolved: the character controller sweeps the capsule from `from` to `to` against fixed geometry and other people; it slides along them and stops at contact. | none (a reaction, not an action) | — |
| A person pushes an object aside by walking | If the stride's box meets a loose object, the person is carried along the stride over fixed sub-steps as a kinematic body and the object, dynamic, is pushed; then the person is depenetrated, so an object jammed against a wall stops the person instead of being entered (PC-e2). | — | — |
| `kick` an object | Impulse away from the kicker (direction from the kicker's centre to the object's, so it needs no aim), fixed strength, resolved until rest or 180 sub-steps (3 s). | same place, within 800 mm, target available | `TooFarAway`, `TargetUnavailable` (not a loose object, or not in the place), `malformed-payload` |
| `throw` an object | The thrower picks up and throws in one action: the object starts at chest height in front of the thrower with a velocity toward `toward`, clamped to 6 m, and is resolved until rest or 240 sub-steps (4 s). | same place, within 800 mm, target available | as kick, plus `PreconditionFailed` when `toward` is outside the place's geometry bounds |
| `shove` a person | The target is moved 500 mm away from the shover by the character controller (so a wall or a third person stops them), stated as `person-shoved` plus presence's `arrived`. | same place, within 1 000 mm, target available | `TooFarAway`, `TargetUnavailable`; `Busy` while the target is in a process its owner marks uninterruptible is out of scope (no such query exists, QB-8) |

Not in scope: knocking a person over, health, damage, objects hitting people having consequences beyond
being stopped by them, carrying (an inventory concern), doors, ragdolls, vehicles. A thrown object that
hits a person stops or bounces off the person's capsule; nothing else happens to the person.

## 8.2 Change amplification (`CLAUDE.md` §4 rule 5)

What the step adds:

```text
systems/bodies/                 the pack: components, facts, actions, resolution, Rapier behind it
systems/installed               one entry (ARC-33)
worlds/<a world with bodies>    body: sections on places and objects; see §11, PR 12c
clients/ (3D, and the 2D when it exists)   colliders for people, reconciliation, intents
docs                            ARC-39, DEP-13, DEP-14, MODULE_SPEC §4.1 (the body: section),
                                server/PROTOCOL.md §6.2 (reconciliation), ADOPTION.md
```

What it edits that already exists, and only this:

```text
systems/presence   Arrived gains `from`; arrival() fills it; VERSION 3. Operator-material (QB-2).
```

What it must **not** edit: `kernel/`, `contracts/`, `persistence/`, `server/src` (the server carries any
action unchanged; only the document `server/PROTOCOL.md` changes), `systems/movement`, any other System Pack, `worldpack` (sections
are bound through `ARC-31`), `cognition/` (the paced controller attempts complete affordances it was
never compiled against, `ARC-34`). If any of these turns out to need an edit, the work stops and the
question returns to the operator.

**One kernel-adjacent fact to verify in PR 12a, not assumed:** that a system may subscribe to a fact type
it also states (bodies subscribes to `arrived` and states `arrived`). `ARC-26`'s install check requires
only a declared emission and a dependency on the owner. If the kernel refuses this combination, that is a
kernel change and goes to the operator.

---

# 9. The determinism prototype

The prototype answers one risky assumption before any architecture is built on it
(`REUSE_POLICY.md` §15): **can Rapier's results be reproduced byte for byte across two runs, across a
snapshot and a resume, and is the comparison able to see a difference at all?** Without that, a
physics System Pack cannot satisfy `ARC-25`, under which a resumed world re-executes its journal and
must regenerate every logged fact byte for byte.

## 9.1 What it is

A standalone Cargo binary at `/Users/yuema137/mineworld-demos/physics-spike/`, depending on
`rapier3d 0.36.0` with the features `enhanced-determinism` and `serde-serialize`, plus `bincode 2`,
`serde` and `sha2`. It depends on no MineWorld crate. It is never committed to MineWorld; its source
is recorded in §9.5.

The scene is café-sized, matching `worlds/social-cafe/places/cafe.yaml` (8.32 m × 10.32 m inside):

```text
floor        one fixed cuboid at z = 0, the place's floor (z up, CORE_CONCEPTS §6.1)
walls        four fixed cuboids around the room, 0.2 m thick
obstacles    a counter (fixed cuboid) across the back of the room
people       15 capsules, radius 0.30 m, height 1.72 m (the 3D client's own capsule,
             `player.gd`), kinematic: they are moved, they are not knocked over
props        4 dynamic bodies: two boxes (0.4 m cube), one ball (r 0.11 m), one crate (0.6 m)
inputs       a pure function of the step or request index: which person moves, by how much,
             and in which direction, from a fixed linear congruential sequence
```

It runs in two modes, because the design (§6) must choose between them:

```text
P  persistent   one Rapier world lives for the whole run and is stepped at a fixed
                dt = 1/60 s. People move by the kinematic character controller each step;
                props are pushed by the character impulses Rapier provides
                (`solve_character_collision_impulses`) and by scripted kicks. The state
                compared is Rapier's whole serialized PhysicsWorld (bincode).
Q  per request  the authoritative state is integers only: every body's position in
                millimetres (`i32`, as `LocalPosition`). Each request rebuilds a Rapier world
                from that state, inserting bodies in a fixed id order, moves one person by
                a stride of at most 2 m with the character controller, lets touched props
                settle for at most 120 fixed steps, and quantizes every position back to
                whole millimetres. Nothing of Rapier survives between requests. The state
                compared is the integer state.
```

Mode Q is what a System Pack can be: `System` methods take `&self` and a system keeps no fields
(`kernel/src/system.rs`, "a system that kept simulation state in its own fields would be state no
event log could reconstruct"), so a Rapier world that lived between requests would have to be a
component.

## 9.2 Criteria, stated before any run (`ARC-23`)

Each criterion names its measurement, its bound and its verdict rule. Bounds are literals fixed
here, never derived from the quantity measured (`ARC-23` point 2). A run that has not happened is not
a pass (`CLAUDE.md` §3.1).

| ID | Claim | Measurement | PASS iff |
| --- | --- | --- | --- |
| **PC-a** | Same inputs, same bytes. | Run each mode twice, **as two separate processes**: P for N = 6 000 steps (100 s of simulated time), Q for R = 3 000 requests. Print the SHA-256 of the final serialized state. | Both processes print the same digest, for each mode. |
| **PC-b** | Snapshot, drop, restore, continue = uninterrupted. | P: write the serialized PhysicsWorld at step K = 2 500 to a file and exit; a **new process** reads it, restores, and continues to N. Q: the same with the integer state at request K = 1 300, continuing to R. | The resumed digest equals the uninterrupted digest from PC-a, for each mode. |
| **PC-c** | The comparison can see a difference. | Three deliberately different variants, each run as its own process: **c1** the people inserted in reverse order; **c2** one prop's initial x moved by 1 mm (P: 0.001 m; Q: 1 mm); **c3** one input of request/step 1 000 changed by 1 mm. The instrument must also **locate** the difference: print the first body whose state differs and by how much. | c2 and c3 each produce a digest different from PC-a's, and the locator names a body. c1 is a finding either way: Rapier's documentation says insertion order is part of the initial conditions, so a difference is expected in P; whatever is observed is recorded, not tuned. If c2 or c3 shows **no** difference, the instrument is broken and every other result in this section is void. |
| **PC-d** | What it costs. | Release build, this machine (Apple silicon). P: mean wall time per step over the N steps. Q: mean wall time per request (build + move + settle + quantize) over the R requests. | Reported, not passed or failed, against two stated budgets: P is ruled out for the headless server if stepping one café at 60 Hz through 300 simulated days (1.555 × 10⁹ steps, because a stepped world cannot skip idle time the way the discrete-event clock does, `DEP-6`) would cost more than **60 s** of wall time, i.e. more than **38.6 ns per step**; Q is acceptable if its mean is at most **100 µs per request**, which adds at most 17 s to a 300-day run at the measured movement volume (§6.4). |
| **PC-e** | Bodies do not interpenetrate; a person pushes a box. | e1: two capsules walking straight at each other along one line, 1.45 m/s each, for 4 s, in both modes. e2: one person walking straight into a 0.4 m box, in both modes. | e1: the smallest centre-to-centre distance in the plane over the whole run is at least **2 × 0.30 m − 0.005 m = 0.595 m** (overlap of at most 5 mm). e2: the box's final position is at least **0.100 m** further along the walking direction than where it started, and the person never overlaps it by more than 5 mm. |

Two further checks are listed because they cost nothing to state and bear on `AC-8`; they are not
part of the operator's five:

| ID | Claim | PASS iff |
| --- | --- | --- |
| **PC-f** | Optimization level does not change the result. | The digests of PC-a are identical when the binary is built with the dev profile at `opt-level = 1` (what `ARC-30` makes the workspace's dev profile) and with the release profile. |
| **PC-g** | Another architecture gives the same bytes. | The same digests from an `x86_64-apple-darwin` build run under Rosetta. Requires installing that target, which this session is not permitted to do (`rustup` is outside its allowed commands), so PC-g is expected to be `NOT RUN` and is recorded as such. |

## 9.3 Criteria added after the first runs, before running them

The first runs (§9.4) produced two results that the criteria above do not settle, so two criteria
are added here **before** the runs that answer them. Neither replaces or relaxes a criterion of §9.2;
the §9.2 verdicts stand as recorded. (This section was numbered 8.2a when committed; the document's
sections were renumbered afterwards, not its content.)

- In mode Q, variants c2 and c3 ended with the **same** digest as the baseline. By §9.2's own rule,
  the Q results are void until the instrument is shown to see a difference.
- Mode Q costs more than its 100 µs budget. Most of the cost is stepping the world through a whole
  stride at walking speed, even when nothing dynamic is in the way.

| ID | Claim | Measurement | PASS iff |
| --- | --- | --- | --- |
| **PC-c′** | In Q, the comparison sees a perturbation where it takes effect; an equal final digest is then convergence, not blindness. | `q trace <variant>` writes the SHA-256 of the integer state after every request. Compare the trace of base with c2's and c3's: report the first and the last request at which they differ. | c2 differs from request 0 onward (prop 0 starts 1 mm apart) and c3 differs from request 1 000 onward (that request's stride is 1 mm longer), each for at least one request. If the traces never differ, the instrument is broken and every Q result is void. If they differ and later agree, the result is recorded as **convergence**: Q forgets a perturbation once quantized state is pinned against geometry. |
| **PC-c″** | (added after PC-c′ located c3's request 1 000 as a move straight into the west wall, which no 1 mm change can alter) A perturbation that takes effect is seen. | Variant **c4**: the first request at or after 1 000 that is a move and that, in the base run, moves its person by at least 100 mm, gets 1 mm more stride in x. Traces compared as in PC-c′. | The c4 trace differs from base from that request on, for at least one request. |
| **PC-d2** | A leaner Q fits the budget. | Mode `ql`: as Q, except that a move steps the world only when the swept path's planar box meets a dynamic prop's; otherwise the person is placed at the controller's answer with no stepping. Mean time per request over R = 3 000, release; digests from two separate processes. | Mean at most **100 µs per request**, and the two processes print the same digest. Reported beside Q, never as Q's result. |

## 9.4 Results (2026-10-07, Apple silicon, rustc 1.97.1, rapier3d 0.36.0)

Logs: `/Users/yuema137/mineworld-demos/physics-spike/out/` — `e.log`, `runs-release.log`, `runs-2.log`,
`runs-3.log`, `runs-dev.log`. Every run below is a separate process.

### Verdicts

| ID | Verdict | Evidence |
| --- | --- | --- |
| PC-a | **PASS** (P and Q) | P, N = 6 000: `9ccd94176937fe20…f4d87b3` twice. Q, R = 3 000: `57863631818e04d1…a90772cb` twice. |
| PC-b | **PASS** (P and Q) | P: snapshot at step 2 500, 22 545 bytes; a new process resumed to 6 000: `9ccd9417…` = PC-a. Q: snapshot at request 1 300, 127 bytes; resumed to 3 000: `57863631…` = PC-a. |
| PC-c | **P: PASS. Q: FAIL as specified** (c2 and c3 equal to base), which voided Q until PC-c′/PC-c″ | P: c1 `be7a743b…`, c2 `d8404d81…`, c3 `ed05db47…`, each different; locator: 19 of 19 bodies differ, first `person 0` (c1: y 8.093 07 vs 4.044 774). Q: c1 `58c3ab1c…` different (`person 0` x 2 035 vs 1 621 mm); **c2 and c3 `57863631…`, equal to base**, 0 of 19 bodies differ. |
| PC-c′ | c2 **PASS (convergence)**; c3 **FAIL as specified, cause located** | c2: traces differ after `init` through request 38 (40 of 3 001 lines), then agree. c3: traces never differ. Located with `q show 1000`: request 1 000 is `move person 1 by (−1890, 0) mm from [310, 4389]` — a person already against the west wall (x = 300 mm radius + 10 mm controller gap) walking into it. The controller moves them 0 mm with or without the extra millimetre: the perturbation never took effect. |
| PC-c″ | **PASS** | c4 perturbs request 1 001 (`person 2` moved (929, 378) mm in base). Traces differ at request 1 001, and agree again from 1 002. Located: request 1 002 moves person 2 by (1 245, 1 245) mm to (8 010, 6 260) — the east wall (8 320 − 300 − 10). |
| PC-d | **P ruled out** (≈ 5 000× over budget). **Q over budget.** | P: 195 351 and 187 687 ns per step (base); 171–193 µs across variants. At 60 Hz for 300 days that is ≈ 2.95 × 10⁵ s ≈ **82 hours** per café. Q: 160.9 and 172.8 µs per request against 100 µs; 2 548 of the 3 000 requests were moves, every one stepped. |
| PC-d2 | **PASS** | `ql`: 58.6 and 58.5 µs per request; digest `2ca7c6fe8f16eb00…4b12b72fe` in both processes; only 263 of 2 548 moves met an object and stepped. (Its digest differs from Q's because it is a different algorithm, as expected.) |
| PC-e1 | **PASS** (P and Q) | Smallest centre distance: P 0.6045 m, Q 0.6100 m; bound 0.595 m. Q's final positions (4 000, 5 000) and (4 610, 5 000): the two stop 10 mm apart (the controller's offset). |
| PC-e2 | **P: FAIL. Q: PASS** | P: the box moved 5.121 m (≥ 0.100), but the person overlapped it by up to 0.179 m. Located (`DEBUG=1`): the overlap begins when the box reaches the east wall (box x 8.121 = 8.32 − 0.2) and the kinematic person, which ignores dynamic bodies so that it can push them, walks on into it. Q: the box moved 5.135 m and the worst gap was 0.000 m: each request ends by depenetrating the mover, so the person stops at the jammed box (person 7 625 mm, box 8 135 mm). |
| PC-f | **PASS** | Dev profile at `opt-level = 1`: P `9ccd9417…`, Q `57863631…`, `ql` `2ca7c6fe…` — identical to release. (Times: Q 201.7 µs, `ql` 72.0 µs.) |
| PC-g | **NOT RUN** | Needs `rustup target add x86_64-apple-darwin`, outside this session's permitted commands. QB-12. |

### Findings

- **F-P1 — a Rapier 0.36.0 defect, found and worked around.** On the first runs PC-e2 showed a box that
  never moved, and a box placed in the air did not fall. `PhysicsWorld::detect_collisions` on a fresh
  world, before its first `step`, leaves every dynamic body un-integrated. Cause, from source:
  `CollisionPipeline::step` takes the bodies' "modified" flags and calls
  `handle_user_changes_to_rigid_bodies(None, …)` with no island manager
  (`rapier3d-0.36.0/src/pipeline/collision_pipeline.rs` lines 173–188), so the next physics step never
  registers them; `wake_up_all` does not help. Workaround: re-mark each dynamic body as modified
  (`set_translation(current, true)`) after `detect_collisions`. Keeping the instrument: `NOWAKE=1`
  reproduces the defect (`BOXZ=0.8` box stays at 0.800; with the workaround it falls to 0.291 within
  20 steps). Mode Q needs `detect_collisions` because its queries run before any step. To be reported
  upstream; DC-7.
- **F-P2 — insertion order changes results in both modes** (PC-c1). Rapier's documentation says so; the
  prototype confirms it applies even to a world rebuilt per request. Canonical order is an invariant
  (I-4).
- **F-P3 — quantized, geometry-bound state forgets.** A 1 mm difference disappears within one or a few
  requests once a body is stopped by a wall (PC-c′, PC-c″). Two consequences: comparing only the final
  state of a long run is a weak instrument, and the trace — the facts each request regenerates, which is
  exactly what `ARC-25` compares — is the right one; and the world is robust to tiny differences, which
  is good, but not a substitute for determinism, because a difference that is not pinned by geometry
  persists (c2 lasted 38 requests).
- **F-P4 — a kinematic pusher must be depenetrated at the end of a resolution** (PC-e2 P vs Q). The
  persistent mode has no natural point to do so; the per-request mode does it once per request.
- **F-P5 — cost.** A persistent stepped world is out of the question for the headless server. The
  per-request model is affordable only if it steps the world when an object is actually involved (10 %
  of moves in this run); otherwise it is the character controller's query alone.

### Conclusion

Rapier with `enhanced-determinism`, rebuilt per resolution from integer state in a canonical order, gives
byte-identical results across processes, across a snapshot and resume, and across optimization levels,
and the comparison has been shown to see a 1 mm difference. Cross-architecture identity is the one claim
not yet evidenced (PC-g). The design adopts mode `ql` (§6.1).

## 9.5 Prototype source (excerpts) and commands

Standalone project, `/Users/yuema137/mineworld-demos/physics-spike/` (one file, `src/main.rs`, about 850
lines). Not in the MineWorld repository.

```toml
# Cargo.toml
[dependencies]
bincode = { version = "2", features = ["serde"] }
rapier3d = { version = "0.36.0", features = ["enhanced-determinism", "serde-serialize"] }
serde = { version = "1.0.229", features = ["derive"] }
sha2 = "0.11.0"

[profile.dev]
opt-level = 1
```

The scene and the person (`insert_statics`, `person_body`, `controller`):

```rust
const RADIUS: f32 = 0.30;
const HALF_SEGMENT: f32 = 0.56; // 1.72 m tall
const CENTRE_Z: f32 = 0.87;     // feet 1 cm above the floor
// eight unit directions as literals: no transcendental function at run time (DC-3)
const DIRS: [(f32, f32); 8] = [(1.0, 0.0), (0.707_106_77, 0.707_106_77), /* … */];

fn person_body(x: f32, y: f32) -> (RigidBodyBuilder, ColliderBuilder) {
    (RigidBodyBuilder::kinematic_position_based().translation(Vector::new(x, y, CENTRE_Z)),
     ColliderBuilder::capsule_z(HALF_SEGMENT, RADIUS))
}

fn controller() -> KinematicCharacterController {
    KinematicCharacterController { up: Vector::Z, offset: CharacterLength::Absolute(0.01),
        slide: true, autostep: None, snap_to_ground: None, ..Default::default() }
}

/// People and walls block; dynamic objects do not (they are pushed by the solver instead).
fn kcc_translation(world: &PhysicsWorld, me: RigidBodyHandle, desired: Vector) -> Vector {
    let filter = QueryFilter::exclude_dynamic().exclude_rigid_body(me);
    let queries = world.query_pipeline_with_filter(filter);
    let pos = *world.bodies[me].position();
    let moved = controller().move_shape(DT, &queries, &capsule(), &pos, desired, |_| {});
    Vector::new(moved.translation.x, moved.translation.y, 0.0)
}
```

Mode Q / `ql`, one request (the integer state is `StateQ { people: Vec<[i32; 2]>, props: Vec<[i32; 3]>,
request: u32 }`, millimetres):

```rust
fn apply(&mut self, req: &Request, variant: Variant) {
    let (mut world, people, props) = self.build(variant); // fresh world, canonical order,
                                                          // then refresh_queries()
    match *req {
        Request::Move { person, dx_mm, dy_mm } => {
            let me = people[person];
            let t = kcc_translation(&world, me, Vector::new(m(dx_mm), m(dy_mm), 0.0));
            if LEAN.load(Ordering::Relaxed) && !path_meets_prop(&world, me, t) {
                let next = world.bodies[me].translation() + t;  // ql: query only
                world.bodies[me].set_translation(next, false);
            } else {
                walk_kinematic(&mut world, me, t);   // stride at 1.45 m/s in 1/60 s sub-steps,
                                                     // then settle ≤ 60 steps
                depenetrate(&mut world, me);         // F-P4
            }
        }
        Request::Kick { prop, dir } => {
            let (dx, dy) = DIRS[dir];
            let h = props[prop];
            let mass = world.bodies[h].mass();
            world.bodies[h].apply_impulse(Vector::new(dx, dy, 0.6) * (4.0 * mass), true);
            settle(&mut world, &props, 180);         // until at rest or 180 sub-steps
        }
    }
    self.quantize(&world, &people, &props);          // (v * 1000.0).round() as i32
    self.request += 1;
}

fn refresh_queries(world: &mut PhysicsWorld) {      // F-P1
    world.detect_collisions(&(), &());
    for (_, body) in world.bodies.iter_mut() {
        if body.is_dynamic() {
            let t = body.translation();
            body.set_translation(t, true);
        }
    }
}
```

Mode P keeps `SimP { world: PhysicsWorld, people, props, step }` and serializes it whole with
`bincode::serde::encode_to_vec(&sim, bincode::config::standard())` for both the digest and the
snapshot.

Commands, run from the project directory (`B=target/release/physics-spike`):

```sh
cargo build --release
$B p e1; $B p e2; $B q e1; $B q e2                       # PC-e
$B p run base out/p-base-1.loc; $B p run base out/p-base-2.loc       # PC-a (P); same for q
$B p snap base out/p.snap; $B p resume out/p.snap out/p-resumed.loc  # PC-b (P); same for q
$B p run c1 out/p-c1.loc; $B diff out/p-base-1.loc out/p-c1.loc      # PC-c; c2, c3; same for q
$B q trace base out/q-base.trace; $B q trace c2 out/q-c2.trace
$B tracediff out/q-base.trace out/q-c2.trace                         # PC-c′; c3; PC-c″ with c4
$B q show 1000                                                       # locating c3
$B ql run base out/ql-1.loc; $B ql run base out/ql-2.loc             # PC-d2
cargo build && target/debug/physics-spike p run base out/p-dev.loc   # PC-f; q and ql too
DEBUG=1 $B p e2; NOWAKE=1 DEBUG=1 BOXZ=0.8 $B p e2                   # locating PC-e2 P; F-P1
```

---

# 10. Proposed contracts and invariants

## 10.1 Frozen invariants (proposed; frozen only by the operator or primary session)

- **I-1 The kernel and the contracts are untouched.** `kernel/`, `contracts/`, `persistence/` and
  `server/src` have no diff in any S15 PR, and neither has `systems/movement` or any System Pack other
  than `presence` (one change, I-7) and the new `bodies`. A need to edit any of them is a material stop.
- **I-2 Single ownership.** `Presence` and `present-in` are written only by presence's reductions.
  Body shapes, place geometry and where a loose object lies are written only by bodies' reductions. A
  change to where a person is travels only as presence's `arrived`, stated under `ARC-26`.
- **I-3 No float is persisted.** Every value in a component, fact or journal entry is an integer:
  millimetres, millidegrees, counts. Floats exist only inside one resolution.
- **I-4 Determinism conditions.** `rapier3d` pinned exactly, `enhanced-determinism` on, `simd8` and
  `parallel` off; canonical insertion order; fixed dt and bounded sub-steps; one quantization function;
  directions without `std` transcendental functions (DC-1 to DC-6).
- **I-5 Nothing of Rapier survives a resolution.** No component, field, global or cache holds Rapier
  state. A resolution is a pure function of the world's integer state and the request.
- **I-6 Uniformity (R-2).** In a world with bodies, every `arrived` with a local position is resolved,
  whoever stated it. No pack can bypass it, because bodies subscribes to the fact, not to an action.
- **I-7 Existing worlds change only by `from`.** After the presence change, every existing world's run
  differs from before only in the `arrived` payloads gaining `from`; every other line of the seed-7
  summaries is unchanged, and the re-baselined digests are recorded with that statement.
- **I-8 Clients decide nothing.** No rule about blocking, pushing, reach or force exists in a client. Local
  collision is prediction; removing it changes no outcome.
- **I-9 Removability (`AC-2`).** A world without bodies answers `kick`, `throw` and `shove` with
  `ActionUnavailable` and is otherwise the same world as before this step.
- **I-10 Locate before counting (`ARC-23`).** The non-interpenetration check reports the closest pair, the
  place and the instant, not only a count; and it is shown to fail on a world where bodies is disabled.

## 10.2 The pack, `systems/bodies` (crate `mineworld-bodies`, `SystemId "bodies"`)

```text
depends     presence
owns        BodyShape        on a Person or an Item: Capsule { radius, height } | Box { half
                             extents } | Ball { radius }, millimetres
            PlaceShape       on a Place: fixed boxes in the place's frame (min, max corners, mm)
            Lying            on an Item that is a loose object: its Location
provides    kick   { }                       target: an Item lying in the same place
            throw  { toward: LocalPosition } target: an Item lying in the same place
            shove  { }                       target: a Person in the same place
emits       body-formed, place-shaped, object-placed       genesis, from the `body:` sections
            stride-blocked, object-moved, person-shoved    reactions and actions
            arrived (presence's, ARC-26)
subscribes  arrived (presence's); its own genesis facts
authored    one section, `body:` (ARC-31), in person, place and item files (F-47: one section per
            pack)
discloses   PlaceShape to whoever perceives the place, and BodyShape and Lying of what the
            observation lists — so a client can build the same colliders the server uses (R-B4)
```

## 10.3 What presence gains (QB-2)

```text
Arrived { person: PersonId, location: Location, from: Option<Location> }   schema 2
arrival(world, person, location)   unchanged signature; fills `from` from the current Presence
PresenceSystem::VERSION            2 → 3
```

## 10.4 World Pack and validation

- `body:` on a place: `walls:` as boxes, in millimetres in the place's frame. On a person: optional shape
  override. On an item: `shape:` and `at:` (place key, x, y), which makes the item a single loose object
  (QB-3).
- The world validator (bodies' `AuthoredSection` refusals) refuses: a passage whose `there` point is not
  free for the default capsule; two authored people, or a person and an object, overlapping at genesis; an
  object outside its place's walls. An item with a `body:` that is also held in an inventory must be
  refused too, but bodies cannot see inventory's section without depending on it; where that check
  lives is part of QB-3.
- `docs/MODULE_SPEC.md` §4.1 gains the `body:` section.

---

# 11. Proposed PR split

PR numbers follow the step files' convention (`step-09` → PR 10, `step-10` → PR 11): this is PR 12.
Each PR has an integration checkpoint that runs the product, not a unit.

| PR | Scope | Integration checkpoint | Adversarial criterion |
| --- | --- | --- | --- |
| **12a** People do not interpenetrate | DEP-13, ARC-39 into `docs/DECISIONS.md`; presence `from` (VERSION 3, digests re-baselined per I-7); `systems/bodies` with `BodyShape`, `PlaceShape`, the `body:` section, genesis facts, and the arrival resolution against people and fixed geometry (character controller only, no dynamics); F-P1 regression test; a test world with walls and a crowd. | Headless: a seeded test world with twelve people in two walled rooms runs 30 days; at every observation instant no two people in one place are closer than 595 mm, and nobody is outside the walls; the run is SIGKILLed mid-way and resumes byte-identical (`ARC-25`); the same run with bodies removed shows overlaps (I-10) and answers nothing new. | Disable the resolution (mutation): the overlap check fails, naming the pair. A save from before is refused by name. |
| **12b** Objects: push, kick, throw; shove | `Lying`, `object-moved`, `person-shoved`; walking pushes objects (stepped only when the stride meets one, `ql`); `kick`, `throw`, `shove` with declared requirements; complete affordances for `kick` and `shove`. | Headless: in the test world the unchanged paced controller kicks and shoves (it attempts complete affordances, `ARC-34`); facts are caused by actions; a person walking into a box moves it; a jammed box stops the person; resume byte-identical. Cost bound of QB-11 measured on a 300-day run. | A thrown object never ends inside a wall or a person; a shoved person never ends inside anything; `TooFarAway` from 801 mm and acceptance at 800 mm. |
| **12c** The town gets bodies | `body:` sections for social-cafe's places (café, street, the shops) matching the 3D slice's layout, its people's default capsules, and a few loose objects; bodies installed in social-cafe and market-town (QB-5); the validator checks of §10.4. | The 300-day seed-7 runs of both worlds hold the activity precondition (every seat moved and talked in every 30-day bucket, `ARC-23`) with walls in place; no overlap at any observation; runs reproduce byte for byte; wall-time increase within QB-11's bound. | A doorway authored inside a wall is refused at load by name. |
| **12d** The 3D client | Jolt selected explicitly; people and objects get colliders at observed positions (shapes from disclosure); reconciliation on difference (150 mm); `kick`, `throw`, `shove` from the camera ray; objects animated along `path`; `server/PROTOCOL.md` §6.2 and `ADOPTION.md` updated. Lands on, or after, the visual track's `vis/3d-godot-2-environment` (QB-13). | The client run for real (`ENGINEERING_RULES.md` §19): a scripted drive walks the player into Alice and stops at her; walks into a box and pushes it; kicks it; is shoved by a test seat; frames captured and inspected, and the server's facts match what is drawn. | No blocking rule in the client: with the client's people colliders removed, the server still stops the player and the client reconciles. |

The 2D client's intents (`kick`, `throw`, `shove` by click) belong to S12, which builds that client; they
need nothing from the server that 12b does not already provide.

---

# 12. Where this step sits

```text
S9   11f, the AC-1 proof — finishing now. S15 starts after 11f merges: the presence change
     re-baselines every world's digest, and must not land inside the AC-1 measurement.
S15  this step: 12a → 12b → 12c, then 12d.
S10  cognition (reduced): independent of S15.
S11  server: nothing new needed; the server carries any action unchanged.
S12  2D client: consumes 12b's actions; no dependency the other way.
S13  CI and container parity: PC-g (another architecture) belongs here if not run in 12a — the
     container is the second architecture `AC-8` cares about.
S14  Demo B: its required "collision / basic navigation" and "interact with at least one
     Item/Object" (ENGINEERING_RULES §10) are delivered in substance by 12c and 12d. S15
     therefore precedes S14.
```

## 12.1 Proposed amendment to `overall.md` (not applied; for the operator)

In §3, after S14:

```markdown
### S15 — Bodies and physical interaction *(inserted 2026-10-07, operator requirement)*

**Design:** [`step-11-bodies.md`](step-11-bodies.md). People never pass through each other or through
walls; walking pushes loose objects aside; people can kick, throw and shove. A `bodies` System Pack on
Rapier (`DEP-13`) resolves every arrival (`ARC-39`); presence's `arrived` gains `from`; the 3D client
collides with people and reconciles on difference (Jolt, `DEP-14`). No kernel or contract change.

- **Depends on:** S9 complete (11f merged). **Feeds:** S14 (`AC-14` collision and object interaction),
  S12 (the same three actions).
- **Acceptance checkpoint:** at every observation instant of a seeded 30-day run, no two bodies in one
  place overlap by more than 5 mm; a person walking into a box moves it; kick, throw and shove are
  resolved server-side and replay byte for byte after SIGKILL; removing `bodies` makes the three actions
  unavailable and changes nothing else.
```

In §4, a row: `Operator requirement 2026-10-07: bodies do not interpenetrate; push, kick, throw | S15`.

In §7, under "Next": `S15 bodies (step-11, DRAFT 2026-10-07): PRs 12a–12d after 11f; awaiting QB-1,
QB-2, QB-3, QB-5, QB-10, QB-12.`

---

# 13. Open questions

Operator-material questions are marked **[OM]**. Each has a recommendation.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QB-1 [OM]** | Accept option C — bodies corrects arrivals after the fact, so a blocked stride leaves `arrived X`, `stride-blocked`, `arrived X′` in the log — over A (movement depends on bodies) or B (kernel geometry seam)? | **C.** It is the only option that makes non-interpenetration a property of the world for every mover, with no kernel change and bodies removable. Record as ARC-39. |
| **QB-2 [OM]** | Change presence's `arrived` to carry `from` (schema 2, presence VERSION 3, old saves refused, every world's digest re-baselined)? | **Yes, in 12a, after 11f merges.** C cannot see a stride without it, and it is generic. |
| **QB-3 [OM]** | What is a loose object? (a) an Item file with a `body:` section, which then names exactly one physical object — amending `ARC-36` ("stacked items only") for such items; (b) a new World Pack content kind `objects/` (a `worldpack` change); (c) pack-local keys that are not entities (against "everything persistent is an Entity"). | **(a)**, with the validator refusing an item that has a body and is also held in an inventory. |
| **QB-4** | `rapier3d` (z up, capsules) or `rapier2d` (circles and boxes on the floor)? | **`rapier3d`.** Objects resting on furniture and a later stairs/ramp need the third axis; the cost is measured (§9.4) and acceptable with `ql`. |
| **QB-5 [OM]** | Which worlds install bodies? | **social-cafe and market-town, in 12c.** The 3D slice walks in social-cafe, and that is where the operator saw the defect. Both worlds' seed-7 runs are re-baselined with the activity precondition checked first. |
| **QB-6** | Should a kicked or thrown object land later (a deferred fact at the end of its flight) rather than at the instant of the kick? | **Not now.** Resolve at the instant and state the path; the client animates it. A deferred landing is a later refinement and needs no contract change. |
| **QB-7** | Should the 3D client predict object pushes locally with a Jolt `RigidBody3D`? | **Not in 12d.** Objects are drawn where the server says; pushes show at the next observation (≤ 100 ms). Revisit if it feels wrong when played. |
| **QB-8** | Should `shove` be refused (`Busy`) while the target is seated or in an activity? | **Not now.** There is no generic "may this person be interrupted" query; adding one is outside this step. |
| **QB-9** | Line of access (talking through a wall) now that a layer owns geometry? | **A later step**, probably the kernel geometry seam (option B), because every pack's `SpatialRequirement` would use it. Not needed for R-1 to R-4. |
| **QB-10 [OM]** | Walking into a person: blocked (stop and slide), or nudged aside ("挤开")? | **Blocked.** People are displaced only by an explicit `shove`. A crowd that parts on its own is a product choice, and easy to add later inside bodies. |
| **QB-11** | The wall-time bound for a 300-day run with bodies installed. | **At most +50 %** over the same world without bodies, measured in 12b and 12c. A bound from the requirement, not from what the code costs. |
| **QB-12 [OM]** | Authorize `rustup target add x86_64-apple-darwin` so PC-g (another architecture under Rosetta) can be run as 12a evidence? | **Yes.** It is the cheapest available evidence for `AC-8` before S13's container. |
| **QB-13** | The 3D slice (`slice_link.gd`) lives on the unmerged visual branch `vis/3d-godot-2-environment`. Does 12d land there, or after it merges? | **After it merges**, so the client work is reviewed on main. 12a–12c do not depend on it. |
| **QB-14** | A speed limit (S6 L-1)? | **Out of scope.** Bodies does not change how often a client may move. |

---

# 14. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-B1** | Rapier breaks its API about monthly (0.33–0.36 in four months). | Pin exactly (DC-1). An upgrade is its own PR: bump bodies' VERSION, re-run the prototype's criteria in the pack's tests, re-baseline. Rapier sits behind one module of the pack. |
| **R-B2** | More Rapier defects like F-P1. | Regression test per defect; the per-request model touches a small, well-trodden part of the API (character controller, insertion, a few steps). |
| **R-B3** | Headless cost grows past QB-11's bound. | The integer pre-check (skip Rapier when no body's box meets the stride's box); per-place worlds are small. Measured in 12b and 12c, never assumed. |
| **R-B4** | Server geometry (`body:` sections) and the client's scene disagree, so players are corrected at invisible walls. | One source: bodies discloses `PlaceShape`, and the 3D client builds its collision from the disclosure, not from its scene. The visual scene stays the client's. |
| **R-B5** | Event volume: a blocked stride records three facts instead of one. | Only blocked strides pay. Measured in 12c's 300-day runs and reported. |
| **R-B6** | A subscriber reacts to the transient `arrived X`. | Audited: only presence subscribes to `arrived` today; employment and group-activity subscribe to `person-entered-place`, which a correction never re-emits (it never changes place). Any new subscriber must be written knowing ARC-39. |
| **R-B7** | Cross-architecture float identity is unproven (PC-g not run). | QB-12 in 12a; S13's container run as the second check. Until then, a save is only claimed resumable on the architecture that wrote it. |
| **R-B8** | Walls make the paced controller's wandering less productive, and activity drops. | `ARC-23` activity precondition before any determinism claim in 12c; a remedy, if needed, is in world data (room layout), not in the controller (`ARC-27`). |

---

# 15. Proposed decision records (not yet in `docs/DECISIONS.md`)

These are drafts for the operator. They are added to `docs/DECISIONS.md` only in PR 12a, after approval,
and their numbers are the next free ones at that time.

## 15.1 DEP-13 — Server physics: Rapier (`rapier3d`, `enhanced-determinism`) inside the `bodies` pack

**Problem.** People must not interpenetrate, and must push, kick and throw objects (operator,
2026-10-07), decided on the server and replayable byte for byte (`ARC-25`), on more than one machine
(`AC-8`).

**Options.** Rapier; `parry` alone; our own circle-and-box code; Jolt through Rust bindings; Avian.

**Choice: `rapier3d =0.36.0`** with `enhanced-determinism`, without `simd8` or `parallel`, used only
inside `systems/bodies`.

**Why not ourselves.** Kick and throw need dynamics: integration, contacts, friction, rest. That is a
physics engine, commodity infrastructure (`REUSE_POLICY.md` §4), and ours would have to earn the
cross-platform determinism Rapier documents. **Why not the others.** `parry` alone lacks dynamics; the
Jolt bindings are unmaintained since May 2024 and cannot enable Jolt's cross-platform determinism; Avian
requires Bevy (§3).

**Isolating interface.** The pack's resolution module: integer state in, integer state and facts out. No
Rapier type appears in a component, a fact, a contract or another crate. Nothing of Rapier survives a
resolution.

**Accepted limitations.** Monthly breaking releases (pin; an upgrade bumps the pack's version). A defect
found in 0.36.0 (F-P1) and worked around. Cross-architecture identity not yet evidenced. No momentum
between resolutions. Apache-2.0, compatible with MIT.

## 15.2 DEP-14 — Client collision: Godot's built-in Jolt, never authoritative

**Problem.** The 3D client must not let the player walk through people and must predict collisions
locally (`NETWORKING.md` §4) while the server decides.

**Choice: Jolt Physics, built into Godot 4.7.2**, selected explicitly in `project.godot`, with
`CharacterBody3D` as today. MIT. Godot states its physics is not deterministic; that is acceptable because
nothing the client simulates is authoritative.

**Not chosen.** Godot Physics (works, but Jolt is the engine's default since 4.6 and its better solver);
the Rapier Godot addon (symmetry with the server buys nothing when the server always wins; its 3D part is
incomplete).

**Isolating interface.** The wire protocol. Colliders are built from disclosed shapes; the client's rule is
only "reconcile on a difference above 150 mm".

## 15.3 ARC-39 — Bodies resolve arrivals: movement decides whether, bodies decide what happens

**Problem.** Non-interpenetration must hold for every way a person can come to be somewhere, without a
kernel change and without making the deciding packs depend on physics.

**Choice.** A `bodies` System Pack depends on presence and subscribes to presence's `arrived`. For every
arrival with a local position it resolves the stride, from the arrival's new `from`, against the place's
fixed geometry, other people and loose objects. If the body stopped short it states `stride-blocked` and
presence's `arrived` at the reached position (`ARC-26`); loose objects pushed aside are its own
`object-moved`. Movement still alone decides whether a move is allowed. Presence's `arrived` gains `from`,
filled by presence's constructor.

**Consequence stated, not hidden.** A blocked stride leaves two `arrived` facts in one instant, the second
caused by the first. No observation shows the first. The cascade ends at depth 3, because the corrected
position resolves to itself.

**Rejected.** Movement depending on bodies (bodies no longer removable; every future mover must
remember); a kernel geometry seam (a kernel change, unnecessary for this requirement, the likely route for
line of access later); a persistent stepped world (no idle skipping: about 82 hours per café per 300
days, §9.4).
