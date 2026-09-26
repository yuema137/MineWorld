# Assets used by the 2D presentation spike

## Short answer

**No third-party art is used. Every pixel on screen is authored in-engine.**

There are two sources of imagery in this spike and nothing else:

| Source | What it produces | Licence |
| --- | --- | --- |
| `tools/gen_art.py` (written for this spike) | all 43 SVG sprites in `art/svg/` — buildings, trees, street furniture, boats, people | Part of this repository; same licence as the repository (MIT) |
| `scripts/Ground.gd` (written for this spike) | the ground plane — water, quay, paving, grass — drawn with Godot's own 2D primitives | Part of this repository; same licence as the repository (MIT) |

Fonts: Godot's built-in default theme font (Open Sans, Apache-2.0, ships with the
engine). Nothing is bundled or redistributed.

## Why nothing was downloaded

The art direction is a warm, illustrated, ordinary-life isometric town
(`presentation/mineworld-default/2D/references/`). Two searches were run before
any drawing started, and the shared asset research in
`scratchpad/asset-research/FINDINGS.md` reached the same conclusion
independently: **no open-licensed set matching that direction exists.**

Checked and rejected:

- **Kenney — Isometric Tiles City / Isometric Tiles Buildings / Isometric Roads**
  (<https://kenney.nl>, CC0). Downloaded and inspected the preview. These are
  flat-shaded 3D renders of abstract beige blocks: no shopfronts, no awnings,
  no foliage clumps, no characters, and a cool toy palette. Correctly
  non-pixel and very readable, but they would have set the look to
  "block diorama", not "warm storybook street". Usable as grey-box layout
  only, which this spike did not need.
- **Summer Engine "Isometric Vector Buildings"** — repackages Kenney; no new art.
- **itch.io / OpenGameArt isometric CC0 tags** — overwhelmingly pixel art, which
  the direction explicitly excludes, and mixed per-asset licences (CC-BY-SA and
  GPL are unusable in an MIT project).
- **Vecteezy, Freepik** — not open. They require attribution *and* forbid
  redistributing the file, so the art could not live in this repository.

Available if texture or material bases are ever needed, but not needed here
because nothing in this spike is textured: **ambientCG** (<https://ambientcg.com>,
CC0) and **Poly Haven** (<https://polyhaven.com>, CC0, redistribution permitted).

## The palette

Sampled by eye from the four reference plates and fixed once, before any
content was placed, in two matching tables:

- `tools/gen_art.py` → `P` (sprites)
- `scripts/Ground.gd` → `C` (ground)

One warm daylight condition, low contrast, soft contact shadows under every
prop, no night or overcast variant. Anything that did not sit inside that
palette was not drawn.

## Scale

Also fixed mechanically before content, in `scripts/Iso.gd`: 2:1 isometric,
one world unit = 2 m, 1 m = 32 px vertically. A person is 1.75 m ≈ 56 px, a
shop storey is 3.5 m ≈ 112 px, and every per-prop scale in `Main.gd:SCALE` is
derived from that rather than eyeballed per object.
