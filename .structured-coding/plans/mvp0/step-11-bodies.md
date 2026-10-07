# Step 11 — Bodies and physical interaction (S15)

**Role:** step document for a new step, proposed as **S15 — Bodies and physical interaction**. It
records the operator's requirement, the audit, the answers to the planning questions, the Rapier
determinism prototype, the proposed ownership, facts and contracts, a PR split, risks and open
questions. It holds no frozen PR design yet: the first PR is detailed only after the operator decides
the material questions in §13 (`CLAUDE.md` §3, "detail one step ahead").
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S11, S12, S14), §7.
**Lifecycle:** step design `DESIGN FROZEN (2026-10-07)`, revision 1.

- The architecture, the ownership and the five-PR split are frozen.
- No PR design in this document is frozen yet. Each of 12a–12e is detailed to the commit and frozen
  in turn, and implementation starts only after 11f merges.
- Nothing here yet authorizes implementation.

**Freeze record (primary session, 2026-10-07).** The operator decided:

- **QB-1:** resolve before recording.
- **QB-10:** walking nudges people aside.
- **QB-2:** withdrawn under QB-1.
- **QB-3, QB-5, QB-12:** as recommended.
- **QB-15: F1, registration at start-up** (decided 2026-10-07, after revision 1).

Bounds on QB-15, binding:

- The catalog is written once, from the build's installed set, and is immutable afterwards.
- A host that never registers it must fail loudly, never run silently without resolvers. 12a
  proves this with a test.
- Runtime `World::disable` of a resolver pack is not honoured until QB-17 is taken up. ARC-39
  records that limitation in its own words.

**QB-16 (head-on bias)** is decided and tested in 12b. **QB-17** is deferred, as recommended.

Remaining QBs are accepted as recommended. The proposed `overall.md` amendment (§12.1) is applied by
the primary session in the S9 closeout PR.
**Revision history:** revision 0 (commits `f8da73a`…`bb0f3b4`) recommended correcting arrivals after
they were recorded and blocking walkers at people; both were decided against (§1.4). Sections superseded
by revision 1 say so where they stand, and the evidence that led to them is kept.
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

## 1.3 The answer in brief (revision 1)

```text
Seam     presence resolves an arrival BEFORE it is recorded. Its constructor `arrival()` asks every
         installed ArrivalResolver (a new presence-owned trait, carried by `installed!` the way
         PerceptionProvider is) what the stride actually achieves, and records only the true
         result: the walker's arrived, an arrived for each person nudged aside, and presence's own
         stride-blocked when the walker stopped short. With no resolver installed, every world
         records exactly what it records today, byte for byte. A framework precursor (PR 12a).
Server   a new System Pack, `bodies`, is the first resolver. It rebuilds a Rapier world for the
         place from integer state, sweeps the walker with the character controller, nudges people
         in the way (at most 300 mm each, at most 2 generations and 4 people per stride, never
         through a wall), verifies the result and degrades to blocked, halved or stay-put if any
         pair would overlap, and quantizes to whole millimetres. It never decides whether a move
         is allowed: `movement` still does, and still does not know `bodies` exists. bodies
         pushes loose objects aside in its reactions to the recorded arrivals, and provides
         `kick`, `throw` and `shove`, each with a declared SpatialRequirement.
Kernel   unchanged. Contracts unchanged. `arrived` unchanged (no `from`).
Clients  Godot keeps its local CharacterBody3D, now with Jolt selected explicitly, and gives every
         other person and every loose object a collider at its observed position. It reconciles
         to the authoritative position whenever the two differ by more than a stated tolerance —
         which now also happens when somebody else nudges you. Kick, throw and shove are intents
         with a target.
```

## 1.4 Operator decisions on revision 0 (2026-10-07, relayed by the coordinator)

```text
QB-1   Rejected: correct-after-the-fact. The log contains only true arrivals — no transient
       `arrived X` followed by a correction. Resolve before recording, through a provider-trait
       seam like PerceptionProvider, wired by `installed!`; movement still unaware of bodies;
       byte-identical with no resolver installed. A framework precursor PR (sdk, installed,
       presence) proven with a synthetic resolver, like 11c's chimes. Not a kernel change; if it
       needs kernel support, that is operator-material. Re-check whether `from` is still needed;
       prefer not changing presence's event version.
QB-10  Walking into a person nudges them aside, as with a box. Presence alone writes positions;
       the nudged person's new position is a true fact caused by the walker's request (AC-9);
       the nudge is small and bounded per stride; chains are bounded (or the stride is blocked);
       nobody is pushed through a wall or out of the place; deterministic under the per-request
       rebuild and i32 mm quantization. `shove` remains the deliberate, larger displacement.
       Measure head-on, one stationary person, and a crowd of five, criteria first.
Accepted as recommended: QB-2 (re-checked under QB-1: §4.4.7, withdrawn), QB-3, QB-4 (rapier3d), QB-5
       (social-cafe and market-town), QB-12 (cross-architecture run; rustup target permitted).
       The other questions stay as recommended.
PR split: the precursor seam PR first; the step still starts after 11f merges.
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

## 4.2 Options considered (revision 0; kept as evidence)

Revision 0 compared five options and recommended C. The operator rejected C on 2026-10-07 (§1.4), and
revision 1 adopts a sixth, F, in §4.3. The comparison is kept because it records why A, B, D and E were
set aside, and those reasons still hold.

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

## 4.3 Revision 1: (F) resolve before recording, through a presence-owned resolver seam

```text
(F) presence asks, then records    presence's arrival constructor asks every installed
                                   ArrivalResolver what the stride achieves, and records only
                                   the true result. bodies is a resolver. movement is unaware.
```

| | A | B | C (rejected) | F (adopted) |
| --- | --- | --- | --- | --- |
| Kernel or contract change | no | **yes** | no | no (§4.4.4 names the one place it could become one) |
| Edit to an existing pack | movement (dependency, calls) | movement (calls) | presence (`from`) | presence (the seam); movement: two mechanical lines, still unaware of bodies (§4.4.2) |
| `bodies` removable (`AC-2`) | no | yes | yes | yes |
| Uniform for every mover (R-2) | no | only callers | yes | **yes**: every arrival goes through presence's constructors, which consult the resolvers or refuse (§4.4.2) |
| Log holds only true arrivals | yes | yes | **no** | **yes** |
| `arrived` schema change | no | no | yes | **no** (§4.4.7) |

F keeps what made C attractive — non-interpenetration as a property of every arrival, whoever states it —
and removes what the operator rejected: nothing untrue is ever recorded. It is the `PerceptionProvider`
pattern applied to the one question perception does not ask: not "what may this person attempt" but "what
does this stride actually achieve".

### Revision 0's recommendation, superseded

The text below is revision 0's case for C. It is superseded by §4.3 above and §4.4, and kept so the
decision trail is readable.

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

## 4.4 The seam, concretely (revision 1)

### 4.4.1 The trait, owned by presence

In `systems/presence/src/resolve.rs`, next to `PerceptionProvider` in `interaction.rs`:

```rust
/// One stride a person is about to take, as presence knows it before recording anything.
pub struct Stride {
    person: PersonId,
    from: Option<Location>,   // the person's current Presence, read by presence; None at a first
                              // placement
    to: Location,             // where the stating system decided the person goes
}

/// What a stride achieves: where the person ends, who else moved, and what cut it short.
pub struct Resolution {
    reached: Location,                       // == to when nothing intervened
    displaced: Vec<(PersonId, Location)>,    // other people moved by this stride, in the order
                                             // they are to be recorded
    stopped_by: Option<EntityId>,            // what the person stopped at, if `reached != to`
}

impl Resolution {
    /// The stride exactly as decided: what presence records when no resolver is installed.
    pub fn unchanged(stride: &Stride) -> Self;
}

/// A pack's answer to "what does this stride actually achieve".
///
/// Asked by presence while an arrival is being built, before anything is recorded. Handed a
/// `WorldRead` and nothing else, as `PerceptionProvider` and `System::validate` are (BD-6): it can
/// consult any state and write none. A pure function of its arguments: it keeps no state, no cache,
/// no clock. It must return `so_far` unchanged when the world holds none of its own state about the
/// people involved — which is what keeps a world without its pack byte-identical.
pub trait ArrivalResolver: Send + Sync {
    /// The system this resolver belongs to. Resolvers are asked in this id's order.
    fn resolver_of(&self) -> SystemId;

