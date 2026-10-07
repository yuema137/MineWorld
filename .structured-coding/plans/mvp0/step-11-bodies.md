# Step 11 — Bodies and physical interaction (S15)

**Role:** step document for a new step, proposed as **S15 — Bodies and physical interaction**. It
records the operator's requirement, the audit, the answers to the planning questions, the Rapier
determinism prototype, the proposed ownership, facts and contracts, a PR split, risks and open
questions. It holds no frozen PR design yet: the first PR is detailed only after the operator decides
the material questions in §12 (`CLAUDE.md` §3, "detail one step ahead").
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S11, S12, S14), §7.
**Lifecycle:** `DRAFT — awaiting operator review`. Nothing in this document is frozen, and nothing in
it authorizes implementation.
**Branch:** `design/physics` from `main @ 7f6960e`. Worktree `/Users/yuema137/mineworld-worktrees/physics`.
**Prototype:** `/Users/yuema137/mineworld-demos/physics-spike/`, a standalone Cargo project outside the
repository, never committed to MineWorld (§8).

## Why the file is `step-11`, and the step is S15

Step files are numbered in the order they were written, not by step: `step-05` is S5V and `step-06`
is S5. The next free file number is 11. The step itself is new, so it takes the next free step
number, **S15**, after the fourteen steps of `overall.md` §3. Its number says nothing about its
order; §11 places it in the sequence.

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

---

# 8. The determinism prototype

The prototype answers one risky assumption before any architecture is built on it
(`REUSE_POLICY.md` §15): **can Rapier's results be reproduced byte for byte across two runs, across a
snapshot and a resume, and is the comparison able to see a difference at all?** Without that, a
physics System Pack cannot satisfy `ARC-25`, under which a resumed world re-executes its journal and
must regenerate every logged fact byte for byte.

## 8.1 What it is

A standalone Cargo binary at `/Users/yuema137/mineworld-demos/physics-spike/`, depending on
`rapier3d 0.36.0` with the features `enhanced-determinism` and `serde-serialize`, plus `bincode 2`,
`serde` and `sha2`. It depends on no MineWorld crate. It is never committed to MineWorld; its source
is recorded in §8.5.

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

It runs in two modes, because the design (§5) must choose between them:

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

## 8.2 Criteria, stated before any run (`ARC-23`)

Each criterion names its measurement, its bound and its verdict rule. Bounds are literals fixed
here, never derived from the quantity measured (`ARC-23` point 2). A run that has not happened is not
a pass (`CLAUDE.md` §3.1).

| ID | Claim | Measurement | PASS iff |
| --- | --- | --- | --- |
| **PC-a** | Same inputs, same bytes. | Run each mode twice, **as two separate processes**: P for N = 6 000 steps (100 s of simulated time), Q for R = 3 000 requests. Print the SHA-256 of the final serialized state. | Both processes print the same digest, for each mode. |
| **PC-b** | Snapshot, drop, restore, continue = uninterrupted. | P: write the serialized PhysicsWorld at step K = 2 500 to a file and exit; a **new process** reads it, restores, and continues to N. Q: the same with the integer state at request K = 1 300, continuing to R. | The resumed digest equals the uninterrupted digest from PC-a, for each mode. |
| **PC-c** | The comparison can see a difference. | Three deliberately different variants, each run as its own process: **c1** the people inserted in reverse order; **c2** one prop's initial x moved by 1 mm (P: 0.001 m; Q: 1 mm); **c3** one input of request/step 1 000 changed by 1 mm. The instrument must also **locate** the difference: print the first body whose state differs and by how much. | c2 and c3 each produce a digest different from PC-a's, and the locator names a body. c1 is a finding either way: Rapier's documentation says insertion order is part of the initial conditions, so a difference is expected in P; whatever is observed is recorded, not tuned. If c2 or c3 shows **no** difference, the instrument is broken and every other result in this section is void. |
| **PC-d** | What it costs. | Release build, this machine (Apple silicon). P: mean wall time per step over the N steps. Q: mean wall time per request (build + move + settle + quantize) over the R requests. | Reported, not passed or failed, against two stated budgets: P is ruled out for the headless server if stepping one café at 60 Hz through 300 simulated days (1.555 × 10⁹ steps, because a stepped world cannot skip idle time the way the discrete-event clock does, `DEP-6`) would cost more than **60 s** of wall time, i.e. more than **38.6 ns per step**; Q is acceptable if its mean is at most **100 µs per request**, which adds at most 17 s to a 300-day run at the measured movement volume (§9.3). |
| **PC-e** | Bodies do not interpenetrate; a person pushes a box. | e1: two capsules walking straight at each other along one line, 1.45 m/s each, for 4 s, in both modes. e2: one person walking straight into a 0.4 m box, in both modes. | e1: the smallest centre-to-centre distance in the plane over the whole run is at least **2 × 0.30 m − 0.005 m = 0.595 m** (overlap of at most 5 mm). e2: the box's final position is at least **0.100 m** further along the walking direction than where it started, and the person never overlaps it by more than 5 mm. |

Two further checks are listed because they cost nothing to state and bear on `AC-8`; they are not
part of the operator's five:

| ID | Claim | PASS iff |
| --- | --- | --- |
| **PC-f** | Optimization level does not change the result. | The digests of PC-a are identical when the binary is built with the dev profile at `opt-level = 1` (what `ARC-30` makes the workspace's dev profile) and with the release profile. |
| **PC-g** | Another architecture gives the same bytes. | The same digests from an `x86_64-apple-darwin` build run under Rosetta. Requires installing that target, which this session is not permitted to do (`rustup` is outside its allowed commands), so PC-g is expected to be `NOT RUN` and is recorded as such. |

## 8.3 Results

Pending: recorded after the runs, with the exact output.
