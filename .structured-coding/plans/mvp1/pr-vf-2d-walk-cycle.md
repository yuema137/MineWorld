# PR vf-2d-walk-cycle — 2D people walk with a whole leg cycle, not two alternating pictures

**Status.** DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended except Q3, ruled 8 facings; §6). Implemented in branch
`fix/vf-2d-walk-cycle`; the record is §8. A prototype of the recommended route (§4) ran on
three bodies; its tooling lives on branch `proto/2d-walk-cycle` and is not part of this docs-only PR.

**Branch** `design/2d-walk-cycle` from `origin/main @0abbe5ff`. **Lane** promo video (priority lane, 2026-10-20).

## 1. The defect, as recorded

Operator, 2026-10-20, on promo v2: "in 2D, people walking look like only two images alternating, not legs moving
continuously." That is an exact description of what the client draws.

## 2. Audit: what the 2D client draws while a person walks (`origin/main @0abbe5ff`)

| Question | Answer | Where |
| --- | --- | --- |
| Rendered views per body | 2: `front` (walking toward the viewer and right, plan east) and `back` (away and right, plan north). | `tools/character_pipeline/blender/render_sprites.py` |
| Facings on screen | 4 diagonals. Leftward travel mirrors the same two views (`body.scale.x = -1`). | `clients/2d/scripts/scene/people.gd` `_animate`, `_facing` |
| Frames per view | 2: `stride_a` (largest left-foot-forward separation of the `Walk` clip) and `stride_b` (largest right-foot-forward). | `render_sprites.py`; probe dump in `tools/character_pipeline/godot/pose_sheet.gd` `_ground` |
| How frames advance | By distance, not by the clock: `phase += metres / STRIDE_M * PI`; the drawn frame swaps every `STRIDE_M` = 0.72 m. | `people.gd:21`, `:309`, `:319` |
| Resulting frame rate | At the walk clip's speed (about 1.06 m/s) the picture changes about 1.5 times a second. No in-between pose exists, so the legs jump from one extreme to the other. | arithmetic |
| Standing | The `front`/`back` file, which is `stride_a`: a standing person is drawn mid-stride, legs apart. | `people.gd:316–327` |
| Files | 22 bodies × 4 PNGs, about 52 × 114 px each, drawn at scale 0.5; `cast/` is 868 KB. | `presentation/mineworld-default/2D/art/cast/` |

So the defect is not in the frame driver. Driving frames by distance is already the right model, because it keeps
the feet from sliding. The defect is the data: two key poses per view. A walk needs the in-between poses
(contact, down, passing, up), on both legs. The standard minimum is 8 per cycle; 12 reads as smooth at this size.

The 3D source already has every pose. The ground probe plays one whole `Walk` cycle (40 frames at 30 fps) for
every body, after the 3D client's retarget, `Posture` and ground contact, and the sprite renderer can draw any of
those poses. Only two were kept.

## 3. Approaches compared

| | A. Pre-rendered walk sheet from the 3D rig (Blender headless) | B. Same, captured in Godot (SubViewport per frame) | C. Live 3D-in-2D (a SubViewport per drawn person) |
| --- | --- | --- | --- |
| What it is | The existing `render_sprites.py` draws N poses of one cycle per view, all at one scale and anchor. The client picks the frame from distance walked. | A Godot tool scene poses the 3D body and captures a SubViewport to PNG; the client side is the same as A. | Each 2D person is a small 3D viewport showing the real animated 3D body, composited into the 2D scene. |
| Industry use | The classic "3D to 2D sprite" route (Diablo II, Fallout 1/2, Factorio, Age of Empires II). Add-ons exist for it, e.g. Render 8 Directions and Blender Sprite Studio; our renderer already does the same thing. | Used by Godot projects that avoid a Blender dependency. | Used for a few hero characters, not crowds. |
| Look | Same matte painted look as today's sprites (same lights, outline, supersampling). | Godot's renderer, not the matte EEVEE look; needs its own look tuning. | Real-time 3D look; does not match the painted 2D style without a toon shader pass. |
| Runtime cost | One `Sprite2D` per person; no change. | Same as A. | One viewport and one skeleton per person; 20+ people on screen is expensive, and worst on the web client. |
| Package size | Measured: 26 PNGs, 304 KB per body for 12 frames × 2 views + standing. 22 bodies ≈ 6.7 MB as loose PNGs, about 8× today; an atlas per body is smaller (see Q5). | Same as A. | Ships the 3D bodies to the 2D client (tens of MB). |
| New code | About 90 lines in the renderer, 6 in the probe, about 20 in `people.gd`. | A new capture tool to write and keep in step with the renderer. | A new presentation path; breaks the 2D client's "sprites only" contract and its headless tests. |
| Same pose as 3D | Yes: the probe dumps the 3D client's own pose. | Yes. | Yes. |

Two more options were rejected early. A 2D skeletal cut-out rig (Spine, DragonBones, Godot `Skeleton2D`) would
need hand-cut parts and hand animation per body; that is per-asset hand work, which the charter forbids, and it
throws away the 3D source. In-betweening the two existing frames with image interpolation or optical flow cannot
invent a passing pose; it smears the legs.

