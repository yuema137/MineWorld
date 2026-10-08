# What a 2D Presentation Pack contains for `mineworld-2d`

**Audience:** coding agents writing or changing a Presentation Pack for the 2D reference client, or
the client's loader. A specification, not an introduction; [`README.md`](README.md) is the
orientation.
**Authority:** [`docs/DECISIONS.md`](../../docs/DECISIONS.md) `ARC-46` (roles and bindings), `ARC-45`
(how a town is laid out and dressed), `DEP-16` (the file format), `ARC-14` (the default style).
[`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §6.1 names the two files; this document specifies
their contents for this client. The loader is `scripts/presentation.gd`; where this document and the
loader disagree, that is a defect to be recorded and fixed, never left.

---

## 1. Selecting a pack

```text
--presentation=<dir>     a pack directory, absolute or relative to the repository root
--presentation=none      no pack: everything is drawn plainly (§6)
(absent)                 presentation/mineworld-default/2D
--variant=<set>          a binding set of that pack (§3); absent: the pack's default_set
```

The pack is read **at runtime, from disk**. It is never imported into the client's Godot project, has
no `.import` sidecars, and contains no GDScript. Images are read with `Image.load_from_file` (PNG) and
`Image.load_svg_from_buffer` (SVG, rasterized at the sprite's own scale × 2 so edges stay clean);
shaders are compiled from their source text. A file the pack names and does not contain is a load
error naming the file, and the client then draws that role plainly — it never stops the world from
being shown.

## 2. The file format

Both files are YAML documents written in **YAML's JSON-compatible subset**: a single JSON object, so any
YAML tool reads them and Godot's built-in `JSON` parser reads them too (`DEP-16`). No comments — this
document is where they are explained. Every file path in them is relative to the pack directory and
uses `/`. Unknown keys are ignored with a warning, so a newer pack degrades rather than fails.

## 3. `assets/asset_bindings.yaml`

```json
{
  "format": 1,
  "default_set": "town",
  "sprites": {
    "<sprite id>": { "file": "art/…", "anchor": [ax, ay], "scale": k, "ink": "none|veg|shop|prop" }
  },
  "sets": {
    "<set>": { "extends": "<set>", "roles": { "<role>": "<sprite id>" } }
  }
}
```

- **`sprites`** — every drawable image once. `anchor` is the pixel, in the image as stored, that stands
  on the ground: the painter's sort and the position both use it. `scale` is the draw scale; the
  default pack's generated sprites are cut so that pixel height = height in metres × 64, hence `0.5`
  at 32 px per metre. `ink` selects the contour strength of `renderer/godot.yaml`'s `ink` effect
  (`none` for art that carries its own line).
- **Directional sprites** are four ids sharing a base: `<base>_front`, `<base>_front_b`, `<base>_back`,
  `<base>_back_b` (two stride poses per view). A role bound to a base resolves to whichever exist; a
  sprite with no `_back` is shown from the front only, and with no `_b` pose the gait is a bob only.
- **`sets`** — `ARC-14`'s variants. A set's `roles` override the set it `extends`; a role bound
  nowhere in the chain is unbound (§6). `default_set` names the set used without `--variant`.

### 3.1 The roles the client asks for

| Role | Meaning | Chosen how |
| --- | --- | --- |
| `player` | the Person this client controls | always |
| `person:<n>`, n = 0, 1, … | the cast; the client uses as many as the set binds contiguously from 0 | a stable hash of the person's id **string**, modulo the cast size |
| `facade:<tag>` | the outside of a place whose tags include `<tag>` | the place's tags, in the order the world lists them; the first bound one wins |
| `facade:unknown:<n>` | a place not yet visited in this instance (`ARC-45` point 3) | the stable hash of the place's id string |
| `prop:<name>` | anything `renderer/godot.yaml` places: dressing, fittings, edge planting | named by that file |
| `texture:floor`, `texture:paper` | the interior floor tile; the paper tooth of the `grade` effect | named by that file |

A role the client asks for that the set does not bind is drawn plainly. The client never asks for a
role by a world key (`alice`, `cafe`), only by tag, index or hash.

## 4. `renderer/godot.yaml`

```json
{
  "format": 1,
  "projection": { "kind": "isometric", "px_per_metre": 32 },
  "camera": { "zoom": 1.55 },
  "clear_color": "#rrggbb",
  "palette": { "<name>": "#rrggbb" },
  "effects": {
    "grade": { "shader": "renderer/grade.gdshader", "paper": "texture:paper" },
    "ink":   { "shader": "renderer/ink.gdshader", "px": 1.35, "bite": { "veg": 0.08, "shop": 0.13, "prop": 0.24 } }
  },
  "ground": { "root_margin_m": 5.0, "land_margin_m": 60.0 },
  "facades": { "*": { "dressing": [ … ] }, "<tag>": { "dressing": [ … ] } },
  "outdoor": { "<tag>": { "width_m": w, "depth_m": d, "door_from_left_m": a, "dressing": [ … ] } },
  "interiors": {
    "*":     { "width_m": w, "depth_m": d, "door_from_left_m": a, "fittings": [ … ], "lights": [ … ] },
    "<tag>": { … }
  },
  "edges": { "roles": [ "prop:…" ], "rows": 3, "spacing_m": 3.9, "row_gap_m": 2.6 }
}
```

- **`projection`.** `isometric`: 2:1, a plan metre is `px_per_metre / 2` pixels along each screen
  diagonal, a metre of height `px_per_metre` pixels up; plan `+x` (east) runs down-right, plan north
  up-right. `plan`: north up, `px_per_metre` pixels per metre. The projection is presentation only:
  every request the client sends is computed in world millimetres before any projection (`I-5`).
- **`palette`.** Named colours for the drawn ground (`stone`, `stone_hi`, `stone_lo`, `joint`,
  `grass`, `grass_hi`, `grass_lo`, `wall`, `wall_lo`, `wall_side`, `wall_side_lo`, `cut`,
  `cut_edge`, `plinth`, `skirt`); a missing name falls back to the plain colour of §6.
- **Anchoring (`ARC-45` point 4).** Every placement is relative to a disclosed doorway: `along_m` along
  the frontage (positive to the right as seen from the street), `out_m` out into the place the
  doorway is disclosed from, `in_m` into the place it leads to. Which way "in" is follows from the
  doorway's position relative to the centre of the rectangle the root place's doorways span: a doorway
  north of that centre leads north, and so on along the dominant axis. Nothing is placed at an
  absolute coordinate or by a world key.
- **`facades`** — dressing outside each doorway, by the destination's tag once known; `*` applies to
  every doorway, before the tag's own list.
- **`outdoor`** — a destination tag drawn as open ground instead of a building (no façade, nothing
  lifts): its footprint and dressing. The footprint is decoration and is never enforced
  (`ARC-45` point 5).
- **`interiors`** — the footprint of the room behind a façade, drawn when the observer stands in that
  place (the cut-away: the façade fades out, walls stand on the far sides, stubs on the near sides):
  `width_m` along the frontage, `depth_m` inward, the doorway `door_from_left_m` from its left edge;
  `fittings` are `{ "role", "along_m", "in_m" }` measured from the doorway; `lights` are
  `{ "along_m", "in_m" }` pools of lamplight. `*` is used for a tag with no entry.
- **`edges`** — planting beyond the root place's extent, so the town does not end at the edge of the
  frame: `rows` rows, `spacing_m` apart along each side, cycling through `roles`.

## 5. What a pack may not do

It may not decide whether anything is valid, reachable or allowed (`ART_DIRECTION.md` §19,
`ARC-47`); it may not position anything by a world's keys or coordinates (`ARC-45`); and the client
never derives a request from a pack value — a pack changes how the same requests look, never which
requests are sent (`I-5`, step-13 AC-W10).

## 6. Plain drawing, with no pack or an unbound role

People are discs with a facing tick and their label; the player's disc is ringed; a façade is an
outlined block with a door mark; ground is flat colour; the projection is `plan`. Plain drawing is
complete enough to play: the scripted checks pass with `--presentation=none`.