    /// What the stride achieves, given what earlier resolvers decided.
    fn resolve(&self, world: &WorldRead<'_>, stride: &Stride, so_far: Resolution) -> Resolution;
}
```

A resolver cannot emit a fact. Its pack's own consequences of a stride — bodies pushing a loose object —
happen in that pack's reactions to the facts presence records (§4.5.4).

### 4.4.2 Where it is asked: presence's constructors

Presence already has the only public way to build an `arrived`: `arrival(world, person, location)`
(`systems/presence/src/event.rs`), which every stating system calls under `ARC-26`. Revision 1 makes the
constructors the seam:

```text
arrivals(world, person, to) -> Result<Vec<Emission>, Rejection>          NEW; for movers
    1  admit(person, to)                                       as today
    2  stride = { person, from: current Presence, to }
    3  resolution = fold over the resolvers in SystemId order, starting from unchanged(stride)
    4  presence's own checks of the resolution (§4.4.6); a violation → Rejection, which the
       stating system turns into KernelError::FactRefusedByOwner, as it does today
    5  emissions, in this order:
         arrived { person, reached }
         arrived { other, location }            for each displaced, in the resolution's order
         stride-blocked { person, wanted: to,   when reached != to
                          reached, by: stopped_by }

arrival(world, person, to) -> Result<Emission, Rejection>                 KEPT; for placement
    steps 1–4 as above; if the resolution is not `unchanged` (the stride was cut short or moved
    anybody else), refuse with PreconditionFailed. Otherwise the one arrived, exactly as today.
```

Why two constructors, and why `arrival()` refuses rather than resolves. Sixteen files call `arrival()`
today: thirteen test files that place people (`systems/*/tests/support`, `presence`, `naming`,
`conversation`, `tests/acceptance`), the World Pack loader's genesis placement (`worldpack/src/load.rs`,
`catalog.rs`), and movement. Changing its return type would edit all of them.
Keeping it, and making it refuse whatever it cannot record truthfully, gives the same uniformity with no
cascade: **a stride is recorded as resolved, or it is refused** — never recorded unresolved. A genesis
placement that overlaps another body is therefore refused at load, loudly, by the owner.

Movement's edit is two mechanical lines, and it still names nothing of bodies:

```text
systems/movement/src/system.rs   resolve: `arrival(..)` → `arrivals(..)`, returning the Vec
                                 declaration: `.emitting::<StrideBlocked>()`  (ARC-26: a stating
                                 system declares what it may state; the kernel checks it)
```

### 4.4.3 What presence records, and who owns it

| Fact | Owner (vocabulary, reducer) | Stated by | Caused by |
| --- | --- | --- | --- |
| `arrived { person, location }` for the walker, at `reached` | presence (unchanged schema) | the stating system, e.g. movement | the walker's `ActionIntent` |
| `arrived { other, location }` for each nudged person | presence | the same stating system, in the same emission list | the walker's `ActionIntent` (`AC-9`: a true fact about the nudged person, traceable to the request that nudged them) |
| `stride-blocked { person, wanted, reached, by }` | **presence** — new event type, schema 1. A stride ending short of where it was asked to go is a fact about where a person is, which is presence's domain; it names no physics | the same stating system | the walker's `ActionIntent` |

Presence reduces each `arrived` exactly as today; nudges never change place, so they never emit
`person-entered-place`. `stride-blocked` is reduced into nothing (presence keeps no state for it); it
exists for biographies, controllers and clients.

### 4.4.4 How the resolvers reach presence

This is the one part of the seam that cannot copy `PerceptionProvider` exactly, and the reason is in
source. Perception is asked **outside** the world: the host calls `observe(world, providers, …)` with the
provider list `worldpack::compose` built (`ComposedWorld::providers`). An arrival is built **inside**
`World::dispatch`, where the only things in reach are the stating system's `&self` and a `WorldRead`
(`kernel/src/view.rs`: entities, components, relations, processes — no composition, no extension). So
presence's constructor cannot be handed the list per call. Two ways to reach it:

```text
F1  a resolver catalog in presence, registered once per process      no kernel change
      installed!      gains a line:   resolution: mineworld_presence::ArrivalResolver => [Bodies];
                      and expands    Capability::resolvers() -> Vec<Box<dyn ArrivalResolver>>
                                     for the listed variants only (each must implement the trait;
                                     the compiler checks it at the list). Today the list is [].
      worldpack       compose() calls mineworld_presence::register_resolvers(Capability::resolvers())
                      beside building `providers` — the one assembly path every host uses
                      (server, run, replay, inspect, biography)
      presence        a write-once catalog (OnceLock). Registering the same list again (same
                      SystemIds in the same order) is a no-op; a different list panics, naming
                      both — one build has one catalog
F2  a generic extension slot in the kernel                           kernel change (operator-material)
      World::provide(Box<dyn Any + Send + Sync>) at assembly; WorldRead::provided::<T>()
      The kernel stores values it does not interpret (INV-12 holds), and resolvers become
      per-world rather than per-process.
```

**Recommendation: F1 for MVP-0, with F2 named as the principled alternative (QB-15, operator-material).**
F1's catalog is not world state: it is the build's compiled-in list of resolver code, identical for every
world in the process, set before any world runs and never changed — the same kind of thing as the
`installed!` list itself. Per-world applicability needs no filter, because a resolver is required to be
inert where its own state is absent (§4.4.1): a world that does not install bodies has no `BodyShape`
table, and a lookup in a missing table answers `None` rather than failing (`kernel/src/components.rs`
`ComponentStore::get`: `self.rows::<C>()?.get(&entity)`), so bodies' resolver returns `so_far`. Its one weakness is a host that assembles a world without
`worldpack::compose` and forgets to register: such a host would record unresolved strides. Two guards:
every host already composes through `worldpack`; and bodies, reacting to every `arrived` (§4.5.4), checks
the invariant it exists to keep and fails the dispatch, naming the pair, if a recorded arrival overlaps —
so the omission is loud at the first contact, and a replay by such a host diverges and is refused
(`ARC-25`).

Runtime `World::disable` of a resolver pack is **not** honoured by F1: `WorldRead` cannot see whether a
system is enabled. Disabling exists for `AC-2` tests; `AC-2` for bodies is shown at world level (a world
that does not list bodies), which F1 handles. Honouring runtime disable needs an additive kernel read,
`WorldRead::is_enabled(SystemId)`: operator-material, not proposed now (QB-17).

### 4.4.5 Several resolvers: order and composition

Resolvers are asked in **ascending `SystemId` order**, each receiving the previous one's `Resolution` as
`so_far`. The order is a fixed function of the resolvers' names, so it does not depend on the order of the
`installed!` list or of a world's `systems:` list, and a save resumed by another build of the same
packs resolves identically. MVP-0 has one resolver; the order is stated so that the second one does not
have to invent it. A resolver that disagrees with an earlier one may only narrow its result further,
because presence checks the final result (§4.4.6).

### 4.4.6 What a resolver may read, and what presence still decides

May read: anything a `WorldRead` exposes. May not: write, emit, keep state, read a clock or a random
source. Presence checks the final resolution before building any fact — the owner still decides
(`ARC-26`):

```text
reached.place == to.place                         a resolver never moves anybody to another place
|reached − from| ≤ |to − from|  (same place)      it may shorten or bend a stride, never lengthen it,
                                                  so movement's MAX_STRIDE decision still bounds it
every displaced person: a living person, not the walker, listed once, currently in the walker's
                        place, displaced within that place
admit() on every location                         as today
```

A violation is a defect in the resolver and is refused, never recorded.

### 4.4.7 QB-2 re-checked: `from` is not needed

Revision 0 needed `from` on `arrived` because bodies resolved *after* recording and could not see where a
stride began. Under F, presence builds the `Stride` itself, from its own `Presence`, before recording; the
resolver receives `from` directly. Nothing downstream needs it either: bodies' object pushes are resolved
from each recorded arrival's end position (§4.5.4). **`arrived` keeps schema 1; presence's event version
does not change.** QB-2 is withdrawn.

What does change in presence is its declaration (it emits `stride-blocked`) and its version (2 → 3,
because `arrivals()` and the resolver catalog change what it answers). Saves written before 12a are
refused by name, as `ARC-25` requires of any composition change.

### 4.4.8 Byte-identity without resolvers

With the catalog empty — the state of every build until bodies is listed — `arrivals()` returns exactly
the one `arrived` that `arrival()` returns today, and `arrival()` returns what it returns today. So every
fact and every journal input of social-cafe and market-town is byte-identical. What differs is the
composition record of a save (presence 3, the new `stride-blocked` declarations), which `ARC-25` puts in
the save and refuses on mismatch: old saves are refused by name, new runs reproduce the old facts. PR 12a
proves this on the 300-day seed-7 runs of both worlds before and after (I-7).

### 4.4.9 The synthetic resolver that proves the seam (PR 12a, like 11c's `chimes`)

A test-only pack, `fences`, never in `systems/installed`, with one resolver: a stride that would cross the
line `x = 5 000 mm` stops 10 mm short of it, and a person standing within 300 mm beyond the line of the
stopping point is "displaced" 100 mm along +x. It knows nothing of bodies, physics or Rapier. With it the
precursor shows, through the real `World::dispatch` and the real persistence:

```text
SC-1  without fences installed: social-cafe's facts byte-identical to main (§4.4.8)
SC-2  with fences: the recorded arrived is the stopped one; stride-blocked names wanted, reached and
      the fence; the displaced person's arrived is recorded, caused by the walker's ActionId (AC-9)
SC-3  presence refuses a resolver that lengthens a stride, changes place, or displaces the walker
SC-4  two synthetic resolvers compose in SystemId order, whatever the installed order
SC-5  arrival() refuses a placement the resolver would change; arrivals() records it
SC-6  SIGKILL mid-run, resume byte-identical (ARC-25); replay by a second process identical
SC-7  movement's source names no resolver pack (a structural test, like presence's)
```

## 4.5 Nudging people aside (QB-10)

### 4.5.1 The rule

Bodies' resolver, per stride, on a Rapier world rebuilt for the place from integer state (§6.1). The
algorithm is the one the prototype measured as R′ (§9.6, §9.7):

```text
1  walls-only reach W     the character controller sweeps the walker from `from` toward `to`
                          against fixed geometry only
2  contact reach B        the same sweep with people solid
3  candidate              W if |W − from| ≤ |B − from| + NUDGE_MAX, else the point on the
                          walls-only path NUDGE_MAX beyond contact
4  nudge pass             generations 1..CHAIN_MAX: every person not yet moved who overlaps a
                          pusher of the previous generation, in EntityId order, is moved directly
                          away from that pusher's centre by the overlap + GAP, by the character
                          controller against fixed geometry. Fails if a nudge needs more than
                          NUDGE_MAX + GAP, if the walls cut it short, if overlap remains after
                          CHAIN_MAX generations, or if more than NUDGED_MAX people would move
5  on failure             blocked: the walker ends at B, nobody else moves
6  objects                bodies' reactions will push loose objects out of each recorded body
                          (§4.5.4); the resolver runs that same reaction sequence on the
                          resolved state and, if an object could not be placed, re-resolves with
                          objects solid
7  quantize               whole millimetres
8  verify, then degrade   on the quantized result, no pair closer than 600 − 5 mm that was not
                          already that close; otherwise blocked; otherwise the advance along the
                          blocked path halved up to 8 times; otherwise the walker stays
```

```text
NUDGE_MAX   300 mm      the most one stride moves anybody else; `shove` moves 500 mm, deliberately
GAP          10 mm      the character controller's own offset
CHAIN_MAX     2         a nudged person may nudge one further generation, no more
NUDGED_MAX    4         people moved by one stride, at most
```

These are bodies' policy constants, published like `MAX_STRIDE`, until a pack needs them configurable.

### 4.5.2 What holds, and why

- **Presence alone writes positions.** The resolver returns the nudged people's new locations; presence
  records them as `arrived` facts caused by the walker's request.
- **Bounded per stride.** No nudge exceeds `NUDGE_MAX + GAP` (310 mm); the prototype's largest was
  309–310 mm over 3 000 requests (§9.8).
- **Bounded chains.** At most two generations and four people; a stride needing more is blocked.
- **Nobody through a wall, nobody out of the place.** A nudge is a character-controller sweep against
  the place's fixed geometry, and presence refuses a displacement into another place.
- **Never an overlap, even when the character controller errs.** Step 8 exists because the prototype
  caught the controller letting a walker slide onto a person standing against a wall (§9.8, R N-1 FAIL at
  request 551). The check is on integers, after quantization, and the last fallback — stay where you were
  — is valid by induction: the state before the request passed the same check.
- **Deterministic.** The same per-request rebuild, canonical insertion order and quantization as
  everything else (§6.3); the prototype's scenarios gave the same digests in two processes and on two
  architectures (§9.8).

### 4.5.3 What it looks like (prototype, §9.8)

```text
stationary person (n2)   walked into at 0.5 m strides: nudged ≤ 289 mm per stride, 495 mm in
                         all, the walker passes; with nudging forced off, 0 mm and the walker
                         stops at contact
crowd of five (n3)       2 generations, up to 2 people per stride, nudges ≤ 301 mm, 3 of 12
                         strides blocked, closest pair 600 mm, nobody outside the room
head-on (n1)             exactly collinear walkers push each other back 300 mm in turn and never
                         pass: a nudge straight back has no sideways part. QB-16: give a nearly
                         collinear nudge a fixed sideways bias, decided in 12b
```

### 4.5.4 Loose objects, under revision 1

Objects are bodies' state, so presence cannot record their moves and the resolver cannot emit them.
Bodies subscribes to presence's `arrived` and, for each recorded arrival, pushes any loose object the
person's capsule now overlaps out of it — away from the person's centre, by the character controller
against fixed geometry and people as they now stand — recording `object-moved { how: pushed }`, caused by
that arrival and so by the request. Because the resolver runs exactly this reaction sequence on the
resolved state before answering (step 6), the reactions always find room: the walker never ends inside a
jammed box. The same reaction checks the non-overlap invariant and fails the dispatch if it is broken
(§4.4.4's guard).

The prototype measured a simpler object path (the walker carried kinematically over its stride with
objects dynamic, then a re-resolve with objects solid when that left an overlap — 28 times in 3 000
requests). The per-arrival form above is the design; PR 12c measures it against the same criteria.

## 4.6 Ownership

| State | Owner | Written only while | Notes |
| --- | --- | --- | --- |
| Where a person is (`Presence`, `present-in`) | **presence** | reducing `arrived` | Unchanged. Every `arrived` — the walker's, a nudged person's, a shoved person's — is built by presence's constructors after the resolvers have answered (§4.4); presence reduces it and may refuse it. |
| What a stride achieved when cut short (`stride-blocked`) | **presence** | — (a fact, no state) | Revision 1: presence's vocabulary, stated by the stating system (§4.4.3). |
| A person's body: shape (capsule radius and height) | **bodies** | reducing `body-formed` (genesis) | Every person in a world with bodies has one: an authored `body:` section or the default human capsule, r 300 mm, height 1 720 mm (the 3D client's own, `player.gd`). |
| A place's fixed geometry: walls, counters, tables as boxes in the place's frame | **bodies** | reducing `place-shaped` (genesis) | Authored as the place's `body:` section (`ARC-31`). A doorway is a gap. Static for MVP; doors that open are a later pack. |
| A loose object's body and where it lies (place and local position) | **bodies** | reducing `object-placed` (genesis) and `object-moved` | See QB-3: an Item file is a kind (`ARC-36`); a loose object is one Item entity whose `body:` section places exactly one physical instance. |
| Velocity, contacts, sleep state | **nobody** | — | Never persisted. Everything is at rest between resolutions (§6.1). |
| Passages | movement | unchanged | bodies does not read them; a doorway must simply be a gap in the place's geometry, which the world validator checks (§10.4). |

**Single ownership holds.** No component has two writers. bodies never writes `Presence`; it answers
presence's question, and for `shove` it states facts presence builds and reduces. Movement writes
nothing new.

## 4.7 Facts (revision 1)

```text
presence's vocabulary (owner and only reducer: presence)
  arrived          unchanged schema. Now also the nudged people's, in the same emission list
                   as the walker's
  stride-blocked   NEW. { person, wanted: Location, reached: Location, by: Option<EntityId> }
                   by = what the walker stopped at; None for fixed geometry

bodies' own vocabulary (owner and only reducer: bodies)
  body-formed      genesis     a person or an object has this shape
  place-shaped     genesis     a place has this fixed geometry
  object-placed    genesis     a loose object lies here
  object-moved     reaction    a loose object moved      { object, from, to, how, by: PersonId,
                   or action                               path: Vec<LocalPosition> }
                               how ∈ { pushed, kicked, thrown }
  person-shoved    action      { by, person, from, to }

stated by bodies in presence's vocabulary (ARC-26; bodies depends on presence)
  arrived, stride-blocked      for `shove`: the shoved person's new position, built by
                               presence's arrivals() — so a shove into a crowd nudges too, under
                               the same bounds
```

Revision 0 had `stride-blocked` in bodies' vocabulary, stated as a correction; it is now presence's,
stated before recording.

`path` is a short list of quantized keyframes (at most one per 0.1 s of simulated motion, at most 40)
for a client to animate an object's flight. It is presentation data stated by the owner because only the
owner knows it; the client may ignore it and draw the end position.

Bodies subscribes to: `arrived` (presence's: object pushes and the invariant guard, §4.5.4) and its own
genesis facts.

---

# 5. Question 3 — Semantic space and continuous space

- **A Place is semantic; bodies are continuous inside one.** A resolution is always **per place**: the
  Rapier world built for it contains the place's fixed geometry, every person whose `Presence` is in that
  place with a local position, and every loose object lying there. Nothing from another place is in it.
  This follows `CORE_CONCEPTS.md` §6.2: two positions are comparable only within one place's frame.
- **A world without positions loses nothing.** An arrival with `local: None` is not resolved (there is no
  body to place), so a semantic or 2D-without-positions world behaves exactly as now. This is the same
  degeneracy `SpatialRequirement::evaluate` already has.
- **Passages.** A stride that changes place starts in one frame and ends in another. The resolver does
  not sweep across frames: when `from` is in another place, it treats the stride as a placement at `to` —
  the same nudge rule with the walker's capsule at the far doorway point as the pusher, and, if the
  doorway is crowded beyond the nudge bounds, the stride is blocked (the person stays on their side of the
  door) rather than placed inside anybody. The doorway on the far side must be clear in the place's
  authored geometry; the world validator checks that every passage's `there` point is free for the
  default human capsule (§10.4). Nudges never change anybody's place (§4.4.6).
- **Travel between towns** stays a Process that ends in `arrived` (`ARC-26`). Its arrival goes through the
  same constructors and is resolved like any other. Nothing here prevents it.
- **Height.** z is the floor for people (positions keep `z = 0`; the capsule is held at floor height);
  objects may come to rest above the floor (on a counter), so an object's `z` is meaningful. The 3D client
  already never reports height (`REPORT_HEIGHT = false`), and a jump stays rendered movement.

---

# 6. Question 4 — Determinism and persistence

## 6.1 Recommended model: integer state, resolution rebuilt each time (mode R′; revision 0: Q / `ql`)

Revision 1 keeps this model unchanged and moves where it runs: the people part of a resolution runs
inside bodies' `ArrivalResolver` (before recording), the object part inside bodies' reactions to the
recorded arrivals (§4.5.4). Both rebuild from integers and quantize; neither keeps anything.

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
DC-8  (revision 1) Verify, then degrade: the non-overlap invariant is checked on the quantized
      integers after every resolution, with blocked → halved → stay as fallbacks (§4.5.1 step 8).
      The character controller alone does not guarantee it: §9.8 caught it letting a walker onto a
      person against a wall, and the revision-0 `ql` design left two people 235 mm apart.
DC-9  (revision 1) Cross-architecture identity is evidenced (PC-g, §9.8): arm64 and x86_64 under
      Rosetta give identical digests and identical snapshot bytes, and an arm64 snapshot resumes on
      x86_64 byte for byte. Rosetta translates x86_64 instructions on the same machine; a native
      x86_64 host (S13's container on such a host) remains the stronger check.
```

## 6.4 Headless cost

Movement volume, measured on main: a 30-day social-cafe run accepted 16 595 moves (`overall.md` §7,
10a), so about **166 000 arrivals per 300 days**. Every one would pass through bodies.

```text
mode Q    160.9 µs per request (§9.4)  →  ≈ 26.7 s per 300-day run
mode ql    58.6 µs per request (§9.4)  →  ≈  9.7 s per 300-day run
mode R′    61.0 µs per request (§9.8)  →  ≈ 10.1 s per 300-day run     revision 1, with nudging
                                          and verify-then-degrade
```

Against current 300-day runs of 15.2 s (market-town, 11d) to 33.7 s (11e with `--save`), `ql` and R′
add roughly 30–65 %. Under Rosetta (x86_64 translated) R′ measured 104.7 µs, which says nothing about a
native x86_64 host and is recorded only so nobody reads it as one. Two further reductions are designed in, not yet measured: skipping Rapier altogether
when no other body's box meets the stride's box (an integer test), which is most strides in an open
street; and building the place's fixed geometry once per resolution from a pre-sorted list. The bound
to hold in PR 12c's and 12d's acceptance is stated there (§11.1, QB-11), not derived from what the code happens to cost.

Only a world that installs bodies pays. With no resolver registered, `arrivals()` costs one empty fold
over the catalog. `social-cafe` and `market-town` install bodies only in PR 12d (§11); until then their
facts and run times are unchanged (§4.4.8).

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
   position. That covers a stride stopped short by the server, being nudged aside by somebody walking
   into you (revision 1), a shove, and a refusal, with one rule.
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
| People never interpenetrate | (revision 1) Every arrival resolved before it is recorded (§4.4): the walker slides along walls; verify-then-degrade guarantees the invariant on integers. | none (part of every arrival, not an action) | — |
| A person nudges another aside by walking | (revision 1, QB-10) §4.5: at most 300 mm per person per stride, two generations, four people; otherwise blocked. Recorded as presence's `arrived` for each nudged person, caused by the walker's request. | — | — |
| A person pushes an object aside by walking | (revision 1) bodies' reaction to each recorded arrival pushes overlapping loose objects out, against walls and people as they now stand; the resolver has already checked there is room (§4.5.4). | — | — |
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
worlds/<a world with bodies>    body: sections on places and objects; see §11.1, PR 12d
clients/ (3D, and the 2D when it exists)   colliders for people, reconciliation, intents
docs                            ARC-39, DEP-13, DEP-14, MODULE_SPEC §4.1 (the body: section),
                                server/PROTOCOL.md §6.2 (reconciliation), ADOPTION.md
```

What it edits that already exists, and only this (revision 1; all in PR 12a, the precursor):

```text
systems/presence     the ArrivalResolver trait, the resolver catalog, arrivals(), arrival() refusing
                     what it cannot record truthfully, stride-blocked; VERSION 3 (§4.4)
sdk/rust             installed!: the `resolution: <trait> => [<variants>]` line and
                     Capability::resolvers() (§4.4.4). sdk still names no pack.
systems/installed    `resolution: mineworld_presence::ArrivalResolver => [];` (bodies joins in 12b)
worldpack            compose() registers the catalog beside building providers (one call)
systems/movement     two mechanical lines: `arrivals()` in resolve, `.emitting::<StrideBlocked>()`
                     in the declaration. It names no resolver and no other pack (SC-7)
```

Revision 0's `from` on `arrived` is withdrawn (§4.4.7).

What it must **not** edit: `kernel/`, `contracts/`, `persistence/`, `server/src` (the server carries any
action unchanged; only the document `server/PROTOCOL.md` changes), any System Pack other than presence and
movement as above, `cognition/` (the paced controller attempts complete affordances it was never compiled
against, `ARC-34`). If any of these turns out to need an edit — in particular if F1 cannot work and the
kernel extension slot F2 is needed — the work stops and the question returns to the operator.

**One kernel-adjacent fact to verify in PR 12a, not assumed:** that a system may subscribe to a fact type
it also states (bodies subscribes to `arrived` and, for `shove`, states `arrived`). `ARC-26`'s install
check requires only a declared emission and a dependency on the owner. If the kernel refuses this
combination, that is a kernel change and goes to the operator.

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
| PC-g | **NOT RUN** in revision 0 | Needs `rustup target add x86_64-apple-darwin`, outside revision 0's permitted commands. QB-12. **Run in revision 1 after the operator permitted it: PASS (§9.8).** |

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

## 9.6 Revision 1 (operator decisions 2026-10-07): criteria for mode R, stated before any run

The operator rejected correcting arrivals after they are recorded (QB-1) and chose nudging over blocking
when a walker meets a person (QB-10). The prototype gains a mode **R**, the shape of the revised design
(§4.6): each move is **resolved before anything is recorded**, and the recorded state is the resolved
state. Its algorithm, fixed here before the code is written:

```text
per move request, on a world rebuilt from integer state in canonical order:
1  walls-only reach W: the character controller sweeps the walker from `from` toward `to`
   against fixed geometry only
2  contact reach B: the same sweep with people solid (objects never block at this stage)
3  candidate: W if |W − from| ≤ |B − from| + NUDGE_MAX, else the point on the walls-only path
   NUDGE_MAX beyond contact               NUDGE_MAX = 300 mm
4  nudge pass, generations g = 1 .. CHAIN_MAX (CHAIN_MAX = 2), at most NUDGED_MAX = 4 people:
   every person not yet moved who overlaps a pusher of the previous generation (in EntityId
   order) is moved directly away from that pusher's centre by exactly the overlap + 10 mm, by the
   character controller against fixed geometry. A required nudge above NUDGE_MAX + 10 mm, a nudge
   the walls cut short by more than 1 mm, an overlap left after CHAIN_MAX generations, or more
   than NUDGED_MAX people → the whole nudge fails
5  on failure: the walker ends at B, nobody else moves (blocked)
6  objects: as in `ql` — stepped only if the walker's path box meets an object, with the nudged
   people fixed at their new positions; if the walker then overlaps an object by more than 5 mm,
   the request is re-resolved with objects solid and no stepping
7  quantize everything to whole millimetres
```

| ID | Claim | Measurement | PASS iff |
| --- | --- | --- | --- |
| **PC-a/R**, **PC-b/R** | As PC-a and PC-b, for mode R. | R = 3 000 requests from the same request sequence as Q; snapshot at 1 300. Two processes; a third resumes. | Equal digests. |
| **PC-c/R** | As PC-c″ for mode R. | Traces of base, c2 and c4. | c2 and c4 each differ from base at the request where they take effect. |
| **PC-d/R** | Cost. | Mean µs per request, release. | At most **100 µs**. |
| **N-1** | Nobody interpenetrates. | After every request of the R long run and of scenarios n1–n3: the smallest centre distance over every pair of people in the place; the closest pair and request are printed. | Always ≥ **0.595 m**. |
| **N-2** | A nudge is small. | Every nudge's length, per stride. | Every nudge ≤ **310 mm**. |
| **N-3** | Chains are bounded. | Per stride: generations used and people nudged. | Generations ≤ **2**, people nudged ≤ **4**; strides that needed more are counted as blocked and reported. |
| **N-4** | Nobody leaves the room or enters the counter. | Every person's centre after every request. | Inside the walls (0.300 m from each wall, 5 mm tolerance) and outside the counter's footprint grown by the radius (5 mm tolerance). |
| **N-5** | The measurement sees a nudge. | n2 with nudging on, and again with nudging forced to fail. | On: the stationary person moves at least **100 mm** in total. Forced off: they move **0 mm** and the walker stops at contact. |
| **N-6** | Determinism of the scenarios. | Digest of each scenario's final state, two processes. | Equal. |
| **PC-g** | Another architecture gives the same bytes. | Build `x86_64-apple-darwin`, run under Rosetta: PC-a for P, Q, `ql` and R; PC-b for each with the snapshot written **by the arm64 binary** and resumed **by the x86_64 one**. | Every x86_64 digest equals its arm64 counterpart. |

Scenarios, each a fixed request sequence in mode R in the café room:

```text
n1 head-on     A (2.00, 5.00) and B (6.32, 5.00) alternate 0.5 m strides toward each other,
               12 each
n2 stationary  A from (2.00, 5.00) walks +x in 12 strides of 0.5 m; B stands at (4.00, 5.10)
n3 crowd       A from (2.00, 5.00) walks +x in 12 strides of 0.5 m into five standing people at
               (5.00, 5.00) (5.00, 5.65) (5.00, 4.35) (5.65, 5.00) (5.65, 5.65)
```

## 9.7 Revision 1, second set of criteria: mode R′, stated before its runs

The first R runs (§9.8) failed N-1 in the long run: after request 551 two people stood **33 mm**
apart. Located: person 8 walked diagonally toward the south wall and slid along it; person 2 stood
against that wall (y = 314 mm); the character controller **with people solid** (step 2) let the walker
slide onto person 2, so even the blocked fallback overlapped. The character controller is therefore not
a guarantee of non-interpenetration on its own. And c4 picked request 1 000, a move the walls clip to
100 mm, so its 1 mm went nowhere — PC-c″'s fault again, in a new run.

Mode **R′** is R with one added rule, fixed here before it is coded:

```text
8  verify, then degrade. After steps 1–7, check every pair of people in the place on the quantized
   result. If any pair is closer than 600 mm − 5 mm and was not already that close before the
   request, the request is re-resolved as blocked (step 5); if the blocked result still fails the
   check, the walker's advance along the blocked path is halved, up to 8 times; if that still fails,
   the walker stays where they were. A walker who stays is a valid result: the state before the
   request passed the same check.
```

Criteria for R′: PC-a/R, PC-b/R, PC-d/R and N-1 to N-6 exactly as in §9.6, applied to R′. PC-c/R is
measured with c2 and with **c5**: the first move at or after request 1 000 whose base result reaches
its requested destination exactly (it was clipped by nothing), given 1 mm more stride in x. PASS iff
the c5 trace differs from base at that request. The verify-and-degrade fallbacks are counted and
reported by level (blocked, halved, stayed).

The same overlap and inside checks are also **reported, not judged,** for the earlier modes Q and `ql`
over their 3 000 requests, because §9.4 never measured them: the old design blocked strides with the
same character controller, and whether it ever let people overlap is evidence about §6.1's claims.

## 9.8 Revision 1 results (2026-10-07)

Logs in `out/`: `nudge.log`, `nudge-off.log` (R), `runs-r.log`, `rv-n1.log` … `rv-n3.log`,
`runs-rv.log` (R′), `runs-x86.log` (PC-g). Source: `src/resolve.rs` (new) and `src/main.rs`.

### Mode R (§9.6) — superseded by R′, kept as evidence

| ID | Verdict | Evidence |
| --- | --- | --- |
| PC-a/R | PASS | `3261dec016fd15d6…28fd48a7` twice; 59.5 / 59.3 µs per request |
| PC-b/R | PASS | snapshot at 1 300 (129 bytes); resumed to `3261dec0…` |
| PC-c/R | c2 PASS (differs init … request 40, then converges); **c4 FAIL as specified** | c4 picked request 1 000, `person 1 moved (−100, 0) mm`: a move clipped by the walls, so +1 mm went nowhere — the same defect of the c4 rule as PC-c″ located in Q; replaced by c5 in §9.7 |
| N-1 (long run) | **FAIL** | closest pair **33 mm**, persons 2 and 8 after request 551. Located (`r show 551`, `DEBUG=1`): `move person 8 by (1412, −1412) mm from [2536, 782]`; person 2 stood at (2 970, 314), against the south wall. Both sweeps slid the walker along the wall onto person 2: `t_b = (0.433, −0.435)` m, the blocked fallback itself, ended at (2 969, 347). The character controller with people solid let a sliding walker through a person who was touching the wall. |
| N-1…N-6 (scenarios) | PASS | identical to R′ below: the scenarios never reached the failing geometry |

### Mode R′ (§9.7) — the design

| ID | Verdict | Evidence |
| --- | --- | --- |
| **PC-a/R′** | **PASS** | `c98ead06cac20eb7…a84e7b62` in two processes |
| **PC-b/R′** | **PASS** | snapshot at 1 300 (127 bytes); a new process resumed to `c98ead06…` |
| **PC-c/R′** | **PASS** | c2: traces differ from `init` through request 40, then converge. c5 perturbs request 1 001 (`person 2 moved (1153, 0) mm unclipped in base`): traces differ from 1 001 through 1 016, then converge |
| **PC-d/R′** | **PASS** | 61.0 / 61.1 µs per request (≤ 100). 225 of 2 548 moves stepped for objects |
| **N-1** | **PASS** | long run: closest pair **600.1 mm** (persons 2 and 8, request 551 — the R failure, now halved: `degraded: blocked 0, halved 1, stayed 0`). Scenarios: n1 610, n2 610, n3 600 mm |
| **N-2** | **PASS** | largest nudge: long run 309 mm; n1 300, n2 289, n3 301 mm (bound 310) |
| **N-3** | **PASS** | long run: max 2 generations, max 3 people per stride; 178 strides blocked, 398 strides nudged someone (454 nudges). n3: 2 generations, 2 people, 3 of 12 strides blocked |
| **N-4** | **PASS** | nobody outside the room or inside the counter, in the long run or any scenario |
| **N-5** | **PASS** | n2 on: the standing person moved 495 mm in all (≥ 100). Forced off (`NUDGE_OFF=1`): 0 mm, and the walker stopped at contact after 1 403 mm, 12 strides blocked |
| **N-6** | **PASS** | n1 `47ec2419…`, n2 `a690a4f7…`, n3 `61941cab…`, each identical in a second process |
| **PC-g** | **PASS** | x86_64 build (`rustup target add x86_64-apple-darwin`, the stable 1.97.1 toolchain the arm64 build used), run with `arch -x86_64` under Rosetta: P `9ccd9417…`, Q `57863631…`, `ql` `2ca7c6fe…`, R′ `c98ead06…` — each equal to arm64. Snapshots written by the **arm64** binary (P, Q, `ql`, R′) resumed by the **x86_64** binary to the same digests. Snapshots written on x86_64 are byte-identical files to arm64's (P, Q, R′; `cmp`). Scenario digests n1–n3 equal. |

### Reported, not judged: the revision-0 modes

```text
Q    closest pair 604.4 mm (persons 5, 7, request 451)   never overlapped
ql   closest pair 235.0 mm (persons 2, 8, request 2 801) OVERLAPPED — the lean path trusted the
                                                         character controller's answer
```

So revision 0's recommended mode, `ql`, would have let people interpenetrate. That is a finding against
revision 0's §6.1 as written, not only against R; DC-8 is the remedy in either design.

### Findings (revision 1)

- **F-P6 — the character controller is not an interpenetration guarantee.** Sliding along a wall, it let
  the walker onto a person touching that wall (R, request 551) and, in `ql`, left two people 235 mm apart.
  Verify-then-degrade on the quantized integers (DC-8) closed it: one halving in 3 000 requests.
- **F-P7 — collinear head-on walkers never pass.** Each stride nudges the other straight back; they
  oscillate 300 mm (n1). Not a failure of any criterion; a behaviour to fix with a sideways bias (QB-16).
- **F-P8 — the perturbation rule must pick an effective input.** c3 (Q) and c4 (R) both chose moves the
  walls clipped. c5 ("reaches its destination unclipped") is the rule that works; PR tests that perturb
  inputs use it.
- **F-P9 — cross-architecture identity holds under Rosetta**, including across a snapshot written on one
  architecture and resumed on the other. This is the first evidence for `AC-8`'s physics half.

### Commands (revision 1)

```sh
cargo build --release                                     # arm64
B=target/release/physics-spike
for n in n1 n2 n3; do $B rv scenario $n; done             # N-1..N-6 (run twice for N-6)
NUDGE_OFF=1 $B rv scenario n2                             # N-5 off
$B rv run base out/rv-base-1.loc; $B rv run base out/rv-base-2.loc        # PC-a/R′
$B rv snap base out/rv.snap; $B rv resume out/rv.snap out/rv-resumed.loc  # PC-b/R′
$B rv trace base out/rv-base.trace; $B rv trace c5 out/rv-c5.trace
$B tracediff out/rv-base.trace out/rv-c5.trace                            # PC-c/R′
$B r run base out/r-base-1.loc; $B r show 551; DEBUG=1 $B r show 551      # R, and locating N-1
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/physics-spike out/spike-x86_64
arch -x86_64 out/spike-x86_64 rv run base out/x86-rv.loc                  # PC-g (a); p, q, ql too
arch -x86_64 out/spike-x86_64 rv resume out/rv.snap out/x86-rv-resumed.loc   # PC-g (b), arm64 snapshot
arch -x86_64 out/spike-x86_64 rv snap base out/x86-rv.snap; cmp out/x86-rv.snap out/rv.snap
```

The resolver, as the prototype implements it (`src/resolve.rs`, excerpt; the design's trait shape is
§4.4.1):

```rust
pub const NUDGE_MAX: f32 = 0.300;
pub const GAP: f32 = 0.010;
pub const CHAIN_MAX: usize = 2;
pub const NUDGED_MAX: usize = 4;

fn resolve_people(world: &mut PhysicsWorld, people: &[RigidBodyHandle], walker: usize,
                  desired: Vector, objects_solid: bool, force_block: bool)
                  -> (Vector, Option<(Vec<(usize, f32)>, usize)>) {
    let me = people[walker];
    let from = world.bodies[me].translation();
    let walls = if objects_solid { QueryFilter::exclude_kinematic() } else { QueryFilter::only_fixed() };
    let t_w = sweep(world, me, desired, walls);                       // 1 walls-only reach
    let solid = if objects_solid { QueryFilter::default() } else { QueryFilter::exclude_dynamic() };
    let t_b = sweep(world, me, desired, solid);                       // 2 contact reach
    let blocked_at = from + t_b;
    let (s_w, s_b) = (len(t_w), len(t_b));
    let candidate = if s_w <= s_b + NUDGE_MAX { from + t_w }          // 3 candidate
                    else { from + t_w * ((s_b + NUDGE_MAX) / s_w) };
    // 4 nudge pass: generations of pushers; each overlapping person is swept against fixed
    //   geometry only, directly away from its pusher, by the overlap + GAP; any bound broken,
    //   or any overlap left, → 5 blocked (walker at blocked_at, everyone else restored)
    /* … */
}

