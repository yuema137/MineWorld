# PR 16f — The 3D client reads a Presentation Pack and builds places from the server's disclosure

## DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 0 (2026-10-20), DRAFT
Author:                 16f PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-16f, branch docs/pr-design-16f
Planning base:          origin/main @ 6fe45d58 (audit read at 68a248d1; the two commits between touch only
                        the football model, clients/3d-spike/assets/models/football/, and nothing below)
Approved by / evidence: overall.md "Physics list" decision 3 (QPL-17, G-PL-1) and "16f placement
                        (2026-10-15, S33 Q-S33-4)" bind and are not re-asked. Everything in §12 is open
Implementation base:    main at each sub-PR's C0 (recorded there)
Start condition:        this design frozen; per sub-PR, the dependencies in §5
Execution contract:     §14 (empty until the freeze)
Implementer:            16f-a, 16f-b, 16f-c: Haiku-class against this frozen design; 16f-d1, 16f-d2: Opus-class
                        (asset export, parity). Reviewer: the S14 lane lead
Lifecycle:              DRAFT
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-15-demo-3d.md`](step-15-demo-3d.md) §13 (the 16f row) ·
**Parent:** [`overall.md`](overall.md) §3 ("16f placement") and §5 ("Physics list", decision 3) · **Seed:**
[`pr-s33c-world-seam.md`](pr-s33c-world-seam.md) (`ARC-144`, `ARC-146`, merged as #381) · **Coordinates with, does
not redesign:** #386 (S33-d revision 2, Lakeside's village as data read by generic builders; proposed `ARC-177`),
#387 (S33-a, Lakeside's setting and far fields), S33-b (Lakeside's `body:` sections, frozen, not merged) ·
**Binding parents:** `CLAUDE.md` §§1, 2, 4 (items 3, 5, 7, 11, 13–16); `docs/ENGINEERING_RULES.md` §§2–3, 5–12;
`docs/MODULE_SPEC.md` §6; `docs/PACKAGE_FORMAT.md` §§2, 3, 6; the lane-lead charter (speed first; no
world-specific hard-coding; tools first).

**Decision ids.** S14's range is `ARC-50` … `ARC-52`. `ARC-50` is taken (12e's correction rule,
`docs/decisions/ARC-50.md`). `ARC-51` and `ARC-52` are free on `main` (no file under `docs/decisions/`; only plan
documents name them). This design proposes `ARC-51` (the 3D client reads the pack it is given) and `ARC-52`
(places from disclosure; art decorates). Any further record is `ARC-TBD (coordinator assigns)`.

No model is called by this design, and no language model is involved anywhere in 16f.

---

## 0. The requirement, quoted

From `overall.md` §5, "Physics list", decision 3 (QPL-17):

> The 3D client reads a Presentation Pack and builds places from the server's disclosure, in MVP-0.
> - This is a new S14 PR, **16f**, after 12e.
> - Walls, doors and places come from disclosed geometry, and art only decorates.
> - Changing the Presentation Pack changes the look.
> - Today's hand-built café becomes part of the default Presentation Pack.
> - This completes the "One world, two views" rule.

From `overall.md` §3 ("16f placement"): S33-c is 16f's seed. 16f "generalises that seam to depictions loaded from
a Presentation Pack and walls built from disclosure".

From `ARC-144` (Consequences): "16f keeps `SliceDepiction` as the boundary and changes only where depictions come
from (a Presentation Pack, walls from disclosure; R-16f-c-1)."

From the charter (2026-10-18): mechanisms are generic and data-driven, and every acceptance criterion names a
second world or a generated variant.

---

## 1. Goal, in plain words

Today the 3D client draws a world from GDScript that knows that world. The café's walls, the street's
frontages and the Lakeside stub's boxes are all typed into client code. The client reads only three things from
the Presentation Pack: the far field, the cast and the furnishings. And it reads them from one fixed folder.

After 16f:

1. **The client is given a pack, and reads it from disk.** `--presentation=<dir>` picks the pack, as it already
   does in the 2D client. Swap the folder, and the people, the furniture, the walls and the buildings look
   different. No client file changes.
2. **The server's disclosure decides where walls, doors and floors are.** Each place the server discloses comes
   with its floor, its fixed solids and its doorways. The client builds the walls from those numbers. Only
   those walls stop the player. Art may cover them, but art never blocks, and a probe checks that art and
   disclosure agree.
3. **A world needs no client code to be drawn.** A world is drawn from its pack's data file. Where the pack has
   no art for a place, the place is drawn plainly from its disclosure: a floor, walls with door openings, and
   boxes for the solids.
4. **The café moves into the pack.** Today's hand-built café and street become glTF assets in
   `presentation/mineworld-default/3D/`, placed by the pack's data file. The client keeps no café code.

Example. Today, `./mineworld-slice --world=bodies-yard` is refused (`PRESENTATION_WORLD_MISSING`), because no
script draws bodies-yard. After 16f-b it opens: two grey rooms with the server's own walls, a doorway between
them, and twelve people. With `--presentation=presentation/mineworld-greybox/3D`, Social Café is drawn as grey
boxes too, from the same disclosure, and every probe still passes.

---

## 2. Audit — what exists on `main @ 6fe45d58`

Paths are under `clients/3d-spike/` unless stated.

### 2.1 What 16f builds on (already merged)

| # | Fact | Evidence |
| --- | --- | --- |
| A-1 | The depiction seam exists: `SliceDepiction` with `build`, `place_volumes`, `depictions`, `doors`, `spawn`, `viewpoints`, `sit_points`, `far_field_key`, `hud_note`, and `tag()` as the one place a tag literal is written | `scripts/depictions/depiction.gd:23-131`; #381 (merge `e7c095d0`) |
| A-2 | The registry finds a depiction by path convention, `res://scripts/depictions/<key>/depiction.gd`, then tries `3d-<key>.pck`, then refuses `PRESENTATION_WORLD_MISSING` | `scripts/depictions/registry.gd:17, 35-56`; `ARC-146` |
| A-3 | Three depictions are scripts: `social-cafe` wraps `SliceWorld.build` (61 lines), `market-town` extends it and borrows its far field (8 lines), `lakeside` is a stub data table (82 lines) | `scripts/depictions/{social-cafe,market-town,lakeside}/depiction.gd` |
| A-4 | `SliceBoxDepiction` builds a stub from a data table: ground, one walled box per place, a gap at each doorway | `scripts/depictions/box_depiction.gd:53-90, 149-180` |
| A-5 | `SliceClientBodies` builds invisible colliders on `LAYER_DISCLOSED` from the current place's `place-shape`: four perimeter walls 0.20 m thick and 3.0 m high, a 2.0 m gap at each doorway within 0.5 m of an edge, one box per solid | `scripts/slice/client_bodies.gd:20-33, 42-98, 100-135` (12e, #254 `effd23ae`) |
| A-6 | The same "split a wall at the doorways" logic is written twice: `SliceClientBodies._edge` and `SliceBoxDepiction._walls` | `client_bodies.gd:100-135`; `box_depiction.gd:149-180` |
| A-7 | The geometry probe checks, both ways, that scene colliders (`LAYER_WORLD`) lie within 150 mm of disclosed faces, in the café, the street and the store | `scripts/slice/slice_probe_geometry.gd:1-40`; CI `scripts/ci_layer.py:210` |
| A-8 | Furnishings are drawn from disclosure, by kind, through the pack's `furniture/furnishings.yaml`; offline, from a recorded, drift-checked observation | `scripts/slice/furnishings.gd:25-27, 79-91`; `PRESENTATION.md` §5; `ARC-307` (#343 `88735fcf`) |
| A-9 | The cast binding is read from the pack (`characters/cast.yaml`) | `scripts/slice/cast_binding.gd`; `PRESENTATION.md` §§2-3; `ARC-92` |
| A-10 | `SliceLayout` binds disclosed places to depiction entries by tag set and doorway coincidence; a place no entry matches is a reported miss and is not drawn | `scripts/slice/slice_layout.gd:87-110, 203-350` |
| A-11 | The 2D client already selects a pack with `--presentation=<dir>` or `--presentation=none`, reads it at runtime from disk, and holds that a pack contains no GDScript; with no pack it draws floors and solids plainly and stays playable | `clients/2d/PRESENTATION.md` §§1, 6; `clients/2d/scripts/app.gd:46, 94` |
| A-12 | The default 3D pack has `pack.yaml` (`id: mineworld-default-3d`), `manifest.yaml`, `far_field/{social-cafe,lakeside}.yaml`, `characters/`, `furniture/furnishings.yaml` | `presentation/mineworld-default/3D/` |
| A-13 | Runtime glTF loading is already used: `Props.gltf` falls back to `GLTFDocument` when no import cache exists | `scripts/props.gd:55-95` |
| A-14 | The project is Godot 4.7 | `project.godot:11` |

### 2.2 What remains (16f's scope)

