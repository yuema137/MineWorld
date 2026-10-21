## DEP-78 — Lakeside's near water: the slice's own `water.gdshader`, extended

**Date** 2026-10-21 · **Status** accepted (step-33 §4.1, §4.9, approved 2026-10-15; re-verified by S33-d C1) ·
**Relates to** `ARC-177`, `ARC-170`, `ARC-102` (the far field's water), [`REUSE_POLICY.md`](../REUSE_POLICY.md)
· **Design** `.structured-coding/plans/mvp0/pr-s33d-terrain-water-exterior.md` §5.3

**Problem.** The near lake must continue the far field's water sector at the same level and colours, fade with
depth at the shore and round the pier posts, and carry lily pads in the shallows — no flow, no foam simulation.

**Options, re-verified 2026-10-21.**

| Candidate | Fact (source) | Verdict |
| --- | --- | --- |
| "Stylized Water for Godot 4.x" (godotshaders.com/shader/stylized-water-for-godot-4-x) | code MIT per the page's listing (images excluded); Godot 4 | rejected: a stylised look beside the slice's PBR palette; adopting it means a second water look for Social Café |
| "Water shader 3D (Godot 4.3)" (godotshaders.com/shader/water-shader-3d-godot-4-3) | Gerstner waves, caustics, foam; licence not read (INCONCLUSIVE) | rejected: an ocean model; the brief is a still mountain lake |
| ours: `clients/3d-spike/shaders/water.gdshader` | 65 lines, already the slice's water | **adopted, extended** with `depth_fade_m`, `shore_foam` and a lily mask, each behind a uniform whose default leaves Social Café's water unchanged |

Reflections: one `ReflectionProbe` over the near lake plus the sky; SSR off and measured once; planar
reflection rejected (step-33 §4.1). No fact reverses the step's decision.
