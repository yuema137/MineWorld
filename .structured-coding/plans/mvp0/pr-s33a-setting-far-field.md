# PR S33-a — Lakeside's real setting (Lake Cuyamaca) and its far field, the lake moved south in both packs

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: freeze with the recommended answers)

```text
Design revision:        revision 1 (2026-10-17), FROZEN 2026-10-18
Author:                 S33-a/b/c PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s33abc, branch docs/pr-designs-s33abc,
                        planning base origin/main @ da80316f (#224; S29-a/S30-ab designs #234 and S30-a #238
                        merged). Designed against PR #220 (RL-e0) head e765df47
Approved by / evidence: the step's rulings (Q-S33-1…11, 2026-10-15) bind and are not re-asked; §14's
                        Q-S33a-1 ruled 2026-10-18 by the operator in the primary session, as recommended
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of RL-e0, S29-a,
                        S29-c
Start condition:        (1) Milestone F's checklist handed to the operator (Q-S33-10); (2) RL-e0 merged
                        (met: #220, merge 621ea0e2); (3) S29-a merged (NOT met at 2026-10-18); (4) S29-c
                        merged (NOT met: no PR design exists yet); (5) Q-S33a-1 ruled (met: 2026-10-18);
                        (6) this design frozen (met: 2026-10-18); (7) the primary session's dispatch
Execution contract:     §12
Implementer:            Haiku-class (step-33 §12.1, the S33-a row: "data by a fixed method"); the profile and
                        the site are reviewed by Opus against the map before C3 (the row's "Opus reviews the
                        profile against the map")
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) (APPROVED
2026-10-15; Q-S33-1…11 ruled as recommended, §14.1) §§0.2, 2.3 (F-S33-2), 3 (all), 4.0, 4.7, 9.1, 9.4 (X-1, X-2),
10 (I-S33-1, I-S33-4, I-S33-7), 11 (AC-S33-1, AC-S33-10), 12.1 (the S33-a row), 15 (`ARC-143`), 16.4, 16.5 ·
**Parent:** [`overall.md`](overall.md) · **Consumes:**
[`../mvp1/pr-rl-e0-far-field-mandatory.md`](../mvp1/pr-rl-e0-far-field-mandatory.md) (FROZEN; PR #220 open;
format 1, App. B.2/B.3, AC-FF-4, the V-gate rule), [`step-29-station-landmark.md`](step-29-station-landmark.md)
§§4.3, 6.7 (the `filled` slot, `FAR_FIELD_SLOT_TOWN`), [`pr-s29a-town-contract.md`](pr-s29a-town-contract.md)
(DRAFT; the halt's places and `settlement:`) · **Followed by:** S33-d (the near terrain and near lake built to
these numbers; AC-S33-1 on three OSes), S33-h (the 2D far field's look) · **Binding parents:** `CLAUDE.md`
§§2, 3, 4 (items 2, 3, 7, 9, 14, 16); `docs/MODULE_SPEC.md` §6.1.1 (as RL-e0 and S29-c leave it);
`docs/ART_DIRECTION.md` §4; `docs/DECISIONS.md` `ARC-102`…`ARC-104`, `ARC-107`.

**Decision ids.** `ARC-143` only (assigned by the step, §15), written by C1. No other id is used.

No hosted model, image model or paid API is called by this design. Public USGS services are read (no key, no
cost); their responses are cached in the repository so that no later run or CI job reads the network (§5.3).

---

## 0. Binding inputs, exactly restated

- **Scope (step-33 §12.1, S33-a row).** "C1 spec: `ARC-143`; a dated amendment note on RL-e0 App. B.2/B.3 and
  S29 §6.7. C2: 3DEP and NHD reads by RL-b C8's method, the profile table with sources. C3:
  `far_field/lakeside.yaml` in 3D and 2D, lake south, slots filled."
- **Depends on.** "RL-e0 merged; S29-a (halt place); S29-c (`filled`); Q-S33-1/2 ruled."
- **Integration checkpoint.** "`mineworld validate worlds/lakeside --packs …` passes; the far-field check runs
  on a stub Lakeside depiction (S33-c) or on RL-e0's harness."
- **Adversarial.** X-1 ("delete the lake's water sector from the 3D far field" → `FAR_FIELD_GAP` at validate, or
  `FF-BELOW` in the south sectors at VP-L-3) and X-2 ("move the 3D lake to the north only" → RL-e0's AC-FF-4).
- **Rulings that bind.** Q-S33-1: **Lake Cuyamaca, the village on the north shore, the lake to the south.**
  Q-S33-2: **the halt is a stated departure from history, recorded in the far field's `source`.** Q-S33-9:
  **`reach_m` 80 with air walls**, raised only if the halt's platform end lies outside. Q-S33-11: **the location
  is presentation only** (no `configure/`, no calendar location, no weather station in `worlds/lakeside`).
  Q-S33-7(a, c): 0 Meshy credits, nothing bought.
- **Content (step-33 §3.4).** The 3D file's `scope`, `boundary`, `datum_m 1408`, `sky`, `foreground`, `water
  cuyamaca` (`kind lake`, `level_m 1406`, `horizon closed`, the source line), `terrain_silhouettes` (near: the far
  shore and the rises round it; mid: Cuyamaca Peak, Middle and North Peak, Stonewall Peak; far: the Laguna and
  Volcan Mountains, named only if the profile shows them above the near terrain), `skyline` (sparse, seeded,
  slots filled). The 2D file: "the same lake sector, level, colours and source (AC-FF-4), base `land`, haze
  re-picked. The surround keeps RL-e0's extent." "No schema change."
- **R-LOAD-1 (coordinator rule, in force).** Locally: only the commit's targeted checks (`cargo test -p
  mineworld-packages --test far_field`, the `mineworld` validate commands, one Godot probe at a time). Never a
  local `cargo test --workspace`. Local timings are INCONCLUSIVE. CI on the exact head is the gate.

---

## 1. Goal

Give Lakeside a real, cited setting — the north shore of Lake Cuyamaca, San Diego County — and replace RL-e0's
"design content" Lakeside far fields (App. B.2, B.3) with files derived from public data by RL-b C8's method:
the lake to the **south** in both the 3D and the 2D pack (F-S33-2), Cuyamaca Peak and its neighbours as the
southern skyline, the slots `station` and `landmark` filled with the halt's and the pier's positions, and every
number carrying its source. Nothing in the kernel, contracts, Systems or `worlds/` changes (I-S33-1, Q-S33-11).

---

## 2. Scope

### 2.1 In scope

1. `presentation/mineworld-default/3D/far_field/lakeside.yaml` — rewritten (§5.4).
2. `presentation/mineworld-default/2D/far_field/lakeside.yaml` — rewritten (§5.5).
3. `tools/far-field/lakeside/` (new): `derive.py` (standard library only), `inputs.json` (the site, the summit
   table with citations, the sample plan), `cache/epqs.json` and `cache/nhd.json` (the recorded USGS
   responses), `SOURCE.md` (the reproduction command, the base SHA, the profile table) (§5.3).
4. `packages/tests/far_field.rs`: `two_d_and_three_d_water_agree` also compares `kind` and `level_m` (§5.6).
5. `docs/DECISIONS.md`: `ARC-143`.
6. `docs/ART_DIRECTION.md` §4: the setting rule of step-33 §3.3 (step §16.5).
7. Plan files: dated "landed" notes in the RL-e0 design App. B.2/B.3 and step-29 §6.7 (the "replaced by S33-a"
   notes already exist, §3.2 F-S33a-1); this file's ledger.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Drawing Lakeside in 3D (world selection, a depiction, the rendered check from VP-L-1…7) | S33-c (seam, stub), S33-d (terrain, near lake, AC-S33-1 on three OSes) |
| The near lake surface from the shore line to `reach_m`, the woods ring, the air walls | S33-d |
| The 2D far field's drawn look beyond the data (S33-h's façades and names) | S33-h |
| Any `worlds/lakeside` change (location, calendar, weather, bodies) | never here (Q-S33-11; bodies are S33-b) |
| The `filled` slot state and `FAR_FIELD_SLOT_TOWN` | S29-c (this PR only fills Lakeside's values) |
| A far-field schema change | none (step §3.4: "No schema change") |
| `docs/VISUAL_SLICE.md`'s dated note (step §16.5) | S33-d (it adds Lakeside's review views) |

### 2.3 Dependencies

**Required before C0:** RL-e0 merged (the files, the validator, AC-FF-4's test exist only on #220); S29-a merged
(`settlement:` and the halt's three places on `main`, so the station slot has a place and a position); S29-c
merged (`filled` exists in Rust and GDScript; Lakeside's slots are then `filled` with design content that this
PR replaces). Q-S33a-1 ruled (the halt's end, §14). Not required: S33-b, S33-c (if S33-c has merged by C4, the
rendered check is run on its stub; otherwise it is recorded as S33-d's, §8 AC-a-6).

---

## 3. Audit (`origin/main @ da80316f`; RL-e0 at `origin/mvp1/pr-rl-e0 @ e765df47`)

### 3.1 Source anchors

| # | Fact | Where |
| --- | --- | --- |
| A-1 | No far-field file exists on `main`. On RL-e0: `presentation/mineworld-default/3D/far_field/{lakeside,social-cafe}.yaml`, `2D/far_field/{bodies-yard,lakeside,market-town,social-cafe}.yaml` | rl-e0 tree |
| A-2 | The 3D `lakeside.yaml` is App. B.2 verbatim: boundary `natural`/`reach_m 80`/`beyond water` (:4); water `lake`, sector 300→60 (:8), `shore_m 90`, `level_m 0`, `closed` (:9); `datum_m 2` (:14–15); silhouettes `far-shore` 300→60 (:23) and `hills` 60→300 (:32); skyline seed 7 with both slots `unplaced` (:52); every `source` "design content; lakeside declares no real location" | rl-e0 `3D/far_field/lakeside.yaml` |
| A-3 | The 2D `lakeside.yaml` (18 lines): the same `lake` water (:5–13), `surround { extent_m 400, base land, haze #c9d3d6 60→400 }` (:14–17). The RL-e0 design's text at Doc:543 says "water base north"; the file says `land` | rl-e0 `2D/far_field/lakeside.yaml`; Doc:543 |
| A-4 | Format 1 (Doc:265–366): bearings true, [0, 360), sectors clockwise and may wrap, `from == to` refused; heights above sea level; unknown keys refused; files are YAML's JSON subset (no comments, DEP-16). `ProfilePoint.distance_m > reach_m`; profile bearings strictly increasing, first at `from_deg`, last at `to_deg`. `Water.shore_m ≥ reach_m`, `level_m` [−430, 5000]. `datum_m` [−430, 5000] with `datum_source`. `Sky.visibility_m` [1000, 300000] with `visibility_source`. `Foreground.drop_m` [0, 500], `run_m` [10, 2000] | Doc §5.1.2 |
| A-5 | `FAR_FIELD_GAP` (3D): the union of silhouette sectors, skyline sectors and **open** water sectors must cover [0, 360); **closed water does not count**; the message names the first gap to 0.1° | Doc §5.1.3 (:368–387); `packages/src/far_field_check.rs:401` `check_coverage` |
| A-6 | The validator: `packages/src/far_field.rs` (`check_pack` :200–233; `require_for_world` :244–257 is **presence only**); `far_field_check.rs` `parse` :17–38 runs shape, scope, sections, citations, ranges, boundary (`INSIDE_BOUNDARY`) :335, profiles :374, coverage :401; the schema in `far_field_schema.rs` (`Water` :203, `Slots` :314, `Slot` :323) | rl-e0 |
| A-7 | **`mineworld validate worlds/lakeside --packs …` reads only that the file exists** (`resolve.rs:151` → `require_for_world`). The content is checked by `mineworld packs validate <pack-dir>` (`tools/cli/src/packs.rs:260`, presentation branch :287–301 → `far_field::check_pack`), by the clients' loaders, and by the `packages` tests | rl-e0 |
| A-8 | AC-FF-4's test `two_d_and_three_d_water_agree` (`packages/tests/far_field.rs:214–251`) zips each world's `water` arrays in order and requires equal count and `name`, `from_deg`/`to_deg` within 1°, `shore_m` within 1 m, equal `horizon`. **It does not compare `kind`, `level_m` or `colour`.** :250 `assert_eq!(compared, 2, "social-cafe and lakeside have both")` | rl-e0 |
| A-9 | The other far-field tests: `every_planted_far_field_is_refused_by_its_code_and_names_the_fault` :53, `every_corpus_file_gives_the_code_its_name_says` :118, `the_shipped_shape_of_a_3d_and_a_2d_far_field_is_accepted` :145 (corpus `packages/tests/fixtures/far_field/*.yaml`) | rl-e0 |
| A-10 | The rendered checks: `tools/cli/tests/client_far_field.rs` (all `#[ignore]`): `three_d_far_field_closes_every_open_sector` :127 (social-cafe only; the 3D slice hard-codes `const WORLD_KEY := "social-cafe"`, `slice_main.gd:80`); `two_d_clear_colour_is_never_visible` :157 loops over worlds and runs Lakeside with `--packs entities --packs presentation/mineworld-default` (:167–175); `the_two_validators_agree` :212 | rl-e0 |
| A-11 | CI placement (RL-e0 ledger, ruling 2026-10-17, Doc:1468–1472): the 3D check runs per PR on Linux (`farfield` job, 90 min), nightly on Windows, on macOS only locally; the 2D check and the validator agreement on all three OSes | rl-e0 Doc |
| A-12 | RL-b C8's method: elevation angle `(h − h_eye − d²·(1 − k)/(2R)) / d`, R = 6 371 km, k = 0.13 (Gauss, via Hirt et al. 2010, JGR 115 D21102); haversine bearing and distance. The summit table (re-read 2026-10-09) includes "Cuyamaca Peak 1 985 m (NGS 'Cuyamaca reset')". The reverted code `58bc0c4:clients/3d-spike/scripts/slice/skyline.gd` (`SUMMITS` :64–84, Cuyamaca :77 at 32.946743453, −116.606723761; `bearing_distance` :123; `elevation_deg` :136). Live on RL-e0: `far_field_view.gd:41–43`, `_crest` :272–289 | `pr-rl-b-3d-budget.md:204–205, :619–630`; `58bc0c4` |
| A-13 | The V-gate rule: "A later PR that changes a far field compares its shots with these by `frame_diff.gd` and presents both to the operator" | Doc:700–702 |
| A-14 | Step-29 §6.7: `filled` = `{"state":"filled","place","bearing_deg","distance_m">0,"footprint_m">0,"source"}`; may lie within `reach_m`; massing keeps clear of its footprint; a town fills both slots with `station.place == settlement.station`, `landmark.place == settlement.landmark`, else `FAR_FIELD_SLOT_TOWN`; `kind: none` refuses `filled` | `step-29-station-landmark.md:499–531` |
| A-15 | S29-a rev. 2 (DRAFT): Lakeside's halt is `station` (`station, public`), passage `lane`: here (4000, 400), there (400, 3000), "at the lane's **west** end"; `station-tickets` here (1500, 400), there (6000, 400); `station-platform` here (0, 3600), there (4000, 4600), "≈ 40 × 4 m, origin on the track edge, mid-length"; `pier` gains `landmark`; `settlement: { kind: town, station: station, landmark: pier }` | `pr-s29a-town-contract.md:410–429` |
| A-16 | Lakeside's places (semantic, mm, x east, y north): lane ≈ 30 × 6 m, origin SW corner, no passages; north side: cottages (3000, 5800), boathouse (9000, 5800), bakery (15000, 5800), square (21000, 5800); south side: shore (12000, 200), pier (18000, 200); the pier ≈ 4 × 20 m running south, foot on its north edge (here (2000, 19800)) | `worlds/lakeside/places/*.yaml` |
| A-17 | The scene frame (step-33 SD-S33-1): metres, +x east, +z south, y up, y = 0 the lane's surface, the lane's centre the origin. A lane point (x, y) mm is at scene (x/1000 − 15, ·, 3 − y/1000) | step-33 §4.0 |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S33a-1** | The step's C1 asks for "a dated amendment note on RL-e0 App. B.2/B.3 and S29 §6.7". Both notes were already applied by #228 (RL-e0 Doc:1662–1667 on #220, :1401–1406 on `main`; step-29:525–527) | C1 adds only a one-line **landed** note under each ("replaced in S33-a at `<sha>`"), no new amendment text |
| **F-S33a-2** | The step's checkpoint says `mineworld validate worlds/lakeside --packs …` passes. That command reads only the file's presence (A-7); a wrong file passes it | The checkpoint is met **and** strengthened: `mineworld packs validate presentation/mineworld-default/3D` and `…/2D`, and `cargo test -p mineworld-packages --test far_field` are the content gates (§8 AC-a-2) |
| **F-S33a-3** | X-1 as written cannot fire `FAR_FIELD_GAP`: the lake is `closed`, and closed water never counts toward coverage (A-5). Deleting the 3D water sector **alone** is caught by AC-FF-4 (count mismatch, A-8); deleting it from **both** files is caught by nothing before a rendered check (FF-BELOW at VP-L-3) | X-1 is recorded as two plants: X-1a (3D only → `two_d_and_three_d_water_agree` red, here) and X-1b (both → FF-BELOW at VP-L-3, S33-d's; or here if S33-c's stub is merged, AC-a-6). The step's row (§9.4) gets the correction (§11) |
| **F-S33a-4** | AC-FF-4 does not compare `level_m` or `kind` (A-8), yet step §3.4 requires the 2D file to carry "the same lake sector, level, colours". After this PR the lake's level moves from 0 to 1 406 m; a 2D file left at 0 would pass | Bounded addition (§5.6): the test also requires equal `kind` and `level_m` within 0.5 m for every world. Colours are **not** gated (each pack's palette may differ by design; both Lakeside files carry the same values, checked by review). If `social-cafe`'s two files disagree in `kind` or `level_m`, that is a **material stop** (another world's data), not an edit |
| **F-S33a-5** | **The halt's end is not settled.** Step-33 §4.0 and step-29's dated note (step-29:302–305) ask S29-a for the lane's **east** end with a north–south platform ending at a buffer stop near the shore. S29-a rev. 2 (written before #228) puts it at the **west** end (A-15) and gives the platform a frame 40 m along x (east–west) with the track on its south edge, which is the lake's side. The station slot's bearing and the `reach_m` confirmation depend on it | Q-S33a-1 (§14), asked of the primary before S29-a freezes. This PR reads whatever S29-a merges (C0) |
| **F-S33a-6** | No summit data file exists anywhere; the only table is RL-b's (doc and reverted `skyline.gd`, A-12). RL-b C8's method does not read a DEM | §5.3: the method is the summit table **plus** 3DEP point elevations from the USGS Elevation Point Query Service along rays for the near rim and for the visibility check (R-S33-8); the derivation is reproducible offline from a committed cache |
| **F-S33a-7** | RL-e0's 3D view anchors silhouettes to the camera and draws water from `shore_m` to its horizon ribbon (Doc §5.4; `far_field_view.gd` `_build_water` :426). With the near lake built by S33-d to `reach_m`, `shore_m` = `reach_m` = 80 joins the two surfaces | `shore_m` 80 (allowed: `shore_m ≥ reach_m`, A-4) |