| # | Gap | Evidence |
| --- | --- | --- |
| G-1 | **The pack is a constant.** `PACK_DIR := "../../presentation/mineworld-default/3D"` is read in four files; `identity_check.gd` has its own copy. No flag selects another pack | `scripts/slice/slice_main.gd:110, 162`; `slice_link.gd:148-151`; `slice_world.gd:99-100`; `identity_check.gd:17` |
| G-2 | **The 3D client's own words are not in its pack.** They sit in the shared layer "until the 3D client reads a Presentation Pack (S14 16f)"; the 3D client loads no pack layer | `clients/shared/SETTINGS.md:333`; `slice_main.gd:87, 199` (`load_layers("", …)`) |
| G-3 | **Disclosed walls are invisible.** The only visible walls are art. A place no depiction draws is not drawn at all, even though its floor, solids and doorways are disclosed | A-5, A-10 (`is_drawn`, `client_bodies.gd:50`) |
| G-4 | **Art blocks the player.** Art colliders are on `LAYER_WORLD`, which is in the player's mask, so art and disclosure both stop the body | `scripts/build.gd:6, 41, 71, 112, 156, 169`; `scripts/player.gd:37-38, 47` |
| G-5 | **A world without a client script cannot be drawn.** bodies-yard has `body:` sections on both places and no 3D far field and no depiction | `worlds/bodies-yard/places/*.yaml`; A-2 |
| G-6 | **Depictions are client code, not pack data.** S33-d revision 2 (#386, draft) adds `presentation/<style>/3D/depictions/<key>.yaml` and `SliceDataDepiction`, but still needs a one-line script per world: "A world present with no depiction script (data file alone, found by the registry)" is listed as 16f's | #386 §2.2 (the row "16f, which changes only where depictions come from") |
| G-7 | **The café is client code.** `SliceWorld.build` calls `SliceStreet`, `SliceCafe`, `SliceTerrace` (eleven frontage units as tables), `SliceStreetscape`, the interiors, `SliceBatch`, `SliceBudget`, `SliceOccluders`; about 4 000 lines under `scripts/slice/` draw only Social Café's street | `slice_world.gd:85-122, 131-250`; `cafe.gd` (474), `cafe_interior.gd` (585), `shop_interior.gd` (403), `streetscape.gd` (411), `terrace.gd` (361), `street.gd` (136) |
| G-8 | **Market Town borrows the café's script and far field.** Its depiction is "that depiction, and that street's far field, until market-town has a depiction of its own" | `scripts/depictions/market-town/depiction.gd:1-8` |
| G-9 | **No second pack exists** to prove that swapping the pack changes the look | `presentation/` holds only `mineworld-default/{2D,3D}` |

### 2.3 Facts that constrain the design

| # | Fact | Evidence | Consequence |
| --- | --- | --- | --- |
| C-1 | Disclosure is the observer's own place only: "a person learns the walls and the objects of the room they stand in and of no other" | `systems/bodies/src/system.rs:307-344`; `ARC-307` "Not decided here" (Q-FD-3) | From the street, the client knows the street's floor and doorways, not the neighbours' rooms. 16f draws a neighbour from art, or from memory once visited (§4.3). Widening disclosure is §12 Q-16f-2 |
| C-2 | Frames are glued at disclosed passages by translation only | `ARC-45` points 1-2 | An unbound place is placed in the scene by its passage, never by a guess |
| C-3 | A published asset is glTF 2.0 (`.glb` preferred); no engine-native scene is a published asset; an asset carries `asset.yaml` with `semantic_bindings` | `PACKAGE_FORMAT.md` §§2-3; `MODULE_SPEC.md` §6.3 | The café moves into the pack as glTF, not as `.tscn` or GDScript |
| C-4 | GDScript in a pack is Tier 2 (renderer extension); Tier 2 is "available by virtue of Godot; unused" in MVP-0 | `PACKAGE_FORMAT.md` §§6, 8 | 16f keeps packs Tier 0: data and media only |
| C-5 | A fallback depiction was declined: "a fallback looks like the world and is not it" | `ARC-146` option (b), Q-S33-3 | Plain drawing from disclosure is not that fallback (every wall drawn is the server's), but it changes `ARC-146`'s refusal, so it is §12 Q-16f-1 |
| C-6 | Lakeside has no `body:` sections on `main`; S33-b adds them (frozen, not merged) | `grep -l "^body:" worlds/lakeside/places/*.yaml` is empty | Lakeside's disclosure checks wait for S33-b; 16f's two-world acceptance uses social-cafe and bodies-yard, plus market-town |
| C-7 | Market Town's geometry is Social Café's ("at the same position as in Social Café") | `worlds/market-town/world.yaml:66` | Market Town alone is not a second geometry; bodies-yard is |
| C-8 | Runtime glTF loads miss the importer's LODs and shadow meshes; props without LODs cost 8-9.5 M of a frame's primitives | `scripts/props.gd:58-65` (E-RLb-2) | 16f-d must keep the perf budget (§10 R-3) |

---

## 3. Reuse, both ways (tools first)

| Need | Options compared | Choice | Why |
| --- | --- | --- | --- |
| Visible walls with door openings, from a floor rectangle and doorway points | (1) **Godot CSG** (`CSGBox3D` minus one box per opening, Manifold backend, `bake_static_mesh()`); (2) **GridMap + MeshLibrary** (a modular tile kit); (3) **func_godot** (MIT; Quake `.map` brushes to meshes and colliders); (4) the existing span split (A-6) as one shared function, then `Build.box` pieces | **(4), with (1) kept as the documented upgrade** | Every opening the format can state is a rectangle in a straight wall, so a span split is exact and already written twice. CSG costs a boolean pass per place and a bake, and its triangulation is not promised stable across Godot versions, which would move the shot hashes. GridMap needs grid-aligned footprints; disclosed floors are any millimetre size. func_godot reads authored map files, not runtime rectangles. One shared function also removes the duplicate in A-6 |
| Pack selection on the command line | (1) the 2D client's `--presentation=<dir>` (A-11); (2) a new 3D flag name | **(1)** | One flag for both clients; the 2D spec already defines its meaning |
| The café as pack content | (1) **Godot `GLTFDocument`** export (built in: `append_from_scene`, `write_to_filesystem`); (2) **Blender** glTF exporter (Apache-2.0 add-on, run headless); (3) **glTF-Transform** (MIT) as an optional post-step (dedupe, KTX2/Basis textures, meshopt) | **(1) to export; (3) only if the size or perf gate fails** | The café is built by Godot code today; Godot's own exporter writes exactly what Godot built. Blender would need the scene re-imported first. glTF-Transform does not export, it only optimises |
| Loading pack art at runtime | (1) `GLTFDocument.append_from_file` + `generate_scene` (A-13); (2) importing into the project (`.import`); (3) `.pck` per world (`ARC-146`) | **(1)**, with `ImporterMesh.generate_lods()` on load if the perf gate needs it | A pack read from disk is what makes "swap the folder" true (A-11). Import binds art to the client project. `.pck` is engine-native and stays only as `ARC-146`'s export packaging |
| Data depictions (terrain, shells, trees) | S33-d revision 2's `depictions/<key>.yaml` and builders (#386) versus a 16f format | **S33-d's format** | It exists in a design already under review; 16f adds only what a world without a script and the café need (§4.4, §4.5) |

Rejected own code: no new wall solver, no new file format beside S33-d's, no new layout rule beside `ARC-45` and
16c's binding.

---

## 4. Design

### 4.1 The pack the client is given (16f-a; `ARC-51`)

- **Selecting it.** `--presentation=<dir>`: a 3D pack directory, absolute or relative to the repository root.
  Absent: `presentation/mineworld-default/3D`. The launcher `mineworld-slice` forwards the flag. (There is no
  `--presentation=none` in 3D: every view needs a far field, `ARC-102`; plain drawing is what a pack gets for a
  place it has no art for, §4.3.)
- **Reading it.** One small class, `SlicePack` (`scripts/pack/pack.gd`), resolves the directory once and is the
  only place the path is built. It reads `pack.yaml` and checks `type: presentation-pack`, and reads
  `manifest.yaml` and checks that `dimension` contains `3d`. Every reader that used `PACK_DIR` (G-1) asks
  `SlicePack.current.path("<relative path>")` instead.
- **Refusing.** A directory with no `pack.yaml`, the wrong type or no `3d` dimension is refused by name in
  RL-e0's line format: `[mineworld-slice] PRESENTATION_PACK_INVALID: { pack: <dir>, why: <text> }`, exit 3 in a
  scripted run. A pack that lacks an optional file (`characters/cast.yaml`, `furniture/furnishings.yaml`,
  `places/shells.yaml`, `depictions/`) is not refused: that part is drawn plainly, as `PRESENTATION.md` §3 and
  §5 already say.
- **Words.** The 3D client loads its pack's `i18n/` as the middle layer (`MineWorldText.load_layers(pack_dir,
  …)`), and its own keys move, unchanged, from `clients/shared/settings/locale/` to
  `presentation/mineworld-default/3D/i18n/` (the obligation in `clients/shared/SETTINGS.md:333`).
- **The second pack.** `presentation/mineworld-greybox/3D/` (Q-16f-3): `pack.yaml`, `manifest.yaml`, a
  `README.md`, and `far_field/<key>.yaml` copied from the default pack for every world it presents. No cast, no
  furnishing bindings, no depictions. Drawn with it, people are the client's plain figure (`PRESENTATION.md` §3,
  "no pack binds"), and furniture is plain boxes over its solids (`PRESENTATION.md` §5). From 16f-b on, it also
  carries `places/shells.yaml` with flat greys, so places are greybox rooms.

Example. `./mineworld-slice --world --presentation=presentation/mineworld-greybox/3D` opens the same server
and the same café. The people are plain capsule figures, the tables and chairs are timber boxes, and the
`FURNISHED` line reports every kind `unbound`. `git diff --stat clients/` is empty.

### 4.2 Blocking geometry comes only from disclosure (16f-b; `ARC-52`)

- A new layer, `LAYER_ART = 16`, in `Build`. Every collider an art builder makes for a wall, a façade, a solid or
  a door jamb is put on `LAYER_ART`. The ground and terrain stay on `LAYER_WORLD`: the server's plan has no
  height, so the drawn ground is what the feet stand on (12e, unchanged).
- The player's mask becomes `LAYER_WORLD | LAYER_DISCLOSED | LAYER_BODIES | LAYER_OBJECTS` without `LAYER_ART`
  (`player.gd:37-38, 47`). The camera boom keeps meeting art, so the camera does not pass through a drawn wall.
  The target ray keeps its mask.
- The geometry probe (A-7) switches its scene mask from `LAYER_WORLD` to `LAYER_ART | LAYER_WORLD`, so it still
  measures art against disclosure. It now runs in every place the player can stand in, on every world in §8.
- A place with no disclosed `place-shape` (a world without `bodies`) keeps its art colliders on `LAYER_WORLD`,
  as today, because nothing else would stop the body. `SliceClientBodies` already builds nothing there (A-5).

Example. In the café, today the body is stopped by the café's stone wall (art) and by the invisible disclosed
wall 0.2 m behind it, whichever comes first. After 16f-b only the disclosed wall stops it. If the art wall is
drawn 0.3 m off, the probe fails and names the face; the player is never stopped by a wall the server does not
have.

### 4.3 Places drawn from disclosure (16f-b; `ARC-52`)

- **The shell builder.** `SliceShells` (`scripts/places/shells.gd`) draws one place from its disclosed
  `place-shape` and its passages: the floor as a slab; four perimeter walls outside the floor, split at every
  doorway by the one shared span function (§3, replaces A-6's two copies); a lintel over each opening; each
  shell solid (a solid that names no furnishing, `CORE_CONCEPTS.md` §6.5) as a box of its footprint and height.
  The colliders stay `SliceClientBodies`'s; the shell builder draws only.
- **Its look comes from the pack.** `places/shells.yaml` (new optional pack file, YAML's JSON subset,
  `DEP-16`), bound by tag with `.`-prefix fallback like furnishings:

  ```json
  {
    "format": 1,
    "default": { "wall": { "material": "plaster_grey", "height_m": 2.7 }, "floor": { "material": "concrete" },
                 "opening": { "width_m": 1.0, "height_m": 2.1 }, "solid": { "material": "timber_plain" } },
    "tags": {
      "cafe":  { "wall": { "material": "painted_plaster_warm" }, "floor": { "material": "oak_boards" } },
      "street": { "wall": null, "floor": { "material": "setts" } }
    }
  }
  ```

  A place's binding is the first tag of its disclosed tags, in disclosed order, that `tags` binds; otherwise
  `default`. `"wall": null` draws no walls (a street's edges are the buildings' fronts, which art or the
  neighbours draw). A material is a slug of the client's texture library, as in S33-d's format. A missing file
  draws the client's plain grey. An opening's drawn width is a drawing size, never a rule: the server decides
  who passes.
- **Which places get a shell.** Every disclosed place that no depiction entry draws (A-10's misses today) is
  glued into the scene by its passage (`ARC-45` point 1) and drawn by the shell builder. A place a depiction
  draws keeps its art and gets no shell; the probe checks that art (§4.2).
- **What stays drawn.** The current place, and every place learned in this world instance, stays drawn
  (`ARC-45` point 2: a presentation cache, dropped when `instance` changes). People and objects are drawn only
  in the current place, unchanged (`INV-13`).
- **A world with no depiction.** Under Q-16f-1 (A), a world whose pack carries its far field but no depiction is
  drawn this way, with the HUD note `hud.depiction.plain` ("drawn from the world's own walls"), like S33-c's stub
  label. A world whose pack lacks its far field is still refused (`FAR_FIELD_WORLD_UNKNOWN`, unchanged).
  `PRESENTATION_WORLD_MISSING` remains only for a key that is not a valid world key. bodies-yard gets
  `far_field/bodies-yard.yaml` in the default pack (plain countryside, written with `scripts/far_field_variant.py`
  from social-cafe's; presentation data, cited in its `source`).
- **Offline.** With no server, the shells of the anchor place come from the recorded observation that
  furnishings already use (`tests/fixtures/observations/furnishings/`, A-8), extended to one file per world in §8
  that has one.

Example. bodies-yard has two rooms. Standing in the first, the client draws its floor, four grey walls and a gap
where the passage leads to the second. Walk through: the second is disclosed, glued at the passage, and drawn;
the first stays drawn behind you.

### 4.4 Depictions from the pack's data (16f-c)

- **Lookup order** in `SliceWorlds.open(key)`:
  1. `<pack>/depictions/<key>.yaml` → `SliceDataDepiction` (S33-d revision 2's reader and builders), with no
     script under `scripts/depictions/<key>/`;
  2. `res://scripts/depictions/<key>/depiction.gd` (a bespoke builder shipped with the client; after 16f-d2 no
     world has one);
  3. `3d-<key>.pck` (`ARC-146`, unchanged);
  4. Q-16f-1 (A): plain drawing from disclosure (§4.3); (B): refuse, as today.
