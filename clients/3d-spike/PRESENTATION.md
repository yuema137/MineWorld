# What a 3D Presentation Pack contains for the 3D reference client

**Audience:** coding agents writing or changing a 3D Presentation Pack, or the 3D client's code that
reads one. A specification, not an introduction; [`README.md`](README.md) is the orientation.
**Authority:** [`docs/DECISIONS.md`](../../docs/DECISIONS.md) `ARC-92` (the cast binding), `ARC-91` (the
`look` a world discloses), `DEP-16` (the file format), `ARC-3` (packs share meaning, not art).
[`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §6.1 names the file; this document specifies its
contents for this client. Where this document and the client disagree, that is a defect to be recorded
and fixed, never left.

**Status (2026-10-18).** Specified by S26-a. The slice reads `characters/cast.yaml` through
`scripts/slice/cast_binding.gd` (`SliceCast`) for every figure and for the player (PR VF-PID,
`.structured-coding/plans/mvp0/pr-vf-person-identity.md`). Since PR S26-c1 (2026-10-19,
`pr-s26c1-cast-client.md`) the default pack binds twenty-two looks: `vera-lindgren` (the reference body),
the eleven café-town characters (Wes Calloway included) and the ten lakeside characters, each drawn from
its manifest (§2.1). Pending operator question Q-PID-2: an unbound look with no extras is drawn as the
client's town figure, not as `default` (§3).

---

## 0. Selecting a pack

The client is given one 3D Presentation Pack per run and reads it from disk (`docs/decisions/ARC-51.md`),
as the 2D client does ([`clients/2d/PRESENTATION.md`](../2d/PRESENTATION.md) §1):

    ./mineworld-slice --world                                                   the default pack
    ./mineworld-slice --world --presentation=presentation/mineworld-greybox/3D  another pack

- **`--presentation=<dir>`** names the pack's directory, absolute or relative to the repository root. A
  path outside the repository is allowed (a user's own pack). Absent, the pack is
  `presentation/mineworld-default/3D`. There is no `--presentation=none`: every view needs a far field
  (`ARC-102`).
- **`SlicePack`** (`scripts/pack/pack.gd`) resolves the directory once, before anything is drawn or
  worded, and is the only place a pack path is built. Every reader (the far field, §2's cast binding,
  §5's furnishings, the identity check, the wording) asks `SlicePack.current.path(<relative path>)`;
  `tests/acceptance/tests/client_rules.rs` refuses a pack path written anywhere else.
- **A directory that is not a 3D pack is refused by name.** It does not exist, it has no `pack.yaml`,
  its `type` is not `presentation-pack`, or its `manifest.yaml` lists no `3d` in `dimension`. The client
  prints and shows `[mineworld-slice] PRESENTATION_PACK_INVALID: { pack: <dir>, why: <text> }`, and a
  scripted or harness run exits 3. Only these identity fields are read; `mineworld packs validate`
  checks a pack in full.
- **A missing optional file is not refused.** Without `characters/cast.yaml` every person is the
  client's plain figure (§3); without `furniture/furnishings.yaml` every furnishing is a plain box over
  its solids (§5).
- **Words.** The pack's `i18n/` is the middle layer of the client's wording
  ([`clients/shared/SETTINGS.md`](../shared/SETTINGS.md) §6.2). The default pack holds the 3D client's
  own keys (`hint.*`, `camera.*`, `hud.*`, `link.*`, the 3D menus' lines); a pack overrides a shared key
  with an entry marked `#. override`.

Two packs ship: `presentation/mineworld-default/3D` and `presentation/mineworld-greybox/3D`, which has no
cast, no furnishing bindings and the default pack's far fields and words copied unchanged. Drawn with
it, the same world shows plain figures and plain furniture, and the server's answers are the same.

## 1. What the client reads from a 3D pack

Only the cast binding, as data. A character's mesh and texture are resources the client's Godot project
imports, under `clients/3d-spike/assets/characters/cast/<id>/`, because the importer applies the
humanoid bone map at import time and the client cannot load a skinned, retargeted model from a runtime
pack today. Each kind of file has one location; nothing is copied between them.

Also read as data: the far field (`far_field/<world-key>.yaml`, `docs/MODULE_SPEC.md` §6.1.1), the
furnishing bindings (§5) and, for a world drawn from data, its depiction file
(`depictions/<world-key>.yaml`, §7; `ARC-177`) and the place assets it places (`assets/places/`, §8; loaded
from 16f-d2), and a world's sit-point depictions (`sit_points/<world-key>.yaml`, §10; `ARC-145`).

## 2. `characters/cast.yaml`

YAML's JSON-compatible subset: one JSON object, no comments (`DEP-16`). Godot's built-in `JSON` parser
reads it.

    {
      "format": 1,
      "looks":   { "<look key>": "<character id>" },
      "extras":  [ "<character id>" ],
      "casts":   { "<world key>": [ "<look key>" ] },
      "default": "<character id>"
    }

- **`format`** — the integer `1`.
- **`looks`** — a disclosed `look` (`ARC-91`) to the character that draws that person. Several looks map
  to one character only where the operator has accepted it; the default pack maps none twice.
- **`extras`** — the characters that draw a person with no look, a person whose look this pack does not
  bind, and decorative figures. Distinct ids. Every entry passes the same standard as a named person
  (step-26 §6; `ARC-94` once S26-b1 records it); there is no cruder class.
- **`casts`** (optional; S33-g, `scripts/check_cast.py` CC-12) — for each world key, the looks that
  world's own people wear. Every entry is a key of `looks`, and no look is in two worlds' casts. A
  world's depiction may take its extras from another world's cast
  (`SliceDepiction.extras_from()`, step-26 E-2): those people's characters then replace `extras` for
  that world, never including one of the world's own people nor the `default` character (the
  protagonist is nobody's extra). A world variant that reuses another world's people (Market Town)
  lists no cast of its own. Decor figures a depiction states (`SliceDepiction.decor()`, at most five)
  are drawn from the same extras, only as cast bodies, never as the plain figure.
- **`default`** — the character used when no world is connected: the standalone slice and portrait
  captures.
- A look key and a character id are each 1–48 bytes of `a`–`z`, `0`–`9` and `-`. A character id names
  the directory `characters/cast/<id>/` in this pack (its `character.yaml`, provenance, brief and
  approved references; `ARC-93` once S26-c1 records it) and the client resource directory of §1.
- A key the client does not know is ignored with a warning, so a newer pack degrades rather than fails.
  The default pack is held to no unknown key by `scripts/check_cast.py`.

## 2.1 `characters/cast/<id>/character.yaml`

The per-character manifest (step-26 §4.5, `ARC-93`), one JSON object, written beside the character's
brief and provenance in the pack. The client builds the character's body from it alone
(`Human.build_character`); there is no per-person code.

    {
      "format": 1,
      "id": "alice-moreau",
      "scene": "res://assets/characters/cast/alice-moreau/alice-moreau.gltf",
      "material": "MW_Cast",
      "stature_m": 1.61,
      "authored_height_m": 1.61,
      "clips": { "walk_mps": 1.016, "jog_mps": 2.573, "measured_by": "…" },
      "posture": "natural",
      "grip": null,
      "gaze": null,
      "surface": { "roughness": 0.9, "specular": 0.25 }
    }

- **`id`** equals the directory name. **`scene`** is the imported glTF in this client (§1).
- **`material`** names the body's one material; it keeps its baked base colour and takes **`surface`**'s
  roughness and specular level (metallic 0). Any other surface is reported.
- **`stature_m`** = **`authored_height_m`**: the body is authored at the person's true stature and drawn
  at instance scale 1 (`SD-S26-3`); a requested height does not stretch it.
- **`clips`** are the ground speeds the walk and jog clips cover on this body at scale 1, measured by
  `tools/measure_stride.gd -- --character <id>`; they place the clips in the locomotion blend so the feet
  do not skate.
- **`posture`** `"natural"` is `Posture.natural_stance()`; **`grip`** and **`gaze`** must be null (no cast
  member holds a strap; the reference body's grip and gaze stay in `human.gd` until her constants move
  into her own manifest).
- **`stand_gates`** (PR S26-c-pose) is this body's measured standing pose (`tools/character_pipeline/PIPELINE.md`
  §15.2): the worst palm, supination, reach, clearance and arm joint angles over every frame of the `Stand` loop,
  the misses and the verdict. A record, not an input: every body plays the same recorded idle
  (`assets/characters/stand_lib.glb`, `DEP-146`). There is no per-body pose data.
