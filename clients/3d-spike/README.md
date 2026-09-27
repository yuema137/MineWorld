# MineWorld 3D presentation spike

A runnable first-person walking demo, built to answer one question: **does this
feel like the walking, travel and life-simulation experience we want?**

```sh
./mineworld-3d            # from the repository root -- walk around it
```

Needs **Godot 4.7** on `PATH` (`brew install godot`). Nothing else. The first
run builds Godot's resource cache automatically and takes about 30 s extra.

| Control | |
| --- | --- |
| `W` / `S` | walk forward / back |
| `A` / `D` | strafe |
| mouse | look (X turns, Y looks up/down) |
| `Shift` | jog |
| `Esc` | release / recapture the mouse |
| — | **no jump.** This is a walking simulator. |

Two scripted modes, both of which exit on their own:

```sh
./mineworld-3d --drive    # headless movement + collision test, prints results
./mineworld-3d --shots    # re-capture clients/3d-spike/shots/*.png
```

## What is here

One lakeside promenade: a nine-unit shop row, a plaza with a fountain on a
promontory, a side street receding south, and a stretch of shore trail with a
jetty. Deliberately compact -- it exists to be judged at three distances, not
to be a town.

- **near** — doors, glazing, stone stallrisers, awnings, planters, sandwich boards
- **medium** — the street, the buildings, NPCs moving through it
- **far** — lake, far shore, treeline, snow-capped ridges

`screenshots/` holds ten captured viewpoints. `ASSETS.md` records every
third-party file and its licence.

## Shape of the code

The whole scene is built from script at startup, so the look lives in readable
code rather than a `.tscn` blob.

| File | |
| --- | --- |
| `scripts/main.gd` | lighting rig, then the town, then the player |
| `scripts/town.gd` | the scene layout and the buildings |
| `scripts/props.gd` | street furniture and vegetation |
| `scripts/npc.gd` | people, at real proportions |
| `scripts/player.gd` | the walking controller |
| `scripts/mats.gd` | the material library |
| `scripts/procgen.gd` | generated leaf / flower / grass cut-outs |
| `scripts/build.gd` | geometry and collision helpers |
| `scripts/shots.gd` | the scripted capture and drive test |

**Presentation only.** No networking, no ActionIntent, no conversation, no
semantics. An NPC here knows where it walks and nothing whatsoever about what
is allowed in the world.
