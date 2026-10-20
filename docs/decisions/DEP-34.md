## DEP-34 — Route search: `pathfinding`'s A*, over a visibility graph `bodies` builds on integers

**Date** 2026-10-09 · **Status** selected; one dependency added to `systems/bodies` only · **Approved by**
the primary session at PR 12n's design freeze (step-11 §21.0, QN-4) · **Relates to** `ARC-25`,
`ARC-39`, `ARC-55`, `ARC-75`, `DEP-13`, `REUSE_POLICY.md` §§11–12, §17 · **Design**
`.structured-coding/plans/mvp0/step-11-bodies.md` §21.4 (S15, PR 12n-1)

**Problem.** `bodies` must answer "how does a person get from here to there in this place" round the
place's solids and loose objects, with an answer that becomes positions in the log: exact, and the
same on every machine (`ARC-25`, `AC-8`), with no C++ toolchain (Windows) and no engine bound in. A
place holds at most 64 solids and 32 objects, all axis-aligned boxes or discs.

**Options considered** (step-11 §21.4; crates.io and the projects' pages, 2026-10-09; both directions of
`REUSE_POLICY.md`).

```text
search
  (a) pathfinding 4.16.0 (evenfurther), MIT / Apache-2.0 — A*, Dijkstra, BFS, fringe, IDA*, Yen;
      generic over node and integer cost; since 2016, ≈ 3 M downloads; MSRV 1.88       chosen
  (b) our own A*                                       a mature, small, generic search exists
graph
  (c) pathfinding's Grid at 50 or 100 mm               110 k–440 k cells on the street; zig-zag paths
                                                       needing string-pulling against the same boxes;
                                                       a second discretization. Kept as the fallback
                                                       if the graph's cost fails (step-11 NV-10)
  (d) landmass 0.9.2 — a navigation system with        its own agent loop, velocities and frame delta;
      steering and avoidance                           f32 (glam), no determinism statement
  (e) oxidized_navigation 0.12.0                       a Bevy plugin: bound to Bevy's ECS and schedule
  (f) recastnavigation-rs 0.1.0 (Recast/Detour)        C++ on every platform; MPL-2.0 binding; floats
  (g) polyanya 0.17.1 — any-angle paths on a mesh      f32 triangulation (glam, spade), no determinism
                                                       statement; the reference for a later, larger
                                                       geometry
  (h) a visibility graph over the grown boxes, ours    chosen
```

**Choice: (a) for the search and (h) for the graph.** `pathfinding = "=4.16.0"`, an exact pin, in
`systems/bodies/Cargo.toml` only; `systems/bodies/src/route.rs` is the only file that names it.
The graph: every solid's footprint and every loose object's footprint (a ball as its bounding square),
and the person who stopped the walker when there is one, each grown on every side by `PERSON_RADIUS +
GAP + PLAN_MARGIN` (300 + 10 + 50 mm); the floor shrunk by the same. Nodes are the start, the goal and
every grown corner strictly inside the shrunk floor and outside every other grown box, in authored
order. An edge exists when its segment meets no grown box's open interior, decided with integer cross
products; its cost is ⌈√(dx² + dy²)⌉ mm and the heuristic ⌊√(dx² + dy²)⌋, admissible and consistent.
Edges are evaluated lazily as A* expands a node, successors in node order. A goal inside a grown box or
outside the shrunk floor is moved to the nearest free point of the 50 mm lattice (distance, then y,
then x — the entry order of `ARC-39` note 3); none, an enclosed goal, or more than `WAYPOINTS_MAX` (64)
waypoints is `unreachable`.

**Why the graph is ours** (rejecting a mature wheel needs a reason, `CLAUDE.md` §4 rule 16). Every
navmesh candidate is float geometry whose cross-machine bit-identity is unproven, and a route's
waypoints become positions in the log, so they must be exact (`CORE_CONCEPTS.md` §6.2). Rapier is
admitted only behind quantization and a verify-then-degrade check (`DEP-13`); a planner's output has no
such check to fall back on. For axis-aligned boxes the shortest path bends only at grown corners — a
graph of at most 386 nodes — so the case our geometry is made of is the one case where a visibility
graph is both optimal and small.

**Determinism.** Pure integer code in and out. `pathfinding`'s A* keeps its parents in an `FxIndexMap`
(a fixed hasher, insertion order) and orders its heap by estimated cost, then by cost, so equal inputs
give equal paths on every machine; `bodies`' successor order is fixed. An upgrade may change
tie-breaking and so results: it bumps `bodies`' `VERSION`, as a Rapier upgrade does.

**Licences** (`ARC-55`): `pathfinding` is Apache-2.0 or MIT, and so are the crates it brings
(`deprecate-until`, `indexmap`, `integer-sqrt`, `num-traits`, `rustc-hash`, `thiserror`), each recorded in
step-11 §21.15 as `Cargo.lock` admits them.

**Accepted limitations.** Grown boxes are squares, so a route keeps at least the margin from a corner
and more than it needs diagonally. A route is planned per request from current state, so its cost is
bounded by measurement (step-11 NV-10: a plan's maximum ≤ 5 ms, p99 ≤ 1 ms), with the grid as the named
fallback. Overhead geometry and slopes are out of scope (`body:` has none).
