# MineWorld — 2D presentation spike

A walkable slice of one quayside square, built so a human can look at it and
answer one question: **is this MineWorld's default 2D presentation?**

It is a spike. It requires no kernel changes, contains no semantics — no
dialogue, no inventory, no networking, no `ActionIntent` — and invents no world
rules. `scripts/Demo.gd` holds the only data model, and a `DemoPlace` or
`DemoPerson` answers "what do I draw and where" and nothing else.

## Run it

From the repository root:

```
./mineworld-2d
```

Needs Godot 4.4+ on `PATH` (developed against 4.7.2; `brew install godot`).
Set `GODOT=/path/to/godot` if it is not on `PATH`.

**Controls:** `WASD` or the arrow keys to walk, `Esc` to quit. Movement is
free and continuous, not tile-stepped.

Two extra modes, both used to verify the spike:

```
./mineworld-2d --shots    # capture stills into clients/2d-spike/shots/
./mineworld-2d --drive    # press every movement key in turn and report distances
```

`--shots` includes `06_ref_framing`, framed to match
`presentation/mineworld-default/2D/references/02_cafe_street.png` so the two
can be compared like for like. The other stills are all wider than any
reference plate.

## What is in it

One square, bounded on three sides and open to the water:

- four shopfronts — Café, Bakery, Books, Bloom — with striped awnings, lit
  interiors, upper-storey window boxes and legible hanging signs
- a fountain, and paving that rings it
- a quay with iron railings, a jetty, a moored boat and gulls
- a park corner with grass, a curving path, benches and flowerbeds
- café terraces with parasols, tables and two seated people (the `Sit`
  affordance the references show, in 2D form)
- street furniture: lampposts, planters, pots, chalkboard A-frames, a
  wayfinding signpost, bicycles, barrels
- five background people walking fixed routes, plus a dog
- nine tree sprites across six silhouettes, so a foliage belt of dozens of
  trees does not read as one tree stamped forty times
- a player, distinguished by an amber jacket, teal trousers, slightly greater
  height and a warm ring on the ground

## How it is put together

| File | Role |
| --- | --- |
| `scripts/Iso.gd` | the projection and the metre-to-pixel rule, fixed once |
| `tools/gen_art.py` | generates every sprite in `art/svg/` plus `art/props.json` |
| `scripts/Ground.gd` | draws water, quay, paving and grass with 2D primitives |
| `scripts/Shadows.gd` | soft cast shadows and canopy dapple, from one gradient texture |
| `scripts/Grade.gd` | the full-screen paper-and-grade pass |
| `art/grade.gdshader` | paper tooth, colour grade, vignette |
| `art/ink.gdshader` | the dark contour drawn on every prop |
| `art/paper_grain.png` | scanned cold-press paper tooth (CC0 — see `ASSETS.md`) |
| `tools/make_grain.gd` | derives that tile from the CC0 source scans |
| `tools/sheet.gd` | contact sheet of chosen sprites, for looking at art alone |
| `scripts/Demo.gd` | `DemoPlace` / `DemoPerson` — deliberately dumb |
| `scripts/Main.gd` | lays the square out, drives the player and the crowd |

### Where the look comes from

Three things do most of the work, and none of them is an asset:

- **Shading is smooth because it is real.** The generator used to fake every
  ramp with ten stacked bands, assuming ThorVG — which Godot uses to rasterise
  these SVGs — could not do gradients. It can, `stop-opacity` included. Every
  ramp, contact shadow and trunk is now a real `<linearGradient>` or
  `<radialGradient>`, and the ground shades per-slab through
  `draw_polygon`'s per-vertex colours.
- **Surfaces have tooth.** `grade.gdshader` runs a scanned paper grain and a
  colour grade over the finished frame. The coarse mottle is anchored to the
  world so it reads as texture lying on the scene rather than dirt on the
  lens; the fine tooth is anchored to the screen so it reads as paper.
- **Everything is inked.** `ink.gdshader` draws a dark contour inside each
  prop's silhouette. Crop the demo and `references/02_cafe_street.png` to the
  same region at 1:1 and the absence of that line is the loudest difference
  between them — louder than colour, shading or texture.

Foliage is drawn in five passes (dark silhouette, ramped body, leaf stipple,
crevices, rim light) across nine tree sprites in six silhouettes and four
foliage ramps. Flat vector fails at foliage because it has no detail below the
shape scale, and that is the scale the eye reads as paint.

Art is regenerated with `python3 clients/2d-spike/tools/gen_art.py` (no
dependencies beyond the standard library). Sprites are authored at 2x and
drawn down, so edges stay clean. Godot rasterises the SVGs with its built-in
ThorVG.

Depth is a painter's sort: every prop is anchored at the point where it meets
the ground, and the scene root is `y_sort_enabled`.

See `ASSETS.md` for licensing. The short version: one third-party asset, a
CC0 paper-grain scan, and everything else is authored here. That file also
records which asset categories were searched and what was rejected — the
earlier conclusion that nothing open matches this art direction holds for
props, characters and buildings, and is wrong for textures.