- **`ground`** (PR S26-c-ground) is this body's measured ground contact (PIPELINE.md §15.1): the lowest
  skinned vertex against the floor at rest, over the `Stand` loop and every frame of one `Walk` and one `Jog`
  cycle, and the arms' worst clearance over the `Stand` loop. It is a record, not an input: the client stands every body on its
  own soles from its geometry (`ground_contact.gd`). The reference body's is in its index entry.
- A missing or malformed field draws the person as the town figure with one warning naming the field. So
  does a body whose `.bin` or `_base_color.png` is a Git LFS pointer file (`version https://git-lfs…`, a
  checkout without LFS, `DEP-141`), or one that was never imported.
- `scripts/check_cast.py` holds the default pack to this shape (CC-7, CC-8), every body (the reference body
  through its index entry) to a passing ground measurement (CC-9), every arm to its rest and standing depth
  inside the body core within the tolerance file's bounds (CC-10, G-9w's recorded `ground.arms`), and every
  gated arm to the standing bounds (CC-11 supination and joint ranges; palm and reach recorded).

The reference body (`vera-lindgren`) is not described by a manifest yet: the client's resource index
`assets/characters/cast/index.json` names it `"body": "reference"`, and that entry wins. Its entry also carries
her measured `ground` and `stand_gates`, and `grip` (the side her strap grip holds: `human.gd`'s additive layer, not
the clip, so that arm is not gated).


## 2.2 How a body stands on the ground (the grounding chain)

Every body the client draws, whichever pack supplied it, goes through the same chain after its animation. Each
link is a `SkeletonModifier3D` on the body's skeleton, in this order:

1. **The animation** (`AnimationTree`): the retargeted clip, at the gait `Human.set_gait` chose.
2. **`Posture`** (`posture.gd`): the standing correction (arms, thighs, the reference body's strap grip).
3. **`GroundContact`** (`ground_contact.gd`, PR S26-c-ground): skins the body's own support points (taken once
   per scene from its feet) and moves the `Hips` vertically, so the lowest point is never under the ground and,
   standing or walking, is on it. Since PR S36-P4 it judges each foot against its own ground and settles the
   hips on the **lower** of the two (`GroundContact.lift_for`); with both grounds at the floor node's `y = 0` it
   is the S26-c-ground rule exactly.
