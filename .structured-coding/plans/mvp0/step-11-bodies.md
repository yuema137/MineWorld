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
- **PR 12a is detailed to the commit in §16** (planning session, branch `mvp0/s15-12a-plan`, from
  `main @ b8afd4f`, 2026-10-07). It is not frozen. Its open questions are QR-1 … QR-12 (§16.8).
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

---

# 16. PR 12a — the arrival-resolver seam (full design; DESIGN FROZEN 2026-10-07)

## 16.0 Freeze record

**DESIGN FROZEN (2026-10-07), primary session.** The execution contract (§16.9) is confirmed. This
record binds and overrides any other text in §16.

The design was detailed by the planning session on `mvp0/s15-12a-plan`, from `main @ b8afd4f`. That
base has 11f merged as `fea2516`; the S9 closeout #52 merged after it and changed Markdown only.

- **QR-2 — accepted. This is the primary session's call, not the operator's.**
  - The VERSION 3 bump updates the three "presence v2" test literals. Their claim is unchanged,
    under the same rule as step-09 I-5. RS-1 reads "every existing test passes, the three named
    literals updated with unchanged claims".
  - The operator's QB-15 decision chose start-up registration. The test literals are mechanics
    below that decision.
- **QR-4 — accepted as recommended. This is the primary session's call.** The "fails loudly" bound
  in the step freeze record was written by the primary session, not by the operator. Its intent is
  that a resolver must never be silently skipped, and it is met as follows:
  - `compose` always registers the catalog.
  - A resolver pack panics at install if the catalog is missing or does not list it.
  - A host with no resolver pack runs as it does today.
  - RS-9 and its mutation prove the panic.
- **QR-1, QR-3, QR-5 … QR-12 — accepted as recommended.** This includes the renames to `Arriving`
  and `stopped-short`, the amendment that makes 12b own overlapping-at-genesis, and real cross-build
  evidence for the save refusal.
- **Merge:** a merge commit. **Implementation:** in a fresh session on `mvp0/pr-12a-resolver-seam`,
  in its own worktree.

This section refines §4.4, §4.4.9, §8.2, §10.1 and §11.1's 12a row from source. Where they and §16
disagree, §16 governs, and each difference is named with the finding that caused it (§16.2) and the
question that asks for it (§16.8). Two differences need the operator: **QR-2** (an existing test literal
the version bump breaks) and **QR-4** (how "a host that never registers fails loudly" is delivered).

## 16.1 Identity, base, approved scope

```text
PR            12a — the arrival-resolver seam (S15, first of five; a framework precursor that names no
              physics)
base          main @ b8afd4f, or the main the primary session names at freeze. Re-audit §16.2 if anything
              under systems/presence, systems/movement, sdk/rust, systems/installed, worldpack/src,
              tests/acceptance, tools/cli/tests or kernel/src moved
branch        mvp0/pr-12a-resolver-seam, in its own worktree, held by the implementing session only
audit         §16.2 (b8afd4f), including the planning measurement E-RS0 (§16.10)
scope         §4.4 as refined by SD-R1 … SD-R14; §4.4.9's SC-1 … SC-7 plus SC-8 (the kernel-adjacent
              fact of §8.2); QB-1, QB-15 (F1, with the operator's three bounds); QR-1 … QR-12 as answered
depends on    11f merged (fea2516). Nothing of S15 before it.
merge         a merge commit, never a squash
```

**Goal.** Presence resolves an arrival before it records it. Its constructors ask every registered
`ArrivalResolver` what the arrival actually achieves and record only that: the walker's `arrived`, an
`arrived` for each other person the arrival displaced, and presence's own `stopped-short` when the walker
ends short of where they asked. With no resolver registered — every build until 12b lists `bodies` —
every world records exactly what it records today. The seam is proven by synthetic resolvers that live in
test files only and name no physics.

**Change set.** Every path this PR may touch:

```text
docs/DECISIONS.md                         ARC-39 (new; appended after ARC-38)
docs/MODULE_SPEC.md                       §3.1: the installed set's `resolution:` line; a resolver pack's
                                          obligations
systems/README.md                         "Adding a pack": a pack that resolves arrivals
systems/presence/src/resolve.rs           new: ArrivalResolver, Arriving, Resolution, the catalog
systems/presence/src/event.rs             StoppedShort; arrivals(); arrival() refusing what a resolver
                                          would change
systems/presence/src/system.rs            VERSION 3, and its doc comment
systems/presence/src/lib.rs               module, re-exports, the doc table
systems/presence/README.md                the doc table, one paragraph
systems/presence/tests/resolver_catalog.rs   new: the catalog's registration rules (RS-8)
systems/movement/src/system.rs            the two lines of §4.4.2 (SD-R9)
sdk/rust/src/installed.rs                 installed!'s optional `resolution:` line; Capability::resolvers()
sdk/rust/src/lib.rs                       the doc table (one line)
systems/installed/src/lib.rs              `resolution: mineworld_presence::ArrivalResolver => [];` and its
                                          doc
systems/installed/tests/resolution.rs     new: every listed resolver is an installed pack (SD-R8)
worldpack/src/load.rs                     compose() registers the catalog (one call) and its doc
worldpack/tests/registration.rs           new: compose registers the installed set (RS-10)
tests/acceptance/Cargo.toml               dev-dependencies mineworld-movement, mineworld-persistence
                                          (both existing workspace entries); one [[test]] harness = false
tests/acceptance/src/lib.rs               the doc table (comments)
tests/acceptance/tests/resolvers/mod.rs   new: the synthetic packs (SD-R11) and the world they share
tests/acceptance/tests/arrival_resolvers.rs              new: SC-2 … SC-5, SC-8, inertness, QB-17
tests/acceptance/tests/arrival_resolvers_unregistered.rs new: a host that never registers (RS-9)
tests/acceptance/tests/arrival_resolvers_resume.rs       new: SIGKILL and resume (RS-11)
tests/acceptance/tests/seam_vocabulary.rs                new: SC-7 and "names no physics" (RS-13)
tools/cli/tests/inspect.rs                one literal: `presence v2` → `presence v3` (QR-2)
tools/cli/tests/social_composition.rs     two literals: the same (QR-2)
Cargo.lock                                mineworld-acceptance's dependency list gains two names; nothing
                                          else
docs/MVP_STATUS.md                        one capability row and one evidence row (RS-C8)
.structured-coding/plans/mvp0/{step-11-bodies,handoff}.md   this ledger, the handoff
```

No path under `kernel/`, `contracts/`, `persistence/`, `server/`, `cognition/`, `clients/`, `worlds/`,
`authoring/`, `tools/cli/src/`, nor the root `Cargo.toml`. No System Pack other than presence and
movement. No existing test is edited except QR-2's three literals.

**Non-goals.** No `bodies` pack, no Rapier, no geometry, no nudge rule (12b). No new action. No change to
`arrived`'s schema (QB-2 withdrawn, §4.4.7). No client or protocol change: no world records a
`stopped-short` until a resolver is listed. No kernel read for runtime disable (QB-17 deferred). No
re-resolution of an `arrived` at reduction (F-R11).

## 16.2 Source audit (`main @ b8afd4f`, 2026-10-07)

Every finding below was read in this session from the file named, or measured.

