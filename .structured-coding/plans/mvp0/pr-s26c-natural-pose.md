# PR S26-c-pose — a natural standing pose from the mesh's own palm, a recorded idle and anatomy

**Lifecycle:** READY FOR OPERATOR REVIEW (implementation, 2026-10-20) · was `DESIGN FROZEN — 2026-10-19 (operator approved in the primary session 2026-10-19: every open question answered as recommended) — the approach is the operator's ruling of 2026-10-19 (relayed by the coordinator):
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

### 2.2 Revision of part E — the forearm-rotation gate (operator ruling 2026-10-19, option (a); a recorded deviation)

**Evidence.** The first probe run measured the adopted recording (100STYLE `Neutral`) on Vera's free right arm,
worst over the Stand loop:
- palm 56.7° from facing the thigh;
- fingertips 38.7 mm ahead of the thigh front;
- joint angles all inside AAOS (abduction 2–3°, elbow 3°, pronation 48°).
The hand looks natural: beside the thigh, slightly pronated. Real mocap of a relaxed stand therefore sits outside
the design's palm ±30° and fingertips-behind-the-thigh bounds. Two instrument bugs were also found and fixed: the
clearance outline counted a carried backpack, and the palm plane took in the curled fingers.

**Ruling.**
- The palm-direction gate becomes a forearm-rotation gate: no supination (pronation ≥ 0), and pronation from the
  thumb-forward neutral within the relaxed band 0–60°, inside AAOS's 0–80° (Greene & Heckman 1994; relaxed band:
  operator ruling 2026-10-19 on the research note's neutral-to-slight-pronation hang).
- The palm angle, the reach and the CC-10 clearance are measured against the body only (carried items left out)
  and recorded per body, not gated, until the 22-body table is in.
- A reach bound is then proposed with a citation, and applied if published data and the table support it.

**Bounded.** It changes part E's palm and reach rows and makes CC-10 record-only for now. ROM and ground are
unchanged.

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

## 5. Commit plan and ledger

Status: READY FOR OPERATOR REVIEW. Branch `mvp0/pr-s26c-natural-pose-impl`, base `origin/main @ 5b88827f`.

| Commit | Implementation | Validation | Review |
| --- | --- | --- | --- |
| C1 bind-time palm frame (A, revised R1 §2.1) + G-5 (B) | [x] `bf180974`, `43bd9dbe`, `52c70938`, `7038d4f4` (the measurement on the auto-weighted hand) | [x] AC-1: the pre-R1 exports fail G-5 (Alba −94/+93, Dev −103/+102, Greta −100/+100, Ines −89/+89, Alice −86/+84); after R1 every body reads ≤ 3.5° (table below); G-1…G-6 and G-14 pass on all 21 and on Vera's re-derivation | [x] three instrument faults found and fixed during the run, each recorded in its commit |
| C2 100STYLE Neutral idle → `stand_lib.glb`, DEP-146, ASSETS/LICENSE (C) | [x] `426d8d39` | [x] retargets through the bone map and plays on every cast body and Vera (`test3` frames, every pose sheet) | [x] |
| C3 `Stand` from the clip; delete the stance solver (D) | [x] `0c419000` (`stance.gd` and its probe path in C4 `a068f980`) | [x] AC-5: `git grep -w stance` finds no stance block, solver or keying code | [x] |
| C4 ROM, forearm-rotation and recorded gates on every frame; CC-10/CC-11 (E, revised §2.2) | [x] `a068f980`, `852ceeb4`, `2d39956c`, `40b5063d` | [x] AC-2: the self-test plants a forearm twisted 120° (fails `forearm_pronation`, AAOS range named), a supinated and an over-pronated forearm; AC-3 reach is record-only under §2.2 (no bound yet, see findings); AC-4: ground passes on all 22 over the loop; `check_cast.py --self-test` passes | [x] |
| C5 re-bind and re-export the 22 bodies; 2D sprites; review page (F) | [x] `64ed0853`, `45d09659` (dump), `811d44bc` (21 bodies + records), `00902e74` (88 sprites) | [x] AC-6: every sprite sidecar names the clip and its pose file's SHA-256; AC-7: `mineworld-demos/casting/npc-generated.html` shows #319, main and this PR, front and side, for all 22, with the gate table; `identity.sh` PASS (social-cafe 23, market-town 23); `ci_layer.py fast` PASS (28 commands) | [x] |

### 5.1 G-5 palm turn, joints to mesh, about the forearm (degrees, left, right)

| body | before (main's export) | turned at bind | after |
| --- | --- | --- | --- |
| alice-moreau | -85.9, 84.1 | +86.6, -84.6 | 0.8, -0.3 |
| bob-achterberg | -11.1, 10.3 | +12.1, -10.4 | 1.3, -0.3 |
| carol-mensah | -10.0, 10.1 | +10.2, -10.9 | -0.8, -0.2 |
| dev-raman | -102.6, 101.8 | +103.6, -103.0 | 2.0, -1.5 |
| erin-walsh | -15.4, 15.3 | +16.9, -14.5 | 0.4, 1.8 |
| felix-okafor | -13.7, 10.0 | +15.1, -11.4 | 0.4, -0.6 |
| grace-liu | -6.9, 8.2 | +7.2, -7.3 | 0.4, 0.9 |
| hana-sato | -9.9, 8.1 | +11.4, -7.4 | -0.1, 1.9 |
| ivan-petrov | -16.2, 15.7 | +17.6, -16.0 | 1.3, -0.1 |
| otto-brandt | -1.3, 1.3 | +2.7, -1.4 | 1.3, 0.0 |
| wes-calloway | -11.1, 5.6 | +10.9, -7.7 | 0.5, -2.4 |
| alba-ferreira | -94.2, 93.4 | +95.2, -93.3 | 1.3, -0.4 |
| bruno-lindqvist | -5.4, 3.1 | +6.5, -3.0 | 0.8, 0.2 |
| celia-okafor | -15.0, 13.7 | +15.6, -13.7 | 0.0, 0.7 |
| dario-russo | -7.9, 5.3 | +9.8, -6.0 | 1.9, -0.8 |
| elin-berg | -20.4, 20.4 | +22.2, -21.7 | -0.7, 1.0 |
| finn-halvorsen | 19.5, -20.3 | -16.7, +18.4 | 2.1, -1.3 |
| greta-novak | -99.9, 99.8 | +101.4, -100.2 | 2.9, -2.8 |
| hugo-marchetti | -7.5, 5.6 | +8.6, -5.6 | 0.9, 0.1 |
| ines-duarte | -89.3, 88.6 | +90.6, -88.8 | 0.7, -0.6 |
| nils-aaltonen | 2.9, -2.9 | -2.0, +3.7 | 1.0, 0.5 |
| vera-lindgren (re-derivation; client keeps `meshy_d.glb`) | −9.9, −1.8 (`meshy_d.glb`) | −0.5, +1.8 | 1.7, 3.5 |

### 5.2 Stand gates over every frame of the loop (the recorded idle, after R1)

Forearm pronation from the thumb-forward neutral (gate: 0–60°), palm angle from facing the thigh (recorded),
fingertips against the thigh front (recorded, mm, negative = behind), clearance (recorded, mm, the slab outline,
packs left out), worst ground sink (mm); left / right. Vera's left arm holds the strap and is not gated.

| body | pronation | palm | reach | clearance | sink | gates |
| --- | --- | --- | --- | --- | --- | --- |
| vera-lindgren | -51.2..-51.2 / 47.4..48.8 | 69.2 / 77.0 | -14.4 / 38.7 | -54.9 / -41.4 | -0.8 | PASS |
| alba-ferreira | 20.1..21.2 / 47.4..48.8 | 31.7 / 79.4 | -119.6 / -25.5 | -62.7 / -100.9 | -2.1 | PASS |
| bob-achterberg | 20.1..21.2 / 47.4..48.8 | 23.1 / 58.3 | -146.2 / -75.6 | -91.8 / -103.6 | -2.9 | PASS |
| carol-mensah | 20.1..21.2 / 47.4..48.8 | 13.3 / 73.0 | -71.8 / -26.0 | -60.8 / -100.6 | -1.3 | PASS |
| dario-russo | 20.1..21.2 / 47.4..48.8 | 18.9 / 46.4 | -151.6 / -108.5 | -91.8 / -97.8 | -1.7 | PASS |
| elin-berg | 20.1..21.2 / 47.4..48.8 | 40.2 / 75.2 | -121.5 / -72.4 | -56.0 / -68.8 | -0.8 | PASS |
| felix-okafor | 20.1..21.2 / 47.4..48.8 | 31.6 / 60.9 | -145.5 / -63.8 | -68.5 / -100.5 | -1.6 | PASS |
| grace-liu | 20.1..21.2 / 47.4..48.8 | 9.6 / 61.4 | -69.2 / -54.7 | -123.5 / -175.0 | -1.0 | PASS |
| hana-sato | 20.1..21.2 / 47.4..48.8 | 46.1 / 74.1 | -146.3 / -57.8 | -62.6 / -86.8 | -0.7 | PASS |
| ines-duarte | 20.1..21.2 / 47.4..48.8 | 27.4 / 68.5 | -77.5 / -28.9 | -70.1 / -71.0 | -1.0 | PASS |
| nils-aaltonen | 20.1..21.2 / 47.4..48.8 | 42.0 / 74.0 | -160.5 / -111.7 | -56.9 / -77.5 | -0.9 | PASS |
| wes-calloway | 20.1..21.2 / 47.4..48.8 | 49.0 / 66.0 | -191.7 / -123.2 | -75.3 / -98.4 | -0.6 | PASS |
| alice-moreau | 20.1..21.2 / 47.4..48.8 | 27.0 / 70.1 | -124.2 / -44.5 | -67.0 / -95.0 | -1.9 | PASS |
| bruno-lindqvist | 20.1..21.2 / 47.4..48.8 | 47.8 / 77.6 | -122.7 / -52.9 | -51.9 / -67.6 | -2.2 | PASS |
| celia-okafor | 20.1..21.2 / 47.4..48.8 | 33.9 / 68.0 | -102.7 / -62.7 | -53.2 / -68.4 | -2.0 | PASS |
| dev-raman | 20.1..21.2 / 47.4..48.8 | 24.3 / 74.3 | -39.3 / -17.8 | -67.8 / -92.1 | -0.5 | PASS |
| erin-walsh | 20.1..21.2 / 47.4..48.8 | 37.4 / 76.9 | -187.2 / -100.0 | -60.8 / -67.5 | -3.1 | PASS |
| finn-halvorsen | 20.1..21.2 / 47.4..48.8 | 43.9 / 65.1 | -134.8 / -49.6 | -62.3 / -71.5 | -2.7 | PASS |
| greta-novak | 20.1..21.2 / 47.4..48.8 | 32.9 / 82.2 | -182.9 / -69.2 | -120.1 / -145.7 | -2.3 | PASS |
| hugo-marchetti | 20.1..21.2 / 47.4..48.8 | 132.4 / 70.9 | -157.0 / -73.5 | -75.8 / -82.5 | -0.4 | PASS |
| ivan-petrov | 20.1..21.2 / 47.4..48.8 | 4.8 / 53.2 | -101.8 / -118.4 | -84.6 / -86.0 | -1.8 | PASS |
| otto-brandt | 20.1..21.2 / 47.4..48.8 | 59.0 / 93.4 | -147.6 / -77.0 | -61.2 / -76.9 | -3.1 | PASS |

All 22 pass: ROM inside AAOS on every frame, the forearm-rotation gate, and ground. One recording retargeted onto the
profile gives every body the same forearm angles (left 20–21°, right 47–49°).

### 5.3 Findings

- F-1 **Reach bound not applied.** Every re-bound cast body keeps its fingertips behind the thigh front (worst −17.8 mm,
  Dev's right). Vera's right hand is 38.7 mm ahead: she keeps `meshy_d.glb` and her own proportions. No published
  anthropometric value states a fingertip–thigh-front relation for a relaxed stand. The research note's reach row is
  a design proposal, not a citation. Under the ruling ("ask if no citation supports it") the bound is left for the
  operator; the table above is the evidence.
- F-2 **Clearance is not a usable bound for real mocap.** With the recorded idle every arm reads −42 to −175 mm by the
  slab outline. The arms hang at 2–3° abduction and the sleeves and hands rest against the hip, as the frames show,
  and the polar outline counts a garment flaring past the hand as "inside". CC-10 stays record-only. A true contact
  measure (signed distance, S36's libigl checks) is the right instrument.
- F-3 **The palm angle is recorded, not gated.** It reads 5–59° left and 46–93° right, and Hugo's left 132°, while
  every body has the identical forearm rotation and G-5 aligns mesh and joints within 3.5°. The plane fit on a curled
  hand is a noisy instrument; the forearm-rotation gate is the measure the ruling adopted.
- F-4 **Vera's client body is unchanged** (`meshy_d.glb`, the D+ export, G-5 −9.9/−1.8). Her re-derivation through
  the cast pipeline with R1 passes every static gate (palm 1.7/3.5). It was not swapped into the client: her grip
  constants and materials are solved on `meshy_d`. Her parity against `meshy_d` now differs in the hands only (vertex
  ≤ 4.1 mm, finger weights) — the deliberate R1 change; joints and every other vertex match.
- F-5 Instrument faults found and fixed during the run, each in its commit:
  - the template hand radius swallowed the torso;
  - the template-side palm sign failed beyond a quarter turn;
  - the nearest-bone hand set took Dario's cuff;
  - the pose dump used pose × rest⁻¹;
  - the clearance outline counted the backpack;
  - the palm plane took in the curled fingers.
- F-6 Six of the first `-g2` sheet runs and some machine-slot waits stalled under contention; the lane's priority slots
  (`CAST_PRIORITY=1`) carried the rest.
- Not run locally (R-LOAD-1): the Rust workspace and the Godot suites; CI on the exact head is the gate.

## 6. Risks

- `overwrite_axis` may rewrite the rolls fixed in A. Mitigation: AC-1 runs on the imported skeleton. If it
  fails, the cast switches to `RetargetModifier3D`, which keeps rests. That would be a recorded deviation,
  brought back to the coordinator.
- The 100STYLE Neutral idle may not suit broad bodies (one performer). Mitigation: the clearance gate fails
  visibly. A residual is reported, never tuned away. The fallback clip is listed in C.
- The re-bind changes Vera's export bytes. Mitigation: her parity is re-baselined and recorded, and her frames
  are compared on the page.