**Recommendation: A.** It reuses the tool that already draws the 2D cast from the 3D bodies, and keeps the look and
the runtime cost. The only new cost is package size, and Q5 handles that.

## 4. The recommended design

1. **Probe** (`pose_sheet.gd` `_ground`, `--dump-pose`): besides `stride_a`/`stride_b`, dump every frame of the
   one `Walk` cycle it already plays (`walk_cycle`: one bone-delta map per frame), and the distance that cycle
   covers at the body's own clip speed (`walk_cycle_m` = speed × cycle length; 1.35–1.45 m on the three prototype
   bodies).
2. **Renderer** (`render_sprites.py --walk-frames N`): per view, draw the standing pose plus N poses evenly spaced
   over the cycle, starting at the largest left-foot-forward stride. Every frame of a body uses the same phase-0
   convention, so frame k means the same gait phase for every body. All frames of a view share one scale (set by
   the standing body, pixel height = stature × 64), one canvas (the union of all frames' extents) and one anchor:
   the armature origin projected onto the floor. Today each frame is re-cropped and re-scaled to its own height,
   which would make a cycle pump and jitter. The renderer writes a `<key>_walk.json` sidecar with `frames_per_cycle`,
   `cycle_m`, the per-view frame list, size and anchor, and the source and pose hashes.
3. **Client** (`people.gd` `_animate`): when a person's sprite set has a walk sheet, the frame is
   `floor(fract(walked_m / cycle_m) × N)`. That is driven by distance, so the feet stay planted whatever the
   server's pace. When the person stands, it draws the standing frame. Sets without a sheet keep today's two-frame
   behaviour, so other packs are unaffected. Facing, the `STRIDE_HOLD_S` pause rule and the mirrored left views
   stay as they are (PR #288).
4. **Presentation contract** (`clients/2d/PRESENTATION.md` §3): directional sprites gain an optional walk sheet
   (`<base>_walk`: frames, `cycle_m`, anchor). This is generic data and no world is named. A second pack, or the
   lakeside cast, uses it the same way.
5. **Gate**: the existing walk drive (`--drive=walk`, which reads what the client draws) asserts that a walking
   person shows at least N distinct frames per `cycle_m` walked, and shows its standing frame when stopped. Run it on
   `social-cafe` and the lakeside world.
6. **Re-render** all 22 bodies with the shipped renderer, in one commit, with refreshed provenance sidecars and
   `PROVENANCE.md` row.

Acceptance: the before/after clip, re-recorded in the real 2D client, on café town and the lakeside world.

## 5. Prototype evidence (2026-10-20)

- Tooling: branch `proto/2d-walk-cycle` (probe dump + `--walk-frames`), ran on `vera-lindgren`, `alice-moreau` and
  `dario-russo`. Ground probe: 40-frame cycles, `walk_cycle_m` 1.417 / 1.355 / 1.449 m.
- Clip: `/Users/yuema137/mineworld-demos/video/verify-1019/fixes/2d-walk-before-after.mp4` (28 s; 14 s at speed, then
  the first 6 s at half speed). Left: main's sprites with main's frame rule. Right: the 12-frame sheets driven by
  distance over `cycle_m`. Both use the same speed, path, isometric projection (32 px/m, scale 0.5) and 2.5× zoom.
  It was recorded by a small Godot harness that reimplements both frame rules. It is not the real client; step 3
  above does the real-client wiring.
- What the prototype shows: legs pass through contact, passing and lift poses in order; a stopped person stands
  with feet together instead of mid-stride. Not yet checked: the planted-foot drift against the floor grid over a
  full cycle (the gate in step 5 measures it).

## 6. Freeze questions (recommended answers)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended, except Q3: the operator chose 8 facings now (not 4), so the sheet is 8 views × 12 frames plus a standing frame per view.

- **Q1. Route?** Recommended: **A**, the pre-rendered walk sheet from the 3D rig through the existing Blender
  renderer, with frames driven by distance walked.
- **Q2. Frames per cycle?** Recommended: **12** (the prototype). 8 is the minimum that reads as a walk; 16 costs a
  third more for little gain at 57 px figure height.
- **Q3. Directions?** Recommended: **keep 4 facings (2 rendered views + mirror) in this PR.** 8 facings (5 rendered
  views) add pure up, down and sideways walks but cost 2.5× the files; that is a follow-up if the video needs it.
- **Q4. Standing frame?** Recommended: **yes**, draw the `Stand` pose (100STYLE Neutral) when still, instead of
  stride A.
- **Q5. Storage?** Recommended: **one atlas PNG per body and view** (13 cells) plus the JSON sidecar, instead of 26
  loose files. The client slices with `AtlasTexture`. Expected ≤ 4 MB for 22 bodies; measured in the PR.
- **Q6. Jog?** Recommended: **not now.** The 2D client has no jog gait yet; the same route adds it later.

## 7. Sources

- Classic 3D-to-sprite workflow and tools: <https://opengameart.org/forumtopic/how-to-make-a-2d-isometric-spritesheet-from-a-3d-model-using-blender>,
  <https://auteddy.itch.io/8-directions-render-plugin-for-blender>,
  <https://sorceress.games/blog/3d-to-2d-render-your-3d-models-into-game-ready-2d-sprite-sheets>.

## 8. Implementation record (branch `fix/vf-2d-walk-cycle`, 2026-10-20)

**What shipped.**

- Probe (`pose_sheet.gd`): `--dump-pose` adds `walk_cycle`, `walk_cycle_m` and `body_scale`.
- Renderer (`render_sprites.py --walk-frames 12`): five drawn views (`down`, `down_right`, `right`, `up_right`,
  `up`; 0°…180° between the walk and the direction toward the camera). With mirroring, the client gets eight
  facings (Q3 ruling). Each view has 13 columns (the `Stand` pose, then 12 cycle frames). All are in one atlas per
  body with one cell size and one anchor, plus the foot-drift reading.
- Batch: `cast.py walk-sheets` (`mwcast/walk_sheets.py`) renders each body, converts it to lossless WebP (`cwebp`),
  and writes it into the pack with its binding (`cast_<id>_walk`) and provenance. PIPELINE.md §19.
- Client: `presentation.gd` reads the optional `sheet`. `people.gd` draws a walk-sheet person with one `Sprite2D`
  (`hframes`/`vframes`). It picks one of eight facings from plan travel, relative to the projection's "toward the
  viewer", with 30° hysteresis; a one-sector (45°) turn must hold for two 0.25 m samples (corrected after merge,
  see "Post-merge correction" below). The column is chosen by
  metres walked over `cycle_m`, and the standing frame is drawn after 0.4 s still. The spike's synthetic bob is
  not added. A set without a sheet is unchanged. Contract: PRESENTATION.md §3 "Walk sheet".
- Drive (`--drive=walk`): new checks "walk draws the whole cycle" and "stands still in the standing frame".
  "facing is steady" now compares the eight-way view too.

**Package size.** The 22 atlases total 6.1 MB as lossless WebP (the same atlases as PNG: about 11 MB). The 2D pack
grows from 52 MB to 58 MB. The two-frame sprites (868 KB) stay as the fallback for sets without a sheet.

**Foot drift over a full cycle, as drawn** (sidecar `foot_drift_mm`, PIPELINE.md §19 item 4). Measured on the 12
drawn frames:

- Worst single step of a planted foot: 15–114 mm over the 22 bodies; most bodies 40–60 mm.
- Worst whole stance: 33–154 mm.
- Within a held frame the drawn foot moves with the body by `cycle_m / 12` = 113–122 mm (≈ 4 px at zoom 1).

The two-frame sprites held one picture for 0.72 m of travel, so their drawn foot slid about 720 mm per hold. The
largest per-step slips (`wes-calloway` 114 mm, `dev-raman` 102 mm) follow those bodies' 3D gait and are reported
here for the cast lane, not fixed in 2D.

**Validation (local, real server + real client).**

- `--drive=walk` PASS on `social-cafe` (as `visitor`, Vera; also with `--town`) and on `market-town`:
  "walk draws the whole cycle 12 of 12 over 14.2 cycles", "stands still in the standing frame", "facing is
  steady", and "drawn at its height" (Vera 57.0 px, want 56.5 ± 2).
- Mutation: drawing only two of the twelve columns turns "walk draws the whole cycle" red ("2 of 12").
- `tests/test_walk_sheets.py` PASS. ruff, ruff format and pyright are clean on the new Python.
  `check_client_rules.py` PASS.
- Lakeside needs `--packs`; it was not run here.

**Clip.** `/Users/yuema137/mineworld-demos/video/verify-1019/fixes/2d-walk-real-client.mp4` (80 s). Real server
with `--town`, real 2D client recorded by Movie Maker. Left: main. Right: this branch. The first 40 s show the
street; then a closer crop at half speed. The two runs are not frame-synchronised, because the town's people
decide independently.

**Post-merge correction (2026-10-21, branch `fix/clients-milestone`).** The nightly `clients` run 39594613270 failed
"facing is steady" in `walks_from_the_apartments_into_the_cafe`, `the_requests_do_not_depend_on_the_presentation`
and `two_d_harness_ignores_the_players_settings` on all three platforms: near the café door the route bends for
about half a metre, and the figure turned `view 3 → 2 → 3` (Linux, Windows) or `view 3 → 1 → 3` (macOS) within
0.72 m. Two 0.25 m samples (0.5 m) are shorter than the one pace the drive's bound is, and a two-sector (90°) turn
was made at once. Correction: a turn of up to 90° is made only once the travel has held the new facing for one
pace (`STRIDE_M`, accumulated drift); a larger turn (a reversal) is still made at once. The drive's requirement
"never turn and turn back within one pace" is then true by construction, on any route. The check is unchanged.
