# PR RL-e0 — The far field is mandatory: every presented world closes every view on a far field

## DESIGN FROZEN 2026-10-15 (primary session; operator Q-FF-1, 2, 3, 5, 6, 7, 8)

```text
Design revision:        revision 2 (2026-10-15): revision 1 (PR #215, 1f4d342) with §17.1's rulings applied;
                        origin/main merged and the anchors re-checked (§3.0)
Author:                 RL-e0 design agent, worktree /Users/yuema137/mineworld-worktrees/design-rl-e0,
                        branch docs/rl-e0-design, planning base origin/main @ 4c59cdb0, re-checked against
                        the merged origin/main of 2026-10-15 (§3.0)
Approved by / evidence: questions ruled 2026-10-15 (operator: Q-FF-1, 2, 3, 5, 6, 7, 8 "全部按推荐";
                        primary: Q-FF-4, 9, 10; §17.1). Frozen by the primary session itself, 2026-10-15;
                        the operator answered directly in the primary session
Implementation base:    origin/main at C0 (C0 records the commit)
Start condition:        the primary session's dispatch after freeze; before Milestone F's checklist is
                        handed to the operator (§14.3)
Execution contract:     §13 (template filled where the design can fill it; the rest is the primary's)
Lifecycle:              FROZEN (2026-10-15)
```

