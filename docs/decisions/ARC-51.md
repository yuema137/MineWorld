## ARC-51 — The 3D client reads the Presentation Pack it is given, from disk, selected by `--presentation=<dir>`

**Date** 2026-10-20 · **Status** accepted (PR 16f design frozen 2026-10-20); implemented by S14 PR 16f-a · **Approved by** the operator's freeze of `pr-16f-presentation-pack-3d.md` (2026-10-20; Q-16f-3, Q-16f-7, Q-16f-8 as recommended) · **Relates to** `ARC-46` (the 2D client reads its pack at runtime), `ARC-70` (the wording's layers), `ARC-102` (every view has a far field), `ARC-144`, `ARC-146`, `overall.md` §5 "Physics list" decision 3 (QPL-17) · **Design** `.structured-coding/plans/mvp0/pr-16f-presentation-pack-3d.md` §4.1

**Problem.** Before this record the 3D client read one fixed folder, `presentation/mineworld-default/3D`, through a constant written in four scripts and copied in a fifth. No user could hand it another pack, so "changing the Presentation Pack changes the look" (QPL-17) could not be shown, and the 3D client's own words sat in the shared catalog layer because the client loaded no pack layer (`clients/shared/SETTINGS.md` §6.2).

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) Keep the constant; a second pack is a code edit | Declined: a pack the client cannot be given is not a pack |
| (b) A new 3D flag name | Declined: one flag for both clients |
| (c) **The 2D client's `--presentation=<dir>`, resolved once by one class (`SlicePack`) that is the only place a pack path is built; a directory that is not a 3D Presentation Pack is refused by name** | **Chosen** |

**Choice.**

1. `--presentation=<dir>` names a 3D Presentation Pack directory, absolute or relative to the repository root (a path that leaves the repository is allowed: a user's own pack). Absent, the client reads `presentation/mineworld-default/3D`. The launcher `mineworld-slice` passes the flag through. There is no `--presentation=none` in 3D: every view needs a far field (`ARC-102`).
2. `SlicePack` (`clients/3d-spike/scripts/pack/pack.gd`) resolves the directory once, before anything is drawn or worded, and every reader of the pack (the far field, the cast binding, the furnishings binding, the identity check, the wording) asks it for the path. `tests/acceptance/tests/client_rules.rs` refuses a string literal naming a path under `presentation/`, or a `PACK_DIR` identifier, in any other 3D client script.
3. A directory that does not exist, has no `pack.yaml`, whose `pack.yaml` `type` is not `presentation-pack`, or whose `manifest.yaml` lists no `3d` dimension is refused: `[mineworld-slice] PRESENTATION_PACK_INVALID: { pack: <dir>, why: <text> }` is printed and shown full-screen, and a scripted or harness run exits 3. The client reads only these two identity fields; `mineworld packs validate` validates a pack in full. A pack that lacks an optional file (`characters/cast.yaml`, `furniture/furnishings.yaml`) is not refused: people are the client's plain figure and furniture is plain boxes over its solids (`clients/3d-spike/PRESENTATION.md` §§3, 5).
4. The 3D client loads its pack's `i18n/` as the middle layer of its wording. The 3D client's own keys moved, keys and text unchanged, from `clients/shared/settings/locale/` to `presentation/mineworld-default/3D/i18n/` (Q-16f-8).
5. A second shipped pack, `presentation/mineworld-greybox/3D/` (Q-16f-3), carries `pack.yaml`, `manifest.yaml`, a README, far fields copied from the default pack, and the default pack's words copied unchanged, and no cast and no furnishing bindings. With it the same world is drawn with plain figures and plain furniture, and nothing outside `presentation/` differs.

**Consequences.** A pack is still Tier 0: data and media only (`PACKAGE_FORMAT.md` §6). 16f-b adds `places/shells.yaml` to the greybox pack and makes disclosed geometry the only thing that blocks the player (`ARC-52`); 16f-c and 16f-d move depictions and the café's art into the pack. The 2D client is unchanged.

**Limitation accepted.** The greybox pack's words are a copy of the default pack's (two catalogs to keep in step), because a pack is read alone (multi-pack loading, PK-3, is after MVP-0) and a pack with no words would show every 3D key as its readable form.

**Revisit** when packs may depend on other packs (PK-3), so a pack can inherit the default pack's words instead of copying them.
