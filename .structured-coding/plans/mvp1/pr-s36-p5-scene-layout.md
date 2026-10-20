# PR S36-P5 — Scene layout: points clear of solids, solids apart, drawn props inside their solids and on their support

**Lifecycle:** `DESIGN FROZEN — 2026-10-19 (operator approved in the primary session: every question answered as recommended)`. No implementation before a `DESIGN FROZEN` header from the operator and a filled
contract (§11).

**Parent:** [`step-36-interpenetration.md`](step-36-interpenetration.md) §4.5. **Depends on:** S36-P1 merged (#318:
`contact.py`, `contact.json`, C-3/C-4, the tolerance file); #320 and #323 merged (the prop fixes and cuts). It
answers **Q-S36-P1-2** (how planted and flush items are judged) and **Q-S36-9 / Q-VF-F1** (how a drawn prop links to
its `Solid`). **Audit base:** `origin/main @ 5a8c6f28` (2026-10-19).

**Design session:** worktree `/Users/yuema137/mineworld-worktrees/design-s36-p345`, branch
`docs/pr-design-s36-p345`.

**Shipping.** One design, two PRs that can run in parallel: **S36-P5a** (server data and the Rust test, commits 1–3)
and **S36-P5b** (the client dump and the Python check, commits 4–7). P5b does not need P5a's code, only its content
fixes, so whichever lands second rebases on the first.

---

## 1. Goal

Two kinds of "people walk through furniture" exist, and each needs its own check.

1. **The server's truth is wrong.** A person starts inside a table, a doorway point sits in a chair, a seat's
   standing point is in a counter, or two solids were authored into each other. The server then resolves people
   against nonsense. This is a check of world data, in Rust, over every world.
