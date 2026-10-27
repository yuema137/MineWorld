## DEP-77 — Lakeside's terrain: Godot's built-in `ArrayMesh` and `HeightMapShape3D`, not a terrain addon

**Date** 2026-10-21 · **Status** accepted (step-33 §4.9, approved 2026-10-15; re-verified by S33-d C1) ·
**Relates to** `ARC-177`, `ARC-170`, [`REUSE_POLICY.md`](../REUSE_POLICY.md) · **Design**
`.structured-coding/plans/mvp0/step-33-lakeside-3d.md` §4.1, §4.9; `pr-s33d-terrain-water-exterior.md` §4

**Problem.** Lakeside's near field (≈ 200 × 200 m at 1 m cells) needs one heightfield, a splat of at most four
layers and a collision shape, on macOS, Linux and Windows, built from the world's depiction file.

**Options, re-verified 2026-10-21.**

| Candidate | Fact (source) | Verdict |
| --- | --- | --- |
| Terrain3D (TokisanGames) | MIT; "Written in C++ as a GDExtension addon"; "Supports Windows, Linux, and macOS"; release 1.0.2 lists Godot 4.4–4.6 and "may work with later versions" — Godot 4.7 is not listed (github.com/TokisanGames/Terrain3D; terrain3d.readthedocs.io installation page) | rejected: a GDExtension (no native binary per platform in this repository) with no stated Godot 4.7 support; its strength (large sculpted worlds) is not needed for a 200 m field |
| HTerrain (Zylann `godot_heightmap_plugin`) | MIT (LICENSE.md, © 2016-2020 Marc Gilleron); the author describes it as not in active development, bug fixes only; README targets Godot 4.6+ | rejected: maintenance mode, editor-sculpted data rather than a data file the validator reads |
| Godot built-ins (`ArrayMesh` / `SurfaceTool`, `HeightMapShape3D`, `Geometry2D`, `Curve`) | in the engine on every platform | **adopted** |

No fact reverses the step's decision. **Consequences.** The generic `DepictionTerrain` builder rasterises the
grid from the depiction file at load, deterministically; no dependency or GDExtension is added.
