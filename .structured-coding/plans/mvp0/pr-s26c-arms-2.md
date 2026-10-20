# PR S26-c-arms-2 — every free arm clears the body by 20 mm, the protagonist included

Status: READY FOR OPERATOR REVIEW. Stacked on PR #319 (`mvp0/pr-s26c-ground`, head 64139a4d). Ordered by the lead,
2026-10-19, zero Meshy credits. The 2D arm rule is unchanged (accepted by the operator); no sprites change.

## 1. What was short after #319

Right-arm clearance: Wes −27.7 mm, Dario −4.6 mm, Grace 11.6 mm (target 30, bound 20). The protagonist was not
solved: her free (right) hand sat 46.5 mm inside her hip, her look kept by `human.gd`'s REFERENCE constants.

## 2. The rule change (generic)

- `stance.gd`: the relaxed limits (abduction ≤ 15°, forward swing ≤ 12°, elbow 14°) stay first. A side that cannot
  clear within them — the side where the stand's weight shift pushes the hip out to the widest point — takes a second
  search (abduction ≤ 24°, forward swing ≤ 21°, elbow 14/20/26°, least deviation first), which lets the hand hang
  ahead of the hip instead of out from it. `solve(base, sides)` solves only the given sides.
- The protagonist: the probe solves the arms her strap grip does not key (an arm whose `stand_key` keys differ with
  and without the grip is held) and `cast.py ground --write` records her `stance` in her index entry
  (`clients/3d-spike/assets/characters/cast/index.json`); `Human.build` keys it into her `Stand`/`StandGrip` clips
  when the REFERENCE scene is her own (`meshy_d.glb`). No new per-name constant; her grip constants are untouched.
- Gate: `scripts/check_cast.py` CC-10 now also holds every arm a stance solves to ≥ 20 mm as built
  (`ground.arm_clearance_mm`), for every cast manifest and the reference entry; an arm the stance leaves alone (her
  strap hand) is not held to it. The `ground` block keeps its name and structure.
- All stances were re-solved under the new rule and the ground probe re-run (all 22 pass §15.1, worst −3.1 mm).

## 3. Arms, before (#319) → after (mm; palm turn in degrees, ≈ 15–30 faces the thigh)