/// R′ step 8: verify on integers, then degrade.
pub fn move_r(state: &mut StateQ, variant: Variant, walker: usize, desired: Vector) {
    let before = state.clone();
    /* resolve, quantize */
    if !VERIFY.load(Ordering::Relaxed) || !new_overlap(&before.people, &state.people) { return; }
    /* blocked → halved along the blocked path (1/2 … 1/256, integer division) → stay */
}
```

---

# 10. Proposed contracts and invariants

## 10.1 Frozen invariants (proposed; frozen only by the operator or primary session)

Revised for revision 1: I-1, I-6 and I-7 are rewritten, I-11 to I-13 are new.

- **I-1 The kernel and the contracts are untouched.** `kernel/`, `contracts/`, `persistence/` and
  `server/src` have no diff in any S15 PR. Outside the new `bodies`, only the precursor's set changes —
  presence, the sdk's `installed!`, `systems/installed`, `worldpack`'s compose, and movement's two lines
  (§8.2) — and only in PR 12a. A need to edit anything else, or to add the kernel extension slot (F2), is
  a material stop.
- **I-2 Single ownership.** `Presence` and `present-in` are written only by presence's reductions.
  Body shapes, place geometry and where a loose object lies are written only by bodies' reductions. A
  change to where a person is travels only as presence's `arrived`, built by presence's constructors.
- **I-3 No float is persisted.** Every value in a component, fact or journal entry is an integer:
  millimetres, millidegrees, counts. Floats exist only inside one resolution.
- **I-4 Determinism conditions.** `rapier3d` pinned exactly, `enhanced-determinism` on, `simd8` and
  `parallel` off; canonical insertion order; fixed dt and bounded sub-steps; one quantization function;
  directions without `std` transcendental functions (DC-1 to DC-6).
- **I-5 Nothing of Rapier survives a resolution.** No component, field, global or cache holds Rapier
  state. A resolution is a pure function of the world's integer state and the request.
- **I-6 Uniformity (R-2), resolved before recorded (QB-1).** Every `arrived` is built by presence's
  constructors after every registered resolver has answered; `arrival()` refuses what it cannot record
  truthfully. No `arrived` is ever recorded and then corrected: the log holds only true arrivals.
- **I-7 Existing worlds are byte-identical.** With no resolver registered, the facts and journal inputs
  of the 300-day seed-7 runs of social-cafe and market-town are identical before and after PR 12a; only a
  save's composition record differs, and old saves are refused by name (§4.4.8). No digest is
  re-baselined in 12a.
- **I-8 Clients decide nothing.** No rule about blocking, pushing, reach or force exists in a client. Local
  collision is prediction; removing it changes no outcome.
- **I-9 Removability (`AC-2`).** A world without bodies answers `kick`, `throw` and `shove` with
  `ActionUnavailable` and is otherwise the same world as before this step.
- **I-10 Locate before counting (`ARC-23`).** The non-interpenetration check reports the closest pair, the
  place and the instant, not only a count; and it is shown to fail on a world where bodies is disabled.
- **I-11 Nudges are bounded (QB-10).** Per stride: no person moved by more than `NUDGE_MAX + GAP`
  (310 mm), at most `CHAIN_MAX` (2) generations and `NUDGED_MAX` (4) people; a stride that needs more is
  blocked. A nudge never changes place and never passes through fixed geometry.
- **I-12 The invariant is checked, not trusted.** After every resolution, on quantized integers, no pair
  of people in a place is closer than 595 mm unless they already were; the fallbacks are blocked, halved,
  stay (DC-8). The character controller's answer is never the last word (F-P6).
- **I-13 Resolvers are pure, inert when absent, and ordered.** A resolver reads only its `WorldRead`,
  writes and emits nothing, keeps nothing, returns `so_far` when its own state does not name the people
  involved, and is asked in ascending `SystemId` order. Movement names no resolver.

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
            object-moved, person-shoved                    reactions and actions
            arrived, stride-blocked (presence's, ARC-26)   for `shove`, through arrivals()
resolves    ArrivalResolver (presence's trait), listed in `installed!`'s `resolution:` line
subscribes  arrived (presence's: object pushes, invariant guard); its own genesis facts
authored    one section, `body:` (ARC-31), in person, place and item files (F-47: one section per
            pack)
discloses   PlaceShape to whoever perceives the place, and BodyShape and Lying of what the
            observation lists — so a client can build the same colliders the server uses (R-B4)
```

