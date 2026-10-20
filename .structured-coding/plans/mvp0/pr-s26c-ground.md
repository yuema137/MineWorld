# PR S26-c-ground — every body on the floor, and a relaxed stand

Status: READY FOR OPERATOR REVIEW (bug fix ordered by the operator, 2026-10-19, top priority, zero Meshy credits).
Base: `mvp0/pr-s26c1-cast-client` (PR #306). Branch: `mvp0/pr-s26c-ground`.
Specifications changed with it: `tools/character_pipeline/PIPELINE.md` §§15.1–15.2, `clients/3d-spike/PRESENTATION.md` §2.

## 1. The defects

- **Bug 1 (sinking).** In the 3D client and the pose sheets many cast bodies stood with their feet partly or wholly
  under the floor (Elin Berg, Hugo Marchetti, Greta, Ines, Celia, …), and the protagonist clipped into it walking.
- **Bug 2 (stiff stand).** Every NPC's standing arms were pressed against the body, with the hands at a strange angle
  (palms forward).

## 2. Root causes (measured, `cast.py ground`, all 22 bodies)

1. **The `Stand` clip's hips were keyed in metres into a normalised track.** The mixer multiplies every bone
   position track by `Skeleton3D.motion_scale`, which `Normalize Position Tracks` sets to the body's own hips height.
   `human.gd` `_stand_clip` keyed `rest.origin + HIP_DROP` in metres, so a standing body's hips stood at
   (rest + drop) × hips height: 166 mm low on Elin (hips 0.82 m), 138 mm on Greta, 96 mm on Hugo, 13 mm on Vera,
   55 mm *high* on Wes (hips > 1 m). Measured hips heights matched (0.8211 − 0.009) × 0.8211 = 0.6668 m exactly.
2. **Every export's soles are 17.7 mm below its origin** (pipeline standard G-4, `SOLE_Z_M = −0.0177`, inherited
   from the protagonist's D+ export), and the client stands a scene's origin on the floor: every body, Vera
   included, stood 17.7 mm into it at rest.
3. **The retargeted gaits are exact only for feet shaped like the mannequin's.** Their hips translation is scaled by
   hips height; heel strike and toe-off roll each body's own foot about its own ankle (ankles 0.049–0.102 m above
   the sole), so the toe or heel drops a body-dependent amount. Beyond causes 1–2, every body pushed a toe, a heel
   or a trouser hem 5–38 mm further through the floor (most minima: `LeftToes`/`RightToes`, toe-off). The error
   follows the foot's roll through the cycle (Vera's per-frame lowest point ranged over 55 mm), so no single hips
   scale or constant offset fixes it. (The lead's hypothesis — the hips-height normalisation against differing
   leg-chain proportions — is confirmed for the Stand clip, cause 1, and is the mechanism of cause 3 through the
   foot; the leg-bend term alone is small, < 1 mm.)
4. **Bug 2.** The `Stand` arms (`stand_key`) were authored on the narrow town body: on the broader generated
   bodies the hanging forearms and hands sit 46–146 mm inside the hips' outline (worst on the weight-bearing right
   hip), and on about a quarter of the bodies the generated hand mesh lies a quarter turn from its finger joints,
   so a joint-neutral wrist hangs palm forward.

## 3. The fix (generic; no per-name constants)

- `human.gd` `_stand_clip`: position keys divided by `motion_scale` (cause 1).
- `scripts/ground_contact.gd` (new `SkeletonModifier3D`, after `Posture`), added by `Human._stand_on_soles` for every
  body (cast, protagonist and town): once per scene it takes the body's support points (the lowest vertices of its
  shins, feet and toes over a fan of foot orientations, with their skin weights) and its rest sole; the scene is
  raised by the rest sole (cause 2), and every frame the points are skinned and the `Hips` moved so nothing goes
  below the floor and, standing or walking (`pin` 1, set by `set_gait`), the lowest point is held on it; at a jog
  (`pin` 0) it only lifts (cause 3). Cost: 20–33 ms once per body type, 0.034 ms per body per frame (71–79 points).
- `tools/character_pipeline/godot/stance.gd` solves each cast body's relaxed arms on its own mesh (PIPELINE §15.2)
  into its manifest's `stance.arms`; `Human.build_character` keys them into that body's `Stand` clip. The 2D sprite
  renderer (`blender/render_sprites.py`) applies the same rule in Blender; all 22 sprite sets were re-rendered.
- Gates: the probe enforces §15.1's bounds (5 mm sink, 20 mm stance float) and the pose sheet flags a failing
  ground block; `scripts/check_cast.py` (CI) CC-9 recomputes the verdict from every manifest's recorded `ground`
  block and the reference body's index entry, CC-10 requires every cast manifest's `stance`.

## 4. Before → after (mm; negative = below the floor / inside the body)