- **The pack decides, not the client.** The client keeps no list of worlds. Which worlds a pack presents is the
  set of its `far_field/` and `depictions/` files.
- **Lakeside.** When S33-d's C3 has merged, `scripts/depictions/lakeside/depiction.gd` (one line,
  `extends SliceDataDepiction`) is deleted; Lakeside is drawn from `depictions/lakeside.yaml` alone. The greybox
  pack has no Lakeside depiction, so under Q-16f-1 (A) Lakeside in greybox is drawn plainly once S33-b gives it
  `body:` sections.
- 16f-c changes nothing in S33-d's format and nothing in its builders. It only changes where the registry looks.

### 4.5 Today's café becomes part of the default pack (16f-d1, 16f-d2)

- **The source moves to a tool.** The café builders (`SliceStreet`, `SliceCafe`, `SliceCafeInterior`,
  `SliceShopInterior`, `SliceTerrace`, `SliceStreetscape`, `SliceProps` dressing, `SlicePalette`) move, unchanged,
  from `clients/3d-spike/scripts/slice/` to `tools/presentation/social_cafe/` (Godot headless). They are now the
  authoring source of an asset, not runtime code.
- **The tool writes glTF.** `tools/presentation/export_place_art.gd` builds the scene as today, splits it into
  pieces (the café, the florist, the north frontage, the south frontage, the street dressing), and writes each
  with `GLTFDocument.append_from_scene` and `write_to_filesystem` to
  `presentation/mineworld-default/3D/assets/places/<asset-id>/<asset-id>.glb`, with an `asset.yaml` in
  `PACKAGE_FORMAT.md` §3's form: `type: place_visual`, `semantic_bindings.entrances` naming the door nodes,
  `semantic_bindings.spaces` naming the room volumes, `scale.unit: meter`, the licence of every source texture
  (Poly Haven CC0, the repository's MIT). Lights travel as `KHR_lights_punctual`. Custom shader materials (glass,
  water) are exported under a material name and re-bound on load by the pack's
  `materials/material_rules.yaml` (`MODULE_SPEC.md` §6.1's listed file), so no shader is lost.
- **The pack places it.** An additive optional section `models` in S33-d's depiction format (amends the proposed
  `ARC-177`; coordinated with #386, Q-16f-7):

  ```json
  "models": [
    { "asset": "social-cafe-cafe", "depicts": ["cafe"], "anchor": { "entrance": "front_door", "at_doorway": true } },
    { "asset": "social-cafe-north-frontage", "depicts": null, "anchor": { "at_mm": [0, 9000], "yaw_deg": 0 } }
  ]
  ```

  `at_doorway` puts the asset's named entrance on the depicted place's doorway, so no coordinate is written twice.
  `depicts: null` is a piece nobody enters. The VoxelGI and reflection probes go in the file's `light` section
  (S33-d's). Social Café's file is `depictions/social-cafe.yaml`; Market Town's is `depictions/market-town.yaml`,
  which places the same assets and names social-cafe's far field (G-8, by data now).
- **16f-d1 adds; 16f-d2 switches.** 16f-d1 adds the tool, the assets and their validation, and writes the two
  depiction files under `tools/presentation/social_cafe/depictions/`, outside the pack, so the registry does not
  find them yet and the runtime is untouched. They can be drawn on demand with S33-d's
  `--slice-depiction-file=<path>`, which is how d1 previews parity. 16f-d2's first commit moves the two files into
  `presentation/mineworld-default/3D/depictions/`, so lookup step 1 finds them; its second commit deletes
  `scripts/depictions/social-cafe/`, `scripts/depictions/market-town/` and the moved builders from the client.
- **Nothing pretends.** Every asset collider is on `LAYER_ART` (§4.2). Every enterable room's art passes the
  geometry probe. Furnishings stay the world's (`ARC-307`); the export leaves them out.

### 4.6 What 16f does not do

| Item | Where |
| --- | --- |
| Widening disclosure to neighbouring places' shapes (Q-FD-3) | not 16f unless Q-16f-2 says so; a later bodies design |
| Lakeside's terrain, water, shells, woods and their format | S33-d (#386) |
| Lakeside's `body:` sections | S33-b |
| Multi-pack loading (a pack that depends on another pack, PK-3) | after MVP-0 |
| `.mwpack`, `mineworld validate asset` | `PACKAGE_FORMAT.md` §8: after the vertical slice |
| Splitting the export into per-world `.pck` files | S33-j |
| Any server, protocol, kernel, contract or System change | never in 16f, unless Q-16f-2 (A) |
| The 2D client | unchanged; it already reads its pack (A-11) |

---

## 5. Sub-PR split

Each sub-PR is mergeable on its own and ends at a real integration checkpoint.

| PR | Scope | Starts | Integration checkpoint (run for real) | Adversarial criterion |
| --- | --- | --- | --- | --- |
| **16f-a** The pack the client is given | §4.1: `SlicePack`, `--presentation=`, the refusal, every `PACK_DIR` reader moved, the 3D i18n layer, the greybox pack (pack.yaml, manifest, far fields) | **Now** | `--world --link` and `--world=market-town --buy` pass with each pack; the greybox shots show plain figures and plain furniture | A pack dir with `type: world-pack` is refused by name, exit 3; a `PACK_DIR` literal anywhere in the client fails a scan |
| **16f-b** Places from disclosure | §§4.2-4.3: `LAYER_ART`, the player's mask, `SliceShells`, `places/shells.yaml`, unbound places glued and drawn, the learned-places cache, the plain world (Q-16f-1), `far_field/bodies-yard.yaml` | **After 16f-a** | `--world=bodies-yard --link` walks both rooms; `--world --geometry` passes with art on `LAYER_ART`; greybox Social Café walks the café and the store | With the art walls moved 0.5 m by a planted mutation, the player is still stopped at the disclosed wall and the probe fails naming the face |
| **16f-c** Depictions from pack data | §4.4: the lookup order; Lakeside's one-line script deleted | **After 16f-a and S33-d revision 2's C3** | `--world=lakeside --link` draws from `depictions/lakeside.yaml` with no `scripts/depictions/lakeside/` | A depiction file whose `scope.world` differs from its stem is refused `DEPICTION_SCOPE_MISMATCH` (S33-d's code) |
| **16f-d1** The café's art as pack assets | §4.5: the tool, the moved builders (tool side only, copies), the `.glb` assets and `asset.yaml`, `materials/material_rules.yaml`, the `models` section and its validation, the two depiction files kept outside the pack | **After 16f-b and 16f-c** | `mineworld packs validate presentation/mineworld-default/3D` passes; the tool re-run is byte-identical (`cmp`) | An asset with no licence, or a `models` entry naming an asset that does not exist, is refused by name |
| **16f-d2** The switch | §4.5: the client draws Social Café and Market Town from the files; the café code leaves the client | **After 16f-d1** | S33-c's shot set on social-cafe meets the parity bar (Q-16f-6); the perf views hold their budget; `--world --link`, `--geometry`, `--bodies`, `--buy` pass | `grep` finds no world-specific builder under `clients/3d-spike/scripts/`; a copy of the client with `presentation/` swapped for greybox passes every probe |

```text
 now ──► 16f-a ──┬──► 16f-b ──────────────┐
                 │                        ├──► 16f-d1 ──► 16f-d2
                 └──► 16f-c ◄── S33-d C3 ─┘
```

16f-b and 16f-c may run in parallel. Within the lane's limit of two code-PR CI runs at once.

---

## 6. Invariants

| Id | Invariant |
| --- | --- |
| I-16f-1 | The client sends only `ActionIntent`s. No pack value changes which requests are sent (`PRESENTATION.md` §4) |
| I-16f-2 | Only disclosed geometry blocks the player where a place discloses `place-shape` (§4.2) |
| I-16f-3 | No client script names a world key, a place key or a tag except through `SliceDepiction.tag()`; `client_rules.rs` check (d) still passes |
| I-16f-4 | A pack is Tier 0: data and media only, no GDScript, no `.tscn` (`PACKAGE_FORMAT.md` §§2, 6) |
| I-16f-5 | No server, protocol, kernel, contract or System file changes (unless Q-16f-2 (A) is chosen) |
| I-16f-6 | Every number in a builder is a unit conversion, a cited physical default shared by every world, or a pack field |
| I-16f-7 | Same input, same scene on every OS: no unseeded randomness (the far field's and S33-d's rule) |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | GDScript parse errors (Godot `--check-only` in CI's `clients` layer); `cargo clippy`/`fmt` for Rust validators | existing CI |
| Unit (Rust) | `mineworld packs validate` of a 3D pack's `places/shells.yaml`, `models` section and `asset.yaml`: each fault refused by its code, a corpus file per code | `packages/tests/` (beside S33-d's depiction corpus) |
| Unit (scan) | No `PACK_DIR` literal; no world-specific builder under `clients/3d-spike/scripts/` after 16f-d2; check (d) | `tests/acceptance/tests/client_rules.rs` |
| Unit (GDScript check) | The shared span function: given a wall and doorways, the pieces and gaps (hand-computed expected values) | `clients/3d-spike/scripts/checks/span_check.gd`, run by `tools/cli/tests/` like the existing client checks |
| Gate 2 (real lifecycle) | A real server, a real client, a real walk: binding, shells, blocking, the probe, shots, perf | the slice probes in `scripts/ci_layer.py`'s `clients-probes` layer |
| Gate 1 | **Not required.** No LM-facing semantics change | — |
| CI impact | `clients-probes` gains the rows in §8; `packages` tests gain the corpus. Not a full-suite trigger except 16f-d2, which completes the step's acceptance (charter: AC or milestone → `full-ci`) | — |

---

## 8. Acceptance criteria (decided before measuring)

Worlds: **social-cafe**, **market-town** (same geometry, different world), **bodies-yard** (different
geometry), and **lakeside** once S33-b and S33-d have merged. Packs: **mineworld-default/3D** and
**mineworld-greybox/3D**.

| Id | PR | Criterion | Evidence |
| --- | --- | --- | --- |
| AC-16f-1 | a | `--presentation=<dir>` selects the pack; absent, the default. Every reader of the pack goes through `SlicePack` | the scan finds no `PACK_DIR` literal; `--world --link` passes with both packs |
| AC-16f-2 | a | A bad pack is refused by name: no `pack.yaml`; `type` not `presentation-pack`; no `3d` dimension. Each prints `PRESENTATION_PACK_INVALID` with the path and exits 3 | three scripted runs, their lines and exit codes recorded |
| AC-16f-3 | a | **Pack swap, people and furniture.** On social-cafe and market-town, the greybox pack draws every person as the plain figure and every furnishing as plain boxes, and `FURNISHED` reports every kind `unbound`; `git diff --stat clients/ systems/ server/` between the two runs is empty | the two shot sets side by side; the `FURNISHED` lines |
| AC-16f-4 | a | The 3D words come from the pack layer: with the default pack every 3D key reads as before; with a pack whose `i18n/en.po` overrides `hud.depiction.stub`, the HUD shows the override | `glyph_check`/`menu_check`-style run; one screenshot |
| AC-16f-5 | b | **Only disclosure blocks.** In the café and the store of social-cafe, and in both rooms of bodies-yard, the player walked at each wall stops within 50 mm of the disclosed face; with the art walls offset 0.5 m by a planted mutation it still stops there, and `--geometry` fails naming the face | `--bodies` rows; the mutation run's failing row |
| AC-16f-6 | b | **Places from disclosure, two geometries.** `--world=bodies-yard --link` walks hall → court → hall (`worlds/bodies-yard/places/{hall,court}.yaml`); both rooms are drawn from their `place-shape` with an opening at the passage; `SHOWN` lists both; no `PRESENTATION_WORLD_MISSING` | the probe's lines; two frames |
| AC-16f-7 | b | **Pack swap, places.** With greybox, social-cafe's café, street and store are drawn as shells bound by tag from `places/shells.yaml`, and `--link`, `--geometry`, `--bodies` pass; with the default pack the same run draws the café's art | both shot sets; the probe lines |
| AC-16f-8 | b | A place no depiction entry draws is glued at its passage and drawn as a shell, never a reported miss. Example: social-cafe's station hall (S29-a; its passage leads to the street), walked into from the street by the link probe | the `--layout` line naming the place as `shell`; one frame |
| AC-16f-9 | c | Lakeside is drawn from `depictions/lakeside.yaml` with no Lakeside script; S33-d's variant `lakeside-v7` draws with `--slice-depiction-file=` and no GDScript change | `--world=lakeside --link`; the variant's shot |
| AC-16f-10 | c | With greybox (no Lakeside depiction), Lakeside is drawn plainly from disclosure (needs S33-b) or, under Q-16f-1 (B), refused by name | the run's line |
| AC-16f-11 | d1 | `mineworld packs validate` passes on both packs and refuses each corpus fault by its code; the export tool re-run writes byte-identical `.glb` files | the test output; `cmp` |
| AC-16f-12 | d2 | **The café is the pack's.** No world-specific builder or depiction script remains under `clients/3d-spike/scripts/`; social-cafe and market-town are drawn from `depictions/*.yaml` and `.glb` assets | the scan; `ls scripts/depictions/` lists only `depiction.gd`, `registry.gd`, `box_depiction.gd`, `data/` |
| AC-16f-13 | d2 | **Parity.** On S33-c's social-cafe shot set, each frame meets the bar chosen in Q-16f-6 against the base captures taken at 16f-d2's C0; the operator looks at the pairs | the per-shot metric table; the pairs in the review queue |
| AC-16f-14 | d2 | **Perf.** The perf views (`SlicePerf.VIEWS`) hold the frame budget step-15 set, ±10 % of the base | `--perf` table |
| AC-16f-15 | d2 | **Whole swap.** A run with `--presentation=presentation/mineworld-greybox/3D` on social-cafe, market-town and bodies-yard passes `--link`, `--geometry`, `--bodies` and `--buy` (market-town), and draws no café art | probe lines; shots |

---

## 9. Commit plan

Every commit follows the same procedure: implement; run only the checks it owns (R-LOAD-1: one Godot probe at a
time, through `with-slot.sh`; never `cargo test --workspace`); update the ledger (§15); commit. Each item has
three boxes: **[impl]**, **[valid]**, **[review]** (an LLM logic review of the diff against this design). None
completes another.

### 16f-a

**a-C0 — Base and ledger (Markdown only).** Record `main`'s commit; confirm A-1…A-14 still hold; list every
`PACK_DIR`/`PACK` reader.
- [ ] impl · [ ] valid: the audit table re-checked with `grep` · [ ] review

**a-C1 — `SlicePack`, the flag, the refusal.** `scripts/pack/pack.gd`; `slice_main.gd` resolves the pack before
the far field; `mineworld-slice` forwards `--presentation=`.
- Scope: `scripts/pack/pack.gd` (new), `slice_main.gd`, `mineworld-slice`. Non-goal: no reader moves yet.
- [ ] impl · [ ] valid: AC-16f-2's three runs · [ ] review
- Failure cases: a relative path resolves against the repository root, as in 2D; a path with `..` that leaves
  the repository is allowed (a user's own pack); a missing directory is `PRESENTATION_PACK_INVALID`.

**a-C2 — Every reader through `SlicePack`.** `slice_main.gd:162`, `slice_link.gd:148-151`,
`slice_world.gd:99-100`, `identity_check.gd:17`, the cast and furnishings loaders; `PACK_DIR` deleted.
- [ ] impl · [ ] valid: `--world --link`, `--world=market-town --buy` with the default pack, outputs equal to
  the base · [ ] review

**a-C3 — The 3D words in the pack.** Move the 3D client's own keys to `presentation/mineworld-default/3D/i18n/`
(`en.po`, `zh_Hans.po`), keys and text unchanged; `load_layers(pack_dir, …)` in `slice_main.gd:87, 199`;
`SETTINGS.md` row updated.
- [ ] impl · [ ] valid: AC-16f-4 · [ ] review

**a-C4 — The greybox pack.** `presentation/mineworld-greybox/3D/{pack.yaml, manifest.yaml, README.md,
far_field/social-cafe.yaml, far_field/lakeside.yaml}`; `mineworld packs validate` on it.
- [ ] impl · [ ] valid: AC-16f-3; `packs validate` output · [ ] review

**a-C5 — The scan and close-out.** `client_rules.rs`: no `PACK_DIR`-style pack path literal in the 3D client;
`PRESENTATION.md` §0 (selecting a pack, mirroring 2D §1); `ARC-51`; `MODULE_SPEC.md` §6.1 one line.
- [ ] impl · [ ] valid: `cargo test -p mineworld-acceptance --test client_rules` with a planted literal (red) and
  without (green) · [ ] review

### 16f-b

**b-C0 — Base, captures.** Record `main`; capture `--geometry`, `--bodies` and shots on social-cafe as the base.
- [ ] impl · [ ] valid · [ ] review

**b-C1 — One span function.** `scripts/places/spans.gd`: split a wall at openings; `SliceClientBodies._edge` and
`SliceBoxDepiction._walls` call it; `span_check.gd` with hand-computed cases (no doorway, one, two overlapping, one
at a corner, one outside the span).
- [ ] impl · [ ] valid: `span_check`; `--world --geometry` and `--world=lakeside --link` unchanged · [ ] review

**b-C2 — `LAYER_ART`.** `Build` gains the layer; every art collider builder (`build.gd:41, 71, 112, 156, 169` and
the callers that pass a wall flag) uses it except ground; the player's and press-cast masks drop it; the boom
keeps it; the probe masks `LAYER_ART | LAYER_WORLD`.
- Failure case: a place with no `place-shape` keeps art on `LAYER_WORLD` (§4.2, last point).
- [ ] impl · [ ] valid: AC-16f-5 with its planted mutation · [ ] review

**b-C3 — `SliceShells` and `places/shells.yaml`.** The builder, the binding by tag with `.`-fallback, the plain
default; the Rust validator for the file and its corpus (`SHELLS_MALFORMED`, `SHELLS_OUT_OF_RANGE`,
`SHELLS_REFERENCE`).
- [ ] impl · [ ] valid: the corpus test; one shot of a shell · [ ] review

**b-C4 — Unbound places drawn.** `SliceLayout` glues a miss at its passage and hands it to `SliceShells`; the
learned-places cache, dropped on `instance` change.
- [ ] impl · [ ] valid: AC-16f-8 · [ ] review

**b-C5 — A world with no depiction (per Q-16f-1).** The registry's last step; `hud.depiction.plain`;
`far_field/bodies-yard.yaml` (default and greybox); the offline recorded observation for bodies-yard.
- [ ] impl · [ ] valid: AC-16f-6 · [ ] review

**b-C6 — Greybox shells; CI rows; close-out.** `presentation/mineworld-greybox/3D/places/shells.yaml`;
`ci_layer.py` gains `slice --world=bodies-yard --link`, `slice --world --geometry --presentation=…greybox…`;
`ARC-52`; `ARC-146` note per Q-16f-1; `PRESENTATION.md` §6 (shells); `CORE_CONCEPTS.md` §6.5's "drawn by the
client's depiction" sentence amended to "by the pack, or plainly from disclosure".
- [ ] impl · [ ] valid: AC-16f-7 · [ ] review

### 16f-c

**c-C0 — Base.** Confirm S33-d's C3 is on `main` (`SliceDataDepiction`, `SliceDepictionFile`).
- [x] impl · [x] valid · [x] review

**c-C1 — The lookup order.** `registry.gd`: step 1 (pack file) before step 2 (script); the depiction's source
printed (`[slice] world <key>, drawn by <path>`).
- [x] impl · [x] valid: `--world --link` still drawn by the script (no file yet) · [x] review

**c-C2 — Lakeside from data alone.** Delete `scripts/depictions/lakeside/`.
- [x] impl · [x] valid: AC-16f-9, AC-16f-10 · [x] review

**c-C3 — Close-out.** `ARC-144` note ("Revisit when 16f loads depictions from a Presentation Pack": done);
`ARC-146` note.
- [x] impl · [x] valid: docs only · [x] review

### 16f-d1

**d1-C0 — Base.** Record `main`; capture S33-c's social-cafe shot set and the perf table as the base for d2.
- [ ] impl · [ ] valid · [ ] review

**d1-C1 — The tool and its sources.** `tools/presentation/social_cafe/` (copies of the builders, in d1 only);
`tools/presentation/export_place_art.gd`; run through `with-slot.sh`.
- [ ] impl · [ ] valid: two runs, `cmp` equal · [ ] review

**d1-C2 — Assets and `asset.yaml`.** The `.glb` pieces under `presentation/mineworld-default/3D/assets/places/`;
`asset.yaml` each; `LICENSES/` entries; `materials/material_rules.yaml`.
- Size check: the pack's added bytes recorded; if above the budget the coordinator sets (Q-16f-6), glTF-Transform
  is run (§3).
- [ ] impl · [ ] valid: `packs validate`; licence check · [ ] review

**d1-C3 — The `models` section and its validator.** Additive to S33-d's format (codes `DEPICTION_REFERENCE` for
an unknown asset or entrance, `DEPICTION_DOOR` for `at_doorway` without `depicts`); the client reader accepts it.
- [ ] impl · [ ] valid: corpus files per code · [ ] review

**d1-C4 — The two files, outside the pack.** `tools/presentation/social_cafe/depictions/social-cafe.yaml` and
`market-town.yaml` (`models`, `light`, `viewpoints`, `spawn`, bindings), validated; drawn only with
`--slice-depiction-file=` for a parity preview.
- [ ] impl · [ ] valid: AC-16f-11; one preview shot per file · [ ] review

### 16f-d2

**d2-C1 — The switch.** The two files move into `presentation/mineworld-default/3D/depictions/`, so the
registry's step 1 finds them; the models loader (`GLTFDocument.append_from_file`, the material re-binding, LODs if needed).
- [ ] impl · [ ] valid: AC-16f-13, AC-16f-14 · [ ] review

**d2-C2 — The café leaves the client.** Delete `scripts/depictions/social-cafe/`, `scripts/depictions/market-town/`
and the moved builders under `scripts/slice/`; the probes that read their constants read the depiction's
`viewpoints()` and `doors()` instead.
- [ ] impl · [ ] valid: AC-16f-12, AC-16f-15 · [ ] review

**d2-C3 — Close-out.** `MVP_STATUS.md` 3D rows; `VISUAL_SLICE.md` note; step-15 §13's 16f row marked done;
the operator's preview package (both packs, three worlds).
- [ ] impl · [ ] valid: `full-ci` on the head (AC completion) · [ ] review

---

## 10. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-1 | The café's glTF export loses look: shader materials, decals, batching | `material_rules.yaml` re-binds by material name; the parity bar (Q-16f-6) decides; d1 merges without switching, so a failed parity blocks only d2 |
| R-2 | Asset size: the café's textures copied into the pack | textures referenced from the client's library by slug where possible; glTF-Transform if over budget |
| R-3 | Perf: runtime glTF has no importer LODs (C-8) | `ImporterMesh.generate_lods()` on load; `SliceBatch` and `SliceBudget` still run on the loaded scene; AC-16f-14 |
| R-4 | S33-d revision 2 changes shape before freeze | 16f-c and 16f-d1 depend on its C3; if its format moves, 16f's additive `models` section moves with it, nothing else |
| R-5 | With art no longer blocking, a mismatch the probe missed lets the camera see through a gap | the probe now runs on every world and pack (§4.2); the boom still meets art |
| R-6 | Plain drawing (Q-16f-1 A) is mistaken for a finished world | the HUD note `hud.depiction.plain`, as S33-c's stub label |
| R-7 | Moving the café code breaks probes that read its constants (`SliceWorld.CAFE_PLACE`, `SliceCafe.DEPTH`) | d2-C2 moves those reads to the depiction's interface first; scan for every reference at d2-C0 |

---

## 11. Files touched (expected)

| Sub-PR | Files |
| --- | --- |
| 16f-a | `clients/3d-spike/scripts/pack/pack.gd` (new); `scripts/slice/{slice_main,slice_link,slice_world,identity_check,cast_binding,furnishings}.gd`; `mineworld-slice`; `presentation/mineworld-default/3D/i18n/` (moved keys); `clients/shared/settings/locale/*.po` (keys removed); `clients/shared/SETTINGS.md`; `presentation/mineworld-greybox/3D/` (new); `tests/acceptance/tests/client_rules.rs`; `clients/3d-spike/PRESENTATION.md`; `docs/decisions/ARC-51.md`; `docs/decisions/INDEX.md`; `docs/MODULE_SPEC.md` §6.1 |
| 16f-b | `scripts/places/{spans,shells}.gd` (new); `scripts/build.gd`; `scripts/player.gd`; `scripts/camera_rig.gd` (mask); `scripts/slice/{client_bodies,slice_layout,slice_probe_geometry}.gd`; `scripts/depictions/{box_depiction,registry}.gd`; `scripts/checks/span_check.gd` (new); `packages/src/` and `packages/tests/` (shells validator, corpus); `tools/cli/src/packs.rs`; `presentation/*/3D/places/shells.yaml`; `presentation/*/3D/far_field/bodies-yard.yaml`; `tests/fixtures/observations/`; `scripts/ci_layer.py`; `docs/decisions/ARC-52.md`, `ARC-146.md` note; `docs/CORE_CONCEPTS.md` §6.5; `PRESENTATION.md` §6 |
| 16f-c | `scripts/depictions/registry.gd`; `scripts/depictions/lakeside/` (deleted); `docs/decisions/ARC-144.md`, `ARC-146.md` notes |
| 16f-d1 | `tools/presentation/` (new, with `social_cafe/depictions/{social-cafe,market-town}.yaml`); `presentation/mineworld-default/3D/{assets/places/,materials/material_rules.yaml,LICENSES}`; the depiction validator and corpus (S33-d's files); `MODULE_SPEC.md` §6.1.2 (`models`) |
| 16f-d2 | `presentation/mineworld-default/3D/depictions/{social-cafe,market-town}.yaml` (moved in); `scripts/depictions/` (social-cafe, market-town deleted; the models loader in `data/`); `scripts/slice/` (café builders deleted; probes read the interface); `docs/MVP_STATUS.md`; `docs/VISUAL_SLICE.md`; `step-15-demo-3d.md` §13 |

---

## 12. Freeze questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-16f-1** | A world whose pack has its far field but no depiction: (A) draw it plainly from the server's disclosure (floor, walls, door openings, solids) with a "plain" HUD note; or (B) keep refusing it (`ARC-146`)? | **(A).** Every wall drawn is the server's own, so it is not the "looks like the world and is not it" fallback the operator declined. It is what lets bodies-yard, and any new world, open in 3D with no client code, as the 2D client already does |
| **Q-16f-2** | The server discloses only the room you stand in. (A) Widen `bodies` to also disclose the shapes of places one doorway away, so façades come from disclosure too; or (B) keep it, draw neighbours from art or from memory once visited, and leave widening to a later design? | **(B).** It needs no server or contract change, matches S33-d's Q-S33d-5 (A), and the probe catches any drift between art and the server. Widening is a bodies design of its own |
| **Q-16f-3** | The second pack that proves a swap changes the look: (A) a small shipped `mineworld-greybox` 3D pack (plain materials, grey shells, copied far fields); or (B) a generated test-only variant? | **(A).** A real pack a person can pass with `--presentation=` is the honest proof, and it doubles as the authoring baseline for new worlds |
| **Q-16f-4** | How the hand-built café joins the default pack: (A) export it to glTF assets with `asset.yaml`, placed by the pack's data file; (B) move its GDScript into the pack (Tier 2 code in a pack); or (C) re-author it in S33-d's data format? | **(A).** glTF is the canonical asset format and keeps packs code-free; (B) makes a downloaded pack run code; (C) is a large rewrite with certain visual change |
| **Q-16f-5** | Should only disclosed walls stop the player (art colliders on a layer the player ignores, ground excepted)? | **Yes.** It is the literal meaning of "art only decorates", and the probe guards the look |
| **Q-16f-6** | The café after export cannot be pixel-equal. Accept (A) SSIM ≥ 0.98 per shot on S33-c's set, perf within ±10 %, plus the operator's look; or (B) a stricter or looser bar? | **(A).** It allows export noise and still catches a missing wall or a lost material |
| **Q-16f-7** | Records: `ARC-51` (the pack the 3D client is given) and `ARC-52` (places from disclosure; art decorates), both free; the `models` section as an additive amendment to S33-d's proposed `ARC-177`? | **Yes.** The coordinator confirms `ARC-177` with #386 |
| **Q-16f-8** | Move the 3D client's own words into the default pack's `i18n/` in 16f-a, as `SETTINGS.md` already promises? | **Yes.** Keys and text unchanged |
| **Q-16f-9** | Order: 16f-c and 16f-d1 wait for S33-d revision 2's C3 (the data reader). Accept, rather than 16f writing its own reader? | **Yes.** One format and one reader |

---

## 13. Requirements this design places on other PRs

- **On S33-d (#386).** Keep `SliceDataDepiction`'s constructor free of a world key so the registry can build it
  from a file path alone; accept an additive optional `models` section in format 1 (Q-16f-7).
- **On S33-b.** None new; AC-16f-10 waits for its `body:` sections.
- **On S33-a (#387).** None; Lakeside's far field stays in `far_field/lakeside.yaml`, and the greybox pack copies
  it when it lands.
- **On S33-j.** The export splits by world as planned; after 16f-d2 the café's assets are pack files, not
  client resources, so they ship with the pack.

---

## 14. Execution contract (empty until the freeze)

```text
Frozen by / date:
Sub-PRs authorized:
Implementation base per sub-PR:
Autonomy granted (commit, push, open PR):
Validation budget (Godot probes, slots):
Stop conditions:            any change to a frozen invariant (§6), a public contract, an ownership boundary,
                            a material dependency, or scope
Merge authority:            the coordinator
```

---

## 15. Ledger (live during implementation)

### Commit ledger

| Sub-PR | Commit | SHA | impl | valid | review |
| --- | --- | --- | --- | --- | --- |
| 16f-a | a-C0 base and ledger (base `origin/main @ fec906f2`; A-1…A-14 re-checked by `grep`; readers: `slice_main.gd:110,162`, `slice_link.gd:148-151`, `slice_world.gd:99-100`, `identity_check.gd:17`) | `f4ca9f25` | [x] | [x] grep | [x] |
| 16f-a | a-C1 + a-C2 `SlicePack`, the flag, the refusal, every reader | `e2b967a5`, `6783383e` | [x] | [x] AC-16f-2 four runs; `--world --link`, `--world=market-town --link` PASS | [x] |
| 16f-a | a-C3 the 3D words in the pack | `df6baf26` | [x] | [x] `client_text` PASS; AC-16f-4 | [x] |
| 16f-a | a-C4 the greybox pack | `a5090f77` | [x] | [x] `packs validate` both packs; AC-16f-3 | [x] |
| 16f-a | a-C5 the scan and close-out (`client_rules.rs`, `PRESENTATION.md` §0, `MODULE_SPEC.md` §6.1, `ARC-51`) | `d30c4aa3` | [x] | [x] planted literal red, removed green | [x] |
| 16f-a | fix: the depiction's HUD note re-worded on a language change (F-a-4) | `db33d3c1` | [x] | [x] `--world=lakeside --settings` PASS, both packs | [x] |
| 16f-b | b-C0 base: stacked on 16f-a's head `49919672` (PR #425, not merged: merge freeze on `main`); `main` at `5f9d8b0b` when 16f-b was pushed. Base captures not taken (the machine was reserved: no local Godot or cargo, lane lead's instruction) | — | [x] | N/A: no local run allowed | [x] |
| 16f-b | b-C1 one span function (`scripts/places/spans.gd`, `span_check.gd`, `tools/cli/tests/client_3d_spans.rs`, in `ci_godot.py`'s Godot tests) | `5f69f3bb` | [x] | [ ] INCONCLUSIVE: `span_check` not run (no Godot); `rustfmt --check` PASS | [x] |
| 16f-b | b-C2 `LAYER_ART`; floors stay `LAYER_WORLD`; the player's mask drops art where a place discloses `place-shape` (F-b-1); boom, ray and probes mask art | `c454db00` | [x] | [ ] INCONCLUSIVE: AC-16f-5 not run; `check_client_rules.py` PASS | [x] |
| 16f-b | b-C3 `places/shells.yaml`: `packages/src/shells.rs`, `packs validate`, corpus `packages/tests/fixtures/shells/`, both packs' files (F-b-2) | `df544201` | [x] | [ ] INCONCLUSIVE locally (no cargo); `fast` on the PR head compiles and lints it; the corpus test runs at the milestone | [x] |
| 16f-b | b-C4 `SliceShells`; the layout glues a miss as a SHELL; thresholds; current-place hysteresis (F-b-3); the cache dropped on a new instance | `956c6b33` | [x] | [ ] INCONCLUSIVE: AC-16f-8 not run | [x] |
| 16f-b | b-C5 `SlicePlainDepiction`; registry; `hud.depiction.plain`; `far_field/bodies-yard.yaml` in both packs; the plain world's passage walk (`_passages_check`); CI rows; `client_far_field.rs` refusal rows (F-b-4) | `f03c2e95` | [x] | [ ] INCONCLUSIVE: AC-16f-6 not run | [x] |
| 16f-b | b-C6 `ARC-52`, the `ARC-146` note, `PRESENTATION.md` §6, `CORE_CONCEPTS.md` §6.5, `MODULE_SPEC.md` §6.1, greybox README | `5b4157e1` | [x] | [x] `check_decision_ids.py --write-index`, `check_doc_headings.py` PASS | [x] |
| 16f-c | c-C0 base: branch `mvp0/pr-16f-c` from 16f-b's head `9fdaa791` (PR #451, stacked on #425), merged with `origin/mvp0/pr-s33d` (S33-d rev2, PR #452, head `7522ead1`), whose C3 holds `SliceDataDepiction` and `SliceDepictionFile`; neither is on `main` yet (F-c-1). Conflicts resolved keeping both sides (F-c-3) | `3cdd4e56` | [x] | [x] Godot `--import` no parse error | [x] |
| 16f-c | c-C1 the lookup order (`registry.gd`): `--slice-depiction-file=`, the pack's `depictions/<key>.yaml`, the script, `3d-<key>.pck`, plain; the start line names the source; `SliceDataDepiction`'s fallback path from `SlicePack` | `775d6282` | [x] | [x] `--world --link` still drawn by `res://scripts/depictions/social-cafe/depiction.gd`, PASS | [x] |
| 16f-c | c-C2 `scripts/depictions/lakeside/` deleted | `dd814b8e` | [x] | [x] AC-16f-9 (pack half) PASS, AC-16f-10 PASS; variant half INCONCLUSIVE (F-c-1) | [x] |
| 16f-c | c-C3 `ARC-144` and `ARC-146` notes; `PRESENTATION.md` §7's lookup text; this ledger | (this commit) | [x] | [x] `check_decision_ids.py --write-index`, `check_doc_headings.py`, `check_client_rules.py` PASS | [x] |

### Evidence

16f-a, local, macOS (Apple M5), Godot 4.7.2, through `with-slot.sh`; logs under `/tmp/16fa-ev/` (not kept).

| AC | Run | Result |
| --- | --- | --- |
| AC-16f-1 | `ci_godot.py slice --world --link` default / `--presentation=presentation/mineworld-greybox/3D`; the same on `--world=market-town`; `client_rules` scan | **PASS**: four times `all link checks pass`; scan green |
| AC-16f-2 | `./mineworld-slice --drive --presentation=<dir>` for a dir with no `pack.yaml`, `type: world-pack`, `dimension: [2d]`, and a missing dir | **PASS**: exit 3 each; `PRESENTATION_PACK_INVALID: { pack: …, why: no pack.yaml }` / `pack.yaml says type: world-pack, not presentation-pack` / `manifest.yaml lists dimension ["2d"], not 3d` / `no such directory` |
| AC-16f-3 | `--world --cast` and `--world=market-town --cast`, each pack; the `FURNISHED` line of each `--link` run | **PASS**: greybox draws every person with `character ""`, body `vitruvian.glb` (the plain figure), every furnishing a plain box; `FURNISHED` 16/16 `unbound: true` (default: 16/16 `false`); `cast_room` frames compared by eye; nothing outside `presentation/` differs between the runs (same head) |
| AC-16f-4 | `--world=lakeside --settings` with the default pack, and with a copy of greybox whose `i18n/en.po` overrides `hud.depiction.stub` | **PASS** (after F-a-4): `all settings checks pass` both; the override's text `OVERRIDDEN BY THE PACK: Lakeside stub` is the HUD label's text in the EVIDENCE lines |
| static | `cargo test -p mineworld-acceptance --test client_text --test client_rules` | **PASS** (6 + 6) |
| packs | `mineworld packs validate` on both 3D packs | **PASS**: each `is a valid presentation-pack` |

The design's `--world=market-town --buy` names no launcher mode; the market probe (`--market --steps=`) is driven by
`tools/cli/tests/client_3d_market.rs`, which generates its steps. Market Town was run with `--link` and `--cast`
instead (F-a-1). bodies-yard is not drawable before 16f-b (G-5), so 16f-a's second world is market-town, as §5 says.

16f-b, 2026-10-20. **The machine was reserved** (lane lead's instruction): no local `cargo build`, `cargo test` or
Godot run, through `with-slot.sh` or otherwise. Only cheap static checks ran locally; GitHub CI's `fast` compiles,
lints and formats the Rust. `fast` runs no Godot (ARC-154: tests deferred to the milestone), so every Godot
criterion below is INCONCLUSIVE until the milestone run.

| AC | Run | Result |
| --- | --- | --- |
| AC-16f-5 | `--world --bodies`, `--world --geometry`, and the planted mutation (the café's colliding side walls in `cafe.gd` moved 0.5 m outward) | **INCONCLUSIVE**: not run |
| AC-16f-6 | `ci_godot.py slice --world=bodies-yard --link` (the plain world's passage walk: hall → court → hall) | **INCONCLUSIVE**: not run; CI row added to `clients-probes` |
| AC-16f-7 | `--world --link`, `--geometry`, `--bodies` with `--presentation=presentation/mineworld-greybox/3D` | **INCONCLUSIVE**: not run; scope narrowed by F-b-5 |
| AC-16f-8 | `--world --layout`; a place no entry draws named `shell` | **INCONCLUSIVE**: not run; see F-b-6 |
| static | `check_client_rules.py`, `check_decision_ids.py --write-index`, `check_doc_headings.py`, `check_scratch.py scan`, `check_secrets.py scan`, `ci_changes.py --self-test`, `ci_godot.py --self-test`; `rustfmt --edition 2024 --check` on every changed Rust file | **PASS** |
| `fast` | GitHub CI on the PR head | see the PR |

16f-c, 2026-10-21, local, macOS, Godot 4.7.2, through `with-slot.sh` (local runs allowed again); logs under
`/tmp/16fc-ev/` (not kept).

| AC | Run | Result |
| --- | --- | --- |
| AC-16f-9 (pack) | `ci_godot.py slice --world=lakeside --link` (default pack; no Lakeside script) | **PASS**: `[slice] world lakeside, drawn by SliceDataDepiction from …/presentation/mineworld-default/3D/depictions/lakeside.yaml`; `all link checks pass` (112 s) |
| AC-16f-9 (variant) | `lakeside-v7` with `--depiction-file=` | **INCONCLUSIVE**: the variant (`packages/tests/fixtures/depiction/variants/lakeside-v7/`) is S33-d's C12 and not on #452's head yet (F-c-1). The flag path itself ran: a copy of `lakeside.yaml` at `/tmp/16fc-ev/var/lakeside.yaml` with the greybox pack printed `drawn by SliceDataDepiction from /tmp/16fc-ev/var/lakeside.yaml`; that run's link walk FAILed only `nobody was drawn as a cast character` (greybox has no cast; F-c-2) |
| AC-16f-10 | `--world=lakeside --link --presentation=presentation/mineworld-greybox/3D` | **PASS**: `drawn by the world's disclosure (no depiction of it in this pack or client; drawn plainly)`; `all link checks pass` |
| refusal (§5 row) | `--world=lakeside --link --depiction-file=/tmp/16fc-ev/other.yaml` (Lakeside's file under another stem) | **PASS**: `[mineworld-slice] DEPICTION_SCOPE_MISMATCH: { world: lakeside }: …: scope.world lakeside belongs in lakeside.yaml, not other.yaml`, exit 3 |
| c-C1 | `--world --link` (Social Café still has its script) | **PASS**: `drawn by res://scripts/depictions/social-cafe/depiction.gd`; `all link checks pass` |
| static | `check_client_rules.py` (no `PACK_DIR` literal), `check_decision_ids.py --write-index`, `check_doc_headings.py` | **PASS** |
| `fast` | GitHub CI on the PR head | see the PR |

### Mutations

| Planted | Expected | Seen |
| --- | --- | --- |
| `const PACK_DIR := "../../presentation/mineworld-default/3D"` appended to `scripts/controls_hud.gd` | `client_rules` red, naming the file | exit 101; `controls_hud.gd:267: "../../presentation/mineworld-default/3D" names a Presentation Pack's path` and `` `PACK_DIR` `` findings; green again once removed |
| 16f-b: the café's west art wall moved 0.5 m (AC-16f-5's adversarial criterion) | the player still stops at the disclosed wall; `--geometry` FAILs naming the face | **not run** (machine reserved); left for the milestone |

### Deviations and findings

- **F-a-1 (bounded).** `--buy` does not exist (see Evidence); market-town ran `--link` and `--cast`.
- **F-a-2 (bounded).** The accessor is `SlicePack.select()` (resolves once, then returns `SlicePack.current`), so
  scripts that run outside the slice scene (`identity_check.gd`, `controls_hud.gd`) get the same pack; §4.1's
  `SlicePack.current.path(…)` holds once selected. a-C1 and a-C2 are one commit, because `slice_main.gd` holds both.
- **F-a-3 (bounded).** The greybox pack also carries `i18n/`, a copy of the default pack's words: a pack is read alone
  (PK-3 is after MVP-0), and without it every 3D key would read as its readable form. Recorded in `ARC-51`'s
  limitation. The launcher needed no change (it passes unknown flags through); only its help gained the flag.
- **F-a-4 (bounded, fixed).** The depiction's HUD note (`hud.depiction.stub`) was set once and not re-worded on a
  language change, which failed `--world=lakeside --settings` on main too; fixed in `db33d3c1`.
- **F-a-5 (observation).** A missing optional file (`characters/cast.yaml`, `furniture/furnishings.yaml`) still
  prints the loaders' existing `cannot read` warning; it is not a refusal. Left as is.
- `ARC-52` is 16f-b's (b-C6) and is not written here.

16f-b:

- **F-b-1 (bounded).** §4.2's last point keeps art on `LAYER_WORLD` where no `place-shape` is disclosed. The layer
  cannot be chosen when art is built (disclosure arrives later, per place), so art is always on `LAYER_ART` and the
  *player's mask* includes `LAYER_ART` until the current place discloses its walls (`Player.set_art_blocks`, set by
  the link after `SliceClientBodies` rebuilds). Same behaviour; recorded in `ARC-52`. Floors, ramps and terrain are
  told from art by the call site (`Build.box(…, layer)`; the café's and the florist's floors, the carriageway and
  the pavements pass `LAYER_WORLD`); the street's parapets, plinth and banks are art.
- **F-b-2 (bounded, additive to §4.3's format).** A material slug in `places/shells.yaml` is declared in the file's
  own `materials` table (colour, roughness, optional client-library `texture` and `metres`), so the pack is
  self-contained and `SHELLS_REFERENCE` is checkable by `packs validate` without the client's library. A texture
  slug the client lacks draws the colour alone. The default `opening.width_m` is 2.0 m, the disclosed colliders'
  gap, so nothing drawn is passable and nothing passable is drawn shut.
- **F-b-3 (bounded).** Glued frames overlap at a doorway by the two doorway insets (0.8 m; `ARC-45`), so the
  smallest-volume rule would report the body in the next place while it still stands inside this one. A shell's
  volume never takes a point from the current place's volume (`SliceShells.keep`); the place changes when the body
  leaves the current floor, which only a doorway's gap allows. A shell neighbour not yet stood in has no disclosed
  shape (C-1), so it gets a *threshold* volume just past the gap (2.0 m by 1.0 m), and walking through it reports
  the body there.
- **F-b-4 (bounded).** §4.3 names `FAR_FIELD_WORLD_UNKNOWN` for a world whose pack lacks its far field; the loader's
  code for that case is `FAR_FIELD_MISSING` (`FAR_FIELD_WORLD_UNKNOWN` is the no-key case). `client_far_field.rs`'s
  refusal test now expects `FAR_FIELD_MISSING` for `--world-key=nowhere` and keeps
  `PRESENTATION_WORLD_MISSING` for a key that is a path (`no/where`).
- **F-b-5 (design inconsistency, resolved by §4.3's rule; operator to note).** AC-16f-7 says that with greybox
  social-cafe's café, street and store are drawn as shells. In 16f-b the café street is still drawn by the client's
  `scripts/depictions/social-cafe/` script whatever the pack, and §4.3 says "a place a depiction draws keeps its art
  and gets no shell". So in 16f-b greybox changes people and furniture (16f-a) and shells (bodies-yard, and any
  place no entry draws); the café becoming greybox shells needs the café's art in the default pack and not in the
  client, which is 16f-d2 (AC-16f-15 already states it there). Not a change of scope.
- **F-b-6 (bounded).** AC-16f-8's example, social-cafe's station hall, is bound by the social-cafe depiction as a
  closed DOOR entry (the halt's door) and its forecourt lies outside the street's art volume (x −34…34 m), so it is
  not "a place no depiction entry draws". The mechanism is exercised generically on bodies-yard (every place a
  shell, glued at its passage). Places bound as DOOR stay doors in 16f-b.
- **F-b-7 (deferred).** §4.3's offline recorded observation for bodies-yard (a drift-checked fixture beside
  `tests/fixtures/observations/furnishings/`) is not added: offline, a plain world draws its far field and ground
  only. Recorded in `ARC-52`'s limitation.
- **F-b-8 (deferred).** §4.2's "the probe now runs in every place the player can stand in, on every world" is not
  done: `--geometry` still walks Social Café's café, street and store, now with `LAYER_WORLD | LAYER_ART` and also
  with greybox (CI row). A shell draws no collider but its floor, so the probe has nothing to hold against
  disclosure on a plain world. Left for the milestone.
- **CI (PR #425).** After merging main (`b692e732`; #394 landed the design, so this file keeps main's header plus
  this ledger), `fast` failed on `cargo fmt` and then on clippy `manual_contains` in `client_rules.rs`. Fixed in
  `6610bd02` and `bfe14ee2`; `fast` passed on `bfe14ee2`. Two untracked `.gd.uid` files that the Godot import
  generated for scripts this PR does not touch (`furnishings.gd`, `inspect_panel.gd`) were discarded.

16f-c:

- **F-c-1 (bounded; base).** c-C0 asks for S33-d's C3 on `main`. It is not merged; 16f-c is stacked on #451 and
  #452 instead, by the lane lead's instruction, so it merges after both. `lakeside-v7` (S33-d C12) does not exist
  on #452's head, so AC-16f-9's variant half waits for it; the `--depiction-file=` path it needs is exercised.
- **F-c-2 (observation, not 16f-c's).** The link probe's box/data-depiction walk fails `nobody was drawn as a cast
  character` whenever the pack has no cast (greybox), whatever the depiction. A pack-swap run of a data-drawn
  world with greybox therefore fails that one row. The plain-drawn run (AC-16f-10) does not walk that route.
  Left for the milestone: the row should skip when the pack binds no cast (as 16f-a's `FURNISHED` reads `unbound`).
- **F-c-3 (bounded; merge).** 16f-b's `Build.LAYER_ART` and S33-d's `Build.LAYER_CAMERA` both took bit 16.
  `LAYER_ART` keeps 16 (it is in the frozen design and `ARC-52`); `LAYER_CAMERA` moves to 32, and `CAMERA_MASK`
  also meets `LAYER_ART`, so the boom keeps both PRs' behaviour. `PRESENTATION.md`: shells stay §6, depictions
  become §7 (`MODULE_SPEC.md` and `ARC-177` re-pointed). The merge left `SliceDataDepiction` reading
  `SliceMain.PACK_DIR`, which 16f-a removed; c-C1 reads `SlicePack` instead, so the merge commit alone does not
  parse that file at run time.
- **F-c-4 (scope note).** The lane lead's brief mentions the `models` section (Q-16f-7). §5 and §9 give it to
  16f-d1 ("the `models` section and its validation"); §4.4 says 16f-c changes nothing in S33-d's format. 16f-c
  does not add it.
- **F-c-5 (bounded).** Lookup step 0: an explicit `--slice-depiction-file=` is honoured before the pack's file and
  before any script, so a variant draws "with no GDScript change" (AC-16f-9) even on a world whose pack has no
  file. §4.4 lists the flag only inside S33-d's reader; making it the registry's first step is the same rule.

### Design review