## 10.3 What presence gains (revision 1; revision 0's `from` is withdrawn)

```text
trait ArrivalResolver, Stride, Resolution          §4.4.1
register_resolvers(Vec<Box<dyn ArrivalResolver>>)  write-once catalog (§4.4.4, F1)
arrivals(world, person, to) -> Vec<Emission>        resolves; walker, nudged, stride-blocked
arrival(world, person, to)  -> Emission             unchanged signature; refuses a placement a
                                                    resolver would change
StrideBlocked { person, wanted, reached, by }       new event type, schema 1
Arrived                                             unchanged, schema 1
PresenceSystem::VERSION                             2 → 3
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
Each PR has an integration checkpoint that runs the product, not a unit. All start after 11f merges.

## 11.1 Revision 1 split: the precursor seam first

| PR | Scope | Integration checkpoint | Adversarial criterion |
| --- | --- | --- | --- |
| **12a** The arrival-resolver seam (framework precursor; names no physics) | ARC-39 into `docs/DECISIONS.md`. presence: `ArrivalResolver`, `Stride`, `Resolution`, the catalog, `arrivals()`, `arrival()` refusing what it cannot record truthfully, `stride-blocked`, VERSION 3. sdk: `installed!`'s `resolution:` line and `Capability::resolvers()`. `systems/installed`: `resolution: … => [];`. worldpack: compose registers the catalog. movement: the two lines. A test-only `fences` resolver (§4.4.9), never installed. | SC-1 to SC-7 through the real dispatch and persistence: with fences, the recorded arrival is the stopped one, the displaced person's arrival is caused by the walker's `ActionId`, SIGKILL and resume byte-identical; without it, the 300-day seed-7 facts of social-cafe and market-town are byte-identical to main (I-7). | A resolver that lengthens a stride, changes place or displaces the walker is refused, never recorded. A scan in the style of the I-2 scan: the PR names no body, physics, Rapier, nudge or collision. |
| **12b** People: walls and nudging | DEP-13. `systems/bodies` with `BodyShape`, `PlaceShape`, the `body:` section, genesis facts, and its resolver: walls, nudging, verify-then-degrade (§4.5), Rapier behind one module; F-P1 regression test; the invariant guard; a test world with walls and a crowd. | Headless: a seeded test world with twelve people in two walled rooms runs 30 days; after every request no two people in a place are closer than 595 mm, nobody is outside the walls, every nudge ≤ 310 mm, chains ≤ 2 generations and 4 people; SIGKILL and resume byte-identical; n1–n3 reproduced as tests; the same world without bodies shows overlaps (I-10). | Mutations: remove verify-then-degrade (the R N-1 failure must reappear), raise `NUDGE_MAX`: the bound test fails by name. Sideways bias for collinear head-on walkers decided here (QB-16). |
| **12c** Objects: push, kick, throw; shove | `Lying`, `object-moved`, `person-shoved`; bodies' reaction pushes objects out of each recorded arrival, predicted by the resolver (§4.5.4); `kick`, `throw`, `shove` with declared requirements; complete affordances for `kick` and `shove`. | Headless: the unchanged paced controller kicks and shoves (`ARC-34`); every fact is caused by an action; a person walking into a box moves it; a jammed box stops the person; a shove into a crowd nudges within I-11; resume byte-identical; QB-11's cost bound on a 300-day run. | A thrown object never ends inside a wall or a person; `TooFarAway` from 801 mm and acceptance at 800 mm. |
| **12d** The town gets bodies | `body:` sections for social-cafe's places matching the 3D slice's layout, default capsules, a few loose objects; bodies installed in social-cafe and market-town (QB-5); the validator checks of §10.4. Digests re-baselined here, and only here, with the activity precondition checked first. | The 300-day seed-7 runs of both worlds hold the activity precondition (`ARC-23`) with walls in place; no overlap after any request; runs reproduce byte for byte; wall time within QB-11. | A doorway authored inside a wall, or two people authored overlapping, is refused at load by name. |
| **12e** The 3D client | Jolt selected explicitly; people and objects get colliders at observed positions (shapes from disclosure); reconciliation on difference (150 mm), which now also covers being nudged; `kick`, `throw`, `shove` from the camera ray; objects animated along `path`; `server/PROTOCOL.md` §6.2 and `ADOPTION.md`. After `vis/3d-godot-2-environment` merges (QB-13). | The client run for real (`ENGINEERING_RULES.md` §19): a scripted drive walks the player into Alice, who is nudged aside; into a crowd, which blocks; into a box, which moves; kicks it; is shoved; frames inspected, and the server's facts match what is drawn. | No rule in the client: with its people colliders removed, the server still resolves and the client reconciles. |

## 11.2 Revision 0's split (superseded)

| PR | Scope | Integration checkpoint | Adversarial criterion |
| --- | --- | --- | --- |
| **12a** People do not interpenetrate | DEP-13, ARC-39 into `docs/DECISIONS.md`; presence `from` (VERSION 3, digests re-baselined per I-7); `systems/bodies` with `BodyShape`, `PlaceShape`, the `body:` section, genesis facts, and the arrival resolution against people and fixed geometry (character controller only, no dynamics); F-P1 regression test; a test world with walls and a crowd. | Headless: a seeded test world with twelve people in two walled rooms runs 30 days; at every observation instant no two people in one place are closer than 595 mm, and nobody is outside the walls; the run is SIGKILLed mid-way and resumes byte-identical (`ARC-25`); the same run with bodies removed shows overlaps (I-10) and answers nothing new. | Disable the resolution (mutation): the overlap check fails, naming the pair. A save from before is refused by name. |
| **12b** Objects: push, kick, throw; shove | `Lying`, `object-moved`, `person-shoved`; walking pushes objects (stepped only when the stride meets one, `ql`); `kick`, `throw`, `shove` with declared requirements; complete affordances for `kick` and `shove`. | Headless: in the test world the unchanged paced controller kicks and shoves (it attempts complete affordances, `ARC-34`); facts are caused by actions; a person walking into a box moves it; a jammed box stops the person; resume byte-identical. Cost bound of QB-11 measured on a 300-day run. | A thrown object never ends inside a wall or a person; a shoved person never ends inside anything; `TooFarAway` from 801 mm and acceptance at 800 mm. |
| **12c** The town gets bodies | `body:` sections for social-cafe's places (café, street, the shops) matching the 3D slice's layout, its people's default capsules, and a few loose objects; bodies installed in social-cafe and market-town (QB-5); the validator checks of §10.4. | The 300-day seed-7 runs of both worlds hold the activity precondition (every seat moved and talked in every 30-day bucket, `ARC-23`) with walls in place; no overlap at any observation; runs reproduce byte for byte; wall-time increase within QB-11's bound. | A doorway authored inside a wall is refused at load by name. |
| **12d** The 3D client | Jolt selected explicitly; people and objects get colliders at observed positions (shapes from disclosure); reconciliation on difference (150 mm); `kick`, `throw`, `shove` from the camera ray; objects animated along `path`; `server/PROTOCOL.md` §6.2 and `ADOPTION.md` updated. Lands on, or after, the visual track's `vis/3d-godot-2-environment` (QB-13). | The client run for real (`ENGINEERING_RULES.md` §19): a scripted drive walks the player into Alice and stops at her; walks into a box and pushes it; kicks it; is shoved by a test seat; frames captured and inspected, and the server's facts match what is drawn. | No blocking rule in the client: with the client's people colliders removed, the server still stops the player and the client reconciles. |

The 2D client's intents (`kick`, `throw`, `shove` by click) belong to S12, which builds that client; they
need nothing from the server that 12c does not already provide.

---

# 12. Where this step sits

```text
S9   11f, the AC-1 proof — finishing now. S15 starts after 11f merges: 12a changes presence,
     movement, the sdk and worldpack, which must not land inside the AC-1 measurement, and 12d
     re-baselines both worlds' digests.
