## DEP-55 — Rigging: the fitted CharMorph armature is kept; the other rig sources are declined with reasons

**Date** 2026-10-14 · **Status** accepted (design, step-26) · **Approved by** the operator's step-26 rulings and Q-B1-2 (2026-10-14) · **Relates to** `DEP-8`, `DEP-51`, `DEP-52`, `ARC-94` · **Design** step-26 §5.4; `pr-s26b1-pipeline.md` §§2.2, 14.1

**Problem.** The protagonist's route rigged the mesh on a 52-joint CharMorph-derived armature (`vitruvian.glb`). The cast needs a rig source per character, and four others were considered.

**Options considered and verified facts (2026-10-14).**

| Option | Fact checked | Verdict |
| --- | --- | --- |
| Our fitted CharMorph armature (`vitruvian.glb`) with one recorded number (`charmorph_front_y_at_hips`) | D+ reproduced exactly (A-B1-9) | **Chosen** (`Q-B1-2`, option a) |
| Meshy rigging API as the rig | The rigging page (docs.meshy.ai/en/api/rigging) states no bone count, no finger bones, and no per-task price; it says rigging "currently only works well with standard humanoid (bipedal) assets" | **Declined as the rig.** Evaluated in S26-b2 as a joint source only (`step-26` §5.4 R) |
| Mixamo | Not fetched in B1; declined on the reason recorded in step-26 §5.4 | Declined (step-26 §5.4); re-verified before any use |
| AccuRIG (Reallusion) | Product page and CLI/SDK availability could not be read on 2026-10-14 (HTTP 404 and 403 from the fetcher); no command-line interface is verified | Declined as the rig; the decline does not depend on the CLI question. Re-checked before S26-b2 |
| UniRig (`VAST-AI-Research/UniRig`) | MIT licence; README requires a CUDA GPU with at least 8 GB VRAM and Python 3.11 with PyTorch >= 2.3.1 | Declined for the cast's CI-free local path and for the hardware floor; evaluated in S26-b2 only if its weight quality is shown |
| RigNet (`zhan-xu/RigNet`) | GPL-3.0 (`LICENSE-GPLv3`) | Declined: a GPL-3.0 dependency in the mesh path would bind the pipeline's licence; not used |

**Choice.** Our fitted armature is kept. **Amended 2026-10-14 (primary ruling F-B1-B10-2, option (b) of `Q-B1-2`):** the one recorded number is retired, because the CharMorph front surface is read from the committed CC0 body export `tools/character_pipeline/rig_source/body_g5.glb` (6.4 MB; SHA-256 `75b93806…`, provenance in `rig_source/PROVENANCE.txt`). The armature still comes from `vitruvian.glb`. S26-b2's weight-transfer prototype may still ask for the body's own weights under `DEP-53`.

**Consequences.** The rig owns a 52-joint skeleton whose bone names G-5 checks; Meshy's rig is not a joint source in this PR.

**Revisit** when S26-b2 measures a weight-transfer need, or when a rig source is shown to beat the fitted armature on the gates.
