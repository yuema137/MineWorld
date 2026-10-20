# PR S26-c-pose — a natural standing pose from the mesh's own palm, a recorded idle and anatomy

**Lifecycle:** `DESIGN FROZEN — 2026-10-19 (operator approved in the primary session 2026-10-19: every open question answered as recommended) — the approach is the operator's ruling of 2026-10-19 (relayed by the coordinator):
approve the plan in mineworld-demos/research/natural-pose-tools.md; awaiting the coordinator's freeze`.
**Parent:** [`step-26-cast.md`](step-26-cast.md) §§6, 9 · **Base:** `origin/main @ 1308db6d` (#306, #319 and #321
merged) · **Lane:** S26-c · **Worktree:** `/Users/yuema137/mineworld-worktrees/impl-s26c-pose`, branch
`mvp0/pr-s26c-natural-pose` · **Credits:** none (no Meshy, no image calls).

## 1. The problem, measured

The operator (2026-10-19): the cast's standing pose is unnatural. Some hands reach forward, and some palms face
outward, which no human arm can do. Before this PR, the arms came from a search: `tools/character_pipeline/godot/
stance.gd` tried abduction, forward swing, elbow bend and forearm twist per body (PIPELINE §15.2, CC-10). The
operator rejected that approach: hand-tuned pose numbers and searches are not robust.

The lead measured all 22 bodies on #319 and on main (`cast-scratch-s26c/review/palm-diagnosis.md`). The palm
normal comes from a plane fit over the skinned hand and finger vertices, signed toward the thumb side. On Vera's
free hand it reads 8°, which matches her frames.

1. **Bind error, the root cause the research named.** `fit_bind.py` lines 205–212 move the CharMorph hand chain
   onto the wrist→fingertip direction with a shortest-arc `rotation_difference`. Line 249 keeps the imported
   rolls. The hand's twist about that axis therefore stays the reference body's own. On 5 of 21 cast bodies the
   mesh's palm is turned 63–77° about the forearm away from the finger-joint plane: Alba, Alice, Dev, Greta and
   Ines (left −64…−75°, right +63…+77°). It is mirror-symmetric, so it is not a left/right sign error. On these
   same five, and only these, no vertex is weighted to a thumb distal bone, because the thumb joints lie outside
   the mesh's thumb. Every other body is within ±30°. The source rolls vary between bodies, but they do not
   separate these five from the rest.
2. **The search made it worse.** The search's palm test could not tell a palm from the back of a hand (180°),
   and it had no joint limits. It twisted forearms by up to ±135°, so palms faced outward on main (for example
   Celia right 76°, Elin right 122°, Ivan right 81°). It also scored a hand ahead of the hip as clearance, so
   hands hang 147–434 mm ahead of the hip.

## 2. Scope (operator ruling, items 1–5)

| # | Change | Where |
| --- | --- | --- |
| A | **Fix at bind.** Measure the mesh's palm frame from the hand vertices fit_bind already isolates: the thin PCA axis is the palm normal, and the thumb side signs it. Set the forearm and hand rolls with `EditBone.align_roll` so the bone's palm axis equals the measured normal. Split the twist between forearm and hand, as the radius and ulna do. Place the finger and thumb joints in that frame, so the thumb chain lies inside the thumb. Then re-run the finger weights. Delete the shortest-arc-only alignment. | `blender/fit_bind.py`, `blender/weights_fingers.py` (unchanged unless needed) |
| B | **G-5 revised.** The forearm, hand and finger rolls now deliberately differ from the reference. G-5 keeps names, parents, weights and the other bones' rolls. For the hand chain, it checks instead that the bone palm axis agrees with the mesh palm normal within 15° at rest. That bound is a literal in `standard.py`, with its source. Vera's re-derivation parity (#211) stays a G-5 regression: her bind error is 8°/7°, so her bones are re-derived with the new rule and the parity bounds are re-baselined, with the change recorded. | `mwcast/gates.py`, `standard.py`, `PIPELINE.md` §6 |
| C | **Recorded idle.** Adopt the 100STYLE `Neutral` idle (Mason, Starke & Komura 2022, CC BY 4.0). Its attribution goes in `clients/3d-spike/ASSETS.md`, which drops the old "100STYLE excluded on licence" line, and in a LICENSE file beside the clip. Convert it offline: BVH → Blender's BVH importer → map to `mixamorig_*` → bake → trim → `assets/characters/stand_lib.glb`. Root translation is stripped and the loop is closed. Fallback, if the Neutral idle is not relaxed or has artefacts: a Quaternius UAL CC0 idle, recorded as such. Decision record **DEP-146** (next free id; main and every open PR end at DEP-145) adopts 100STYLE; the operator overrides the earlier exclusion. | `clients/3d-spike/assets/characters/`, `tools/animation_trim.py` (reuse), `docs/decisions/DEP-146.md` |
| D | **Standard retarget.** The idle imports through Godot's `BoneMap` + `SkeletonProfileHumanoid` + rest fixer (`overwrite_axis`), the same path as every cast body and the UAL clips. `Human` drives `Stand` from that clip for every body, cast and Vera alike (Vera's strap grip stays an additive layer on her left arm only). Finger curl comes from data, the clip or a fixed relaxed curl, not from a search. `stance.gd`'s solver, the manifests' `stance` blocks and the code that keys them are **deleted**, not layered on. | `clients/3d-spike/scripts/human.gd`, `posture.gd`, `tools/character_pipeline/godot/stance.gd` (deleted), manifests, `cast/index.json` |
| E | **Gates on every frame of the stand loop.** **ROM:** shoulder, elbow, forearm and wrist angles, decomposed into swing and twist about the profile axes, must lie inside AAOS normal ranges (Greene & Heckman 1994). Elbow and forearm also use Zwerus et al. 2017; shoulder and elbow cite CDC Soucie et al. 2010/2011 where it is stricter. **Palm:** the skinned palm normal faces the same-side thigh within ±30°, with forearm supination 0 (the thumb ahead of the palm). **Reach:** the fingertips stay behind the thigh's front surface. **Clearance:** CC-10 stays (≥ 20 mm), as a gate only. The ground gate (§15.1) also runs over the loop. The probe measures; `check_cast.py` re-checks the recorded numbers in CI. A miss fails the build. Nothing is tuned to pass. | `pose_sheet.gd` / `cast.py ground` probe, `scripts/check_cast.py`, `PIPELINE.md` §15 |
| F | **All 22 bodies, both clients.** Re-bind the 21 cast bodies from their kept downloads and committed `fit.yaml`: zero credits, the same picks, the new A. Re-derive Vera the same way. Re-export the bodies, which are LFS objects, and re-render the 2D sprites from the posed 3D bodies, so 2D and 3D stay one pose. Rebuild the review page with before/after standing frames (front and side) and the gate table. | `clients/3d-spike/assets/characters/cast/`, `presentation/mineworld-default/2D/…`, review page |

