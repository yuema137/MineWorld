# PR S36-P5c — Content fixes: every recorded scene finding fixed in both towns

**Lifecycle:** `DESIGN FROZEN — operator-approved 2026-10-20` (relayed by the coordinator: "fix all 58 findings as one
content PR covering both towns, in data, generically, no per-world hard-coding in code; anything that truly cannot be
fixed in data stays recorded with a reason").

**Parent:** [`pr-s36-p5-scene-layout.md`](pr-s36-p5-scene-layout.md) (ledger §10, commit 7: "58 scene findings over
24 models … owed a fix together, in both towns, by the world lane"). **Base:** `origin/mvp1/pr-s36-p5b @ 25354173`
(#352: the props dump and `cast.py contact-scene`) with `origin/mvp1/pr-s36-p5a @ 61bfa08a` (#372: the
solids-overlap rule, `group`, `layout.rs`) merged in. Both are unmerged; this PR carries their commits until they land.

**Branch / worktree:** `mvp1/pr-s36-p5c`, `/Users/yuema137/mineworld-worktrees/impl-s36-p5c`.

---

## 1. Goal

`cast.py contact-scene` judges every prop the 3D slice draws against the place's disclosed solids (inside), against
the other props (apart) and against the drawn surface under it (support). On both towns' dumps 148 rows fail, recorded
as 58 findings over 24 models in `clients/3d-spike/assets/models/contact.json`. This PR fixes them so that the check
passes on both towns with no recorded scene finding left, except any listed in §5 with its reason.

`social-cafe` and `market-town` draw the same café, street and store from identical place files
(`worlds/<town>/places/{cafe,street,store}.yaml`, byte-identical copies), so every fix is made in both copies.

## 2. The findings, grouped by cause

Measured on the dump of scratch nightly 39402676433 (the fixture `tests/fixtures/props-social-cafe/`).

| # | Cause | Rows | Findings (model: stop rule) | Fix |
| --- | --- | --- | --- | --- |
| A | **Hand-set heights.** A prop is stood on a `y` typed beside it; the drawn surface is elsewhere: worktop cups and pots 25 mm above (`fy + 0.95` on a 0.925 top), cups 12.9 mm above their saucers, vases on the store's shelves 5–81 mm off the board, a vase 230 mm over the display shelves' board, the manhole's flange 43.8 mm proud of the road, shrubs in pots 3–41 mm off. A shrub whose glTF nodes are rotated is stood by its loose bounds, so its rest plane is 278–629 mm above the soil. | 60 | tea_set_01 GE-1/GE-2 support; pot_enamel_01, brass_pot_01, food_apple_01 GE-1 support; ceramic_vase_02, antique_ceramic_vase_01, jug_01, wicker_basket_01, planter_pot_clay, shrub_03 GE-3 support; shrub_02, shrub_03, water_manhole_cover GE-2 support | **Placement by support** (§3.1): every prop `SliceProps.put` places is lowered or raised onto the drawn surface under it, by the same nine samples the probe takes; a `ground` mount by its declared rest plane. A pot is filled with soil so a plant stands on something. |
| B | **Drawing with no solid.** The café's back-wall dresser (two bays of shelving and their contents) has no solid; the store's floor plants and the pot by its door have none; the terrace chairs have none. | 47 | wine_bottles_01, wicker_basket_01, wooden_crate_01, pot_enamel_01, jug_01, brass_pot_01, ceramic_vase_02, antique_ceramic_vase_01, wooden_bowl_01 GE-1 inside; potted_plant_02, planter_pot_clay, calathea_orbifolia_01 GE-3 inside; outdoor_table_chair_set_01 GE-2 inside | **Solids for what is drawn** (§3.2): the dresser bays, the store's two floor plants and the door pot, the six terrace chairs (each in its table's `group`), each with a scene collider. |
| C | **Solids smaller than the drawing.** The `× 0.92` collider rule sized the display shelves' solids 43 mm inside the drawn piece; the lamp posts' solids hold r 130 mm and the drawn base is r 148 mm; the A-board's solid is 0.2 mm short; the planter boxes' shrubs spill 0.35 m past their boxes. | 17 | wooden_display_shelves_01 GE-1/GE-3 inside; street_lamp_01, standing_chalkboard_01, shrub_02 GE-2 inside | The solid takes the drawn footprint in the body band (§3.2). |
| D | **Dressing placed off its support.** The store's west shelves are turned across the wall, and their vases stand 0.28 m beside the unit (on the long axis of the room, not the unit); the display shelves' calathea is 0.45 m off the unit's end; a 0.44 m basket on a 0.20 m wall shelf; a vase taller than the board pitch; the counter jug inside the bowl; the pastry case 0.25 m off the counter's front; the terrace's potted plant and tea set between two tables; the stone planters' shrub 0.34 m off centre and turned across the planter. | 24 | ceramic_vase_02, antique_ceramic_vase_01, jug_01, wicker_basket_01, Shelf_01 GE-3 inside/apart; calathea_orbifolia_01, ceramic_vase_02, wooden_display_shelves_01, wicker_basket_01, Shelf_01, jug_01, wooden_bowl_01, food_apple_01, croissant, carrot_cake GE-1 inside/apart; potted_plant_02, tea_set_01, outdoor_table_chair_set_01, shrub_02 GE-2 inside/apart | **Dressing data moved onto its support** (§3.3). |

The 148 rows are the 58 findings' instances; every finding is in at least one row group above.

## 3. Design

### 3.1 Placement by support (client, generic)

`SliceProps.put` stands a prop's bounding box on the `y` its caller names. That `y` is typed beside the call, and
it is wrong whenever the caller misjudged the surface (a worktop's top, a saucer's rim) or the bounds are loose (a
glTF node rotated inside the file). The fix is the probe's own rule run once at placement: after `put` positions a
prop, `SliceSettle.settle` casts nine rays down from 50 mm above the prop's lowest point (above its rest plane for a
`ground` mount) at its bounds' corners, edge midpoints and centre, against the drawn meshes under the same parent
(the prop's own excluded), takes the hit nearest the prop's plane, and moves the prop vertically by that distance.

- **The plane** is the prop's true lowest vertex (computed from the mesh, cached per model and kept parts, invariant
  under the prop's yaw), or, for a `ground` mount, its declared rest plane (`rest_mm` in `contact.json`). The client
  reads `contact.json` once (the probe already does).
- **Mounts.** `floor`, `surface` and `ground` settle; `wall` and `ceiling` never do; a model with no entry does not.
- **A bound.** A move larger than 0.30 m is not made and is warned: a prop that far from any surface is misplaced
  data, and dropping it to the floor would hide that.
- **Geometry** is asked through Godot's `TriangleMesh` (`Mesh.generate_triangle_mesh`, `intersect_segment`), cached
  per mesh resource; no physics frame, no colliders. Reuse: this is the engine's own BVH ray query; no library is
  added.
- **Order.** Each prop settles when it is placed, so a cup placed after its saucer stands on the saucer.

Pots get soil: `SliceProps.potted` puts a pot, a soil disc 45 mm below its rim (0.42 × its width in radius) and the
plant on the soil. A plant in an empty pot otherwise stands on whichever rim sample is nearest.

### 3.2 Solids for what is drawn (both towns' place files, with the slice's colliders)

The rule (ARC-306): every drawn piece a body could meet is a solid, and its solid covers what is drawn of it in the
body band. Each new or enlarged solid gets a scene collider (`Build.box_blocker`, or a solid `Build.box`) so the
geometry probe (`--slice-geometry`, 150 mm both ways) holds. Numbers are the drawn footprint in the band, measured
from the glTF meshes with the placement transforms, rounded 5 mm outward.

| Place | Solid | From → to (mm) |
| --- | --- | --- |
| café | back-wall dresser, west of the worktop | new `x 1525..3860, y 9720..10320`, height 3300 |
| café | back-wall dresser, behind the worktop (two) | new `x 3860..4595` and `x 4825..7295`, `y 9910..10320`, height 3300 |
| café | `shelves.west` | `x 0..836, y 5539..5881` → `x 0..885, y 5520..5900` |
| store | west display shelves (two) | `x 0..716` → `x 0..765`, `y` 19 mm out each side |
| store | the floor plants (two), the pot by the door | new, from the drawn plant |
| street | the seven lamps | ±91 → ±150 about each post |
| street | the six terrace chairs | new, each in its table's `group` (`terrace-table.N`) |
| street | the north A-board | `x −1643..−557, y 1131..2169` → `x −1690..−510, y 1085..2215` |
| street | the two planter boxes | the box's solid → box ∪ its shrub's footprint |

Every new solid is checked against the P5a rules: no overlap with another solid outside its group, no person, seat
or doorway point within a radius and a gap (`layout.rs`, `genesis`, `doorways_genesis`).

### 3.3 Dressing moved onto its support (client data)

- Store west shelves: the vase, jug and antique vase stand along the unit (its long axis is across the room), on its
  boards; the antique vase (0.44 m) is taller than the board pitch (0.38 m) and stands on the top.
- Store back-wall shelves (`Shelf_01`, 0.20 m boards): a jug (0.17 m deep) instead of a 0.22 m vase, and a pot of
  flowers instead of the 0.38 m basket.
- Café display shelves (`furnishings.yaml` `shelves.display`): the vase on the 0.79 m board (it was 1.02 m, which
  put a 0.31 m vase through the 1.17 m board); the calathea on the 1.17 m board inside the unit.
- Café wall shelf (`shelves.wall`): a jug on the 0.94 m board instead of the basket.
- Café counter: the jug 0.40 m west of the fruit bowl it stood in; the pastry case centred on the counter.
- Street terrace: a small calathea on the first table's top and the cup and saucer on the third table, both at the
  table's drawn centre (read from the set's own mesh, as `_chair_seat` does); the stone planters' shrub centred by its
  true footprint and turned along the planter.