S15  this step: 12a (seam) → 12b (people) → 12c (objects, actions) → 12d (the town), then 12e
     (3D client).
S10  cognition (reduced): independent of S15.
S11  server: nothing new needed; the server carries any action unchanged.
S12  2D client: consumes 12c's actions; no dependency the other way.
S13  CI and container parity: PC-g ran under Rosetta (§9.8); a native x86_64 host in S13's
     container is the stronger check of AC-8 for physics.
S14  Demo B: its required "collision / basic navigation" and "interact with at least one
     Item/Object" (ENGINEERING_RULES §10) are delivered in substance by 12d and 12e. S15
     therefore precedes S14.
```

## 12.1 Proposed amendment to `overall.md` (not applied; for the operator)

In §3, after S14:

```markdown
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
```

In §4, a row: `Operator requirement 2026-10-07: bodies do not interpenetrate; nudge, push, kick, throw |
S15`.

In §7, under "Next": `S15 bodies (step-11, DRAFT revision 1, 2026-10-07): PRs 12a–12e after 11f; the
operator's decisions on QB-1, QB-10, QB-2, QB-3, QB-4, QB-5, QB-12 applied; open: QB-15 (catalog or
kernel slot), QB-16, QB-17.`

---

# 13. Open questions

Operator-material questions are marked **[OM]**. Each has a recommendation.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QB-1 [OM]** | Accept option C — bodies corrects arrivals after the fact, so a blocked stride leaves `arrived X`, `stride-blocked`, `arrived X′` in the log — over A (movement depends on bodies) or B (kernel geometry seam)? | Revision 0: **C.** **DECIDED 2026-10-07: rejected.** Resolve before recording through a provider seam (§4.3–§4.4, F). |
| **QB-2 [OM]** | Change presence's `arrived` to carry `from` (schema 2, presence VERSION 3, old saves refused, every world's digest re-baselined)? | Revision 0: yes. **Accepted, then re-checked under QB-1's decision: withdrawn.** The resolver receives `from` from presence directly; `arrived` keeps schema 1 (§4.4.7). |
| **QB-3 [OM]** | What is a loose object? (a) an Item file with a `body:` section, which then names exactly one physical object — amending `ARC-36` ("stacked items only") for such items; (b) a new World Pack content kind `objects/` (a `worldpack` change); (c) pack-local keys that are not entities (against "everything persistent is an Entity"). | **(a)**, with the validator refusing an item that has a body and is also held in an inventory. **DECIDED: (a), amending ARC-36's wording** (in 12c). |
| **QB-4** | `rapier3d` (z up, capsules) or `rapier2d` (circles and boxes on the floor)? | **`rapier3d`.** Objects resting on furniture and a later stairs/ramp need the third axis; the cost is measured (§9.4, §9.8) and acceptable. **DECIDED: rapier3d.** |
| **QB-5 [OM]** | Which worlds install bodies? | **social-cafe and market-town**, now in **12d**. The 3D slice walks in social-cafe, and that is where the operator saw the defect. Both worlds' seed-7 runs are re-baselined there, with the activity precondition checked first. **DECIDED as recommended.** |
| **QB-6** | Should a kicked or thrown object land later (a deferred fact at the end of its flight) rather than at the instant of the kick? | **Not now.** Resolve at the instant and state the path; the client animates it. A deferred landing is a later refinement and needs no contract change. |
| **QB-7** | Should the 3D client predict object pushes locally with a Jolt `RigidBody3D`? | **Not in 12e.** Objects are drawn where the server says; pushes show at the next observation (≤ 100 ms). Revisit if it feels wrong when played. |
| **QB-8** | Should `shove` be refused (`Busy`) while the target is seated or in an activity? | **Not now.** There is no generic "may this person be interrupted" query; adding one is outside this step. |
| **QB-9** | Line of access (talking through a wall) now that a layer owns geometry? | **A later step**, probably the kernel geometry seam (option B), because every pack's `SpatialRequirement` would use it. Not needed for R-1 to R-4. |
| **QB-10 [OM]** | Walking into a person: blocked (stop and slide), or nudged aside ("挤开")? | Revision 0: blocked. **DECIDED 2026-10-07: nudged aside**, bounded (§4.5; I-11). `shove` stays the larger, deliberate displacement. |
| **QB-11** | The wall-time bound for a 300-day run with bodies installed. | **At most +50 %** over the same world without bodies, measured in 12c and 12d. A bound from the requirement, not from what the code costs. |
| **QB-12 [OM]** | Authorize `rustup target add x86_64-apple-darwin` so PC-g (another architecture under Rosetta) can be run? | **DECIDED: authorized. Run in revision 1: PASS** (§9.8). |
| **QB-13** | The 3D slice (`slice_link.gd`) lives on the unmerged visual branch `vis/3d-godot-2-environment`. Does the client PR (now 12e) land there, or after it merges? | **After it merges**, so the client work is reviewed on main. 12a–12d do not depend on it. |
| **QB-14** | A speed limit (S6 L-1)? | **Out of scope.** Bodies does not change how often a client may move. |
| **QB-15 [OM]** | How do resolvers reach presence's constructor inside `dispatch`? F1: a write-once, process-wide resolver catalog in presence, registered by `worldpack::compose` from `installed!` (no kernel change). F2: a generic extension slot in the kernel (`World::provide` / `WorldRead::provided`), per world (a kernel change). | **F1** for MVP-0 (§4.4.4): the catalog is compiled-in code identical for every world, not world state; resolvers are inert where their state is absent; bodies' reaction guard and `ARC-25` replay make a forgotten registration loud. F2 if the operator prefers no process-wide value at all. |
| **QB-16** | Collinear head-on walkers push each other back and never pass (F-P7). Add a fixed sideways bias to a nearly collinear nudge? | **Yes, decided and measured in 12b** with the n1 scenario as its test: they must pass within a stated number of strides. |
| **QB-17 [OM]** | Honour runtime `World::disable` of a resolver pack? Needs an additive kernel read, `WorldRead::is_enabled(SystemId)`. | **Not now.** `AC-2` for bodies is shown at world level (a world that does not list it). Revisit if a world ever needs to disable bodies while running. |

---

# 14. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-B1** | Rapier breaks its API about monthly (0.33–0.36 in four months). | Pin exactly (DC-1). An upgrade is its own PR: bump bodies' VERSION, re-run the prototype's criteria in the pack's tests, re-baseline. Rapier sits behind one module of the pack. |
| **R-B2** | More Rapier defects like F-P1. | Regression test per defect; the per-request model touches a small, well-trodden part of the API (character controller, insertion, a few steps). |
| **R-B3** | Headless cost grows past QB-11's bound. | The integer pre-check (skip Rapier when no body's box meets the stride's box); per-place worlds are small. Measured in 12c and 12d, never assumed. |
| **R-B4** | Server geometry (`body:` sections) and the client's scene disagree, so players are corrected at invisible walls. | One source: bodies discloses `PlaceShape`, and the 3D client builds its collision from the disclosure, not from its scene. The visual scene stays the client's. |
| **R-B5** | Event volume: a blocked stride records three facts instead of one. | Only blocked strides pay. Measured in 12c's 300-day runs and reported. |
| **R-B6** | ~~A subscriber reacts to the transient `arrived X`.~~ Retired by revision 1: nothing transient is recorded. Remaining form: a subscriber to `arrived` now also sees nudged people's arrivals. | Audited: only presence subscribes to `arrived` today; employment and group-activity subscribe to `person-entered-place`, which a nudge never emits (it never changes place). |
| **R-B7** | Cross-architecture float identity. | Revision 1: **evidenced under Rosetta** (PC-g PASS, §9.8), including an arm64 snapshot resumed on x86_64. A native x86_64 host (S13) is the remaining, stronger check. |
| **R-B9** | The character controller is trusted where it errs (F-P6). | DC-8 / I-12: checked on integers after every resolution, with fallbacks; a mutation test in 12b removes the check and must fail. |
| **R-B10** | A host assembles a world without `worldpack::compose` and never registers the resolver catalog (F1). | Every host composes through worldpack today; bodies' reaction guard fails the dispatch at the first overlapping arrival; a replay by such a host diverges and is refused (`ARC-25`). F2 (QB-15) removes the risk at the cost of a kernel change. |
| **R-B11** | A second resolver pack interacts badly with bodies. | Order fixed by `SystemId`; presence's final checks (§4.4.6) bound what any resolver can do; out of MVP-0 scope until a second resolver exists. |
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
found in 0.36.0 (F-P1) and worked around. Its character controller is not an interpenetration guarantee
(F-P6); the pack checks the invariant itself (DC-8). Cross-architecture identity evidenced under Rosetta
(arm64 ↔ x86_64, §9.8), not yet on a native x86_64 host. No momentum between resolutions. Apache-2.0,
compatible with MIT.

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

## 15.3 ARC-39 — An arrival is resolved before it is recorded (revision 1)

**Problem.** Non-interpenetration must hold for every way a person can come to be somewhere, without a
kernel change, without making the deciding packs depend on physics, and — the operator's decision of
2026-10-07 — without ever recording an arrival that is then corrected.

**Choice.** Presence asks before it records. Its constructors build a `Stride` from its own `Presence` and
fold it through every registered `ArrivalResolver` (a presence-owned trait, read-only, pure, asked in
`SystemId` order), then check the result as the owner and record only it: the walker's `arrived`, an
`arrived` for each person moved by the stride, and its own `stride-blocked` when the walker stopped
short — all stated by the stating system, in one emission list, caused by the request. `arrival()` keeps
its signature for placement and refuses what a resolver would change; `arrivals()` is for movers. The
resolvers reach presence through a write-once catalog registered from `installed!`'s `resolution:` line
by `worldpack::compose` (F1). With no resolver registered every world records exactly what it recorded
before. A `bodies` System Pack is the first resolver; movement still alone decides whether a move is
allowed and names no resolver.

**Bounds stated with the choice.** Bodies' nudge: at most 310 mm per person per stride, two generations,
four people, never through fixed geometry or into another place; the invariant is checked on integers
after every resolution, with blocked, halved and stay as fallbacks.

**Rejected.** Correcting after recording (revision 0's C: true-in-sequence but transient facts, refused
by the operator); movement depending on bodies (bodies no longer removable; every future mover must
remember); a kernel geometry seam (a kernel change, the likely route for line of access later); a kernel
extension slot for the resolvers (F2: per-world rather than per-process, but a kernel change — QB-15); a
persistent stepped world (no idle skipping: about 82 hours per café per 300 days, §9.4).

**Accepted limitations.** A process-wide catalog (one per build). Runtime disable of a resolver pack is
not honoured (QB-17). A resolver cannot emit; its pack's consequences happen in its reactions to the
recorded facts, which the resolver must predict (bodies' object pushes, §4.5.4).
