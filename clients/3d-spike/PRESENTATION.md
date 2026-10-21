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

## 1. What the client reads from a 3D pack

Only the cast binding, as data. A character's mesh and texture are resources the client's Godot project
imports, under `clients/3d-spike/assets/characters/cast/<id>/`, because the importer applies the
humanoid bone map at import time and the client cannot load a skinned, retargeted model from a runtime
pack today. Each kind of file has one location; nothing is copied between them.

Also read as data: the far field (`far_field/<world-key>.yaml`, `docs/MODULE_SPEC.md` §6.1.1), the
furnishing bindings (§5) and, for a world drawn from data, its depiction file
(`depictions/<world-key>.yaml`, §6; `ARC-177`).

## 2. `characters/cast.yaml`

YAML's JSON-compatible subset: one JSON object, no comments (`DEP-16`). Godot's built-in `JSON` parser
reads it.

    {
      "format": 1,
      "looks":   { "<look key>": "<character id>" },
      "extras":  [ "<character id>" ],
      "default": "<character id>"
    }

- **`format`** — the integer `1`.
- **`looks`** — a disclosed `look` (`ARC-91`) to the character that draws that person. Several looks map
  to one character only where the operator has accepted it; the default pack maps none twice.
- **`extras`** — the characters that draw a person with no look, a person whose look this pack does not
  bind, and decorative figures. Distinct ids. Every entry passes the same standard as a named person
  (step-26 §6; `ARC-94` once S26-b1 records it); there is no cruder class.
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
  through its index entry) to a passing ground measurement (CC-9), and every gated arm to the standing
  bounds (CC-10 clearance, CC-11 palm, supination, reach and joint ranges).

The reference body (`vera-lindgren`) is not described by a manifest yet: the client's resource index
`assets/characters/cast/index.json` names it `"body": "reference"`, and that entry wins. Its entry also carries
her measured `ground` and `stand_gates`, and `grip` (the side her strap grip holds: `human.gd`'s additive layer, not
the clip, so that arm is not gated).


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

## 6. `depictions/<world-key>.yaml`

A world's 3D look as data (`ARC-177`; the rules every reader keeps and the eight refusal codes are
`docs/MODULE_SPEC.md` §6.1.2). Read by `scripts/depictions/data/depiction_file.gd` (`SliceDepictionFile`)
and drawn by `SliceDataDepiction` and the generic builders beside it; validated in full by
`mineworld packs validate` (`mineworld_packages::depiction`). A world whose depiction script is
`extends SliceDataDepiction` reads `<pack>/depictions/<key>.yaml`, or the file named by
`--slice-depiction-file=<path>` (launcher `--depiction-file=<path>`), whose `scope.world` must equal its
stem.

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
| `light` | no | `{"gi"?: [{"box_mm": rect, "height_m", "subdiv": 64 \| 128 \| 256}] (≤ 4), "probes"?: [{"box_mm", "height_m", "centre_up_m", "update": "once"}]}` |
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
  "offset_m" | "at_doorway": true, "width_m", "height_m", "sill_m"?, "state": "closed" | "open" | "dark"}],
  "chimney"?: {"at_mm", "size_m": [w, d], "above_ridge_m"}}`. `depicts: null` is a shell nobody enters; it may
  not stand on any place's floor. A door may be `open` only `at_doorway` of a building that depicts a place.
- **species** (*cited*): `{"name", "latin", "form", "card", "height_m": {"min", "max"}}` (0.3…100 m). `form` is
  a builder form and `card` a card of this client's library; the client refuses an unknown one
  (`DEPICTION_REFERENCE`). Forms today: `conifer`, `broadleaf`; cards: `needle`, `leaf`.
- **stand**: `{"name", "area": {"ring": {"centre_mm", "inner_m", "outer_m"}} | {"polygon_mm"}, "spacing_m"
  (≥ 1.0), "density": {"from", "to", "rise_m": [a, b]}, "mix": [{"species", "weight", "prefer"?: "low" |
  "rise"}], "seed"}`, at most about 4 000 trees.
- `material` is a texture slug of the client's library (`assets/textures/<slug>/`); `tint` is `"#rrggbb"`.
  On an edge (and a shell) the texture repeats every 2 m (schema default).
- **edges** are drawn on the ground along their line: a kerb, wall or gate is a block `height_m` tall and
  `width_m` thick; a quay or revetment a wall from `drop_m` below the ground to `height_m` above it; a rail
  `width_m`-square posts every `posts_m` with a top and a middle rail. A gap of the stub's door width (1.4 m)
  is left at each doorway, of a place tagged in `open_at`, within 1 m of the line. Edges build no collider:
  what a body meets is the server's walls.
- **scatter** is placed by its seed, uniformly in its polygon or in its strip, on the ground: boulders are the
  client's procedural granite boulder at the drawn size; reeds, grass and wildflowers are cards in one
  `MultiMesh` per entry (reeds half as wide as tall), hidden beyond `range_m`.

**Defaults the client draws with no section.** No `terrain`: the stub's flat ground. While a file has no
`buildings`, the client draws each place as the stub's labelled grey box (S33-c §5.5) and shows the stub's HUD
note; on terrain, only the boxes' walls (the ground is the floor). Every builder number that is not a field is a unit conversion, a cap the validator also enforces, or a
cited physical default shared by every world.
