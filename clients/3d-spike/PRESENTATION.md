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
