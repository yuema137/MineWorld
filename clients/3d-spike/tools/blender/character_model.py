#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-2.0-or-later
"""Assemble the MineWorld default character, in Blender, into one rigged GLB.

    blender --background --python character_model.py -- \
        --body <whole-body.glb> --out <dir> [--no-hair]

**Input** is the whole-body CC0 Vitruvian mesh, morphed and rigged by CharMorph
and exported as glTF.  That export is scratchpad-only tooling: CharMorph is
AGPL-3.0 and only its CC0 *output* enters this repository
(`docs/CHARACTER_ASSET_AUDIT.md` §"Re-baking the body").  Everything in this
directory works on the exported mesh and never imports CharMorph.

**Output** is `<dir>/vitruvian.glb`: the trimmed body, the clothes, the hair and
the shoes, all skinned to the same 52-joint `mixamorig:*` armature that
`vitruvian_bonemap.tres` maps onto `SkeletonProfileHumanoid`.

**The order matters and is the point of this file.**

```text
load whole body  →  cut each garment out of it  →  build hair and shoes
                 →  trim the body the clothes now cover  →  clean →  export
```

The body is trimmed *last*.  Upstream shipped it already occlusion-deleted under
its clothing, which is precisely why it could not be re-dressed — there was no
torso under the t-shirt to cut a new garment from.  Carrying the whole 39k-vertex
body through the modelling and deleting the covered faces at the end is what
makes a different wardrobe possible at all.

Textures are a separate concern; see `character_textures.py`.
"""

from __future__ import annotations

import argparse
import os
import sys

import bpy  # pylint: disable=import-error
from mathutils import Vector  # pylint: disable=import-error

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import drape  # noqa: E402  pylint: disable=wrong-import-position
import garments  # noqa: E402  pylint: disable=wrong-import-position
import hair as hair_mod  # noqa: E402  pylint: disable=wrong-import-position
from model_lib import (  # noqa: E402  pylint: disable=wrong-import-position
    bind, dominant_group, load_body, report, transfer_weights,
)


# CharMorph leaves eight colour layers and four UV sets on the mesh from its
# UDIM workflow.  The body export already normalised the one it keeps into the
# per-tile layout the committed textures are baked for; the rest cost ~30 % of
# the file and nothing reads them.  Blender's glTF importer renames UV sets, so
# the survivor is identified as the active one rather than by name.


def clean_layers(obj) -> None:
    me = obj.data
    for attr in [a.name for a in me.color_attributes]:
        me.color_attributes.remove(me.color_attributes[attr])
    keep = me.uv_layers.active.name if me.uv_layers.active else None
    for name in [layer.name for layer in me.uv_layers]:
        if name != keep:
            me.uv_layers.remove(me.uv_layers[name])


# Where freckles sit on this face, as Gaussian blobs in the character's own
# space: the bridge of the nose and both upper cheeks, thinning toward the
# temples.  `CHARACTER_IDENTITY.md` §3 calls them an identity feature at
# portrait framing — "a face without them is a different face" — so they are
# placed against the geometry rather than painted by eye onto a UV sheet.
# Read off this character's own face, not guessed: the nose tip is at
# z = 1.597, the eyes span z = 1.623..1.652 and the mouth 1.555..1.590.
FRECKLE_BLOBS = [
    ((0.000, -0.084, 1.614), 0.019, 1.00),     # bridge and tip of the nose
    ((-0.042, -0.062, 1.607), 0.025, 0.95),    # her right cheek
    ((0.042, -0.062, 1.607), 0.025, 0.95),     # her left cheek
    ((0.000, -0.074, 1.658), 0.015, 0.22),     # a few between the brows
]
# Kept off: the eyeballs and lids, and the lips.  A freckle on an eyelid or a
# lip is not a freckle, it is a blemish, and the first pass put a ring of them
# round both eyes.
FRECKLE_AVOID = [((-0.033, -0.052, 1.638), 0.024), ((0.033, -0.052, 1.638), 0.024),
                 ((0.000, -0.075, 1.572), 0.026)]
FRECKLE_COUNT = 560
FRECKLE_SEED = 20260927


def write_freckles(body, path: str) -> int:
    """Emit the UV positions of the freckles, for `character_textures.py`.

    The texture tool has no geometry, and a freckle placed by eye on a UV sheet
    lands somewhere arbitrary on the face.  Here we have both: for every vertex
    of the face we know where it is in space *and* where it is in the texture,
    so the pattern is defined in space and read off in UV.
    """
    import json          # pylint: disable=import-outside-toplevel
    import math          # pylint: disable=import-outside-toplevel
    import random        # pylint: disable=import-outside-toplevel

    me = body.data
    slot = [m.name for m in me.materials].index("MW_Face")
    lay = me.uv_layers.active
    pts = {}
    for poly in me.polygons:
        if poly.material_index != slot:
            continue
        for li, vi in zip(poly.loop_indices, poly.vertices):
            pts.setdefault(vi, tuple(lay.data[li].uv))

    weighted = []
    for vi, uv in pts.items():
        co = me.vertices[vi].co
        if co.y > -0.02:                      # the front of the face only
            continue
        w = 0.0
        for (cx, cy, cz), sigma, amp in FRECKLE_BLOBS:
            d2 = (co.x - cx) ** 2 + (co.y - cy) ** 2 + (co.z - cz) ** 2
            w += amp * math.exp(-d2 / (2 * sigma * sigma))
        for (ax, ay, az), ar in FRECKLE_AVOID:
            if (co.x - ax) ** 2 + (co.y - ay) ** 2 + (co.z - az) ** 2 < ar * ar:
                w = 0.0
                break
        if w > 0.02:
            weighted.append((w, uv))
    total = sum(w for w, _ in weighted)
    rng = random.Random(FRECKLE_SEED)
    out = []
    for _ in range(FRECKLE_COUNT):
        r = rng.random() * total
        acc = 0.0
        for w, uv in weighted:
            acc += w
            if acc >= r:
                out.append({"u": round(uv[0] + rng.uniform(-0.0016, 0.0016), 5),
                            "v": round(uv[1] + rng.uniform(-0.0016, 0.0016), 5),
                            "r": round(rng.uniform(1.5, 3.4), 2),
                            "a": round(rng.uniform(0.22, 0.52), 3)})
                break
    with open(path, "w", encoding="utf-8") as fh:
        json.dump({"tile": "1001", "seed": FRECKLE_SEED, "points": out}, fh)
    print(f"  freckles: {len(out)} points over {len(weighted)} eligible face verts")
    return len(out)


