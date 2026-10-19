# PR RL-e0b — Far-field polish: the five defects RL-e0's review logged, each made checkable

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: every §15 question as recommended)

```text
Design revision:        revision 1 (2026-10-18), FROZEN 2026-10-18
Author:                 far-field polish design agent, worktree
                        /Users/yuema137/mineworld-worktrees/design-ff-polish, branch docs/pr-design-ff-polish,
                        planning base origin/main @ b85f2804 (RL-e0 #220 merged as 621ea0e2)
Approved by / evidence: the operator, 2026-10-18, in the primary session: every §15 question as recommended. Release scope ruled by the operator 2026-10-18 (relayed by the primary
                        session): "these must be fixed before release"; frozen 2026-10-18 with Q-1 … Q-7 as recommended
Implementation base:    origin/main @ 94183574 (recorded at C0, 2026-10-18)
Start condition:        freeze of this document; the primary session's dispatch
Execution contract:     §13 (template filled where the design can fill it; lines marked primary are the
                        primary session's at freeze)
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**Effort:** `mvp1` · **Step:** [`step-22-realism.md`](step-22-realism.md) (S22), a new row **RL-e0b** after
RL-e0 and before RL-e (§14 proposes the step edit) · **Runs as:** a release-blocking visual-track PR (the
operator's ruling of 2026-10-18 puts it in release scope) · **Parent PR:**
[`pr-rl-e0-far-field-mandatory.md`](pr-rl-e0-far-field-mandatory.md) (merged, #220; its §§5.4, 6, 7, 8 and
its ledger §18 are the base this design changes) · **Companion record:**
[`inv-f13c-impl-5-software-renderers.md`](inv-f13c-impl-5-software-renderers.md) (the investigation of why the
3D slice is slow or broken on hosted runners; this PR's CI cost depends on it, §8.2).

**Ids.** Within this file: D-n (the review's defects), A-n (audit anchors), SD-n (design decisions), FF-x
(new criteria of the rendered check), X-n (planted mutations, continuing RL-e0's X-1 … X-5), I-n
(invariants), AC-n (acceptance), Q-n (questions), R-n (risks). One decision record is needed; its number is
the primary session's (assigned at freeze: `ARC-290`, §11).

---

## 0. Binding inputs

### 0.1 The review's defects (logged on RL-e0, #220; relayed 2026-10-18)

| Id | Defect, as logged | Seen in |
| --- | --- | --- |
| **D-1** | From a high viewpoint, the sea covers Coronado | `clients/3d-spike/shots/social-cafe/farfield/ff_3_south_aerial.jpg` (Coronado absent; the sea runs to the horizon across 195°–245°) |
| **D-2** | Point Loma sits too low | `ff_5_west_aerial.jpg` (from 40 m Point Loma is a sliver on the sea horizon) |
| **D-3** | Colour-band seams show hard vertical lines | `ff_3` (x ≈ 90 and ≈ 410 of 960), `ff_5` (x ≈ 700 and ≈ 865), `ff_7_east_street_end.jpg` (x ≈ 865) |
| **D-4** | The distant mesas are untextured flat slabs | `ff_4_east_aerial.jpg`, `ff_7` (one olive colour across the whole band) |
| **D-5** | From above, an empty grass ring is visible outside the town | `ff_3`, `ff_4`, `ff_5` (dry grass from the street's 60 m disc to the massing at 250 m) |

The operator's ruling with them: the player cannot reach 40 m today, **but these must be fixed before
release**. The rendered check must be able to see each one (task statement: "add assertions for each defect,
and plant mutations").

### 0.2 Rulings this PR carries unchanged

| Source | Ruling | Effect here |
| --- | --- | --- |
| RL-e0 §0.1 (operator, 2026-10-15) | Every presented world closes every view on a far field | Unchanged; this PR only corrects how the far field is drawn and checked |
| RL-e0 Q-FF-1 (operator) | FF-BELOW 90 %, FF-ABOVE 25 %, seven viewpoints, 8/255 | Unchanged and still run; the new criteria are added beside them (§6) |
| RL-e0 Q-FF-2 (operator) | The bay: `water.gdshader`, Fresnel ≤ 0.6, no reflections | Unchanged |
| RL-e0 Q-FF-6 (operator) | Low-rise massing, 6–14 m | The ring fill (SD-5) keeps the same palette and heights, capped lower only where a sightline needs it |
| RL-e0 SD-FF-3, A-5 | `CameraRig.FAR` = 1 600 m must not change | Unchanged (SD-1 keeps every proxy inside it) |
| RL-e0 SD-FF-4, I-FF-8 | No asserted winding; visibility proven by rendering; X-1 must fail | Unchanged; X-1 is re-run (AC-8) |
| RL-e0 R-FF-1 ruling (b), then (a); primary 2026-10-17 | Check frames with shadows, SSAO and glow off; the 3D check runs per PR on Linux, nightly on Windows, locally on macOS | Unchanged. The added renders' cost is bounded in §8.2 |
| RL-e0 §5.4 "no invented relief" | Crest heights are interpolated between cited profile points, never invented | Kept for the **crest**. The texture of SD-4 changes colour inside the band, never its outline (I-3) |
| RL-e0 I-FF-3 | The far field is decoration beyond `reach_m`, never reachable, nothing suggests an approach | The ring fill starts beyond `reach_m` and is massing, not paths (I-5) |
| Standing rule "Realistic defaults" | Physical values follow the real world and cite a source | The band-end overlaps of §5.6 are marked approximate, as RL-e0's were |
| Standing rule "Local load: CI is the gate" (R-LOAD-1) | No local full-workspace runs under load | Local Godot runs only through `with-slot.sh`; CI on the exact head gates |

### 0.3 Not in scope

| Item | Where |
| --- | --- |
| Real 3DEP terrain and a terrain shader with splats | RL-e (unchanged) |
| A reachable 40 m viewpoint (a hill, a tower, a ride's window) | S21 / later; this PR fixes what such a view would show |
| Lakeside's far field in 3D | S33-a (content), S33-d (drawing). They inherit SD-1 … SD-5 through `SliceFarFieldView` (§9.2) |
| The far-field **schema** (`MODULE_SPEC.md` §6.1.1) | Unchanged: no field is added or removed (SD-6). Q-2 asks whether to add one |
| The slowness of the slice on hosted software renderers | The companion investigation and its fix PR; this PR only bounds its own added cost (§8.2) |

---

## 1. Goal

Make the far field read right from every height the review looked from, and make each of the five defects a
**named failure of the rendered check**: nearer land is drawn over farther water (D-1); every band's crest
stands at its true angle from any camera height (D-2); no band or water edge ends in a vertical cut (D-3);
near bands carry surface texture whose contrast fades with distance as haze does (D-4); and the ground
between the street's boundary and the distant town is built on, with roofs kept below the sightlines that
show the far field from the street (D-5). The check gains an **ID render** — every far-field part drawn in a
flat identifying colour — from which five new criteria are computed against the far-field file's own numbers,
not against the view's code.

---

## 2. Audit (`origin/main @ b85f2804`, 2026-10-18)

### 2.1 Source anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | Proxy distances are per **layer**: `PROXY_M = {near 1480, far … 1560}`; `OUTER_M` 1 450; `SEA_RIBBON_M` 1 400 | `clients/3d-spike/scripts/slice/far_field_view.gd:35-39` | The sea ribbon (1 400 m) is nearer than **every** silhouette (≥ 1 480 m), so it is drawn over any land whose angle lies below the sea horizon: **D-1's root cause** |
| A-2 | `follow()` moves the silhouette root to `(x, rise, z)` and each **layer** mesh by `−rise · proxy / median(distance of every profile point in that layer)` | `far_field_view.gd:115-129`, `_build_silhouettes` :230-247 | One correction per layer, not per column: a band whose distance differs from its layer's median is drawn at the wrong angle once the camera rises. **D-2's root cause** (table in §2.2) |
| A-3 | A band's column is a flat quad from its crest to `base_y`; the first and last column of a band stand at full height; colour per column is `haze(albedo, horizon(az))` with the horizon colour cached per **whole degree** | `_ribbon_column` :251-266, `_horizon_sampler` :304-339 | A band ends in a vertical edge from crest to base (**D-3**), and its colour is uniform inside the band except a 1-degree step and an 8 % vertical darkening (**D-4**) |
| A-4 | Each water body's sea ribbon spans exactly its sector; its ends are vertical; the bay's surface is a sector from `shore_m` to `OUTER_M` with **radial** side edges | `_build_water` :426-474 | From above the origin a radial edge projects to a vertical line, and so does the ribbon's end (**D-3**, the lines at the bay's ends in `ff_3` and `ff_5`) |
| A-5 | The foreground annulus from `reach_m` to 1 450 m is one surface tint; the massing is drawn only where `skyline.sectors[].from_m` begins: **250 m** in both shipped sectors | `_build_foreground` :346-393; `presentation/mineworld-default/3D/far_field/social-cafe.yaml` `skyline.sectors` | 60 m … 250 m is bare dry grass (**D-5**) |
| A-6 | The massing places boxes by seeded rejection sampling, each standing on `_ground_y(r)`, heights uniform in the sector's `[min, max]`; one mesh per 45° | `_build_skyline` :482-535 | The ring fill (SD-5) is a change to this function and to the content, not a new part |
| A-7 | The rendered check renders S, H and E only, at pitch 0, 1 440 × 240, 50° horizontal; FF-BELOW and FF-ABOVE count pixels that **differ**; massing counts as far field | `slice_far_field_check.gd:22-102` | Neither criterion can see D-1 … D-5: covering Coronado with sea, drawing Point Loma low, a vertical cut, a flat band, or a bare ring all leave S ≠ H in the bands. A new instrument is needed (SD-7) |
| A-8 | RL-e0 X-4 (foreground cut at 450 m) survived because the ribbon base and the massing fill the sky under the horizon (RL-e0 F-FF-2) | RL-e0 ledger §18 | Confirms A-7: the existing criteria judge presence below and above the horizon, not position, order or surface |
| A-9 | The band list of `social-cafe.yaml`: `mesas` near 315→135; `soledad` mid 320→340; `east-mid` mid 30→125; `east-far` far 55→120; `south-hills` mid 125→240; `coronado` near 195→245; `point-loma` near 235→315; water `san-diego-bay` 170→300, shore 1 000 m, closed | the file | Abutting ends without overlap: `point-loma`/`mesas` at 315°, `east-mid`/`south-hills` at 125°. A taper (SD-3) would open a notch there unless the content overlaps (§5.6) |
| A-10 | The check's run on hosted runners: Linux per-PR `farfield` job 46.9 min for the 3D check (limit 90); Windows nightly `farfield-windows` 40.4 min (2 427 s, run 39106087666), limit 120; macOS locally (18 s) | RL-e0 ledger E-FF-CI-2, F-FF-CI-3; job 117613487069 | §8.2's budget for the added renders |
| A-11 | `SliceFarFieldView` is the class S33-d will draw Lakeside with (S33-a F-S33a-7 names `_build_water`) | `.structured-coding/plans/mvp0/pr-s33a-setting-far-field.md` §3.2 | Every change here is general (no `social-cafe` constant in the view); S33-d inherits it |

### 2.2 The numbers behind D-1 and D-2 (VP-high, the review's 40 m)

The camera at VP-high is 40.0 m above the street's scene origin: `rise = 40.0 − EYE0 = 38.2 m`, eye 58 m above
sea level (datum 18 m). True elevation angle `α = (h − h_eye − d²(1 − k)/(2R)) / d` (RL-e0 §5.4; R = 6 371 km,
k = 0.13). Sea horizon dip `√(2 h_eye / R)` = 0.245°; sea horizon distance `√(2 R h_eye)` ≈ 27.2 km.

| Band point | h, d | true α at VP-high | drawn today | error |
| --- | --- | --- | --- | --- |
| point-loma 249° | 126 m, 7 900 m | **+0.462°** | −0.135° (layer `near` median d = 2 500 m: drop `38.2 · 1480 / 2500` = 22.6 m at the proxy) | **0.60° low** (16 rows of the check frame) |
| coronado 220° | 18 m, 2 500 m | **−0.927°** (below the sea horizon, seen against the water) | −0.927°, but behind the sea ribbon (1 400 m < 1 480 m), which fills everything below −0.245° | **hidden** |
| mesas 45° | 105 m, 2 000 m | +1.339° | +1.558° | 0.22° high |

At street eye height (`rise` = 0) every band is drawn at its true angle, and Coronado (−0.05°) stands above the
sea horizon (−0.143°): the defects appear only as the camera rises, which is why the street-level shots and
the frozen check (eye level plus one 40 m viewpoint judged only for presence) did not show them.

### 2.3 What does not exist

No per-column parallax; no depth order by real distance; no band-end treatment; no surface texture on a band;
no massing inside 250 m; no rendered criterion for position, order, seams, texture or coverage of the ground.

---

## 3. Reuse, in both directions

| Need | Choice | Why |
| --- | --- | --- |
| Per-vertex parallax and texture on the bands | **Build** one small spatial shader `shaders/far_silhouette.gdshader` (unshaded, fog off, vertex colour) | `StandardMaterial3D` has no per-vertex parallax uniform; a 60-line shader beside the existing `water.gdshader` is the engine's own mechanism, not a dependency |
| Noise for the texture | Engine built-in value noise written in the shader (the same hash `water.gdshader` uses), no texture | No asset to import or license; deterministic on every renderer |
| Identifying parts in a frame | **Engine built-in** `material_override` swapped to flat unshaded materials for one render | The check already toggles visibility per part; swapping a material is the same mechanism |
| Ground radius of a pixel (FF-RING) | Ray–ground intersection in GDScript against `_ground_y(r)` (two fixed-point iterations) | A few lines; no physics query (the far field has no colliders, RL-e0 I-FF-3) |
| Terrain texture from data (NLCD land cover, a splat shader) | **Not now**: RL-e | The wrong-wheel half of the question: forcing a terrain system into a placeholder far field |

No dependency is adopted or rejected, so no `DEP` record is needed.

---

## 4. The fixes, one per defect

### 4.1 D-1 — nearer land over farther water: depth follows real distance (SD-1)

**SD-1.** Every camera-anchored far-field column is placed at a proxy radius that is a monotonic function of
its **real** distance, and the sea ribbon at the proxy of the **sea horizon's** distance from the current eye,
so the depth test orders them as reality does:

```text
P(d) = P_MIN + (P_MAX − P_MIN) · clamp( ln(d / D_LO) / ln(D_HI / D_LO), 0, 1 )
P_MIN 1 460 m (beyond OUTER_M 1 450)   P_MAX 1 590 m (inside CameraRig.FAR 1 600)
D_LO 1 500 m                           D_HI 120 000 m
sea ribbon: P(d_h), d_h = √(2 · R · max(cam_y + datum − level, 0.01)), recomputed in follow()
```

A band column at real distance d is built at radius `P(d)` with its crest at `EYE0 + P(d) · tan α₀` (α₀ its
angle from the street's eye), so its angle is unchanged; only its depth changes. Coronado (2.5 km) is then in
front of the sea ribbon from VP-high (27 km) and is drawn against the water, as it is seen from a hill;
south-hills at 26 km lie in front of the ribbon from VP-high and behind it from the street (15.9 km), as in
reality. World-anchored parts (foreground, water surface, massing) stay inside 1 450 m, in front of every
proxy. `PROXY_M` and `SEA_RIBBON_M` are deleted. *Rejected:* moving the ribbon behind all bands (it would draw
far bands' bases over the sea below the horizon, e.g. south-hills from the street); per-band draw order with
`render_priority` (it ignores depth and breaks where a near band and the sea overlap only partly).

### 4.2 D-2 — every band at its true angle from any height: parallax per vertex (SD-2)

**SD-2.** The silhouette root follows the camera in x and z and **not** in y; each vertex carries `P(d)/d` for
its own column (in `UV2.x`), and the shader adds `rise · (1 − P(d)/d)` to `VERTEX.y`, `rise = cam_y − EYE0`
(one uniform per material, set in `follow()`). This is the small-angle form of the true change of elevation
angle with eye height, `Δα = −rise / d`, applied per column instead of per layer. The per-layer median of A-2
and its node offset are deleted. Point Loma at VP-high is then drawn at +0.462° (§2.2), within the rows FF-CREST
allows (§6.3).

### 4.3 D-3 — no vertical cuts: band tapers, a curved shore, ribbon tapers (SD-3)

**SD-3.** Three edges become slopes:

1. **Band ends.** Over the outer `T_BAND` = 3.0° of each end of a band's sector (or a quarter of the sector,
   whichever is less), the crest angle falls smoothly to the sea-level horizon:
   `α_eff(b) = α_hz + (α(b) − α_hz) · smoothstep(0, T_BAND, e(b))`, where `e(b)` is the angular distance from
   `b` to the nearer end of the sector and `α_hz = −√(2 (datum + EYE0) / R)`. Columns are emitted at
   `COLUMNS_PER_DEG` = 2 as today, so the descent spans at least six columns.
2. **The bay's sides.** The water's near edge is no longer radial at the sector's ends: over the outer
   `T_WATER` = 4.0° of each end, `shore(b) = shore_m + (OUTER_M − shore_m) · (1 − smoothstep(0, T_WATER, e(b)))`,
   and the foreground in a water sector ends at `shore(b)` (it reaches `OUTER_M` at the ends). The shore becomes
   a curve that meets the outer edge, so no radial edge remains.
3. **The sea ribbon's ends.** Over the same `T_WATER`, the ribbon's top descends to its base with the same
   smoothstep, so the sea narrows into the headlands instead of stopping in a vertical line.

The horizon colour that each column hazes toward is interpolated between whole degrees (`_horizon_sampler`
returns `lerp(c[floor], c[ceil], frac)`), removing A-3's 1° colour step.

### 4.4 D-4 — near bands textured, contrast fading with haze (SD-4)

**SD-4.** The band shader modulates the vertex colour's luminance by a two-octave value noise in **angular**
coordinates scaled to a ground feature size at the column's real distance:

```text
u = bearing_deg · (π d / 180) / F     v = elevation_deg · (π d / 180) / F      (F = 40 m, then F/4 = 10 m)
n = 0.65 · noise(u · (1, 3)) + 0.35 · noise(4u, 4v)      anisotropic first octave: vertical gullies
L' = L · (1 + A · τ(d) · (n − 0.5)),   A = 0.45,   τ(d) = exp(−2.996 · d / V)
```

`d` and `V` reach the fragment as varyings (`UV2.y` carries d); τ is the same Koschmieder transmittance that
already sets the band's colour (RL-e0 §5.4), so a band 2 km away keeps ~93 % of its texture contrast and one
at 60 km almost none — haze erases texture exactly as it erases colour. The crest is not moved (I-3). *Rejected
for now:* a per-band `cover` field (scrub / urban / forest) — a schema change (Q-2); NLCD-driven materials —
RL-e.

### 4.5 D-5 — the ring between the street and the distant town is built on (SD-5)

**SD-5.** Two changes, content and drawing, with no schema change:

1. **Content.** `social-cafe.yaml`'s two massing sectors start at **70 m** (`reach_m` 60 + 10) instead of 250 m
   (§5.6). Heights and densities are unchanged.
2. **Drawing: the sightline cap.** The view takes a list of **protected eyes** from the scene that builds it
   (the slice passes the street's two ends and the spawn: VP-west, VP-east, VP-spawn of RL-e0 §6.1). A
   candidate box's height is capped so that, from every protected eye, its roof stays at least
   `CAP_MARGIN_DEG` = 0.25° **below what it stands in front of**: the highest far-field crest (`α_eff`) at the
   box's bearing from that eye, or, in a water sector, the angle of the water's near edge (`shore(b)` at
   `level_m`). The cap is computed at build time from the file's numbers. A box whose cap is under 2.5 m is
   built at `max(cap, 1.0)` m (a garden wall, a shed); one whose cap is under 1.0 m is not built.

The ring then reads as town from above, while the parapet end still looks over roofs to the bay and Point
Loma, and the east end over roofs to the mesas (FF-BAY and FF-CREST guard both, §6.3). *Rejected:* filling
only the north and south (the review's aerials show the ring in every direction); a vegetation ring (a new
far-field part and its schema: Q-1 (b)); moving `reach_m` inward (it bounds near construction, RL-e0 A-3).

### 4.6 SD-6 — the schema does not change

Every fix reads fields the schema already has (`distance_m`, `sector`, `shore_m`, `level_m`, `visibility_m`,
`from_m`). `MODULE_SPEC.md` §6.1.1, both validators and the corpus are unchanged; AC-FF-3 (the two validators
agree) is re-run, not rewritten. The **drawing rules** of §§4.1–4.5 are client behaviour and are written into
`docs/VISUAL_SLICE.md` (engine-neutral wording, Appendix A) and the 3D client's header comments.

---

## 5. Code and content

### 5.1 `clients/3d-spike/shaders/far_silhouette.gdshader` (new)

`shader_type spatial; render_mode unshaded, fog_disabled, cull_back;` uniforms `rise` (float), `visibility_m`,
`texture_amp` (default 0.45), `id_mode` (bool), `id_colour` (vec3, source_color). Vertex: `VERTEX.y += rise *
(1.0 - UV2.x)`; pass `UV2.y` (real distance) and the column's bearing and elevation (from the model-space
vertex) to the fragment. Fragment: `id_mode ? id_colour : COLOR.rgb * (1 + texture_amp * τ * (n − 0.5))`.

### 5.2 `clients/3d-spike/scripts/slice/far_field_view.gd`

| Function | Change |
| --- | --- |
| constants | delete `PROXY_M`, `SEA_RIBBON_M`; add `P_MIN`, `P_MAX`, `D_LO`, `D_HI`, `T_BAND`, `T_WATER`, `CAP_MARGIN_DEG`, `CAP_MIN_M` 2.5, `CAP_FLOOR_M` 1.0, each with its reason |
| `build(parent, field)` | gains `protected_eyes: PackedVector3Array = []` (scene positions); stores them |
| `_build_silhouettes` | **one mesh per band** (not per layer) with its own `ShaderMaterial` (so the ID render can colour each band and `rise` is set per material); vertices carry `UV2 = (P(d)/d, d)` |
| `_ribbon_column` | build at `P(d)` per column; apply SD-3.1's `α_eff` |
| `_crest` | unchanged formula; new `_crest_eff(band, bearing)` applies the taper |
| `follow()` | silhouettes root at `(x, 0, z)`; per band material `rise`; sea ribbon root scaled in x/z to `P(d_h)/P_REF` and its height to the dip, as today |
| `_build_foreground`, `_build_water` | `shore(b)` of SD-3.2 replaces `shore_m` in both; the ribbon's columns carry SD-3.3's end taper |
| `_horizon_sampler` | interpolate between whole degrees |
| `_build_skyline` | the cap of SD-5.2 before a box is accepted; boxes keep the seeded order so the layout stays deterministic |
| new `ids()` | returns `[node, id_name]` for every far-field mesh (the check's ID render, §6.2) |

Expected size after the change: ≈ 720 lines (past the ~500-line review mark, under the ~800 strong warning).
If it passes 800, the massing moves to `far_field_massing.gd` (a bounded split, recorded).

### 5.3 `clients/3d-spike/scripts/slice/slice_world.gd`

`SliceFarFieldView.build(g, far_field, PROTECTED_EYES)` with `PROTECTED_EYES` = the three eye positions
(scene metres) of VP-west, VP-east and VP-spawn — the slice's own reachable ends, named where the street is
built.

### 5.4 `clients/3d-spike/scripts/slice/slice_far_field_check.gd`

The criteria of §6. If the file passes ~500 lines, the ID arithmetic moves to `slice_far_field_ids.gd`.

### 5.5 `clients/3d-spike/mineworld-slice`, `README.md`

No new flag: `--farfield` runs the old and the new criteria together; `--farfield-shots` re-shoots §7.

### 5.6 Content: `presentation/mineworld-default/3D/far_field/social-cafe.yaml`

| Change | From | To | `source` addition |
| --- | --- | --- | --- |
| massing sector 300→170 `from_m` | 250 | 70 | "ring from the boundary; heights capped by the slice's sightlines (RL-e0b SD-5)" |
| massing sector 170→300 `from_m` | 250 | 70 | same |
| `point-loma` sector end | 315 | 320, last point `{320, 55, 7 200}` | "end extended to overlap the mesas (RL-e0b SD-3); approximate; replaced by RL-e (USGS 3DEP)" |
| `mesas` sector start | 315 | 310, first point `{310, 68, 2 500}` | same wording |
| `east-mid` sector end | 125 | 130, last point `{130, 340, 20 500}` | same wording |
| `south-hills` sector start | 125 | 120, first point `{120, 360, 24 000}` | same wording |

Every other value is unchanged. `mineworld packs validate presentation/mineworld-default/3D` must still pass
(the overlaps and `from_m` 70 > `reach_m` 60 are inside the schema). The 2D files are unchanged (their water
agrees with the 3D file's, AC-FF-4, which this table does not touch).

---

## 6. The rendered check gains an ID render and five criteria (fixed before any run, `ARC-23`)

### 6.1 Which renders

| Render | Where | Count |
| --- | --- | --- |
| S, H, E (RL-e0, unchanged) | 7 viewpoints × 8 azimuths, pitch 0, 1 440 × 240, 50° horizontal | 120 |
| **I** — far field in ID colours, the slice's own nodes hidden | **VP-west, VP-east, VP-high** × 8 azimuths, pitch 0, same viewport | 24 |
| **G** — ground ID render, the slice shown in its own materials | VP-high × 8 azimuths, **pitch −27°**, 960 × 540, 70° horizontal (vertical ≈ 43°: depressions 5.5°–48.5°, ground radii ≈ 35–415 m) | 8 |

Total **152** renders (+32, +27 %). I and G use the same cost-effects-off state as S/H/E (RL-e0 R-FF-1 (b)).

### 6.2 The ID render

For I and G, the check swaps each far-field mesh's `material_override` (from `SliceFarFieldView.ids()`) to a
flat unshaded, fog-free material in its ID colour — the band shader's `id_mode` for bands — and for the render
only sets the SubViewport's `screen_space_aa` and `msaa_3d` off, `use_debanding` off, and the environment's
tonemapper to linear with exposure 1; everything is restored after. ID colours, sRGB:

```text
near-ground disc  #202020    foreground  #404040    water surface  #0040ff    sea ribbon  #00a0ff
massing           #ff8000    band k (k = 0 … 11, in file order)   #ff00XX, XX = 0x10 + 0x14·k
```

A pixel decodes to the ID whose colour is within 24/255 on every channel; in I, a pixel equal (within 8/255) to
E at that azimuth is **sky**; any other pixel is **unknown**. An I render whose unknown pixels exceed 1 % of
the frame is reported `FAIL farfield-id <VP> <sector>: unreadable (<n> unknown px)` — the instrument cannot
judge, which is a failure, never a pass. In G, the slice is shown, so its pixels are **other** and excluded.

### 6.3 The criteria

Rows and columns map to angles exactly as RL-e0 §6.3 (`elev(row)`, `off(x)`). "Covered" pixels are those where
H ≠ E (something near is drawn there), as in RL-e0. Every expected angle is computed **in the check** from
the far-field file with the formulas of §4 (`α`, `α_eff`, the per-height shift `−rise/d`, the dip, `P` is not
needed), not by calling the view's code.

| Id | Defect | Rule (per viewpoint V ∈ {VP-west, VP-east, VP-high} and sector, unless said) | Pass |
| --- | --- | --- | --- |
| **FF-ORDER** | D-1 | Judged columns: bearing inside a water sector **and** inside band B's sector, with B's distance at that bearing `< d_h(V)` (the sea horizon's distance from V) and B's expected crest row inside the frame and not covered. The pixel two rows below B's expected crest must decode to B or to a nearer part (massing, foreground) — not to water, sea ribbon or a farther band | ≥ 90 % of judged columns; a sector with < 20 judged columns is `none to judge` |
| **FF-CREST** | D-2 (and D-5's sightlines) | Judged columns: every column whose expected top far-field angle `A(b) = max over bands of α_eff(b) − rise/d` lies inside the frame and is not covered within ±3 rows. Measured: the topmost row decoding to any far-field ID. It must decode to a **band** (massing over the crest fails) and lie within **±0.10°** (3 rows) of A(b) | ≥ 95 % of judged columns |
| **FF-SEAM** | D-3 | For each adjacent column pair, the longest vertical run of rows (elevation ≥ `e_h − 1.0°`) where the two columns decode to **different** IDs, neither is massing, neither is covered, and at least one is a band, the sea ribbon or water. A run of **≥ 6 rows** (0.22°) is a seam | **0 seams** in every I render; each seam is printed with its column's bearing and the two IDs |
| **FF-RELIEF** | D-4 | For each band of layer `near` with ≥ 2 000 decoded, uncovered pixels lying ≥ 3 rows below its measured crest: in the S render, the 8-bit luma (Rec. 709) of those pixels, per row, in windows of 29 columns (1°); the mean over windows of the luma's standard deviation | **≥ 2.5 / 255** per band per render |
| **FF-RING** | D-5 | In each G render: each decoded ground pixel (near disc, foreground, massing) is given its ground radius by ray–ground intersection; radii binned in 20 m bins from `reach_m + 20` to 400 m, per 45° sector. A bin with ≥ 200 pixels and < 50 % water | massing ≥ **15 %** of the bin's pixels in every judged bin |
| **FF-BAY** (guard for SD-5) | — | VP-west, azimuths 225° and 270° only: columns whose bearing is inside the water sector, uncovered between the expected shore row and the horizon row | ≥ 90 % of those columns show ≥ 1 row of water or sea ribbon |

**Why these, and why these numbers.** FF-ORDER asks the exact question D-1 answers wrongly: is nearer land in
front of farther water. FF-CREST measures position, which FF-ABOVE never did; ±0.10° is 3 rows, wider than
anti-aliasing (1 row) and the small-angle form's error at 40 m (< 0.01° for d ≥ 1.7 km), and narrower than
D-2's 0.60°. FF-SEAM's 6 rows (0.22°) is far above any cited profile's steepest real slope in the frame (≤ 0.2
rows per column, e.g. Cuyamaca's 0.76° over 9°) and far below a band's full height cut (≥ 8 rows). FF-RELIEF's
2.5/255: today's flat band varies only by the 1° horizon step and the 8 % vertical gradient (≤ 1/255 across a
row window); SD-4's A = 0.45 at τ ≈ 0.93 gives a luma deviation of roughly 5–8/255. FF-RING's 15 %: the shipped
density (0.25–0.35 of plan area) seen obliquely from 40 m covers well over a third of the ground; 15 % fails
only a bin that is nearly bare. The operator rules the numbers (Q-3); they are never tuned after a run.

### 6.4 Verdict lines

```text
[farfield-order]  VP-high SW  coronado  41/41 columns land over water (100.0%) >= 90%   ok
[farfield-crest]  VP-high W   612/640 columns within 0.10 deg (95.6%) >= 95%   ok
[farfield-seam]   VP-east E   0 seams   ok
[farfield-relief] VP-east NE  mesas  luma sd 6.1/255 >= 2.5   ok
[farfield-ring]   VP-high S   80-100 m  massing 38.2% >= 15%   ok
[farfield-bay]    VP-west W   188/190 columns show water (98.9%) >= 90%   ok
FAIL farfield-crest VP-high W: 31/640 within 0.10 deg (4.8%) < 95% (point-loma low by 0.60 deg)
all farfield checks pass            |   <n> FARFIELD CHECKS FAILED
```

The summary line is the same one RL-e0's harness judges (`tools/cli/tests/client_far_field.rs` `judge`), so the
Rust test needs only its minimum verdict-line count raised (the new lines carry the same `[farfield` prefix
family: the harness's prefix becomes `[farfield`).

### 6.5 Planted mutations (each must fail by name, then be reverted)

| Id | Plant | Must fail with |
| --- | --- | --- |
| **X-6** | the sea ribbon at a fixed 1 400 m again (SD-1 off for the ribbon) | `FAIL farfield-order VP-high SW … coronado` |
| **X-7** | the per-layer median parallax of A-2 restored | `FAIL farfield-crest VP-high W` (and NW, SW) |
| **X-8** | `T_BAND` = 0 | `FAIL farfield-seam` at a band end (e.g. `VP-east E` at `mesas`' 135° end, `VP-high SE`) |
| **X-9** | `T_WATER` = 0 (radial shore, square ribbon ends) | `FAIL farfield-seam VP-high S` / `W` naming `water` or `sea-ribbon` |
| **X-10** | `texture_amp` = 0 | `FAIL farfield-relief VP-east … mesas` |
| **X-11** | both massing sectors' `from_m` back to 250 | `FAIL farfield-ring VP-high <every sector> 80-240 m` |
| **X-12** | the sightline cap off (ring boxes at full height) | `FAIL farfield-crest VP-east E` (massing over the crest) or `FAIL farfield-bay VP-west` |
| **X-13** | (the instrument) ID colours left on the slice's own materials in I | `FAIL farfield-id … unreadable` |
| X-1 (RL-e0) | ribbon winding reversed | still `36 FARFIELD CHECKS FAILED` or more (AC-8) |

---

## 7. Regression shots and V-gates

Re-shoot RL-e0 §7's seven views (`./mineworld-slice --farfield-shots`), plus two aerials the review needs
and RL-e0 did not have: `ff_8_southwest_aerial` (0, 40, 0) yaw −135 → bearing 225 (Coronado, Point Loma, the
bay's south end) pitch −4, and `ff_9_ring_aerial` (0, 40, 0) yaw 0 pitch −27 (the ring, as G sees it). The
before/after pairs (base vs head) go to the operator (RL-e0 §7's V-gate rule: a PR that changes the far field
presents both). **V-1 (RL-e0 §7)** holds: accepted near views outside the far-field mask change by at most the
base's noise + 0.5 points. Expected exception, recorded before running: the ring massing at 70 m is visible
past the façades' ends in `01_street_wide`-type views; those pixels are far-field pixels (S ≠ H) and so are
inside the mask by construction.

---

## 8. Performance and CI cost

### 8.1 Frame cost (RL-b R-2 budget; RL-e0 §8's method)

Estimate: silhouettes 7 meshes instead of 3 (+4 draws, + depth prepass = +8); ring boxes ≈ 350 more
(≈ +6 k primitives); the band shader replaces an unshaded `StandardMaterial3D` (fill only on band pixels,
which are a thin strip at eye level). Caps for this PR: **≤ +10 draw calls, ≤ +20 000 primitives, ≤ +4 MB**
video memory per RL-b M-1 view; **≤ +0.3 ms p95** (operator-measured, as RL-e0 FF-5). Local `--perf`
counts only; timings INCONCLUSIVE on a loaded host (R-LOAD-1).

### 8.2 CI cost

+32 renders on the per-PR Linux `farfield` job: at RL-e0's measured ≈ 19 s per render on hosted lavapipe
(A-10), ≈ +10 min → ≈ 57 min of the job's 90; on `farfield-windows` (≈ 20 s per render) ≈ +11 min → ≈ 52 of
120. Both fit without the companion investigation's fixes; if those land first, the check is faster still
(see the companion record's proposed fixes). A job that no longer fits is a material stop (R-1), never a
reason to drop renders.

---

## 9. Invariants, dependencies, overlaps

### 9.1 Invariants

| Id | Invariant |
| --- | --- |
| I-1 | RL-e0's I-FF-1 … I-FF-8 hold; FF-BELOW and FF-ABOVE still pass on the head |
| I-2 | No schema, validator, kernel, contract, protocol or persistence change (SD-6) |
| I-3 | A band's **outline** comes only from its cited profile points, the curvature/refraction formula, the end taper and the per-height parallax; the texture changes colour only |
| I-4 | Every expected value in the check is computed from the far-field file and §4's formulas in the check's own code |
| I-5 | Nothing added lies within `reach_m`; the ring is massing (walls and roofs), never paving, steps or paths |
| I-6 | `CameraRig.FAR` unchanged; every proxy lies in (1 450, 1 600) m |
| I-7 | The layout is deterministic from the file's seed: the same file gives the same boxes, caps included |
| I-8 | No criterion's number is changed after any run (`ARC-23`) |

### 9.2 Dependencies and order

```text
origin/main (RL-e0 merged) ─► RL-e0b ─► release
                                │
                                ├─ S33-d (Lakeside in 3D) draws with SliceFarFieldView: RECOMMENDED after RL-e0b,
                                │   so Lakeside inherits SD-1 … SD-5 and passes Lakeside's own protected eyes
                                ├─ RL-b2 (#212), 16c (#245), 12e (#254): touch slice_world.gd / slice_main.gd
                                │   (adjacent lines; mechanical rebase either way)
                                └─ the companion investigation's fix PR: independent; either order
```

### 9.3 File overlaps

| File | Others | Resolution |
| --- | --- | --- |
| `far_field_view.gd` | S33-d (not yet designed in detail) | RL-e0b first; S33-d builds on it |
| `slice_world.gd` | RL-b2, 16c | the one `build` call line; mechanical |
| `slice_far_field_check.gd`, `client_far_field.rs` | none open | — |
| `social-cafe.yaml` (3D) | none open | — |

---

## 10. Acceptance (fixed before measuring)

| Id | Criterion | Evidence |
| --- | --- | --- |
| AC-1 | `./mineworld-slice --farfield` prints `all farfield checks pass` on macOS locally, with every line of §6.4's six families present for the three viewpoints | log |
| AC-2 | The same on hosted Linux (per-PR `farfield` job) and Windows (`farfield-windows`, one dispatched nightly on the exact head) | CI logs |
| AC-3 | Mutations X-6 … X-13 each fail by name on macOS locally; X-7 and X-11 also once on a scratch nightly or a dispatched `farfield` run | ledger |
| AC-4 | `mineworld packs validate presentation/mineworld-default/3D` and `/2D` pass; `cargo test -p mineworld-packages --test far_field` passes (AC-FF-4 water agreement unchanged) | logs |
| AC-5 | `cargo test -p mineworld-cli --test client_far_field -- --ignored --test-threads=1` 3/3 locally | log |
| AC-6 | V-1 holds (§7) | `frame_diff` output |
| AC-7 | §8.1's count caps hold on the M-series (`--perf`, one run per side) | ledger |
| AC-8 | X-1 (RL-e0) still fails with at least its 36 named lines | ledger |
| AC-9 | Shots `ff_1` … `ff_9` committed; the operator's look (FF-1 … FF-3 of RL-e0 plus FF-7 below) | files; operator |
| AC-10 | `python3 scripts/check_client_rules.py` and `tests/acceptance/tests/client_rules.rs` pass (no rule word in client code, RL-e0 D-FF-16) | logs |

Operator item added to Milestone F / release checks: **FF-7** — open `ff_3`, `ff_5`, `ff_8`, `ff_9` beside RL-e0's
versions: Coronado over the bay from above; Point Loma standing above the sea horizon; no vertical line where a
band or the bay ends; the mesas textured; the ground between the street and the distant town built on.

---

## 11. Test ownership

| Layer | Owns | Here |
| --- | --- | --- |
| Static | format, lint, doc headings, decision ids, client-rules scan | `cargo fmt`, `clippy -D warnings`, `check_doc_headings.py`, `check_decision_ids.py`, `check_client_rules.py` |
| Unit / contract (Rust) | the content stays valid; water agreement | `packages/tests/far_field.rs` (unchanged, re-run) |
| Godot, real rendering (`#[ignore]`; per-PR Linux, nightly Windows, local macOS) | D-1 … D-5 as FF-ORDER, FF-CREST, FF-SEAM, FF-RELIEF, FF-RING, FF-BAY | `tools/cli/tests/client_far_field.rs` (verdict count raised) |
| Mutation | each criterion can fail | §6.5, ledger |
| Operator | the look; timings | FF-7; FF-5 of RL-e0 re-run |
| Real-LLM Gate | — | N/A: no model involved |

Decision record (C1): **`ARC-290` — the far field is drawn in real depth order and judged by an ID render.**
Problem: D-1 … D-5 and A-7 (presence criteria cannot see position, order, seams, texture or ground cover).
Choice: §4's five drawing rules and §6's six criteria from flat-ID frames of the same run. Not chosen: golden
images (ARC-103's reasons); per-defect hand checks only (a regression would merge unseen). Amends ARC-103 by
addition; DEP-45's dated note covers ID frames (they are frames of the same run).

---

## 12. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and its recorded evidence; an inapplicable item is `N/A` with the audited reason.

### C0 — Freeze and contract (Markdown only) · Haiku

- [ ] Implementation: record the base; confirm the frozen header and §13; create `handoff-rl-e0b.md`.
- [ ] Validation: `check_doc_headings.py`, `check_decision_ids.py`.
- [ ] Review: no authority line widened; merge stays the operator's.

### C1 — Specifications before code · Haiku (texts in Appendix A)

- Goal: the drawing rules and the new criteria exist as specification before the code (`CLAUDE.md` §2.2).
- Scope: `docs/VISUAL_SLICE.md` (far-field row: Appendix A.1), `docs/ART_DIRECTION.md` §21 (A.2), the decision
  record `ARC-290` (A.3) as `docs/decisions/ARC-290.md` (#258 split the records into files).
- [ ] Implementation: paste A.1 … A.3.
- [ ] Validation: doc checks PASS; `grep -n "real distance" docs/VISUAL_SLICE.md` shows the rule.
- [ ] Review: engine-neutral wording in VISUAL_SLICE (no node, shader or Godot term); terms exact.

### C2 — The band shader and per-column depth and parallax (SD-1, SD-2) · **Opus**

- Scope: `shaders/far_silhouette.gdshader` (new, without texture yet: `texture_amp` uniform present, used in
  C4); `far_field_view.gd` constants, `_build_silhouettes`, `_ribbon_column`, `follow`, the sea ribbon's proxy.
- [ ] Implementation: one mesh per band; `UV2 = (P(d)/d, d)`; `rise` per material; ribbon at `P(d_h)`.
- [ ] Validation: `./mineworld-slice --farfield` (old criteria) all pass; `--farfield-shots` `ff_3`, `ff_5`
  looked at: Coronado drawn over the bay, Point Loma above the sea horizon; RL-e0 X-1 re-run → named FAILs.
- [ ] Review: no proxy outside (1 450, 1 600); the parapet view (`ff_6`) unchanged in the far field's depth
  order at eye level (the bay in front of south-hills' base).

### C3 — Tapers, curved shore, horizon interpolation, content overlaps (SD-3, §5.6) · Opus

- [ ] Implementation: `_crest_eff`; `shore(b)` in foreground and water; ribbon end taper; `_horizon_sampler`
  interpolation; the four band-end overlaps in `social-cafe.yaml`.
- [ ] Validation: `mineworld packs validate presentation/mineworld-default/3D` PASS; `cargo test -p
  mineworld-packages --test far_field` PASS; old criteria pass; shots looked at for notches at 125° and 315°.
- [ ] Review: the overlaps' sources say approximate and name RL-e; the taper never changes a profile point.

### C4 — Band texture (SD-4) · Opus

- [ ] Implementation: the fragment noise with τ(d); `texture_amp` 0.45.
- [ ] Validation: old criteria pass; `ff_4`, `ff_7` looked at; the far layer visibly flat (τ small).
- [ ] Review: crest rows identical with `texture_amp` 0 and 0.45 (I-3: a frame diff of I renders is empty).

### C5 — The ring and the sightline cap (SD-5) · Opus

- [ ] Implementation: `protected_eyes` through `slice_world.gd`; the cap in `_build_skyline`; `from_m` 70 in the
  content.
- [ ] Validation: old criteria pass; `ff_6`, `ff_7`, `ff_9` looked at (the bay and the mesas still seen from
  the street's ends); massing count printed (expect ≈ +350 boxes).
- [ ] Review: the cap uses the same `α_eff` as the drawing; deterministic (two runs, identical box count and
  first/last box positions printed).

### C6 — The ID render and the six criteria (§6) · Haiku, reviewed by Opus

- [ ] Implementation: `ids()` in the view; I and G renders; decoding; FF-ORDER, FF-CREST, FF-SEAM, FF-RELIEF,
  FF-RING, FF-BAY with §6.3's constants; §6.4's lines; `client_far_field.rs` verdict minimum and prefix.
- [ ] Validation: AC-1 on macOS; X-6 … X-13 one by one (each restored); X-1; AC-5.
- [ ] Review: every expected angle computed in the check from the file (I-4); no constant changed after a run
  (I-8); unknown pixels make a FAIL, never a pass.

### C7 — Shots, perf counts, CI evidence · Haiku

- [ ] Implementation: `--farfield-shots` with `ff_8`, `ff_9`; re-shoot all nine; `.gitignore` unchanged
  (the directory is already excepted).
- [ ] Validation: V-1 (AC-6); `--perf` counts each side (AC-7); push; the per-PR `farfield` job and one
  dispatched `farfield-windows` on the exact head (AC-2); X-7 and X-11 once on hosted (AC-3).
- [ ] Review: images ≤ 300 KB each; none from a mutated run.

### C8 — Review package · Haiku

- [ ] Implementation: ledger; FF-7 into the release checklist (`docs/MVP_STATUS.md`) and
  `docs/HUMAN_REVIEW_QUEUE.md` (before/after pairs, launch commands, Q-1's choice as shipped).
- [ ] Validation: doc checks.
- [ ] Review: every operator item starts with `cd` to the repository and names the full command.

---

## 13. Execution contract (template; *primary* lines are the primary session's at freeze)

```text
PROJECT / PR:            MineWorld S22 visual track, RL-e0b — far-field polish (release scope, 2026-10-18)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp1/pr-rl-e0b-far-field-polish.md
RELATED / BINDING DOCS:  pr-rl-e0-far-field-mandatory.md §§5.4, 6, 7, 8, 18; step-22-realism.md;
                         docs/MODULE_SPEC.md §6.1.1; docs/VISUAL_SLICE.md; docs/ART_DIRECTION.md §21;
                         docs/VISUAL_FIDELITY.md §§5, 8; DECISIONS ARC-23, ARC-102, ARC-103, DEP-45; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-rl-e0b, its own, one session   (primary)
BRANCH:                  mvp1/pr-rl-e0b from origin/main                                        (primary)
APPROVED SCOPE:          §§4, 5, 6, 7 as frozen
FROZEN INVARIANTS:       I-1 … I-8; §6.3's rules and numbers; §8.1's caps
SEQUENCE:                C0 … C8
IMPLEMENTERS:            Opus for C2–C5; Haiku for C0, C1, C6–C8 (C6 reviewed by Opus)          (primary)
ALLOWED COMMANDS:        cargo -p <crate> *; git; gh (never merge); python3 scripts/*; godot via
                         /Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh; ./mineworld-slice *;
                         target/*/mineworld *; sed -n
NEVER:                   python3 -c; sed -i; curl; git add -A; cargo test --workspace locally; tuning §6.3
                         after a run; a schema change; edits outside scope
MATERIAL STOPS:          a criterion cannot be met without changing its number; a count cap exceeded; the
                         per-PR farfield job or farfield-windows exceeds its limit (R-1); V-1 fails outside
                         the ring's expected pixels; a needed schema or CameraRig.FAR change; Q-1's chosen
                         option fails FF-RING or FF-BAY
VALIDATION BUDGET:       local: per-crate tests, the probes on macOS through with-slot.sh, one --perf counts
                         run per side; CI: the PR's runs + one dispatched farfield-windows + one scratch
                         dispatch for AC-3                                                       (primary)
HANDOFF:                 .structured-coding/plans/mvp1/handoff-rl-e0b.md
ENDPOINT AUTHORITY:      implementation, local validation, semantic commits, branch push, PR creation and
                         update, CI repair to review readiness                                  (primary)
                         merge: never by the implementation session
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 14. Edits to parent documents (the primary session's, after freeze or merge)

- `step-22-realism.md` §9.1: insert `RL-e0b far-field polish (release)` after RL-e0; §9.2: a row pointing
  here; RL-e's adversarial criterion keeps reading the profile points (unchanged by this PR).
- `../mvp0/overall.md` release checklist: FF-7.
- RL-e0's ledger: a dated note under §18 "Review defects D-1 … D-5 → RL-e0b".

---

## 15. Open questions (each with a recommendation)

Ruled by the operator on 2026-10-18 in the primary session: every question answered with its recommendation.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-1 [OPERATOR]** | **What fills the ring** between the street's boundary (60 m) and the distant town (250 m)? (a) the same low-rise massing from 70 m, heights capped so roofs stay ≥ 0.25° below what the street's ends look at (the bay, Point Loma, the mesas); (b) a landscape ring — scrub, tree clumps, terraces — as a new far-field part (schema change); (c) leave it: no player sees it until a high viewpoint exists | **(a).** It is what the place is (a town continues past its street), uses the existing schema, and FF-BAY / FF-CREST prove the street's views survive. (b) adds a part and a schema field for a ring that real San Diego fills with buildings; (c) contradicts the release ruling |
| **Q-2 [OPERATOR]** | **How the mesas get texture.** (a) a procedural angular noise in the band shader, contrast fading with Koschmieder transmittance, no schema change; (b) also add a per-band `cover` field (scrub, urban, forest) now; (c) wait for RL-e's NLCD-driven terrain shader | **(a).** It fixes D-4 for every band at release; (b) is a schema change whose values RL-e will derive from data anyway; (c) leaves a release defect |
| **Q-3 [OPERATOR]** | **The new criteria's numbers:** FF-ORDER 90 %, FF-CREST ±0.10° in 95 % of columns, FF-SEAM runs ≥ 6 rows (0.22°) forbidden, FF-RELIEF luma deviation ≥ 2.5/255, FF-RING ≥ 15 % massing per 20 m bin, FF-BAY 90 % — accept? | **Accept.** Each is placed between the defect's measured size and the renderer's noise (§6.3's paragraph) |
| **Q-4 [OPERATOR]** | **Where the new criteria look from:** VP-west, VP-east and VP-high (+32 renders, ≈ +10 min on hosted Linux), or all seven viewpoints (+64 renders, ≈ +20 min)? | **The three.** The defects are seen from height (D-1, D-2, D-5) and at the street's ends (D-3, D-4); the four mid-street viewpoints look at façades in N and S |
| **Q-5 [OPERATOR]** | **Two extra regression aerials** (`ff_8` south-west at 40 m, `ff_9` the ring at pitch −27°) — add them to the committed shots? | **Yes.** Each defect needs a view at the resolution of its claim (`VISUAL_FIDELITY.md` §8); RL-e0's seven had none aimed at Coronado or at the ring |
| **Q-6 [primary]** | Order against S33-d (Lakeside's 3D far field uses the same view) | RL-e0b first; S33-d passes Lakeside's own protected eyes (its pier and halt ends) |
| **Q-7 [primary]** | The four band-end overlaps of §5.6 are hand-estimated (approximate, replaced by RL-e) — accept as content? | Accept; they move two ends by 5° each and invent no summit |

---

## 16. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-1 | The +32 renders push a hosted job past its limit | §8.2's margin; the companion fixes; material stop, never fewer renders |
| R-2 | ID colours shift under some driver's colour pipeline | linear tonemap, AA off, debanding off for I/G; tolerance 24/255; X-13 proves an unreadable frame fails |
| R-3 | The cap empties the ring in the bay corridor so FF-RING fails there | SD-5's 1 m floor (walls and sheds) keeps cover; a residual failure is Q-1's material stop, with the measured bins |
| R-4 | Proxy depths 1 460–1 590 m lose depth precision | Godot 4.7's reversed-Z float depth; 130 m spread across a log scale; FF-ORDER would show a failure |
| R-5 | The texture reads as noise, not terrain | the anisotropic first octave (gullies) and τ(d); judged by the operator (FF-7), revisited by RL-e |
| R-6 | S33-d diverges on the view | §9.2 order; S33-d's design reads this one |

---

## 17. Ledger (live during implementation)

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | see `git log` | [x] | [x] doc checks | [x] |
| C1 | see `git log` | [x] | [x] doc checks | [x] |
| C2–C5 | see `git log` (drawing) | [x] | [x] E-0b-1 | [x] self-review; independent review deferred (charter, speed first) |
| C6 | see `git log` (check) | [x] | [x] E-0b-1 | [x] I-4: expected values from the loaded file only |
| C7 | see `git log` (shots) | [x] nine shots, ≤ 160 KB each | [~] V-1, `--perf` and hosted runs deferred to the milestone (charter) | [x] |
| C8 | see `git log` (docs) | [x] FF-7, review queue | [x] doc checks | [x] |

### Evidence

```text
E-0b-design  2026-10-18  audit of origin/main @ b85f2804 (§2); RL-e0's shots read as images; the numbers of
                         §2.2 computed from social-cafe.yaml and far_field_view.gd
E-0b-1       2026-10-19  macOS, through with-slot.sh, on the final head's content:
                         ./mineworld-slice --farfield                                → all farfield checks pass
                         ... --far-field-file=<social-cafe.yaml rotated 90 deg>      → all farfield checks pass
                             (scripts/far_field_variant.py; the generality proof the operator asked for)
                         ... --far-field-file=presentation/.../3D/far_field/lakeside.yaml → all farfield checks pass
                         mineworld packs validate 3D and 2D PASS; cargo test -p mineworld-packages --test far_field
                         6/6; cargo fmt --check, clippy -p mineworld-cli -D warnings PASS; check_client_rules PASS
E-0b-2       2026-10-19  mutations X-6 … X-13, X-1, V-1, --perf counts and the hosted farfield / farfield-windows
                         runs: NOT RUN in this PR — the lane charter (operator, 2026-10-18, speed first) moves them
                         to the milestone run. AC-2, AC-3, AC-6, AC-7, AC-8 stay open until then.
```

### Bounded corrections and one deviation found by running (2026-10-19)

Nothing in §6.3's numbers changed (I-8). Found while making the check pass on the shipped file, the rotated
variant and Lakeside's file:

```text
F-0b-1  bounded  FF-RELIEF measured 2.1–2.4/255 with the noise applied to linear colour; §6.3 defines L as
                 the displayed 8-bit luma, so the shader now modulates in sRGB and converts back
                 (far_silhouette.gdshader). Same A = 0.45 and F; measured 4.5–5.7/255, as §6.3 predicted.
F-0b-2  bounded  the sightline cap judged a box only at its centre's bearing; a 20 m box 40 m from an eye
                 spans ~30 deg, so roofs stood over the tapered crest (FF-CREST VP-east SE 83 %) and over
                 the bay (FF-BAY VP-east S 72 %). The cap now takes the lowest limit across the bearings
                 the box covers from each eye.
F-0b-3  bounded  the sea ribbon used StandardMaterial3D.disable_fog and was still drawn in the depth fog's
                 colour (it lies beyond fog_depth_end), i.e. as the sky: FF-BELOW failed wherever the water
                 faced north (rotated variant, Lakeside). It is now drawn by a fog-free shader, its colour
                 per bearing (not a sector mean), the reflected sky clamped to the fog's colour, and a
                 Fresnel share of 0.4 (inside Q-FF-2's cap of 0.6).
F-0b-4  bounded  FF-ORDER judged columns where the end taper leaves a band less than the probe's two rows
                 tall (the probe then lies on the water by construction); such columns are not judged.
                 FF-BAY likewise judges the bearings outside T_WATER of the water's ends, where SD-3.2/3.3
                 narrow the water to nothing on purpose.
F-0b-5  content  Lakeside's two bands abutted at 60 and 300 deg (A-9's case): each end now overlaps its
                 neighbour by 5 deg, behind it; its massing starts at reach + 10 = 90 m (SD-5.1's rule).
                 Lakeside's water is unchanged (AC-FF-4).
F-0b-6  split    far_field_view.gd passed 800 lines; the massing's geometry helpers moved to
                 far_field_massing.gd (§5.2's planned bounded split).
D-0b-1  DEVIATION (needs the operator's ruling; R-3's material stop) — SD-5.2 named the water's NEAR edge
                 as what a roof must stay below in a water sector. From the street's ends (eye 1.8 m, the
                 shore 1 km away, the ground falling only slowly before 140 m) every cap within ~140 m in
                 the bay's bearings came out under 1 m, no box was built, and FF-RING failed in all eight
                 bay bins (VP-high S/SW/W, 80–140 m, massing 0–8.9 %). Shipped instead: the roof stays
                 CAP_MARGIN_DEG below the water's FAR side (the foot of the nearest band in front of the
                 sea horizon, or the sea horizon), blending back to the near edge within T_WATER of the
                 water's ends. FF-BAY (≥ 1 row of water) and FF-RING both pass; the bay from the parapet end
                 is a strip between roofs and Coronado/Point Loma rather than the whole bay (ff_6).
                 Alternative if the operator prefers the whole bay in view: keep the near edge and accept
                 that the bay corridor stays bare (FF-RING then needs an exception for water-facing bins).
```

---

## Appendix A — Specification texts (C1 pastes these)

### A.1 `docs/VISUAL_SLICE.md`, §4 far-field row, appended

"Seen from any height a player or a review camera can take: land nearer than the water's horizon is drawn in
front of the water; every band of distant terrain stands at its true elevation angle for that eye height; no
band, shoreline or water edge ends in a vertical line — each meets the horizon or the land beside it in a
slope; near terrain shows surface texture whose contrast fades with distance as the haze does; and the ground
between the reachable area and the distant town is built on, with roofs kept below the lines along which the
reachable area sees the far field."

### A.2 `docs/ART_DIRECTION.md` §21 "Environment", appended

"From above as from the street: no edge of a far-field element is a vertical cut, distant land is not a
flat-coloured slab, and no bare ring of ground separates the town from its surroundings."

### A.3 Decision record `ARC-290` (`docs/decisions/ARC-290.md`)

As §11's paragraph, with Date, Status, Relates to (`ARC-23`, `ARC-102`, `ARC-103`, `DEP-45`) and Design
(this file).