| ID | Finding | Evidence | Consequence for 12a |
| --- | --- | --- | --- |
| **F-R1** | **Presence's sources may not contain the word `stride`.** Two merged structural tests forbid it: presence's own scan lowercases each line and refuses `stride`, `movement`, `passage`, `talk`, `spoke`, `conversation`, `utterance`, `f32`, `f64`; movement's scan refuses `stride`, `Passage`, `passage`, `MAX_STRIDE`, `MovementSystem`, `mineworld_movement` in presence's `src/`, case-sensitively. | `systems/presence/tests/presence.rs:813–858`; `systems/movement/tests/movement.rs:499–523` | §4.4's `Stride` and `stride-blocked` cannot be written in presence without editing two tests that hold `ARC-26`'s "presence never names who moves them". Renamed (SD-R1, **QR-1**): `Arriving`, `stopped-short` / `StoppedShort`. Presence's prose must also avoid those substrings, including inside other words (`bespoke` contains `spoke`). |
| **F-R2** | **Three existing test literals name presence's version.** `inspect` prints each system as `<id> v<version>`, and three assertions hold `presence v2`. | `tools/cli/tests/inspect.rs:36`; `tools/cli/tests/social_composition.rs:380, 420`; `tools/cli/src/inspect.rs:72–83`; `SystemVersion`'s `Display` is `v{n}` (`kernel/src/system.rs`) | VERSION 3 fails them. "Every existing test passes unchanged" and "presence's VERSION changes" cannot both hold literally (**QR-2, operator-material**). |
| **F-R3** | **Who calls `arrival()`: sixteen files, as §4.4.2 counted, but split differently.** Library callers: `systems/movement/src/system.rs:110` (resolve) and `worldpack/src/catalog.rs:82` (`located`, every authored location at genesis). Test callers: thirteen integration-test files — `systems/{consumption,economy,employment,group-activity,inventory,item-transfer,movement,relationships,schedule}/tests/support/mod.rs`, `systems/conversation/tests/conversation_and_presence.rs`, `systems/naming/tests/naming.rs`, `systems/presence/tests/presence.rs`, `tests/acceptance/tests/complete_affordances.rs` — plus one unit test, `worldpack/src/load.rs:424–455` (`Trespasser`, inside `#[cfg(test)]`). §4.4.2 named `load.rs` as genesis placement; genesis placement is `catalog.rs`'s `located`, called from `load.rs`'s `initial_facts`. | `git grep -n -w arrival` | `arrival()` keeps its signature, so none of the sixteen changes. Its new refusal can trigger only where a resolver is registered and its state is present, which no existing caller has. |
| **F-R4** | **At genesis a resolver sees nobody.** `WorldPack::initial_facts` builds every location fact against the assembled world before genesis applies any of them, so every `arrival()` at load sees no `Presence` anywhere: `from` is `None` and no other person is placed. | `worldpack/src/load.rs:243, 287–322`; `catalog.rs:77–87` | §4.4.2's "a genesis placement that overlaps another body is therefore refused at load, loudly, by the owner" is not delivered by `arrival()`. Two authored people overlapping must be refused by bodies' validator (§10.4), as 12b and 12d already plan (**QR-7**). |
| **F-R5** | **A save records each system's whole declaration, three times.** The manifest's `composition`, every snapshot's `composition`, and the genesis journal row's `before` snapshot each hold `InstalledSystemRecord { declaration: SystemDeclaration { system, version, depends_on, owns, provides, emits, emits_owned_by_others, subscribes }, enabled }`. `check_composition` refuses the first difference, and a same-position version difference by name: `PersistedSystemOutdated`, "system 'presence' is v3 here, but the save was written by the older v2". | `persistence/src/format.rs:19–34`; `input.rs:46–57`; `replay.rs:128–170`; `error.rs:43–76`; `kernel/src/snapshot.rs:45–60`; `kernel/src/system.rs:100–113` | What changes in a save is exactly SD-R13. Facts and every non-genesis journal row are unchanged. |
| **F-R6** | **The 300-day comparison reads facts only.** `mineworld run` in memory prints counts, activity and an FNV fingerprint of every recorded fact's encoding; no composition reaches its output. | `tools/cli/src/run.rs:168–176, 370–508` | RS-1's two digests compare exactly the facts. E-RS0 re-measured both on this base: unchanged from 11f. |
| **F-R7** | **Subscribe-and-emit is allowed by the kernel, and a system hears what it stated.** `check_installable` refuses only an emitted foreign vocabulary whose owner is not a declared dependency; nothing refuses subscribing to a type one also emits. `reduce` delivers each fact to every enabled subscriber in registration order, the emitter included. No merged pack does this with a foreign vocabulary yet (`economy` states inventory's `items-transferred` but does not subscribe to it). | `kernel/src/registry.rs:170–205`; `kernel/src/dispatch.rs:603–660`; `kernel/src/system.rs:161–183`; every pack's declaration | 12b's `shove` needs no kernel change. 12a commits the evidence as SC-8 (RS-12). Termination stays the subscriber's duty (`CASCADE_DEPTH_LIMIT` = 16). |
| **F-R8** | **`WorldRead` cannot see the composition.** It holds entities, components, relations and processes; no registry, no enabled state. `component::<C>` answers `None` for a type no installed system declared. | `kernel/src/view.rs:53–167`; `kernel/src/components.rs:185–211` | Confirms §4.4.4: the catalog must reach presence outside the world (F1), a resolver is inert where its table is absent, and runtime disable cannot be honoured without a kernel read (QB-17). |
| **F-R9** | **Every production host composes through `worldpack`.** The only `World::new()` in production code is `WorldPack::compose`; `server` is handed a built world (`HostedWorld::new`); every `mineworld` command (`run`, `server`, `create`, `biography`, `inspect`) goes through `load`, `assemble` or `compose`. Tests build worlds by hand: kernel, persistence, server and every pack's tests, about twenty files. | `git grep "World::new()"` over `*/src`; `tools/cli/src/{main,run,create,biography}.rs`; `server/src/lib.rs:49–51` | Registration in `compose` reaches every host. Presence's constructors must treat "never registered" as "no resolver", or about twenty test files fail (**QR-4**). |
| **F-R10** | **`installed!` is invoked twice, once by a test with no resolver.** `systems/installed/tests/installed.rs:128–133` (`twins`) invokes the real macro with two stub packs and no other line. A mandatory `resolution:` line would stop that test compiling. And `macro_rules!` may not follow a `ty` or `path` fragment with `]`. | `sdk/rust/src/installed.rs:36–41`; the Rust reference, "Follow-set ambiguity restrictions" | The line is optional, and lists resolver types each followed by a comma (SD-R8, **QR-6**). |
| **F-R11** | **`Arrived::new` is public, and the reduction does not re-resolve.** A stating system that encodes `Arrived::new` itself bypasses `arrival()`; presence's reduction re-asks only `admit`. Presence's own test uses exactly that bypass. | `systems/presence/src/event.rs:42–46, 121–134`; `system.rs:121–160`; `systems/presence/tests/presence.rs:903–911` | Re-resolving at reduction would re-run the resolvers on the displaced people's arrivals against a state the walker's arrival has already changed. 12a keeps `ARC-26`'s rule — the contract's way is the constructor — and ARC-39 states the bypass as a limitation; 12b's guard catches its physical consequence. |
| **F-R12** | **Presence's declaration must not claim `stopped-short`.** The kernel records a fact only if its emitter declares the type, and genesis records a fact as its owner's. Presence never states `stopped-short`: not at genesis (`arrival()` refuses a changed placement) and not in `react`. | `kernel/src/dispatch.rs:495–527, 662–705`; `kernel/src/system.rs:161–175` ("the declaration … must be true") | Only stating systems declare it (movement, SD-R9). §4.4.7's "its declaration (it emits `stride-blocked`)" is corrected (**QR-3**). Presence's record in a save then differs only in its version. |
| **F-R13** | **Nothing scans 12a's lines for market words, and 12a adds none.** The I-2 scan's rows are 11a, 11b and 11c, each merged, so each reads `base..M^2`. AC-1's check 1 reads the two transformation merges; check 2 reads crate names and dependency edges; check 3 reads the two worlds. 12a names no market crate, adds no dependency path from a framework crate to a market pack (presence still depends on contracts, kernel, sdk and serde; the sdk names no pack), and edits no world. | `tests/acceptance/tests/precursor_vocabulary.rs:63–82, 350–410`; `ac1_composability.rs` (SD-29 … SD-32) | Both tests are run in RS-C8 to confirm (RS-15); no row, no allow-list entry. |
| **F-R14** | **Process-wide state is a named exception, not a hidden one.** `ENGINEERING_RULES.md` §15: "Do not introduce hidden global state". | `docs/ENGINEERING_RULES.md` §15 | ARC-39 names the catalog as the one process-wide value, says why it is not world state, and lists its three entry points. |

## 16.3 Design (SD-R1 … SD-R14)

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-R1** | **Names.** §4.4.1's `Stride` is **`Arriving`**: `{ person: PersonId, from: Option<Location>, to: Location }`, built only by presence (`pub(crate)` constructor; `person()`, `from()`, `to()` accessors). §4.4.3's `stride-blocked` is **`stopped-short`**, type **`StoppedShort { person: PersonId, wanted: Location, reached: Location, by: Option<EntityId> }`**, `Event::OWNER` presence, schema 1. `ArrivalResolver`, `Resolution`, `arrivals()` keep §4.4's names. | F-R1. "Stopped short" says what is true of the person without naming who moved them or why. (**QR-1**) |
| **SD-R2** | **`Resolution`** has private fields `reached: Location`, `displaced: Vec<(PersonId, Location)>`, `stopped_by: Option<EntityId>`, derives `Debug, Clone, PartialEq, Eq`, and offers `unchanged(&Arriving)`, `stopped_at(self, reached, by: Option<EntityId>) -> Self`, `displacing(self, person, to) -> Self` and the three accessors. A resolver can only start from what it is handed and change it through these. | §4.4.1's shape, made constructible by a resolver without exposing fields. |
| **SD-R3** | **`ArrivalResolver: Send + Sync`**, two methods: `resolver_of(&self) -> SystemId` and `resolve(&self, world: &WorldRead<'_>, arriving: &Arriving, so_far: Resolution) -> Resolution`. Its doc carries the contract: read only the `WorldRead`; keep nothing; no clock, no random source, no process-wide state; return `so_far` unchanged when the world holds none of the pack's state about the people and place involved; the pack's `System::install` calls `require_registered(&Self::ID)` first (SD-R6). | §4.4.1, §4.4.6, I-13. `Send + Sync` because the catalog is a `static`. |
| **SD-R4** | **The catalog** (`resolve.rs`): `static CATALOG: OnceLock<Vec<Box<dyn ArrivalResolver>>>`. `register_resolvers(Vec<Box<dyn ArrivalResolver>>)`: sorts by `resolver_of()`; a list naming one id twice panics, naming it; if unset, sets it; if set to the same ids, does nothing; otherwise panics naming both lists. `registered_resolvers() -> Option<Vec<SystemId>>`: `None` until registered. Nothing else reads or writes it. | QB-15 bounds 1 and "a double registration with a different set fails". Sorting at registration makes the order a function of the ids alone (§4.4.5), whatever order a list is written in. A panic, because one build has one catalog and a second, different one is a defect of the host, not a condition to recover from (**QR-5**). |
| **SD-R5** | **Never registered means no resolver.** `arrivals()` and `arrival()` consult the registered list, or none when there is none. A process that never registers — every test that composes a world by hand — therefore records exactly what it records today. | F-R9: about twenty test files build worlds without `worldpack`, and must pass unchanged. |
| **SD-R6** | **How a host that never registers fails loudly** (QB-15 bound 2). Three guards, none in the kernel: (1) `worldpack::compose` — the one assembly path of every host (F-R9) — registers `Capability::resolvers()` before installing anything, so a host cannot compose without registering; (2) **a resolver's pack refuses to join a world while its resolver is not registered**: `require_registered(&SystemId)` panics, naming the pack, when the catalog is unset or does not list it, and the trait requires the pack's `install` to call it first, so a host that installs a resolver pack and never registered dies at assembly, before any arrival; (3) 12b's bodies reaction guard (§4.4.4) catches the physical consequence of any arrival that escaped resolution. A host that never registers and installs **no** resolver pack runs exactly as before: there is nothing to resolve, so nothing runs "silently without resolvers". | The kernel's `install` hook can refuse only with a `KernelError`, and no variant says "a resolver was not registered"; adding one is a kernel change. The panic is at assembly and names the pack, `register_resolvers` and ARC-39. (**QR-4, operator-material**: this is how the operator's bound is read.) |
| **SD-R7** | **`arrivals(world, person, to) -> Result<Vec<Emission>, Rejection>`.** (1) `admit(person, to)`; (2) `from` = the person's `Presence`, if any; (3) fold the registered resolvers in ascending `SystemId` from `Resolution::unchanged`, **checking after each one** (SD-R10) and naming the resolver on a refusal; (4) emissions, in this order: `arrived { person, reached }`; `arrived { other, location }` for each displaced, in the resolution's order; `stopped-short { person, wanted: to, reached, by }` when `reached ≠ to`. Each `arrived` is built as `arrival()` builds it today (visibility, subject, participants, place); `stopped-short` is heard in `reached`'s place, about the person. **`arrival()`** keeps its signature, runs (1)–(3), and refuses with SD-R10's code when the result is not `unchanged`; otherwise it returns exactly today's emission. | §4.4.2. Checking each step localizes a defective resolver (`ARC-23`); a final-only check could not name it. |
| **SD-R8** | **`installed!`'s optional `resolution:` line**, after `perception:`: `resolution: <trait path> => [ <type>, … ];` with each type followed by a comma (`[]` when there is none; `[mineworld_bodies::BodiesSystem,]` in 12b). When present the macro adds `pub fn resolvers() -> Vec<Box<dyn <trait>>>` to `impl Capability`, one `Default` value per listed type; the compiler refuses a listed type that does not implement the trait. When absent, it adds nothing, so `twins` compiles unchanged. A new test, `systems/installed/tests/resolution.rs`, holds that every `resolvers()` id is an `AVAILABLE` id and no id repeats, with a stub installed set that lists a resolver not installed as its negative control. | F-R10. Types, not §4.4.4's variants: mapping a variant to its type inside `macro_rules!` needs a generated helper macro, the "clever metaprogramming" `ENGINEERING_STANDARDS.md` warns against. The guard test covers what the compiler then cannot (**QR-6**). |
| **SD-R9** | **Movement's edit** (`systems/movement/src/system.rs`): `resolve` calls `arrivals(&read, person, requested.to())` instead of `arrival(..)`, keeps its `map_err` to `FactRefusedByOwner`, and returns the `Vec`; `declaration()` gains `.emitting::<StoppedShort>()`. The `use` line names `arrivals` and `StoppedShort` instead of `arrival`. Its `VERSION` stays 1. It names no resolver, no catalog and no resolver pack. | §4.4.2's "two mechanical lines". A save's movement record changes in its declaration and is refused by presence's version first (SD-R13), so no movement version is needed to refuse an old save. |
| **SD-R10** | **What presence checks of a resolution, after each resolver** (refines §4.4.6). Refused with `Rejection::System { code: "resolution-refused", detail: "<resolver id>: <rule>" }`, which the stating system turns into `FactRefusedByOwner` as today: (a) `reached.place == to.place`; (b) `reached.facing == to.facing`, and `reached` has a local position exactly when `to` does; (c) when `from` is in `to`'s place and both have local positions, `|reached − from|² ≤ |to − from|²`, in `i128` millimetres; (d) `admit(person, reached)`; (e) every displaced entry: a living `Person`, not the walker, listed once, whose current `Presence` is in `to`'s place, displaced to a location in that place that keeps the local-position rule of (b) and passes `admit`; (f) `stopped_by`, when present, names an entity of this world, and is present only when `reached ≠ to`. | §4.4.6 plus (b) and (f): a resolver may shorten or bend an arrival and move others within the place, never invent or drop a position, turn a person, or claim a cause that did not stop them. Integers only (presence's no-float test). |
| **SD-R11** | **The synthetic packs** (`tests/acceptance/tests/resolvers/mod.rs`, compiled into no library, never in `systems/installed`). Each owns one component on a `Place`, written by reducing its own genesis fact, so it is inert wherever that state is absent (I-13): **`test-fences`** — `Fence { x }`. For an arrival into a fenced place with a local position, from west of the line (`from` in the same place with `x < X`, or `from` absent or elsewhere) to `to.x ≥ X`: `reached = (X − 10, to.y, to.z)`, `stopped_by = None`. Then every other person in that place, in `EntityId` order, whose position is within 300 mm of `reached` on both axes, is displaced by +100 mm in x. **`test-grid`** — `Grid { step }`. When `from` is in the same place and `reached.x > from.x`, `reached.x` becomes the largest multiple of `step` that is `≤ reached.x` and `≥ from.x` (unchanged when there is none). **`test-rogue`** — `Rogue { mode, elsewhere }`: one deliberate violation of SD-R10 per mode. **`test-echo`** (not a resolver; SC-8): depends on presence; declares emitting `arrived`, `stopped-short` and its own `heard { person }`, and subscribing to `arrived`; reacts to every `arrived` with `heard`, and to its configured trigger's `arrived` by stating `arrivals()` for its configured follower 600 mm east. **`test-placer`** (not a resolver; SC-5): provides `test-place { person, to }` and states `arrivals()`. Fences, grid and rogue call `require_registered` in `install`. | §4.4.9. A component rather than configuration in the value (11c's `chimes` kept its belfry in the value), because a resolver value lives in the process-wide catalog, shared by every world of the process, while a fence belongs to one world. Grid exists only so two resolvers compose in an order that shows (RS-5). |
| **SD-R12** | **Which test runs where.** The catalog is per process, and Cargo runs each `tests/*.rs` file as its own process. So: `systems/presence/tests/resolver_catalog.rs` — one `#[test]` that walks the registration rules in sequence (RS-8); `tests/acceptance/tests/arrival_resolvers.rs` — every test first registers `[grid, rogue, fences]` (a no-op after the first), and worlds install only what each test needs; `arrival_resolvers_unregistered.rs` — never registers (RS-9); `arrival_resolvers_resume.rs` — `harness = false`, every child registers `[fences]` before composing (RS-11); `worldpack/tests/registration.rs` — composes, then tries a different list (RS-10). No file that registers a synthetic list calls `worldpack`. | A test that registers its own list and then composes through `worldpack` would hit SD-R4's panic by design. Separating them is what lets one rule hold for hosts and tests alike. |
| **SD-R13** | **What changes in a save** (F-R5). For the same world, seed and inputs, a save written after 12a differs from one written before only here: in the manifest's `composition`, in every snapshot row's `composition`, and in revision 1's journal row (`before.composition`) — presence's `version` 2 → 3, and movement's `emits` and `emits_owned_by_others` gaining `stopped-short` (owner presence). Facts, every other journal row, every component row, relation, schedule and counter are byte-identical. A save written before 12a is refused at the first differing record: presence, by version — `PersistedSystemOutdated`, "system 'presence' is v3 here, but the save was written by the older v2" — before any snapshot is restored. A save written after 12a resumed by an older build is refused as `PersistedSystemTooNew`. | `ARC-25`: versions are refused, never guessed. Presence's version rises because its constructors now answer through resolvers and its vocabulary gained a fact (§4.4.7); its declaration does not change (F-R12). |
| **SD-R14** | **Documents.** ARC-39 (§15.3's draft, made exact by SD-R1 … SD-R13): the seam; the catalog as the one named process-wide value (F-R14); the three guards of SD-R6; **"Runtime `World::disable` of a resolver pack is not honoured: a disabled pack's resolver is still asked, and its state still answers, until a kernel read of enabled state is decided (step-11 QB-17)"**; how tests that compose by hand behave (SD-R5, SD-R12); accepted limitations: the constructor bypass (F-R11), genesis (F-R4), no length bound on an arrival from another place or from nowhere beyond the place rule (**QR-12**). The bodies bounds of §15.3 move to 12b with DEP-13 (**QR-11**). MODULE_SPEC §3.1: the `resolution:` line and a resolver pack's install duty. `systems/README.md`: one paragraph. Presence's `lib.rs` table and README: `stopped-short`, `arrivals`, the seam. | `CLAUDE.md` §2.2: the documents first (RS-C1). |

## 16.4 Acceptance (decided before measuring, `ARC-23`)

Each guarded criterion names the mutation shown to break it. A mutation is applied in the working tree,
observed to fail by name, and reverted; `git status` and `git grep MUTATION` are recorded afterwards.
Every number in a test is a literal from the test's own layout, never computed by the code under test
(test rules §25).

```text
RS-1  Nothing existing moves (I-7, SC-1). On the PR head, with the real installed set (no resolver):
        - `mineworld run worlds/social-cafe --headless --seed 7 --days 300`: faults 0, 365 330 facts,
          sha-256 of every line but `wall` = ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b
          (E-RS0);
        - `mineworld run worlds/market-town --headless --seed 7 --days 300`: faults 0, 372 755 facts,
          sha-256 of every line but `wall` = 365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d
          (E-RS0);
        - `mineworld validate` of both worlds byte-identical to the base's output;
        - every existing test passes; the only edits to existing tests are QR-2's three literals,
          `presence v2` → `presence v3`, each claim unchanged (inspect reports the save's composition).
      M-RS1  arrivals() states `stopped-short` even when reached = to → the social-cafe sha differs
             from E-RS0, and `facts      stopped-short` appears in the summary (the instrument sees a
             seam that records one extra fact per move).
RS-2  What a save holds, and the refusal (SD-R13). Real evidence across two builds, recorded, not a
      committed test (QR-9): the base binary (built from the PR's base before any edit) writes
      `run worlds/social-cafe --headless --seed 7 --days 2 --save B`; the PR binary writes the same into
      `--save N`.
        - `inspect B` and `inspect N`: the same head revision, journal kinds and fact counts; the
          `systems` lines differ only in `presence v2` / `presence v3`;
        - the PR binary on B: `run … --days 3 --save B` and `server worlds/social-cafe --save B` each exit
          non-zero, naming "system 'presence' is v3 here, but the save was written by the older v2";
          B is byte-identical before and after (`cmp`);
        - positive control: the PR binary resumes N to day 3 and reports resuming at N's head.
RS-3  Resolve before record (SC-2). World: presence, movement, fences; a fenced place `yard`
      (Fence x = 5 000); alice at (4 000, 2 000), bob at (5 100, 2 100), carol at (5 400, 2 000). alice
      moves to (5 500, 2 000). The request is Accepted with exactly these facts, in order:
        arrived alice (4 990, 2 000) · arrived bob (5 200, 2 100) ·
        stopped-short { alice, wanted (5 500, 2 000), reached (4 990, 2 000), by None }
      each `Causation::Action(alice's ActionId)`, provenance emitted_by `movement`, controller decision
      alice's ActionId (AC-9); carol, 410 mm from the stop, is not displaced. Presence afterwards: alice
      (4 990, 2 000), bob (5 200, 2 100), carol unchanged. No `arrived` in the whole log names alice at
      (5 500, 2 000): the log holds only true arrivals.
      M-RS2  arrivals() folds over no resolver → fails: alice recorded at (5 500, 2 000).
      M-RS3  arrivals() drops the displaced emissions → fails, naming bob's missing arrival.
RS-4  Presence still decides (SC-3). In a place whose Rogue mode makes one violation each — lengthens
      the arrival, ends it in another place, turns the walker, drops the local position, displaces the
      walker, displaces one person twice, displaces a person who is in another place, displaces a place
      entity, displaces a destroyed person, names a `stopped_by` that does not exist, names a
      `stopped_by` while reaching `to` — the dispatch returns `Err(FactRefusedByOwner { system: presence,
      event_type: arrived, reason: System { code: resolution-refused, detail } })` whose detail names
      `test-rogue` and the rule; no fact is recorded and presence's owned state is byte-identical before
      and after (presence's `owned_state` pattern). A rogue resolution that obeys every rule is recorded
      (positive control).
      M-RS4  remove check (c) → the "lengthens" row fails.
RS-5  Order (SC-4). fences and grid both installed, registered as [grid, rogue, fences]; `yard` has
      Fence 5 000 and Grid 1 000; alice at (3 600, 2 000) moves to (5 500, 2 000). Ascending SystemId
      asks fences, then grid: recorded at (4 000, 2 000), with stopped-short { wanted (5 500, 2 000),
      reached (4 000, 2 000) }. (Grid then fences would give (4 990, 2 000).) registered_resolvers() is
      [test-fences, test-grid, test-rogue].
      M-RS5  the catalog keeps registration order → fails: (4 990, 2 000).
RS-6  Placement (SC-5). In `yard` (Fence 5 000), for dan who has no Presence:
        - arrival(dan, (5 500, 1 000)) → Err(System { code: resolution-refused }) naming test-fences;
        - test-placer stating arrivals() for the same → Accepted: arrived dan (4 990, 1 000),
          stopped-short { dan, wanted (5 500, 1 000), reached (4 990, 1 000), by None };
        - arrival(dan, (4 000, 1 000)) → Ok, its payload bytes equal `Arrived::new(dan, (4 000, 1 000))`'s
          encoding (positive control).
      M-RS6  arrival() skips the "would change" refusal → the first bullet fails.
RS-7  Inert where absent (I-13). In the registered process: a world with presence and movement only,
      and a world with fences installed whose places have no Fence, each run the same 12-move script:
      every accepted move records exactly `arrived` at the requested location (payload bytes equal to
      `Arrived::new`'s encoding), plus `person-entered-place` on a crossing — never `stopped-short`.
      The same script in the never-registered process (RS-9's file) gives the same per-move facts.
RS-8  Registration rules (QB-15 bounds 1, 3; one process, one #[test], in sequence):
      registered_resolvers() is None, and arrivals() returns arrival()'s one emission; register [b, a] →
      Some([a, b]); register [a, b] → no-op; register [a] → panics naming both lists; register [c, c] →
      panics naming `c` twice; afterwards still Some([a, b]).
      M-RS7  a different list is silently ignored → the [a] step fails (no panic).
RS-9  A host that never registers fails loudly (QB-15 bound 2; never-registered process): installing
      test-fences into a world panics with a message naming `test-fences`, `register_resolvers` and
      ARC-39; a world without any resolver pack runs RS-7's script unchanged.
      M-RS8  remove fences' require_registered call → the panic test fails (fences installs, and a
             crossing would be recorded unresolved).
RS-10 compose registers the installed set (QB-15 bound 1): in a fresh process,
      `WorldPack::read(worlds/social-cafe)` then `compose()` → registered_resolvers() = Some([]); a
      second compose → no-op; then register_resolvers([a stub]) → panics naming both lists (a double
      registration with a different set fails). installed's guard (SD-R8) passes on the real set and
      fails, naming the type, on a stub set listing a resolver that is not installed.
      M-RS9  remove compose's registration call → fails: None.
RS-11 SIGKILL and resume, with a resolver installed (SC-6; harness = false). World: presence, movement,
      fences, four people, `yard` with Fence 5 000; a deterministic script of moves (a fixed mix of
      request index and seat, as persistence's kill_and_resume). Activity first: the control run's
      counts of stopped-short and of displaced arrivals are located and each > 0, before any comparison.
      Then for kill points early, middle and late: the killed child exited by SIGKILL short of the end;
      a new child resumed from the head on disk with a tail > 0 at least once; facts, journal and
      snapshots byte-identical to the control's; verify() from genesis passes.
      M-RS10 fences adds a process-global counter's parity to its stop distance (an impure resolver) →
      fails: the resumed save diverges (ReplayDiverged or differing rows).
RS-12 Subscribe-and-emit (SC-8, §8.2's kernel-adjacent fact). test-echo installs (it depends on
      presence, emits presence's `arrived`, subscribes to `arrived`); alice's move records, in order:
      arrived alice (Action) · heard alice and arrived bob (Event: alice's arrived) · heard bob (Event:
      bob's arrived — echo heard the arrival it stated) — and nothing more (the cascade ends at depth 3).
RS-13 Names (SC-7, the seam names no physics). `tests/acceptance/tests/seam_vocabulary.rs`, over the
      current text of systems/presence/src, systems/movement/src, sdk/rust/src, systems/installed/src,
      worldpack/src and every file this PR adds under */tests/ but the scan's own file, which must name
      its vocabulary — all non-Markdown, comments included,
      split into words as the I-2 scan splits them: no word beginning `body`, `bodies`, `bodily`,
      `physic`, `rapier`, `nudg`, `collision`, `collid`, `capsule`, `jolt`; and movement's src names none
      of `ArrivalResolver`, `Resolution`, `register_resolvers`, `require_registered`, `test-fences`,
      `test-grid`, `test-rogue`. A listed directory that is missing fails the test. Presence's and
      movement's existing structural tests pass unchanged (F-R1).
      Planted: `// a capsule nudges` in presence's resolve.rs → fails naming the file, line and both
      words; `use mineworld_presence::ArrivalResolver as _;` in movement → fails naming it. Removed.
RS-14 Runtime disable is not honoured (QB-15 bound 3, QB-17), shown rather than assumed: in RS-3's
      world, `world.disable(test-fences)` and the same move → still recorded at (4 990, 2 000). ARC-39
      states this limitation in its own words.
RS-15 The documents say it first, and nothing else bites: ARC-39 and MODULE_SPEC §3.1 committed
      before any code (RS-C1); both doc checks pass; `cargo test -p mineworld-acceptance --test
      ac1_composability` and `--test precursor_vocabulary` pass unchanged (F-R13).
RS-16 Scope (I-1). `git diff --name-only <base>...HEAD` ⊆ §16.1's change set; kernel/, contracts/,
      persistence/, server/, cognition/, clients/, worlds/, authoring/, tools/cli/src/ and the root
      Cargo.toml have an empty diff; Cargo.lock changes only mineworld-acceptance's dependency list;
      `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean.
```

## 16.5 Commit plan

Each commit tracks implementation, validation and review separately. Evidence goes into §16.10 as
`E-RS<n>`; deviations into §16.11. A planned commit may become several coherent commits; the mapping is
recorded. Each commit leaves the workspace's tests green.

### RS-C0 — Design (this section) — docs only

- [x] Implementation: §16 and the header line, by the planning session on `mvp0/s15-12a-plan`, from the
  audit in §16.2.
- [x] Validation: `python3 scripts/check_doc_headings.py`, `python3 scripts/check_decision_ids.py`
  (E-RS0); the two 300-day baselines re-measured on this base (E-RS0).
- [x] Review: every claim in §16.2 cites a file and line, a command, or a measurement; each departure
  from §4.4 is named with its finding and question; the operator-material questions are marked (§16.8).
  Self-review by the planning session only; the primary session's review is pending.

### RS-C1 — Specs before code: ARC-39, MODULE_SPEC §3.1, systems/README

**Goal.** The seam exists as a reviewable decision before any code relies on it (`CLAUDE.md` §2.2).

**Scope.** `docs/DECISIONS.md`: **ARC-39** appended after ARC-38 — *An arrival is resolved before it is
recorded* — from §15.3 made exact by SD-R1 … SD-R14 (header: Date, Approved by (the operator's QB-1,
QB-15; this PR's freeze), Implements `ARCHITECTURE.md` §§7–8, Relates to `INV-7`, `INV-15`, `AC-2`,
`AC-9`, `ARC-15`, `ARC-23`, `ARC-25`, `ARC-26`, `ARC-33`, Design step-11 §§4.4, 16). `docs/MODULE_SPEC.md`
§3.1: after the installed-set block, one paragraph and the line form of SD-R8, and a resolver pack's two
duties (inert where its state is absent; `require_registered` in `install`). `systems/README.md` "Adding a
pack": one paragraph pointing at §3.1 and ARC-39. Before writing: `git fetch` and confirm ARC-39 is free
on every `origin/*` branch (it was on 2026-10-07, §16.10).

**Depends on:** freeze. **Non-goals:** no code; no DEP-13 (12b); no bodies bound in ARC-39 (QR-11).

- [x] Implementation: ARC-39 appended after ARC-38 (`docs/DECISIONS.md`); MODULE_SPEC §3.1 "A pack
  that resolves arrivals" (the `resolution:` line, three lines to install a resolver pack, its two
  duties); `systems/README.md` "Adding a pack": one paragraph. ARC-39 confirmed free on every
  `origin/*` ref after `git fetch` (E-RS1).
- [x] Validation: both doc checks pass (E-RS1); cited sections exist: ARCHITECTURE §7 (event
  sourcing) and §8 (persistence), ENGINEERING_RULES §15 ("Do not introduce hidden global state"),
  MODULE_SPEC §3.1.
- [x] Review: ARC-39 has problem / options (a)–(e) / choice items 1–8 / accepted limitations; the
  runtime-disable limitation has its own paragraph in its own words; "Tests that compose worlds by
  hand" is its own paragraph; item 5 names the catalog as the one process-wide value, its three entry
  points and why it is not world state (F-R14). The diff, scanned for RS-13's vocabulary
  (`git diff -U0 docs/ systems/README.md | grep -i 'bod|physic|rapier|nudg|collision|collid|capsule|jolt'`),
  matches only the step's name and file name ("Step S15 (bodies and physical interaction)",
  `step-11-bodies.md`) and the word `nobody`; one "physical consequence" in the limitations was
  reworded before commit. No defined term redefined: `Person`, `Place`, `Event`, `System`, `World
  Pack` are used in their CORE_CONCEPTS sense; `Arriving`, `Resolution`, `stopped-short` are new
  names of presence's, not synonyms of a defined term.

**Commit boundary.** Documentation only.

### RS-C2 — presence: the seam

**Goal.** RS-3 … RS-8's mechanism, inside presence, inert until something registers.

**Scope.**
- `systems/presence/src/resolve.rs` (new): `ArrivalResolver`, `Arriving`, `Resolution` (SD-R1 … SD-R3);
  the catalog, `register_resolvers`, `registered_resolvers`, `require_registered` (SD-R4, SD-R6); the
  per-step checks (SD-R10) as one function the two constructors share.
- `systems/presence/src/event.rs`: `StoppedShort` (+ `Event` impl, accessors, a crate-private emission
  builder like `entered_place`); `arrivals()`; `arrival()` routed through the same fold and refusing a
  changed resolution (SD-R7). Docs say what each constructor is for.
- `systems/presence/src/system.rs`: `VERSION` 3, its doc (why 3: the constructors answer through
  resolvers; a new fact type; old saves refused by name). Declaration unchanged (F-R12).
- `systems/presence/src/lib.rs`: `pub mod resolve;`; re-exports; the doc table gains `stopped-short`
  (stated by a stating system through `arrivals`, never reduced) and the seam paragraph.
- `systems/presence/README.md`: the table; one paragraph on the seam.
- `systems/presence/tests/resolver_catalog.rs` (new): RS-8 as one `#[test]`, with two stub resolvers
  `a`, `b` and `c` (identity resolvers) defined in the file.
- `tools/cli/tests/inspect.rs:36`, `tools/cli/tests/social_composition.rs:380, 420`: `presence v2` →
  `presence v3` (QR-2), in this commit because the version bump is what breaks them.

**Depends on:** RS-C1. **Non-goals:** no caller changes yet; nothing registers.

- [x] Implementation: as scoped. `resolve.rs`: `Arriving` (crate-private `new`), `Resolution`
  (private fields; `unchanged`, `stopped_at`, `displacing`, three accessors), `ArrivalResolver`,
  `CATALOG: OnceLock`, `register_resolvers` / `registered_resolvers` / `require_registered`, and the
  crate-private fold `resolve()` with `check()` / `check_displaced()` (SD-R10, after each resolver,
  returning the first resolver that changed the arrival). `event.rs`: `StoppedShort` + `Event` impl +
  accessors + crate-private `stopped_short` builder; `arrivals()`; `arrival()` through the same fold,
  refusing `resolution-refused` naming the first resolver that changed a placement; one private
  `arrived()` builder used by both. `system.rs`: VERSION 3 and its doc; declaration unchanged.
  `lib.rs` and README: the table row `stopped-short`, the "Asking before recording" paragraph. The
  three QR-2 literals. Presence's prose avoids F-R1's substrings (its own scan passes). Deviations
  DR-1, DR-2 (§16.11).
- [x] Validation (E-RS2): `cargo test -p mineworld-presence -p mineworld-movement` → presence 15 + 1,
  movement 3 + 7 + 1, all pass (both structural scans unchanged and green); `cargo clippy -p
  mineworld-presence --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean after `cargo
  fmt`; `cargo test -p mineworld-cli --test inspect --test social_composition` → 3 + 4 pass; M-RS7
  applied, failed by name, reverted.
- [x] Review: no float, no `stride`, no other pack's word in presence's src (the scan says so, and so
  does movement's); `arrival()` builds its emission with the same private `arrived()` builder
  `arrivals()` uses, after a fold that with no catalog runs no resolver — so its bytes are today's, and
  the RS-8 test asserts `arrivals == vec![arrival]` unregistered; the fold iterates the catalog as
  stored (sorted once in `register_resolvers`); every SD-R10 rule is a distinct early return with its
  own message, each driven by RS-4's table in RS-C5; the static `CATALOG` is written only by
  `register_resolvers` and read by `registered_resolvers`, `require_registered` and the crate-private
  fold that the two constructors share.

### RS-C3 — the installed set carries resolvers; compose registers them

**Goal.** RS-10: every host registers the build's resolvers by composing.

**Scope.**
- `sdk/rust/src/installed.rs`: the optional `resolution:` line and `Capability::resolvers()` (SD-R8);
  the macro's doc block shows it. `sdk/rust/src/lib.rs`: one line of the doc table.
- `systems/installed/src/lib.rs`: `resolution: mineworld_presence::ArrivalResolver => [];` and a doc
  sentence. `systems/installed/tests/resolution.rs` (new): the guard and its stub negative control.
- `worldpack/src/load.rs`: `compose()` calls `mineworld_presence::register_resolvers(Capability::
  resolvers())` before installing; its doc says why (ARC-39). `worldpack/tests/registration.rs` (new):
  RS-10.

**Depends on:** RS-C2.

- [x] Implementation: as scoped. `installed!` now has two public arms — with and without
  `resolution: $trait => [ $( $Resolver:ty , )* ];` — both forwarding to one internal `@catalog` arm
  holding the former body; the resolution arm adds one `impl Capability { pub fn resolvers() }`.
  `systems/installed`: `resolution: mineworld_presence::ArrivalResolver => [];` and a doc paragraph.
  `systems/installed/tests/resolution.rs`: the guard (`faults`) on the real set, and the `stray` stub
  set (one pack `lone`, resolution line `[Stray, Stray,]`) as its negative control. `worldpack`
  `compose()`: `mineworld_presence::register_resolvers(Capability::resolvers())` first, and its doc.
  `worldpack/tests/registration.rs`: RS-10. The `twins` test is not edited.
- [x] Validation (E-RS3): `cargo test -p mineworld-sdk -p mineworld-installed-systems -p
  mineworld-worldpack` all pass, worldpack's structure test included; the CLI's inspect and
  social_composition re-run at this state, 3 + 4 pass; M-RS9 applied, failed by name, reverted; the
  stub negative control yields its three expected faults inside its own test.
- [x] Review: the sdk names no pack and no trait (the trait path comes from the invocation); both
  public arms call the same `@catalog` arm, so they cannot drift, and differ only by `resolvers()`;
  `register_resolvers` is the first statement of `compose`, before `World::new()` and every install,
  and `assemble` → `compose`, `load` → `assemble`, so both inherit it. Worldpack names
  `mineworld_presence` already (allow-listed for `location`), so no new crate name appears there.

### RS-C4 — movement states through `arrivals`

**Goal.** Movement's arrivals are resolved (SD-R9).

**Scope.** `systems/movement/src/system.rs`: the `use` line, the `resolve` call, the declaration line.
Nothing else in movement.

**Depends on:** RS-C2.

- [x] Implementation: as scoped; `git diff --stat systems/movement` → `systems/movement/src/system.rs
  | 20 +++++++++-----------`, one file: the `use` line (`StoppedShort`, `arrivals` instead of
  `arrival`), `.emitting::<StoppedShort>()`, and `resolve` returning `arrivals(..)` mapped to
  `FactRefusedByOwner` as before, with its doc line.
- [x] Validation (E-RS4): `cargo test -p mineworld-movement` 3 + 7 + 1 pass, unedited; clippy clean;
  the 300-day social-cafe run at this state = E-RS0's sha.
- [x] Review: movement names `arrivals` and `StoppedShort` — presence's constructor and fact — and no
  `ArrivalResolver`, `Resolution`, catalog function or resolver pack (RS-13 checks it mechanically in
  RS-C7); the refusal is mapped exactly as before (same system, same event type, presence's reason
  passed through); `VERSION` stays 1.

### RS-C5 — the synthetic resolvers and SC-2 … SC-5, SC-8

**Goal.** RS-3 … RS-7, RS-9, RS-12, RS-14 through the real `World::dispatch`.

**Scope.**
- `tests/acceptance/Cargo.toml`: dev-dependencies `mineworld-movement`, `mineworld-persistence`
  (`workspace = true`; persistence is used by RS-C6).
- `tests/acceptance/tests/resolvers/mod.rs` (new): fences, grid, rogue, echo, placer (SD-R11); a world
  builder (`yard` and `lane`, the people of RS-3, placed by genesis through `arrival()`), a dispatcher
  that allocates ids and instants, and `owned_state`. Genesis builds every placement before it applies
  any fact, the fence's own included, so bob and carol can be placed east of the line (F-R4); RS-6's
  refusal is asked after genesis, when the fence exists.
- `tests/acceptance/tests/arrival_resolvers.rs` (new): RS-3, RS-4 (table-driven), RS-5, RS-6, RS-7 (the
  registered half), RS-12, RS-14.
- `tests/acceptance/tests/arrival_resolvers_unregistered.rs` (new): RS-9 and RS-7's never-registered
  half.
- `tests/acceptance/src/lib.rs`: the doc table.

**Depends on:** RS-C3, RS-C4.

- [x] Implementation: as scoped; every test of `arrival_resolvers.rs` calls `register_all()` first
  (SD-R12). `resolvers/mod.rs`: `Fences` (+ `Fence`, `test-fence-raised`), `Grid` (+ `GridStep`,
  `test-grid-laid`), `Rogue` (+ `RogueMode { Violation }`, `test-rogue-set`; fourteen breaching modes
  and `Obey`), `Echo` (+ `test-heard`), `Placer` (+ `test-place`); `Plan` / `Cast` / `Town` (yard and
  lane, a doorway yard (9 000, 2 000) ↔ lane (0, 2 000), genesis placements through `arrival`);
  `owned_state`; `Fact` / `fact` / `facts`; RS-7's twelve-move `SCRIPT` and `run_inert_script`.
  The acceptance crate's doc table names the four new files (the RS-C6 and RS-C7 rows written ahead).
  DR-3 (the support file's size) in §16.11.
- [x] Validation (E-RS5): 7 + 2 pass; clippy and fmt clean; M-RS2, M-RS3, M-RS4, M-RS5, M-RS6, M-RS8
  each applied, failed by name, reverted.
- [x] Review: every expected position in an assertion is a literal from the test's layout (4 990,
  5 200, 4 000, 4 500, 5 500, 5 600, …), never read from the code under test; the synthetic packs are
  defined only under `tests/acceptance/tests/`, which no library compiles, and `systems/installed`'s
  `resolution:` line is still `[]`; `git diff Cargo.lock` is exactly two added lines in
  `mineworld-acceptance`'s dependency list (`mineworld-movement`, `mineworld-persistence`). RS-4 drives
  every reachable rule of SD-R10 by its own message. Not separately reachable, by construction:
  (d) `admit(person, reached)` — `admit(person, to)` passed and rule (a) puts `reached` in `to`'s
  place, so the same person and place are asked again; and (e)'s `admit` of a displaced person, after
  the living-person and same-place checks. Both are kept as SD-R10 states them: they are the owner's
  own check, and cost nothing.

### RS-C6 — SIGKILL and resume with a resolver installed

**Goal.** RS-11.

**Scope.** `tests/acceptance/tests/arrival_resolvers_resume.rs` (new, `harness = false`, persistence's
`kill_and_resume` pattern: the same binary is the parent that kills and the child that is killed,
selected by an environment variable); `tests/acceptance/Cargo.toml`: its `[[test]]` entry, with the
reason in a comment.

**Depends on:** RS-C5.

- [x] Implementation: as scoped; every child — and the parent, before `verify` composes — registers
  `[fences]` before composing. World: presence, movement, fences; alice (3 800, 1 500), bob (4 200,
  2 500), carol (5 300, 1 800), dan (6 000, 2 200); Fence 5 000. 400 steps: seat `index % 4` asks to
  move to a point of the 3 m × 2 m box x 3 500 … 6 499, y 1 000 … 2 999 from `mix(index, 13)`.
  PowerLoss durability, a snapshot every 32 revisions. Rows are compared through the public
  `PersistenceBackend` (journal_after, facts_of, snapshot_revisions / snapshot_at), so acceptance gains
  no `rusqlite`. `[[test]] harness = false` with its reason in `tests/acceptance/Cargo.toml`.
- [x] Validation (E-RS6): the harness passes; M-RS10 applied, failed (ReplayDiverged), reverted.
- [x] Review: the four exclusions hold — victim status is SIGKILL, no "done", head short of the
  control's; the survivor reports the head it resumed from (= the file's) and replayed = head −
  snapshot; tails 16, 11, 7 (> 0 every time); the control's 123 stopped-short facts and 41 displaced
  arrivals are counted, and asserted > 0, before any comparison.

### RS-C7 — the seam names no physics

**Goal.** RS-13.

**Scope.** `tests/acceptance/tests/seam_vocabulary.rs` (new). The word splitter is the I-2 scan's rule,
written again here (about twenty lines) rather than moved, so the I-2 scan's file stays unedited.

**Depends on:** RS-C5 (the files it lists exist).

- [ ] Implementation: as scoped.
- [ ] Validation: the test passes; the two planted violations fail by name, removed.
- [ ] Review: the list of scanned paths is every code path §16.1 changes; Markdown is the only exclusion
  and is stated.

### RS-C8 — Close: real runs, the refusal, status, full gate, ledger

- [ ] RS-1: the two 300-day runs on the final executable head; both `validate` diffs; M-RS1 applied to
  one run and reverted.
- [ ] RS-2: the cross-build evidence, with the base binary built from the base before RS-C2 and kept at
  `/tmp/s15-12a/base-mineworld`.
- [ ] RS-15, RS-16: the AC-1 test and the I-2 scan; the diff-scope check; fmt and clippy.
- [ ] Full gate once on the final executable head: `cargo test --workspace --no-fail-fast` in the
  background; counts and wall time recorded.
- [ ] Documentation: `docs/MVP_STATUS.md` (a capability row for the seam, an evidence row); §16
  checkboxes; §16.10; the handoff.
- [ ] Review: RS-1 … RS-16 each with evidence; deviations listed in §16.11.

**PR 12a lifecycle:** IN PROGRESS (12a session, branch `mvp0/pr-12a-resolver-seam` from main @ 6d48e03).

## 16.6 Test ownership

```text
STATIC      cargo fmt; cargo clippy -D warnings; the compiler at the installed set (a listed resolver
            type that does not implement ArrivalResolver does not compile); Resolution's private fields
            (a resolver cannot fabricate one); Arriving's crate-private constructor
UNIT        presence: the registration rules (RS-8, one process); installed: resolvers ⊆ installed packs,
            with a stub negative control (SD-R8); worldpack: compose registers (RS-10, one process)
INTEGRATION through the real World::dispatch with synthetic resolvers: SC-2 … SC-5, inertness, SC-8, the
            QB-17 limitation (RS-3 … RS-7, RS-12, RS-14); a host that never registers (RS-9); every
            existing test, presence's and movement's structural scans included (RS-1)
STRUCTURAL  the seam's names (RS-13); scope (RS-16); the AC-1 test and the I-2 scan, unchanged (RS-15)
REAL RUN    (Gate 2's role) the two 300-day seed-7 runs against E-RS0 (RS-1); the cross-build save
            refusal (RS-2); SIGKILL and resume in real processes with a resolver installed (RS-11)
GATE 1      NOT REQUIRED: no model is involved, and no LM-facing semantics change
CI          none configured (S13); the full local gate runs once on the final executable head
```

Owned elsewhere and not repeated: a version difference refused by name (`persistence/tests/save.rs`,
`PersistedSystemOutdated` / `TooNew`); a foreign vocabulary needing a declared dependency
(`ARC-26`'s kernel tests). Each failure class above has one owner.

## 16.7 Is any of this material?

Yes, in two places, and both are raised rather than assumed:

- **QR-2.** The version bump the operator's "old saves refused by name" requires breaks three existing
  test literals. Either three literals change (recommended) or the version stays and old saves are refused
  less precisely. It conflicts with the stated binding "every existing test passes unchanged".
- **QR-4.** How "a host that never registers it must fail loudly" is delivered: never registered is read as
  "no resolver", and the loud failure is attached to installing a resolver pack without registering.
  The literal alternative breaks about twenty existing test files.

Not material, and recorded: the renames of QR-1 (forced by two merged structural tests; semantics
unchanged); QR-3's narrower declaration; QR-5 … QR-12, each bounded by §4.4 and the step's freeze.

No kernel, contract, persistence, server or cognition change is needed. F-R7 confirms the one
kernel-adjacent fact 12b depends on (a system may subscribe to a fact type it states) from source, so no
kernel question arises. F2 (QB-15) is not needed. No frozen invariant of an earlier step changes; I-7 is
met as SD-R13 states it.

## 16.8 Questions (QR-1 …)

Operator-material questions are marked **[OM]**. Each has a recommendation; the others are the primary
session's to decide at freeze.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QR-1** | Presence's sources cannot contain `stride` (F-R1). Rename §4.4's `Stride` to `Arriving` and `stride-blocked` / `StrideBlocked` to `stopped-short` / `StoppedShort`, or edit the two structural tests that forbid the word? | **Rename.** The tests hold `ARC-26` (presence never names who moves a person), and "stride" is movement's word (`MAX_STRIDE`). The meaning is unchanged. §§4.4, 4.7, 10.3 and the 12b–12e rows read the new names from here on; the planning session updates their wording at freeze. |
| **QR-2 [OM]** | Presence's `VERSION` 2 → 3 fails three existing test literals, `presence v2` in `tools/cli/tests/inspect.rs:36` and `social_composition.rs:380, 420` (F-R2). (a) Bump, and change the three literals; (b) do not bump: a save from before 12a is then refused at movement's position as `CompositionDiffers` (movement's declaration changed), not by presence's version, and a save of a world without movement resumes — correctly, since nothing changed for it. | **(a).** It is what "old saves are refused by name, because presence's VERSION changes" asks, and the literals' claim — `inspect` reports the save's composition — is unchanged. Listed as the only existing-test edits (RS-1), in the commit that bumps the version (RS-C2). |
| **QR-3** | §4.4.7 has presence's declaration emit `stride-blocked`. Presence never states one (F-R12), and a declaration must be true. | **Presence's declaration does not change.** Only stating systems declare `stopped-short` (movement now; bodies' `shove` in 12c). Presence's record in a save then differs only in its version (SD-R13). |
| **QR-4 [OM]** | QB-15 bound 2, "a host that never registers it must fail loudly". Literally failing in presence's constructors when nothing registered would fail about twenty existing test files that compose worlds by hand (F-R9). | **SD-R5 and SD-R6:** never registered means no resolver; every host registers by composing; a resolver's pack refuses, loudly, to be installed while its resolver is not registered (a panic at assembly naming the pack, `register_resolvers` and ARC-39), shown by RS-9 with its mutation; bodies' guard (12b) catches anything else. A host that installs no resolver pack and never registers runs exactly as before, because there is nothing to resolve. |
| **QR-5** | A second, different registration and the install guard: panic, or an error value? | **Panic.** One build has one catalog: a second, different one, or a resolver pack in a world whose host never registered, is a defect of the host found at assembly. An error value would need a new `KernelError` variant (a kernel change) or a new `PackError` variant threaded through every `compose` caller for a condition no caller can recover from. |
| **QR-6** | `installed!`'s `resolution:` line lists **types** (`[T,]`, each followed by a comma), not §4.4.4's variants, and is optional (F-R10). | **Accept.** The compiler checks a listed type implements the trait; a guard test checks it is an installed pack (SD-R8). Variants would need a generated helper macro. Optional, so the `twins` test compiles unchanged. In 12b, installing bodies is then three lines in `systems/installed`, not two: MODULE_SPEC §3.1 says so. |
| **QR-7** | At genesis a resolver sees nobody (F-R4), so `arrival()` cannot refuse two authored people who overlap, as §4.4.2 says it does. | **Amend §4.4.2's sentence; 12b's validator owns it** (§10.4 already lists "two authored people … overlapping at genesis"). Not a defect of the seam: genesis facts are built before any is applied, by design (`ARC-15`, `load.rs`). |
| **QR-8** | Presence's checks go beyond §4.4.6: facing and position-ness kept, `stopped_by` validated, a check after each resolver naming it, one refusal code `resolution-refused` (SD-R10). | **Accept** — each narrows what a resolver may do and none widens it; per-step checks localize a defect (`ARC-23`). |
| **QR-9** | Prove the refusal of a save from before 12a by real cross-build evidence (RS-2) rather than a committed test? | **Yes.** The generic refusal is owned by `persistence/tests/save.rs`; what 12a adds is a one-time fact — this version changed — best shown on a real old save, written by the real base binary. A committed test would forge the old save by editing a row. |
| **QR-10** | §10.1 I-1 names "presence, the sdk's `installed!`, `systems/installed`, `worldpack`'s compose, and movement's two lines" as 12a's only edits. Are `tests/acceptance` (new files, two dev-dependencies, one `[[test]]`), `docs/`, the new test files in presence, installed and worldpack, and QR-2's literals within it? | **Yes**: I-1 bounds what code may change behaviour; tests and documents are how the change is shown, with 11c's and 11f's precedent. §16.1 lists every path, and RS-16 checks the diff against it. |
| **QR-11** | §15.3's ARC-39 draft states bodies' nudge bounds. In 12a, before bodies exists? | **No.** ARC-39 in 12a states the seam and names the S15 step it serves; the bounds land with bodies in 12b, as DEP-13 and a note on ARC-39. The seam then names no physics in code, and its record names none it does not need (RS-13's spirit). |
| **QR-12** | SD-R10 (c) bounds a resolver's result only when the walker was already in the destination place. An arrival from another place, or a placement from nowhere, is bounded only by "same place". Add a bound? | **Not in 12a.** No seam-level number exists without geometry; bodies' bounds (§5, 12b) apply to its own resolutions, and ARC-39 states the limitation. |

## 16.9 Proposed execution contract for PR 12a (confirmed at freeze)

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12a — the arrival-resolver seam (S15, first of five; a framework
                    precursor that names no physics)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §16; evidence in §16.10 (E-RS<n>);
                    deviations in §16.11
RELATED / BINDING   this file: the header's freeze record (QB-1, QB-10, QB-15 and its three bounds),
                    §§4.3–4.4, 4.6–4.7, 8.2, 10.1 (I-1, I-2, I-6, I-7, I-13), 11.1, 15.3; overall.md §3
                    (S15), §7; DECISIONS ARC-15, ARC-23, ARC-25, ARC-26, ARC-33, ARC-35 and its notes,
                    DEP-12; MODULE_SPEC §3.1; ENGINEERING_RULES §15; CLAUDE.md §§2–4
IMPLEMENTATION BASE the main named at freeze (main @ b8afd4f + this planning branch once merged); branch
                    mvp0/pr-12a-resolver-seam; worktree /Users/yuema137/mineworld-worktrees/s15-12a
                    (proposed), held by the implementing session only
APPROVED SCOPE      §16.1's change set; RS-C1 … RS-C8; SD-R1 … SD-R14 as answered by QR-1 … QR-12
FROZEN INVARIANTS   I-1 as QR-10 reads it: no edit under kernel/, contracts/, persistence/, server/,
                    cognition/, clients/, worlds/, authoring/, tools/cli/src/, nor the root Cargo.toml;
                    no System Pack but presence and movement; movement: exactly SD-R9.
                    I-7 / RS-1: social-cafe sha ad49c723…c64b and market-town sha 365b50e0…1d1d over the
                    300-day seed-7 runs (365 330 and 372 755 facts), faults 0; no digest re-baselined.
                    Existing tests unchanged except QR-2's three literals. Presence's declaration
                    unchanged (QR-3); VERSION 3 (QR-2).
                    QB-15: the catalog written once, from the build's installed set by compose; never
                    registered = no resolver; a resolver pack refuses to install unregistered; a
                    different second registration panics; runtime disable not honoured, said in ARC-39.
                    The seam names no physics (RS-13); the synthetic packs live in test files only.
                    No market word or crate added (F-R13).
SEQUENCE            RS-C1 → RS-C2 → RS-C3 → RS-C4 → RS-C5 → RS-C6 → RS-C7 → RS-C8, each committed and
                    pushed when coherent; RS-C3 and RS-C4 may swap; the base binary for RS-2 is built
                    before RS-C2
VALIDATION BUDGET   unit/integration/static unrestricted; real runs: each 300-day run (~13–15 s) at most
                    four times (RS-C4, RS-C8, M-RS1, one re-run); the RS-2 cross-build runs (two 2-day
                    runs, one refused resume, one refused server start, one positive resume); the
                    SIGKILL harness (expected under two minutes) at most four times; one full workspace
                    gate on the final head (background, ~5 min); about one hour in total;
                    real-model: NOT REQUIRED
LIVE DOCUMENTATION  §16 checkboxes; §16.10 E-RS ledger; §16.11 deviations
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for 12a at RS-C1
ENDPOINT AUTHORITY
  implementation + local validation   unresolved until the primary session's freeze message
  semantic commits, branch push       recommended authorized, as for 11a–11f
  PR creation / update                recommended authorized, as for 11a–11f
  scratch base build (RS-2)           recommended authorized: a build of the base in the implementing
                                      worktree before RS-C2, the binary copied to /tmp; no branch
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only, with a merge commit; never inherited, never widened
POST-MERGE SYNC     the planning session owns the step header, §§1–15, overall and MVP_STATUS's Updated
                    and S15 lines; the implementing session owns §16 and the evidence rows of RS-C8
NORMAL STOP         PR 12a READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a needed edit outside §16.1's change set — above all in kernel/ or contracts/ (F2, or a
                    kernel error variant); either 300-day digest differing from E-RS0; an existing test
                    failing for a reason other than QR-2's literals; presence's or movement's structural
                    scan needing an edit; a physics or market word the seam cannot do without; an answer
                    to QR-2 or QR-4 other than the design's
```

## 16.10 Evidence ledger

```text
E-RS0 RS-C0, 2026-10-07, planning session, on main @ fea2516's code (b8afd4f changes Markdown only).
      Debug profile, opt-level 1 (ARC-30); `cargo build -p mineworld-cli` 23.1 s. Logs under
      /tmp/s15-12a-plan/.
      `mineworld run worlds/social-cafe --headless --seed 7 --days 300` → exit 0, 339 lines, faults 0,
        history 365 330 facts (arrived 180 677, person-entered-place 20 720), sha-256 of every line but
        `wall` = ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b = step-10 E-0; wall 12.3 s.
      `mineworld run worlds/market-town --headless --seed 7 --days 300` → exit 0, 355 lines, faults 0,
        history 372 755 facts (arrived 173 125, person-entered-place 20 812), sha-256 of every line but
        `wall` = 365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d = step-10 E-P0; wall 14.7 s.
      ARC-39 and DEP-13 are free on every origin/* branch (git fetch; git grep over each ref's
        docs/DECISIONS.md for ARC-39+ and DEP-13+: no match).
      No .github/workflows: no CI.
      `python3 scripts/check_doc_headings.py` → 176 numbered sections across 25 documents, none
        duplicated. `python3 scripts/check_decision_ids.py` → 49 decision ids, all distinct.
      No cargo test run: this commit is documentation only. The two runs fix RS-1's references, not a
        gate.

E-RS-base  12a session, 2026-10-07, before any edit, on main @ 6d48e03 (b8afd4f + #53, Markdown only).
      `cargo build -p mineworld-cli` 30.6 s; target/debug/mineworld copied to
      /tmp/s15-12a/base-mineworld (sha-256 1f9ade07…993b). With it:
      `validate worlds/social-cafe` and `validate worlds/market-town` → exit 0, outputs kept as
        /tmp/s15-12a/base-validate-{social-cafe,market-town}.txt (RS-1's validate reference);
      `run worlds/social-cafe --headless --seed 7 --days 2 --save /tmp/s15-12a/B.mwsave` → exit 0,
        faults 0, 2 647 facts; `inspect B` → "systems presence v2, movement v1, …", head revision
        1933; B's world.sqlite sha-256 ebe3a0c9…135b, a pristine copy kept as B.orig.mwsave (RS-2).

E-RS1 RS-C1, 2026-10-07. `git fetch origin`; `git show <ref>:docs/DECISIONS.md | grep 'ARC-39\|ARC-4[0-9]'`
      over every refs/remotes/origin/* → no match: ARC-39 free.
      `python3 scripts/check_doc_headings.py` → 176 numbered sections across 25 documents, none
        duplicated. `python3 scripts/check_decision_ids.py` → 50 decision ids, all distinct (49 + ARC-39).
      PASS. Documentation only; no cargo run.

E-RS2 RS-C2, 2026-10-07, working tree on 0d1f4c7 + RS-C2's paths.
      `cargo test -p mineworld-presence -p mineworld-movement` (5.6 s): presence tests/presence.rs 15
        passed (incl. no_other_packs_vocabulary_and_no_floating_point_appear_in_this_crate),
        tests/resolver_catalog.rs 1 passed; movement disclosure 3, movement 7 (incl.
        this_pack_has_no_float_and_presence_and_conversation_know_nothing_of_it), persisted 1. PASS.
      `cargo test -p mineworld-cli --test inspect --test social_composition` (22.3 s): 3 + 4 passed.
        PASS. Attribution: this run's build started while RS-C3's sdk/installed edits were being
        written in the same tree, so it is re-run at RS-C3 (E-RS3), where it is attributed exactly.
      clippy -p mineworld-presence --all-targets -D warnings: clean. fmt --all --check: clean after
        `cargo fmt --all` (it re-wrapped lib.rs's use order, resolve.rs's trait signature, and one
        `let` in resolver_catalog.rs).
      M-RS7 (`true || current == offered` in register_resolvers): resolver_catalog FAILED —
        "panicked at systems/presence/tests/resolver_catalog.rs:46:10: this registration must panic:
        ()" (the [a] step). Reverted; `git grep -n MUTATION -- systems sdk worldpack tests tools` empty.

E-RS3 RS-C3, 2026-10-07, working tree on df23827 + RS-C3's paths.
      `cargo test -p mineworld-sdk -p mineworld-installed-systems -p mineworld-worldpack` (8.2 s):
        installed: installed.rs 3 (twins unedited), resolution.rs 2; sdk unit 1; worldpack unit 4,
        content_kinds 1, refusals 38, registration 1, social_cafe 15, structure 2; doc-tests 1. PASS.
      `cargo test -p mineworld-cli --test inspect --test social_composition` (19.9 s): 3 + 4. PASS
        (the exact-state re-run E-RS2 deferred here).
      clippy -p mineworld-sdk -p mineworld-installed-systems -p mineworld-worldpack --all-targets
        -D warnings: clean. fmt --all --check: clean after `cargo fmt --all` (two test files re-wrapped).
      M-RS9 (compose's register_resolvers line commented out): registration FAILED — "assertion
        `left == right` failed: composing registered the installed set's resolution: list, which is
        empty in this build / left: None / right: Some([])". Reverted; `git grep -n MUTATION --
        systems sdk worldpack tests tools` empty.
      Tool-discipline note: one read-only `awk` was used to reformat this run's test-count lines; it
        touched no file. Not repeated.

E-RS4 RS-C4, 2026-10-07, working tree on afda4e6 + movement's one file.
      `cargo test -p mineworld-movement`: disclosure 3, movement 7, persisted 1. PASS.
      clippy -p mineworld-movement --all-targets -D warnings: clean.
      `cargo build -p mineworld-cli`; `mineworld run worlds/social-cafe --headless --seed 7 --days 300`
        → exit 0, 339 lines, faults 0, history 365 330 facts (arrived 180 677, person-entered-place
        20 720), no `stopped-short` line; sha-256 of every line but `wall` =
        ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b = E-RS0. Wall 13.4 s.
        PASS (RS-1's first bullet, early; 300-day run 1 of 4). Log /tmp/s15-12a/rsc4-cafe300.txt.

E-RS5 RS-C5, 2026-10-07, working tree on b1d4038 + RS-C5's paths.
      `cargo test -p mineworld-acceptance --test arrival_resolvers --test
        arrival_resolvers_unregistered` → 7 passed (RS-3, RS-4, RS-5, RS-6, RS-7 registered, RS-12,
        RS-14) + 2 passed (RS-9, RS-7 unregistered). PASS. The first run passed as written; the
        mutations below are what show each assertion bites.
      clippy -p mineworld-acceptance -p mineworld-presence --all-targets -D warnings: clean after two
        type aliases (`Spot`, `Resident`) and `type Row` for `type_complexity`. fmt clean.
      `git diff Cargo.lock`: + "mineworld-movement", + "mineworld-persistence" in mineworld-acceptance's
        dependencies; nothing else.
      Mutations (each applied in the working tree, run, reverted; `git grep -n MUTATION -- systems sdk
      worldpack tests tools` empty afterwards; 7 + 2 green again):
        M-RS2  the fold asks no resolver (`.iter().take(0)` on the catalog) → RS-3 FAILED: "left:
               [Arrived(PersonId(EntityId(3)), … x: Millimetres(5500) …)]" — alice recorded at
               (5 500, 2 000); RS-4, RS-5, RS-6, RS-14 failed with it.
        M-RS3  arrivals drops the displaced emissions (`.take(0)`) → RS-3 FAILED: the right side holds
               "Arrived(PersonId(EntityId(4)), … x: 5200, y: 2100 …)", bob's, missing on the left.
        M-RS4  check (c) disabled (`&& false`) → RS-4 FAILED: "lengthened the arrival: recorded
               [Arrived(… x: Millimetres(6500) …), StoppedShort { … }]".
        M-RS5  `register_resolvers` does not sort → RS-5 FAILED with alice at x 4990 instead of 4000.
               The first run of this mutation failed earlier, at the registered_resolvers() assertion
               ([test-grid, test-rogue, test-fences]); the position assertion was moved first, so the
               behavioural claim is what fails, and the id-order assertion stays after it.
        M-RS6  arrival() never refuses a changed placement (`.filter(|_| false)`) → RS-6 FAILED: "left:
               Ok(Emission { … arrived …}) right: Err(System { code: resolution-refused, detail:
               test-fences: would change a placement; … })".
        M-RS8  fences' `require_registered` commented out → RS-9 FAILED: "installing test-fences must
               panic: Ok(())"; RS-7's unregistered half still passed (it installs no resolver pack).

E-RS6 RS-C6, 2026-10-07, working tree on 9acbe04 + RS-C6's paths.
      `cargo test -p mineworld-acceptance --test arrival_resolvers_resume` (harness run 1 of ≤ 4):
        "[resolver-yard] control: 401 revisions, 516 facts, 14 snapshots; 346 moves accepted, 54
        refused; 123 stopped-short, 41 displaced arrivals"; early: kill at 80, 80 committed, survivor
        restored snapshot 64, re-executed 16 revisions (24 facts); middle: kill at 203, 203 committed,
        snapshot 192, re-executed 11 (12 facts); late: kill at 327, 327 committed, snapshot 320,
        re-executed 7 (9 facts); each byte-identical to the control and verify() = 401 revisions;
        "PASS in 0.3 s". PASS.
      M-RS10 (run 2): fences' stop distance minus a process-global AtomicI32 counter's parity →
        FAILED: the early survivor happened to agree, the middle survivor was refused — "resumes:
        ReplayDiverged { revision: WorldRevision(194), detail: \"fact 248 differs from the logged fact
        248\" }"; "resume failed: exit status: 101". Reverted; `git grep -n MUTATION -- systems sdk
        worldpack tests tools` empty; the failed run's scratch saves under $TMPDIR removed.
      Run 3, after the revert and `cargo fmt`: identical numbers, "PASS in 0.2 s". clippy -p
        mineworld-acceptance --all-targets -D warnings clean.
```

## 16.11 Deviations and discoveries during implementation (12a session)

**DR-1 (bounded) — what "keeps the local-position rule of (b)" means for a displaced person.**
- Previous assumption: SD-R10 (e) says a displaced person's new location "keeps the local-position
  rule of (b)". Rule (b) compares `reached` with `to`, the walker's destination.
- Audit evidence: a displaced person has no `to` of their own; their own "as asked" is where they are
  now. SD-R10's rationale reads "never invent or drop a position, turn a person".
- Corrected understanding: for a displaced person the comparison is with their current `Presence`:
  the new location has a local position exactly when their current one does, **and the same facing**.
  Facing is not named in (e)'s text, but "turn a person" is in the rationale; keeping it narrows what a
  resolver may do and widens nothing (QR-8's direction).
- Implementation: `check_displaced` in `resolve.rs` ("invented or dropped a displaced person's local
  position", "turned a displaced person").
- Validation: RS-4's table gains one row, "turns a displaced person", beside the eleven named.

**DR-2 (bounded) — RS-10's guard names the resolver's id, not its type.**
- Previous assumption: RS-10 says the installed set's guard "fails, naming the type".
- Audit evidence: `Capability::resolvers()` returns `Box<dyn ArrivalResolver>` (SD-R8);
  `type_name_of_val` on it gives the trait object's name, not the listed type. What the value can
  answer is `resolver_of()`.
- Implementation: the guard names the resolver's `SystemId`, which is the name the catalog, the
  panics and every refusal use. The negative control's stub resolver has a distinct id.
- Impact: none on what is caught; only the word in the message.

**DR-3 (bounded, reviewed) — `tests/acceptance/tests/resolvers/mod.rs` is 988 lines.**
- The file holds the five synthetic packs SD-R11 names (three resolvers, each a component, a genesis
  fact, a system and a resolver), the world builder and the RS-7 script. `ENGINEERING_STANDARDS.md`
  and `CLAUDE.md` §4 make ~800 lines a strong warning, a review trigger rather than a rule.
- Options: split into `resolvers/{fences,grid,rogue,echo,town}.rs` — each a path outside §16.1's
  change set, which names one support file, and the kickoff makes any path outside §16's change set a
  material stop; or compress the three resolver packs with a local macro — the "clever
  metaprogramming" the standards warn against.
- Decision: keep one file, with one section per pack in SD-R11's order. It is test support compiled
  into no library. Flagged for the operator's review; a split is a mechanical follow-up if wanted.
