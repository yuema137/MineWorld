# SPDX-License-Identifier: GPL-3.0-or-later
"""R-5: the 11-region partition of the `hm08` template (pr-s32p-wardrobe-pilot.md §4.3; step-32 §6.1).

The rule is the data's source:

- every template vertex outside MPFB2's `body` group (eyes, teeth, tongue, lashes, joint cubes, the hair, skirt and
  tights helpers) is not drawn and gets region -1;
- a body vertex takes the region of its dominant `mixamo` bone weight: Neck → neck; Spine, Spine1, Spine2 and the
  Shoulders → torso; Arm → upper-arms; ForeArm → forearms; Hand and fingers → hands; Hips → hips; UpLeg → thighs;
  Leg → shins; Foot and ToeBase → feet; Head → face or scalp;
- a Head vertex is **face** when it lies at or below the brow plane (35 mm above the eye joint cubes' centre) and
  in front of the ear line (75 mm behind that centre); otherwise **scalp**.

Built on the neutral template (MPFB2's default macros), with MPFB2's own `mixamo` weights.
Params: {"out": <json path>}.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import mpfbcommon as mc  # noqa: E402

BROW_ABOVE_EYES_M = 0.035
EAR_BEHIND_EYES_M = 0.075


def main():
    p = mc.params()
    mc.clear_scene()
    HumanService, TargetService = mc.mpfb()
    basemesh = HumanService.create_human(macro_detail_dict=TargetService.get_default_macro_info_dict())
    HumanService.add_builtin_rig(basemesh, "mixamo")
    positions, _ = mc.evaluated_positions(basemesh)
    body = set(mc.body_group_indices(basemesh))
    dominant = mc.dominant_bones(basemesh)
    eye_l = mc.joint_cube_centre(basemesh, positions, "joint-l-eye")
    eye_r = mc.joint_cube_centre(basemesh, positions, "joint-r-eye")
    eyes = (eye_l + eye_r) / 2
    regions = []
    counts = {name: 0 for name in mc.REGIONS}
    for index in range(len(basemesh.data.vertices)):
        if index not in body:
            regions.append(-1)
            continue
        region = mc.region_of_bone(dominant[index])
        if region == "head":
            pos = positions[index]
            in_front = pos.y <= eyes.y + EAR_BEHIND_EYES_M  # the character faces -Y in Blender
            below = pos.z <= eyes.z + BROW_ABOVE_EYES_M
            region = "face" if in_front and below else "scalp"
        counts[region] += 1
        regions.append(mc.REGIONS.index(region))
    data = {
        "_rule": __doc__.split("The rule is the data's source:")[1].split("Built on")[0].strip(),
        "source": "MPFB2 2.0.18 hm08 basemesh, default macros, built-in mixamo rig and weights (CC0 data)",
        "vertex_count": len(regions),
        "regions": list(mc.REGIONS),
        "counts": counts,
        "vertex_region": regions,
    }
    Path(p["out"]).write_text(
        mc.json.dumps(data, separators=(",", ":"), sort_keys=True) + "\n", encoding="utf-8"
    )
    mc.log("regions", counts, "helpers", regions.count(-1))


main()