2. **The drawing disagrees with the truth.** The server's chair `Solid` is 0.58 m wide, but the drawn chair is 0.70 m
   wide; or a chair is drawn where no solid is. A person then walks visibly through the drawn part. This is a check
   of the drawn scene against the disclosed solids, in Python with trimesh and python-fcl (P1's module), on a dump
   the client writes.

Example of the first: in both towns' streets, a bollard's solid runs 88 mm into a stone planter's solid
(§2.2). Example of the second: `put_solid` sizes a chair's collider at 0.92 × its drawn bounds, so a drawn chair
0.80 m wide overhangs its solid by 32 mm on each side; a 2.0 m table would overhang by 80 mm, past the 40 mm bound.

## 2. Audit (`origin/main @ 5a8c6f28`)

### 2.1 Server

| Fact | Where |
| --- | --- |
| `Solid { min, max: Corner, height }`, integer mm, `deny_unknown_fields`, no id, kind or group | `systems/bodies/src/component.rs` 77–106 |
| `PlaceShape::checked` (the one check behind a decoded `PlaceShape` and the `body:` section): bounds, floor size, at most 64 solids, non-empty footprints, heights 1–10 000 mm. **No solid–solid test** | `component.rs` 188–238 |
| The `body:` section's written form `Written { floor, solids: Option<Vec<Solid>>, shape, at }` → `PlaceShape::checked` | `systems/bodies/src/section.rs` 62–107 |
| Genesis `fits`: each person ≥ `PERSON_RADIUS` (250 mm) from the floor's edge and every solid (no `GAP`); people ≥ `CLEARANCE` apart; `doorways_fit`: each doorway point (both sides of every passage) ≥ `R + GAP` (260 mm) from the edge and every solid; `placed`: objects outside solids, people and each other | `systems/bodies/src/genesis.rs` 66–200, 227–341 |
| `PERSON_RADIUS` 250, `GAP` 10, `TOLERANCE` 5 | `systems/bodies/src/geometry.rs` 15–25 |
| `Room::admits`, `clear_of_solids`, `solid_within` are `pub(crate)` | `geometry.rs` 214–240 |
| **Seats are sit points** (`posture`, S34): `{ key, group?, at, facing }` in a place file's `sit_points:`; `at` is free floor in front of the seat; spacing ≥ 510 mm. `posture` does not depend on `bodies`, so **nothing checks a sit point against a solid** | `systems/posture/src/point.rs`; `systems/posture/Cargo.toml` |
| No shipped world has `sit_points:` yet (S34-c and S33-f add them) | grep `sit_points` in `worlds/`: none |
| Worlds: `bodies-yard`, `lakeside`, `market-town`, `social-cafe`. `lakeside`'s places have **no `body:` section** (S33-b, frozen, not merged) | `worlds/*/places/*.yaml` |
| The CLI tests' independent geometry oracle reads place files' text, never bodies' code (DO-12): `rooms_of`, `R`, `GAP`, `scan_in` | `tools/cli/tests/bodies/mod.rs`; `tools/cli/tests/town_bodies.rs` header |
| `tools/cli/tests/packs.rs` already enumerates `worlds/` by `read_dir` | `packs.rs` 200 |
| Generated scenes: `systems/bodies/tests/route.rs` `Scene::generate` draws up to 64 random solids, **overlaps allowed** (it tests routing, not layout) | `route.rs` 113–160 |

### 2.2 Measured: solid–solid overlaps in the shipped worlds

Measured on the place files' text with a scratch script (`/tmp/s36p5/overlaps.py`, not committed). Both towns carry
the same eleven pairs; `bodies-yard` has none.

| Place | Pair (solid index, authored order) | Overlap | Reading |
| --- | --- | --- | --- |
| café | table east (4) × its chairs (9, 10) | 49 × 474, 29 × 474 mm | a chair tucked under its table: by design if the drawn meshes do not meet |
| café | table north (5) × its chairs (11, 12) | 63 × 496, 43 × 496 mm | the same |
| café | table west (6) × chair (13) | 14 × 440 mm | the same |
| street | each terrace table's two crossed boxes (42 × 43, 44 × 45, 46 × 47) | 806 × 806 mm | one round table authored as two boxes: by design |
| street | bollard (9) × stone planter (36); bollard (15) × planter (37) | 226 × 88 mm each | a bollard standing 88 mm inside a planter: probably a content defect |
| store | window staging (0) × crate by the window (8) | 197 × 450 mm | a crate inside the staging: probably a content defect |

So a plain "no two solids overlap" rule would reject both shipped towns, mostly for shapes that are right. The rule
needs to know which boxes belong to one thing (§4, commit 1; Q-S36-P5-1).

### 2.3 Client

| Fact | Where |
| --- | --- |
| Every prop is loaded through one function, `Props.gltf(slug)`; placed by `Props.place` (origin at `pos`), `SliceProps.put` (bounds' bottom on `pos`, optional `keep(parts)` that trims the glTF to some nodes), `SliceProps.put_solid` (`put` plus a `LAYER_WORLD` blocker box at 0.92 × the placed bounds) and `SliceProps.hang` | `scripts/props.gd` 118–125; `scripts/slice/dressing.gd` 72–161 |
| 85 placement calls in 7 files, all GDScript constants (`cafe_interior.gd` 41, `shop_interior.gd` 18, `streetscape.gd` 14, `town.gd` 9, …) | grep |
| No id links a drawn prop to a server solid. The world's solids are a hand transcription of the slice's colliders (step-11 §19.3.1, `ARC-306`) | `pr-vf-furniture-solids.md` §2.1 |
| 12e's geometry probe (`--slice-geometry`, nightly `clients-probes`): disclosed faces ↔ scene **colliders** within 150 mm, both ways, in the café, the street and the store; the body band `BAND_LOW` 0.05 m – `BAND_HIGH` 1.72 m. It compares colliders, never drawn meshes | `scripts/slice/slice_probe_geometry.gd`; `scripts/ci_layer.py` 208 |
| `contact.json` (P1, #320, #323): mounts `floor`, `surface`, `wall`, `ceiling`; recorded findings; four props sink by construction — `grass_medium_01` 25.6 mm, `shrub_02` 25.2 mm, `celandine_01` 7.0 mm, `water_manhole_cover` 28.8 mm (collar below the origin; flange top 38.8 mm above it) | `clients/3d-spike/assets/models/contact.json` |
| The manhole cover is placed at y = +0.005 m on the road | `scripts/slice/street.gd` 115 |
| `contact.py` offers `scene_parts(path)`, `world_vertices`, `overlaps(parts)` (fcl), `lowest`, `clearance` | `tools/character_pipeline/mwcast/contact.py` 191–269 |

### 2.4 Findings

- **F-P5-1.** Shipped worlds overlap solids in eleven pairs, eight by design (§2.2). The rule needs a grouping.
- **F-P5-2.** Seats (sit points) are checked by nobody against solids, because `posture` and `bodies` do not know each
  other. Only a world-level check can do it.
- **F-P5-3.** People starts are held to `R` at genesis, doorway points to `R + GAP`. The step's O-7 asks `R + GAP` for
  every point a person is placed at. The test holds all of them to `R + GAP`; genesis is not changed.
- **F-P5-4.** A drawn prop can be a trimmed part of its glTF (`keep(parts)`), so a check that loads the whole file
  would judge geometry nobody sees. The dump records the kept nodes.
- **F-P5-5.** `put_solid`'s 0.92 factor makes the overhang grow with the prop's size; the check will name any prop
  past 40 mm.

## 3. Scope

**P5a (server data, Rust):** `systems/bodies/src/section.rs` (an authored solid's optional `group`; the overlap rule);
the generated authoring reference for the `body:` section; the shipped place files (groups, and fixes for real
overlaps); `tools/cli/tests/layout.rs` (new); a new `ARC` record; tests.

**P5b (client and Python):** `scripts/props.gd` (`Props.gltf` tags each prop node); `scripts/slice/slice_probe_props.gd`
(new probe mode, sibling of `slice_probe_geometry.gd`); `contact.json` (mount `ground`, `walk_through`);
`mwcast/contact_gates.py` (C-3 for `ground`); `mwcast/scene_check.py` and `cast.py contact-scene` (new);
`scripts/ci_layer.py` (`clients-probes`); `tools/character_pipeline/PIPELINE.md` §18; tests.

**Out:** where the client's furniture comes from (Q-VF-F1 (b), §6); `PlaceShape`'s stored form and its disclosure (no
digest moves in P5a); genesis margins; the lakeside's solids (S33-b) and terrain (S33-d), which join by enumeration
when they land.

## 4. Commits — S36-P5a

### Commit 1 — solids of one thing share a `group`; others may not overlap

**Goal.** An author can say "these boxes are one thing", and every other overlap is refused when the world loads.

**Scope.**

- `section.rs`: the written form takes `solids: Option<Vec<AuthoredSolid>>`, where `AuthoredSolid` is `Solid`'s three
  fields plus an optional `group` (a key: lower-case letters, digits and `-`, 1–40 characters, the rule `posture`'s
  `SitGroupKey` uses). After `PlaceShape::checked` passes, the section checks every pair: two solids whose footprints
  share an area greater than zero (touching edges do not count) must carry the same `group`; otherwise the place is
  refused with the code `bodies-solids-overlap` and a message naming both solids by index, their overlap in mm, and
  the remedy ("solids of one thing share a `group`").
- The `group` is **authoring data only**. `PlaceShape` does not store it, so the `place-shaped` fact, snapshots,
  disclosure and every digest stay as they are. A `PlaceShape` decoded from a fact is not re-checked for overlaps
  (it was checked when authored).
- `docs/decisions/ARC-<n>.md` (id from the coordinator's range): the rule, the reason (§2.2), and that Q-VF-F1 (b)
  may later promote `group` to a stored, disclosed key with `kind` and `yaw` (§6). It amends `ARC-39`'s written form.
  `scripts/check_decision_ids.py --write-index`.
- The generated authoring reference for the `body:` section is regenerated (the `schemars` schema gains `group`).

**Content (both towns).** `group` on the three crossed terrace tables and on each café table with its tucked chairs.
The bollard–planter and crate–staging pairs are read in the drawn scene (`streetscape.gd`, `shop_interior.gd`): when
the drawn meshes meet, the solid **and** the drawn prop move apart by the overlap plus `GAP` (content fix in both
places, the 30-day fixture `social-cafe-seed-7-30-days.txt` re-captured if its digest moves); when they do not meet,
the pair is grouped with a comment saying why. Either outcome is recorded in §10 with the numbers.

**Implementation.**

- [ ] `AuthoredSolid`, the pair rule, the rejection code and message.
- [ ] Groups and content fixes in `social-cafe` and `market-town`; the reference regenerated; `ARC-332`.

**Validation.**

- `systems/bodies/tests/genesis.rs` (or a new `layout_section.rs`): a place with two ungrouped overlapping solids →
  refused, naming both and the overlap; the same pair with one `group` → accepted; two solids touching along an edge
  → accepted; a `group` key with an upper-case letter → refused naming the key.
- **Generated variant:** a seeded generator (in the test) draws 200 places of non-overlapping solids (rejection
  sampling) → all accepted; then moves one solid of each into its neighbour by 1–300 mm → each refused by index.
- `mineworld validate` on all four worlds: green.

**Acceptance.** Both towns validate with the groups; the generated overlaps are refused by name; no digest moves
except where a content fix moved a solid (recorded).

**Commit boundary.** The rule and the shipped content that satisfies it.

### Commit 2 — `layout.rs`: every point a person is placed at clears every solid

**Goal.** One test, over every world found in `worlds/`, of every authored point the server puts a person at.

**Scope.** `tools/cli/tests/layout.rs` (new), using `bodies/mod.rs`'s oracle (`rooms_of`, `R`, `GAP`; DO-12: read
from the files' text, never from bodies' code). For each world directory with a `world.yaml` (`read_dir`, as
`packs.rs` does):

1. `mineworld validate` succeeds (genesis runs every bodies check: people at `R`, doorways at `R + GAP`, objects).
2. In every place with a `body:` section, each of these lies inside the floor shrunk by `R + GAP` and at least
   `R + GAP` from every solid:
   - every person's start (`people/*.yaml` `location`);
   - every doorway point (`passages` `here`, and the `there` of passages into the place);
   - every sit point's `at` (`sit_points:`, when the world has them).
3. A place without a `body:` section is listed as "no body: not judged" (today all of `lakeside`).
4. The test prints one row per point: world, place, kind, key, nearest solid index and distance, margin.

**Implementation.**

- [ ] `layout.rs` with the enumeration, the three point kinds and the printed rows.
- [ ] `bodies/mod.rs` gains the text readers it lacks (sit points, starts), beside `rooms_of`.

**Validation.**

- The four shipped worlds pass, or each failure is a content defect fixed in this commit (a start moved, its fixture
  re-captured) and recorded with the numbers.
- **Generated variants (in the test):** a copy of `social-cafe` with Alice's start moved 100 mm into the counter's
  `R + GAP` band → fails naming `alice`, the café, solid 0 and the distance; a copy of `bodies-yard` with a sit point
  section added 200 mm from its solid → fails naming the key; the seeded generator of commit 1 places starts and sit
  points on its free lattice → all pass.
- Mutation: `R + GAP` replaced by `R − 50` in the test → the moved-start variant passes, so the test goes red;
  restored.

**Acceptance.** Every world is enumerated, every point kind is judged at `R + GAP`, and each planted variant fails by
name.

**Commit boundary.** The test.

### Commit 3 — records (P5a)

- [ ] Step 36 §4.5 updated by §9 here; `ARC-332`; this ledger, §10, with the overlap pairs and their outcome.

## 5. Commits — S36-P5b

### Commit 4 — planted and flush items: mount `ground` (Q-S36-P1-2)

**Goal.** Grass, ground cover and a manhole cover are judged by where they meet the ground, not by their lowest
vertex.

**The rule.** `contact.json` gains a fifth mount, `ground`, with three fields:

- `rest_mm`: the height, in the prop's own frame, of the plane that meets the ground. Default 0, the origin (Poly
  Haven's convention for ground plants). For a cover set flush into the road, it is the top of its flange.
- `embed_mm`: how far the mesh goes below that plane by design (roots, a frame's collar).
- `why`: one sentence. A `ground` entry without it is refused.

**C-3 for `ground`** (asset level): the measured depth of the lowest point below `rest_mm` equals `embed_mm` within
`item.rest.floor` (3 mm). So the declaration stays true when an asset changes, and an ordinary floor prop cannot hide
a real sink behind it. The scene check (commit 6) then judges the rest plane against the ground under the prop.

**Walk-through.** An optional `walk_through: true` with its own `why` says that a person may pass through the prop
(grass, low ground cover). Such a prop needs no solid (commit 6). Anything a person should not walk through
(a shrub in open ground, a bollard) may not declare it.

**Content.** `grass_medium_01` (rest 0, embed 25.6, walk-through), `celandine_01` (rest 0, embed 7.0,
walk-through), `shrub_02` (rest 0, embed 25.2; it stands in planters, whose solids hold it), `water_manhole_cover`
(rest 38.8, the flange top, flush; embed 67.6). Their recorded C-3 findings are removed: the gate now passes them on
their declaration.

**Implementation.**

- [ ] `load_mounts` and `ItemSpec`: `ground`, `rest_mm`, `embed_mm`, `walk_through`, `why`; unknown or missing fields
      refused by name.
- [ ] `gate_c3` for `ground`.
- [ ] `contact.json` entries; PIPELINE.md §18.

**Validation.** `test_contact_items.py`: a generated library box declared `ground` with `embed_mm: 20` that sinks
20 mm → PASS; sinks 26 mm → FAIL; a `ground` entry without `why` → refused. The shipped library exits 0 with the four
findings gone.

**Commit boundary.** The mount and the four props.

### Commit 5 — the client dumps what it drew

**Goal.** The check sees the exact props the player sees, where they are.

**Scope.**

- `Props.gltf(slug)` sets `set_meta("mw_prop", slug)` on the node it returns. `SliceProps.keep(n, parts)` sets
  `set_meta("mw_parts", parts)`. Nothing else changes in placement.
- `scripts/slice/slice_probe_props.gd` (new, `--slice-props`, a sibling of `slice_probe_geometry.gd` that reuses its
  walk through the café, the street and the store). In each drawn place it writes `props-<place>.json`:
  - `place`, the disclosed `place-shape` as observed (mm, the place's frame) and the same converted to the scene's
    frame by `SliceLayout.to_scene` (so Python never re-implements the mapping);
  - the body band (`BAND_LOW`, `BAND_HIGH` from `SliceProbeGeometry`, one source);
  - per tagged node: `slug`, `parts` (or all), its global transform (4 × 4, scene frame, metres), and **support
    samples**: the height of the drawn surface under nine points of its footprint (corners, edge midpoints, centre).
- **Support samples are taken against drawn geometry, not colliders.** A `put_solid` blocker is 0.92 × the prop's
  height, so a cup's ray would meet it 60 mm under the tabletop. The probe therefore builds temporary trimesh
  colliders from every drawn `MeshInstance3D` of the place on a probe-only physics layer (`create_trimesh_collision`,
  layer 32), excluding the prop's own meshes, and casts each ray down from 5 mm above the prop's lowest point. The
  temporary colliders are freed after the dump.

**Implementation.**

- [ ] The two `set_meta` calls.
- [ ] `slice_probe_props.gd`, its flag in `slice_probe.gd`'s mode table, the JSON writer.

**Validation.** One headless run (through `with-slot.sh`): every place writes its file; the café's file lists every
prop `cafe_interior.gd` places (count equal to its placement calls reached); the café table's support samples read
the floor within 1 mm; a cup's read the tabletop within 1 mm.

**Commit boundary.** The dump; no verdict yet.

### Commit 6 — `cast.py contact-scene`: drawn inside truth, apart, on support

**Goal.** Judge the dump with P1's module and the tolerance file.

**Scope.** `mwcast/scene_check.py` and `cast.py contact-scene <dir> [--tolerances] [--root <models>]`. For each dumped
place, each prop is loaded with `contact.scene_parts` (filtered to its `parts`) and moved by its transform. Then:

1. **Inside the solids (presentation vs truth).** Take the prop's vertices, and samples every 20 mm along its edges,
   whose height lies in the body band and whose position lies inside the walkable floor. Each must lie within
   `scene.mesh_over_solid` (40 mm) of the union of the place's solids (in the scene frame), or within that distance
   of the floor's edge (a wall-mounted clock protrudes from the wall). The prop's **link** is the solid holding most
   of its sampled points, printed with the overhang. A prop with no point in any solid and points in the band is
   **"drawn where no solid is"** (`ARC-306`'s defect), unless it declares `walk_through`.
2. **Apart.** Every pair of props whose bounds meet: `contact.overlaps` on their parts; a depth beyond
   `item.overlap.parts` (2 mm) fails, naming both props and parts. A tucked chair and its table are exactly such a
   pair, so the grouping of commit 1 is checked here against the drawn meshes. Pairs inside one prop stay C-4's.
3. **On their support.** `floor` and `surface` mounts: the lowest point lies within `item.rest.floor` of the highest
   support sample under it (it neither floats nor sinks into what holds it). `ground`: the rest plane (`rest_mm`)
   lies within `item.rest.floor` of the support. `wall` and `ceiling`: listed, not judged (no wall or ceiling
   sample in the dump; recorded as a later extension).

Every row prints place, prop, rule, measured value, bound key and margin. The command exits 1 on any failure; a
recorded finding is allowed only through `contact.json`'s existing ratchet (Q-S36-P1-1), scoped per place.

**Implementation.**

- [ ] `scene_check.py` with the three rules; `cast.py contact-scene`.
- [ ] `scripts/ci_layer.py` `clients-probes` (nightly): `slice --world --props` (the probe) then `uv run --locked
      python tools/character_pipeline/cast.py contact-scene <dir>`. Per-PR CI runs only the Python tests.

**Validation (generated variants, in `tests/test_scene_check.py`).** A synthetic dump built from the shipped café
dump's structure with a generated library: a box prop drawn 60 mm wider than its solid → FAIL "overhangs solid 4 by
60 mm (scene.mesh_over_solid 40 mm)"; 35 mm → PASS; a box with no solid under it → FAIL "drawn where no solid is";
the same box declared `walk_through` → PASS; a chair box moved 30 mm into its table box → FAIL naming both; a cup
floating 6 mm over its table → FAIL; a `ground` prop whose rest plane sits 5 mm above the support → FAIL.
Mutation: the band clip removed → a ceiling lamp above a table reads as an overhang and the shipped-dump test goes
red; restored.

**Acceptance.** Each planted variant fails exactly where planted and passes below its bound.

**Commit boundary.** The check.

### Commit 7 — the real run and the records (P5b)

- [ ] The probe and the check on `social-cafe` and `market-town` (café, street, store). Each failure is fixed (the
      prop and its solid moved or resized together, both towns) or recorded with its numbers for the asset or
      world lane. Expected from the audit: the manhole cover sticks up 43.8 mm (placed at +5 mm with its flange top
      at 38.8 mm), and any `put_solid` prop wider than 1.0 m overhangs past 40 mm (F-P5-5).
- [ ] `lakeside` listed as not yet drawn by the slice (S33-d); it joins by enumeration.
- [ ] PIPELINE.md §18 (`contact-scene`, the dump, the rules); this ledger, §10.

## 6. The link from a drawn prop to its `Solid` (Q-S36-9 with Q-VF-F1)

Today the client places furniture from GDScript constants, and someone copies the positions into the place files.
`ARC-306` was that copy going wrong. There are two separate questions:

1. **How do we know a drawn prop and a solid agree?** This PR answers it by geometry: rule 1 of commit 6 links each
   prop to the solid that holds it, and fails the prop that overhangs or has no solid. No new field is needed, and
   any world drawn any way is checked the same way.
2. **Where should the furniture come from?** That is Q-VF-F1. Its recommended option (b) gives solids a `kind` and
   a `yaw`, discloses them, and lets each client draw a kind's asset at each disclosed solid, so the copy disappears.
   It changes `PlaceShape`'s stored and disclosed form, both clients' furniture code, and every town's digests.
   That is a contract change of its own, not a check.

**Recommended (Q-S36-P5-2):** P5 ships the geometric link. Q-VF-F1 (b) follows as its own PR design (proposed
`S36-P6`, after P5b), and promotes commit 1's `group` to the stored key that carries `kind` and `yaw`. P5's checks keep
the same inputs after it, so they guard that change too.

## 7. Acceptance criteria

1. **Server truth (P5a).** Every world found in `worlds/` validates; in every place with a `body:` section, every
   person start, doorway point and sit point clears every solid and the floor's edge by `R + GAP`; no two solids of
   different groups overlap. The generated variants (moved solids, a moved start, an added sit point, 200 seeded
   places) fail or pass exactly as planted.
2. **Drawing vs truth (P5b).** In `social-cafe` and `market-town`'s drawn places (café, street, store), every prop's
   in-band geometry lies within 40 mm of a solid or a wall, no two props interpenetrate beyond 2 mm, and every
   floor, surface and ground prop rests on its support within 3 mm; each remaining failure is fixed or a recorded
   finding with numbers. The synthetic variants of commit 6 fail by name.
3. **Ground items (Q-S36-P1-2).** The four sink-by-design props pass C-3 on their declaration; their findings are
   gone; a `ground` prop sunk deeper than declared fails.
4. `lakeside` is enumerated and listed as not judged until its `body:` sections (S33-b) and drawn scene (S33-d)
   land; nothing names it in code (I-S36-5).
5. `fast` is green on each PR's final head.

## 8. Invariants

I-S36-1, I-S36-2, I-S36-4, I-S36-5 (step §5), and:

- **I-P5-1.** The server's stored and disclosed `PlaceShape` does not change in P5 (no digest moves except content
  fixes, each recorded).
- **I-P5-2.** The check never reads a prop's, place's or world's name to decide; it reads the dump, `contact.json` and
  the tolerance file.

## 9. Proposed edits to shared documents

- Step 36 §4.5: "no two `Solid`s of one place overlap" → "no two solids of different `group`s overlap" (§2.2); "seats
  and anchors" are sit points, starts and doorway points (F-P5-2, F-P5-3); the link answer of §6; Q-S36-P1-2's answer.
- `pr-s36-p1-contact-module.md` §8.6: Q-S36-P1-2 answered here.
- `pr-vf-furniture-solids.md` §4: Q-VF-F1 deferred to `S36-P6` with §6's reason, if Q-S36-P5-2 is approved.

## 10. Ledger

| Commit | Implementation | Validation | Review | Evidence |
| --- | --- | --- | --- | --- |
| 1 | [x] | [x] | [x] | P5a: `systems/bodies/tests/layout_section.rs` 7/7 PASS locally (refused naming `solids 1 and 2`, `60 mm x 400 mm`; group, one furnishing and touching edges load; `Seating` refused; 200 seeded places load and each pushed solid is refused by index); `furnishings`, `genesis`, `doorways_genesis` PASS; all four worlds validate (layout.rs step 1) |
| 2 | [x] | [x] | [x] | P5a: `tools/cli/tests/layout.rs` 4/4 PASS locally, 60 points judged over the four worlds, smallest margin 0 mm (wanderer, café, solid 5: the TD-D4 inset, exactly at R + GAP), lakeside's seven places listed "no body: not judged"; planted variants fail by name (`social-cafe cafe Start alice: solid 0 at 255 mm … margin -5 mm`; `bodies-yard court Sit planted.1: solid 0 at 200 mm`); mutation `CLEAR = R − 50` turns `a_start_inside_a_solids_band…` red (see §10.1) |
| 3 | [x] | N/A (records) | [x] | P5a: ARC-332, step 36 §4.5, MODULE_SPEC `body:`, this ledger |
| 4 | [ ] | [ ] | [ ] | — |
| 5 | [ ] | [ ] | [ ] | — |
| 6 | [ ] | [ ] | [ ] | — |
| 7 | [ ] | [ ] | [ ] | — |

### 10.1 P5a findings and bounded discoveries (2026-10-19, audit base `origin/main @ 88735fcf`)

- **D-P5a-1 (bounded, after #343 / ARC-307).** Solids now name a `furnishing`; a furnishing is one piece, so
  overlapping solids that name one furnishing count as one thing without a `group`. The `group` stays for separate
  pieces that interlock (a chair under its table). The street and the store have no furnishings yet, so the terrace
  tables carry `group`s.
- **D-P5a-2 (bounded).** The group grammar is the one `posture`'s `SitGroupKey` actually uses, and `bodies`'
  furnishing key: 1–48 bytes of `a`–`z`, `0`–`9`, `-`, `.` (the design's "letters, digits, `-`, 1–40" misread it).
- **D-P5a-3 (bounded).** The code `bodies-solids-overlap` leads the decode message (a section's decode error carries
  no rejection code of its own); the place is refused when the world loads, at the section's line.
- **D-P5a-4 (bounded).** The moved-start variant is 5 mm inside the band (255 mm), not 100 mm: at 160 mm genesis (R)
  refuses it already, and the `R − 50` mutation would still flag it, so it could not show what this test adds.
- **D-P5a-5 (bounded).** `validate` is given the repository's pack roots (`entities/`, `presentation/mineworld-default`),
  as `packs.rs` does: lakeside requires a data Presentation Pack.
- **Overlap pairs (§2.2), outcome.** Café tables × tucked chairs (5 pairs, 14–63 mm): grouped `seating.east|north|back`.
  Terrace tables' crossed boxes (3 pairs): grouped `terrace-table.1…3`. Bollard × stone planter (2 pairs, 226 × 88 mm
  solid): the drawn meshes meet (bollard base r 0.16 m at z −3.92; planter lip edge at z −3.895, 185 mm overlap), so
  the north planter row moved 200 mm from the kerb, drawn (`streetscape.gd`, z −4.42 → −4.62) and solid
  (y −955…−5 → −755…195), both towns. Window staging × crate (197 × 450 mm): the drawn staging's bottom tier and end leg
  stand inside the drawn crate, so the staging ends 250 mm sooner, drawn (`shop_interior.gd`, `w/2 − 1.00 → w/2 − 1.25`)
  and solid (x max 7200 → 6950), 53 mm clear of the crate, both towns.
- **Golden.** `social-cafe-seed-7-30-days.txt` re-captured (the street's and the store's solids moved): fingerprint
  `1c6f2951889b2159` → `9f690adbf7994320`, facts 115 405 → 105 334, `no-route` rejections 2 024 → 4 051.
- **F-P5a-1 (for the bodies lane, not fixed here).** In the re-captured run hana moves 543 times in 30 days (4 364
  before) and `no-route` doubles: hana spends the later days in the café with routes refused. The café's stored geometry
  did not change (groups are not stored), so the trap is pre-existing and this seed now reaches it; recorded for the
  milestone's integration run.

## 11. Questions for the operator

| Id | Question | Recommendation |
| --- | --- | --- |
| Q-S36-P5-1 | Both towns have 11 overlapping solid pairs, 8 by design (a round table as two crossed boxes, chairs tucked under tables). Let an author mark boxes of one thing with a `group`, and refuse any other overlap when the world loads? | Yes. The group is authoring data only, so no digest moves |
| Q-S36-P5-2 | (Q-S36-9 with Q-VF-F1) Link a drawn prop to its solid by geometry in the check now, and do Q-VF-F1 (b), furniture drawn from solids with `kind` and `yaw`, as its own PR (`S36-P6`) afterwards? | Yes |
| Q-S36-P1-2 | Planted and flush items: a fifth mount `ground` with a declared rest plane, embed depth and reason; C-3 checks the declaration, the scene check checks the rest plane on the ground; grass may declare `walk_through`? | Yes |
| Q-S36-P5-3 | Hold every person start to `R + GAP` (260 mm) from solids in the test, as the step's O-7 says, while genesis keeps `R` (250 mm)? | Yes; content that fails is moved |

## 12. Execution contract (filled at freeze)

```text
PROJECT / PR:          S36-P5a — server layout (commits 1-3); S36-P5b — drawn props vs solids (commits 4-7)
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp1/pr-s36-p5-scene-layout.md
RELATED / BINDING:     step-36-interpenetration.md; pr-s36-p1-contact-module.md; pr-vf-furniture-solids.md;
                       ARC-39; ARC-306; DO-12 (tools/cli/tests/bodies/mod.rs)
IMPLEMENTATION BASE:   origin/main at freeze
APPROVED SCOPE:        §3
FROZEN INVARIANTS:     I-S36-1, I-S36-2, I-S36-4, I-S36-5; I-P5-1, I-P5-2
SEQUENCE:              P5a commits 1-3; P5b commits 4-7 (parallel)
VALIDATION BUDGET:     cargo and Godot only through cast-slots/with-slot.sh; no money
LIVE DOCUMENTATION:    this file, §10
```