Columns: rest; Stand lowest; Walk lowest; Jog lowest; after only: walk stance float (highest per-frame lowest),
jog foot contact (higher foot's lowest point); arm clearance L / R (lower forearm and hand vs torso, hips, thighs);
palm turn L / R (degrees; ≈ 20–30 is a hand facing its thigh, ≥ 40 faces forward; Vera's left holds her strap).

| body | rest | Stand | Walk | Jog | walk float | jog contact | arms L / R | palm L / R |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| vera-lindgren | -17.7 → 0.0 | -40.5 → -0.2 | -55.2 → -0.8 | -55.7 → -0.8 | 0.0 | 0.0 | -20.2 / -46.5 → -20.2 / -46.5 | 154.1 / 19.8 → 154.1 / 19.8 |
| alba-ferreira | -17.7 → 0.0 | -66.1 → 0.0 | -48.0 → -1.9 | -36.7 → -2.1 | 0.0 | -1.1 | 21.8 / -53.2 → 43.2 / 32.4 | 42.9 / 42.8 → 24.1 / 22.5 |
| alice-moreau | -17.7 → 0.0 | -86.4 → -1.2 | -36.0 → -1.9 | -53.1 → -1.9 | 0.0 | -1.5 | -5.0 / -78.4 → 31.4 / 36.8 | 46.0 / 46.7 → 29.3 / 29.6 |
| bob-achterberg | -17.7 → 0.0 | -7.1 → 0.0 | -43.6 → -2.4 | -28.9 → -2.9 | 0.0 | -2.7 | -35.1 / -117.8 → 35.9 / 33.6 | 21.9 / 20.1 → 29.9 / 24.1 |
| bruno-lindqvist | -17.6 → 0.0 | -70.6 → -0.3 | -51.4 → -1.8 | -41.6 → -2.2 | 0.0 | -0.8 | 11.9 / -57.8 → 46.3 / 32.6 | 19.2 / 19.7 → 28.1 / 26.7 |
| carol-mensah | -17.5 → 0.0 | -115.6 → -0.8 | -34.4 → -0.8 | -42.6 → -1.3 | 0.0 | 0.0 | 2.0 / -78.2 → 32.4 / 31.5 | 17.1 / 19.3 → 30.9 / 29.1 |
| celia-okafor | -17.7 → 0.0 | -77.5 → -0.7 | -36.7 → -1.4 | -35.3 → -2.0 | 0.0 | -0.6 | 7.9 / -69.5 → 36.4 / 34.5 | 25.8 / 23.0 → 26.4 / 26.2 |
| dario-russo | -17.8 → 0.0 | -28.9 → -0.1 | -46.0 → -1.4 | -37.3 → -1.7 | 0.0 | -1.6 | -43.5 / -123.5 → 32.8 / -4.6 | 19.8 / 24.4 → 24.8 / 27.1 |
| dev-raman | -17.7 → 0.0 | -71.2 → -0.2 | -39.0 → -0.5 | -50.8 → -0.3 | 0.0 | -0.1 | 3.2 / -71.4 → 45.6 / 32.3 | 37.4 / 50.3 → 26.2 / 27.2 |
| elin-berg | -17.7 → 0.0 | -165.7 → 0.0 | -27.5 → -0.7 | -24.0 → -0.8 | 0.0 | -0.7 | -2.4 / -69.2 → 33.4 / 32.7 | 23.6 / 33.2 → 27.6 / 27.7 |
| erin-walsh | -17.7 → 0.0 | -129.3 → -1.0 | -45.8 → -3.1 | -77.9 → -2.7 | 0.0 | -2.3 | -5.4 / -71.8 → 30.7 / 32.2 | 27.3 / 24.8 → 24.9 / 21.4 |
| felix-okafor | -17.6 → 0.0 | -19.4 → 0.0 | -42.9 → -1.6 | -50.1 → -0.7 | 0.0 | -0.5 | -9.6 / -87.6 → 34.4 / 30.9 | 21.8 / 20.3 → 23.8 / 20.8 |
| finn-halvorsen | -17.7 → 0.0 | -34.0 → -0.5 | -39.1 → -2.7 | -29.5 → -1.7 | 0.0 | -0.3 | 19.2 / -50.9 → 31.8 / 34.2 | 24.1 / 26.5 → 28.5 / 27.8 |
| grace-liu | -17.8 → 0.0 | -131.9 → -0.3 | -34.1 → -1.0 | -44.2 → -0.5 | 0.0 | 0.0 | -52.4 / -133.2 → 21.2 / 11.6 | 20.7 / 15.1 → 25.3 / 22.1 |
| greta-novak | -17.8 → 0.0 | -137.8 → -1.3 | -43.0 → -2.3 | -39.8 → -2.3 | 0.0 | -1.3 | -65.6 / -127.7 → 19.7 / 25.2 | 47.2 / 51.4 → 19.8 / 20.4 |
| hana-sato | -17.6 → 0.0 | -45.5 → -0.6 | -42.4 → -0.5 | -53.7 → -0.7 | 0.0 | -0.1 | 9.3 / -68.7 → 39.2 / 31.1 | 21.1 / 21.5 → 27.3 / 28.4 |
| hugo-marchetti | -17.6 → 0.0 | -96.2 → 0.0 | -45.3 → -0.4 | -22.9 → 0.0 | 0.0 | 0.0 | -11.0 / -102.6 → 31.4 / 22.0 | 24.3 / 24.2 → 27.1 / 24.5 |
| ines-duarte | -17.7 → 0.0 | -97.2 → -0.7 | -28.9 → -1.0 | -21.1 → -1.0 | 0.0 | 0.0 | -7.5 / -77.2 → 34.0 / 29.5 | 31.3 / 31.4 → 26.7 / 22.5 |
| ivan-petrov | -17.8 → 0.0 | -94.2 → 0.0 | -45.2 → -1.8 | -45.4 → -0.6 | 0.0 | -0.1 | -13.3 / -105.5 → 31.6 / 22.8 | 23.1 / 27.8 → 32.3 / 29.9 |
| nils-aaltonen | -17.7 → 0.0 | 7.6 → -0.9 | -54.0 → -0.9 | -51.2 → -0.4 | 0.0 | -0.2 | 15.1 / -69.3 → 32.2 / 26.4 | 20.1 / 23.3 → 25.9 / 26.9 |
| otto-brandt | -17.8 → 0.0 | -71.5 → -0.2 | -44.2 → -3.1 | -53.0 → -2.9 | 0.0 | -1.6 | 2.0 / -65.0 → 31.0 / 28.9 | 21.3 / 20.0 → 23.8 / 21.2 |
| wes-calloway | -17.7 → 0.0 | 55.0 → 0.0 | -47.9 → -0.5 | -51.1 → -0.6 | 0.0 | 0.0 | -47.2 / -145.6 → 28.2 / -27.7 | 25.8 / 26.1 → 24.0 / 26.5 |

All 22 pass §15.1 after (worst sink −3.1 mm, Otto's walk; no stance float). Before, every body failed.
Arms: 19 cast bodies reach ≥ 20 mm on both sides; Grace (21 / 12), Dario (33 / −5) and Wes (28 / −28) are short on
the right within the 15° abduction and 12° forward limits (their right hip, under the stand's weight shift, is the
widest point; the elbow-to-hand look is relaxed in their sheets). Vera keeps her look (her grip stance is
`human.gd`'s REFERENCE constants); her right hand still measures −46 mm.

## 5. Evidence

- Probe runs (scratch, `cast-scratch-s26c/ground/`): `before/` (the head of #306), `after/`, `arms-before/`,
  `arms-after/` (`ground.json`, `probe.log`). Determinism: two runs of the same commit gave identical tables.
- Pose sheets for all 22 after the fix: `cast-scratch-s26c/sheets/<id>-<run>-g/` (Vera: `vera-lindgren-ref-g`),
  cast subjects built from their manifests (`--manifest`), so the sheets show the stance the client draws.
- 3D client (its own slice scene, five cast people and Vera walking along the pavement, then standing):
  `mineworld-demos/casting/client-shots/ground-{before,after}-walk-{0..3}.png`, `ground-{before,after}-standing.png`.
- Review page: `mineworld-demos/casting/npc-generated.html`, rebuilt by `cast-scratch-s26c/tools/make_page.py` (minimally adjusted: `SHEET_SUFFIX=-g`, `GROUND_BEFORE`/`GROUND_AFTER`/`ARMS_BEFORE`) with the `-g` sheets, standing front and three-quarter before/after per body, and the ground and arm table.
- `python3 scripts/check_cast.py` and `--self-test`: PASS (22 looks; 19 cases). `ruff check`/`format --check` on
  `tools/character_pipeline`: PASS. `pytest tools/character_pipeline/tests`: 135 pass, 1 FAIL pre-existing
  (`test_check_characters_on_the_repository…`: `check-characters` refuses `character.yaml`, present since #306).

## 6. Findings and limits

- F-1 Bug 1's cause 2 lives in the pipeline standard (G-4 `SOLE_Z_M`); the client now stands every body on its
  measured sole, so the standard can stay; a later export at y = 0 would simply be lifted by 0.
- F-2 `GroundContact` also runs on the town body (one generic path). It was not measured here.
- F-3 Stance clearance metric is conservative near loose hems and fused packs (a radial outline); Wes's and Dario's
  right sides read short while their sheets show the hand off the hip.
- F-4 The 2D sprite pass (sub-agent) used its own clearance rule in Blender (30 mm past the elbow, 12 mm at it) and
  abduction up to 32°; it also re-rendered Vera's sprites, which showed the same defect.
- F-5 The pose sheets' P-4 flags are the known protagonist seating-rule finding (`protagonist_rule_error_mm`, seat contact itself within 5 mm everywhere). Building a cast subject from its manifest moves several bodies across its 20 mm line: Alice, Erin, Hugo, Ivan and Otto now pass, Felix (+38) and Finn (+24) now flag.
- Not run locally (R-LOAD-1): the Rust workspace and the Godot test suites; CI on the exact head is the gate.
