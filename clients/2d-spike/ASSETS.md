# Assets used by the 2D presentation spike

## Short answer

Almost everything on screen is authored in this repository. One third-party
source is used, and it is CC0.

| Source | What it produces | Licence |
| --- | --- | --- |
| `tools/gen_art.py` (written for this spike) | all 51 SVG sprites in `art/svg/` — buildings, trees, street furniture, boats, people | Part of this repository; same licence as the repository (MIT) |
| `scripts/Ground.gd` (written for this spike) | the ground plane — water, quay, paving, grass — drawn with Godot's own 2D primitives | Part of this repository; same licence as the repository (MIT) |
| `art/grade.gdshader`, `art/ink.gdshader` (written for this spike) | the paper-and-grade pass and the ink contour on every prop | Part of this repository; same licence as the repository (MIT) |
| **PuzzleAndy, "CC0 Watercolor Textures", OpenGameArt** | `art/paper_grain.png` — derived, see below | **CC0 1.0** |

Fonts: Godot's built-in default theme font (Open Sans, Apache-2.0, ships with the
engine). Nothing is bundled or redistributed.

## The one third-party asset

**`art/paper_grain.png`** — 512×512, neutral grey, no colour and no composition.

- **Source:** <https://opengameart.org/content/cc0-watercolor-textures>
- **Author:** PuzzleAndy
- **Licence as stated on the source page:** `CC0`
  (<http://creativecommons.org/publicdomain/zero/1.0/>) — public domain
  dedication. Commercial use, modification, redistribution and bundling are all
  permitted, and no attribution is required. It is recorded here anyway,
  because `DEP-8` asks for provenance regardless of whether the licence
  compels it.
- **Files used:** `watercolor_3`, `watercolor_6`, `watercolor_8`,
  `watercolor_11`.
- **What was done to them:** `tools/make_grain.gd` takes a square centre crop
  of each scan, discards the colour, subtracts a very slightly blurred copy of
  its own luminance to leave only high-frequency detail, clamps the residue so
  ink splatter cannot survive as black specks, averages the four, and
  normalises the result around mid grey. What survives is the tooth of the
  cold-press paper the originals were painted on. The washes, the brushwork
  and the composition are all gone — deliberately, because at the blur radii
  that keep them, they read as smears of dirt over the scene rather than as
  texture in it.
- **Why a real scan rather than noise:** value noise is uniformly random at
  every point. Paper has fibre, direction and clumping, and the eye can tell.
  The shader still uses procedural noise for the broad world-anchored mottle;
  the scan does the fine tooth, which is the part noise cannot fake.
- **Reproducing it:** the source scans are not committed — they are 1–7 MB
  JPEGs and only the derived tile is needed. Download the four files listed
  above into a directory and run:

  ```
  MWGRAIN_SRC=/path/to/scans godot --headless --path clients/2d-spike \
      --script tools/make_grain.gd
  ```

## What was searched, and what was rejected

The art direction is a warm, illustrated, ordinary-life isometric town
(`presentation/mineworld-default/2D/references/`). `docs/DECISIONS.md` `DEP-8`
records an earlier conclusion that no open-licensed set matches it. That
conclusion was re-tested rather than assumed, with a fresh search across
itch.io, OpenGameArt, Kenney, game-icons.net, ambientCG, Poly Haven and the
public-domain illustration archives.

**It holds for every category except textures.** Per category:

- **Trees and foliage** — nothing. The closest is OpenGameArt's *Isometric
  Trees*, which is mostly CC0 but contains one CC-BY-SA file and is in a
  semi-realistic fantasy-RPG style, not a storybook one. *Free Isometric
  Plants-Pack* is CC0 but is low-poly 3D renders. This is the highest-value
  category and the one that fails hardest.
- **Characters** — nothing. CC0 isometric character sets are overwhelmingly
  pixel art, which the direction excludes; none offer modern casual dress or
  the seated pose the references show.
- **Street furniture** — nothing usable in colour. *Withering Systems, City
  Game Tileset* is CC-BY and has the right subjects and projection, but is
  black-and-white pen sketch. game-icons.net's bench is CC-BY but is a
  monochrome pictogram.
- **Shopfront buildings** — nothing. The style matches are paid and
  non-redistributable (Penzilla's *Cozy Isometric RPG Village* is covered by a
  licence that forbids redistributing the source files, so it fails the test
  regardless of how well it fits); the licence matches are pixel art.
- **Textures** — **this is where the earlier conclusion was wrong.** Several
  genuinely usable CC0 options exist, and one of them is now in the repository.
  Also verified but not currently needed: Voxel Core Lab's *16 Hand-Painted
  Watercolor Terrain Textures* (itch.io, CC0 1.0), OpenGameArt's *Handpainted
  Stone Wall Textures* (PamNawi, offered as either CC-BY 4.0 or CC0 — take the
  CC0 option), ambientCG and Poly Haven (both CC0, but photographic scans
  rather than painted).

Rejected on licence irrespective of fit: anything non-commercial, CC-BY-SA or
GPL; Freepik, Vecteezy, Envato and Adobe Stock, which require attribution *and*
forbid redistributing the file, so the art could not live in this repository;
and tilingtextures.com, whose pages state no usage terms at all.

## The palette

Measured, not eyeballed. A k-means over the reference plates gives the two
tables the art is built from:

- the foliage ramp, from the green pixels of plates 01/02/04:
  `#22412C → #365E39 → #527B3C → #739940 → #98B543 → #BFCE4B`
- the neutrals, from their low-saturation pixels:
  `#8D847D`, `#9F9893`, `#BAB1AA`, `#F3E8D6` — the paving in the plates is a
  warm **grey**, not a cream

Both live in two matching tables: `tools/gen_art.py` → `P` (sprites) and
`scripts/Ground.gd` → `C` (ground). One warm daylight condition, low contrast,
soft contact shadows under every prop, no night or overcast variant.

The earlier hand-picked palette ran the foliage from `#4C8A3B` to `#B8E282`
and outlined canopies in a mid green — no dark half at all, which is most of
why the trees read as bright plastic. The plates put roughly 15% of their
foliage pixels below `#365E39`.

## Scale

Fixed mechanically before content, in `scripts/Iso.gd`: 2:1 isometric, one
world unit = 2 m, 1 m = 32 px vertically. A person is 1.75 m ≈ 56 px and a
shop storey is 3.5 m ≈ 112 px.

Vegetation scale is **derived**, not tuned: `Main.gd:VEG_M` gives each sprite a
target height in metres and the draw scale falls out of that against the
sprite's authored pixel height. Nine tree sprites drawn at nine different
pixel heights have to agree about how tall a tree is, and hand-tuned numbers
stop agreeing the moment a tenth is added.