**Not in scope:** walk and jog clips (unchanged, but the ground gate still runs on them); wardrobe (MVP-0 is
protagonist-only); new characters.

### 2.1 Revision of part A — R1 (operator ruling 2026-10-19; a recorded deviation)

**Evidence.** Godot's rest fixer (`overwrite_axis`) overwrites the bind-time hand rolls. The imported rest basis of
every body's `LeftHand`/`RightHand` is identical in Godot (left: x = (0,0,−1), z = (0,−1,0)) across all 22 bodies,
although the exports' glTF rolls differ by up to 95° (Dario's hand roll −8°, Vera's +89°;
`cast-scratch-s26c/palm-diag/out3/palm.json` and `rolls.txt`). An `align_roll` fix would therefore never reach the
client. `RetargetModifier3D` (§6's fallback) would not help either: a twist about the bone is the same angle whatever
the roll, and a global-pose transfer carries the mesh's rest palm with it.

**Ruling (R1).** At bind, `fit_bind.py`:
- measures each hand's palm frame by PCA of the hand vertices;
- turns the hand vertices about the forearm axis into the template's (the humanoid profile's) convention before
  skinning, applying the pose as rest, the turn split half forearm, half hand;
- keeps the finger and thumb joints in that frame;
- re-weights the fingers (stage 8b).
Applied to every body through the same path.

**Bounded.** It changes part A's mechanism only. Rolls stay as imported, so G-5's roll check is unchanged. G-5 adds
the palm-alignment check (15°). The rest of the design is unchanged.

## 3. Invariants

- I-1: no hand-tuned pose numbers and no per-body search. Each body's pose comes from its mesh (A), from the
  recorded clip (C/D) and from the standard retarget. The only literals are gate bounds, each with a citation.
- I-2: no per-name code; everything is data and generic code.
- I-3: the ground gate keeps its name and structure for S36-P2 (G-4w).
- I-4: licences: the 100STYLE attribution travels with the clip; nothing from SMPL/AMASS-style
  redistributions; no GPL add-on code is vendored.

## 4. Acceptance (each shown failing when broken)

- AC-1: the palm gate passes on all 22 bodies on every stand frame. Mutation: a body bound with the old
  shortest-arc rule fails it on Alba, Alice, Dev, Greta and Ines.
- AC-2: the ROM gate passes. Mutation: a forearm twisted 120° fails, naming the joint and the range.
- AC-3: the reach gate passes. Mutation: a shoulder flexed 25° fails.
- AC-4: CC-10 and the ground gate pass over the loop.
- AC-5: `stance.gd` and every `stance` block are gone (grep finds nothing).
- AC-6: the 2D sprite of each body is rendered from the same posed 3D body (provenance names the clip).
- AC-7: the operator's review page shows before/after for all 22.

## 5. Commit plan

| Commit | Implementation | Validation | Review |
| --- | --- | --- | --- |
| C1 bind-time palm frame (A) + G-5 revision (B) | [ ] | [ ] AC-1 mutation; the palm table on 22 re-bound bodies | [ ] |
| C2 100STYLE Neutral idle → `stand_lib.glb`, DEP-146, ASSETS/LICENSE (C) | [ ] | [ ] the clip plays on the town body and on Vera | [ ] |
| C3 `Stand` from the clip; delete the stance solver (D) | [ ] | [ ] AC-5 | [ ] |
| C4 ROM, palm and reach gates on every frame; CC-10 kept (E) | [ ] | [ ] AC-2, AC-3, AC-4 | [ ] |
| C5 re-bind and re-export the 22 bodies; 2D sprites; review page (F) | [ ] | [ ] AC-6, AC-7; `identity.sh`; fast layer | [ ] |

## 6. Risks

- `overwrite_axis` may rewrite the rolls fixed in A. Mitigation: AC-1 runs on the imported skeleton. If it
  fails, the cast switches to `RetargetModifier3D`, which keeps rests. That would be a recorded deviation,
  brought back to the coordinator.
- The 100STYLE Neutral idle may not suit broad bodies (one performer). Mitigation: the clearance gate fails
  visibly. A residual is reported, never tuned away. The fallback clip is listed in C.
- The re-bind changes Vera's export bytes. Mitigation: her parity is re-baselined and recorded, and her frames
  are compared on the page.