- Store: the east floor plant 0.50 m clear of the counter it was drawn into; the door pot 0.25 m further in, clear of
  the door leaf.

### 3.4 Non-goals

No change to `scene_check.py`'s rules or bounds, the tolerance file, the probe, or any server code. No contract change.

## 4. Commits

1. `docs(plans)`: this design.
2. `feat(3d)`: placement by support (`settle.gd`, `SliceProps.put` settles, `SliceProps.potted`,
   `SliceProps.true_box`).
3. `fix(worlds)`: the content (place files of both towns, `furnishings.yaml`, the slice's dressing and colliders),
   the recorded furnishings observation, the 30-day golden re-captured.
4. `chore(contact)`: findings removed from `contact.json`, the fixture dump re-captured, PIPELINE §18, ledger.

## 5. Acceptance

- **AC-P5c-1.** `cast.py contact-scene` on a fresh nightly dump of both towns exits 0 with 0 failing rows and no
  recorded scene finding left in `contact.json`, except those listed here with a reason (none expected).
- **AC-P5c-2.** `layout.rs`, `furnishings.rs`, `genesis`, `doorways_genesis` and `validate` of all four worlds pass.
- **AC-P5c-3.** The geometry probe (`--slice-geometry`) passes on the same nightly.
- **AC-P5c-4.** Goldens whose digest moves are re-captured with the diff recorded in the ledger.
- **AC-P5c-5.** `test_scene_check.py` passes on the re-captured shipped dump.

## 6. Ledger

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| 1 | [x] | N/A (docs) | [x] | this file |
| 2 | [x] `settle.gd`; `SliceProps.put` stands by plane then settles; `true_box`, `put(true_centre)`, `potted` | [x] PASS: the scratch nightlies run it on Linux, macOS and Windows with no script error; the one settle warning (a trough shrub 379 mm off, run 39504825584) led to troughs with soil | [x] self-review | runs 39504825584, 39540403533 |
| 3 | [x] both towns' place files, `furnishings.yaml`, café/store/street dressing and colliders; recorded observation (`MINEWORLD_WRITE_FIXTURES=1`, `--test furnishings`); 30-day golden | [x] PASS: `layout.rs` 4/4; bodies `genesis`, `doorways_genesis`, `layout_section`, `furnishings` pass; `decisions` 7/7 after re-capture; `contact-scene` on run 39549116523: 6 places, 394 props, 0 failing; geometry probe: every row touching this PR's solids passes (7 pre-existing rows remain, F-P5c-1) | [x] self-review; iterations in §7 | runs 39540403533, 39544789796, 39549116523 |
| 4 | [x] 58 findings removed from `contact.json`; fixture re-captured from run 39549116523; the shipped-dump test expects no finding; PIPELINE §18 | [x] PASS: `test_scene_check.py` 10 passed; `contact-items` none failing; `contact-scene` on the fixture 0 failing | [x] self-review | local 2026-10-21 |

## 7. Findings and bounded discoveries

- **D-P5c-1 (bounded).** A place holds at most 64 solids and the street had 60: six chair solids would make 66. Each
  terrace set's east-west crossed box is widened to hold its two chairs instead (they stand east and west of the
  table), so the street keeps 60; the client's collider is the same widened box.
