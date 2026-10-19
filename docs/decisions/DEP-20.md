## DEP-20 — Client collision: Godot's built-in Jolt Physics, never authoritative

**Date** 2026-10-08 · **Status** selected; in force from S14 PR 16a · **Approved by** the operator
(step-11 §1.2 D-2, Jolt as the direction) and the primary session at PR 16a's design freeze (step-15
§19.0) · **Relates to** `DEP-4`, `DEP-13`, [`NETWORKING.md`](NETWORKING.md) §4,
[`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §12 · **Design** `.structured-coding/plans/mvp0/step-11-bodies.md`
§§3.2, 15.2 (drafted there as "DEP-14", renumbered by `overall.md` "Parallel build-out, 2026-10-08",
ruling 6); `step-15-demo-3d.md` §§8.1, 19

**Problem.** The 3D client must stop the player at walls and, from S15 PR 12e, at people and objects,
predicting locally what the server decides (`NETWORKING.md` §4). It needs a collision engine for its own
body; nothing it simulates is authoritative.

**Options considered** (`REUSE_POLICY.md` §§11–12, §17 — both directions).

```text
(a) Jolt Physics, built into Godot 4.7.2, MIT          chosen
(b) Godot Physics (what ran before; the setting's      kept as the one-line fallback
    DEFAULT in this project)
(c) appsinacup/godot-rapier-physics (GDExtension)      symmetry with the server buys nothing when the
                                                       server always wins; a binary per platform
(d) godot-jolt/godot-jolt (the former extension)       maintenance mode; supports Godot 4.3-4.6 only
(e) our own collision                                  a physics engine is commodity infrastructure
```

**Choice: Jolt Physics, selected explicitly** — `[physics] 3d/physics_engine="Jolt Physics"` in
`clients/3d-spike/project.godot`, every other Jolt setting at its default. `CharacterBody3D.move_and_slide`
is the only physics the player uses, unchanged. Jolt has been built into Godot since 4.4 and is the
default for new projects since 4.6; existing projects keep Godot Physics unless they select it.

**Why not ourselves.** Collision is commodity infrastructure (`REUSE_POLICY.md` §4), and the engine
already ships two.

**Isolating interface.** The project setting is the only place that names Jolt. Every script uses the
generic `PhysicsServer3D` nodes and queries, so the fallback to Godot Physics is that one line
(step-15 §9.4). No physics concept reaches the wire protocol: colliders are built from disclosed shapes,
and the client's only rule is to adopt the server's answer (12e).

**Facts, measured 2026-10-08** (step-15 §19.7 E16a-1, E16a-2; Godot `4.7.2.stable.official.ed1daf0bf`,
Apple M5):

```text
which engine   the server singleton reports the abstract PhysicsServer3D under either engine; the
               running engine is told by a new space's solver iterations: 16 with Godot Physics
               (its own setting), 8 with Jolt (clients/3d-spike/tools/physics_engine.gd). With the
               setting DEFAULT this project ran Godot Physics
the slice      every accepted check passes unchanged on Jolt: --drive (the loop closes within
               0.10 m, jumps 0.488 m, 0.0000 m camera switches), --threshold, --character,
               --world --link (50 moves accepted, none refused), --world --conversation; the body
               settles within 2 mm of where it settled on Godot Physics
frame time     within run-to-run noise on both scenes (step-15 §19.7 E16a-2)
```

**Accepted limitations.** Godot states its physics, on either engine, is not deterministic; acceptable,
because nothing the client simulates is authoritative. Jolt's documented differences from Godot Physics
(position-only stabilization, kinematic bodies not reporting contacts with static or kinematic bodies
unless `generate_all_kinematic_contacts` is set) are re-checked by 12e, which first gives the player
bodies to meet.

**Revisit** if a client check fails on Jolt in a way the client cannot fix (step-15 QS14-12): fall back
to Godot Physics only with the operator, and amend this record.