### 3.3 What does not exist

No Lakeside location anywhere in data; no `tools/far-field/`; no cached elevation; no 3D client able to draw
Lakeside (A-10); no `filled` slot on `main` (S29-c).

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Decision | Why |
| --- | --- | --- |
| Elevations | **USGS 3DEP through the Elevation Point Query Service** (public, no key, JSON) for ≤ 600 sample points; responses cached in the repository | A DEM tile reader (GDAL/rasterio, ≈ 100 MB of native libraries; or a hand-written TIFF parser) is rejected for 600 points: the service returns exactly the 3DEP value at a coordinate. Declined: SRTM tiles (30 m, older), commercial terrain APIs (keys, terms) |
| The lake outline | **USGS NHD** waterbody polygon for Cuyamaca Reservoir, read once from the National Map's ArcGIS REST query (JSON), cached | the authoritative federal outline; declined: OpenStreetMap (ODbL attribution obligations for a derived file; a second licence for one polygon) |
| Bearing, distance, elevation angle | **RL-b C8's formulas** (A-12), restated in `derive.py` (≈ 40 lines, standard library) | one method in the project; already reviewed. Declined: `pyproj`/`geographiclib` (a dependency for haversine at ≤ 10 km, where the spherical error is < 0.3 % of distance and far below 1° of bearing) |
| Visibility | the IMPROVE network's annual mean standard visual range at the nearest Class I site (Agua Tibia Wilderness, `AGTI1`), read in C2 | a measured, cited range for the region's mountain air. If it cannot be read, RL-e0's 80 km is used and marked "approximate; replace with IMPROVE AGTI1" (I-S33-7) |
| Water colours | sampled in pixels from `06_lake_trail.png` with Godot's `Image` API in a scratch script (not committed); pixel coordinates recorded | ACCEPTANCE §3.1's rule (read references in pixels); no new tool |

