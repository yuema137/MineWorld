## ARC-46 — A Presentation Pack binds roles to art; a client core names no asset

**Date** 2026-10-08 · **Approved by** the primary session at S12 PR 13a's design freeze · **Relates
to** `ARC-1`, `ARC-3`, `ARC-11`, `ARC-14`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §6.1 · **Specified in**
[`clients/2d/PRESENTATION.md`](../clients/2d/PRESENTATION.md)

**Problem.** `MODULE_SPEC.md` §6.1 names `assets/asset_bindings.yaml` and `renderer/godot.yaml` and
specifies neither. The 2D spike resolved art through role maps compiled into its scene script, so its
four variants (`ARC-14`) were code, and the pack it drew from was its own project directory.

**Decided.**

1. **The core asks for roles, never for files.** `person:<n>`, `player`, `facade:<tag>`,
   `facade:unknown:<n>`, `ground:<tag>`, `interior:<tag>:<fitting>`, `prop:<name>`. The pack's
   `assets/asset_bindings.yaml` resolves a role to a file under the pack, per binding set; a role a
   set does not bind is drawn plainly (shapes and text), so the client runs and is testable with no
   pack at all (`--presentation=none`).
2. **`ARC-14`'s variants are binding sets** in one pack: `town` (default), `full`, `people`,
   `procedural`, selected by `--variant=`.
3. **`renderer/godot.yaml`** states the renderer's parameters: projection, pixels per metre, the
   sprite scale rule, the post-process and contour shaders, façade dressing and interior footprints
   per tag, all anchored as `ARC-45` point 4 requires.
4. **The pack is data, loaded at runtime from any directory**: images through `Image.load_from_file`
   and `Image.load_svg_from_buffer`, shaders from their source text. It is never imported into the
   client's Godot project and it contains no GDScript, so `--presentation=<dir>` takes any pack.
5. **Both files are written in YAML's JSON-compatible subset** (`DEP-16`): valid YAML for any YAML
   tool, read by Godot's built-in JSON parser. Their comments live in `PRESENTATION.md`.

**What it does not decide.** Whether an interaction is valid (`ART_DIRECTION.md` §19), or anything
about a world: a pack may depict any world, and a world may be depicted by any pack.

**Note 1, 2026-10-14 (S26-a; `ARC-92`) — the role list gains `look:<key>`.** A person's disclosed `look` (`ARC-91`) is asked for as `look:<key>` before `person:<n>`; for the observer, before `player`. A `look` is a disclosed component value, like the tag in `facade:<tag>`, so "the core never asks for a role by a world key" stands, and the list of role kinds gains one. Specified in `clients/2d/PRESENTATION.md` §3.1; read by the client from S26-e.
