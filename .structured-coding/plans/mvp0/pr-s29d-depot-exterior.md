# PR S29-d — The 3D depot: towers, arch, hall shell, arcade and roofs, the forecourt, and the landmark check

## DESIGN FROZEN — 2026-10-19 (operator approved in the primary session 2026-10-19: every open question answered as recommended)

```text
Design revision:        revision 1 (2026-10-19), DRAFT; reconciled 2026-10-19 with S29-a revision 4, which
                        carries R-S29a-d-1 exactly (its §5.5; Q-S29d-1 is asked there as Q-S29a-8); role
                        tags are CORE_CONCEPTS §6.5 (§6.4 is "Sit point")
Author:                 S31/S29 lane design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s29de, branch docs/pr-design-s29de,
                        planning base origin/main @ 44f050e3 (#171, 12d, merged; #258 decision-log split
                        merged; #261 ARC-153 merged). Designed against PR #245 (16c) head 4e39ea69, PR #254
                        (12e) head e8d481f4, PR #212 (RL-b2) head d138d153, and the frozen S33-c design
                        (pr-s33c-world-seam.md, not implemented)
Approved by / evidence: none yet. Step-29's rulings Q-S29-1 … Q-S29-10 (operator, 2026-10-15) and S29-a's
                        Q-S29a-1 … Q-S29a-7 bind and are not re-asked
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of 16c, 12e,
                        S33-c, S29-a, S29-c and RL-b2
Start condition:        (1) Milestone F's checklist handed to the operator (step-29 §11.2, operator
                        2026-10-15); (2) S29-a merged, carrying §10's R-S29a-d-1 (Q-S29d-1); (3) S29-c
                        merged (Social Café's `filled` slots, reach_m 100); (4) 16c merged (#245: OPEN at
                        44f050e3); (5) S33-c merged (Q-S29d-5; frozen, not started); (6) this design
                        frozen; (7) the lane lead's dispatch. RL-b2 (#212: OPEN, draft, operator sessions
                        parked) is NOT required (Q-S29d-5). 12e (#254: OPEN) is used when merged and
                        recorded N/A otherwise (§2.3). NONE of (1)–(6) holds at 44f050e3
Execution contract:     §12
Implementer:            Opus-class (step-29 §11.1, the S29-d row: "Opus (art judgement, perf levers);
                        mechanical parts delegable"); no separate reviewer per PR (SPEED FIRST)
Lifecycle:              DESIGN FROZEN 2026-10-19 —
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-29-station-landmark.md`](step-29-station-landmark.md)
(APPROVED 2026-10-15) §§1 (outcomes 2, 5, 6, 8), 2.2, 6.1, 6.2, 6.3, 6.4, 6.6, 9.1, 9.2, 9.5 (X-6), 10 (I-S29-5,
I-S29-6, I-S29-7, I-S29-8), 11.1 (the S29-d row), 13 (Q-S29-1, Q-S29-4, Q-S29-5, Q-S29-8, Q-S29-10), 14.4 ·
**Parent:** [`overall.md`](overall.md) · **Consumes:** [`pr-s29a-town-contract.md`](pr-s29a-town-contract.md)
(the station places, as §10 asks it to revise them), S29-c (not yet designed: the `filled` slot, `reach_m` 100),
16c (`SliceLayout`, the door registry, binding by doorway coincidence), [`pr-s33c-world-seam.md`](pr-s33c-world-seam.md)
(FROZEN: `SliceDepiction`, binding by tag set), 12e (the disclosed-geometry probe), RL-e0 (the rendered
far-field check and its harness) · **Followed by:** [`pr-s29e-depot-interior.md`](pr-s29e-depot-interior.md)
(the fit-out, the platform, `station_state.gd`), S33-e (Lakeside's halt with S29-e's pieces; its pier uses
this PR's landmark check) · **Binding parents:** `CLAUDE.md` §§1, 2, 4 (items 3, 5, 6, 7, 10, 11, 13, 14, 15,
16); `docs/ENGINEERING_RULES.md` §§2–3, 5–6, 10–12; `docs/VISUAL_SLICE.md` §§3, 4, 7.1, 12;
`docs/MODULE_SPEC.md` §6.1; `docs/decisions` `ARC-45`, `ARC-102`, `ARC-103`, `ARC-105`, `ARC-106`, `ARC-107`,
`ARC-144`, `DEP-8`, `DEP-16`, `DEP-60`.

**Decision ids.** This PR writes **`ARC-192`** (new, from the range ARC-192 … ARC-195 given to this lane): a
station is drawn from a **station kit** — a Presentation Pack data file — placed at a site the world depiction
declares, and bound to the world's places by role tag. It adds one dated note under **`ARC-106`** (the measured
dimensions, the tower placement of Q-S29d-2, the measured bearing) and closes **`DEP-60`** (withdrawn, or
recorded if a reviewed frame triggers it). `ARC-193` … `ARC-195` and `DEP-102` … `DEP-104` are not used. S29-e
adds a note under `ARC-192` (the pieces contract) and one under `ARC-108`.

No hosted model, image model or paid API is called by this design or by anything it specifies.

---

## 0. Binding inputs, exactly restated

- **Scope (step-29 §11.1, S29-d row).** "`station.gd` (towers, arch, hall shell, arcade, roofs);
  `glazed_tile.gdshader`; forecourt in `street.gd`; binding by role tag; `--slice-measure` rows; HABS references
  and provenance; perf views; `st_1`–`st_4`; `ff_*` re-shot; `--slice-landmark`."
- **Integration checkpoint (same row).** "`./mineworld-slice --world`: walk from spawn into the hall; the
  server's place changes to `station`; LM-1 and LM-2 pass on three OSes; counts within §6.6."
- **Adversarial criteria (same row).** "X-6; the RL-e0 rendered check must still pass; a tile shader that writes
  to an intent fails the client-rules scan."
- **Operator principles (relayed 2026-10-18; binding).** The station follows the 1915 Santa Fe Depot in San
  Diego: Mission Revival, two tiled towers, the town's landmark. The hall is at real size, 51.8 × 16.8 m
  (Q-S29-4). It is the one sanctioned style exception, within the pack's palette and materials (Q-S29-5). It adds
  at most 6 MB to `3d.pck`, with a target of zero new textures (Q-S29-10). **The 3D building is generic: built
  from data (a station description bound by role tag), never hard-coded to one town. Every acceptance criterion
  names a second world or variant.** Realistic dimensions with cited sources.
