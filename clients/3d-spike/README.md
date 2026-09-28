# MineWorld 3D presentation spike

A runnable walking demo, built to answer one question: **does this feel like the
walking, travel and life-simulation experience we want?** Three camera modes,
one movement controller.

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
| `F5` | cycle the camera: first person -> third rear -> third front |
| `Esc` | release / recapture the mouse |
| — | **no jump.** This is a walking simulator. |

The current mode is named in the corner of the screen. `F5` is free here: it is
the Godot *editor's* run shortcut, not a game binding, and the running game has
focus when you press it. Rebind it in `project.godot` under `camera_cycle` if it
clashes with anything on your machine.

## The three cameras

| Mode | |
| --- | --- |
| 1 first person | the eye, at 1.66 m. No visible body -- see below. |
| 2 third person rear | behind and above: the character, where they are going, and the street. Minecraft's rear view in feel. |
| 3 third person front | ahead, looking back: for inspecting the character and taking pictures of them. |

**One movement controller, three cameras observing it.** `scripts/player.gd`
decides where the body is; `scripts/camera_rig.gd` only reads it. Switching mode
calls `make_current()` on one of three cameras that already exist and are
already in position -- nothing is created, freed or reset, so position,
velocity, orientation and collision state carry straight across a switch.
`--drive` measures exactly that (see below) rather than asserting it.

Both third-person cameras ride a boom from a pivot at the character's
shoulders, along the view axis, so looking up swings the camera down and
vice versa. A single ray runs from the pivot to where the camera wants to be and
the camera is pulled in to the first thing it hits, which keeps it out of walls,
buildings and terrain. One ray, deliberately: predictable beats clever.

**Known limitations**, stated plainly:

- First person shows **no body**. That is on purpose. A first-person body is its
  own problem -- mesh culling, animation clipping, the camera inside the head --
  and was explicitly out of scope for this spike.
- Backed hard against a wall, the rear camera pulls in to about 0.6 m and you
  get the back of the character's head filling the frame (screenshot 15). That
  is what one ray and no shoulder offset buys you, and it is honest.
- The character is the NPC mannequin: right proportions, ordinary clothes,
  minimal face. Not a character pipeline. A consequence worth knowing before
  you judge the front view: at full-frame scale the face is too small to read,
  so modes 2 and 3 look alike until you crop in or turn the character. The
  `12b`/`13b` crops and the facing block in `--drive` both exist so that this
  is decidable rather than arguable.
- The mannequin is authored facing its local **+Z**, which is npc.gd's
  convention and the opposite of Godot's. The player controller turns the body
  180 degrees when it attaches it. That compensation is at one call site and
  documented at both ends, but it is a trap for the next consumer.
- Camera distance, height, FOV and mouse sensitivity are first-guess defaults in
  `camera_rig.gd` and `player.gd`, left deliberately untuned.

Two scripted modes, both of which exit on their own:

```sh
./mineworld-3d --drive    # headless movement + collision test, prints results
./mineworld-3d --shots    # re-capture clients/3d-spike/shots/*.png
```

`--drive` walks the real controller through the real input path and prints what
the body actually did: gravity, speeds, strafe, mouse look, walls -- and, for
the cameras, four camera switches mid-stride each measured against an adjacent
identical window with no switch, a switch mid-fall, and the rear boom length
with and without a building behind the player.

## What is here

One lakeside promenade: a nine-unit shop row, a plaza with a fountain on a
promontory, a side street receding south, and a stretch of shore trail with a
jetty. Deliberately compact -- it exists to be judged at three distances, not
to be a town.

- **near** — doors, glazing, stone stallrisers, awnings, planters, sandwich boards
- **medium** — the street, the buildings, NPCs moving through it
- **far** — lake, far shore, treeline, snow-capped ridges

`screenshots/` holds fifteen captured viewpoints: ten of the town (1-10), the
same standing position in each of the three camera modes (11-13), the character
in a wide scenic view (14), and the rear camera pulled in by a wall (15). Two
head-and-shoulders crops (`12b`, `13b`) are cut from frames 12 and 13 by the
same `--shots` run and shown at 2x: the mannequin's face is deliberately
minimal, so at 1600x900 the head is about 60 px and mode 2 and mode 3 are not
reliably distinguishable at a glance. In `12b` the head is featureless; in
`13b` you can see brows, eyes and a nose. `ASSETS.md` records every third-party
file and its licence.

## Shape of the code

The whole scene is built from script at startup, so the look lives in readable
code rather than a `.tscn` blob.

| File | |
| --- | --- |
| `scripts/main.gd` | lighting rig, then the town, then the player |
| `scripts/town.gd` | the scene layout and the buildings |
| `scripts/props.gd` | street furniture and vegetation |
| `scripts/npc.gd` | people, at real proportions |
| `scripts/player.gd` | the walking controller -- the only one -- and the player character |
| `scripts/camera_rig.gd` | the three cameras, and their wall collision |
| `scripts/mats.gd` | the material library |
| `scripts/procgen.gd` | generated leaf / flower / grass cut-outs |
| `scripts/build.gd` | geometry and collision helpers |
| `scripts/shots.gd` | the scripted capture and drive test |

**Presentation only.** No networking, no ActionIntent, no conversation, no
semantics. An NPC here knows where it walks and nothing whatsoever about what
is allowed in the world.
