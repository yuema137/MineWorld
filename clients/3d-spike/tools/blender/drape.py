#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Cloth simulation, so a garment hangs and folds instead of standing off the body.

Called by `garments.py`.  Frame: +Z up, +X the character's own left, -Y the way
she faces.

**Why.**  The hoodie used to be the body surface pushed out 40 mm along its
normals.  A dilated body has no drape and cannot fold: sleeves of uniform
diameter, a chest standing off the ribs, a hood that was a level roll -- it read
as a padded jacket in both previews, the direction of `VISUAL_FIDELITY.md` §5's
own worked example.  Real garments are cut larger than the body and *hang*:
the shoulders carry the weight, the sleeve falls onto the top of the arm and
gathers at the cuff, the front panels drop, and the fabric folds wherever it
has more length than the body under it.  Blender's cloth solver does exactly
that, and this file drives it headless:

```text
a loose starting shape  ->  pin the seams that are held (collar, cuffs)
                        ->  collide with the body (and anything else given)
                        ->  run under gravity until it settles  ->  freeze
```

The starting shape may be anything with the garment's topology and enough
slack; it is the simulation, not the starting shape, that decides how the
fabric lies.  The result is frozen into the mesh, and the garment is then
skinned by weight transfer from the body like every other garment.

Deterministic: Blender's cloth solver is deterministic for a given mesh,
settings and frame count, so the bake is re-runnable.
"""

from __future__ import annotations

import bpy  # pylint: disable=import-error

# Cotton jersey, in Blender's cloth units.  Soft in bending (jersey has almost
# no bending stiffness, which is what lets it fold), moderately stiff in
# tension so a sleeve does not stretch to the floor under its own weight.
JERSEY = {
    "quality": 10,
    "mass": 0.30,
    "air_damping": 4.0,
    "tension_stiffness": 18.0,
    "compression_stiffness": 18.0,
    "shear_stiffness": 6.0,
    "bending_stiffness": 0.35,
    "tension_damping": 8.0,
    "compression_damping": 8.0,
    "shear_damping": 6.0,
    "bending_damping": 0.6,
}


def add_collider(obj, thickness: float = 0.004, friction: float = 6.0) -> None:
    """Make `obj` something cloth rests on."""
    import os  # pylint: disable=import-outside-toplevel
    if os.environ.get("MW_DRAPE_NOCOLLIDE") == "1":     # diagnostic
        return
    if not any(m.type == "COLLISION" for m in obj.modifiers):
        obj.modifiers.new("Collision", "COLLISION")
    col = obj.collision
    col.thickness_outer = thickness
    col.thickness_inner = 0.002
    col.cloth_friction = friction
    col.damping = 0.6


def remove_collider(obj) -> None:
    for m in [m for m in obj.modifiers if m.type == "COLLISION"]:
        obj.modifiers.remove(m)


def simulate(obj, pin, frames: int = 50, settings=None,
             self_collision: bool = True, shrink: float = 0.0) -> None:
    """Drape `obj` and freeze the result into its mesh.

    `pin(vertex) -> float` is the pin weight per vertex, 0 free to 1 held in
    place.  `shrink` shortens every spring by that fraction, which pulls an
    over-sized garment in toward the body as a real fit would.
    """
    s = dict(JERSEY)
    s.update(settings or {})
    z0 = (min(v.co.z for v in obj.data.vertices), max(v.co.z for v in obj.data.vertices))
    grp = obj.vertex_groups.get("pin") or obj.vertex_groups.new(name="pin")
    held = 0
    for v in obj.data.vertices:
        w = pin(v)
        if w > 0.0:
            grp.add([v.index], min(1.0, w), "REPLACE")
            held += 1
    mod = obj.modifiers.new("Cloth", "CLOTH")
    cs = mod.settings
    for key, val in s.items():
        setattr(cs, key, val)
    cs.vertex_group_mass = "pin"
    cs.pin_stiffness = 1.0
    cs.shrink_min = shrink
    cc = mod.collision_settings
    cc.use_collision = True
    cc.distance_min = 0.003
    cc.collision_quality = 4
    import os  # pylint: disable=import-outside-toplevel
    if os.environ.get("MW_DRAPE_SELF") is not None:     # diagnostic override
        self_collision = os.environ["MW_DRAPE_SELF"] == "1"
    cc.use_self_collision = self_collision
    cc.self_distance_min = 0.0025
    cc.self_friction = 5.0
    scene = bpy.context.scene
    scene.frame_start = 1
    scene.frame_end = frames
    mod.point_cache.frame_start = 1
    mod.point_cache.frame_end = frames
    for f in range(1, frames + 1):
        scene.frame_set(f)
        if f % 10 == 0 or f == frames:
            me = obj.evaluated_get(bpy.context.evaluated_depsgraph_get()).data
            d = [(a.co - b.co).length for a, b in zip(obj.data.vertices, me.vertices)]
            dz = [b.co.z - a.co.z for a, b in zip(obj.data.vertices, me.vertices)]
            print(f"    frame {f}: mean move {sum(d) / len(d) * 1000:.1f} mm, "
                  f"max {max(d) * 1000:.1f} mm, mean dz {sum(dz) / len(dz) * 1000:+.1f} mm")
    # freeze the evaluated mesh, then drop the modifier and the pin group
    dg = bpy.context.evaluated_depsgraph_get()
    ev = obj.evaluated_get(dg)
    frozen = bpy.data.meshes.new_from_object(ev, preserve_all_data_layers=True,
                                             depsgraph=dg)
    moved = sum(1 for a, b in zip(obj.data.vertices, frozen.vertices)
                if (a.co - b.co).length > 0.001)
    old = obj.data
    obj.modifiers.remove(mod)
    obj.data = frozen
    frozen.name = old.name
    bpy.data.meshes.remove(old)
    obj.vertex_groups.remove(obj.vertex_groups["pin"])
    scene.frame_set(1)
    zs = [v.co.z for v in obj.data.vertices]
    print(f"  drape {obj.name}: {len(obj.data.vertices)} verts, {held} pinned, "
          f"{moved} moved > 1 mm over {frames} frames; z {min(zs):.3f}..{max(zs):.3f} "
          f"(was {z0[0]:.3f}..{z0[1]:.3f})")
