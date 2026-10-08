# Blender scripts (GPL-2.0-or-later)

The scripts in this directory build the default character inside Blender, for example
`blender --background --python character_model.py -- …`. They use Blender's Python API (`bpy`,
`bmesh`, `mathutils`), and Blender is licensed under the GPL, so these scripts are distributed
under **GPL-2.0-or-later**. See [`LICENSE`](LICENSE) and the SPDX header in each file.

The GPL applies to these scripts only:

- **The rest of the repository is MIT** (see the top-level `LICENSE`).
- **The assets the scripts produce** (GLBs, textures) are not covered by the scripts' licence.
  Their licences are recorded in `clients/3d-spike/ASSETS.md` and `presentation/*/LICENSES/`.

Nothing outside this directory imports these scripts. They import only each other, from this
directory.
