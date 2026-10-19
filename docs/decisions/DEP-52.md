## DEP-52 — Blender, headless and pinned, as the mesh runtime; no Blender in CI

**Date** 2026-10-14 · **Status** accepted (design, step-26) · **Approved by** the operator's step-26 rulings and Q-B1-4 (2026-10-14) · **Relates to** `DEP-8`, `ARC-9`, `ARC-94`, `DEP-55` · **Design** step-26 §§5.1–5.2; `pr-s26b1-pipeline.md` §§5.10, 5.13

**Problem.** Mesh decimation, bone fitting, weight painting, texture baking and export need a DCC tool that runs headless and is scriptable.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Blender, pinned to `Blender 5.2.2 LTS`, run headless with `--factory-startup`, scripts in `tools/character_pipeline/blender/` under GPL-2.0-or-later | **Chosen** |
| (b) A pinned Blender in CI | Declined: the Meshy download and the accepted intermediates are outside the repository, so CI would have nothing real to run on; the bake is device-dependent (Cycles on Metal against a CPU runner cannot be compared texel for texel); a several-hundred-megabyte download per OS per run for a stage that changes only with `blender/`; CI owns the stdlib part (`§5.10`) |
| (c) A Python mesh library (trimesh, pygltflib) for the stages | Declined: the D+ stages use Blender's bake, weight-painting operators and glTF export; a port to a library would reimplement them and the parity claim would then be about a different algorithm |

**Choice.** Option (a). The licence split is explicit: the driver is MIT and standard-library; the Blender scripts are GPL-2.0-or-later with their own `LICENSE` (byte-identical copy of `clients/3d-spike/tools/blender/LICENSE`), and they are never imported, only run as subprocesses. Blender's licence page (read 2026-10-14) states that "What you create with Blender is your sole property", so the outputs are ours. The pinned version is checked at run start; a different version needs `--allow-blender` and is recorded as a new parity baseline. The Blender stages are proven by the local real-lifecycle Gate on macOS (B10), re-run whenever `blender/` changes; CI is not asked to run Blender.

**Consequences.** Q-B1-4 is recorded here: no Blender in CI, for the four reasons of `pr-s26b1-pipeline.md` §5.10. Blender's bake device is recorded in every run manifest.

**Revisit** when a CI runner can hold the inputs without the operator's download, or when a second device needs a parity baseline.