4. **`FootGrounding`** (`foot_grounding.gd`, PR S36-P4, `DEP-144`): per foot, a ray against `Build.LAYER_WORLD`
   (the drawn floors; never the server's disclosed solids, never another body) finds the ground under the foot.
   One `TwoBoneIK3D` per leg (`UpperLeg` → `LowerLeg` → `Foot`) puts a **planted** foot on the ground under it:
   raised onto higher ground, or lowered (by at most `max_planted_lower_m`) where the clip leaves it floating. A
   `FootTilt` modifier turns the foot by the rotation from the vertical to the ground's normal (capped). The
   pipeline's G-4w gate still judges the asset, because it measures bodies with grounding off
   (`Human.foot_grounding = false` in the pose-sheet probe).
5. **`LimitAngularVelocityModifier3D`**: holds the leg bones' turn rate, so a correction never pops.

Every tunable of step 4 is in `assets/characters/foot_grounding.json` with its reason; the bounds its probe
(`tools/feet_course.gd`, judged by `cast.py feet`) is held to are in
`tools/character_pipeline/contact_tolerances.json`. Seated (`Human.sit`), steps 3–5 stand down: the seat is the
contact. Beyond `ik_distance_m` from the camera, steps 4–5 stand down and step 3 keeps working.

The ground the rays meet is what is **drawn**: every drawn step (a kerbstone, a door step, a threshold) has its
own collider on `LAYER_WORLD` (`Build.step`, PR S36-P4b). The slopes that let the player's capsule walk up a step
instead of catching on its edge are walk aids on `Build.LAYER_WALK_AID`, which only the player's body masks; no
foot ray, camera boom or targeting ray meets them.

What is presentation and what is not: this whole chain changes only how a body is **drawn**. It never moves the
body's root (where the server put the person, `DEP-20`), sends nothing and writes no authoritative state
(I-S36-6). A pack cannot tune it per body: the support points are measured on each body at runtime and the
tunables are the client's.

## 3. Resolution

    person with a disclosed look this pack binds       → that character
    person with no look, or one this pack does not bind → extras[ posmod(hash(id string), len(extras)) ]
    this pack binds no extras                           → default
    no pack                                             → the client's plain figure

The observer is resolved the same way: a player is drawn as the Person they occupy (`INV-1`). The hash
is of the entity's id string, the same stable hash the 2D client uses for `person:<n>`
([`clients/2d/PRESENTATION.md`](../2d/PRESENTATION.md) §3.1).

## 4. What a pack may not do

It may not decide whether anything is valid, reachable or allowed; it changes how a person looks, never
which requests the client sends; and a look key is never shown as text — it chooses a character, it is
not a name.

## 5. `furniture/furnishings.yaml`

How the client draws a world's **furnishings** (`ARC-307`, `docs/CORE_CONCEPTS.md` §6.5). The world says
what stands where and what it is (`table.round` at (4 860, 3 160) mm, facing 171 406 millidegrees); this
file says which model of the client's library draws that kind. YAML's JSON-compatible subset (`DEP-16`),
read by `scripts/slice/furnishings.gd`, the one script that draws furnishings.

```json
{
  "format": 1,
  "kinds": {
    "table.round": { "model": "round_wooden_table_02", "front_md": 180000, "tint": [1.04, 0.96, 0.86],
                     "dressing": [[ { "model": "tea_set_01", "parts": ["_cup_small_01"],
                                      "at": [-0.04, -0.03], "up": 0.764, "yaw_md": 64000 } ]] },
    "chair": { "model": "dining_chair_02", "front_md": 180000 }
  }
}
```

| Field | Meaning |
| --- | --- |
| `kinds.<kind>` | the binding of one kind. A furnishing's kind is looked up as written, then by each shorter `.`-prefix (`chair.painted` → `chair`) |
| `model` | a prop slug under `assets/models/`, whose S36-P1 mount in `assets/models/contact.json` is `floor` |
| `front_md` | the model's Godot yaw, in millidegrees, that turns it to face north. The drawn yaw is `front_md − facing`, in radians about +Y. Poly Haven models face +Z, so `180000` |
| `tint`, `parts` | as `SliceProps.put` takes them: a colour multiplier, and the members of a variant set to keep |
| `dressing` | hand-scale props that follow the piece (cups on a table): a list of sets, each a list of props. One set is chosen by FNV-1a 32 of `<place id>/<furnishing key>`, so a piece keeps its cups and a moved table carries them. Each prop: `at` = `[x, z]` metres in the model's own frame, `up` = metres above the floor, `yaw_md` relative to the model; its mount is `surface`. Dressing is never a solid (`ARC-306` 1) |
| `repeat` | `true`: the model is copied along the longer side of the piece's own solids, evenly, as many times as its drawn length fits (rounded): a 3.1 m bench of a 1.5 m model is two benches end to end (S33-f) |
| `form` | instead of `model`, for a piece the library has no model of (S33-f): a generic form built over the piece's own solids in `material` (a texture slug) and `tint` — `table`: a top `top_m` thick at the solid's height on four legs `leg_m` square; `bench`: the same, a backless seat a sitter may face either way; `hull`: a boat's hull tapered to a bow on the side it faces, on a cradle `cradle_m` tall. Dressing follows a form as it follows a model |
| `piece` | instead of `model`: a station piece (`StationPieces.furnishing`, §9.1) built to the furnishing's disclosed solids -- their footprint, in the piece's own frame, and their height. `bench_row` draws a 6 m bench row on its 6 × 0.6 m solid and a 1.9 m bench on its own; `kiosk` draws a news kiosk that shows open or shut by `MineWorldStationState`. The piece's front is −Z, so `front_md` is `0` |
| `style` | with `piece`: the materials, `oak` (the depot's) or `timber` (a halt's) |

The model is centred on `at`: its yawed bounding box's centre stands on the furnishing's point, on the
floor. A disclosed furnishing whose kind no binding resolves is drawn as a plain box over each of its
solids, in neutral timber, and reported `unbound` in the `FURNISHED` line; it is never left out. The
client builds no collider for a furnishing: `client_bodies.gd` already builds one from every disclosed
solid. `tools/cli/tests/furnishings.rs` checks that every kind every shipped world names resolves here,
and that each model's mount is right.

`slice_main --furnishings-dump=<dir>` writes, for each furnished place drawn, `<dir>/<place>/furnishings.json`:
each piece's kind, binding, glTF, the 4×4 (row-major) from the model's own metres to the place's frame
(x east, y north, z up), the indices of the disclosed solids that name it, its dressing's placed points,
and the shell solids. It is S36-P5's input for its mesh-against-solid check.

## 6. `places/shells.yaml`

How the client draws a place **plainly from its disclosure** (`docs/decisions/ARC-52.md`). A place the
server discloses and no depiction entry draws is glued into the scene at its passage and drawn by
`scripts/places/shells.gd` (`SliceShells`) from its own `place-shape`: a floor slab, four walls outside
the floor with an opening and a lintel at each doorway, and a box per shell solid (a solid that names no
furnishing; furnishings are §5's). A world with no depiction at all is drawn this way entirely
(`SlicePlainDepiction`, the HUD note `hud.depiction.plain`). The walls' positions are the server's; this
optional file says only how they look. YAML's JSON-compatible subset (`DEP-16`).

```json
{
  "format": 1,
  "materials": {
    "plaster_warm": { "colour": [0.86, 0.82, 0.74], "roughness": 0.9, "texture": "painted_plaster_wall", "metres": 2.0 },
    "setts": { "colour": [0.85, 0.83, 0.80], "roughness": 0.9, "texture": "cobblestone_floor_08", "metres": 2.0 }
  },
  "default": {
    "wall": { "material": "plaster_warm", "height_m": 2.7 },
    "floor": { "material": "plaster_warm" },
    "opening": { "width_m": 2.0, "height_m": 2.1 },
    "solid": { "material": "plaster_warm" }
  },
  "tags": { "street": { "wall": null, "floor": { "material": "setts" } } }
}
```

| Field | Meaning |
| --- | --- |
| `materials.<slug>` | `colour` (linear RGB, each 0 to 2), `roughness` (0 to 1), and optionally `texture`, a slug of the client's texture library (`assets/textures/<slug>/`), repeated every `metres` (above 0, at most 100; default 2). Without `texture` the colour is drawn flat |
| `default` | the style of a place no tag binds. Each part is optional; a part left out is the client's plain grey |
| `default.wall` | `material`, and `height_m` (above 0, at most 20; default 2.7). The wall's thickness is the disclosed colliders' (0.20 m), so what is drawn is what stops the body |
| `default.floor`, `default.solid` | `material` |
| `default.opening` | a doorway's drawn opening: `width_m` (default 2.0, the colliders' gap) and `height_m` (default 2.1), each above 0 and at most 10. A drawing size, never a rule: the server decides who passes |
| `tags.<tag>` | the parts of the default this tag changes, key by key; `"wall": null` draws no walls (a street's edges are its neighbours' fronts). A place's tag is looked up as written, then by each shorter `.`-prefix; the first of the place's disclosed tags, in disclosed order, that binds is used |

Every material a style names must be declared under `materials`. `mineworld packs validate` refuses a
faulty file by code: `SHELLS_MALFORMED` (unparseable, an unknown key, a wrong type, `format` not 1),
`SHELLS_OUT_OF_RANGE` (a number outside its range), `SHELLS_REFERENCE` (a material not declared); the
corpus is `packages/tests/fixtures/shells/`. A missing file is not a fault: every shell is plain grey.
The client reads the file once; a faulty one is warned about and drawn plain grey. A texture slug the
client's library does not hold is drawn with its colour alone.

A shell builds **no collider** but its floor, which is ground (`Build.LAYER_WORLD`): the walls and solids
that stop the body are `scripts/slice/client_bodies.gd`'s, from the same disclosure. Art a depiction
draws collides on `Build.LAYER_ART`, which the body ignores wherever the current place discloses its
walls (`ARC-52`).

## 7. `depictions/<world-key>.yaml`

A world's 3D look as data (`ARC-177`; the rules every reader keeps and the eight refusal codes are
`docs/MODULE_SPEC.md` §6.1.2). Read by `scripts/depictions/data/depiction_file.gd` (`SliceDepictionFile`)
and drawn by `SliceDataDepiction` and the generic builders beside it; validated in full by
`mineworld packs validate` (`mineworld_packages::depiction`). The registry (`SliceWorlds.open`, 16f-c)
draws a world from `<pack>/depictions/<key>.yaml` whenever the selected pack has that file, before any
depiction script; the file named by `--slice-depiction-file=<path>` (launcher `--depiction-file=<path>`)
comes first of all. Its `scope.world` must equal its stem. A world needs no script to be drawn from data,
and the pack, not the client, decides which worlds it draws so. Without the file (and without a script)
the world is drawn plainly from disclosure (§6). The start line names the file:
`[slice] world <key>, drawn by SliceDataDepiction from <path>`.

