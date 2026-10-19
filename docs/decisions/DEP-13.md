## DEP-13 — Server physics: Rapier (`rapier3d`, `enhanced-determinism`) inside the `bodies` pack

**Date** 2026-10-07 · **Status** selected; dependency added in S15 PR 12b · **Approved by** the
operator (step-11 D-1 … D-5, QB-4, QB-12) and the primary session at PR 12b's design freeze (step-11
§17.0: QP-3 overruled, so the pin lives in the pack's own manifest) · **Relates to** `ARC-25`,
`ARC-30`, `ARC-33`, `ARC-39`, [`MVP.md`](MVP.md) §9 `AC-8`, `AC-12` · **Design**
`.structured-coding/plans/mvp0/step-11-bodies.md` §§3, 6, 9, 15.1, 17 (S15, PR 12b)

**Problem.** People must not pass through each other or through walls, and later must push, kick and
throw objects (the operator, 2026-10-07). The answer is decided on the server and must replay byte for
byte (`ARC-25`), on more than one machine (`AC-8`). Sweeping a body against walls and other bodies, and
later integrating a kicked object until it rests, is a physics engine's work.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions).

```text
(a) rapier3d (dimforge), Apache-2.0           chosen
(b) parry3d alone (Rapier's geometry crate)   no dynamics: kick and throw would need our own integrator
(c) our own circle-and-box code               a physics engine is commodity infrastructure
(d) Jolt through Rust bindings                bindings unmaintained since May 2024; cannot enable Jolt's
                                              cross-platform determinism
(e) Avian                                     requires Bevy's ECS and scheduler
```

**Choice: `rapier3d =0.36.0`**, with the feature `enhanced-determinism`, and without `simd8`,
`parallel` and `serde-serialize`. Declared **in `systems/bodies/Cargo.toml` only**, not in the root
`[workspace.dependencies]`: installing a pack touches only `systems/**` and `Cargo.lock` (`ARC-33`),
and the primary session chose that over the root's one-version convenience (step-11 §17.0, QP-3). No
other crate depends on it.

**Why not ourselves** (`REUSE_POLICY.md` §12). Kick and throw need integration, contacts, friction and
rest, which is a physics engine (`REUSE_POLICY.md` §4). Ours would also have to earn the cross-platform
determinism Rapier documents and the prototype measured (step-11 §§9.4, 9.8).

**Why not the others.** `parry3d` alone has no dynamics (missing required semantics). The Jolt bindings
are unmaintained and cannot enable Jolt's determinism (unmaintained project). Avian would put the
simulation inside Bevy's execution model (architecture mismatch, unacceptable lock-in).

**Isolating interface.** `systems/bodies/src/rapier.rs` is the only source file that names
`rapier3d`, and a structural test holds that. It takes integers in — a place's floor and solids, the
people standing in it, one sweep request — and gives integer millimetres out. No Rapier type appears
in a component, a fact, a contract, a public signature or another crate. Nothing of Rapier survives a
call: every resolution builds a fresh world for one place, in a canonical insertion order, and drops
it. Every float is converted from integers by one function and quantized back by one function,
`(metres × 1000).round()` to `i32`.

**Facts, re-verified 2026-10-07** (the crate's own `Cargo.toml` in the local registry, and crates.io,
the licence file, the determinism guide and the changelog):

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
parry3d      ^0.31.1 required; 0.31.1 newest (2026-09-18), Apache-2.0, no MSRV declared; 0.31.0 yanked
determinism  cross-platform needs `enhanced-determinism` and IEEE 754-2008 targets; it cannot be
             combined with `simd8`; inputs computed with functions beyond + − × ÷ must use nalgebra's
             ComplexField/RealField. MineWorld computes every direction and length in integers instead.
