# The Blender stages (GPL-2.0-or-later)

These scripts are the character pipeline's Blender stages: stage 6 (`gameready.py`), stages 7–8a (`fit_bind.py`),
8b (`weights_fingers.py`), 8c (`weights_pack.py`), 9 (`flecks.py`), 10 (`export.py`), and the review and
armature-dump support (`render_ortho.py`, `render_bones.py`, `dump_rig.py`). Each is a port of a D+ script in
`/Users/yuema137/mineworld-demos/i23d/`, named in its header, and PIPELINE.md §2.3 states the port rule.

They use Blender's Python API (`bpy`, `bmesh`, `mathutils`), so they are distributed under
**GPL-2.0-or-later**. See [`LICENSE`](LICENSE) and the SPDX header in each file.

The GPL applies to these scripts only:

- **The rest of the repository is MIT**, and the driver (`mwcast/`, `cast.py`) never imports these scripts. It only
  runs them, as Blender subprocesses, with a parameters file.
- **The outputs** (GLB files, textures, `.blend` files) are not covered by the scripts' licence. Blender's licence
  page says what you create with Blender is your property.

Run them only through the driver (`cast.py run`), so that the pinned Blender version and the parameters are
recorded. A change to any script in this directory re-runs the local real-lifecycle Gate (PIPELINE.md §11).