**Frame and units.** Plan coordinates are the street frame's integer millimetres `[x, y]` (x east, y north).
A plan point `(x, y)` at height `h` stands in the scene at `frame.origin_m + (x / 1000, h, −y / 1000)`.
Heights are metres above the street's surface. Polygons are 3 to 64 `[x, y]` vertices, simple, closed
implicitly. Lines have at least two points. Rectangles are `[x, y, width, height]`, (x, y) the south-west
corner. Every block marked *cited* needs a non-empty `source`.

| Section | Required | Fields |
| --- | --- | --- |
| `format` | yes | `1` |
| `scope` | yes | `{"world": key}`, equal to the file's stem |
| `frame` | yes | `{"origin_m": [x, y, z]}` |
| `street` | yes | `{"tags": [tag, ...], "footprint_mm": rect}` |
| `places` | yes | `[{"tags", "label", "footprint_mm": rect, "doorway_mm": [x, y]}]`: the street-side point of each place's door |
| `spawn` | yes | `{"at_mm", "up_m", "yaw_deg"}`, the offline spawn, `up_m` above the drawn ground |
| `terrain` | no (flat ground) | `{"extent_mm": rect, "cell_m" (0.5…4), "blend_m", "seed", "layers": [{"name", "material", "repeat_m", "tint"?}] (≤ 4), "default_layer", "regions": [region], "profiles"?: [profile], "bed"?: {"depth_m", "run_m", "source"}}` |
| `water` | no (none) | `{"level_m", "far_field_water", "extent_m", "polygon_mm", "shader"?: {"depth_fade_m"?, "shore_foam"?}, "lilies"?: [{"polygon_mm", "cover", "seed"}], "source"}`; `far_field_water` names a water of the same world's far field, whose colours it takes |
| `edges` | no | `[{"name", "kind": "kerb" \| "quay" \| "revetment" \| "rail" \| "wall" \| "gate", "line_mm", "height_m", "width_m", "material", "tint"?, "open_at"?: [tag], "drop_m" (quay, revetment only), "posts_m" (rail only), "source"}]`; a gap is left at the doorway of each place tagged in `open_at` |
| `paths` | no | `[{"name", "line_mm", "width_m", "layer"}]`, painted with a terrain layer |
| `buildings` | no | `[building]` (≤ 64) |
| `trees` | no | `{"species": [species] (≤ 16), "stands"?: [stand], "singles"?: [{"species", "at_mm", "height_m"}], "keep_clear": {"places", "paths_m"}, "lod_m", "air_walls"?: [{"ring": {"centre_mm", "radius_m"}} \| {"line_mm"}], "source"}` |
| `scatter` | no | `[{"name", "kind": "boulder" \| "reeds" \| "grass" \| "wildflower", "area": {"polygon_mm"} \| {"along_mm", "width_m"}, "count" (reeds ≤ 40), "size_m": {"min", "max"}, "seed", "range_m"?, "source"}]` |
| `decks` | no | `[deck]`: walked floors on piles — a pier, a jetty, a boardwalk, a platform (`ARC-178`) |
| `light` | no | `{"gi"?: [{"box_mm": rect, "height_m", "subdiv": 64 \| 128 \| 256}] (≤ 4), "probes"?: [{"box_mm", "height_m", "centre_up_m", "update": "once"}]}` |
| `models` | no (none) | `[model]` (≤ 64): place assets of the same pack (§8), placed in the scene (16f-d1; `ARC-177` amended) |
| `extras_from` | no (the pack's own) | a world key: whose extras draw a person the pack does not bind and every decor figure (step-26 E-2, S33-g §5.2; `SliceCast.use_extras_from`) |
| `decor` | no (none) | `[{"pose": "walking", "from_mm", "to_mm"} \| {"pose": "standing", "at_mm", "yaw_deg"?}]` (≤ 5): decorative figures with no world entity, on the street, on the drawn ground; a walker goes out and back (S33-g §5.3) |
| `viewpoints` | yes | `{"farfield": [{"id", "at_mm", "eye_m", "roles"?: ["protected" \| "judged" \| "high"]}], "farfield_shots"?: [view], "perf"?: [view], "perf_breakdown"?: [perf view name], "probe_route"?: [[tag, ...], ...]}`; a view is `{"name", "at_mm", "eye_m", "yaw_deg", "pitch_deg"}` |

- **region** (*cited*): `{"name", "polygon_mm", "height", "layer"}`. A `height` is exactly one of
  `{"flat_m", "crown_m"?, "crown_along"?: "x" | "y"}` (crown and axis together) or
  `{"ramp": {"from_mm": [p, p], "from_m", "to_mm": [p, p], "to_m"}}`, linear in the distance between two lines
  parallel within 1°. Heights −20…60 m.
- **profile** (*cited*): `{"name", "bearing_deg", "samples": [{"distance_m", "height_m"}]}`. Outside every
  region the ground follows the two profiles whose bearings bracket a point — bearing and distance taken from
  the scene's origin, where the far field is centred (true north is −z) — each sampled linearly (the end
  samples' heights beyond them) and blended by angle. Inside `water.polygon_mm` the ground is the bed:
  `level_m − bed.depth_m · min(1, d / bed.run_m)`, `d` the distance to the polygon's edge. A region blends
  into its surroundings over `blend_m` (smoothstep); later regions win. The rasterised grid's SHA-256 is
  printed when it is built (`[depiction] terrain …`).
- **building** (*cited*): `{"name", "depicts": [tag, ...] | null, "footprint_mm", "storeys" (1…4), "storey_m"
  (2.4…4.0), "plinth_m", "walls": {"material", "tint"?}, "roof": {"form": "gable" | "hip" | "mono" | "flat",
  "ridge"?: "x" | "y", "falls" (mono only): "n" | "s" | "e" | "w", "pitch_deg" (0…60), "overhang_m"?, "material",
  "tint"?}, "openings"?: [{"side", "kind": "door" | "double_door" | "window" | "shop_window", "storey",
  "offset_m" | "at_doorway": true, "width_m", "height_m", "sill_m"?, "state": "closed" | "open" | "dark" |
  "glazed"}], "chimney"?: {"at_mm", "size_m": [w, d], "above_ridge_m"}, "interior"?: interior}`.
  `depicts: null` is a shell nobody enters; it may not stand on any place's floor. A door may be `open` only
  `at_doorway` of a building that depicts a place; an opening may be `glazed` only in a building with an
  `interior` (`DEPICTION_DOOR`).
- **interior** (*cited*; S33-f): the room inside a shell that depicts a place, drawn by
  `scripts/depictions/data/interior_builder.gd` (`DepictionInteriors`): `{"floor": surface, "ceiling": surface,
  "fittings"?: [{"name", "box_mm": rect, "height_m" (0…10], "material", "tint"?, "source"}] (≤ 32),
  "lights"?: [{"at_mm", "up_m" (0…`storey_m`], "energy" (0…16], "range_m" (0…30], "colour"?}] (≤ 8)}`. Only a
  building that `depicts` a place has one (`DEPICTION_MALFORMED`). The room is the footprint less half the
  wall's thickness: the floor's finish lies 6 mm over the lowest ground under the footprint, the ceiling one
  `storey_m` above it. A **fitting** is a part of the room's shell that the world states as a solid naming no
  furnishing — an oven wall, a fitted counter, a stair — drawn as a block from the ground under its centre to
  `height_m`, in its material, and colliding as drawn on the world layer, so the geometry probe meets it
  where the server's solid is; its box lies inside the footprint (`DEPICTION_OUT_OF_RANGE`). Furniture is
  never a fitting: every free-standing piece is the world's `body: furnishings` (§5, `ARC-307`). A **light**
  is an omni lamp `up_m` above the ground at `at_mm`, falloff 1.15; a lamp just inside the door is the
  threshold light (VISUAL_SLICE §6.2).
- **species** (*cited*): `{"name", "latin", "form", "card", "height_m": {"min", "max"}}` (0.3…100 m). `form` is
  a builder form and `card` a card of this client's library; the client refuses an unknown one
  (`DEPICTION_REFERENCE`). Forms today: `conifer`, `broadleaf`; cards: `needle`, `leaf`.
- **stand**: `{"name", "area": {"ring": {"centre_mm", "inner_m", "outer_m"}} | {"polygon_mm"}, "spacing_m"
  (≥ 1.0), "density": {"from", "to", "rise_m": [a, b]}, "mix": [{"species", "weight", "prefer"?: "low" |
  "rise"}], "seed"}`, at most about 4 000 trees.
- **model**: `{"asset": <asset id>, "depicts": [tag, ...] | null, "anchor": anchor}`. `asset` names a
  folder of the pack's `assets/places/` (§8). `depicts` binds the model to the place of `places` that carries
  every tag, as a building's `depicts` does; `null` is a piece nobody enters (a frontage, a street). The
  `anchor` is exactly one of:
  - `{"entrance": <entrance>, "at_doorway": true}`: the asset is moved, by translation only (`ARC-45`), so its
    named entrance node stands on the depicted place's `doorway_mm`. No coordinate is written twice: a moved
    doorway moves the asset. `at_doorway` on a model whose `depicts` is `null` is refused `DEPICTION_DOOR`.
  - `{"at_mm": [x, y], "yaw_deg"?: d}`: the asset's origin stands at the plan point, turned `d` degrees about
    the vertical (−360…360, default 0).

  The validator refuses an anchor that is neither (`DEPICTION_MALFORMED`), a tag no place carries, an asset the
  pack lacks or an entrance its asset lacks (`DEPICTION_REFERENCE`); the client checks the anchor's shape and
  trusts a validated pack for the names. A model draws no collider that stops the player (`ARC-52`: art is on
  `LAYER_ART`).
- `material` is a texture slug of the client's library (`assets/textures/<slug>/`); `tint` is `"#rrggbb"`.
  On an edge (and a shell) the texture repeats every 2 m (schema default).
- **edges** are drawn on the ground along their line: a kerb, wall or gate is a block `height_m` tall and
  `width_m` thick; a quay or revetment a wall from `drop_m` below the ground to `height_m` above it; a rail
  `width_m`-square posts every `posts_m` with a top and a middle rail. A gap of the stub's door width (1.4 m)
  is left at each doorway, of a place tagged in `open_at`, within 1 m of the line. Every edge collides as
  drawn, on `Build.LAYER_ART` (`ARC-52`), so an edge on a place's boundary is where the geometry probe finds the server's wall; the server's
  walls still decide who passes.
- **buildings** stand on the lowest ground under their footprint. Walls are 0.20 m thick (the disclosed walls'
  thickness), up to the eaves (`storeys` × `storey_m`): centred on the footprint's edge for a shell that depicts
  a place (its footprint is the place's floor; two places sharing an edge share a wall), inside the footprint
  for any other shell (its footprint is its outer extent); a plinth course stands
  5 cm proud; every opening is a hole through the wall (its reveal) with a head, and a window also a sill;
  `dark` fills it with a dark card at the wall's inner face, `closed` with a timber leaf, `glazed` with clear
  glass in the wall's middle, `open` with nothing.
  `offset_m` counts from the facade's left end seen from outside. The roof covers the footprint's bounding
  rectangle with `overhang_m` eaves. Every shell collides on `Build.LAYER_ART` (`ARC-52`), with a gap at least the stub's
  door width (1.4 m) at each `at_doorway` opening; a leaf never collides (the server decides who passes). While a file has `buildings`, nothing of the stub is drawn.
- **deck** (*cited*, `ARC-178`): `{"name", "depicts": [tag, ...], "footprint_mm": rect, "surface_m",
  "thickness_m" (0.05…1), "boards": {"material", "tint"?, "across": "x" | "y", "width_mm" (50…400), "gap_mm"
  (0…20)}, "approach"?: {"side": "n" | "s" | "e" | "w", "length_m", "from_m"}, "piles": {"bents" (2…64),
  "diameter_m" (0.1…1), "inset_m", "to_m", "bracing": bool, "material"}, "rails"?: {"sides": [side], "height_m"
  (0.3…1.5), "rail_m", "post_m", "posts_m", "material"}, "curb"?: {"height_m" (0…0.3), "width_m"}, "shelter"?:
  {"posts_mm": [p, ...] (≤ 16), "post_m", "eaves_m" (1.8…6), "roof": roof, "lantern"?: {"post": index, "up_m"}},
  "moored"?: [{"kind": "rowing_boat", "at_mm", "yaw_deg", "length_m" (2…12), "beam_m" (0.8…4)}] (≤ 16)}`. A deck
  is the floor of the place it depicts: its footprint lies within 50 mm of that place's, and that place's floor
  height in the scene is `surface_m` (its door's in-point, its frame origin, the server's solids and its
  furnishings stand there). The boards cover the footprint at `surface_m`, `thickness_m` deep; an `approach` is a
  ramp inside the footprint from its `side` edge at `from_m` to the surface over `length_m`, no steeper than 1:12
  (2010 ADA Standards §405.2; `DEPICTION_OUT_OF_RANGE`). Piles stand in `bents` pairs `inset_m` in from the
  sides, evenly from `inset_m` of each end, from `to_m` up to the underside, with a cap beam and, with `bracing`,
  an X-brace each. Rails are round, `height_m` above the surface (on an approach, above the ramp), on each listed
  side except across the approach's edge; posts every `posts_m`; a curb runs under each railed side. A shelter's
  posts must stand on the deck (`DEPICTION_ON_FLOOR`); its roof covers the posts' bounding box like a building's
  roof, eaves `eaves_m` above the surface. `moored` craft are static (`ARC-171`), each tied by a line to its
  nearest pile, and may not stand on any place's floor (`DEPICTION_ON_FLOOR`). The deck's surface and approach
  collide as ground (the group `mw_depiction_ground`); rails, curb and shelter posts collide as drawn. The terrain
  under a deck, and within one cell of its sides other than the approach's, stays `thickness_m` below the
  surface there and below `surface_m`. Nodes are tagged `mw_category` `landmark.<name>` (craft `landmark.<name>.boat`) when the depicted
  place is tagged `landmark`, else `deck.<name>`. A drawn person stands on the depiction's ground colliders
  under its observed position. There is no field for a bench or any other furniture (`ARC-307`).
- **scatter** is placed by its seed, uniformly in its polygon or in its strip, on the ground: boulders are the
  client's procedural granite boulder at the drawn size; reeds, grass and wildflowers are cards in one
  `MultiMesh` per entry (reeds half as wide as tall), hidden beyond `range_m`.

**Defaults the client draws with no section.** No `terrain`: the stub's flat ground. While a file has no
`buildings`, the client draws each place as the stub's labelled grey box (S33-c §5.5) and shows the stub's HUD
note; on terrain, only the boxes' walls (the ground is the floor). Every builder number that is not a field is a unit conversion, a cap the validator also enforces, or a
cited physical default shared by every world.

**Status of `models` (16f-d1).** The validator and the reader accept the section; the client does not yet load
the assets it names. Loading them (`GLTFDocument`, the re-binding of §8, LODs) is 16f-d2, which also moves
Social Café's and Market Town's files from `tools/presentation/social_cafe/depictions/` into the default pack.

## 8. `assets/places/<asset-id>/` and `materials/material_rules.yaml`

A **place asset** is drawn art for a place or a piece of a street: a glTF 2.0 binary and its contract
(`docs/PACKAGE_FORMAT.md` §3; 16f-d1). A depiction file's `models` place it (§7). The server never reads it.

    assets/places/<asset-id>/
      <asset-id>.glb      the model: meshes, lights (KHR_lights_punctual), named empty nodes
      asset.yaml          what it means (YAML's JSON subset as the export tool writes it)
    materials/
      material_rules.yaml what glTF cannot carry, by name, for every asset of the pack
      textures/<md5>.png  textures made at build time (leaf, flower and grass cards), named by content

**`asset.yaml`.** `PACKAGE_FORMAT.md` §3's fields, and the ones the 3D client needs:

| Field | Required | Meaning |
| --- | --- | --- |
| `id` | yes | the folder's name, a key (1–48 of `a-z 0-9 -`) |
| `type` | yes | `place_visual` |
| `format` | yes | `{"model": "<asset-id>.glb", "rules"?: "materials/material_rules.yaml"}`; `rules` is required when the model names a recipe |
| `semantic_bindings` | yes | `{"entrances"?: [name], "interaction_anchors"?: [name], "spaces"?: [name]}`: each a node of the model. An **entrance** stands at a drawn door's street-side point (16c's out-point, 0.4 m in front of the sill), its +z axis facing out of the building (the façade's outward normal); a **space** stands at a room's centre, its scale the room's size in metres |
| `doors` | no | `[{"name": <entrance>, "label", "space": <space> \| null, "depth_m", "floor_m"}]`: per entrance, the door registry's row (D-16c-4): its label, the room behind it (`null`: a closed door), the depth from the sill to the room's inner face, the room floor's height above the sill |
| `presentation` | no | `{"style_family": [style id]}` |
| `scale` | yes | `{"unit": "meter"}` |
| `license` | yes | `{"spdx": <SPDX expression>, "sources"?: [{"what", "spdx", "url", "files": [path]}]}`: the licence of what the asset's author made, and every library file it references by path, under that file's licence |
| `source` | no | `{"tool", "authoring"}`: the tool that wrote it and the source it built it from |

**What glTF cannot carry travels by name.** The model's materials are named `m_<hash>` and its nodes
`<kind>_<hash>_<n>`, where `<hash>` is 12 hex digits of the MD5 of the object's **recipe**: its Godot class and
every stored property that differs from a fresh instance of that class (sub-resources inline; a file of the
3D client's library by its path, `{"library": "assets/…"}`; a texture made at build time by its file,
`{"file": "materials/textures/<md5>.png"}`; any other value in Godot's own text form, `{"variant": …}`).
`material_rules.yaml` holds each recipe once:

    {"format": 1, "source": "<tool>",
     "materials": {"m_<hash>": {"class": "ORMMaterial3D", "set": {…}}, …},
     "nodes": {"<hash>": {"class": "Label3D", "set": {…}}, …}}

The kinds: `mesh` (a mesh instance; glTF carries its geometry and a metallic-roughness material, the recipe
its shadow casting and frame-cost category), `light` (a glTF light; the recipe adds what
`KHR_lights_punctual` lacks), `node` (an empty node standing for anything glTF has no form for: a Label3D
sign, a body and its collision shapes, a reflection probe). A `node` recipe with a `library` key stands for a
whole library scene of the client, such as a Poly Haven prop, which the client instances with its imported
LODs; its `surfaces` list `[node path, surface, material name]` for each re-tinted surface. A plain group is
`g_<n>`. The client re-binds every named object from its recipe, so nothing is lost and nothing is guessed;
an unnamed or unknown object is drawn as glTF gives it. Recipes are named by content, so assets of several
sources share one file, and a re-export writes the same bytes.

**Validated** by `mineworld packs validate` (`mineworld_packages::place_asset`), before the depiction files,
first fault refused, in this order:

| Code | Fault |
| --- | --- |
| `ASSET_MALFORMED` | not parseable; an unknown key; a wrong type; `type` not `place_visual`; `id` not the folder's name; `scale.unit` not `meter`; `format.model` not a `.glb` beside `asset.yaml`; the model not glTF 2.0 binary; the rules file not format 1 |
| `ASSET_UNLICENSED` | `license.spdx` empty or not an SPDX expression; a source's `spdx` likewise |
| `ASSET_REFERENCE` | the model missing; a binding the model has no node for; a door's entrance or space the bindings do not list; a material or node recipe name of the model that the rules file lacks |

**Made by a tool, not by hand.** `tools/presentation/export_place_art.gd` builds the pieces an authoring source
lists (`tools/presentation/social_cafe/source.gd` is Social Café's street) with the client's own builders and
writes them with Godot's `GLTFDocument`. It refuses to write an asset that references a library file under no
licence its source states. Two runs write byte-identical files.

## 9. `stations/<kit>.yaml`

How the client draws a station building (`ARC-192`, S29-d). A world's depiction places a kit at a **site**;
the kit says what the building is. YAML's JSON-compatible subset (`DEP-16`), read by
`scripts/station/kit.gd` (`StationKit`) and drawn by `scripts/station/depot.gd` (`StationDepot`). The
shipped kit is `depot.yaml`, the Santa Fe Depot (`ARC-106`).

**Frames.** Lengths are millimetres. A kit place's numbers are in that place's own frame, the world's frame
(x east, y north, `CORE_CONCEPTS.md` §6.1). So a door's `at` is the passage point the world states, and the
client checks the two agree: the binding misses by name when a world's doorway lies more than 0.15 m from
the drawn door.

**The site** (in the depiction, not in the kit): `{ kit, facade_x, axis_z, facing }`. The arch's façade
stands at scene x = `facade_x`, the arch's doorway on z = `axis_z`. Only `facing: "east"` is drawn.

| Key | Meaning |
| --- | --- |
| `format` | `1` |
| `kit`, `style`, `source` | the kit's name, its style and its overall source |
| `places.<name>` | one drawn place. `tags`: the role tags it depicts (`CORE_CONCEPTS.md` §6.5); a world's place binds to the entry whose tags are all among its own, the most tags winning. `volume`: the scene volume's name. `shell`: `main` (the arched building; exactly one), `annex` (a walled room off one of its walls) or `open` (an open place under the arcade along one of its walls). `floor`: `{min, max}`. `source` |
| `doors[]` | `name`; `from` (`outside` for the one door from the street, else the main place) and `to`; `at`, the doorway point on the main place's side; `wall` (`east`, `west`, `north`, `south`) of the main place; `there`, the point on the far side (inner doors); `leaves` and `leaf_mm` [width, height]; `source` |
| `shell` | `walls_mm` {`east`, `other`}, `eaves_mm`, `ridge_mm`, `plinth_mm`, `overhang_mm`, `clerestory` {`count`, `size_mm`, `sill_mm`}, `source` |
| `arch` | `clear_mm` [width, crown], `screen_mm` (the door screen's height), `bays` (odd: one bay stands on the axis), `gable_mm` (the parapet's crest), `source` |
| `towers` | `plan_mm`, `forward_mm` (east face in front of the façade), `inner_offset_mm` (inner face from the axis), `shaft_mm`, `belfry_mm`, `drum_mm`, `dome_mm` [diameter, rise], `lantern_mm`, `finial_mm`, `source` |
| `arcade` | `depth_mm` (from the wall to the piers), `pier_mm`, `pitch_mm`, `length_mm`, `spring_mm`, `rise_mm`, `eaves_mm`, `source` |
| `annex` | `eaves_mm`, `source` |
| `tile` | the glazed-tile pattern of each surface: `dome`, `lantern`, `emblem`, `floor` (`zigzag`, `emblem`, `quarry`, `frieze`; `shaders/glazed_tile.gdshader`) |
| `fitout` | the inside (S29-e): `wainscot_mm` (the tiled wainscot's height), `arches` (transverse arches, between the clerestory bays), `rib_mm` [width, depth], `spring_mm`, `pilaster_mm` (how far the pilaster stands proud; ≤ 150 mm, 12e's tolerance), `purlins`, `chandelier_mm` [diameter, height], `chandelier_low_mm` (its lowest point), `board_mm` [width, height] and `board_up_mm` (its centre), `clock_mm` and `clock_up_mm`, `counter_mm`, `counter_depth_mm`, `screen_mm` (the ticket window's screen top), `grilles`, `platform_board_mm`, `source` |
| `track` | the platform's edge and the line (S29-e): `gauge_mm`, `edge_mm` (platform top above the rail head), `near_mm` (edge to the near track's centreline), `centres_mm`, `tracks`, `tie_mm` [length, width], `tie_pitch_mm`, `rail_mm` [width, height], `strip_mm` (the detectable-warning strip), `coping_mm`, `fence_mm`, `cutting_mm` (the cutting's length beyond each platform end), `bank_mm` (its banks' height), `source` |

**Refusals,** in this order, each printed `[mineworld-slice] <CODE>: <path>: <detail>`, shown full-screen,
and exit 3 in a scripted run: `STATION_KIT_MISSING` (no file); `STATION_KIT_MALFORMED` (not JSON, a wrong
type, an unknown key, a door naming an unknown place, not exactly one main place or one street door,
`format` other than 1); `STATION_KIT_UNCITED` (a place, door or group without `source`);
`STATION_KIT_OUT_OF_RANGE` (a leaf outside `VISUAL_SLICE.md` §3.2; a number not positive; an even
`bays`; a doorway point not 400 mm inside its wall within 1 mm, on either side); `STATION_SITE_FACING` (a
site not facing east).

**What a kit may not do.** It decides nothing about reachability or availability. It never names a world or
a place key. Benches, the kiosk and other free-standing furniture are never in a kit: they are the world's
furnishings (§5). Walls, towers, piers and the counter are the building's shell, so they are in the kit,
with colliders where a body can meet them.

**Checks.** `--slice-station-kit [--slice-station-kit-file=<path>]` prints the kit and measures the built
depot against it (rays against its colliders). `--slice-landmark` is the rendered landmark check (S29-d
§5.9). `--slice-station-shots` writes `shots/<world>/station/st_*.jpg`. `--slice-station-walk`, connected,
walks from the seat into the station. `--slice-station-kit` also builds the station pieces at the kit's sizes
and at village scale and measures them (`PIECE` rows, S29-e AC-e-2), then prints what each drawn fixture shows.

### 9.1 Station pieces

The inside and the line are drawn by pieces (`scripts/station/pieces.gd`, `scripts/station/track.gd`;
`ARC-192`'s note of 2026-10-20). Each is a function of its sizes in metres and a style, and reads no kit, no
place and no world, so another building -- a village halt -- draws the same pieces at its own sizes.

| Piece | Sizes | Shows |
| --- | --- | --- |
| `bench_row` | length, depth, seat height | -- |
| `kiosk` | footprint, height | `kiosk`: shutter down (shut) or up with papers (open) |
| `window_bay` | width, counter height and depth, screen height, grilles | `window`: shutters down and the `sign.window_closed` plate (shut), or up (open; no clerk -- a clerk is a disclosed person) |
| `board` | width, height | `board`: `sign.departures` over `sign.no_service`, or one row per disclosed line |
| `clock` | diameter | `clock`: hands at the disclosed time of day, or none |
| `StationTrack.platform` | floor rectangle, edge side, top, depth, strip, coping | -- |
| `StationTrack.track` | ends, gauge, tie, tie pitch, rail; end `through` or `buffer` | -- |
| `StationTrack.cutting`, `fence` | ends, width and banks; ends, height and gate | -- |

A piece decides nothing a world owns. What a fixture shows is `MineWorldStationState` and
`MineWorldStationClock` (`clients/shared/station/`, `ARC-108`) of the place it stands in, remembered per place
by `StationFixtures`: the last observation made there, or the defaults for a place never observed.

## 10. `sit_points/<world-key>.yaml`

How a person seated at each of a world's sit points is drawn (`ARC-145`, step-34 §8.2; S33-f). A sit point
is the world's (`posture`'s `sit-points:`, `CORE_CONCEPTS.md` §6.4); this file is its depiction, bound by the
place's tags and the point's key. YAML's JSON subset, read by the shared sit decision
(`clients/shared/sitting/sit_rule.gd`, `MineWorldSitRule`) in both clients and drawn by
`scripts/slice/seating.gd` (`SliceSeating`).

```json
{"format": 1, "scope": {"world": "lakeside"},
 "points": [{"tags": ["bakery"], "point": "long-table.3", "group": "long-table.chair-3",
             "prop": "long-table.chair-3", "seat_height_m": 0.45, "seat_offset_m": -0.57,
             "fallback": {"at": {"x": 1660, "y": 3000}, "facing": 270000}}]}
```

| Field | Meaning |
| --- | --- |
| `tags`, `point` | the place's disclosed tags (all must be the place's) and the point's key |
| `seat_height_m` | the drawn seat's surface above the floor; a seated body's own seat rests on it (`Human.sit_on`), and `--seat` measures it against the drawn seat within 20 mm |
| `seat_offset_m` | signed: the pelvis is drawn at the point's `at` moved this far **against** its facing — positive back onto a bench behind a point in front of it, negative forward onto a chair in front of a point behind it (Q-S34c-2) |
| `prop` | the key of the furnishing whose seat it is (`body: furnishings`) |
| `group` | the point's group, as the world will state it |
| `fallback` | `{"at": {"x", "y"}, "facing"}`, the point's position (mm, the place's frame) and facing (millidegrees), for the shared rule's fallback branch while the world discloses no sit points (Q-S33f-1 (A)); S34-c copies them into the world's place files and removes them here |

**The shared decision.** Where the place discloses `sit-points`, a person is drawn seated iff their disclosed
`posture-state` names a point. Otherwise a person is drawn seated at the nearest fallback point within 0.40 m
of their disclosed position once they have not moved for 2 s of world time, unless a person with a lower
entity id is drawn there already (step-33 SD-S33-7). Neither client holds a copy of the rule
(`client_rules.rs`, I-f-6); `tools/cli/tests/client_sit_rule.rs` checks that both clients draw the same person
on a contested seat.