- **D-P5c-2 (bounded).** Iterations: run 39540403533 left 9 apart rows per town (vases across the display shelves'
  bay partitions, the counter calathea in the re-centred pastry case, trough shrubs through the box walls), fixed by
  bay-centred dressing, moving the calathea and heaping trough soil 20 mm over the rim; run 39544789796 left the
  widened A-board and display-shelf solids 40–45 mm past their x 0.92 colliders, fixed by colliders of the drawn
  footprint. The scratch branches reordered `clients-probes` (props and geometry first) only to see past the
  pre-existing `target` failure; `ci_layer.py` is unchanged here.
- **Golden.** `social-cafe-seed-7-30-days.txt` re-captured: fingerprint `0850509ca57ada14` → `d833dd775a4df917`,
  facts 108 361 → 110 323, `no-route` rejections 3 922 → 3 714.
- **F-P5c-1 (pre-existing, not this PR).** The geometry probe's 7 remaining rows are the street's station forecourt
  (floor edges and colliders west of x −37 500, from S29-a's station on main). The branch base (p5a+p5b merged,
  scratch run 39540412921) also fails the `target` probe ("no person perceived on the street") and the 2D client
  tests (`walks_from_the_apartments_into_the_cafe`, the settings tests: a 2D script reads a passage's `to`).
- **F-P5c-2 (pre-existing, main drift).** `furnishings.rs every_kind_every_world_names_is_bound_in_each_default_pack`
  fails on this base: main binds the station kinds (`bench.station`, `kiosk.news`), the p5a/p5b base does not. It
  clears when the branch meets main.
- **Remaining recorded findings: none.**