**Effort:** `mvp1` · **Step:** [`step-22-realism.md`](step-22-realism.md) (S22), a new row **RL-e0** before
RL-e (§15 of this file proposes the step edits) · **Runs as:** an MVP-0 visual-track PR, like RL-b and RL-b2,
because the operator ruled that it lands **before Milestone F** · **Parents:** [`../mvp0/overall.md`](../mvp0/overall.md)
"Milestone F"; [`pr-rl-b-3d-budget.md`](pr-rl-b-3d-budget.md) (merged; its §5 summit table and the reverted
C8 code are reused here); [`pr-rl-b2-two-views-and-textures.md`](pr-rl-b2-two-views-and-textures.md) (frozen,
draft PR #212, touches the same scene files) · **Format model:** [`../mvp0/pr-s10-p6-lm-controller.md`](../mvp0/pr-s10-p6-lm-controller.md)
§9.0 and §12.

**Why this file is under `mvp1/` and not `mvp0/`.** RL-e0 is an S22 (realism) PR, and S22's step document and
its two earlier PR designs (RL-b, RL-b2) live under `mvp1/` although both ran as MVP-0 visual-track PRs. One
step keeps one directory, so a reader who opens S22 finds all of its PRs together. The MVP-0 dependency (it
lands before Milestone F) is stated in §2.3 and proposed for `../mvp0/overall.md` in §15; it does not need
the file to move.

**Ids.** Decisions `ARC-102`, `ARC-103`, `ARC-104`, assigned by the primary session on 2026-10-15 (they
replace the placeholders ARC-RLe0-a, -b, -c; the next free are ARC-105 and DEP-60). No new `DEP`: RL-e0
adds no dependency (§4). Within this file: SD-FF-n (design decisions), A-n (audit anchors), AC-FF-n
(acceptance), X-n (mutations), V-n (visual guards), I-FF-n (invariants), Q-FF-n (questions), E-FF-n
(evidence), R-FF-n (risks).

---

## 0. Binding inputs, verbatim or exactly restated

### 0.1 Operator rulings, 2026-10-15 (verbatim)

> "再小的世界，再小的城镇，都要有远景！可以有空气墙，但是不可以没有远景…只有这样才能做到沉浸式体验"

> Timing: "里程碑 F 之前做"

In English, for the specification documents (`CLAUDE.md` §2.1 rule 5: English is authoritative, the Chinese
is quoted as the source): *"However small the world, however small the town, it must have a far field. An
invisible wall is allowed, but having no far field is not. Only that makes the experience immersive."* And:
*"Do it before Milestone F."*

### 0.2 Earlier rulings this PR carries

| Source | Ruling | Effect here |
| --- | --- | --- |
| QRL-2, QRL-3 (operator, 2026-10-09; step-22 §13.1) | Real San Diego terrain from USGS 3DEP, skyline at about 1–2°; Market Town on a **bay or coast edge**; the lakeside references keep their authority over architecture, materials, light and mood | The default far field is San Diego's, at real bearings, distances and angles, with the bay (§5.6). The town's distant massing is low-rise in the slice's architecture, not downtown towers (Q-FF-6) |
| RL-b R-5 (operator, 2026-10-09; `pr-rl-b-3d-budget.md` §0.2) and the overall note (`../mvp0/overall.md`, RL-b M-6 line) | "The skyline preview is not kept; it returns with S22's sea surface and real terrain" | RL-e0 brings the skyline back **with the bay**, which is what R-5 waited for. The preview's technique (camera-anchored ribbons, Koschmieder haze, `disable_fog`) is reused (§4) |
| RL-b R-2, R-6 (operator) | p95 ≤ 16.7 ms and video memory ≤ 2 048 MB are binding; draw calls and primitives are advisory; measurement with the display awake, three runs | §8 |
| The station and landmark (operator, relayed with this task) | A station plus a landmark modelled on San Diego's Santa Fe Depot, **after F** | RL-e0 leaves two named slots, `station` and `landmark`, and fills neither (§5.1.6) |
| Step-22 RL-e (MVP-1) | The real 3DEP terrain rings, `tools/terrain-bake` | Out of scope; RL-e replaces RL-e0's hand-authored profile points inside the same schema (§2.2) |
| Standing rule "Realistic defaults" (operator memory) | Physical defaults follow real-world values and cite the reference | Every distance, height and visibility in the default content carries a `source` (I-FF-4) |
| Standing rule "All platforms" | macOS, Linux and Windows | The rendered check runs on all three hosted runners (§6.5) |
| Standing rule "Local load: CI is the gate" (R-LOAD-1) | No local full-workspace runs under load; CI on the exact head gates | Local timings are INCONCLUSIVE; counts only (§8) |

### 0.3 Evidence from the investigation (2026-10-15; images in `/Users/yuema137/mineworld-demos/inv-farfield/`)

| Id | Finding | Evidence |
| --- | --- | --- |
| E-FF-1 | The demo users get is the café street slice, `./mineworld-slice` (`scenes/slice.tscn`) | launcher; step-22 §2.1 |
| E-FF-2 | The slice has no water at all (F-RL-1) | step-22 §2.1 |
| E-FF-3 | The three backdrop ridges (`clients/3d-spike/scripts/slice/street.gd:147-205`) are wound counter-clockwise as seen from the street, and their material keeps the default `CULL_BACK`. Godot treats clockwise as front-facing, so the faces the player sees are culled. A diagnostic run with `CULL_DISABLED` shows all three ridges | `sbs/diag_cull_ff_2_north_aerial.jpg`, `main/*_nocull*.png` |
| E-FF-4 | The ridges cover only about ±44° around north (span 1 420 m at 760 m: `atan(710/760)` = 43°) | `street.gd:153-155` |
| E-FF-5 | East, west and south show the 900 m grass apron cut off hard against the sky | `main/ff_3_south_aerial_s.jpg`, `main/ff_6_west_street_end_s.jpg` |
| E-FF-6 | The accepted state (`690c9d09`) and `main` render identically | `sbs/sbs_*.jpg` |
| E-FF-7 | In 2D, the clear colour `#e0ecee` is visible past the grass diamond | `clients/2d/scripts/app.gd:82`; `presentation/mineworld-default/2D/renderer/godot.yaml` `clear_color` |
| E-FF-8 | `docs/VISUAL_SLICE.md:144` requires a backdrop that closes the view; `:129-130` forbids hard walls across open views. The code violates `:144` | the spec lines |

---

## 1. Goal, in one paragraph

Make "a world with no far field" impossible to ship. A Presentation Pack gains a **required `far_field`**,
one file per presented world (or, from S21, per region), declaring the sky, terrain silhouettes covering
360° of azimuth at real distances and heights, water, the town's distant massing with named slots for a
later station and landmark, and the boundary. A pack that lacks the far field for a world is refused **by
name** at authoring time (`mineworld validate`, `mineworld packs validate`) and at client load. Both
reference clients draw it: the 3D slice closes every direction on hills, bay and townscape and no longer shows
its apron's edge against the sky; the 2D client draws a surround so that its clear colour is never on screen.
Because the defect that started this was **render-time culling**, which no ray-cast or schema check can see,
a **rendered check** proves the result from pixels, on Linux, macOS and Windows, and planted mutations prove
the check can fail.

---

## 2. Scope

### 2.1 In scope

1. **The `far_field` section** (§5.1): its typed schema, its validation and named refusals in Rust
   (`packages`), its loader and the same refusals in the clients' shared GDScript module (§5.3), and the two
   refusal points (authoring and client load).
2. **Specifications first** (`CLAUDE.md` §2.2): `docs/MODULE_SPEC.md` §6.1 (+ new §6.1.1), `docs/ART_DIRECTION.md`
   §12 and §21, `docs/VISUAL_SLICE.md` §1, §4 (`:129-130`, `:144`, out-of-scope line), §12, `clients/2d/PRESENTATION.md`,
   three decision records (Appendix A).
3. **Default content** (§5.6, Appendix B): the 3D pack's far fields for `social-cafe` and `lakeside`; the 2D
   pack's for `market-town`, `social-cafe`, `lakeside` and `bodies-yard`.
4. **3D** (`clients/3d-spike`): the far-field view replacing `SliceStreet.backdrop`/`_ridge` (360° camera-anchored
   ribbons, foreground ground ring that hides the apron edge, bay water with a horizon ribbon, low-rise distant
   massing, the two reserved slots); correct winding through `SurfaceTool.generate_normals()`, with the
   rendered check — not a winding assertion — proving visibility.
5. **2D** (`clients/2d`): a declared `surround` drawn past the reachable area, sized for the widest view the
   client allows (§5.5).
6. **The rendered check** (§6) in both clients, its Rust test harness on the nightly `clients` layer of all
   three OSes, and the planted mutations X-1 … X-5.
7. **Regression shots** (§7): committed under `clients/3d-spike/shots/<world>/farfield/` and
   `clients/2d/shots/<world>/farfield/`, including the parapet-end view `ff_6`.
8. **The generated reference** (§5.8): a `far-field` entry on S25 25a's pack-manifest page, conditional on
   PR #200's merge order.
9. **Housekeeping the operator named:** the stale `--skyline-check` line in `clients/3d-spike/README.md:7`;
   the promenade spike `./mineworld-3d` stops being walkable and keeps its harness modes (Q-FF-3).
11. **A per-PR Linux far-field job**, path-filtered to client and presentation changes (Q-FF-5).
10. **Performance counts** against RL-b/RL-b2's budget, and an operator perf item in Milestone F's batched list
    (§8).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Real 3DEP terrain, `tools/terrain-bake`, terrain shader with splats, 1° profile-vs-DEM check | RL-e (MVP-1). It writes the same `terrain_silhouettes` profile points from data instead of by hand |
| Authoritative water (the `water` System Pack), flow, reflections of the town, wading | RL-f, RL-g. RL-e0's water is a visual plane only and is never disclosed or reached |
| The station and the landmark (Santa Fe Depot) | a later PR after F; RL-e0 reserves `skyline.slots.station` and `.landmark` |
| Fog, sky and sun driven by the world clock | TW-e |
| A general 3D Presentation Pack loader (rooms, bindings) | 16f. RL-e0 reads only `far_field/` from the 3D pack |
| The world's pack id in the protocol's `WorldSummary` | S21 (Q-FF-4, ruled 2026-10-15); RL-e0 identifies the world by a launcher argument (SD-FF-7) |
| Regions | S21. The schema accepts a `region` scope (§5.1.2); no client resolves one yet |
| Making the per-PR far-field job a required check | the operator; RL-e0 adds the job (Q-FF-5) but not as required |

### 2.3 Dependencies and ordering

```text
origin/main ─► RL-e0 ─► Milestone F checklist (§14.3)
                 │
                 ├─ RL-b2 (#212, draft, operator sessions parked): independent. Shared files (§14.1)
                 │   rebase mechanically in either order. RECOMMENDED: RL-e0 merges before RL-b2's
                 │   final operator measurement (RL-b2 §8.2), so the 16.7 ms verdict includes the far field.
                 ├─ 16c (living street, after 12d #171): independent. RL-e0 RECOMMENDED first; if 16c
                 │   merges first, RL-e0 rebases (adjacent lines in slice_world.gd, §14.1).
                 └─ S25 25a (#200): independent. Whichever merges second adds the reference entry (§5.8).
```

No prerequisite must merge first. RL-e0 depends only on `origin/main`.

---

## 3. Audit (`origin/main @ 4c59cdb0`, 2026-10-15)

### 3.0 Re-check after merging `origin/main @ e67af075` (2026-10-15; SR-b, FG-b2, 25c, DL-b)

`git diff --stat 4c59cdb0 e67af075` over every path this design anchors — `clients/`, `mineworld-slice`,
`mineworld-2d`, `mineworld-3d`, `.github/`, `packages/`, `presentation/`, `worlds/`, `server/src/protocol*`,
`scripts/ci_godot.py`, `scripts/ci_layer.py`, `scripts/ci_changes.py` — is **empty**. `docs/MODULE_SPEC.md`
changed (§§4, 5.2, 8 for DL-b and FG-b2) but not §6.1: its anchors moved to lines 1385 (`## 6.1`), 1401 (the
`renderer/` layout line), 1405 ("Not every pack needs every directory") and 1422 (the `dimension` sentence),
with the text Appendix A.1 edits unchanged. `tools/cli/src/packs.rs` and `packages/src/resolve.rs` are
unchanged. Every A-n below holds; line numbers outside MODULE_SPEC are as cited.

### 3.1 Source anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | `SliceWorld.build` calls `SliceStreet.backdrop(g)` right after `SliceStreet.build(g)`; then frontages, café, streetscape, `_place_volumes`, `SliceBatch.merge`, `SliceBudget.apply`, `SliceOccluders.build` | `scripts/slice/slice_world.gd:31-60` | The far-field view replaces the `backdrop` line; it must be excluded from `SliceBatch` (its own meshes), `SliceBudget` (no screen-size range) and occluders |
| A-2 | `backdrop()` builds three ridges at 320/520/760 m north, spans 620/980/1 420 m, heights 62/138/214 m, flat `StandardMaterial3D`, `cast_shadow` off, meta `mw_category = "backdrop"`. `_ridge` computes `n = (b−a)×(c−a)` and `push_error`s if `n.z < 0` — it **asserts the wrong convention** (a counter-clockwise normal toward the viewer), so the check passed while Godot culled the faces | `street.gd:147-205` | E-FF-3's root cause: a winding assertion that encodes a mistaken convention passes forever. RL-e0 removes it and proves visibility from pixels (SD-FF-4) |
| A-3 | `_surrounds`: a 900 × 900 m `leafy_grass` apron at y = −0.14 ("so no frame ever shows the void"); west parapet boxes at `X_MIN − 1`, a setts slab to x ≈ −44; east planted banks to x ≈ +50 | `street.gd:122-144` | The apron's edge at 450 m is E-FF-5. Near construction reaches \|x\| ≈ 50 m, so `boundary.reach_m = 60` for the slice (Appendix B.1) |
| A-4 | Scene frame: +x east, −z north, +y up; the street runs x = −34 … +34, façades at z = ±8.1, pavement surface `WALK_Y = 0.14` | `street.gd:1-40`; `clients/protocol/mineworld/space.gd:10-11` | Bearing b maps to the scene direction `(sin b, 0, −cos b)` |
| A-5 | Camera: `CameraRig.FOV = 70` (vertical), `EYE_HEIGHT = 1.66`, `FAR = 1600` m, which must not change (RL-b V-2) | `scripts/camera_rig.gd:27-32,77` | Anything beyond ~1 500 m is drawn as a camera-anchored proxy (SD-FF-3) |
| A-6 | Environment: one panorama sky (`qwantani_puresky_2k.hdr`, CC0), depth fog 70 → 1 500 m, `fog_aerial_perspective = 0.60` | `scripts/slice/slice_main.gd:209-275` | Far-field ribbons disable fog and carry their own Koschmieder haze (as RL-b C8 did); world-anchored foreground and massing keep scene fog |
| A-7 | The 3D client loads **no Presentation Pack** yet ("S14 16f") | `slice_main.gd` `_settings()` comment | RL-e0 adds the narrowest read: `far_field/<world>.yaml` of `presentation/mineworld-default/3D` (SD-FF-6) |
| A-8 | The reverted skyline preview (RL-b C8, commit `58bc0c4`): `skyline.gd` (391 lines), camera-anchored ribbons at proxies 900/1 100/1 300 m, crest `D·tan(angle)`, curvature and refraction (k = 0.13), Koschmieder at V = 80 km, horizon colour sampled from the HDRI, `disable_fog`, summits re-read at source 2026-10-09 | `git show 58bc0c4:clients/3d-spike/scripts/slice/skyline.gd` | Reused as the starting point of the 3D far-field view (§4) |
| A-9 | Probe modes are flags `--slice-<mode>` in `SliceProbe.MODES`; verdicts print `all <mode> checks pass` or `<n> <MODE> CHECKS FAILED`; `scripts/ci_godot.py judge_probe` parses them and never trusts the exit code | `slice_probe.gd:1-20,117-135`; `scripts/ci_godot.py:200-216` | The 3D check is a new mode `farfield`, printing the same verdict shapes |
| A-10 | `slice_probe.gd` is 1 278 lines (past the ~800-line strong warning) | `wc -l` | The check lives in its own file, `slice_far_field_check.gd`, as `SlicePerf` does (`slice_perf.gd`) |
| A-11 | Nightly `clients` layer runs the `#[ignore]`d Godot tests on ubuntu-24.04 (Xvfb + Mesa lavapipe), macos-26 (Apple Paravirtual) and windows-2025 (D3D12 WARP); `clients-probes` (bash launchers) runs on Linux and macOS only; per-PR CI runs no Godot | `.github/workflows/nightly.yml:355-450`; `.github/actions/godot/action.yml`; `scripts/ci_layer.py:146-170`; `scripts/ci_godot.py:73-105` | The rendered check is a Rust `#[ignore]` test (runs on all three OSes), not a launcher probe (§6.5) |
| A-12 | The **connected** 3D slice "hung" on all three hosted runners (F-13c-impl-5, root cause open); offline modes were not reported hanging | `scripts/ci_godot.py:78-99` | The 3D check runs the slice **offline** with `--gi=none` and offscreen `SubViewport`s (§6.2); hosted-runner viability is C6's first validation step (R-FF-1) |
| A-13 | `DEP-45`'s accepted limitation: windowed tests on Linux "judge behaviour, never frames (`ARC-17`)" | `docs/DECISIONS.md:5904-5950` | The rendered check judges frames **only against frames of the same run** (shown vs hidden), never against a stored image. This needs a dated note on `DEP-45` (ARC-103) |
| A-14 | 2D: clear colour from the pack (`#e0ecee`), `camera.zoom` fixed from the pack (1.55), no zoom control; `land` = paved hub rect grown by `ground.land_margin_m` (60 m); stretch `canvas_items`, aspect `expand`; window size bounds 640×360 … 7 680×4 320 | `clients/2d/scripts/app.gd:82,105`; `scripts/scene/places.gd:179-194`; `scripts/scene/ground.gd:35-48`; `clients/2d/project.godot:20-23`; `clients/shared/SETTINGS.md:135` | With aspect `expand`, a 7 680×360 window shows a 19 200 × 900 base-pixel view (21.3 : 1). The surround must cover that (SD-FF-9) |
| A-15 | 2D stills: `harness/capture.gd` `shoot()` — needs a window, waits `frame_post_draw` | `clients/2d/scripts/harness/capture.gd` | The 2D check renders offscreen `SubViewport`s sharing the main `world_2d` |
| A-16 | Pack validation: `mineworld packs validate` reads `pack.yaml` and the style manifest (`id`, `dimension`); world resolution `packages/src/resolve.rs` `resolve` (l.115) and `requirement` (l.185) accept a `presentation-pack` requirement; errors are `PackageError` (`packages/src/error.rs`) | `tools/cli/src/packs.rs:259-300`; `packages/src/manifest.rs:45-170`; `packages/src/resolve.rs` | The far-field refusals join `PackageError`; `validate` checks every pack directory; `resolve` checks the world's own far field in each required Presentation Pack |
| A-17 | Only `lakeside` requires the Presentation Packs (`requires: mineworld-default-2d, mineworld-default-3d`); Milestone E's test validates it with `--packs presentation/mineworld-default` | `worlds/lakeside/world.yaml:16-20` | Both packs must ship `far_field/lakeside.yaml` in the same commit that makes the check strict, or Milestone E's test breaks (C3 before C2's strictness takes effect, §12) |
| A-18 | 2D Godot tests launch the 2D client for `market-town` (`godot2d/mod.rs:29`), a scratch copy of `market-town` (`client_2d_interact.rs:371`) and `social-cafe` (`client_2d_interact.rs:433`, `client_settings.rs:30`) | the test files | Their launch helpers must pass `--world-key` (SD-FF-7) |
| A-19 | Shared client modules are symlinks (`clients/{2d,3d-spike}/mineworld` → `../protocol/mineworld`, `mineworld_settings` → `../shared/settings`); both launchers loop over them; the Windows nightly checks they are directories | `mineworld-2d:75-80`; `mineworld-slice:73-78`; `nightly.yml` "The shared client modules are directories" | A new module `mineworld_environment` → `../shared/environment` (step-22 §7.1's planned home) is added in all four places |
| A-20 | The 3D slice shots directory is ignored except `shots/slice/preview/` and `shots/slice/review/`; 2D ignores `shots/*` except `shots/preview/` | `clients/3d-spike/.gitignore:4-8`; `clients/2d/.gitignore:7-8` | §7's committed shots need two `!` exceptions per client |
| A-21 | `clients/3d-spike/README.md:7` still advertises `./mineworld-slice --skyline-check`, removed by RL-b C8-R | the README | Fixed in C9 |
| A-22 | The promenade spike `./mineworld-3d` (`scenes/main.tscn`) is launchable; its mountains are north only, a 220 m lawn south; no CI job or root README uses it; S23's release plan lists it among the three launchers | `mineworld-3d`; `scripts/town.gd:61-155`; `git grep mineworld-3d`; step-23 §97 | Q-FF-3 |
| A-23 | PR #200 (S25 25a, open) generates `docs/reference/pack-manifest.md` from `mineworld_packages::{pack_file_schema, style_manifest_schema}` via `tools/cli/src/reference/entries.rs` `pack_manifest`; `mineworld reference --check` fails on drift | `origin/mvp0/pr-s25a` | §5.8 |
| A-24 | `WorldSummary` (the welcome's `world`) carries no world or pack id | `server/src/protocol/summary.rs:92-120` | SD-FF-7, Q-FF-4 |
| A-25 | Real Santa Fe Depot: 32.7166 N, −117.1695 W (approximate; re-read at freeze); from the slice origin 32.7157, −117.1611 that is bearing ≈ 277°, distance ≈ 0.79 km | equirectangular from published coordinates | Default `station` slot position (Appendix B.1) |

### 3.2 Findings

- **F-FF-1. A winding assertion encoded the wrong convention and protected the bug.** A-2. The fix is not "assert
  harder" but to stop asserting a convention: build with Godot's convention, let `generate_normals()` derive
  normals from it, and prove visibility by rendering.
- **F-FF-2. The far field is everyone's and nobody's.** No document makes it required, no schema names it, the
  3D client has no pack to put it in, and `VISUAL_SLICE.md` §4's backdrop row asks only for "a distant element
  closing the view where the frontage ends", which the north-only ridges satisfied on paper.
- **F-FF-3. The angular reality of a far field is sub-degree at eye height.** From 1.8 m, ground at 1 500 m is
  0.07° below the horizontal and the sea horizon 0.14° below it (from a 20 m datum). A single "differ by N %"
  over a symmetric band cannot tell an open sea sector from a land sector with its silhouette deleted (both
  differ only below the horizon). The check therefore has two named criteria, fixed in §6.3.
- **F-FF-4. The 2D client cannot tell which world it presents.** A-24. Neither can a joined remote session.

### 3.3 What does not exist

No `far_field` anywhere; no 3D Presentation Pack reader; no `clients/shared/environment/`; no rendered
check of any kind in CI (DEP-45: "never frames"); no water in the slice; no committed far-field frames.

---

## 4. Reuse, in both directions (`REUSE_POLICY.md`)

| Need | Adopt / adapt / build | Why |
| --- | --- | --- |
| Distant silhouettes without parallax | **Adapt our own** RL-b C8 `skyline.gd` (`58bc0c4`): its band builder, curvature/refraction angle, Koschmieder haze, HDRI horizon sampling and camera-anchored root; its summit table moves into pack content | It was implemented, reviewed and measured, then reverted only because the operator wanted it **with the sea** (R-5). Rewriting it would discard reviewed work |
| Water before RL-f | **Reuse our own** `clients/3d-spike/shaders/water.gdshader` (41 lines, value-noise normals), with a capped Fresnel (SD-FF-5) | RL-f owns real water (flow maps, Gerstner/FFT, reflection probes, `DEP-RL-d`). A third-party ocean addon now would be decided twice |
| Far terrain from data | **Not now**: Terrain3D (runtime addon) and `tools/terrain-bake` are RL-e/RL-j decisions (`DEP-RL-a`) | Forcing a terrain framework into a placeholder far field is the wrong-wheel half of the reuse question |
| Offscreen rendering for the check | **Engine built-in**: `SubViewport` sharing the main `World3D`/`World2D`, `Camera3D.keep_aspect = KEEP_WIDTH`, `Viewport.get_texture().get_image()` | No image library; the pixel arithmetic is a few loops in GDScript |
| Pixel comparison | **Reuse** `clients/3d-spike/tools/frame_diff.gd`'s threshold convention (a pixel differs when a channel differs by more than 8/255) | One convention across V-gates and this check |
| Schema documentation | **Reuse** S25 25a's `schemars` reference (#200) | One reference generator |
| Typed parse in Rust | **Reuse** `serde` + `serde-saphyr` (DEP in force, `packages` already uses it) | — |

No new dependency is adopted and none is rejected in favour of our own code, so no `DEP` record is needed;
the reuse of `58bc0c4` is recorded in ARC-102's consequences.

---

## 5. Design

### 5.1 The `far_field` section of a Presentation Pack

#### 5.1.1 Where it lives

```text
presentation/<family>/<dimension pack>/
  pack.yaml                 package identity (unchanged)
  manifest.yaml             style manifest; its `dimension` list says 2d or 3d (unchanged)
  far_field/                REQUIRED (this PR)
    <world-key>.yaml        one per presented world; the file name is the scope's key
    region-<region-key>.yaml  one per S21 region (accepted now, resolved by no client until S21)
```

Each file is written in **YAML's JSON subset** (`DEP-16`), like `renderer/godot.yaml`, because the clients
read it with Godot's JSON parser and `packages` reads it with `serde-saphyr`. One file per scope keeps a pack
diffable and lets S21 stream a region's far field with its Asset Pack.

**SD-FF-1 — per dimension pack, not shared.** The 2D and 3D packs are separate packs (`ARC-3`), and their far
fields need different sections (a sky and silhouettes in 3D, a surround in 2D). Geography the two must agree on
(water bearings) is checked across the two packs by a test (AC-FF-4), not by a shared file that would make one
pack depend on another's layout. *Alternatives:* one family-level `far_field/` (crosses `ARC-3`'s
one-dimension-per-manifest rule); `far_field` inside `renderer/godot.yaml` (engine-specific file for
engine-neutral content).

#### 5.1.2 The schema (format 1)

Every field is required unless marked *optional*. Unknown keys are refused. Angles are **true bearings in
degrees, clockwise from north, in `[0, 360)`**. A **sector** `{from_deg, to_deg}` runs clockwise from
`from_deg` to `to_deg` and may wrap through 0 (`{315, 45}` is the 90° around north); `from_deg == to_deg` is
refused (use two sectors for a full ring). Distances are metres from the presented area's **origin** (3D: the
scene origin, the street's centre; 2D: the centre of the hub's extent). Heights are metres **above sea
level** unless the field says otherwise.

```text
FarField
  format              1                                   (only 1 exists)
  scope               {"world": WorldKey} | {"region": RegionKey}
                      WorldKey is a World Pack directory name; it must equal the file's stem.
                      RegionKey must equal the stem after "region-".
  boundary            Boundary
  water               [Water]                             (may be [] — a world with no water in sight)
  --- 3D packs (manifest dimension contains "3d"): all four required ---
  datum_m             f64 in [-430, 5000]                 the presented ground's elevation above sea level
  datum_source        string, non-empty                   its citation (I-FF-4)
  sky                 Sky
  foreground          Foreground
  terrain_silhouettes [Silhouette], at least one
  skyline             Skyline
  --- 2D packs (manifest dimension contains "2d"): required ---
  surround            Surround
  A section of the other dimension present in a file is refused (FAR_FIELD_SECTION_FOREIGN).

Boundary
  kind                "invisible_wall" | "natural"
                      natural = built or landscape terminations (parapet, bank, façades, dense planting)
  reach_m             f64 > 0      radius from the origin containing every reachable point AND every near
                                   construction of the presentation; every far-field element lies beyond it
  beyond              "drop" | "water" | "scrub" | "planting" | "rough_ground"
                      what the ground just past the boundary is. No value names paving, a path, steps,
                      a road or a bridge: nothing beyond the boundary suggests a walkable approach

Sky (3D)
  kind                "panorama" | "procedural"
  panorama            AssetName                           required iff kind = panorama; the client
                                                          resolves it in its own asset catalogue
  procedural          {"top": Colour, "horizon": Colour}  required iff kind = procedural
  visibility_m        f64 in [1000, 300000]               meteorological optical range V (Koschmieder)
  visibility_source   string, non-empty

Foreground (3D)    the ground from boundary.reach_m outward to the client's far distance
  surface             "scrub" | "dry_grass" | "grass" | "rough_ground"
  drop_m              f64 in [0, 500]   how far the ground falls below the datum, reached over run_m
  run_m               f64 in [10, 2000] the horizontal distance over which it falls
                      (In a water sector the ground ends at the shore, at the water's level instead.)

Silhouette (3D)    one band of distant terrain
  name                Key
  layer               "near" | "mid" | "far"              draw order and proxy distance (§5.4)
  sector              Sector                              where the band exists
  albedo              Colour                              surface colour before haze
  profile             [ProfilePoint], ≥ 2, bearings strictly increasing along the sector (wrap allowed),
                      first point at sector.from_deg and last at sector.to_deg
  source              string, non-empty
ProfilePoint
  bearing_deg         f64
  height_m            f64 in [-430, 9000]   crest height above sea level at that bearing
  distance_m          f64 > boundary.reach_m

Water
  name                Key
  kind                "sea" | "bay" | "lake" | "river"
  sector              Sector
  shore_m             f64 ≥ boundary.reach_m   nearest shoreline in the sector
  level_m             f64 in [-430, 5000]      surface above sea level (sea and bay: 0)
  horizon             "open" | "closed"        open = water to the horizon with nothing behind it
  colour              {"deep": Colour, "shallow": Colour}
  source              string, non-empty
  --- river only ---
  width_m             f64 in [1, 5000]
  course              [{"bearing_deg": f64, "distance_m": f64}], ≥ 2

Skyline (3D)       the town's distant massing
  sectors             [MassingSector], ≥ 1
  seed                u32       the massing's deterministic layout
  slots               {"station": Slot, "landmark": Slot}   both keys required, no others
MassingSector
  sector              Sector
  from_m, to_m        f64, boundary.reach_m < from_m < to_m
  height_m            {"min": f64, "max": f64}, 0 < min ≤ max ≤ 400, above the ground it stands on
  density             f64 in (0, 1]   share of plan area covered by buildings
  source              string, non-empty
Slot
  {"state": "unplaced"}
  | {"state": "reserved", "bearing_deg": f64, "distance_m": f64 > reach_m, "footprint_m": f64 > 0,
     "source": string}
  A reserved slot keeps its footprint clear of massing. "filled" does not exist in format 1: the station PR
  adds it (an additive change, not a format bump).

Surround (2D)
  extent_m            f64 > 0   drawn this far beyond the land rectangle on every side (plan metres)
  base                "land" | "water"   the kind that fills the surround where no water body is declared
  haze                {"colour": Colour, "from_m": f64 ≥ 0, "to_m": f64 > from_m}
                      blend toward this colour with distance beyond the land rectangle

Colour   "#rrggbb"           Key   1–48 of a-z, 0-9, -           AssetName   1–64 of a-z, 0-9, _, -
```

#### 5.1.3 Validation rules (beyond types), each with its refusal code

| Code | Refused when |
| --- | --- |
| `FAR_FIELD_MISSING` | a presented world (or a world that `requires` the pack) has no `far_field/<world-key>.yaml` in the pack |
| `FAR_FIELD_WORLD_UNKNOWN` | a client was started without a world key it can resolve (SD-FF-7) |
| `FAR_FIELD_MALFORMED` | not parseable, an unknown key, a wrong type or enum value (with line and column where the parser gives them) |
| `FAR_FIELD_SCOPE_MISMATCH` | `scope` disagrees with the file name |
| `FAR_FIELD_SECTION_MISSING` | a section the pack's dimension requires is absent |
| `FAR_FIELD_SECTION_FOREIGN` | a section of the other dimension is present |
| `FAR_FIELD_GAP` | **3D:** the union of all `terrain_silhouettes[].sector`, `skyline.sectors[].sector` and the sectors of `water` with `horizon: open` does not cover `[0, 360)`; the message names the first uncovered interval `[a, b)` to 0.1°. **2D:** `surround.extent_m` is not positive |
| `FAR_FIELD_INSIDE_BOUNDARY` | any distance (`profile[].distance_m`, `shore_m`, `from_m`, a reserved slot's `distance_m`) ≤ `boundary.reach_m` |
| `FAR_FIELD_PROFILE` | profile points out of sector, not strictly increasing along the sector, or not starting/ending at the sector's edges |
| `FAR_FIELD_OUT_OF_RANGE` | a number outside the ranges above |
| `FAR_FIELD_UNCITED` | a `source` (or `datum_source`, `visibility_source`) is empty |
| `FAR_FIELD_SLOT` | a slot key other than `station`/`landmark`, a missing one, or a state other than `unplaced`/`reserved` |

The codes are one list, shared by Rust (`FarFieldCode::as_str`) and GDScript (`MineWorldFarField.CODES`); a
cross-check (AC-FF-3) runs both validators over one corpus of planted-bad files and requires the same code
for each.

#### 5.1.4 Where a missing far field is refused

1. **Authoring:** `mineworld packs validate <pack-dir>` checks every file under `far_field/` (and refuses a
   Presentation Pack with **no** `far_field/` directory: `FAR_FIELD_MISSING` naming the pack and "no
   far_field/"). `mineworld validate <world>` (through `packages::resolve`) refuses a world that `requires`
   a Presentation Pack lacking `far_field/<world-key>.yaml`: `FAR_FIELD_MISSING { pack, world }`.
2. **Client load:** before drawing anything, each client resolves `<pack>/far_field/<world-key>.yaml`, parses
   and validates it with the shared module, and on any refusal prints `[mineworld-<client>] FAR_FIELD_<CODE>:
   <detail>`, shows the same line on screen, and — in any scripted or harness mode — exits with status 3.
   Interactive play shows the error screen and does not enter the world. **No fallback far field exists**
   (SD-FF-8).

**Exemption, stated so it is not a loophole:** the 2D client's `--presentation=none` is a *plain diagnostic
drawing with no Presentation Pack* (A-14, `presentation.is_plain()`); it is not a presentation and has no
far field. It prints `[mineworld-2d] plain drawing: no Presentation Pack, no far field (diagnostic mode)`.
The operator ruled it so (Q-FF-7, 2026-10-15).

#### 5.1.5 How the rule reads in the specification (summary; full text Appendix A.1)

`MODULE_SPEC.md` §6.1's layout gains `far_field/` marked **required**, and "Not every pack needs every
directory" gains "except `far_field/`". A new §6.1.1 carries §5.1.2–§5.1.4 as the normative schema.

#### 5.1.6 The two named slots

`skyline.slots.station` and `skyline.slots.landmark` exist in every 3D far field. RL-e0 sets `station` to
`reserved` at the real Santa Fe Depot's bearing and distance from the slice origin (A-25), with a 120 m
footprint kept clear of massing, and `landmark` to `unplaced` (Q-FF-8, ruled 2026-10-15: **the station is the landmark**; the landmark slot stays `unplaced` for the
station-and-landmark PR to decide). The station PR
after F adds the `filled` state with its model reference.

### 5.2 Rust: `packages/src/far_field.rs`

```rust
//! A Presentation Pack's far field (`MODULE_SPEC.md` §6.1.1; ARC-102).
pub const FAR_FIELD_DIR: &str = "far_field";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FarFieldCode { Missing, WorldUnknown, Malformed, ScopeMismatch, SectionMissing,
    SectionForeign, Gap, InsideBoundary, Profile, OutOfRange, Uncited, Slot }
impl FarFieldCode { pub const fn as_str(self) -> &'static str { /* "FAR_FIELD_MISSING", … */ } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dimension { TwoD, ThreeD }          // from the style manifest's `dimension`

#[derive(Debug, Deserialize)] #[serde(deny_unknown_fields)]
pub struct FarField { pub format: u32, pub scope: Scope, pub boundary: Boundary,
    pub water: Vec<Water>,
    #[serde(default)] pub datum_m: Option<f64>, #[serde(default)] pub datum_source: Option<String>,
    #[serde(default)] pub sky: Option<Sky>, #[serde(default)] pub foreground: Option<Foreground>,
    #[serde(default)] pub terrain_silhouettes: Option<Vec<Silhouette>>,
    #[serde(default)] pub skyline: Option<Skyline>, #[serde(default)] pub surround: Option<Surround> }
// Scope, Boundary, BoundaryKind, Beyond, Sky, SkyKind, Foreground, Surface, Silhouette, Layer,
// ProfilePoint, Water, WaterKind, Horizon, WaterColour, CoursePoint, Skyline, MassingSector,
// HeightRange, Slots { station: Slot, landmark: Slot }, Slot (#[serde(tag = "state")]),
// Surround, SurroundBase, Haze, Sector { from_deg: f64, to_deg: f64 }, Colour (newtype, parses "#rrggbb"),
// Key, AssetName (newtypes with the patterns of §5.1.2) — each a named type, no maps, no `Value`.

pub fn read(path: &Path, dimension: Dimension) -> Result<FarField, PackageError>;   // parse + validate
pub fn check_pack(dir: &Path, dimension: Dimension) -> Result<Vec<String>, PackageError>; // all files; keys
pub fn require_for_world(dir: &Path, pack: &str, world: &str) -> Result<(), PackageError>;
pub fn uncovered(sectors: &[Sector]) -> Option<(f64, f64)>;  // first gap, 0.1° resolution, for FAR_FIELD_GAP
```

`PackageError` gains `FarField { path: PathBuf, code: FarFieldCode, detail: String }` and
`FarFieldMissing { pack: String, scope: String }`; their `Display` starts with the code string so a user and
a test both see the name. `check_style_manifest` is extended to return the parsed `dimension` (its signature
changes to `-> Result<Vec<Dimension>, PackageError>`; its one caller in `tools/cli/src/packs.rs` is updated).
A style manifest listing both `2d` and `3d` validates its far fields against both sets of requirements.

### 5.3 The shared GDScript module: `clients/shared/environment/far_field.gd`

`class_name MineWorldFarField` (`RefCounted`). `static func load_for(pack_dir: String, world_key: String,
dimension: String) -> MineWorldFarField` returns an instance whose `error_code` is `""` on success or one of
`CODES`, and `error_detail`. It parses with `JSON.parse_string` (DEP-16), applies the same rules as §5.1.3
(sector coverage at 0.1° steps, distances beyond `reach_m`, required sections by dimension), and exposes typed
read-only accessors used by the two clients (`boundary()`, `water()`, `sky()`, `foreground()`,
`silhouettes()`, `skyline()`, `surround()`, `datum_m()`). It never writes anything and never reaches an
intent (`INV-RL-2`, the client-rules scan). The module is symlinked into both clients as
`mineworld_environment` (A-19), the home step-22 §7.1 planned for `clients/shared/environment/`.

### 5.4 3D: the far-field view (`clients/3d-spike/scripts/slice/far_field_view.gd`)

`class_name SliceFarFieldView extends Node3D`. `SliceWorld.build` calls `SliceFarFieldView.build(g,
far_field)` in place of `SliceStreet.backdrop(g)`; `street.gd`'s `backdrop()` and `_ridge()` are deleted and
`_surrounds()` loses its 900 m apron (the near ground becomes a disc of radius `reach_m`, built by the same
annulus builder with inner radius 0). Every node the view creates carries meta `mw_category =
"farfield.<part>"` with part ∈ {`silhouette`, `water`, `foreground`, `skyline`}; the near-ground disc keeps
`mw_category = "ground"`. `SliceBatch`, `SliceBudget` and `SliceOccluders` skip `farfield.*` (A-1).

| Part | Anchoring | Geometry | Material |
| --- | --- | --- | --- |
| **silhouette** (one mesh per layer) | camera-anchored: a child root copies the active camera's **x and z** every frame; its **y** is offset per layer by `−(cam_y − eye0) · D_p / d_layer` so an elevated camera sees distant terrain drop by its true angle (d_layer = the layer's median `distance_m`; eye0 = `WALK_Y + EYE_HEIGHT`) | ribbons at proxy distance `D_p` = 900 / 1 100 / 1 300 m for near / mid / far, 1 column per 0.5° of the sector; crest at `eye0 + D_p · tan(α)` with `α = (h − datum − eye0 − d²(1−k)/(2R)) / d` (R = 6 371 km, k = 0.13; RL-b C8), heights interpolated between profile points with smoothstep (no invented relief); **base at `−(foreground.drop_m + 40)` m** so the ribbon always reaches below the ground it stands behind | unshaded, `disable_fog = true`, `cast_shadow` off, colour `lerp(horizon, albedo, exp(−2.996 · d / V))` per column (V = `sky.visibility_m`; horizon sampled from the panorama ±1° about the horizon at that bearing), −8 % luminance crest → base; **default `CULL_BACK`** |
| **foreground** | world-anchored at the origin | annulus from `reach_m` to 1 450 m, 128 radial × 16 ring segments; y(r) = `−drop_m · smoothstep(reach_m, reach_m + run_m, r)`; in a water sector it ends at `shore_m` at the water's level (`level_m − datum_m`) | the existing `leafy_grass` PBR, world-space UVs, tinted to the declared `surface` (scrub/dry grass: sRGB ≈ (0.38, 0.36, 0.27), RL-b C8's chaparral albedo inside DEP-8's 0.2–0.7 band); scene fog applies |
| **water** | the surface world-anchored; the horizon ribbon camera-anchored | a disc sector from `shore_m` to 1 450 m at `level_m − datum_m`; a **horizon ribbon** at `D_w = 1 400 m` from `y_w − 50` up to the line of sight at the true dip, `crest_y = cam_y − D_w · tan(dip(cam_y − y_w))`, `dip(h) = sqrt(2h/R)`, updated in `_process` by the node's `scale.y` | `water.gdshader` with `colour.deep`/`shallow`, normals from its noise, Fresnel **capped at 0.6** (SD-FF-5); the ribbon unshaded in the colour the surface reaches at the horizon (`lerp(horizon, deep, 0.4)`), no fog |
| **skyline** | world-anchored | for each massing sector, deterministic boxes from `seed`: footprint 8–20 × 8–16 m, height uniform in `[min, max]`, placed by rejection sampling until `density` of the annular sector's plan area is covered, standing on the foreground's y at their radius, none inside a reserved slot's footprint, merged into **one `ArrayMesh` per sector**; a low pitched cap on half of them (seeded) | wall palette from `SlicePalette` (three colours), roofs one tile colour; scene fog; `cast_shadow` off |

**SD-FF-3 — proxies inside the far plane.** Real distances are 2–72 km, the camera's far plane is 1 600 m and
must not change (A-5). Silhouettes are therefore camera-anchored proxies at their real *angles* (as RL-b C8),
while things within 1.45 km (foreground, the bay's near water, massing) are world-anchored, so walking 34 m
along the street shows true parallax where it is visible and none where it would not be.

**SD-FF-4 — winding by convention, visibility by evidence.** Ribbons are emitted in Godot's front-face order
(clockwise as seen from the camera, i.e. from inside the ring) and `SurfaceTool.generate_normals()` derives
the normals; the material keeps `CULL_BACK`. No code asserts a winding. The rendered check (§6) is what proves
the faces are drawn, and X-1 (reverse the order) must make it fail. *Rejected:* `CULL_DISABLED` (it would
hide the next winding mistake instead of exposing it, and X-1 could no longer fail); keeping a corrected
winding assertion (F-FF-1: an assertion of a convention is exactly what failed).

**SD-FF-5 — the bay before RL-f.** A visual plane only: no reflection probe, no flow, no foam, no wind
coupling. Fresnel is capped at 0.6 because a wind-roughened sea near the horizon is darker than the sky
behind it — which is why the real sea horizon is a visible line — and because an uncapped grazing Fresnel
would render the water sky-coloured exactly where the check needs it to differ from the sky (R-FF-3).
The operator accepted this (Q-FF-2, 2026-10-15).

### 5.5 2D: the surround (`clients/2d/scripts/scene/ground.gd`, `places.gd`)

`Ground` gains `var surround: Rect2` and `var far_field`, and draws, **before** `land`:

1. the surround's base: the land rectangle grown by `extent_m`, filled with the base kind's palette colour
   (`grass_lo` for land, a new palette key `water` for water);
2. each `water` body intersected with the surround: the sector wedge from the origin, clipped to
   `[shore_m, land edge + extent_m]` in plan, filled with `water` and a few seeded ripple strokes (the
   existing `_grass_strokes` pattern);
3. haze: eight concentric rectangles from `haze.from_m` to `haze.to_m` beyond the land edge, each blended
   toward `haze.colour` by its distance share.

`places.gd` sets `ground.surround = ground.land.grow(extent_m)` beside `ground.land` (A-14). **No camera
clamping:** the camera follows the player as today.

**SD-FF-9 — draw far enough; do not clamp.** The operator offered either. Clamping `Camera2D` limits to the
drawn extent does not prevent the clear colour when the viewport is wider than the limits (a 21.3 : 1 window,
A-14), and it would stop the player's camera following the player near an edge. Drawing far enough is checked
at the widest aspect the settings allow (§6.4). The default `extent_m = 400` covers the 21.3 : 1 view at zoom
1.55 (≈ 194 m half-width along the screen's horizontal, × √2 for the isometric diamond ≈ 275 m, + margin).
If C7's check shows a shortfall, C7 raises `extent_m` to the measured need + 20 % and records it (a bounded
discovery).

### 5.6 Default content (values in Appendix B)

- **3D `social-cafe`** (the slice depicts `social-cafe`'s street; the location is Market Town's,
  32.7157 N −117.1611 W, per QRL-2/QRL-3 — ruled by Q-FF-9): datum 18 m;
  V = 80 km; panorama `qwantani_puresky_2k`; foreground `dry_grass`, drop 18 m over 300 m (the bay lies
  ≈ 18 m below the street); silhouettes: Balboa/Bankers Hill mesas (near), Mount Soledad, Cowles, Helix and
  San Miguel (mid), Cuyamaca, Monument Peak and Otay (far), Point Loma and Coronado (near, across the bay),
  the hills south of the border (mid) — 360° with RL-b C8's re-read summits (2026-10-09) and hand-estimated
  mesa and lowland points marked *approximate, replaced by RL-e*; water: San Diego Bay 170° → 300°, shore
  1 000 m, closed horizon; skyline: low-rise 6–14 m, 250–1 400 m, density 0.25–0.35, the station slot
  reserved at 277°, 790 m; boundary natural, reach 60 m, beyond `drop`.
- **3D `lakeside`** (required by A-17; not drawn by any client yet): design content, no real place —
  `lakeside` has no location (`worlds/lakeside/` has no `configure/calendar.yaml`), so every `source` says
  "design content; lakeside declares no real location".
- **2D `market-town`, `social-cafe`** (San Diego): base land, extent 400 m, haze toward the slice's fog colour;
  the bay at 1 000 m lies beyond the 2D surround and is therefore not drawn — the 2D file still declares it,
  so AC-FF-4's cross-pack water check holds.
- **2D `lakeside`**: water base north (a lake from the land edge), land elsewhere. **2D `bodies-yard`**: land.

### 5.7 Which world a client presents (SD-FF-7)

The welcome carries no world id (A-24). RL-e0 adds a client argument **`--world-key=<key>`**:
`mineworld-2d` passes the basename of its `--world` directory (default `market-town`); a `--server` join
requires `--world-key` and otherwise refuses with `FAR_FIELD_WORLD_UNKNOWN`; the 2D Godot test helpers pass
it (A-18). The 3D slice **always** uses `social-cafe`, because the scene *is* `social-cafe`'s street (16f
generalises the 3D pack). *Alternative recommended for later:* `WorldSummary.pack: PackId` in the protocol
(Q-FF-4) — a contract change, outside a presentation-only PR (`INV-RL-11`).

**SD-FF-8 — no fallback far field.** A pack-wide default would make the refusal unreachable for the shipped
packs and would draw San Diego's bay behind an alpine world. Every presented world names its far field.
Ruled by the operator (Q-FF-7, 2026-10-15).

### 5.8 The generated reference (S25 25a, PR #200)

The far-field types derive `schemars::JsonSchema` with a doc comment on every field and one example file
(`social-cafe`'s), and `tools/cli/src/reference/entries.rs` `pack_manifest` gains a third entry:

```text
slug "far-field", title "`far_field/<world-key>.yaml` (required)",
about: "A Presentation Pack's far field, one file per presented world (`MODULE_SPEC.md` §6.1.1). A pack
        without one for a world is refused (FAR_FIELD_MISSING)."
schema: mineworld_packages::far_field_schema()
```

`docs/reference/pack-manifest.md` and `docs/reference/schema/far-field.schema.json` are regenerated with
`mineworld reference --out docs/reference` and verified with `--check`. **Order:** if #200 is merged at C2,
C2 includes this; otherwise commit C2-ref is skipped in RL-e0 and becomes the first commit of whichever of
#200 / RL-e0 merges second (§14.2).

---

## 6. The rendered check — fixed before anything is measured (`ARC-23`)

A ray-cast or schema check is not enough: the bug was render-time culling. The check therefore compares
rendered frames of **the same run**: the far field shown (S), hidden (H), and the sky alone (E). It never
compares with a stored image (A-13).

### 6.1 3D viewpoints (social-cafe; scene coordinates, metres)

| Id | Position (x, y, z) | Why |
| --- | --- | --- |
| VP-spawn | (−11.5, 1.80, −5.55) | `SliceWorld.SPAWN` at eye height (`WALK_Y` 0.14 + 1.66) |
| VP-street | (0.0, 1.66, 0.0) | the `street.main` Place: the carriageway centre at eye height |
| VP-cafe | (3.45, 1.80, −5.60) | the `cafe.main` Place's doorway, from the pavement (`slice_probe.gd` `outside_pavement`) |
| VP-florist | (11.84, 1.80, −5.60) | the `florist.main` Place's doorway, from the pavement |
| VP-west | (−33.5, 1.80, 0.0) | the parapet end (the investigation's `ff_6` direction) |
| VP-east | (33.5, 1.66, 0.0) | the planted-bank end |
| VP-high | (0.0, 40.0, 0.0) | the one elevated point (the investigation's aerials' height; also S21's window-scenery case) |

At each viewpoint, **8 azimuths** at bearings 0°, 45°, …, 315°, pitch 0.

### 6.2 Renders

- One offscreen `SubViewport`, **1 440 × 240**, sharing the main `World3D`; a `Camera3D` with `keep_aspect =
  KEEP_WIDTH`, `fov = 50` (horizontal), `near = 0.05`, `far = CameraRig.FAR`. Vertical field ≈ 8.89°,
  ≈ 27 rows per degree at the centre.
- The slice runs **offline** (`--settings=none`, `--gi=none`; GI does not change what is drawn where), with the
  player body and HUD hidden.
- Per viewpoint and azimuth: **S** (everything shown) and **H** (every node with `mw_category` beginning
  `farfield.` hidden). Per azimuth once: **E** (the slice root hidden; sky only — E does not depend on
  position). 7 × 8 × 2 + 8 = **120 renders**. Each render waits 3 frames and `RenderingServer.frame_post_draw`.
- Pixel arithmetic on 8-bit sRGB from `get_image()`: two pixels **differ** when any channel differs by more
  than **8** (frame_diff's 8/255); a pixel is **open** when H and E do *not* differ there (nothing near is
  drawn; the sky shows through).

### 6.3 Rows, columns and thresholds

- **Sector columns:** columns whose azimuth offset from the render's centre, `atan((x + 0.5 − 720) / 720 ·
  tan 25°)`, lies within ±22.5°. The 8 sectors tile 360° exactly.
- **Horizon:** `e_h = −dip(datum_m + cam_y)` with `dip(h) = degrees(sqrt(2h / 6 371 000))` (≈ −0.14° at eye
  height, ≈ −0.24° at VP-high). A row's elevation is `atan((119.5 − row) / 120 · tan 4.444°)`.
- **Below band:** rows with elevation in `[e_h − 1.0°, e_h − 0.1°]`. **Above band:** `[e_h, e_h + 0.25°]`.
- **FF-BELOW (horizon closure), N_below = 90 %.** In every sector, of the open below-band pixels, at least 90 %
  must differ between S and H. Exempt (`covered-below`) when open below-band pixels are fewer than 2 % of the
  sector's below-band pixels.
- **FF-ABOVE (silhouette presence), N_above = 25 %.** In every sector whose centre bearing is **not** inside a
  `water` sector with `horizon: open`, of the open above-band pixels, at least 25 % must differ between S and H.
  Exempt (`covered-above`) when open above-band pixels are fewer than 2 % of the sector's above-band pixels.
- **Why two criteria (F-FF-3).** Below the horizon a correct far field replaces *all* the sky (foreground,
  water), so 90 % holds with margin; a missing or culled far field leaves sky there. Above the horizon, terrain
  covers part of the band and the open sea legitimately covers none, so only land sectors are held to it; a
  deleted or culled silhouette leaves 0 %. One symmetric N over one band would have to be low enough for an
  open sea (≈ 4 % at eye height) and would then pass a deleted south band.
- **Sector naming:** N, NE, E, SE, S, SW, W, NW by centre bearing.

### 6.4 2D

- For each world with a 2D far field (four), host the world as the 2D tests do and run the 2D client with
  `--drive=farfield --world-key=<key>`. After the town is drawn, for each of the **8 reachable extremes** —
  the corners and edge midpoints of `town.hub_extent()` — and each **aspect** in {1 600 × 900 (16:9),
  3 200 × 900 (32:9), 4 800 × 225 (21.3 : 1, the settings' 7 680 × 360), 506 × 3 416 (1 : 6.75, the settings'
  640 × 4 320)}, render an offscreen `SubViewport` of that size sharing the main `world_2d`, with a `Camera2D`
  centred on the extreme at `zoom = pack zoom × (H / 900)` (the `canvas_items`/`expand` equivalent).
- **Two sentinels:** render once with `RenderingServer.set_default_clear_color(#ff00ff)` and once with
  `#00ff00`; a pixel that differs between the two is a clear-colour pixel. **PASS: zero such pixels in every
  render** (4 worlds × 8 × 4 × 2 = 256 renders). The camera's zoom is the pack's fixed zoom, which is the
  widest the client allows today; a later zoom control must declare its minimum and the check uses it.

### 6.5 Where it runs

- **Rust harness** `tools/cli/tests/client_far_field.rs`, two `#[ignore]` tests modelled on
  `client_settings.rs` (`run_slice`, `check`): `three_d_far_field_closes_every_open_sector` (offline slice,
  `--slice-farfield`) and `two_d_clear_colour_is_never_visible` (hosts each world, `--drive=farfield`). Each
  judges the printed verdict lines, never the exit status alone.
- `scripts/ci_godot.py` `GODOT_TESTS` gains `"client_far_field"` → the nightly `clients` layer runs both on
  **ubuntu-24.04 (Xvfb, lavapipe), macos-26 (Apple Paravirtual) and windows-2025 (D3D12 WARP)** — offscreen
  through `SubViewport`s in a real (virtual or hosted) display session. True `--headless` is impossible: Godot's
  headless display server uses a dummy renderer and draws no pixels.
- Locally: `./mineworld-slice --farfield` and `./mineworld-2d --drive=farfield --capture` print the same
  verdicts.
- Per PR (Q-FF-5, ruled 2026-10-15): a Linux job, path-filtered to `clients/**`, `presentation/**` and the
  launchers (C8 step 4).

### 6.6 Verdict lines

```text
[farfield] VP-west W  below 412/412 open, 412 differ (100.0%) >= 90%   ok
[farfield] VP-west W  above  96/96  open,  71 differ ( 74.0%) >= 25%   ok
[farfield] VP-cafe N  covered (open 0.0% < 2%)
FAIL farfield VP-high S above: 0/88 differ (0.0%) < 25%
all farfield checks pass            |   <n> FARFIELD CHECKS FAILED
[farfield2d] market-town NE 4800x225: 0 clear-colour px   ok
FAIL farfield2d market-town E 4800x225: 18 211 clear-colour px
```

### 6.7 Planted mutations (each must FAIL by name, then be reverted)

| Id | Plant | Must fail with |
| --- | --- | --- |
| **X-1** back-face the ridges again | in `far_field_view.gd`'s ribbon builder, swap the second and third vertex of every triangle | `FAIL farfield … above` in every non-covered land sector (and `below` at VP-high) |
| **X-2a** delete the south band (content) | remove the silhouette whose sector covers 180° from `presentation/mineworld-default/3D/far_field/social-cafe.yaml` | `mineworld packs validate presentation/mineworld-default/3D` → `FAR_FIELD_GAP [a, b)` naming the gap; the client refuses at load with the same code |
| **X-2b** delete the south band (render) | in `far_field_view.gd`, skip building any silhouette whose sector contains 180° | `FAIL farfield VP-east S above` and `VP-high S above` (the bay keeps FF-BELOW passing — which is why FF-ABOVE exists) |
| **X-3** shrink the 2D surround | set `surround.extent_m` to `10` in `presentation/mineworld-default/2D/far_field/market-town.yaml` | `FAIL farfield2d market-town … clear-colour px` at the 21.3 : 1 aspect |
| **X-4** the apron edge returns | build the foreground annulus to 450 m instead of 1 450 m | `FAIL farfield VP-high <sectors> below` |
| **X-5** no far field at all | delete `presentation/mineworld-default/3D/far_field/social-cafe.yaml` | slice refuses `FAR_FIELD_MISSING` and exits 3; `mineworld validate worlds/lakeside --packs …` unaffected (lakeside's file stays); deleting `lakeside.yaml` instead → `FAR_FIELD_MISSING { pack: mineworld-default-3d, world: lakeside }` |

---

## 7. Regression shots

| Client | Path (committed) | Views |
| --- | --- | --- |
| 3D | `clients/3d-spike/shots/social-cafe/farfield/` | the investigation's poses, so before/after pairs exist: `ff_1_north_street` (0, 1.8, −1) yaw 0 pitch 8; `ff_2_north_aerial` (0, 40, 25) yaw 0 pitch −4; `ff_3_south_aerial` (0, 40, −25) yaw 180 pitch −4; `ff_4_east_aerial` (−30, 40, 0) yaw −90 pitch −4; `ff_5_west_aerial` (30, 40, 0) yaw 90 pitch −4; **`ff_6_west_street_end`** (−24, 1.8, 0) yaw 90 pitch 2 (the parapet end); `ff_7_east_street_end` (24, 1.8, 0) yaw −90 pitch 2 |
| 2D | `clients/2d/shots/<world>/farfield/` for the four worlds | `ff2d_n`, `ff2d_e`, `ff2d_s`, `ff2d_w` (camera at the reachable extreme in that direction, 1 600 × 900), `ff2d_wide` (the east extreme at 4 800 × 225) |

The operator's "four compass views plus one elevated view per world": in 3D the four compass views are
`ff_2`–`ff_5` (from 40 m, where every direction is open), the eye-level views `ff_1`, `ff_6`, `ff_7` are added
because the street hides N and S at eye height; in 2D the four compass extremes plus the widest view. Yaw 0
faces north (−z); FOV 70; 1 600 × 900, saved as JPEG quality 85 at 960 × 540 (≈ 150 KB each; 7 + 20 files).
Written by `./mineworld-slice --farfield-shots` and `./mineworld-2d --drive=farfield --capture
--shots=clients/2d/shots/<world>/farfield`. `.gitignore` exceptions: `!shots/social-cafe/`,
`shots/social-cafe/*`, `!shots/social-cafe/farfield/` in `clients/3d-spike/.gitignore`; the same pattern per
world in `clients/2d/.gitignore`.

**V-gates for later PRs.** A later PR that changes a far field compares its shots with these by `frame_diff.gd`
and presents both to the operator; a PR that does not intend to change the far field must keep every far-field
view within the base's own noise + 0.5 points (16c §20.6's rule), and the rendered check stays green.

**V-1 for this PR** (`VIS-3D-GODOT-2` accepted views must not regress except in far-field pixels): for every
`--shots` view, base vs head `frame_diff` **outside the far-field mask** (pixels where this PR's head renders S
≠ H at that pose) is within the base's own noise + 0.5 points.

---

## 8. Performance

**Budget.** Step-22 §6.6 allots at Default: far terrain +1.0 ms (RL-e), sky and atmosphere +0.5 ms, water
+1.5 ms (RL-f). RL-e0 spends **at most +0.5 ms from RL-e's far-terrain share** (silhouettes, foreground,
skyline) **and +0.5 ms from RL-f's water share**, **≤ +1.0 ms p95 in every RL-b M-1 view**, and in counts
**≤ +24 draw calls, ≤ +100 000 primitives, ≤ +32 MB video memory** per view. Absolute gates stay R-2's: p95 ≤
16.7 ms, video memory ≤ 2 048 MB; draw calls and primitives advisory. Two views (cafe frontage 17.11 ms,
doorway 18.59 ms) are over today and are RL-b2's to fix; RL-e0 is judged on its **increment**, and must not
raise those two views' p95 by more than its share.

**Estimate.** Removed: 3 ridge meshes. Added: 3 silhouette meshes, 1 foreground, 1 water surface, 1 horizon
ribbon, 2–3 massing meshes ≈ +5 meshes × (depth prepass + colour; none casts a shadow) ≈ **+10 draws**;
≈ 4.3 k (ribbons) + 4.1 k (foreground) + 2 k (water) + ≈ 6 k (massing) ≈ **+17 k primitives**; no new texture
(`leafy_grass` reused, procedural water normals) ≈ **+0–4 MB**. The foreground replaces the 900 m apron's
fill, so fill-rate grows only where water or ribbons are on screen.

**Measurement.** Locally: one `./mineworld-slice --perf` run on the base and on the head, **counts only**
(draw calls, primitives, objects, video memory — deterministic); timings from a loaded development machine are
recorded as **INCONCLUSIVE**. The operator runs RL-b's three-run protocol (R-6, display awake) on the head, as
an item of Milestone F's batched list (§14.3). A count increment above the cap is a material stop; a measured
p95 increment above +1.0 ms in any view is a material stop.

---

## 9. Invariants

| Id | Invariant |
| --- | --- |
| I-FF-1 | Every presented world has a far field. A missing one is refused by name at authoring (`validate`, `packs validate`) and at client load; no fallback is drawn. The only exemption is the 2D plain diagnostic mode, which says so |
| I-FF-2 | The far field closes every direction: 3D sectors cover 360° (schema) and the rendered check passes FF-BELOW and FF-ABOVE at every fixed viewpoint; in 2D no clear-colour pixel is drawn at any reachable extreme and aspect |
| I-FF-3 | The far field is decoration: beyond `boundary.reach_m`, unreachable, never disclosed, never in an intent; nothing beyond the boundary suggests a walkable approach (no path/paving/steps/road values exist) |
| I-FF-4 | Real units; every realistic default cites its source (`INV-RL-6`); approximate values say so and name their replacement (RL-e) |
| I-FF-5 | No kernel, contract, protocol or persistence change (`INV-RL-11`); the server never reads a far field |
| I-FF-6 | Accepted near views do not regress outside far-field pixels (V-1, §7) |
| I-FF-7 | RL-e0's cost stays within §8's caps |
| I-FF-8 | No asserted winding: visibility is proven by rendering, and X-1 fails |

---

## 10. Acceptance and adversarial criteria (fixed before measuring)

| Id | Criterion | Evidence |
| --- | --- | --- |
| AC-FF-1 | `cargo test -p mineworld-packages` passes, including one table test per refusal code of §5.1.3 asserting the code and the named detail (the gap interval for `FAR_FIELD_GAP`) | test log |
| AC-FF-2 | `mineworld packs validate presentation/mineworld-default/2D` and `…/3D` print the identity and exit 0; `mineworld validate worlds/lakeside --packs entities --packs presentation/mineworld-default` exits 0; Milestone E's test (`tools/cli/tests/milestone_e.rs`) passes | command output |
| AC-FF-3 | The GDScript validator and the Rust validator return the same code for every file of `packages/tests/fixtures/far_field/` (≥ 1 file per code, plus the four shipped valid files of each dimension) | `client_far_field::the_two_validators_agree` |
| AC-FF-4 | For each world with both a 2D and a 3D far field, the water sectors agree within 1° and the shores within 1 m | `packages` test `two_d_and_three_d_water_agree` |
| AC-FF-5 | The 3D rendered check prints `all farfield checks pass` on the head on all three nightly OSes (or the PR's dispatched nightly run on the exact head) | CI job logs |
| AC-FF-6 | The 2D rendered check prints `all farfield2d checks pass` on all three OSes | CI job logs |
| AC-FF-7 | X-1, X-2a, X-2b, X-3, X-4, X-5 each fail with the named line (§6.7), locally on macOS; X-1 and X-3 also on one nightly run with the mutation on a scratch branch (`scratch/**`), then deleted | ledger |
| AC-FF-8 | The committed shots exist (7 + 20), and `ff_6` shows the bay and Point Loma across the drop beyond the parapet with no apron edge | files; the operator's look (§14.3) |
| AC-FF-9 | V-1 passes (§7) | `frame_diff` output |
| AC-FF-10 | Counts within §8's caps on the M5; operator timings recorded or the item handed to F's list | ledger |
| AC-FF-11 | `clients/3d-spike/README.md` no longer mentions `--skyline-check`; `./mineworld-3d` with no harness flag refuses to start a walk and points at the slice, while `--drive`, `--shots` and `--portrait` still work (Q-FF-3) | grep; the launcher's output |
| AC-FF-12 | The client-rules scan (`tests/acceptance/tests/client_rules.rs`, `scripts/check_client_rules.py`) passes with the new module | test log |

---

## 11. Test ownership (test rules §26)

| Layer | Owns | Here |
| --- | --- | --- |
| Static | format, lint, doc headings, decision ids | `cargo fmt`, `clippy -D warnings`, `check_doc_headings.py`, `check_decision_ids.py`; `gdlint` N/A (none in the repository) |
| Unit / contract (Rust) | the schema, every refusal, the gap arithmetic, cross-pack water agreement | `packages/tests/far_field.rs` |
| Integration (Rust, real binary) | `packs validate`, `validate` with requirements | `tools/cli/tests/packs.rs` additions |
| Godot, real rendering (`#[ignore]`, nightly, three OSes) | the rendered checks; the two validators agree | `tools/cli/tests/client_far_field.rs` |
| Mutation | the check can fail | §6.7, recorded in §18 |
| Operator (by hand) | the look; timings on real GPUs | §14.3 |
| Real-LLM Gate | — | N/A: no language model is involved |

No test asserts a getter, an enum listing or a library's documented behaviour (`CLAUDE.md` §4 rule 8).

---

## 12. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately;
`[x]` needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason.
**Implementer** says which model the primary should assign (operator memory: well-specified → Haiku 5.5;
complex → Opus 5.5).

### 12.0 The procedure for every commit

Run every command from the worktree root named in §13. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full,
   before editing it.
2. Make the numbered implementation steps, in order. Do not do work that belongs to a later commit.
3. Run the commit's validation commands. Record for each: the command, the test count, the wall time, and
   PASS / FAIL / INCONCLUSIVE (a test not run is not a pass).
4. For each mutation the commit lists: plant it, run the named check, record that it failed (with the failing
   line), revert it, run the check again green. Never commit a planted mutation.
5. Static checks: Rust commits: `cargo fmt --all --check`; `cargo clippy -p <crate> --all-targets -- -D
   warnings`; `cargo test -p <crate>` (one crate at a time; never the whole workspace locally, R-LOAD-1).
   Markdown: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`. Godot commits:
   the client's own probe named in the commit.
6. Review: answer each review question of the commit in §18 in one or two sentences.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files** list. If
   another path changed, revert it or stop and record why.
8. Update §18 (ledger row, evidence, findings) and `handoff-rl-e0.md` in the same commit.
9. Stage by explicit path (`git add <path> …`, never `git add -A`), then `git commit -m "<message>"` with the
   message given, plus the attribution line your session specifies. Separate commands; no heredoc, no `sed
   -i`, no `python3 -c`, no `curl`.
10. Stop and report (a material stop, §13) instead of improvising if a step cannot be done as written.

### C0 — Freeze and contract (Markdown only) · Implementer: Haiku

- **Goal.** Start RL-e0 against the frozen design. **Files:** this file (§18); `.structured-coding/plans/mvp1/handoff-rl-e0.md` (new).
- Steps:
  1. `git log --oneline -1`; record the base commit in §18 and the handoff.
  2. Confirm the header reads `DESIGN FROZEN` and §13 is filled; copy §17.1's rulings into the handoff verbatim.
  3. Create the handoff with the fields of the working rules (project/PR, primary design, binding docs, branch,
     base, HEAD, working-tree fingerprint, approved scope, frozen invariants, endpoint authority with sources,
     sequence C0–C10, budget, checkpoint, next actions, stop conditions).
- [ ] Implementation: steps 1–3.
- [ ] Validation: both doc checks.
- [ ] Review: no authority line widened.
- **Commit message:** `docs(plan): RL-e0 — open the ledger and handoff`.

### C1 — Specifications before code · Implementer: Haiku (texts given in Appendix A)

- **Goal.** The rule exists in the specifications before any code (`CLAUDE.md` §2.2). **Files:**
  `docs/MODULE_SPEC.md`, `docs/ART_DIRECTION.md`, `docs/VISUAL_SLICE.md`, `clients/2d/PRESENTATION.md`,
  `docs/DECISIONS.md`.
- Steps:
  1. `MODULE_SPEC.md` §6.1: apply Appendix A.1 (layout line, the "except `far_field/`" sentence, new §6.1.1).
  2. `ART_DIRECTION.md` §12 and §21: apply Appendix A.2.
  3. `VISUAL_SLICE.md`: apply Appendix A.3 (§1 note, §4 termination paragraph `:129-130`, backdrop row `:144`
     → far field row, out-of-scope line, §12 view 11). Keep the file engine-neutral (its §0): no renderer,
     node or shader names.
  4. `clients/2d/PRESENTATION.md`: apply Appendix A.4.
  5. `DECISIONS.md`: append ARC-102, ARC-103 and ARC-104 (Appendix A.5),
     and the dated note on `DEP-45` (Appendix A.5, last block).
- [ ] Implementation: steps 1–5.
- [ ] Validation: both doc checks; `grep -n "far_field" docs/MODULE_SPEC.md docs/VISUAL_SLICE.md` shows the new text.
- [ ] Review: terms (`Presentation Pack`, `World Pack`, `Place`) used exactly; no rule stated only in a README;
  `VISUAL_SLICE.md` names no engine feature.
- **Commit message:** `docs: the far field is a required part of a Presentation Pack (RL-e0)`.

### C2 — `packages`: the typed far field and its refusals · Implementer: Haiku (types given in §5.2)

- **Goal.** One authoritative schema with named refusals. **Files:** `packages/src/far_field.rs` (new),
  `packages/src/error.rs`, `packages/src/lib.rs`, `packages/src/manifest.rs`, `tools/cli/src/packs.rs` (only
  the changed `check_style_manifest` signature at its one call), `packages/tests/far_field.rs` (new),
  `packages/tests/fixtures/far_field/*.yaml` (new corpus); with #200 merged also `packages/src/schema.rs`,
  `tools/cli/src/reference/entries.rs`, `docs/reference/pack-manifest.md`, `docs/reference/schema/far-field.schema.json`.
- Steps:
  1. Read `packages/src/manifest.rs` and `packages/src/error.rs` in full; mirror their style (doc comments,
     `deny_unknown_fields`, `malformed()` with the parser's position).
  2. Write `far_field.rs` per §5.2: the types, `read`, `check_pack`, `require_for_world`, `uncovered` (step the
     circle in 0.1° from 0.0 to 359.9 and return the first maximal uncovered run).
  3. `error.rs`: the two variants of §5.2, `Display` beginning with the code string.
  4. `manifest.rs`: `check_style_manifest` returns the dimensions; unknown dimension strings → `Malformed`.
  5. Fixtures: one file per code of §5.1.3 (except `WORLD_UNKNOWN`, which is client-only), named
     `<code-in-lower-case>.yaml`, plus `valid-3d.yaml` and `valid-2d.yaml` (copies of Appendix B.1 and B.3), and
     `gap-half-degree.yaml` (a ring with a 0.5° hole at 179.5–180.0).
  6. `packages/tests/far_field.rs`: one table test over the fixtures asserting each code and its detail; a test
     of `uncovered` (a ring with a 10° hole at 175–185 → `(175.0, 185.0)`).
  7. If PR #200 is merged in the base: §5.8 (derive `JsonSchema`, `far_field_schema()`, the entry, regenerate
     with `mineworld reference --out docs/reference`, then `--check`). If not: N/A, recorded; §14.2 applies.
  **Not in this commit:** wiring into `mineworld packs validate` (C3, together with the content, so the shipped
  packs never fail validation at any commit) and into world resolution (C3b).
- [ ] Implementation: steps 1–7.
- [ ] Validation: `cargo test -p mineworld-packages`; clippy; fmt. Mutation: change `uncovered`'s step from 0.1°
  to 1.0° → `gap-half-degree.yaml` validates → its row turns red → revert.
- [ ] Review: no `HashMap`/`Value` in the types; every refusal names the file and starts with its code.
- **Commit message:** `feat(packages): a Presentation Pack's far field, typed, with named refusals (RL-e0)`.

### C3 — Default content: the shipped far fields · Implementer: Haiku (values in Appendix B)

- **Goal.** Every shipped pack has the far fields the shipped worlds need. **Files:**
  `presentation/mineworld-default/3D/far_field/{social-cafe,lakeside}.yaml`,
  `presentation/mineworld-default/2D/far_field/{market-town,social-cafe,lakeside,bodies-yard}.yaml`,
  `presentation/mineworld-default/2D/renderer/godot.yaml` (palette key `water`), `tools/cli/src/packs.rs`,
  `tools/cli/tests/packs.rs`, `packages/tests/far_field.rs`.
- Steps:
  1. Write each file from Appendix B in YAML's JSON subset (JSON objects and arrays; no comments — DEP-16).
  2. Add `"water": "#5f8f9a"` to `godot.yaml`'s palette.
  3. `packs.rs` `validate` (presentation branch, A-16): after the style manifest, `far_field::check_pack(dir,
     …)` for each dimension; a pack with no `far_field/` directory is `FarFieldMissing { pack, scope: "no
     far_field/" }`.
  4. `tools/cli/tests/packs.rs`: `packs validate` on a planted pack without `far_field/` → exit non-zero, stderr
     starts with `FAR_FIELD_MISSING`.
  5. `packages/tests/far_field.rs`: `two_d_and_three_d_water_agree` over the shipped packs (AC-FF-4).
- [ ] Implementation: steps 1–5.
- [ ] Validation: `cargo build -p mineworld-cli`; `target/debug/mineworld packs validate presentation/mineworld-default/2D`;
  `… /3D`; `cargo test -p mineworld-packages`; `cargo test -p mineworld-cli --test packs`. Mutation X-2a (§6.7)
  → `FAR_FIELD_GAP` named → revert.
- [ ] Review: every `source` is a real citation or says "approximate … replaced by RL-e" / "design content";
  bearings are true (not scene) bearings.
- **Commit message:** `content(presentation): far fields for the shipped worlds (RL-e0)`.

### C3b — `validate` refuses a world whose Presentation Pack lacks its far field · Implementer: Haiku

- **Goal.** The authoring-time refusal for worlds (§5.1.4 item 1). **Files:** `packages/src/resolve.rs`,
  `packages/tests/resolve.rs`.
- Steps:
  1. `resolve.rs` `resolve` (A-16): for each resolved requirement whose type is `presentation-pack` and whose
     source is a directory, call `far_field::require_for_world(dir, pack_id, world_key)`.
  2. `packages/tests/resolve.rs`: a scratch world requiring a scratch Presentation Pack whose `far_field/` lacks
     the world → `FarFieldMissing { pack, world }`; with the file → resolves.
- [ ] Implementation: steps 1–2.
- [ ] Validation: `cargo test -p mineworld-packages`; `cargo build -p mineworld-cli`; `target/debug/mineworld
  validate worlds/lakeside --packs entities --packs presentation/mineworld-default` exit 0;
  `cargo test -p mineworld-cli --test milestone_e`. Mutation X-5 (lakeside variant) → named → revert.
- [ ] Review: only `presentation-pack` requirements are checked; bundled code packs untouched.
- **Commit message:** `feat(packages): a world's Presentation Packs must carry its far field (RL-e0)`.

### C4 — The shared loader in both clients · Implementer: Haiku

- **Goal.** One GDScript reader and validator, symlinked into both clients. **Files:**
  `clients/shared/environment/far_field.gd` (+ `.uid` generated by Godot), `clients/2d/mineworld_environment`
  (symlink → `../shared/environment`), `clients/3d-spike/mineworld_environment` (symlink), `mineworld-2d`,
  `mineworld-slice` (module loops), `.github/workflows/nightly.yml` (Windows directory check),
  `clients/shared/README.md`, `clients/shared/checks/far_field_check.gd` (new, a `--script` check that runs the
  GDScript validator over `packages/tests/fixtures/far_field/` and prints `<file> <code>` per line).
- Steps:
  1. Read `clients/shared/settings/store.gd` for the module style and `clients/shared/checks/store_check.gd` for
     the check style.
  2. Write `far_field.gd` per §5.3 with `CODES` equal to §5.1.3's list in order.
  3. Create both symlinks with `ln -s ../shared/environment clients/2d/mineworld_environment` (and 3d-spike);
     add `mineworld_environment` to both launchers' `for module in …` loops and to the nightly Windows check.
  4. Write `far_field_check.gd`.
- [ ] Implementation: steps 1–4.
- [ ] Validation: run `far_field_check.gd` the way `tools/cli/tests/client_settings.rs`'s `check()` runs the
  existing `clients/shared/checks/*_check.gd` (read that helper first and copy its project path and arguments);
  record the exact command. Every fixture's printed code equals its file stem; the two `valid-*` files print
  `ok`. (The Rust test that compares both validators is C8's `the_two_validators_agree`.)
- [ ] Review: the module writes nothing, opens no socket, references no intent.
- **Commit message:** `clients: a shared far-field reader with the packs' refusals (RL-e0)`.

### C5 — 3D: the far-field view · Implementer: **Opus** (geometry, anchoring, haze, water; judgement on look)

- **Goal.** The slice closes every direction (§5.4). **Files:** `clients/3d-spike/scripts/slice/far_field_view.gd`
  (new), `clients/3d-spike/scripts/slice/street.gd`, `clients/3d-spike/scripts/slice/slice_world.gd`,
  `clients/3d-spike/scripts/slice/slice_main.gd` (load the far field; refuse; exit 3 in scripted modes),
  `clients/3d-spike/scripts/slice/batch.gd`, `budget.gd`, `occluders.gd` (skip `farfield.*`),
  `clients/3d-spike/shaders/water.gdshader` (uniforms for colours and the Fresnel cap; the promenade's call keeps
  its defaults).
- Steps:
  1. `git show 58bc0c4:clients/3d-spike/scripts/slice/skyline.gd` and read it in full; keep its angle,
     haze and horizon-sampling functions, replace its summit table by the far field's profile points.
  2. Build the four parts of §5.4. Ribbon triangles in Godot's front-face order seen from inside the ring;
     `st.generate_normals()`; no winding assertion; delete `street.gd` `backdrop()`/`_ridge()` and the apron.
  3. `slice_main.gd`: before `SliceWorld.build`, `MineWorldFarField.load_for(repo pack dir, "social-cafe",
     "3d")`; on a code, print `[mineworld-slice] FAR_FIELD_<CODE>: …`, show it, and in scripted modes `quit(3)`.
  4. Skip `farfield.*` in batch, budget and occluders.
- [ ] Implementation: steps 1–4.
- [ ] Validation: `./mineworld-slice --shots` (windowed, macOS) completes; `./mineworld-slice --drive` passes;
  `./mineworld-slice --measure` passes; `./mineworld-slice --perf` **once** for counts (§8) on base and head;
  look at `ff_6` and the four aerials. INCONCLUSIVE timings recorded as such.
- [ ] Review: no far-field node collides or is targetable (no collider, not under the targeting root); fog is
  not double-counted on ribbons; the station footprint is empty of massing.
- **Commit message:** `3d: a 360° far field with the bay, from the Presentation Pack (RL-e0)`.

### C6 — 3D: the rendered check · Implementer: Haiku, reviewed by Opus (the algorithm is fixed in §6)

- **Goal.** §6.1–6.3, 6.6 in the slice. **Files:** `clients/3d-spike/scripts/slice/slice_far_field_check.gd`
  (new), `clients/3d-spike/scripts/slice/slice_probe.gd` (`MODES` += `"farfield"`, `"farfield-shots"`; two
  dispatch lines), `mineworld-slice` (`--farfield`, `--farfield-shots` in the flag list; both windowed),
  `clients/3d-spike/README.md` (two command lines).
- Steps:
  1. Read `slice_perf.gd` (the split-out pattern) and `slice_probe.gd` `_run`, `_settle`, `_verdict`.
  2. Implement §6 exactly: viewpoints, 1 440 × 240 `SubViewport`, S/H/E, rows, columns, the two criteria with
     N_below = 0.90 and N_above = 0.25, the 2 % exemptions, the verdict lines.
  3. **Hosted-runner viability first (R-FF-1):** push the branch to `scratch/rl-e0-c6` and dispatch the nightly's
     `clients` group on it (`gh workflow run nightly.yml --ref scratch/rl-e0-c6` with the group input the
     workflow defines — read `nightly.yml`'s `workflow_dispatch` inputs first). If any OS cannot render a
     `SubViewport` frame within the job's limit, **stop: material stop** with the logs.
  4. `--farfield-shots`: the §7 poses, JPEG q85 at 960 × 540, into `shots/social-cafe/farfield/`.
- [ ] Implementation: steps 1, 2, 4.
- [ ] Validation: `./mineworld-slice --farfield` prints `all farfield checks pass` on macOS; step 3's run.
  Mutations X-1, X-2b, X-4 → each named FAIL → revert.
- [ ] Review: thresholds are constants with this section cited; nothing tunes N after a run; E is rendered with
  the slice root hidden, not by a second scene.
- **Commit message:** `3d: a rendered far-field check, shown against hidden (RL-e0)`.

### C7 — 2D: the surround and its check · Implementer: Haiku

- **Goal.** §5.5 and §6.4. **Files:** `clients/2d/scripts/scene/ground.gd`, `clients/2d/scripts/scene/places.gd`,
  `clients/2d/scripts/app.gd` (`--world-key`, load and refuse), `clients/2d/scripts/harness/drive.gd` (scenario
  `farfield`), `clients/2d/scripts/harness/far_field_2d_check.gd` (new), `mineworld-2d` (`--world-key` passed
  from `--world`; required with `--server`), `tools/cli/tests/godot2d/mod.rs`, `tools/cli/tests/client_2d_interact.rs`,
  `tools/cli/tests/client_settings.rs` (pass `--world-key`).
- Steps:
  1. Read `ground.gd`, `places.gd`, `harness/capture.gd`, `harness/drive.gd`'s scenario dispatch.
  2. Load the far field after the pack (`app.gd:70-75`); refuse per §5.1.4; plain mode prints the exemption.
  3. Draw the surround (§5.5).
  4. Implement `--drive=farfield` per §6.4 (two sentinels, 8 extremes, 4 aspects) and, with `--capture`, the five
     shots of §7.
  5. Pass `--world-key` in the launcher and the three test helpers.
- [ ] Implementation: steps 1–5.
- [ ] Validation: `./mineworld-2d --drive=farfield --capture` for each of the four worlds (`--world worlds/<w>`):
  `all farfield2d checks pass`; `cargo test -p mineworld-cli --test client_2d -- --ignored --test-threads=1`
  (one file, it starts Godot). Mutation X-3 → named FAIL → revert. If the 21.3 : 1 aspect fails with the
  shipped 400 m, apply §5.5's bounded rule and record.
- [ ] Review: the camera still follows the player; the haze never covers the land rectangle.
- **Commit message:** `2d: a surround past the reachable area; the clear colour is never drawn (RL-e0)`.

### C8 — CI: the Rust harness · Implementer: Haiku

- **Goal.** §6.5. **Files:** `tools/cli/tests/client_far_field.rs` (new), `scripts/ci_godot.py` (`GODOT_TESTS`),
  per Q-FF-5's ruling `.github/workflows/ci.yml` and `scripts/ci_changes.py`.
- Steps:
  1. Read `tools/cli/tests/client_settings.rs` (`run_slice`, `check`, `verdicts`) and `godot2d/mod.rs`.
  2. Three `#[ignore = "needs Godot 4.7: cargo test -p mineworld-cli --test client_far_field -- --ignored"]`
     tests: `three_d_far_field_closes_every_open_sector`, `two_d_clear_colour_is_never_visible` (loops the four
     worlds), `the_two_validators_agree` (runs C4's check, compares with `mineworld_packages`' codes).
  3. `GODOT_TESTS.append("client_far_field")`; `python3 scripts/ci_godot.py coverage` stays green.
  4. Q-FF-5 (ruled yes, 2026-10-15): a `farfield` job in `ci.yml` on ubuntu-24.04 using `./.github/actions/godot`,
     run only when `ci_changes.py` classifies a change under `clients/**`, `presentation/**`, `mineworld-2d`,
     `mineworld-slice` (read `ci_changes.py`'s classification and its self-test first; add the class and a
     self-test row for it). It runs `cargo test -p mineworld-cli --test client_far_field -- --ignored
     --test-threads=1` and is not a required check until the operator says so.
- [ ] Implementation: steps 1–4.
- [ ] Validation: `cargo test -p mineworld-cli --test client_far_field -- --ignored --test-threads=1` on macOS;
  `python3 scripts/ci_godot.py coverage`; one dispatched nightly `clients` run on the branch head, all three
  OSes: AC-FF-5/6 PASS or the evidence of a hosted-runner limit (then material stop, R-FF-1).
- [ ] Review: verdicts are parsed from lines; a run with no summary line is INCONCLUSIVE, not PASS.
- **Commit message:** `ci: the far-field checks run nightly on Linux, macOS and Windows (RL-e0)`.

### C9 — Shots, the stale README line, the promenade · Implementer: Haiku

- **Goal.** §7 committed; A-21; Q-FF-3. **Files:** `clients/3d-spike/shots/social-cafe/farfield/*.jpg`,
  `clients/2d/shots/<world>/farfield/*.jpg`, both `.gitignore`s, `clients/3d-spike/README.md`, and per Q-FF-3
  `mineworld-3d` (+ `clients/3d-spike/README.md`'s promenade lines).
- Steps:
  1. Add the `.gitignore` exceptions of §7.
  2. `./mineworld-slice --farfield-shots`; `./mineworld-2d --world worlds/<w> --drive=farfield --capture
     --shots=clients/2d/shots/<w>/farfield` for the four worlds.
  3. README line 7: replace `the far skyline: ./mineworld-slice --skyline-check` by `the far field:
     ./mineworld-slice --farfield (the rendered check) and --farfield-shots (the regression views)`.
  4. Q-FF-3 (ruled (a), 2026-10-15): `mineworld-3d` without a harness flag prints "the promenade spike has no
     far field (operator ruling 2026-10-15); walk the world in ./mineworld-slice" and exits 2; `--drive`,
     `--shots`, `--portrait` keep working.
- [ ] Implementation: steps 1–4.
- [ ] Validation: `git status --short` lists exactly 27 images plus the edited files; `git grep -n
  "skyline-check" clients/` is empty; `./mineworld-3d --drive` passes.
- [ ] Review: every image is ≤ 300 KB; no image from a mutated run.
- **Commit message:** `3d, 2d: far-field regression shots; README; the promenade spike (RL-e0)`.

### C10 — Review package and Milestone F items · Implementer: Haiku

- **Goal.** §8's counts recorded; F's checklist carries the far field. **Files:** this file (§18),
  `docs/MVP_STATUS.md` ("Carried to milestone F": the items of §14.3), `docs/HUMAN_REVIEW_QUEUE.md` (an RL-e0
  entry with the launch commands and the shots' paths).
- [ ] Implementation: the three files.
- [ ] Validation: both doc checks.
- [ ] Review: the F items are runnable as written.
- **Commit message:** `docs: RL-e0 review package; far-field items carried to Milestone F`.
- **Then:** `git push -u origin mvp0/pr-rl-e0`; open the PR with `gh pr create` (body: what changed, evidence,
  mutations, counts, the operator's items); wait for CI on the exact head; READY FOR OPERATOR REVIEW — DO NOT
  MERGE.

---

## 13. Execution contract (template; lines marked *primary* are the primary session's at freeze)

```text
PROJECT / PR:            MineWorld mvp0 visual track (S22), RL-e0 — the far field is mandatory
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp1/pr-rl-e0-far-field-mandatory.md
RELATED / BINDING DOCS:  step-22-realism.md §§6.6, 7, 8, 9; pr-rl-b-3d-budget.md §§0.2, 3, 5; pr-rl-b2 §§0, 10;
                         docs/MODULE_SPEC.md §6; docs/VISUAL_SLICE.md; docs/ART_DIRECTION.md §§12, 21;
                         docs/DECISIONS.md DEP-8, DEP-16, DEP-45, ARC-17, ARC-23; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-rl-e0, its own, held by one session
BRANCH:                  mvp0/pr-rl-e0 from origin/main                                    (primary)
START CONDITION:         on the primary session's dispatch after freeze                    (primary)
IMPLEMENTATION BASE:     origin/main at C0; C0 records the commit
APPROVED SCOPE:          §2.1 items 1–10                                                   (primary)
FROZEN INVARIANTS:       I-FF-1 … I-FF-8; §6's viewpoints, renders, bands and thresholds; §8's caps
SEQUENCE:                C0, C1, C2, C3, C3b, C4, C5, C6, C7, C8, C9, C10 (§12)
IMPLEMENTERS:            Haiku for C0–C4, C6–C10 (C6 reviewed by Opus); Opus for C5          (primary)
ALLOWED COMMANDS:        cargo -p <crate> *; git; gh (never merge); python3 scripts/*; godot --headless
                         --import / --script; ./mineworld-slice *; ./mineworld-2d *; ./mineworld-3d *;
                         target/*/mineworld *; ln -s; mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace
                         locally; tuning N_below/N_above or the viewpoints after a run; edits outside scope
MATERIAL STOPS:          a hosted runner cannot render the check (R-FF-1); a count cap of §8 exceeded; an
                         operator-measured p95 increment > +1.0 ms; V-1 fails; a needed change to
                         camera_rig.gd (FAR), the protocol, a kernel/contract crate, or a frozen RL-b2 file
                         beyond §14.1's lines; the 2D surround needs camera clamping after all
PLATFORMS:               Linux, macOS, Windows (nightly clients layer; the PR's CI)
VALIDATION BUDGET:       local: per-crate tests, the probes on macOS, one --perf counts run per side;
                         CI: two dispatched nightly clients runs (C6 step 3, C8) + the PR's runs   (primary)
LIVE DOCUMENTATION:      this document (§18)
HANDOFF:                 .structured-coding/plans/mvp1/handoff-rl-e0.md
ENDPOINT AUTHORITY:      implementation + local validation; semantic commits; branch push; PR creation /
                         update; CI repair to review readiness                              (primary)
                         merge: never by the implementation session; only with the operator's explicit
                         authorization after the primary session's review
POST-MERGE SYNC OWNER:   primary: step-22, overall.md (mvp0 Milestone F; mvp1 S22); implementer: §18
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 14. File overlaps, merge order, and Milestone F

### 14.1 Overlaps

| File | RL-b2 (#212) | 16c (frozen, after 12d) | RL-e0 | Resolution |
| --- | --- | --- | --- | --- |
| `slice_world.gd` | `SliceBatch.merge(g, SliceTrial.cell_size())` | `build(parent, connected)` | the `backdrop` line → `SliceFarFieldView.build` | adjacent lines; mechanical |
| `slice_main.gd` | trial wiring | link changes | far-field load before `SliceWorld.build` | different functions; mechanical |
| `slice_probe.gd` | — | modes | `MODES` += 2, two dispatch lines | mechanical |
| `batch.gd` | cell parameter | — | skip `farfield.*` | different lines |
| `mineworld-slice` | `--trial`, `--trials` | — | `--farfield`, `--farfield-shots` | the flag list; mechanical |
| `clients/3d-spike/README.md` | perf lines | — | line 7 | adjacent |
| `street.gd` | — | doorway points (read) | `backdrop()`, `_ridge()`, apron removed | 16c reads `street.gd` constants only |

### 14.2 Order

No PR waits for another. Recommended: **RL-e0 → RL-b2's final operator session → 16c**. Whichever of
RL-e0 / RL-b2 / 16c merges later rebases; overlaps are mechanical. For #200: whichever of #200 / RL-e0 merges
second adds the reference entry (§5.8) as its own commit, and `mineworld reference --check` must pass on its
head.

### 14.3 Milestone F's checklist gains (proposed for `overall.md` and `docs/MVP_STATUS.md`)

Prerequisite row: `RL-e0, the far field | S22 | merged before the checklist is handed over`. Items:

```text
FF-1  ./mineworld-slice : from the spawn, turn a full circle: hills, townscape, the bay — no edge, no void
FF-2  walk to the west parapet (the ff_6 view): the bay below, Point Loma across, nothing suggesting a path
FF-3  walk to the east end: the mesas and the eastern mountains; jump: the horizon stays closed
FF-4  ./mineworld-2d : walk to each corner of the town; resize the window very wide and very tall:
      the background colour never shows
FF-5  (perf) ./mineworld-slice --perf, three runs, display awake (RL-b R-6): no view's p95 grows by
      more than 1.0 ms against the run before RL-e0 (the numbers the ledger records)
FF-6  Windows and Linux machines when available: FF-1 and FF-4
```

---

## 15. Edits to parent documents (owner: the primary session, after freeze or merge)

- **step-22** §7.3: replace "No far-field terrain in the isometric view (a hillshade vignette at most, presentation
  pack's choice)" by "A declared `surround` (RL-e0), drawn past the reachable area; the clear colour is never
  visible". §7.2 Far field: "RL-e0 makes it required and 360°; RL-e replaces its hand-authored profile points by
  3DEP-baked ones within the same schema". §9.1: insert `RL-e0 far field mandatory (before F)` before `RL-e`.
  §9.2: an RL-e0 row pointing here; RL-e's adversarial criterion reads the far field's profile points. §8: add
  "INV-RL-12 (= I-FF-1, I-FF-2)".
- **`../mvp0/overall.md`** Milestone F table and §14.3's items; the RL-b M-6 note's "it returns with S22's sea
  surface and real terrain" → "returned with RL-e0 (bay) and RL-e (terrain)".
- **step-23** (release): the exported 3D and 2D clients must carry `presentation/mineworld-default/*/far_field/`
  (R-a's pack list).

---

## 16. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-FF-1 | Hosted software renderers (lavapipe, WARP, Apple Paravirtual) cannot render 120 frames of the full slice within the job's 75 min, or hang as the connected slice did (A-12) | offline slice, `--gi=none`, small offscreen viewports; C6 step 3 tests viability on all three before the rest is built on it; if one OS cannot, material stop with options (lower resolution; a far-field-only scene — which would no longer test near occlusion) |
| R-FF-2 | Real angles make the far field thin at eye height (sub-degree), and the operator rejected a low skyline once (R-5) | R-5's stated reason was the missing sea, now present; mesas at ≈ 2° N–E are real; the thresholds were ruled by the operator (Q-FF-1); the look is judged at F (FF-1 … FF-3) |
| R-FF-3 | Grazing Fresnel renders distant water sky-coloured, so FF-BELOW fails over the bay | SD-FF-5's cap at 0.6 (physically motivated) |
| R-FF-4 | Koschmieder haze at 72 km makes the far layer differ from the sky by < 8/255 | FF-ABOVE does not depend on the far layer: near and mid layers exist in every land sector (Appendix B.1) |
| R-FF-5 | Hand-estimated mesa and lowland heights are wrong | marked approximate in every `source`; RL-e replaces them from 3DEP; the summits are re-read values |
| R-FF-6 | The two validators drift | AC-FF-3's shared corpus |
| R-FF-7 | Requiring `--world-key` breaks an unlisted launch path | C7 greps every Godot launch of `clients/2d`; a missing key refuses loudly (never silently draws the wrong world) |
| R-FF-8 | The camera-anchored silhouettes look painted when the player jumps or the camera booms | per-layer y offset from camera height (§5.4); VP-high is in the check |
| R-FF-9 | Committed images grow the repository | 27 JPEGs ≈ 4 MB; re-shot only when the far field changes |

---

## 17. Operator questions (each with a recommendation; all ruled 2026-10-15, §17.1)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-FF-1 [OPERATOR]** | **N, the rows and the viewpoints.** Accept N_below = **90 %** over `[e_h − 1.0°, e_h − 0.1°]` and N_above = **25 %** over `[e_h, e_h + 0.25°]` (land sectors only), the seven viewpoints of §6.1, and 8/255 per pixel? | **Accept.** Two criteria are needed because an open sea and a deleted south band look alike below the horizon (F-FF-3). 90 % leaves room for anti-aliased edges; 25 % passes the lowest real San Diego sector (south hills ≈ 0.4–0.6°) and fails any deleted or culled band (0 %) |
| **Q-FF-2 [OPERATOR]** | **What the bay looks like before RL-f.** (a) our `water.gdshader` with San Diego Bay colours, noise normals, Fresnel capped at 0.6, no reflections of the town; (b) a flat unshaded colour; (c) adopt an ocean addon now | **(a).** Reuses reviewed code, reads as water at a distance, costs ≤ +0.5 ms; RL-f decides real water once |
| **Q-FF-3 [OPERATOR]** | **Does the promenade spike (`./mineworld-3d`) stay launchable for walking?** (a) walking refused with a pointer to the slice, harness modes kept; (b) give it a far field (≈ one more commit: its north lake plus a fallback ring); (c) delete the scene | **(a).** It is the movement/camera artefact (QRL-1), not a world; (a) satisfies "no walkable scene without a far field" in one launcher change and keeps its evidence |
| **Q-FF-4** | Carry the world's pack id in `WorldSummary` so a joined client knows its far field without `--world-key`? | **Later, with S21** (regions will be disclosed then). RL-e0 stays presentation-only (I-FF-5) |
| **Q-FF-5 [OPERATOR]** | Run the rendered check **per PR** on Linux (≈ 8–12 min, path-filtered) in addition to nightly on all three? | **Yes.** The defect class surfaces at render time; a night's delay lets it merge |
| **Q-FF-6 [OPERATOR]** | **The town's distant massing:** low-rise 6–14 m in the slice's architecture, or San Diego's real downtown towers (to ≈ 150 m)? | **Low-rise.** QRL-2/3: the lakeside references keep authority over architecture; towers would contradict the town the slice depicts |
| **Q-FF-7 [OPERATOR]** | **No fallback far field**, and the 2D plain diagnostic mode exempt — accept? | **Accept both.** A fallback makes the refusal unreachable and draws the wrong place; plain mode is not a presentation |
| **Q-FF-8 [OPERATOR]** | **The landmark slot:** which landmark, and is the station itself the Santa Fe Depot–modelled building? | RL-e0 reserves `station` at the real depot's bearing (277°, 790 m) and leaves `landmark` unplaced; the post-F PR places it |
| **Q-FF-9** | May the slice (which depicts `social-cafe`) claim Market Town's San Diego location for its far field? | **Yes.** `social-cafe` declares no location; QRL-2/3 put the default world on San Diego Bay |
| **Q-FF-10** | Invisible walls: the slice keeps its natural terminations (parapet, bank); RL-e0 adds no invisible wall. Accept? | **Accept.** The ruling permits invisible walls; it does not require one where the slice already terminates naturally |

### 17.1 Rulings

Answered 2026-10-15; relayed by the coordinator. **Binding.** The operator answered in the primary session:
"全部按推荐" (everything as recommended).

| Question | Ruled by | Ruling | Applied in |
| --- | --- | --- | --- |
| Q-FF-1 | operator | **N_below = 90 %** over `[e_h − 1.0°, e_h − 0.1°]`, **N_above = 25 %** over `[e_h, e_h + 0.25°]` (land sectors only); the seven viewpoints of §6.1; 8/255 per pixel | §6 (now frozen content) |
| Q-FF-2 | operator | The bay uses `water.gdshader` with San Diego Bay colours, **Fresnel ≤ 0.6**, **no reflections of the town** | §5.4, SD-FF-5 |
| Q-FF-3 | operator | **(a)** The promenade spike stops being walkable; `--drive`, `--shots`, `--portrait` stay | §2.1 item 9, C9 step 4, AC-FF-11 |
| Q-FF-4 | primary | `WorldSummary` pack id **later, with S21**; RL-e0 uses `--world-key` | §2.2, SD-FF-7 |
| Q-FF-5 | operator | **Yes:** a per-PR Linux far-field job, filtered to client and presentation changes (not required until the operator says so) | §2.1 item 11, §6.5, C8 step 4 |
| Q-FF-6 | operator | **Low-rise** distant massing (6–14 m) | §5.6, Appendix B.1 |
| Q-FF-7 | operator | **No fallback** far field; the 2D `--presentation=none` diagnostic mode is **exempt** and says so | §5.1.4, SD-FF-8, I-FF-1 |
| Q-FF-8 | operator | RL-e0 **reserves `station`** at the Santa Fe Depot (277°, 790 m) and leaves **`landmark` unplaced**. The station-and-landmark PR comes **after F**; **the station is the landmark** | §5.1.6, Appendix B.1 |
| Q-FF-9 | primary | **Yes:** the slice (`social-cafe`) uses Market Town's San Diego location | §5.6, Appendix B.1 |
| Q-FF-10 | primary | **Accept:** the slice keeps its natural terminations; RL-e0 adds no invisible wall | Appendix B.1 (`boundary: natural`) |
| Ids | primary | ARC-RLe0-a/b/c → **ARC-102, ARC-103, ARC-104**; next free ARC-105; DEP-59 is taken by the F-tick-1 fix, so a DEP here would be DEP-60 (none is needed) | header, C1, Appendix A.5 |

The coordinator's relay named this record "§14.1"; in this file §14.1 is the file-overlap table, so the rulings
live here, in §17.1, beside the questions they answer.

---

## 18. Ledger (live during implementation)

### Commit ledger

Implementation base: **origin/main @ 2d07c6c0** (Merge #215, this design's freeze), recorded at C0 on
2026-10-15. Branch `mvp1/pr-rl-e0` (the dispatch's name; §13 said `mvp0/pr-rl-e0` — the dispatch's explicit
setup command replaces it). Worktree `/Users/yuema137/mineworld-worktrees/impl-rl-e0`, one session.

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | 80afda22 | [x] base recorded; header `DESIGN FROZEN` and §13 confirmed; handoff created with §17.1 verbatim | [x] doc checks (below) | [x] no authority line widened: merge stays operator-only; the branch-name line is the dispatch's |
| C1 | 2b554ed5 | [x] A.1 → `MODULE_SPEC.md` §6.1 + new §6.1.1 (schema, codes, refusal points, exemption verbatim); A.2 → `ART_DIRECTION.md` §12 layout + exception, §21 Environment; A.3 → `VISUAL_SLICE.md` §1 note, §4 terminations, far-field row, out-of-scope line, §12 view 11; A.4 → `clients/2d/PRESENTATION.md` new §8; A.5 → `DECISIONS.md` ARC-102/103/104 appended, DEP-45 dated note | [x] `check_doc_headings.py` 239 sections, none duplicated PASS; `check_decision_ids.py` 128 ids distinct PASS; `grep far_field docs/MODULE_SPEC.md` shows the layout line, the exception and §6.1.1 (VISUAL_SLICE's A.3 text says "far field" in words, no `far_field` token, by design) | [x] terms exact (`Presentation Pack`, `World Pack`, `ActionIntent`, `System`); the rule lives in MODULE_SPEC §6.1.1, no README carries it; VISUAL_SLICE additions name no engine, node or shader |
| C2 | 2b6ff714 | [x] `packages/src/far_field.rs` (codes, `Dimension`, `read`, `check_pack`, `require_for_world`, `uncovered`, `missing_message`), `far_field_schema.rs` (every type of §5.1.2, `deny_unknown_fields`, newtypes `Key`/`AssetName`/`Colour`, `Slots`/`Slot` with their own deserializer so slot faults are `FAR_FIELD_SLOT`), `far_field_check.rs` (the ordered rules); `error.rs` `FarField`/`FarFieldMissing` (Display starts with the code); `manifest.rs` `check_style_manifest → Vec<Dimension>` (unknown dimension → `Malformed`); `lib.rs` exports; corpus `packages/tests/fixtures/far_field/` (16 files); `packages/tests/far_field.rs` (5 tests). Step 7 N/A: #200 not merged (§14.2) | [x] `cargo test -p mineworld-packages` 9 binaries, 28 tests PASS (load ≈ 117, wall time dominated by other sessions' builds); `cargo clippy -p mineworld-packages --all-targets -- -D warnings` clean PASS; `cargo fmt --all --check` PASS. Mutation (step 0.1° → 1.0°): `gap-half-degree` validated, both corpus tests FAILED by that name → reverted → 5/5 PASS | [x] no `HashMap`/`Value`/map in the public types (the slot deserializer reads keys as strings and builds the typed `Slots`); every refusal is `PackageError::FarField { path, code, detail }` whose Display is `<CODE>: <path>: <detail>`, or `FarFieldMissing` starting `FAR_FIELD_MISSING` |
| C3 | 2edc512e | [x] `presentation/mineworld-default/3D/far_field/{social-cafe,lakeside}.yaml` (B.1, B.2), `2D/far_field/{market-town,social-cafe,lakeside,bodies-yard}.yaml` (B.3), JSON subset; `2D/renderer/godot.yaml` palette `water: #5f8f9a`; `tools/cli/src/packs.rs` `validate` (presentation branch): `check_style_manifest` → dimensions, no `far_field/` → `FarFieldMissing { pack: <id>, scope: "no far_field/" }`, then `far_field::check_pack`; `tools/cli/tests/packs.rs` `a_presentation_pack_without_a_far_field_is_refused_by_name` (missing → valid with a file → `FAR_FIELD_GAP` with extent 0); `packages/tests/far_field.rs` `two_d_and_three_d_water_agree` (AC-FF-4) | [x] `cargo build -p mineworld-cli` PASS; `mineworld packs validate presentation/mineworld-default/2D` and `…/3D`: identity printed, "is a valid presentation-pack", rc 0 PASS; `cargo test -p mineworld-packages --test far_field` 6/6 PASS; `cargo test -p mineworld-cli --test packs` 7/7 PASS (80 s, load ≈ 50–70; the tree also held C3b's uncommitted `resolve.rs` edit, which `packs validate` does not reach). Mutation X-2a: see the table and F-FF-1 | [x] every `source` is a citation or says "approximate; replaced by RL-e (USGS 3DEP)" / "design content; lakeside declares no real location" (Appendix B; the "approx." strings carry the full suffix B's preamble requires); bearings are true bearings from B, unconverted |
| C3b | 152eba80 | [x] `packages/src/resolve.rs` `resolve`: for each resolved requirement of type `presentation-pack` with `Source::Directory(dir)`, `far_field::require_for_world(dir, id, world.id)`; `packages/tests/resolve.rs` `a_world_whose_presentation_pack_lacks_its_far_field_is_refused` (`FarFieldMissing { pack: style-a, scope: elsewhere }`; an Entity Pack requirement asks for none) with fixture roots (D-FF-5); test helpers of `tools/cli/tests/requirements.rs`, `worldpack/tests/requirements.rs` give their scratch pack a far field; `the_default_presentation_pack_can_be_required` → `…_needs_a_far_field_for_the_world` (D-FF-6) | [x] `cargo test -p mineworld-packages` all binaries PASS (resolve 6/6); `cargo build -p mineworld-cli`; `mineworld validate worlds/lakeside --packs entities --packs presentation/mineworld-default` → "worlds/lakeside is a valid World Pack.", rc 0 PASS; `cargo test -p mineworld-cli --test milestone_e` 7/7 PASS (151 s); `--test requirements` 6/6 PASS; `cargo test -p mineworld-worldpack --test requirements` 4/4 PASS; clippy `-p mineworld-packages -p mineworld-cli --all-targets -D warnings` clean; fmt clean. Mutation X-5 (lakeside variant): see the table | [x] only `presentation-pack` requirements found in a directory are checked; code packs (`Source::Build`) and Entity Packs are untouched; the check is presence only, content stays `packs validate`'s |
| C4 | d8c7482b | [x] `clients/shared/environment/far_field.gd` (`MineWorldFarField`: `CODES` in §5.1.3's order = `FarFieldCode::ALL`; `load_for`, `load_file`, `refusal_line`, read-only accessors; the shape check mirrors serde's `deny_unknown_fields`, enums, newtype patterns and the slot deserializer; then the rules in D-FF-4's order); symlinks `clients/{2d,3d-spike}/mineworld_environment → ../shared/environment`; `mineworld_environment` in both launchers' module loops and the nightly Windows directory check; `clients/shared/checks/far_field_check.gd` (`FARFIELD <name> <code|ok>` per corpus file, a `missing` row, `--pack=<dir>:<dim>` for shipped packs); `clients/shared/README.md` | [x] `godot --headless --path clients/shared --import` rc 0; `godot --headless --path clients/shared --script $PWD/clients/shared/checks/far_field_check.gd -- --pack=$PWD/presentation/mineworld-default/2D:2d --pack=$PWD/presentation/mineworld-default/3D:3d` (the `client_settings.rs` `check()` form): 23 lines; every corpus file's code equals its stem's code, `valid-2d`/`valid-3d` and all six shipped files `ok`, `missing` → `FAR_FIELD_MISSING` PASS. First run hung (parse error: `_Shape.surround` missing; a SceneTree script that fails to compile never quits) → fixed. `python3 scripts/check_client_rules.py` PASS (0 findings) | [x] the module only reads (`FileAccess.get_file_as_string`, `file_exists`); no write, socket, request or intent; accessors return deep copies |
| C5 | 41742fe7 | [x] `scripts/slice/far_field_view.gd` (`SliceFarFieldView`: near-ground disc of `reach_m` (category `ground`, the apron's tint and collision); silhouettes per layer, camera-anchored, crest by curvature/refraction from the profile, Koschmieder haze to the panorama's horizon colour (RL-b C8's sampler), wound clockwise-from-inside, `generate_normals()`, `CULL_BACK`, no assertion; foreground annulus to 1 450 m (shore in a water sector), falling `drop_m` over `run_m`, blending from the near grass to the declared surface; the bay's surface and a camera-anchored sea ribbon stretched to the true dip; seeded low-rise massing, 8 chunk meshes, station footprint kept clear; `--slice-farfield-hidden` diagnostic for V-1); `street.gd` `backdrop()`/`_ridge()` and the 900 m apron removed; `slice_world.gd` builds the view in place of the backdrop; `slice_main.gd` reads `far_field/social-cafe.yaml` before anything is drawn, refuses by name (exit 3 in harness modes), takes the sky from it; `batch.gd`, `budget.gd`, `occluders.gd` skip `farfield.*`; `water.gdshader` `FAR_WATER` variant | [x] `./mineworld-slice --drive` all drive checks pass (headless; 9 011 massing boxes, 8 meshes, far-field build 431–525 ms) PASS; `--shots` completes (30 frames) PASS; `--measure`: 1 scale check out of range (occupant stature 1.802 m) — **pre-existing**, identical on base 2d07c6c0 (`/tmp` logs), not this PR's; V-1 PASS (table below); counts (§8, one `--perf` run each side at 1920×1080, voxel GI): within every cap (table below); timings INCONCLUSIVE (loaded host, R-LOAD-1); looked at `ff_3`, `ff_4`, `ff_6` | [x] no far-field node has a collider (only the near disc, category `ground`, keeps the apron's) or sits under a targeting root; ribbons and the sea ribbon `disable_fog` (their haze is Koschmieder's), world-anchored foreground/water/massing take scene fog once; the station's 60 m radius is excluded from massing placement (`_free`) |
| C6 | 5db6539c | [x] `scripts/slice/slice_far_field_check.gd` (`SliceFarFieldCheck`, the `SlicePerf` split-out pattern): §6.1's 7 viewpoints × 8 azimuths, one 1 440 × 240 `SubViewport` on the slice's `World3D`, `KEEP_WIDTH`, 50° horizontal, S / H / E (E with the slice root hidden, once per azimuth), the main viewport's 3D off meanwhile, the far field's camera-anchored parts following the check's camera (`anchor_camera`); §6.3's rows, columns, bands, FF-BELOW 0.90 / FF-ABOVE 0.25 / 2 % / 8 as constants; §6.6's lines; `--farfield-shots` writes §7's seven poses, JPEG q85 960 × 540; `slice_probe.gd` `MODES` += `farfield`, `farfield-shots`, two dispatch lines; `mineworld-slice` `--farfield` (adds `--gi=none` unless given), `--farfield-shots`, both `--always-on-top` (D-FF-10); `clients/3d-spike/README.md` line 7 now names both (C9 step 3 done here: its text *is* C6's two command lines) and the third symlink | [x] `./mineworld-slice --farfield` on macOS (Apple M-series, Metal): 120 renders in 18 s, **all farfield checks pass** (112 sector lines: 56 below, 56 above; covered where the street's own buildings fill the band) PASS. Mutations X-1, X-2b → named FAILs; X-4 survives (F-FF-2); X-5 and X-2a′ at the client → exit 3 with the code (table). Step 3 (R-FF-1 hosted-runner viability): moved to the one dispatched nightly `clients` run on the branch once C8's harness exists (D-FF-11) | [x] thresholds are `const`s with §6 cited in the header; no constant changed after any run; E hides `slice.world` in the same scene (no second scene) |
| C7 | 5e3e0b1d | [x] `scripts/app.gd`: `--world-key`, `_load_far_field` after the pack (refusal printed, shown, exit 3 under `--drive`/`--capture`, the world not entered; plain mode prints the exemption); `scene/places.gd`: `ground.surround = land.grow(extent_m)`, origin = the hub's centre, no camera clamping; `scene/ground.gd` `_surround()` beneath the land (base, water wedges clipped to the surround with seeded ripples, eight haze bands from `from_m` to `to_m`, full haze beyond); `harness/drive.gd` scenario `farfield` (steps out to the hub first, D-FF-13); `harness/far_field_2d_check.gd` (8 extremes × 4 aspects × 2 sentinels, D-FF-12's zoom; `--capture` writes §7's five views); `mineworld-2d` passes `--world-key=<basename of --world>`, runs `--drive=farfield` windowed and on top; `godot2d/mod.rs` passes `--world-key=market-town` unless a caller names one; `client_2d_interact.rs` the social-cafe run names `social-cafe`; `client_settings.rs` unchanged (its `social-cafe` runs are the 3D slice's, which always presents `social-cafe`) | [x] `./mineworld-2d --world worlds/<w> --drive=farfield` (lakeside with `MINEWORLD_PACKS=entities:presentation/mineworld-default --agent=none --seat alba`; bodies-yard `--agent=none --seat gus`, D-FF-14): market-town, social-cafe, lakeside, bodies-yard each **all farfield2d checks pass** (32 renders × 2 sentinels, 0 clear-colour px everywhere) PASS; `cargo test -p mineworld-cli --test client_2d -- --ignored --test-threads=1` 8/8 PASS (168 s). Mutation X-3 → 16 named FAILs (table); the shipped 400 m suffices at 21.3 : 1, so §5.5's bounded raise was not needed | [x] the player's camera is untouched (the check renders its own `Camera2D` in a `SubViewport`; `app.gd` `_process` still follows the body); the haze starts at `land.grow(from_m ≥ 0)` and the land is drawn after it, so it never covers the land rectangle |
| C8 | 8bc55052 | [x] `tools/cli/tests/client_far_field.rs`: `three_d_far_field_closes_every_open_sector` (offline slice, `--gi=none`, on top, 40 min limit; verdicts parsed, no summary = INCONCLUSIVE), `two_d_clear_colour_is_never_visible` (each world hosted; lakeside with `--packs`, seats per D-FF-14), `the_two_validators_agree` (C4's check vs `mineworld_packages` over the corpus, an absent key and every shipped file); `scripts/ci_godot.py` `GODOT_TESTS` += `client_far_field`; Q-FF-5: `scripts/ci_changes.py` `clients` output (`touches_clients`: `clients/`, `presentation/`, the two launchers, the harness; fails closed) with 8 self-test rows, `scripts/ci_layer.py` layer `farfield` (D-FF-15), `.github/workflows/ci.yml` job `farfield` (ubuntu-24.04, `.github/actions/godot` + `native` layer `farfield`, not required) | [x] `cargo test -p mineworld-cli --test client_far_field -- --ignored --test-threads=1` on macOS: 3/3 PASS (65 s); `python3 scripts/ci_godot.py coverage` PASS (31 ignored Godot tests, 29 on a hosted leg); `ci_godot.py --self-test`, `ci_changes.py --self-test` (28 cases) PASS; `check_ci_pins.py` (via `uv run`) PASS; `check_scratch.py scan` PASS; clippy `-p mineworld-cli -p mineworld-packages -p mineworld-worldpack --all-targets -D warnings` clean; fmt clean. Dispatched nightly `clients`: see E-FF-CI | [x] every verdict comes from printed lines (`judge`): a missing summary line panics as INCONCLUSIVE, a FAIL line or a CHECKS FAILED summary fails, and a minimum number of verdict lines is required |
| C9 | 78fc189e | [x] `.gitignore` exceptions (3D `shots/social-cafe/farfield/`; 2D `shots/<world>/farfield/` for the four worlds); 7 + 20 JPEGs from the final code (`./mineworld-slice --farfield-shots`; `./mineworld-2d --world worlds/<w> --drive=farfield --capture --shots=clients/2d/shots/<w>/farfield` with D-FF-14's options); README line 7 was done in C6; the promenade: `mineworld-3d` with no harness flag prints the ruling and exits 2, `--drive`/`--shots`/`--portrait` unchanged; `clients/3d-spike/README.md`'s promenade lines say so | [x] 27 image files staged (7 + 4 × 5), no other path but the edited ones; `git grep -n skyline-check clients/` empty; `./mineworld-3d` → rc 2 with the message; `./mineworld-3d --drive` runs its report to the end, rc 0; all four 2D captures printed `all farfield2d checks pass` | [x] every image ≤ 300 KB (2.3 MB total, largest ~153 KB); none from a mutated run (re-shot after every mutation was restored) |
| C10 | cb8cf440 | [x] this ledger (counts, evidence, parked operator checks below); `docs/MVP_STATUS.md` "Carried to milestone F": the RL-e0 block FF-1 … FF-6; `docs/HUMAN_REVIEW_QUEUE.md`: an RL-e0 entry (what changed, the shots' paths, launch commands, questions) | [x] `check_doc_headings.py`, `check_decision_ids.py` PASS (below) | [x] every FF item starts with `cd` to the repository and names the full command; FF-5 creates and removes its own base worktree at 2d07c6c0 |

### Parked operator checks (Milestone F; RL-e0 §14.3) — not run by this PR

| Item | Do | Check |
| --- | --- | --- |
| FF-1 | `cd /Users/yuema137/MineWorld && git switch main && git pull && ./mineworld-slice` — then, from the spawn, turn a full circle with the mouse | Hills, the town's roofs and the bay in every direction; no edge of the ground and no empty sky below the horizon anywhere |
| FF-2 | `cd /Users/yuema137/MineWorld && ./mineworld-slice` — walk west to the stone parapet at the end of the street and look out (the `ff_6` view) | The drop, the bay beyond the roofs and Point Loma across it; nothing beyond the parapet suggests a path, steps or a road |
| FF-3 | `cd /Users/yuema137/MineWorld && ./mineworld-slice` — walk east to the planted bank; jump (Space) and look around while in the air | The mesas and the eastern mountains; the horizon stays closed while jumping |
| FF-4 | `cd /Users/yuema137/MineWorld && ./mineworld-2d` — walk to each corner of the town; then Esc → settings → window size, very wide and very tall, or drag the window | The pale background colour never shows: ground and haze reach every edge of the window |
| FF-5 | `cd /Users/yuema137/MineWorld && git worktree add /tmp/mw-before-rl-e0 2d07c6c0 && cd /tmp/mw-before-rl-e0 && caffeinate -d ./mineworld-slice --perf`, then `cd /Users/yuema137/MineWorld && caffeinate -d ./mineworld-slice --perf` (display awake) | No view's median p95 grows by more than 1.0 ms (§8; a larger increment is a material stop). Then `cd /Users/yuema137/MineWorld && git worktree remove /tmp/mw-before-rl-e0` |
| FF-6 | Windows (Git Bash): `cd /c/Users/<you>/MineWorld && git config core.symlinks true && git pull && ./mineworld-slice`, then `./mineworld-2d`; Linux: `cd ~/MineWorld && git pull && ./mineworld-slice`, then `./mineworld-2d` | FF-1's and FF-4's checks on each machine |

### Evidence

```text
E-FF-design  2026-10-15  audit of origin/main @ 4c59cdb0 (§3); investigation frames (§0.3);
                         58bc0c4 read (A-8); PR #200 head read (A-23)
E-FF-C0      2026-10-15  base 2d07c6c0; PR #200 OPEN (not merged) → C2 step 7 N/A, §14.2 applies;
                         PR #212 OPEN draft. python3 scripts/check_doc_headings.py, check_decision_ids.py
E-FF-V1      2026-10-15  V-1 (§7, AC-FF-9). Base worktree 2d07c6c0 (/Users/yuema137/mineworld-worktrees/
                         impl-rl-e0-base). --shots twice on base (noise), once on head, once on head with
                         --slice-farfield-hidden; mask = head vs hidden, any channel > 8 (far field 0.00 –
                         1.31 % of a frame); frame_diff base1 head 8 --mask: every view's unmasked share
                         0.000 – 0.215 % (worst 22_florist_looking_out 0.215 %, 21_florist_interior 0.208 %),
                         base1 vs base2 0.000 – 0.005 % on the views read; bound noise + 0.5 points → PASS
E-FF-perf    2026-10-15  §8 counts, one run per side (`--slice-perf --res=1920x1080 --gi=voxel`), head − base:
                         view            draws    primitives
                         street wide       −7      +61 494
                         cafe frontage     −6      +75 604
                         interior          −7      +57 042
                         doorway           −7      +56 894
                         street east       −7      +37 706
                         south side        +5      +42 402
                         florist interior  −7      +57 042
                         skyline east      −7      +61 494
                         video memory 1 608.1 → 1 614.5 MB (+6.4 MB). Caps ≤ +24 draws, ≤ +100 000
                         primitives, ≤ +32 MB: PASS. p95 (INCONCLUSIVE, loaded host): base → head
                         10.01→10.23, 16.40→16.75, 12.86→13.87, 18.38→18.80, 10.35→10.90, 9.52→9.99,
                         13.63→14.94, 10.02→10.51 ms — FF-5 (operator, display awake, three runs) decides
```

### Mutations

| Id | Commit | Plant | Expected | Observed | Verdict |
| --- | --- | --- | --- | --- | --- |
| C2-M1 | C2 | `uncovered` steps 1.0° instead of 0.1° | `gap-half-degree.yaml` validates; its row turns red | `every_corpus_file_gives_the_code_its_name_says … FAILED: gap-half-degree`; `every_planted_far_field_is_refused… FAILED (gap-half-degree: FarField {…})` | killed; reverted, 5/5 PASS |
| X-2a | C3 | remove `south-hills` (the silhouette whose sector covers 180°) from `3D/far_field/social-cafe.yaml` | `FAR_FIELD_GAP [a, b)` | `mineworld packs validate presentation/mineworld-default/3D` → "is a valid presentation-pack", rc 0 | **SURVIVED, by the frozen rule** (F-FF-1); restored |
| X-2a′ | C3 | as X-2a, and also remove the skyline sector 170→300 | `FAR_FIELD_GAP` naming the gap | `[mineworld] FAR_FIELD_GAP: presentation/mineworld-default/3D/far_field/social-cafe.yaml: the far field leaves [170.0, 195.0) open`, rc 1 | killed; restored, rc 0 |
| X-1 | C6 | ribbon columns emitted `[bot0, top1, top0], [bot0, bot1, top1]` (second and third vertex swapped) | `FAIL farfield … above` in every non-covered land sector, `below` at VP-high | 36 FAILs: `above` in all 22 non-covered eye-level land sectors (e.g. `FAIL farfield VP-east S above: 0/6111 differ (0.0%) < 25%`), all 8 VP-high `above`, and VP-high `below` N/NE/E/SE/S/NW (e.g. `FAIL farfield VP-high N below: 0/32000 differ (0.0%) < 90%`) — `36 FARFIELD CHECKS FAILED` | killed; restored |
| X-2b | C6 | skip every silhouette whose sector contains 180° | `FAIL farfield VP-east S above` and `VP-high S above` | `FAIL farfield VP-west S above: 0/5460`, `FAIL farfield VP-east S above: 0/6111`, `FAIL farfield VP-high SE below 50.0 %`, `S below 71.2 %`, `S above 0/8960`, `SW above 0.8 %` — `6 FARFIELD CHECKS FAILED` | killed; restored |
| X-4 | C6 | the foreground annulus to 450 m instead of 1 450 m | `FAIL farfield VP-high <sectors> below` | `all farfield checks pass`; with X-4 planted the aerials (`/tmp/impl-rl-e0-x4shots/ff_4_east_aerial.jpg`) show no sky under the horizon either | **SURVIVED — equivalent for the property** (F-FF-2); restored |
| X-5 (slice) | C6 | delete `3D/far_field/social-cafe.yaml` | slice refuses `FAR_FIELD_MISSING`, exits 3 | `rc=3`, `[mineworld-slice] FAR_FIELD_MISSING: …/3D/far_field/social-cafe.yaml: the Presentation Pack …/3D has no far field for the world social-cafe (far_field/social-cafe.yaml)` | killed; restored |
| X-2a′ (slice) | C6 | X-2a′'s file | the client refuses with the same code | `rc=3`, `[mineworld-slice] FAR_FIELD_GAP: …: the far field leaves [170.0, 195.0) open` | killed; restored |
| X-3 | C7 | `surround.extent_m` 10 in `2D/far_field/market-town.yaml` | `FAIL farfield2d market-town … clear-colour px` at 21.3 : 1 | `FAIL farfield2d market-town N 4800x225: 194238 clear-colour px` (and every extreme at 4800×225 and 506×3416) — `16 FARFIELD2D CHECKS FAILED`, `drive complete: FAIL` | killed; restored, all pass |
| X-5 (lakeside) | C3b | delete `3D/far_field/lakeside.yaml` | `FAR_FIELD_MISSING { pack: mineworld-default-3d, world: lakeside }` | `[mineworld] worlds/lakeside/world.yaml: FAR_FIELD_MISSING: the Presentation Pack mineworld-default-3d has no far field for the world lakeside (far_field/lakeside.yaml)`, rc 1 | killed; restored, rc 0 |

### Deviations and findings

- **D-FF-1 (C2, bounded).** *Deviation:* the module is three flat files, `far_field.rs` (API), `far_field_schema.rs`
  (types) and `far_field_check.rs` (rules), not one. *Reason:* one file came to 1 143 lines, past the ~800-line
  strong warning (`CLAUDE.md` §4); flat siblings rather than `far_field/` because `packages/tests/structure.rs`
  reads `src/` as a flat directory. *Impact:* none on the API (`mineworld_packages::far_field::*`).
- **D-FF-2 (C2, bounded).** `read` and `check_pack` take `&[Dimension]`, not one `Dimension`: §5.2 requires a
  manifest listing both `2d` and `3d` to validate against both sets, which one value cannot express.
- **D-FF-3 (C2, bounded).** Fixture names use `-` for `_` (`scope-mismatch.yaml` for `FAR_FIELD_SCOPE_MISMATCH`):
  a file's stem must equal its `scope` key, and a key admits no `_`. A variant is `<code>-<variant>.yaml`; stems
  ending `-2d` are read as a 2D pack's, all others as a 3D pack's. `FAR_FIELD_MISSING` has no fixture file (it is
  the absence of one): Rust tests it with `require_for_world`/`check_pack`; the GDScript check asks for an absent
  key. `FAR_FIELD_WORLD_UNKNOWN` is client-only (as §12 C2 step 5 says).
- **D-FF-4 (C2, bounded; both validators).** The order of checks, which decides the one code a file gets:
  MALFORMED (SLOT during parsing) → SCOPE_MISMATCH → SECTION_MISSING → SECTION_FOREIGN → UNCITED → OUT_OF_RANGE
  → INSIDE_BOUNDARY → PROFILE → GAP. Classifications the schema leaves open: `format ≠ 1`, a sky's
  `panorama`/`procedural` not matching its kind, river-only fields on a non-river (or missing on a river) →
  MALFORMED; an empty `terrain_silhouettes` or `skyline.sectors` → SECTION_MISSING; `from_deg == to_deg`, haze
  `to_m ≤ from_m`, massing `to_m ≤ from_m` → OUT_OF_RANGE; `shore_m == reach_m` → INSIDE_BOUNDARY (§5.1.3's "≤"
  governs §5.1.2's "≥"); a gap through north is reported as one run (`[350.0, 10.0)`).
- **F-FF-1 (C3, finding for the primary; no frozen rule changed).** *Previous assumption:* X-2a (§6.7) —
  removing the silhouette that covers 180° from `social-cafe.yaml` — makes `packs validate` refuse
  `FAR_FIELD_GAP`. *Audit evidence:* §5.1.3 (frozen, now `MODULE_SPEC.md` §6.1.1) counts `skyline.sectors[]`
  in the coverage union, and Appendix B.1's two skyline sectors (300→170, 170→300) cover 360° on their own
  (B.1's own note: "skyline sectors also cover 360°"). The literal X-2a therefore validates (table above).
  *Corrected understanding:* the schema check proves that *something* is declared in every direction; a
  missing terrain band behind declared massing is caught by the rendered check (X-2b, FF-ABOVE), which is
  the reason §6.3 gives FF-ABOVE. *Implementation consequence:* none; the frozen rule and content are kept.
  *Validation consequence:* X-2a′ (the band **and** the skyline sector over the same bearings) proves the
  content-level GAP refusal on the shipped file; X-2b carries the deleted-band case. Recorded for the primary
  session to acknowledge; if the operator wants X-2a literal to fail, the smallest revision is to drop
  `skyline.sectors` from the GAP union (a §6.1.1 rule change, not made here).
- **D-FF-5 (C3/C3b, bounded).** Tests that build a scratch Presentation Pack a world requires now give it a
  far-field file: `tools/cli/tests/requirements.rs` and `worldpack/tests/requirements.rs` (`style()`), and
  `packages/tests/resolve.rs` points a found Presentation Pack at `packages/tests/fixtures/roots/<id>/`
  (with `far_field/the-world.yaml`) instead of the virtual `/roots/<id>`. Outside C3b's file list; needed
  because C3b makes the far field a resolution rule, as designed.
- **D-FF-6 (C3b, bounded).** `tools/cli/tests/requirements.rs` `the_default_presentation_pack_can_be_required`
  required `mineworld-default-3d` from a scratch world `the-world`, which that pack has no far field for; under
  C3b it is refused, as I-FF-1 requires. Renamed `the_default_presentation_pack_needs_a_far_field_for_the_world`
  and asserting the refusal by name; the positive case (a world the pack presents resolves with it) is
  `milestone_e.rs`'s Lakeside, green.
- **D-FF-7 (C5, bounded; geometry, not a frozen invariant).** *Previous assumption:* §5.4's proxy distances
  900 / 1 100 / 1 300 m. *Audit evidence:* the world-anchored foreground and bay reach 1 450 m and the bay's
  shore is at 1 000 m; a ribbon at 900 m reaching 58 m below the datum (§5.4's base) stands in front of
  everything world-anchored beyond 900 m, so it would hide the bay (AC-FF-8's "`ff_6` shows the bay") and the
  far massing. *Corrected:* proxies 1 480 / 1 520 / 1 560 m: beyond the world-anchored parts, inside
  `CameraRig.FAR` 1 600 m (unchanged). Angles, the per-layer height formula and the haze are as designed; only
  the proxy radius (which carries no meaning beyond depth order) moved.
- **D-FF-8 (C5, bounded).** The bay's "Fresnel capped at 0.6, no reflections of the town" (SD-FF-5, Q-FF-2):
  Godot's lit specular Fresnel tends to 1 at grazing whatever `SPECULAR` is, so it cannot be capped. The
  `FAR_WATER` branch of `water.gdshader` is unshaded and mixes the water colour with the sky's horizon colour
  by a Schlick Fresnel `min(…, fresnel_cap)` on the noise normals; `SliceFarFieldView` prepends
  `#define FAR_WATER` to the shader's code at runtime, so the promenade's lake (`town.gd`) compiles the
  unchanged lit branch.
- **D-FF-9 (C5, bounded; look).** Foreground colour: §5.4's sRGB (0.38, 0.36, 0.27) is an albedo; applied as a
  tint on the green `leafy_grass` texture it rendered a dark brown. The tint is the albedo over the texture's
  mean colour, and the ground blends by vertex colour from the near disc's grass at the boundary to that
  surface over `run_m`, so it does not change colour in a line at 60 m. The near disc keeps the old apron's
  tint so accepted near views do not change (V-1).
- **D-FF-10 (C5/C6, bounded).** A Godot window that macOS considers hidden is not drawn, and a check waiting on
  `frame_post_draw` then waits forever (observed once: an X-1 run sat 10 min in the run loop's frame delay; the
  2D launcher already records this for `--capture`). `./mineworld-slice --farfield` and `--farfield-shots` pass
  `--always-on-top`, as `./mineworld-2d --capture` does.
- **F-FF-2 (C6, finding for the primary; no frozen rule changed).** X-4 (the foreground to 450 m) does not fail
  the rendered check, and is equivalent for the property it was meant to prove. *Audit evidence:* every
  ribbon reaches 40 m below the foreground's lowest point (§5.4, "so the ribbon always reaches below the
  ground it stands behind"), the silhouettes cover 360° (B.1), and the massing stands between 250 and
  1 400 m; with the foreground cut at 450 m, the angles below the horizon are filled by massing and ribbon
  base, not sky — the aerials taken with X-4 planted show no void. The defect X-4 models (the ground's edge
  against the sky, E-FF-5) is therefore structurally excluded by §5.4's ribbon base, and FF-BELOW is what
  would catch a ribbon base that stopped short (X-1 at VP-high). Recorded for the primary; no change made.
- **D-FF-11 (C6/C8, bounded; CI budget unchanged).** R-FF-1's viability run (C6 step 3) and C8's dispatched
  nightly are one run: the hosted runners run the check through C8's Rust harness, so a C6-only dispatch would
  need a throwaway harness. The branch is pushed after C8 and the nightly `clients` group dispatched once on
  it; if a runner cannot render, that is the material stop R-FF-1 names, before C9.
- **D-FF-12 (C7, bounded; makes §6.4 say what it means).** §6.4 sets the check camera's zoom to "pack zoom ×
  (H / 900) (the `canvas_items`/`expand` equivalent)". That equals the client only for windows wider than
  16:9. `expand` keeps the whole 1 600 × 900 base visible, so a tall window (the settings' 640 × 4 320, rendered
  506 × 3 416) shows the full base width and far more height; H / 900 there would zoom 3.8 × too far in and
  check a smaller view than the player sees. The check uses `pack zoom × min(W / 1600, H / 900)`, which is the
  same as §6.4 for the three wide aspects and the true equivalent for the tall one.
- **D-FF-13 (C7, bounded).** §6.4's extremes are those of `town.hub_extent()`, which the client learns only
  from the hub: the 2D client starts the player indoors (market-town: the apartments), where the hub's extent
  is empty and every extreme was the origin (the first X-3 run showed identical counts at all eight). The
  `farfield` scenario first walks out through the starting place's doorway (`_walk`'s first leg). A world
  with no hub (bodies-yard: two rooms, one doorway each) uses the union of its drawn places' footprints.
- **D-FF-14 (C7, bounded; launcher, not the client).** `./mineworld-2d` hosts every world with `--agent alice`
  and seat `carol`; lakeside and bodies-yard have neither, and lakeside is found only through its pack roots.
  Their runs pass `--agent=none --seat alba` with `MINEWORLD_PACKS=<repo>/entities:<repo>/presentation/mineworld-default`,
  and `--agent=none --seat gus`; the Rust test does the same through `World::start` (`--packs`).
- **D-FF-15 (C8, bounded).** The `farfield` job runs through `.github/actions/native`, which names a
  `scripts/ci_layer.py` layer, never a command (step-14 I-S13-9); so the per-PR command is the new layer
  `farfield` (build, the three tests, the scratch check), in a file C8's list did not name.
- **D-FF-16 (CI repair after C10).** (a) PR CI `test` failed `tests/acceptance/tests/client_rules.rs`
  `no_client_script_names_an_action_copies_a_rule_or_reads_a_pack`: the scan reads the identifier `reach` in a
  client script as a rule name (I-S14-1). It was only the far field's boundary radius (presentation), so the
  local variables are renamed `edge_m` / `outer_m` (`far_field_view.gd`, `ground.gd`, `far_field.gd`); the
  scan is unchanged. AC-FF-12's Python half had passed; its Rust half had not been run locally — it is now,
  3/3 PASS. (b) Scratch nightly 38793596092 (head 8bc55052), `clients-windows`: `the_two_validators_agree` and
  `two_d_clear_colour_is_never_visible` **passed on Windows (D3D12 WARP)**; `three_d_far_field_closes_every_open_sector`
  failed only because the first `--import` of the 3D project exceeded the harness's 600 s import limit
  (`left: None` at the import assertion) — raised to 30 min (`IMPORT_LIMIT`). The run's three
  `client_settings` failures on Windows (`three_d_controls_apply_live_and_persist`,
  `three_d_settings_never_reach_the_server`, `two_d_the_settings_file_holds_only_settings_and_a_bad_one_is_kept`)
  are pre-existing: main's nightly 38757085415 (3a9b9eff) fails the same three plus a fourth.

### MATERIAL STOP — R-FF-1 (2026-10-15): the 3D rendered check does not finish on the hosted Linux runner

*Evidence.* PR CI run 38803249486, job `farfield` (ubuntu-24.04, Xvfb + Mesa lavapipe, head 1bf3cff8):
`the_two_validators_agree` ok (6 s); `two_d_clear_colour_is_never_visible` ok (≈ 3.5 min, four worlds);
`three_d_far_field_closes_every_open_sector` **INCONCLUSIVE: the slice did not finish in 2400 s** — it printed 26
of its 112 verdict lines (VP-spawn complete, VP-street to S), every one agreeing with macOS (e.g. `VP-spawn E
below 3521/30720 open, 3521 differ (100.0%)` against 3 523 locally): the renders are correct, about a minute
each, so the 120 renders need about two hours. The scratch nightly 38793596092 (head 8bc55052): `clients-linux`
ran 80 min and was cancelled at the job's 75-min limit (main's `clients-linux` takes 34 min); `clients-windows`
(D3D12 WARP): the 2D check and the validators **passed**, the 3D check never started (its first import of the 3D
project exceeded the 600 s limit, since raised, D-FF-16); `clients-mac`: see E-FF-CI. Locally (Apple GPU) the
same check takes 18 s.

*Why it is material.* R-FF-1 names this case (§16) and §13 lists it as a material stop; the remedies change
§6.2's frozen renders or §6.5's where-it-runs, which are the operator's.

*Options (smallest first).*
(a) Keep §6 as frozen and run the 3D rendered check on hosted **macOS** only (and locally), with the 2D check
    and the validators on all three OSes; the per-PR `farfield` job runs the 2D check and the validators on
    Linux, and a macOS per-PR job (or the nightly's `clients-mac`) runs the 3D check.
(b) Render the check frames with the slice's cost-only effects off (shadows, SSAO, glow — none decides where
    the far field is drawn; the far field casts no shadow and is unshaded or fog-free), keeping §6's
    viewpoints, size, bands and thresholds; re-measure the hosted time.
(c) §16's own fallbacks: a lower check resolution, or a far-field-only scene (which would no longer test near
    occlusion).
(d) Give the Linux and Windows legs a 2–3 hour budget for this one test (nightly only).
Recommendation: (b), then (a) for whatever OS still cannot finish.

*State.* Everything else is implemented and committed; the PR is **not** READY FOR OPERATOR REVIEW until the
operator rules on R-FF-1.

**R-FF-1 — Ruled 2026-10-15 (operator answered directly in the primary session): (b), then (a).**
(b) For the check frames only, directional shadows, SSAO and glow are off (`SliceFarFieldCheck._cost_effects_off`,
restored afterwards; `--farfield-shots` keeps the full look); §6's viewpoints, sizes, bands and thresholds are
unchanged. (a) Any OS whose job still cannot finish the 3D check within its limit runs it on hosted macOS only
(and locally); the 2D check and the validators' agreement stay on all three OSes. Local after (b): 112 verdict
lines, all pass; X-1 still `36 FARFIELD CHECKS FAILED`. Per-OS hosted timings and the resulting CI layout:
E-FF-CI-2 below. *Dated note on §6.2/§6.5 (2026-10-15):* the renders of §6.2 are taken with those three effects
off; where §6.5 says the 3D check runs on all three runners, read the OSes E-FF-CI-2 names.

```text
E-FF-CI  2026-10-15
  PR #220, head 1bf3cff8 (CI run 38803249486): changes, fast, test (44 min; the client_rules fix of
           D-FF-16 confirmed), test-windows, platforms (macos-26, windows-2025), python ×3: PASS;
           test-macos: pending at the stop; farfield: FAIL — INCONCLUSIVE 3D check (R-FF-1, above);
           its 2D check and validators PASS on Linux
  scratch nightly 38793596092 (head 8bc55052, import limit 600 s):
           clients-windows: client_far_field 2/3 (2D, validators PASS; 3D: import > 600 s);
             client_settings: 3 failures, pre-existing on main (nightly 38757085415)
           clients-mac (Apple Paravirtual): client_far_field 2/3 (2D, validators PASS; 3D: import > 600 s);
             the job cancelled at its 75-min limit
           clients-linux: cancelled at 75 min (logs not retained)
  scratch nightly 38803248313 (head 1bf3cff8, import limit 30 min): running at the stop — it will show
           whether hosted macOS and Windows finish the 3D check
```

---

## Appendix A — Specification texts (C1 pastes these)

### A.1 `docs/MODULE_SPEC.md` §6.1

In the first layout block, after `renderer/godot.yaml, renderer/unreal.yaml`, add:

```text
  far_field/<world-key>.yaml   REQUIRED: one per presented world (§6.1.1)
```

Replace "Not every pack needs every directory." with "Not every pack needs every directory, except
`far_field/`, which every Presentation Pack carries (§6.1.1)." After the paragraph ending "…that the style
manifest has an `id` and a `dimension` list.", add "It also checks every file under `far_field/` (§6.1.1)."

New subsection after §6.1 (before §6.2):

> ## 6.1.1 The far field (required)
>
> **However small the world, every presented view closes on a far field** (operator ruling 2026-10-15,
> `DECISIONS.md` ARC-102). An invisible wall is allowed; a view with no far field is not.
>
> A Presentation Pack carries `far_field/<world-key>.yaml` for every world it presents, and
> `far_field/region-<region-key>.yaml` for a region (read from S21). Each file is YAML's JSON subset
> (`DEP-16`). *[Insert §5.1.2's schema block, §5.1.3's table and §5.1.4's two refusal points verbatim, with
> "§5.1.x" references changed to this section.]*
>
> The far field is decoration. It lies beyond `boundary.reach_m`, is never reachable, never disclosed by a
> System and never part of an `ActionIntent`. A far field that suggests a walkable approach beyond the
> boundary is a defect even if it validates.
>
> The reference clients prove the far field by rendering (`DECISIONS.md` ARC-103): the 3D client compares
> frames with the far field shown and hidden from fixed viewpoints, and the 2D client counts clear-colour
> pixels at the reachable extremes.

### A.2 `docs/ART_DIRECTION.md`

§12, after "Not every style pack must contain every directory.": "The exception is `far_field/`, which every
pack carries: one file per presented world (`MODULE_SPEC.md` §6.1.1)." In the layout block add
`far_field/` under `renderer/`. §21 "Environment": append "In every direction the view closes on a far field
— sky, distant terrain, water where the place has it, and the town's distant massing — at the real place's
distances and angular sizes. An edge of the ground against the sky, or an empty background colour, fails this
criterion."

### A.3 `docs/VISUAL_SLICE.md` (engine-neutral)

- §1, after the paragraph "**The town is not to be expanded.**": "*(Amended 2026-10-15.)* Far-field water,
  terrain and distant townscape (§4, far field) are not an expansion: they have no interior, no approach and
  nothing to do, and a harbour in this sentence means a walkable one."
- §4, replace lines 129–130 ("Acceptable terminations: … **A hard invisible wall across an open view is not an
  acceptable termination.**") by: "Acceptable terminations: a corner turning out of sight, a rise, a drop, a
  planted bank, a bridge, a closed gate, dense foliage, or an invisible boundary. *(Amended 2026-10-15, operator
  ruling.)* **An invisible boundary is acceptable; a view that does not close on a far field is not.** Beyond
  any termination, nothing suggests a walkable approach."
- §4 table, replace the **backdrop** row by: "**far field** | every direction, 360°, closes on a far field: sky,
  distant terrain at the real place's distances and angular sizes, water where the place has it, and the
  town's distant massing (`MODULE_SPEC.md` §6.1.1). A backdrop that closes only the frontage's ends does not
  satisfy this row. Neither the ground's edge nor an empty background may be visible from any reachable point
  or from the elevated review view."
- §4 out of scope: "any building the player can see but never reach" → "any building **in the street** the
  player can see but never reach (distant massing in the far field is not a building of the slice)".
- §12, add: "11 far field — four compass views and one elevated view, plus the view from each end of the
  street (`clients/<client>/shots/<world>/farfield/`)".

### A.4 `clients/2d/PRESENTATION.md`

A section "The far field (required)": the 2D pack's `far_field/<world-key>.yaml` carries `boundary`, `water`
and `surround` (`MODULE_SPEC.md` §6.1.1); the client draws the surround past the land rectangle, never
clamps the camera, and is checked to draw no clear-colour pixel at any reachable extreme at the widest aspect
the settings allow. `clear_color` remains in `renderer/godot.yaml` but must never be visible.

### A.5 `docs/DECISIONS.md` (append; numbers assigned 2026-10-15)

> **ARC-102 — Every presented world has a far field; a Presentation Pack without one is refused.**
> Date 2026-10-15 · Approved by the operator (ruling 2026-10-15, quoted) and the primary session (freeze) ·
> Design `pr-rl-e0-far-field-mandatory.md`. *Problem:* E-FF-1…8. *Options:* (a) a required, typed `far_field`
> per world in each dimension pack, refused by name at authoring and at load — chosen; (b) a client-side
> backdrop convention (what failed); (c) a pack-wide fallback (makes refusal unreachable, draws the wrong place).
> *Consequences:* the schema of `MODULE_SPEC.md` §6.1.1; the 3D client reads `far_field/` before 16f; RL-b C8's
> reverted skyline technique is reused; RL-e fills the same schema from 3DEP; the clients take `--world-key`
> until the protocol carries a pack id (revisit with S21).

> **ARC-103 — The far field is proven by rendering, against the same run's own frames.**
> *Problem:* a winding assertion passed while the GPU culled the faces (F-FF-1); schema and ray-cast checks
> cannot see render-time culling. *Choice:* shown-vs-hidden-vs-sky renders from fixed viewpoints with two fixed
> criteria (FF-BELOW 90 %, FF-ABOVE 25 %), and two-sentinel clear-colour counts in 2D, on all three hosted
> runners; planted mutations prove failure. *Not chosen:* golden images (renderer-dependent, `ARC-17`'s
> vagueness in pixel form); a ray-cast check (blind to culling).

> **ARC-104 — Invisible boundaries are allowed; open views without a far field are not.** Amends
> `VISUAL_SLICE.md` §4 (A.3) on the operator's ruling.

> **DEP-45, dated note 2026-10-15:** the accepted limitation "judge behaviour, never frames" is narrowed: the
> far-field checks judge frames **only against frames rendered in the same run** (ARC-103), never against a
> stored image; every other windowed test is unchanged.

---

## Appendix B — Default content (C3 writes these; numbers as given)

Bearings true, clockwise from north. "Summit" values are RL-b C8's, re-read at source 2026-10-09 (E-RLb-7;
`git show 58bc0c4:clients/3d-spike/scripts/slice/skyline.gd`, the `SUMMITS` table). "Approx." values are this
design's estimates from published elevations of the named places; every such `source` string ends with
"approximate; replaced by RL-e (USGS 3DEP)".

### B.1 3D `social-cafe`

```text
format 1; scope {"world": "social-cafe"}
boundary   natural; reach_m 60; beyond "drop"
datum_m    18      source "Market Town's location 32.7157 N, -117.1611 W; street about 18 m above sea level
                          (RL-b C8 EYE_M 20 m minus eye height); approximate; replaced by RL-e (USGS 3DEP)"
sky        panorama "qwantani_puresky_2k"; visibility_m 80000
           visibility_source "clear coastal afternoon, RL-b Q-RLb-5 (operator-judged default); Koschmieder at
                              WMO 5 % contrast (WMO-No. 8, Part I ch. 9)"
foreground dry_grass; drop_m 18; run_m 300

terrain_silhouettes (bearing: height_m @ distance_m)
  mesas       near  sector 315→135  albedo #6b6648
              315: 70@2400  345: 85@1900  15: 95@1700  45: 105@2000  75: 95@2200  105: 75@2000  135: 55@2400
              source "Bankers Hill, Hillcrest, Balboa Park and Golden Hill mesa tops; approx."
  soledad     mid   sector 320→340  albedo #6f6a50   320: 120@15000  330: 251@15900  340: 120@15000
              source "Mount Soledad 251 m, NGS datasheet 'Soledad' (summit, re-read 2026-10-09)"
  east-mid    mid   sector 30→125   albedo #6f6a50
              30: 300@16000  49: 486@16300  71: 416@17500  98: 783@19000  125: 350@20000
              source "Cowles Mountain 486 m (GNIS/Peakbagger), Mount Helix 416 m (GNIS), San Miguel Mountain
                      783 m (NGS 'San Miguel Reset'); summits re-read 2026-10-09; between them approx."
  east-far    far   sector 55→120   albedo #77735c
              55: 1200@55000  64: 1985@57900  75: 1911@72000  114: 1088@32600  120: 800@32000
              source "Cuyamaca Peak 1985 m (NGS 'Cuyamaca reset'), Monument Peak 1911 m (hundredpeaks.org),
                      Otay Mountain 1088 m (NGS 'Otay' DC2046); summits re-read 2026-10-09"
  south-hills mid   sector 125→240  albedo #6f6a50
              125: 350@24000  150: 300@22000  175: 260@26000  200: 200@26000  240: 40@14000
              source "hills south and south-east of the bay (National City, Otay Mesa, Tijuana); approx."
  coronado    near  sector 195→245  albedo #5f6150   195: 12@3500  220: 18@2500  245: 10@3800
              source "Coronado, flat at about 5 m with low buildings, across the bay; approx."
  point-loma  near  sector 235→315  albedo #66624a
              235: 40@9000  249: 126@7900  270: 120@6800  297: 110@6300  315: 60@7000
              source "Point Loma 126 m (GeoNames/GNIS, Old Point Loma Lighthouse; re-read 2026-10-09); the
                      peninsula's ridge northward and Loma Portal approx."

water
  san-diego-bay  bay  sector 170→300  shore_m 1000  level_m 0  horizon closed
                 colour deep #3c6670 shallow #5f8f9a
                 source "San Diego Bay: shoreline about 1.0 km west and 1.2 km south-west of the origin; approx."

skyline  seed 20261015
  sectors  sector 300→170  from 250 to 1400  height 6–14  density 0.35
           sector 170→300  from 250 to  900  height 6–12  density 0.25
           source (both) "low-rise town massing in the slice's architecture (QRL-2/3: the lakeside references
                          govern architecture); storey heights VISUAL_SLICE.md §3.2"
  slots    station  reserved  bearing 277  distance 790  footprint 120
                    source "Santa Fe Depot, 32.7166 N -117.1695 W (approximate; re-read at freeze); bearing and
                            distance from the origin by equirectangular approximation"
           landmark unplaced
```

Coverage check by hand: silhouettes cover 315→135 ∪ 30→125 ∪ 55→120 ∪ 125→240 ∪ 195→245 ∪ 235→315 = 360°;
skyline sectors also cover 360°. FF-ABOVE in every land sector is carried by a near or mid layer.

### B.2 3D `lakeside` (design content; no real location)

> *Dated note (2026-10-15, S33 approval; not a change to RL-e0's frozen scope).* B.2 and B.3's `lakeside`
> rows are **replaced by S33-a** ([`../mvp0/step-33-lakeside-3d.md`](../mvp0/step-33-lakeside-3d.md) §3.4,
> `ARC-143`): Lakeside is set at Lake Cuyamaca (Q-S33-1), with a cited far field. The lake moves **south**
> in both the 3D and 2D files (S33 F-S33-2): the world puts the shore and the pier on the lane's south side,
> so the 300°→60° sector below is wrong against the world's own data. RL-e0 ships B.2/B.3 as written; S33-a
> replaces them, and AC-FF-4 holds the two packs together.

```text
boundary natural; reach_m 80; beyond "water"
datum_m 2; datum_source "design content; lakeside declares no real location"
sky panorama "qwantani_puresky_2k"; visibility_m 40000; visibility_source "design content: a hazy lake afternoon"
foreground grass; drop_m 2; run_m 60
terrain_silhouettes
  far-shore  mid  sector 300→60   #5d6a4a  300: 250@4000  0: 320@4500  60: 260@4000
  hills      mid  sector 60→300   #5d6a4a  60: 260@4000  120: 180@3000  180: 150@2500  240: 200@3000  300: 250@4000
  (sources: "design content; lakeside declares no real location")
water  lake  lake  sector 300→60  shore_m 90  level_m 0  horizon closed  deep #3d5f63 shallow #6a9295
skyline seed 7; sector 60→300 from 150 to 900 height 6–10 density 0.2; slots station unplaced, landmark unplaced
```

### B.3 2D files

```text
market-town, social-cafe:
  boundary natural; reach_m 60; beyond "drop"
  water  san-diego-bay as B.1 (the same sector, shore, level, horizon, colours, source)
  surround extent_m 400; base "land"; haze colour "#c9d3d6" from_m 60 to_m 400
lakeside:
  boundary natural; reach_m 80; beyond "water"
  water  lake as B.2
  surround extent_m 400; base "land"; haze "#c9d3d6" 60→400
bodies-yard:
  boundary invisible_wall; reach_m 40; beyond "rough_ground"; water []
  surround extent_m 400; base "land"; haze "#c9d3d6" 40→400
```
