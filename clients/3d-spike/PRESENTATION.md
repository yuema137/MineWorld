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
- **`stance`** (PR S26-c-ground) holds **`arms`**: bone → three rest-relative euler degrees (`stand_key`'s
  convention) for the upper arms, forearms and fingers of this body's relaxed stand, solved on its own mesh
  by `cast.py ground --solve-stance --write` (`tools/character_pipeline/PIPELINE.md` §15.2), with the
  clearance and palm turn it measured. `Human.build_character` keys them into this body's `Stand` clip in
  place of `stand_key`'s arms; without it the body keeps `stand_key`'s.
- **`ground`** (PR S26-c-ground) is this body's measured ground contact (PIPELINE.md §15.1): the lowest
  skinned vertex against the floor at rest, over the `Stand` loop and every frame of one `Walk` and one `Jog`
  cycle, and the arms' clearance as built. It is a record, not an input: the client stands every body on its
  own soles from its geometry (`ground_contact.gd`). The reference body's is in its index entry.
- A missing or malformed field draws the person as the town figure with one warning naming the field. So
  does a body whose `.bin` or `_base_color.png` is a Git LFS pointer file (`version https://git-lfs…`, a
  checkout without LFS, `DEP-141`), or one that was never imported.
- `scripts/check_cast.py` holds the default pack to this shape (CC-7, CC-8), every body (the reference body
  through its index entry) to a passing ground measurement (CC-9), and every cast manifest to a stance
  (CC-10).

The reference body (`vera-lindgren`) is not described by a manifest yet: the client's resource index
`assets/characters/cast/index.json` names it `"body": "reference"`, and that entry wins. Its entry also carries
her measured `ground` and her `stance` (the free arm only; her strap arm stays `human.gd`'s grip), which `Human.build`
reads for her own scene (PR S26-c-arms-2).

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