```

**Accepted limitations.**
- **Monthly breaking releases** (0.33 to 0.36 in four months). The pin is exact. An upgrade is a change
  of results, so it bumps `bodies`' `VERSION` and an old save is refused by name rather than diverging
  on replay; a test holds the pack's version and the locked Rapier version together.
- **A defect in 0.36.0, worked around** (step-11 F-P1). `PhysicsWorld::detect_collisions` on a fresh
  world leaves every dynamic body un-integrated by the next step. The cause is upstream behaviour
  (changelog v0.35.0: "`CollisionPipeline::step` now clears the rigid-bodies' modified flags"). The
  adapter re-marks each dynamic body after detecting collisions; a canary test asserts the defect as it
  is, so an upstream change turns it red and the workaround is then removed deliberately.
- **The character controller is not an interpenetration guarantee** (step-11 F-P6). The pack verifies
  the non-overlap invariant on integers after every resolution and degrades the result (`ARC-39` note).
- **Cross-architecture identity is evidenced under Rosetta only** (arm64 and x86_64 translated, on one
  machine, step-11 §9.8 and PR 12b's PB-12). A native x86_64 host (S13) is the stronger check.
- **No momentum between resolutions.** Everything is at rest between requests; a kicked object's whole
  motion is resolved at the instant of the kick (S15's later PR).
- **The licence tree is permissive**: every package Rapier brings, with these features, is Apache-2.0,
  MIT, Zlib, Unlicense or Unicode-3.0, each alone or as one of a permissive choice (recorded in step-11
  §17.10).

**Note, 2026-10-08 (S15, PR 12c; step-11 §18.0, SD-O14, SD-O16) — dynamics are now used.** `kick` and
`throw` simulate one object's flight at the instant of the request (step-11 QB-6): the first stepped
simulation in MineWorld. The isolating interface is unchanged — integers in, integers out, nothing
kept, every float converted by the same two functions — and what the flight adds is bounded:
- **One dynamic body per flight.** A flight scene holds the place's fixed geometry, the people as
  kinematic capsules in `EntityId` order, the other objects as fixed colliders in `ItemId` order, and
  the flying object last: dynamic, rotations locked (no rotation is ever persisted), with continuous
  collision detection so a fast object cannot pass a thin solid. Friction 0.5, restitution 0.1.
- **Bounded steps and a rest rule.** Sub-steps of 1/60 s, at most 180 for a kick and 240 for a throw;
  the object has come to rest once it has been slower than 50 mm/s for 10 consecutive sub-steps. Both
  are pure functions of the state.
- **F-P1's re-mark is exercised.** The adapter's `refresh` runs after the scene is built and before the
  first step; without it the flying object would never move (the canary above).
- **The engine's answer is never the last word.** The landing is quantized and verified on integers
  against the stored-state invariant of objects; if it fails, the nearest verified point of a 50 mm
  lattice is taken, and if none verifies, the object stays where it was.
- **Launch velocities are integers** (millimetres per second, computed with + − × ÷ only), converted
  into Rapier's metres by the adapter's one conversion.

**Note, 2026-10-08 (S15, PR 12d-0; step-11 §20, SD-Z2, SD-Z5) — what of Rapier a people scene now
uses.** A scene of people builds Rapier's broad phase and nothing else (`BroadPhaseBvh::update`, the
update `CollisionPipeline::step` makes, without the narrow phase): no query of a sweep reads contacts.
A `Pile` and a flight keep the full collision detection and F-P1's re-mark. A contact sweep may leave
named people out by a query predicate (SD-Z5); a sweep that leaves nobody out builds exactly the filter
it built before. Two properties of the character controller were measured and are now part of this
decision's record: it is not local (its end moves by up to 14 mm when colliders far from anything the
stride touches are removed from the scene), and it is not exact at its own offset from a face (a stride
starting at or gliding along it may stop up to 1.6 m short, or drift 1–2 mm in open floor) — step-11
E-Z1, E-Z2, E-Z5. A pruned scene or an integer replacement of the sweep therefore changes results.