| body | arm clearance L / R | palm turn L / R | abduction L / R (deg) | elbow L / R (deg) | worst sink |
| --- | --- | --- | --- | --- | --- |
| vera-lindgren | -20.2 / -46.5 → -20.2 / 41.7 | 154.1 / 19.8 → 154.1 / 22.2 | 0.0 / 6.0 | 14 | -0.8 |
| alba-ferreira | 43.2 / 32.4 → 38.5 / 34.5 | 24.1 / 22.5 → 22.7 / 22.6 | 5.9 / 8.0 | 14 / 14 | -2.1 |
| alice-moreau | 31.4 / 36.8 → 32.6 / 31.5 | 29.3 / 29.6 → 36.5 / 39.4 | 6.9 / 7.0 | 14 / 14 | -1.9 |
| bob-achterberg | 35.9 / 33.6 → 33.7 / 43.2 | 29.9 / 24.1 → 32.0 / 21.3 | 11.9 / 6.0 | 14 / 20 | -2.9 |
| bruno-lindqvist | 46.3 / 32.6 → 39.6 / 31.5 | 28.1 / 26.7 → 25.1 / 20.2 | 5.9 / 10.0 | 14 / 14 | -2.2 |
| carol-mensah | 32.4 / 31.5 → 40.0 / 30.6 | 30.9 / 29.1 → 25.3 / 25.2 | 5.9 / 9.0 | 14 / 14 | -1.3 |
| celia-okafor | 36.4 / 34.5 → 32.2 / 31.1 | 26.4 / 26.2 → 21.7 / 35.8 | 5.9 / 11.9 | 14 / 14 | -2.0 |
| dario-russo | 32.8 / -4.6 → 31.3 / 34.0 | 24.8 / 27.1 → 23.5 / 22.9 | 13.9 / 8.0 | 14 / 26 | -1.7 |
| dev-raman | 45.6 / 32.3 → 45.4 / 34.0 | 26.2 / 27.2 → 29.0 / 23.4 | 6.0 / 6.0 | 14 / 14 | -0.5 |
| elin-berg | 33.4 / 32.7 → 33.3 / 34.2 | 27.6 / 27.7 → 21.6 / 31.4 | 8.9 / 13.9 | 14 / 14 | -0.8 |
| erin-walsh | 30.7 / 32.2 → 32.4 / 37.6 | 24.9 / 21.4 → 31.3 / 18.1 | 11.9 / 6.0 | 14 / 20 | -3.1 |
| felix-okafor | 34.4 / 30.9 → 33.8 / 43.8 | 23.8 / 20.8 → 19.3 / 16.1 | 9.9 / 6.0 | 14 / 20 | -1.6 |
| finn-halvorsen | 31.8 / 34.2 → 34.6 / 34.9 | 28.5 / 27.8 → 22.8 / 29.9 | 5.9 / 9.0 | 14 / 14 | -2.7 |
| grace-liu | 21.2 / 11.6 → 35.0 / 40.9 | 25.3 / 22.1 → 31.1 / 14.8 | 9.9 / 6.0 | 14 / 14 | -1.0 |
| greta-novak | 19.7 / 25.2 → 35.8 / 30.9 | 19.8 / 20.4 → 13.2 / 15.4 | 5.9 / 8.0 | 26 / 26 | -2.3 |
| hana-sato | 39.2 / 31.1 → 39.0 / 34.0 | 27.3 / 28.4 → 23.9 / 26.8 | 5.9 / 11.0 | 14 / 14 | -0.7 |
| hugo-marchetti | 31.4 / 22.0 → 33.5 / 30.9 | 27.1 / 24.5 → 32.4 / 27.9 | 10.9 / 7.0 | 14 / 26 | -0.4 |
| ines-duarte | 34.0 / 29.5 → 35.6 / 32.7 | 26.7 / 22.5 → 33.9 / 30.5 | 6.9 / 12.9 | 14 / 14 | -1.0 |
| ivan-petrov | 31.6 / 22.8 → 36.8 / 31.3 | 32.3 / 29.9 → 29.5 / 27.5 | 6.9 / 6.0 | 14 / 20 | -1.8 |
| nils-aaltonen | 32.2 / 26.4 → 32.4 / 39.5 | 25.9 / 26.9 → 21.7 / 25.0 | 8.9 / 6.0 | 14 / 26 | -0.9 |
| otto-brandt | 31.0 / 28.9 → 29.9 / 36.2 | 23.8 / 21.2 → 25.5 / 17.7 | 10.9 / 6.0 | 14 / 26 | -3.1 |
| wes-calloway | 28.2 / -27.7 → 35.8 / 30.7 | 24.0 / 26.5 → 19.7 / 20.7 | 5.9 / 10.9 | 20 / 26 | -0.6 |

Every solved arm ≥ 29.9 mm after (bound 20). Vera's left hand holds her strap (−20.2 mm, on her chest, as before);
her palm-turn reading on that side measures the grip, not a hanging hand.

## 4. Evidence

- Probe: `cast-scratch-s26c/ground/arms2-solve/` (the solve) and `arms2-after/` (as built), `ground.json`.
- Pose sheets for all 22: `cast-scratch-s26c/sheets/<id>-<run>-g2/` (cast built from their manifests; Vera with grip).
- 3D client, Vera standing (front, three-quarter) and walking, before (#319) and after:
  `mineworld-demos/casting/client-shots/ground-vera-{before,after}-{standing-front,standing-three-quarter,walking-50,walking-60}.png`.
- Review page `mineworld-demos/casting/npc-generated.html` rebuilt from the `-g2` sheets and these numbers.
- `python3 scripts/check_cast.py`, `--self-test`: PASS. `scripts/ci_layer.py fast`: PASS (28 commands; ruff, format, pyright
  included). Pipeline pytest without the contact tests (they need libigl/fcl): 153 passed.
## 5. Findings

- F-1 Where the second search bends the elbow to 26° (Wes, Dario and Hugo right; Greta, Nils and Otto on one or both
  sides), the hands hang a little ahead of the hips with the palms turned up toward the thighs. They clear the body,
  but the hands read as slightly held. A later idle clip could replace the bend with a weight shift that brings the
  hip in. Operator to judge from the `-g2` sheets.
- F-2 Six of the 22 sheet runs stalled at three concurrent windows (macOS stops drawing). They were re-rendered one
  at a time with identical inputs.
- F-3 The protagonist's left (strap) hand is not held to the bound; her grip is unchanged.

- Not run locally (R-LOAD-1): the Rust workspace and the Godot suites; CI on the exact head is the gate.
