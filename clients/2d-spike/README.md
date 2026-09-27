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
- a player, distinguished by an amber jacket, teal trousers, slightly greater
  height and a warm ring on the ground

## How it is put together

| File | Role |
| --- | --- |
| `scripts/Iso.gd` | the projection and the metre-to-pixel rule, fixed once |
| `tools/gen_art.py` | generates every sprite in `art/svg/` plus `art/props.json` |
| `scripts/Ground.gd` | draws water, quay, paving and grass with 2D primitives |
| `scripts/Demo.gd` | `DemoPlace` / `DemoPerson` — deliberately dumb |
| `scripts/Main.gd` | lays the square out, drives the player and the crowd |

Art is regenerated with `python3 clients/2d-spike/tools/gen_art.py` (no
dependencies beyond the standard library). Sprites are authored at 2x and
drawn down, so edges stay clean. Godot rasterises the SVGs with its built-in
ThorVG.

Depth is a painter's sort: every prop is anchored at the point where it meets
the ground, and the scene root is `y_sort_enabled`.

See `ASSETS.md` for licensing — the short version is that nothing is
third-party.