def add_blink(body, path: str) -> None:
    """A `Blink` shape key on the final body, from the export's lid deltas.

    The rig has no eyelid bones, and a person who never blinks reads as a
    mannequin however good the pose. The export writes CharMorph's L3
    `Eyes_Closed_Left/Right` (CC0) as position + delta -- base-mesh vertex
    indices do not survive the trim here, positions do -- so each lid vertex
    is found again by position. The match count and the worst distance are
    printed, because a blink that silently matched nothing looks exactly like
    a blink that was never added.
    """
    import json          # pylint: disable=import-outside-toplevel
    from mathutils.kdtree import KDTree  # pylint: disable=import-error,import-outside-toplevel

    if not os.path.exists(path):
        print(f"  blink: {path} not found -- no Blink shape key")
        return
    pts = json.load(open(path, encoding="utf-8"))["points"]
    me = body.data
    kd = KDTree(len(me.vertices))
    for v in me.vertices:
        kd.insert(v.co, v.index)
    kd.balance()
    body.shape_key_add(name="Basis", from_mix=False)
    kb = body.shape_key_add(name="Blink", from_mix=False)
    matched, worst, travel = 0, 0.0, 0.0
    for x, y, z, dx, dy, dz in pts:
        co, i, d = kd.find((x, y, z))
        if d > 2e-4:
            continue
        kb.data[i].co = co + Vector((dx, dy, dz))
        matched += 1
        worst = max(worst, d)
        travel = max(travel, Vector((dx, dy, dz)).length)
    print(f"  blink: {matched} of {len(pts)} lid vertices matched "
          f"(worst {worst * 1000:.3f} mm), lid travel up to {travel * 1000:.1f} mm")


def main() -> int:
    argv = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
    ap = argparse.ArgumentParser()
    ap.add_argument("--body", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--no-hair", action="store_true")
    args = ap.parse_args(argv)

    body, arm = load_body(args.body)
    dom = dominant_group(body)
    print(f"body: {len(body.data.vertices)} verts, {len(arm.data.bones)} bones")

    tee = garments.build_tee(body, dom)
    jeans = garments.build_jeans(body, dom)
    # the hoodie is draped onto the body and the two garments under it
    hoodie = garments.build_hoodie(body, dom, arm, colliders=(tee, jeans))
    for o in (body, tee, jeans):
        drape.remove_collider(o)
    made = [
        tee,
        jeans,
        hoodie,
        garments.build_shoes(body, dom),
        garments.build_pack(body, dom, arm, over=hoodie),
    ]
    if not args.no_hair:
        made.append(hair_mod.build_hair(body, dom, arm))

    # Weights first, body trim second: the transfer reads the nearest body
    # surface, and trimming first would hand every covered garment vertex the
    # weights of whatever survives next to the hole.
    for o in made:
        transfer_weights(o, body)
        if o is hoodie:
            print(f"  hoodie: {garments.hem_to_pelvis(o)} verts moved off the thighs onto the pelvis")
        bind(o, arm)
        clean_layers(o)
    garments.trim_body(body, dom)
    clean_layers(body)
    body.name = body.data.name = "Body"
    os.makedirs(args.out, exist_ok=True)
    report([body] + made)
    write_freckles(body, os.path.join(args.out, "freckles.json"))
    add_blink(body, os.path.splitext(args.body)[0] + "_blink.json")

    out = os.path.join(args.out, "vitruvian.glb")
    for o in bpy.data.objects:
        o.select_set(False)
    for o in [body, arm] + made:
        o.select_set(True)
    bpy.context.view_layer.objects.active = body
    bpy.ops.export_scene.gltf(
        filepath=out, export_format="GLB", use_selection=True,
        export_apply=False, export_yup=True, export_materials="EXPORT",
        export_normals=True, export_tangents=False, export_texcoords=True,
        export_skins=True, export_animations=False, export_morph=True,
        export_morph_normal=False)

    # Hair is excluded from the authored height on purpose.  `human.gd` scales
    # an instance by `height_m / CANONICAL_HEIGHT`, and `height_mm` in the world
    # means how tall the person is, not how tall her bun is: measuring the
    # silhouette including 5 cm of gathered hair shrinks every body by that much.
    solid = [body] + [o for o in made if o.name != "Hair"]
    lo = min(min(v.co.z for v in o.data.vertices) for o in solid)
    hi = max(max(v.co.z for v in o.data.vertices) for o in solid)
    with_hair = max(max(v.co.z for v in o.data.vertices) for o in [body] + made)
    print(f"height  {hi - lo:.4f} m   (sole {lo:.4f} .. crown {hi:.4f}; "
          f"hair reaches {with_hair:.4f})")
    print(f"wrote   {out}  {os.path.getsize(out) / 1e6:.2f} MB")
    return 0


if __name__ == "__main__":
    sys.exit(main())
