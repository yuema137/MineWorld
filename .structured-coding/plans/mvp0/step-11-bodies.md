# Step 11 — Bodies and physical interaction (S15)

**Role:** step document for a new step, proposed as **S15 — Bodies and physical interaction**. It
records the operator's requirement, the audit, the answers to the planning questions, the Rapier
determinism prototype, the proposed ownership, facts and contracts, a PR split, risks and open
questions. It holds no frozen PR design yet: the first PR is detailed only after the operator decides
the material questions in §13 (`CLAUDE.md` §3, "detail one step ahead").
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S11, S12, S14), §7.
**Lifecycle:** step design `DESIGN FROZEN (2026-10-07)`, revision 1.

- The architecture, the ownership and the five-PR split are frozen.
- Each of 12a–12e is detailed to the commit and frozen in turn.
- **PR 12a — MERGED** as GitHub #56, merge commit `03f1d7c` (2026-10-07), PR head `dcaeae3`, final
  executable head `dc2b6b4`. Design, evidence and the operator's review in §16; review record §16.12.
- **PR 12b — MERGED** as GitHub #59, merge commit `9c617ed` (2026-10-07), PR head `ce986fe`, final
  executable head `0733c77`. Design, evidence, deviations DB-1 … DB-10 and the DB-10 ruling in §17;
  the operator's review in §17.12. Carried to 12d: QB-11's ≤ 1.5× bound on the towns (DB-10 (b2)),
  F-B7 and FU-12a-1.
- **PR 12c — MERGED** as GitHub #67, merge commit `889d217` (2026-10-08), PR head `5ef5bff`, final
  executable head `145c7f7`. Design, evidence, deviations DO-1 … DO-18 and the AO-2 rulings in §18;
  the operator's review in §18.12. Carried to 12d: FU-12c-1, with QB-11's ≤ 1.5× bound on the towns
  (DB-10 (b2)), F-B7 and FU-12a-1 still open.
- **PR 12d-0 — bodies' cost — MERGED** as GitHub #84, merge commit `78ca5ae` (2026-10-08), PR head
  `995d556` (CI run 37853993552: `fast` and `test` green on that head). A precursor inserted by the
  primary session's QD-2 ruling. Landed SD-Z2, SD-Z6 and SD-Z5 (`bodies` version 3); SD-Z1, SD-Z3 and
  SD-Z4 dropped after measuring; QB-11 re-scoped by the operator to TZ-9a (≤ 3.0×, social-cafe's 3.60×
  accepted, not passed) and TZ-9b (≤ 50 ms). Design, evidence, deviations Z-D1 … Z-D9 and the rulings in
  §20; the post-merge record in §20.14.
- **PR 12d** — the town gets bodies: detailed in §19, refreshed against `main @ 78ca5ae` (§19's
  "Refresh" note); **DESIGN FROZEN (2026-10-08), primary session**, to be implemented in a fresh
  session in `impl-12d` on `mvp0/pr-12d-towns`. The operator decided QD-1
  (amend check 3) and the primary session ruled QD-2 … QD-13 (§19's header); QD-14 … are new.
  **Paused** at `8814aad` on material stop TD-D7 (the walkers do not route round the new geometry), by
  the operator's ruling of 2026-10-08: "add pathfinding; 12d waits for it".
- **PR 12n — navigation** (12n-1 the walk, 12n-2 people walk there): §21, **DESIGN FROZEN
  (2026-10-09, primary session)**; to be implemented in a fresh session in `impl-12n` on
  `mvp0/pr-12n-navigation` (§21.14). Drafted on `plan/s15-12n`, **revision 1** after the primary
  session's review (R-12n-1: strides are embodied `walk-step` requests at wall cadence, not a world-time
  Process). Also decides 12d's TD-D8 (§21.6).

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

# 16. PR 12a — the arrival-resolver seam (full design; DESIGN FROZEN 2026-10-07; MERGED #56, `03f1d7c`)

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

- [x] Implementation: as scoped. Three tests: `the_seam_names_no_physics` (the five directories,
  recursively, and the seven added test files; a missing one fails; ≥ 33 files),
  `movement_names_no_resolver`, and a splitter check (`RigidBody`, `body_shape`, `Bodies`, `collide`
  found; `nobody`, `somebody`, `embody` not). Three pre-existing lines in files 12a does not touch are
  admitted by a self-checking exemption list (DR-4).
- [x] Validation (E-RS7): 3 pass; both planted violations fail by name, removed.
- [x] Review: the scanned code paths are every non-Markdown path §16.1 changes or adds outside `docs/`,
  `tools/cli/tests` (QR-2's two literals), `Cargo.lock`, `tests/acceptance/Cargo.toml` and
  `tests/acceptance/src/lib.rs` — those last are manifests and a doc table that RS-13 does not list,
  and none names a word of the vocabulary (checked by eye on the diff); Markdown is the only file-type
  exclusion, and the scan's own file the only file exclusion, both stated in its header.

### RS-C8 — Close: real runs, the refusal, status, full gate, ledger

- [x] RS-1: the two 300-day runs on the final executable head; both `validate` diffs; M-RS1 applied to
  one run and reverted (E-RS8).
- [x] RS-2: the cross-build evidence, with the base binary built from the base before RS-C2 and kept at
  `/tmp/s15-12a/base-mineworld` (E-RS-base, E-RS8).
- [x] RS-15, RS-16: the AC-1 test (13) and the I-2 scan (4) pass unedited inside the gate; the
  diff-scope check; fmt and clippy (E-RS8).
- [x] Full gate once on the final executable head `dc2b6b4`, in the background: 542 passed, 0 failed,
  130 harness binaries plus 2 `harness = false` programs, 249 s (E-RS8).
- [x] Documentation: `docs/MVP_STATUS.md` (the "Arrival resolution" capability row, one evidence row);
  §16 checkboxes; §16.10; the handoff.
- [x] Review: RS-1 … RS-16 each with evidence (the table below E-RS8); deviations DR-1 … DR-4 in
  §16.11.

**PR 12a lifecycle:** **MERGED** — GitHub #56, merge commit `03f1d7c` (a merge commit, as frozen), on
the operator's authorization, after the operator's own review (§16.12). PR head `dcaeae3`; final
executable head `dc2b6b4`; later commits Markdown only. Implementation context CLOSED. POST-MERGE SYNC:
done by the planning session on `docs/s15-12a-merged` — the step header, §16.12, `overall.md` §7 and
MVP_STATUS's Updated and S15 lines.

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

E-RS7 RS-C7, 2026-10-07, working tree on 0d21436 + seam_vocabulary.rs.
      First run: FAILED on its own file-count floor (33 files, floor guessed at > 40); the floor was set
        to the counted 33 (presence 8, movement 6, sdk 4, installed 1, worldpack 7, added tests 7).
      Second run: FAILED — systems/movement/src/action.rs:14 `body`; after the first exemption, third
        run FAILED — action.rs:21 `physical`, worldpack/src/read.rs:354 `collision`. All three are
        pre-existing prose in files PR 12a does not touch (`git diff 6d48e03 -- <file>` empty) → DR-4.
      Fourth run: 3 passed. PASS.
      Planted (both at once): `// a capsule nudges` at systems/presence/src/resolve.rs:38 and
        `use mineworld_presence::ArrivalResolver as _;` at systems/movement/src/system.rs:20 → FAILED:
        "…/systems/presence/src/resolve.rs:38: capsule", "…/resolve.rs:38: nudges";
        "…/systems/movement/src/system.rs:20: ArrivalResolver". Removed; `git diff --stat systems`
        empty; presence 15 + 1 and movement 3 + 7 + 1 pass again (their own structural scans unedited
        and green); seam_vocabulary 3 pass. clippy and fmt clean.

E-RS8 RS-C8, 2026-10-07, on the final executable head dc2b6b4 (`cargo build -p mineworld-cli`, binary
      copied to /tmp/s15-12a/pr-mineworld).
      RS-1 (300-day runs 2 and 3 of 4):
        `run worlds/social-cafe --headless --seed 7 --days 300` → exit 0, faults 0, 365 330 facts, no
          `stopped-short` line, sha-256 of every line but `wall` =
          ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b = E-RS0. Wall 12.5 s. PASS.
        `run worlds/market-town --headless --seed 7 --days 300` → exit 0, faults 0, 372 755 facts,
          sha-256 = 365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d = E-RS0. Wall
          14.7 s. PASS.
        `validate worlds/social-cafe` and `validate worlds/market-town` with the PR binary: exit 0 each,
          `cmp` against the base binary's outputs (E-RS-base): identical. PASS.
        M-RS1 (run 4 of 4; `if reached != to || true` in arrivals) → social-cafe sha
          ded1ffd256d3651b5de33d9320815eb8641e745553bbb41b49cbd17cf19d2fcb ≠ E-RS0, and the summary
          gains "facts      stopped-short 180665" (history 545 995 facts): the instrument sees a seam that
          records one extra fact per move. Reverted; `git grep -n MUTATION -- systems sdk worldpack
          tests tools` empty; `git diff dc2b6b4` outside docs/ and .structured-coding/ empty.
      RS-2 (cross-build, real binaries; no committed test, QR-9):
        PR binary `run worlds/social-cafe --headless --seed 7 --days 2 --save /tmp/s15-12a/N.mwsave` →
          exit 0, faults 0, 2 647 facts, fingerprint ce6b20e9f416f9b3 — the base run's (E-RS-base).
        `inspect B` and `inspect N` (PR binary): `diff` shows only the save path, the instance id
          (excluded by ARC-27) and "systems    presence v2, …" / "systems    presence v3, …"; the same
          head (revision 1933 at t171910), journal kinds and fact counts. PASS.
        PR binary `run worlds/social-cafe --headless --seed 7 --days 3 --save B` → exit 1:
          "[mineworld] system 'presence' is v3 here, but the save was written by the older v2".
        PR binary `server worlds/social-cafe --save B` → exit 1: "[mineworld] Social Café
          (social-cafe) — 7 system(s), 11 seat(s)" / "[mineworld] the world could not be built: system
          'presence' is v3 here, but the save was written by the older v2".
        `cmp B.mwsave/world.sqlite B.orig.mwsave/world.sqlite`: identical (sha-256 ebe3a0c9…135b
          before and after). PASS.
        Positive control: PR binary `run … --days 3 --save N` → exit 0, "resumed
          /tmp/s15-12a/N.mwsave/world.sqlite at revision 1933 (snapshot 1920 + 13 re-executed)", faults
          0, 3 851 facts; `inspect N` head revision 2868 at day 3. PASS.
      RS-16: `git diff --name-only 6d48e03...HEAD` = 28 paths, every one in §16.1's change set (plus
        docs/MVP_STATUS.md in the closing commit); kernel/, contracts/, persistence/, server/,
        cognition/, clients/, worlds/, authoring/, tools/cli/src/, root Cargo.toml: empty diff;
        Cargo.lock: two added lines in mineworld-acceptance's dependency list. PASS.
      Full gate (once, background, started 21:57:04, total 255 s):
        `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --all-features -- -D
        warnings` exit 0; `cargo test --workspace --no-fail-fast` exit 0, wall 249 s: 130 harness
        binaries, 542 passed, 0 failed, 0 ignored; the two `harness = false` programs passed —
        persistence's kill_and_resume ("[cafe] PASS", "[clock] PASS") and arrival_resolvers_resume
        ("[resolver-yard] PASS"); ac1_composability 13 and precursor_vocabulary 4 passed unedited
        (RS-15); `python3 scripts/check_doc_headings.py` → 176 sections, none duplicated;
        `python3 scripts/check_decision_ids.py` → 50 ids, all distinct. PASS.
        Logs: /tmp/s15-12a/gate.log, test.log, clippy.log.

RS-1 … RS-16, with where each is shown:
  RS-1   PASS  E-RS8 (both digests, validate identical, every existing test in the gate; the only
               existing-test edits are QR-2's three literals, df23827); M-RS1 seen (E-RS8)
  RS-2   PASS  E-RS8 (cross-build, refusal by name, B unchanged, positive control)
  RS-3   PASS  arrival_resolvers::an_arrival_is_resolved_before_it_is_recorded; M-RS2, M-RS3 (E-RS5)
  RS-4   PASS  …::presence_refuses_a_resolution_that_breaks_its_rules_and_names_the_resolver (14 rows +
               positive control; DR-1); M-RS4 (E-RS5)
  RS-5   PASS  …::resolvers_are_asked_in_ascending_system_id; M-RS5 (E-RS5)
  RS-6   PASS  …::arrival_refuses_a_placement_a_resolver_would_change_and_arrivals_records_it; M-RS6
  RS-7   PASS  …::a_registered_resolver_is_inert_where_its_state_is_absent and
               arrival_resolvers_unregistered::a_world_without_a_resolver_pack_runs_unchanged_…
  RS-8   PASS  presence tests/resolver_catalog.rs; M-RS7 (E-RS2)
  RS-9   PASS  arrival_resolvers_unregistered::installing_a_resolver_pack_…_panics_naming_why; M-RS8
  RS-10  PASS  worldpack tests/registration.rs; installed tests/resolution.rs (DR-2); M-RS9 (E-RS3)
  RS-11  PASS  arrival_resolvers_resume (E-RS6, and inside the gate); M-RS10 (E-RS6)
  RS-12  PASS  arrival_resolvers::a_system_may_state_arrived_and_hear_it
  RS-13  PASS  seam_vocabulary (3 tests; DR-4); both planted violations failed by name (E-RS7)
  RS-14  PASS  arrival_resolvers::runtime_disable_of_a_resolver_pack_is_not_honoured; ARC-39's own
               paragraph
  RS-15  PASS  ARC-39 and MODULE_SPEC §3.1 in RS-C1 (0d1f4c7) before any code; both doc checks; the
               AC-1 test and the I-2 scan in the gate, unedited
  RS-16  PASS  E-RS8 scope; fmt and clippy in the gate
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

**DR-4 (bounded; flagged for the operator) — RS-13 "over the current text" meets three pre-existing
words.**
- Previous assumption: RS-13 scans the current text of five source directories for the physics
  vocabulary and expects none. §16.2 audited the vocabulary only against the lines 12a adds.
- Audit evidence: on the base, three lines already match, none of them the seam:
  `systems/movement/src/action.rs:14` "A body walking or jogging that reports…" and `:21` "this pack's
  policy rather than a physical constant" (MAX_STRIDE's documentation, `41bb073`);
  `worldpack/src/read.rs:354` "…the normal case rather than a collision" (seat keys, `ff49368`). 12a
  may not edit either file (I-1: movement exactly SD-R9; `read.rs` is outside §16.1).
- Options: (a) edit the three comments — two frozen-scope breaches; (b) scan only the lines 12a adds
  — weaker than RS-13, which deliberately reads the current text so that a later PR cannot add the
  vocabulary to the seam either; (c) keep the whole-text scan and admit exactly those three words on
  exactly those lines, with a reason each, through a list that fails if an entry admits nothing
  (ARC-35 item 7's allow-list discipline).
- Decision: (c), in `seam_vocabulary.rs`'s `PRE_EXISTING`. Every other word on every line of the
  five directories is still refused, and any new use is refused. The operator may prefer (a) as a
  two-file comment edit in a later PR, after which the entries fail as unused and are removed.
- Validation: E-RS7 (the three failures, then green; the planted violations still fail by name).

## 16.12 Operator review and merge (post-merge record, planning session)

**Merged:** GitHub #56, merge commit `03f1d7c` (2026-10-07), with a merge commit as frozen. PR head
`dcaeae3`; final executable head `dc2b6b4`, unchanged by the Markdown-only commit after it.

**The operator's review evidence, on the PR head, before the merge:**

- **Gates re-run:** 542 passed, 0 failed. The `harness = false` programs passed: `resolver-yard`
  (`arrival_resolvers_resume`), `cafe` and `clock` (persistence's `kill_and_resume`). This matches
  E-RS8.
- **Scope:** the diff has no path under `kernel/`, `contracts/`, `server/`, `persistence/`,
  `cognition/` or `worlds/` (I-1).
- **The operator's own mutation:** `arrivals()` suppressed the `stopped-short` fact but kept the
  shortened arrival. Four acceptance tests failed:
  - `an_arrival_is_resolved_before_it_is_recorded` (RS-3);
  - `resolvers_are_asked_in_ascending_system_id` (RS-5);
  - `arrival_refuses_a_placement_a_resolver_would_change_and_arrivals_records_it` (RS-6);
  - `presence_refuses_a_resolution_that_breaks_its_rules_and_names_the_resolver` (RS-4).

  The mutation was reverted. It is independent of the implementing session's M-RS1 … M-RS10: none of
  those removed `stopped-short` alone.
- **After the merge, on main:** `ac1_composability` passes 13/13 and `precursor_vocabulary` passes
  4/4 (RS-15 holds on the merged tree).

**Deviations ruled on:**

- **DR-1, DR-2:** bounded, as recorded in §16.11; no ruling needed.
- **DR-3: accepted.** `tests/acceptance/tests/resolvers/mod.rs` stays one 988-line file, because §16.1
  names a single support file. A split is not scheduled.
- **DR-4: accepted, with a follow-up.** The self-checking allow-list `PRE_EXISTING` in
  `tests/acceptance/tests/seam_vocabulary.rs` admits three pre-existing words:
  - `systems/movement/src/action.rs:14` (`body`);
  - `systems/movement/src/action.rs:21` (`physical`);
  - `worldpack/src/read.rs:354` (`collision`).

  **Follow-up FU-12a-1:** the next PR whose frozen scope may edit movement's `action.rs` and
  worldpack's `read.rs` rewords those three comments. The three entries then fail as unused, as
  designed, and are removed in the same PR. 12b's change set does not include either file (§17.1), so
  the follow-up stays open after 12b.

**What 12a leaves for 12b.** The `resolution:` line in `systems/installed` reads `[]`. 12b adds
`mineworld_bodies::BodiesSystem,` to it. With that, bodies is the first registered resolver in every
build. It stays inert in every world that does not install `bodies`.

---

# 17. PR 12b — people: walls and nudging (full design; DESIGN FROZEN 2026-10-07; MERGED #59, `9c617ed`)

**Lifecycle:** the planning session drafted this on `mvp0/s15-12b-plan` on 2026-10-07. The primary
session froze it the same day; the freeze record is §17.0. **MERGED** as GitHub #59, merge commit
`9c617ed` (2026-10-07); the review and merge record is §17.12.

## 17.0 Freeze record

**DESIGN FROZEN (2026-10-07), primary session.** The §17.9 execution contract is confirmed. This
record binds and overrides any other text in §17.

The planning session marked five questions [OM]. None of them contradicts a decision the operator
made: QB-1, QB-10, QB-15, QB-2, QB-3, QB-5 and QB-12 are untouched. Each refines text written by the
planning session or the primary session, so the primary session decides them here:

- **QP-1 — accepted.** ARC-39's promised reaction guard is replaced by a check at the start of each
  resolution. It panics if the place already holds an overlap, naming the pair. The replacement is
  recorded as a dated note on ARC-39.
  - An overlap there is an invariant violation, not a state the world may continue in.
  - The 30-day and 300-day runs must show it never fires, and a mutation must show that it does
    fire.
- **QP-2 — accepted.** The two named tests are edited with their claims unchanged, under the same
  rule as step-09 I-5 and 12a's QR-2.
- **QP-3 — overruled.** `rapier3d =0.36.0` is declared in `systems/bodies/Cargo.toml` itself, not in
  root `[workspace.dependencies]`.
  - Installing `bodies` must touch only `systems/**` and `Cargo.lock`, as ARC-33 states for every
    pack. The convenience of a workspace dependency is not worth an exception to that.
  - DEP-13 records the pin's location. PB-2's pin test reads the pack's manifest.
- **QP-7 — accepted.** A crossing into a jammed doorway places the person at the nearest free point,
  never inside another body or a solid. A room-capacity check at load guarantees such a point exists.
  §5's sentence is amended to match.
- **QP-9 — accepted conditionally.** If, and only if, PB-14's ≤ 1.5× dev-profile bound fails, 12b may
  add per-package dev `opt-level` overrides for `rapier3d` and `parry3d` in the root manifest. If it
  does:
  - it records an ARC-30 note;
  - it shows the bound passing afterwards;
  - it shows that no output changed.

  If the bound passes without the overrides, the root manifest is not touched.
- **QP-4 … QP-6, QP-8, QP-10 … QP-16 — accepted as recommended.** These include:
  - the committed `worlds/bodies-yard`;
  - the `body:` place section;
  - the head-on bias design and its fallback ladder. If no rung passes, it comes back to the
    primary session before it goes to the operator.
- **FU-12a-1 stays open.** 12b does not edit movement's `action.rs` or worldpack's `read.rs`.
- **The 12d finding is recorded for 12d:** the café's doorway point lies 200 mm from its wall, which is
  less than a person's radius.
- **Merge:** with a merge commit.
- **Implementation:** in a fresh session on `mvp0/pr-12b-people`, in its own worktree.

## 17.1 Identity, base, approved scope

```text
PR            12b — people: walls and nudging (S15, second of five; the first resolver pack)
base          main after #57 (the 12a post-merge docs PR) merges — 03f1d7c + Markdown only. Re-audit
              §17.2 if anything under systems/presence, systems/movement, systems/installed, sdk/rust,
              worldpack, authoring, kernel/src, tests/acceptance or tools/cli/tests moved
branch        mvp0/pr-12b-people, in its own worktree, held by the implementing session only
audit         §17.2 (main @ 03f1d7c, 2026-10-07)
scope         §4.5 (the nudge rule), §6.3 (DC-1 … DC-9), §9.7–§9.8 (R′, verify-then-degrade), §10.1
              (I-2 … I-5, I-8, I-10 … I-13), §11.1's 12b row, QB-10, QB-16 — as refined by SD-B1 …
              SD-B16 and answered by QP-1 … QP-16
depends on    12a merged (03f1d7c). The ArrivalResolver seam is used as merged, unchanged.
merge         a merge commit, never a squash
```

**Goal.** People have bodies wherever a world gives a place geometry. A new System Pack, `bodies`, is
the build's first registered `ArrivalResolver`. Before presence records an arrival into a place that
has geometry, it does three things:
- it stops the walker at walls and furniture;
- it nudges the people in the way aside, within the bounds the operator set (QB-10);
- it verifies the integer result, and degrades it when a pair would overlap.

So the log holds only arrivals in which no two people overlap and nobody stands in a wall. Rapier
(`DEP-13`) does the sweeping, behind one module. Nothing of Rapier outlives one resolution. Every
world without geometry records exactly what it records today, byte for byte. That includes
social-cafe and market-town, which gain geometry only in 12d.

**Change set.** Every path this PR may touch:

```text
docs/DECISIONS.md                     DEP-13 (new, after ARC-39); a dated note on ARC-39 (QP-1, the
                                      bounds QR-11 moved here)
docs/MODULE_SPEC.md                   §4.1: the `body` section (place files), its row in the section
                                      table, "six sections" → seven
docs/MVP_STATUS.md                    one capability row, one evidence row
Cargo.toml (root)                     [workspace.dependencies] rapier3d, exact pin (QP-3). Contingent,
                                      only if PB-14(b) fails and the operator approved QP-9:
                                      [profile.dev.package.rapier3d] and .parry3d opt-level
Cargo.lock                            rapier3d =0.36.0 and its transitive packages; mineworld-bodies;
                                      no existing package changes version
systems/bodies/**                     new: the pack (crate mineworld-bodies, SystemId "bodies")
systems/installed/Cargo.toml          mineworld-bodies = { path = "../bodies" }
systems/installed/src/lib.rs          Bodies => mineworld_bodies::BodiesSystem, and the resolution line
                                      [mineworld_bodies::BodiesSystem,]; one doc sentence
worlds/bodies-yard/**                 new: the integration world (SD-B13, QP-4)
worldpack/tests/registration.rs       the installed set's list is now [bodies] (QP-2)
tests/acceptance/tests/seam_vocabulary.rs   a self-checking admission for the installed set's two
                                      lines that name the pack (QP-2)
tools/cli/tests/bodies_yard.rs        new: the 30-day real run, its scan and its counterfactual
tools/cli/tests/bodies_yard_restart.rs     new: two processes, SIGKILL and resume
tools/cli/tests/bodies/mod.rs         new: the scan, shared by the two files above
.structured-coding/plans/mvp0/{step-11-bodies,handoff}.md   this ledger, the handoff
```

Paths with no diff:
- `kernel/`, `contracts/`, `persistence/`, `server/`, `cognition/`, `clients/`, `authoring/`,
  `sdk/`;
- `systems/presence/`, `systems/movement/`, and every other existing System Pack;
- `worldpack/src/`, `tools/cli/src/`;
- `worlds/social-cafe/`, `worlds/market-town/`.

No existing test is edited except QP-2's two files.

**Non-goals.**
- Loose objects, `kick`, `throw` and `shove` belong to 12c. The `Lying` component, `object-moved` and
  `person-shoved` come with them, and so does any `BodyShape` on an Item.
- Geometry for social-cafe and market-town, and their digest re-baselines, belong to 12d.
- Godot, Jolt (`DEP-14`), colliders and reconciliation belong to 12e.
- Per-person body shapes are deferred: every person is the default capsule (QP-6).
- No line of access (QB-9), no speed limit (QB-14), and no runtime disable of a resolver (QB-17).
- No change to presence, movement, the seam or the kernel.

## 17.2 Source audit (`main @ 03f1d7c`, 2026-10-07)

Each finding below was read in this session from the file named, or measured.

| ID | Finding | Evidence | Consequence for 12b |
| --- | --- | --- | --- |
| **F-B1** | **A guard that checks each recorded `arrived` sees half-applied states.** Reduction walks a generation one fact at a time, and lets every subscriber react to a fact before it moves on to the next. `arrivals()` returns the walker's `arrived` first and then the displaced people's. So when bodies hears the walker's arrival, the people it displaced have not moved yet, and the walker overlaps them for that moment. Genesis is the same: one generation, reduced in order. | `kernel/src/dispatch.rs:603–647` (`reduce`), `:509–527` (`genesis`); `systems/presence/src/event.rs:135–162` (`arrivals` order) | ARC-39 item 7, bullet 3 says bodies checks the invariant "in its reactions to the recorded arrivals". If built that way, every nudge would fail the dispatch. That check cannot be built. QP-1 **[OM]** proposes a check on the state a resolution starts from instead (SD-B9). |
| **F-B2** | **Two merged tests assume an empty resolver list, or a seam that names no physics.**<br>- `worldpack/tests/registration.rs` asserts `registered_resolvers() == Some(Vec::new())` three times, and that the panic message holds `already registered []`.<br>- `tests/acceptance/tests/seam_vocabulary.rs` scans `systems/installed/src`, where 12b's two installed lines carry the word `bodies` (`mineworld_bodies`, `BodiesSystem`). | `worldpack/tests/registration.rs:34–64`; `seam_vocabulary.rs:41–87` (`PHYSICS`, `PRE_EXISTING`, `SEAM_DIRECTORIES`) | Installing bodies fails both tests, although the claim each holds is unchanged. Both are edited, and each edit is named (QP-2 **[OM]**). |
| **F-B3** | **A resolver can neither refuse nor leave the place.** `Resolution` offers only `stopped_at` and `displacing`. Presence refuses any `reached` that is not in `to`'s place (rule (a)). | `systems/presence/src/resolve.rs:93–133, 295–326` | §5 says a crossing into a crowded doorway "is blocked (the person stays on their side of the door)". The seam cannot express that. A crossing must end somewhere in the destination place, so SD-B8 finds a free spot there, guaranteed by a capacity check at genesis (QP-7 **[OM]**). |
| **F-B4** | **Genesis order lets bodies check authored positions only when it reduces its own fact.**<br>- Genesis states passages, then locations, then sections: items', organizations', places', people's.<br>- A section's `seed` sees the assembled world with no state, by design.<br>- The whole list is one generation, reduced in order.<br>So bodies' `place-shaped` is reduced after every authored `arrived`. | `worldpack/src/load.rs:31–33, 295–341`; `authoring/src/section.rs` (`Seeding` doc); `kernel/src/dispatch.rs:509–527` | The overlap check for authored people (QR-7) lives in bodies' reduction of `place-shaped`, and so do the inside-the-floor, out-of-solids and capacity checks. Each refuses with `FactRefusedByOwner`. The refusal surfaces as a load error, which is what `mineworld validate` runs (`tools/cli/src/main.rs:251`). |
| **F-B5** | **No place carries geometry, and no pack owns a place section yet.** Every section today is carried by people, items or organizations. The loader already reads, refuses and seeds sections on place files. | `git grep CARRIED_BY systems/*/src`; `worldpack/src/read.rs:191–215`; `cafe.yaml` ("A World Pack authors no geometry beyond these positions") | `body:` on a place file is the first place section (`ARC-31`). The loader needs no edit. |
| **F-B6** | **The paced controller crosses a doorway onto the exact `there` point.**<br>- `through` asks for `passage.there()` once the walker is within a stride of `here`.<br>- `wander` draws up to ±1 400 mm per axis.<br>- `approach` stops 1 000 mm short of its target. | `cognition/rule-controller/src/paced.rs:66–73, 265–281, 333–363` | With bodies, nearly every crossing lands on an occupied or recently occupied point. Entry placement (SD-B8) is exercised constantly, not rarely. Contacts within a place come mostly from wandering and from the doorways, not from approaches. |
| **F-B7** | **The café's doorway point is 200 mm from its front wall, closer than a person's radius.** The street side is the same kind of point. | `worlds/social-cafe/places/cafe.yaml` (`here: {x: 1610, y: 200}`) | Not 12b's (the world is untouched). Recorded for 12d: either move the doorway points at least 310 mm inside, or accept that entry placement shifts a crossing person off the point. |
| **F-B8** | **A new external dependency is declared in the root manifest.** The root says: "One authoritative version per dependency across the workspace. A crate that needs a dependency not listed here is adding a dependency, which is a reviewable decision". `ARC-33` item 3 keeps that rule. No pack declares an external dependency outside `[workspace.dependencies]`. | `Cargo.toml` (`[workspace.dependencies]` comment); `DECISIONS.md` ARC-33 item 3; `systems/*/Cargo.toml` | `rapier3d` goes in the root (QP-3 **[OM]**). Installing *this* pack therefore edits the root manifest once, on top of `ARC-33`'s three lines. `DEP-13` says so. |
| **F-B9** | **Rule (c) compares squared 3D distances, and a quantized float can lengthen an arrival by a millimetre.**<br>- Presence checks `|reached − from|² ≤ |to − from|²` in `i128`, z included.<br>- It requires `reached` to keep `to`'s facing, and to have a local position exactly when `to` does.<br>- Rapier's answer is `f32` metres. Rounding to millimetres can land 1 mm past `to`. | `systems/presence/src/resolve.rs:295–326` | Bodies keeps `to`'s z and facing, and treats a sweep that ends within 1 mm of its target as reaching it exactly (SD-B6). Without that, presence would refuse a clean stride as "lengthened", and movement would fail the dispatch. |
| **F-B10** | **The prototype calls `f32::sqrt` from std** (`dist`, `len` in `src/resolve.rs:35–38, 97–99`). DC-3 forbids that on values that reach Rapier. | `/Users/yuema137/mineworld-demos/physics-spike/src/resolve.rs` | All geometry outside Rapier is integer millimetres. Lengths and directions use `i64::isqrt` and `i128` products (SD-B5). Rapier is handed only literals and values converted from integers. |
| **F-B11** | **The cause of F-P1 is upstream behaviour, not a regression.**<br>- Rapier's v0.35.0 changelog says: "`CollisionPipeline::step` now clears the rigid-bodies' modified flags, so a collision-only pipeline no longer accumulates stale change tracking".<br>- 0.36.0 is the newest version on crates.io (2026-09-25). The changelog's `v0.36.1` heading (2026-10-04, Python bindings only) is not published. | crates.io API and the dimforge `CHANGELOG.md`, read 2026-10-07 (§17.3 DEP facts) | The workaround stays. 12b has no dynamic body, but it does carry the workaround and its regression test (PB-3), so 12c starts from a guarded adapter. |
| **F-B12** | **The dev profile is `opt-level = 1` for every crate, Rapier included.** The prototype measured R′ in release at 61.0 µs per request. It measured the dev profile only for `ql`: 72.0 µs against 58.6 µs in release. | root `Cargo.toml` `[profile.dev]` (ARC-30); §9.4 PC-f | `mineworld run` and every test run in the dev profile. So the 300-day cost bound (PB-14(b)) is measured there. Expect about 1.2× the release cost. QP-9 **[OM]** names the contingent remedy. |
| **F-B13** | **Who stands in a place can be read without a new relation.**<br>- `WorldRead::components::<Presence>()` iterates every person's location in `EntityId` order.<br>- `relations_touching(place)` gives the `present-in` edges.<br>- `WorldRead::component::<C>` answers `None` for a type no installed system declared. | `kernel/src/view.rs:118–160`; `systems/presence/src/system.rs:41–57` | Bodies builds its people list from `Presence`, in `EntityId` order, which is the canonical Rapier insertion order (DC-2). In a world without bodies, `component::<PlaceShape>` is `None`, which is the inert rule (SD-B2). |
| **F-B14** | **After 12b, every host process registers `[bodies]`.** That includes `mineworld run`, the server, the CLI tests and persistence's `kill_and_resume` (it links the installed set, F-58). Nothing installs every `AVAILABLE` capability by hand. Each test that registers a synthetic list (12a's three files) never composes through `worldpack`. | `git grep AVAILABLE`; `worldpack/src/load.rs:192–202`; the 12a files of SD-R12 | Every existing world is inert to bodies. A test that installs `BodiesSystem` into a hand-built world must register `[bodies]` first, or `require_registered` panics (by design, ARC-39 item 7). Bodies' own test files each register once. |

## 17.3 Design (SD-B1 … SD-B16)

### 17.3.1 Constants (bodies' policy, published like `MAX_STRIDE`)

```text
PERSON_RADIUS     300 mm      the 3D client's capsule (player.gd); every person in 12b (QP-6)
PERSON_HEIGHT   1 720 mm      ditto; Rapier capsule_z(half-segment 560 mm, radius 300 mm), its centre
                              870 mm above the floor (feet 10 mm up, as the prototype)
GAP                10 mm      the character controller's offset; a stopped walker ends 2R + GAP =
                              610 mm from what stopped it
CLEARANCE         595 mm      the integer invariant: no two people in one place closer (2R − 5 mm);
                              also the genesis bound (SD-B4)
TOLERANCE           5 mm      a centre may come this close to the floor's edge shrunk by R, or to a
                              solid grown by R, and no closer
NUDGE_MAX         300 mm      per stride, on top of the overlap the stride creates; a nudge is at most
                              NUDGE_MAX + GAP = 310 mm
CHAIN_MAX           2         generations of nudges per stride
NUDGED_MAX          4         people moved by one stride
HALVINGS            8         verify-then-degrade: the blocked advance is halved at most this often
BIAS_BAND         200 mm      QB-16: a person met within this distance of the walker's line counts as
                              head-on
BIAS_TURN       (4, 1)        QB-16: the walker's stride is turned right by atan(1/4) ≈ 14.04°,
                              computed as (4·d + right(d)) / √17 in integers (SD-B10)
SNAP                1 mm      a sweep ending within this of its target reached the target exactly
LATTICE            50 mm      entry placement's search lattice (SD-B8)
CAPACITY_GRID     650 mm      = 13 × LATTICE; the sub-lattice the capacity check counts (SD-B4)
COORDINATE_BOUND 100 000 mm   no authored body coordinate beyond ±100 m: f32's step there is
                              under 0.01 mm
```

### 17.3.2 Decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-B1** | **The pack, and where Rapier lives.** Crate `mineworld-bodies` at `systems/bodies`, `SystemId "bodies"`, `VERSION 1`. Modules:<br>- `section.rs`: the `body:` section;<br>- `component.rs`: `PlaceShape`;<br>- `event.rs`: `PlaceShaped`;<br>- `system.rs`: the declaration, install, the reduction with its genesis checks, perception, `SystemPack`;<br>- `geometry.rs`: integer geometry, free-point tests, the lattice, capacity, verification;<br>- `resolve.rs`: the `ArrivalResolver`, i.e. the order of SD-B6 … SD-B10;<br>- `rapier.rs`: **the only file that names `rapier3d`**;<br>- `codec.rs`;<br>- `lib.rs`: the doc table.<br>`rapier.rs` takes integers in and gives integers out: a place's shape, the people in it, and one sweep or nudge request. It returns quantized millimetres. | `DEP-13`'s isolating interface (§15.1). I-5: nothing of Rapier survives a call. A structural test holds the boundary (PB-16). |
| **SD-B2** | **State, and the inert rule.** Bodies owns one component, `PlaceShape` (on a Place). A person's body is not stored: in 12b every person standing in a shaped place, with a local position, is the default capsule at that position (QP-6). The resolver returns `so_far` unchanged unless both hold: `to` has a local position, and `to`'s place has a `PlaceShape`. A person whose `Presence` has no local position has no body, and other people's resolutions ignore them. People do have heights, but every person stands on the floor: z is kept and never used for collision. | I-13, ARC-39 item 1: inert where its own state is absent. In a world without bodies the table does not exist and the lookup is `None` (F-B13). The 3D slice reports no height (`REPORT_HEIGHT = false`). |
| **SD-B3** | **The `body:` section, on place files** (`ARC-31`; QP-5). `CARRIED_BY = [Place]` in 12b. Its type is its validation, and it denies unknown fields:<br>`floor: { min: {x, y}, max: {x, y} }` is the walkable rectangle in the place's frame. Each side is at least 620 mm (2R + 2·GAP).<br>`solids: [ { min: {x, y}, max: {x, y}, height } ]` is optional, at most 64. Each box is non-empty, and `height` is in 1 … 10 000.<br>Every coordinate is within `COORDINATE_BOUND`. No references. `seed` states one `place-shaped { place, shape }`. | The place's walls are the floor's edge. A doorway needs no gap, because crossing between places is a placement (SD-B8), not a sweep across frames (§5). For 12e the mapping is direct: `MineWorldSpace.to_3d` maps (x, y, z) mm → Godot (x, z, −y) m, from the place's origin. An axis-aligned floor and box stay axis-aligned. 12e builds the perimeter walls with gaps at the disclosed passages, and the solids as static boxes. |
| **SD-B4** | **Genesis checks, in bodies' reduction of `place-shaped`** (QR-7, F-B4). Each check refuses with `FactRefusedByOwner { system: bodies, event_type: place-shaped, reason: System { code, detail } }`, and the detail names the people, the place and the numbers:<br>- `bodies-overlap`: two people located in the place are closer than `CLEARANCE`;<br>- `bodies-outside`: a person's centre is outside the floor shrunk by R;<br>- `bodies-in-solid`: a person's centre is closer than R to a solid;<br>- `bodies-capacity`: the place cannot hold the world's population. Count the capacity-grid points, anchored at `floor.min + (R, R)`, that lie inside the floor shrunk by R and at least R from every solid. The count must be ≥ 4 · (people in the world − 1) + 1.<br>The checks use integers only. A `PlaceShape` is written only if every check passes. | Each other person blocks at most four capacity points: a disc of radius 610 spans less than two 650 mm steps. So entry placement always finds a free point (SD-B8). That turns "a resolver cannot refuse" (F-B3) into a guarantee rather than a hope. Genesis checks surface through `mineworld validate`. |
| **SD-B5** | **Integer geometry** (DC-3, F-B10). Everything outside `rapier.rs` uses `i32` positions, with `i64`/`i128` products and `i64::isqrt`:<br>- distances, compared squared;<br>- a point against the floor and the solids;<br>- a point against a segment, for the corridor;<br>- nudge directions and lengths, with the length rounded up so the separation is reached;<br>- the bias turn, the lattice and the verification.<br>Rapier is given metres converted from integers by one function, and returns metres quantized by one function, `(m × 1000).round() as i32` (DC-6). No `sin`, `cos` or `sqrt` from std anywhere in the pack. | DC-3, DC-6. The integer side is exactly reproducible, and only the sweep is float. |
| **SD-B6** | **A stride within the place** (the walker's `from` is in `to`'s place, both with local positions), in this order:<br>1. **Guard** (SD-B9).<br>2. **Fast path.** If the corridor is clear, the resolution is unchanged and Rapier is not used. Clear means: the centre's segment from `from` to `to` stays inside the floor shrunk by R + GAP; the segment's box, grown by R + GAP, meets no solid's box; and every other person is at least 2R + GAP from the segment (exact, `i128`).<br>3. **Head-on bias** (SD-B10) may turn the target.<br>4. **R′ steps 1–5** (§4.5.1, as §9.6 states them):<br>&nbsp;&nbsp;- walls-only reach W;<br>&nbsp;&nbsp;- contact reach B, people solid;<br>&nbsp;&nbsp;- candidate: W if \|W\| ≤ \|B\| + NUDGE_MAX, else the walls-only path cut at \|B\| + NUDGE_MAX;<br>&nbsp;&nbsp;- the nudge pass (SD-B7);<br>&nbsp;&nbsp;- on failure, blocked at B with nobody displaced.<br>5. **Quantize, then snap.** A walker within SNAP of its target is at the target exactly. A nudged person within SNAP of their nudge's end is at that end.<br>6. **Verify, then degrade** (DC-8, I-12), on integers:<br>&nbsp;&nbsp;- V1: every pair in the place is at least CLEARANCE apart;<br>&nbsp;&nbsp;- V2: every moved centre is inside the floor shrunk by R − TOLERANCE;<br>&nbsp;&nbsp;- V3: every moved centre is at least R − TOLERANCE from every solid.<br>&nbsp;&nbsp;If any check fails, try blocked; then the advance along `from → B` halved, `from + (B − from) / 2^k` for k = 1 … HALVINGS; then stay, with `reached` = `from`'s position and `to`'s facing.<br>7. **`stopped_by`**: the first person the walker touched, by contact order, then `EntityId`. `None` when walls or a solid stopped them, as §4.7 says. | §4.5 as the prototype measured it, plus four things the seam makes necessary. The snap is F-B9: without it, rounding can "lengthen" a clean stride by 1 mm and presence would refuse it. The integer fast path is designed in §6.4 and is what makes the cost bound reachable. It is the definition of the clear case, not an approximation of Rapier's answer. Stay is valid by induction: the starting state passed V1–V3 (SD-B4, SD-B9). |
| **SD-B7** | **The nudge pass** (QB-10, I-11).<br>- Generation 1's pushers are the walker at the candidate. Generation g's pushers are the people nudged in g − 1.<br>- In each generation, every person not yet moved, in `EntityId` order, who is closer than 2R to a pusher is moved straight away from that pusher's centre. The distance is just enough to reach 2R + GAP, computed in integers. When two centres coincide, the walker's stride direction is used.<br>- Each nudge is a Rapier sweep against fixed geometry only.<br>- The pass fails if a needed nudge exceeds NUDGE_MAX + GAP, if the walls cut a nudge more than SNAP short, if more than NUDGED_MAX people would move, or if any pair is still closer than 2R − TOLERANCE after CHAIN_MAX generations.<br>- `displaced` lists the nudged people in the order they were moved. Each keeps their z and their facing. | §4.5.1 step 4 and the prototype's `resolve_people`, with integer directions. A nudge never changes place (the resolution names only `to`'s place, and presence's rule (e) re-checks it). A nudge never passes through fixed geometry: it is a shape cast against it, and V2/V3 check the end. |
| **SD-B8** | **Entry placement**, for an arrival that comes from another place, from nowhere, or from a position-less presence in this place. This is the case F-B3 forces, and §5's crossing rule restated (QP-7). In order:<br>- **E1:** if a capsule at `to` is free, the resolution is unchanged. Free means inside the floor shrunk by R, at least R from every solid, and at least 2R + GAP from everyone.<br>- **E2:** if only people make `to` unfree, run the nudge pass (SD-B7) with the arriving person as generation 1's pusher at `to`, under the same bounds. A coincident centre is pushed +x. If the result passes V1–V3, the person ends at `to` and the nudged people are displaced.<br>- **E3:** otherwise, the free point (E1's test) nearest `to` on the place's LATTICE, anchored at `floor.min + (R, R)`, ordered by (distance² to `to`, y, x). No one is displaced. `stopped_by` is the lowest-`EntityId` person whose disc covers `to`; `None` when a solid or the floor's edge does. | A crossing must end in the destination place (rule (a)). E3 always finds a point, because the capacity grid is a sub-lattice of LATTICE and SD-B4 leaves at least one capacity point free. E3's search is bounded by the floor's area over 2 500 mm², and runs only when E1 and E2 both fail. Rule (c) does not bound a crossing (ARC-39's limitation): E3 may place someone farther from the doorway point than `to`. QP-7 asks whether that is acceptable. |
| **SD-B9** | **The guard, replacing the per-arrival reaction guard** (F-B1, QP-1 **[OM]**). At the start of every resolution that is not inert, the resolver checks the place's *current* state: every pair of positioned people at least CLEARANCE apart, and each inside the floor and out of solids (V2, V3). Every resolved arrival keeps that state, and genesis checked it. So a violation means an arrival escaped resolution: a constructor bypass (ARC-39 limitation, F-R11), or a host that registered nothing. The resolver then **panics**, naming the pair (or the person), the place, the distance and `ARC-39`. | ARC-39 item 7 bullet 3, made buildable. A reaction cannot see a whole emission list (F-B1). The state a resolution starts from is complete. It panics because a resolver returns a `Resolution` and has no error path, and `require_registered` already set that precedent for host defects (ARC-39 item 7). The check is O(n²) for the n people in one place. |
| **SD-B10** | **QB-16 — the head-on bias.**<br>- Let d = to − from in the plane, and right(d) = (d.y, −d.x), the walker's right with z up.<br>- The *first person met* is, among the other positioned people p in the place, the one with the smallest `along = d·(p − from) > 0` whose `cross = d × (p − from)` satisfies cross² < (2R + GAP)² · \|d\|², and with `along ≤ |d|² + (2R + GAP)·|d|`. Ties go to `EntityId`.<br>- If such a person exists and cross² < BIAS_BAND² · \|d\|², the target becomes `from + trunc((4·d + right(d)) · 1 000 / 4 124)` per axis, with z = to.z. 4 124 = ⌈√17 · 1 000⌉, so the turned stride is never longer than d (rule (c)).<br>- The turned target then goes through SD-B6 step 4 like any other.<br>- If it is reached, `reached ≠ to`, and presence records `stopped-short { wanted: to, reached, by: that person }`. | F-P7: walkers meeting exactly head-on push each other straight back forever. Turning right lets them clear each other at an angle: the met person is then hit off-centre, so their nudge has a sideways part too. Turning both walkers to their own right is the "keep right" rule, so their sideways moves add up instead of cancelling. The turn changes only the walker's own path. Nudges stay "straight away", so I-11's 310 mm bound holds unchanged, which was not true of a biased nudge: computed for a 300 mm overlap, a 26.6° biased nudge needs 327 mm. The criterion is fixed before measuring (PB-13), with a pre-declared fallback ladder. |
| **SD-B11** | **The Rapier adapter** (`rapier.rs`). Each call builds a fresh `PhysicsWorld` for one place, inserting in canonical order (DC-2): the floor slab, the four perimeter cuboids (west, east, south, north; 200 mm thick, 3 000 mm high), the solids in authored order, then the people by `EntityId` as kinematic position-based capsules.<br>It then calls `detect_collisions`, followed by the F-P1 re-mark of every dynamic body (none in 12b; kept for 12c, PB-3).<br>The character controller is the prototype's: up Z, offset 10 mm, slide on, no autostep, no snap to ground, `dt` 1/60 s. The filters are `only_fixed` for walls-only sweeps and nudges, and `exclude_dynamic` for people-solid sweeps; the moving body is always excluded. There is no `step` in 12b. Nothing is cached. | The prototype's `build`, `refresh_queries`, `controller` and `sweep`, which measured R′ (§9.5, §9.8). |
| **SD-B12** | **Declaration, install, perception.**<br>- `depending_on([presence])`, `owning::<PlaceShape>()`, `emitting::<PlaceShaped>()`, `subscribing_to::<PlaceShaped>()`. No action, and no foreign vocabulary: bodies states no `arrived`, because only movers do.<br>- `install`: `require_registered(&Self::ID)` first, then the table.<br>- `impl SystemPack` with `owns_section!()`, and `BIOGRAPHICAL` empty.<br>- `PerceptionProvider::discloses`: a place's `PlaceShape`, to whoever perceives the place, as movement discloses `Passages`.<br>- `impl ArrivalResolver for BodiesSystem`, so the installed set lists the system type itself. | ARC-39 items 1 and 7; ARC-33; R-B4 (the client builds the server's geometry from one source). A pack that only resolves states nothing in presence's vocabulary, so it needs no `emitting::<Arrived>`. |
| **SD-B13** | **The integration world, `worlds/bodies-yard`** (QP-4). It is a real World Pack: systems `presence, movement, conversation, bodies`; twelve people, all seats.<br>- `hall`: floor (0, 0)–(12 000, 9 000), a table solid (5 000, 4 000)–(7 000, 5 000) of height 750.<br>- `court`: floor (0, 0)–(10 000, 10 000), a pillar (4 700, 4 700)–(5 300, 5 300) of height 3 000.<br>- One passage, hall (11 600, 4 500) ↔ court (400, 5 000). Both points are free for a capsule.<br>- Six people in each room, at least 1 m apart. People files carry `location` and tags only.<br>- A short `README.md`. | It runs through the real loader and `body:` sections, `mineworld validate`, `mineworld run` with the unchanged paced controller, `--save`, SIGKILL and resume, and the established test-time copy for removing a pack (`market_composition.rs`). A hand-built world exercises none of these. It stays the pack's fixture for 12c, and it is a world the operator can run. Conversation is included because the controller's social behaviour is what moves people toward each other. A crossing through a doorway onto an occupied point (F-B6) is exercised constantly. |
| **SD-B14** | **`DEP-13`, and how Rapier is declared.** In the root `[workspace.dependencies]`: `rapier3d = { version = "=0.36.0", features = ["enhanced-determinism"] }`. That means the default features (`dim3`, `f32`, `std`) plus enhanced determinism, without `simd8`, `parallel` or `serde-serialize`. Bodies declares `rapier3d = { workspace = true }`. The facts the record states are in §17.3.3. | QP-3 **[OM]**, F-B8. `serde-serialize` is off because nothing of Rapier is persisted (I-5). `parallel` is off: the documentation says it is bitwise equal to sequential, but 12b runs no solver, only queries, and rayon threads in a single-threaded host buy nothing (QP-10). |
| **SD-B15** | **An earlier resolver.** Bodies is asked in `SystemId` order, and in this build it is the only resolver. If `so_far` is not the unchanged resolution — another resolver has already moved the arrival or someone else — bodies returns `so_far` unchanged. Any overlap that results is caught by the next resolution's guard (SD-B9). | R-B11. Composing two geometric resolvers is out of MVP-0. Saying so beats guessing a composition rule. |
| **SD-B16** | **Documents.**<br>- `DEP-13`: §15.1's draft made exact by §17.3.3.<br>- A dated **note on ARC-39**: the first resolver's bounds (QR-11), the guard as SD-B9 (QP-1), and entry placement as SD-B8 (QP-7).<br>- `MODULE_SPEC.md` §4.1: the `body` section row, the place example, "seven sections".<br>- `systems/bodies/README.md`, for humans, and `lib.rs`'s table.<br>- `MVP_STATUS.md`. | `CLAUDE.md` §2.2: the documents come first (PB-C1). |

### 17.3.3 `DEP-13` facts, re-verified 2026-10-07

Two sources, read on the day: the crate source in the local Cargo registry (`rapier3d-0.36.0/Cargo.toml`,
`parry3d-0.31.1`), and the primary pages, fetched by a web agent.

```text
rapier3d     newest non-yanked on crates.io: 0.36.0, published 2026-09-25 (sebcrozet); 71 versions.
             The dimforge CHANGELOG's top heading is v0.36.1 (2026-10-04, Python bindings only),
             which is NOT on crates.io. Pin: =0.36.0.
licence      Apache-2.0 (crates.io field and the crate's Cargo.toml; LICENSE: "Apache License
             Version 2.0", "Copyright 2020 Sébastien Crozet"). Compatible with MineWorld's MIT.
MSRV         rust-version 1.86, edition 2024 (crate Cargo.toml and crates.io). Ours: 1.97.1.
features     default = [dim3, f32, std]
             enhanced-determinism = [simba/libm_force, parry3d/enhanced-determinism]
             parallel = [dep:rayon, std, parry3d/parallel]
             simd8 = [parry3d/simd8]  — the only SIMD feature in 0.36.0
             serde-serialize = [arrayvec/serde, nalgebra/serde-serialize, parry3d/serde-serialize,
                               dep:serde, std]
parry3d      ^0.31.1 required; 0.31.1 newest (2026-09-18), Apache-2.0, no MSRV declared; 0.31.0 yanked
determinism  (rapier.rs, Rust guide 0.36) cross-platform needs `enhanced-determinism` and IEEE 754-2008
             targets; "`enhanced-determinism` feature cannot be enabled at the same time as the
             `simd8` feature"; "The `parallel` feature, on the other hand, can be combined with it",
             "the results of the parallel solver are identical to the results of the sequential
             one"; inputs computed with functions beyond + − × ÷ must use nalgebra's
             ComplexField/RealField
F-P1 cause   CHANGELOG v0.35.0: "CollisionPipeline::step now clears the rigid-bodies' modified
             flags, so a collision-only pipeline no longer accumulates stale change tracking"
             (issue #662); v0.36.0 added PhysicsWorld::detect_collisions. Nothing since mentions
             the interaction. Workaround kept (PB-3).
licences of  cargo tree on the prototype (rapier3d's normal dependencies, with serde-serialize on,
the tree     a superset of 12b's): Apache-2.0; MIT; MIT OR Apache-2.0; Zlib; Zlib OR Apache-2.0 OR
             MIT; Unlicense OR MIT; (MIT OR Apache-2.0) AND Unicode-3.0. All permissive. Re-run on
             12b's own lock, with 12b's features, in PB-C2 (PB-2).
cross-arch   PC-g (§9.8): arm64 and x86_64-under-Rosetta give identical digests, and snapshots resume
             across them. The x86_64-apple-darwin target is installed on this machine (rustup target
             list --installed, 2026-10-07). PB-12 repeats the check with the real pack.
```

## 17.4 Acceptance (decided before measuring, `ARC-23`)

Rules for every criterion below:
- Each guarded criterion names the mutation shown to break it. A mutation is applied in the working
  tree, observed to fail by name, and reverted. `git status` and `git grep MUTATION` are recorded
  afterwards.
- Every expected position is a literal from the test's own layout, never computed by the code under
  test (test rules §25).
- Positions that pass through Rapier are asserted to within ±1 mm of a hand-computed literal. An
  example: a centre stopped by the east wall at x = 8 320 − 300 − 10 = 8 010.
- The scenario room is the prototype's café: floor (0, 0)–(8 320, 10 320), with the counter (3 860,
  6 570)–(8 320, 7 170) at height 1 100. The prototype's numbers are therefore comparable.

```text
PB-1  Nothing existing moves. On the PR head, with bodies installed and [bodies] registered by every
      composing host:
        - `mineworld run worlds/social-cafe --headless --seed 7 --days 300`: faults 0, 365 330 facts,
          sha-256 of every line but `wall` = ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b;
        - `mineworld run worlds/market-town --headless --seed 7 --days 300`: faults 0, 372 755 facts,
          sha-256 = 365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d;
        - `mineworld validate` of both worlds byte-identical to the base binary's output;
        - every existing test passes; the only edits to existing tests are QP-2's two files.
      M-PB1  the inert rule dropped (a place without PlaceShape treated as an unbounded floor, so
             people are nudged anywhere) → PB-19 fails, and the 300-day social-cafe sha differs from
             the above, with `stopped-short` appearing in its summary.
PB-2  DEP-13 and the pin. DEP-13 is committed before any code (PB-C1), and both doc checks pass.
      `systems/bodies/tests/rapier_pin.rs`:
        - Cargo.lock holds exactly one rapier3d, at 0.36.0, and one parry3d, at 0.31.1;
        - `cargo metadata`'s resolved features for rapier3d include `enhanced-determinism` and exclude
          `simd8`, `parallel` and `serde-serialize`;
        - (BodiesSystem::VERSION, the locked rapier3d version) = (1, "0.36.0"), so an upgrade must move
          both together (DC-5).
      Recorded, not a test: `cargo tree -p rapier3d -e normal -f "{p} {l}"` on 12b's lock, with each
      licence in §17.3.3's permissive set.
      M-PB2  `parallel` added to the root line → rapier_pin fails, naming `parallel`.
PB-3  F-P1, as unit tests in rapier.rs:
        - (canary) a fresh world with a 0.4 m dynamic cube at z 0.800 m above the floor;
          `detect_collisions` without the re-mark; 20 steps of 1/60 s → z is still 0.800 ± 0.001.
          The defect is present in the pin. If this ever fails, upstream changed it, and the workaround
          is removed deliberately;
        - (fix) the adapter's refresh, then 20 steps → z ≤ 0.300 m (the prototype measured 0.291).
      M-PB3  the re-mark removed from the refresh → (fix) fails.
PB-4  Authored people are checked at load (QR-7, SD-B4), through the real `mineworld validate` on
      test-time copies of worlds/bodies-yard (tools/cli/tests/bodies_yard.rs). Each refusal exits
      non-zero and names its subject:
        - two hall people 420 mm apart → "bodies-overlap", both people, "hall", "420 mm";
        - a person whose centre is 100 mm inside the table → "bodies-in-solid";
        - a person 200 mm from the floor's edge → "bodies-outside";
        - a 1 300 × 1 300 mm hall in the 12-person world → "bodies-capacity", with the counts;
        - `floor` with min.x ≥ max.x, an unknown key `wall:`, and a solid of height 0 → each refused
          by the loader at its line and column, with bodies' own message.
      Positive control: worlds/bodies-yard itself validates.
      M-PB4  the pair check removed → the overlap copy validates.
PB-5  Walls (systems/bodies/tests/scenarios.rs: a hand-built world with presence, movement and bodies,
      through the real World::dispatch, [bodies] registered). Each case states the facts in order,
      each caused by the walker's ActionId and emitted_by movement:
        a  east from (7 000, 3 000) to (8 500, 3 000) → arrived (8 010 ± 1, 3 000);
           stopped-short { wanted (8 500, 3 000), reached, by None }
        b  north from (5 000, 5 500) to (5 000, 7 000) → arrived (5 000, 6 260 ± 1), against the
           counter (6 570 − 310); stopped-short, by None
        c  diagonal from (7 000, 3 000) to (8 400, 4 400) → x = 8 010 ± 1 and y > 4 010 (it slid);
           |reached − from|² ≤ |to − from|²
        d  a stride meeting nothing → exactly `to`, one fact, payload bytes equal to
           Arrived::new(person, to)'s encoding (the fast path)
PB-6  Nudging (QB-10, I-11), same file:
        - n2: the walker goes from (2 000, 5 000) in 12 strides of +500 mm in x; B stands at (4 000,
          5 100). B moves at least 100 mm in all. Every nudge is at most 310 mm, recorded as
          `arrived { B }` in the walker's emission list and caused by the walker's ActionId. After
          every request the pair is at least 595 mm apart.
        - n3: the crowd of §9.6. At most 4 displaced per request. At least one stride is blocked. The
          closest pair is at least 595 mm. Nobody is outside the floor.
        - a chain that needs a third generation is blocked: the walker ends at contact (± 1 mm) with
          the first person; nobody else moves; stopped-short names the first person.
        - Each case's layout keeps the first person met outside BIAS_BAND, so the bias (PB-C5) does
          not change it.
      Generations are read from the resolver core's outcome in the same tests.
      M-PB5  CHAIN_MAX 2 → 3 → the chain case fails: the third person moved.
      M-PB6  NUDGE_MAX 300 → 400 → n2's bound fails, naming the nudge's length.
PB-7  Never through a wall, never out of the place:
        - B stands with its centre 320 mm from the east wall, and the walker comes from the west. B
          can move only 10 mm, so the stride is blocked: the walker is at contact, B is unchanged, and
          stopped-short names B.
        - The same holds against the counter.
        - Every displaced `arrived` names the walker's place.
PB-8  Verify-then-degrade reproduces F-P6 (unit test in resolve.rs, under a test policy with the bias
      off). The setup is R's request 551: the walker at (2 536, 782) asks for (+1 412, −1 412), and a
      person stands at (2 970, 314) against the south wall. With verification on, the closest pair is
      at least 595 mm and the degradation level is recorded. With the production policy (bias on),
      the same start also passes V1–V3.
      M-PB7  verification off → the overlap reappears (the prototype: 33 mm), naming the pair.
      If two people alone do not reproduce the slide in the pack, the session first rebuilds the
      prototype's full state before request 551 (15 people, its request sequence). If that still
      cannot reproduce it, M-PB7 is INCONCLUSIVE and is replaced by a unit test of verification on a
      fabricated overlapping candidate, recorded as a deviation.
PB-9  The 30-day real run (tools/cli/tests/bodies_yard.rs): `mineworld run worlds/bodies-yard --headless
      --seed 7 --days 30 --save S` → exit 0, faults 0.
      Activity first (ARC-23), per 10-day bucket: every seat moved; at least one stopped-short; at
      least one displaced arrival; at least one crossing in each direction. The counts are printed.
      Then the scan. It replays positions from S's facts in EventId order and checks after each
      request's facts:
        - no pair in one place is closer than 595 mm;
        - no centre is outside the floor shrunk by 295 mm, or within 295 mm of a solid (the geometry
          is the world file's literals);
        - every displaced arrival (an `arrived` after the first in one request's list) stays in its
          place and moves at most 310 mm, with at most 4 per request;
        - every stopped-short and displaced arrival is caused by the request's ActionId, with
          emitted_by movement.
      It prints the closest pair, its place and its revision (located, not counted).
PB-10 The instrument sees, and bodies can be removed (I-10, AC-2 at world level). A test-time copy of
      worlds/bodies-yard drops `bodies` from `systems` and every `body:` section (the
      market_composition pattern). It runs 30 days with faults 0, and every seat moves. The same scan
      reports a violation, naming the closest pair (< 595 mm), the place and the revision.
PB-11 Determinism (tools/cli/tests/bodies_yard_restart.rs, the run_restart pattern):
        - two processes, `--save A` and `--save B`, for 30 days: facts, journal and snapshots
          byte-identical;
        - SIGKILL at days 5, 15 and 25, then the same command: each survivor reports the head it
          resumed from, with a re-executed tail > 0 at least once, and is byte-identical to A;
        - verify from genesis passes.
      Exclusions, as in run_restart: the victim's status is a signal, it printed no summary, and its
      head is short of A's.
      M-PB8  an impure resolver (a process-global counter's parity added to GAP) → a survivor is
             refused (ReplayDiverged), or its rows differ.
PB-12 Cross-architecture (recorded real evidence, Gate 2's role; no committed test). An
      x86_64-apple-darwin build of `mineworld`, run under Rosetta (`arch -x86_64`):
        - the 30-day bodies-yard run's summary sha-256 (every line but `wall`) equals arm64's;
        - an arm64 save written to day 15, resumed by the x86_64 binary to day 30 → the same sha as
          arm64's uninterrupted 30-day run, and it reports resuming at the save's head;
        - the reverse: an x86_64 save at day 15, resumed by arm64.
      If the CLI cannot be cross-built (rusqlite's bundled C), the fallback is the pack's
      scenarios and long_run test binaries built for x86_64 and run under Rosetta, with their
      printed digests compared. That is recorded as PARTIAL, never as PASS.
PB-13 QB-16, the head-on bias (decided now, measured in PB-C5). Scenario n1, in the café room: A at
      (2 000, 5 000), B at (6 320, 5 000). They alternate requests: A asks +500 mm in x from where
      presence says A stands, B asks −500 mm. Twelve requests each.
        HB-1  after the 24 requests, A.x > B.x: they have passed each other;
        HB-2  after every request: the pair is at least 595 mm apart, every nudge is at most 310 mm,
              and both centres are inside the floor shrunk by 295 mm;
        HB-3  n2 and n3 still meet PB-6, and in n2 the walker ends east of B;
        HB-4  n1–n3's final states and fact digests are byte-identical in two processes.
      M-PB9  BIAS_BAND = 0 → HB-1 fails: A.x < B.x at the end (F-P7 reappears).
      If HB-1 fails with the frozen constants, a ladder is fixed now. It is recorded as a bounded
      deviation, and the criterion does not change:
        BIAS_TURN (2, 1), i.e. 26.6°; then BIAS_BAND 300 mm; then both.
      If none passes, that is a material stop: QB-16 returns to the operator with the traces.
PB-14 Cost (QB-11; recorded real evidence).
        a  systems/bodies/tests/long_run.rs: the prototype's 3 000-request sequence without props (the
           café room, 15 people), through World::dispatch. A committed test: N-1 … N-4 hold after
           every request, the closest pair is located, and the final digest is the same in two
           processes. Its release run (`cargo test --release -p mineworld-bodies --test long_run --
           --nocapture`) prints the mean time per move that reaches Rapier, and the fast-path share.
           Target ≤ 61 µs. PASS ≤ 100 µs (PC-d's budget). FAIL above it.
        b  300 days of bodies-yard, seed 7, in the dev profile, as `mineworld run` is measured
           (ARC-30): with bodies, and as PB-10's copy without, two runs each, consecutive, on one
           machine. PASS iff max(with) ≤ 1.5 × min(without), and faults are 0 in all four.
           If it fails: QP-9's remedy if the operator approved it at freeze, otherwise a material stop
           with the numbers.
PB-15 Structural (systems/bodies/tests/isolation.rs; and the existing scans):
        - in systems/bodies/src, only rapier.rs names `rapier`;
        - no code file (*.rs, Cargo.toml) outside systems/bodies names `rapier3d`, except the root
          Cargo.toml's one dependency line;
        - component.rs, event.rs and section.rs hold no f32 or f64 (I-3);
        - bodies' dependencies name no System Pack other than presence;
        - seam_vocabulary passes with QP-2's admission, and that admission fails if unused;
        - ac1_composability 13/13 and precursor_vocabulary 4/4 pass unedited;
        - presence's and movement's own structural scans pass unedited.
      Planted: `use rapier3d as _;` in resolve.rs → the first bullet fails, naming the file. Removed.
PB-16 Disclosure (pack test): an observer in the hall is told the hall's `place-shape`, i.e. the
      floor and solids as authored literals. The court's shape is not in that observation.
PB-17 The guard (SD-B9, QP-1). A test-only stating pack in bodies' tests states `Arrived::new` directly,
      bypassing `arrivals` (F-R11), and so puts bob 200 mm from alice in a shaped place. The next
      `move` into that place panics, and the message names alice, bob, the place, "200 mm" and ARC-39.
      M-PB10 the guard removed → no panic.
PB-18 Entry placement (SD-B8), each case through movement's crossing in a hand-built two-place world:
        a  onto a free doorway point → exactly there; no stopped-short;
        b  onto a point occupied by one person → that person is nudged ≤ 310 mm, and the crosser is
           at the point;
        c  onto a point where the nudge fails (the occupant is backed against the floor's edge) →
           the crosser is at the nearest free lattice point (a literal from the layout), with
           stopped-short { wanted, reached, by: the occupant };
        d  onto a `to` inside a solid → the nearest free point, by None.
      After each, every pair is at least 595 mm apart.
      M-PB11 E3 returns `to` → case c fails, naming the pair.
PB-19 Inert where absent (I-13). In a world with bodies installed:
        - every `move` into a place without `body:` records exactly Arrived::new's bytes;
        - so does a `move` whose `to` has no local position.
      This is checked across the 12-move script of 12a's RS-7, so the two seams agree.
PB-20 Scope and the gate:
        - `git diff --name-only <base>...HEAD` ⊆ §17.1's change set;
        - the no-diff paths are empty;
        - Cargo.lock adds packages and changes no existing package's version;
        - `cargo fmt --check` and `cargo clippy --workspace --all-targets --all-features -D warnings`
          are clean;
        - the full workspace gate runs once, on the final executable head.
```

## 17.5 Commit plan

Rules for every commit:
- Each commit tracks implementation, validation and review separately.
- Evidence goes into §17.10 as `E-PB<n>`, deviations into §17.11.
- A planned commit may become several coherent commits; the mapping is recorded.
- Each commit leaves the workspace's tests green.
- Commands run from the worktree root, with `$HOME/.cargo/bin/cargo` if `cargo` is not on PATH.
- Anything longer than about two minutes runs in the background: the x86_64 build, 300-day runs in
  pairs, and the full gate.

### PB-C0 — Design (this section) — docs only

- [x] Implementation: §17 and the header line, by the planning session on `mvp0/s15-12b-plan`, from
  the audit in §17.2.
- [x] Validation: `python3 scripts/check_doc_headings.py` and `python3 scripts/check_decision_ids.py`
  (E-PB0). The DEP-13 facts were re-verified at source on the day (§17.3.3). DEP-13 and ARC-40 are
  free on every `origin/*` branch.
- [x] Review: every claim in §17.2 cites a file and a line, a command, or a measurement. Each
  departure from §§4.5, 5, 10 is named with its finding and its question. The operator-material
  questions are marked. This is the planning session's self-review only; the operator's review is
  pending.

### PB-C1 — Specs before code: DEP-13, the ARC-39 note, MODULE_SPEC §4.1

**Goal.** The dependency and the first resolver's rule are reviewable decisions before any code
relies on them (`CLAUDE.md` §2.2; the operator's binding "DEP-13 within 12b, before any code").

**Scope.**
- `docs/DECISIONS.md`: append **DEP-13** after ARC-39. It is §15.1's draft made exact by §17.3.3:
  - problem; options (Rapier, `parry` alone, our own, Jolt bindings, Avian);
  - choice: `=0.36.0`, `enhanced-determinism`, no `simd8`, `parallel` or `serde-serialize`, declared
    in the root workspace (QP-3);
  - why not ourselves, and why not the others;
  - the isolating interface: `systems/bodies/src/rapier.rs`, integers in and out;
  - accepted limitations: monthly breaking releases, so an upgrade bumps bodies' VERSION (DC-5);
    F-P1 and its canary; F-P6 and verify-then-degrade; cross-architecture identity evidenced under
    Rosetta only; no momentum between resolutions;
  - the licence and MSRV facts, with their date.
- `docs/DECISIONS.md`: a dated **note on ARC-39**, without rewriting it:
  - the first resolver's bounds (QR-11): NUDGE_MAX + GAP, CHAIN_MAX, NUDGED_MAX, CLEARANCE;
  - item 7 bullet 3 is realized as SD-B9's guard on the starting state, with the reason (F-B1);
  - a crossing that cannot land on its point is placed at the nearest free point (SD-B8, F-B3).
- `docs/MODULE_SPEC.md` §4.1:
  - `# places/<key>.yaml` gains a commented `body:` example;
  - the section table gains the `body` row (bodies, places: floor, solids, the bounds of SD-B3,
    disclosed to whoever perceives the place);
  - "six sections" becomes "seven".
- Before writing, run `git fetch` and confirm DEP-13 is free on every `origin/*` ref.

**Depends on:** freeze, including QP-1, QP-3 and QP-7. **Non-goals:** no code.

- [x] Implementation: as scoped, with §17.0's override: DEP-13 states the pin lives in
  `systems/bodies/Cargo.toml`, not the root (QP-3 overruled; DB-1). ARC-39 gains a dated note with
  three points (bounds; item 7 bullet 3 realized as the starting-state guard; placement of a
  crossing). MODULE_SPEC §4.1: the commented `body:` example in the place file, the `body` row,
  "seven sections". Handoff reinitialized for 12b. DEP-13 and ARC-40 confirmed free on every
  `origin/*` ref after `git fetch` (E-PB1).
- [x] Validation: both doc checks pass, decision ids 50 → 51 (E-PB1). Cited sections exist:
  REUSE_POLICY §§4, 11, 12, 17; MVP §9 (AC-8, AC-12); ARC-25, ARC-30, ARC-33. DEP-13's facts block
  is §17.3.3's lines for rapier3d, licence, MSRV, features and parry3d copied verbatim; the
  `serde-serialize` features line and the prose determinism lines are summarized, not altered.
- [x] Review: DEP-13 answers §11's six questions (problem; options (a)–(e); why this; why not ourselves;
  isolating interface `rapier.rs`; limitations) and gives §12's reason for each rejected option. The
  ARC-39 note says it refines item 7 only and leaves items 1–8 otherwise unchanged. No defined term
  redefined; `body`, `floor`, `solid` are section words.

**Commit boundary.** Documentation only.

### PB-C2 — The pack's skeleton and the Rapier adapter

**Goal.** Rapier is in the build, pinned and isolated. PB-2 and PB-3 hold before any rule exists.

**Scope.**
- Root `Cargo.toml`: one `[workspace.dependencies]` line, `rapier3d = { version = "=0.36.0",
  features = ["enhanced-determinism"] }`, with a comment naming DEP-13.
- `systems/bodies/Cargo.toml`, with each dependency's reason as a comment: authoring, contracts,
  kernel, presence, sdk, serde, serde_json, rapier3d. Dev-dependencies: movement, and whatever the
  tests need.
- `systems/bodies/src/{lib.rs, rapier.rs, codec.rs}`:
  - `rapier.rs` holds the canonical world build, `refresh` (detect_collisions plus the F-P1
    re-mark), the controller, the two sweep filters, metres from and to millimetres (one function
    each), and the PB-3 unit tests.
- `systems/bodies/tests/rapier_pin.rs` (PB-2).
- `systems/bodies/tests/isolation.rs`: PB-15's first four bullets.
- `Cargo.lock`, regenerated.
- The pack is not yet installed.

**Depends on:** PB-C1.

- [x] Implementation: as scoped, with §17.0's override (DB-1): no root-manifest edit;
  `systems/bodies/Cargo.toml` declares `rapier3d = { version = "=0.36.0", features =
  ["enhanced-determinism"] }` with its reason. `src/lib.rs` (doc table, `forbid(unsafe_code)`,
  `warn(missing_docs)`), `src/codec.rs`, `src/geometry.rs` (the published constants of §17.3.1 as
  `Millimetres`/`usize`/`u32`, and the crate-private integer `Point`, `Area`, `Room` the adapter
  takes), `src/rapier.rs` (`Scene::build` in SD-B11's order, `Scene::sweep` with `Against::{Fixed,
  FixedAndPeople}` returning the quantized end and the first person touched, `refresh`, the
  controller, `metres`/`millimetres`, and the PB-3 unit tests plus a round-trip test of the two
  conversions over ±100 000 mm). `tests/rapier_pin.rs` (PB-2's lock and features; the VERSION pair
  joins in PB-C3, when `BodiesSystem` exists), `tests/isolation.rs` (PB-15's first four bullets: the
  crate name `rapier3d` only in `rapier.rs`, nowhere outside the pack, no `f32`/`f64` in any source
  file but `rapier.rs` — widened from §17.4's three files, DB-3 — and presence the only pack
  dependency). Two `#[allow(dead_code)]` on `mod codec` and `mod rapier` until PB-C3/PB-C4 use them.
- [x] Validation (E-PB2): `cargo test -p mineworld-bodies` 3 + 4 + 1 pass; clippy `-D warnings` and fmt
  clean; `git diff Cargo.lock`: 39 packages added, no line removed, no existing version changed;
  licence tree recorded. M-PB2 and M-PB3 each fail by name and are reverted (M-PB3 as first written
  survived — DB-2); the planted `use rapier3d as _;` fails isolation naming the file, removed.
- [x] Review: no Rapier type in a `pub` or `pub(crate)` signature outside `rapier.rs` (`Scene`,
  `Swept`, `Against` carry only `Point`, `usize` and Rapier handles in private fields); `metres` and
  `millimetres` are the only float↔integer crossings; insertion order is slab, west, east, south,
  north, solids in authored order, people in the given order — SD-B11's; the canary asserts z stays
  at 800 ± 1 mm, the defect as it is.

### PB-C3 — Place geometry: the `body:` section, PlaceShape, genesis checks, disclosure

**Goal.** A place can be given a body, and a world whose authored people violate it does not load.

**Scope.**
- `systems/bodies/src/{section.rs, component.rs, event.rs, system.rs, geometry.rs}`:
  - the section type and its validation (SD-B3);
  - `PlaceShape` (component `place-shape`, schema 1);
  - `PlaceShaped` (event `place-shaped`, schema 1);
  - the declaration, install (`require_registered` first) and the reduction with SD-B4's four
    checks;
  - `discloses` (SD-B12);
  - in `geometry.rs`, the integer half SD-B4 needs: the free-point test, the capacity count, and
    pair distances.
- `systems/bodies/tests/support/mod.rs`: a hand-built world (presence, movement, bodies) that
  registers `[bodies]` first, places people through genesis, and has a dispatcher.
- `systems/bodies/tests/genesis.rs`:
  - each SD-B4 refusal in a hand-built genesis, naming its subject;
  - PB-16's disclosure through presence's `observe`.

**Depends on:** PB-C2.

- [x] Implementation: as scoped. `component.rs`: `PlaceShape { floor: Floor, solids: Vec<Solid> }`
  with `Corner`/`Floor`/`Solid` in `Millimetres`, `deny_unknown_fields`, and `try_from` an `Authored`
  form that holds SD-B3's rules (coordinate bound, sides ≥ 620, ≤ 64 solids, non-empty, height
  1 … 10 000) — the section's type is the component's. `event.rs`: `PlaceShaped`, `place_shaped()`
  (public, as movement's `passage`). `section.rs`: `body`, `CARRIED_BY = [Place]`. `system.rs`:
  `BodiesSystem` (VERSION 1), declaration and install (`require_registered` first), the reduction
  with SD-B4's checks in the order capacity → outside → in-solid → overlap (capacity first so a
  too-small room is named as such even when its people are also outside it), `discloses`.
  `geometry.rs`: `Area::holds`/`distance2`, `Room::admits`/`clear_of_solids`/`solid_within`/
  `capacity`, `lattice`, `distance2`, `closest_pair`. `resolve.rs`: `impl ArrivalResolver` returning
  `so_far` (inert until PB-C4), so the pack can be registered. `tests/support/mod.rs`, 
  `tests/genesis.rs`; `rapier_pin.rs` gains the VERSION pair.
- [x] Validation (E-PB3): `cargo test -p mineworld-bodies` → lib 3, genesis 6, isolation 4,
  rapier_pin 1. M-PB4 (pack level) fails by name, reverted. clippy and fmt clean.
- [x] Review: details name keys (`alice and bob stand 420 mm apart in room`); the capacity lattice is
  anchored at `floor.min + (300, 300)`, spacing 650, admitted at margin 300 — SD-B4's; disclosure is
  `codec::to_value` of the component as reduced, asserted against the authored literals; nothing is
  written unless `fits` returns `Ok`.

### PB-C4 — The resolver: walls, nudging, verify-then-degrade, entry placement, the guard

**Goal.** PB-5 … PB-8, PB-17 … PB-19 through the real dispatch.

**Scope.**
- `systems/bodies/src/resolve.rs`:
  - `impl ArrivalResolver for BodiesSystem`, in this order: inert rule (SD-B2) → SD-B15 → guard
    (SD-B9) → same-place (SD-B6, without the bias) or entry placement (SD-B8);
  - a crate-private core returning the resolution and an `Outcome { generations, nudged, degraded }`
    for tests;
  - a crate-private `Policy { bias, verify }`. Production uses one constant; unit tests may turn a
    flag off (PB-8).
- `geometry.rs`: the rest of SD-B5. That is the corridor and fast path, nudge vectors, the lattice
  search, and V1–V3.
- `rapier.rs`: the people sweeps and the nudge sweeps.
- `systems/bodies/tests/scenarios.rs`: PB-5, PB-6 (n2, n3, the chain), PB-7, PB-18, PB-19.
- A test-only bypassing pack for PB-17.
- PB-8 as a unit test in `resolve.rs`.

**Depends on:** PB-C3.

- [x] Implementation: as scoped. `resolve.rs`: `impl ArrivalResolver` (SD-B15 pass-through → inert
  rule → guard → `stride` or `entry`), `answer` (the crate-private core), `Policy { verify }` with
  `PRODUCTION`, `guard` (SD-B9: pair < 595 mm or a centre outside the floor/in a solid by more than
  the tolerance → panic naming both keys, the place, the distance and ARC-39), `stride` (SD-B6:
  corridor fast path → one `Scene` → `reach` (W, B, candidate) → `nudge` → verify → `degrade`),
  `nudge` (SD-B7), `entry` (SD-B8 E1/E2/E3), `relocated` (keeps z and facing), and a public
  `explain(world, person, to) -> Option<Outcome>` with `Outcome { route, generations, nudged,
  nudge_failed, degraded }` — the resolver's own account, which the tests read generations and
  degradation from (§17.4 PB-6: "read from the resolver core's outcome"; DB-7). `geometry.rs`:
  `Point` arithmetic, `scaled_down`, `at_least`, `no_longer_than`, `segment_clear_of`,
  `Room::corridor_clear`/`free_at`/`nearest_free`. `rapier.rs`: `Swept.contact`, the capsule's
  position at its first person contact (DB-4). Tests: `tests/scenarios.rs` (15), PB-8's two unit
  tests in `resolve.rs`, `Bypass` and `Moved`/`Fact` in the support.
- [x] Validation (E-PB4): `cargo test -p mineworld-bodies` → lib 5, genesis 6, isolation 4, rapier_pin
  1, scenarios 15. M-PB5, M-PB6, M-PB7, M-PB10, M-PB11 and M-PB1's pack half each fail by name and
  are reverted. Every scenario's facts are checked for order, causation (`Action(id)`),
  `emitted_by` movement and the controller decision.
- [x] Review: every degrade path ends in a state `verifies` accepted, and stay is the start the
  guard admitted. The snap moves a point onto the target and `no_longer_than` then clamps, so
  neither lengthens (rule (c)); `at_least` rounds a nudge up and the pass then refuses any nudge
  over 310 mm, so the bound holds by construction. Displaced order is pusher by pusher, then
  `EntityId` within a pusher — generation order. `isolation` holds no float outside `rapier.rs`.
  No static, cache or clock: `Scene` is built and dropped inside `stride`/`entry`. `stride` was
  split (`reach`) to stay under the ~100-line warning; resolve.rs is 700 lines with its tests.

### PB-C5 — QB-16: the head-on bias

**Goal.** PB-13. Walkers who meet head-on pass each other.

**Scope.**
- `resolve.rs` and `geometry.rs`: SD-B10 (first person met, the band, the integer turn).
- `scenarios.rs`: n1, and HB-1 … HB-4. The second process for HB-4 is the same test binary run
  twice, with its digests printed and compared.

**Depends on:** PB-C4.

- [x] Implementation: as scoped. `geometry.rs`: `BIAS_BAND` 200 mm, `BIAS_TURN` (4, 1), the
  ceiling 4 124, `first_met` (SD-B10's three conditions, exact in `i128`, ties to `EntityId`),
  `head_on`, `turned_right` (asserts the turn is never longer). `resolve.rs`: `Policy { bias, verify
  }`, the bias after the fast path and before the sweeps; a walker who reaches the turned aim is
  stopped-short `by` the person met; `Outcome.biased`. `scenarios.rs`: a shared `scenario(n1|n2|n3)`
  runner over `bounded_stride` (HB-2's checks on every request), n1 (HB-1), n2's east-of-b (HB-3),
  HB-4 as a second process of the same test binary comparing every fact and the final state byte for
  byte. PB-8 gains its production-policy case.
- [x] Validation (E-PB5): with the frozen constants, HB-1 … HB-4 PASS on the first run; the ladder of
  PB-13 was not needed. M-PB9 (`BIAS_BAND` 0) fails HB-1 by name, reverted. All bodies tests pass.
- [x] Review: the code asserts the turn never lengthens; a unit test walks every stride a `move` may
  ask on a 37 mm grid (9 000+), checking never longer, to the right, and between atan(1/5) and
  atan(1/3). Only `aim` changes; nudges are still straight away from their pusher (I-11's 310 mm
  unchanged). The chain and pinned layouts keep their first person 300 and 219 mm off the line;
  n2 and n3 do not (DB-5) and pass under the bias (HB-3).

### PB-C6 — Install: the build's first resolver

**Goal.** Every host registers `[bodies]`, and every existing world is unchanged (PB-1).

**Scope.**
- `systems/installed/Cargo.toml`: one line.
- `systems/installed/src/lib.rs`: `Bodies => mineworld_bodies::BodiesSystem,`, the resolution line
  `[mineworld_bodies::BodiesSystem,]`, and one doc sentence.
- `worldpack/tests/registration.rs`:
  - the three `Some(Vec::new())` become `Some(vec![SystemId::from_static("bodies")])`;
  - the panic message's `already registered []` becomes `[SystemId("bodies")]`;
  - the doc comment drops "empty in this build" (QP-2).
- `tests/acceptance/tests/seam_vocabulary.rs`: a second self-checking list, `INSTALLED_PACK_LINES`.
  It admits the word `bodies` only on lines of `systems/installed/src/lib.rs` that contain
  `mineworld_bodies::BodiesSystem`, with its reason (ARC-33: the installed set names its packs). It
  fails if unused, like `PRE_EXISTING` (QP-2).

**Depends on:** PB-C5.

- [x] Implementation: as scoped. `systems/installed`: the manifest line, `Bodies =>
  mineworld_bodies::BodiesSystem,`, `resolution: … => [mineworld_bodies::BodiesSystem,]`, and the doc
  paragraph reworded (it said the line was empty). `worldpack/tests/registration.rs`: the expected
  list is `[bodies]`, written once as `const LISTED`, the panic text follows it, the doc drops "empty
  in this build". `seam_vocabulary.rs`: `INSTALLED_PACK_LINES` with two self-checking entries — the
  installed set's lines naming `mineworld_bodies::BodiesSystem`, and registration.rs's `LISTED` line
  (DB-8) — joined to `PRE_EXISTING` through `admissions()`.
- [x] Validation (E-PB6): the three crates' tests pass (AC-1 13/13, the I-2 scan 4/4, seam_vocabulary
  3, registration 1, resolution 2, `[resolver-yard] PASS`); `cargo test -p mineworld-cli` 41 passed in
  18 binaries, 195 s; base binary built before any code (E-PB-base); the PR binary's two 300-day runs
  equal PB-1's digests; both `validate` outputs `cmp`-identical.
- [x] Review: `tests/resolution.rs` passes unedited (bodies is an installed pack, listed once). The two
  QP-2 edits keep their claims: registration still holds "compose registers the installed list, a
  second compose is a no-op, a different list panics naming both"; the seam scan still refuses every
  physics word on every other line. No other existing test needed an edit.

### PB-C7 — The world: worlds/bodies-yard, the real run, the counterfactual, restart

**Goal.** PB-4, PB-9, PB-10, PB-11, and PB-14(a)'s committed half.

**Scope.**
- `worlds/bodies-yard/{world.yaml, README.md, places/{hall,court}.yaml, people/*.yaml}` (SD-B13).
- `tools/cli/tests/bodies/mod.rs`: the scan, the world's geometry as literals, and the test-time copy
  without bodies.
- `tools/cli/tests/bodies_yard.rs`: PB-4 (validate refusals on copies), PB-9, PB-10.
- `tools/cli/tests/bodies_yard_restart.rs`: PB-11.
- `systems/bodies/tests/long_run.rs`: PB-14(a). The prototype's request sequence is re-derived from
  its `mix` and `draw`, as recorded in §9.5, without its kicks.

**Depends on:** PB-C6.

- [x] Implementation: as scoped. `worlds/bodies-yard`: world.yaml (presence, movement, conversation,
  bodies; places court, hall; twelve people, all seats), places/hall.yaml (passage to the court,
  floor 12 000 × 9 000, the table) and court.yaml (10 000², the pillar), twelve people files
  (location and tags), README.md — SD-B13's layout exactly. `tools/cli/tests/bodies/mod.rs`: run,
  copy, `without_bodies`, the geometry literals with `the_world_file_still_says_the_geometry`, `scan`
  (replays presence's facts by request, checks after each request) and `assert_active` (per 10-day
  bucket). `bodies_yard.rs`: PB-4, PB-9, PB-10. `bodies_yard_restart.rs`: PB-11 (two processes,
  SIGKILL at 5/15/25, `mineworld replay` from genesis). `systems/bodies/tests/long_run.rs`: PB-14(a),
  with its own second process.
- [x] Validation (E-PB7): `mineworld validate worlds/bodies-yard` exit 0; bodies_yard 3 and
  bodies_yard_restart 1 pass; long_run passes. The activity precondition held on the first world
  layout, with no tuning. M-PB4 (binary) and M-PB8 each fail by name and are reverted.
- [x] Review: the scan reads only the save (`Tables`) and presence's published fact types; geometry
  literals are asserted against the files. Survivor exclusions asserted: SIGKILL status, no
  `history` line, head short of the control's, resume reported at the head on disk, a tail > 0. The
  scan locates its closest pair by request and instant (DB-9).

### PB-C8 — Close: cross-architecture, cost, status, the gate, the ledger

- [x] PB-12: the x86_64 build (34 s, needed the network once), three Rosetta comparisons — PASS
  (E-PB8). The fallback was not needed.
- [x] PB-14(a): release `long_run`, 45.9 µs per swept move — PASS (E-PB8).
- [x] PB-14(b): FAILED as frozen (2.06×), QP-9 tried and reverted, material stop; re-scoped by the
  primary session to (b1) ≤ 25 s — PASS at 19.3 s — and (b2) carried to 12d (DB-10, E-PB8, E-PB9).
- [x] PB-1 on the final executable head: both digests and both `validate` comparisons — PASS. M-PB1's
  300-day half applied once and reverted — the digest differs, stopped-short appears (E-PB9).
- [x] PB-15, PB-20: structural tests in the gate, the scope check; fmt, clippy, the full gate once —
  582 passed, 0 failed, 279 s (E-PB9).
- [x] Documentation: `docs/MVP_STATUS.md` (the "Bodies: walls and nudging" capability row and one
  evidence row); §17's checkboxes; §17.10; §17.11; the handoff.
- [x] Review: PB-1 … PB-20 each with evidence (the table below); deviations DB-1 … DB-10 in §17.11;
  FU-12a-1 still open (12b touches neither file).

```text
PB-1   PASS  E-PB6, E-PB9 (both digests twice, validate identical, every existing test in the gate;
             existing-test edits: QP-2's two files only); M-PB1 seen in both halves (E-PB4, E-PB9)
PB-2   PASS  rapier_pin (E-PB2); M-PB2
PB-3   PASS  rapier.rs canary and fix; M-PB3 (second form; DB-2)
PB-4   PASS  bodies_yard (E-PB7); M-PB4 at pack and binary level
PB-5   PASS  scenarios: east wall, counter, slide, fast path
PB-6   PASS  scenarios: n2, n3, the chain (DB-4, DB-5, DB-6); M-PB5, M-PB6
PB-7   PASS  scenarios: pinned against the east wall and the counter
PB-8   PASS  resolve.rs unit tests (F-P6 reproduced with two people); M-PB7
PB-9   PASS  bodies_yard (activity first; 30 803 requests, 0 violations, closest 595 mm)
PB-10  PASS  bodies_yard (without bodies: closest 0 mm, 269 202 violations)
PB-11  PASS  bodies_yard_restart; M-PB8
PB-12  PASS  E-PB8 (Rosetta, both directions)
PB-13  PASS  scenarios n1, HB-1 … HB-4 with the frozen constants; M-PB9
PB-14  (a) PASS 45.9 µs; (b) FAILED as frozen 2.06×, re-scoped by ruling: (b1) PASS 19.3 s ≤ 25 s,
             (b2) carried to 12d (DB-10)
PB-15  PASS  isolation 4; planted import; seam_vocabulary, ac1 13, precursor 4, presence's and
             movement's scans in the gate
PB-16  PASS  genesis.rs disclosure
PB-17  PASS  scenarios guard; M-PB10
PB-18  PASS  scenarios entries a–d; M-PB11
PB-19  PASS  scenarios inert; M-PB1 (pack half)
PB-20  PASS  E-PB9 (scope, gate)
```

**PR 12b lifecycle:** READY FOR OPERATOR REVIEW — DO NOT MERGE. Implementation context CLOSED /
AWAITING OPERATOR ACTION. Final executable head `0733c77`; the PR head is the commit that carries this
line. Merge with a merge commit. POST-MERGE SYNC: the planning session (step header, §§1–15, overall,
MVP_STATUS's Updated and S15 lines), with 12d's carried items (DB-10 ruling, F-B7, FU-12a-1).

## 17.6 Test ownership

```text
STATIC      cargo fmt; cargo clippy -D warnings (HashMap banned); the compiler at the installed set (a
            listed resolver type that is not an ArrivalResolver does not compile); the section type
            (an invalid `body:` cannot be constructed); Resolution's private fields (12a)
UNIT        rapier.rs: F-P1 canary and fix (PB-3); resolve.rs: verify-then-degrade on R's request-551
            geometry with the bias off (PB-8); geometry.rs: none beyond what the scenarios drive —
            the integer helpers are owned by the scenarios that exercise them
INTEGRATION through the real World::dispatch, hand-built worlds with presence, movement and bodies:
            walls, nudges, chains, wall-backed nudges, entry placement, inertness, the guard, the
            head-on bias, disclosure, genesis refusals (PB-5 … PB-7, PB-13, PB-16 … PB-19); the
            3 000-request long run's invariants and digest (PB-14 a, committed half)
STRUCTURAL  the pin and resolved features (PB-2); Rapier isolation, no persisted float, bodies names
            only presence (PB-15); seam_vocabulary, ac1_composability, precursor_vocabulary, presence's
            and movement's scans (PB-15); scope (PB-20)
REAL RUN    (Gate 2's role) the real binary: bodies-yard validate refusals (PB-4), the 30-day run and
            its scan (PB-9), the counterfactual (PB-10), two processes and SIGKILL (PB-11) — committed
            tests; the two 300-day digests (PB-1), the Rosetta runs (PB-12) and the cost runs (PB-14)
            — recorded evidence on the final head
GATE 1      NOT REQUIRED: no model is involved, and no LM-facing semantics change
CI          none configured (S13); the full local gate runs once on the final executable head
```

Owned elsewhere and not repeated:
- 12a owns the seam's refusals (RS-4), registration (RS-8 … RS-10) and the catalog's order (RS-5).
- persistence owns version refusals.
- worldpack owns the section-namespace rule and the line-and-column refusals of malformed sections.

PB-4 shows only that bodies' type is the one decoding. Each failure class above has one owner.

## 17.7 Is any of this material?

Yes, in five places. Each is raised, not assumed:

- **QP-1: the guard.** ARC-39 item 7, bullet 3 promises a reaction guard that F-B1 shows cannot be
  built. The proposal is a guard on the starting state, which panics. It amends an operator-approved
  decision record.
- **QP-2: two existing tests are edited.** Their claims are unchanged, as QR-2's were.
- **QP-3: the root manifest gains `rapier3d`.** This follows the root rule and ARC-33 item 3.
  Installing this pack therefore touches one file outside `systems/`. ARC-33's "no other file is
  edited" holds for packs that bring no external dependency.
- **QP-7: entry placement.** The step design's "blocked; the person stays on their side of the door"
  (§5) cannot be expressed by the seam (F-B3). A crossing into a jammed doorway places the person at
  the nearest free point instead, guaranteed by a capacity check at load.
- **QP-9: a contingent cost remedy.** If PB-14(b) fails, it is a dev-profile optimization override
  for Rapier in the root manifest.

Not material, and recorded:
- the test world (QP-4), which the operator suggested;
- the `body:` format (QP-5) and the default capsule for everyone (QP-6);
- the bias design (QP-8), which the operator delegated to 12b;
- `parallel` off (QP-10) and the F-P1 canary (QP-11);
- disclosure in 12b (QP-12), the single clearance number (QP-13), bodiless semantic presences
  (QP-14), an earlier resolver (QP-15) and the cross-architecture fallback (QP-16).

The following are not needed:
- any kernel, contract, persistence, server, cognition, presence or movement change;
- 12a's F2 kernel slot, or a kernel read of enabled state (QB-17);
- a change to the I-2 scan or to AC-1's checks.

AC-1's check 2 reads only market crates. Check 1 reads only the 11d and 11e merges. The I-2 scan reads
only 11a … 11c. `bodies` is none of these, so it trips none of them (PB-15 runs all three).

## 17.8 Questions (QP-1 …)

Operator-material questions are marked **[OM]**. Each has a recommendation; the others are the primary
session's to decide at freeze.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QP-1 [OM]** | **The guard.** ARC-39 item 7, bullet 3 says the first resolver pack checks the invariant "in its reactions to the recorded arrivals". F-B1 shows such a reaction sees each arrival before the rest of its list is reduced, and would fail every nudge. Options:<br>(a) a guard on the **starting state** of every non-inert resolution, which panics naming the pair, the place and ARC-39 (SD-B9);<br>(b) no guard; rely on the tests;<br>(c) a kernel hook at the end of a generation (a kernel change). | **(a)**, recorded as a dated note on ARC-39 (PB-C1). It catches exactly what bullet 3 was for — a bypass (F-R11) or an unregistered host — at the next arrival into that place. Its failure mode is the one ARC-39 already chose for host defects (a panic, like `require_registered`). PB-17 and M-PB10 show it. |
| **QP-2 [OM]** | **Two existing tests are edited** (F-B2).<br>- `worldpack/tests/registration.rs` pins the installed set's empty resolver list.<br>- `seam_vocabulary.rs` scans `systems/installed/src`, where the installed set must now name `mineworld_bodies::BodiesSystem`.<br>Edit them, or leave bodies out of the installed set? | **Edit both, keeping their claims.** In registration.rs, the expected list becomes `[bodies]`. In seam_vocabulary.rs, a second self-checking admission list admits `bodies` on the installed set's two lines only, with ARC-33 as its reason. Every other directory and word is still refused. Precedent: QR-2's three literals. Leaving bodies uninstalled would make it unusable by any host. |
| **QP-3 [OM]** | **Where `rapier3d` is declared** (F-B8). (a) In the root `[workspace.dependencies]`, by the root's own rule. (b) In `systems/bodies/Cargo.toml` only, so installing bodies touches nothing outside `systems/`. | **(a).** The root rule — one version per dependency, and a new one is a reviewable decision — is what ARC-33 item 3 kept. DEP-13 is that decision. DEP-13 also states plainly that installing a pack which brings a new external dependency adds one root line. (b) would start a second place where external versions live. |
| **QP-4** | **Test world:** a committed `worlds/bodies-yard`, or a world built inside tests? | **Committed** (SD-B13). Only a real World Pack exercises the real `body:` path through the loader, `validate`, `mineworld run` with the unchanged controller, `--save`, SIGKILL, and world-level removal by the established copy pattern. It also stays 12c's fixture. |
| **QP-5** | **The geometry format:** §10.4's `walls:` as boxes, or `floor` plus `solids` (SD-B3)? Section name `body` (§10.2) or `bodies`? | **`body:` with `floor` and `solids`.** The floor's edge is the walls. A doorway needs no gap, because a crossing is a placement. Boxes stay axis-aligned under the 3D client's frame mapping. `body` keeps §10.2's name and the singular style of `name`, `routine`, `item` and `job`. |
| **QP-6** | **Per-person shapes:** §10.2 lists `BodyShape` and `body-formed`. Needed in 12b? | **No.** Every person is the default capsule. No requirement asks for others, and `CLAUDE.md` rule 11 says no premature abstraction. 12c adds `BodyShape` for objects. Person overrides come when a world needs them. |
| **QP-7 [OM]** | **A crossing into a jammed doorway** (F-B3). §5 says the person stays on their side of the door. The seam cannot express that: a resolution must end in `to`'s place. | **Entry placement** (SD-B8):<br>- `to` if free;<br>- else nudge from `to` within I-11;<br>- else the nearest free 50 mm lattice point. This is guaranteed by SD-B4's capacity check at load.<br>The alternative — a way for a resolver to refuse — is a presence and movement change (movement would have to ask in `validate`), against I-1 and 12a's frozen seam. |
| **QP-8** | **QB-16, the bias** (SD-B10): turn the walker 14° right when the first person met is within 200 mm of its line, against criteria HB-1 … HB-4 fixed now, with a ladder if needed. | **Accept.** It is the operator's delegated decision (QB-16), made before measuring. It keeps I-11's bound, because the nudges are unchanged. |
| **QP-9 [OM]** | **If PB-14(b) fails** (bodies-yard with bodies over 1.5× without, in the dev profile): may 12b add `[profile.dev.package.rapier3d]` and `[profile.dev.package.parry3d]` with `opt-level = 3` to the root manifest? | **Yes, as a contingent permission granted at freeze.** It applies only if (b) fails. It needs re-showing that results are identical across optimization levels: PC-f did this for the prototype, and here PB-11's digests are compared with and without the override. Otherwise (b) failing is a material stop. |
| **QP-10** | `parallel`? | **Off.** The documentation says it is bitwise equal to the sequential solver, but 12b runs queries, not the solver. Threads in a single-threaded host buy nothing. Revisit only with evidence in 12c. |
| **QP-11** | F-P1's regression test, when 12b has no dynamic body? | **Keep it** (operator's binding). It is a canary that turns red when upstream changes the behaviour, and a fix test for the adapter 12c will use. |
| **QP-12** | Disclose `PlaceShape` in 12b, or leave it to 12e? | **In 12b.** It is the owner's own state, it costs one function and one test, and 12e then changes no server code (R-B4). |
| **QP-13** | One number for "too close": CLEARANCE 595 mm, at genesis and at run time? | **Yes.** One number, so a world that loads is a world the resolver accepts. |
| **QP-14** | A person with no local position in a shaped place? | **Bodiless:** ignored by others' resolutions. Their move to a position is an entry placement. Positionless arrivals are inert. |
| **QP-15** | Another resolver before bodies changed the arrival? | **Bodies passes it through** (SD-B15). The guard catches any resulting overlap at the next arrival. Two geometric resolvers are out of MVP-0 (R-B11). |
| **QP-16** | PB-12 if the CLI cannot be cross-built for x86_64? | **Fall back** to the pack's test binaries under Rosetta, recorded as PARTIAL. A native x86_64 host remains S13's. |

## 17.9 Proposed execution contract for PR 12b

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12b — people: walls and nudging (S15, second of five; the first
                    resolver pack, `bodies`, on Rapier)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §17; evidence in §17.10 (E-PB<n>);
                    deviations in §17.11
RELATED / BINDING   this file: the header's freeze record (QB-1, QB-10, QB-15), §§4.4–4.7, 5, 6.3, 9.6–9.8,
                    10.1 (I-1 … I-5, I-8, I-10 … I-13), 11.1, 15.1, §16 (12a as merged, §16.12);
                    overall.md §3 (S15), §7; DECISIONS ARC-15, ARC-23, ARC-25, ARC-26, ARC-27, ARC-31,
                    ARC-33, ARC-35, ARC-39, DEP-12; REUSE_POLICY §§11–12, 15, 17; MODULE_SPEC §§3.1, 4.1;
                    CLAUDE.md §§2–4
IMPLEMENTATION BASE main after #57 merges (03f1d7c + Markdown only); branch mvp0/pr-12b-people;
                    worktree /Users/yuema137/mineworld-worktrees/s15-12b (proposed), held by the
                    implementing session only
APPROVED SCOPE      §17.1's change set; PB-C1 … PB-C8; SD-B1 … SD-B16 as answered by QP-1 … QP-16
FROZEN INVARIANTS   No edit under kernel/, contracts/, persistence/, server/, cognition/, clients/,
                    authoring/, sdk/, systems/presence/, systems/movement/, worldpack/src/, tools/cli/src/,
                    worlds/social-cafe/, worlds/market-town/; no other existing System Pack.
                    PB-1: social-cafe sha ad49c723…c64b and market-town sha 365b50e0…1d1d over the
                    300-day seed-7 runs (365 330 and 372 755 facts), faults 0 — no digest re-baselined.
                    Existing tests unchanged except QP-2's two files.
                    rapier3d =0.36.0, enhanced-determinism, no simd8 / parallel / serde-serialize; Rapier
                    named only in systems/bodies/src/rapier.rs and the root manifest's one line;
                    nothing of Rapier survives a resolution (I-5); no float persisted (I-3).
                    I-11: ≤ 310 mm per nudge, ≤ 2 generations, ≤ 4 people; never another place, never
                    through fixed geometry. I-12: CLEARANCE 595 mm checked on integers after every
                    resolution, fallbacks blocked → halved → stay. I-13: inert where PlaceShape is
                    absent.
                    DEP-13 committed before any code.
SEQUENCE            PB-C1 → PB-C2 → PB-C3 → PB-C4 → PB-C5 → PB-C6 → PB-C7 → PB-C8, each committed and
                    pushed when coherent; the base binary for PB-1's validate comparison is built
                    before PB-C6
VALIDATION BUDGET   unit/integration/static unrestricted; real runs: each 300-day run (~13–25 s) at most
                    eight times in all (PB-1 ×2 at PB-C6 and ×2 at PB-C8, M-PB1 ×1, PB-14(b) ×4, one
                    re-run each); 30-day bodies-yard runs inside their committed tests; the x86_64
                    build once (background, ~5 min) and its three Rosetta runs; one full workspace gate
                    on the final head (background, ~6 min); about 90 minutes in total;
                    real-model: NOT REQUIRED
LIVE DOCUMENTATION  §17 checkboxes; §17.10 E-PB ledger; §17.11 deviations
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for 12b at PB-C1
ENDPOINT AUTHORITY
  implementation + local validation   unresolved until the operator's freeze message
  semantic commits, branch push       recommended authorized, as for 12a
  PR creation / update                recommended authorized, as for 12a
  scratch builds                      recommended authorized: the base binary (PB-C6) and the
                                      x86_64-apple-darwin build (PB-12), copied to /tmp/s15-12b; no
                                      branch; the target is already installed
  root-manifest profile override      only if QP-9 is approved and PB-14(b) fails
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only, with a merge commit; never inherited, never widened
POST-MERGE SYNC     the planning session owns the step header, §§1–15, overall and MVP_STATUS's Updated
                    and S15 lines; the implementing session owns §17 and the evidence rows of PB-C8
NORMAL STOP         PR 12b READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a needed edit outside §17.1's change set — above all in presence, movement, the
                    kernel or contracts; either 300-day digest differing from PB-1's; an existing test
                    failing for a reason other than QP-2; HB-1 failing on every rung of PB-13's ladder;
                    PB-14(a) above 100 µs, or (b) above 1.5× without QP-9; cross-architecture
                    digests differing (PB-12 FAIL, not PARTIAL); an answer to QP-1, QP-2, QP-3, QP-7 or
                    QP-9 other than the design's
```

## 17.10 Evidence ledger

```text
E-PB0 PB-C0, 2026-10-07, planning session, on main @ 03f1d7c (12a merged) + the 12a post-merge docs.
      Rapier facts re-verified at source (§17.3.3): the crate's own Cargo.toml in the local registry
        (rapier3d-0.36.0: license Apache-2.0, rust-version 1.86, edition 2024, the features map) and
        the primary pages (crates.io API, LICENSE, rapier.rs determinism guide, CHANGELOG) fetched by a
        web agent the same day.
      `cargo tree -e normal -p rapier3d -f "{p} | {l}"` on the prototype (offline): licences as in
        §17.3.3.
      `rustup target list --installed` → aarch64-apple-darwin, x86_64-apple-darwin.
      The prototype's R request 551, re-shown (`physics-spike r show 551`): "move person 8 by (1412,
        -1412) mm from [2536, 782] … closest after: 33 mm, persons 2 [2970, 314] and 8 [2969, 347]"
        — PB-8's geometry.
      DEP-13, DEP-14 and ARC-40: no match in docs/DECISIONS.md on any of the 53 origin/* refs.
      `python3 scripts/check_doc_headings.py` → 176 numbered sections across 25 documents, none
        duplicated. `python3 scripts/check_decision_ids.py` → 50 decision ids, all distinct.
      No cargo build or test: this commit is documentation only.

E-PB1 PB-C1, 2026-10-07, 12b implementation session, on main @ 918c869 + PB-C1's three files.
      `git fetch origin`; `git show <ref>:docs/DECISIONS.md | grep 'DEP-13\|ARC-40'` over every
        refs/remotes/origin/* → no match: DEP-13 free.
      `python3 scripts/check_doc_headings.py` → 176 numbered sections across 25 documents, none
        duplicated. `python3 scripts/check_decision_ids.py` → 51 decision ids, all distinct (50 + DEP-13).
      PASS. Documentation only; no cargo run.

E-PB-base 12b session, 2026-10-07, before any code, on 5a5bbd0 (= base 918c869 + Markdown only;
      `git diff --stat 918c869 HEAD -- ':!*.md'` empty). `cargo build -p mineworld-cli` 25.1 s;
      target/debug/mineworld copied to /tmp/s15-12b/base-mineworld (sha-256 13eb2ac9…7c86b). With it:
      `validate worlds/social-cafe` and `validate worlds/market-town` → exit 0 each, outputs kept as
      /tmp/s15-12b/base-validate-{social-cafe,market-town}.txt (27 and 51 lines): PB-1's reference.

E-PB2 PB-C2, 2026-10-07, working tree on 5a5bbd0 + systems/bodies/** + Cargo.lock.
      First Rapier build: `cargo test -p mineworld-bodies --no-run` 18.8 s (rapier3d, parry3d and 37
        more compiled at opt-level 1).
      `cargo test -p mineworld-bodies`: lib 3 (canary: z 800 mm after 20 steps without the re-mark;
        fix: z ≤ 300 mm; round trip of ±100 000 mm exact), isolation 4, rapier_pin 1 ("locked:
        rapier3d ["0.36.0"], parry3d ["0.31.1"]"; resolved features ["alloc", "default", "dim3",
        "enhanced-determinism", "f32", "std"]). PASS.
      clippy -p mineworld-bodies --all-targets -D warnings: clean. fmt --all: re-wrapped two test
        files; --check clean afterwards.
      `git diff Cargo.lock`: +390 lines, 39 `name =` lines added (mineworld-bodies, rapier3d, parry3d
        and their dependencies), no `-` line: no existing package changed.
      Licence tree (`cargo tree -p rapier3d -e normal --offline -f '{p} | {l}' --prefix none`, deduplicated):
        Apache-2.0; MIT; MIT OR Apache-2.0; MIT/Apache-2.0; Apache-2.0 OR MIT; Zlib; Zlib OR Apache-2.0
        OR MIT; Unlicense OR MIT; (MIT OR Apache-2.0) AND Unicode-3.0 — every one in §17.3.3's
        permissive set. Tool-discipline note: this one read-only pipeline used `awk` to cut the licence
        column; it touched no file, and it is not repeated.
      M-PB2 (`"parallel"` added to the pack's rapier3d line; needs `rayon`, fetched online once) →
        rapier_pin FAILED: "parallel must be off (DEP-13): [..., "parallel", "std"]". Reverted; the
        manifest and Cargo.lock restored from copies (`grep -c rayon Cargo.lock` → 0).
      M-PB3, first form (`set_translation` replaced by a no-op inside the loop) → all 3 lib tests
        PASSED: the mutation survived (DB-2). Second form (the whole re-mark loop removed) → FAILED:
        "the box fell: 800 mm". Reverted.
      Planted `use rapier3d as _;` at the end of geometry.rs (resolve.rs does not exist yet) →
        isolation FAILED: "only systems/bodies/src/rapier.rs may name rapier3d (DEP-13):
        …/systems/bodies/src/geometry.rs:87". Removed; `git grep -n MUTATION -- systems` empty; all
        green again.

E-PB3 PB-C3, 2026-10-07, working tree on 3074db9 + PB-C3's paths.
      `cargo test -p mineworld-bodies`: lib 3, genesis 6, isolation 4, rapier_pin 1 (with the VERSION
        pair), all PASS on the first run. The refusals as printed:
        "bodies-overlap: alice and bob stand 420 mm apart in room; people stand at least 595 mm apart";
        "bodies-outside: carol stands at (200, 3000) in room, outside its floor shrunk by 300 mm";
        "bodies-in-solid: dan stands 0 mm from a solid in room; …";
        "bodies-capacity: room's floor has room for a person at 4 points of the 650 mm grid; a world of
        12 people needs 45". Disclosure: alice (hall) is told exactly the hall's shape; no court record.
      M-PB4 (pack level: `&& false` on the pair check) → genesis FAILED:
        two_people_closer_than_595_mm_… "this genesis must be refused". Reverted; `git grep -n
        MUTATION -- systems` empty.
      clippy -p mineworld-bodies --all-targets -D warnings: clean after four type aliases in the test
        support (`Rect`, `Xy`, `Resident`, `Side`). fmt clean.

E-PB4 PB-C4, 2026-10-07, working tree on b48cecc + PB-C4's paths.
      First run of scenarios.rs (B = the contact sweep's end, as the prototype): 12 of 15 passed. The
        three failures were all "the walker ends at contact": the chain 612 mm, the two pinned cases
        624 mm from the person — the controller slides on along the person's curve after the first
        touch. Second run (B = the first-touch point for both uses): the chain 608 mm, and n3 had
        0 blocked strides. Third (DB-4's reading: the candidate rule measures the swept end, a
        blocked walker stops at the first touch): every behavioural claim held; the chain's 608 mm is
        the oracle's error (DB-6). Final: the position oracles as DB-6 states them. Then 15 of 15.
      Printed: n2 — b moved 507 mm in all, in 3 nudges (≤ 310 each); n3 — 6 strides blocked, at most
        2 moved, 2 generations; chain — walker at (2 871, 5 000), stopped-short by p1, nobody moved;
        pinned — walker at (7 451, 2 983) and (5 266, 5 701); guard — "bodies: alice and bob stand
        200 mm apart in room, closer than 595 mm, … (DECISIONS.md ARC-39)". Entries: (900, 2 000)
        exactly; the occupant nudged to (1 510, 2 000); placed at (1 100, 1 850) by the occupant;
        placed at (5 000, 6 250) by None.
      PB-8 (lib): verification off → walker (2 983, 383), the person against the wall (2 970, 314),
        70 mm apart (the prototype: 33 mm) — F-P6 reproduced with two people; on → walker (2 563,
        758), 602 mm apart, `degraded: Halved(4)`.
      Mutations (each applied, run, reverted; `git grep -n MUTATION -- systems` empty afterwards):
        M-PB5  CHAIN_MAX 3 → the chain case FAILED: p3 recorded at (4 464, 6 098) — the third
               generation, exactly the hand-computed 4 376 + 88, 6 032 + 66.
        M-PB6  NUDGE_MAX 400 → n2 FAILED "b nudged 338 mm, more than 310 (I-11)"; n3 FAILED "c1 nudged
               402 mm".
        M-PB7  PRODUCTION verify off → PB-8 FAILED "walker (2983, 383) and the person against the
               wall (2970, 314) are 70 mm apart".
        M-PB10 the guard call removed → PB-17 FAILED "the next arrival into the place must panic".
        M-PB11 E3 answers `to` → case c FAILED "occupant and walker stand 400 mm apart in the hall";
               case d FAILED too.
        M-PB1  (pack half) a place without a shape treated as a ±100 m floor → PB-19 FAILED, at the
               guard: "bob and carol stand 316 mm apart in yard".
      clippy and fmt clean. Tool-discipline note: PB-8's test module was appended with a `cat >>`
        heredoc rather than the Edit tool; content as intended; not repeated.

E-PB5 PB-C5, 2026-10-07, working tree on c16ab2a + PB-C5's paths. Criteria HB-1 … HB-4 as §17.4
      PB-13 states them, applied unchanged; constants as frozen (BIAS_BAND 200, BIAS_TURN (4, 1)).
      `cargo test -p mineworld-bodies --test scenarios -- --nocapture`: 17 passed.
        HB-1 PASS: n1 — "a ends at (7575, 4675), b at (550, 5285); 1 strides turned by the bias". The
          turn came at request 7 (b, meeting a head-on: b → (4 336, 5 121), a nudged), then two
          off-centre strides with one nudge each, then both walk on clear corridors.
        HB-2 PASS: every one of the 24 requests passed `bounded_stride` (pair ≥ 595 mm, every nudge
          ≤ 310 mm, ≤ 4 moved, ≤ 2 generations, both inside the floor shrunk by 295 mm, every fact
          caused by its request and stated by movement).
        HB-3 PASS: n2 — b moved 211 mm in all, in 2 nudges; a ends at (7 562, 4 698), east of b at
          (4 038, 5 308); n3 — 2 strides blocked, at most 4 moved, 2 generations.
        HB-4 PASS: a second process of the same binary printed n1 (26 605 bytes), n2 (15 046), n3
          (35 268), each equal to this process's bytes (every fact's envelope and the final state).
      The ladder of PB-13 was not walked: the first rung's criterion passed.
      PB-8 under the production policy: biased, walker (2 552, 756), 608 mm from the person against
        the wall, not degraded.
      M-PB9 (BIAS_BAND 0) → n1 FAILED: "HB-1: after 24 requests a has passed b: a (3710, 5000), b
        (4320, 5000)" — F-P7 reappears. Reverted; `git grep -n MUTATION -- systems` empty.
      geometry unit test: the turn over 9 000+ strides on a 37 mm grid — never longer, right, ~14°.
      clippy (one type alias `Keyed`) and fmt clean.

E-PB6 PB-C6, 2026-10-07, working tree on 539ebbf + PB-C6's four paths + Cargo.lock.
      `cargo test --offline -p mineworld-installed-systems -p mineworld-worldpack -p
        mineworld-acceptance`: every binary ok — ac1_composability 13, precursor_vocabulary 4,
        seam_vocabulary 3, arrival_resolvers 7, arrival_resolvers_unregistered 2,
        arrival_resolvers_resume "[resolver-yard] PASS in 0.2 s", installed 3, resolution 2,
        registration 1, refusals 38, social_cafe 15, … PASS.
      `cargo test --offline -p mineworld-cli`: 18 test binaries, 41 passed, 0 failed, 3 min 15 s. PASS.
      PR binary (target/debug/mineworld → /tmp/s15-12b/pr-mineworld), 300-day runs 1 and 2 of 8:
        social-cafe → exit 0, faults 0, 365 330 facts, no stopped-short line, sha-256 of every line
          but `wall` = ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b. Wall 12.7 s.
        market-town → exit 0, faults 0, 372 755 facts, sha-256 =
          365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d. Wall 15.1 s. PASS.
        `validate` of both, `cmp` against the base binary's outputs: identical. PASS.

E-PB7 PB-C7, 2026-10-07, working tree on 1b86701 + PB-C7's paths.
      `mineworld validate worlds/bodies-yard` (PR binary) → exit 0, 15 genesis facts, "a valid World
        Pack". A first look, `run worlds/bodies-yard --headless --seed 7 --days 30` → faults 0, moves
        accepted 15 679, talks 15 124, arrived 22 484, stopped-short 9 849, person-entered-place 2 329;
        wall 3.6 s.
      `cargo test -p mineworld-cli --test bodies_yard --test bodies_yard_restart -- --nocapture`: 3 + 1
        passed (the counterfactual's first run failed on the test's own check that the body: section
        was stripped — a comment line says "floor"; the check now looks for the section's lines).
        PB-4: each copy refused, naming its subject — "bodies-overlap … ada and ben stand 420 mm apart
          in hall"; "bodies-in-solid … cleo stands 0 mm from a solid in hall"; "bodies-outside …
          cleo stands at (200, 4500) in hall"; "bodies-capacity … 4 points … 12 people needs 45";
          the loader at hall.yaml "line 23 column 3: the floor runs from min to max and each side is
          at least 620 mm; this one is 0 mm by 9000 mm"; "line 26 column 3: unknown field `wall`,
          expected one of floor, solids"; "line 23 column 3: solid 0 is 0 mm high; a solid is 1 to
          10000 mm high". The yard itself validates. PASS.
        PB-9: activity first, per 10-day bucket — days 1–10: every seat 400 … 470 moves, stopped-short
          3 095, displaced 2 269, crossings 364 each way; 11–20: 3 385 / 2 229 / 389 each way; 21–30:
          3 369 / 2 295 / 411 and 412. Then the scan: 30 803 requests, no violation; closest pair gus
          and kai, 595 mm apart in court, after request ActionId(22106) at 1 860 306 s. PASS.
        PB-10: without bodies, faults 0, every seat moves in every bucket, no stopped-short or
          displaced arrival; the scan reports 269 202 violations, closest ada and cleo, 0 mm apart in
          court, after request ActionId(365) at 30 602 s. PASS (the instrument sees overlaps).
        PB-11: control 59 619 facts, 30 804 journal rows, 482 snapshots, 9 849 stopped-short, 6 793
          displaced; a second process byte-identical; killed at revisions 5 137, 15 434, 25 673,
          each resumed there with 17, 10, 9 re-executed, byte-identical to the control; `mineworld
          replay` → "30804 revision(s) re-executed from genesis, 59619 fact(s) and 482 snapshot(s)
          reproduced byte for byte". PASS.
      `cargo test -p mineworld-bodies --test long_run -- --nocapture` (dev): "2548 moves; closest pair
        p09 and p11, 600 mm, after request 1613"; 1 302 moves reached Rapier, mean 38.9 µs each; the
        fast path answered 1 246 (48.9 %); 4 091 738 bytes identical in a second process. PASS.
      M-PB4 (binary; `&& false` on the pair check) → bodies_yard FAILED: "overlap: the copy must be
        refused" (the copy validated). Reverted.
      M-PB8 (a process-global counter's parity added to the nudge spacing) → bodies_yard_restart
        FAILED: the day-5 survivor "replay diverged at revision r5121: fact 9859 differs from the
        logged fact 9859". Reverted; `git grep -n MUTATION` empty.
      clippy (-p mineworld-cli -p mineworld-bodies, two type aliases in the scan) and fmt clean.

E-PB8 PB-C8 (partial — stopped at PB-14(b), a material stop), 2026-10-07, on db64471's code.
      PB-12 (cross-architecture; recorded real evidence). `cargo build -p mineworld-cli --target
        x86_64-apple-darwin` (online once: `--offline` lacked an x86-only crate; Cargo.lock unchanged)
        34.1 s → Mach-O x86_64. arm64 binary rebuilt at the same code. Logs /tmp/s15-12b/pb12/.
        (a) `run worlds/bodies-yard --headless --seed 7 --days 30`: arm64 and `arch -x86_64` x86_64
            each exit 0, faults 0, 59 619 facts; sha-256 of every line but `wall` =
            3a2c3322bcebc39e8d50e2969cfe25bd73da36e743a64f728b567337fb5eaa9d for both. Wall 2.0 s and
            3.3 s.
        (b) arm64 save to day 15, resumed by x86_64 to day 30: "resumed … at revision 15434 (snapshot
            15424 + 10 re-executed)", history 59 619 facts, fingerprint 8dd003dddc41858d = the arm64
            uninterrupted run's; then arm64 `replay` of that save: "30804 revision(s) re-executed from
            genesis, 59619 fact(s) and 482 snapshot(s) reproduced byte for byte".
        (c) the reverse: x86_64 save to day 15, resumed by arm64 — the same resume line and history;
            x86_64 `replay` of it reproduces byte for byte. And x86_64 `replay` of the arm64
            uninterrupted save: byte for byte.
        PASS (under Rosetta; a native x86_64 host remains S13's).
      PB-14 (a) (release): `cargo test --release -p mineworld-bodies --test long_run -- --nocapture`
        (19.6 s with the build): 2 548 moves; 1 302 reached Rapier, mean 45.9 µs per such move (target
        about 61, PASS ≤ 100); the fast path answered 1 246 (48.9 %); closest pair 600 mm; 4 091 738
        bytes identical in a second process. PASS. (Dev, E-PB7: 38.9 µs.)
      PB-14 (b) (dev profile, 300 days, seed 7, one machine, consecutive; 300-day runs 3–6 and 7–10 of
        the budget — the second four are QP-9's re-measurement):
        with bodies (worlds/bodies-yard)          19.4 s, 19.3 s   faults 0, 0
        without (PB-10's copy, target/tmp/…)       9.4 s,  9.4 s   faults 0, 0
        max(with) / min(without) = 2.06 > 1.5. FAIL.
        QP-9's remedy, applied as permitted: `[profile.dev.package.rapier3d]` and `.parry3d`
          `opt-level = 3` in the root manifest; rebuilt (16.5 s); with 19.1 s, 19.2 s; without 9.3 s,
          9.3 s — still 2.06×. Every printed line but `wall` identical with and without the override
          (`diff`), for both worlds. The remedy does not move the bound, so the root-manifest edit was
          reverted rather than kept without its justification. FAIL with QP-9 → material stop (§17.9).
        Where the time goes (from the runs' own summaries): both worlds ask the same — 345 600
          consults, ≈ 158 000 moves and ≈ 150 600 talks accepted. With bodies, 99 499 moves end
          stopped short (63 %) and 70 262 displaced arrivals are recorded: 599 401 facts against
          429 606. The paced controller's wander (±1 400 mm a stride) knows no walls, so in two
          walled rooms most moves reach a wall and are swept; the long run puts a swept move at about
          40 µs (dev) and a fast-path move far below it, and each extra fact is recorded, reduced and
          fingerprinted. ≈ 9.9 s over ≈ 158 000 moves is ≈ 62 µs per move.
      Not run, pending the decision: PB-1 on the final head and M-PB1's 300-day half; the full gate.

E-PB9 PB-C8 close, 2026-10-07, after the DB-10 ruling. Final executable head 0733c77 (code identical
      to db64471: every later commit changes Markdown only).
      M-PB1, 300-day half (the inert rule dropped: a place without a shape read as a ±100 m floor; built
        once into /tmp/s15-12b/mpb1-mineworld, then the source reverted — `git grep MUTATION` empty):
        `run worlds/social-cafe --headless --seed 7 --days 300` → exit 0, faults 0, 439 775 facts,
        "facts      stopped-short 37737", sha-256 of every line but `wall` = f0cc1c47bfdbd5b4…0e98f9 ≠
        PB-1's ad49c723…c64b. Wall 664.9 s (every entry searches a 200 m lattice; a first attempt was
        killed after 5 min of silence and the run restarted in the background). PASS (it fails PB-1).
      Full gate (once, background, 23:39:59 – 23:44:43; the M-PB1 run shared the CPU):
        `cargo fmt --all --check` exit 0; `cargo clippy --workspace --all-targets --all-features -- -D
        warnings` exit 0; `cargo test --workspace --no-fail-fast` exit 0, wall 279 s: 139 harness
        binaries, 582 passed, 0 failed; the `harness = false` programs "[resolver-yard] PASS",
        "[cafe] PASS", "[clock] PASS"; ac1_composability 13, precursor_vocabulary 4, seam_vocabulary 3,
        bodies_yard 3, bodies_yard_restart 1, scenarios 17, long_run 1, rapier_pin 1;
        `check_doc_headings.py` → 176 sections, none duplicated; `check_decision_ids.py` → 51 ids, all
        distinct. Logs /tmp/s15-12b/gate.log, test.log, clippy.log. PASS.
      PB-1 on the final head (binary from the gate's build, /tmp/s15-12b/final-mineworld): social-cafe
        exit 0, faults 0, 365 330 facts, sha ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b;
        market-town exit 0, faults 0, 372 755 facts, sha
        365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d; both `validate` outputs `cmp`
        identical to the base binary's. PASS.
      PB-14 (b1) (the ruling's guard, fixed before this measurement: ≤ 25 s): `run worlds/bodies-yard
        --headless --seed 7 --days 300`, dev, quiet machine → exit 0, faults 0, 599 401 facts, wall
        19.3 s; every other line identical to E-PB8's runs. PASS. Ratio to the world without bodies:
        2.06× (E-PB8), recorded as information.
      PB-20: `git diff --name-only 918c869...HEAD` = 45 paths, every one in §17.1's change set (with
        §17.0's override: no root manifest); `git diff --stat` over kernel/, contracts/,
        persistence/, server/, cognition/, clients/, authoring/, sdk/, systems/presence/,
        systems/movement/, worldpack/src/, tools/cli/src/, worlds/social-cafe/, worlds/market-town/ and
        the root Cargo.toml: empty; Cargo.lock: no line removed (packages added only). PASS.
      300-day runs in all: 13 (PB-1 ×4, PB-14(b) ×4, QP-9's re-measurement ×4, b1 ×1) plus M-PB1 ×1
        and its killed first attempt — over the budget of 8; the QP-9 overrun accepted by the ruling,
        b1 required by it.
```

## 17.11 Deviations and discoveries during implementation

**DB-1 (bounded; follows §17.0) — what QP-3's overruling changes in the text below it.**
- §17.0 overrules QP-3: `rapier3d =0.36.0` is declared in `systems/bodies/Cargo.toml`. Several
  sentences written before the freeze still say "root": §17.1's change set (`Cargo.toml (root)`
  `[workspace.dependencies]`), SD-B14, PB-C1 ("declared in the root workspace"), PB-C2's first scope
  bullet, PB-2's M-PB2 ("added to the root line"), PB-15's second bullet ("except the root
  Cargo.toml's one dependency line"), and the §17.9 invariant ("the root manifest's one line").
- Reading, as §17.0 binds: each "root" there means `systems/bodies/Cargo.toml`. The root manifest is
  not touched (unless QP-9's contingency fires). M-PB2 adds `parallel` to the pack's line; PB-15's
  allowance is the pack's manifest.

**DB-2 (discovery) — what actually re-marks the bodies in F-P1's workaround.**
- Previous assumption: §9.4 and SD-B11 say the workaround is `set_translation(current, true)` on each
  dynamic body after `detect_collisions`.
- Audit evidence: M-PB3 written as "replace `set_translation` with a no-op inside the loop" survived —
  the box still fell. `rapier3d-0.36.0/src/dynamics/rigid_body_set.rs:407–415`: `iter_mut` clears
  `modified_bodies` and pushes every body it yields onto it. Borrowing the set mutably is the re-mark.
- Corrected understanding: the loop over `bodies.iter_mut()` is the workaround; `set_translation(..,
  true)` additionally wakes the body and states the intent.
- Implementation: the code is unchanged; `refresh`'s comment says which line does what. M-PB3 is
  "the whole loop removed", which fails the fix test (E-PB2).

**DB-3 (bounded, tightening) — the no-float scan covers every source file but `rapier.rs`.** §17.4
PB-15 names component.rs, event.rs and section.rs. PB-C4's review asks "no float outside `rapier.rs`".
The scan holds the latter, which includes the former; it lists each offending line.

**DB-4 (bounded; flagged for the operator) — where a blocked walker stops.**
- Previous assumption: §4.5.1 step 2 defines the contact reach B as "the same sweep with people
  solid", step 5 puts a blocked walker at B, and §17.4 PB-6/PB-7 expect the blocked walker "at contact
  (± 1 mm)" with the first person.
- Audit evidence (E-PB4): with `slide: true` (SD-B11), once the capsule touches a person the
  controller slides it on along their curve. For an oblique approach the sweep's end is 612 mm
  (chain) and 624 mm (pinned) from the person — not at contact. Using the first-touch point for both
  of B's uses keeps the contact, but changes the candidate rule's measurement and n3 then never
  blocks (0 of 12), against PB-6's "at least one stride is blocked" and the prototype's n3 (3 of 12).
- The text is inconsistent for oblique contacts: no single B satisfies both "B is the sweep" and
  "a blocked walker is at contact".
- Decision: B's two uses take its two meanings. The candidate rule (step 3) measures how far the
  walker gets with people solid — the sweep's end, sliding included, exactly the prototype's B. A
  blocked walker (step 5, and the halving path of step 8) stops where that sweep first touched a
  person: `Swept.contact`, from Rapier's `CharacterCollision::translation_applied`. Without a person
  touched, both are the sweep's end.
- Invariants: unchanged (I-11, I-12: the contact point is ≥ 600 mm from the person, and verify still
  runs). Scope: unchanged. The R′ algorithm's numbers differ from the prototype's only where a
  blocked walker would have slid on.
- Validation: every PB-5 … PB-8, PB-17 … PB-19 scenario passes; n3 blocks 6 of 12; the long run
  (PB-14 a) measures N-1 … N-4 on it.

**DB-5 (discovery) — n2's and n3's first person met lies inside the head-on band.** PB-6 says each
case's layout keeps the first person met outside `BIAS_BAND`. n2's B stands 100 mm off the walker's
line and n3's first person exactly on it, as the prototype's scenarios (§9.6) fix them. Those two
layouts are not changed; HB-3 already requires that "n2 and n3 still meet PB-6" with the bias on, so
PB-C5 re-runs them under it. The chain case (300 mm off) and the two pinned cases (219 mm off) are laid
out outside the band, as PB-6 asks.

**DB-6 (bounded) — the hand-computed contact point of an oblique stop.** PB-6/PB-7's "at contact
(± 1 mm)" was first written as 610 mm = 2R + GAP from the person. Rapier keeps the controller's 10 mm
offset along the direction of motion, so a walker meeting a person at angle θ off their line of
centres stops 600 + 10·cos θ from them: 608.7 mm in the chain layout, 609.3 mm in the pinned ones.
The tests now assert, as §17.4's rule for positions through Rapier states, the reached point within
±1 mm per axis of the contact point computed by hand from the layout — (2 870, 5 000), (7 452, 2 984),
(5 266, 5 702) — and that the walker touches without overlapping (600 … 611 mm). Rapier's points:
(2 871, 5 000), (7 451, 2 983), (5 266, 5 701).

**DB-7 (bounded) — `explain`, a public account of a resolution.** PB-6 reads generations "from the
resolver core's outcome in the same tests", and `scenarios.rs` is an integration test that sees only
the crate's public API. `mineworld_bodies::explain(world, person, to) -> Option<Outcome>` runs the
same core as the resolver, reading `from` from presence as presence does. It writes nothing and
names no Rapier type; it is the pack's own surface, not a seam change.

**DB-8 (bounded; within QP-2) — the seam scan's admission covers registration.rs too.** QP-2 admits
`bodies` "on the installed set's two lines only". But `worldpack/tests/registration.rs` is itself one
of the seam scan's files (12a's `ADDED_TEST_FILES`), and QP-2's own edit makes it name the pack. Both
files are QP-2's; the test names the pack on exactly one line (`const LISTED: &str = "bodies";`), and
`INSTALLED_PACK_LINES` admits the word there and nowhere else in that file, with the same
fails-if-unused rule.

**DB-9 (bounded) — the scan locates by request and instant, not by revision.** PB-9 asks the scan to
print "the closest pair, its place and its revision". A stored fact carries its `EventId`, its
instant and its cause, not the journal revision that produced it. The scan names the request
(`ActionId`) and the instant, which locate the same moment in the save without reading the journal.

**DB-10 — MATERIAL STOP: PB-14(b) fails, with QP-9's remedy applied (E-PB8).**
- Criterion (fixed before measuring): 300 days of bodies-yard in the dev profile, max(with bodies) ≤
  1.5 × min(without). Measured 19.4 / 19.3 s against 9.4 / 9.4 s: 2.06×. With QP-9's opt-level 3 for
  rapier3d and parry3d: 19.1 / 19.2 s against 9.3 / 9.3 s, still 2.06×, outputs identical. §17.9 makes
  this a material stop; the override is reverted, the root manifest is untouched.
- What the evidence says: the cost is the world, not a defect. Both runs ask the same requests; with
  bodies, 63 % of moves end stopped short at a wall or a person and 70 262 people are nudged, so
  nearly every move is swept and 170 000 more facts are recorded. Release cost per swept move is
  45.9 µs (PB-14 a, PASS). Every other criterion that ran passed.
- Options for the decision (none taken):
  1. Read QB-11's bound as it was meant (step-11 §13: "measured in 12c and 12d", against the town),
     and for bodies-yard — a world built to make contacts happen — record the measured ratio rather
     than gate on it; PB-14(a)'s per-move cost stays the gate here.
  2. Make swept moves cheaper inside the frozen design, then re-measure: build the scene's broad
     phase without the narrow phase (`detect_collisions` computes contacts no query uses), and put in
     the scene only the people a stride can reach. Both change the adapter, not the rules; the first
     needs F-P1's re-check, the second a recorded change to SD-B11's canonical order.
  3. Answer more walls-only strides by integers (the floor's edge is axis-aligned). A change to
     SD-B6's fast path, so to Rapier's slide results: a design change.
  4. A layout change to bodies-yard that makes its people walk into walls less. The bound would then
     measure the world's layout more than the pack.

**DB-10 ruling (primary session, 2026-10-07): option 1 — a re-scope decided after a failed
measurement, recorded as such.**
- **PB-14(b) FAILED as frozen:** 2.06× in all four runs (19.4 / 19.3 s with bodies, 9.4 / 9.4 s
  without). QP-9's opt-level remedy was tried, did not move the ratio (19.1 / 19.2 s against
  9.3 / 9.3 s), and was reverted.
- **Why the bound is re-scoped:** QB-11's "+50 % on a 300-day run" was stated for the towns — §13:
  "measured in 12c and 12d". Applying it to bodies-yard, a deliberately contact-heavy stress world
  where 63 % of moves end stopped short, was 12b's own extension in §17.4. The primary session
  re-scoped the bound to what QB-11 bounds, after the failure, and says so here.
- **Re-scoping is not loosening.** For 12b, PB-14(b) is replaced by two checks:
  - **(b1) An absolute regression guard on bodies-yard,** fixed by the ruling before any further
    measurement: a dev-profile 300-day seed-7 run of bodies-yard with bodies takes **≤ 25 s on this
    machine**. The ratio to the world without bodies (2.06×) is recorded as information, not as a
    pass or a fail.
  - **(b2) QB-11's bound stays exactly as written for the towns.** 12d must show ≤ 1.5× on
    social-cafe and market-town with geometry. If a town fails, DB-10's options 2 and 3 (a cheaper
    sweep; integer walls-only strides) become 12d's work, raised as a design change at that point.
    The town bound is never re-scoped.
- **Carried to 12d** (with FU-12a-1 and F-B7): the ≤ 1.5× bound on both towns' 300-day seed-7 dev
  runs, with versus without bodies; on failure, DB-10 options 2 and 3 as a design change. Not
  re-scoped.
- **Accepted as bounded by the same ruling:** DB-4, DB-6, DB-7, DB-8, DB-9, and the 300-day budget
  overrun from QP-9's re-measurement (10 runs before the close against 8, the extra four being the
  re-measurement). Each stands as recorded above.
- **Tool discipline:** the two slips recorded in E-PB2 (`awk`) and E-PB4 (`cat >>` heredoc) are not
  repeated; every file change is made with the Edit and Write tools.

## 17.12 Operator review and merge (post-merge record, planning session)

**Merged:** GitHub #59, merge commit `9c617ed` (2026-10-07), with a merge commit as frozen. PR head
`ce986fe`; final executable head `0733c77`, unchanged in code by the Markdown-only commit after it
(E-PB9).

**The operator's review evidence, on the PR head, before the merge:**

- **Gates re-run:** 582 passed, 0 failed. This matches E-PB9.
- **Scope:** the root `Cargo.toml` has no diff (§17.0's overruling of QP-3 held; QP-9's contingent
  override was tried and reverted, DB-10). No path under `kernel/`, `contracts/`, `server/`,
  `persistence/` or `cognition/` is touched (I-1).
- **Rapier isolation:** no use of `rapier3d` or `parry3d` outside `systems/bodies/src/rapier.rs`. The
  other mentions are comments and the `mod rapier;` declaration (DEP-13's isolating interface; PB-15).
- **The operator's own mutation:** `CLEARANCE` set to 300 mm. It failed
  `resolve::tests::verify_then_degrade_keeps_the_pair_apart` (PB-8). The mutation was reverted. It is
  independent of the implementing session's M-PB1 … M-PB11: none of those changed the clearance
  constant itself.
- **After the merge, on main:** `ac1_composability` passes 13/13 and `precursor_vocabulary` (the I-2
  scan) 4/4 (PB-15 holds on the merged tree).

**PB-14(b), recorded as it happened.** The criterion failed as frozen: 2.06× in all four runs, with
QP-9's remedy tried and reverted (E-PB8). The primary session's DB-10 ruling re-scoped it after the
failure, and says so:

- **(b1)**, the absolute guard fixed before its measurement, ≤ 25 s for a dev-profile 300-day seed-7
  run of bodies-yard: **PASS at 19.3 s** (E-PB9);
- **(b2)**, QB-11's ≤ 1.5× bound on social-cafe and market-town with geometry: **carried to 12d
  unchanged**, never re-scoped.

**Deviations:** DB-1 … DB-9 stand as bounded, as the DB-10 ruling accepted them. DB-10 is closed by its
ruling.

**What 12b leaves.**

- **For 12c:** `bodies` is the build's first and only registered resolver, inert wherever a place has
  no `body:` section. The adapter already carries the F-P1 re-mark and its canary (PB-3) for the
  dynamic bodies 12c introduces. `worlds/bodies-yard` is the pack's fixture (SD-B13). 12c is detailed
  as §18.
- **For 12d, carried:** QB-11's ≤ 1.5× bound on both towns' 300-day seed-7 dev runs, with versus
  without bodies, and on failure DB-10's options 2 and 3 as a design change (DB-10 (b2)); F-B7, the
  café's doorway point 200 mm from its wall; FU-12a-1, the three allow-listed comments.

---

# 18. PR 12c — objects: walking pushes them; kick, throw and shove (full design; DESIGN FROZEN 2026-10-08; MERGED #67, `889d217`)

**Lifecycle:** drafted by the planning session on `mvp0/s15-12c-plan` on 2026-10-08, stacked on the 12b
post-merge docs PR (#60). Frozen by the primary session on 2026-10-08 (§18.0). **READY FOR OPERATOR
REVIEW** (2026-10-08): implemented on `mvp0/pr-12c-objects`, final executable head 145c7f7's tree (the
squashed PR's code is that tree); the implementation context is CLOSED / AWAITING OPERATOR ACTION.
Merge with a merge commit, by the operator only. Post-merge sync: the planning session owns the step
header, §§1–15, overall and MVP_STATUS's Updated and S15 lines; this session owns §18.
**MERGED** as GitHub #67, merge commit `889d217` (2026-10-08), with a merge commit as frozen; the review
and merge record is §18.12, written by the planning session at the operator's instruction.

## 18.0 Freeze record

**DESIGN FROZEN (2026-10-08), primary session.** The execution contract (§18.9) is confirmed. This
record binds, and overrides any other text in §18.

- **QO-1 to QO-20 are accepted as recommended.** None of them, as recommended, contradicts an operator
  decision.
  - QO-3: objects are authored in their item file. QB-3's letter stands.
  - QO-5: 12c has no pick-up or put-down. Scope is unchanged. Carrying is designed only, and waits on
    a unique-held-object representation (ARC-36 item 3).
- **The `bodies` → `mineworld-item` crate dependency is accepted, with bounds.** `bodies` uses it
  only to call the read-only `mineworld_item::is_declared`, so that an Item cannot carry both `body:`
  and `item:`.
  - This couples a physics pack to a market pack's crate. AC-1's check 2 allows it, because the
    dependent lives under `systems/`. It is still a coupling, so 12c must:
    - (a) record it in ARC-39's 12c note, naming the one function used;
    - (b) add a test asserting that `bodies` names nothing else from `mineworld_item`, with a
      mutation showing that the test bites;
    - (c) declare no *system* dependency on `item`. A world may install `bodies` without `item`,
      and `validate` and a run must show that.
  - If the implementation needs anything more from `item`, that is a material stop.
- **QO-16:** the named test edits are accepted with their claims unchanged. They are `rapier_pin`
  (VERSION 2), `isolation`'s dependency claim, and the bodies-yard scan and activity lines.
  - 12b's 30-day bodies-yard sha is not frozen and may change.
  - The towns' digests and 12b's long-run base capture may not change.
- **The activity ladder (AO-1 to AO-3) is fixed now.** Any fix goes to content or pack offer policy,
  never to the controller.
- **The cost bounds are fixed now and are not re-scoped after measuring:**
  - release ≤ 100 µs per swept move;
  - release ≤ 2 ms per kick or throw;
  - 300-day dev run ≤ 40 s.
- **Merge:** with a merge commit.
- **Implementation:** in a fresh session on `mvp0/pr-12c-objects`, in its own worktree.

This section refines §4.5.4, §4.6, §4.7, §7.2, §8.1, §10.1–§10.4 and §11.1's 12c row from merged
source. Where they and §18 disagree, §18 governs, and each difference is named with the finding that
caused it (§18.2) and the question that asks for it (§18.8).

## 18.1 Identity, base, approved scope

```text
PR            12c — objects: walking pushes them; kick, throw and shove (S15, third of five)
base          main after #60 (the 12b post-merge docs PR) and the PR carrying this design merge —
              9c617ed + Markdown only. Re-audit §18.2 if anything under systems/bodies, systems/presence,
              systems/item, systems/inventory, worldpack/src, authoring/src, kernel/src,
              cognition/rule-controller/src, worlds/bodies-yard or tools/cli/tests moved
branch        mvp0/pr-12c-objects, in its own worktree, held by the implementing session only
audit         §18.2 (main @ 9c617ed, 2026-10-08)
scope         §4.5.4 (objects pushed by walking, predicted by the resolver), §8.1's kick, throw and shove,
              QB-3 (an Item with a body is one physical object; ARC-36 amended), QB-6 (kick and throw land
              at the instant), QB-8 (no Busy), as refined by SD-O1 … SD-O22 and answered by QO-1 … QO-20
depends on    12b merged (9c617ed). The ArrivalResolver seam (12a) and bodies' people resolution (12b) are
              used as merged; 12b's people path is changed only where objects enter it (SD-O9)
merge         a merge commit, never a squash
```

**Goal.** A world with bodies has loose objects, and people interact with them and with each other
physically, decided on the server:

- **Walking pushes objects.** A stride that ends overlapping a loose object pushes it out of the way,
  never through a wall, a solid or another object, and never into a person. The resolver predicts the
  pushes before presence records the stride; when they cannot all be made, the stride is resolved with
  objects solid, so the walker never ends inside a jammed box (§4.5.4, the operator's binding).
- **`kick`, `throw` and `shove`** are actions of `bodies`, each with a declared spatial requirement,
  each offered as a **complete affordance** (`ARC-34`), so the unchanged paced controller attempts them.
  Kick and throw are resolved at the instant (QB-6): the object's flight is simulated, and the object
  lands at a deterministic, verified point. Shove moves a person 500 mm through presence's `arrivals()`,
  so walls stop them and the people behind them are nudged within I-11.
- **Everything stays deterministic** under i32-millimetre quantization: two processes, SIGKILL and
  resume, and arm64 against x86_64 under Rosetta are byte-identical.

Social-cafe and market-town are untouched: neither installs `bodies` (their geometry is 12d's), so both
300-day seed-7 digests are unchanged.

**Change set.** Every path this PR may touch:

```text
docs/DECISIONS.md                     notes: ARC-36 (QB-3: an Item with a body is one physical object),
                                      ARC-39 (objects, shove), DEP-13 (dynamics are now used)
docs/MODULE_SPEC.md                   §4.1: the `body` section on item files; the item-file paragraph
docs/MVP_STATUS.md                    one capability row, one evidence row
systems/bodies/**                     the pack: objects, pushes, kick, throw, shove (crate
                                      mineworld-bodies; VERSION 1 → 2)
Cargo.lock                            mineworld-bodies' dependency list gains mineworld-item; nothing else
worlds/bodies-yard/**                 items: and items/*.yaml with body: sections; README
tools/cli/tests/bodies/mod.rs         the scan: objects, and the facts shove and pushes state
tools/cli/tests/bodies_yard.rs        refusals of objects at load; the 30-day run's activity criteria
tools/cli/tests/bodies_yard_restart.rs   its activity line gains the new facts (QO-16)
systems/bodies/tests/{rapier_pin,isolation}.rs   QO-16's literal and claim; the other 12b tests unedited
.structured-coding/plans/mvp0/{step-11-bodies,handoff}.md   this ledger, the handoff
```

Paths with no diff:
- `kernel/`, `contracts/`, `persistence/`, `server/`, `cognition/`, `clients/`, `authoring/`, `sdk/`;
- `systems/presence/`, `systems/movement/`, `systems/item/`, `systems/inventory/`, `systems/installed/`
  and every other existing System Pack;
- `worldpack/src/`, `tools/cli/src/`, `tests/acceptance/`;
- `worlds/social-cafe/`, `worlds/market-town/`;
- the root `Cargo.toml`.

**Non-goals.**
- Picking an object up, carrying it and putting it down (§8.1: "carrying (an inventory concern)"). The
  ownership rule for that crossing is settled here (SD-O3), and the actions are a later step (QO-5).
- Objects in social-cafe and market-town, and their digests (12d).
- Godot, Jolt (`DEP-14`), colliders for objects and the animation of `path` (12e).
- Momentum between requests, a deferred landing (QB-6), chain reactions between objects, rotation,
  stacking, damage, knocking anybody over, `Busy` (QB-8), doors.
- No change to presence, movement, the seam, the controller or the kernel.

## 18.2 Source audit (`main @ 9c617ed`, 2026-10-08)

Each finding below was read in this session from the file named.

| ID | Finding | Evidence | Consequence for 12c |
| --- | --- | --- | --- |
| **F-O1** | **Perception can neither list a loose object nor target one.**<br>- An observation lists the observer's place and the people in it (`present_with` reads `Presence`).<br>- Offers are asked for target `None` and for each present person only.<br>- `verdict` reads a target's position from `Presence` alone.<br>An Item has no `Presence`. | `systems/presence/src/observe.rs:100–112` (`present_with`), `:239` (`targets`), `:260–270` (`verdict`); `ARC-37` limitation "items are never perceived" | §7.2's `{ action: kick, target: <object> }` cannot be offered without editing presence (I-1). `kick` and `throw` are **target-less, with the object in the payload**, as `buy` is (F-O6). What a client needs to draw objects is disclosed **on the place** (F-O5). QO-6. |
| **F-O2** | **Reduction is breadth-first.** Every subscriber reacts to each fact of a generation, and what the reactions state is reduced only after the whole generation. So bodies' reactions to the walker's `arrived` and to each displaced person's `arrived` all see the objects as they lay before the request; an `object-moved` stated in reaction to one is not reduced before the next reaction runs. The walker's reaction also sees the displaced people where they stood before (F-B1). | `kernel/src/dispatch.rs:603–660` (`reduce`); `systems/presence/src/event.rs` (`arrivals` order); step-11 F-B1 | §4.5.4's reaction "against fixed geometry and people as they now stand" cannot be built as written: people are not where they now stand, and two reactions in one list could push one object twice from the same start. Each arrival's pushes are computed **against the objects as they lay before the request, with people not as obstacles**; the resolver predicts the whole list's pushes, checks them against everybody's final positions, and falls back to objects solid when they conflict (SD-O9, SD-O10). QO-9. |
| **F-O3** | **An item's sections are reduced before its place's.** Genesis states passages, then locations, then sections — items', organizations', places', people's, each in key order — and reduces the whole list as one generation, in order. | `worldpack/src/load.rs:31–33, :646`; `kernel/src/dispatch.rs:509–527` (`genesis`); `authoring/src/section.rs:129–135` (`Seeding`: no state is visible) | A fact from an item's `body:` section is reduced before the `place-shaped` of the place it names, so its reduction cannot check the object against the place's floor, solids, people or capacity, or even that the place has a shape. The checks need a **second generation**: the item's `body-formed` (generation 1), to which bodies reacts by stating `object-placed` (generation 2), reduced with every check (SD-O5). |
| **F-O4** | **One pack, one section, one authored type.** `AuthoredSection` has one `SECTION`, one `CARRIED_BY` and one `Authored` type; `owns_section!` gives a pack exactly one. | `authoring/src/section.rs:172–203`; `sdk/rust/src/pack.rs:32, :64`; step-10 F-47 | `body:` on an item file is the same section as on a place file, with one type that decodes both forms and refuses the wrong one for the file's kind (SD-O4). |
| **F-O5** | **A place may carry another file's content, joined at disclosure.** `economy` keeps a `Shop` on the Place, authored on its operator's section, and discloses the shop's listing to whoever perceives the place, built from current state at disclosure. | `systems/economy/src/component.rs:60–67`; `system.rs:255–260`; `offer.rs` (`listing`); `ARC-38` | Precedent for **`LooseObjects` on the Place**: where each object lies is the place's row, disclosed with the place; each object's shape is its own `BodyShape` on the Item, joined into the listing at disclosure (SD-O2). |
| **F-O6** | **A target-less offer can say only "at that place".** `buy_requirement(shop)` is `at_place(shop).requiring_target_available()`, with the item in the payload. `SpatialRequirement::evaluate` answers `PreconditionFailed` to a same-place or range requirement with no target. | `systems/economy/src/action.rs:40–47`; step-10 F-48; `contracts/src/spatial.rs:470–516` | `kick`'s and `throw`'s offers carry `at_place(the object's place).requiring_target_available()`; `validate` evaluates the action's declared requirement — same place, within reach — against the object's position, which only bodies knows (SD-O12). |
| **F-O7** | **Range is measured in three dimensions.** `LocalPosition::within` sums dx², dy² and dz². | `contracts/src/spatial.rs:173–180` | Reach to an object is measured to its **ground point** (z = 0), so an object's own height does not shorten it; `kick` takes only objects lying on the floor, `throw` any within reach (SD-O12). |
| **F-O8** | **The offer band, and what the paced controller does near people and objects.**<br>- One band, after the agenda and social bands and before the walking roll: the available complete affordances in observation order; draw 14 below 20 of 100 attempts one, chosen uniformly by draw 15.<br>- `approach` stops 1 000 mm short of the person approached.<br>- Nothing approaches an object, and no request sets a facing. | `cognition/rule-controller/src/offered.rs`; `paced.rs:66, :333–353` (`APPROACH_STOPS_AT`, `approach`, `wander`); `ARC-34` items 4, 6 | A shove reach of 1 000 mm makes a shove available after nearly every approach: the band will shove often, and kicks only where wandering brings people within reach of an object. The activity criteria, including an upper bound, are fixed before measuring, with a remedy ladder in content and in the pack (SD-O18, QO-13). The default throw is aimed from geometry, not from a facing nobody sets (SD-O13). |
| **F-O9** | **12b's pack, as merged, knows people only.**<br>- Its declaration provides no action and states nothing in presence's vocabulary.<br>- Its sweeps filter `only_fixed` (walls-only) and `exclude_dynamic` (people solid).<br>- Its integer fast path checks people only.<br>- `refresh` already re-marks dynamic bodies (F-P1), with its canary.<br>- `rapier_pin` holds (VERSION, Rapier) = (1, "0.36.0"); `isolation` holds "presence is the only pack dependency". | `systems/bodies/src/system.rs:49, :55–64`; `rapier.rs:128–131, :260–271`; `resolve.rs:253–260`; `tests/rapier_pin.rs`; `tests/isolation.rs` | 12c extends each, and says where (SD-O9 … SD-O16). Bodies' results change for a world with objects, and its vocabulary grows, so `VERSION` 1 → 2: a 12b save is refused by name (`ARC-25`). Two of bodies' own tests change a literal or a claim (QO-16). |
| **F-O10** | **What may be held is a declared kind.** `item::is_declared` is a living Item with an `ItemKind`. Inventory refuses a transfer, a production, a consumption and a genesis `stocked` of an item that is not a declared kind. `item` has no system dependency. | `systems/item/src/system.rs:86–92`; `systems/inventory/src/admit.rs:62–170`; `ARC-37` items 1, 3 | **An Item with a body that is not a declared kind can never be held** — inventory already refuses it, at genesis and at run time. Bodies refuses the one remaining case, an Item with both `body:` and `item:`, by asking `mineworld_item::is_declared` in its generation-2 reduction (SD-O3). That is a crate dependency on `mineworld-item`, not a system dependency: in a world without `item`, the answer is "not declared". QO-4. |
| **F-O11** | **Nothing that scans the repository objects to bodies naming items.** AC-1 check 2 requires a market crate's dependents to lie under `systems/` and no normal or build path from the framework crates (kernel, contracts, persistence, server, authoring, sdk, rule-controller) to a market pack; no framework crate depends on `bodies`. The I-2 scan reads only the 11a–11c merge ranges. `seam_vocabulary` scans presence, movement, sdk, installed and worldpack. | `tests/acceptance/tests/ac1_composability.rs:57–64, :643–654, :873–905`; `precursor_vocabulary.rs`; `seam_vocabulary.rs` | `bodies → item` and item words in bodies trip none of the three; 12c edits none of the scanned files. PO-15 runs all three unedited. |
| **F-O12** | **bodies-yard is the only world that installs bodies.** social-cafe's `systems:` lists presence, movement, conversation, group-activity, relationships, naming, schedule; market-town's has no `bodies`. | `worlds/social-cafe/world.yaml:32–39`; `worlds/market-town/world.yaml` | The towns' digests cannot change in 12c (PO-1). In bodies-yard, shove is offered from 12c on, so 12b's 30-day facts change and its scan's claim "displaced arrivals and stopped-short are stated by movement" becomes "by movement or by bodies" (QO-16). |
| **F-O13** | **12b's long run can be captured.** `long_run.rs`'s second-process mode prints the whole run's bytes (every fact and the final positions) on one line. | `systems/bodies/tests/long_run.rs:167–212` | Captured on the base before any 12c code, it is the regression reference for "a world with no objects resolves exactly as in 12b" (PO-13 b). |
| **F-O14** | **Nothing measures a single dynamic body.** The prototype's stepped mode cost ≈ 190 µs per step for the whole café with 19 bodies (§9.4 PC-d); 12b runs no step. | step-11 §9.4; `systems/bodies/src/rapier.rs` (no `step` outside the F-P1 tests) | Kick and throw are 12c's first stepping. Their cost bound is stated before measuring (PO-13, PO-14), with the scene kept to one place and one dynamic body (SD-O14). |
| **F-O15** | **What presence accepts from a resolver and from a stater.** Rule (f): `stopped_by` names an entity of this world — an Item qualifies. Rule (c) bounds a shove's arrival by the target's own position. A stating system that depends on presence may state `arrived` and `stopped-short` through `arrivals()`, and may subscribe to `arrived` too (F-R7). | `systems/presence/src/resolve.rs` (rules (a)–(f)); `ARC-39` items 3–4; step-11 F-R7 | A walker stopped by a jammed box records `stopped-short { by: the box }`. `shove` states presence's facts through `arrivals()`, so a shove is resolved, nudges and is bounded exactly as a stride is (SD-O15). |

## 18.3 Design (SD-O1 … SD-O21)

### 18.3.1 Constants (bodies' policy, published like 12b's)

Integers, in `geometry.rs`, except the three Rapier material values, which live in `rapier.rs` because no
float may appear outside it (PB-15, DB-3).

```text
OBJECT_HALF_MIN          50 mm     a box's smallest half-extent; a ball's smallest radius
OBJECT_HALF_MAX         400 mm     a box's largest half-extent in x and y; a ball's largest radius. An
                                   object touching a person is then within reach: 300 + 10 + 400 = 710
OBJECT_HALF_HEIGHT_MAX  500 mm     a box's largest half-extent in z
OBJECTS_MAX              32        loose objects lying in one place
KICK_REACH              800 mm     actor's centre to the object's ground point (§8.1)
THROW_REACH             800 mm     the same (§8.1)
SHOVE_REACH           1 000 mm     actor's centre to the target's centre (§8.1)
SHOVE_DISTANCE          500 mm     how far a shove asks to move its target (§8.1; QB-10)
KICK_SPEED            5 000 mm/s   a kicked object's initial speed: horizontal, away from the kicker
KICK_STEPS              180        sub-steps of 1/60 s at most: 3 s (§8.1)
THROW_STEPS             240        4 s (§8.1)
THROW_FLIGHT             48        sub-steps a throw's arc is aimed to take to its point: 0.8 s
THROW_DEFAULT         3 000 mm     how far beyond the object an unaimed throw is aimed
THROW_RANGE_MAX       6 000 mm     an aimed throw's point lies at most this far from the object (§8.1)
REST_SPEED               50 mm/s   an object slower than this ...
REST_STEPS               10        ... for this many consecutive sub-steps has come to rest
PATH_EVERY                6        a path keyframe every 6 sub-steps: 0.1 s (§4.7)
PATH_MAX                 40        keyframes at most (§4.7)
PUSH_SEARCH           1 024 mm     the push bisection's upper bound: above R + GAP + √2·OBJECT_HALF_MAX
                                   (≈ 876), a power of two so the bisection takes exactly ten halvings
rapier.rs only          friction 0.5, restitution 0.1, rotations locked (no rotation is ever persisted)
```

12b's constants are unchanged. Every check of an object against a person uses the person's disc of
`PERSON_RADIUS` and the object's **footprint** (a box's x–y rectangle, a ball's disc), with
`TOLERANCE` as 12b uses it.

### 18.3.2 Decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-O1** | **Ownership (I-2's form).**<br>- **bodies** owns where a loose object lies and what shape it has. They are written only by bodies' reductions: `body-formed` writes the object's shape, `object-placed` and `object-moved` where it lies.<br>- **inventory** owns holdings and never holds an object with a body (SD-O3).<br>- **presence** owns where people are, unchanged.<br>A new `objects` pack is not created (QO-1). | §4.6, frozen with the step, already gives "a loose object's body and where it lies" to bodies. An `objects` pack would own a position that only bodies can decide — every push, kick and throw would be bodies stating another pack's fact — which is two packs for one physical state, and bodies could not be installed without it. |
| **SD-O2** | **State.**<br>- **`BodyShape`** (component `body-shape`, schema 1) on an **Item**: `Box { half: { x, y, z } }` or `Ball { radius }`, millimetres, within §18.3.1's bounds. Never disclosed alone: items are never perceived (F-O1).<br>- **`LooseObjects`** (component `loose-objects`, schema 1) on a **Place**: a list of `Lying { object: ItemId, at: LocalPosition }`, sorted by `ItemId`, at most `OBJECTS_MAX`. `at` is the object's centre: x, y, and z its centre's height (a box on the floor: its half-height; on a solid: the solid's height plus its half-height).<br>- **Disclosed** to whoever perceives the place, as a listing `{ object, shape, at }` per object, the shape joined from `BodyShape` at disclosure (F-O5's precedent).<br>- **The invariant of the stored state** (checked at every write, SD-O5, SD-O6): each object lies in at most one place; its footprint is within the floor; it rests on the floor, or on one solid's top with its footprint within that top; its footprint is clear of every other solid; no two objects' footprints overlap by more than `TOLERANCE` (no stacking); no person's disc overlaps an object's footprint by more than `TOLERANCE`. | §10.2's `Lying` on the Item cannot be disclosed: perception asks only about the place and the people (F-O1), and §7.1 needs a client to draw objects where the server says (R-B4, as QP-12 did for walls). Per place is also how a resolution reads them (§5): one row, already in `ItemId` order, the canonical insertion order. No stacking keeps a push from leaving an object hanging in the air. QO-2. |
| **SD-O3** | **QB-3: an object is never held, and the crossing when it one day is.**<br>- An Item file with a `body:` section is **one physical object**, not a kind (`ARC-36` note). It has no `item:` section: bodies' generation-2 reduction refuses `object-placed` for a declared kind (`mineworld_item::is_declared`), with `bodies-held-kind`.<br>- So inventory, which takes only declared kinds, refuses every holding of it — at genesis (`stocked`) and at run time (every admit), by its own rule (F-O10). Lying and held are disjoint by construction, with no edit to inventory or item.<br>- **12c has no crossing.** `throw` picks up and throws in one action; the object never leaves `LooseObjects`.<br>- **The crossing, settled for the step that adds carrying** (QO-5). A carrying pack depends on bodies and on the holding pack. *Pick-up* states, in one emission list: bodies' `object-lifted { object, by }`, built by bodies' checked constructor `lift(world, object, by)`, which refuses unless the object lies within reach (its reduction removes the row); and the holding pack's own fact, through that pack's constructor. *Put-down* is the reverse: the holding pack's fact, then bodies' `object-set-down { object, at }` through `set_down(world, object, at)`, which refuses unless `at` satisfies the stored-state invariant. Each owner states and checks its half; one emission list is recorded whole or refused whole, so an object is never in both states. | QB-3 (decided): "the validator refusing an item that has a body and is also held". Inventory holds counts of kinds and cannot represent one unique object; that representation (`ARC-36` item 3, instances) is the first decision the carrying step needs, and it is not 12c's (§8.1). |
| **SD-O4** | **The `body:` section: one type, two forms** (F-O4). `CARRIED_BY = [Place, Item]`. The authored type denies unknown fields and has four optional keys, `floor`, `solids`, `shape` and `at`. It decodes into exactly one form:<br>- **place form**: `floor` (required) and `solids` (optional), decoded and checked exactly as in 12b, with 12b's messages;<br>- **object form**: `shape` and `at` (both required). `shape` is `{ box: { x, y, z } }` (half-extents) or `{ ball: <radius> }`. `at` is `{ place: <place key>, x, y }`. The object lies on the floor there.<br>Mixing the forms, or neither, is refused, naming the keys.<br>`references`: the object form's place, which must be a Place. `seed`: the place form on a Place states `place-shaped` (12b); the object form on an Item states `body-formed`; a form on the other kind of file is refused with `bodies-section-kind`. | One pack owns one section. A place's walls and an object's shape are both "the body" of the file they are in. Authoring an object in its own item file is QB-3 as decided (QO-3). |
| **SD-O5** | **Genesis in two generations** (F-O3).<br>- **Generation 1:** the item's `body-formed { object, shape, lies: Location }`. Bodies reduces it into the Item's `BodyShape`, after checking a living Item that has no `BodyShape` yet. Its reaction to the same fact states `object-placed`.<br>- **Generation 2:** `object-placed { object, place, at }`, reduced into the place's `LooseObjects` after these checks, in this order. Each refuses with `FactRefusedByOwner { bodies, object-placed, System { code, detail } }`, naming the object, the place and the numbers:<br>&nbsp;&nbsp;`bodies-unshaped` — the place has no `PlaceShape`;<br>&nbsp;&nbsp;`bodies-held-kind` — the Item is a declared kind;<br>&nbsp;&nbsp;`bodies-objects-max` — the place would hold more than 32;<br>&nbsp;&nbsp;`bodies-object-outside` — the footprint leaves the floor;<br>&nbsp;&nbsp;`bodies-object-in-solid` — the footprint meets a solid;<br>&nbsp;&nbsp;`bodies-object-overlap` — it overlaps an object already placed;<br>&nbsp;&nbsp;`bodies-object-on-person` — a person's disc overlaps it;<br>&nbsp;&nbsp;`bodies-capacity` — the place's capacity points (12b's: on the 650 mm grid, free of the solids) are fewer than `4 · (people − 1) + 1 + Σ blocks(o)` over the objects lying in the place, where `blocks(o) = (⌊(2·hx + 2R) / 650⌋ + 1) · (⌊(2·hy + 2R) / 650⌋ + 1)` is the most capacity points the object can cover wherever it lies (a ball: hx = hy = its radius).<br>Generation 2 runs after every generation-1 fact, so the place's shape, every person and every earlier object are in place. | The checks need the place's shape and the people, which an item's own generation cannot see. A second generation is the kernel's own ordering (`BD-7`), not a new mechanism. The capacity sum is position-free: objects never change place, so 12b's guarantee that entry placement always finds a free point survives every later push, kick and throw (SD-B4, SD-B8). |
| **SD-O6** | **Facts** (bodies' vocabulary; owner and only reducer: bodies):<br>`body-formed` — genesis, generation 1: `{ object, shape, lies }` → `BodyShape`;<br>`object-placed` — genesis, generation 2: `{ object, place, at }` → a `LooseObjects` row;<br>`object-moved` — a reaction (`pushed`) or an action (`kicked`, `thrown`): `{ object, place, from, to, how, by: PersonId, path: Vec<LocalPosition> }` → the row moved;<br>`person-shoved` — an action: `{ by, person }` → nothing; it exists for biographies, controllers and clients, as `stopped-short` does.<br>Stated by bodies in presence's vocabulary (`ARC-26`; bodies depends on presence): `arrived` and `stopped-short`, for `shove` only, through `arrivals()`.<br>Every fact is public at its place. `object-moved` is about the object and `by`; `person-shoved` about both people. Nothing is biographical in 12c (QO-17).<br>`path`: a keyframe every `PATH_EVERY` sub-steps, at most `PATH_MAX`, the first being `from`; empty for `pushed`, which is one straight line.<br>`object-moved`'s reduction is the owner's check: the object lies in `place` at `from`, and `to` keeps SD-O2's invariant against the state being reduced into. A push is reduced in the generation after the arrivals, so every person is already where they ended. | §4.7, with `body-formed` given the job F-O3 needs and `person-shoved` reduced to who shoved whom: where the shoved person ended is presence's `arrived`, and where they were asked to go is `stopped-short`'s `wanted`. Reduction checks the same invariant the resolver and the actions guarantee (`ARC-26`: the owner still decides). |
| **SD-O7** | **Declaration and install.** `depending_on([presence])`, unchanged. Owning `PlaceShape`, `BodyShape`, `LooseObjects`. Providing `Kick`, `Throw`, `Shove`. Emitting `place-shaped`, `body-formed`, `object-placed`, `object-moved`, `person-shoved`, and presence's `arrived` and `stopped-short`. Subscribing to its own `place-shaped`, `body-formed`, `object-placed` and `object-moved`, and to presence's `arrived`. `install`: `require_registered` first, then the three tables. **`VERSION` 2.** | SD-B12 said bodies states no `arrived` "because only movers do"; with `shove` it is a mover. F-R7: subscribing to a fact type one also states is allowed. The version moves because results and vocabulary change: a 12b save is refused by name (`ARC-25`). |
| **SD-O8** | **The push, one function, used twice** — by the resolver's prediction (SD-O9 step 6) and by bodies' reaction to every `arrived`.<br>`pushes(room, objects as they lay before the request, person at p)` returns, in `ItemId` order, a push for every object lying on the floor whose footprint is closer than R to p:<br>- the direction is p → the footprint's centre, made an integer unit as `at_least` does; (1, 0) when they coincide;<br>- the length is the smallest whole millimetre t in [0, `PUSH_SEARCH`] at which the footprint is at least R + `GAP` from p, by integer bisection;<br>- one Rapier shape cast of the object along that offset, against the fixed geometry and every other object as it lay before the request (F-O2). A cast cut short by more than `SNAP` is a **jam**.<br>The reaction to `arrived` (in a shaped place, with a position) states one `object-moved { how: pushed, by: the person }` per push, caused by that arrival and so by the request (`AC-9`). If `pushes` jams, the reaction fails the dispatch with `FactRefusedByOwner { bodies, object-moved, System { code: bodies-object-jammed } }`, naming the object and the person. For an arrival the resolver resolved this never happens (SD-O9); reaching it means an arrival escaped resolution — `ARC-39` item 7's guard, which for objects can be built at the reaction because the push is computed from the arrival and the objects alone.<br>Inert: an arrival without a position, a place without a shape or without objects, an object on a solid (a person cannot reach it, SD-O2). Genesis arrivals push nothing: no object is placed before generation 2. | §4.5.4 as the operator bound it, made buildable under F-O2. Every input of a push is the same in the prediction and in the reaction: the person's end point, the room, the objects as they lay before the request. So what the reactions push is what the resolver checked. People are not obstacles to the cast, because in the reaction they are not yet where they end (F-B1); the resolver checks the final positions instead. |
| **SD-O9** | **The resolver, extended** (SD-B6 with objects; order unchanged except where objects enter):<br>1. **Inert rule, SD-B15, guard.** The guard (SD-B9) also checks the stored-state invariant for objects (SD-O2), and panics naming the person and the object, the place and `ARC-39`.<br>2. **Fast path:** the corridor is clear of people (12b) **and** of every object's footprint grown by R + `GAP` (integer segment-to-footprint distance) → unchanged.<br>3. **Head-on bias:** unchanged; people only.<br>4. **Sweeps:** the scene adds the objects as fixed colliders in their own group (SD-O16). W, walls-only: the fixed geometry, objects excluded. B, contact: the fixed geometry, the people and the objects. The candidate rule is unchanged, so a walker advances at most `NUDGE_MAX` past first contact with a person **or an object**.<br>5. **Nudge pass:** unchanged.<br>6. **Objects** (§4.5.1 step 6): `pushes` for the walker at the candidate, then for each displaced person, in emission order, each against the objects as they lay. Accepted iff no object is pushed twice, nothing jams, and in the final state — everybody where they end, pushed objects at their ends, the rest where they lay — no person overlaps an object and no two objects overlap. Otherwise **objects solid**: steps 4–5 again with the objects counted as fixed geometry for W and for the nudges, and nothing pushed.<br>7. **Quantize and snap;** **verify** V1–V3 and **V4** (no person's disc overlaps any object's footprint, as the objects will lie). **Degrade** as 12b — blocked at first contact with a person or an object, then halved, then stay — with the objects where they lay and V4 checked.<br>8. **`stopped_by`:** the first person or object touched.<br>**Entry placement:** E1's free test and E3's lattice search also require the person's disc to be clear of every footprint by R; E2 runs only when people alone make `to` unfree, and its result must pass V4. An entry pushes nothing. | The operator's binding: the resolver runs the same reaction sequence first, so the walker never ends inside a jammed box. Steps 2 and 4 stop tunnelling: a walker can pass at most 300 mm into an object before the candidate rule stops it, and a corridor through an object is never the fast path. Staying is still valid by induction: the starting state passed the guard, V4 included. |
| **SD-O10** | **Why prediction and reaction agree.** Both call `pushes` with the same three inputs, in the same order — the emission list's — and at the reactions' generation no `object-moved` has yet been reduced, so the objects are exactly where they lay before the request (F-O2). The prediction accepts only push sets that are conflict-free and whose final state keeps the invariant. Otherwise nobody ends overlapping an object, so the reactions push nothing. | The one property §4.5.4 rests on, stated so a test can hold it (PO-3 f, M-PO3). |
| **SD-O11** | **`kick { object: ItemId }`**, target-less (F-O1, F-O6).<br>Declared requirement: `same_place().within(KICK_REACH).requiring_target_available()`, which `validate` evaluates against the object's ground point.<br>`validate`, in order:<br>- the payload decodes;<br>- the actor is a living Person with a position (`PreconditionFailed`);<br>- the object lies somewhere — an Item with a `BodyShape` in some place's `LooseObjects` (`TargetUnavailable` otherwise);<br>- it lies on the floor (`TargetUnavailable` for an object on a solid);<br>- the requirement: another place, or beyond 800 mm → `TooFarAway`.<br>`resolve`: SD-O14's flight, launched at `KICK_SPEED` along the integer unit from the kicker's centre to the object's (fallback (1, 0)), horizontal, at most `KICK_STEPS`; states `object-moved { how: kicked, by: the kicker }`.<br>A world without bodies answers `ActionResult::Unavailable` (I-9). | §8.1: "direction from the kicker's centre to the object's, so it needs no aim". No direction parameter, so one complete offer per object (QO-8). Kicking a box off a table is not a kick. |
| **SD-O12** | **The offers** (complete affordances, `ARC-34`). Nothing to a positionless observer, and nothing in a place without a shape.<br>- **Target `None`:** for each object in the observer's place whose ground point lies within 800 mm, in `ItemId` order, the kicks first and then the throws:<br>&nbsp;&nbsp;`kick { object }`, only for an object on the floor;<br>&nbsp;&nbsp;`throw { object, toward: null }`.<br>&nbsp;&nbsp;Each has the requirement `at_place(place).requiring_target_available()` and is available. Objects beyond reach are **not offered** (QO-7).<br>- **Target a person** — a different, living Person present: `shove {}`, with requirement `same_place().within(SHOVE_REACH).requiring_target_available()`. It is available iff the target stands at a position in a shaped place; perception prices `TooFarAway`.<br>- Never a shove to oneself (step-10 F-40). | F-O1 and F-O6 decide the shape: a target-less offer can say only "at that place", so reach is decided by which objects are offered rather than misreported as `TargetUnavailable`, and a direct request beyond reach is answered `TooFarAway` by `validate`. An observation grows by at most two offers per object in reach (`ARC-34`'s limitation). |
| **SD-O13** | **`throw { object, toward: Option<{ x, y }> }`**, target-less.<br>The **complete form** is `toward: null`, which means the default aim: along the thrower's centre → the object's centre (fallback (1, 0)), `THROW_DEFAULT` beyond the object, pulled back along that line into the floor shrunk by the object's half-size plus `GAP`. An aimed throw names its point in the thrower's place frame.<br>Declared requirement: as kick's, with `THROW_REACH`.<br>`validate`: as kick's, but any lying object within reach, on the floor or on a solid. With `toward` given: the point inside the floor shrunk by the object's half-size, and within `THROW_RANGE_MAX` of the object's ground point; otherwise `PreconditionFailed`.<br>`resolve`: the object is launched from where it lies (QO-11). Its velocity is aimed to reach the point after `THROW_FLIGHT` sub-steps:<br>- in the plane, (point − start) / T;<br>- in z, (z_rest − z_start) / T + g·T / 2, with T = `THROW_FLIGHT` / 60 s.<br>Only + − × ÷ are used, on values converted from integers (DC-3). SD-O14's flight follows, at most `THROW_STEPS`, and states `object-moved { how: thrown }`. | §7.2 had `throw` carry a free-form point, which no offer can enumerate. An optional point gives one complete offer per object with a meaningful default, while a client that aims still sends the point. No combinatorial offers (QO-8). The aim does not use a facing, because nothing headless sets one (F-O8). |
| **SD-O14** | **Flight and landing** (kick and throw; QB-6: resolved at the instant).<br>A flight scene for the place: the fixed geometry; the people as kinematic capsules (`EntityId` order); the other objects as fixed colliders (`ItemId` order); the flying object last, dynamic, rotations locked, with CCD. Then `refresh` (F-P1's re-mark) and the launch velocity. Step at 1/60 s until the object has been slower than `REST_SPEED` for `REST_STEPS` consecutive sub-steps, or the step bound. A keyframe every `PATH_EVERY` sub-steps. Quantize the end.<br>**Verify (V-O):** the stored-state invariant of SD-O2 for this object — within the floor; resting on the floor or a solid's top within `TOLERANCE`; clear of other solids; no overlap with any person's disc or any other object.<br>**Degrade:** (1) the simulated end, if it verifies; (2) else the nearest point of the 50 mm `LATTICE` on the floor to the end's ground point that verifies, ordered by (distance², y, x); (3) else the object stays, and `object-moved` records `to` = `from`.<br>The path is the simulated flight in every case; `to` is the authoritative end. Nothing else moves: people are kinematic, and other objects are fixed for the flight. | DC-4's bounded sub-steps and rest test, both pure functions of the state. CCD stops a fast object passing a thin solid; its determinism is part of what PO-11 and PO-12 measure. Verify-then-degrade is DC-8 applied to objects: the engine's answer is never the last word. Rung (3) is valid by induction, and a kick that moves nothing is still a true fact about a kick. Other objects fixed during a flight is R-3: no chain reactions (QO-10). |
| **SD-O15** | **`shove {}`**, targeting a Person.<br>Declared requirement: `same_place().within(SHOVE_REACH).requiring_target_available()`.<br>`validate`, in order:<br>- the payload decodes;<br>- the actor is a living Person with a position (`PreconditionFailed`);<br>- the target is present, a different living Person (`NoSupportedInteraction`);<br>- available iff the target stands at a position in a shaped place (`TargetUnavailable`);<br>- the requirement (`TooFarAway`).<br>No `Busy` (QB-8).<br>`resolve`: d = target − actor, which is never zero (CLEARANCE); `to` = target + d scaled down to `SHOVE_DISTANCE` in integers, keeping z and facing. The facts are `person-shoved { by, person }`, then presence's `arrivals(world, target, to)`, a refusal becoming `FactRefusedByOwner` as in movement. The shoved person's arrival is resolved by bodies' resolver like any other: walls stop them, the people behind are nudged within I-11, and objects are pushed or block. | §8.1 and QB-10: shove is the larger, deliberate displacement, under the same bounds. Through `arrivals()`, presence still decides and the log holds only true arrivals (I-6). Rule (c) bounds it from the target's own position (F-O15). The head-on bias may turn a shove that would drive the target straight into somebody; that is the resolver's rule for every arrival. |
| **SD-O16** | **The Rapier adapter** (`rapier.rs` only; I-5).<br>- The canonical order gains the objects: slab, walls, solids, people (`EntityId`), objects (`ItemId`), and in a flight scene the flying object last.<br>- Objects are cuboids or balls in an object collision group. The query filters are walls-only (fixed, not objects), contact (fixed, people and objects) and objects-solid walls (fixed and objects).<br>- New calls: a shape cast for a push, and a flight (build, refresh, set velocity, step loop with the rest test and keyframes).<br>- Friction 0.5, restitution 0.1, rotations locked, CCD on the flying object.<br>- Integers in, integers out; nothing kept. | DEP-13's isolating interface, extended without exposing a Rapier type (PB-15 still holds). |
| **SD-O17** | **Integer geometry** (`geometry.rs`, no float):<br>- footprints, and the distance² from a point to a footprint and from a segment to a footprint;<br>- the push bisection;<br>- V4 and V-O;<br>- the object lattice search;<br>- `blocks(o)` for capacity. | DC-3, DC-6: only the cast and the flight are float. |
| **SD-O18** | **Activity, decided before measuring** (`ARC-23`; the I-9-like rule). The paced controller is unchanged (no diff under `cognition/`). The criteria (PO-9) bound activity from below **and from above**, so that "people kick and shove constantly" is a failure, not a tuning. If the run fails them, the remedies are tried in this order, each recorded as a bounded deviation, with the criteria unchanged:<br>- **c1, content:** move the objects onto the people's paths (beside the doorway points and around the table), up to 8 per room;<br>- **p1, the pack's offer policy:** offer `shove` complete only when the target stands within 800 mm; the request's requirement stays 1 000 mm;<br>- **p2:** offer `kick` and `throw` complete only for the one nearest object.<br>If none passes, the question returns to the primary session with the counts. The controller is never the remedy. | `ARC-34` item 4: if a world behaves badly, the remedy is in what its packs offer. F-O8 predicts shove's frequency; an upper bound makes that prediction checkable. |
| **SD-O19** | **The test world: `worlds/bodies-yard`, extended** (QO-15).<br>- `items:` gains seven objects, each `items/<key>.yaml` with tags and a `body:` in object form; no `item:` section; the `item` pack is not installed.<br>- The hall: two boxes, half 200 mm; a crate, half 300 mm; two balls, radius 110 mm.<br>- The court: a box, half 200 mm; a ball, radius 110 mm.<br>- Every object at least 700 mm from every authored person and from the doorway points, and clear of the table and the pillar.<br>- README: the objects and the three actions. | The world already runs every path 12c needs: the real loader, `validate`, `mineworld run` with the unchanged controller, `--save`, SIGKILL and resume, and world-level removal. Shove changes it in any case (F-O12). A sibling world would duplicate the harness and re-prove nothing new. |
| **SD-O20** | **What changes in a save.** bodies `VERSION` 2: a save written by 12b is refused at bodies' record, "system 'bodies' is v2 here, but the save was written by the older v1". No other pack's version changes. | `ARC-25`: versions are refused, never guessed. |
| **SD-O21** | **Documents** (PO-C1, before code):<br>- **`ARC-36` note** (QB-3): an Item file with a `body:` section is one physical object, not a kind; it is never a declared kind, so it is never held; instances in general are still out.<br>- **`ARC-39` note 2:** the resolver predicts object pushes and falls back to objects solid (SD-O9, SD-O10); the reaction pushes against the objects as they lay (F-O2); the guard covers objects; `shove` states through `arrivals()`.<br>- **`DEP-13` note:** dynamics are now used — one dynamic body per flight, rotations locked, bounded steps, the rest rule, CCD, F-P1's re-mark exercised.<br>- **`MODULE_SPEC.md` §4.1:** the `body` row's object form; the item-file paragraph ("an item file declares an item kind, unless it carries `body:`"); an item example.<br>- `systems/bodies/README.md` and `lib.rs`'s table; `MVP_STATUS.md`. | `CLAUDE.md` §2.2: the documents come first. QB-3 said "amending ARC-36's wording (in 12c)". |

**Note on SD-O13 (and SD-O11), 2026-10-08 — rung p3, by the primary session's ruling on the AO-2
stop (§18.11 DO-16).** An unaimed kick or throw — a kick always, a throw with `toward: null`, the
complete affordance's default — sends the object toward the room's **free centre**: the floor
rectangle's centre, or, when the object's footprint there would meet a solid, the nearest point of the
object's 50 mm lattice where it meets none, ordered by (distance², y, x). A kick launches along object →
free centre at `KICK_SPEED` (an object already there goes along kicker → object, then east). An unaimed
throw aims at the free centre if it is within `THROW_DEFAULT`, else `THROW_DEFAULT` along the line to it.
The object still launches from where it lies; reach is still measured from the kicker; an aimed throw is
unchanged. Rungs p1 stays; p2 stays reverted. The AO criteria are unchanged, and must hold on seeds 7,
8 and 9.

**Note on SD-O13, 2026-10-08 — rule p4, by the primary session's ruling on the second AO-2 stop (§18.11
DO-17): a launched object never comes to rest within 300 mm of a solid unless its flight was blocked.**
`REST_CLEARANCE` = 300 mm, fixed before measuring (a person's radius). (1) The free centre is the
nearest lattice point whose footprint keeps 300 mm from every solid (a ball's centre 300 mm + its radius
away), same ordering. (2) A flight whose end rests on the floor within 300 mm of a solid is pulled back
along its line toward where it began, in 10 mm steps, to the first point that keeps the clearance; when
no point of the line does, the flight was blocked and the end stands. (3) The landing ladder's lattice
rung prefers clear points. An end on a solid's top is unaffected.

## 18.4 Acceptance (decided before measuring, `ARC-23`)

Rules for every criterion below, as in §17.4:
- Each guarded criterion names the mutation shown to break it. A mutation is applied in the working
  tree, observed to fail by name, and reverted. `git status` and `git grep MUTATION` are recorded
  afterwards.
- Every expected position is a literal computed by hand from the test's own layout, never by the code
  under test (test rules §25). Positions that pass through a Rapier sweep or cast are asserted to
  within ±1 mm of the hand-computed literal. Positions that pass through a **flight** cannot be
  hand-computed to the millimetre; they are asserted against **bounds fixed here**, and their bytes
  against a second process.
- The scenario room is 12b's: the prototype's café, floor (0, 0)–(8 320, 10 320), the counter (3 860,
  6 570)–(8 320, 7 170) at height 1 100. A box "half 200" is a 400 mm cube; a ball "r 110" has radius
  110 mm.

```text
PO-1  Nothing existing moves. On the PR head:
        - `mineworld run worlds/social-cafe --headless --seed 7 --days 300`: faults 0, 365 330 facts,
          sha-256 of every line but `wall` = ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b;
        - `mineworld run worlds/market-town --headless --seed 7 --days 300`: faults 0, 372 755 facts,
          sha-256 = 365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d;
        - `mineworld validate` of both worlds byte-identical to the base binary's output;
        - every existing test passes; the only edits to existing tests are those QO-16 names.
      (Neither town installs bodies, F-O12; the guard on bodies' own inertness is PO-17 and M-PO1.)
PO-2  Objects at load (SD-O4, SD-O5), through the real `mineworld validate` on test-time copies of
      worlds/bodies-yard (tools/cli/tests/bodies_yard.rs). Each refusal exits non-zero and names its
      subject and the numbers:
        - a ball whose place has no `body:` (the copy strips the court's) → "bodies-unshaped";
        - a ball 200 mm from a person → "bodies-object-on-person";
        - a box overlapping the table → "bodies-object-in-solid";
        - a ball 50 mm from the floor's edge → "bodies-object-outside";
        - two balls 100 mm apart → "bodies-object-overlap";
        - a hall too small for its people and objects → "bodies-capacity", with the counts;
        - `shape: { ball: 0 }`, `shape: { box: { x: 500, y: 100, z: 100 } }`, an unknown key, and a
          section with both `floor` and `shape` → each refused by the loader at its line and column,
          with bodies' own message;
        - the object form in a place file, and the place form in an item file → "bodies-section-kind".
      Positive control: worlds/bodies-yard itself validates.
      M-PO2  the person-overlap check removed → the 200 mm copy validates.
PO-3  Walking pushes objects (systems/bodies/tests/objects.rs: a hand-built world with presence,
      movement and bodies, through the real World::dispatch, [bodies] registered). Facts are checked
      in order, with causes: a pushed object's `object-moved` is Causation::Event(the pushing
      arrival), with the walker's ActionId as its decision (AC-9).
        a  push: the walker at (2 800, 5 000) asks +500 mm in x; a box half 200 stands at
           (3 500, 5 000), its face at x = 3 300. Contact is at 2 990 (face − R − GAP); the candidate
           rule stops the walker 300 mm past it, at (3 290 ± 1, 5 000); the box is pushed +x until its
           face is 310 mm from the walker's centre, to (3 800 ± 1, 5 000); stopped-short { wanted
           (3 300, 5 000), reached, by: the box }.
        b  jam: a box half 200 stands with its face on the east wall, at (8 120, 5 000); the walker
           at (7 300, 5 000) asks +500 mm. The push would jam, so the stride is resolved with objects
           solid: the walker at (7 610 ± 1, 5 000), the box unchanged, stopped-short by the box.
        c  no tunnelling: a ball r 110 at (4 000, 5 000); the walker at (3 000, 5 000) asks +2 000 mm.
           The walker ends at (3 880 ± 1, 5 000) — 300 mm past contact — and the ball is pushed to
           (4 300 ± 1, 5 000); it is never passed over.
        d  the fast path: a stride whose corridor stays 311 mm clear of every footprint → exactly
           `to`, one fact, payload bytes equal to Arrived::new(person, to)'s encoding.
        e  a nudged person pushes: the walker nudges B, and B's new disc overlaps a ball beside B →
           the ball's `object-moved` is by B, caused by B's `arrived`; nobody overlaps the ball.
        f  conflict: the walker and a nudged person would both push one ball → the stride is
           resolved with objects solid (the resolver's own account, `explain`, says so); the ball is
           unchanged; nobody overlaps it.
        g  never through a solid: a box pushed toward the counter jams like (b).
      After every request: V1–V4 hold, and every object keeps SD-O2's invariant.
      M-PO3  the prediction (SD-O9 step 6) skipped → (b) fails: the walker is recorded at 7 800, and
             bodies' reaction refuses with `bodies-object-jammed`.
      M-PO4  the fast path and the contact sweep ignore objects → (c) fails: the walker reaches
             (5 000, 5 000) and the ball is not where (c) says.
      M-PO10 a push set with one object pushed twice is accepted → (f) fails.
PO-4  kick (same file, and the scenarios' world):
        a  reach: the kicker at (2 000, 5 000); a ball r 110 at (2 800, 5 000) is kicked (Accepted); at
           (2 801, 5 000) the answer is Rejected(TooFarAway).
        b  refusals: a ball in the other place → TooFarAway; an Item without a body → TargetUnavailable;
           a box lying on the counter → TargetUnavailable; a positionless kicker → PreconditionFailed;
           a malformed payload → the codec's refusal, as item-transfer's.
        c  open floor: the kicker at (2 000, 5 000), a ball r 110 at (2 700, 5 000) → one fact,
           `object-moved { how: kicked, by: the kicker }`, Causation::Action, emitted_by bodies. Bounds
           fixed now: the ball's x grows by 1 500 … 3 500 mm, |Δy| ≤ 50 mm, z = 110 ± 5; `path` has
           1 … 40 keyframes, the first equal to `from`, every one inside the floor; nobody moved.
        d  a wall: the ball at (7 500, 5 000), kicked +x from (6 800, 5 000) → it ends inside the floor,
           x ≤ 8 320 − 110 + 5, and keeps SD-O2's invariant.
        e  a person in the way: C stands at (4 000, 5 000); the kick of (c) → the ball ends at least
           405 mm from C's centre; C has no new fact.
        f  the fallback: under a test policy that cuts a throw's flight after 2 sub-steps, the object
           is still in the air; the landing is the nearest free lattice point to its ground point, at
           rest height (a literal of the 50 mm lattice).
        g  without bodies: a world of presence and movement answers `kick`, `throw` and `shove`
           ActionResult::Unavailable (I-9).
      M-PO5  verification of a landing (V-O) off → (f) fails: the object is recorded in the air.
PO-5  throw:
        a  complete form: the thrower at (2 000, 5 000), a ball r 110 at (2 600, 5 000), `toward: null`
           → the default aim is (5 600, 5 000). Bounds fixed now: the ball ends with x in 4 600 … 7 600
           and |Δy| ≤ 100, at rest on the floor; one fact, how: thrown.
        b  aimed: `toward: (2 600, 8 600)` from the same start → it ends within 1 500 mm of the point,
           at rest, on the floor or on the counter, keeping SD-O2's invariant.
        c  refusals: `toward` outside the floor → PreconditionFailed; `toward` 6 001 mm from the ball →
           PreconditionFailed, 6 000 mm → Accepted; reach as PO-4 a.
        d  onto the counter: aimed at a point on the counter top → the ball ends either on its top
           (z = 1 100 + 110 ± 5, footprint within the counter) or on the floor; never inside it. Which,
           is recorded.
        e  into people: aimed at a point among three standing people → it ends overlapping none of them
           (≥ 405 mm from each centre), and none of them moves.
PO-6  shove:
        a  A at (3 400, 5 000) shoves B at (4 000, 5 000) → facts in order: person-shoved { by A,
           person B }, arrived B (4 500, 5 000); each Causation::Action(A's ActionId), emitted_by
           bodies; no stopped-short.
        b  a wall: A at (7 100, 5 000), B at (7 700, 5 000) → B arrives at (8 010 ± 1, 5 000), with
           stopped-short { B, wanted (8 200, 5 000), reached, by None }.
        c  into a person: C stands off B's line beyond BIAS_BAND, 541 mm from B's end point → C is
           nudged to a hand-computed literal, ≤ 310 mm, and recorded as a displaced `arrived` in the
           same list; every pair ≥ 595 mm afterwards.
        d  into an object: a ball behind B → pushed by B, caused by B's `arrived`.
        e  refusals: B at 1 000 mm → Accepted, 1 001 mm → TooFarAway; oneself → NoSupportedInteraction;
           a positionless B, or B in a place without a shape → TargetUnavailable; B in another place →
           TooFarAway; a positionless A → PreconditionFailed.
      M-PO7  SHOVE_DISTANCE 500 → 800 → (a) fails, naming B's x.
      M-PO8  shove states Arrived::new directly, bypassing arrivals() → (c) fails (C not nudged).
PO-7  The offers (pack test through presence's observe, with bodies as a provider). The observer at
      (2 000, 5 000); a ball r 110 at (2 700, 5 000) (700 mm); a box half 200 at (2 000, 5 900)
      (900 mm); P at (2 900, 5 400) (985 mm); Q at (3 200, 5 000) (1 200 mm):
        a  exactly these affordances from bodies, in this order, with these JSON payloads: kick { ball }
           and throw { ball, toward: null } (target none, requirement at_place(the room), available);
           shove → P (available); shove → Q (unavailable, TooFarAway). None for the box, none to
           oneself.
        b  each available affordance, submitted unchanged through Affordance::request, is Accepted
           (in a fresh copy of the world each).
        c  in a place without a shape, and for a positionless observer: no affordance from bodies.
      M-PO11 objects beyond reach offered → (a) fails, naming the box.
PO-8  Disclosure (pack test): an observer in the hall is told the hall's `loose-objects` listing — each
      object, its shape and its position, as authored — and nothing of the court's.
PO-9  The 30-day real run (tools/cli/tests/bodies_yard.rs): `mineworld run worlds/bodies-yard
      --headless --seed 7 --days 30 --save S` → exit 0, faults 0.
      Activity first (ARC-23; SD-O18), per 10-day bucket, fixed now:
        AO-1  every seat moves and talks;
        AO-2  at least one accepted kick, one throw, one shove, one `object-moved { pushed }` and one
              stopped-short by an object;
        AO-3  for every seat, kicks + throws + shoves accepted ≤ 144 — a tenth of the seat's 1 440
              consults: nobody does something physical more than once every 100 simulated minutes
              on average.
      The counts are printed. Then the scan, replaying S's facts in EventId order and checking after
      each request:
        - people: no pair in one place closer than 595 mm; no centre outside the floor shrunk by
          295 mm or within 295 mm of a solid; every displaced arrival (an `arrived` after the first in
          a request's list) stays in its place, moves ≤ 310 mm, at most 4 per request, and is stated by
          movement (a move) or by bodies (a shove); a shoved person's own arrival moves ≤ 500 mm;
        - objects: each within its floor, at rest on the floor or a solid's top, out of every other
          solid, ≥ 295 mm from every person's centre measured to its footprint, overlapping no other
          object;
        - every `object-moved` starts where the scan last saw the object; a pushed one is caused by an
          `arrived` of the same request, a kicked or thrown one by the request itself.
      It prints the closest person–person and person–object approaches, with their place and request.
PO-10 The instrument sees, and bodies can be removed (I-10; AC-2 at world level). A test-time copy
      drops `bodies` from `systems` and every `body:` section, places' and items'. It runs 30 days with
      faults 0, every seat moves, and no kick, throw, shove, `object-moved` or `person-shoved` is
      recorded (nothing offers them). The scan reports a person–person overlap, naming the pair.
PO-11 Determinism (tools/cli/tests/bodies_yard_restart.rs, as PB-11, on the extended world): activity
      first — the control holds kicks, throws, shoves and pushes, each > 0; two processes for 30 days
      byte-identical; SIGKILL at days 5, 15 and 25, each survivor resumed at the head on disk with a
      re-executed tail > 0 at least once, byte-identical to the control; `mineworld replay` from
      genesis reproduces every fact.
      M-PO9  an impure flight (a process-global counter's parity added to KICK_SPEED) → a survivor is
             refused (ReplayDiverged), or its rows differ.
PO-12 Cross-architecture (recorded real evidence, as PB-12): the 30-day summary sha-256 of the extended
      bodies-yard equal on arm64 and x86_64 under Rosetta; an arm64 save at day 15 resumed by x86_64 to
      day 30, and the reverse, each equal to the uninterrupted run. The fallback of QP-16 applies.
PO-13 Long runs (systems/bodies/tests):
        a  long_run_objects.rs: the prototype's 3 000-request sequence (§9.5) with its four props —
           two boxes half 200, a ball r 110, a crate half 300 — and its kicks as `kick` requests.
           After every request N-1 … N-4 hold and every object keeps SD-O2's invariant; the closest
           person–object approach is located; the final bytes are identical in a second process.
           Release (`cargo test --release -p mineworld-bodies --test long_run_objects -- --nocapture`)
           prints the mean time per move that reaches Rapier — PASS ≤ 100 µs — and per kick —
           PASS ≤ 2 000 µs.
        b  long_run.rs (12b's, no objects) unchanged: its second-process bytes on the PR head equal the
           bytes captured on the base before any 12c code (E-PO-base). A world without objects is
           resolved exactly as in 12b.
      M-PO12 NUDGE_MAX 300 → 299 → (b) fails: the bytes differ from the base's. This shows the
             comparison sees a change to the people path; that objects leave it alone where there are
             none is what (b) then holds.
PO-14 Cost (recorded real evidence; QB-11 stays 12d's): 300 days of the extended bodies-yard, seed 7,
      dev profile, as `mineworld run` is measured → faults 0 and wall time ≤ 40 s on this machine (12b's
      guard of 25 s, plus 15 s for objects and the three actions, fixed now). The ratio to PO-10's copy
      is recorded as information.
PO-15 Structural (bodies' own tests and the existing scans):
        - only systems/bodies/src/rapier.rs names rapier; no f32 or f64 outside it;
        - bodies' system dependency is presence alone; its pack crates are presence and item (QO-4);
        - rapier_pin holds (bodies VERSION, rapier3d) = (2, "0.36.0");
        - ac1_composability 13/13, precursor_vocabulary 4/4 and seam_vocabulary 3 pass unedited;
          presence's and movement's structural scans pass unedited;
        - `git diff --stat <base>...HEAD -- cognition/` is empty: the controller is unchanged.
PO-16 QB-3, end to end (validate on copies):
        - a copy that installs `item` and gives the hall's ball an `item: { category: toy }` section →
          refused "bodies-held-kind", naming the ball;
        - a copy that installs `item` and `inventory` and gives ada `holdings: { <the ball>: 1 }` (no
          `item:` on the ball) → refused at load by inventory's `stocked` (not a declared kind).
PO-17 Inert where absent (I-13, as PB-19): in a world with bodies installed, a place without `body:`
      offers nothing from bodies, its `move`s record exactly Arrived::new's bytes, and `kick` of an
      object there is impossible (no object can lie there, PO-2).
      M-PO1  shove offered in a place without a shape → the first bullet fails.
PO-18 Scope and the gate: `git diff --name-only <base>...HEAD` ⊆ §18.1's change set; the no-diff paths
      empty; Cargo.lock changes only mineworld-bodies' dependency list; `cargo fmt --check` and
      `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean; the full workspace
      gate runs once, on the final executable head.
```

## 18.5 Commit plan

Rules for every commit, as in §17.5:
- Each commit tracks implementation, validation and review separately.
- Evidence goes into §18.10 as `E-PO<n>`, deviations into §18.11.
- A planned commit may become several coherent commits; the mapping is recorded.
- Each commit leaves the workspace's tests green.
- Commands run from the worktree root, with `$HOME/.cargo/bin/cargo` if `cargo` is not on PATH.
- Anything longer than about two minutes runs in the background: the x86_64 build, 300-day runs, the
  CLI test binaries, the full gate.
- Every file change is made with the Edit and Write tools; no `sed -i`, `awk`, heredoc appends or inline
  `python3 -c` (DB-10's tool-discipline note).

### PO-C0 — Design (this section) — docs only

- [x] Implementation: §18 and the header line, by the planning session on `mvp0/s15-12c-plan`, from the
  audit in §18.2.
- [x] Validation: `python3 scripts/check_doc_headings.py` and `python3 scripts/check_decision_ids.py`
  (E-PO0).
- [x] Review: every claim in §18.2 cites a file and a line, or a section of this document. Each departure
  from §§4.5.4, 4.6, 4.7, 7.2, 8.1 and 10.2 is named with its finding and its question. This is the
  planning session's self-review; the freeze is the primary session's.

### PO-C1 — Specs before code, and the base captures

**Goal.** QB-3's amendment, the object rules and the dynamics are reviewable decisions before any code
relies on them (`CLAUDE.md` §2.2). The references that 12c must not move are captured before it can
move them.

**Scope.**
- `docs/DECISIONS.md`: dated notes on **ARC-36** (QB-3), **ARC-39** (note 2: objects and shove) and
  **DEP-13** (dynamics), each as SD-O21 states it, without rewriting the records.
- `docs/MODULE_SPEC.md` §4.1: the `body` row's object form and its refusals; the item-file paragraph;
  a commented item example with `body:`.
- The handoff, reinitialized for 12c.
- **E-PO-base**, before any code, on the base:
  - the base binary (`cargo build -p mineworld-cli`, copied to `/tmp/s15-12c/base-mineworld`);
  - `validate` of social-cafe and market-town with it, kept as PO-1's reference;
  - the long run's bytes: `BODIES_LONG_RUN_SECOND_PROCESS=1 cargo test -p mineworld-bodies --test
    long_run -- --nocapture`, the `LONG-RUN` line kept, with its sha-256 (PO-13 b);
  - for information only, the 12b bodies-yard 30-day summary sha-256 (E-PB8's `3a2c3322…`) re-run.

**Depends on:** freeze. **Non-goals:** no code.

- [x] Implementation: the three notes, MODULE_SPEC §4.1, the handoff; E-PO-base captured (E-PO-base;
  ARC-36 note, ARC-39 note 2 with the `is_declared` crate dependency named — §18.0 condition (a) —,
  DEP-13 note; MODULE_SPEC §4.1's body row, item example and item-file paragraph).
- [x] Validation: both doc checks pass; the decision-id count is unchanged (notes add no id); every
  captured file exists, with its sha-256 recorded (E-PO-base, E-PO1).
- [x] Review (self, 12c session): the ARC-36 note states QB-3 — one physical object, never a declared
  kind, never held — and keeps instances out (its point 3); `Item` keeps CORE_CONCEPTS §7's meaning
  ("unique items" is a reading it allows); ARC-39 note 2 states SD-O8 … SD-O10 (point 1), the guard
  for objects (point 2, item 7 refined for objects only), shove through `arrivals()` with QO-18
  (point 3), and QO-4's one read (point 4). Original review item: the ARC-36 note says exactly what QB-3 decided and nothing more (instances in general
  stay out); no defined term is redefined (`Item` keeps `CORE_CONCEPTS.md` §7's meaning: "unique
  items" is one of the readings it already allows); the ARC-39 note states SD-O8 … SD-O10 and refines
  item 7 for objects only.

**Commit boundary.** Documentation only.

### PO-C2 — Objects at genesis: the section, the two generations, the state, disclosure

**Goal.** A world can author loose objects, and a world whose objects do not fit does not load (PO-2's
pack half, PO-8, PO-16's first bullet).

**Scope.**
- `systems/bodies/src/section.rs`: the two-form type (SD-O4), `CARRIED_BY = [Place, Item]`,
  `references`, `seed`.
- `src/component.rs`: `BodyShape`, `LooseObjects`, `Lying`, with their authored forms and bounds.
- `src/event.rs`: `BodyFormed`, `ObjectPlaced`, and their constructors.
- `src/system.rs`:
  - the declaration of SD-O7, as far as these facts go;
  - `VERSION` 2;
  - install with three tables;
  - the reductions of `body-formed` (and its reaction stating `object-placed`) and of `object-placed`,
    with SD-O5's checks in order;
  - disclosure of the listing.
- `src/geometry.rs`: footprints, the stored-state invariant, `blocks(o)`, the capacity sum.
- `systems/bodies/Cargo.toml`: `mineworld-item`, with its reason (QO-4); `Cargo.lock`.
- Tests:
  - `tests/genesis.rs`: each refusal of SD-O5 in a hand-built genesis, naming its subject; PO-8;
  - `tests/rapier_pin.rs`: the pair becomes (2, "0.36.0") (QO-16);
  - `tests/isolation.rs`: the dependency claim becomes "system dependency presence; pack crates
    presence and item" (QO-16).

**Depends on:** PO-C1.

**Failure and edge cases.** An object authored in a place listed after it (generation 2 sees it); two
items naming the same place (both placed, in key order); an Item with a `body:` in a world without
`item` (not declared: placed); a place with 32 objects (placed) and 33 (refused).

- [x] Implementation: as scoped, with DO-1 and DO-2's file layout (E-PO2): `section.rs` (`Body`, two
  forms), `component.rs` (`BodyShape`, `LooseObjects`, `Lying`), `event.rs` (`BodyFormed`,
  `ObjectPlaced`, and `ObjectMoved`/`PersonShoved` types), `genesis.rs`, `objects.rs`, `footprint.rs`,
  `system.rs` (VERSION 2, three tables, reductions, the listing), Cargo.toml (`mineworld-item`).
- [x] Validation: `cargo test -p mineworld-bodies` (every 12b test still green, `rapier_pin` and
  `isolation` with QO-16's edits); M-PO2's pack half fails a genesis test by name, reverted; M-ITEM
  (§18.0 (b)) bites; clippy and fmt clean (E-PO2).
- [x] Review (self): writes only in `react` — `PlaceShape` (place-shaped), `BodyShape` (body-formed),
  `LooseObjects` (object-placed, object-moved); generation 2's checks run in SD-O5's order (genesis.rs
  `placed`: unshaped, held-kind, objects-max, outside, in-solid, overlap, on-person, capacity); each
  refusal names the object's and the place's keys and the numbers; the listing is built at disclosure
  (`objects::listing`), never stored; the place form goes through `PlaceShape::checked`, the same check
  and messages as 12b's decode (12b's genesis tests unedited and green). Original review item: no write outside the two reductions; generation 2's checks run in SD-O5's order; each
  refusal names a key a world author recognizes; the listing is built at disclosure, not stored; the
  place form decodes and refuses exactly as in 12b (12b's genesis tests unedited).

### PO-C3 — Walking pushes objects

**Goal.** PO-3 and PO-13 b through the real dispatch: walking pushes objects, a jammed object blocks,
and a world without objects resolves exactly as 12b did.

**Scope.**
- `src/rapier.rs`: objects in a scene (their own group, inserted after the people in `ItemId` order);
  the walls-only, contact and objects-solid filters; the push's shape cast (SD-O16).
- `src/geometry.rs`: point and segment to footprint, the push bisection, V4 (SD-O17).
- `src/push.rs` (new): `pushes`, the one function of SD-O8.
- `src/event.rs`: `ObjectMoved` and its constructor.
- `src/system.rs`: the reaction to `arrived` (SD-O8) and the reduction of `object-moved` (SD-O6).
- `src/resolve.rs`: the guard with objects; the fast path with objects; the contact sweep with
  objects; step 6's prediction and the objects-solid re-resolution; V4 in verify and degrade; entries
  clear of objects; `stopped_by` naming an object; `Outcome` gains how objects were treated (none,
  pushed, solid) for `explain`.
- `tests/objects.rs` (new): PO-3 a … g.

**Depends on:** PO-C2.

**Failure and edge cases.** A pusher whose centre coincides with the footprint's centre ((1, 0)); a push
the bisection cannot satisfy within `PUSH_SEARCH` (a jam); an object touched at exactly R (not pushed);
an arrival into a shaped place with no objects (inert, byte-identical to 12b); an object on the counter
(never pushed).

- [x] Implementation: as scoped, in the files DO-2 names: `rapier.rs` (`Against::{Walls,
  WallsAndObjects, Contact}`, `Touch`, objects in `Scene` as extruded footprints — DO-4 —, `Pile` and its
  shape cast), `push.rs` (`Lay::pushes`, `Lay::predict`), `stride.rs`, `entry.rs`, `resolve.rs` (guard
  with objects, `Objects` in `Outcome`), `system.rs` (the reaction to `arrived`, `object-moved`'s
  reduction), `tests/objects.rs`; DO-5's trigger (E-PO3).
- [x] Validation: `cargo test -p mineworld-bodies` — `objects` (PO-3), every 12b scenario unedited, and
  `long_run` against E-PO-base (PO-13 b, `cmp` identical); M-PO3 (two variants), M-PO4, M-PO10 and
  M-PO12 each fail by name and are reverted (E-PO3).
- [x] Review (self): the resolver's `Lay::predict` and the reaction's `Lay::pushes` run the same function
  on the same inputs — the end point, the room, `objects::lying_in` as they lay (no object-moved of the
  request is reduced before the reactions, F-O2); the objects-solid path predicts nothing and its V4
  requires nobody under an object, so its reactions push nothing; every degrade rung is checked with V4
  against the objects as they lay; stay is the start the guard admitted (with DO-5, it pushes nothing);
  no float outside `rapier.rs` (isolation green); the new files are under 500 lines except rapier.rs
  (526, the one file allowed to name Rapier — reviewed: scene, pile and conversions, one concern).
  Original review item: prediction and reaction call the same `pushes` with the same inputs (SD-O10); the
  objects-solid path pushes nothing; every degrade path ends in a state that passes V1–V4; stay is the
  start the guard admitted; no float outside `rapier.rs`; `resolve.rs` and the new files stay under the
  size warnings (split along SD-O8 / SD-O9 if not).

### PO-C4 — kick and throw

**Goal.** PO-4, PO-5 and the kick and throw half of PO-7.

**Scope.**
- `src/action.rs` (new): `Kick`, `Throw` and their requirements; `src/codec.rs`: their decoding.
- `src/rapier.rs`: the flight (SD-O14).
- `src/flight.rs` (new): launch velocities in integers (SD-O11, SD-O13), V-O and the landing ladder.
- `src/system.rs`: `validate` and `resolve` for both; providing them.
- `src/offer.rs` (new): the target-less offers of SD-O12.
- `tests/actions.rs` (new): PO-4, PO-5, PO-7's kick and throw rows.

**Depends on:** PO-C3.

**Failure and edge cases.** A flight that never comes to rest (the step bound); a flight that leaves the
object in the air (rung 2); a crowded room where no lattice point verifies (rung 3, `to` = `from`); an
aimed point on a solid; a kick of an object touching a wall; CCD against the counter's thin edge.

- [x] Implementation: as scoped (E-PO4): `action.rs` (Kick, Throw, Toward, Shove and the three
  requirements), `launch.rs` (validate and resolve for both, the payload codec), `flight.rs`,
  `rapier.rs` (`Launch`, `fly`), `offer.rs`, `system.rs` (providing, validate, resolve, offers),
  `tests/actions.rs`; DO-8 … DO-10.
- [x] Validation: `cargo test -p mineworld-bodies --test actions` and the rest of the pack; the flight's
  bytes identical in a second process of the same test binary; M-PO5 and M-PO11 fail by name, reverted
  (E-PO4).
- [x] Review (self): velocities are integers (`flight.rs`: `at_least`, `d · 60 / 48`, `9 810 · 48 /
  120`) and reach Rapier only through `metres`; the flying object is inserted last, after the people
  (EntityId order) and the other objects (ItemId order), with rotations locked and CCD; `refresh` runs
  before the velocity is set and before the first step; the rest test (speed² < 50² mm²/s² for 10
  sub-steps) and the step bound are pure functions of the state; the offer's requirement is
  `at_place(place).requiring_target_available()`, `validate` evaluates the declared `same_place().within`
  against the ground point (F-O6). Original review item: only + − × ÷ on values that reach Rapier (DC-3); one dynamic body per scene, inserted
  last; `refresh` runs before the first step (F-P1); the rest test and the step bound are pure functions
  of the state; the offer's requirement is at_place and `validate`'s is the declared one (F-O6).

### PO-C5 — shove

**Goal.** PO-6, the shove half of PO-7, PO-17 and PO-4 g.

**Scope.**
- `src/action.rs`: `Shove` and its requirement; `src/event.rs`: `PersonShoved`.
- `src/system.rs`: `validate` and `resolve` (through presence's `arrivals()`); the declaration's
  presence facts (SD-O7).
- `src/offer.rs`: the shove offers.
- `tests/actions.rs`: PO-6, PO-7's shove rows; `tests/scenarios.rs`'s inert case gains offers (PO-17);
  PO-4 g.

**Depends on:** PO-C4.

- [x] Implementation: as scoped (E-PO5): `shove.rs` (validate, resolve through `arrivals()`),
  `offer.rs` (the shove offers), `system.rs` (providing Shove; emitting PersonShoved, Arrived,
  StoppedShort); `tests/actions.rs` (PO-6, PO-7's shove rows, PO-4 g, PO-17 — DO-1: in actions.rs, not
  scenarios.rs); DO-11.
- [x] Validation: the pack's tests; M-PO7, M-PO8 and M-PO1 fail by name, reverted; PO-13 b re-checked
  (E-PO5).
- [x] Review (self): `shove::resolve` returns `person-shoved` then exactly `arrivals()`'s list — no
  `Arrived::new` in the pack's sources; presence's refusal becomes `FactRefusedByOwner { presence,
  arrived }` as in movement; `shove` to oneself is NoSupportedInteraction and never offered; the
  target's availability is `has_body` (a position in a shaped place), shared by validate and the offer.
  Original review item: shove never writes a position — presence records it; the emission order is person-shoved,
  then `arrivals()`'s list; a refusal of presence's becomes `FactRefusedByOwner` as movement's does; no
  shove to oneself; the target's availability is "has a body".

### PO-C6 — The world and the real runs

**Goal.** PO-2's binary half, PO-9, PO-10, PO-11, PO-13 a and PO-16, on the extended
`worlds/bodies-yard`.

**Scope.**
- `worlds/bodies-yard`: `items:` and seven `items/<key>.yaml` (SD-O19); the README.
- `tools/cli/tests/bodies/mod.rs`: the scan's object checks and the facts shove and pushes state
  (QO-16); the copy without bodies also strips items' `body:`.
- `tools/cli/tests/bodies_yard.rs`: PO-2's refusals, PO-9 with AO-1 … AO-3 checked **before** the scan,
  PO-10, PO-16.
- `tools/cli/tests/bodies_yard_restart.rs`: PO-11's activity line (QO-16).
- `systems/bodies/tests/long_run_objects.rs` (new): PO-13 a.

**Depends on:** PO-C5.

**Activity first.** AO-1 … AO-3 are evaluated on the first 30-day run before any other claim. If one
fails, SD-O18's ladder is walked in order — c1, p1, p2 — each rung recorded in §18.11 with its counts,
and the criteria are never changed. If no rung passes, the work stops and returns to the primary
session.

- [x] Implementation: as scoped, with the four AO-2 stops and their rulings (DO-13, DO-16, DO-17,
  DO-18, the AO-2 rulings in §18.11): the world (16 objects, c1), p1, p3, p4, DO-18; the scan with
  AO-2′; PO-2 and PO-16 binary refusals; PO-10's assertions; PO-11's activity line;
  long_run_objects (E-PO6 … E-PO11).
- [x] Validation: `mineworld validate worlds/bodies-yard` exit 0; bodies_yard (4 tests + the ignored
  three-seed evidence test) and bodies_yard_restart PASS; long_run_objects PASS; M-PO2 (binary) and
  M-PO9 fail by name, reverted (E-PO11).
- [x] Review (self): the scan reads only the save — presence's and conversation's facts through their
  published types, bodies' through their JSON (DO-12); the rooms' literals are asserted against the
  place files, heights included; the survivors' exclusions as PB-11; the activity counts are printed per
  bucket, per seed. Original review item: the scan reads only the save and the packs' published fact types; its geometry literals
  are asserted against the world files; the survivors' exclusions are asserted as in PB-11; the
  activity counts are printed per bucket.

### PO-C7 — Close: cross-architecture, cost, the towns, the gate, the ledger

- [x] PO-12: the x86_64 build and the three Rosetta comparisons (E-PO11: PASS).
- [x] PO-13 a release: 48.1 µs per swept move, 198.9 µs per kick (E-PO11: PASS).
- [x] PO-14: 30.7 s (≤ 40), the copy 9.9 s, ratio 3.1× (E-PO11: PASS).
- [x] PO-1 on the final binary: both town digests, both `validate` comparisons (E-PO11: PASS).
- [x] PO-15, PO-18: the structural tests in the gate; the scope check; fmt, clippy; the full gate —
  run twice: the first found a test-harness race, fixed; the second, on the final head, PASS (E-PO11).
- [x] Documentation: `docs/MVP_STATUS.md` (the capability row and an evidence row),
  `systems/bodies/README.md` (new) and `lib.rs`'s table, §18's checkboxes, §18.10, §18.11, the handoff.
- [x] Review (self): PO-1 … PO-18 each with evidence (E-PO1 … E-PO11; PO-9 under the AO-2 rulings);
  deviations DO-1 … DO-18 and the AO-2 rulings named; FU-12a-1 and F-B7 untouched (12d's); FU-12c-1
  carried to planning.

## 18.6 Test ownership

```text
STATIC      cargo fmt; cargo clippy -D warnings; the section's type (an invalid `body:` of either form
            cannot be constructed); Resolution's private fields (12a); no Rapier type in a signature
            outside rapier.rs (the compiler, with PB-15's scan)
UNIT        rapier.rs: F-P1's canary and fix, unchanged; resolve.rs: PB-8, unchanged; flight.rs: the
            integer launch velocities against hand-computed literals; push.rs: the bisection against
            hand-computed literals (a ball, a box head-on and corner-on)
INTEGRATION through the real World::dispatch in hand-built worlds with presence, movement and bodies:
            objects at genesis and their refusals, disclosure (PO-2 pack half, PO-8); pushes, jams,
            tunnelling, nudged pushers, conflicts (PO-3); kick (PO-4); throw (PO-5); shove (PO-6); the
            offers through presence's observe (PO-7); inertness (PO-17); the 3 000-request long run
            with props (PO-13 a, committed half) and 12b's long run against the base (PO-13 b)
STRUCTURAL  the pin and VERSION pair, isolation, the pack's dependencies (PO-15); ac1_composability,
            precursor_vocabulary, seam_vocabulary, presence's and movement's scans, unedited; no diff
            under cognition/ (PO-15); scope (PO-18)
REAL RUN    (Gate 2's role) the real binary on worlds/bodies-yard: validate refusals (PO-2, PO-16), the
            30-day run with its activity criteria and its scan (PO-9), the counterfactual (PO-10), two
            processes and SIGKILL (PO-11) — committed tests; the town digests (PO-1), Rosetta (PO-12),
            the release costs (PO-13 a) and the 300-day cost (PO-14) — recorded evidence on the final
            head
GATE 1      NOT REQUIRED: no model is involved, and no LM-facing semantics change. The paced controller
            is a rule and is not edited
CI          none configured (S13); the full local gate runs once on the final executable head
```

Owned elsewhere and not repeated:
- 12a owns the seam's refusals; 12b owns walls, nudges, the bias, entry placement, the guard for people,
  and the F-P1 canary — their tests run unedited (except QO-16's two literals) and are the regression
  net for the people path, with PO-13 b.
- `ARC-34`'s band (rate, choice, position) is owned by the controller's tests; 12c only shows that the
  unchanged band attempts what bodies offers (PO-9's AO-2).
- Inventory owns its refusal of an undeclared kind; PO-16 shows only that the composition reaches it.

Each failure class above has one owner.

## 18.7 Is any of this material?

**As recommended, no.** No answer below changes 12c's scope beyond §8.1 and §11.1, contradicts an
operator decision, touches a public contract beyond `bodies`' own vocabulary, or touches the kernel.
Each departure from the step text is bounded and named:

- **Kick and throw are target-less** (F-O1, QO-6). §7.2 wrote `target: <object>`. The unchanged
  perception cannot offer or price such a target, and `buy` is the precedent (F-O6). The action's
  payload is bodies' own vocabulary.
- **Where an object lies is a row on its place**, not `Lying` on the Item (F-O5, QO-2). The owner is
  still bodies (§4.6).
- **The push reaction** computes against the objects as they lay before the request, with people
  checked rather than swept against (F-O2, QO-9). §4.5.4's binding — the resolver runs the same
  reaction sequence first, so the walker never ends inside a jammed box — holds exactly.
- **Genesis in two generations** (F-O3, SD-O5). The kernel's own ordering.
- **`throw` is a complete affordance** (QO-8), where §7.2 said it would not be. The operator's
  instruction for this design asks that all three be complete.
- **Throw launches from where the object lies** (QO-11), not "at chest height in front of the
  thrower" (§8.1).
- **bodies reads `item`** (QO-4): a crate dependency, not a system dependency, to deliver QB-3's
  refusal.

**Two questions become operator-material if the alternative is chosen** (QO-3, QO-5): authoring
objects in the place's file instead of the item's (it would revise QB-3's letter), and adding pick-up
and put-down to 12c (a scope change).

Not needed: any kernel, contract, persistence, server, cognition, presence, movement, item, inventory or
worldpack change; a change to the I-2 scan or to AC-1's checks (F-O11).

## 18.8 Questions (QO-1 …)

None is operator-material as recommended. **[OM if changed]** marks the two that become so if the
alternative is chosen. The others are the primary session's to decide at freeze.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QO-1** | **Who owns a loose object's position:** bodies, or a new `objects` pack? | **bodies** (SD-O1), as §4.6 froze it. An `objects` pack would own a position only bodies can decide, so bodies would state its facts for every push, kick and throw, and could not be installed without it. |
| **QO-2** | **Where the state lives:** `LooseObjects` on the Place plus `BodyShape` on the Item (SD-O2), or §10.2's `Lying` on the Item? | **`LooseObjects` on the Place.** `Lying` on an Item can never be disclosed (F-O1), so a client could not draw objects (R-B4); a place-row is what a resolution reads anyway, already in canonical order; `Shop` is the precedent (F-O5). |
| **QO-3 [OM if changed]** | **Where an object is authored:** (a) in its own item file's `body:` section, as QB-3 decided, which needs genesis in two generations (F-O3, SD-O5); or (b) as a list in the place file's `body:`, naming items by key, checked in one reduction. | **(a)**, as decided. (b) is simpler — one fact, one reduction, no object in an unshaped place by construction — but it revises the letter of QB-3 ("an Item file with a `body:` section"), so it is the operator's to choose. |
| **QO-4** | **QB-3's refusal of an object that is also held.** Bodies refuses an Item with both `body:` and `item:` by asking `mineworld_item::is_declared` (a crate dependency on `item`, no system dependency); inventory's own declared-kind rule then refuses every holding of an object (F-O10). Alternatives: no check (QB-3 unmet); bodies depending on inventory (heavier, and inventory's section is not needed). | **The crate dependency on `item`.** `item` depends on no pack; in a world without `item` the answer is "not declared"; AC-1's checks are unaffected (F-O11). `isolation.rs`'s claim is edited to say so (QO-16). |
| **QO-5 [OM if changed]** | **Pick-up and put-down in 12c?** | **No.** §8.1 keeps carrying out; the operator's requirement is push, kick and throw (R-4). The crossing is settled now (SD-O3: each owner states its half through its own checked constructor, in one emission list), and its prerequisite — how a unique object is held (`ARC-36` item 3) — is named for the step that adds it. Adding it here changes scope. |
| **QO-6** | **Kick's and throw's target:** in the payload, with the request target-less (F-O1, F-O6), or the request's target (§7.2)? | **In the payload.** The request target would need presence's perception to list and locate items: an edit to presence (I-1), and to the observation's meaning. `buy` already works this way. |
| **QO-7** | **Which objects are offered:** only those within reach (available), or every object in the place with `TargetUnavailable` beyond reach? | **Only those within reach.** A target-less offer cannot say `TooFarAway` (F-O6, F-48); omitting is honest where `TargetUnavailable` would misreport distance, and keeps observations small. A direct request beyond reach is answered `TooFarAway` by `validate`. |
| **QO-8** | **Direction parameters.** Kick: none (away from the kicker, §8.1). Throw: `toward: Option<{x, y}>`, the complete form `null` meaning the default aim (SD-O13). Alternatives: eight compass headings (eight offers per object); a target person (objects × people offers); the thrower's facing (nobody headless sets one, F-O8). | **As recommended.** One complete offer per action per object; a client that aims sends the point. Kick needs no aim: a player aims a kick by where they stand. |
| **QO-9** | **The push reaction under breadth-first reduction** (F-O2): against the objects as they lay before the request, people not swept against but checked in the final state by the resolver, with objects solid as the fallback (SD-O8 … SD-O10). | **Accept.** It is the only form in which the reaction can compute exactly what the resolver predicted. §4.5.4's binding holds. |
| **QO-10** | **Simplifications of the flight:** one dynamic body per flight (other objects fixed; no chain reactions); rotations locked and never persisted; no stacking (an object rests on the floor or a solid). | **Accept** (R-3). Each removes a state a later push could leave invalid (an object hanging in the air) or a cost nobody asked for. |
| **QO-11** | **Where a throw starts:** where the object lies, or at chest height in front of the thrower (§8.1)? | **Where it lies.** A start in front of the thrower may lie in a wall, a solid or a person when the thrower faces one; where the object lies is valid by induction. "Picks up and throws in one action" is kept as a single fact. |
| **QO-12** | **The landing ladder:** the simulated end; else the nearest verified lattice point on the floor; else the object stays (`object-moved` with `to = from`). | **Accept.** DC-8 for objects. The last rung records a kick that moved nothing — true, and counted in the runs. |
| **QO-13** | **Activity criteria and the remedy ladder** (SD-O18, PO-9): AO-1 every seat moves and talks; AO-2 every action and a push happen in every bucket; AO-3 at most 144 physical actions per seat per 10 days. Remedies: c1 content, p1 shove offered within 800 mm only, p2 one nearest object offered; never the controller. | **Accept, fixed before measuring.** AO-3 is what makes "kick everything constantly" a failure rather than a tuning. |
| **QO-14** | **Cost guards:** release ≤ 100 µs per swept move and ≤ 2 000 µs per kick in the long run with props (PO-13 a); 300 days of the extended bodies-yard ≤ 40 s in the dev profile (PO-14). QB-11's ratio bound stays 12d's, on the towns. | **Accept.** b1-style absolute bounds, fixed now. A failure is a material stop with the numbers, not a re-scope. |
| **QO-15** | **Test world:** extend `worlds/bodies-yard`, or add a sibling world? | **Extend** (SD-O19). Shove changes bodies-yard in any case (F-O12); a sibling would duplicate the harness. "No objects, no change" is held by PO-13 b instead. |
| **QO-16** | **Existing tests edited**, each claim stated: `rapier_pin.rs` — (1, "0.36.0") → (2, "0.36.0"), the claim "the pack's version moves with Rapier's" unchanged; `isolation.rs` — "presence the only pack dependency" → "presence the only system dependency; presence and item the only pack crates"; `tools/cli/tests/bodies/mod.rs` — displaced arrivals and stopped-short "stated by movement" → "by movement or bodies", a shoved person's own arrival ≤ 500 mm, objects checked; `bodies_yard.rs` and `bodies_yard_restart.rs` — the activity lines gain the new facts. 12b's 30-day bodies-yard sha (E-PB8) is not a frozen value and changes. | **Accept.** Each edit follows from a behaviour 12c adds (shove, objects) or from SD-O20's version; none weakens what 12b proved about people, which `scenarios.rs`, `genesis.rs`, `long_run.rs` (PO-13 b) and the scan keep holding. |
| **QO-17** | **Is a shove, a kick or a throw biographical?** | **Not in 12c.** `BIOGRAPHICAL` stays empty; which physical events belong in a life is a content question for when biographies are read in the clients (S12). |
| **QO-18** | **Does the head-on bias apply to a shoved person's arrival?** | **Yes.** It is the resolver's rule for every arrival within a place; a shove into somebody standing on the line turns, exactly as a walker's stride would. Stated in ARC-39's note 2. |
| **QO-19** | **Version.** bodies `VERSION` 1 → 2 (SD-O20). | **Yes.** Results and vocabulary change; a 12b save is refused by name (`ARC-25`). |
| **QO-20** | **Entries and objects:** an arrival from another place never pushes an object (E1 and E3 avoid them; E2 must pass V4). | **Accept.** A crossing lands on a free point; pushing objects on entry would add a second push path for no requirement. |

## 18.9 Proposed execution contract for PR 12c

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12c — objects: walking pushes them; kick, throw and shove (S15,
                    third of five)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §18; evidence in §18.10 (E-PO<n>);
                    deviations in §18.11
RELATED / BINDING   this file: the header's freeze record (QB-1, QB-3, QB-6, QB-8, QB-10, QB-15), §§4.4–4.7,
                    5, 6.3, 7.2, 8.1, 9.5, 10.1 (I-1 … I-13), 11.1, §17 (12b as merged, §17.12), §18;
                    overall.md §3 (S15), §7; DECISIONS ARC-23, ARC-25, ARC-26, ARC-27, ARC-31, ARC-33,
                    ARC-34, ARC-36, ARC-37, ARC-38 (the Shop precedent), ARC-39 and its note, DEP-13;
                    MODULE_SPEC §4.1; CLAUDE.md §§2–4
IMPLEMENTATION BASE main after #60 and this design's PR merge (9c617ed + Markdown only); branch
                    mvp0/pr-12c-objects; worktree /Users/yuema137/mineworld-worktrees/s15-12c (proposed),
                    held by the implementing session only
APPROVED SCOPE      §18.1's change set; PO-C1 … PO-C7; SD-O1 … SD-O21 as answered by QO-1 … QO-20
FROZEN INVARIANTS   No edit under kernel/, contracts/, persistence/, server/, cognition/, clients/,
                    authoring/, sdk/, systems/presence/, systems/movement/, systems/item/,
                    systems/inventory/, systems/installed/, worldpack/src/, tools/cli/src/,
                    tests/acceptance/, worlds/social-cafe/, worlds/market-town/, nor the root Cargo.toml;
                    no other existing System Pack.
                    PO-1: social-cafe sha ad49c723…c64b and market-town sha 365b50e0…1d1d over the 300-day
                    seed-7 runs (365 330 and 372 755 facts), faults 0 — no digest re-baselined.
                    PO-13 b: 12b's long run byte-identical to E-PO-base.
                    Existing tests unchanged except QO-16's.
                    The paced controller unchanged (I-9-like): a bad activity profile is fixed in content
                    or in bodies' offers only (SD-O18), with AO-1 … AO-3 fixed before measuring.
                    I-2: bodies alone writes BodyShape, LooseObjects and PlaceShape; presence alone
                    writes positions of people (shove states through arrivals()); inventory never holds
                    an object with a body.
                    I-3, I-4, I-5: no float persisted or outside rapier.rs; rapier3d =0.36.0 as DEP-13
                    states; nothing of Rapier survives a call; one dynamic body per flight, bounded steps.
                    I-11, I-12 unchanged for people; V4 and V-O for objects.
                    QB-6: kick and throw resolved at the instant. QB-8: no Busy.
SEQUENCE            PO-C1 → PO-C2 → PO-C3 → PO-C4 → PO-C5 → PO-C6 → PO-C7, each committed and pushed
                    when coherent; E-PO-base captured in PO-C1 before any code
VALIDATION BUDGET   unit/integration/static unrestricted; real runs: each 300-day run (~13–40 s) at most
                    six times (PO-1 ×2, PO-14 ×2, one re-run each of PO-1 and PO-14); 30-day bodies-yard
                    runs inside their committed tests, re-run at most three times for SD-O18's ladder; the
                    x86_64 build once (background) and its three Rosetta runs; one full workspace gate on
                    the final head (background, ~6 min); about 90 minutes in total; real-model: NOT
                    REQUIRED
LIVE DOCUMENTATION  §18 checkboxes; §18.10 E-PO ledger; §18.11 deviations
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for 12c at PO-C1
ENDPOINT AUTHORITY
  implementation + local validation   unresolved until the freeze message
  semantic commits, branch push       recommended authorized, as for 12a and 12b
  PR creation / update                recommended authorized, as for 12a and 12b
  scratch builds                      recommended authorized: the base binary (PO-C1) and the
                                      x86_64-apple-darwin build (PO-12), copied to /tmp/s15-12c; no branch
  CI repair                           N/A — no CI workflow (S13)
  merge                               operator only, with a merge commit; never inherited, never widened
POST-MERGE SYNC     the planning session owns the step header, §§1–15, overall and MVP_STATUS's Updated
                    and S15 lines; the implementing session owns §18 and the evidence rows of PO-C7
NORMAL STOP         PR 12c READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a needed edit outside §18.1's change set — above all in presence, movement, item,
                    inventory, the controller, the kernel or contracts; either town digest differing from
                    PO-1's; PO-13 b's bytes differing from E-PO-base; an existing test failing for a reason
                    other than QO-16; AO-1 … AO-3 failing on every rung of SD-O18's ladder; PO-13 a above
                    its bounds, or PO-14 above 40 s; cross-architecture digests differing (PO-12 FAIL, not
                    PARTIAL); an answer to QO-3 or QO-5 other than the design's
```

## 18.10 Evidence ledger

```text
E-PO0 PO-C0, 2026-10-08, planning session, on main @ 9c617ed (12b merged) + the 12b post-merge docs
      (#60, docs/s15-12b-merged @ 8be5fb8).
      Source read for §18.2, each at the file and lines it cites: systems/presence/src/{observe,
        interaction,resolve}.rs; systems/bodies/src/{lib,system,section,event,codec,resolve,rapier,
        geometry,component}.rs and tests/{long_run,isolation,rapier_pin}.rs; systems/item/src/system.rs;
        systems/inventory/src/admit.rs; systems/item-transfer/src/{action,offer,system}.rs;
        systems/economy/src/{action,offer,component,system}.rs; systems/consumption/src/offer.rs;
        cognition/rule-controller/src/{paced,offered}.rs; kernel/src/dispatch.rs; worldpack/src/load.rs;
        authoring/src/section.rs; contracts/src/{spatial,action}.rs; tests/acceptance/tests/
        ac1_composability.rs; worlds/{social-cafe,market-town,bodies-yard}; tools/cli/tests/bodies*.
      `python3 scripts/check_doc_headings.py` → 176 numbered sections across 25 documents, none
        duplicated. `python3 scripts/check_decision_ids.py` → 51 decision ids, all distinct.
      No cargo build or test: this design is documentation only.

E-PO-base 12c implementation session, 2026-10-08, on mvp0/pr-12c-objects @ 0fd0be3 (= main: 9c617ed +
      Markdown only), before any 12c code or document edit.
      `cargo build -p mineworld-cli` 36.2 s; target/debug/mineworld copied to /tmp/s15-12c/base-mineworld
        (sha-256 08a2affdb6fb0c0edf3c03a51a1af0aef4a6b52162f4bf02334ec397edb48363).
      With it: `validate worlds/social-cafe` → exit 0, 27 lines, kept as
        /tmp/s15-12c/base-validate-social-cafe.txt (sha-256 ebcd60a0…f56a8); `validate
        worlds/market-town` → exit 0, 51 lines, base-validate-market-town.txt (sha-256 64f41086…73502).
        PO-1's references.
      `BODIES_LONG_RUN_SECOND_PROCESS=1 cargo test -p mineworld-bodies --test long_run -- --nocapture`
        → exit 0; its one `LONG-RUN` line kept as /tmp/s15-12c/base-longrun-line.txt, 4 091 748 bytes
        (9 + E-PB7's 4 091 738 + newline), sha-256
        d7025dbcdb63c0aa5162c10552f43e2650d24e510dc67f4662ac30d1b2479eaf. PO-13 b's reference.
      For information: `run worlds/bodies-yard --headless --seed 7 --days 30` → exit 0, faults 0,
        59 619 facts, summary sha-256 (every line but `wall`)
        3a2c3322bcebc39e8d50e2969cfe25bd73da36e743a64f728b567337fb5eaa9d = E-PB8's. Kept as
        /tmp/s15-12c/base-yard30.txt.

E-PO1 PO-C1, 2026-10-08, working tree on 0fd0be3 + docs/DECISIONS.md, docs/MODULE_SPEC.md, the handoff
      and this ledger.
      `python3 scripts/check_doc_headings.py` → 176 numbered sections across 25 documents, none
        duplicated. `python3 scripts/check_decision_ids.py` → 51 decision ids, all distinct (unchanged:
        the three notes add no id). PASS. Documentation only; no cargo run beyond E-PO-base.

E-PO2 PO-C2, 2026-10-08, working tree on 2fb6d6a + PO-C2's paths (systems/bodies/**, Cargo.lock).
      `cargo test -p mineworld-bodies --offline`: lib 10 (12b's 4 + footprint's 3 + … all PASS),
        genesis 6 (12b's, unedited), isolation 5 (QO-16's claim + the new item guard), long_run 1,
        objects_genesis 10 (new), rapier_pin 1 (pair (2, "0.36.0")), scenarios 17 (unedited). PASS.
      The refusals as printed (objects_genesis --nocapture):
        "bodies-unshaped: ball lies in street, which has no `body:` section: no floor to lie on";
        "bodies-held-kind: ball carries both `body:` and `item:`: an object is one physical thing and
          never a kind anybody holds (ARC-36 note)";
        "bodies-objects-max: room already holds 32 loose objects; b32 would be one more";
        "bodies-object-outside: ball at (50, 3000) in room reaches beyond its floor";
        "bodies-object-in-solid: crate at (5000, 6500) in room meets solid 0";
        "bodies-object-overlap: b-ball at (4100, 3000) overlaps a-ball at (4000, 3000) in room";
        "bodies-object-on-person: ball at (2200, 3000) lies under alice, who stands at (2000, 3000) in
          room; a person keeps 300 mm from an object";
        "bodies-capacity: room's floor has room for a person at 4 points of the 650 mm grid; a world
          of 1 people with 1 objects there needs 5".
        PO-8: alice (room) is told exactly the room's listing — [{object ball, shape {ball:110}, at
          (4000, 3000, 110)}, {object crate, shape {box:{200,200,200}}, at (2000, 5000, 200)}] — and
          nothing of the yard's object. An object beside a declared kind (item installed) is placed;
          32 objects are placed and a 33rd refused.
      Mutations (applied, run, reverted; `git grep -n MUTATION -- systems` empty afterwards):
        M-PO2 (pack half: the person-overlap check `&& false`) → objects_genesis FAILED
          an_object_under_a_person_is_refused ("this genesis must be refused").
        M-ITEM (§18.0 (b): `use mineworld_item::ItemKind as _;` planted in objects.rs) → isolation
          FAILED this_pack_uses_nothing_of_the_item_crate_but_is_declared, naming
          "systems/bodies/src/objects.rs:13: use mineworld_item::ItemKind as _;".
      `git diff Cargo.lock`: one line, mineworld-bodies' dependency list gains "mineworld-item".
      `cargo fmt --all`; `cargo clippy -p mineworld-bodies --all-targets --all-features -- -D warnings`
        clean.

E-PO3 PO-C3, 2026-10-08, working tree on 7cb6d0e + PO-C3's paths (systems/bodies/** only).
      `cargo test -p mineworld-bodies --offline`: lib 10, genesis 6, isolation 5, long_run 1, objects 7
        (new: PO-3 a … g), objects_genesis 10, rapier_pin 1, scenarios 17 — every 12b test unedited
        and green. PASS on the first run of objects.rs except (f)'s layout, re-laid for M-PO10 (DO-6).
        (a) walker (3 290 ± 1, 5 000), box (3 800 ± 1, 5 000, 200), facts [arrived, stopped-short (by
          the box), object-moved], the push Causation::Event(the arrived), decision = the walker's
          ActionId, emitted by bodies; (b) explain → Objects::Solid; walker (7 610 ± 1), box unmoved,
          stopped-short by the box; (c) walker (3 880 ± 1), ball (4 300 ± 1); (d) Route::Clear, one
          fact = Arrived::new's bytes; (e) b nudged, the ball pushed by b, caused by b's arrived;
          (f) explain → { nudge_failed: true, objects: Solid }, ball unmoved, nothing pushed; (g) walker
          (5 000, 5 860 ± 1), box unmoved, stopped by it. After every request `assert_holds`: V1–V4
          and SD-O2 from the test's literals.
      PO-13 b: `BODIES_LONG_RUN_SECOND_PROCESS=1 cargo test -p mineworld-bodies --test long_run --
        --nocapture` → its LONG-RUN line `cmp` identical to /tmp/s15-12c/base-longrun-line.txt (sha
        d7025dbc…79eaf). PASS: a world without objects is resolved exactly as in 12b. Recorded
        evidence, not a committed comparison: the base bytes live outside the repository, and 12b's
        long_run.rs stays unedited.
      Mutations (applied, run, reverted; `git grep -n MUTATION -- systems` empty afterwards):
        M-PO3 variant 1 (step 6's fallback to objects solid removed; V4 kept) → objects FAILED (b)
          "Outcome { … degraded: Blocked, objects: Untouched }" and (f); (g) survived — V4 and the
          degrade ladder still stopped the walker at contact (defence in depth, recorded).
        M-PO3 variant 2 (step 6 removed entirely: no prediction, V4 given no objects) → objects
          FAILED (b) and (f) at `explain`, and (g) at dispatch: "FactRefusedByOwner { bodies,
          object-moved, bodies-object-jammed: box cannot be pushed out of walker's way at (5000,
          6000) in room … }" — the reaction's guard (ARC-39 note 2, point 2) fires.
        M-PO4 (the fast path and the contact sweep ignore objects) → objects FAILED (a) "walker at
          (3300, 5000)", (b), (c) "walker at (5000, 5000)" — the walker passes over the ball —, (g).
        M-PO10 (a push set with one object pushed twice accepted) → (f) FAILED: "Outcome { …
          nudged: 2, objects: Pushed(2) }".
        M-PO12 (NUDGE_MAX 300 → 299) → long_run's LONG-RUN line differs from the base's at byte
          43 935 (4 075 132 bytes against 4 091 748): the comparison sees the people path.
      DO-4's experiment (objects inserted as their real shapes in people's sweeps, reverted) → (c)
        FAILED "walker at (3933, 5000)": 53 mm past the footprint's contact point.
      `cargo fmt --all`; `cargo clippy -p mineworld-bodies --all-targets --all-features -- -D warnings`
        clean. Source sizes (at PO-C3): rapier.rs 526, component.rs 451, geometry.rs 476, resolve.rs 436 (was
        758: split into resolve.rs, stride.rs 406, entry.rs 120, push.rs 147 — DO-2).

E-PO4 PO-C4, 2026-10-08, working tree on f6243db + PO-C4's paths (systems/bodies/** only).
      `cargo test -p mineworld-bodies --offline`: lib 13 (+ flight.rs's 3), actions 15 (new; PO-4 a … e
        and g, PO-5 a … e, PO-7's kick and throw rows, a second-process flight check), objects 7,
        objects_genesis 10, genesis 6, isolation 5, long_run 1, rapier_pin 1, scenarios 17. PASS.
      Flight ends as printed, each inside the bounds §18.4 fixed before measuring:
        PO-4 c  kicked from (2 700, 5 000, 110) to (5 238, 5 000, 110): Δx 2 538 ∈ 1 500 … 3 500,
                Δy 0, z 110; 12 keyframes, the first = from; the kicker unmoved.
        PO-4 d  against the wall: (8 206, 5 000, 110) ≤ 8 215.
        PO-4 e  toward c at (4 000, 5 000): (3 560, 5 010, 110) — a point of the 50 mm lattice for r
                110, so rung 2 placed it (V-O refused the simulated end); 440 mm from c's centre ≥ 405;
                c has no fact.
        PO-4 f  (flight.rs unit) after 2 sub-steps (2 725, 5 000, 235), in the air; landed at
                (2 710, 5 010, 110): on the 50 mm lattice for r 110 (x, y ≡ 110 mod 50), nearest.
        PO-5 a  unaimed: (2 600, 5 000, 110) → (5 945, 5 000, 110): x ∈ 4 600 … 7 600, Δy 0.
        PO-5 b  aimed at (2 600, 8 600): (2 600, 9 287, 110), 687 mm on (≤ 1 500).
        PO-5 c  outside the floor and 6 001 mm → PreconditionFailed; 6 000 mm → Accepted; the ball
                801 mm away → TooFarAway.
        PO-5 d  a box half 100 thrown at the counter's top: (4 950, 6 870, 1 199) — on its top
                (1 100 + 100 ± 5), recorded; then its kick → TargetUnavailable (PO-4 b's last row).
        PO-5 e  among p1, p2, p3: (3 860, 5 010, 110), ≥ 405 mm from each; nobody moved.
        PO-4 a, b, g; PO-7 a (kick { ball }, throw { ball, toward: null }, at_place(room), available;
          nothing for the box 900 mm away), b (each submitted through Affordance::request: Accepted),
          c (unshaped place, positionless observer: no bodies affordance). PASS.
      Second process: `flights_are_byte_identical_in_a_second_process` — 5 725 bytes, equal. PASS.
      Mutations (applied, run, reverted; `git grep -n MUTATION -- systems` empty afterwards):
        M-PO5 (V-O off in `land`) → flight.rs FAILED a_flight_cut_short_lands… "at rest height left:
          235" (the object recorded in the air); actions also FAILED PO-4 e and PO-5 e at dispatch
          (object-moved's own reduction refused the overlapping end).
        M-PO11 (objects beyond reach offered) → actions FAILED the_kicks_and_throws_offered… "kick and
          throw of the ball, nothing for the box", and each_offered_… (the box's throw TooFarAway).
      `cargo fmt --all`; `cargo clippy -p mineworld-bodies --all-targets --all-features -- -D warnings`
        clean.

E-PO5 PO-C5, 2026-10-08, working tree on 92dd1be + PO-C5's paths (systems/bodies/** only).
      `cargo test -p mineworld-bodies --offline`: lib 13, actions 23 (+ PO-6 a … e, the 600 mm
        record, PO-7's shove rows, PO-4 g's shove, PO-17), genesis 6, isolation 5, long_run 1,
        objects 7, objects_genesis 10, rapier_pin 1, scenarios 17. PASS. long_run's LONG-RUN line
        `cmp` identical to the base again (PO-13 b holds with shove in the pack).
      First run of PO-6 with §18.4's layouts (shover 600 mm behind the target): (a) → person-shoved,
        arrived b (4 301, 5 000), stopped-short { by: None }; (b) → b at (8 000, 5 000). Moving the
        shover to 700 mm: (a) → b exactly (4 500, 5 000), no stopped-short. DO-11.
      PO-6 as passing (shover 700 mm behind): (a) [person-shoved, arrived] — b (4 500, 5 000), a
        unmoved, both facts Causation::Action(a's id), emitted by bodies; (b) b (8 010 ± 1, 5 000),
        stopped-short { b, wanted (8 200, 5 000), by None }; (c) [person-shoved, arrived b, arrived c,
        stopped-short]: b (4 499, 4 999), c (5 063, 5 234) — each ±1 mm of the literals (4 500, 5 000)
        and (5 064, 5 234) (b's stride is swept, so b's end is Rapier's, a millimetre short: a true
        stopped-short), c nudged 69 mm ≤ 310; (d) the ball pushed by b, Causation::Event(b's arrived),
        to (4 919, 5 001) ± 1 of (4 920, 5 000); (e) 1 000 mm Accepted, 1 001 TooFarAway, oneself
        NoSupportedInteraction, a positionless b and b in an unshaped place TargetUnavailable, b in
        another shaped place TooFarAway, a positionless a PreconditionFailed. PASS.
      PO-7 shove rows: shove → p { payload {}, available }, shove → q { unavailable TooFarAway },
        requirement same_place().within(1 000).requiring_target_available(); none to oneself; the
        available shove submitted through Affordance::request is Accepted. PO-4 g: without bodies,
        kick, throw and shove → ActionResult::Unavailable. PO-17: in a world with bodies, an unshaped
        place offers nothing from bodies, and a move there records exactly Arrived::new's bytes. PASS.
      Mutations (applied, run, reverted; `git grep -n MUTATION -- systems` empty afterwards):
        M-PO7 (SHOVE_DISTANCE 500 → 800) → actions FAILED (a) "b 500 mm on: left Some((4800, 5000))",
          (b) "wanted (8500, 5000)", (c), (d).
        M-PO8 (shove states Arrived::new directly, bypassing arrivals()) → actions FAILED (c): two facts,
          c not nudged; (b) and the 600 mm record: no stopped-short. (d) survived: b's unresolved
          arrival overlaps the ball, and the reaction's push happens to be valid there.
        M-PO1 (shove offered before the place-shape check) → actions FAILED PO-17: "nothing from
          bodies: [("shove", Some("other"), "{}", false)]".
      `cargo fmt --all --check`; clippy -p mineworld-bodies -D warnings clean.

E-PO6 PO-C6 (partial — STOPPED at PO-9's activity criteria, a material stop), 2026-10-08, working tree
      on b18b465 + the extended world, the scan's QO-16 edits, SD-O18's rungs c1 and p1, and the shove
      fix (DO-14). Every run: `cargo test -p mineworld-cli --test bodies_yard thirty_days -- --nocapture`
      (the real binary, `run worlds/bodies-yard --headless --seed 7 --days 30 --save S`, then the scan
      of S), ≈ 5 s each. AO-1 … AO-3 as §18.4 fixed them, unchanged; counts per 10-day bucket as
      kicks / throws / shoves / pushes / stopped-short by an object; AO-3's maximum per seat; AO-1
      held in every run (every seat moved ≥ 342 and talked ≥ 297 times per bucket).
        run 1  SD-O19's seven objects, no rung        1–10: 4/4/1 176/8/10   11–20: 0/0/1 237/0/0
               21–30: 1/0/1 264/1/0; AO-3 max 124.    FAIL (AO-2, bucket 2: no kick, throw, push,
               stopped-short by an object)
        run 2  c1: 13 objects, six beside the doors and furniture (door objects ≈ 1 000 mm from the
               doorway points)                         1–10: 14/9/1 200/14/17  11–20: 1/2/1 253/1/1
               21–30: 4/1/1 196/0/4; max 124.          FAIL (AO-2, bucket 3: no push)
        run 3  c1 + p1 (shove complete within 800 mm) 1–10: 18/15/966/34/60  11–20: 4/3/1 040/1/9
               21–30: 7/8/1 071/1/4; max 105.          AO PASS — but the scan then found 385
               violations "moved 500 mm (at most 500)": a shove asked for a fraction of a millimetre
               more than 500 mm (DO-14, a defect, fixed with a regression test)
        run 4  c1 + p1, the shove fixed               1–10: 11/8/1 048/13/28  11–20: 4/1/1 025/3/3
               21–30: 2/0/1 030/1/3; max 110.          FAIL (AO-2, bucket 3: no throw)
        run 5  c1 completed (8 objects per room) + p1 1–10: 16/13/1 000/28/33  11–20: 0/1/1 049/1/2
               21–30: 1/0/1 027/2/0; max 108.          FAIL (AO-2, bucket 2: no kick)
        run 6  c1 with the door objects 750 mm from the doorway points (within reach of every
               crossing) + p1                          1–10: 19/16/969/28/56  11–20: 9/7/1 007/3/16
               21–30: 1/0/1 022/0/9; max 101.          FAIL (AO-2, bucket 3: no throw, no push)
        run 7  c1 + p1 + p2 (kick and throw complete for the nearest object only)
                                                       1–10: 11/9/1 006/27/39  11–20: 1/2/1 044/3/0
               21–30: 6/1/1 036/3/4; max 114.          FAIL (AO-2, bucket 2: no stopped-short by an
               object). p2 reverted afterwards; the tree holds c1 (run 6's content) + p1.
      Every rung of SD-O18's ladder fails AO-2 in a later bucket: §18.9's material stop "AO-1 … AO-3
        failing on every rung of SD-O18's ladder". Budget: 8 runs (1 … 7 and 5b) against §18.9's "at
        most three re-runs" for the ladder — each ≈ 5 s; runs 3 and 4 were forced by the shove defect,
        5b was a diagnostic.
      What the counts say (for the decision, not a remedy taken):
        - AO-1 and AO-3 hold with a wide margin in every run (AO-3 maximum 101 … 124 of 144).
        - AO-2 holds in days 1–10 every time and decays afterwards. Objects that are kicked or thrown
          travel 2 … 3 m and often end against a wall — runs 5 and 6 each end with 5 of 16 objects
          within 250 mm of a wall, two of them in corners — while objects nobody reaches stay where
          they were authored (run 6: hall-box-west, hall-ball-2 and court-box-east never moved in 30
          days). People wander ±1 400 mm and gather to talk, and seldom come within 800 mm of an
          object again: in a diagnostic re-run of run 5's content (run 5b, the scan printing each
          object's nearest approach), the nearest any person came to court-box-east's centre in 30
          days was 1 274 mm, to hall-box-west's 802 mm. Later buckets see 0 … 9 kicks and 0 … 8
          throws, so pass or fail is a matter of the seed's trajectory (run 3 passed; run 4 — the
          same content and policy with one shove defect fixed — failed).
        - Shoves dominate the offered band (≈ 1 000 per bucket) with or without p1.
      Not run, pending the decision: PO-9's scan verdict on the final content, PO-10, PO-11, PO-12,
        PO-14 (each depends on the yard's final content), PO-2's and PO-16's binary refusals, and
        PO-C7.

E-PO6b PO-13 a (independent of the yard's content, run while the decision is pending), 2026-10-08,
      same working tree. `systems/bodies/tests/long_run_objects.rs` (new): the prototype's 3 000-request
      sequence with its four props, kicks by the nearest person (DO-15).
        dev: 1 472 moves reached Rapier, mean 48.5 µs; 363 kicks accepted, mean 127.3 µs; 89 refused;
          closest person–object centres p06 and prop2, 412 mm, after request 1 383; every request's
          N-1 … N-4 and SD-O2 held (`assert_holds` after each); 537 454 bytes identical in a second
          process. PASS.
        release (`cargo test --release --offline -p mineworld-bodies --test long_run_objects --
          --nocapture`, 19.3 s build): mean 52.8 µs per move that reached Rapier (bound ≤ 100 µs:
          PASS), 109.7 µs per accepted kick (bound ≤ 2 000 µs: PASS); the same 537 454 bytes in a
          second process. Log /tmp/s15-12c/release-long-run-objects.log.

E-PO7 Rung p3 (the primary session's ruling), 2026-10-08, working tree on d4ebe46 + p3 (flight.rs,
      launch.rs) and its tests. STOPPED again: AO-2 fails on all three seeds.
      `cargo test -p mineworld-bodies`: all green — lib 14 (free_centre and the new default aim
        against hand-computed literals: café (4 160, 5 160); the hall's table-covered centre → (6 010,
        5 110); a kick from (2 700, 5 000) → (4 973, 545, 0) mm/s; an unaimed throw from (200, 200) →
        (2 071, 2 544)), actions 25 — the new scenario `a_ball_kicked_near_a_wall_travels_toward_the_
        centre`: ball (8 100, 5 000) against the east wall, kicker (8 000, 4 300) → (5 563, 5 103),
        2 503 mm nearer the centre; PO-4 c restated along the new line → (5 226, 5 277): 2 541 along,
        0 off; PO-5 a → (4 154, 5 159), at the free centre.
      M-P3 (the kick's direction back to kicker → object) → the new scenario FAILED: "(8100, 5000) →
        (8203, 6460): at least 1 500 mm nearer the centre: 3943 → 4246 mm" — along the wall. Reverted;
        `git grep MUTATION` empty. fmt and clippy (bodies, cli) clean; long_run's bytes unchanged
        (no kick in it).
      AO runs (c1 + p1 + p3; 30 days; kicks / throws / shoves / pushes / stopped-short by an object):
        seed 7  1–10: 21/17/950/36/55   11–20: 4/10/1 028/6/11   21–30: 3/3/1 071/0/6   FAIL (no push,
                days 21–30); AO-3 max 104
        seed 8  1–10: 22/25/1 000/45/62  11–20: 9/8/1 077/1/13   21–30: 3/2/1 011/0/8   FAIL (no push,
                days 21–30); AO-3 max 106
        seed 9  1–10: 18/24/1 016/33/40  11–20: 7/7/1 039/0/14   21–30: 7/5/1 043/6/16  FAIL (no push,
                days 11–20)
        AO-1 held on every seed. Runs used: 3 of the 6 granted.
      What changed and what did not: kicks and throws now persist in every bucket (3 … 10), and
        stopped-short by an object too (6 … 16). Pushes do not: the objects now gather around the
        room's centre — at day 30 on seed 7, seven of the court's eight lie within 750 mm of the
        pillar's centre (the eighth never moved), and six of the hall's eight against the table's
        sides (the other two never moved) — so a walker who reaches one usually finds it jammed against the
        pillar, the table or another object: the stride is resolved with objects solid (a stopped-short
        by the object), not a push.

E-PO8 Rule p4 (the primary session's second ruling), 2026-10-08, working tree on 589813e + p4.
      STOPPED a third time: AO-2 fails on all three seeds.
      `cargo test -p mineworld-bodies`: all green — lib 14 (free_centre now (6 010, 5 410) for the
        hall's table, hand-computed), actions 26 — `a_thrown_ball_does_not_come_to_rest_against_the_
        counter`: aimed at (5 000, 6 400), 60 mm short of the counter's face, the ball rests at (5 000,
        6 160, 110), the first point of its line 300 mm clear.
      M-P4 (REST_CLEARANCE 300 → 0) → the scenario FAILED: "(5000, 4000, 110) → (5000, 6460, 110)",
        against the counter; and the free-centre unit test FAILED: (6 010, 5 110). Reverted.
      AO runs (c1 + p1 + p3 + p4; the 3 runs the ruling granted; counts kicks / throws / shoves /
        pushes / stopped-short by an object):
        seed 7  1–10: 31/25/993/68/66   11–20: 5/0/1 042/1/5   21–30: 4/0/1 017/4/11  FAIL (no throw,
                days 11–30)
        seed 8  1–10: 28/20/995/48/52   11–20: 2/4/1 056/0/7   21–30: 2/2/1 018/1/3   FAIL (no push,
                days 11–20)
        seed 9  1–10: 22/14/1 000/51/38 11–20: 3/3/1 009/5/3   21–30: 2/0/1 094/4/5   FAIL (no throw,
                days 21–30)
        AO-1 held on every seed; AO-3 maxima 107, 126, 112.
      Objects at day 30 (centre x, y, z):
        seed 7  court: ball (5 503, 4 781) · ball-door (4 584, 5 277) · ball-north (5 236, 6 256) ·
                ball-pillar (5 152, 3 947) · ball-west (4 455, 6 461) · box (4 432, 4 077) · box-door
                (4 850, 4 200) · box-east (7 500, 5 000, never moved); hall: ball (5 555, 3 587) ·
                ball-2 (3 500, 6 200, never moved) · ball-door (7 213, 5 186) · ball-table (6 010,
                5 411) · box-door (6 879, 5 205) · box-east (5 927, 3 547) · box-west (3 000, 4 500,
                never moved) · crate (7 000, 7 800, never moved)
        seed 8  court: ball (6 232, 3 814) · ball-door (4 589, 5 010) · ball-north (5 841, 3 173) ·
                ball-pillar (5 111, 4 286) · ball-west (5 542, 5 632) · box (4 979, 3 867) · box-door
                (4 781, 4 271) · box-east (7 500, 5 000, never moved); hall: ball (4 922, 2 209) ·
                ball-2 (3 500, 6 200, never moved) · ball-door (4 893, 5 473) · ball-table (5 955,
                5 751) · box-door (6 017, 3 500) · box-east (5 618, 3 579) · box-west (3 531, 3 754) ·
                crate (6 550, 3 400)
        seed 9  court: ball (6 500, 3 500, never moved) · ball-door (4 584, 5 141) · ball-north
                (5 068, 5 905) · ball-pillar (5 566, 4 571) · ball-west (6 629, 3 685) · box (5 798,
                3 994) · box-door (5 398, 4 010) · box-east (7 500, 5 000, never moved); hall: ball
                (5 000, 2 500, never moved) · ball-2 (3 500, 6 200, never moved) · ball-door (4 414,
                5 797) · ball-table (6 236, 6 269) · box-door (6 000, 3 500) · box-east (6 211, 3 086)
                · box-west (4 334, 4 105) · crate (6 938, 7 867)
      Reading: every object now rests clear of the furniture, and the days 1–10 counts are the highest
        of any configuration; activity in days 11–30 is still 0 … 5 per action per bucket, so a
        bucket with no throw or no push is a matter of the seed. Per the ruling: no rung, no criterion
        changed; reported for the primary session's open decision.

E-PO9 The AO-2 ruling (primary session, 2026-10-08, decided after failed measurements and recorded
      as such; §18.11 "AO-2 ruling"). **AO-2 FAILED as frozen** — with p1 (E-PO6 runs 3–6), with p3
      (E-PO7: seeds 7, 8, 9) and with p4 (E-PO8: seeds 7, 8, 9); the three tables stand in E-PO6,
      E-PO7 and E-PO8. AO-2 is replaced by AO-2′, checked on the p4 saves already made (no new run):
      the three saves copied to /tmp/s15-12c/p4-saves/bodies-yard-30-seed-{7,8,9} and read by
      `BODIES_YARD_SAVES=/tmp/s15-12c/p4-saves cargo test -p mineworld-cli --test bodies_yard
      thirty_days_of_bodies_yard_at_seeds_7_8_and_9 -- --ignored --nocapture`.
        (a) aggregate, every bucket summed over the seeds (kicks / throws / shoves / pushes / stopped
            by an object): days 1–10 81/59/2 988/167/156; 11–20 10/7/3 107/6/15; 21–30 8/2/3 129/9/19.
            HOLDS.
        (c) every action in days 1–10 of every seed. HOLDS.
        (b) every moved object at day 30 ≥ 300 mm from every solid or on a solid's top. FAILS on every
            seed — ten objects (last move: where from → where to, gap to the solid):
            seed 7  court-ball pushed (5 711, 5 008) → (5 503, 4 781), 93 mm; court-ball-door kicked
                    (4 589, 5 277) → (4 584, 5 277), 6 mm; hall-ball-door kicked (7 208, 5 186) →
                    (7 213, 5 186), 172 mm; hall-box-door kicked (6 879, 5 204) → (6 879, 5 205), 5 mm;
                    hall-box-east pushed (5 980, 3 500) → (5 927, 3 547), 253 mm
            seed 8  court-ball-door kicked (4 545, 5 010) → (4 589, 5 010), 1 mm; court-box-door kicked
                    (4 793, 4 467) → (4 781, 4 271), 229 mm; hall-box-east pushed (5 704, 3 598) →
                    (5 618, 3 579), 221 mm
            seed 9  court-ball-door kicked (4 589, 5 141) → (4 584, 5 141), 6 mm; court-ball-pillar
                    thrown (5 000, 4 400) → (5 566, 4 571), 185 mm
        AO-1 and AO-3 hold on all seeds (E-PO8). Per the ruling's point 4: STOPPED and reported.
      What the list shows — three ways an object comes to rest near a solid that p4 does not cover:
        1. pushes: p4 governs launched objects only; a push moves an object just clear of its pusher,
           and that can be against the side of the pillar or the table (seed 7 court-ball, hall-box-east;
           seed 8 hall-box-east);
        2. launches that start near a solid: when no point of the line from the start to the end keeps
           the clearance, DO-17's reading of "blocked" lets the end stand (seed 9 court-ball-pillar,
           thrown from (5 000, 4 400), itself 190 mm from the pillar);
        3. p3 kicks toward the free centre whatever the kicker's side: a kicker standing between the
           object and the free centre kicks it into their own capsule, and it moves 0 … 5 mm, from a
           start that was already against the solid (the four "door" objects).

E-PO10 The final AO-2 ruling's measurement, 2026-10-08, working tree on ecce915 + DO-18 + (b′).
      DO-18 (a defect fix to p3): `cargo test -p mineworld-bodies` all green — lib 14 (kick_velocity
        with the kicker at (3 400, 5 080) between the ball (2 700, 5 000) and the café's centre →
        (−4 972, −569, 0) mm/s, hand-computed), actions 27 — the new scenario `a_kicker_between_the_
        ball_and_the_centre_kicks_it_away_from_themselves`: the ball goes (2 700, 5 000) → (174, 4 711).
        M-DO18 (the into-kicker branch disabled) → the scenario FAILED: "(2700, 5000, 110) → (3010,
        4960, 110): at least a metre west" — the ball kicked into the kicker. Reverted.
      The one measurement (3 runs, seeds 7, 8, 9): `cargo test -p mineworld-cli --test bodies_yard
        thirty_days -- --include-ignored --nocapture` → the three-seed evidence test PASSED; log
        /tmp/s15-12c/final-ao.log. Counts (kicks / throws / shoves / pushes / stopped by an object):
        seed 7  1–10: 19/21/1 002/48/77   11–20: 0/3/1 057/3/4    21–30: 4/5/1 066/5/6
        seed 8  1–10: 33/21/1 010/32/58   11–20: 4/8/1 067/9/14   21–30: 1/0/1 071/3/8
        seed 9  1–10: 17/17/1 003/41/47   11–20: 7/4/1 013/5/7    21–30: 3/11/1 057/16/10
        (a) summed: 1–10 69/59/3 015/121/182; 11–20 11/15/3 137/17/25; 21–30 8/16/3 194/24/24. HOLDS.
        (b′) every object, every seed, has free standing points within 800 mm — fewest 171
             (seed 8, court-ball-pillar at (4 760, 4 110)). HOLDS.
        (c) every action in days 1–10 of every seed. HOLDS.
        AO-1 HOLDS; AO-3 HOLDS (maxima 104, 116, 111). The scan: 0 violations on every seed (31 219,
        31 221, 31 287 requests); closest pairs 595 mm; closest person–object 295 … 297 mm. PASS.
      The committed seed-7 test ran concurrently with the evidence test on the same save path and
        failed "storage: database is locked" — a harness defect (two tests sharing one save), not the
        pack: each test now saves under its own name. Re-run alone (a 4th run, needed by the gate in
        any case): PASS, 31 219 requests, 0 violations — the evidence test's seed-7 numbers exactly.
      Tool-discipline slip, recorded: one Bash call ran an empty `python3 - <<EOF` heredoc while
        reaching for the Edit tool; it wrote nothing and was not repeated.

E-PO11 PO-C6's remainder and PO-C7, 2026-10-08, on the code of 6c44db1 (the source tree of the final
      executable head; later commits change one test file's directory names and Markdown only).
      PO-2 (binary) and PO-16, `objects_that_do_not_fit_and_held_objects_are_refused_at_load`: each copy
        refused by the real `validate`, naming its subject — "bodies-unshaped: court-ball lies in
        court …"; "bodies-object-on-person: hall-ball at (4200, 2000) lies under ben …";
        "bodies-object-in-solid: hall-box-west at (4900, 4500) in hall meets solid 0";
        "bodies-object-outside: hall-ball at (50, 2500) …"; "bodies-object-overlap: hall-ball-2 at (3500,
        6200) overlaps hall-ball at (3600, 6200) …"; "bodies-capacity: hall's floor has room for a person
        at 226 points … a world of 12 people with 21 objects there needs 234" (24 boxes half 400 added);
        the loader at the line: "line 8 column 10: a ball's radius is 50 to 400 mm; this one is 0 mm",
        "… a box's half-extents are 50 to 400 mm in x and y …", "unknown field `spin`, expected one of
        floor, solids, shape, at", "… never both"; "bodies-section-kind" for the object form in court.yaml
        and the place form in hall-ball.yaml; PO-16: "bodies-held-kind: hall-ball carries both `body:` and
        `item:` …" and "system 'inventory' refused to reduce 'stocked': PreconditionFailed" (inventory's
        own words). PASS. The yard validates (41 genesis facts). M-PO2 (binary: the person check `&&
        false`) → FAILED "on-person: the copy must be refused". Reverted.
      PO-10: the copy without bodies (16 items' and 2 places' `body:` stripped) runs 30 days, faults 0,
        every seat moves, no stopped-short, displaced arrival, kick, throw, shove or push in any bucket,
        none of kick/throw/shove/object-moved/person-shoved/object-placed in its summary; the scan sees
        269 202 violations, ada and cleo 0 mm apart. PASS.
      PO-11 (`bodies_yard_restart`, 41.8 s): control 62 855 facts, 31 220 journal rows, 488 snapshots;
        10 334 stopped-short, 7 246 displaced, 23 kicks, 29 throws, 3 125 shoves, 56 pushes; a second
        process byte-identical; SIGKILL at days 5, 15, 25 → resumed at revisions 5 222, 15 649, 26 031
        with 38, 33, 47 re-executed, byte-identical to the control; `replay` → "31220 revision(s)
        re-executed from genesis, 62855 fact(s) and 488 snapshot(s) reproduced byte for byte". PASS.
        M-PO9 (a process-global counter's parity added to KICK_SPEED) → FAILED "killed at day 5: facts
        row counts differ". Reverted.
      PO-12 (Rosetta; binaries /tmp/s15-12c/final-mineworld and x86-mineworld, Mach-O x86_64, built
        48.1 s; logs /tmp/s15-12c/pb12-final): (a) 30 days, seed 7, arm64 and `arch -x86_64`: summary
        sha-256 (every line but `wall`) 6e4c4015…c8395 both, 62 855 facts; (b) arm64 save to day 15,
        x86_64 resumed "at revision 15649 (snapshot 15616 + 33 re-executed)", 62 855 facts, fingerprint
        e9398cf4… = uninterrupted; arm64 `replay` of it byte for byte; (c) the reverse, the same. PASS.
      PO-13 a on the final code (release): 1 652 swept moves, mean 48.1 µs (≤ 100); 310 kicks, mean
        198.9 µs (≤ 2 000); 569 932 bytes identical in a second process. PASS. PO-13 b: long_run's
        line `cmp`-identical to the base. PASS.
      PO-14 (dev, 300 days, seed 7): worlds/bodies-yard → faults 0, 628 413 facts, wall 30.7 s (≤ 40:
        PASS); the copy without bodies → faults 0, 429 606 facts, 9.9 s; ratio 3.1×, information.
      PO-1 on the final binary: social-cafe 365 330 facts, sha ad49c723…c64b, 14.7 s; market-town
        372 755 facts, sha 365b50e0…1d1d, 15.0 s; faults 0; both `validate` `cmp`-identical to the base.
        PASS. 300-day runs in all: 6 (PO-1 ×4, PO-14 ×2) — the budget's six.
      Full gate, first run (12:58–13:05, on 6c44db1): fmt 0, clippy 0, doc checks clean; `cargo test
        --workspace --no-fail-fast`: 635 passed, 1 failed, 1 ignored in 143 binaries — the failure is a
        harness race: the new object refusals and 12b's refusals made copies under the same directory
        names ("overlap", "in-solid", …) at the same time ("reads: NotFound"). Fixed by giving the new
        copies their own names (one test file); the gate re-run on the final head (below).
      **Full gate on the final executable head 145c7f7** (13:05:52–13:10:31, 4 min 39 s): `cargo fmt
        --all --check` exit 0; `cargo clippy --workspace --all-targets --all-features -- -D warnings`
        exit 0; `cargo test --workspace --no-fail-fast` exit 0 — 143 test binaries, 636 passed, 0
        failed, 1 ignored (`thirty_days_of_bodies_yard_at_seeds_7_8_and_9`, the evidence test); the
        `harness = false` programs "[resolver-yard] PASS", "[cafe] PASS", "[clock] PASS";
        ac1_composability 13, precursor_vocabulary 4, seam_vocabulary 3, bodies_yard 4 (+1 ignored),
        bodies_yard_restart 1, objects 7, scenarios 17; `check_doc_headings.py` 176 sections, none
        duplicated; `check_decision_ids.py` 51 ids, all distinct. Logs /tmp/s15-12c/gate2*.log. PASS.
      PO-15 / PO-18: Rapier named only in rapier.rs and no float outside it (isolation 5); rapier_pin
        (2, "0.36.0"); the system dependency presence alone, the crates presence and item, item read for
        is_declared only; `git diff --stat 0fd0be3 --` kernel/, contracts/, persistence/, server/,
        clients/, authoring/, sdk/, cognition/, systems/{presence,movement,item,inventory,installed},
        worldpack/src, tools/cli/src, tests/acceptance, both towns and the root Cargo.toml → empty;
        `git diff --name-only 0fd0be3` ⊆ §18.1's change set; Cargo.lock: one line added. PASS.
      Tool-discipline slip, recorded: one read-only `grep | awk` tallied the first gate's counts; it
        touched no file and is not repeated.

E-PO6c Independent checks on the WIP tree (the code of the WIP commit), 2026-10-08.
      PO-1 (300-day runs 1 and 2 of §18.9's six), dev binary target/debug/mineworld:
        social-cafe → exit 0, faults 0, 365 330 facts, sha-256 of every line but `wall` =
          ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b; wall 12.6 s.
        market-town → exit 0, faults 0, 372 755 facts, sha-256 =
          365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d; wall 14.7 s.
        `validate` of both `cmp`-identical to the base binary's outputs. PASS. (To be repeated on the
        final executable head.)
      PB-10 / PO-10's harness: `without_bodies` now strips the items' `body:` sections too (2 places +
        16 objects); the counterfactual runs 30 days, faults 0, and the scan sees 269 202 violations,
        closest ada and cleo 0 mm apart in court — the 12b numbers: the items are inert without bodies.
        PO-10's new assertions (no kick, throw, shove, object-moved, person-shoved) are not yet
        written (pending the decision).
      Gates, unedited: `cargo test -p mineworld-acceptance --test ac1_composability --test
        precursor_vocabulary --test seam_vocabulary` → 13, 4, 3 passed. `git diff --stat 0fd0be3 --
        cognition/ tests/acceptance/ kernel/ contracts/ systems/presence systems/movement systems/item
        systems/inventory systems/installed worldpack/src tools/cli/src worlds/social-cafe
        worlds/market-town Cargo.toml` → empty.
```

## 18.11 Deviations and discoveries during implementation

**DO-1 (bounded) — new test files instead of edits to 12b's.** PO-C2's scope adds SD-O5's refusals to
`tests/genesis.rs` and PO-C5's adds offers to `tests/scenarios.rs`'s inert case, while PO-C2's review and
§18.1 keep "the other 12b tests unedited". Both new claims go into new files (`tests/objects_genesis.rs`
here; PO-17 in `tests/actions.rs`), so 12b's genesis and scenario tests stay byte-unedited. The shared
`tests/support/mod.rs` gains fields and helpers only additively (`Plan::objects`, `Plan::kinds`,
`Yard::items`, `Yard::objects`, `ball`, `cube`): with both lists empty, genesis states exactly what it
stated before, and `long_run`'s bytes are the regression check (PO-13 b).

**DO-2 (bounded; the operator's modularity directive, relayed 2026-10-08) — where the new code lives.**
SD-O17 puts object geometry in `geometry.rs` and PO-C2 puts the reductions in `system.rs`. To keep each
module one responsibility and under the 500-line review threshold: object geometry is `footprint.rs`
(footprints, the stored-state invariant, `blocks`, the push bisection, the object lattice); the
genesis checks — 12b's `fits` moved out of `system.rs` unchanged, plus SD-O5's — are `genesis.rs`; the
place's objects as read, the owner's check on a move and the disclosed listing are `objects.rs`.
`system.rs` keeps the declaration, install, the reduction dispatch and perception. The constants stay in
`geometry.rs` as SD-O17 says. Code that later commits use is marked `#[expect(dead_code, reason = …)]`, so
each commit is clippy-clean and the compiler forces the marks' removal once the code is used.

**DO-3 (recorded per the operator's directive: reuse before reinvention) — integer predicates beside
Rapier.** The directive asks that Rapier's own facilities be used wherever they fit, and that any
deliberate exception be recorded. Rapier does every motion: the character controller sweeps people (12b),
a shape cast moves a pushed object (SD-O8, PO-C3), and the dynamics solver with CCD flies a kicked or
thrown object (SD-O14, PO-C4). What stays integer is the *verification* — footprint containment, overlap
by more than 5 mm, resting height, capacity — because the frozen design requires it (DC-3, DC-6, DC-8,
SD-O17: the stored state is checked on integers so that the check is exactly reproducible on every
machine and the engine's answer is never the last word). Parry's distance queries are `f32` and would put
a float comparison into the invariant every save is replayed against; they were not used for that.

**DO-4 (bounded; discovery) — a person's sweep sees an object as its footprint, extruded.**
- Previous assumption: SD-O16 inserts objects into every scene as their real cuboids and balls.
- Audit evidence (E-PO3): with a ball's real shape in the people's scene, PO-3 c's walker ended at
  (3 933, 5 000), 53 mm past the footprint's contact point (3 880): the capsule's rounded bottom (centre
  310 mm up) meets a ball of radius 110 below the ball's equator, at a horizontal distance of √(410² −
  200²) ≈ 358 mm instead of 410 mm. The integer checks (V4, the invariant, the push trigger) all read the
  2D footprint, so the sweep and the checks disagreed.
- Decision: in people's scenes (`Scene`), an object is its footprint extruded from below the floor to the
  walls' height — a box as a tall cuboid, a ball as a tall capsule of its radius. The push's cast
  (`Pile`) and the flight keep the real shapes (§18.3.1). The canonical order is unchanged (objects after
  people, in `ItemId` order), and a world without objects builds exactly 12b's scene (PO-13 b holds).
- Impact: the sweep's notion of an object is the checks' notion. No invariant, constant or contract
  changes.

**DO-5 (bounded; tightening) — the push trigger is "closer than `PERSON_RADIUS − TOLERANCE`".** SD-O8 says
an object is pushed when its footprint is "closer than R" to the person. The stored-state invariant
(SD-O2) and V4 allow a person within 295 … 300 mm of a footprint. With the trigger at R, a walker who
*stays* where they were (the last rung of the degrade ladder, valid by induction) could stand 297 mm from
an object, and the reaction to their recorded `arrived` would push it — a push nobody predicted, which
could jam and fail the dispatch. The trigger is therefore the invariant's own threshold, R − TOLERANCE =
295 mm (`Footprint::under`): every state the invariant admits pushes nothing, so "stay" is valid by
induction for objects too, and V4 means exactly "the reactions push nothing". Pushes still move the
object to R + GAP = 310 mm, as SD-O8 says. Every PO-3 literal is unchanged by it (each overlap there is
far deeper than 5 mm).

**DO-6 (bounded) — PO-3 f's layout: two nudged people, not the walker and one nudged person.** PO-3 f
asks for "the walker and a nudged person would both push one ball". Under the bounds that layout is hard
to build: a nudged person moves straight away from the walker, so their disc can newly reach a ball only
if the ball lies roughly beyond them as seen from the walker, while the walker's own end — at least
610 mm from them — must also come within 405 mm (R − TOLERANCE + a 110 mm ball's radius) of it, and before
the request the person must have kept 295 mm from it. The session searched three families of such
layouts by hand (a ball beside the walker's line, a box ahead of it, a person between) and found none
with margins wider than the rounding of the nudge. It did not prove that none exists. The test uses a
case that clearly exists: the walker's end nudges two people either side of its line, and each
of their new discs overlaps one ball (r 375). Each push alone would keep the invariant, so only the
"pushed twice" rule refuses the set — which is what M-PO10 must show, and does (E-PO3). The first layout
tried (a ball of r 270) was refused by the final-state check instead, so M-PO10 survived it; that is why
the layout was changed.

**DO-7 (bounded) — what an entry placed elsewhere names as `stopped_by`.** SD-B8 E3 names the
lowest-`EntityId` person whose disc covers `to`, else `None`. With objects, `to` can be unfree because of
an object alone (E1 now requires a radius of clearance from every footprint): E3 then names the first such
object in `ItemId` order — "the first person or object touched", as SD-O9 step 8 says for strides.

**DO-8 (bounded; a layout defect in §18.4) — PO-7's Q.** PO-7 places P at (2 900, 5 400) and Q at
(3 200, 5 000): 500 mm apart, which genesis refuses (`bodies-overlap`, CLEARANCE 595). Q stands at
(3 200, 4 600) instead: 1 265 mm from the observer (still beyond SHOVE_REACH, so its shove is still
`TooFarAway`), 854 mm from P, 640 mm from the ball's centre. Every claim of PO-7 is unchanged.

**DO-9 (bounded) — PO-5 d uses a box of half 100 and lays out the thrower beside the counter.** §18.4
does not fix PO-5 d's layout. A thrower must keep 295 mm from the counter and 405 mm (for a 110 mm
footprint) from the object, and the object must fit the counter's 600 mm depth: thrower (4 500, 5 800),
box half 100 at (4 950, 5 800), aimed at (4 950, 6 870), the middle of the counter's top. It landed on
the top, which also gives PO-4 b's last refusal (a box lying on the counter cannot be kicked) a real
object to refuse, instead of one placed there by hand.

**DO-10 (bounded) — PO-4 f and M-PO5 are a unit test of `flight.rs`.** PO-4 f needs "a test policy that
cuts a throw's flight after 2 sub-steps". The step bound is a parameter of `rapier::fly`, so the unit test
flies the throw with 2 sub-steps and lands it through the production `land`; no policy switch exists in
production code. M-PO5 is a real mutation of `land`.

**DO-11 (bounded; discovery about 12b's people path, kept unchanged) — a person within 610 mm of
somebody is not swept away from them.**
- Previous assumption: PO-6 a and b lay the shover 600 mm behind the target and expect the target 500 mm
  on (a) and at the wall (b).
- Audit evidence (E-PO5): with those layouts the target moved 301 mm with `stopped-short { by: None }`
  (a), and 300 mm (b). With the shover 700 mm behind, (a) is exactly (4 500, 5 000) and (b) (8 010 ± 1,
  5 000). The cause is 12b's resolver, unchanged: the shover stands within 2R + GAP = 610 mm of the
  target's path, so the corridor is not clear (SD-B6's definition) and the stride is swept; the contact
  sweep starts inside the character controller's 10 mm offset of the shover and advances 1 mm, and the
  candidate rule allows 300 mm beyond that. Every walker in that position — between CLEARANCE (595)
  and 610 mm from somebody, which a blocked walker reaches (608.7 mm, DB-6) — has the same short first
  stride away from them. The result is bounded and true (a stopped-short is recorded); it is not an
  invariant violation.
- Decision: the people path is not changed. PO-13 b freezes it byte for byte (a change here is a
  material stop), and §18.1 changes it only where objects enter. PO-6 a … d lay the shover 700 mm behind
  the target; a separate test, `a_shove_from_600_mm_is_cut_short_by_the_shover`, keeps §18.4's own
  layout and pins what it does.
- Follow-up (FU-12c-1, for the planning session; 12d or later): whether a stride moving away from
  somebody within the offset should be swept without them — a change to SD-B6's contact sweep and to
  12b's results, which needs its own design and a re-captured long-run base.

**DO-12 (bounded) — the scan reads bodies' facts as JSON, not through the pack's types.** The 12b scan
decodes presence's facts with presence's published types, which `tools/cli` depends on. `tools/cli`
does not depend on `mineworld-bodies`, and its `Cargo.toml` is outside §18.1's change set, so the scan
reads `body-formed`, `object-placed`, `object-moved` and `person-shoved` by event type and decodes their
JSON payloads field by field (entity references through the contracts' `EntityId`, positions through
`LocalPosition`). It reads the save only, never the code under test, and is an independent reading of
the encoding.

**DO-13 (SD-O18's ladder, walked; MATERIAL STOP) — see E-PO6.** Rung c1 (content) put six more objects
beside the doorways and the furniture — eight per room, the door objects 750 mm from the doorway
points. Rung p1 (`SHOVE_OFFER_REACH` = 800 mm: a shove is offered complete only within 800 mm, the
request's requirement staying 1 000 mm; beyond, it is offered without a payload). Rung p2 (kick and
throw complete for the nearest object only) was tried and reverted. AO-2 failed in a later bucket on
every rung. The tree holds c1 + p1, and PO-7's test adds a person `r` at 700 mm so that one shove offer
there is complete (p at 985 mm is now offered without a payload; PO-7 b submits the complete offers).
The criteria were never changed. The decision returns to the primary session (§18.9).

**DO-14 (defect, fixed) — a shove asked for up to half a millimetre more than 500 mm.** `shove::resolve`
scaled the shover → target vector by 500 / ⌊|d|⌋; for a vector whose length is not a whole number of
millimetres the result can be longer than 500 mm (e.g. (600, 3) → (500, 2), 500.004 mm). The 30-day
scan caught it (run 3: 385 × "moved 500 mm (at most 500)"). Fixed by dividing by ⌈|d|⌉;
`a_shove_never_asks_for_more_than_half_a_metre` failed before the fix ("b moved 250004 mm², more than
500²") and passes after it.

**DO-15 (bounded) — PO-13 a's kicks.** The prototype kicks a prop with an impulse in one of eight
directions; MineWorld's `kick` is a person's request, from within 800 mm, away from them. Each of the
sequence's kicks (`draw(3, r, 0) % 100 ≥ 85`, prop `draw(7, r, 0) % 4`, the prototype's own draws) is
made by the person nearest the prop (ties by key); one beyond reach first steps toward it, to 750 mm from
its centre, in one `move` of at most 1 999 mm (movement's MAX_STRIDE is 2 000, and |d| is rounded down).
363 kicks were accepted and 89 refused (the nearest person still out of reach after their one step:
the step was stopped short by somebody or something in the way).

**DO-16 (the primary session's ruling on DO-13, 2026-10-08) — rung p3, and seed robustness; MATERIAL
STOP again.** p3 amends SD-O13 and SD-O11 (the dated note after §18.3.2's table): an unaimed kick or
throw goes toward the room's free centre. PO-4 c's and PO-5 a's flight bounds, fixed in §18.4 for the old
direction, are restated along the new line with their magnitudes unchanged (1 500 … 3 500 mm along and
≤ 50 mm off for a kick; from 1 000 mm short of the aim to 2 000 mm past it and ≤ 100 mm off for a throw).
The ruling also fixed, before measuring, that AO-1 … AO-3 hold on seeds 7, 8 and 9: the committed test
keeps seed 7; `thirty_days_of_bodies_yard_at_seeds_8_and_9` (ignored, run with `--ignored`) is the
evidence for 8 and 9. All three seeds fail AO-2 for want of a push in a later bucket (E-PO7). Per the
ruling, no rung is added; the stop is reported with the counts.

**DO-17 (the primary session's second ruling, 2026-10-08) — rule p4: no launched object comes to rest
within 300 mm of a solid unless its flight was blocked; MATERIAL STOP a third time.** Implemented as the
dated note on SD-O13 states (`flight.rs`: `rests_clear`, `pulled_back`, `free_centre`;
`Footprint::keeps`; `REST_CLEARANCE`, `PULL_BACK_STEP`). "Blocked" is read as: no point of the line from
where the flight began to its end keeps the clearance. Scenario and mutation in E-PO8. All three seeds
fail AO-2 in a later bucket (seed 7 and 9 for want of a throw, seed 8 of a push); per the ruling, stopped,
with the counts and day-30 positions recorded.

**AO-2 ruling (primary session, 2026-10-08 — a decision taken after failed measurements, recorded as
such).** Finding: AO-2 as frozen asks every rare action in every 10-day bucket of every seed; after day
10 kicks, throws and pushes run at 0 … 5 per bucket in this 12-person, 16-object, two-room sandbox, so a
zero among 18 later cells is near-certain — the criterion measures the sandbox's content density, not
the pack, whose correctness is carried by PO-3 … PO-6 (± 1 mm literals), the full scan (no overlap, no
object in a solid) and determinism. **AO-2 FAILED as frozen, with p1, with p3 and with p4** (E-PO6,
E-PO7, E-PO8). Replaced by AO-2′: (a) each of kick, throw, shove, push and stopped-short by an object at
least once in every bucket summed over seeds 7, 8 and 9; (b) at day 30, on every seed, every object that
moved rests 300 mm (REST_CLEARANCE) from every solid or on a solid's top; (c) every action in days 1–10
of every seed. AO-1 and AO-3 unchanged. The committed seed-7 test asserts AO-1, AO-3, (b) and (c);
(a) lives in the ignored three-seed evidence test `thirty_days_of_bodies_yard_at_seeds_7_8_and_9`.
Result on the existing p4 saves (E-PO9): (a) holds, (c) holds, **(b) fails on every seed** (ten objects;
pushes, launches starting near a solid, and p3 kicks into the kicker) → stopped again, per the ruling's
point 4.

**Final AO-2 ruling (primary session, 2026-10-08; closes AO-2): (b) replaced by (b′) after the
measurement.** (b) as written was over-specified by the primary session: it required objects to rest
clear of solids even after ordinary pushes, which is not physically reasonable — pushing a box against a
pillar is legitimate. Its intent was "no degeneration: every object stays interactable". **(b′)
reachability:** at day 30, on every seed, every object has at least one free standing point within kick
reach (800 mm to its ground point), a free standing point being a point of the room's 50 mm lattice
where a person's 300 mm disc overlaps no solid and no other object. The ten objects that failed (b) are
E-PO9's table. The committed seed-7 test asserts AO-1, AO-3, (b′) and (c); the ignored three-seed test
asserts (a). Measured once with DO-18 (E-PO10): everything holds.

**DO-18 (defect fix to p3, by the same ruling) — an unaimed kick never drives the object into the
kicker.** When the line from the object toward the free centre runs ahead into the kicker's body (the
kicker ahead along it and the line within `PERSON_RADIUS + half + GAP` of the kicker's centre), the
object goes away from the kicker — along kicker → object, east if they coincide. Scenario test and
mutation M-DO18 in E-PO10.

## 18.12 Operator review and merge (post-merge record, planning session)

**Merged:** GitHub #67, merge commit `889d217` (2026-10-08), with a merge commit as frozen. PR head
`5ef5bff`; final executable head `145c7f7`, whose tree is the PR's code (E-PO11).

**The operator's review evidence, on the PR head, before the merge:**

- **Gates re-run:** 636 passed, 0 failed, 1 ignored (the three-seed evidence test,
  `thirty_days_of_bodies_yard_at_seeds_7_8_and_9`). The `harness = false` programs passed:
  `resolver-yard`, `cafe` and `clock`. This matches E-PO11.
- **Scope:** no forbidden path is touched. Re-checked by the planning session on the merge itself,
  `git diff --name-only 889d217^1 889d217`: every path lies under `systems/bodies/`,
  `worlds/bodies-yard/` or `tools/cli/tests/`, or is `Cargo.lock`, `docs/DECISIONS.md`,
  `docs/MODULE_SPEC.md`, `docs/MVP_STATUS.md` or this effort's plans. Nothing under `kernel/`,
  `contracts/`, `persistence/`, `server/`, `cognition/`, `clients/`, `authoring/`, `sdk/`,
  `worldpack/src/`, `tools/cli/src/`, `tests/acceptance/`, `systems/{presence,movement,item,inventory,
  installed}`, either town, or the root `Cargo.toml` (§18.9's frozen invariants).
- **The operator's own mutation:** `KICK_REACH` 800 → 1 000 mm. It failed
  `a_kick_reaches_800_mm_and_no_farther` (PO-4 a). The mutation was reverted. It is independent of the
  implementing session's mutations: none of M-PO1 … M-PO12, M-ITEM, M-P3, M-P4 or M-DO18 changed a
  reach constant.
- **After the merge, on main:** `ac1_composability` passes 13/13 and `precursor_vocabulary` (the I-2
  scan) 4/4 (PO-15 holds on the merged tree).

**Recorded as it happened.** AO-2 failed as frozen, and was decided by the primary session after the
measurements, not before them:

- **AO-2 failed as frozen** on every rung of SD-O18's ladder (c1, p1, p2; E-PO6). The primary session
  then added two rules that were not in the frozen design: **p3** (an unaimed kick or throw heads for
  the room's free centre; E-PO7) and **p4** (no launched object comes to rest within 300 mm of a solid
  unless blocked; E-PO8). AO-2 still failed with each.
- **DO-18 is a defect fix to p3**, not a new rule: p3 could kick an object into its own kicker.
- **AO-2 was replaced by AO-2′** after those failures. Its clause (b), "every moved object rests
  300 mm clear of every solid", was then **replaced by (b′), reachability**, after measuring that (b)
  failed on every seed (E-PO9). The reason given: (b) was over-specified — it forbade an ordinary push
  against a pillar, which is legitimate — and its intent was that every object stays interactable,
  which (b′) states directly. AO-2′ with (b′) held on seeds 7, 8 and 9 (E-PO10).

**Deviations:** DO-1 … DO-12, DO-14 and DO-15 stand as bounded, as recorded in §18.11. DO-13, DO-16 and
DO-17 are the material stops the AO-2 rulings closed. DO-18 is a defect fix.

**What 12c leaves.**

- **For 12d, carried:** QB-11's ≤ 1.5× bound on both towns' 300-day seed-7 dev runs, with versus
  without bodies, never re-scoped (DB-10 (b2)); F-B7 (the café's doorway point 200 mm from its wall);
  FU-12a-1 (the three allow-listed comments); **FU-12c-1** (a stride moving away from somebody within
  the controller's offset is swept with them, DO-11). 12d is being detailed by the planning session
  on `mvp0/s15-12d-plan`.
- **Measured for 12d's risk, not a bound on it:** bodies-yard's 300-day dev run with objects took
  30.7 s against 9.9 s without bodies, 3.1× (PO-14). The towns' bound stays 1.5× (DB-10 (b2)).

---

# 19. PR 12d — the town gets bodies (full design; DESIGN FROZEN 2026-10-08)

**DESIGN FROZEN (2026-10-08), primary session**, relayed by the coordinator, within authority the
operator has already given (the operator's own 12d-0 acceptance and overall ruling 5). The rulings on
QD-14 … QD-18 are in §19.8; the execution contract's freeze fields (worktree `impl-12d`, branch
`mvp0/pr-12d-towns`, tool discipline, material stops, budget) are in §19.9. Implementation in a fresh
session; nothing here was implemented by the planning session. One contract field is flagged for the
primary session before TD-C8 (§19.13, "Freeze note").

**Lifecycle:** drafted by the planning session on `mvp0/s15-12d-plan` on 2026-10-08, stacked on the 12c
post-merge docs PR (#72); held while 12d-0 (§20) ran; **refreshed by the planning session on
`plan/s15-12d-refresh` against `main @ 78ca5ae` (12d-0 merged), 2026-10-08 — PR design, ready for
freeze review.** Not frozen. Nothing in §19 authorizes implementation. The questions are §19.8 (QD-1 …);
QD-14 … are the refresh's.

**Refresh (2026-10-08, planning session, against `main @ 78ca5ae`).** What 12d-0 (§20, merged as #84)
and the lanes merged since 21f96ff change here, each edited in place and named:

- **Base and references.** §19's base is `main @ 78ca5ae` plus this refresh's docs PR. TD-14's
  bodies-yard and `long_run` references are 12d-0's re-captured bases (§20.14): bodies-yard 30-day
  `bd6a1002…80e6`, `long_run` `23f7fa76…5125`, `long_run_objects` `c8358f8b…c5b4`. TD-1's "before" is
  re-measured on the new base (E-TD0c).
- **Versions and notes.** 12d-0 took `bodies` to version 3 and `ARC-39` to note 3, so QD-6's doorway
  refusal takes `bodies` to **4**, and 12d's `ARC-39` note is **note 4** (SD-D5, SD-D16, TD-C1, TD-C3,
  TD-14, §19.6).
- **FU-12c-1 is closed** by 12d-0's SD-Z5 (QD-10 answered by events); 12d neither carries nor touches it.
- **The cost criterion.** QB-11's 1.5 × no longer exists as a bound: the operator re-scoped it for
  12d-0 after measuring (§20.5's amendments; E-Z3, E-Z7). TD-12 is rewritten as 12d's own criterion,
  fixed now, before any 12d measurement (`ARC-23`): the with-bodies towns stay within 12d-0's accepted
  TZ-9a numbers plus 10 %, and TZ-9b's 50 ms holds (§19.4 TD-12; QD-14 **[OM]**).
- **Inputs owed by other lanes** are listed in §19.10a; the source re-audit of §19.2 against the new
  base is §19.2a.

**Rulings received (2026-10-08, relayed by the coordinator; §19 is otherwise kept as drafted):**

- **QD-1 — operator decision: amend.** In the operator's terms: Social Café may own `items/` (loose
  objects as Item files with `body:`); Market Town carries them unchanged; the claim "Market Town =
  Social Café + installed packs + configuration" stays word for word; market-owned item sections stay
  Market Town's only. Check 3 and `ARC-35` are amended as a dated note quoting the operator, and a
  mutation shows the amended check still bites: **a market-owned section in a Social Café item file is
  refused** (added to TD-4 at freeze, beside M-TD2 … M-TD4).
- **QD-2 — primary-session ruling: precursor 12d-0, "bodies' cost"** (§20). QB-11's 1.5 × is not
  re-scoped. 12d-0's own acceptance measures E-TD0b's prototype against 1.5 × before 12d is frozen; if it
  cannot reach it, the work stops and goes back to the operator.
- **QD-7 — accepted as recommended;** the operator judges the doors in play.
- **QD-3 … QD-6, QD-8 … QD-13 — accepted as recommended;** QD-11: S12 has been told that 13a reads the
  passage. QD-10 places FU-12c-1 in 12d-0 (§20).
- **Consequence for §19 at its freeze:** 12d-0 moves bodies to `VERSION` 3 (§20), so QD-6's doorway
  refusal takes bodies to **4**, and TD-14's bodies-yard and `long_run` references are 12d-0's
  re-captured bases, not 12c's. §19's base becomes main after 12d-0. *(Applied by the refresh above.)*
- **Rulings on 12d-0 that bind 12d (2026-10-08, §20.13):** QB-11's 1.5 × re-scoped by the operator to
  TZ-9a ≤ 3.0 × and TZ-9b ≤ 50 ms (Z-D6), social-cafe's 3.60 × accepted, not passed (Z-D9); L1 (Class R)
  named as the next cost candidate; SD-Z1, SD-Z3, SD-Z4 dropped (Z-D4, Z-D7).

This section refines §5, §10.4, §11.1's 12d row, QB-5 and QB-11 from merged source. Where they and §19
disagree, §19 governs, and each difference is named with the finding that caused it (§19.2) and the
question that asks for it (§19.8).

## 19.1 Identity, base, approved scope

```text
PR            12d — the town gets bodies (S15, fourth of five)
base          main @ 78ca5ae (12d-0 merged as #84) and the PR carrying this refresh — Markdown only
              (was 21f96ff before the refresh). Re-audit §19.2 / §19.2a if anything under
              worlds/social-cafe, worlds/market-town, systems/bodies, systems/item, systems/movement,
              worldpack/src, tests/acceptance, tools/cli/tests, persistence/tests or
              clients/3d-spike/scripts moves again before the freeze
branch        mvp0/pr-12d-towns (the freeze's name), in /Users/yuema137/mineworld-worktrees/impl-12d,
              held by the implementing session only
audit         §19.2 (main @ 21f96ff, 2026-10-08), re-audited in §19.2a (main @ 78ca5ae)
scope         §11.1's 12d row; QB-5 (both towns install bodies); the cost criterion TD-12 (12d-0's
              accepted TZ-9a numbers + 10 %, TZ-9b ≤ 50 ms; QB-11's 1.5 × superseded by the operator,
              §20.5); F-B7; FU-12a-1; S14's R-12d-1 … R-12d-4 (step-15 §11.3); S12's
              R-PK-2 (`step-13-client-2d.md`, R-PK-2, RK-5, QS12-3) by coordination ruling 5 (overall.md "Parallel build-out");
              as refined by SD-D1 … SD-D16 and answered by QD-1 …
depends on    12c merged (889d217) and 12d-0 merged (78ca5ae). bodies, presence's seam and movement
              used as merged
merge         a merge commit, never a squash
```

**Goal.** Both towns get bodies, and their digests move once, here and nowhere else:

- **Every place of `social-cafe` and `market-town` has a `body:` section**: a floor and solid boxes in
  integer millimetres. The café's and the street's match the 3D slice's colliders within 150 mm, the
  tolerance S14 set for its geometry probe (step-15 §4.2). The store is The Flower Room (R-12d-2).
  The apartments, the workplace and the park, which the slice does not draw inside, get rooms that
  hold their people and match their files' own descriptions.
- **Every doorway point lies where a person fits** (F-B7): at least 310 mm (R + GAP) inside its floor
  and from every solid, on both sides. A doorway that does not is refused at load, by name (§11.1's
  adversarial criterion).
- **Loose objects lie where players meet them**: a box and a ball in the café and on the pavement
  outside it (R-12d-3), each at least 700 mm from every doorway point.
- **`bodies` is installed in both towns**, the same place geometry in both (R-12d-4, `ARC-35` check 3).
- **S12's item names (R-PK-2)** are folded into the same re-baseline: `item:` gains a `name`, and
  `item` discloses a catalogue of declared kinds on every perceived place.
- **The digests are re-baselined once**, with the before and after recorded, after the activity
  precondition (`ARC-23`) and the full scan hold, and the cost criterion (TD-12) holds on both towns.

**Change set.** Every path this PR may touch:

```text
worlds/social-cafe/world.yaml          systems += bodies; items: (the four objects)
worlds/social-cafe/places/*.yaml       body: sections; passage points moved (SD-D3, SD-D4)
worlds/social-cafe/items/*.yaml        new: cafe-ball, cafe-box, street-ball, street-box (body: only)
worlds/social-cafe/README.md           geometry, objects, the three actions
worlds/market-town/world.yaml          systems += bodies (same position as social-cafe's); items: the
                                       four objects added
worlds/market-town/places/*.yaml       byte-identical to social-cafe's (R-12d-4, check 3)
worlds/market-town/items/*.yaml        the four object files, byte-identical to social-cafe's; each of
                                       the twenty kinds' `item:` gains `name` (R-PK-2)
worlds/market-town/README.md           the same, and the names
systems/item/**                        R-PK-2: ItemName, the section's `name`, item-kind-declared and
                                       ItemKind carry it, the catalogue disclosed on a place; VERSION 2
systems/bodies/src/{genesis,system}.rs the doorway check (SD-D5); VERSION 4 (QD-6; 12d-0 took it to 3)
systems/bodies/Cargo.toml, Cargo.lock  mineworld-movement as a crate dependency, for Passages only
                                       (SD-D5, QD-5)
systems/bodies/tests/{rapier_pin,isolation,genesis}.rs   QD-6's literal; QD-5's dependency claim; the
                                       doorway refusals
tests/acceptance/tests/ac1_composability.rs   check 3 admits Social Café's own items (SD-D9; QD-1 [OM])
systems/movement/src/action.rs, worldpack/src/read.rs,
tests/acceptance/tests/seam_vocabulary.rs     FU-12a-1: three comments reworded, three allow-list
                                       entries removed (QD-9)
tools/cli/tests/town_bodies.rs         new: the towns' activity, scan, counterfactual and cost
tools/cli/tests/bodies/mod.rs          the scan reads any world's place files (SD-D12)
existing tests                         the named literal edits of §19.6, each claim unchanged (QD-8)
docs/DECISIONS.md                      notes: ARC-35 (check 3), ARC-37 (the catalogue), ARC-39 note 4
                                       (the towns install bodies; doorways)
docs/MODULE_SPEC.md                    §4.1: `item`'s name and catalogue; `body`'s doorway refusal
docs/MVP_STATUS.md                     the S15 rows, the evidence rows
.structured-coding/plans/mvp0/{step-11-bodies,handoff}.md   this ledger, the handoff
```

Paths with no diff:
- `kernel/`, `contracts/`, `persistence/src/`, `server/`, `cognition/`, `clients/`, `authoring/`,
  `sdk/`;
- `systems/presence/`, and every System Pack other than `bodies`, `item` and FU-12a-1's comment lines
  in `movement`;
- `tools/cli/src/`, `worldpack/src/` beyond FU-12a-1's comment;
- `worlds/bodies-yard/`;
- the root `Cargo.toml`.

**Non-goals.**
- The 3D client: colliders, the correction rule, the frame binding's new doorway offset, the
  florist bound to `store` — 12e and 16c (step-15 §13).
- The 2D client reading the catalogue (13c) or drawing bodies (13d).
- FU-12c-1 (a stride moving away from somebody within the controller's offset): closed by 12d-0's
  SD-Z5 (§20.14); nothing of it is 12d's.
- Names for loose objects and organizations: an object is not a declared kind, so the catalogue does
  not name it (QD-4); organizations are not needed by any Demo A interaction (R-PK-2).
- R-PK-1 (doorways say where they lead): S12 13c, with no digest change.
- Any change to presence, movement's rules, the paced controller, the kernel or contracts.
- Any change to bodies' rules (constants, sweep, nudge, push, flight, entry placement). If TD-12 fails,
  §20.7's L1 (Class R) is the named next candidate — a design change brought to the operator, not done
  here (§19.4 TD-12).

## 19.2 Source audit (`main @ 21f96ff`, 2026-10-08)

Each finding was read in this session from the file named, or measured (E-TD0). The 3D slice was read
on `main` (`clients/3d-spike`, merged as #50), read-only. S12 13a and S14 16a were read in their
implementation worktrees, read-only, at the states named in F-D12.

| ID | Finding | Evidence | Consequence for 12d |
| --- | --- | --- | --- |
| **F-D1** | **AC-1's check 3 forbids any item in Social Café, and any `body:` in a Market Town item file.** `compare_manifests` fails "`items` must be absent in Social Café and present in Market Town"; `world_delta_failures` fails "items/: must exist in Market Town only"; `market_only_content` admits in an item file only `tags`, `note` and sections a market pack owns, and `body` is owned by `bodies`. `ARC-35` item 4, approved by the operator, says the same in words: "`items/` and `organizations/`: present only in Market Town". | `tests/acceptance/tests/ac1_composability.rs:1080–1082, 1121–1138, 1178–1189`; `docs/DECISIONS.md:2495–2500` | A loose object is an Item file (`ARC-36` note, QB-3). So **no town can carry an object without amending check 3 and `ARC-35` item 4**: in Social Café the `items:` list and `items/` fail; in Market Town alone, the object's `body:` fails. R-12d-3 (objects where players meet them) and the operator's "at least one box and one ball" therefore need an operator decision (QD-1 **[OM]**, SD-D9). |
| **F-D2** | **Check 3 accepts geometry that is identical in both towns.** Every key of a Social Café place file must be present in Market Town's with an equal value (structural YAML comparison, map order free); `systems` must be Social Café's list in order, then exactly the six market packs. | `ac1_composability.rs:1028–1054, 1094–1121` | `body:` sections byte-identical in both towns pass. `bodies` must sit at the same position in both `systems:` lists, before Market Town's six (SD-D1). Check 1 (the 11d/11e merges) and check 2 (crate edges) are untouched: `bodies → item` and `bodies → movement` are `systems/` edges (F-O11). |
| **F-D3** | **Every doorway point is too close to a wall, on both sides.** `here`: the café (1 610, 200), the apartments (2 000, 200), the store (1 500, 200), the workplace (1 500, −200) — each 200 mm from its front wall — and the park (1 000, 14 800), 200 mm from a 15 m edge. `there` on the street: the north doors at y 3 000, 200 mm inside the slice's north façade line (y 3 200); the south doors at y −13 000, which **is** the slice's south façade line — outside any floor the slice allows. | `worlds/social-cafe/places/*.yaml`; slice: `street.gd:31–38`, `terrace.gd:19, 114` (F-D6) | F-B7 generalized: with a floor, every crossing would land by entry placement's E3 rather than on the doorway (SD-B8), and the south doors would be outside the street. Every point moves to ≥ 310 mm inside (SD-D3). |
| **F-D4** | **Nothing refuses a doorway inside a wall.** Bodies' genesis checks people (overlap, outside, in a solid, capacity) and objects; it never reads movement's `Passages`. Bodies depends on presence only and does not link `mineworld-movement`. | `systems/bodies/src/genesis.rs:62–137`; `systems/bodies/Cargo.toml` | §11.1's adversarial criterion for 12d ("a doorway authored inside a wall … is refused at load by name") needs a new check that reads `Passages` (SD-D5, QD-5). |
| **F-D5** | **The slice's café, in the café's own frame, is exactly the world's room.** `MineWorldSpace.to_3d` maps (x, y, z) mm to Godot (x, z, −y) m. `SliceLink` binds a frame by translating the place's disclosed doorway onto a scene door point: the café's `here` onto a point 0.2 m inside the façade's inner face, the street's `there` onto a point 0.2 m outside the façade. That puts the café's origin on the room's inner south-west corner, scene (1.84, −8.44): the inner faces are café x 0 and 8 320, y 0 and 10 320, as `cafe.yaml` says. The street's origin is scene (3.45, −4.90). | `clients/protocol/mineworld/space.gd:54–61`; `slice_link.gd:50–53, 62, 70–92`; `slice/cafe.gd:26–38`; `cafe_interior.gd:35–37` | The café's floor is (0, 0)–(8 320, 10 320). **Moving a doorway point moves the client's binding** by the same amount unless the client's 0.2 m constants move with it: a 12e/16c change, stated as a cross-lane impact (§19.10). |
| **F-D6** | **The slice's colliders, in world millimetres.** Café: the counter (3 860, 6 570)–(8 320, 7 350) h 1 060; the back worktop (3 860, 9 290)–(8 320, 9 910) h 880; the window bench slab (2 360, 460)–(8 020, 900), top 470; the west dresser (0, 2 110)–(340, 4 510) h 840; three round tables at (4 860, 3 160), (7 110, 4 560), (2 260, 8 560); the open door leaf, a diagonal board whose box is (785, −187)–(1 120, 734); the back-room blocker (195, 10 230)–(1 245, 10 370). The door's clear opening is café x 1 120 … 2 100. Street: floor band y −13 000 … 3 200 (south and north façades), x −37 450 … 30 550 (the street volume); a retaining wall (15 050, 1 000)–(26 050, 3 200) h 1 900; 7 lamps, 13 bollards, 2 bins, 12 trees (cylinders r 130 … 264), 7 stone planters (boxes), 3 terrace tables (cylinders r 620), 2 planter boxes, 3 clay pots, 2 A-boards, 2 benches, a fingerpost and a bicycle. The Flower Room: interior 7 900 × 7 350, door 8 390 mm east of the café's door, its own colliders (staging, shelves, counter, step, table, crate, door leaf). The other façade doors are not enterable: the Flats' house door at street x −19 700 (north), a house door at −5 700 (south), the Lakeside Deli at 4 710 (south), a house door at 15 300 (south), among others. | `cafe_interior.gd:22–24, 136–152, 285–293, 334–336, 436–488, 533`; `cafe.gd:216–218, 258–281`; `street.gd:31–38, 62–139`; `streetscape.gd:41–325`; `slice_world.gd:117–194`; `shop_interior.gd:25–304`; `terrace.gd:83–147, 220–233`; `profile.gd:385–419` (planning agent's audit, cited line by line) | The geometry of SD-D2. Three facts shape it: (1) slice cylinders and a diagonal leaf are not boxes (SD-D2's approximation rules, QD-3); (2) colliders placed with `put_solid` (the tables, pots, planter boxes, A-boards, the florist's shelves, table and crate) are sized at run time from the glTF asset's bounds × 0.92, so their sizes are not in code — 12d reads them from the assets (SD-D2); (3) the street has about 60 colliders, under bodies' 64-solid limit (`SOLIDS_MAX`). |
| **F-D7** | **The slice's doors do not match four of the world's.** The world's store door (street x 12 000) lies 3 610 mm east of The Flower Room's door, inside its façade; the apartments' (−12 000, north) lies on the Flats' façade 7 700 mm from its door; the park's (8 000, south) lies 1 950 mm inside a unit with no door near; the workplace's (−6 000, south) is 300 mm from a house door. | as F-D6; `worlds/social-cafe/places/{store,apartments,park,workplace}.yaml` | R-12d-2: each street-side point moves onto a slice door (SD-D4). Walks get longer on the street (QD-7). |
| **F-D8** | **Capacity is per place, against the whole world's population.** A shaped place must offer at least 4·(people − 1) + 1 points of the 650 mm grid inside its floor shrunk by 300 mm and clear of solids by 300 mm, plus each object's `blocks`. With twelve people: **45 points**, in every shaped place, the smallest included. | `systems/bodies/src/genesis.rs:79–92`; `geometry.rs:276–280`; SD-O5 | A floor needs about 4.5 m × 4.5 m net of solids. Every room of SD-D2 has margin; the numbers are part of validation (TD-2). |
| **F-D9** | **Identities: items come after people, in key order.** Social Café's ids 1–18 do not move when items are added. In Market Town the four object keys sort among the twenty kinds (`cafe-ball` after `bread`), so most kinds' ids and both organizations' move. | `worldpack/src/load.rs:1–33`; `worldpack/src/read.rs:176–205` (a `BTreeMap`, key order) | Expected under a re-baseline; named so no test or client assumes a kind's raw id (§19.6). |
| **F-D10** | **Thirteen test files hold a literal or a behaviour that 12d's content changes** — the systems list (`commands.rs:34`, `server_command.rs:43–53`, `worldpack/tests/social_cafe.rs:67–77`), `inspect`'s system lines (`inspect.rs:36–37`, `social_composition.rs:380, 420`), genesis counts ("53 genesis fact(s)": `commands.rs:40`, `content_kinds.rs:136`, `social_cafe.rs:219, 378`), entity counts (`server_command.rs:34, 85`, `social_cafe.rs:109`, `content_kinds.rs:122–132`), the old doorway (`social_cafe.rs:647, 803, 832, 843`), Social Café's absent `items:` (`content_kinds.rs:39–40, 114`; the copy in `social_cafe.rs:338–391`), otto's single arrival (`routines.rs:225`), and straight walks across the café whose every stride must be accepted at the requested point (`ac15_one_alice.rs`, `restart.rs`, `social_cafe.rs:709–717`, `ac13_semantic_parity.rs` replaying the frozen `clients/protocol/evidence/request-{2d,3d}.json`). | planning agent's audit of every file, cited by line; §19.6 lists each | Literal edits with unchanged claims (QD-8). The straight walks are a geometry constraint on SD-D2: the café's solids and objects keep clear of those lines (SD-D2, TD-9). |
| **F-D11** | **R-PK-2 meets an `item` pack that discloses nothing.** `AuthoredItem { category }` denies unknown fields; `item-kind-declared { item, category }` (schema 1) is reduced into `ItemKind { category }` (schema 1); `PerceptionProvider for ItemSystem {}` is empty; `ARC-37`: "items are never perceived". | `systems/item/src/{section,event,component,system}.rs`; `docs/MODULE_SPEC.md:418–420` | R-PK-2 is an `item` change (SD-D10): a section field, a fact field, a component field, a disclosure, `VERSION` 2, and an `ARC-37` note. Market Town's genesis facts change; Social Café does not install `item`. |
| **F-D12** | **The in-flight clients assume today's doorways.** S12 13a (`impl-s12-13a`, uncommitted work included): `drive.gd:133–149` finds the café's door on the street by exact equality with (0, 3 000); `godot.yaml:58` draws the café from `door_from_left_m 1.61`; `client_2d.rs:165–181` walks the wanderer straight toward Alice's counter and carol from her genesis point. S14 16a (`impl-s14-16a`): A16-7 holds the café's people, the door (1 610, 200) ↔ (0, 3 000), and Ivan; T-2 relies on the florist being reported as the street; `slice_link.gd:80–86` binds the doorways at 0.2 m. Neither has a geometry probe; the 150 mm probe is 12e's (`--world --geometry`, step-15 §4.2). | planning agent's audit of both worktrees, cited by line | Cross-lane impacts (§19.10): each moved point breaks an exact-equality lookup or a 0.2 m binding. S14 anticipated it (R-12d-1, R-12d-2, F-S14-9); S12 13a's exact match on (0, 3 000) does not (QD-11). |
| **F-D13** | **The offered band is shared.** The paced controller picks uniformly among all available complete affordances. In Market Town those are `give`, `buy`, `eat` and `drink`; with bodies, `kick` and `throw` per object within 800 mm and `shove` per person within 800 mm (12c's p1) join them. | `cognition/rule-controller/src/offered.rs`; `systems/bodies/src/offer.rs:22–89`; `geometry.rs:96` | Market activity is drawn less often wherever people stand close — the café and the store. Market Town's `market_lives` (a purchase, a wage per holder, production, consumption and a give from every seat, in every bucket) is the guard (TD-6); the remedy, if needed, is bodies' offer policy, never the controller (`ARC-34` item 4, SD-O18). |
| **F-D14** | **The paced controller's walking is bounded by nothing in the world.** `wander` draws ±1 400 mm per axis; `approach` stops 1 000 mm short; `through` aims for the exact doorway point. In bodies-yard, 63 % of moves end stopped short. | `cognition/rule-controller/src/paced.rs:66–105, 266–362`; DB-10 | With walls the towns' moves will often be swept, not fast-pathed: the cost risk of TD-12, and the activity risk of TD-5 (R-B8). |
| **F-D15** | **Baselines reproduce on this base.** Both 300-day seed-7 runs give E-RS0's digests and counts (E-TD0). | E-TD0 | The "before" of the re-baseline (TD-1). |

## 19.2a Re-audit (`main @ 78ca5ae`, 2026-10-08, the refresh)

A read-only re-audit of every finding above and of §19.6's rows against `main @ 78ca5ae`, by the planning
session's audit agent, cited by line; E-TD0c measured. Since 21f96ff main gained 12d-0 (#84), test
hygiene (#77), E-b package identities and requirements (#78, `ARC-53`), S14 16a (#79), 13a's CI (#63)
and S19 documents. **Every finding holds in substance;** the deltas:

| ID | Holds? | What moved, and the consequence |
| --- | --- | --- |
| F-D1 | holds | Citations shift: `ac1_composability.rs` items/organizations arm 1079–1081; `market_only_content` 1122–1137 (`ENTITY_FIELDS` 1008 still `["tags","note"]`); `world_delta_failures` from 1156, the market-only loop 1177–1187; `ARC-35` item 4 now `DECISIONS.md:2508–2513`. |
| F-D2 | holds | `compare_systems` 1026–1052, `compare_content` 1093–1120. New since 21f96ff: `ARC-53`'s `world.version`, `license` and top-level `mineworld: "^0.1"` in all three worlds, identical in both towns — 12d must keep them identical (it has no reason to touch them). |
| F-D3, F-D6, F-D7 | holds, citations unchanged | No place file and no slice scene file (`cafe_interior.gd`, `cafe.gd`, `street.gd`, `streetscape.gd`, `slice_world.gd`, `shop_interior.gd`, `terrace.gd`, `profile.gd`) changed. |
| F-D4, F-D8 | holds, citations unchanged | `genesis.rs` unchanged (`fits` 62–137, capacity 79–92); bodies still links no `mineworld-movement` (`Cargo.toml:10–25`). |
| F-D5 | holds | `slice_link.gd` citations +3: `PLACE_KEY` 73–77 (the florist still bound to `street`), `door_point` 83–88 with the 0.2 m constants at 87–88, `origin` 93–95; the binding still reads the disclosed passages (101–117). |
| F-D9 | holds | `read.rs` citation now 230–259. `clients/protocol/evidence/affordances-market-town.log:12–14` records raw Market Town item ids 19 and 34; no check asserts them (`run.sh:124–129` greps only), so they change only if that evidence is re-recorded — not 12d's (QD-18). |
| F-D10 | holds, **one gap** | Per-row citations in §19.6 (refreshed). Missing before: `tools/cli/tests/milestone_c.rs:172–215` (`walk_into_the_cafe`) walks Market Town straight from a seat to its door, along the street to the café's door and in, every stride `walk_accepted`; it reads the doorways from the pack (moving them is fine), but with bodies installed its straight lines must not meet a street prop or a person-free solid — added to §19.6 beside ac15/restart. Also restated by 12d: `worldpack/tests/social_cafe.rs:769–770`'s doc comment (the old door points); `server_command.rs:96`'s perceived café entities `[2, 7, 8, 17, 18]` should survive (objects are disclosed in `LooseObjects`, not perceived as entities) — confirmed in TD-C5. |
| F-D11 | holds | `item` unchanged but for `const PACKAGE` (`system.rs:29`); `VERSION` 1 (`system.rs:41`); `MODULE_SPEC.md`'s "Disclosed to nobody" now 440–442. |
| F-D12 | **changed** | S12 13a's client (`drive.gd`, `godot.yaml`, `client_2d.rs`) is **not on main** (only 13a's CI, #63); nothing on main finds the café door by equality with (0, 3 000). S14 16a merged (#79): `slice_probe_world.gd` (modes `link`, `conversation`, `target`) hard-codes scene points (the counter at (8.16, 0, −10.20), the door at (DOOR_X, −9.6)) valid only while the 0.2 m binding holds, and its `_street_watch` prints — never fails on — each disclosed street doorway's distance to the nearest drawn door. **There is still no `--geometry` mode and no 150 mm probe on main** (only specified, step-15 §4.2): TD-3 stays a hand-computed table, its executable form 12e's. |
| F-D13, F-D14 | holds | `offer.rs`, `offered.rs`, `paced.rs` unchanged. |
| F-D15 | holds | Re-measured on 78ca5ae: both towns' 300-day digests and counts equal E-TD0 (E-TD0c). |
| *new* F-D16 | — | **`bodies` is version 3 and `ARC-39` has notes 1–3** after 12d-0 (`system.rs:65`; `DECISIONS.md:3226, 3264, 3300`): 12d takes `bodies` to 4 and writes `ARC-39` note 4. **No package or requirements change follows** from installing `bodies` in a town or from either version bump: package versions are the workspace's `CARGO_PKG_VERSION` (`packages/src/declared.rs:125–135`), `bodies` is in the bundled installed set (`systems/installed/src/lib.rs:34, 48`), and a world without `requires:` resolves within the build (`worldpack/src/read.rs:89–90`, `requirements.rs:59–72`). The new tests `packs.rs`, `requirements.rs` (CLI and worldpack) and `server_command.rs`'s SA-4 hold no literal 12d's content changes. |
| *new* F-D17 | — | **12d-0's cost is 12d's starting point** (§20.14): on E-TD0b's prototype ≈ 45–49 s of CPU per 300 days with bodies, 2.999 × and 3.60 ×; Rapier strides ≈ 60 % of bodies' remaining cost, E3's search gone (E-Z7's addendum). The prototype is §19.3.1's geometry with placeholder put_solid sizes, so the real towns' cost may differ slightly — the 10 % of TD-12's headroom. |

## 19.3 Design (SD-D1 … SD-D16)

### 19.3.1 The geometry, place by place (SD-D2)

Every coordinate is integer millimetres in the place's own frame, which does not move: each place's
origin stays where its file's header comment puts it, so every authored person keeps their position.
`h` is a solid's height above the floor. A slice cylinder of radius r becomes a box (QD-3):

```text
r ≤ 500 mm   one square of half-side ⌊r · 0.7071⌋ (inscribed): a face lies at most 0.293·r inside the
             cylinder and never outside it — at most 147 mm for r = 500
r > 500 mm   two crossed boxes of half-extents (r, ⌊0.65·r⌋) and (⌊0.65·r⌋, r): for r = 620, (620, 403);
             the worst point is 38 mm off the cylinder
a diagonal   the open door leaves: their axis-aligned box, clipped to the floor. Its faces lie up to the
board        board's diagonal from the board; the 150 mm probe cannot hold a diagonal against an
             axis-aligned box, so 12e's probe exempts the leaves by name (QD-3)
put_solid    the asset's glTF bounds (the POSITION accessor's min/max of the mesh the scene loads) × 0.92,
             centred where the scene places it, as `dressing.gd:135–140` sizes the collider. Read in
             TD-C1 and recorded per prop (E-TD1); the planning sizes below are placeholders, marked ≈
```

**The café** (`places/cafe.yaml`; frame unchanged — the inner south-west corner, F-D5):

```text
floor          (0, 0) – (8 320, 10 320)                       the inner faces (cafe_interior.gd:35–37)
solids         counter         (3 860, 6 570) – (8 320, 7 350)   h 1 060
               back worktop    (3 860, 9 290) – (8 320, 9 910)   h   880
               window bench    (2 360,   460) – (8 020,   900)   h   470   the seat slab; the stools
                                                                           and front board are visual
               dresser         (    0, 2 110) – (  340, 4 510)   h   840
               table east      ≈ (4 860, 3 160) ± 300            h   750   put_solid
               table north     ≈ (7 110, 4 560) ± 300            h   750   put_solid
               table back      ≈ (2 260, 8 560) ± 300            h   750   put_solid
               door leaf       (  785,     0) – (1 120,   734)   h 2 070   clipped at y 0 (QD-3)
               back-room board (  195, 10 230) – (1 245, 10 320) h 2 000   clipped at the floor
doorway        here (1 610, 400) — was (1 610, 200)              F-B7: 400 from the front wall,
                                                                  490 from the leaf's box, centred on
                                                                  the door's clear opening x 1 120 … 2 100
```

**The street** (`places/street.yaml`; frame unchanged — the café's door at x 0, the north façade's
doorways at y 3 000 before 12d):

```text
floor          (−37 450, −13 000) – (30 550, 3 200)    the façade lines and the street's volume
solids         about 60 (≤ 64, SOLIDS_MAX), each from streetscape.gd / street.gd / slice_world.gd:
               retaining wall (15 050, 1 000) – (26 050, 3 200) h 1 900; lamps (squares, half 91),
               bollards (half 113), bins (half 169), trees (half 156 … 186), stone planters (their
               boxes), terrace tables (crossed pairs, (620, 403)), planter boxes, clay pots and A-boards
               (put_solid), benches (their boxes), the fingerpost (half 98), the bicycle (half 353)
               — the full list, with each source line, is TD-C1's table (E-TD1)
doorways       café         there (     0,   2 800)   was (0, 3 000): 400 inside the north façade
               store        there (  8 390,   2 800)   was (12 000, 3 000): The Flower Room's door
               apartments   there (−19 700,   2 800)   was (−12 000, 3 000): the Flats' house door
               workplace    there ( −5 700, −12 600)   was (−6 000, −13 000): the house door 300 mm east
               park         there ( 15 300, −12 600)   was (8 000, −13 000): the unit's house door (QD-7)
```

**The store** (`places/store.yaml`; frame: the inner south-west corner of The Flower Room — the store's
header already defines its origin as its inner south-west corner, so the frame's meaning is kept and
only its extent is now authored):

```text
floor          (0, 0) – (7 900, 7 350)                 shop_interior.gd:39–42
solids         window staging  (2 100,   110) – (7 200,   950)   h 1 200
               work counter    (4 500, 5 750) – (7 650, 6 450)   h   950
               bucket step     (7 230, 1 150) – (7 850, 4 550)   h   220
               display shelves ≈ (220, 2 550) and (220, 3 800)   put_solid
               shelves         ≈ (5 350, 7 190) and (6 650, 7 190) put_solid
               display table   ≈ (4 050, 3 350)                 put_solid
               crate           ≈ (7 400,   600)                 put_solid
               door leaf       (  230,     0) – (  560,   850)   h 2 080   clipped (QD-3)
doorway        here (1 040, 400) — was (1 500, 200): the door's centre, fx 1 040 (opening 530 … 1 550)
person         felix (3 000, 3 000), unchanged: 600 mm from the display table's placeholder box
```

**The apartments, the workplace, the park** (no slice interior; rooms sized to their headers'
descriptions and to their people, and nothing else — `VISUAL_SLICE.md` §4 keeps them unenterable in 3D,
so there is nothing to match):

```text
apartments  floor (0, 0) – (8 000, 6 000)           "an entrance hall … and the stairs up"
            solids  stairs (0, 4 200) – (2 400, 6 000) h 3 000
            here (2 000, 400) — was (2 000, 200);   carol (3 000, 2 500), otto (6 000, 4 000) unchanged
workplace   floor (0, −7 000) – (9 000, 0)           the origin is the inner north-west corner
            solids  meeting table (4 000, −5 800) – (6 400, −4 800) h 750
                    desks (6 500, −2 000) – (8 400, −1 200) h 750
                    kitchenette (0, −7 000) – (2 400, −6 400) h 900
            here (1 500, −400) — was (1 500, −200); grace (3 000, −3 000), hana (5 000, −3 500)
park        floor (0, 0) – (20 000, 15 000)          "about 20 m by 15 m"
            solids  bench (3 000, 12 000) – (4 900, 12 700) h 840; bench (12 000, 5 000) – (13 900,
                    5 700) h 840; a fountain (9 000, 6 000) – (11 000, 8 000) h 600
            here (1 000, 14 600) — was (1 000, 14 800); dev (5 000, 9 000), erin (6 500, 9 500)
```

**The objects** (`items/<key>.yaml`, the object form of `body:`, tags only — no `item:`, so never a
declared kind, QB-3):

```text
cafe-box      box half (200, 200, 200)   café   (7 300, 2 200)   by the window, east of the lane
cafe-ball     ball 110                    café   (2 600, 5 600)   west of the lane to the counter
street-box    box half (250, 250, 250)   street (−3 000, 1 200)  on the pavement west of the door
street-ball   ball 110                    street ( 2 200,   600)  on the pavement, between the terrace
                                                                   tables and the kerb
```

Each lies at least 700 mm from every doorway point and every authored person, and at least 420 mm
(R + GAP + its half-size) from the straight lines F-D10's tests walk, so those strides still meet
nothing (TD-9).

### 19.3.2 Decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-D1** | **Install.** `bodies` is appended to Social Café's `systems:` after `schedule`, and inserted at the same position in Market Town's, before its six market packs. Every earlier reduction order stays as it was. | F-D2: check 3 requires Social Café's list as Market Town's prefix. Bodies depends on presence only. |
| **SD-D2** | **Geometry as §19.3.1.** Every slice collider standing on a walkable floor of the café, the street or The Flower Room is a solid; nothing the slice does not collide with is a solid. The apartments, workplace and park get rooms from their own descriptions. A solid's height is the collider's top above the floor. | The server is the single source of layout ("One world, two views"); the 150 mm probe is two-directional (step-15 §4.2), so a collider missing on either side is a defect. Decoration is the client's. |
| **SD-D3** | **Doorway points, inside.** Every `here` and every street-side `there` lies 400 mm inside its floor's edge, at least 310 mm (R + GAP) from every solid. 400 rather than 310: the binding constant a client moves (F-D5) is then a round 0.4 m, and 90 mm of margin absorbs a solid's rounding. | F-B7 (R-12d-1). E1 then places a crossing exactly on the point (SD-B8). |
| **SD-D4** | **The street's doors onto the slice's** (R-12d-2): the store onto The Flower Room's door; the apartments onto the Flats' house door; the workplace onto the house door 300 mm east; the park onto the unit house door at x 15 300 (QD-7). Each `there` keeps its side (north y 2 800, south y −12 600). The store's and the street's header comments, which state the old distances, are rewritten with the new ones. | F-D7. Every door a person walks through in 3D is a door the scene draws. |
| **SD-D5** | **A doorway inside a wall is refused at load** (§11.1). Bodies' reduction of `place-shaped` also checks every doorway point of the place, from movement's `Passages` (`here` of each passage out of it, and `there` of each passage into it): inside the floor shrunk by R + GAP, and at least R + GAP from every solid. Refusal `bodies-doorway`, naming the place, the passage's other place and the point, and the distance. A crate dependency on `mineworld-movement` for the read-only `Passages` type, as 12c's on `mineworld-item` (QD-5); no system dependency — a world without movement has no passages to check. Bodies `VERSION` 4 (QD-6; 12d-0 took it to 3). | F-D4. A doorway inside a wall would otherwise be a silent E3 shift on every crossing — exactly F-B7 — rather than a load error. Genesis orders passages before sections (F-B4), so `Passages` is reduced when `place-shaped` is. |
| **SD-D6** | **Objects** as §19.3.1, identical files in both towns. Their keys sort after the people's, so Social Café's ids 1–18 do not move (F-D9). | R-12d-3, R-12d-4; S14 §3.1's "interacts with one object" happens in the town. |
| **SD-D7** | **People are not moved.** Every authored position stays; each is checked by bodies' genesis (outside, in a solid, overlap, capacity) through `mineworld validate`. | The towns' people files stay byte-identical, so check 3's person comparison is untouched and no test that names a person's position changes. |
| **SD-D8** | **The 3D binding is not 12d's.** 12d moves the doorway points; the slice's `door_point` constants (0.2 m, `slice_link.gd:80–86`) move to 0.4 m in 12e (or 16c, whichever first connects to a 12d world), with the florist bound to `store` (F-S14-9). Until then a connected slice draws the café 200 mm off and the street 200 mm off, and the florist hack fails against the street's floor. | I-S14-2: no S14 PR touches `worlds/`; no 12d path touches `clients/`. Stated in §19.10 so neither lane is surprised. |
| **SD-D9** | **Check 3 admits Social Café's own items** (QD-1 **[OM]**). Amended rule, in `ac1_composability.rs` and as a dated note on `ARC-35` item 4: Social Café may have `items:` and `items/`; Market Town's `items:` contains Social Café's (as a set); every Social Café item file exists in Market Town with an equal value (`compare_content`, as places and people); every item file only Market Town has carries only the format's fields and market sections (`market_only_content`, unchanged); `organizations/` stays Market Town's only. The claim is unchanged: **Market Town is Social Café plus configuration.** Its unit test gains the cases (a social item missing in Market Town; differing; a market-only item with `body:`). | F-D1. A loose object is Social Café's content, so it belongs to both towns, exactly as a place's `body:` does. Without the amendment, no town can carry an object. The operator approved `ARC-35` and accepted AC-1 on this test; amending its letter is theirs. |
| **SD-D10** | **R-PK-2, item names, in `item`.** `item: { category, name }`, `name` an `ItemName` (1–64 bytes, no control characters, no surrounding whitespace — naming's display-name rule, restated in `item`, which depends on no pack), **required**. `item-kind-declared` and `ItemKind` carry it (schema 2 each); `VERSION` 2. `item` discloses, to whoever perceives a place, an `item-catalogue` record on the place: `[{ item, category, name }]` for every declared kind, in `ItemId` order, built from `ItemKind` at disclosure (economy's `Listing` precedent, F-O5). The twenty Market Town kinds get names (QD-4). `ARC-37` gains a dated note: kinds are now disclosed as a catalogue; items are still never perceived as entities. | Coordination ruling 5: the names change Market Town's genesis facts, so they land in the one re-baseline (QS12-3, RK-5). Required, because a kind without a name is what F-41 is. A disclosed-only record type is audited first in TD-C2 (QD-12). |
| **SD-D11** | **Activity before determinism, decided now** (`ARC-23`; §19.4 TD-5 … TD-8). With bodies, both towns must still live as they did — every existing activity assertion of `run.rs`, `run_restart.rs`, `routines.rs`, `market_town.rs`, `milestone_b.rs`, `milestone_c.rs` and `market_composition.rs` passes unedited in its claim — and the physical actions must happen and stay rare. A remedy, if one is needed, is in content (object and doorway placement, §19.4's ladder), never in the controller (`ARC-34` item 4, SD-O18), never in bodies' rules within 12d (they would move bodies-yard), and never by changing a criterion. | 12c's AO-2 was decided after failed measurements (§18.12); 12d fixes its criteria before any town run, and states them per run, not per 10-day bucket, for the rare actions — the lesson of AO-2′ (a). |
| **SD-D12** | **The scan, for any world.** `tools/cli/tests/bodies/mod.rs` today asserts bodies-yard's room literals. It gains a reader of a world's place files (`body:` floors and solids, by the YAML, never by bodies' code) and runs unchanged on bodies-yard and both towns. | One instrument for the invariant on every world with bodies; read from the save and the authored files only (DO-12). |
| **SD-D13** | **The counterfactual and the cost copy.** A test-time copy of each town without `bodies` in `systems` and without every `body:` (places' and items' — an item file left with tags only is an inert entity, F-O10). It loads, runs 300 days with faults 0, and is TD-12's denominator. | DB-10 (b2): "with versus without bodies", on the towns (the measure kept by the re-scoped criterion). The same copy shows the instrument sees (I-10). |
| **SD-D14** | **Determinism evidence on the towns.** The existing `run_restart.rs` (social-cafe, SIGKILL) and `market_town.rs` (SIGKILL at four days) now exercise bodies; a new two-process test is not added. Rosetta: the 30-day social-cafe and market-town summaries on arm64 and x86_64, and an arm64 save resumed on x86_64 (recorded evidence, as PO-12). | The towns' existing restart tests already are the strongest committed check; they become bodies tests by content alone. |
| **SD-D15** | **Re-baseline once.** After TD-5 … TD-11 hold on the final executable head, the new 300-day seed-7 digests of both towns are recorded in E-TD (with their fact counts and the old ones), in `MVP_STATUS.md`'s evidence row, and in `overall.md` §7. No code holds a town digest (F-D10's audit: none does), so nothing else is edited for them. | Coordination ruling 5. S13 13b records `ac8.ref` after 12d (QS13-12); S11, S16 and S10 compare against main (§19.10). |
| **SD-D16** | **Documents first** (TD-C1): the `ARC-35` note (SD-D9, after the operator's QD-1), the `ARC-37` note (SD-D10), an `ARC-39` note 4 (the towns install bodies; the doorway refusal, SD-D5; note 3 is 12d-0's), `MODULE_SPEC.md` §4.1 (`item`'s `name` and catalogue; `body`'s doorway refusal), both towns' READMEs. | `CLAUDE.md` §2.2. |

## 19.4 Acceptance (decided before measuring, `ARC-23`)

Rules, as in §17.4 and §18.4:
- Each guarded criterion names the mutation shown to break it. A mutation is applied in the working
  tree, observed to fail by name, and reverted; `git status` and `git grep MUTATION` are recorded.
- Every expected value is a literal from the authored files or the slice's source, never computed by
  the code under test.
- **Order of evidence:** TD-5 … TD-8 (activity, then the scan) on a run before any determinism or cost
  claim is made on it; the re-baseline (TD-1's "after") only once TD-2 … TD-15 hold on the final
  executable head.
- **No criterion below is changed after a measurement.** A failure is either a bounded remedy this
  section already names, or a material stop with the numbers.

```text
TD-1  The re-baseline, once. Before (E-TD0 on 21f96ff; re-measured on 78ca5ae, unchanged, E-TD0c; captured
      again on the implementation base in E-TD-base): social-cafe 365 330 facts, sha-256 of every
      summary line but `wall` ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b;
      market-town 372 755 facts, 365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d;
      faults 0. After: the same two runs on the final executable head, faults 0, their counts and
      digests recorded in E-TD, MVP_STATUS and overall §7 — and nowhere in code (SD-D15).
TD-2  Load and refusals (tools/cli/tests/town_bodies.rs, through the real `mineworld validate`):
        - both towns validate; the summary lists `bodies`, the six shaped places, the four objects;
        - on test-time copies, each refused, exit non-zero, naming its subject and the numbers:
            the café's `here` back at (1 610, 200)  → "bodies-doorway", cafe, street, "(1610, 200)",
                                                      "200 mm";
            the street's café `there` at (0, 3 100)  → "bodies-doorway", street, "100 mm";
            a doorway point inside the counter       → "bodies-doorway", "solid";
            bob moved to (4 500, 6 400)              → "bodies-in-solid" (130 mm into the counter's
                                                      clearance);
            visitor moved onto the wanderer          → "bodies-overlap";
            the café ball moved onto bob             → "bodies-object-on-person".
      M-TD1  the doorway check removed → the first copy validates.
TD-3  The geometry matches the slice. A table in E-TD1, one row per authored solid and floor edge of
      the café, the street and the store: its slice source (file:line), the slice's shape, the authored
      box, and the largest distance between them, hand-computed; every row ≤ 150 mm except the door
      leaves (QD-3), each named. Reviewed in TD-C1; the executable form is 12e's `--world --geometry`.
      Every doorway point is ≥ 310 mm from its floor's edge and every solid (TD-2's check, on the real
      towns).
TD-4  AC-1 holds, with check 3 amended (only if QD-1 is answered "amend"):
        - `ac1_composability` passes all its tests, including the new check-3 cases of SD-D9;
        - `precursor_vocabulary` 4/4 unedited (F-O11);
        - the sections Market Town adds to Social Café's files are unchanged in kind (the test's
          printed list: holdings, economy, job).
      M-TD2  a Market Town-only item file with a `body:` → check 3 fails, naming the file and `body`.
      M-TD3  one of Social Café's object files deleted from Market Town → check 3 fails, naming it.
      M-TD4  `bodies` removed from Market Town's `systems` only → check 3 fails ("does not begin with
             Social Café's list").
TD-5  The towns still live (activity first). On the final head, unedited in their claims:
        - run.rs: every seat moves and talks in every 30-day bucket of the 300-day social-cafe run;
          every one of the six places is entered; the social precondition holds;
        - run_restart.rs, routines.rs (≥ 90 % of agenda segments reached per seat), milestone_b.rs,
          milestone_c.rs;
        - market_town.rs: every seat moves and talks, and the market lives (a purchase, a wage paid to
          each holder, production, consumption, a give from every seat), in every bucket; no wage
          unpaid, no drained wallet;
        - market_composition.rs: every copy without a market pack still lives.
TD-6  Bodies happen, and stay rare (town_bodies.rs, both towns, 300 days, seed 7):
        - per 30-day bucket: at least one `stopped-short` and at least one displaced `arrived`;
        - over the run: at least one accepted kick, one throw, one shove, one `object-moved { pushed }`;
        - per seat per 30-day bucket: kicks + throws + shoves accepted ≤ 288 (a tenth of its 2 880
          consults, AO-3's rate);
        - at day 300, every object has a free standing point within 800 mm (AO-2′ (b′)).
      The counts are printed per bucket.
TD-7  Nobody overlaps (the scan, SD-D12), after every request of both 300-day saves: no pair in one
      place closer than 595 mm; no centre outside its floor shrunk by 295 mm or within 295 mm of a
      solid; every displaced arrival stays in its place, ≤ 310 mm, ≤ 4 per request; every object
      within its floor, at rest, out of every solid, ≥ 295 mm from every person. The closest pair, the
      closest person–object approach, their place and request are printed.
TD-8  The instrument sees (I-10): the SD-D13 copy of social-cafe runs 30 days with faults 0, every seat
      moves, and no kick, throw, shove, `object-moved`, `person-shoved` or `stopped-short` is recorded;
      the scan, applied to it with the geometry of the real town, reports a violation, naming the pair
      or the person and the solid.
TD-9  Every existing test passes. The only edits are §19.6's list, each a literal or a route whose
      claim is stated unchanged there (QD-8). No assertion is weakened, no threshold lowered.
TD-10 Determinism, committed: run_restart.rs (two processes, SIGKILL early/middle/late) and
      market_town.rs (SIGKILL) pass on the towns with bodies; `mineworld replay` of a 30-day market-town
      save reproduces every fact from genesis.
      M-TD5  an impure resolver (12b's M-PB8, a process-global counter's parity in GAP) → a survivor is
             refused or diverges.
TD-11 Cross-architecture (recorded, as PO-12): an x86_64-apple-darwin `mineworld` under Rosetta gives
      the same 30-day summary sha-256 (every line but `wall`) as arm64 for both towns; an arm64
      market-town save at day 15 resumes on x86_64 to day 30 with the uninterrupted sha, and the
      reverse. The QP-16 fallback (PARTIAL, never PASS) applies if the CLI cannot be cross-built.
TD-12 Cost — 12d's own criterion, fixed by the refresh before any 12d measurement (`ARC-23`; QD-14 [OM]).
      History, kept as record: QB-11's ≤ 1.5 × (DB-10 (b2)) was re-scoped by the operator after 12d-0
      measured it unreachable (§20.5's amendments, E-Z3) to TZ-9a ≤ 3.0 × and TZ-9b ≤ 50 ms; on
      E-TD0b's prototype 12d-0 gave market-town 2.999 × and social-cafe 3.60 × — the latter accepted,
      not passed (E-Z7, Z-D9). 12d inherits those accepted numbers, with 10 % of headroom for what the
      real towns add to the prototype (the put_solid props' measured sizes, R-PK-2's catalogue):
  TD-12a  Relative CPU. §20.6.1's instrument, unchanged: user + sys CPU from `/usr/bin/time -l` of
          `mineworld run <town> --headless --seed 7 --days 300` (dev profile, the final executable
          head), each town against its SD-D13 copy; per town two runs per side, interleaved (without,
          with, without, with); wall time and load average recorded as information only. If a town's
          two pair ratios (with ÷ without) differ by more than 10 %, the measurement is contaminated:
          re-run that town once; if still contaminated, TD-12a is INCONCLUSIVE — reported with every
          number, neither PASS nor FAIL, and the primary session schedules a quiet-machine run.
          PASS iff, for each town, max(with) ÷ min(without) is at most 12d-0's accepted ratio × 1.10:
            social-cafe   ≤ 3.96 ×   (3.60 × 1.10)
            market-town   ≤ 3.30 ×   (2.999 × 1.10, rounded up to the hundredth)
          and faults 0 in every run. The absolute CPU with bodies is reported beside each ratio (12d-0:
          ≈ 45–49 s per 300 days; information).
  TD-12b  Per resolution. TZ-9b's method (§20.5): a scratch build of the final executable head, outside
          the repository and never committed, timing every call of bodies' `ArrivalResolver::resolve`;
          market-town, 300 days, seed 7; its fact count and fingerprint equal to the plain run's (the
          timing changes nothing). PASS iff the maximum ≤ 50 ms (CP-B4's tick bound, step-12-server.md
          §9); count, p50, p99 and p99.9 reported.
      A clear failure (TD-12a: both pairs of a town agree within 10 % and exceed the bound; TD-12b: the
      maximum above 50 ms) is a material stop with the numbers. Nothing in 12d changes bodies'
      resolution to pass it; §20.7's L1, as Class R, is the named next candidate — a design change for
      the operator. TD-12 is never re-scoped inside 12d.
TD-13 R-PK-2 (systems/item tests and town_bodies.rs):
        - market-town validates with twenty names; an `item:` without `name`, with an empty name, or
          with a 65-byte name is refused by the loader at its line;
        - an observer in the café of market-town is disclosed `item-catalogue` on the café: twenty
          entries `{ item, category, name }`, in ItemId order, as authored; an observer in social-cafe
          (no `item`) is disclosed none; market_composition's copy without `item` discloses none;
        - item VERSION 2, `item-kind-declared` and `item-kind` schema 2.
      M-TD6  the catalogue omits `name` → the café disclosure test fails, naming the first kind.
TD-14 Structural, unedited unless named: bodies' `isolation` (Rapier only in rapier.rs; no float
      outside it; system dependency presence alone; pack crates presence, item and — QD-5 — movement,
      each read for one named item); `rapier_pin` (bodies VERSION 4, "0.36.0" — QD-6); seam_vocabulary
      with FU-12a-1's three entries removed (they then fail as unused if a comment still names its
      word); ac1_composability per TD-4; `git diff <base> -- worlds/bodies-yard systems/bodies/src/{resolve,
      stride,entry,push,flight,launch,shove,rapier,reach,walls}.rs` empty (reach and walls do not exist;
      the names guard against their reintroduction); bodies' rules did not move — equal to 12d-0's
      re-captured bases (§20.14), captured again on 12d's base in E-TD-base:
        bodies-yard 30 days, seed 7, summary sha  bd6a10026f608dba1bb4d48f1399ccaa26e353c7c570b4190ef99039975c80e6
        long_run.rs second-process bytes          23f7fa76016294ab18ae5b6a6b568b61d1b36fc0741ee51eb7952276a1de5125
        long_run_objects.rs bytes                 c8358f8bbc06c94fbd7db33375dfe21ad0da80ddd39ee93ce9a72d542798c5b4
TD-15 Milestones and AC-15, on the final head: milestone_b.rs, milestone_c.rs, market_town.rs (300
      days), ac15_one_alice.rs, restart.rs, server_command.rs, ac13_semantic_parity.rs pass, with
      §19.6's literals only.
TD-16 Scope and the gate: `git diff --name-only <base>...HEAD` ⊆ §19.1's change set; the no-diff paths
      empty; Cargo.lock changes only mineworld-bodies' dependency list; `cargo fmt --check` and `cargo
      clippy --workspace --all-targets --all-features -- -D warnings` clean; both doc checks; the full
      workspace gate once, on the final executable head.
```

**Remedy ladder, fixed now (SD-D11), used only for TD-5 or TD-6, each rung a bounded deviation recorded
with its counts, the criteria never changed:**

```text
c1  content: move an object or a non-slice room's solids (apartments, workplace, park) out of where
    the counts show people jam; slice-matched solids never move
c2  content: QD-7's alternative doors (the apartments onto Maple & Co. at −8 610; the park onto the
    Lakeside Deli at 4 710) if routines.rs or a place-entered count fails for the walk's length
```

No rung touches bodies' rules or offer policy: that would move bodies-yard's results, which TD-14 holds
unchanged, and is a design change. If no rung passes — or TD-5's market activity fails for want of the
offered band's draws (F-D13) — the work stops and returns to the primary session with the counts.

## 19.5 Commit plan

Rules for every commit, as in §18.5:
- Each commit tracks implementation, validation and review separately; evidence into §19.12 as
  `E-TD<n>`, deviations into §19.13. A planned commit may become several coherent commits; the mapping
  is recorded. Each commit leaves the workspace's tests green.
- Commands from the worktree root, with `$HOME/.cargo/bin/cargo`. Anything longer than about two
  minutes runs in the background (300-day runs, the x86_64 build, the CLI test binaries, the gate).
- Every file change with the Edit and Write tools; no `sed -i`, `awk`, heredoc appends or inline
  `python3 -c`.

### TD-C0 — Design (this section) — docs only

- [x] Implementation: §19 and the header line, by the planning session on `mvp0/s15-12d-plan`, from the
  audit in §19.2.
- [x] Validation: both doc checks (E-TD0); the two 300-day baselines re-measured on the base (E-TD0);
  the planning prototype (E-TD0b), recorded as information, never as acceptance.
- [x] Review: every §19.2 claim cites a file and line or a measurement; each departure from §11.1,
  §10.4 and R-12d-1 … R-12d-4 is named with its finding and question; operator-material questions are
  marked. The planning session's self-review; the freeze is the primary session's.

### TD-C1 — Specs before code, the base captures, the geometry table

**Goal.** The decisions 12d relies on are reviewable before any code (`CLAUDE.md` §2.2), the references
12d must not move are captured before it can move them, and the slice's geometry is fixed as numbers.
**Scope.** `docs/DECISIONS.md`: dated notes on `ARC-35` (SD-D9; only as QD-1 is answered), `ARC-37`
(SD-D10), `ARC-39` note 4 (SD-D5, the towns; note 3 is 12d-0's). `docs/MODULE_SPEC.md` §4.1: `item`'s
`name` and the catalogue; `body`'s `bodies-doorway` refusal. The handoff, reinitialized. **E-TD-base**,
before any code: the base `mineworld` binary (copied to `/tmp/s15-12d/base-mineworld`); `validate` of
both towns with it; both 300-day summaries; bodies-yard's 30-day summary sha; the `long_run` bytes
(`BODIES_LONG_RUN_SECOND_PROCESS=1`) and the `long_run_objects` bytes
(`BODIES_LONG_RUN_OBJECTS_SECOND_PROCESS=1`) — each expected equal to §20.14's bases. **E-TD1**, the geometry table of TD-3: every slice collider of the
café, the street and The Flower Room, its source line, its authored box and its deviation; the
`put_solid` props sized from their glTF bounds × 0.92.
**Depends on:** freeze. **Non-goals:** no code, no world file.
**Failure and edge cases.** A `put_solid` prop whose bounds cannot be read from the asset: read from a
headless Godot print of the built collider's AABB in the slice, recorded; never guessed. A street with
more than 64 colliders: stop (SOLIDS_MAX is bodies' rule) — QD-3's answer decides which are merged.

- [ ] Implementation: the three notes, MODULE_SPEC §4.1, the handoff; E-TD-base; E-TD1.
- [ ] Validation: both doc checks; the decision-id count unchanged; every capture's sha-256 recorded.
- [ ] Review: the notes say what SD-D5, SD-D9 and SD-D10 decide and nothing more; no defined term
  redefined (`Item`, `World Pack`); E-TD1's every row ≤ 150 mm or named under QD-3.

### TD-C2 — R-PK-2: item kinds have names, disclosed as a catalogue

**Goal.** TD-13. **Scope.** `systems/item/src/`: `name.rs` (new, `ItemName`), `section.rs` (`name`,
required), `event.rs` and `component.rs` (the field; schema 2), `system.rs` (`VERSION` 2; `discloses`:
the `item-catalogue` record on a place), `lib.rs`'s table; `systems/item/tests/item.rs` (the refusals,
the catalogue). `worlds/market-town/items/*.yaml`: the twenty names. Existing tests that build
`ItemKindDeclared` or an `item:` section by hand gain a name, each listed (§19.6).
**Depends on:** TD-C1. **Non-goals:** names for objects or organizations; R-PK-1; any client.
**Failure and edge cases.** A disclosed record whose type no table holds: audit `presence::observe` and
the server's encoder first (QD-12); if a disclosed type must be an owned table, the catalogue becomes a
component on each place written at genesis — a bounded deviation, recorded. Two kinds with one name:
allowed (names are display, not identity). A name with surrounding whitespace: refused.

- [ ] Implementation: as scoped.
- [ ] Validation: `cargo test -p mineworld-item`; `-p mineworld-inventory -p mineworld-economy -p
  mineworld-consumption -p mineworld-employment -p mineworld-item-transfer` green; `validate
  worlds/market-town`; M-TD6 fails by name, reverted.
- [ ] Review: the catalogue is built at disclosure from `ItemKind`, never stored; `ItemId` order; item
  still depends on no pack; no market word added outside `systems/` and `worlds/` (check 2).

### TD-C3 — bodies refuses a doorway inside a wall

**Goal.** TD-2's doorway rows, M-TD1. **Scope.** `systems/bodies/src/genesis.rs` (`fits` gains the
doorway check, refusal `bodies-doorway`), `system.rs` (`VERSION` 4), `Cargo.toml` (`mineworld-movement`,
with its reason), `Cargo.lock`; `tests/genesis.rs` is left unedited — the new refusals go in a new
`tests/doorways_genesis.rs` (12c's DO-1 rule); `tests/rapier_pin.rs` (4, "0.36.0"); `tests/isolation.rs`
(the crate claim gains movement, read for `Passages` only, with its own "names nothing else" guard and a
mutation, as 12c's `is_declared` guard).
**Depends on:** TD-C1. **Non-goals:** any change to resolution; a doorway check against objects or
people (objects keep ≥ 700 mm by content, TD-2; people are entry-placed, SD-B8).
**Failure and edge cases.** A passage with no local point on one side (a semantic world): nothing to
check. A place without `body:`: nothing to check. bodies-yard: its four doorway points pass (12b
authored them free) — its 30-day sha unchanged (TD-14).

- [ ] Implementation: as scoped.
- [ ] Validation: `cargo test -p mineworld-bodies`; bodies-yard's 30-day sha, the `long_run` and
  `long_run_objects` bytes equal E-TD-base (= §20.14); M-TD1 fails by name, reverted; the movement-names-nothing-else mutation bites.
- [ ] Review: the check reads `Passages` only; integer arithmetic; the message names both places, the
  point and the distance; the doorway check runs after the people checks, in `fits`' order.

### TD-C4 — AC-1's check 3 admits Social Café's own items (only if QD-1 is "amend")

**Goal.** TD-4. **Scope.** `tests/acceptance/tests/ac1_composability.rs`: `compare_manifests` (Social
Café's `items` a subset of Market Town's), `world_delta_failures` (`items/` compared like `places/` for
Social Café's files; Market Town-only files as today), the unit test's new cases.
**Depends on:** TD-C1's `ARC-35` note. **Non-goals:** checks 1 and 2; `organizations/`.
**Failure and edge cases.** Social Café with no `items:` (every world before 12d): the check behaves
exactly as today — shown by running it on the base's two towns.

- [ ] Implementation: as scoped.
- [ ] Validation: `cargo test -p mineworld-acceptance --test ac1_composability` on this commit (the
  towns unchanged: passes as before) and again after TD-C5; M-TD2, M-TD3, M-TD4 fail by name.
- [ ] Review: the claim "Market Town is Social Café plus configuration" is stated in the test's doc and
  unchanged; no case is weakened; the operator's QD-1 answer is cited in the doc comment.

### TD-C5 — The towns get bodies

**Goal.** TD-2, TD-3 on the real towns; TD-9 for the literal edits. **Scope.** §19.1's world paths: both
`world.yaml`s (SD-D1, the four objects), every place file (SD-D2, SD-D3, SD-D4), the four object files
in both towns, both READMEs; the existing tests' literal edits of §19.6.
**Depends on:** TD-C2, TD-C3, TD-C4.
**Failure and edge cases.** A genesis refusal of a slice-matched solid (a person within 300 mm): the
person does not move (SD-D7); the solid is re-read (E-TD1) and, if the slice really places them
overlapping, that is a QD to the primary session, not an edit. A straight-walk test meeting a solid: its
route gains a waypoint around it, the claim unchanged (§19.6).

- [ ] Implementation: as scoped.
- [ ] Validation: `validate` both towns (exit 0, the six shapes, the four objects); `ac1_composability`
  13/13+; every edited test passes; the town places' files byte-identical across the two towns (`cmp`).
- [ ] Review: each edited test's claim is unchanged (§19.6 row by row); no person file changed; no
  slice-matched coordinate differs from E-TD1.

### TD-C6 — FU-12a-1: three comments reworded (QD-9)

**Goal.** Close FU-12a-1. **Scope.** `systems/movement/src/action.rs:14, 21` and `worldpack/src/read.rs:354`
(comments only); `tests/acceptance/tests/seam_vocabulary.rs`: the three `PRE_EXISTING` entries removed.
**Depends on:** none. **Non-goals:** any code line.

- [ ] Implementation: as scoped. The `MAX_STRIDE` comment no longer says movement "does not see walls"
  as a gap: walls are bodies' (ARC-39), and movement still decides only the stride and the passage.
- [ ] Validation: `cargo test -p mineworld-acceptance --test seam_vocabulary`; `-p mineworld-movement`;
  `git diff` shows comment lines only.
- [ ] Review: each reworded comment is still true; no new allow-list entry.

### TD-C7 — The towns, run for real

**Goal.** TD-5 … TD-8, TD-13's town half. **Scope.** `tools/cli/tests/bodies/mod.rs` (SD-D12: the place
reader); `tools/cli/tests/town_bodies.rs` (new): TD-2's refusals on copies, the 300-day run of each town
with TD-6's counts **checked before** TD-7's scan, TD-8's counterfactual, TD-13's catalogue disclosure.
**Depends on:** TD-C5. **Activity first:** TD-5 and TD-6 on the first runs; on a failure, §19.4's ladder
in order, each rung recorded with its counts; no rung passing → stop.

- [ ] Implementation: as scoped.
- [ ] Validation: `cargo test -p mineworld-cli --test town_bodies` (both towns), and run.rs, run_restart.rs,
  routines.rs, market_town.rs, market_composition.rs, milestone_b.rs, milestone_c.rs; bodies-yard's
  tests unchanged and green.
- [ ] Review: the scan reads the save and the place files only; the counts printed per bucket; the
  counterfactual's geometry is the real town's.

### TD-C8 — Close: cost, cross-architecture, the re-baseline, the gate, the ledger

- [ ] TD-12a: the eight 300-day runs, interleaved per town under §20.6.1's instrument (a contaminated
  town re-run once; INCONCLUSIVE reported, not stopped on). TD-12b: the scratch timing run of
  market-town (E-TD).
- [ ] TD-11: the x86_64 build and the Rosetta comparisons.
- [ ] TD-10, TD-14, TD-15, TD-16; the full gate once on the final executable head.
- [ ] TD-1: the after digests recorded (E-TD, MVP_STATUS, the handoff for overall §7).
- [ ] Documentation: `docs/MVP_STATUS.md`; both READMEs; §19's checkboxes, §19.12, §19.13; the handoff.
- [ ] Review: TD-1 … TD-16 each with evidence; deviations named; nothing of 12d-0's code touched.

**Refresh note (2026-10-08).** The paragraph below is the original planning record. QD-2 was answered
(the precursor, §20), 12d-0 merged, and its accepted numbers set TD-12 (E-TD0c): the commit plan is now
executable as it stands, its cost step measured against TD-12a/b rather than 1.5 ×.

**The cost bound is predicted to fail (E-TD0b), so QD-2 [OM] is answered before the freeze.** The
planning prototype — this design's geometry in a scratch copy of social-cafe, run by the merged binary —
took 191.4 and 155.2 s against 13.7 and 13.5 s without bodies: **14.2 ×** as TD-12 computes it
(max(with) ÷ min(without); 11.3 × at best) against QB-11's
1.5 ×. With the street's props removed (floors, walls and the retaining wall only) it took 46.5 s:
**3.4 ×**. The commit plan above is the plan *if* QD-2 is answered by a precursor that makes the bound
reachable; it is not executable as it stands, because TD-12 would stop it at TD-C8 after everything else
was built.

## 19.6 Test ownership, and the existing tests 12d edits

```text
STATIC      cargo fmt; clippy -D warnings; ItemName's type (an invalid name cannot be built); the body:
            section's type (12b, 12c), unchanged
UNIT        systems/item: ItemName, the section's refusals, the catalogue order; bodies: the doorway
            check against hand-computed points (doorways_genesis.rs)
INTEGRATION hand-built worlds through World::dispatch: item's catalogue disclosure (item.rs); bodies'
            doorway refusals (doorways_genesis.rs)
STRUCTURAL  ac1_composability with check 3 amended (TD-4); precursor_vocabulary, seam_vocabulary
            (FU-12a-1), bodies' isolation and rapier_pin (TD-14); no diff in the no-diff paths (TD-16)
REAL RUN    (Gate 2's role) the real binary on both towns: validate and its refusals (TD-2), the 300-day
            runs with activity, the scan and the counterfactual (TD-5 … TD-8) — committed in
            town_bodies.rs and the existing CLI tests; the cost (TD-12), Rosetta (TD-11), the
            re-baseline (TD-1) — recorded evidence on the final head
GATE 1      NOT REQUIRED: no model; the paced controller is a rule and is not edited
CI          S13 13a's CI if merged by then (its layer 2 runs the workspace tests); otherwise the full
            local gate once on the final executable head
```

Owned elsewhere and not repeated: 12a–12c own the seam, people, objects and the three actions on
bodies-yard; 12e owns the 150 mm probe (`--world --geometry`) that executes TD-3; S12 13c owns the
client's reading of the catalogue.

**Existing tests 12d edits (QD-8)** — each a literal or a route, each claim unchanged; F-D10's audit:

| File | What changes | The claim, unchanged |
| --- | --- | --- |
| `tools/cli/tests/commands.rs:34, 40` | the systems line gains `bodies`; "53 genesis fact(s)" → the new count | `validate` prints the world's systems and its genesis |
| `tools/cli/tests/inspect.rs:36–37`; `social_composition.rs:379, 419` | `inspect`'s systems line gains `, bodies v4` | `inspect` reports the save's composition |
| `tools/cli/tests/server_command.rs:34, 43–53, 85` | entities 18 → 22; the systems array gains `bodies` | the server reports the world it hosts |
| `worldpack/tests/social_cafe.rs:67–77, 109, 219` | the systems list, entities 22, genesis count | the pack loads as authored |
| `worldpack/tests/social_cafe.rs:338–390` ("17 facts" at 377) | the bare copy also copies `items/` and drops `bodies` with the `body:` sections; its "17 facts" and byte-equal prefix restated for the copy | removing naming and schedule changes nothing else |
| `worldpack/tests/social_cafe.rs:646, 769–770, 802, 831, 842` | the doorway (1 610, 200) → (1 610, 400); the street point (500, 3 000) → (500, 2 800); the doc comment at 769–770 naming both old points; the crossing's fact kinds as recorded | a person walks out of the café through its door |
| `tools/cli/tests/content_kinds.rs:39–40, 114–136` | the copy appends its two kinds to Social Café's `items:` instead of adding the key; ids and genesis counts restated | tags-only items and an organization change nothing a run decides |
| `tools/cli/tests/routines.rs:225` | "otto arrived once" → "every `arrived` of otto after genesis is caused by another person's request" | schedule moves nobody |
| `tools/cli/tests/{ac15_one_alice,restart}.rs` (door start (1 610, 600) at `ac15_one_alice.rs:76`, `restart.rs:41`), `worldpack/tests/social_cafe.rs:708–716` | a straight walk that meets a table gains one waypoint around it (only where TD-C5 shows it is needed) | the walker reaches the counter and talks to Alice |
| `tools/cli/tests/milestone_c.rs:172–215` (`walk_into_the_cafe`, Market Town; added by the refresh, §19.2a F-D10) | a straight stride that meets a street prop or a solid gains one waypoint around it (only where TD-C5 shows it is needed); the doorway points are read from the pack, so their move needs no edit | a seat walks to its door, along the street and into the café, every stride accepted |
| `tools/cli/tests/server_command.rs:96` | none expected: the perceived café entities `[2, 7, 8, 17, 18]` (objects are disclosed in `LooseObjects`, not perceived as entities); confirmed in TD-C5 | the server reports who is perceived in the café |
| `tools/cli/tests/market_composition.rs:40–47` | none expected: the object files carry no `item:`, so the carrier counts stay 20 / 14 / 14 / 2 | the six packs are removable |
| `clients/protocol/evidence/request-{2d,3d}.json` (`ac13_semantic_parity.rs`) | none expected: the frozen strides pass ≥ 900 mm from every café solid by SD-D2's numbers; if TD-C5 shows otherwise, it is a stop (frozen evidence is not re-recorded by 12d) | AC-13 on the protocol flavours |
| `systems/bodies/tests/{rapier_pin,isolation}.rs` | `rapier_pin.rs:9, 107–108`: (4, "0.36.0") — 12d-0 left it at 3; the crate claim gains movement (QD-5, QD-6) | the pack's version moves with Rapier's; bodies reads two packs' crates for one item each |
| `tests/acceptance/tests/{ac1_composability,seam_vocabulary}.rs` | TD-C4 (QD-1); TD-C6 (FU-12a-1) | AC-1; the seam names no physics |

## 19.7 Is any of this material?

**Yes — three things, each marked [OM] in §19.8:**

1. **Objects in the towns need `ARC-35` item 4 and AC-1's check 3 amended** (F-D1, QD-1). The operator
   approved `ARC-35` and accepted AC-1 on this test. No design puts a loose object in either town
   without it.
2. **QB-11's ≤ 1.5 × is predicted to fail by an order of magnitude** (E-TD0b, QD-2). DB-10 (b2) makes
   the bound unchangeable and names options 2 and 3 as the design change; the evidence says that change
   must come **before** 12d, as its own reviewed PR, not as a stop discovered at TD-C8.
   *Refresh: answered — 12d-0 ran, the operator re-scoped the bound and accepted 12d-0's numbers
   (§20.14). What remains material is 12d's own criterion, TD-12, derived from them (QD-14).*
3. **R-12d-2 moves three of the town's doors far from where the world put them** (QD-7): the apartments
   7.7 m west, the park 7.3 m east. That is content the operator may want to see, because it lengthens
   walks in a world whose people already walk slowly (L-12), and the prototype halved entries and talks
   (E-TD0b).

**Refresh (2026-10-08).** QD-1 and QD-7 were decided as recommended and QD-2 is closed by 12d-0. Two
operator-material questions remain open for the freeze: **QD-14** (12d's cost criterion, TD-12) and
**QD-16** (confirming that QS12-3 is answered by ruling 5).

Bounded, for the primary session: the bodies → movement crate read (QD-5, 12c's precedent), bodies
`VERSION` 4 (QD-6), the item names' authorship (QD-15), the queued-content check (QD-17), the recorded
client evidence (QD-18), R-PK-2 inside 12d (QD-4), the test edits (QD-8), FU-12a-1 inside 12d (QD-9), the
approximation rules (QD-3), the disclosed-only catalogue type (QD-12).

Not needed: any kernel, contract, persistence, server, cognition, presence or client change.

## 19.8 Questions (QD-1 …)

**[OM]** marks the operator's. The others are the primary session's at freeze. Each has a
recommendation.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QD-1 [OM]** | **Objects need check 3 amended** (F-D1). (a) Amend `ARC-35` item 4 and check 3 so Social Café may own items, and Market Town carries them unchanged (SD-D9); (b) no objects in the towns — R-12d-3 unmet, AC-14's object interaction only in bodies-yard; (c) objects in Market Town only — also fails check 3 (`body` is not a market section), so it needs a different amendment that makes the towns differ by more than configuration. | **(a).** It keeps the claim word for word — Market Town is Social Café plus configuration — and treats an object as what it is, Social Café's content, exactly as a place's `body:` already is. (c) weakens the claim; (b) fails the operator's requirement. |
| **QD-2 [OM]** | **QB-11 is predicted to fail by an order of magnitude** (E-TD0b: 14.2 × as TD-12 computes it, 11.3 × at best, against ≤ 1.5 ×; 3.4 × with the street's props removed). DB-10 (b2) forbids re-scoping and names options 2 and 3 as the design change. When? (a) **A precursor PR, "12d-0: bodies' cost"**, designed and frozen before 12d: DB-10 option 2 (a scene of only what a stride can reach — solids, people and objects whose boxes meet the stride's swept box, chosen by an integer test; broad phase without the narrow phase), option 3 (a stride stopped only by the floor's axis-aligned edge or one solid's face, with nobody and nothing else near, answered by integers — the stop and the slide along an axis-aligned face are exact in integers — with Rapier kept for every other case), and a fast path for nudge-free contacts; its own acceptance re-captures bodies-yard's and `long_run`'s bases, because results may move by a millimetre, and it runs before PL-b (S17), which freezes the default's results; (b) build 12d as designed and stop at TD-12 — the evidence says it will stop; (c) author fewer street solids, and have the slice drop the matching colliders (a client change in 12e/16f, against the accepted slice). | **(a), with (c) as a content option the operator may add.** The prototype says no content choice alone gets near 1.5 × (3.4 × with nearly every street prop gone), so the pack must get cheaper. Doing it first means 12d is frozen against a bound it can meet, and PL-b re-checks one set of results, not two. 12d's design is otherwise unchanged by (a). *Refresh: answered (a) and done — 12d-0 merged (#84); the bound re-scoped and 12d-0's numbers accepted by the operator (§20.14); 12d's criterion is QD-14.* |
| **QD-3** | **Shapes the slice has and bodies does not**: cylinders (inscribed square for r ≤ 500; crossed boxes above), the diagonal door leaves (their box, exempted by name in 12e's probe), `put_solid` props (glTF bounds × 0.92). | **As SD-D2.** Bodies' solids are axis-aligned boxes by design (SD-B3); a rotated or round solid is a bodies change nobody has asked for. The leaves are a probe exemption, not a content lie. |
| **QD-4** | **R-PK-2's code inside 12d**, or S12 13c lands the code first with `name` optional (no fact changes) and 12d adds the names? | **Inside 12d**, `name` required. One PR changes Market Town's genesis once; S12 is busy with 13a; a required name is what F-41 asks. Objects stay unnamed (they are not declared kinds); `R-S15-2` (tags in the listing) is the later route for them. |
| **QD-5** | **bodies reads movement's `Passages`** (a crate dependency, no system dependency) for SD-D5's refusal. Alternatives: a content test in `tools/cli/tests` that checks doorway points (not a load refusal, so §11.1's "refused at load by name" is unmet); a kernel-level read of passages (no). | **The crate read**, bounded as 12c's `is_declared`: named in `ARC-39` note 3, `isolation.rs` asserts bodies names nothing else of movement, with a mutation. Movement is a framework pack (`worldpack` names it), so AC-1's checks are untouched. |
| **QD-6** | **bodies `VERSION` 3 → 4** for the new refusal (refresh: 12d-0 took it from 2 to 3). | **Yes.** A save of a world the refusal now rejects must be refused by name, not re-executed into a genesis error. `rapier_pin`'s literal moves with it. |
| **QD-7 [OM]** | **Which slice doors the apartments and the park use** (R-12d-2). (a) The Flats' house door (−19 700, north) and the unit house door (15 300, south): right buildings, but 7.7 m and 7.3 m from today's points, so longer walks; (b) Maple & Co. (−8 610) and the Lakeside Deli (4 710): 3.4 m and 3.3 m off, but shops' doors used as a home's and a park's. | **(a)**, with (b) as §19.4's ladder rung c2 if routines or entries fail TD-5. The doors should be the buildings the scene says they are; the prototype's halved entries (E-TD0b) are mostly the props' doing (V1 restored most of them), which QD-2 addresses. |
| **QD-8** | **The existing tests edited** (§19.6), each a literal or a waypoint with its claim stated. | **Accept.** Each follows from content 12d is asked to change; none weakens an assertion. Frozen AC-13 evidence is never re-recorded by 12d. |
| **QD-9** | **FU-12a-1 inside 12d** (three comments in movement and worldpack, three allow-list entries removed), although 12d otherwise needs no edit there. | **Yes, as TD-C6**, comment-only. 12d makes the `MAX_STRIDE` comment's "does not see walls" stale in the towns, so this is its natural home; the scan's allow-list then shrinks. |
| **QD-10** | **FU-12c-1** (a stride away from somebody within the controller's offset is swept with them). | **Not in 12d.** It changes 12b's results and long-run base and needs its own design. If QD-2 (a) is taken, it belongs in 12d-0, which re-captures those bases anyway. *Refresh: closed — done by 12d-0's SD-Z5 (§20.14).* |
| **QD-11** | **S12 13a's exact match on (0, 3 000)** (F-D12) breaks when the café's street point moves to (0, 2 800). | **13a reads the café's `there` from the disclosed passage that leads to the café**, rather than an equality on a literal — a 13a change, told to S12 now (§19.10). 12d does not keep a point inside the façade's clearance to suit a client test. |
| **QD-12** | **The catalogue as a disclosed record of a type no table holds.** | **Audit first** (TD-C2): if `presence::observe` or the server's encoder requires a disclosed type to be an installed component, the catalogue becomes a component on each place written at genesis — bounded, recorded. |
| **QD-13** | **Activity criteria for the physical actions**: per run, not per bucket (AO-2′ (a)'s lesson); stopped-short and displaced arrivals per bucket; ≤ 288 per seat per bucket; objects reachable at day 300. | **Accept, fixed now.** |

**Questions added by the refresh (2026-10-08).** QD-1 … QD-13 stand as ruled; QD-2 and QD-10 are closed
by 12d-0, QD-6 now reads 3 → 4.

**Rulings at the freeze (primary session, 2026-10-08):** **QD-14** — TD-12 accepted as written (TD-12a
≤ 3.96 × / ≤ 3.30 × with the contamination rule; TD-12b ≤ 50 ms). **QD-15** — 12d writes the twenty
names; S12 reviews them in 12d's PR. **QD-16** — confirmed: QS12-3 is answered by overall ruling 5, and
R-PK-2 is carried in 12d's one re-baseline. **QD-17** — nothing else is queued. **QD-18** — as
recommended.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QD-14 [OM]** | **12d's cost criterion (TD-12)**, replacing QB-11's 1.5 ×, which the operator re-scoped for 12d-0 (§20.5). Proposed by the primary session and fixed here before measuring: the with-bodies towns stay within 12d-0's accepted TZ-9a numbers plus 10 % — social-cafe ≤ 3.96 × (3.60 × 1.10), market-town ≤ 3.30 × (2.999 × 1.10) — under §20.6.1's instrument (interleaved CPU, 10 % contamination rule, INCONCLUSIVE rather than a stop when the machine stays loaded); and TD-12b, per-resolution max ≤ 50 ms on market-town (CP-B4). Alternatives: (b) exactly 12d-0's numbers, no headroom (3.60 ×, 2.999 ×) — the real towns' measured put_solid sizes and the catalogue may cost a few per cent the prototype did not, so (b) risks a stop on noise; (c) an absolute CPU budget (e.g. ≤ 55 s per 300 days) — load-sensitive, which §20.6.1 exists to avoid. | **(a), as proposed.** It inherits only what the operator already accepted, adds headroom smaller than the instrument's own 10 % contamination band, and keeps the per-request bound that matters to a live server. A clear failure stops with the numbers; L1 (Class R) is the named next step, not 12d's. |
| **QD-15** | **Who writes the twenty Market Town item names** (R-PK-2's content; §19.10a). | **12d writes them**, each from its kind's key and category in plain English (e.g. `bread` → "Bread"), recorded in TD-C2; S12 reviews them in 12d's PR. Names are display, not identity, so a later wording change is a content change for a later re-baseline, not a blocker. |
| **QD-16 [OM]** | **QS12-3 (operator-material in step-13) is taken as answered by overall.md ruling 5** — R-PK-2 is carried inside 12d's one re-baseline. Confirm at freeze. | **Confirm.** Ruling 5 says so in words; step-13 13c still reads "waits on QS12-3", which the post-merge sync of 12d updates. |
| **QD-17** | **Any other content change queued for the one re-baseline** (overall item 5: "and any other content change queued for it"). | **None known at 78ca5ae.** Anything not named at freeze waits for a later, separately approved re-baseline; 12d's scope is not widened after the freeze. |
| **QD-18** | **Recorded client evidence holding Market Town ids** (`clients/protocol/evidence/affordances-market-town.log:12–14`: ids 19, 34), which F-D9's id shift makes historical. | **Not re-recorded by 12d** (no check reads them, §19.2a); the S12/S14 lane that next records evidence on a 12d world records it anew. Stated in §19.10. |

## 19.9 Proposed execution contract for PR 12d

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12d — the town gets bodies (S15, fourth of five)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §19; evidence §19.12 (E-TD<n>);
                    deviations §19.13
RELATED / BINDING   §20 (12d-0 as merged: §20.5's amendments, §20.6.1's instrument, §20.14's bases and
                    cost record); this file's header (QB-5, QB-11 as re-scoped, DB-10 (b2)), §§5, 10.4,
                    11.1, §17 (12b), §18 (12c as
                    merged, §18.12), §19; overall.md "Parallel build-out" ruling 5, "Framework, not demo"
                    item 3, "One world, two views", "Physics list" (PL-b re-checks 12d); step-15 §11.3
                    (R-12d-1 … R-12d-4), §4.2 (the 150 mm probe); step-13 R-PK-2, QS12-3; DECISIONS ARC-23,
                    ARC-25, ARC-34, ARC-35, ARC-36, ARC-37, ARC-39 and its notes, DEP-13; MODULE_SPEC §4.1;
                    CLAUDE.md §§2–4
PRECONDITION        QD-1 and QD-2 answered (done); 12d-0 merged (78ca5ae; done, base and TD-14 re-read in
                    the refresh); QD-14 … QD-18 ruled at the freeze (2026-10-08, §19.8) — all met
IMPLEMENTATION BASE main @ 78ca5ae or later, with this refresh's PR (#85) merged; re-audit §19.2a if a
                    listed path moved; branch mvp0/pr-12d-towns, from main (the freeze's name; it
                    replaces §19.1's proposed mvp0/pr-12d-town); worktree
                    /Users/yuema137/mineworld-worktrees/impl-12d, held by the implementing session only
APPROVED SCOPE      §19.1's change set (with §19.6's refreshed rows); TD-C1 … TD-C8; SD-D1 … SD-D16 as
                    answered by QD-1 … QD-18
FROZEN INVARIANTS   No edit under kernel/, contracts/, persistence/src/, server/, cognition/, clients/,
                    authoring/, sdk/, systems/presence/, tools/cli/src/, worlds/bodies-yard/, the root
                    Cargo.toml; movement and worldpack sources only FU-12a-1's comment lines; no System Pack
                    other than bodies (SD-D5 only) and item (SD-D10 only).
                    bodies' rules unchanged: bodies-yard's 30-day sha and the long_run and
                    long_run_objects bytes equal to 12d-0's bases (§20.14; TD-14).
                    TD-12 as fixed: social-cafe ≤ 3.96 ×, market-town ≤ 3.30 × (§20.6.1's instrument),
                    per-resolution max ≤ 50 ms; never re-scoped inside 12d.
                    Activity criteria fixed (TD-5, TD-6); the controller unchanged; remedies only §19.4's
                    content ladder.
                    No person file changes; both towns' place files and object files byte-identical to each
                    other (check 3).
                    The digests are re-baselined once, after TD-2 … TD-15 hold, and recorded, never coded.
                    Existing tests unchanged except §19.6's rows.
SEQUENCE            TD-C1 → TD-C2 → TD-C3 → TD-C4 → TD-C5 → TD-C6 → TD-C7 → TD-C8, each committed and pushed
                    when coherent; E-TD-base and E-TD1 in TD-C1 before any code
VALIDATION BUDGET   (the freeze's ruling) six 300-day town runs (~13–17 s of CPU without bodies, ≈ 45–50 s
                    with, by 12d-0's E-Z7); unit/integration/static unrestricted; 30-day runs inside
                    committed tests; the ladder at most three content re-runs; the x86_64 build once and
                    its Rosetta runs; one full gate on the final head; real-model NOT REQUIRED.
                    OPEN, flagged at the freeze for the primary session (§19.13, "Freeze note"): TD-12a
                    as accepted (QD-14) needs eight interleaved 300-day runs (four more on one
                    contaminated town's re-run), TD-1 two and TD-12b one — more than six. Until the
                    primary session says how the six apply, the implementing session stops before TD-C8's
                    runs rather than choose.
TOOL DISCIPLINE     as 12d-0's contract: Read, Edit and Write for files; allowed `cargo` (as
                    `$HOME/.cargo/bin/cargo`), `git`, `gh`, `python3 scripts/*`, `mkdir -p`, `sed -n`,
                    `/usr/bin/time`, `arch -x86_64`, `rustup target list --installed`; never `python3 -c`,
                    `sed -i`, `awk`, `xargs`, `curl`, or `cat >>` / heredoc writes; long jobs in the
                    background; no edit of `.claude/settings*.json` or other worktrees
LIVE DOCUMENTATION  §19 checkboxes; §19.12; §19.13
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for 12d at TD-C1
ENDPOINT AUTHORITY
  implementation + local validation   authorized by the freeze (2026-10-08), for a fresh session
  semantic commits, branch push       authorized, as for 12a–12c
  PR creation / update                authorized, as for 12a–12c
  scratch builds                      authorized: the base binary, the x86_64 build and TD-12b's
                                      timing copy, in /tmp/s15-12d; no branch
  CI repair                           authorized: S13 13a's CI is on main (#63); main's
                                      branch protection requires `fast` and `test` green on the PR's
                                      exact head, reported with the local evidence
  merge                               operator only, with a merge commit; never inherited, never widened
POST-MERGE SYNC     the planning session owns the step header, §§1–15, overall and MVP_STATUS's Updated and
                    S15 lines; the implementing session owns §19 and the evidence rows
NORMAL STOP         PR 12d READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       as §19 lists: an edit outside §19.1's change set; bodies-yard's sha or the long-run
                    bytes moving; TD-5 or TD-6 failing on every rung; a slice-matched solid refused at
                    genesis by an authored person; the frozen AC-13 evidence needing re-recording;
                    cross-architecture digests differing (TD-11 FAIL, not PARTIAL); an answer to QD-1 or
                    QD-7 other than the design's — plus (the freeze's ruling) **any TD-12 failure**
                    (TD-12a INCONCLUSIVE is reported with its numbers, not counted a failure), and **any
                    digest change other than the one planned re-baseline** (TD-1); and the budget
                    question above, before TD-C8's runs
```

## 19.10 Cross-lane impacts

| Lane | What 12d changes for it | What it must do |
| --- | --- | --- |
| **S12 13a** (in implementation, `impl-s12-13a`; refresh: its client is not on main, only its CI #63 — nothing on main matches (0, 3 000)) | The café's street point (0, 3 000) → (0, 2 800); every place's doorway points; walls and solids on the straight strides its tests walk (`client_2d.rs:165–181`: the wanderer toward Alice ends behind the counter's face, and the counter is now solid); `godot.yaml:58`'s `door_from_left_m 1.61` still right (x unchanged) | Before 12d merges, or in a rebase after: find the café's door by the disclosed passage leading to the café, not by equality on (0, 3 000) (QD-11); route its scripted walks to points that are free in the authored geometry (the counter's customer side, y ≤ 6 270). 13d (bodies in 2D) waits on 12d as planned. |
| **S12 13c** (R-PK-2's client half) | `item-catalogue` exists on every perceived place of market-town (SD-D10); Market Town's kind ids shift (F-D9), so `clients/protocol/evidence/affordances-market-town.log`'s ids 19 and 34 become historical (QD-18) | Read names from it; R-PK-1 stays 13c's; evidence recorded on a 12d world is recorded anew. |
| **S14 16a** (refresh: merged as #79; `slice_probe_world.gd`'s `link`, `conversation`, `target` modes hard-code scene points valid while the 0.2 m binding holds; `_street_watch` prints doorway distances and never fails) | A16-7's door (1 610, 200) ↔ (0, 3 000) moves; T-2 relies on the florist being reported as the street, and with bodies the street's floor ends at the façade, so a body inside the florist is outside it | 16a is not affected while it connects to a pre-12d world; if it rebases onto 12d, T-2's florist walk and the door literal need 12e's binding (the next row). |
| **S15 12e / S14 16c** | The doorway points sit 400 mm inside instead of 200; the store's floor is The Flower Room; objects exist in the café and on the street | `slice_link.gd:80–86`'s 0.2 m becomes 0.4 m on both sides; `PLACE_KEY[FLORIST_PLACE] = "store"`; the probe `--world --geometry` exempts the two door leaves by name (QD-3); the four objects are drawn from `loose-objects`. Stated in step-15 R-12d-1 / F-S14-9 already; 12d adds the 0.4 m. |
| **S11** (S11-A in implementation) | Both 300-day digests | I-6 compares against the PR's own base (`step-12-server.md` I-6): E-SA0 is re-recorded if 12d merges first. No code holds a digest. |
| **S11-A's E-SA0, S16's ledger** | The digests recorded in their ledgers become historical | No edit; their invariants already compare against `main`. |
| **S13 13b** (AC-8) | `ac8.ref`'s town references | Recorded **after** 12d merges (QS13-12); 13a's `ci_image.py` (`run worlds/social-cafe --days 1`, faults 0) still holds. |
| **S17 PL-a / PL-b** | PL-b's byte-identity reference becomes 12d's digests (and, with QD-2 (a), 12d-0's bodies-yard and long_run bases) | PL-a may run beside 12d (it touches `installed!`/`worldpack`; 12d touches only `read.rs`'s comment there — a trivial conflict at worst). PL-b after 12d. |
| **S16 E-a** | merged (#70); its `version`/`license` lines in both `world.yaml`s are kept | none |
| **Test hygiene** (`mvp0/pr-test-hygiene`, merged as #77) | `town_bodies.rs` and the copies 12d adds | Merged first (refresh): 12d's new tests and copies use `tools/cli/tests/support`'s scratch helpers, so every save and pack copy is removed when its test ends (`5f2cee9`). |
| **S10** | The digests | I-7 compares against `main` at merge time. |

## 19.10a Inputs owed to 12d by other lanes (the refresh)

| Input | Owed by | Where it stands on `main @ 78ca5ae` | What 12d needs, and when |
| --- | --- | --- | --- |
| **R-PK-2 — item kinds have names** | S12 (`step-13-client-2d.md` R-PK-2, F-41, RK-5) | Specified there: `item:` gains `name`, `item` discloses a `{item, category, name}` catalogue on every perceived place; 13c holds it back, "waits on QS12-3". Not implemented. 12d carries the code (QD-4, SD-D10). | **The twenty display names** of Market Town's kinds (the content, not the code): who writes the text? Proposed: 12d writes them from each kind's key and category, S12 reviews them in the PR (QD-15). Needed at TD-C2. |
| **QS12-3 — R-PK-2 inside 12d's one re-baseline** | S12, operator-material (step-13) | Recommended "carry R-PK-2 in 12d's re-baseline"; overall.md "Parallel build-out" ruling 5 says the re-baseline "also includes S12's item names (R-PK-2, QS12-3)". | Recorded as answered by ruling 5; 12d's freeze confirms (QD-16). If it were reversed, SD-D10 and TD-13 leave 12d and 13c re-baselines again. |
| **QD-11 — doorways read from passages, not literals** | S12 13a (and any client on a moving door) | 13a's client is not on main; nothing on main matches (0, 3 000) (§19.2a F-D12). S14's slice binds by passage, at 0.2 m (F-D5). | Nothing blocks 12d. Owed **after** 12d by 13a (find the café's `there` from the passage) and by 12e/16c (0.2 m → 0.4 m, florist → `store`), as §19.10. |
| **The one-time digest re-baseline** | overall.md "Parallel build-out" item 5 | "Both towns' replay digests are re-baselined once, in S15's 12d. That re-baseline also includes S12's item names (R-PK-2, QS12-3) and any other content change queued for it." Before: E-TD0c. | **Any other content change queued for it** must be named before the freeze, or it waits for a later re-baseline (QD-17). Known now: none besides R-PK-2. |
| **12d-0's bases** | S15 12d-0 | Merged; §20.14. | Read by TD-14 and E-TD-base. Done. |
| **S13 13a's CI** | S13 | Merged (#63); branch protection requires `fast` and `test`. | The PR's exact head green on both, reported with the local evidence (§19.9). |

## 19.11 Demo coverage ("Framework, not demo", item 3)

With 12d, the default demo world — **market-town**, which has everything social-cafe has (QS14-5) —
exercises in a headless 300-day run, by the unchanged paced controller:

```text
exercised   move (walls, solids, doorways, nudging); talk; invite, accept-invitation,
            decline-invitation, join-group-activity, leave-group-activity (group activities);
            relationships forming (passive); names (naming); agendas and their following (schedule);
            give; buy; eat; drink; work and wages (employment, passive); kick; throw (the complete,
            unaimed form); shove; pushing objects by walking; nudging people by walking
not yet     throw with an aimed point (offered incomplete; a client sends it, the controller cannot)
            sleep and needs (MVP §5; QS-10 open)
            persona (S10)
            fishing — the third-party pack of Milestone E (S16)
            the item catalogue's use by a client (13c), and names for loose objects (QD-4)
```

The coverage test the item asks for ("each interaction occurs in a scripted or headless run") is not
12d's; TD-6 and the existing market tests are its headless evidence for the interactions above.

## 19.12 Evidence ledger

```text
E-TD0 TD-C0, 2026-10-08, planning session, on mvp0/s15-12d-plan @ 0f29ce9 (= main 21f96ff + the 12c
      post-merge docs commit). `cargo build -p mineworld-cli` (dev, opt-level 1) 53.3 s.
      `cargo run -p mineworld-cli -- run worlds/<town> --headless --seed 7 --days 300`:
        social-cafe  faults 0, 365 330 facts, sha-256 (every line but `wall`)
                     ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b; wall 33.7 s (the
                     machine was loaded by parallel builds; 13.7 and 13.5 s in E-TD0b's quiet runs)
        market-town  faults 0, 372 755 facts,
                     365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d; wall 16.4 s
      Both equal E-RS0 / PO-1: the "before" of TD-1.
      Doc checks: 176 numbered sections, none duplicated; 51 decision ids, all distinct.

E-TD0b Planning prototype — information for QD-2, not acceptance (ARC-23: no 12d criterion was set or
      changed by it; QB-11's bound predates it). A scratch copy of social-cafe at
      /tmp/s15-12d/proto/worlds/social-cafe, exported by `git --work-tree … checkout HEAD`, with §19.3.1's
      geometry (the street's ≈ 60 props with placeholder sizes where the slice uses put_solid), the moved
      doorways and the four objects; `validate` → valid, 22 entities, 67 genesis facts. Runs by the
      merged binary, dev profile, 300 days, seed 7, consecutive, one machine:
        without bodies (worlds/social-cafe)   13.7 s, 13.5 s; faults 0; 365 330 facts
        with bodies, full geometry            191.4 s, 155.2 s; faults 0; 525 890 facts
            moves accepted 243 397, of which stopped-short 172 052 (71 %); shoves 4 738; kicks 2;
            throws 2; object-moved 29; talks 32 916 (67 752 without); person-entered-place 10 015
            (20 720 without); every seat still moved and talked in every 30-day bucket (talks per seat
            per bucket ≥ 99)
            → 14.2 × as TD-12 computes it (191.4 ÷ 13.5); 11.3 × at best; QB-11 is 1.5 ×
        variant V1: the street's floor and the retaining wall only (no lamp, bollard, tree, planter,
            table, pot, board, bench, post or bicycle)
                                              46.5 s; faults 0; 467 215 facts; moves 194 013,
            stopped-short 58 371; shoves 11 974   → 3.4 ×
      Reading: the street's props multiply stopped strides threefold and the cost fourfold; even without
      them, walls alone put the towns at 3.4 ×. The paced controller wanders ± 1 400 mm with no notion
      of a wall (F-D14), so most strides near an edge are swept. Logs: /tmp/s15-12d/p-*.txt.

E-TD0c The refresh, 2026-10-08, planning session, on plan/s15-12d-refresh @ 78ca5ae (main, 12d-0 merged).
      `CARGO_TARGET_DIR=/tmp/s15-12d0/target-plan cargo build -p mineworld-cli` (dev); `run worlds/<town>
      --headless --seed 7 --days 300` (/tmp/s15-12d0/plan-before.sh):
        social-cafe  faults 0, 365 330 facts, ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b
        market-town  faults 0, 372 755 facts, 365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d
      Both = E-TD0: nothing merged since 21f96ff (12d-0, CI #63, E-b #78, 16a #79, test hygiene #77, S19
      docs) moved the towns. TD-1's "before" stands.
      The cost evidence 12d inherits is 12d-0's, on E-TD0b's prototype (§20.12 E-Z7, §20.14): with bodies
      ≈ 45–49 s of CPU per 300 days (was 105–114 s, E-Z-before; 155–191 s wall in E-TD0b), market-town
      2.999 ×, social-cafe 3.60 × (accepted, not passed), per-resolution max 10.5 ms. TD-12 is set from
      those numbers before any 12d measurement.

E-TD-base, E-TD1 … : the implementing session's.
```

## 19.13 Deviations and discoveries during implementation

None yet.

**Freeze note (planning session, 2026-10-08) — the budget field, for the primary session.** The
freeze sets the budget at six 300-day town runs. TD-12a as accepted at the same freeze (QD-14) needs
eight interleaved 300-day runs (two per side per town), four more if one town is contaminated and
re-run, plus TD-1's two "after" runs and TD-12b's one timing run: eleven to fifteen. The two cannot both
hold. The contract records the ruling as given and asks the primary session which reading applies
(six beyond TD-12's own runs, or TD-12a reduced); the implementing session stops before TD-C8's runs
until it is answered, rather than choose.

---

# 20. PR 12d-0 — bodies' cost (full design; DESIGN FROZEN 2026-10-08; MERGED as #84, `78ca5ae`)

**Lifecycle:** inserted by the primary session's QD-2 ruling (2026-10-08, §19's header) and drafted by
the planning session on `mvp0/s15-12d0-plan`, from `main @ f842c52` (#72 and S11-A #76 merged; S11-A
touches nothing of bodies). **DESIGN FROZEN (2026-10-08), primary session** — the freeze message relayed
by the coordinator: "§20 is accepted as written", QZ-1 … QZ-5 accepted as recommended (QZ-1: SD-Z4 keeps
the whole wall slide as a deliberate definition change, governed by ZR-3's shadow rule — at most 1 % of
requests differing by more than 50 mm; QZ-2: the 2 200 mm margin guarded by its unit test; QZ-3: the gate
on both towns; QZ-4: the L1/L2 ladder fixed, anything beyond it the operator's; QZ-5: the prototype
rebuilt from §20.12's recipe). The byte-identity classes (§20.4) and the TZ-9 gate stand exactly as
written. Implementation in a fresh session (§20.11). §19 stays AWAITING 12d-0 until 12d-0 passes TZ-9.

The ruling, as relayed: implement DB-10's options 2 and 3 — integer wall-only strides that never touch
Rapier, and only the people who can be reached in the scene; decide **before measuring** which results
must stay byte-identical (bodies-yard, `long_run`) and which may legitimately change, and if option 3
changes results, re-capture the bases under a stated rule; measure E-TD0b's town prototype against
QB-11's 1.5 × **before 12d is frozen**; if 12d-0 cannot reach ≤ 1.5 × on the prototype, stop and report —
that goes back to the operator. QB-11 is not re-scoped.

## 20.1 Identity, base, approved scope

```text
PR            12d-0 — bodies' cost (S15, a precursor between 12c and 12d)
base          main after this design's PR merges (f842c52 + Markdown only). Re-audit §20.2 if anything
              under systems/bodies, systems/presence, kernel/src or tools/cli/tests/bodies* moved
branch        mvp0/pr-12d0-cost, in its own worktree, held by the implementing session only
audit         §20.2 (main @ f842c52, 2026-10-08), E-TD0b
scope         DB-10 options 2 and 3 as SD-Z1 … SD-Z5 define them; FU-12c-1 (QD-10); the cost gate on
              E-TD0b's prototype (§20.12)
depends on    12c merged (889d217); the 12c post-merge docs (#72)
merge         a merge commit, never a squash
```

**Goal.** A stride costs what it needs: the scene holds only what the stride can reach, Rapier is not
consulted where integers answer exactly, and the towns of §19 run within QB-11's 1.5 × — measured on the
prototype before §19 is frozen.

**Change set.**

```text
systems/bodies/src/{rapier,stride,entry,resolve,geometry,system}.rs   SD-Z1 … SD-Z5; VERSION 3
systems/bodies/src/reach.rs            new: the reachable set (SD-Z1), integer only
systems/bodies/src/walls.rs            new: integer wall strides (SD-Z4)
systems/bodies/tests/{cull,walls}.rs   new: SD-Z1's identity, SD-Z4's literals
systems/bodies/tests/{rapier_pin,actions}.rs   (3, "0.36.0"); FU-12c-1 flips the DO-11 pin (§20.6)
systems/bodies/README.md, lib.rs's table
docs/DECISIONS.md                      ARC-39 note 4 (SD-Z4, SD-Z5: results changed by design), DEP-13
                                       note (what of Rapier is now consulted)
docs/MVP_STATUS.md                     the S15 rows
.structured-coding/plans/mvp0/{step-11-bodies,handoff}.md
```

No diff: everything outside `systems/bodies/` and the documents above — in particular `worlds/`
(bodies-yard included), `tools/cli/`, presence, movement, the controller, the kernel, contracts, the
root `Cargo.toml`, `Cargo.lock`. The prototype is never committed: it lives in `/tmp`, regenerated
from §20.12's recipe.

**Non-goals.** Any change to a bodies *rule* other than SD-Z4 and SD-Z5 (constants, the nudge, the
bias, the push, the flight, entry placement's lattice); the towns (12d); per-place caching of a scene
(I-5 forbids it); the release profile for `mineworld run`; `parallel` or `simd8` (DC-1).

## 20.2 Source audit (`main @ f842c52`, 2026-10-08)

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| **F-Z1** | **Every swept stride builds the whole place.** `stride` builds one `Scene` with the floor slab, four perimeter walls, **every** solid, **every** person standing in the place and **every** object, then runs a walls-only sweep, a contact sweep, the nudge pass's sweeps, and on a jam a second attempt on the same scene. `entry`'s E2 builds the same. | `stride.rs:111–166`; `entry.rs:47–60`; `rapier.rs:120–155, 420–490` | On the prototype's street (≈ 60 solids, up to 12 people) every swept stride inserts ≈ 70 colliders to answer queries that touch a handful. SD-Z1. |
| **F-Z2** | **A scene runs the narrow phase it never reads.** `refresh` calls `PhysicsWorld::detect_collisions`, which steps the broad phase and the narrow phase. Scene queries read only the broad phase: `query_pipeline_with_filter` is `broad_phase.as_query_pipeline(narrow_phase.query_dispatcher(), …)`. The re-mark of dynamic bodies after it (F-P1) matters only where something is dynamic: a flight. A people scene has no dynamic body. | `rapier.rs:495–515`; `rapier3d-0.36.0/src/pipeline/physics_world.rs:175–185, 569–581` | DB-10 option 2's "broad phase without the narrow phase", for `Scene` only (SD-Z2). `Pile` and `fly` keep `refresh` unchanged. Whether 0.36.0 exposes a broad-phase-only update is audited in ZC-2; if it does not, SD-Z2 is dropped, recorded, and the rest stands. |
| **F-Z3** | **The fast path is coarse against solids.** `corridor_clear` refuses any stride whose segment's bounding box, grown by R + GAP, meets a solid's box — a diagonal stride is swept when a solid lies anywhere in its box, however far from the segment itself. | `geometry.rs:239–261` | On a street of lamps, bollards and trees, many clear strides go to Rapier. An exact integer segment-to-box distance (SD-Z3). |
| **F-Z4** | **A stride stopped only by the floor's edge is answered by Rapier's slide.** The floor is an axis-aligned rectangle; the centre's free region against it alone is the floor shrunk by R + GAP (the controller's offset), a rectangle. 12b measured its stop to ± 1 mm (PB-5 a: 8 010 = 8 320 − 300 − 10). Its slide was asserted only as "it slid" (PB-5 c: y > 4 010): how much tangential motion Rapier keeps is not pinned. | `stride.rs:239–267`; §17.4 PB-5 a, c | An integer clamp reproduces the stop exactly and the slide only approximately: **SD-Z4 changes results** wherever a walker slides along a wall. Hence the re-capture rule (§20.4 ZR). |
| **F-Z5** | **The prototype's strides end mostly at walls.** E-TD0b: 71 % of 243 397 accepted moves ended stopped short with the street's props; 30 % (58 371 of 194 013) with walls only; and the paced controller wanders ± 1 400 mm with no notion of a wall (F-D14). | E-TD0b | SD-Z4 answers the walls-only share without Rapier; SD-Z1 and SD-Z3 make the props' share cheap. Neither changes what the controller asks. |
| **F-Z6** | **What holds bodies' results today.** `long_run.rs` and `long_run_objects.rs` compare a second process's bytes and check N-1 … N-4 after every request; `bodies_yard.rs` and `bodies_yard_restart.rs` check activity, the scan, two processes and SIGKILL. **No test holds a captured digest**: the references (E-PO-base's `long_run` sha, the bodies-yard 30-day summary sha `6e4c4015…c8395`) are ledger entries. | `systems/bodies/tests/long_run*.rs`; `tools/cli/tests/bodies_yard*.rs`; §18.10 | Re-capturing a base edits the ledger only. Which bases may move is decided now (§20.4). |
| **F-Z7** | **FU-12c-1 is one predicate in the contact sweep.** A walker standing 595 … 610 mm from somebody (a blocked walker ends 608.7 mm away, DB-6) is swept with them solid, so a stride directly away advances 1 mm and is cut to `NUDGE_MAX` past it (DO-11, `a_shove_from_600_mm_is_cut_short_by_the_shover`). | `stride.rs:239–267`; `rapier.rs:161–205`; §18.11 DO-11 | SD-Z5. Changes results (class R). |
| **F-Z8** | **12d's prototype exists only in `/tmp`.** It is `worlds/social-cafe` at `21f96ff` with six place files, four item files and `world.yaml` changed (E-TD0b). | E-TD0b | §20.12 states it in full, so the gate is reproducible from the repository. |

## 20.3 Design (SD-Z1 … SD-Z5)

| ID | Decision | Class | Rationale |
| --- | --- | --- | --- |
| **SD-Z1** | **Only what a stride can reach is in its scene** (DB-10 option 2). `reach.rs`, integer only: a stride from `start` asking for `d` has a **reach box** — `start`'s square of half-side ρ = \|d\| + `REACH_MARGIN`, `REACH_MARGIN` = **2 200 mm**. A solid, a person or an object's footprint enters the scene iff its box meets the reach box; the others are left out. Canonical order is kept among those inserted (floor slab, walls, solids in authored order, people by `EntityId`, objects by `ItemId`); scene indexes are mapped back to the place's lists, so `Touch`, `in_scene` and every answer name the same entities. `REACH_MARGIN`'s derivation, stated in `reach.rs` and checked by a unit test from the constants: the walker ends within \|d\| of `start` (no stride is lengthened); a generation-1 nudged person stood within 2R (600) of the walker's end and moves ≤ NUDGE_MAX + GAP (310); its sweep can touch a solid within R (300) of its path → 1 210; a generation-2 person stood within 2R of a generation-1 end → 1 510 and moves ≤ 310 → 1 820, a solid within R of it → 2 120; rounded up to 2 200. Entry's E2 uses the same box around `to` with \|d\| = 0. | **I** (must be byte-identical) | Every collider left out is one no query of this resolution can touch: a shape cast's time of impact against a collider does not depend on other colliders. What could still differ is the order in which Rapier visits equal hits; that is exactly what TZ-2 measures, and a difference is a finding, not a re-capture (§20.4). |
| **SD-Z2** | **A people scene updates its broad phase only** (DB-10 option 2's other half). `Scene::build` replaces `refresh` by the broad-phase update the query pipeline reads, if 0.36.0 exposes one (F-Z2); `Pile` and `fly` keep `refresh`, F-P1's re-mark and its canary unchanged. | **I** | The narrow phase's contacts are read by no sweep. If no broad-phase-only update exists in 0.36.0, SD-Z2 is dropped and recorded (ZC-2). |
| **SD-Z3** | **An exact corridor.** `corridor_clear`'s solid test becomes the exact integer distance from the centre's segment to each solid's box ≥ R + GAP (`i128`), in place of the boxes' overlap. The people and objects tests are already exact. | **I** | A corridor that is exactly clear by R + GAP is one Rapier sweeps to `to` and the snap (1 mm) returns `to`, which is the fast path's answer. A difference is a finding (§20.4). |
| **SD-Z4** | **Integer wall strides** (DB-10 option 3), `walls.rs`. A stride whose corridor is not clear is answered without Rapier when all hold: `start` lies inside C, the floor shrunk by R + GAP; no solid's box, grown by R + GAP, meets the bounding box of `start` and `target`; every other person's centre is at least 2R + GAP from that box; every object's footprint is at least R + GAP from it. Then the walker ends at `clamp(target, C)` — each coordinate clamped to C — which is the controller's stop at the wall (offset GAP) and its slide along it, and which lies in the box, never farther from `start` than `target` (a projection onto a convex set containing `start`). `stopped_by` None; `Route::Walled`; no nudge, no push (nothing else is in reach); V1 … V4 still run. Every other stride goes to Rapier as today. | **R** (may change results) | Most of the towns' stopped strides are walls (F-Z5). The stop is exactly Rapier's (PB-5 a); the slide keeps all the tangential motion, which Rapier's controller may not (F-Z4) — a deliberate, recorded change to SD-B6 step 4 for this one case. |
| **SD-Z5** | **FU-12c-1: a stride away from a person within the offset is not stopped by them.** In the contact sweep (`Against::Contact`) and the nudge pass's sweeps, a person whose centre lies within 2R + GAP of `start` and on the far side of `start` from the stride (d · (p − start) ≤ 0) is excluded from the sweep. Verification (V1) still counts them, so the result can never overlap them. | **R** | DO-11's fix, in the PR that re-captures the bases anyway (QD-10). |

`VERSION` 2 → 3: SD-Z4 and SD-Z5 change results; a 12c save is refused by name (`ARC-25`).

**Amendment (2026-10-08, primary session's ruling on §20.13 Z-D2, option (b)).** SD-Z3 is reclassified
from Class I to **Class R**. Its rationale above ("a corridor exactly clear by R + GAP is one Rapier
sweeps to `to`") is false (E-Z1): Rapier's character controller is not exact on strides that start at,
or glide along, its offset from a face, so every stride moved from Rapier to the integers may change
its result. SD-Z3 stays as designed (the exact integer segment-to-box distance ≥ R + GAP); what changes
is its class: it changes results by design, together with SD-Z4 and SD-Z5, and `VERSION` 3 covers it.
SD-Z2 stays Class I (E-Z1: PASS).

**Amendment (2026-10-08, the operator's ruling on §20.13 Z-D6: "fix the scan, and switch to an absolute
budget").** A new Class I piece, evidence E-Z3 (E3's scan is 49 % of the with-bodies run):

| ID | Decision | Class | Rationale |
| --- | --- | --- | --- |
| **SD-Z6** | **E3's nearest free point, searched outward** (`entry.rs` `nearest_free`). In place of scanning the floor's whole 50 mm lattice and taking the minimum of (distance², y, x) over the free points, the search visits the lattice in square rings of index distance k = 0, 1, 2 … about the lattice point nearest `to` (its indexes clamped into the lattice), keeps the minimum of the same key over the free points it visits, and stops once a ring's lower bound on distance² — (k · LATTICE − m)², m the larger axis offset of `to` from the ring's centre — exceeds the best distance² found, or the rings have left the lattice. Integers only. | **I** | Every lattice point with a smaller key than the best found lies in a ring not yet past the bound, so the minimum over the visited points is the minimum over all: the same point, under the same total order (points are distinct, so (distance², y, x) has no ties). Unit-tested against the old scan, kept as the test's reference; held by ZI-1 … ZI-3 and TZ-1. |

SD-Z6 is implemented first (ZC-3, which SD-Z1's drop vacated), then SD-Z3 and SD-Z4 (ZC-4) as frozen.

**Amendment (2026-10-08, primary session's ruling on §20.13 Z-D7, option (c)): SD-Z3 and SD-Z4 are
dropped.** Evidence E-Z5: their integer answers differ from Rapier's by more than 50 mm on 2.15 % of
those they give (ZR-3's bound 1 %), and no CPU gain from them is measurable over SD-Z6 alone. Their
code stays on `mvp0/s15-12d0-zc4-wip` @ c1b5749, unmerged, for reference only. What remains: Class I
SD-Z2 and SD-Z6; Class R SD-Z5. TZ-4, TZ-5, M-Z3, M-Z4 and ZR-3 are withdrawn with them; TZ-9a
(≤ 3.0 ×) and TZ-9b (≤ 50 ms) stand as fixed.

## 20.4 Which results must not move, and the re-capture rule (decided before measuring)

**Class I (SD-Z1, SD-Z2, SD-Z3) must be byte-identical.** With SD-Z4 and SD-Z5 off — a crate-private
`Policy { integer_walls: false, away_free: false }`, the production default being on — the build after
ZC-3 must reproduce exactly, against E-Z-base captured on the base before any code:

```text
ZI-1  long_run.rs's second-process bytes (12b's 3 000 requests): sha-256 equal
ZI-2  long_run_objects.rs's bytes (12c's): equal
ZI-3  bodies-yard 30-day seed-7: the summary sha-256 (every line but `wall`) equal to 6e4c4015…c8395
ZI-4  every bodies scenario, objects and actions test passing unedited
```

A Class-I piece that moves any of ZI-1 … ZI-3 is not result-preserving. It is reverted, the first
differing request is located and recorded (§20.13), and the primary session decides whether it is
dropped or moved to Class R under ZR. It is never re-captured silently.

**Class R (SD-Z4, SD-Z5) may change results, and the bases are re-captured only when all of ZR hold:**

```text
ZR-1  every bodies test passes unedited, except the one named flip (FU-12c-1's pin, §20.6); PB-5 a's
      8 010 ± 1 and PB-5 c's slide hold through SD-Z4
ZR-2  every invariant holds after every request: long_run's N-1 … N-4, long_run_objects' SD-O2, the
      bodies-yard scan (0 violations); two processes and SIGKILL byte-identical (bodies_yard_restart)
ZR-3  the shadow comparison: over long_run's 3 000 requests and the 300-day prototype run of §20.12,
      every request SD-Z4 answers is also answered by the Rapier path on the same state (a crate-private
      `explain` over both policies); reported: how many, and the per-axis difference's maximum and
      mean. PASS iff every SD-Z4 answer keeps V1 … V4 and at most 1 % of them differ from Rapier's by
      more than 50 mm on an axis; each one above 50 mm is printed with its state
ZR-4  cross-architecture: the new bodies-yard 30-day sha equal on arm64 and on x86_64 under Rosetta
ZR-5  recorded, before and after, in §20.12: ZI-1's and ZI-2's sha-256 and ZI-3's summary sha — the new
      bases 12d's TD-14 and S17's PL-b read
```

**Amendment (2026-10-08, primary session's ruling on §20.13 Z-D2).** Class I is SD-Z1 and SD-Z2;
Class R is SD-Z3, SD-Z4 and SD-Z5. With SD-Z3, SD-Z4 and SD-Z5 off (`Policy { exact_corridor: false,
integer_walls: false, away_free: false }`), the build must still reproduce ZI-1 … ZI-3 exactly. ZR-3's
shadow comparison covers every request SD-Z3 newly answers as Clear (a stride the old box-overlap
corridor would have swept) as well as every request SD-Z4 answers: each is also answered by the Rapier
path on the same state and compared; the bound is unchanged — every such answer keeps V1 … V4, and at
most 1 % of them differ from Rapier's by more than 50 mm on an axis, each such one printed. The
re-capture rule (ZR-1 … ZR-5) applies once, for SD-Z3, SD-Z4 and SD-Z5 together. TZ-4 and M-Z3 stand
as tests of SD-Z3's rule (Class R's ZR-1 net), not of byte-identity.

**Amendment (2026-10-08, primary session's ruling on §20.13 Z-D4): SD-Z1 is dropped.** Evidence E-Z2:
SD-Z1 is not result-preserving at any provable margin (Rapier's controller depends on colliders far
from what a stride touches), and it buys no measurable CPU on the prototype (30 days: 8.95 s base, 9.17 s
with SD-Z1). As Class R it would be a change of results without a benefit. Class I is now SD-Z2 alone;
Class R is SD-Z3, SD-Z4 and SD-Z5. ZC-3, `REACH_MARGIN`, TZ-3, M-Z1 and M-Z2 are withdrawn. The kept
patch (`/tmp/s15-12d0/sd-z1-reverted.patch`) is reference only. Per `ARC-23` ("locate before counting"),
the ZC-1 profile (Z-D3) is taken before ZC-4, and ZC-4 starts only if the profile's estimate says SD-Z3
and SD-Z4 (with §20.7's ladder) can plausibly reach TZ-9's 1.5 ×; otherwise stop and report. TZ-9's
bound is unchanged.

**Amendment (2026-10-08, primary session's ruling on §20.13 Z-D7, option (c)).** SD-Z3 and SD-Z4 are
dropped (§20.3's amendment; evidence E-Z5). Class I is SD-Z2 and SD-Z6; Class R is SD-Z5 alone. ZR-3
(the shadow comparison of integer answers) has no subject left and is withdrawn; ZR-1, ZR-2, ZR-4 and
ZR-5 apply to SD-Z5. ZR-4 is also run for the Class-I build.

Towns: neither installs bodies before 12d, so social-cafe's and market-town's 300-day digests are
unchanged by 12d-0 in every class (TZ-1).

## 20.5 Acceptance (decided before measuring, `ARC-23`)

```text
TZ-1  Nothing else moves: both towns' 300-day seed-7 digests equal E-TD0's (ad49c723…c64b,
      365b50e0…1d1d); `validate` of every world byte-identical to the base binary's.
TZ-2  Class I is byte-identical (ZI-1 … ZI-4), with SD-Z4 and SD-Z5 off.
      M-Z1  REACH_MARGIN 2 200 → 0 → ZI-1 differs, or a nudge scenario fails (a nudged person's wall left
            out of the scene) — named.
      M-Z2  the reach box drops people (solids only) → a scenario fails, naming the person not nudged.
TZ-3  SD-Z1's margin is derived, not tuned: a unit test recomputes 2 200 from PERSON_RADIUS, GAP,
      NUDGE_MAX and CHAIN_MAX and fails if a constant moves without it.
TZ-4  SD-Z3 (walls.rs and geometry tests, hand-computed literals): a diagonal stride 320 mm from a lamp
      post's box (its bounding box overlapping the post) is Clear, exactly `to`; at 300 mm it is swept.
      M-Z3  the corridor back to box overlap → the 320 mm case is Swept.
TZ-5  SD-Z4 (tests/walls.rs, the café room of §17.4, hand-computed literals, through World::dispatch):
        a  east from (7 000, 3 000) to (8 500, 3 000) → (8 010, 3 000) exactly, stopped-short by None,
           Route::Walled, no scene built (a crate-private counter of scenes built reads 0);
        b  diagonal from (7 000, 3 000) to (8 400, 4 400) → (8 010, 4 400) exactly;
        c  into the north-east corner → (8 010, 10 010);
        d  the same stride with a person 700 mm off its box → Route::Swept (Rapier), as before;
        e  a start 305 mm from the wall (outside C) → Swept.
      M-Z4  C shrunk by R instead of R + GAP → a fails: 8 020.
TZ-6  SD-Z5: `a_shove_from_600_mm_…` (renamed `a_shove_from_600_mm_moves_its_target_half_a_metre`)
      → B at (4 500, 5 000), no stopped-short; a stride toward a person 600 mm away is still stopped.
      M-Z5  the d · (p − start) ≤ 0 test dropped (every near person excluded) → the toward case overlaps,
            and V1 degrades it, named.
TZ-7  Class R's rule ZR-1 … ZR-5 holds, and the new bases are recorded.
TZ-8  Determinism: two processes and SIGKILL (bodies_yard_restart.rs), `replay` from genesis; ZR-4.
TZ-9  THE GATE — QB-11 on E-TD0b's prototype (§20.12), never re-scoped: dev profile, 300 days, seed 7,
      for the social-cafe prototype and the market-town prototype: each with bodies and its copy
      without (bodies out of `systems`, every `body:` stripped), two runs each, consecutive, one
      machine, nothing else building or running. PASS iff, for each town, max(with) ≤ 1.5 × min(without),
      faults 0 in all eight runs. Also reported, not judged: release µs per swept move and per Walled
      stride (long_run), the share of strides by Route, and each town's activity lines.
      FAIL → STOP. The numbers go back to the operator, as the ruling requires. No further optimization
      is added inside 12d-0 after a failed measurement except §20.7's pre-declared ladder.
      SUPERSEDED (2026-10-08, the operator's ruling on Z-D6, see the amendment below): the 1.5 ×
      bound is kept here as the record of what was frozen; TZ-9a and TZ-9b replace it.
TZ-10 Structural: Rapier only in rapier.rs, no float outside it (isolation); rapier_pin (3, "0.36.0");
      ac1_composability, precursor_vocabulary, seam_vocabulary unedited; scope ⊆ §20.1; fmt, clippy;
      the full gate once on the final executable head.
```

**Amendment (2026-10-08, the operator's ruling on §20.13 Z-D6: "fix the scan, and switch to an absolute
budget"). TZ-9 is re-scoped openly, fixed here before any measurement (`ARC-23`).** Evidence: E-Z3 —
with every Rapier stride at zero cost and E3's scan removed, the prototype still costs ≈ 2.0 × (bodies'
≈ 45 % more facts), so QB-11's relative 1.5 × (TZ-9 as frozen above, now SUPERSEDED, not deleted) is
not reachable by any piece of §20. Its replacement:

```text
TZ-9a THE GATE, relative, re-scoped: for each of the social-cafe and market-town prototypes (§20.12),
      300 days, seed 7, dev profile: max(CPU, with bodies) ≤ 3.0 × min(CPU, without), faults 0 in
      every run. Instrument §20.6.1 unchanged: user + sys CPU from `/usr/bin/time -l`; per town two
      runs per side interleaved (without, with, without, with); wall and load recorded as information;
      the two pairs' ratios more than 10 % apart → contaminated, re-run once, then stop and report.
TZ-9b THE GATE, absolute: CP-B4's live-mode bound (step-12-server.md §9: the world thread's longest tick
      ≤ 50 ms) holds with bodies on market-town. S11-B's probe (tools/cli/tests/hosted_town.rs) is not
      on main (e98321a), so the measure is the per-request resolve time of bodies' ArrivalResolver over
      the market-town prototype's 300-day run: every call timed by a scratch instrumented build outside
      the repository (Z-D3's method; nothing of it committed), reported as count, p50, p99 and max.
      PASS iff max ≤ 50 ms (a request resolves within one tick); p99 reported.
STOP  TZ-9a or TZ-9b failing after SD-Z6, SD-Z3, SD-Z4 and §20.7's ladder → stop and report with the
      numbers. No further re-scope.
```

**Amendment (2026-10-08, the operator's ruling on §20.13 Z-D9: "accept 3.60 × on social-cafe and open
the PR").** The bound text above is unchanged. TZ-9a is recorded as **accepted, not passed**:
market-town 2.999 × passes it; social-cafe 3.60 × (E-Z7, a clean re-run) exceeds it and is accepted by
the operator, on the absolute numbers — ≈ 45 s of CPU with bodies per 300 prototype days (≈ 110 s
before 12d-0), and TZ-9b's per-resolution time p99 0.33 ms, max 10.5 ms against 50 ms. §20.7's ladder was
not tried. **The next candidate if bodies' cost ever becomes a bottleneck is L1, as Class R** (skipping
the contact sweep when nobody is near; E-Z7's addendum sizes it at the ≈ 37 % of Rapier strides' cost
that 3.0 × would need; it cannot be Class I, E-Z2). Z-D8 is accepted by the same ruling: TZ-6's 600 mm
shove is pinned at its measured point (4 499, 4 999).

## 20.6 Commit plan

Rules as §19.5 (separate implementation, validation and review items; `E-Z<n>` evidence; Edit and Write
only; long runs in the background).

### ZC-0 — Design (this section) — docs only

- [x] Implementation: §20 and the header lines, by the planning session on `mvp0/s15-12d0-plan`.
- [x] Validation: both doc checks (E-Z0).
- [x] Review: every §20.2 claim cites a file and line, Rapier's source, or E-TD0b; Class I and Class R
  fixed before any measurement; the gate's stop stated.

### ZC-1 — Base captures and the prototype, before any code

**Scope.** E-Z-base: the base `mineworld` binary to `/tmp/s15-12d0/base-mineworld`; ZI-1's and ZI-2's
bytes (`BODIES_LONG_RUN_SECOND_PROCESS=1`, and long_run_objects' second-process mode) and their sha-256;
ZI-3's summary; both towns' 300-day digests (TZ-1). The prototypes regenerated by §20.12's recipe, both
towns and both copies without bodies; `validate` of each; **the gate's "before"**: TZ-9's eight runs on
the base binary, recorded (they reproduce E-TD0b's order of magnitude or the recipe is wrong). A
profile, information only: with the base binary, 30 prototype days, the share of strides by Route and
the scene's mean collider count (a crate-private counter read by an ignored test).
- [ ] Implementation: the captures, the prototypes, the profile. Captures and prototypes done
  (E-Z-base). **Pending:** the gate's "before" (eight runs need a quiet machine; the load average was
  23 … 61 throughout, other lanes building and running) and the profile (to be taken with a scratch
  instrumented build outside the repository — a crate-private counter cannot be read by an ignored
  test from the CLI's run; §20.13 Z-D3). The "before" taken under §20.6.1's instrument: 7 of 8 runs,
  the eighth cut by the previous session's end (E-Z-before); the profile taken by sampling, 30 and
  300 days (E-Z3), in place of route counters (Z-D3).
- [ ] Validation: each capture's sha recorded (E-Z-base); the prototypes validate (E-Z-base). The
  "before" reproduces E-TD0b's order of magnitude (E-Z-before: 7.4 … 8.0 × on social-cafe).
- [x] Review: the recipe in §20.12 is what was run, file by file — each place and item file
  transcribed from §19.3.1 and the listing, and checked line for line against E-TD0b's own copy
  (E-Z-base).

### ZC-2 — SD-Z2 and SD-Z3 (Class I)

**Scope.** `rapier.rs` (`Scene::build` without the narrow phase, if 0.36.0 allows; audited first),
`geometry.rs` (`corridor_clear` exact against solids); geometry unit tests (TZ-4).
- [x] Implementation, SD-Z2: `rapier.rs` `index` — the broad-phase update `CollisionPipeline::step`
  makes, without the narrow phase (audit E-Z1); `Scene::build` calls it; `Pile`, `fly` and `refresh`
  unchanged.
- [ ] Implementation, SD-Z3: written (`Area::clear_of_segment`, `crossed_by`; `corridor_clear`; TZ-4's
  unit tests), **moved ZI-1 and ZI-2, reverted** (E-Z1; §20.13 Z-D2, MATERIAL STOP).
- [x] Validation, SD-Z2: ZI-1 … ZI-4 identical with SD-Z2 alone (E-Z1).
- [ ] Validation, SD-Z3: FAILED TZ-2 — not result-preserving (E-Z1). M-Z3 not run (nothing to mutate).
- [x] Review, SD-Z2: `Pile` and `fly` keep `refresh`; F-P1's canary and fix unchanged (unit tests
  pass); the update's parameters and inputs are `CollisionPipeline::step`'s, read in 0.36.0's source.

### ZC-3 — SD-Z1, the reachable scene (Class I)

**Scope.** `reach.rs` (new), `rapier.rs` (`Scene::build` over a subset with an index map), `stride.rs`,
`entry.rs`; `tests/cull.rs` (TZ-3, M-Z1, M-Z2).
- [ ] Implementation: written (`reach.rs` with TZ-3's unit tests, `stride.rs`, `entry.rs`,
  `tests/cull.rs`), **moved ZI-1, ZI-2 and ZI-3, reverted** (E-Z2; §20.13 Z-D4, MATERIAL STOP). The
  patch is kept at `/tmp/s15-12d0/sd-z1-reverted.patch` (sha-256 13dad5e1…f0e31), not committed.
- [ ] Validation: FAILED TZ-2 — not result-preserving (E-Z2). M-Z1, M-Z2 not run.
- SD-Z1 DROPPED (§20.4 amendment). ZC-3 now carries SD-Z6 (§20.3 amendment):

### ZC-3 (amended 2026-10-08) — SD-Z6, E3 searched outward (Class I)

**Scope.** `entry.rs` (`nearest_free`), its unit tests (the outward search equal to the full scan on
generated rooms, targets and occupancies; the old scan kept as the tests' reference).
- [x] Implementation: `entry.rs` `nearest_free` (rings about the clamped nearest indexes, the strict
  bound); `tests` with the old scan as `scanned` (E-Z4).
- [x] Validation: ZI-1 … ZI-4 identical (E-Z4); 3 000 generated cases equal to the scan; M-Z6 (`>=`
  for `>`) → `an_equal_distance_in_a_later_ring_wins_on_y` fails, "ring 5 not visited" (E-Z4).
- [x] Review: integers only (`i64` indexes; `i32` only for points on the floor); the bound's proof in
  the doc comment; `free` is evaluated only for a point whose key would win, which changes nothing (it
  is pure); nothing kept between calls.
- [ ] Review: canonical order kept among those inserted; every `Touch` maps back to the right entity;
  nothing cached.

### ZC-4 — SD-Z4, integer wall strides (Class R)

**Scope.** `walls.rs` (new), `stride.rs` (the route before the scene), `resolve.rs` (`Route::Walled`;
`Policy.integer_walls`), `tests/walls.rs` (TZ-5, M-Z4); the shadow comparison harness (ZR-3) as an
ignored test reading both policies.
- [ ] Implementation: written with SD-Z3 (E-Z5), parked on `mvp0/s15-12d0-zc4-wip` @ c1b5749 — not
  on the PR branch (Z-D7).
- [ ] Validation: TZ-4, TZ-5, M-Z3, M-Z4, ZR-1, ZR-2's long_run invariants and the off-policy identity
  PASS; **ZR-3 FAILS** on long_run (9 of 418 > 50 mm, 2.15 %; E-Z5; §20.13 Z-D7, MATERIAL STOP).
- [ ] Review: the clamp never lengthens (proof in `walls.rs`; a property test over 20 000 generated
  strides in place of the long run's requests); no float; V1 … V4 still applied. Pending the ruling.

### ZC-5 — SD-Z5, FU-12c-1 (Class R)

**Scope.** `rapier.rs` (the sweep's exclusion predicate), `stride.rs`; `tests/actions.rs`: the DO-11 pin
renamed and flipped (TZ-6, M-Z5).
- [x] Implementation: 31e3779 (E-Z6). The nudge pass's sweeps are walls-only and never meet people, so
  only the walker's contact sweep leaves anybody out.
- [x] Validation: TZ-6 (with Z-D8's literal), M-Z5, ZR-1, ZR-2; with SD-Z5 off ZI-1 … ZI-3 = E-Z-base
  (E-Z6).
- [x] Review: excluded only while within 2R + GAP (inclusive) and behind (d · (p − start) ≤ 0, d the
  aim's offset, the bias included); V1 still counts them (`verifies` reads every other person); with
  nobody left out the query filter is the one built before.

### ZC-6 — VERSION, documents, re-capture, the gate, close

- [x] `VERSION` 3; `rapier_pin`; ARC-39 note 3 (the next free number), DEP-13 note; README; MVP_STATUS
  (df94e46).
- [x] ZR-1, ZR-2, ZR-4, ZR-5: the new bases recorded (E-Z6, E-Z7). ZR-3 withdrawn (§20.4 amendment).
- [x] **TZ-9a: market-town 2.999 × passes; social-cafe 3.60 × exceeds and is ACCEPTED by the operator
  (§20.5 amendment, Z-D9 ruling); TZ-9b PASSES (max 10.5 ms)** — E-Z7.
- [x] TZ-1, TZ-8, TZ-10; the full gate once (E-Z7); the ledger; the handoff.
- [ ] Review: every TZ with evidence; deviations named; §19's TD-14 references updated in the ledger
  only (the planning session updates §19 after merge).

### 20.6.1 TZ-9's instrument under machine load (primary session's ruling, 2026-10-08, before any TZ-9 run)

The machine is never quiet (eight lanes in parallel; E-Z-base saw load averages of 23 … 61), so TZ-9's
measure is fixed now, before any run:

```text
measure     user + sys CPU time of each 300-day run, from `/usr/bin/time -l` — far less sensitive to
            load than wall time, and still the cost of bodies, which is what QB-11 bounds
pass        for each town, max(CPU, with bodies) ≤ 1.5 × min(CPU, without) — QB-11, unchanged
order       per town, two runs per side, interleaved: without, with, without, with — both sides share
            the load
recorded    wall time and the load average before and after each run, information only
load stop   if on either town the CPU ratios (with ÷ without) of the two pairs differ from each other
            by more than 10 %, the measurement is contaminated: re-run once; if they still differ,
            stop and report
```

The bound is not re-scoped and §20.7's ladder is unchanged. The same instrument is used for the
"before" (ZC-1) and for each ladder rung.

## 20.7 If the gate fails: the only ladder, fixed now

If TZ-9 fails with SD-Z1 … SD-Z5 in, these may be tried, in order, each measured once, each recorded;
then stop:

```text
L1  (Class I) the contact sweep skipped when no person or object meets the reach box (B = W)
L2  (Class R, under ZR) a nudge whose path stays inside C and meets no solid's grown box is answered by
    integers, as SD-Z4 answers a walker
```

Nothing else — no change to the prototype's geometry to pass (its solids are 12d's question, QD-2 (c),
the operator's), no constant, no controller change. If L2 does not pass, the stop goes to the operator
with every number.

## 20.8 Test ownership

```text
UNIT        reach.rs (the margin's derivation), walls.rs (the clamp), geometry (the exact corridor)
INTEGRATION tests/walls.rs, tests/cull.rs; every existing bodies test (the Class-I and ZR-1 net)
REAL RUN    bodies_yard*.rs (committed); TZ-9's prototype runs, ZR-3's shadow, Rosetta (recorded)
STRUCTURAL  isolation, rapier_pin, the three acceptance scans, scope
GATE 1      NOT REQUIRED
```

## 20.9 Is any of this material?

Within the ruling, no: options 2 and 3 are its words, FU-12c-1 is QD-10's, and the re-capture rule is
the one the ruling asked to be stated. Two things return to the operator if they happen: **TZ-9 failing**
(the ruling's stop), and a Class-I piece that cannot be made byte-identical **and** is needed for TZ-9
(it would then change results beyond what the ruling named).

## 20.10 Questions (QZ-1 …)

| ID | Question | Recommendation |
| --- | --- | --- |
| **QZ-1** | SD-Z4 keeps all tangential motion along a wall where Rapier's controller may keep less (F-Z4). Accept the change of definition? | **Yes**: it is the simpler, exactly stated rule, and ZR-3 measures the difference. |
| **QZ-2** | `REACH_MARGIN` 2 200 mm, derived (SD-Z1). | **Accept.** |
| **QZ-3** | The gate on **both** towns' prototypes, though the ruling names "your prototype" (social-cafe). | **Both**, since 12d's TD-12 binds both; market-town's prototype is the same geometry plus its market. |
| **QZ-4** | The ladder L1, L2 (§20.7) fixed now. | **Accept**; anything beyond it is the operator's. |
| **QZ-5** | The prototype stays uncommitted (`/tmp`, from §20.12's recipe). | **Yes**: 12d owns the towns' content; a committed fixture would be a second copy of it. |

## 20.11 Proposed execution contract for PR 12d-0

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12d-0 — bodies' cost (S15, precursor to 12d)
PRIMARY DESIGN DOC  step-11-bodies.md §20; evidence §20.12 (E-Z<n>); deviations §20.13
RELATED / BINDING   §19's header (the QD-2 ruling), §17.11 DB-10, §18, E-TD0b; DEP-13, ARC-39; CLAUDE.md §§2–4
IMPLEMENTATION BASE main after this design's PR (f842c52 + Markdown); branch mvp0/pr-12d0-cost; worktree
                    /Users/yuema137/mineworld-worktrees/s15-12d0 (proposed)
APPROVED SCOPE      §20.1; ZC-1 … ZC-6; SD-Z1 … SD-Z5; §20.7's ladder only on a failed gate
FROZEN INVARIANTS   no diff outside systems/bodies/ and the named documents; bodies' rules unchanged but
                    SD-Z4 and SD-Z5; Class I byte-identical (ZI-1 … ZI-4); Class R only under ZR; towns'
                    digests unchanged (TZ-1); QB-11 1.5 × on the prototype, never re-scoped; nothing of
                    Rapier survives a resolution (I-5); no float outside rapier.rs
VALIDATION BUDGET   unit/integration unrestricted; 300-day prototype runs: 8 before (ZC-1), 8 for the gate,
                    8 per ladder rung at most; ZR-3's shadow once; the x86_64 build once; one full gate;
                    about two hours; real-model NOT REQUIRED
ENDPOINT AUTHORITY  commits, push, PR: recommended authorized as for 12a–12c; merge: operator only, merge
                    commit
NORMAL STOP         PR 12d-0 READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       TZ-9 failing after §20.7's ladder; a Class-I piece not byte-identical and needed; an edit
                    outside the change set; a town digest moving; Rosetta differing
POST-MERGE SYNC     planning session: header, §19 (its base and TD-14 references), overall, MVP_STATUS
```

## 20.12 Evidence ledger, and the prototype's recipe

```text
E-Z0 ZC-0, 2026-10-08, planning session, on mvp0/s15-12d0-plan (main f842c52 + §19, §20).
     Doc checks: 191 numbered sections across 26 documents, none duplicated; 55 decision ids, all
     distinct. No cargo run: the prototype measurements are E-TD0b's.

E-Z-base ZC-1, 2026-10-08, implementation session, on mvp0/s15-12d0-plan @ 953ff10 (= main f842c52 +
     §§19–20, Markdown only), before any code. Machine: Apple silicon, 10 cores, rustc 1.97.1; other
     lanes building and running in parallel (load average 23 … 61) — none of these captures is a time.
     `cargo build -p mineworld-cli` (dev) 1 min 41 s; target/debug/mineworld copied to
       /tmp/s15-12d0/base-mineworld, sha-256 7ba96db5fa9b3a41e20db4d3365ce5bfc0f5f52a1a41d506da05931aa25040b0.
     /tmp/s15-12d0/capture.sh base (cap-base.log):
       ZI-1  `BODIES_LONG_RUN_SECOND_PROCESS=1 cargo test -p mineworld-bodies --test long_run --
             --nocapture` → exit 0; the `LONG-RUN` line 4 091 748 bytes, sha-256
             d7025dbcdb63c0aa5162c10552f43e2650d24e510dc67f4662ac30d1b2479eaf (= E-PO-base: 12c kept it)
       ZI-2  `BODIES_LONG_RUN_OBJECTS_SECOND_PROCESS=1 … --test long_run_objects` → exit 0; the
             `LONG-RUN-OBJECTS` line 569 950 bytes, sha-256
             53d017d0b8dbf5c3c7e8d6bc7830e2bedcac28eb72bd4fd66f3c4a6cc8595411
       ZI-3  `base-mineworld run worlds/bodies-yard --headless --seed 7 --days 30` → exit 0, faults 0,
             62 855 facts, summary sha-256 (every line but `wall`)
             6e4c4015077924b6747184dd0107164654cefc11532375aff3a9d94a4d7c8395 (= PO-12's 6e4c4015…c8395)
     /tmp/s15-12d0/towns.sh base (towns-base.log), TZ-1's references:
       validate bodies-yard 7356b8f8…12063f, market-town 64f41086…73502 (= E-PO-base), social-cafe
         ebcd60a0…f56a8 (= E-PO-base), each exit 0 (sha-256 of the output)
       social-cafe 300 days: faults 0, 365 330 facts, ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b
       market-town 300 days: faults 0, 372 755 facts, 365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d
       Both = E-TD0. PASS (the references reproduce).
     The prototype, rebuilt from the recipe below into /tmp/s15-12d0/proto/worlds/{social-cafe,
       market-town} (step 1 by `GIT_INDEX_FILE=/tmp/s15-12d0/proto.idx git --work-tree=/tmp/s15-12d0/proto
       checkout 21f96ff -- worlds/social-cafe worlds/market-town`; steps 2–3 by the Write and Edit
       tools, each place file's passages and `body:` and each item file transcribed from §19.3.1 and the
       listing below; market-town's items inserted in key order — cafe-ball and cafe-box after bread,
       street-ball and street-box after soup), and the copies without bodies (step 4) into
       /tmp/s15-12d0/nobodies/worlds/{social-cafe,market-town}: `- bodies` removed, every `body:`
       removed, the item files left with their tags. Header comments replaced by one line naming the
       recipe; every `note:` kept as authored. `validate` (step 5): proto social-cafe 22 entities,
       67 genesis facts (= the recipe's); proto market-town 44 entities, 143; nobodies social-cafe
       22, 53; nobodies market-town 44, 129 — all valid. Cross-check against E-TD0b's own copy
       (/tmp/s15-12d/proto, still on disk): every place and item file of social-cafe equal line for
       line once comments and `note:` text are set aside.

E-Z1 ZC-2, 2026-10-08, working tree on 953ff10 + rapier.rs (SD-Z2) and geometry.rs (SD-Z3).
     SD-Z2's audit (rapier3d 0.36.0 source, ~/.cargo/registry): `PhysicsWorld::detect_collisions`
       (src/pipeline/physics_world.rs:175–186) is `CollisionPipeline::step`
       (src/pipeline/collision_pipeline.rs:162–219): `bodies.take_modified()`,
       `colliders.take_modified()` / `take_removed()` (public, collider_set.rs:87, 103), the user
       changes, then `detect_collisions` (:60–127) = `BroadPhaseBvh::update` (public,
       src/geometry/broad_phase_bvh/update.rs:35) with `IntegrationParameters {
       normalized_prediction_distance: prediction_distance, dt: 0, .. }`, then the narrow phase's
       `register_pairs`, `compute_contacts`, `compute_intersections`. The user changes recompute a
       collider's pose from its parent only on PARENT changes, and attaching a collider already set it
       (rigid_body_components.rs:1242). So 0.36.0 exposes a broad-phase-only update: SD-Z2 is kept,
       as `rapier.rs` `index`.
     `cargo test -p mineworld-bodies --lib`: 16 passed (TZ-4's two new unit tests among them).
     /tmp/s15-12d0/capture.sh c2 (both pieces; cap-c2.log): ZI-1 4 115 336 bytes, sha-256
       b3a570410064e2b81b1004a6e7ac98d50faf842a9ffb0acfa4da5bd948d9b90b ≠ E-Z-base; ZI-2 596 917
       bytes, 03d627deed533e7ea75d2bbe8fdc1d8fcd8432ce311a46f15a302753650e2b94 ≠ E-Z-base; ZI-3
       6e4c4015…c8395 = E-Z-base. `cargo test -p mineworld-bodies`: every test passes (ZI-4).
     Bisected, each by the second-process line of long_run and long_run_objects:
       SD-Z3 alone (`refresh` restored)   ZI-1 b3a57041…, ZI-2 03d627de…   — both moved
       SD-Z2 alone (old corridor)         ZI-1 d7025dbc…, ZI-2 53d017d0…   — both = E-Z-base
     So SD-Z3 is the Class-I piece that is not result-preserving. The first differing request, located
       by a temporary probe in long_run.rs (each move's walker, from, to, explain's Outcome and its
       facts, printed; not committed), comparing the base corridor with SD-Z3's:
       request 307 (action 266), p09 (EntityId 11), from (5 986, 6 260) to (4 164, 6 260): due west
       along the counter's south face at exactly R + GAP (6 570 − 310 = 6 260); not biased.
         base   Swept: Rapier ends at (4 165, 6 211) — 49 mm off the face, 1 mm short — stopped-short
                { by: None }
         SD-Z3  Clear: exactly (4 164, 6 260); no stopped-short
       (request 111 also changed route, Swept → Clear, with the same facts: Rapier reached `to` there.)
     Evidence for the decision, not adopted: SD-Z3 with a strict margin (the segment at least
       R + GAP + 1 = 311 mm from every solid): ZI-2 = E-Z-base, ZI-1
       852c88e14c287d1520ddb0add69c1b0de34e2234997edca67573d5170c630545 ≠. First differing request
       518, p06 from (3 636, 7 480) to (3 636, 9 098), due north, its segment 382 mm from the counter
       at the nearest: base Swept → (3 638, 9 097), stopped-short { by: None } (Rapier drifted 2 mm
       east, 1 mm short, beyond SNAP); strict SD-Z3 Clear → exactly (3 636, 9 098).
     Reading: SD-Z3's rationale (§20.3: "a corridor that is exactly clear by R + GAP is one Rapier
       sweeps to `to` and the snap returns `to`") is false. Rapier's controller does not return `to`
       on every stride whose corridor keeps R + GAP from everything: it ends up to 49 mm off where it
       glides at the offset (request 307), and drifts 2 mm in open floor where it starts at the offset
       (request 518). Any widening of the clear set therefore changes results on the strides it moves
       from Rapier to the integers, whatever the margin. SD-Z3 reverted, with its TZ-4 unit tests
       (§20.13 Z-D2).
     SD-Z2 alone, fresh build (/tmp/s15-12d0/c2b-mineworld; capture.sh c2b, cap-c2b.log): ZI-1
       d7025dbc…79eaf, ZI-2 53d017d0…95411, ZI-3 6e4c4015…c8395 — all three = E-Z-base. ZI-4
       `cargo test -p mineworld-bodies`: lib 14, actions 27, genesis 6, isolation 5, long_run 1,
       long_run_objects 1, objects 7, objects_genesis 10, rapier_pin 1, scenarios 17 — every test
       passes, none edited. TZ-2 for SD-Z2: PASS.

E-Z-before ZC-1, the gate's "before", 2026-10-08 14:24 … 14:33, base binary (/tmp/s15-12d0/base-mineworld,
     7ba96db5…40b0), /tmp/s15-12d0/gate.sh under §20.6.1's instrument (CPU = user + sys from
     `/usr/bin/time -l`; interleaved without, with, without, with; dev binary, 300 days, seed 7). Started
     by the previous session; that session ended during the eighth run, which never finished (no
     process left; its .time file empty). Information, not a gate:
       social-cafe  without 14.05+0.19 = 14.24 s (wall 14.9, load 15.3 → 13.0)
                    with   103.07+1.59 = 104.66 s (wall 110.2, load 13.0 → 7.3)
                    without 14.14+0.15 = 14.29 s (wall 14.7, load 7.3 → 6.5)
                    with   111.90+1.72 = 113.62 s (wall 150.3, load 6.5 → 43.8)
                    pair ratios 7.35 and 7.95 (8.2 % apart, inside the 10 % load stop);
                    max(with) ÷ min(without) = 7.98 ×
       market-town  without 20.32+0.19 = 20.51 s (wall 31.5, load 43.8 → 47.0)
                    with    93.66+0.74 = 94.40 s (wall 95.5, load 47.0 → 12.9)
                    without 15.60+0.09 = 15.69 s (wall 15.8, load 12.9 → 11.0)
                    with    — not finished
                    pair 1 ratio 4.60; max(with) ÷ min(without) ≥ 94.40 ÷ 15.69 = 6.02 ×
     Faults 0 in all seven; with bodies 525 890 facts (social-cafe), 528 325 (market-town). The order of
     magnitude reproduces E-TD0b's (the recipe stands). Not re-run: the "before" judges nothing.

E-Z2 ZC-3, 2026-10-08, working tree on e179132 + SD-Z1 (`reach.rs`, `stride.rs`, `entry.rs`,
     `footprint.rs` `bounds` made crate-visible, `tests/cull.rs`), Class-R pieces absent
     (`Policy.exact_corridor` false; SD-Z4, SD-Z5 not written). Recovered from the previous session's
     uncommitted tree (§20.13 Z-D5). /tmp/s15-12d0/step.sh <label> (dev build, capture.sh, the bodies tests):
       REACH_MARGIN 2 200 (c3)        ZI-1 4 187 312 bytes fda31dd8ca4d5e84dd60269a6d61ab63ec21b9a9035ee469a7bfc8ee1bb45d84
                                      ZI-2 706 094 bytes 7ec3977e3c21ca013c3e937673ad230ddc9f999cf0482876e53c712a3e276309
                                      ZI-3 bc470547d9ca1239a4389d905d1e683e2de90e0898a198a6850b3e37fc8a2bf6
                                        (63 221 facts) — all three ≠ E-Z-base
       REACH_MARGIN 1 000 000 (c3wide) ZI-1 d7025dbc…, ZI-2 53d017d0…, ZI-3 6e4c4015… = E-Z-base: the
                                      subset scene and its index maps are correct when nothing is left out
       REACH_MARGIN 10 000 (c3m10k)   ZI-1, ZI-2, ZI-3 = E-Z-base (on these runs; not a proof)
     First differing stride (a temporary probe in resolve.rs resolving every stride twice, whole place and
       reach, printing the first difference; not committed): long_run, the 419th stride of the first
       process — p07 (EntityId 9) from (4 359, 310) to (5 044, 995), a 969 mm stride:
         whole place  (4 840, 791); EntityId 4 nudged to (5 298, 386), 5 to (4 519, 1 311); stopped by 4
         reach        (4 826, 777); EntityId 4 nudged to (5 299, 391), 5 to (4 525, 1 308); stopped by 4
       The reach box (x 1 190 … 7 528, y −2 859 … 3 479) left out the counter (its nearest edge 6 260 mm
       from the start) and ten people (EntityId 2, 7, 8, 10, 11, 12, 13, 14, 15, 16), none within 1 400 mm
       of anything the stride touched. 14 mm on each axis.
     Which half moves it (same probe): left out only people (every solid kept) → differs, first at
       stride 569; left out only solids (every person kept) → differs, at the same stride 569 with the
       same values: p00 (EntityId 2) from (529, 2 820) to (1 122, 3 413): whole place (1 124, 3 177),
       EntityId 10 to (598, 3 490); reach (1 125, 3 177), EntityId 10 to (599, 3 489) — 1 mm.
     Reading: SD-Z1's rationale (§20.3: "a shape cast's time of impact against a collider does not
       depend on other colliders") does not hold for Rapier 0.36.0's character controller as this pack
       uses it: removing colliders far from everything a stride touches moves its result by 1 … 14 mm,
       whichever kind is removed. The likeliest mechanism — not audited further — is that the
       controller's iterated casts and slide depend on the query pipeline's BVH, whose shape depends on
       every collider in it. No margin is a proof of identity; 10 000 mm reproduced these three runs
       only.
     Cost, information only (not TZ-9; /tmp/s15-12d0/z1info.sh: 30 prototype days of social-cafe, CPU =
       user + sys, two interleaved rounds): base 8.95 / 12.54 s; SD-Z1 at 2 200 9.17 / 13.06 s; SD-Z1 at
       10 000 9.07 / 14.95 s; without bodies (base, once) 1.73 s. **SD-Z1 buys no measurable CPU** on the
       prototype: bodies costs ≈ 7.2 s per 30 days (≈ 5 ×), and leaving ≈ 60 of ≈ 70 colliders out of a
       scene does not reduce it. Where that cost lies is unmeasured (the ZC-1 profile, Z-D3, still
       pending).
     The previous session's tests/cull.rs: two of its three scenarios (`a_walker_meets_the_person_on_its
       _path`, `a_person_nudged_toward_a_post_is_held_by_it`) fail on the whole-place scene as well
       (margin 1 000 000): their expected values were never run. Reverted with the rest.
     SD-Z1 reverted: the tree is e179132's code again (`cargo test -p mineworld-bodies --lib`: 14 passed).

E-Z3 ZC-1's profile (Z-D3, by the primary session's ruling before ZC-4), 2026-10-08, the head's code
     (SD-Z2 only; /tmp/s15-12d0/c2b-mineworld, sha-256 23ccb53f…7bdd0, dev profile, optimized + debuginfo).
     Instrument: macOS `/usr/bin/sample <pid> <s> 1` (1 ms interval) on a social-cafe prototype run with
     bodies, seed 7 (/tmp/s15-12d0/profile.sh, profile300.sh; outputs in /tmp/s15-12d0/profile/). Nothing
     installed. Counts are inclusive samples on the main thread (the tokio workers idle throughout).
       30 days   faults 0, 52 595 facts (= the base's); main thread 16 531 samples
       300 days  faults 0, 525 890 facts (= E-Z-before's); main thread 117 604 samples
                                                        30 days            300 days
       bodies' ArrivalResolver::resolve                 12 453  75.3 %     87 926  74.8 %
         entry → E3 `nearest_free` (integers only)       8 235  49.8 %     57 838  49.2 %
           of it, the lattice filter (`Room::admits`, `free_at`) — self time 7 179 of 8 235
         stride (all routes)                             4 109  24.9 %     28 308  24.1 %
           Rapier inside it: `Scene::sweep`              3 145             ≈ 19 700 (10 571 + 9 115)
                             `Scene::build`, drop         ≈ 900             ≈ 5 100
         the rest of resolve (guard, answer)               ≈ 95             ≈ 1 800
       everything else (kernel, presence, the store,     4 078  24.7 %     29 678  25.2 %
         serialization, the controller …)
     Reading: half of the with-bodies run is E3's nearest-free search: `nearest_free` scans the whole
       floor's lattice (LATTICE 50 mm) and filters every point through `Room::admits` against every
       solid before taking the minimum — on the street (68 000 × 16 200 mm, 59 solids) ≈ 440 000 points ×
       59 boxes per placed entry. Rapier (every swept stride: sweeps, scene build and drop) is a quarter.
     The estimate the ruling asks for (300 days; CPU from E-Z-before: with ≈ 109 s, the mean of 104.66 and
       113.62; without 14.27 s, the mean of 14.24 and 14.29; QB-11's bound 1.5 × 14.24 = 21.4 s):
         E3 nearest_free     49.2 % ≈ 53.6 s
         stride (≈ Rapier)   24.1 % ≈ 26.2 s
         the rest            26.7 % ≈ 29.1 s, of which ≈ 14.3 s is the world without bodies and ≈ 15 s
                             bodies' other cost (≈ 164 000 more facts stored and disclosed: 525 890 vs
                             361 979)
       SD-Z3 + SD-Z4, and L1 + L2, remove only stride's Rapier work. Their best case — every swept stride
       answered by integers at zero cost — leaves 109 − 26.2 ≈ 82.8 s ≈ **5.8 ×** min(without). TZ-9
       needs ≤ 21.4 s. **They cannot reach 1.5 ×, with or without the ladder.** Even removing E3's search
       entirely as well leaves ≈ 29 s ≈ 2.0 ×. (30 days, the same arithmetic on 8.95 s with, 1.73 s
       without: best case for SD-Z3 + SD-Z4 + L1 + L2 ≈ 6.7 s ≈ 3.9 ×.)

E-Z4 ZC-3 (SD-Z6), 2026-10-08, working tree on 8ea28d4 + entry.rs.
     `cargo clippy -p mineworld-bodies --all-targets -D warnings` clean; `cargo test -p mineworld-bodies
       --lib entry`: `the_outward_search_finds_the_scans_point` (3 000 generated cases: floors 600 mm …
       12 m a side anywhere in ±20 m, targets up to 5 m outside, occupancy 0 … 100 %) and
       `an_equal_distance_in_a_later_ring_wins_on_y` pass.
     M-Z6 (`reach * reach >= kept`): `an_equal_distance_in_a_later_ring_wins_on_y` FAILS — "ring 5 not
       visited", (5 150, 5 200) for (5 250, 5 000); the generated cases happen to pass. Restored.
     /tmp/s15-12d0/step.sh z6 (binary /tmp/s15-12d0/z6-mineworld): ZI-1 d7025dbc…79eaf, ZI-2
       53d017d0…95411, ZI-3 6e4c4015…c8395 — all three = E-Z-base (bodies-yard 30 days: wall 2.0 s, was
       15.8 s). ZI-4: `cargo test -p mineworld-bodies` every test passes, none edited (lib 16, actions 27,
       genesis 6, isolation 5, long_run 1, long_run_objects 1, objects 7, objects_genesis 10, rapier_pin
       1, scenarios 17). TZ-2 for SD-Z6: PASS.
     Information (not TZ-9a): social-cafe prototype, 30 days, CPU 4.45 s (base 8.95 s; without 1.73 s).

E-Z5 ZC-4 (SD-Z3 + SD-Z4), 2026-10-08, on 18d8e48 + the ZC-4 code, parked on branch
     `mvp0/s15-12d0-zc4-wip` @ c1b5749 (pushed; not on the PR branch). Code: `geometry.rs` (`Area::
     spanning`, `grown`, `meets`, `clamp`, `clear_of_segment`, `crossed_by`; `corridor_clear(…, exact)`),
     `walls.rs` (new: `walled`, the clamp, its two unit tests), `stride.rs` (the Walled route after the
     clear one, V1 … V4 applied, a failing answer going on to Rapier), `resolve.rs` (`Policy.
     exact_corridor`, `integer_walls`; `RAPIER_PATH`; `Route::Walled`; `#[doc(hidden)] pub fn shadow`
     and `Shadow`, ZR-3's comparison), `tests/walls.rs` (TZ-5), `tests/shadow.rs` (ZR-3, ignored).
     One existing unit test edited to compile (`resolve.rs`
     `without_verification_the_controller_leaves_the_pair_overlapping`: `..PRODUCTION` added to its
     struct literal; its assertion unchanged).
     fmt, clippy -D warnings clean. `cargo test -p mineworld-bodies`: every test passes (lib 22 —
       TZ-4's four, the clamp's two among them —, walls 5, and every existing file unedited, PB-5 a's
       8 010 ± 1 and PB-5 c's slide included): ZR-1 PASS.
     TZ-4 PASS; M-Z3 (the exact test replaced by the box overlap): `a_diagonal_320_mm_from_a_post_is_clear`
       FAILS. TZ-5 a–e PASS; M-Z4 (C shrunk by R): a → (8 020, 3 000), b → (8 020, 4 400), c → (8 020,
       10 020), FAIL as designed.
     Class-I check with SD-Z3 and SD-Z4 off (PRODUCTION's two flags false, temporarily; step.sh
       z34off): ZI-1 d7025dbc…, ZI-2 53d017d0…, ZI-3 6e4c4015… = E-Z-base: the plumbing moves nothing.
     **ZR-3 on long_run's 3 000 requests (`--test shadow -- --ignored`, /tmp/s15-12d0/zr3-longrun.txt):
       418 compared (363 Walled, 55 newly Clear); per-axis difference max 1 605 mm, mean 15.7 mm;
       9 above 50 mm = 2.15 % > 1 % — FAIL.** (Walled 8 of 363 = 2.2 %; newly Clear 1 of 55 = 1.8 %.)
       The nine, each with Rapier's end on the same state:
         64    p09 (7 605, 2 620) → (8 539, 3 554)  Walled (8 010, 3 554)   Rapier (7 872, 3 237)
         221   p10 (316, 1 986) → (−135, 2 437)     Walled (310, 2 437)     Rapier (316, 2 294)
         691   p04 (6 710, 2 116) → (8 056, 770)    Walled (8 010, 770)     Rapier (6 805, 2 198)
         920   p12 (4 765, 1 214) → (3 692, 141)    Walled (3 692, 310)     Rapier (3 857, 310)
         1149  p02 (5 835, 6 260) → (7 764, 6 260)  Clear  (7 764, 6 260)   Rapier (6 159, 6 256)
         1407  p14 (3 879, 10 010) → (2 535, 11 354) Walled (2 535, 10 010) Rapier (3 879, 10 010)
         1634  p08 (8 010, 3 055) → (8 540, 2 525)  Walled (8 010, 2 525)   Rapier (8 010, 2 755)
         1687  p03 (1 241, 310) → (1 665, −114)     Walled (1 665, 310)     Rapier (1 612, 310)
         1867  p14 (1 184, 310) → (137, −737)       Walled (310, 310)       Rapier (1 183, 310)
       In all nine Rapier keeps less of the stride than the integers: it slides less along the wall, and
       where the walker starts at the controller's offset (1407, 1867) or glides along it (1149, the
       counter's face at 6 260) it barely moves or does not move at all — the same behaviour E-Z1 found
       for SD-Z3 (Z-D2). The integer answers keep V1 … V4 after every request (asserted in the test).
     Not run, the stop coming first: ZR-3 on the 300-day prototype, ZR-4, the re-capture (ZR-5), ZC-5.
     Information (not TZ-9a; load average 68 … 87, so CPU itself inflated): 30 prototype days,
       interleaved z6, z34, z6, z34 → 7.40, 7.86, 7.72, 7.68 s. SD-Z3 + SD-Z4 show no CPU gain over
       SD-Z6 alone that this instrument can see under this load.

E-Z6 ZC-5 (SD-Z5), 2026-10-08, commit 31e3779 (on b987d7f: SD-Z2 + SD-Z6; SD-Z3/Z4 dropped).
     Code: `rapier.rs` `Scene::sweep_past` (people at given scene indexes left out of a contact sweep by
       a query predicate; with none left out the filter is exactly the one built before), `stride.rs`
       (`behind`: within 2R + GAP of the start, inclusive, and d · (p − start) ≤ 0; passed to the
       contact sweep only — walls sweeps never meet people), `resolve.rs` (`Policy.away_free`),
       `system.rs` VERSION 3, `tests/rapier_pin.rs` (3, "0.36.0"), `tests/actions.rs` (DO-11's pin
       renamed and flipped; a new "toward" test; a misplaced doc comment moved to the test it describes).
     Class-I check with SD-Z5 off (`away_free: false`, temporarily; step.sh z5off): ZI-1 d7025dbc…,
       ZI-2 53d017d0…, ZI-3 6e4c4015… = E-Z-base, every bodies test passing.
     With SD-Z5 on: fmt, clippy -D warnings clean; `cargo test -p mineworld-bodies` every test passes;
       the only edited tests are the named flip, the pin's literal and the new toward test (ZR-1 PASS).
       PB-5 a and c unedited and passing.
     TZ-6: `a_shove_from_600_mm_moves_its_target_half_a_metre` → b at (4 499, 4 999), stopped-short
       { by: None } — not (4 500, 5 000) without a stopped-short as TZ-6 predicted (§20.13 Z-D8);
       `a_stride_toward_a_person_600_mm_away_is_still_stopped` → a stays at (3 400, 5 000), stopped by
       b, b unmoved.
     M-Z5 (the d · (p − start) test made always true): the toward test FAILS — a at (4 369, 4 758).
       Restored.
     ZR-2: long_run's N-1 … N-4 and long_run_objects' SD-O2 after every request (both pass); bodies-yard
       30-day scan and two processes, SIGKILL and replay (`tools/cli/tests/bodies_yard*.rs`, pass in the
       full gate, E-Z7).
     ZR-5, the new bases (step.sh z5, binary /tmp/s15-12d0/z5-mineworld):
       ZI-1  E-Z-base d7025dbc…79eaf (4 091 748 bytes) → 23f7fa76016294ab18ae5b6a6b568b61d1b36fc0741ee51eb7952276a1de5125
             (4 019 632 bytes)
       ZI-2  E-Z-base 53d017d0…95411 (569 950 bytes) → c8358f8bbc06c94fbd7db33375dfe21ad0da80ddd39ee93ce9a72d542798c5b4
             (612 428 bytes)
       ZI-3  E-Z-base 6e4c4015…c8395 (62 855 facts) → bd6a10026f608dba1bb4d48f1399ccaa26e353c7c570b4190ef99039975c80e6
             (62 385 facts, faults 0)

E-Z7 ZC-6, 2026-10-08, on 31e3779 (code) — /tmp/s15-12d0/close.sh z5, gate-town.sh; binary
     /tmp/s15-12d0/z5-mineworld.
     TZ-1: validate bodies-yard 7356b8f8…, market-town 64f41086…, social-cafe ebcd60a0… — each = E-Z-base;
       social-cafe 300 days ad49c723…c64b, market-town 365b50e0…1d1d — both = E-TD0. PASS.
     ZR-4 / TZ-8: x86_64 build (`--target x86_64-apple-darwin`, own target dir), bodies-yard 30 days
       under `arch -x86_64`: summary sha bd6a1002…80e6 = arm64's. PASS (covers Class I and SD-Z5).
       Two processes, SIGKILL and replay from genesis: `bodies_yard_is_the_same_world_in_two_processes_
       and_after_sigkill` passes.
     TZ-10 / full gate: `cargo fmt --all --check` 0, `cargo clippy --workspace --all-targets -D
       warnings` 0, `cargo test --workspace` 0 (153 result lines, all ok; the ignored are the existing
       ones). Isolation, rapier_pin (3, "0.36.0"), ac1_composability, precursor_vocabulary,
       seam_vocabulary unedited and passing. Doc checks: 191 sections, none duplicated; 55 decision ids.
     **TZ-9a** (§20.5 amendment, §20.6.1's instrument; CPU = user + sys; 300 days, seed 7):
       first set (15:19 … 15:24, load 46 → 5):
         social-cafe  without 14.51, with 47.33, without 13.64, with 49.02 → pair ratios 3.262, 3.594:
                      10.2 % apart → CONTAMINATED, re-run once
         market-town  without 16.93, with 47.17, without 15.73, with 44.90 → pair ratios 2.786, 2.854
                      (2.4 % apart): max(with) ÷ min(without) = 47.17 ÷ 15.73 = **2.999 × ≤ 3.0 — PASS**
       social-cafe re-run (15:24 … 15:26, load 7 → 5): without 12.60, with 45.02, without 13.29, with
         45.40 → pair ratios 3.573, 3.416 (4.6 % apart, clean): max ÷ min = 45.40 ÷ 12.60 = **3.60 × >
         3.0 — FAIL**, both pairs above 3.0.
       Faults 0 in all twelve; with bodies 523 956 facts (social-cafe), 526 120 (market-town), each the
       same in every run; without, 361 979 and 368 608 (= E-Z-before's). Against the "before" (E-Z-before:
       7.98 ×, ≥ 6.02 ×) the cost has halved: with bodies ≈ 45–49 s, was ≈ 105–114 s.
     **TZ-9b**: a scratch copy of 31e3779 (/tmp/s15-12d0/tz9bsrc, never committed) timing every call of
       bodies' `ArrivalResolver::resolve` (Instant, one stderr line per call), market-town prototype,
       300 days, seed 7: faults 0, 526 120 facts, fingerprint 47753fa6… = the gate's (the timing changes
       nothing). 244 165 calls: p50 55 µs, p90 227 µs, p99 332 µs, p99.9 540 µs, max 10.54 ms.
       **max ≤ 50 ms — PASS.** (Load average ≈ 5 during the run.)
     §20.7's ladder not tried: the ruling relayed with option (c) says a clear TZ-9a failure stops.

The prototype (E-TD0b), stated so it can be rebuilt:
  1  GIT_INDEX_FILE=/tmp/s15-12d0/proto.idx git --work-tree=/tmp/s15-12d0/proto checkout 21f96ff -- \
       worlds/social-cafe worlds/market-town
  2  social-cafe: world.yaml gains `- bodies` after `- schedule`, and
       items: [cafe-ball, cafe-box, street-ball, street-box]
     market-town: world.yaml gains `- bodies` after `- schedule` (before `- item`), and the four keys
       in its items list; the four item files and the six place files copied from social-cafe's
  3  the place files' `passages` and `body:` sections, and the item files, exactly as §19.3.1 states
     them, with placeholder sizes where the slice uses put_solid (the café's tables half 300; the
     florist's shelves (0..440, ±400), its shelf units (±450, 6 990..7 350), its table half 450, its crate
     half 300; on the street: planter boxes 1 000 × 500, pots 400 × 400, A-boards 600 × 400, the bicycle
     half 350, benches 1 900 × 700), and on the street exactly the 59 solids listed below (the
     prototype's rounding of §19.3.1: lamps half 90, bollards 110, bins 170)
  4  the copies without bodies: `- bodies` removed, every `body:` section removed (places' and items')
  5  `cargo run -p mineworld-cli -- validate <copy>` → valid (social-cafe: 22 entities, 67 genesis facts)
```

The prototype's street, `body:` (floor, then solids as `min-x min-y max-x max-y height`, in order):

```text
floor  -37450 -13000 30550 3200
15050 1000 26050 3200 1900 | -27540 -690 -27360 -510 2400 | -11540 -690 -11360 -510 2400
5460 -690 5640 -510 2400 | 21460 -690 21640 -510 2400 | -20540 -9290 -20360 -9110 2400
-2540 -9290 -2360 -9110 2400 | 14460 -9290 14640 -9110 2400 | -5560 -1090 -5340 -870 1000
-3210 -1090 -2990 -870 1000 | -860 -1090 -640 -870 1000 | 1490 -1090 1710 -870 1000
3840 -1090 4060 -870 1000 | 6190 -1090 6410 -870 1000 | 8540 -1090 8760 -870 1000
10890 -1090 11110 -870 1000 | 13240 -1090 13460 -870 1000 | 15590 -1090 15810 -870 1000
-22560 -8930 -22340 -8710 1000 | -20160 -8930 -19940 -8710 1000 | 4040 -8930 4260 -8710 1000
-20 -1020 320 -680 900 | -25020 -1020 -24680 -680 900 | -31110 -110 -30790 210 2200
-24120 -120 -23780 220 2200 | -16130 -130 -15770 230 2200 | 16890 -110 17210 210 2200
23380 -120 23720 220 2200 | -32600 -10000 -32300 -9700 2200 | -25610 -10010 -25290 -9690 2200
-16620 -10020 -16280 -9680 2200 | -8630 -10030 -8270 -9670 2200 | -100 -10000 200 -9700 2200
8390 -10010 8710 -9690 2200 | 17380 -10020 17720 -9680 2200 | -10425 -955 -8875 -5 600
-3825 -955 -2275 -5 600 | 10375 -955 11925 -5 600 | 17975 -955 19525 -5 600
-28625 -9770 -27275 -8870 600 | -14125 -9770 -12775 -8870 600 | 2375 -9770 3725 -8870 600
530 1000 1770 1800 850 | 750 780 1550 2020 850 | 2580 1000 3820 1800 850 | 2800 780 3600 2020 850
4630 1000 5870 1800 850 | 4850 780 5650 2020 850 | -2900 2200 -1900 2700 600 | 6800 2200 7800 2700 600
-1350 2520 -950 2920 500 | 970 2520 1370 2920 500 | 5050 2150 5450 2550 500 | -1400 1450 -800 1850 1000
-30950 1200 -30350 1600 1000 | -19400 -150 -17500 550 840 | -7400 -10350 -5500 -9650 840
-4450 -550 -4250 -350 2200 | 8600 1800 9300 2500 1100
```

The other five places' prototype bodies are §19.3.1's, with the placeholders of step 3; the café's
tables are (4 560, 2 860)–(5 160, 3 460), (6 810, 4 260)–(7 410, 4 860), (1 960, 8 260)–(2 560, 8 860),
each h 750; the store's put_solid placeholders are (0, 2 150)–(440, 2 950) and (0, 3 400)–(440, 4 200)
h 1 800, (4 900, 6 990)–(5 800, 7 350) and (6 200, 6 990)–(7 100, 7 350) h 1 800, (3 600, 2 900)–(4 500,
3 800) h 750, (7 100, 300)–(7 700, 900) h 600.

## 20.13 Deviations and discoveries during implementation

**Z-D1 — Branch and worktree (bounded).** §20.11 proposed `mvp0/pr-12d0-cost` in
`/Users/yuema137/mineworld-worktrees/s15-12d0`, based on main after the design PR merged. The
coordinator's kickoff (2026-10-08) names `mvp0/s15-12d0-plan` in `/Users/yuema137/mineworld-worktrees/
impl-12d0`, which already holds the frozen design (953ff10) on main @ f842c52. The kickoff governs: the
PR opened from this branch carries the design commits and the implementation together. Code base
identical (953ff10 differs from f842c52 in Markdown only); no evidence affected.

**Z-D2 — MATERIAL STOP: SD-Z3 (Class I) is not result-preserving (E-Z1).**

```text
Previous assumption:
  §20.3 SD-Z3: a corridor exactly clear by R + GAP is one Rapier sweeps to `to`, the snap returning
  `to` — the fast path's answer — so an exact corridor test changes no result (Class I).
Audit evidence:
  E-Z1. With SD-Z3, ZI-1 and ZI-2 move (ZI-3 does not); SD-Z2 alone keeps all three. First differing
  request: long_run's 307 — a stride along the counter at exactly 310 mm, which Rapier ends 49 mm off
  the face. With a strict 311 mm margin ZI-2 holds but ZI-1 still moves, at request 518: a stride in
  open floor, 382 mm from the counter, that Rapier ends 2 mm east and 1 mm short.
Corrected understanding:
  Rapier's character controller is not exact on clear strides that start at, or glide along, its
  offset from a face; the existing clear path already answers such strides differently from what
  Rapier would. Every stride SD-Z3 moves from Rapier to the integers is one whose answer may change.
  No margin makes SD-Z3 byte-identical.
Implementation consequence:
  SD-Z3 reverted (geometry.rs and its TZ-4 unit tests). SD-Z2 kept: byte-identical (E-Z1).
Validation consequence:
  TZ-4 and M-Z3 cannot be met as Class I. Per §20.4 the primary session decides: drop SD-Z3, or move
  it to Class R under ZR (its newly clear strides then join ZR-3's shadow comparison, compared with
  Rapier's answer on the same state).
```

Stop reported to the coordinator with this record. Work on SD-Z1, SD-Z4 and SD-Z5 not started:
the kickoff makes this a material stop and says not to work around it.

**Ruling (primary session, 2026-10-08, relayed by the coordinator): option (b).** SD-Z3 is Class R;
its newly clear strides join ZR-3's shadow comparison, bound unchanged; the re-capture rule applies
once, with SD-Z4; SD-Z2 stays Class I (PASS). Recorded as dated amendments to §20.3 and §20.4. Same
message: TZ-9's instrument under load fixed before any TZ-9 run (§20.6.1). SD-Z3 is re-applied in ZC-4,
behind `Policy.exact_corridor`, so that the Class-I identity check (all three Class-R pieces off) stays
runnable.

**Z-D3 — The ZC-1 profile's instrument (bounded).** §20.6 ZC-1 asks for the share of strides by Route
and the scene's mean collider count over 30 prototype days, "a crate-private counter read by an
ignored test". The prototype runs through the CLI binary; a test of this crate cannot run a World Pack
from disk, and a counter in the production code would be a static the resolver keeps (I-5's spirit).
Instead: a scratch copy of the source exported outside the repository (as the prototype is), with
counters printed at exit, built into its own target directory; nothing of it is committed. Information
only, as the design states.

**Z-D4 — MATERIAL STOP: SD-Z1 (Class I) is not result-preserving (E-Z2).**

```text
Previous assumption:
  §20.3 SD-Z1: every collider left out of the reach box is one no query of the resolution can touch,
  since a shape cast's time of impact against a collider does not depend on other colliders; only the
  order of equal hits could differ (Class I).
Audit evidence:
  E-Z2. With REACH_MARGIN 2 200, ZI-1, ZI-2 and ZI-3 all move. With 1 000 000 all three equal E-Z-base
  (the subset scene and its index maps are right); with 10 000 too, on those runs. First differing
  stride: long_run's 419th — p07's 969 mm stride ends 14 mm apart on each axis, with the counter and
  ten people, none within 1 400 mm of what the stride touched, left out. Leaving out only far people,
  or only far solids, each moves a result (stride 569, 1 mm).
Corrected understanding:
  Rapier 0.36.0's character controller, as this pack uses it, is not local: its result depends on
  colliders far from anything it touches (by 1 … 14 mm here), most likely through the query
  pipeline's BVH. No reach margin can be shown byte-identical. And, information only: SD-Z1 does not
  reduce the prototype's CPU time measurably (30 days: 8.95 s base, 9.17 s with SD-Z1; 1.73 s without
  bodies) — the scene's size is not where bodies' cost lies.
Implementation consequence:
  SD-Z1 reverted (patch kept in /tmp, sha-256 13dad5e1…f0e31). ZC-4 … ZC-6 not started: SD-Z1 is
  Class I and the kickoff makes its failure a material stop.
Validation consequence:
  TZ-2's M-Z1 and M-Z2, and TZ-3, cannot be met as Class I. Per §20.4 the primary session decides:
  drop SD-Z1 (the measurement says it buys nothing), or move it to Class R under ZR. A further
  question the cost reading raises for the operator: with the scene's size ruled out, TZ-9 rests on
  SD-Z3 and SD-Z4 (taking strides away from Rapier altogether) and on §20.7's ladder; the ZC-1 profile
  (Z-D3), still pending, is what would show where the ≈ 7 s per 30 days goes before more is built.
```

**Z-D6 — MATERIAL STOP: the profile says TZ-9 cannot pass with §20's pieces and ladder (E-Z3).**

```text
Previous assumption:
  §20.2 F-Z1 … F-Z5, DB-10: bodies' cost is Rapier's — scenes too large (SD-Z1), Rapier consulted where
  integers answer (SD-Z3, SD-Z4) — so taking strides away from Rapier brings the towns within 1.5 ×.
Audit evidence:
  E-Z3, sampling profiles of 30 and 300 prototype days. Of the with-bodies run's main thread, 49 % is
  entry's E3 `nearest_free` (an integer scan of the whole floor's 50 mm lattice, every point checked
  against every solid), 24 % is stride (Rapier), 25 % everything else, of which about half is the
  world without bodies. Rapier, removed entirely, leaves ≈ 5.8 × (bound 1.5 ×).
Corrected understanding:
  Rapier is not where most of the cost lies: E3's placement search is, and then the per-fact cost of
  ≈ 45 % more facts. SD-Z3, SD-Z4, L1 and L2 cannot reach the bound however well they work.
Implementation consequence:
  ZC-4 … ZC-6 not started, as ruled ("do not implement into a known failure"). No code changed.
Validation consequence:
  TZ-9 cannot pass within §20. For the operator: a candidate outside §20's scope is an exact E3 — the
  same nearest free lattice point, found by searching outward from `target` instead of scanning the
  floor (Class I by construction: the same minimum under the same (distance², y, x) order). By E-Z3
  even that plus every Rapier stride at zero cost leaves ≈ 2.0 ×, so the remaining ≈ 15 s of bodies'
  other cost (more facts) is also in question — whether QB-11's 1.5 × is reachable at all on this
  prototype is the operator's question, with these numbers.
```

**Ruling on Z-D6 (operator, 2026-10-08, relayed by the coordinator): "fix the scan, and switch to an
absolute budget".** SD-Z6 added as Class I (§20.3 amendment); TZ-9 re-scoped to TZ-9a (3.0 ×) and TZ-9b
(50 ms), the 1.5 × kept as superseded (§20.5 amendment). SD-Z6 done, Class I PASS (E-Z4).

**Z-D7 — MATERIAL STOP: ZR-3 fails for SD-Z3 + SD-Z4 (E-Z5).**

```text
Previous assumption:
  §20.4 ZR-3 (and its amendment): the integer answers of SD-Z3 and SD-Z4 differ from Rapier's by more
  than 50 mm on an axis on at most 1 % of the requests they answer (QZ-1 accepted that the slide keeps
  all tangential motion where Rapier's controller "may" keep less).
Audit evidence:
  E-Z5. Over long_run's 3 000 requests: 418 answered by integers (363 Walled, 55 newly Clear), 9 of them
  more than 50 mm from Rapier's answer on the same state — 2.15 %. In every one Rapier keeps less: it
  slides less, and from or along its own offset it sticks (no motion at all in two, 1 605 mm less in
  one). ZR-1 passes (every test, PB-5 a and c), V1 … V4 hold, TZ-4, TZ-5 and both mutations behave;
  with both pieces off ZI-1 … ZI-3 are E-Z-base's.
Corrected understanding:
  The difference is not a defect of the integers: it is Rapier's controller sticking at its offset
  (E-Z1's finding, again). But the frozen bound measures exactly that difference, and it is exceeded.
Implementation consequence:
  The ZC-4 code is parked on mvp0/s15-12d0-zc4-wip @ c1b5749, not on the PR branch. No re-capture.
  ZC-5, ZC-6 and the gate not started.
Validation consequence:
  For the operator: (a) widen ZR-3's bound (its 1 % / 50 mm was set before E-Z1 and E-Z5 showed what
  Rapier does at its offset); (b) narrow SD-Z4 so that it does not answer strides that start at or
  glide along C's edge (likely to bring most of the nine under 50 mm, at the cost of sending those
  strides back to Rapier); or (c) drop SD-Z3 and SD-Z4 and measure TZ-9a with SD-Z6 alone (+ SD-Z5),
  since under the present load E-Z5 sees no CPU gain from them. Either way TZ-9a is not yet measured.
```

**Ruling on Z-D7 (primary session, 2026-10-08): option (c).** SD-Z3 and SD-Z4 dropped (§20.3, §20.4
amendments); SD-Z5 done (E-Z6); ZC-6 run (E-Z7).

**Z-D8 — TZ-6's literal (bounded).** TZ-6 predicted that with SD-Z5 the 600 mm shove leaves b at
(4 500, 5 000) with no stopped-short. It leaves b at (4 499, 4 999) with `stopped-short { by: None }`:
the shover is no longer in the sweep (the fix holds — 499 mm, not 301, and not stopped by a), but
Rapier's controller drifts 1 mm on each axis in open floor, beyond the 1 mm snap (E-Z1's request 518
showed the same drift). The test pins the measured point and asserts "not stopped by a"; the
prediction, not the rule, was wrong. No scope, invariant or contract moves.

**Z-D9 — MATERIAL STOP: TZ-9a fails on social-cafe (E-Z7).**

```text
Previous assumption:
  §20.5's amendment: with SD-Z6 (and the pieces then in scope) both town prototypes cost at most
  3.0 × their copies without bodies.
Audit evidence:
  E-Z7. market-town 2.999 × (PASS, by a hair). social-cafe: first set contaminated (pair ratios 10.2 %
  apart), re-run clean: 3.573 and 3.416, max ÷ min 3.60 × — both pairs above 3.0. TZ-9b passes (max
  10.5 ms, p99 0.33 ms). Every other gate passes (TZ-1, TZ-2, TZ-6, TZ-8, TZ-10, ZR-1, ZR-2, ZR-4, ZR-5).
Corrected understanding:
  SD-Z6 halves bodies' cost (≈ 110 s → ≈ 46 s of CPU per 300 days) but the social-cafe prototype's
  copy without bodies is cheaper than market-town's (12.6 … 14.5 s against 15.7 … 16.9 s) while the
  with-bodies cost is about the same, so its ratio is higher.
Implementation consequence:
  None further: the ruling says a clear TZ-9a failure stops. §20.7's ladder (L1, L2) not tried: L1
  ("B = W when nobody is in reach") rests on SD-Z1's reach, which E-Z2 showed Rapier's non-locality
  makes not byte-identical; L2 is an integer nudge, of the kind E-Z5 showed differs from Rapier at its
  offset. The PR is not opened as READY.
Validation consequence:
  For the operator, with the numbers above: accept 3.60 × on social-cafe, try the ladder, or a further
  piece. What remains (E-Z7's addendum, a 30-day sample of the head's build, social-cafe prototype,
  /tmp/s15-12d0/profile/z5-sample.txt): main thread 3 345 samples; bodies' resolve 1 724 (51.5 %), of
  which stride (Rapier's sweeps and scene builds) ≈ 1 700 and E3's search ≈ 1 — SD-Z6 removed it. On
  300 days that puts ≈ 20 s of social-cafe's ≈ 33 s of bodies' CPU in Rapier strides and ≈ 12 s in the
  rest (more facts). Passing 3.0 × (≤ 37.8 s) needs ≈ 7.6 s less: about 37 % of the Rapier strides'
  cost. L1 (skipping the contact sweep, one of each attempt's two sweeps, when nobody is near) is the
  size of cut that could reach it, but would have to be Class R (E-Z2's non-locality).
```

**Ruling on Z-D9 and Z-D8 (operator, 2026-10-08, relayed by the coordinator): accept 3.60 × on
social-cafe and open the PR; Z-D8 accepted.** Recorded in §20.5's amendment: TZ-9a accepted, not passed;
L1 (Class R) the next candidate for cost. The branch takes main (2690c1b, CI) before the PR opens.

**Z-D5 — The resumed session (bounded).** The previous implementation session ended (API rate limit)
with an uncommitted tree: the two §20 amendments, §20.6.1 and the Z-D2 ruling (kept, committed with
this record); SD-Z1 (`reach.rs`, `tests/cull.rs`, `stride.rs`, `entry.rs`, `footprint.rs`), and the
first pieces of ZC-4 (`geometry.rs`: `spanning`, `grown`, `meets`, `clamp`, SD-Z3's
`clear_of_segment` and `crossed_by` behind a new `exact` argument; `resolve.rs`: `Policy`'s
`exact_corridor`, `integer_walls`, `away_free`, `RAPIER_PATH`, `Route::Walled`) — not compiling
(`corridor_clear`'s one caller not updated). The resumed session finished SD-Z1 to compile with the
Class-R pieces off (the unused stubs `integer_walls`, `away_free`, `RAPIER_PATH`, `Route::Walled` and
`clamp` removed; `exact_corridor` kept, false), measured it (E-Z2), and reverted all code with it; SD-Z3's
geometry and `exact_corridor` are in the same kept patch. Its background "before" run had ended with the session, 7 of 8 runs done (E-Z-before).

## 20.14 Operator review and merge (post-merge record, planning session)

```text
PR            GitHub #84, "S15 PR 12d-0 — bodies' cost (SD-Z2, SD-Z6, SD-Z5)"
PR head       995d5569f283a7db98c930bf4a3acd7fa04e564a (merge of origin/main 2690c1b, CI, into the
              branch; the only conflict docs/MVP_STATUS.md's S13–S15 rows)
CI            run 37853993552 (pull_request) on that exact head: `fast` pass (1 m 32 s), `test` pass
              (15 m 37 s), `image` skipped — the checks main's branch protection requires
final executable head  31e3779 (SD-Z5); later commits are documents and the merge of main
merged        78ca5ae, a merge commit, 2026-10-08, by the operator
review        the coordinator's, relayed: the scope is clean (only systems/bodies/ and the named
              documents), CI green on the exact head; an independent mutation — the `− m` margin
              dropped from SD-Z6's ring stop bound — was caught by
              `the_outward_search_finds_the_scans_point`
```

**What landed.** SD-Z2 (a people scene builds only Rapier's broad phase; Class I), SD-Z6 (entry E3's
nearest free point searched outward; Class I), SD-Z5 (a stride away from a person within the
controller's offset is not stopped by them; Class R; `bodies` version 3). `ARC-39` note 3 and a `DEP-13`
note record them.

**What was dropped, and why (so it is not tried again unmeasured).** SD-Z1 (a scene of only what a
stride can reach), SD-Z3 (an exact integer corridor) and SD-Z4 (integer wall strides): Rapier 0.36.0's
character controller is neither local (E-Z2) nor exact at its own offset (E-Z1, E-Z5), so each changed
results, and none bought measurable CPU. Their code stays on `mvp0/s15-12d0-zc4-wip` (c1b5749),
unmerged, and in `/tmp/s15-12d0/sd-z1-reverted.patch`, for reference only.

**The bases 12d and PL-b read now (ZR-5, E-Z6):**

```text
ZI-1  long_run.rs's second-process bytes      23f7fa76016294ab18ae5b6a6b568b61d1b36fc0741ee51eb7952276a1de5125
                                              (4 019 632 bytes; was d7025dbc…79eaf)
ZI-2  long_run_objects.rs's bytes             c8358f8bbc06c94fbd7db33375dfe21ad0da80ddd39ee93ce9a72d542798c5b4
                                              (612 428 bytes; was 53d017d0…95411)
ZI-3  bodies-yard 30 days, seed 7, summary    bd6a10026f608dba1bb4d48f1399ccaa26e353c7c570b4190ef99039975c80e6
      sha-256 (every line but `wall`)         (62 385 facts, faults 0; arm64 = x86_64 under Rosetta;
                                              was 6e4c4015…c8395)
towns social-cafe ad49c723…c64b, market-town 365b50e0…1d1d — unchanged by 12d-0 (TZ-1)
```

**The cost record 12d inherits (TZ-9, in order).** QB-11's 1.5 × (frozen, then SUPERSEDED) → the
operator's re-scope after E-Z3 to TZ-9a ≤ 3.0 × and TZ-9b ≤ 50 ms → measured on E-TD0b's prototype at
31e3779 (E-Z7): market-town 2.999 × (pass), social-cafe 3.60 × (clean re-run; **accepted by the
operator, not passed**), ≈ 45–49 s of CPU with bodies per 300 days against ≈ 105–114 s before 12d-0;
per-resolution time p50 55 µs, p99 0.33 ms, max 10.5 ms. If cost ever matters again, the next candidate
is §20.7's L1 as Class R (E-Z7's addendum sizes it).

**Follow-ups carried.** FU-12c-1 is closed by SD-Z5. F-B7 and FU-12a-1 stay with 12d. The ZC-1 route
counters (Z-D3) were replaced by sampling profiles (E-Z3, E-Z7's addendum); no counter exists in code.

---

# 21. PR 12n — navigation: "go to X", routed round walls and furniture (full design; DESIGN FROZEN 2026-10-09)

## 21.0 DESIGN FROZEN

```text
Design revision        §21 revision 1 (plan/s15-12n @ f64cf01) plus this freeze record; frozen sections
                       §21 header … §21.13; live sections §21.12/§21.13 checkboxes, §21.15
Approved by / evidence primary session, 2026-10-09, relayed by the coordinator: "Revision 1 is accepted,
                       and R-12n-1 is resolved … 12n (§21) is DESIGN FROZEN 2026-10-09 (primary session)."
Implementation base    main at the session's start (≥ ec38570), with PR #100 merged; re-audit §21.2 if a
                       listed path moved
Execution contract     §21.14
Lifecycle              FROZEN — implementation in a fresh session in impl-12n
```

**Rulings at the freeze (primary session, 2026-10-09):**
- **QN-12 — client-paced `walk-step`s, routing on the server.** A primary-session ruling, not a new
  operator decision: it follows from rulings the operator already made — step-19 §4.1 defines embodied
  time as the cadence at which embodied inputs arrive (QTW-13), and clients only report intent (`CLAUDE.md`
  §4 rule 15; `ENGINEERING_RULES.md` §8).
- **QN-13 — accepted:** `run` refuses packs with 30 or more seats (SD-N15).
- Earlier rulings (QN-2, QN-4 … QN-11) stand as recorded in §21.10.

**Binding note N-1 (2026-10-09, relayed by the coordinator from the MVP-1 regions/travel design, S21
§10, N-1 — a non-preclusion requirement, part of the frozen design):**
1. **An unreachable place is refused with one stable code, `no-route`.** A `walk-to` whose destination
   place has no passage chain from the walker's place is refused `Rejection::System { code: "no-route" }`
   — the same code as a goal the wayfinder answers `Unreachable` — never `TooFarAway`. The code string is
   part of movement's public vocabulary: stated in `ARC-W` and `MODULE_SPEC.md` §4.1, held by a test, and
   never renamed. `TooFarAway` remains only for a `Person` destination outside the walker's place.
   (SD-N6 and NV-2 (e) amended accordingly.)
2. **`Destination` stays open to extension.** It is declared `#[non_exhaustive]` and serialized with an
   explicit tag per arm, so a later pack-facing arm (a region, a remote place reached by the future travel
   system, an object) is an addition, not a breaking change to `walk-to`'s payload or to any `match` in
   another crate. Nothing in 12n assumes the two arms are all there will be.

**Lifecycle:** DESIGN FROZEN (2026-10-09, primary session), revision 1. Drafted by the planning session on
`plan/s15-12n` from `main @ ec38570`. Record ids (`ARC-W`, `DEP-P`) are placeholders that the implementing
session replaces with the next free `ARC-` / `DEP-` numbers in NV-C1, recording the mapping in §21.15.
Criterion and decision ids (`NV-n`, `NW-n`, `SD-Nn`, `QN-n`) are final.

**Revision 1 (2026-10-09), after the primary session's review of #100.**
- **R-12n-1 (blocking), applied.** Revision 0 stepped the walk with a movement-owned Process woken once
  per *simulated* second. Under a hosted world at the default 12× that is ≈ 16 m/s on screen, which the
  operator's two-time-domain ruling forbids (`step-19-time-weather.md` §4.1–4.4; QTW-13; S11-B's D-SB6):
  embodied pace is wall-clock, calendar time alone scales, no rule reads the scale, and §4.2 option D
  ("embodied actions take world durations") is rejected. Revision 1 is the hybrid the review recommends:
  the **route** is server-side and authoritative (movement owns `walk-to`, plans through `Wayfinder`,
  records and discloses the route as the walker's `Walking` component — no Process, no world duration);
  the **strides** are embodied inputs, one `walk-step` request per stride, sent at the body's cadence: a
  player's client at human speed, a hosted controller through S11-B's wall-second cadence, headless `run`
  at a notional cadence. Changed in place: the answer in brief, §21.2 F-N6, §21.3, §21.5 (constants,
  SD-N1 … SD-N2, SD-N8 … SD-N10, SD-N12 … SD-N15), §21.7, §21.8 (NV-2, NV-4, NW-1 … NW-9), §21.9 … §21.13.
- **Rulings received (primary session, no operator needed):** QN-2 — merge 12n-2 immediately before 12d,
  recorded as one re-baseline event; QN-4 — adopt `pathfinding` 4.16.0 pinned exactly; QN-5 — the TD-D8
  derivations and restated invariants accepted, failing at R 250 is a material stop; QN-6, QN-7, QN-9,
  QN-10, QN-11 as recommended; QN-8 — 13d's client-side planning withdrawn in favour of `walk-to`,
  recorded as a cross-lane change to S12 (the primary session amends S12); QN-1 and QN-3 decided by this
  revision.

**Why this PR exists (operator ruling, 2026-10-08, relayed by the coordinator):** "add pathfinding; 12d
waits for it." 12d (§19, implemented on `mvp0/pr-12d-towns` in `impl-12d`, paused at `8814aad`) put walls
and furniture into the two towns and hit **TD-D7** (§19.13 on that branch): the paced controller walks
straight at its targets and does not go round anything, so 62 % of moves stop short, place entries
halve, 10 of 12 people miss more than 10 % of their agenda, the café's street door is reachable only
through a ≈ 1.1 m gap between an A-board and the terrace tables, and the wasted strides blow TD-12's CPU
budget. §19.4's remedy ladder (c1, c1 + c2) did not help (E-TD6). The operator also asked for realistic
defaults: people who get stuck at doors are not realistic. The coordinator added **TD-D8** (the two
radius-dependent bodies invariants) to this PR's scope (§21.6).

**The answer in brief.**

```text
Who says "go to X"   anybody, as an intent: `walk-to { to: <a Location> | <a Person> }` — a new action
                     of `movement`. The paced controller, the 2D client's "Walk to <name>" and floor
                     click, a future 3D click-to-walk and a language-model controller all submit the
                     same request. No client and no controller computes a route.
Who holds the walk   `movement`: `walk-to` plans the route and records it as the walker's `Walking`
                     component (destination, waypoints, progress) — world state, disclosed, persisted.
                     No Process and no world duration: a walk takes no calendar time (step-19 §4.2,
                     option D rejected).
Who paces it         the body: each stride is one `walk-step` request ("the next stride of my walk",
                     no payload), sent at the embodied cadence — a player's client once per wall second
                     at human speed, a hosted controller through S11-B's wall-second cadence (QTW-13),
                     headless `run` at a notional cadence. Movement takes at most WALK_STRIDE
                     (1 340 mm) along the route per request, checks it by the `move` rule and states it
                     through presence's `arrivals()` (ARC-26, ARC-39). One stride per wall second is
                     1.34 m/s on screen at any time scale; no rule reads the scale.
Who plans the route  whoever owns the place's geometry, through a catalog `movement` owns —
                     `Wayfinder`, an ARC-62 extension line, exactly like presence's ArrivalResolver.
                     `bodies` implements it: an integer visibility graph over its PlaceShape solids and
                     loose objects, grown by the body's radius, searched with the `pathfinding`
                     crate's A*. A world without bodies has no wayfinder for any place, and a walk
                     goes straight — byte-identically to what straight strides would do.
Who decides          unchanged: movement decides each stride is a legal stride, bodies' resolver
                     resolves it (people nudged, walls stop), presence records it. A route is a plan,
                     never a permission.
Kernel, contracts    unchanged. Presence unchanged.
```

**Split.** Two PRs (§21.7): **12n-1, the walk** (framework; byte-identical facts for every existing
world, because nothing yet asks for a walk) and **12n-2, people walk there** (the paced controller asks
for walks and the hosts pace their strides; the towns' digests move, so 12n-2 merges immediately before
12d and the two are recorded as one re-baseline event — QN-2, ruled).

## 21.1 Identity, base, approved scope (proposed)

```text
PRs           12n-1 — the walk; 12n-2 — people walk there (S15; inserted before 12d's resumption)
base          main @ ec38570 plus this design's docs PR (Markdown only). Re-audit §21.2 if anything under
              systems/movement, systems/bodies, systems/presence, systems/installed, cognition/
              rule-controller, kernel/src/{process,view,system}.rs, tools/cli/src/run* or
              tools/cli/tests/{routines,run,run_restart}.rs moves before the freeze
branches      mvp0/pr-12n1-walk, mvp0/pr-12n2-walkers (each in its own worktree, one session each)
depends on    12a … 12d-0 merged (bodies v3, ARC-39 notes 1–3, ARC-62 / IL-a merged). 12n-2's
              acceptance is measured on a scratch merge with 12d's WIP (mvp0/pr-12d-towns @ 8814aad
              or its successor), never committed there
merge         merge commits, never squash; 12n-2 per QN-2
```

**Goal.** Every person can be sent somewhere by one request and gets there round the world's walls,
furniture and objects, decided on the server, on integers, reproducibly; the paced controller uses it,
so the towns with 12d's geometry live again (≥ 90 % of agenda segments), the café door is reachable,
almost no stride stops short, and the run is cheaper, not dearer.

**Non-goals.**
- Travel between places that do not adjoin by a passage chain of the same world (between towns): the
  later travel system (`ARC-26`, "Room for travel").
- Crowd simulation, steering, velocity obstacles, local avoidance of moving people. People are not
  planned round, except the one person who just stopped the walker (SD-N9); nudging and the head-on bias
  (12b) remain how people pass each other.
- Overhead geometry, stairs, slopes, multiple floors (`body:` has none, FU-12d-1).
- Any client change (2D, 3D). Cross-lane impacts are listed (§21.9); clients adopt in their own PRs.
- Changing `move`. A `move` stays a single stride, exactly as today; WASD and reported strides keep it.
- A speed model for `move` (`ARC-26` L-1) — the walk has a pace; `move` still has none.

## 21.2 Source audit (`main @ ec38570`, 2026-10-09)

Every finding was read in this session from the file named. 12d's WIP was read on
`mvp0/pr-12d-towns` (`impl-12d`, read-only) at `df6b4e5` and `8814aad`.

| ID | Finding | Evidence | Consequence for 12n |
| --- | --- | --- | --- |
| **F-N1** | **`move` is one stride, checked and stated by movement; nothing in the world plans a path.** `Move { to }`; `validate` → `reachable`: same place within `MAX_STRIDE` (2 000 mm), or through a passage within a stride of the doorway on both sides; `resolve` states presence's facts through `arrivals()`. `MAX_STRIDE`'s doc: "Nor does it see walls … (DD-7, L-2)". `Move`'s doc already names "a travel `Process` (not built)" as distinct from it. | `systems/movement/src/action.rs:11–41`, `system.rs:77–118, 236–254` | The walk stays in movement, which reuses its private `reachable` for every `walk-step` (SD-N2). *(Revision 1: not the travel Process — that stays the future long-distance system, F-N6.)* |
| **F-N2** | **The paced controller aims straight and holds no state.** `through` strides at the doorway point; `approach` strides toward a person, stopping 1 000 mm short; `wander` draws ±1 400 mm per axis; `head_for` takes the door whose `to` is the agenda's place. Each consult proposes one stride; seats are consulted every 900 s (`ARC-27` note). It is a pure function of the observation. | `cognition/rule-controller/src/paced.rs:63–73, 228–282, 333–362`; `DECISIONS.md` `ARC-27` | A route cannot live in the controller between consults, and recomputing one per consult from disclosed geometry would put a planner in every controller and every client (rejected, §21.4). A walk is world state, held by the world. A 20 m trip today takes ten consults — 2½ simulated hours. |
| **F-N3** | **The reactive `RuleController` never walks.** It answers whoever spoke. | `cognition/rule-controller/src/lib.rs` (no `Move`) | 12n-2 changes the paced controller only. The reactive one and future LM controllers get walking as an ordinary action. |
| **F-N4** | **Bodies owns the geometry and already discloses it.** `PlaceShape { floor, solids ≤ 64 }` (axis-aligned rectangles in integer mm) and `LooseObjects` are bodies' components, disclosed to whoever perceives the place. | `systems/bodies/src/component.rs:96–140`; `system.rs:302–326` | Nothing new needs to be disclosed for planning on the server. Disclosure is not the planner's input: the planner reads bodies' state inside bodies (SD-N4). |
| **F-N5** | **The extension-catalog mechanism exists for exactly this shape of seam.** `ARC-62`: a pack-owned trait, implemented by other packs, one `extension` line in `systems/installed`; write-once, process-wide, pure, inert where the implementing pack's state is absent. Presence's `ArrivalResolver` is the first line. | `DECISIONS.md` `ARC-62` items 1–6, `ARC-39` item 5; `systems/installed/src/lib.rs:36` | Movement owns a second catalog, `Wayfinder` (SD-N3): a third line in `installed!`, nothing in the SDK or the loader. Movement never names bodies. |
| **F-N6** | **A Process is stored state woken in calendar time — the wrong domain for walking.** `start_process`, `wake` at a `WorldTime`; facts from `wake` are `Causation::Process`. Calendar time is what the host's time scale multiplies. | `kernel/src/process.rs:1–40`; `kernel/src/system.rs:571–612`; `step-19-time-weather.md` §4.1–4.2 | *(Revision 1.)* Revision 0 woke the walk each simulated second; at 12× that is ≈ 16 m/s on screen (R-12n-1). The walk is a component, not a Process; strides are requests (SD-N2). |
| **F-N13** | **Embodied pace is the cadence of requests, and hosted cadence is already wall time.** S19 §4.1: embodied time "exists only as the cadence at which embodied inputs arrive at the server"; §4.2 rejects option C (a pack reads the scale) and option D (embodied actions take world durations); §4.4 / QTW-13: a hosted controller's cadence is wall seconds, stated to the world-time seam as `cadence × scale`. Applied in S11-B (D-SB6): `PacedSeat` consults seat `k` at `genesis + (k + m·pace)·scale`, `ReactiveSeat` every wall second; guard `cadence_is_wall_time_whatever_the_scale`. §4.6: headless `run` has no scale; its pace is notional. | `step-19-time-weather.md` §4.1–4.6, QTW-13; `step-12-server.md` §16 D-SB6; `tools/cli/src/hosted.rs:1–30, 137–195`; `tools/cli/src/run.rs:41–117` | The stride cadence lives where cadence already lives: the hosts' adapters and the clients — never in a pack. `PacedSeat` gains wall-second step consults while its person walks (SD-N14); `run` gains notional step consults (SD-N15). |
| **F-N14** | **The paced controller's answer-once rule needs its consults exactly one pace apart.** It answers a line heard in `(t − pace, t]`; extra consults through `decide` would answer twice or never. | `paced.rs:18–24, 201–218`; `ARC-27` item 3 | Step consults call a separate pure function, `PacedRuleController::step`, that only continues a walk and never answers, greets or draws (SD-N16). `decide`'s lattice and windows are untouched. |
| **F-N7** | **Bodies' resolver has a fast path for a clear corridor.** `Route::Clear` — "a stride whose corridor was clear: no scene was built". | `systems/bodies/src/resolve.rs:53–64` | A routed stride keeps a margin from every solid (SD-N5), so almost every walking stride takes the fast path; Rapier runs only near people. This is where 12n should *lower* the towns' cost. |
| **F-N8** | **Bodies' person bounds are partly not expressions of the radius.** `PERSON_RADIUS` 300; `CLEARANCE` 595 (literal); `NUDGE_MAX` 300; `BIAS_BAND` 200; `BIAS_TURN` (4, 1). 12d's WIP takes R to 250 and derives `CLEARANCE`, not `NUDGE_MAX` or `BIAS_BAND`; two invariant tests then lose their meaning (TD-D7, TD-D8). | `systems/bodies/src/geometry.rs:10–53`; 12d `8814aad` §19.13 TD-D8 | SD-N11: derive both from R, byte-identical at R 300. |
| **F-N9** | **12d's café door gap.** Street doorway to the café at (0, 2 800); between the slice's A-board (−1 643, 1 131)–(−557, 2 169) and terrace table 1's crossed pair (530 … 1 770, 780 … 2 020) a ≈ 1.1 m corridor. With a body of R 250 + GAP 10 that leaves 580 mm of lateral slack — passable by anyone who aims for it. | 12d E-TD1 §2, §3b; TD-D7 | NW-2: the door is reached from every street doorway and from both sides of the terrace. |
| **F-N10** | **2D clients plan their own routes today, by design.** 13a's walker splits a click into ≤ 1.9 m strides with doorway routing; 13b's "Walk to <name>" is a client-composed straight walk stopping 1.2 m short (D-b-3); 13d plans routes with Godot's `NavigationServer2D` from the disclosed `place-shape` ("route planning is acquisition"). | `step-13-client-2d.md` §5.4 (lines 436–443), D-b-3 (1595), 13d row (591), RK-b2 (1798) | With a server walk the 2D client needs none of it: a click or a menu entry becomes `walk-to`. 13d's navigation plan is withdrawn (cross-lane, QN-8, ruled). |
| **F-N11** | **Complete affordances change the paced controller's draws.** `offered::attempt` picks uniformly among available *complete* affordances. | `cognition/rule-controller/src/offered.rs`; `ARC-34` | If `walk-to` were offered complete per person, 12n-1 would move every town's digest. It is offered like `move`: once, against nobody, incomplete (SD-N7). |
| **F-N12** | **Run cost today.** Without bodies: social-cafe 300 d 18.55 s user CPU, market-town 15.91 s (E-TD-base, dev). With 12d's WIP geometry: ≈ 105 s per 300 days by proportion (E-TD6, under load). TD-12a's bounds: 3.96 × and 3.30 ×. | §19.12 on `impl-12d` | NW-4 holds 12d's bounds on the scratch merge, and adds a no-bodies bound so headless stays fast. |

## 21.3 Where the route lives — options

The four things that stay distinct (`CLAUDE.md` §4 rule 14; `ENGINEERING_RULES.md` §6), and where each sits
in the recommendation:

```text
MoveIntent          `move` (one stride, a destination point), `walk-to` (a destination) and
                    `walk-step` (the next stride of my walk) — requests, refusable
Travel Process      not used at this scale: a walk across a town takes no calendar time (step-19
                    §4.2 D). The long-distance travel Process (ARC-26 "Room for travel") stays future
Spatial state       Presence, owned by presence; written only by reducing `arrived`. Beside it,
                    movement's `Walking` component: the intention and its route, not a position
Rendered movement   whatever a client draws between two observed positions
```

| Option | What it is | For | Against | Verdict |
| --- | --- | --- | --- | --- |
| (a) server-side travel **Process** that plans and emits strides (revision 0) | `walk-to`; a movement Process wakes once a world second and states a stride. | One implementation for every caller; the walk is world state. | **Wakes in calendar time**, so the on-screen speed is multiplied by the host's time scale (≈ 16 m/s at 12×) — step-19 §4.2 option D, rejected by the operator; correcting it inside the pack would read the scale — option C, also rejected (R-12n-1). | **Withdrawn (revision 1).** |
| **(c′) hybrid: server route, embodied strides** | Movement owns `walk-to` and the route (a component; `Wayfinder` plans it); each stride is a `walk-step` request sent at the body's cadence. | Everything (a) had — one planner, world-state walks, every caller just says "go to X" and then "keep walking" — and pace is wall-clock by construction, exactly as for `move` (step-19 §4.2: "met by construction for anything that is per-request"). No pack reads the scale; no Process wakes. | The sender must send one request per stride (a client timer; the hosted adapter's cadence; `run`'s notional step consults). A client may send steps faster than a human walks — the same accepted limitation as `move` (`ARC-26` L-1). | **Chosen**, with ownership (a1) below. |
| (a1)/(c′) owned by **movement**, geometry through a movement-owned catalog | Movement owns `walk-to`, `walk-step`, `Walking`; bodies answers route queries through `Wayfinder`. | Single ownership as it already stands: movement owns walking and passages, bodies owns geometry. Movement never names bodies (`ARC-62`). Without bodies, walks are straight. | One more catalog (the mechanism exists). | **Chosen.** |
| (a2) owned by **bodies** | Bodies provides `walk-to` and the walk. | Geometry is local. | Walking without bodies would not exist; bodies would duplicate movement's passage and stride rule, and become a God pack. | Rejected. |
| (a3) a new **`wayfinding`** pack | A third pack owns the walk. | Smaller packs. | It must duplicate movement's private stride and passage rule or movement must publish it; three packs for one capability. Premature (`CLAUDE.md` §4 rule 11). | Rejected now; revisit with long-distance travel. |
| **(b) planning service in the SDK** | A library controllers call, over disclosed geometry; each controller strides along its own route. | No new world state. | Every client and controller must embed it: the Godot 2D/3D clients (GDScript) and a Python LM controller could not without a port — duplicated logic (`ENGINEERING_RULES.md` §9). A stateless controller must re-plan every consult. | Rejected. |
| (c) hybrid with an SDK planner for previews | Server walk plus a client-side preview planner. | Previews. | Two planners that can disagree. The disclosed `walking` record (SD-N8) is the preview. | Rejected. |

## 21.4 Reuse comparison (`REUSE_POLICY.md`; `CLAUDE.md` §4 rule 16; both directions)

Facts fetched from crates.io and the projects' pages on 2026-10-09; licences are the crates' declared
ones. The fit criteria: integer state in and out, byte-identical replay on arm64 and x86_64 (`ARC-25`,
`AC-8`), no engine bound in, no C++ toolchain (Windows, `all-platforms`), a world small enough to plan
per request (≤ 64 solids, ≤ 32 objects per place).

| Candidate | Licence | Maturity | Fit | Determinism | Cost | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| **`pathfinding` 4.16.0** (evenfurther) — A*, Dijkstra, BFS, fringe, IDA*, Yen; a `Grid` type. No JPS. | MIT / Apache-2.0 | Since 2016, ≈ 3.0 M downloads, updated 2026-09-07, MSRV 1.88. Deps: indexmap, rustc-hash, num-traits, integer-sqrt, thiserror, deprecate-until. | Generic over node and cost; integer costs (`C: Zero + Ord + Copy`). Searches *our* graph; imposes no geometry and no execution model. | Pure integer code; parents in an `FxIndexMap` (fixed hasher, insertion order); heap order `estimated_cost`, then higher `cost` — so equal inputs give equal paths on every machine. Pinned exactly, like `rapier3d` (`DEP-13`). | One small dependency; no build script; no floats. | **Adopt for the search** (`DEP-P`, QN-4, ruled). |
| `pathfinding`'s `Grid` + A* at 50 mm (or 100 mm) | as above | as above | The street (68 × 16.2 m) is 440 k cells at 50 mm, 110 k at 100 mm; 8-connected paths zig-zag and need string-pulling against the same inflated boxes anyway; a second discretization of a world that is continuous millimetres. | deterministic | ms per plan on the street; memory per plan | **Rejected** as the graph; kept as the fallback if the visibility graph's cost fails NV-10. |
| `landmass` 0.9.2 (andriyDev) | MIT / Apache-2.0 | ≈ 31 k downloads, updated 2026-07. Core crate not bound to Bevy (`bevy_landmass` is separate). Deps: glam, geo, dodgy_2d (ORCA), kdtree, slotmap. | A full agent navigation system: path finding, simplification, steering and local avoidance over a caller-supplied navigation mesh, updated by its own loop with agent velocities and a frame delta. That is forcing MineWorld into its execution model (`REUSE_POLICY.md`, the "forcing a wheel" half): our world advances in whole seconds and is resolved by bodies. | `f32` (glam) throughout, SIMD-dependent; no determinism statement. | Heavy; navmesh generation still needed | **Rejected.** |
| `oxidized_navigation` 0.12.0 | MIT / Apache-2.0 | ≈ 56 k downloads; last release 2024-12. | "A Nav-Mesh generation plugin for Bevy Engine": bound to Bevy's ECS and schedule. | floats | Bevy as a dependency of a System Pack | **Rejected** (engine-bound; `ENGINEERING_RULES.md` §5). |
| `recastnavigation-rs` 0.1.0 (Recast/Detour binding) | **MPL-2.0** (Recast itself zlib) | ≈ 2.5 k downloads, one release (2024-03). | C++ Recast/Detour through a fork; voxelizes triangle soup into a navmesh — far more than axis-aligned boxes need. | Claims cross-platform determinism through a patched fork, tested on Windows/Linux/macOS x64 and arm64 Docker; floats inside C++. | A C++ toolchain on every platform; an unmaintained-looking binding; a file-level copyleft licence to clear under `ARC-55` | **Rejected.** |
| `polyanya` 0.17.1 (vleue) | MIT / Apache-2.0 | ≈ 98 k downloads, updated 2026-08. Not bound to Bevy (`vleue_navigator` is the Bevy layer). Deps: glam, geo, spade, rstar, hashbrown, rerecast. | Optimal any-angle paths on a polygon mesh; can triangulate an outer boundary with obstacle holes. Exactly the path quality we want, but via a constrained Delaunay triangulation in floats. Agent radius is the caller's job (inflate obstacles first). | `f32` geometry (glam, spade); no determinism statement; triangulation robustness depends on float predicates. | Several geometry dependencies for a problem of ≤ 96 boxes | **Rejected** for the authoritative path; it is the reference a later, larger-geometry design should prototype first. |
| **Visibility graph over the grown boxes, built ourselves** | ours | — | For axis-aligned boxes grown by the body's radius the shortest path bends only at grown corners: ≤ 4 × (64 + 32) + 2 = 386 nodes. Exact integer predicates (cross products in `i128`), the style of `geometry.rs`. | exact | ≈ 200 lines in `bodies`; lazy edge evaluation keeps a plan to a few ms (NV-10) | **Build** the graph; search it with `pathfinding`. |

**Why build the graph rather than adopt a navmesh** (the record `DEP-P` carries, because rejecting a
mature wheel needs a reason, `CLAUDE.md` §4 rule 16): every navmesh candidate is float geometry whose
cross-machine bit-identity is unproven, and a route's waypoints become positions in the log, so they
must be exact (`CORE_CONCEPTS.md` §6.2: "Fixed-point, never floating-point"). Rapier is admitted under
`DEP-13` only behind quantization and a verify-then-degrade check; a planner's output has no such
check to fall back on — a waypoint off by one millimetre on another machine is a replay divergence.
The obstacle model is axis-aligned boxes by construction (`PlaceShape`), which is the one case where a
visibility graph is both optimal and tiny. "Writing it ourselves feels cleaner" is not the reason;
determinism and the shape of our geometry are.

## 21.5 Design (SD-N1 … SD-N16)

### 21.5.1 Constants (published; real-world references)

```text
WALK_STRIDE        1 340 mm per `walk-step`         movement's rule: the most one step carries a walker
                                                   along their route (< MAX_STRIDE). A distance, not a
                                                   duration: no rule knows how often steps arrive.
EMBODIED_STEP      1 wall second per `walk-step`    the senders' cadence (host adapters and clients,
                                                   never a pack): WALK_STRIDE per wall second is
                                                   1.34 m/s — Weidmann (1993), mean free walking speed
                                                   of pedestrians; Bohannon (1997), comfortable gait
                                                   1.27–1.46 m/s for adults 20–59. Hosted: stated to
                                                   the world-time seam as EMBODIED_STEP × scale world
                                                   seconds, the D-SB6 form (step-19 §4.4)
RUN_STEP           30 world seconds                 headless `run`'s notional step cadence (step-19
                                                   §4.6: `run` has no scale, its pace is notional). A
                                                   divisor of PACE (900), so seat k's step instants
                                                   genesis + k + n·30 never meet another seat's
                                                   instants while seats < 30 (SD-N15)
PERSON_APPROACH    1 200 mm, centre to centre       Hall (1966), the far limit of personal distance
                                                   (0.46–1.22 m): near enough to talk (conversation's
                                                   range is 3 m), not on top of them; the 2D client's
                                                   D-b-3 already uses 1.2 m
PLAN_MARGIN        50 mm (= LATTICE)               extra clearance a route keeps beyond R + GAP, so a
                                                   routed stride takes bodies' clear-corridor path
STALLS_MAX         3                                consecutive steps with < 50 mm of progress → the walk
                                                   ends `stalled`
REPLANS_MAX        8                                re-plans per walk (on a stopped stride, a moved
                                                   target or a moved object) → then `stalled`
WAYPOINTS_MAX      64                               a longer route is refused `no-route` (never truncated)
```

Owners: movement publishes WALK_STRIDE, PERSON_APPROACH, STALLS_MAX, REPLANS_MAX; bodies publishes
PLAN_MARGIN, WAYPOINTS_MAX — like `MAX_STRIDE`, world configuration later with IL-c (`ARC-61`).
EMBODIED_STEP is **not** a pack constant: it is the senders' cadence, stated once in
`clients/protocol/ADOPTION.md` for clients (beside the `move` reporting rule) and as a host constant in
`tools/cli/src/hosted.rs`; RUN_STEP is `run`'s, in `tools/cli/src/run.rs`. No System Pack reads either,
nor the time scale (step-19 §4.2 option C).

### 21.5.2 Decisions

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-N1** | **`walk-to`, movement's second action.** Payload `WalkTo { to: Destination }`, `Destination = Place(Location) \| Person(PersonId)`. A `Place` destination may be in the walker's place or any place reachable through passages (breadth-first over `Passages`, places in `PlaceId` order); a `Location` without a local position means "enter that place" (end at the passage's `there`). A `Person` destination must be in the walker's place (`SpatialRequirement::same_place`, else `TooFarAway`) and ends within `PERSON_APPROACH` of them, following them if they move. Offered once, against nobody, incomplete — exactly like `move` (SD-N7). | One request for "go to X" from every caller (`ENGINEERING_RULES.md` §9). Objects as destinations wait for a need (their positions are bodies' state; a later `Destination` arm through the same catalog). |
| **SD-N2** | **The walk is movement's `Walking` component; each stride is a `walk-step` request** *(revision 1)*. `resolve` of `walk-to` plans the first leg (SD-N3), writes `Walking { destination, legs, waypoints, progress, stalls, replans }` on the walker, and emits `walk-started { person, destination }` — no stride, no Process. `walk-step` (payload empty; offered once, against nobody, only to a person who has a `Walking`; refused `PreconditionFailed` otherwise) takes **one stride**: toward the next waypoint, at most `WALK_STRIDE`, never past it (a stride ends at a waypoint, so a corner is never cut); at a doorway's `here`, the crossing to `there`, after which the next leg is planned. The stride is checked by movement's own `reachable` (the `move` rule) and stated through presence's `arrivals()` — so bodies' resolver resolves it like any stride and presence records it; then `Walking` is updated, or removed with `walk-ended { person, destination, outcome }`, `outcome ∈ arrived \| stalled \| no-route \| replaced \| stopped`, in the same emission list. A walk nobody steps simply waits: it takes no calendar time and ends no other way. | step-19 §4.1–4.2: embodied pace is the cadence of requests, and "the scale must not affect actions … is met by construction for anything that is per-request". `walk-step` is per-request exactly as `move` is. Keeping the walk in movement reuses `reachable`; the stride rule stays one rule. Every fact is caused by a request (`AC-9`). |
| **SD-N3** | **`Wayfinder`: a catalog movement owns** (`ARC-62`): `trait Wayfinder: Send + Sync { fn wayfinder_of() -> SystemId; fn route(&self, world: &WorldRead, ask: &RouteAsk) -> Option<Waypoints> }`. `RouteAsk { person, place, from, to, avoid: Option<PersonId> }`, built only by movement; `Waypoints` a non-empty list of integer `LocalPosition`s ending at the (possibly snapped) goal, at most `WAYPOINTS_MAX`, or the answer `Unreachable`. Rules, as `ARC-39` item 5 / `ARC-62` item 4: pure (reads only the `WorldRead`), keeps nothing, no clock, no randomness, no process-wide state; inert — `None` — where the implementing pack has no state for the place. Asked in ascending `SystemId`; the first non-`None` answer is the route. No answer: the route is the straight segment `from → to`. One line in `systems/installed`: `extension mineworld_movement::Wayfinder => mineworld_movement::register_wayfinders: [mineworld_bodies::BodiesSystem,];`. | Single ownership: movement owns walking; bodies owns geometry and therefore answers geometric questions. Movement never names bodies; a world without bodies is unchanged. A third catalog is a line (`ARC-62` item 3), proving the abstraction. |
| **SD-N4** | **Bodies' wayfinder: a visibility graph on integers** (`systems/bodies/src/route.rs`, new). Obstacles: every solid's footprint and every loose object's footprint (box half-extents; a ball as its bounding square), each **grown** by `M = PERSON_RADIUS + GAP + PLAN_MARGIN` on every side; the floor **shrunk** by `M`. Nodes: `from`, the goal, and every grown corner strictly inside the shrunk floor and outside every other grown box, in authored order (solids, then objects by `ItemId`; corners SW, SE, NE, NW). An edge exists iff its segment meets no grown box's open interior (touching a boundary is allowed), decided with `i128` cross products — no float, no root. Cost ⌈√(dx² + dy²)⌉ mm, heuristic ⌊√(…)⌋ to the goal (admissible); search `pathfinding::directed::astar`, successors in node order, edges evaluated lazily on expansion. Fast path first: if `from → goal` is visible, the route is `[goal]`. | Shortest paths among convex polygons bend only at their vertices; with axis-aligned grown boxes the graph is ≤ 386 nodes. Integer predicates make the result exact everywhere (`ARC-25`). The margin is what makes a routed stride bodies' `Route::Clear` (F-N7). |
| **SD-N5** | **Start and goal edge cases, decided.** A **start** inside a grown box (someone standing at the counter, 260 mm from its face) ignores the boxes containing it for its own outgoing edges only. A **goal** inside a grown box or outside the shrunk floor is snapped to the nearest free point of the 50 mm lattice — distance, then y, then x (bodies' E3 order, `ARC-39` note 3) — searched outward as SD-Z6; none in the place → `Unreachable`. A goal unreachable from the start (an enclosed pocket) → `Unreachable`. | A destination inside a table is a person asking to go to the table: they arrive beside it. A walk into a pocket is refused at once rather than ending `stalled` later. |
| **SD-N6** | **Validation is the request's answer.** `walk-to`'s `validate` (read-only): payload readable; actor a living person with a `Presence`; destination admitted (`admit`); for a `Person`, same place (else `TooFarAway`); the place chain exists (else `Rejection::System { code: "no-route" }` — note N-1); the first leg's wayfinder answer is not `Unreachable` (else `Rejection::System { code: "no-route" }`). Only the first leg is checked at validation; later legs are planned on entry to their place (a place's geometry may change before then). | The client learns at once that its click cannot be reached (`ENGINEERING_RULES.md` §8's answer list, plus a pack code). |
| **SD-N7** | **Offered incomplete, once, against nobody**, as `move` is; a client or controller composes the request. | F-N11: complete per-person offers would move every town's digest in 12n-1 and add draws the controller never chose. A "Walk to <name>" entry is the client composing `walk-to { Person }` for a perceived person — intent, not a rule. |
| **SD-N8** | **What movement discloses.** To whoever perceives a walking person: `walking { destination, next: [≤ 4 waypoints] }` from the `Walking` component; nothing when not walking. | Clients draw intent (a 2D route line, a 3D NPC's heading) from the server's own plan — no client planner (SD-N12). A sender knows to keep stepping while its own record is disclosed (SD-N14 … SD-N16). |
| **SD-N9** | **Re-planning.** At a `walk-step`, the walk re-plans from where the walker actually is when: the previous stride stopped short (`reached ≠ to`) — with `avoid` = the `stopped_by` person, whose disc grown by `2R + GAP + PLAN_MARGIN` becomes one more box for this plan only; a `Person` target moved more than 500 mm from the planned goal; or the place's `LooseObjects` changed since the plan (bodies answers from current state). At most `REPLANS_MAX` per walk; `STALLS_MAX` consecutive steps with < 50 mm progress end it `stalled`. | Realistic: someone stopped by a person in a doorway steps round them. Bounded: no walk is stepped forever. A stalled walk ends visibly, and the controller may ask again later. |
| **SD-N10** | **Superseding.** A new `walk-to` by the same person ends the walk (`replaced`) before recording the next. A `move` by the walker ends it (`stopped`) — WASD always wins. A shove or a nudge does not end it; the next step re-plans from the new position. | A walk is the person's intention; their own direct control overrides it. |
| **SD-N11** | **TD-D8: person-relative bounds are expressions of the radius** (§21.6). `NUDGE_MAX := PERSON_RADIUS`; `BIAS_BAND := ⌊2 · PERSON_RADIUS / 3⌋`; `BIAS_TURN` stays (4, 1). Byte-identical at R 300 (300 and 200 are today's values). | The coordinator's TD-D8 input; the operator's "defaults follow the real world". |
| **SD-N12** | **Clients** (their own PRs; cross-lane, §21.9). 2D: a floor click → `walk-to { Place(Location) }`; "Walk to <name>" → `walk-to { Person }`; then one `walk-step` per `EMBODIED_STEP` wall second while the player's own `walking` record is disclosed — a timer, the same kind of pacing as `move`'s reporting rule, never a rule about the world. Draw the disclosed `walking` record if wanted. 13d's `NavigationServer2D` plan is withdrawn (QN-8, ruled); 13a's client stride-splitting stays only for WASD. 3D: WASD reports `move` strides as today; a future click-to-walk is `walk-to` + steps; NPCs are drawn from observed positions (one arrival per wall second, interpolated at 10 Hz). Python/LM: `walk-to` and `walk-step` are ordinary typed actions; an LM controller connects as a session and steps at its client's cadence. `clients/protocol/ADOPTION.md` gains the stepping rule. | No route is computed by any client (`CLAUDE.md` §4 rule 15). 2D and 3D share the capability without duplicating logic (§21.8's gating questions). |
| **SD-N13** | **Versions and records.** Movement `VERSION` 2 (two new actions, a component, two facts, a catalog); a save written by v1 is refused by name (`ARC-25`). Bodies' `VERSION` unchanged by 12n-1 (it adds an answer, and SD-N11 changes no result at R 300). `ARC-W` (new): "A walk is movement's state; its route is the geometry owner's answer; its strides are embodied requests" — citing step-19 §4 / `ARC-67` and QTW-13. `DEP-P` (new): `pathfinding`, pinned `=4.16.0`, and the rejected navmesh crates. `ARC-26` note (walking is not the travel Process), `ARC-27` note (step consults, SD-N15), `ARC-42` note (step consults in hosted seats, SD-N14), `ARC-39` note (bodies is also a wayfinder), `MODULE_SPEC.md` §4.1 (movement, bodies) and §8.1 (`run`), `systems/README.md`, `clients/protocol/ADOPTION.md`. | `CLAUDE.md` §2.2: documents first. |
| **SD-N14** | **Hosted seats step at wall cadence** (12n-2; `tools/cli/src/hosted.rs` only). `PacedSeat` keeps its lattice `genesis + (k + m·pace)·scale` for `decide`, and interleaves **step consults** every `EMBODIED_STEP × scale` world seconds (offset `k·scale`, D-SB6's form) at which it calls `PacedRuleController::step`. To keep idle seats cheap it schedules step consults only after a consult whose observation disclosed the seat's own `walking` record or whose request was `walk-to`/`walk-step`, and stops at the first step consult that returns nothing. `ReactiveSeat` (consulted every wall second already) does not walk. S19's live-rescale pass (`now + w × s₁`) covers step consults like any pending consult. | QTW-13 / D-SB6: cadence is the host adapter's, in wall seconds; the server and every pack stay in world time and never see the scale. One stride per wall second is 1.34 m/s on screen at 6×, 12× or 24× (NW-9). |
| **SD-N15** | **Headless `run` steps at a notional cadence** (12n-2; `tools/cli/src/run.rs`). Besides the lattice `genesis + k + m·PACE`, seat `k` is step-consulted at `genesis + k + n·RUN_STEP` whenever its person has a `Walking` component at that instant — read from world state through movement's public `is_walking(world, person)`, so a resumed run schedules exactly what the dead one would have (`ARC-27`'s restart property). A step instant that is also a lattice instant is the lattice consult. `run` refuses a pack with 30 or more seats (was 900): residues mod 30 keep every instant to one seat. | step-19 §4.6: `run` has no scale, and its pace is notional. A 20 m walk becomes 15 steps = 7½ notional minutes instead of 2½ hours of consults. |
| **SD-N16** | **The controller's step is a separate pure function.** `PacedRuleController::step(&self, &Observation) -> Option<ActionRequest>`: `walk-step` iff the observer's own `walking` record is disclosed and `walk-step` is offered; nothing else, no draw. `decide` is unchanged in its lattice, windows and draw indices, except that its walking bands ask `walk-to` (12n-2). | F-N14: `decide` must stay one pace apart for answer-once; `step` touches nothing it relies on. The controller stays stateless (`ARC-27`). |

### 21.5.3 Why the walk does not break the rules it touches

- **Kernel ignorance.** No kernel change; the kernel does not know what a route is.
- **Two time domains (step-19 §4; QTW-13).** A walk takes no calendar time and no rule reads the scale;
  embodied pace is the wall cadence of `walk-step` requests, owned by the senders (clients, host
  adapters) exactly as `move`'s is. At any time scale a hosted walker covers `WALK_STRIDE` per wall
  second.
- **Single ownership.** Positions: presence alone. The walk: movement alone. Obstacles and the answer
  about them: bodies alone. A route is never written by bodies; bodies returns a value movement stores.
  Cadence: the sender alone.
- **Stateless controllers (`ARC-27`).** The controller holds nothing; the walk is world state, persisted
  and replayed like any component. `run` decides step consults from world state.
- **Clients only report intent.** `walk-to` and `walk-step` are intents; the route, each stride and its
  resolution are the server's.
- **Determinism.** Integer geometry, a pinned integer search, `BTreeMap`-ordered state, requests
  journalled with their instants (`ARC-25`). Nothing in a pack reads a clock or a random source.

## 21.6 TD-D8 — the radius-dependent bounds, decided (the coordinator's input)

**What fails at R 250 (12d `8814aad`, TD-D8).** (1) `a_stride_toward_a_person_within_the_offset_is_still_stopped`:
the head-on bias (band 200 mm, a fixed 14° turn) turns the stride, which now clears the smaller disc,
so the walker slides round instead of being stopped. (2) `n3_a_crowd_is_nudged_in_bounded_chains_and_sometimes_blocks`:
`NUDGE_MAX + GAP` (310 mm) and the 500 mm step are fixed, so relative to a 50 cm body the nudge budget
always clears the crowd; 0 of 12 strides block (was 3).

**Real-world reasoning.**

```text
body             adult shoulder breadth 40–46 cm (the operator's figure); Fruin (1971)'s pedestrian
                 body ellipse 0.61 × 0.46 m. R = 250 mm is a 50 cm disc: the shoulder breadth plus
                 clothing and sway.
yielding         a standing person brushed in a crowd yields by a short side-step — of the order of
                 their own half-width, a quarter metre, not a fixed number independent of how big
                 they are. NUDGE_MAX := R keeps QB-10's original relation (300 at R 300) and gives
                 250 mm at R 250.
head-on          two walkers are "head-on" when the lateral offset of their centres is a small fraction
                 of a body's width; BIAS_BAND := ⌊2R/3⌋ (a third of the shoulder breadth 2R) keeps the
                 ratio 12b chose (200 at R 300) and gives 166 mm at R 250.
the turn         an avoidance heading change is a behaviour, not a dimension: BIAS_TURN keeps atan(1/4)
                 ≈ 14°, independent of size.
tolerances       GAP, TOLERANCE, SNAP, LATTICE are numeric tolerances of the solver and the lattice,
                 not body dimensions: they stay absolute.
```

**Decision (SD-N11).** `NUDGE_MAX` and `BIAS_BAND` become expressions of `PERSON_RADIUS` in 12n-1. At
R 300 (main) every value is unchanged, so no result moves and bodies' version does not change; at
R 250 (12d) they scale with the body. Each test scenario's lengths become multiples of R (the 500 mm
step = `5R/3`; the crowd spacing `13R/6`, = 650 at R 300), as 12d's TD-D8 began.

**What the two invariants claim after 12n (fixed now, before measuring):**

1. **"A stride toward a person within the offset is never passed through."** Two cases, both asserted:
   (i) the person dead ahead (inside `BIAS_BAND`): the walker ends either stopped (`stopped-short { by:
   the person }`) or turned by the head-on bias, and in both cases never within `CLEARANCE` of them and
   the person is not ignored by the sweep (the result is never the requested point); (ii) the person
   offset laterally by `BIAS_BAND + 1` mm (outside the band, still overlapping the walker's corridor):
   the walker is **stopped** by them. Case (ii) is SD-Z5's guard: mutation M-Z5 (the "away" exclusion
   widened to every person within the offset) makes the walker pass through and the test fail by name.
2. **"A crowd is nudged in bounded chains and sometimes blocks."** Unchanged in words, with the
   derived constants and R-multiple geometry: every stride keeps the bounds (≤ `CHAIN_MAX` generations,
   ≤ `NUDGED_MAX` people, each ≤ `NUDGE_MAX + GAP`), and at least one of the 12 strides blocks.

**Where it is shown.** In 12n-1 on main (R 300): both tests pass unedited in claim, bodies-yard's digest
unchanged (NV-7). On a scratch build at R 250 (12d's geometry.rs radius applied to 12n-1's head, never
committed): both pass with the restated claims (NV-7). 12d then closes TD-D8 by rebasing. If either
fails at R 250 with derived constants, that is a material stop with the numbers — no constant is
re-tuned to pass.

## 21.7 PR split

| PR | Scope | Integration checkpoint | Adversarial |
| --- | --- | --- | --- |
| **12n-1 — the walk** | movement: `walk-to`, `walk-step`, the `Walking` component, `walk-started`/`walk-ended`, `Wayfinder` catalog, disclosure, `is_walking`, VERSION 2; bodies: `route.rs` (visibility graph + `pathfinding`), the `Wayfinder` impl, SD-N11's derivations; `systems/installed`'s third line; `DEP-P`, `ARC-W`, notes; `clients/protocol/ADOPTION.md`'s stepping rule; tests on a test-time copy of bodies-yard with a 1.1 m slot and a U-shaped pocket, driven by scripted `walk-to` + `walk-step` requests. **No controller or host change.** | A real `mineworld` world: walk through the slot, round the pocket, to a person, into the other place; SIGKILL mid-walk and resume byte-identical; every existing world's facts byte-identical. | Mutations M-N1 … M-N5 (§21.8). |
| **12n-2 — people walk there** | `paced.rs`: the walking bands ask `walk-to`; `PacedRuleController::step` (SD-N16); `tools/cli/src/hosted.rs`: wall-cadence step consults (SD-N14); `tools/cli/src/run.rs`: notional step consults (SD-N15). Towns' digests move (QN-2, ruled). | On a scratch merge with 12d's WIP: TD-5 ≥ 90 %, the café door, stopped-short ≤ 10 %, TD-12's bounds; hosted: the same on-screen pace at 6×, 12×, 24×; on main: the towns without bodies still live and stay fast. | M-N6 (wayfinder line removed → TD-5 red); M-N7 (step cadence not wall time → NW-9 red). |

12n-2 after 12n-1 merges; 12n-2 merges immediately before 12d, and the two are recorded as one
re-baseline event (QN-2, ruled): 12d's TD-1 "before" stays E-TD-base, its "after" is measured on 12d's
final head, and no other PR merges between them.

## 21.8 Acceptance (decided before measuring, `ARC-23`)

Rules as §19.4: each guarded criterion names the mutation shown to break it (applied, observed to fail
by name, reverted; `git status` recorded); every expected value is a literal from authored files or
hand computation, never from the code under test; no criterion changes after a measurement — a failure
is a bounded remedy named here or a material stop with the numbers.

**12n-1:**

```text
NV-1  Byte identity. On the final executable head, with nobody asking for a walk: social-cafe and
      market-town 300 d seed 7, bodies-yard 30 d — the fact streams' sha-256 equal the base's (captured
      on the implementation base first: today ad49c723…c64b, 365b50e0…1d1d, bd6a1002…80e6); long_run and
      long_run_objects bytes equal their bases. Only the composition records differ (movement v2,
      its declared actions, emissions and component), named in the evidence.
NV-2  The walk, for real (tools/cli/tests/walking.rs, a test-time copy of bodies-yard; the real
      `mineworld` binary and server path; a scripted session sends `walk-to`, then `walk-step` until
      `walk-ended`):
        a  the hall gains two solids leaving a 1 100 mm slot (F-N9's width) across the direct line; a
           walk-to the far side ends `arrived` at the requested point, through the slot, every stride
           ≤ 1 340 mm, no `stopped-short`;
        b  a U-shaped pocket of three solids open away from the walker, goal behind its closed side: ends
           `arrived`, the path leaves the pocket's mouth (hand-computed waypoints asserted);
        c  walk-to a person 6 m away behind the table: ends within 1 200 mm of them, `arrived`;
           the person then moved by a scripted `move` mid-walk → re-planned, still `arrived`;
        d  walk-to a point in the court (the other place): crosses at the passage, `person-entered-place`,
           `arrived`;
        e  refusals: a goal inside an enclosed pocket → `no-route` at validation; a Person in another
           place → `TooFarAway`; a place with no passage chain from the walker's (a test-time
           world with an isolated place) → `no-route`, the literal code string asserted (N-1); a
           malformed payload → `malformed-payload`;
        f  superseding: a second walk-to → `walk-ended { replaced }` then `walk-started`; a `move` →
           `walk-ended { stopped }`; a shove mid-walk → the walk continues and arrives;
        g  no calendar time: a walk-to followed by `advance_to` one simulated day with no `walk-step`
           records no stride and ends nothing (the walker has not moved; `Walking` unchanged); a
           `walk-step` with no walk → `PreconditionFailed`.
      M-N0  a stride taken in `walk-to`'s resolve or on a clock wake (any movement Process) → (g) fails.
      M-N1  the wayfinder line removed from systems/installed → (a) and (b) fail (stopped-short at the
            slot's solids, `stalled`).
      M-N2  PLAN_MARGIN 0 → (a)'s "no stopped-short" or bodies' Route::Clear count fails by name.
      M-N3  loose objects omitted from the obstacles → a walk through an object scenario pushes it
            (an `object-moved { pushed }` the test forbids).
NV-3  Without bodies: social-cafe's test-time copy, a walk across the café and into the street is a
      straight route (one waypoint per leg), every stride accepted at the requested point.
NV-4  Determinism. A walk crossing a SIGKILL (two processes, kill between two `walk-step`s) resumes —
      `Walking` restored from the save — to the same bytes as the uninterrupted run; `mineworld replay`
      regenerates every fact; two runs give equal bytes.
NV-11 No pack reads the time scale (structural): movement's and bodies' sources name no scale, no
      host type and no wall clock; `walk-step`'s result is independent of the instant it arrives at
      (the same request at two instants yields the same stride, NV-2 (g)'s fixture).
      M-N4  the successor order made to depend on a process-global counter's parity → NV-4 diverges
            or the resume is refused.
NV-5  The planner's invariant, by an independent oracle (systems/bodies/tests/route.rs): for 2 000
      seeded random scenes (≤ 64 boxes, ≤ 32 objects, seed fixed), every returned route's legs keep
      every point sampled every 10 mm at least R + GAP from every solid and object and inside the floor
      shrunk by R + GAP (the oracle is brute-force distance, sharing no code with route.rs); every
      `Unreachable` is confirmed by a flood fill on a 25 mm grid.
      M-N5  the edge test made "touching counts as blocked → passing" (open/closed interior swapped) →
            the oracle names the first failing scene.
NV-6  Isolation (structural tests): movement's crate names no bodies; `pathfinding` is used only in
      bodies' route.rs; no float in route.rs; kernel/, contracts/, presence/ have no diff.
NV-7  TD-D8 (§21.6): at R 300 both invariant tests pass with derived constants and bodies-yard NV-1
      holds; on a scratch build at R 250 both pass with §21.6's claims (evidence, not committed).
NV-8  Cross-platform: the new tests use Path::join and no Unix-only API; CI's Linux job runs them; an
      x86_64-apple-darwin build under Rosetta gives NV-2's facts byte-identical (recorded, as PO-12).
NV-9  Gate: cargo fmt --check; clippy --workspace --all-targets --all-features -D warnings; the full
      workspace tests once on the final head; both doc checks; Cargo.lock adds only `pathfinding` and
      its dependencies, each licence admitted by ARC-55.
NV-10 Cost of a plan: a scratch timing build over NV-5's 2 000 scenes and the street of 12d's WIP
      (62 solids): plan max ≤ 5 ms, p99 ≤ 1 ms (dev profile). Over the bound → the bounded remedy is
      the 100 mm grid fallback (§21.4), recorded as a deviation.
```

**12n-2 (measured on a scratch merge of 12n-2's head with `mvp0/pr-12d-towns` @ `8814aad` or its
successor, unless "on main"):**

```text
NW-1  12d's TD-5 activity on 12d's WIP geometry: routines.rs (social-cafe, unedited) — every person
      reaches ≥ 90 % of their agenda segments; run.rs (every seat moves and talks in every 30-day
      bucket, all six places entered) on the 300-day run.
NW-2  The café door is reachable: from each of the five street doorways and from (−3 000, 600) and
      (3 000, 600) (both sides of the terrace), walk-to the café ends `arrived` with a
      `person-entered-place { café }`, within 40 strides, no `stalled`.
NW-3  Stopped short: over social-cafe 30 days, stopped-short ≤ 10 % of the walkers' own accepted
      arrivals (12d WIP: 62 %); place entries ≥ 90 % of the same run on the SD-D13 copy without bodies
      (12d WIP: ≈ 50 %). The per-bucket counts are printed.
NW-4  Cost (12d's TD-12 instrument, §20.6.1; interleaved pairs): with ÷ without bodies ≤ 3.96 ×
      (social-cafe) and ≤ 3.30 × (market-town); per-resolution max ≤ 50 ms; and **on main** the towns
      without bodies stay fast: 300 d user CPU ≤ 1.5 × E-TD-base (≤ 27.8 s social-cafe, ≤ 23.9 s
      market-town). Contamination rule as TD-12a (re-run once, then INCONCLUSIVE).
NW-5  Determinism: run_restart.rs (SIGKILL early/middle/late) and market_town.rs pass on the merge;
      two 30-day runs byte-identical; arm64 = x86_64 (Rosetta) 30-day summary sha for both towns.
NW-6  The mutation: the wayfinder line removed from systems/installed (walks go straight) → NW-1 fails
      naming the people below 90 % and NW-2 fails naming the doorways. Recorded with counts.
NW-7  On main (no bodies): every existing activity test (run.rs, run_restart.rs, routines.rs,
      milestone_b.rs, milestone_c.rs, market_town.rs, market_composition.rs, social_composition.rs)
      passes unedited in claim; literal edits only as listed in the commit plan, each claim stated.
NW-8  Scope and gate as NV-9.
NW-9  Embodied pace is wall-clock, whatever the scale (on main; tools/cli/tests/walking_pace.rs, new;
      the real `mineworld server` with `--town`, a hosted paced seat sent on a ≥ 20 m walk across the
      street of a test-time social-cafe copy). The instrument: a connected observer session records
      the walker's observed position and the wall instant of each observation frame (10 Hz); speed =
      straight-line displacement between consecutive changed positions ÷ wall time between those
      frames, the median over the walk. Runs at --time-scale 6, 12 and 24 (and 1 as reference).
      PASS iff every scale's median is 1.34 m/s ± 15 % (one stride per wall second, tolerance for tick
      and frame jitter) and the three medians at 6×, 12× and 24× lie within 10 % of each other; and the
      world-time stride rate scales with s (strides per world second ≈ 1 / s), which shows the scale
      was really applied (ARC-23).
      M-N7  the hosted step cadence stated as EMBODIED_STEP world seconds (not × scale) → the 24×
            median is ≈ 4 × the 6× median → NW-9 fails naming the scales.
NW-10 `run`'s step consults (SD-N15): a `run` killed while a seat is walking and resumed schedules the
      same step consults (run_restart.rs's byte comparison, on main and on the merge); a pack with 30
      seats is refused naming RUN_STEP; seat instants never coincide (a unit test over 29 seats and
      one year of instants).
```

NW-1 … NW-5 run under headless `run`, whose stride cadence is notional (RUN_STEP); they measure where
people get and how often they are stopped, never a speed. Speed on screen is NW-9's alone.

**The two gating questions** (`CLAUDE.md` §4): a Minecraft-like 3D client needs no kernel redesign (it
sends `move` while steering, and `walk-to` + `walk-step` when told to go somewhere); the 2D and 3D clients
use the capability without duplicating game logic (neither plans a route; each only paces its steps).
Both: yes.

## 21.9 Cross-lane impacts (stated, not done here)

- **S12 (2D) — a cross-lane change, ruled (QN-8); the primary session amends S12:** 13b's "Walk to
  <name>" becomes `walk-to { Person }` followed by one `walk-step` per wall second while the player's
  `walking` record is disclosed; a floor click likewise (`walk-to { Place }`); 13d's
  `NavigationServer2D` route planning (§5.4) is withdrawn; RK-b2 (a) disappears; 13a's stride splitting
  stays for WASD only.
- **S14 / 12e (3D):** no required change. Optional: a click-to-walk (`walk-to` + steps) and drawing
  NPCs' disclosed `walking` heading.
- **S11 / S19 (host pacing):** `PacedSeat` gains step consults (SD-N14) in D-SB6's `cadence × scale`
  form; S19's TW-c live-rescale pass must cover them like any pending consult (no seam change).
- **12d:** rebases on 12n-1 (and 12n-2 per QN-2); TD-5, TD-12, TD-D7 and TD-D8 are re-run there; the
  remedy ladder is no longer needed. 12d's QD-5 crate dependency `bodies → movement` is introduced by
  12n-1 (bodies implements movement's trait); 12d reuses it for `Passages`.
- **S10/S17 (Python SDK, LM controllers):** `walk-to` and `walk-step` are ordinary actions; an LM
  controller connects as a session (`ARC-42`) and steps at its client's wall cadence.
- **IL-c:** `WALK_STRIDE`, `PERSON_APPROACH` and the radius become world configuration there.

## 21.10 Questions (QN-1 …)

| ID | Question | Recommendation |
| --- | --- | --- |
**Ruled (primary session, 2026-10-09, within standing operator rules):** QN-2 (merge 12n-2 immediately
before 12d; one re-baseline event), QN-4 (adopt `pathfinding` 4.16.0, pinned exactly), QN-5 (TD-D8 as
§21.6; failing at R 250 is a material stop), QN-6, QN-7, QN-9, QN-10, QN-11 (as recommended), QN-8
(withdraw 13d's client planning; recorded as a cross-lane change to S12, which the primary session
amends). QN-1 and QN-3 are answered by revision 1 (R-12n-1): route server-side in movement through
bodies' `Wayfinder`; strides as embodied `walk-step` requests at 1.34 m/s per wall second.

| ID | Question | Status / recommendation |
| --- | --- | --- |
| QN-1 | Where does route planning live? | **Decided by revision 1:** (c′) with (a1) ownership (§21.3). |
| QN-2 | 12n-2 moves the towns' digests. | **Ruled:** merged immediately before 12d, one re-baseline event. |
| QN-3 | The walk's pace. | **Decided by revision 1:** `WALK_STRIDE` 1 340 mm per `walk-step`, one step per wall second (1.34 m/s, Weidmann 1993); notional in `run`. |
| QN-4 | `pathfinding` or our own search. | **Ruled:** adopt, `=4.16.0`. |
| QN-5 | TD-D8. | **Ruled:** as §21.6. |
| QN-6 | `PERSON_APPROACH` 1 200 mm. | **Ruled:** as recommended. |
| QN-7 | `walk-to` (and `walk-step`) offered incomplete. | **Ruled:** as recommended. |
| QN-8 | Withdraw 13d's client planning. | **Ruled:** withdrawn; S12 amended by the primary session. |
| QN-9 | Plan round the person who stopped the walker. | **Ruled:** as recommended. |
| QN-10 | Remove the 2D client's click stride-splitting. | **Ruled:** yes, in S12. |
| QN-11 | `wander` as a walk. | **Ruled:** yes. |
| **QN-12** — **ruled at the freeze (primary session): client-paced, routing on the server** (§21.0) | Revision 1 adds a second embodied request type the clients must send (`walk-step`, one per wall second while walking). Is "the client paces its own walk, the server routes it" acceptable as the operator's model of click-to-walk? The alternative that keeps clients silent — the server stepping walks on its own wall clock — would make a host-side pacer write world facts outside any request, which no host does today (`ARC-25`'s journal records only requests and `advance_to`). | **Accept the client-paced model.** It is step-19 §4.1's definition of embodied time ("the cadence at which embodied inputs arrive"), identical in kind to the `move` reporting rule clients already follow, and it needs no journal or host change. |
| QN-13 — **accepted at the freeze** | `run` refuses packs with 30 or more seats (was 900) so step instants never collide (SD-N15). | **Accept**; the largest pack has 11 seats. A larger pack moves to a smaller `RUN_STEP` divisor or a sequence tie-break then — bounded, not material. |

## 21.11 Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| RN-1 | Grown boxes are squares; Rapier sweeps a capsule. A leg at exactly `R + GAP + 50` from a face could still meet Rapier's offset numerically. | PLAN_MARGIN 50; NV-2 (a) asserts no stopped-short; NV-5's oracle. |
| RN-2 | Two walkers meet head-on in the 1.1 m gap and stall each other. | Head-on bias + nudges (12b); SD-N9 re-plans round the stopper; STALLS_MAX ends the walk visibly; NW-2 / NW-3 measure it. |
| RN-3 | Cost rises: step consults compute an observation per stride, plus `walk-started/ended`. | Today every stride already costs a consult; wasted strides vanish; idle seats get no step consults (SD-N14, SD-N15). NW-4 bounds the no-bodies towns at 1.5 × and the bodies towns at 12d's bounds. |
| RN-4 | A `pathfinding` upgrade changes tie-breaking and so results. | Pinned `=4.16.0`; an upgrade bumps bodies' VERSION, as `DEP-13` for Rapier. |
| RN-5 | Plan cost on the street (62 solids). | Fast path; lazy edges; NV-10 with the grid fallback named. |
| RN-6 | Merge conflicts with 12d's WIP in `geometry.rs` and bodies' tests. | 12n-1 touches only the two constants and the two tests' geometry; 12d rebases; the resolutions are recorded there. |
| RN-7 | The hosted server: a walk's strides land once per wall second; the 3D view of an NPC moves in 1.34 m hops. | Clients already interpolate between observations (10 Hz); a finer cadence (e.g. 670 mm per half second) is a host constant, not a rule change. |
| RN-8 | A client steps faster than a human walks. | The same accepted limitation as `move` (`ARC-26` L-1: a bound per request, not per second); stated in `ARC-W`. A speed model, if ever, is a separate decision. |
| RN-10 | A walk nobody steps (a disconnected player) lingers as `Walking`. | Harmless: it takes no calendar time and has no effect; a reconnecting client sees its `walking` record and may continue, `move`, or ask again. Disclosure shows it to others as a heading, which is true. |
| RN-11 | Step consults in `run` collide with lattice consults or each other. | SD-N15's residues mod 30; NW-10's unit test. |
| RN-9 | Bodies' `install` calling movement's `require_registered` makes every hand-composed test that installs bodies panic unless it registers wayfinders too (as it already registers resolvers, `ARC-39` "Tests that compose worlds by hand"). | NV-C5 audits every such test file first and adds the registration beside the resolver one; the count is recorded. |

## 21.12 Commit plan — 12n-1 (each commit tracks implementation, validation and review separately)

Rules as §19.5: evidence into a §21 ledger as `E-NV<n>`, deviations as `N-D<n>`; commands from the
worktree root with `$HOME/.cargo/bin/cargo`; anything over ~2 minutes in the background; files changed
with Edit/Write only.

### NV-C0 — Design (this section) — docs only
- [x] Implementation: §21, by the planning session on `plan/s15-12n`.
- [ ] Validation: both doc checks on the PR head.
- [ ] Review: operator / primary-session freeze review of §21.

### NV-C1 — Documents first, and the base captures
Goal: `ARC-W`, `DEP-P`, the `ARC-26` and `ARC-39` notes, `MODULE_SPEC.md` §4.1 (movement's `walk-to`,
`walk-step`, `Walking`, `Wayfinder`; bodies as wayfinder), `systems/README.md`. (The `ARC-27` / `ARC-42`
notes and §8.1 land with 12n-2's host change.) Base captures: NV-1's digests and
bytes on the implementation base. Non-goal: any code.
- [ ] Implementation: the records and spec edits; E-NV-base.
- [ ] Validation: doc checks; the captures equal the values in NV-1 or the difference is recorded.
- [ ] Review: terminology (`Process`, `ActionIntent`, no new synonym); the records state both reuse directions.

### NV-C2 — movement: the catalog
Goal: `Wayfinder`, `RouteAsk`, `Waypoints`, `register_wayfinders`, `registered_wayfinders`,
`require_registered` (movement's own, mirroring presence's, `ARC-62` item 4); the `installed!` line
listing nobody yet. Files: `systems/movement/src/{wayfinder.rs,lib.rs}`, `systems/installed/src/lib.rs`.
- [ ] Implementation: the trait and catalog; the empty line.
- [ ] Validation: the installed set's guard tests; a synthetic wayfinder in a movement test (own process)
  answers and is asked in `SystemId` order; never-registered means straight lines.
- [ ] Review: catalog rules match `ARC-62` item 4 word for word; nothing reads it but `walk`.

### NV-C3 — movement: `walk-to`, `walk-step`, `Walking`, facts, disclosure, `is_walking`, VERSION 2
Files: `systems/movement/src/{action.rs,walk.rs,event.rs,system.rs,component.rs}`;
`clients/protocol/ADOPTION.md` (the stepping rule beside the `move` reporting rule).
- [ ] Implementation: SD-N1, SD-N2, SD-N6 … SD-N10; `reachable` reused for each stride; no Process.
- [ ] Validation: movement tests with no wayfinder: straight walk, cross-place walk, person walk, every
  refusal, superseding, NV-2 (g)'s "no calendar time" and M-N0, a snapshot round trip mid-walk.
- [ ] Review: every stride goes through `reachable` then `arrivals()`; no position is written by movement;
  nothing in movement reads a clock or a scale (NV-11); a `move` ends a walk in the same dispatch.

### NV-C4 — bodies: the route (`route.rs`) and `DEP-P`'s dependency
Files: `systems/bodies/src/route.rs`, `Cargo.toml` (`pathfinding = "=4.16.0"`, `mineworld-movement`
as a crate dependency), `Cargo.lock`, `systems/bodies/tests/route.rs`.
- [ ] Implementation: SD-N4, SD-N5, SD-N9's `avoid`.
- [ ] Validation: NV-5 (oracle, 2 000 scenes) and M-N5; NV-10's timing in a scratch build.
- [ ] Review: no float, no `HashMap`; every predicate in `i128`; corner order and successor order are as
  specified; snapping reuses E3's order.

### NV-C5 — bodies answers: the `Wayfinder` impl, the installed line, SD-N11
Files: `systems/bodies/src/{system.rs,geometry.rs}`, `systems/installed/src/lib.rs`,
`systems/bodies/tests/{actions,scenarios,isolation}.rs`.
- [ ] Implementation: `impl Wayfinder for BodiesSystem` (inert without `PlaceShape`); `install` calls
  movement's `require_registered`; the line lists bodies; `NUDGE_MAX`, `BIAS_BAND` derived; the two
  tests per §21.6.
- [ ] Validation: bodies' full suite; NV-7 at R 300 and the scratch R 250 run; NV-6.
- [ ] Review: no result changes at R 300 (bodies-yard digest, long-run bytes).

### NV-C6 — the real walk
Files: `tools/cli/tests/walking.rs` (new; test-time world copies).
- [ ] Implementation: NV-2 (a–f), NV-3, NV-4.
- [ ] Validation: the tests; M-N1 … M-N4 applied and reverted.
- [ ] Review: every expected waypoint and distance is hand-computed in the test's comments.

### NV-C7 — close
- [ ] Implementation: ledger, `MVP_STATUS.md` rows, handoff.
- [ ] Validation: NV-1 (the four captures), NV-8 (Rosetta), NV-9 (gate) on the final head.
- [ ] Review: scope (`git diff --name-only`), the no-diff paths, the records match the code.

## 21.13 Commit plan — 12n-2 (medium detail; detailed at its own freeze after 12n-1 merges)

### NW-C1 — the paced controller asks for walks, and steps
Files: `cognition/rule-controller/src/{paced.rs,paced_tests.rs,agenda.rs}`.
- [ ] Implementation: `through`/`head_for`/`leave` → `walk-to { Place }` (enter the place);
  `approach` → `walk-to { Person }`; `wander` → `walk-to` a seeded offset in the same place; a person
  whose own `walking` record already leads to the chosen place asks for nothing; draw indices unchanged;
  the straight-stride code removed (`CLAUDE.md` §4 rule 12); `step` (SD-N16).
- [ ] Validation: paced_tests rewritten in claim-preserving form (each claim stated); the controller is
  still a pure function (two decisions on one observation are equal); `step` returns only `walk-step`,
  takes no draw and never answers.
- [ ] Review: no geometry and no distance rule in the controller beyond choosing destinations.

### NW-C1b — the hosts pace the steps
Files: `tools/cli/src/{hosted.rs,run.rs}`, `tools/cli/tests/walking_pace.rs` (new), `docs/DECISIONS.md`
(`ARC-27`, `ARC-42` notes), `docs/MODULE_SPEC.md` §8.1.
- [ ] Implementation: SD-N14 (wall-cadence step consults in `PacedSeat`, `EMBODIED_STEP × scale`),
  SD-N15 (`run`'s RUN_STEP consults from `is_walking`, the 30-seat refusal).
- [ ] Validation: NW-9 at 1×, 6×, 12×, 24× and M-N7; NW-10; hosted.rs's existing
  `cadence_is_wall_time_whatever_the_scale` extended to step consults.
- [ ] Review: no pack and no controller sees the scale; idle seats get no step consults; a step instant
  never shares an instant with another seat in `run`.

### NW-C2 — the towns on main, and the literal edits
- [ ] Implementation: only literal edits whose claims are unchanged, each listed.
- [ ] Validation: NW-7, NW-4's no-bodies half.
- [ ] Review: no assertion weakened, no threshold lowered.

### NW-C3 — the 12d WIP measurement (scratch merge, never committed)
- [ ] Implementation: the scratch merge recipe recorded (`git worktree` at 12n-2's head, `git merge
  --no-commit mvp0/pr-12d-towns`).
- [ ] Validation: NW-1, NW-2, NW-3, NW-4, NW-5, NW-6, NW-10 with counts.
- [ ] Review: the measured tree's fingerprint recorded; no result is copied into a committed test.

### NW-C4 — close: ledger, gate, handoff to 12d.
- [ ] Implementation / [ ] Validation (NW-8) / [ ] Review.

## 21.14 Execution contract for PR 12n (filled at the freeze, 2026-10-09)

```text
PROJECT / PR        MVP-0 · Step 11 / PR 12n — navigation (S15): 12n-1 the walk, then 12n-2 people walk
                    there — two PRs, in that order
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-11-bodies.md §21; evidence and deviations §21.15
                    (E-NV<n>, E-NW<n>; N-D<n>)
RELATED / BINDING   §§16–20 (12a … 12d-0 as merged; ARC-39 and its notes; DEP-13); §19 and 12d's WIP
                    (mvp0/pr-12d-towns @ 8814aad or successor, read-only: TD-D7, TD-D8, E-TD1, E-TD6);
                    step-19-time-weather.md §4 (two time domains, QTW-13); step-12-server.md §16 (D-SB6);
                    DECISIONS ARC-23, ARC-25, ARC-26, ARC-27, ARC-34, ARC-42, ARC-55, ARC-62;
                    REUSE_POLICY.md; ENGINEERING_RULES.md §§4–9; CLAUDE.md §§2–4
PRECONDITION        §21 frozen (done, §21.0); PR #100 merged
IMPLEMENTATION BASE main at the session's start, with #100 merged; re-audit §21.2's paths first
BRANCH / WORKTREE   12n-1: branch mvp0/pr-12n-navigation from main, worktree
                    /Users/yuema137/mineworld-worktrees/impl-12n, held by the implementing session only.
                    12n-2: after 12n-1 merges, a fresh session in the same worktree (once 12n-1's session
                    has closed), branch mvp0/pr-12n-navigation-2 from main. Never impl-12d.
APPROVED SCOPE      12n-1: NV-C1 … NV-C7 (§21.12), SD-N1 … SD-N13, SD-N11 (TD-D8); paths: systems/movement,
                    systems/bodies (route.rs, the Wayfinder impl, the two constants, the two tests' geometry,
                    Cargo.toml), systems/installed, Cargo.lock (pathfinding and its dependencies only),
                    tools/cli/tests/walking.rs (+ fixtures under tools/cli/tests/), clients/protocol/
                    ADOPTION.md, docs/{DECISIONS,MODULE_SPEC,MVP_STATUS}.md, systems/README.md, this ledger
                    and handoff.md.
                    12n-2: NW-C1 … NW-C4 (§21.13), SD-N14 … SD-N16; paths: cognition/rule-controller/src,
                    tools/cli/src/{hosted,run}.rs, tools/cli/tests/ (walking_pace.rs; literal edits listed
                    per NW-C2), docs (ARC-27, ARC-42 notes; MODULE_SPEC §8.1), this ledger, handoff.md
FROZEN INVARIANTS   no edit under kernel/, contracts/, persistence/src/, server/, systems/presence/,
                    clients/ (except ADOPTION.md's stepping rule), worlds/; no pack or controller reads the
                    time scale or a wall clock; bodies' resolution rules unchanged (only NUDGE_MAX and
                    BIAS_BAND become expressions of R, value-identical at R 300); 12n-1 changes no fact of
                    any existing world (NV-1); every criterion of §21.8 as written, never changed after a
                    measurement
SEQUENCE            12n-1: NV-C1 (documents, E-NV-base before any code) → C2 → C3 → C4 → C5 → C6 → C7, each
                    committed and pushed when coherent; 12n-2 starts only after 12n-1 merges; its NW-C3
                    measurements on the scratch merge with 12d's WIP, never committed there; 12n-2 merges
                    immediately before 12d (QN-2)
COMMANDS            as 12d's contract (§19.9): from the worktree root, `$HOME/.cargo/bin/cargo`;
                    CARGO_TARGET_DIR for scratch builds under /tmp/s15-12n; long jobs in the background
TOOL DISCIPLINE     as 12d's contract: Read, Edit and Write for files; allowed `cargo`, `git`, `gh`,
                    `python3 scripts/*`, `mkdir -p`, `sed -n`, `/usr/bin/time`, `arch -x86_64`,
                    `rustup target list --installed`; never `python3 -c`, `sed -i`, `awk`, `xargs`,
                    `curl`, or `cat >>` / heredoc writes; no edit of `.claude/settings*.json` or other
                    worktrees (12d's is read-only; the scratch merge is its own /tmp worktree)
VALIDATION BUDGET   12n-1: four 300-day town runs by hand (NV-1: social-cafe and market-town, base and
                    final head); bodies-yard 30-day runs and committed tests unrestricted; NV-5's 2 000
                    scenes; NV-10's timing build once; the x86_64 build once and its Rosetta runs; one full
                    gate on the final head.
                    12n-2: on the scratch merge, NW-4's TD-12a set (8 interleaved 300-day runs, +4 if one
                    town is contaminated and re-run once) and one per-resolution timing run, counted
                    separately; on main, NW-4's no-bodies pair (2 runs) — plus at most 4 other 300-day
                    runs (debugging, re-baseline evidence): hard cap 8 + 4 + 1 + 2 + 4 = 19; NW-9's hosted
                    runs at 1×, 6×, 12×, 24×, at most two attempts per scale (each ≤ 3 wall minutes);
                    30-day diagnostic runs unrestricted. Reaching a cap is a material stop. Real-model
                    NOT REQUIRED.
LIVE DOCUMENTATION  §21.12 / §21.13 checkboxes; §21.15
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for 12n at NV-C1 (and again for
                    12n-2)
ENDPOINT AUTHORITY
  implementation + local validation   authorized by the freeze, for a fresh session
  semantic commits, branch push       authorized
  PR creation / update                authorized
  scratch builds and the scratch merge authorized, under /tmp/s15-12n; no branch pushed from them
  CI repair                           authorized; `fast` and `test` green on the PR's exact head,
                                      reported with the local evidence; a known flake (F-12n-CI1) is
                                      re-run once and recorded, never "fixed" here
  merge                               operator only, with a merge commit; never inherited
POST-MERGE SYNC     the planning session owns the step header, overall and MVP_STATUS's S15 lines; the
                    implementing session owns §21's ledger and evidence rows
NORMAL STOP         PR 12n-1 (then 12n-2) READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       any change to presence, the kernel, contracts or bodies' resolution rules; any pack or
                    controller reading the time scale or a wall clock; an edit outside the approved
                    paths; NV-1 not byte-identical; NV-7 failing at R 250 with derived constants (QN-5);
                    NW-9 failing; NW-1 / NW-2 failing (no remedy is pre-approved — only NV-10's grid
                    fallback, for cost); any digest change in 12n-1; cross-architecture digests
                    differing; reaching a budget cap
```

## 21.15 Evidence ledger, deviations and findings (live)

```text
F-12n-CI1  (finding, owned by the S11 lane — raised there by the coordinator, 2026-10-09)
           tools/cli/tests/hosted_town.rs `the_hosted_town_lives_within_its_tick_budget` failed on PR
           #100's head f64cf01, a Markdown-only change: "the p99 tick took 57.3 ms, over 50 ms (p50 0.3
           ms, max 198 ms)" (CI run 37977525973, job 113979682214). The failed job re-run once on the
           same head passed (job 113983484403). Reading: a wall-clock tick budget measured on a shared
           CI runner is sensitive to load. Not 12n's to fix; 12n's sessions re-run it once if it recurs
           and record the run ids here.
```


