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
`.structured-coding/plans/mvp0/pr-vf-person-identity.md`). The default pack binds `vera-lindgren` only,
the one character that exists. Pending operator question Q-PID-2: an unbound look with no extras is drawn
as the client's town figure, not as `default` (§3).

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