No dependency is added to any lockfile.

---

## 5. Design

### 5.1 The far field's origin and frame

The origin is the scene origin of SD-S33-1 (the lane's centre, A-17). True north is scene −z. A semantic point of
place P maps to scene coordinates by 16c's doorway translation (`ARC-45`) through the lane; C2's script carries
the lane frame of A-17 and the passages read at C0 (as constants in `inputs.json`, each with its file and line).

The geographic **site** is the point on the Earth that the origin stands on: a point on Lake Cuyamaca's north
shore, chosen by §5.2's rule. Every bearing and distance in both files is measured from it.

### 5.2 Choosing the site (C2, reviewed by Opus before C3)

1. From the NHD outline, list candidate points 20 m inland of the north shoreline (the lane is ≈ 3–13 m above
   the shore's water line in scene terms; 20 m covers the shore place's 10 m depth plus the lane's half-width),
   every 50 m along the north shore.
2. Keep a candidate if, from it: (a) the lake's surface occupies the bearings 150°–210° continuously out to at
   least 300 m (the pier and the view south lie over water); (b) no land rises between 0 and `reach_m` in the
   sector 120°–240° (the near field is shore and water); (c) Cuyamaca Peak's crest angle exceeds the highest
   3DEP sample on its ray inside 4 km by ≥ 0.3° (the peak is visible: R-S33-8).
3. Of the kept candidates, choose the one whose shoreline runs closest to east–west (the lane runs east–west,
   A-16), ties broken by the larger lake extent along 180°. Record every candidate's numbers in `SOURCE.md`.
4. If no candidate satisfies (a)–(c), **material stop** with the table (the setting ruling would need the
   operator: R-S33-8).

### 5.3 The derivation tool (`tools/far-field/lakeside/`)

- `inputs.json`: `site_rule` (§5.2's numbers), `lane_frame` (A-17), `passages` (C0's values with file:line),
  `summits` (name, latitude, longitude, height_m, layer hint, citation): Cuyamaca Peak (1 985 m, NGS "Cuyamaca
  reset", the RL-b row), Middle Peak, North Peak, Stonewall Peak, and the Laguna and Volcan Mountains' named
  high points, each height and coordinate cited (NGS datasheet, GNIS or USGS 7.5′ quadrangle; Wikipedia only as
  a pointer to one of those), `sample_plan` (bearings every 5°, distances 100, 200, 400, 800, 1 600, 3 200 m for
  the near rim; the peak rays of §5.2 (c) at 250 m steps to 4 km).
- `derive.py` (standard library: `json`, `math`, `urllib.request`), three subcommands:
  - `fetch` — reads EPQS and NHD for every point and query of `inputs.json` that is not in `cache/`, writes the
    cache (sorted keys, one entry per `lat,lon` rounded to 1e-6). The only command that uses the network.
  - `site` — §5.2 from the cache; prints the candidate table.
  - `profile` — from the cache and the chosen site: each summit's bearing, distance, crest angle (A-12, eye height
    1.66 m above `datum_m`); the near-rim crest per 5° sector (the maximum angle sample), its height and
    distance; the lake sector (the bearings whose ray meets NHD water before land, rounded outward to 1°); the
    station and landmark slots (§5.4). Prints the profile table that `SOURCE.md` records, and **writes nothing
    else** (the YAML files are written by hand in C3 from that table).
  - `derive.py profile` run twice gives byte-identical output (no clock, no network).
- `SOURCE.md` (data record): the command lines, the base SHA, the fetch date, each service's URL template as
  used, the candidate table, the profile table, every citation.

### 5.4 The 3D file (`3D/far_field/lakeside.yaml`; format 1; replaces App. B.2)

Values marked *derive* come from `derive.py profile` and are copied verbatim with their source line. Nothing
else is invented.

| Field | Value |
| --- | --- |
| `format`, `scope` | `1`, `{"world": "lakeside"}` |
| `boundary` | `{"kind": "natural", "reach_m": 80, "beyond": "planting"}` (woods in the land sectors; Q-S33-9). C0 confirms 80 contains the pier head (≈ 23 m) and every point of the halt's three floors as S29-a merges them, computed from A-17's frame; if not, raise to the smallest multiple of 10 m that does and record it (Q-S33-9's own condition) |
| `datum_m`, `datum_source` | `1408`; "lake surface 1 406 m (Lake Cuyamaca, full pool) + 2 m, the lane above the water (approximate; S33-b's floors and S33-d's grading fix it)" |
| `sky` | `{"kind": "panorama", "panorama": "qwantani_puresky_2k", "visibility_m": derive, "visibility_source": §4's IMPROVE line, or the approximate fallback}` |
| `foreground` | `{"surface": "dry_grass", "drop_m": derive (the 3DEP fall from the site to the woods' edge beyond `reach_m`, clamped to [0, 500]; expected 2–10), "run_m": derive}` |
| `water` | one body `{"name": "cuyamaca", "kind": "lake", "sector": derive (expected ≈ 120°→240°), "shore_m": 80, "level_m": 1406, "horizon": "closed", "colour": {"deep", "shallow"} sampled from 06 (§4), "source": "Lake Cuyamaca (Cuyamaca Reservoir), 45 ha, surface 1 406 m; outline from USGS NHD; read <date>"}` |
| `terrain_silhouettes` | **near** `far-shore`: the lake sector's far-shore crest points (from the near-rim samples inside the lake sector), `distance_m` > 80; **near** `rises`: the land sectors' near rim; **mid** `cuyamaca-peaks`: Cuyamaca, Middle, North Peak and Stonewall Peak as profile points with intervening rim samples; **far** `laguna`, `volcan` only if `profile` shows their crest angle above the near rim on their bearings (else omitted and recorded). Every profile point's `source` names its sample (EPQS 3DEP point or the summit's citation). Coverage: silhouettes + skyline cover [0, 360) (closed water does not count, A-5) |
| `skyline` | `sectors`: a single low-density massing sector only where real buildings stand within view (the lake's store and restaurant site, if the profile shows it visible; else no massing sector and the silhouettes alone cover the circle); `seed` 7; `slots`: `station` `{"state": "filled", "place": "station", "bearing_deg", "distance_m", "footprint_m", "source"}` and `landmark` `{"state": "filled", "place": "pier", …}` from §5.1's translation of the halt's and the pier's floor centres (footprint: the longest side); the station's `source` carries Q-S33-2's departure: "the San Diego, Cuyamaca and Eastern Railway (1889–1916) ended at Foster; the line is carried to the lake here, a stated departure from history (Q-S33-2)" |

### 5.5 The 2D file (`2D/far_field/lakeside.yaml`; replaces App. B.3)

`boundary` as the 3D file's; `water` **identical** to the 3D water body (name, kind, sector, `shore_m`, `level_m`,
horizon, colours, source); `surround` `{"extent_m": 400, "base": "land", "haze": {"colour": re-picked from 06's
far-shore haze in pixels, "from_m": 60, "to_m": 400}}`.

### 5.6 AC-FF-4, strengthened (F-S33a-4)

In `two_d_and_three_d_water_agree`, per zipped water body, add: `kind` equal; `level_m` within 0.5 m. The failure
message names the world, the body and the field. `compared == 2` stays.

### 5.7 Records and spec text

- `ARC-143` (C1): Lakeside's setting is Lake Cuyamaca's north shore, presentation only (Q-S33-1, Q-S33-11); the
  far fields are derived by §5.3's reproducible method from USGS 3DEP and NHD and RL-b C8's formulas; the lake
  lies south in both packs (F-S33-2); the halt is a stated departure from history (Q-S33-2); references govern
  architecture, materials, light and mood, the setting governs terrain, water, vegetation and the far field
  (step §3.3).
- `docs/ART_DIRECTION.md` §4: under "lakeside towns", the setting rule of step §3.3 in two sentences, linking
  `ARC-143`.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-a-1 (I-S33-1) | No path under `kernel/`, `contracts/`, `systems/`, `server/`, `worlds/` changes | C5 diff scan |
| I-a-2 | Both Lakeside files pass `packs validate` and every `packages` far-field test; AC-FF-4 (as strengthened) holds for both worlds | AC-a-2, AC-a-3 |
| I-a-3 (I-S33-7) | Every number in both files has a `source` naming a dataset, a citation or "approximate; <who replaces it>" | AC-a-4 review; `FAR_FIELD_UNCITED` |
| I-a-4 | The lake lies south: the water sector contains 180° in both files | AC-a-5 |
| I-a-5 | `derive.py profile` is offline and deterministic; the files equal its table | AC-a-4 |
| I-a-6 | Social Café's far fields and every other world's are byte-unchanged | C5 diff scan |
| I-a-7 | No schema change: no edit under `packages/src/` | C5 diff scan |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | the far-field validator (schema, ranges, citations, coverage, slots) over the shipped files | `mineworld packs validate`; `packages` tests (CI `core`) |
| Unit (Rust) | AC-FF-4's cross-pack agreement, now with `kind` and `level_m` (§5.6) | `packages/tests/far_field.rs` |
| Tool determinism | `derive.py profile` twice → identical output | C2 evidence (recorded command and `cmp`) |
| Rendered (Godot) | 2D: `two_d_clear_colour_is_never_visible` on Lakeside (already in the loop, A-10). 3D: only if S33-c is merged (AC-a-6) | CI `farfield` (Linux, per PR) and nightly; local one run each, timings INCONCLUSIVE |
| Review (Opus) | the site and the profile against the map (USGS topographic quadrangle "Cuyamaca Peak"), the colours against 06 | C2/C3 review items |
| Gate 1 / Gate 2 | NOT REQUIRED: no model, no lifecycle property | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-a-1 | `target/debug/mineworld validate worlds/lakeside --packs entities --packs presentation/mineworld-default` exits 0 | §12.1 checkpoint |
| AC-a-2 | `mineworld packs validate presentation/mineworld-default/3D` and `…/2D` exit 0; `cargo test -p mineworld-packages --test far_field` passes, every test named in A-8/A-9 | F-S33a-2 |
| AC-a-3 | `two_d_and_three_d_water_agree` (strengthened) passes; `compared == 2` | AC-FF-4 |
| AC-a-4 | `derive.py profile` run twice from the committed cache is byte-identical, and every value in both files equals its table row (reviewed line by line, recorded) | I-S33-7 |
| AC-a-5 | The water sector contains 180° in both files; Cuyamaca Peak is a profile point within 1° of its computed bearing, at its cited height | F-S33-2; §3.4 |
| AC-a-6 | 3D rendered: **if S33-c is merged at C4**, `./mineworld-slice --world=lakeside --farfield` on the stub depiction prints its verdict and the shots go to the operator (V-gate), recorded, not gated (the stub has no near field; AC-S33-1 is S33-d's). Otherwise N/A with that reason | §12.1 checkpoint |
| AC-a-7 | 2D rendered: `two_d_clear_colour_is_never_visible` passes for Lakeside on CI | AC-FF-3 (RL-e0) |
| AC-a-8 | X-1a and X-2 fail by name (§9 C4); X-1b recorded as S33-d's (or measured under AC-a-6) | AC-S33-10 (X-1, X-2) |
| AC-a-9 | The diff touches only §2.1's paths | I-a-1, I-a-6, I-a-7 |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s33a`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites; re-read every file you will edit, in full.
2. Make the numbered steps in order; nothing from a later commit.
3. Run the commit's validation commands; record command, count, wall time, PASS / FAIL / INCONCLUSIVE.
4. For each listed mutation: plant it, run the named check, record it red by name, restore from a byte copy
   (`cmp` clean), run green.
5. Doc checks: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`; `python3
   scripts/ci_layer.py docs`. **R-LOAD-1:** never `cargo test --workspace`; only `cargo test -p
   mineworld-packages --test far_field`; Godot runs one at a time.
6. Answer the commit's review questions in §15.
7. `git status --short`, `git diff --stat`: every path must be in the commit's **Files**.
8. Update §15 and the handoff in the same commit.
9. Stage by explicit path (never `git add -A`); commit with the message given plus the attribution line. No
   heredoc writes, no `sed -i`, no `python3 -c`.
10. On anything outside the plan: stop and report (§12 material stops).

#### C0 — Base, prerequisites and ledger (Markdown only)

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s33a.md` (new).
- Steps:
  1. Record the base commit. Confirm on it: RL-e0's files and tests (A-1, A-6, A-8) exist; `settlement:` and the
     halt's three places exist in `worlds/lakeside` (S29-a); `filled` and `FAR_FIELD_SLOT_TOWN` exist in
     `far_field_schema.rs` and `far_field.gd` (S29-c). Any missing → material stop.
  2. Record the halt's merged passages and the platform's frame (A-15 as merged) and Q-S33a-1's ruling; compute
     every halt floor corner and the pier head in scene metres (A-17); confirm `reach_m` 80 contains them, or
     record the raised value (§5.4).
  3. Re-read A-1…A-17 on the base; record moved lines (bounded) or changed statements (stop).
  4. Record whether S33-c has merged (AC-a-6).
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: every start condition holds, with its evidence.
- **Commit message:** `docs(plan): S33-a — open the ledger at the base`

#### C1 — `ARC-143`, the art-direction rule, the landed notes' placeholders

- **Files:** `docs/DECISIONS.md`, `docs/ART_DIRECTION.md`, `.structured-coding/plans/mvp1/pr-rl-e0-far-field-mandatory.md`
  (App. B.2/B.3: one dated line each), `.structured-coding/plans/mvp0/step-29-station-landmark.md` (§6.7: one
  dated line), this file.
- Steps: §5.7; the landed lines read "*(2026-MM-DD, S33-a)* Replaced: see `presentation/mineworld-default/{3D,2D}/far_field/lakeside.yaml` and `ARC-143`."
  (the SHA is filled at C5).
- Validation: doc scripts.
- [ ] Implementation. [ ] Validation. [ ] Review: `ARC-143` states no number that §5.4 has not derived; nothing
  in it binds S33-d's art.
- **Commit message:** `docs: ARC-143 — Lakeside is set at Lake Cuyamaca, presentation only (S33-a)`

#### C2 — The derivation tool, the cached reads, the site and the profile

- **Files:** `tools/far-field/lakeside/{derive.py, inputs.json, cache/epqs.json, cache/nhd.json, SOURCE.md}`, this
  file.
- Steps:
  1. `inputs.json` per §5.3 (each summit cited; the passages from C0 with file:line).
  2. `derive.py` per §5.3; RL-b's formulas restated with A-12's citation in its docstring.
  3. `python3 tools/far-field/lakeside/derive.py fetch` (the only network step; record the call count and date);
     `… site`; choose per §5.2; `… profile`.
  4. IMPROVE AGTI1 read (§4); the two colours sampled from 06 (§4) — values and pixel coordinates into `SOURCE.md`.
  5. `SOURCE.md` per §5.3.
- Validation: `derive.py profile` twice, `cmp` of the two outputs (AC-a-4's first half); `uv run --locked ruff
  check tools/far-field` if the workspace's ruff configuration covers `tools/` (else N/A with that reason).
- [ ] Implementation: steps 1–5. [ ] Validation. [ ] Review (**Opus**, before C3): the site against the USGS
  "Cuyamaca Peak" 7.5′ quadrangle; the lake sector against the NHD outline; Cuyamaca Peak's bearing against
  step §3.2's ≈ 209° at ≈ 5 km from the lake's named coordinate (the site differs, so a difference of a few
  degrees is expected and explained); no profile point inside `reach_m`.
- **Commit message:** `feat(presentation): Lakeside's site and far-field profile from USGS 3DEP and NHD (S33-a)`

#### C3 — The two Lakeside far fields

- **Files:** `presentation/mineworld-default/3D/far_field/lakeside.yaml`,
  `presentation/mineworld-default/2D/far_field/lakeside.yaml`, this file.
- Steps: §5.4 and §5.5, from `SOURCE.md`'s table, value by value.
- Validation: `cargo build -p mineworld-cli`; AC-a-1; AC-a-2's two `packs validate` commands; `cargo test -p
  mineworld-packages --test far_field` (before §5.6: the existing tests pass with the new files).
- [ ] Implementation. [ ] Validation. [ ] Review: every value traced to a `SOURCE.md` row (record the trace
  table); the station `source` carries Q-S33-2's sentence; the two water bodies are byte-identical.
- **Commit message:** `feat(presentation): Lakeside's far fields — Lake Cuyamaca, the lake to the south (S33-a)`

#### C4 — AC-FF-4 strengthened; the mutations

- **Files:** `packages/tests/far_field.rs`, this file.
- Steps: §5.6.
- Validation: `cargo test -p mineworld-packages --test far_field`. Mutations: **X-1a** (delete the 3D water
  body) → `two_d_and_three_d_water_agree` red (count); **X-2** (3D sector moved to 300→60 only) → the same test
  red (sector); **M-a-1** (2D `level_m` back to 0) → red (`level_m`, the new comparison); each restored by `cmp`.
  AC-a-6 if S33-c is merged; AC-a-7's 2D check once locally (INCONCLUSIVE timing; CI gates).
- [ ] Implementation. [ ] Validation: the test and the three mutations. [ ] Review: the new comparison passes
  for `social-cafe` without touching its files (else stop, F-S33a-4).
- **Commit message:** `test(packages): the two packs' lakes agree in kind and level too (S33-a)`

#### C5 — Close-out

- **Files:** this file (§15), `handoff-s33a.md`, the two landed notes (the SHA).
- Steps: AC-a-9 (`git diff --name-only origin/main...HEAD`); push `mvp0/pr-s33a`; `gh pr create` with the V-gate
  shots if AC-a-6 applied; CI on the exact head; repair to green.
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S33-a close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-S33d-a-1 (on S33-d).** Build the near lake to `level_m` and `shore_m` of this file; run AC-S33-1 at
  VP-L-1…7 on three OSes; own X-1b (FF-BELOW at VP-L-3 with the water removed from both files).
- **R-S33h-a-1 (on S33-h).** The 2D file's water is the 3D file's; S33-h changes the 2D look, never the lake's data.
- **R-S29a-a-1 (on S29-a, through the primary).** Q-S33a-1's ruling is applied in S29-a's revision before its
  freeze.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-33-lakeside-3d.md` §9.4 X-1 | "must fail with `FAR_FIELD_GAP` …" → X-1a (3D only: AC-FF-4 count) and X-1b (both: FF-BELOW at VP-L-3, S33-d) (F-S33a-3) |
| `step-33-lakeside-3d.md` §12.1 S33-a row | checkpoint: add `packs validate` and the `packages` tests (F-S33a-2); C1's amendment notes are "landed" lines (F-S33a-1) |
| `pr-s29a-town-contract.md` | Q-S33a-1's ruling (the halt's end and the platform's frame) |
| `overall.md` | S33-a frozen / merged; `ARC-143` |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S33-a — Lakeside's setting (Lake Cuyamaca) and far fields
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s33a-setting-far-field.md
RELATED / BINDING DOCS:  step-33-lakeside-3d.md (§§0, 3, 4.0, 4.7, 9, 10, 11, 12, 14, 15, 16);
                         ../mvp1/pr-rl-e0-far-field-mandatory.md (§§5, 6, App. B); step-29-station-landmark.md
                         (§§4.3, 6.7); pr-s29a-town-contract.md (as merged); pr-rl-b-3d-budget.md (C8);
                         docs/MODULE_SPEC.md §6.1.1; docs/ART_DIRECTION.md §4; docs/DECISIONS.md ARC-102…104,
                         ARC-107; CLAUDE.md; docs/ENGINEERING_STANDARDS.md; docs/REUSE_POLICY.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s33a, its own, held by one session
BRANCH:                  mvp0/pr-s33a, created from origin/main
START CONDITION:         Milestone F's checklist handed over; RL-e0 (met: #220, 621ea0e2), S29-a, S29-c
                         merged; Q-S33a-1 ruled (met: 2026-10-18); this design frozen (met: 2026-10-18); the
                         primary session's dispatch                                             (primary)
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–7
FROZEN INVARIANTS:       step-33 I-S33-1, I-S33-4, I-S33-7; §6 I-a-1 … I-a-7; RL-e0 format 1 unchanged;
                         Q-S33-1, -2, -9, -11 as ruled
SEQUENCE:                C0 … C5 (§9)
ALLOWED COMMANDS:        cargo build -p mineworld-cli; cargo test -p mineworld-packages --test far_field;
                         target/debug/mineworld validate / packs validate; python3 tools/far-field/lakeside/
                         derive.py (fetch only in C2); python3 scripts/*; ./mineworld-2d and ./mineworld-slice
                         probe runs, one at a time; godot --headless for the scratch colour sample; git; gh
                         (never merge); mkdir -p; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace (R-LOAD-1);
                         any edit under kernel/, contracts/, systems/, server/, worlds/, packages/src/; another
                         world's far field; any paid service; reading any key or .env
MATERIAL STOPS:          a start condition missing at C0; no site satisfies §5.2 (R-S33-8); reach_m must rise
                         above 80 for a reason other than the halt's platform end; social-cafe fails the
                         strengthened AC-FF-4; a schema change appears necessary; USGS services unreachable for
                         the whole C2 (the cache cannot be built)
PLATFORMS:               the packages tests and the 2D check on Linux, macOS, Windows (CI); the 3D check per
                         RL-e0's ruled placement
VALIDATION BUDGET:       local: the listed targeted commands; ≤ 600 EPQS reads and ≤ 3 NHD queries, once; CI:
                         the PR's runs. Gate 1/2: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s33a.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                                  (source: the operator's freeze of this design, 2026-10-18)
                         semantic commits: authorized                   (working rules §14)
                         branch push: authorized                        (working rules §21)
                         PR creation / update: authorized               (working rules §21)
                         CI repair to review readiness: authorized      (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized — only with the operator's explicit authorization (§22)
POST-MERGE SYNC OWNER:   primary: step-33, overall.md, step-29, RL-e0 design; implementation session: this ledger
STOP CONDITION:          S33-a READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-a-1 | From the north shore the near ridges hide Cuyamaca Peak (R-S33-8) | §5.2 (c) is a site criterion; the far field shows what is visible; no site → material stop |
| R-a-2 | USGS services change or are slow | one fetch, cached; `fetch` resumes from the cache; profile is offline |
| R-a-3 | The lake's 45 ha make the far shore close (a few hundred metres), so the near silhouette dominates the south | correct and intended; S33-d's near lake joins at `shore_m` 80 |
| R-a-4 | S29-a's halt lands at the west end with an east–west platform over the lake's side | Q-S33a-1 before S29-a's freeze; this PR reads what merges and fills the slot from it either way |
| R-a-5 | The strengthened AC-FF-4 trips on social-cafe | material stop, not an edit (F-S33a-4) |
| R-a-6 | A Haiku implementer misreads the site rule | the Opus review gates C3; the candidate table is recorded |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S33a-1 [primary / operator — the halt's position, owned by S29-a]** | Step-33 §4.0 and step-29's dated note ask S29-a to put Lakeside's halt at the lane's **east** end, with a north–south platform ending at a buffer stop near the shore. S29-a's revision 2 (written before that note) puts it at the **west** end, with the platform's frame 40 m east–west and the track on its south edge, the lake's side (F-S33a-5). Which does S29-a's next revision carry? | **East end, north–south**, as the approved step asks: `station` joined to the lane's east edge (lane point (29 600, 3 000), 400 mm inside the edge, 12d's inset); `station-platform` written 4 m (x) × 40 m (y) with the track on its west edge and the buffer stop at its south end. A terminus comes in from inland and stops short of the water; at the west end the track would run along the shore beside the cottages, and an east–west platform puts the line between the village and the lake. The far field, S33-b's bodies and S33-e follow whichever S29-a merges. **Ruled 2026-10-18 (operator): east end, north–south, as recommended — the small station at the lane's east end, a north–south platform with its end against the shore; S29-a's next revision carries it (R-S29a-a-1).** |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C5 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; X-1a, X-2, M-a-1 as §9 C4 lists them)

### Deviations and findings

F-S33a-1 … F-S33a-7 (§3.2), recorded at design time.

- *(2026-10-19, lane lead)* **C2 prepared ahead of C0**, on base `a1e05874`; C0 re-reads the halt's passages
  (provisional in `inputs.json`) when S29-a merges, and re-runs `profile` offline. C2's Opus review has not run.
  Pending for it: **(d)**, a bounded addition to §5.2 — no NHD water within 50 m on 270°→90°, because without it
  rule 3 chose a spit with water 40 m north of the village (`tools/far-field/lakeside/SOURCE.md`); the
  visibility fallback (80 km, approximate) because IMPROVE AGTI1's annual mean could not be read; Stonewall Peak
  found from a pointer coordinate because GNIS lists no summit of that name; Volcan Mountain omitted for the
  same reason. One path beyond §5.3's list: `cache/gnis.json`, the cached GNIS summit reads (bounded; AC-a-9's
  diff scan must list it).
