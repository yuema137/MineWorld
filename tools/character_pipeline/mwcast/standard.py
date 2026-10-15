"""Every gate bound and parity bound of the per-character standard, each a literal with its source.

The standard is ARC-94 (ARC-23 governs how bounds are fixed). No bound is computed from a candidate, and no
gate takes a bound as an argument (I-B1-6). A bound changes only by amending `docs/DECISIONS.md` `ARC-94`,
and never after a Gate run (design §12).
"""

from __future__ import annotations

# Pinned Blender (PIPELINE.md §3, §11). Checked at each run start.
BLENDER_PIN = "Blender 5.2.2 LTS"

# The reference skeleton every character's armature must equal (G-5).
REFERENCE_SKELETON = "clients/3d-spike/assets/characters/vitruvian/vitruvian.glb"

# The accepted asset of the protagonist, the parity reference (AB1-8).
ACCEPTED_ASSET = "clients/3d-spike/assets/characters/meshy_d/meshy_d.glb"

# G-1: triangle count of the one skinned mesh.
MAX_TRIANGLES = 100_000  # D+ §5.2, the game-budget ceiling

# G-2: the one material.
MATERIAL = "MW_Cast"  # step-26 §4.5, G-2; the export renames the material to this

# G-3: base-colour texture size in pixels (square).
TEXTURE_PX = 2048  # D+ §5.2

# G-4: sole and eye heights, metres in glTF Y-up.
SOLE_Z_M = -0.0177  # step-26 G-4; the current character's sole height (D+ gameready_mesh.py:24)
SOLE_TOL_M = 0.005  # step-26 G-4
EYE_RATIO = 0.9366  # step-26 G-4: sole-to-eye-line over stature (1.655 / 1.767)
EYE_TOL_M = 0.010  # step-26 G-4

# G-5: skeleton and weights.
WEIGHT_SUM_TOL = 1e-3  # step-26 G-5: every vertex's weight sum is 1 within this
UNWEIGHTED_SUM = (
    1e-4  # step-26 G-5: a vertex whose weight sum is below this is unweighted; the bound is zero of them
)
ROLL_TOL_RAD = 1e-4  # step-26 G-5: bone rolls within this of the reference's (measured in radians)

# G-6: proportions (Winter) and the inside-the-body check.
WINTER_RATIOS: dict[
    str, float
] = {  # Winter, Biomechanics and Motor Control of Human Movement, 4th ed., Fig. 4.1
    "upper_arm": 0.186,
    "forearm": 0.146,
    "thigh": 0.245,
    "shank": 0.246,
}
WINTER_TOL = 0.12  # step-26 G-6: relative (±12 % of Winter's ratio); the reading is recorded in PIPELINE §6
SLAB_HALF_M = 0.010  # step-26 G-6: half-height of the slab of mesh vertices around a joint
NEIGHBOUR_M = 0.15  # step-26 G-6: horizontal reach of the neighbourhood around a joint

# Joint names carry the Mixamo prefix in the reference skeleton (verified in B3 against vitruvian.glb).
_M = "mixamorig:"

# The trunk and limb joints G-6(b) checks, per step-26 G-6.
BODY_JOINTS: tuple[str, ...] = tuple(
    _M + name
    for name in (
        "Hips",
        "Spine",
        "Spine1",
        "Spine2",
        "Neck",
        "Head",
        "LeftArm",
        "LeftForeArm",
        "LeftHand",
        "LeftUpLeg",
        "LeftLeg",
        "LeftFoot",
        "RightArm",
        "RightForeArm",
        "RightHand",
        "RightUpLeg",
        "RightLeg",
        "RightFoot",
    )
)

# G-6(a): the four limb segments measured, per side: (segment, proximal joint, distal joint).
LIMB_SEGMENTS: tuple[tuple[str, str, str], ...] = (
    ("upper_arm", _M + "LeftArm", _M + "LeftForeArm"),
    ("forearm", _M + "LeftForeArm", _M + "LeftHand"),
    ("thigh", _M + "LeftUpLeg", _M + "LeftLeg"),
    ("shank", _M + "LeftLeg", _M + "LeftFoot"),
    ("upper_arm", _M + "RightArm", _M + "RightForeArm"),
    ("forearm", _M + "RightForeArm", _M + "RightHand"),
    ("thigh", _M + "RightUpLeg", _M + "RightLeg"),
    ("shank", _M + "RightLeg", _M + "RightFoot"),
)

# Parity (design §5.7, AB1-8 and AB1-9): the texel bounds of the base-colour comparison. Positions, joints and
# weights are bounded with the compare command's rows (added with B8).
PARITY_TEXEL_TOLERANCE = 2  # of 255, per channel (design §5.7: "every channel differs by ≤ 2/255")
PARITY_TEXEL_SHARE = (
    0.999  # design §5.7 and AB1-9: the share of texels within the tolerance, same device only
)

# G-14: the provenance keys required, and the AI-identifier key patterns. The patterns are copied from
# scripts/check_secrets.py (AC-S26-9); the two lists are one rule, and a test keeps them equal.
PROVENANCE_KEYS: tuple[str, ...] = (
    "source_type",
    "ai_generated",
    "generator",
    "meshy_task",
    "references",
    "reference_generator",
    "postprocess",
    "pipeline",
)
KEY_PATTERNS: tuple[str, ...] = (
    r"sk-[A-Za-z0-9_-]{20,}",
    r"sk-proj-[A-Za-z0-9_-]{20,}",
    r"msy_[A-Za-z0-9]{20,}",
    r"Bearer [A-Za-z0-9._-]{20,}",
    r"Key-Pair-Id=[A-Z0-9]{8,}",
)
UUID_PATTERN = r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"
SHA256_PATTERN = r"[0-9a-f]{64}"