- **Rulings that bind (step-29 §13).** Q-S29-1 (P-1: the street's west end, real building, compressed distance);
  Q-S29-4 (the real hall); Q-S29-5 (the sanctioned exception); Q-S29-8 (HABS CA-1965 photographs may be
  committed; CC BY-SA linked only); Q-S29-10 (≤ 6 MB, target 0 new textures). Not re-asked.
- **SPEED FIRST (charter, operator 2026-10-18).** Per PR: the build and the tests the commit writes, plus `fast`
  CI. The step's rendered acceptance (LM-1, LM-2 on three OSes, the RL-e0 check, the perf counts and X-6) is
  kept and **run at the milestone**; failures there are fixed in one batch. The endpoint is MERGE-READY; the
  coordinator merges.
- **R-LOAD-1.** Heavy local jobs (Godot, cargo) only through `cast-slots/with-slot.sh`; local timings are
  INCONCLUSIVE, counts are recorded.

---

## 1. Goal

Make the town's signature building stand at the west end of the 3D street: two tiled towers, the arch on the
street's axis, the 51.8 × 16.8 m hall behind it, the arcade along its track side and the Mission-tile roofs over
all of it, on a setts forecourt that replaces today's parapet. A player at spawn turns west and sees the towers
close the street. They walk about 40 m, pass under the arch, and the server says they are in the station.

The building is **not** a Social Café scene. It is built by one builder from a **station kit** — a data file in
the Presentation Pack that states the depot's dimensions, each with its source, and the role tags of the places
it depicts. A world's depiction only says *where* the kit stands (its site). The client binds the drawn hall,
ticket bay and platform to whatever places the world discloses with those role tags, at those doorways; it never
names a place key. A second world with a station gets the same depot by placing the same kit; a world whose
station is not its landmark (Lakeside) does not match the depot's tags and gets its own kit (S33-e).

A rendered landmark check (`--slice-landmark`, LM-1 and LM-2) holds the "closes the view" claim in place, using
RL-e0's shown/hidden method.

---

## 2. Scope

### 2.1 In scope

1. **Specifications and records (C1).** `docs/decisions/ARC-192.md` (new) and the regenerated
   `docs/decisions/INDEX.md`; a dated note in `docs/decisions/ARC-106.md`; `clients/3d-spike/PRESENTATION.md`
   §5 "`stations/<kit>.yaml`" (new); one line in `docs/MODULE_SPEC.md` §6.1's directory list;
   `docs/ART_DIRECTION.md`'s "Architecture" section: the landmark exception (step-29 §14.4; F-S29d-9).
2. **References (C1).** `presentation/mineworld-default/3D/references/station/` (new): at most six HABS
   CA-1965 photographs and the data-page PDF, with `PROVENANCE.md` (URL, LoC call number, SHA-256, public
   domain under DEP-8) and `MEASUREMENTS.md` (the method and the values of §5.2).
3. **The kit (C2).** `presentation/mineworld-default/3D/stations/depot.yaml` (new);
   `clients/3d-spike/scripts/station/kit.gd` (new, `StationKit`: read, validate, refuse by code).
4. **The tile shader (C3).** `clients/3d-spike/shaders/glazed_tile.gdshader` (new, ≈ 80–120 lines, our own,
   MIT) and its material constructor in `clients/3d-spike/scripts/station/tile.gd` (new, `StationTile`).
5. **The building (C4).** `clients/3d-spike/scripts/station/depot.gd` (new, `StationDepot`; the step calls it
   `station.gd`, F-S29d-8): hall shell, towers, domes and lanterns, the arch and its doors, the arcade, the
   ticket-bay annex shell, the roofs; door-registry entries; `mw_category` `station.*`; occluder boxes.
   `clients/3d-spike/scripts/slice/batch.gd` (`SliceBatch`: keep `station.` / `landmark.` categories through the
   merge, F-S29d-6); `clients/3d-spike/scripts/slice/occluders.gd` (the depot's boxes).
6. **The forecourt and the site (C4).** `clients/3d-spike/scripts/slice/street.gd` (`_surrounds` → the west end
   is either today's parapet or the forecourt up to a station site); the Social Café depiction
   (`scripts/worlds/social-cafe/depiction.gd`, S33-c's file): the site, the kit's entries in `depictions()`, the
   depot's viewpoints.
7. **Probes (C5, C6).** `--slice-measure` rows for the depot; the 12e `--slice-geometry` probe extended to the
   station's places; `--slice-station-kit` (new mode: build any kit in an empty scene and measure it, including a
   variant); `--slice-landmark` (new, `clients/3d-spike/scripts/slice/slice_landmark_check.gd`); three perf views;
   the shots `st_1`–`st_4` and the re-shot `ff_*`; one ignored Rust test in `tools/cli/tests/client_far_field.rs`.
8. **Records of review (C7).** `docs/HUMAN_REVIEW_QUEUE.md`: one row for the operator's look at `st_1`–`st_4` and
   both `ff_*` sets (RL-e0's V-gate, taken at the milestone under SPEED FIRST).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Hall fit-out (beams, purlins, nine arches, wainscot, benches, kiosk, counter, boards, clock, chandeliers) | S29-e |
| The platform slab, coping, fence and gates, the two tracks, the platform board | S29-e |
| `clients/shared/station/station_state.gd`; the window, kiosk and board rules | S29-e (or S29-f / S33-e2, whoever is first) |
| `st_5`–`st_8`; the threshold measurement at the arch | S29-e |
| Station place files, bodies, passages, routines, `settlement:` | S29-a (revised per §10) |
| The `filled` slot, `FAR_FIELD_SLOT_TOWN`, Social Café's `reach_m` 60 → 100 | S29-c |
| The 2D station | S29-f |
| Lakeside's halt and pier | S33-e (it reuses the landmark check and S29-e's pieces) |
| Any path under `kernel/`, `contracts/`, `systems/`, `server/`, `worlds/`, `clients/protocol/`, `cognition/` | never (I-S29-1; I-d-1) |

### 2.3 Dependencies

- **S29-a**, merged with §10's revision (R-S29a-d-1). Without it, the depot's ticket bay and platform cannot be
  bound (F-S29d-1, F-S29d-2): C0 stops.
- **S29-c**, merged: Social Café's `station` and `landmark` slots `filled`, `reach_m` 100. The far field's near
  ground then reaches 100 m (`far_field_view.gd:192` `_near_ground`), and the skyline keeps clear of the slot's
  footprint. Without it, the depot stands on nothing past 60 m: C0 stops.
- **16c (#245)**, merged: `SliceLayout`, the door registry entries with depth and floor height, binding by doorway
  coincidence. Without it there is no binding at all.
- **S33-c**, merged: the depiction seam. S29-d adds its site and entries to the Social Café depiction and its
  viewpoints to `viewpoints()`. (Q-S29d-5; if the operator rules otherwise, §11's fallback applies.)
- **12e (#254)**: when merged, C5 extends `--slice-geometry` to the station's places; when not, those rows are
  `N/A (12e not merged)` and the milestone runs them.
- **RL-b2 (#212)**: not required (Q-S29d-5). Counts are compared with the RL-b M-1 base on `main`
  (`slice_perf.gd:33-45`); the operator's three-run timings are a milestone item.

---

## 3. Audit (`origin/main @ 44f050e3`; 16c @ `4e39ea69`; 12e @ `e8d481f4`; 2026-10-19)

### 3.1 Source anchors (paths under `clients/3d-spike/` unless stated)

| # | Fact | Where | Consequence |
| --- | --- | --- | --- |
| A-1 | The slice's frame: +x along the street west → east, the slice runs x −34 … +34; +z south; y up; north façade z −8.1, south z +8.1; `WALK_Y` 0.14 | `scripts/slice/street.gd:1-44` | The forecourt and the depot use this frame |
| A-2 | The west end is a parapet pair at `X_MIN − 1` and a setts slab to x ≈ −44 | `street.gd:121-130` (`_surrounds`) | C4 replaces it with a forecourt when the depiction declares a station site; otherwise unchanged |
| A-3 | **Binding only translates.** "The world frame is fixed (+x east, +y north, `CORE_CONCEPTS.md` §6.1), so a binding only translates." A room's origin is `in_point(door) − here`; the street binds a target when a doorway's scene point lies within `DOOR_MATCH` 0.15 m of a drawn door's out-point; `DOOR_INSET` 0.40 m | 16c `scripts/slice/slice_layout.gd:1-30, 44, 49, 184-244`; `docs/CORE_CONCEPTS.md:431-447` | Every station place's frame must agree with the drawn plan **without rotation**; S29-a's towns' bay and platform frames do not (F-S29d-1, F-S29d-2) |
| A-4 | The door registry: `static var doors` in `SliceTerrace`; entries `[sill (global), façade yaw, label, volume, depth to inner face, floor height]` | `scripts/slice/terrace.gd:32, 151, 237`; `cafe.gd:253`; 16c `slice_layout.gd:21-25` | The depot appends three entries (arch, bay door, trackside door) |
| A-5 | S33-c's seam (FROZEN): `SliceDepiction.depictions()` entries `{tags, volume, drawn_as, door, origin}`; a disclosed place binds to the candidate entry whose `tags ⊆` its tags, most tags winning, a tie a miss; the rest by doorway coincidence; `viewpoints()` holds `perf`, `farfield`, `landmark`, `probe_route`; `SliceDepiction.tag()` is the one place a tag literal is written; check (d) of `client_rules.rs` flags any string literal equal to a world place key | `pr-s33c-world-seam.md` §§5.2, 5.4, 5.6 | Binding "by role tag" is S33-c's rule. The kit's tags are data, so no tag literal appears in GDScript (F-S29d-7) |
| A-6 | `SliceBatch.merge` groups by (material, shadow, 24 m cell) and gives each merged mesh the category `ground`, `foliage` or `merged`; it skips only `farfield.*` subtrees | `scripts/slice/batch.gd:38, 47-76, 78-108, 114-124` | A batched depot loses its `station.*` category, so a check that hides `station.*` cannot hide it (F-S29d-6) |
| A-7 | `SliceOccluders.build` places occluder boxes "from the same numbers the buildings" use and refuses an occluder overlapping glazing or a door opening | `scripts/slice/occluders.gd:1-43, 106-143` | The depot adds boxes for the towers and the hall's long walls |
| A-8 | `SliceBudget.apply` sets screen-size `visibility_range_end` on every non-merged instance | `scripts/slice/budget.gd:1-60` | Small depot props are culled at distance with no extra code. **No "café visibility pattern" exists** (step-29 §6.6's last lever, F-S29d-10) |
| A-9 | Perf views M-1: eight `[name, pos, yaw, pitch]`; gates p95 16.7 ms, 2 000 draws, 3 M primitives, 2 048 MB | `scripts/slice/slice_perf.gd:33-58` (moved into the depiction by S33-c) | Three station views are added; step-29 §6.6's increment caps apply in every existing view that sees the depot |
| A-10 | RL-e0's rendered check: S (shown), H (`farfield.*` hidden), E (sky alone); `DIFF` 8; seven viewpoints incl. `VP-west` (−33.5, 1.8, 0); shots `ff_1` … `ff_7` in `res://shots/social-cafe/farfield` with Godot yaw (0 north, −90 east, 90 west) | `scripts/slice/slice_far_field_check.gd:1-58, 128-190` | `--slice-landmark` reuses its render setup and pixel rules; `ff_5` and `ff_6` must show the depot |
| A-11 | The far field draws walkable near ground out to `reach_m` and the falling foreground beyond | `scripts/slice/far_field_view.gd:13, 155-160, 192-219, 346-393` | With S29-c's `reach_m` 100 the depot stands on far-field near ground; S29-d draws only the site's own surfaces |
| A-12 | Probe modes: `SliceProbe.MODES` incl. `measure` (`_measure` prints dimensions against VISUAL_SLICE §3) and `threshold` | `scripts/slice/slice_probe.gd:8-12, 112, 571-602, 701-760` | `measure` gains the depot's rows; `station-kit` and `landmark` are new modes |
| A-13 | 12e's `--slice-geometry`: scene colliders and disclosed geometry agree within 150 mm in both directions in every drawn place, doorway gaps exempt | 12e `scripts/slice/slice_probe_geometry.gd:1-45` | Every disclosed floor edge of the forecourt, hall, bay and platform needs a drawn collider within 150 mm, and no drawn collider may stand on walkable floor (F-S29d-3) |
| A-14 | The far-field CI job: `tools/cli/tests/client_far_field.rs` runs Godot tests, all `#[ignore]`d (`three_d_far_field_closes_every_open_sector` at :126-138), in the `farfield` layer of `scripts/ci_layer.py:183-202` | as named | `three_d_landmark_closes_the_west_view` joins that file and layer |
| A-15 | The client-rules scan forbids action-name literals, constants whose names contain `reach`, `range`, `clearance`, `nudge` or `capacity`, and `worlds/` / `systems/` path literals in client GDScript | `tests/acceptance/tests/client_rules.rs:40-113, 174` | No depot constant is named with those words (the arcade's "clear span" is `ARCADE_SPAN`, not a clearance) |
| A-16 | The Presentation Pack holds `far_field/<world>.yaml` (JSON subset, DEP-16) read by `MineWorldFarField.load_for(pack_dir, key, dimension)`; `characters/cast.yaml` is specified for this client in `PRESENTATION.md` (`ARC-92`) | `clients/shared/environment/far_field.gd:1-62`; `docs/MODULE_SPEC.md:2301-2340`; `clients/3d-spike/PRESENTATION.md` | The station kit follows both precedents: a JSON-subset file in the pack, specified in `PRESENTATION.md` §5 |
| A-17 | Shipped CC0 textures include `painted_plaster_wall`, `clay_roof_tiles_02`, `brown_planks_09`, `wood_floor_worn`, `long_white_tiles`, `sandstone_blocks_05`, `cobblestone_floor_08`, `rectangular_paving` (42 MB in all) | `assets/textures/` | Stucco, Mission tile, timber and setts need no new texture (Q-S29-10's target) |
| A-18 | Social Café's 12d street: `body.floor` x −37 450 … 30 550, y −13 000 … 3 200 (axis y −4 900 → scene z 0); the two west parapet solids at x −37 450 … −36 950 | `worlds/social-cafe/places/street.yaml:28-29, 104-106` (and `market-town` identically) | S29-a moves `min.x` to −62 450 and deletes the parapets; the façade line is then scene x ≈ −59.0 (C0 reads the exact value from `--layout`'s street origin) |
| A-19 | S29-a's hall: floor 16 800 × 51 800 (x east, y north); arch passage `here` (16 400, 25 900), street `there` (−62 050, −4 900); trackside door `there` (400, 25 900); bay door `there` (8 400, 400); 35 solids (bench rows, kiosk) | `pr-s29a-town-contract.md` §5.5 | The hall's frame needs no change |
| A-20 | `VISUAL_SLICE.md` §3.2: door leaf 0.85–1.00 × 2.05–2.20 m; door head > stature + 0.25 m; §7.1 the threshold; §4's out-of-scope "third enterable building" (amended by S29-a C1) | `docs/VISUAL_SLICE.md:88-100, 129-150, 281-298` | The depot's door leaves and the measure rows |
| A-21 | `docs/ART_DIRECTION.md` has no numbered §4; its "Architecture" section starts at :156 | `docs/ART_DIRECTION.md:37, 156` | Step-29 §14.4's "§4" is placed under "Architecture" (F-S29d-9) |
| A-22 | LoC's HABS pages answer automated requests with HTTP 403 (WebFetch, 2026-10-19) | `https://www.loc.gov/item/ca0609/` | The reference fetch has a fallback (R-d-4) |

### 3.2 Facts about the real building used here (read 2026-10-19)

| Fact | Value | Source |
| --- | --- | --- |
| Approach and front | "The main approach was from Broadway, through the court to the **large arched forecourt** with its **glazed window above the entrance doors** to the waiting room." | NRHP nomination 72000248, §7 (form 10-300, prepared 10 Feb 1972; NPGallery asset `d0f262bf-da02-4c71-82f0-dbfa1472f6c9`) |
| Towers | "the great arch, flanked by twin towers of Spanish Renaissance flavor, with delightfully colored tiled domes **capped by tiled lanterns**. The brightly colored tiles with zig-zag patterns, incorporating the railways symbol" | NRHP §7 |
| Roofs | "Simple gable and shed roofs are of Mission tile." | NRHP §7 |
| Track side | "A covered concourse (**west side**) connects the passenger station with the baggage and express section, further north." The 1972 maps show the A.T.&S.F. tracks west of the depot and Kettner Blvd east of it | NRHP §7 and its maps |
| Interior (for S29-e) | "nine arched elements and natural redwood beam and purlin ceilings"; tile wainscot "with plaster above 8'"; "arched clerestory windows, near the ceiling"; "original clay tile flooring in two sizes and colors"; "Long oak benches"; "chandeliers of bronze and glass"; "Wood framed doors with their handcarved lettering on transom scrolls" | NRHP §7 |
| Hall | 170 × 55 ft (51.8 × 16.8 m) | Wikipedia; Gaslamp Foundation (step-29 §2.2) |
| Arcade | 27 × 650 ft (8.2 × 198.1 m), "connecting the passenger terminal with the baggage and express rooms" | Wikipedia (read 2026-10-19) |
| Tower height | **Not found** in Wikipedia, the NRHP form, the Gaslamp Foundation page or a web search | measured in C1 from HABS CA-1965 (§5.2) |

### 3.3 Findings

- **F-S29d-1. S29-a's towns' ticket bay cannot be drawn.** The bay's door point is `here` (3 000, 400), 0.4 m
  inside its **south** edge, and the hall's is `there` (8 400, 400), 0.4 m inside the hall's south edge. Binding
  only translates (A-3), so the bay's frame lands with its y = 400 just south of the hall's south wall and its
  floor running **north**, into the hall (x 5.4 … 11.4 m, y 0 … 4.6 m of the hall). A shared wall needs the two
  points on opposite sides of it: the bay's door must be 0.4 m inside its **north** edge, (3 000, 4 200), and its
  counter on the far (south) wall. S33-e found the same defect in Lakeside's bay (F-S33e-1, R-S29a-e-1). The
  capacity is unchanged by the mirror: 9 × 6 = 54 lattice points with the counter (S29-a F-S29a-8's arithmetic,
  rows mirrored). → R-S29a-d-1, **Q-S29d-1**.
- **F-S29d-2. S29-a's towns' platform runs east–west.** Its floor is x −30 000 … 30 000, y 0 … 6 000 and its door
  `here` (0, 5 600) meets the hall's **west** wall at (400, 25 900). Translated, the platform would lie across the
  hall's west wall, 30 m into the hall. The platform runs north–south along the hall's west side (the NRHP's
  "covered concourse (west side)"; step-29 §6.2). Its frame must be x across, y along: the track edge on the
  **west**, the anchor (0, 0) on that edge at mid-length, the hall's door on the east edge. → R-S29a-d-1,
  **Q-S29d-1**.
- **F-S29d-3. The arcade needs either room or solids.** The arcade is real and 8.2 m deep (§3.2). S29-a's platform
  is 6 m deep, so an 8.2 m arcade would overhang the track by 2.2 m. And the arcade's outer piers stand on the
  platform's walkable floor, where 12e's probe requires a disclosed solid under every drawn collider (A-13). The
  platform therefore grows to 11.0 m (the arcade 8.2 m, the piers' outer face, then a 2.8 m open strip to the
  edge), and S29-a writes the ten piers as solids (10 + 2 benches ≤ 64). → R-S29a-d-1, **Q-S29d-1**.
- **F-S29d-4. A deep porch under the arch would stand on the forecourt's floor.** The towers and a recessed porch
  in front of the hall's façade would cover forecourt floor that `bodies` discloses as walkable (S29-a's street
  floor is 16.2 m wide and ends at the façade line, A-18). Either S29-a adds tower solids to the street, or the
  towers stand **beside** the forecourt: inner faces on the floor's north and south edges (scene z ∓ 8.1), the
  arch in the façade between them. The second needs no world change, lets the tower walls be the drawn colliders
  on the disclosed edges, and matches the NRHP's "large arched forecourt" between the towers. → **Q-S29d-2**.
- **F-S29d-5. The hall's centroid is at 270°, not 277°.** S29-a centres the arch on the hall's length (25 900 of
  51 800), so nothing offsets the hall southward (step-29 §6.3's "southward offset" does not exist). The centroid
  is at ≈ (−68.3, 0) scene, bearing 270°, ≈ 68 m: S29-c's default content (270°, 70 m) agrees. C1 records the
  measured value under `ARC-106`.
- **F-S29d-6. Batching erases the depot's category.** `SliceBatch` merges the depot's boxes with the street's
  (same material, same 24 m cell) and labels the result `merged` (A-6). A landmark check that hides `station.*`
  would then hide nothing. C4 adds a kept-prefix list (`station.`, `landmark.`) to the group key: a primitive under
  an ancestor whose `mw_category` begins with a kept prefix is merged only with its own category, and the merged
  mesh carries it. S33-e's pier (`landmark.*`) and halt (`station.halt`) use the same list. Existing categories are
  untouched, so S33-c's pixel-equal shots stay equal.
- **F-S29d-7. Role tags must not be GDScript literals.** `station` is a place key in three worlds, so S33-c's check
  (d) would flag `"station"` in a GDScript file (A-5). The tags live in the kit file (data), and the builder reads
  them; the only literals are volume names such as `"station.hall"`, which equal no key.
- **F-S29d-8. File names.** The step names `station.gd`. A station is several files here (kit reader, tile, depot;
  S29-e adds pieces), so they go under `scripts/station/`, and the exterior builder is `depot.gd`
  (`StationDepot`), because Lakeside's halt is a station too and is not this building.
- **F-S29d-9. `ART_DIRECTION.md` has no §4** (A-21). The landmark exception goes under its "Architecture" section
  with a dated heading.
- **F-S29d-10. One step lever does not exist.** Step-29 §6.6 lists "the interior is culled from street views by
  the existing visibility pattern (the café's)". No such pattern exists (A-8). The levers that do exist are the
  hall's walls as occluders (A-7), the 24 m batch cells, and `SliceBudget`'s screen-size culling; C6 measures with
  those.
- **F-S29d-11. X-6 cannot fail at `st_2` as written.** From `st_2` (30, 1.8, 0), the depot is ≈ 89 m away. LM-1's
  band runs from the horizon to the street's eaves line at its west end (≈ 8.9° for 10 m eaves at 64 m). The
  hall's ridge alone (≈ 12 m, placeholder) reaches ≈ 6.5°, so it covers ≈ 73 % of the open band with the towers
  hidden. LM-1 (≥ 60 %) then still passes, and X-6 ("hide the towers → FAIL st_2 LM-1") does not turn red. From
  `st_1` (spawn, ≈ 47 m), the hall reaches ≈ 12° of a ≈ 24° band (≈ 50 %), and the towers decide. These numbers
  use placeholder heights; C6 recomputes them from the measured values before any run. → **Q-S29d-4**.
- **F-S29d-12. The operator's "generic" rule needs a second world for every check.** The slice draws one world
  today. §8 names, for each criterion, a second case: a **renamed-key copy** of Social Café (keys `depot`,
  `depot-window`, `depot-platform`, same tags; it must bind identically, proving no key is read), **Market Town**
  joined with `--server` and the Social Café depiction (its station places are identical in frame; it must bind
  with 0 misses), or a **kit variant** built by `--slice-station-kit` (the reduced hall of Q-S29-4's reserved
  lever, 30 × 12 m).

### 3.4 What does not exist

No station kit, no station builder, no glazed-tile shader, no `scripts/station/`, no landmark check, no
`station.*` category, no HABS reference in the tree. No station place on `main` (S29-a is not implemented).

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Decision | Why |
| --- | --- | --- |
| Building geometry | **Build ours** with `Build`, `Mats`, `SliceBatch`, `SliceOccluders` (step-29 §6.4) | The town's one method (I-S29-5). A third-party mission model is a second style and fails DEP-8 more often than not |
| Stucco, Mission tile, timber, setts | **Reuse** `painted_plaster_wall`, `clay_roof_tiles_02`, `brown_planks_09`, `cobblestone_floor_08` / `rectangular_paving` (A-17) | Paid for already |
| Glazed tile (domes, lanterns, wainscot, the floor's clay tile) | **Our own shader**, one file, one parameter set per pattern | Zero bytes. No CC0 set has the depot's zigzag; a photo texture of the real tiles is someone's copyright. Declined: decals (one draw each, and a texture each) |
| The data format of the kit | **YAML's JSON subset read by Godot's `JSON`** (`DEP-16`, as far field and cast) | No new parser. Declined: a Godot `.tres` resource (not readable by `packs` tooling or a human diff); a GDScript constant table (hard-codes one depot in code) |
| The landmark check | **RL-e0's harness** (S/H/E renders, `DIFF` 8, viewport setup) | One method for rendered checks. Declined: a ray-cast coverage test (it would pass with the GPU drawing nothing, RL-e0's own finding) |
| References | **HABS CA-1965** (public domain); Wikimedia Commons' mirrors of the same HABS files as the fetch fallback (R-d-4) | DEP-8; Q-S29-8 |
| A new CC0 texture | Only on a reviewed frame's evidence, under `DEP-60` (≤ 2, ≤ 3 MB each) | Q-S29-10 |

No new dependency. `DEP-60` is closed at C7.

---

## 5. Design

### 5.1 The pieces and how data flows

```text
presentation/mineworld-default/3D/stations/depot.yaml     the kit: dimensions + sources + role tags + doors
        │  StationKit.load(pack_dir, "depot")  → kit | refusal STATION_KIT_* (exit 3 scripted)
        ▼
scripts/worlds/social-cafe/depiction.gd                   the site: { kit: "depot", facade_x, axis_z, facing }
        │  StationDepot.build(root, kit, site)  → node "Station" (mw_category station.*), door entries
        │  depictions() += kit.places → [{tags, volume, ROOM, door}]   (S33-c rule binds them by tag set)
        │  viewpoints() += station perf views, landmark viewpoints, st_1…st_4
        ▼
SliceStreet.build(g, site)                                forecourt from X_MIN to the site's façade line
SliceLayout (16c/S33-c)                                   street doorway → arch door → hall bound by tags
                                                          hall doorways → bay door, trackside door (S29-e draws)
```

The kit is world-independent. The site is the only per-world fact, and it is where the world's doorway already
puts the building: the binding (within 0.15 m) and `--layout` fail by name if the world moves its station.

### 5.2 Real dimensions and how the unknown ones are measured (C1)

| Element | Value | Source / status |
| --- | --- | --- |
| Hall, inside | 16.8 (E–W) × 51.8 (N–S) m | real (§3.2); equals S29-a's floor |
| Hall walls | 0.9 m east (the arch reveal is this depth), 0.6 m elsewhere | design; Mission Revival load-bearing stucco |
| Hall eaves / ridge | ≈ 9 m / ≈ 12 m | approximate; measured in C1 |
| Arch | clear opening ≈ 6 m wide, ≈ 9 m to the crown; two door leaves 0.95 × 2.15 m each (VISUAL_SLICE §3.2) under a glazed window filling the arch | NRHP §7 ("glazed window above the entrance doors"); size measured in C1 |
| Towers | square shafts, ≈ 6.0 m (placeholder), inner faces at the forecourt's edges (Q-S29d-2), east faces 6.0 m forward of the façade; belfry openings; tiled dome with a tiled lantern; the cross emblem on four sides (shader, §5.4) | NRHP §7; plan size and height measured in C1. Height placeholder 30 m (step-29 §6.2) |
| Arcade | 8.2 m deep from the hall's west wall, arched openings on piers 0.6 × 0.6 m at 5.4 m centres, shed roof of Mission tile | real depth (§3.2); pier spacing design (≈ the hall's bay, 51.8 / 10 = 5.18 m), C1 checks against HABS |
| Ticket bay annex | 6.0 × 4.6 m inside, off the hall's south wall, its roof a shed of Mission tile | S29-a (Q-S29a-5) |
| Roofs | gable over the hall (ridge N–S), shed over the arcade and the annex | NRHP §7 |

**Measurement method (C1, R-S29-3).** For each unknown, pick one HABS photograph that shows the element and a
known length in the same plane: a door leaf (2.15 m, VISUAL_SLICE §3.2's design value, flagged), the wainscot
(8 ft = 2.44 m, NRHP; 8½ ft = 2.59 m, Wikipedia — the two sources disagree, both recorded), or the hall's
51.8 m. Measure pixel lengths on the photograph, correct for perspective only where the plane is frontal, and
record per value: the photograph, the reference length, the pixels, the result, and an error estimate (±10 % for
a single frontal measurement). `MEASUREMENTS.md` holds the table; the kit cites it per value. A value that cannot
be measured keeps its placeholder, labelled `"approximate; placeholder (step-29 §6.2)"`.

### 5.3 The kit file (`ARC-192`; `PRESENTATION.md` §5)

`presentation/mineworld-default/3D/stations/<kit>.yaml`, YAML's JSON subset, one object (`DEP-16`). Lengths in
millimetres in the kit's own frames, which are the world's place frames (x east, y north, `CORE_CONCEPTS` §6.1),
so a door position in the kit is literally the passage point a world states.

```text
{ "format": 1,
  "kit": "depot",
  "style": "mission-revival",
  "places": {                                   one entry per drawn place; tags are role tags (CORE_CONCEPTS §6.5)
    "hall":     { "tags": ["station", "landmark"], "volume": "station.hall",
                  "floor": {"min": [0, 0], "max": [16800, 51800]}, "source": "…" },
    "tickets":  { "tags": ["station", "ticket-office"], "volume": "station.tickets",
                  "floor": {"min": [0, 0], "max": [6000, 4600]}, "source": "…" },
    "platform": { "tags": ["station", "platform"], "volume": "station.platform",
                  "floor": {"min": [0, -30000], "max": [11000, 30000]}, "source": "…" } },
  "doors": [                                    each joins two kit places, or a place and "outside"
    { "name": "arch",      "from": "outside", "to": "hall",     "at": [16400, 25900], "wall": "east",
      "leaves": 2, "leaf_mm": [950, 2150] },
    { "name": "bay",       "from": "hall",    "to": "tickets",  "at": [8400, 400],    "wall": "south",
      "there": [3000, 4200], "leaves": 1, "leaf_mm": [950, 2150] },
    { "name": "trackside", "from": "hall",    "to": "platform", "at": [400, 25900],   "wall": "west",
      "there": [10600, 0], "leaves": 2, "leaf_mm": [950, 2150] } ],
  "shell":  { "walls_mm": {"east": 900, "other": 600}, "eaves_mm": …, "ridge_mm": …, "source": "…" },
  "arch":   { "clear_mm": [6000, 9000], "window": true, "source": "…" },
  "towers": { "plan_mm": [6000, 6000], "forward_mm": 6000, "inner_offset_mm": 8100,
              "shaft_mm": …, "dome_mm": …, "lantern_mm": …, "source": "…" },
  "arcade": { "depth_mm": 8200, "pier_mm": 600, "pitch_mm": 5400, "length_mm": 60000, "source": "…" },
  "tile":   { "dome": "zigzag", "lantern": "zigzag", "emblem": "cross", "floor": "quarry" },
  "fitout": { … }                               S29-e's section (benches, kiosk, counter, boards, clock)
}
```

- **Every number group has a `source`** string: a citation, or `"design; …"`, or `"approximate; …"`. A group with
  no source is refused `STATION_KIT_UNCITED` (the realistic-defaults rule made mechanical, as RL-e0's
  `FAR_FIELD_UNCITED`).
- **Refusals** (`StationKit`, printed `[mineworld-slice] <CODE>: <path>: <detail>`, full-screen label, `quit(3)`
  scripted — RL-e0's pattern): `STATION_KIT_MISSING` (no file for the kit a depiction names),
  `STATION_KIT_MALFORMED` (not JSON, wrong types, unknown key, a door naming an unknown place, `format` ≠ 1),
  `STATION_KIT_UNCITED`, `STATION_KIT_OUT_OF_RANGE` (a door leaf outside VISUAL_SLICE §3.2; a floor not positive;
  a door point not 400 mm inside its wall within 1 mm, which is 12d's authoring rule, A-3).
- **Who reads it.** Only the 3D client. It is presentation: it changes what is drawn, never which requests the
  client sends (`PRESENTATION.md` §4). `mineworld packs validate` does not read it (Q-S29d-3).
- **Why the world's frames are repeated in the kit.** The depot must be drawn before anything is observed (the
  standalone slice and the first frame of a connected one). The repetition is checked, not trusted: the binding
  fails by name if a world's doorway is more than 0.15 m from the kit's drawn door (A-3), and 12e's probe fails
  if a drawn wall is more than 150 mm from a disclosed edge (A-13).

### 5.4 The tile shader (`glazed_tile.gdshader`; `StationTile`)

- **One shader, one material per pattern**, built once and cached in `StationTile.material(pattern, palette)`.
  Patterns: `zigzag` (domes and lanterns: chevron bands of blue, green, yellow and white glazed tile, ≈ 150 mm
  tiles, a dark grout line), `emblem` (the Santa Fe cross, a white cross on a blue square, computed in the shader
  from UV, so no decal and no texture), `frieze` (S29-e: the wainscot's geometric field and ziggurat frieze), and
  `quarry` (the floor's clay tile in two sizes and two colours, NRHP).
- **Inputs:** UV in tile units from the mesh, `pattern`, the four glaze colours from `SlicePalette` (warm,
  slightly desaturated, so the depot is "within the pack's palette", Q-S29-5), roughness 0.25 glazed / 0.8
  quarry, a per-tile hash for ±4 % value variation (no two tiles identical, the realism rule).
- **No texture, no `SCREEN_TEXTURE`, no compute.** One material for all tile surfaces of one pattern, so batching
  keeps the draw count to one per pattern per cell.
- **Client rules.** A shader writes only colour; the scan (A-15) covers GDScript, and the shader file contains no
  intent, action or rule word. The step's adversarial "a tile shader that writes to an intent fails the
  client-rules scan" holds by construction: a GDScript caller that sent an intent from `StationTile` would be
  flagged by the existing action-literal check.

### 5.5 The building (`StationDepot.build(root, kit, site) -> Node3D`)

`site = { kit: String, facade_x: float, axis_z: float, facing: "east" }`. `facing` is the direction the arch
faces; only `"east"` is implemented, and any other value is refused by name (`STATION_SITE_FACING`), because a
binding cannot rotate (A-3) and a world with a north-facing station has frames written for it (§10).

The hall frame's (0, 0) is at scene `(facade_x − walls.east − floor.max.x/1000, y, axis_z + arch.at.y/1000)`, and a
kit point (x, y) mm maps to `origin + (x/1000, 0, −y/1000)`. Everything below is placed through that one function.

1. **Hall shell.** Four walls with thickness (VISUAL_SLICE §6.1), `painted_plaster_wall` warm white; a plinth
   course of `sandstone_blocks_05`; arched clerestory openings near the eaves (glass from `Mats`), nine per long
   wall (NRHP's "nine arched elements" read as nine bays of clerestories; S29-e places the nine interior arches);
   the floor in `quarry` tile at the forecourt's level (`WALK_Y`, a flush threshold); a plain ceiling plane at the
   eaves (S29-e replaces it with beams and purlins). The trackside door and the bay door are openings with real
   leaves (VISUAL_SLICE §3.2); further trackside doors are drawn **closed** (leaves, no opening), because the world
   has one passage.
2. **The arch.** In the east wall on the axis: the clear opening, a 0.9 m reveal (the wall's thickness), two door
   leaves, a glazed window filling the arch above the transom, a moulded archivolt; above it a curvilinear Mission
   parapet rising between the towers.
3. **The towers.** Square stucco shafts beside the forecourt, inner faces `inner_offset_mm` from the axis (8 100:
   the kit's source cites the 12d street's walkable half-width, so the faces stand on the forecourt's disclosed
   edges; a world with a wider street needs a kit variant, and 12e's probe catches a mismatch), east
   faces `forward_mm` in front of the façade; belfry openings near the top; a drum; a dome in `zigzag` tile; a tiled
   lantern; the cross emblem on four faces of the drum. Each tower is one `Node3D` with
   `mw_category = "station.tower"`, so X-6 hides exactly them.
4. **The arcade.** Along the hall's west wall, the kit's length centred on the trackside door: piers, round arches,
   a shed roof of `clay_roof_tiles_02` on timber rafters (`brown_planks_09`, tinted redwood). The piers sit at the
   kit platform's x 2 200 … 2 800 (S29-a writes them as solids, F-S29d-3).
5. **The ticket-bay annex.** Its walls, roof and the bay door opening; its interior is S29-e's.
6. **Roofs.** The hall's gable in `clay_roof_tiles_02` with exposed rafter tails; the annex and arcade sheds.
7. **Volumes.** One `Area3D` per kit place (`station.hall`, `station.tickets`, `station.platform`), each the place's
   floor rectangle (from the kit) by 4 m high, at the place's scene origin; the platform's volume is created here so
   that S29-d's walk can end on it, its slab and track are S29-e's.
8. **Doors.** Three registry entries (A-4), in the kit's order: arch (`volume` `station.hall`, depth 0.9 m, floor
   height 0), bay (`station.tickets`, 0.6, 0), trackside (`station.platform`, 0.6, 0). Each in-point and out-point
   is 0.4 m from its wall's faces (`DOOR_INSET`), so the world's passage points coincide.
9. **Categories.** The root `Station` node: `mw_category = "station"`; children `station.hall`, `station.tower`,
   `station.arcade`, `station.roof`, `station.annex`. `SliceBatch` keeps them (F-S29d-6).
10. **Occluders.** Boxes inset 0.02 m inside: each tower shaft, the hall's east and west walls between openings,
    the parapet. `SliceOccluders`' glazing and door-overlap checks run on them unchanged.
11. **Colliders.** Walls, tower shafts and arcade piers on `Build.LAYER_WORLD`, from the same numbers; nothing
    else collides on walkable floor.

### 5.6 The forecourt (`SliceStreet`)

`SliceStreet.build(g, site := {})`: with an empty site, `_surrounds` is today's (byte-for-byte; S33-c's equality
gate holds for any world without a station). With a site, the west end becomes:

- **Surface.** Setts at `WALK_Y` from x `X_MIN` to the façade, z −8.1 … +8.1 (the disclosed floor); the
  carriageway ends at x `X_MIN` with a dropped kerb across its width (a 22° hidden ramp, `street.gd`'s own kerb
  pattern), so the forecourt reads as a plaza, not a road.
- **Edges.** On the disclosed floor edges z = ∓ 8.1, from `X_MIN` to the towers' east faces: low stucco sitting
  walls 0.45 m high and 0.4 m thick (their inner face on the edge, ±150 mm, A-13), each with planting beds and two
  potted palms behind them (shipped props only, listed at C0); between the towers the tower walls are the edge.
- **Beyond the edges** (z ±8.1 … the hall's ends): planted beds and gravel on the far field's near ground, never
  walkable (no disclosed floor).
- **Furniture.** Two lamps, a bench pair against the sitting walls (outside the walkable band), bollards at the
  carriageway's end. All shipped.

### 5.7 Binding by role tag (with S33-c)

The Social Café depiction adds, from the kit, one entry per kit place: `{tags: kit.places.<p>.tags, volume,
drawn_as: ROOM, door: <its registry index>}`. S33-c's rule then binds:

- the hall when the street's passage lands on the arch door and the place carries `station` and `landmark`;
- the ticket office when the hall's passage lands on the bay door and the place carries `station` and
  `ticket-office` (the most tags win over the hall's two);
- the platform likewise, by `station` and `platform`.

A world whose station is not tagged `landmark` (a world that declares another landmark) does not match the depot
and gets a reported miss, never a guess: the depot depicts a landmark station. No GDScript literal names a tag
or key (F-S29d-7).

### 5.8 Viewpoints (Social Café depiction; computed from the site, not typed)

| Name | Pose (scene; yaw as Godot: 0 north, 90 west) | Use |
| --- | --- | --- |
| `st_1_spawn_west` | spawn (−11.5, 1.8, −5.55), yaw 90, pitch 4 | shot; LM-1 |
| `st_2_east_end_west` | (30, 1.8, 0), yaw 90, pitch 2 | shot; LM-1 |
| `st_3_forecourt` | (`facade_x` + forward + 15, 1.7, 0), yaw 90, pitch 8 | shot; perf `station forecourt` |
| `st_4_arch_threshold` | (`facade_x` − 0.45, 1.7, 0.6), yaw 90 then 270 (two frames, both sides) | shot |
| `station hall wide` | the hall's north-east inner corner, 1.7 m, looking south-west | perf |
| `station platform` | on the platform at its north end, 1.7 m, looking south along the edge | perf (S29-e's slab) |
| LM-2 | `ff_5_west_aerial` (RL-e0) | LM-2 |

`st_1`–`st_4` are committed to `clients/3d-spike/shots/social-cafe/station/` (960 × 540, RL-e0's shot settings).

### 5.9 The landmark check (`--slice-landmark`, `SliceLandmarkCheck`)

RL-e0's harness, three renders per viewpoint at fixed constants (Q-FF-1's rule: fixed before any run):

- **S** — everything; **H** — every node whose `mw_category` begins with the check's prefix (`station` here;
  `landmark` for S33-e's pier) hidden; **F** — every near node hidden (only sky and far field).
- **Open in H**: a pixel where H and F do not differ (`DIFF` 8): sky or far field shows there with the depot gone.
- **LM-1** (each of `st_1`, `st_2`): in the azimuth window 270° ± 10° (compass), between the horizon row and the
  band's top row, `|{p open in H : S(p) ≠ H(p)}| / |open in H| ≥ 0.60`. The band's top is the elevation of the
  lower of the two frontage eaves at the street's west end, seen from the viewpoint; the depiction states it per
  landmark viewpoint (`viewpoints().landmark: [[name, pos, yaw, pitch, azimuth_deg, band_top_deg]]`), computed
  once from its frontage numbers and fixed before the first run.
- **LM-2** (`ff_5`): the depot's footprint polygon (the kit's outline from the site, projected) covers N pixels;
  `|{p in footprint : S(p) ≠ H(p)}| / N ≥ 0.60`.
- **Verdicts:** `PASS landmark <view> LM-<n> <fraction>` / `FAIL …`, then `all landmark checks pass` or
  `<n> LANDMARK CHECKS FAILED` (RL-e0's shapes; `ci_godot.py judge_probe` reads them).
- **CI:** `three_d_landmark_closes_the_west_view` in `tools/cli/tests/client_far_field.rs`, `#[ignore]`d, in the
  `farfield` layer (Linux per PR when that layer runs; three OSes nightly). Under SPEED FIRST the verdict is
  recorded at the milestone.

### 5.10 Performance and size

- **Levers built in (step-29 §6.6, as they exist, F-S29d-10).** All depot primitives batched (kept category,
  24 m cells); one material per tile pattern; occluders for towers and long walls; `SliceBudget` culling for the
  small props; no real-time light added by the exterior.
- **Counts.** `--slice-perf` on the three new views and on every M-1 view that sees the depot (`street wide`,
  `skyline east` is east-facing and unaffected, `street east`, `south side`): draw calls, primitives and video
  memory, base (C0, before) and head (C6, after), recorded. Caps (step-29 §6.6): ≤ +40 draws, ≤ +150 000
  primitives, ≤ +48 MB per existing view; absolute gates in new views. A count cap exceeded is a material stop.
- **Timings.** INCONCLUSIVE locally (R-LOAD-1); the operator's three-run protocol at the milestone (AC-S29-6).
- **Size.** Zero new textures; the shader and scripts are text (≈ 0.2 MB estimated). The kit and the references
  live under `presentation/`, outside the `res://` export (A-16's loader reads the pack directory), so they add
  nothing to `3d.pck`; C0 confirms the export filter, and C7 records `3d.pck`'s size before and after (S23 R-a's
  export if merged, else a local `--export-release`, through `with-slot.sh`).

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-d-1 (I-S29-1) | No path under `kernel/`, `contracts/`, `systems/`, `server/`, `worlds/`, `clients/protocol/`, `cognition/` changes | C7 diff scan |
| I-d-2 (I-S29-5) | One method: procedural GDScript, shipped CC0 materials, our shader; no new texture unless `DEP-60` records it | C7 file list; `DEP-60` |
| I-d-3 (I-S29-6) | Every kit number group has a `source`; measured values cite `MEASUREMENTS.md`; the compressions (distance, platform length) are stated | `STATION_KIT_UNCITED`; review |
| I-d-4 | The client keeps no place key and no role-tag literal: the kit's tags are data; binding is S33-c's tag-set rule | S33-c's check (d); C4 review |
| I-d-5 | The depot is world-independent: it is built from the kit and a site; a renamed-key world binds identically | AC-d-3 |
| I-d-6 (I-S29-7) | RL-e0's rendered check stays green; the depot stands inside `reach_m` and keeps clear of the skyline | AC-d-6 |
| I-d-7 | Nothing drawn stands on walkable floor without a disclosed solid; every disclosed edge of the forecourt and hall has a drawn collider within 150 mm | 12e's `--slice-geometry` (AC-d-4) |
| I-d-8 | A world without a station site is drawn exactly as before | S33-c's pixel-equal gate re-run on a no-site build (AC-d-7) |
| I-d-9 (I-S29-8) | Step-29 §6.6's count caps hold | AC-d-8 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | `client_rules.rs` (action literals, rule words, path literals; S33-c's key scan); `check_decision_ids.py`; `check_doc_headings.py` | CI `fast`; local per commit |
| Headless Godot (per commit, local, through `with-slot.sh`) | the kit loads and each refusal code fires on a planted file; `--slice-station-kit` builds `depot` and the reduced-hall variant and prints its measure rows; the shader compiles on import | `--slice-station-kit` |
| Rendered / connected (**milestone**) | `--slice-measure` depot rows; `--world --link` walk; `--layout`; `--slice-geometry`; `--slice-landmark`; RL-e0's `--slice-farfield`; `--slice-perf` counts; shots | `client_far_field.rs` (ignored), `clients-probes` layer |
| Gate 1 (real LLM) | NOT REQUIRED | — |
| Gate 2 (real lifecycle) | the connected walk on a hosted server (milestone) | `mineworld-slice --world --link` |

---

## 8. Acceptance criteria (fixed before measuring)

"Second world / variant" names the case beyond Social Café that each criterion must also pass. **M** = run at the
milestone (SPEED FIRST); **P** = run in the PR.

| ID | Criterion | Second world / variant | When | Step id |
| --- | --- | --- | --- | --- |
| AC-d-1 | The kit loads; each of `STATION_KIT_MISSING`, `_MALFORMED`, `_UNCITED`, `_OUT_OF_RANGE` and `STATION_SITE_FACING` is printed by name for one planted file, and the slice exits 3 scripted | the reduced-hall variant kit (`depot` with `hall.floor` 12 000 × 30 000, doors moved to its wall midpoints) loads and builds | P | I-S29-6 |
| AC-d-2 | `--slice-station-kit=depot` builds the depot in an empty scene and prints, within tolerance: hall inside 16.8 × 51.8 m (± 50 mm); every door leaf in 0.85–1.00 × 2.05–2.20 m; each door's in- and out-points 0.40 m (± 10 mm) from its wall faces; towers' inner faces at `inner_offset_mm` from the axis (± 50 mm); the measured tower height printed beside its source | the reduced-hall variant: the same rows hold, the towers still flank the arch, the arcade spans the variant's length | P | §6.1 "one precision" |
| AC-d-3 | Connected (`./mineworld-slice --world --link`): from spawn, walk to and through the arch; the server's place changes from the street to the hall; `SHOWN` lists the hall `drawn_as` `room` with tags incl. `station`, `landmark`; 0 refusals; `--layout` reports 0 misses for the hall | (a) a **renamed-key copy** of Social Café (`station` → `depot`, `station-tickets` → `depot-window`, `station-platform` → `depot-platform`, same tags, made by the Rust test in a temp dir) binds identically; (b) **Market Town** served and joined with `--server` and `--world-key=social-cafe`: the hall binds, 0 misses for station places | M | AC-S29-3 (first leg) |
| AC-d-4 | `--slice-geometry` (12e) over the street (forecourt) and the hall: every disclosed edge meets a drawn collider within 150 mm, and no drawn collider lies on walkable floor | Market Town joined (b above) | M (N/A if 12e unmerged) | §6.5, I-d-7 |
| AC-d-5 | LM-1 at `st_1` and `st_2`, and LM-2 at `ff_5`, each ≥ 0.60, on Linux, macOS and Windows | the reduced-hall variant built at the same site (`--slice-landmark --station-kit=<variant>`): its fractions are recorded (no threshold), to show the check measures the building, not a constant | M | AC-S29-4 |
| AC-d-6 | RL-e0's `--slice-farfield` still prints `all far-field checks pass`; `ff_1`–`ff_7` re-shot and committed; `ff_5` and `ff_6` show the depot; `ff_6` still shows the bay to the south-west | the renamed-key copy (a) yields identical `ff_*` frames within `frame_diff`'s 8/255 | M | I-S29-7; §9.1 |
| AC-d-7 | A build with no station site is pixel-equal to the base for every existing Social Café shot (S33-c's gate) | — the "variant" here is the absence of a site | P (offline shots) | I-d-8 |
| AC-d-8 | Counts within §5.10's caps in every M-1 view that sees the depot; new views under the absolute gates; `3d.pck` grows by ≤ 6 MB (target ≈ 0.2 MB); timings recorded by the operator | Market Town joined: the `station forecourt` view's counts recorded | M | AC-S29-6 |
| AC-d-9 | `st_1`–`st_4` committed; the operator's look recorded in `HUMAN_REVIEW_QUEUE.md` | — | P (commit), M (look) | §9.1 |

### 8.1 Planted mutations (each must fail by name, then be reverted from a byte copy)

| Id | Plant | Must fail with | When |
| --- | --- | --- | --- |
| **X-6** | hide the towers (`mw_category station.tower` nodes `visible = false` in `StationDepot`) | `FAIL landmark st_1 LM-1` (Q-S29d-4; at `st_2` the hall alone holds, F-S29d-11) | M |
| M-d-1 | delete `source` from the kit's `towers` group | `STATION_KIT_UNCITED: … towers` | P |
| M-d-2 | move the kit's arch door 0.5 m north | `--layout`: the hall is a miss ("no drawn door within 0.15 m") | M |
| M-d-3 | write `"station"` as a literal in `depot.gd` | S33-c's check (d) names the file and line | P (`fast`) |
| M-d-4 | drop the kept-prefix rule from `SliceBatch` | LM-1 fails at every viewpoint (H hides nothing: fraction 0) | M |
| M-d-5 | remove one forecourt sitting wall | `--slice-geometry`: the disclosed edge z −8.1 has no collider within 150 mm | M (12e) |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason. Per-commit validation
is the build and the tests the commit writes (SPEED FIRST); rows marked M in §8 are recorded at the milestone.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s29d`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites, and every file you will edit, in full.
2. Make the numbered steps, in order. Do not do a later commit's work.
3. Godot and cargo only through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`. Record per command:
   what ran, the count of rows, PASS / FAIL / INCONCLUSIVE.
4. Static: `python3 scripts/check_decision_ids.py`; `python3 scripts/check_doc_headings.py`; the client-rules test
   `cargo test -p mineworld-acceptance --test client_rules` (one file) where GDScript changed.
5. `git status --short` and `git diff --stat`: every path is in the commit's **Files**, or revert it or stop.
6. Update §15 and `handoff-s29d.md` in the same commit.
7. Stage by explicit path; `git commit -m "<message>"` ending with
   `Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>`. No heredoc writes, no `sed -i`, no
   `python3 -c`, no `curl`.
8. A material stop (§12) is reported, not improvised around.

#### C0 — Base, prerequisites, ledger (Markdown only)

- **Files:** this file (§15); `.structured-coding/plans/mvp0/handoff-s29d.md` (new).
- **Steps:**
  1. Record the base commit; quote the dispatch confirming Milestone F's checklist was handed over (missing →
     stop).
  2. Record the merge state of S29-a, S29-c, 16c, S33-c, 12e, RL-b2. S29-a: confirm R-S29a-d-1 in the merged
     place files (the bay's door at (3 000, 4 200) and counter at y 0 … 600; the platform floor x 0 … 11 000,
     y −30 000 … 30 000, door (10 600, 0), ten pier solids) — otherwise stop.
  3. Re-read A-1 … A-22 on the base; record moved lines (bounded) or changed statements (stop if a decision here
     changes). Read the street's scene origin from `--layout`'s SHOWN (`origins`) on the base and record the
     façade line `facade_x` (expected ≈ −59.0).
  4. List the shipped Poly Haven props (palm, lamp, bench, bollard) available for §5.6.
  5. Confirm the export filter leaves `presentation/` out of `3d.pck` (A-16; §5.10).
  6. Record base perf counts for the M-1 views (`--slice-perf`, one run, counts only).
- [ ] Implementation. [ ] Validation: the two doc scripts. [ ] Review: every start condition holds; §2.3's
  branches chosen.
- **Commit message:** `docs(plan): S29-d — open the ledger at the base`

#### C1 — Records, references and measurements

- **Goal:** the decisions and the evidence exist before the code they govern (`CLAUDE.md` §2.2).
- **Files:** `docs/decisions/ARC-192.md` (new), `docs/decisions/ARC-106.md` (note), `docs/decisions/INDEX.md`
  (regenerated), `clients/3d-spike/PRESENTATION.md` (§5), `docs/MODULE_SPEC.md` (§6.1 list line),
  `docs/ART_DIRECTION.md` (Architecture: the landmark exception),
  `presentation/mineworld-default/3D/references/station/{PROVENANCE.md, MEASUREMENTS.md, <≤ 6 photographs>,
  <data pages PDF>}`.
- **Steps:**
  1. `ARC-192` in the record format (`## ARC-192 — A station is drawn from a station kit, placed at a site, bound
     by role tag`): problem (the depot must be generic, F-S29d-12); options ((a) a GDScript building for Social
     Café — declined, one town hard-coded; (b) a building generated from disclosure alone — declined, nothing is
     disclosed before the first observation and the exterior must be drawn from spawn; **(c) a kit file in the
     Presentation Pack, a site per depiction, binding by tag set and doorway — chosen**); the choice (§5.3, §5.5,
     §5.7, the refusal codes, Q-S29d-3's answer); consequences (S29-e's pieces, S33-e's halt kit, the kept-prefix
     batching rule).
  2. Note under `ARC-106`: the measured values (tower plan and height, arch, eaves and ridge) with their error;
     the towers beside the forecourt (Q-S29d-2); the hall centroid's bearing 270° (F-S29d-5).
  3. `PRESENTATION.md` §5: the kit format (§5.3), what the client does with each key, the refusals, "what a kit
     may not do" (decide reachability or availability). `MODULE_SPEC.md` §6.1: one line `stations/<kit>.yaml`
     "station kits for the 3D reference client (`PRESENTATION.md` §5, `ARC-192`)".
  4. `ART_DIRECTION.md`: a dated subsection "The landmark exception (2026-10-…, step-29 §14.4)": one signature
     building per town may follow its own vernacular, within the pack's materials, palette, light and precision.
  5. Fetch the references (a small script under the lane's scratch directory; never `curl`): LoC's JPEG
     derivatives of at most six HABS CA-1965 photographs (the front with both towers, one tower, the arch, the
     waiting room, the wainscot, the track side) and the data pages; if LoC refuses automated fetches (A-22),
     Wikimedia Commons' copies of the same HABS files, each verified by its HABS call number in the file
     description. `PROVENANCE.md`: per file, URL(s), LoC call number, photograph number and caption, SHA-256,
     size, "public domain (US federal work, HABS); DEP-8".
  6. `MEASUREMENTS.md`: §5.2's method and table.
- **Validation:** `check_decision_ids.py --write-index` then `check_decision_ids.py`; `check_doc_headings.py`;
  `shasum -a 256` of each reference equals `PROVENANCE.md`.
- [ ] Implementation. [ ] Validation. [ ] Review: no new ontology term; every measured value cites a photograph
  and a reference length; no CC BY-SA file committed (Q-S29-8).
- **Commit message:** `docs: the station kit — ARC-192; HABS CA-1965 references and measurements (S29-d)`

#### C2 — The kit and its reader

- **Files:** `presentation/mineworld-default/3D/stations/depot.yaml` (new), `clients/3d-spike/scripts/station/kit.gd`
  (+ `.uid`), `clients/3d-spike/scripts/slice/slice_probe.gd` (mode `station-kit`, read-only first: load and
  print), a planted-kit corpus `clients/3d-spike/tests/station_kits/` (one file per refusal code).
- **Steps:**
  1. `depot.yaml` per §5.3 with C1's values; the `fitout` key absent (S29-e adds it; the reader accepts its
     absence).
  2. `StationKit.load(pack_dir, kit) -> StationKit` with `error_code`, `error_detail`, `path`, typed getters
     (`place(name)`, `doors()`, `shell()`, …) and `refusal_line(client)`; checks in a fixed order: MISSING →
     MALFORMED → UNCITED → OUT_OF_RANGE.
  3. `--slice-station-kit[=<kit>]`: loads the kit, prints one `KIT` line per group and each refusal.
- **Validation:** headless: `depot` loads; each corpus file prints its code; exit 3 on refusal.
- [ ] Implementation. [ ] Validation. [ ] Review: no rule word in a constant name (A-15); the reader never reads
  `worlds/`.
- **Commit message:** `feat(3d): station kits — the depot's data file and its reader (S29-d)`

#### C3 — The glazed-tile shader

- **Files:** `clients/3d-spike/shaders/glazed_tile.gdshader` (+ `.uid`), `clients/3d-spike/scripts/station/tile.gd`
  (+ `.uid`).
- **Steps:** the shader per §5.4 (patterns `zigzag`, `emblem`, `quarry`; `frieze` declared and implemented, used
  by S29-e); `StationTile.material(pattern)` cached; `--slice-station-kit` renders a 1 m test tile per pattern to
  `shots/social-cafe/station/review/tile_<pattern>.png` (not committed; for the frame review).
- **Validation:** headless import compiles the shader with no error line; four review tiles written.
- [ ] Implementation. [ ] Validation. [ ] Review (Opus frame review): the zigzag reads as glazed tile at 5 m and
  at 40 m; colours inside the pack's palette.
- **Commit message:** `feat(3d): glazed tile — one shader for domes, lanterns, wainscot and floor (S29-d)`

#### C4 — The depot exterior, the forecourt and the binding

- **Files:** `clients/3d-spike/scripts/station/depot.gd` (+ `.uid`), `clients/3d-spike/scripts/slice/street.gd`,
  `clients/3d-spike/scripts/slice/batch.gd`, `clients/3d-spike/scripts/slice/occluders.gd`,
  `clients/3d-spike/scripts/worlds/social-cafe/depiction.gd`.
- **Steps:**
  1. `StationDepot.build(root, kit, site)` per §5.5 items 1–11.
  2. `SliceStreet.build(g, site)` per §5.6; an empty site keeps `_surrounds` byte-for-byte.
  3. `SliceBatch`: `KEPT_PREFIXES := ["station.", "landmark."]`; the group key gains the nearest kept ancestor
     category; the merged mesh carries it (F-S29d-6).
  4. `SliceOccluders`: the depot's boxes (§5.5 item 10).
  5. The Social Café depiction: `STATION_SITE := {kit: "depot", facade_x: <C0's value>, axis_z: 0.0,
     facing: "east"}` with a comment citing S29-a's passage; it loads the kit (refusal → exit 3), calls
     `SliceStreet.build(g, site)` and `StationDepot.build`, adds the kit's entries to `depictions()` (§5.7), and the
     viewpoints of §5.8 to `viewpoints()` (`perf`, `landmark`, a new `station_shots` key).
- **Validation:** offline `./mineworld-slice` builds with no error; `--slice-station-kit=depot` prints the depot's
  rows (AC-d-2) and the variant's; offline shots of the existing views with the site removed equal the base
  (AC-d-7).
- [ ] Implementation. [ ] Validation. [ ] Review (Opus frame review of offline frames from `st_1`–`st_4`'s poses):
  the towers close the street; proportions plausible against the HABS front photograph; no z-fighting; no
  geometry on the walkable band but the sitting walls' inner faces.
- **Commit message:** `feat(3d): the Santa Fe depot — towers, arch, hall shell, arcade, roofs and the forecourt (S29-d)`

#### C5 — Measure, geometry and the walk

- **Files:** `clients/3d-spike/scripts/slice/slice_probe.gd` (`measure` rows), 12e's
  `clients/3d-spike/scripts/slice/slice_probe_geometry.gd` (the station's places in its place list; only if 12e
  merged), the probe route in the depiction (`probe_route` gains `[station, landmark]`), a Rust ignored test
  `tools/cli/tests/client_far_field.rs::three_d_station_binds_by_tag` (serves the renamed-key copy and Market
  Town; runs the `--link` walk and `--layout`).
- **Steps:** the rows of AC-d-2 in `--slice-measure`; the geometry probe's place list read from `SHOWN` (it then
  covers the hall with no new key); the renamed-key copy made by the Rust test (copy `worlds/social-cafe` to a
  temp dir, rename three place files and every reference to them, which are the three keys in `world.yaml`,
  passages and routines).
- **Validation:** local, one connected run through `with-slot.sh`: spawn → arch → hall, the place change printed
  (AC-d-3 first case). The variants and geometry: M.
- [ ] Implementation. [ ] Validation. [ ] Review: the renamed copy changes keys only (diff of the copy against
  the original shows only those names).
- **Commit message:** `test(3d): the depot's measure rows, geometry rows and the walk into the hall (S29-d)`

#### C6 — Perf views, the landmark check and the shots

- **Files:** `clients/3d-spike/scripts/slice/slice_landmark_check.gd` (+ `.uid`), `slice_probe.gd` (mode
  `landmark`, mode `station-shots`), `mineworld-slice` (`--landmark` → `--slice-landmark`, as RL-e0's
  `--farfield`), `tools/cli/tests/client_far_field.rs` (`three_d_landmark_closes_the_west_view`),
  `scripts/ci_layer.py` (the test in the `farfield` layer), `clients/3d-spike/shots/social-cafe/station/st_1…st_4`,
  `clients/3d-spike/shots/social-cafe/farfield/ff_1…ff_7` (re-shot).
- **Steps:**
  1. Recompute F-S29d-11's arithmetic with C1's measured heights; write the band tops into
     `viewpoints().landmark`; record the arithmetic in §15 **before** the first landmark run.
  2. `SliceLandmarkCheck` per §5.9, prefix a parameter (`station`; S33-e passes `landmark`).
  3. `--slice-station-shots` writes `st_1`–`st_4`; `--slice-farfield-shots` re-shoots `ff_*`.
  4. `--slice-perf`: counts for the three new views and the affected M-1 views.
- **Validation:** local, one run each through `with-slot.sh`: the landmark verdict lines (recorded; a FAIL is a
  material stop only at the milestone run on three OSes); counts against the caps (a cap exceeded is a material
  stop now).
- [ ] Implementation. [ ] Validation. [ ] Review (Opus frame review of `st_1`–`st_4` and `ff_5`, `ff_6`): the
  landmark reads from spawn and from the east end; the bay still visible in `ff_6`.
- **Commit message:** `feat(3d): the landmark check, station views and shots; far-field shots re-taken (S29-d)`

#### C7 — Close-out

- **Files:** this file (§15), `docs/decisions/DEP-60.md` (withdrawn, or the texture recorded),
  `docs/HUMAN_REVIEW_QUEUE.md` (one row), `handoff-s29d.md`.
- **Steps:** `DEP-60`'s status; the queue row (st shots and both `ff_*` sets for the operator's look at the
  milestone); `3d.pck` size before and after; the I-d-1 diff scan; push; open the PR; record `fast` on the exact
  head; report MERGE-READY with the PR number and head SHA.
- [ ] Implementation. [ ] Validation: `fast` green on the exact head. [ ] Review: every §8 row is either evidenced
  or marked M with its command.
- **Commit message:** `docs(plan): S29-d — close-out, DEP-60, review queue`

---

## 10. Requirements this PR places on other PRs

- **R-S29a-d-1 (on S29-a, before its freeze; Q-S29d-1).** Write the towns' station places so that binding by
  translation draws them (F-S29d-1 … F-S29d-3):
  - `station-tickets` (social-cafe, market-town): `here` (3 000, **4 200**) / `station` there (8 400, 400)
    unchanged; the counter solid at **y 0 … 600** (the far, south wall), height 1 050. Capacity unchanged (54).
  - `station-platform` (social-cafe, market-town): floor **x 0 … 11 000, y −30 000 … 30 000**; origin (0, 0) on
    the **west** (track) edge, mid-length; `here` (**10 600, 0**) / `station` there (400, 25 900) unchanged; solids:
    ten arcade piers **x 2 200 … 2 800**, y centred at ±2 700, ±8 100, ±13 500, ±18 900, ±24 300 (600 mm square,
    height 4 000), and the two benches against the hall wall at x 10 100 … 10 700 (seat 450), y −10 000 … −8 100
    and 8 100 … 10 000. The comment states the arcade (8.2 m, real) and the 2.8 m open strip.
  - Lakeside: S33-e's R-S29a-e-1 (the bay north of the waiting room) is the same defect; this PR supports it.
- **R-S29c-d-1 (on S29-c).** Social Café's `station` slot: bearing 270° and distance ≈ 68 m (F-S29d-5), footprint
  ≥ 60 m; `reach_m` 100 contains the platform's far corner (≈ 93 m) and the near track.
- **R-S29e-d-1 (on S29-e).** Use `StationKit` for every fit-out number (a `fitout` group with sources), the
  volumes and doors this PR creates, the `station.*` categories, and `viewpoints()`; add `st_5`–`st_8` to
  `station_shots`.
- **R-S33e-d-1 (on S33-e).** `SliceLandmarkCheck` takes a category prefix and the depiction's landmark viewpoints;
  `SliceBatch` keeps `landmark.*`. A halt is a second kit (`stations/halt.yaml`) or S33-e's own builder using
  S29-e's pieces; either way its role tags are data.

## 11. Edits to parent documents (owner: the lane lead / primary session)

| Document | Edit |
| --- | --- |
| `step-29-station-landmark.md` §6.2 | the towers stand beside the forecourt (Q-S29d-2); the tower height measured (C1) |
| `step-29-station-landmark.md` §6.3 | the hall centroid is at 270° (F-S29d-5) |
| `step-29-station-landmark.md` §6.6 | the "café visibility pattern" lever does not exist (F-S29d-10) |
| `step-29-station-landmark.md` §9.5 | X-6 fails at `st_1` (Q-S29d-4) |
| `step-29-station-landmark.md` §11.1 | S29-d depends on S33-c, not on RL-b2 (Q-S29d-5); file names (F-S29d-8) |
| `pr-s29a-town-contract.md` §5.5 | R-S29a-d-1 (revision 4). **Done 2026-10-19:** revision 4 (branch `docs/pr-design-s29a-rev4`, §14.5) writes these frames, pending its Q-S29a-8 |

**Fallback if Q-S29d-5 is ruled "do not wait for S33-c".** C4 adds the depot's three entries to 16c's binding by
generalising `_bind_street_doorways` to any bound room (the S33-c rule's step 4 alone), and S33-c then moves the
site and entries into the depiction with its equality gate. This costs one rebase for whichever lands second.

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S29-d — the 3D depot exterior: station kit, towers, arch, hall shell,
                         arcade, roofs, forecourt, binding by role tag, the landmark check
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s29d-depot-exterior.md
RELATED / BINDING DOCS:  step-29-station-landmark.md (§§1, 2.2, 6, 9.1, 9.2, 9.5, 10, 11, 13, 14.4);
                         pr-s29a-town-contract.md §5.5 (as revised per R-S29a-d-1); pr-s33c-world-seam.md;
                         pr-s29e-depot-interior.md (R-S29e-d-1); docs/VISUAL_SLICE.md §§3, 4, 7.1, 12;
                         docs/MODULE_SPEC.md §6.1; clients/3d-spike/PRESENTATION.md; docs/decisions ARC-45,
                         ARC-102, ARC-103, ARC-105, ARC-106, ARC-107, ARC-144, DEP-8, DEP-16, DEP-60;
                         .structured-coding/standards.md; CLAUDE.md; docs/ENGINEERING_STANDARDS.md;
                         docs/ENGINEERING_RULES.md; /Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s29d, its own, held by one session
BRANCH:                  mvp0/pr-s29d, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator AND S29-a merged with R-S29a-d-1 AND
                         S29-c merged AND 16c merged AND S33-c merged (Q-S29d-5) AND this design frozen AND
                         the lane lead's dispatch                                 (step-29 §11.2; this design)
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–8, as §14's rulings shape them
FROZEN INVARIANTS:       step-29 I-S29-1, I-S29-5 … I-S29-8; §6 I-d-1 … I-d-9; §8 as written
SEQUENCE:                C0 … C7 (§9)
IMPLEMENTER:             Opus-class (art judgement); mechanical parts (the corpus, the Rust test) delegable
ALLOWED COMMANDS:        Godot and cargo only through cast-slots/with-slot.sh: ./mineworld-slice and its
                         --slice-* modes, godot --headless --import, cargo build, cargo test -p <crate> --test
                         <file> [-- --ignored <name>]; python3 scripts/check_decision_ids.py (--write-index),
                         check_doc_headings.py, ci_layer.py --list, a fetch script kept in the lane's scratch
                         directory (C1 step 5); git; gh (pr create/view/checks, run view); mkdir -p; cp; cmp;
                         shasum; sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or a
                         whole member suite locally (R-LOAD-1); reading any key or .env; any model or image
                         call; editing a path I-d-1 forbids; committing a CC BY-SA file; merging
MATERIAL STOPS:          a start condition missing (in particular R-S29a-d-1 not in the merged S29-a); a needed
                         change under I-d-1's paths; a needed new texture beyond DEP-60's two; a count cap of
                         §5.10 exceeded with the levers applied; a p95 increment above cap or an absolute
                         failure in a new view at the milestone (Q-S29-4's reduced hall is the reserved lever,
                         operator only); a binding that needs rotation; any change to a frozen step invariant
                         or acceptance
PLATFORMS:               Linux, macOS, Windows (the farfield layer's nightly; per PR `fast` only)
VALIDATION BUDGET:       per commit: the build, the kit check and one local Godot run per mode touched, through
                         with-slot.sh; the rows marked M in §8 at the milestone, in one batch. Gate 1: NOT
                         REQUIRED. Gate 2: the connected walk (milestone). No paid call
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s29d.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                       (source: the operator's freeze of this design — pending)
                         semantic commits: authorized                    (working rules §14)
                         branch push: authorized                         (working rules §21)
                         PR creation / update: authorized                (working rules §21)
                         CI repair to `fast` green: authorized           (working rules §21; charter SPEED FIRST)
                         endpoint: MERGE-READY = READY FOR OPERATOR REVIEW: pushed, `fast` green on the exact
                         head, reported with the PR number and head SHA (charter SPEED FIRST)
                         merge: NOT by the implementation session — the coordinator merges
POST-MERGE SYNC OWNER:   lane lead: step-29, overall.md, the progress lines; implementation session: this
                         ledger, the merge identity, evidence, deviations
STOP CONDITION:          S29-d MERGE-READY (READY FOR OPERATOR REVIEW) — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-d-1 | The real hall breaks 16.7 ms in its own view (R-S29-1) | the levers of §5.10 from the start; counts per commit; the reduced hall is the operator's lever (material stop) |
| R-d-2 | S29-a freezes without R-S29a-d-1 | Q-S29d-1 is asked now, before S29-a's freeze; C0 stops otherwise |
| R-d-3 | HABS photographs lack scale (R-S29-3) | §5.2's method with stated error; placeholders labelled; the shots do not depend on the exact height |
| R-d-4 | LoC refuses automated downloads (A-22) | Wikimedia Commons' mirrors of the same HABS files, verified by call number; else the operator fetches them by browser into the scratch directory and the implementer hashes and commits them |
| R-d-5 | The Mission Revival depot clashes with the lakeside-referenced street (R-S29-4) | sanctioned (Q-S29-5); same materials and palette; the operator judges `st_1` and `st_2` (queue row) |
| R-d-6 | S33-c lands late and S29-d waits | Q-S29d-5's fallback (§11) |
| R-d-7 | The kept-prefix batching adds draws in the street views | one extra cell group per material per category; counted in C6 against the +40 draws cap |
| R-d-8 | A future world needs a station facing another way | refused by name (`STATION_SITE_FACING`); its frames are written for its orientation, so a rotated kit is never needed for binding (A-3) |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S29d-1 [operator — S29-a's station frames, F-S29d-1 … 3]** *(2026-10-19: asked once, as S29-a's Q-S29a-8; its ruling answers this row)* | The 3D client places a place only by moving its frame, never by turning it. S29-a's towns' ticket bay has its door on the wrong edge (it would be drawn inside the hall), and its platform runs east–west (it would be drawn across the hall). Revise S29-a (still DRAFT) so the bay's door is on its north edge with the counter on the south wall, and the platform runs north–south, 11 × 60 m (the real 8.2 m arcade plus a 2.8 m open strip), with ten arcade piers as solids? | **Yes, as R-S29a-d-1.** S29-a is not frozen or built, so the fix costs two place files and no re-baseline. The alternative — teach the client to rotate frames — changes 16c's and S33-c's frozen binding contract. *中文：S29-a 的售票间门开错了边、站台方向错了（3D 只能平移不能旋转）。在 S29-a 冻结前改：售票间门在北边、柜台在南墙；站台南北向 11×60 m（含真实 8.2 m 拱廊），拱廊柱写成 solids。同意吗？* |
| **Q-S29d-2 [operator — where the towers stand, F-S29d-4]** | The towers can flank the arch over a deep porch (which would stand on walkable street floor and need new solids in the street), or stand beside the forecourt, inner faces on its edges, 6 m forward of the façade, with the arch in the façade between them (the NRHP's "large arched forecourt"). Which? | **Beside the forecourt.** No world change, the tower walls are the drawn edge the server already discloses, and it matches the NRHP's description. *中文：两座塔放在前庭两侧（内墙正好是可走区域的边），拱门在两塔之间的立面上——符合 NRHP 的"大拱形前庭"，世界数据不用改。同意吗？* |
| **Q-S29d-3 [operator — the station kit, `ARC-192`]** | The depot is built from a data file in the Presentation Pack (`3D/stations/depot.yaml`, every number with a source), placed by a world's depiction at a site, and bound by role tag. Should `mineworld packs validate` also check this file, or only the 3D client at load? | **Only the client at load, with named refusals** (as `characters/cast.yaml`). It is presentation for one client; a Rust validator would be a second implementation of a file nothing else reads. Revisit if a second client reads kits. *中文：车站由数据文件"station kit"生成；只由 3D 客户端在加载时校验（带错误码），`packs validate` 不管它。同意吗？* |
| **Q-S29d-4 [operator — X-6's viewpoint, F-S29d-11]** | From the street's east end, the hall alone already fills most of the view's band, so hiding the towers does not fail LM-1 there; from spawn it does. Plant X-6 at `st_1` (spawn) instead of `st_2`? | **Yes, `st_1`**, with C6 recomputing the arithmetic from the measured heights before any run; LM-1 itself (≥ 60 % at both viewpoints) is unchanged. *中文：X-6（隐藏塔）应在出生点视角 st_1 失败，而不是 st_2（在 st_2 只靠大厅就已过线）。改成 st_1？* |
| **Q-S29d-5 [operator / lane lead — order]** | S29-d binds by tag through S33-c's depiction seam (frozen, not built), and the step also makes it wait for RL-b2, which is parked. Start S29-d after S33-c merges, and not wait for RL-b2 (counts against main's RL-b baseline; timings at the milestone)? | **Yes to both.** Building on S33-c avoids writing the tag binding twice; RL-b2 adds textures and trials, not the baseline S29-d's caps are measured against. Fallback in §11 if S33-c is late. *中文：S29-d 在 S33-c 合并后开工（用它的按 tag 绑定），不等已暂停的 RL-b2（计数对照 main 的 RL-b 基线，计时留到里程碑）。同意吗？* |

### 14.1 Rulings

Operator, 2026-10-19, in the primary session, relayed by the lane lead: every question answered as recommended.
Q-S29d-1 (asked as S29-a's Q-S29a-8; R-S29a-d-1 is in the merged S29-a), Q-S29d-2 (the towers stand beside the
forecourt), Q-S29d-3 (only the client validates a kit), Q-S29d-4 (X-6 at `st_1`), Q-S29d-5 (S29-d builds on S33-c
and does not wait for RL-b2).

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C7 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; X-6, M-d-1 … M-d-5 as §8.1 lists them)

### Deviations and findings

F-S29d-1 … F-S29d-12 (§3.3), recorded at design time.
