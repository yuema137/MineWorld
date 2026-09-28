# MineWorld — 2D presentation spike

A walkable slice of one quayside square, built so a human could look at it and
answer one question: **is this MineWorld's default 2D presentation?** It is —
see below.

> ## The default 2D style is `town`
>
> The operator compared four complete scenes and chose `town`; it is the
> default and no longer a candidate. What that means, and what distinguishes
> it, is recorded in [`ARC-13`](../../docs/DECISIONS.md) — read that before
> changing the look.
>
> ```
> ./mineworld-2d                         the default (town)
> ./mineworld-2d --variant=full          generated cast, shopfronts and flora generated here
> ./mineworld-2d --variant=people        generated cast, procedural world
> ./mineworld-2d --variant=procedural    the earlier all-procedural build
> ```
>
> The other three are kept deliberately. They are the demonstration that the
> presentation layer swaps, which is the point of
> [`ARC-11`](../../docs/DECISIONS.md); `--variant=` is the interface, and a
> change that can only be made to `town` is a change made in the wrong place.
> This fixes the *default*, not the style system: a World Pack still ships its
> own Presentation Pack and picks its own style without touching any of this.
>
> **Walk into the café.** It is at the left of the shop row; walk north through
> the doorway and you are inside. No loading screen, no popup, no scene change
> — the room occupies real world coordinates behind the frontage, and the
> frontage lifts away once you are over the threshold.
>
> Stills for all four variants are in `screenshots/`, rendered at one commit
> and at matched framing, including one framed to
> `references/02_cafe_street.png` so the comparison against the plates is like
> for like.

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
free and continuous, not tile-stepped. Add `--variant=<name>` to pick an art
set; see the review box above.

Two extra modes, both used to verify the spike:

```
./mineworld-2d --drive              # numeric checks; headless, ~6 s, exits non-zero on failure
./mineworld-2d --shots              # capture stills into shots/<variant>/  (needs a window)
./mineworld-2d --drive --capture    # the checks, plus stills               (needs a window)
```

`--drive` is the half that can gate CI. It runs with no display and asserts
every motion property, exiting non-zero if any fails:

```
[PASS] step bound on Up               max 0.0800, median 0.0200, bound 0.1200
[PASS] half-second frame is clamped   moved 0.0800, bound 0.1200 (unclamped would be 1.2000)
[PASS] walks inside through the door  entered=true steps=97 max_step=0.0400 gaps=0
[PASS] every walker moved             12 of 12
[PASS] cadence independent of speed   phase/m spread 0.0000 (4.363..4.363)
[PASS] standing still costs no phase  phase +0.000000, moved 0.000000
drive complete: PASS
```

It had to be split to get there: `_save()` awaits
`RenderingServer.frame_post_draw`, and that frame never arrives without a
rendering device, so any harness that screenshots will hang under `--headless`
rather than fail. The measurements now contain no `_save` at all, and the
stills moved behind `--capture`.

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
| `scripts/Interior.gd` | the café's floor, walls, fittings and lamplight |
| `scripts/Main.gd` | lays the square out, drives the player and the crowd |

### Going inside

One café, walkable, in the same world as the square. The room is a rectangle of
ordinary world coordinates behind the frontage; the doorway bridges it to the
plaza, and walkable space is the union of plaza, doorway and room, with the
player sliding along an edge rather than sticking to it. Nothing loads and the
camera never cuts.

What makes it legible is that the café sprite fades out once the player is over
the threshold. The room is always there behind the façade — the façade simply
stops hiding it, which is the roof-lift cutaway isometric games use. The two
near walls are drawn as low stubs so the cut reads as a building opened up
rather than a rug laid on the grass, and the furniture is placed in the
y-sorted world like any other prop, so the player walks in front of and behind
it exactly as with a bench outside.

Indoors there is no sun: the props get a small contact shadow instead of the
long raking one the exterior uses, and the light comes from pools under the
pendants.

### Motion

Simulation time is clamped to `MAX_SIM_DT`, two frames at 60 Hz. Without that,
anything stalling a frame displaces everything by however long the frame took.
The harness was already showing it and it went unread: every key stepped a flat
`0.0200` except the one straight after a PNG encode, which hit `0.2446` — a
quarter of a metre in one frame — and `0.0009` on the recovery frame. A
screenshot was only the trigger; a shader compile or a window drag does the
same. With the clamp, that same PNG hitch now measures `0.0800`.

The bound the harness asserts is an absolute `MAX_FRAME_STEP`, deliberately not
derived from `MAX_SIM_DT`: the first version of that check computed its own
expectation from the constant under test, so raising `MAX_SIM_DT` to 99 raised
the bound to 237 and the assertion passed while the player crossed 2.4 m in one
frame. It is also stated as an absolute rather than as a multiple of the median,
because a clamped hitch legitimately is four times a 120 Hz median step and
that is not the defect.

Gait is a function of **distance travelled**, never of the clock. Phase
advances by PI per stride (`STRIDE_M`, 0.72 m), so one cycle is two steps and
a person walking twice as fast steps twice as often, with the same stride
length. Driving the cycle from `delta` instead — which is what this did
before — makes cadence a function of frame rate and nothing else; measured
across the crowd, phase per metre ranged from 4.4 to 9.8 depending only on how
fast each character happened to walk, and that mismatch is what reads as
skating. `--drive` reports the number:

```
a      speed 0.80  travelled 2.61 m  phase + 11.40  phase/m  4.363
dog    speed 1.25  travelled 4.06 m  phase + 17.73  phase/m  4.363
h      speed 0.48  travelled 1.57 m  phase +  6.84  phase/m  4.363
idle player: phase +0.000000  moved 0.000000
```

The same constant at every speed, and nothing at all when standing still.

Each character has two stride poses per view — four sprites, `front`,
`front_b`, `back`, `back_b` — and the pose alternates every half cycle. A bob
alone was tried first and looked at: at 56 px tall it is sub-pixel and two
frames half a stride apart were indistinguishable, so the legs had to actually
move. Both poses of a pair are normalized with the *same* scale factor and
onto the same canvas, or the character pumps vertically as a lifted heel
shrinks its own bounding box.

Characters with only one pose still work — the alternation is skipped and the
bob runs, so the gait degrades rather than breaking.

### Calibrating against the plates

The plates are **bright pictures with dark accents**, and an early pass on this
branch got that backwards. Measuring luminance over whole frames rather than
sampling the palette extremes is what showed it:

| | mean | p50 | >0.7 | <0.2 |
| --- | --- | --- | --- | --- |
| `references/02_cafe_street.png` | 0.517 | 0.512 | 24.2% | 4.4% |
| that early build | 0.455 | 0.465 | 15.3% | 17.5% |
| `--variant=town`, same framing as the plate | 0.513 | 0.498 | 19.9% | 4.0% |

Four times too much dark mass and a third too few highlights. The individual
findings behind it were sound — the foliage ramp really had no dark half, and
nothing really was inked — but the fixes stacked, and ink plus a darker ramp
plus warm-grey paving plus tooth plus a grade dropped the whole key. The
correction was to overall value, not to any one of those features:
`tools/luma.gd` reports the distribution, and it is the instrument to use for
this, where the earlier k-means was the right instrument for hue and the wrong
one for key.

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
