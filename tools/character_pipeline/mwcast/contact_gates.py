"""The static contact gates C-1…C-4 (step-36 §4.1; pr-s36-p1-contact-module.md).

- **C-1** at rest, no limb goes into the trunk (a fused body GLB, bind pose).
- **C-2** at rest, no body surface pokes through the garment covering it, and no inner layer through an
  outer one (an assembled outfit GLB and its manifest).
- **C-3** a floor-standing prop rests on the floor (commit 4).
- **C-4** no two parts of one prop interpenetrate (commit 4).

Each gate fills `looked_at` with every pair it tested, its vertex count and its worst margin, and then
decides. Every bound comes from the tolerance file (`tolerances.py`, I-S36-2); the geometry is asked through
`contact.py` (I-S36-1). A gate that cannot look reports FAIL with the reason (I-S36-4). No gate reads a
body's, a garment's or a prop's name to decide anything (I-S36-5): bodies and outfits come by path, parts by
the manifest.
"""

from __future__ import annotations

import json
import math
from collections.abc import Container, Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import cast

import numpy as np
from numpy.typing import NDArray

from mwcast import contact
from mwcast.contact import ContactUnavailable, TriMesh
from mwcast.gates import GateId, GateResult, Verdict
from mwcast.glb import (
    Glb,
    GlbError,
    joint_world_positions,
    node_meshes,
    primitive_attribute,
    primitive_counts,
    primitive_positions,
    skinned_primitive_index,
    triangles,
    vertex_weights,
)
from mwcast.tolerances import Tolerances

Floats = NDArray[np.float64]
Ints = NDArray[np.int64]
Vec3 = tuple[float, float, float]

# --- which vertices to ask about: G-9's filters, ported from godot/pose_sheet.gd `_group` / `_checks` -------
# These choose the query vertices; they are not bounds (the bounds are the tolerance file's).

TORSO = ("Spine", "Chest", "UpperChest")
# A vertex with more than this weight share on the other side of the trunk-limb seam is seam, and tested as
# neither (pose_sheet.gd MIX_W).
MIX_W = 0.05
# How far from its bone segment a vertex may lie and still be that limb's surface, metres (pose_sheet.gd
# LIMB_R).
LIMB_R = {"upper_arm": 0.09, "sleeve": 0.08, "hand": 0.08, "thigh": 0.12, "leg": 0.10}
# The proximal cut-off along each segment: the upper arm and the thigh meet the trunk at the shoulder and
# the hip by construction, so only their distal part is asked about (pose_sheet.gd `_checks`, seg[3]).
PROXIMAL_T = {"upper_arm": 0.5, "sleeve": 0.0, "hand": 0.0, "thigh": 0.35, "leg": 0.0}
HAND_REACH_M = 0.18  # the hand's segment runs this far beyond the wrist, along the forearm
NECK_MARGIN_M = 0.05  # the trunk range ends this far below the Neck joint
# The arms' core range starts this far above the higher knee joint, where the thigh's faces end (NECK_MARGIN_M
# mirrored at the knee; PR S36-CC10 §4). A filter, not a bound.
KNEE_MARGIN_M = 0.05
LIMB_GROUPS = ("hand", "sleeve", "upper_arm", "thigh", "leg")
ARM_GROUPS = ("hand", "sleeve", "upper_arm")
# The body core's lower part: the bones whose faces join the trunk band for the arms' test (PR S36-CC10).
CORE_LOWER = ("Hips", "LeftUpperLeg", "RightUpperLeg")

_MIXAMO = {
    "Spine1": "Chest",
    "Spine2": "UpperChest",
    "LeftArm": "LeftUpperArm",
    "RightArm": "RightUpperArm",
    "LeftForeArm": "LeftLowerArm",
    "RightForeArm": "RightLowerArm",
    "LeftUpLeg": "LeftUpperLeg",
    "RightUpLeg": "RightUpperLeg",
    "LeftLeg": "LeftLowerLeg",
    "RightLeg": "RightLowerLeg",
    "LeftToeBase": "LeftToes",
    "RightToeBase": "RightToes",
}


def humanoid(bone: str) -> str:
    """A bone's Godot humanoid name (the retarget the client and the pose sheet use; `*_bonemap.tres`). A
    `mixamorig:` (or `mixamorig_`) name is mapped; a humanoid name is returned unchanged."""
    for prefix in ("mixamorig:", "mixamorig_"):
        if bone.startswith(prefix):
            bare = bone[len(prefix) :]
            if bare in _MIXAMO:
                return _MIXAMO[bare]
            # LeftHandIndex1 -> LeftIndex1: the finger keeps its name; Pinky is the humanoid Little
            for side in ("Left", "Right"):
                if bare.startswith(side + "Hand") and len(bare) > len(side + "Hand"):
                    return side + bare[len(side + "Hand") :].replace("Pinky", "Little")
            return bare
    return bone


def bone_group(bone: str) -> str:
    """The G-9 group of a humanoid bone: torso, hand, sleeve, upper_arm, thigh, leg, or "" (pose_sheet.gd
    `_group`)."""
    if bone in TORSO:
        return "torso"
    if any(finger in bone for finger in ("Thumb", "Index", "Middle", "Ring", "Little")):
        return "hand"
    if bone.endswith("Hand"):
        return "hand"
    if bone.endswith("LowerArm"):
        return "sleeve"
    if bone.endswith("UpperArm"):
        return "upper_arm"
    if bone.endswith("UpperLeg"):
        return "thigh"
    if bone.endswith(("LowerLeg", "Foot", "Toes")):
        return "leg"
    return ""


@dataclass(frozen=True)
class Skinning:
    """Per vertex: its position, its dominant bone (humanoid name), and its trunk and limb weight shares."""

    positions: Floats
    dominant: tuple[str, ...]
    trunk_share: Floats
    limb_share: Floats

    @staticmethod
    def of(positions: Sequence[Sequence[float]], weights: Sequence[Mapping[str, float]]) -> Skinning:
        if len(positions) != len(weights):
            raise ValueError(f"{len(positions)} positions for {len(weights)} weight rows")
        dominant: list[str] = []
        trunk: list[float] = []
        limb: list[float] = []
        for row in weights:
            named = {humanoid(bone): 0.0 for bone in row}
            for bone, weight in row.items():
                named[humanoid(bone)] += weight
            dominant.append(max(named, key=lambda b: named[b]) if named else "")
            trunk.append(sum(w for b, w in named.items() if b in TORSO))
            limb.append(sum(w for b, w in named.items() if bone_group(b) not in ("", "torso")))
        return Skinning(
            np.asarray(positions, dtype=np.float64).reshape(-1, 3),
            tuple(dominant),
            np.asarray(trunk, dtype=np.float64),
            np.asarray(limb, dtype=np.float64),
        )


def _side(bone: str) -> str:
    return "Left" if bone.startswith("Left") else "Right"


def _trunk_range(joints: Mapping[str, Vec3]) -> tuple[Floats, Floats, float, float]:
    hips = np.asarray(joints["Hips"], dtype=np.float64)
    neck = np.asarray(joints["Neck"], dtype=np.float64)
    spine = np.asarray(joints["Spine"], dtype=np.float64)
    up = (neck - hips) / max(float(np.linalg.norm(neck - hips)), 1e-9)
    return hips, up, float((spine - hips) @ up), float((neck - hips) @ up) - NECK_MARGIN_M


def limb_queries(skin: Skinning, joints: Mapping[str, Vec3]) -> dict[str, Ints]:
    """G-9's query vertices, per side and group ("Left hand", "Right thigh", …): dominated by the group's
    bones, trunk share at most MIX_W, distal of the segment's cut-off, within the segment's radius, and
    between the Spine joint and 5 cm below the Neck along Hips→Neck. Joints are keyed by humanoid name.

    The same function filters P2's dumped frames (step-36 §4.2)."""
    hips, up, low, high = _trunk_range(joints)
    return _queries(skin, joints, LIMB_GROUPS, hips, up, low, high)


def arm_queries(skin: Skinning, joints: Mapping[str, Vec3]) -> dict[str, Ints]:
    """The arm groups of `limb_queries` (hand, sleeve, distal upper arm) over the core range: along Hips→Neck
    from `KNEE_MARGIN_M` above the higher knee joint to `NECK_MARGIN_M` below the Neck, so a hand hanging at
    hip or thigh height is asked about (PR S36-CC10 F-CC10-2)."""
    hips, up, _low, high = _trunk_range(joints)
    knee = max(
        float((np.asarray(joints[side + "LowerLeg"], dtype=np.float64) - hips) @ up)
        for side in ("Left", "Right")
    )
    return _queries(skin, joints, ARM_GROUPS, hips, up, knee + KNEE_MARGIN_M, high)


def _queries(
    skin: Skinning,
    joints: Mapping[str, Vec3],
    groups: Sequence[str],
    hips: Floats,
    up: Floats,
    low: float,
    high: float,
) -> dict[str, Ints]:
    segments: dict[str, tuple[Floats, Floats]] = {}
    for side in ("Left", "Right"):
        at = {k: np.asarray(joints[side + k], dtype=np.float64) for k in _SEGMENT_JOINTS}
        wrist, elbow = at["Hand"], at["LowerArm"]
        reach = (wrist - elbow) / max(float(np.linalg.norm(wrist - elbow)), 1e-9) * HAND_REACH_M
        segments[side + " upper_arm"] = (at["UpperArm"], elbow)
        segments[side + " sleeve"] = (elbow, wrist)
        segments[side + " hand"] = (wrist, wrist + reach)
        segments[side + " thigh"] = (at["UpperLeg"], at["LowerLeg"])
        segments[side + " leg"] = (at["LowerLeg"], at["Foot"])
    chosen: dict[str, list[int]] = {}
    points = skin.positions
    for i, bone in enumerate(skin.dominant):
        group = bone_group(bone)
        if group not in groups or skin.trunk_share[i] > MIX_W:
            continue
        key = f"{_side(bone)} {group}"
        a0, a1 = segments[key]
        axis = a1 - a0
        t = float((points[i] - a0) @ axis) / max(float(axis @ axis), 1e-12)
        if t < PROXIMAL_T[group]:
            continue
        offset: Floats = points[i] - (a0 + axis * min(max(t, 0.0), 1.0))
        if math.sqrt(float(offset @ offset)) > LIMB_R[group]:
            continue
        height = float((points[i] - hips) @ up)
        if height < low or height > high:
            continue
        chosen.setdefault(key, []).append(i)
    return {key: np.asarray(found, dtype=np.int64) for key, found in sorted(chosen.items())}


_SEGMENT_JOINTS = ("UpperArm", "LowerArm", "Hand", "UpperLeg", "LowerLeg", "Foot")


# The trunk band: the one torso surface every limb-trunk test builds (C-1 at rest, G-9w per frame, and the
# arms' body core; PR S36-CC10 F-CC10-7, invariant I-CC10-4). A face may carry up to this limb weight share.
# P1's first band kept only pure trunk faces (limb share <= 0.05); on a fused body that band is open at the
# flanks, where the arm's weights blend into the trunk (generalized winding number 0.18-0.44 at the
# protagonist's trunk axis, under the 0.5 threshold), so an arm forced through the flank was never "inside"
# it. Up to this share closes the flank; adding the Hips, the Neck or the shoulders instead wraps the hem and
# the deltoid round a hanging arm (measured on the protagonist, wes-calloway and the planted arm, S36-P2 §8).
SHELL_LIMB_SHARE = 0.25


def _band_vertices(skin: Skinning) -> NDArray[np.bool_]:
    return np.array(
        [bone in TORSO and skin.limb_share[i] <= SHELL_LIMB_SHARE for i, bone in enumerate(skin.dominant)],
        dtype=bool,
    )


def trunk_shell(skin: Skinning, faces: Ints) -> Ints:
    """The trunk band: the faces whose three vertices are trunk-dominant (Spine, Chest, UpperChest) with a
    limb share of at most `SHELL_LIMB_SHARE`."""
    return faces[_band_vertices(skin)[faces].all(axis=1)]


def core_shell(skin: Skinning, faces: Ints) -> Ints:
    """The body core the arms are tested against (PR S36-CC10 §4): the trunk band, plus the faces whose
    vertices are dominated by the Hips or an upper leg. A face is kept when each of its three vertices is a
    band vertex or such a vertex, so the seam faces between the trunk and the Hips close the surface rather
    than leaving a ring open at the waist. On a layered outfit `faces` is every mesh's: the union."""
    lower = np.array([bone in CORE_LOWER for bone in skin.dominant], dtype=bool)
    return faces[(_band_vertices(skin) | lower)[faces].all(axis=1)]


def _thigh_faces(skin: Skinning, faces: Ints) -> NDArray[np.bool_]:
    """Per face: is it the thigh's (at least two of its vertices dominated by an upper leg)?"""
    thigh = np.array([bone_group(bone) == "thigh" for bone in skin.dominant], dtype=bool)
    return thigh[faces].sum(axis=1) >= 2


def _cannot_look(gate: GateId, looked: list[str], why: str) -> GateResult:
    return GateResult(gate, Verdict.FAIL, tuple(looked), (f"cannot look: {why}",))


def _decide(gate: GateId, looked: list[str], reasons: list[str]) -> GateResult:
    return GateResult(gate, Verdict.PASS if not reasons else Verdict.FAIL, tuple(looked), tuple(reasons))


@dataclass(frozen=True)
class LimbDepth:
    """One limb group's reading against the trunk band: how many query vertices, how many inside, the worst
    depth, and how many exceed the group's bound."""

    key: str  # "Left hand", "Right thigh", …
    bound_key: str
    bound_mm: float
    queried: int
    inside: int
    worst_mm: float
    over: int
    surface: str = "trunk"  # "trunk" (the band) or "core" (the arms' band, Hips and thighs)

    @property
    def passed(self) -> bool:
        return self.worst_mm <= self.bound_mm

    def line(self) -> str:
        return (
            f"{self.key} in {self.surface}: {self.queried} vertices, {self.inside} inside, "
            f"worst {self.worst_mm:.1f} mm, margin {self.bound_mm - self.worst_mm:.1f} mm "
            f"({self.bound_key} {self.bound_mm:g} mm)"
        )

    def reason(self) -> str:
        return (
            f"{self.key} is {self.worst_mm:.1f} mm inside the {self.surface} "
            f"({self.over} vertices over {self.bound_mm:g} mm)"
        )


class NoLook(Exception):
    """The limb–trunk test has nothing to test: no trunk band, or no limb query vertex."""


def limb_depths(
    skin: Skinning,
    faces: Ints,
    joints: Mapping[str, Vec3],
    tolerances: Tolerances,
    keep: NDArray[np.bool_] | None = None,
) -> tuple[int, list[LimbDepth]]:
    """The trunk band's face count and each limb group's depth inside it (winding number > 0.5), bounded by
    `self.limb_in_trunk` (`self.hand_in_trunk` for the hand). `keep`, when given, drops the query vertices it
    marks False (G-9w's outermost-layer rule). Raises `NoLook` when there is nothing to test."""
    patch = trunk_shell(skin, faces)
    queries = limb_queries(skin, joints)
    if keep is not None:
        queries = {k: v[keep[v]] for k, v in queries.items() if bool(keep[v].any())}
    if patch.shape[0] == 0:
        raise NoLook("no trunk-dominant face (Spine, Chest, UpperChest)")
    if not queries:
        raise NoLook("no limb vertex passes G-9's filters")
    trunk = TriMesh(skin.positions, patch)
    depths: list[LimbDepth] = []
    for key, indices in queries.items():
        bound_key = "self.hand_in_trunk" if key.endswith(" hand") else "self.limb_in_trunk"
        bound = tolerances.bound(bound_key)
        found = contact.inside(trunk, skin.positions[indices])
        over = int((found.depths_mm > bound).sum())
        depths.append(
            LimbDepth(
                key,
                bound_key,
                bound,
                int(indices.shape[0]),
                int(found.indices.shape[0]),
                found.worst_mm,
                over,
            )
        )
    return int(patch.shape[0]), depths


def arm_depths(
    skin: Skinning,
    faces: Ints,
    joints: Mapping[str, Vec3],
    tolerances: Tolerances,
    keep: NDArray[np.bool_] | None = None,
) -> list[LimbDepth]:
    """Each arm group's depth inside the body core (`core_shell`): a vertex is inside when its winding number
    exceeds 0.5 and the pseudonormal sign agrees (`contact.inside(confirm_sign=True)`). Each inside vertex is
    bounded by the region of its closest core face: a thigh face by `self.arm_in_thigh`, a trunk or Hips face
    by `self.hand_in_trunk` for the hand and `self.limb_in_trunk` otherwise. The group's reading is its vertex
    furthest over its own bound (I-CC10-1). `keep` is G-9w's outermost-layer rule. Raises `NoLook`."""
    core = core_shell(skin, faces)
    queries = arm_queries(skin, joints)
    if keep is not None:
        queries = {k: v[keep[v]] for k, v in queries.items() if bool(keep[v].any())}
    if core.shape[0] == 0:
        raise NoLook("no body-core face (Spine, Chest, UpperChest, Hips, UpperLeg)")
    if not queries:
        raise NoLook("no arm vertex passes G-9's filters")
    surface = TriMesh(skin.positions, core)
    thigh = _thigh_faces(skin, core)
    depths: list[LimbDepth] = []
    for key, indices in queries.items():
        trunk_key = "self.hand_in_trunk" if key.endswith(" hand") else "self.limb_in_trunk"
        found = contact.inside(surface, skin.positions[indices], confirm_sign=True)
        keys = np.where(thigh[found.faces], "self.arm_in_thigh", trunk_key)
        bounds = np.array([tolerances.bound(str(k)) for k in keys], dtype=np.float64)
        excess = found.depths_mm - bounds
        if excess.size:
            at = int(np.argmax(excess))
            bound_key, bound, worst = str(keys[at]), float(bounds[at]), float(found.depths_mm[at])
        else:
            bound_key, bound, worst = trunk_key, tolerances.bound(trunk_key), 0.0
        depths.append(
            LimbDepth(
                key,
                bound_key,
                bound,
                int(indices.shape[0]),
                int(found.indices.shape[0]),
                worst,
                int((excess > 0).sum()),
                "core",
            )
        )
    return depths


def limb_in_trunk(
    skin: Skinning, faces: Sequence[Sequence[int]], joints: Mapping[str, Vec3], tolerances: Tolerances
) -> GateResult:
    """C-1 over arrays: each limb query vertex inside the trunk band (winding number > 0.5) deeper than
    `self.limb_in_trunk` (`self.hand_in_trunk` for the hand) fails. Prints the worst depth per group."""
    looked = [f"tolerances sha256 {tolerances.sha256}"]
    face_array = np.asarray(faces, dtype=np.int64).reshape(-1, 3)
    try:
        patch, depths = limb_depths(skin, face_array, joints, tolerances)
    except NoLook as exc:
        return _cannot_look(GateId.C1, looked, str(exc))
    looked.append(f"trunk band {patch} of {face_array.shape[0]} faces")
    looked += [d.line() for d in depths]
    return _decide(GateId.C1, looked, [d.reason() for d in depths if not d.passed])


def gate_c1(glb: Glb, tolerances: Tolerances) -> GateResult:
    """C-1 on a fused body GLB at rest: the first skinned primitive, its weights and the bind-pose joints."""
    try:
        mesh, primitive = skinned_primitive_index(glb)
        skin = Skinning.of(primitive_positions(glb, mesh, primitive), vertex_weights(glb))
        faces = triangles(glb, mesh, primitive)
        joints = {humanoid(name): at for name, at in joint_world_positions(glb).items()}
    except (GlbError, ValueError) as exc:
        return _cannot_look(GateId.C1, [str(glb.path)], str(exc))
    missing = [j for j in ("Hips", "Spine", "Neck") if j not in joints] + [
        side + j for side in ("Left", "Right") for j in _SEGMENT_JOINTS if side + j not in joints
    ]
    if missing:
        return _cannot_look(GateId.C1, [str(glb.path)], f"the skeleton has no joint {missing}")
    try:
        return limb_in_trunk(skin, faces, joints, tolerances)
    except ContactUnavailable as exc:
        return _cannot_look(GateId.C1, [str(glb.path)], str(exc))


# --- C-2: layer order ---------------------------------------------------------------------------------------


@dataclass(frozen=True)
class PartSpec:
    layer: float
    covers: tuple[str, ...]
    part_class: str
    rigid: bool = False  # weights "rigid <bone>": the part sits on the body (a bag), it is not a shell
    # the covered regions the part does not enclose (an open front, a skirt's hem, a brim): the recipe's
    # `open_regions` (S36-P3b, Q-S36-P3-3), judged by the drape rule instead of the winding number
    open_regions: tuple[str, ...] = ()


@dataclass(frozen=True)
class OutfitManifest:
    """`outfit_manifest.json` as `wardrobe assemble` writes it: the region names (index = TEXCOORD_1's `u`)
    and each part node's layer, covered regions and class."""

    regions: tuple[str, ...]
    parts: Mapping[str, PartSpec]

    @staticmethod
    def load(path: Path) -> OutfitManifest:
        try:
            document = json.loads(path.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise ValueError(f"{path}: {exc}") from exc
        if not isinstance(document, dict):
            raise ValueError(f"{path}: the manifest is not an object")
        top = cast(dict[str, object], document)
        regions = top.get("regions")
        parts = top.get("parts")
        if not isinstance(regions, list) or not isinstance(parts, dict):
            raise ValueError(f"{path}: the manifest needs 'regions' (a list) and 'parts' (an object)")
        specs: dict[str, PartSpec] = {}
        for node, raw in cast(dict[str, object], parts).items():
            if not isinstance(raw, dict):
                raise ValueError(f"{path}: part {node} is not an object")
            entry = cast(dict[str, object], raw)
            layer = entry.get("layer")
            covers = entry.get("covers", [])
            if isinstance(layer, bool) or not isinstance(layer, int | float) or not isinstance(covers, list):
                raise ValueError(f"{path}: part {node} needs a numeric 'layer' and a 'covers' list")
            rigid = entry.get("rigid", False)
            if not isinstance(rigid, bool):
                raise ValueError(f"{path}: part {node}: 'rigid' must be true or false")
            opened = entry.get("open", [])
            if not isinstance(opened, list):
                raise ValueError(f"{path}: part {node}: 'open' must be a list of regions")
            covered = tuple(str(c) for c in cast(list[object], covers))
            open_regions = tuple(str(c) for c in cast(list[object], opened))
            if not set(open_regions) <= set(covered):
                raise ValueError(f"{path}: part {node}: an 'open' region is not in 'covers'")
            specs[node] = PartSpec(float(layer), covered, str(entry.get("class", "")), rigid, open_regions)
        return OutfitManifest(tuple(str(r) for r in cast(list[object], regions)), specs)


@dataclass(frozen=True)
class LayerArrays:
    """One mesh of an outfit as arrays: positions (metres), faces, each vertex's region index (-1 where none)
    and under flag. C-2 reads them from the GLB at rest, W-4w from each frame of the dump, and the push-out's
    tests build them directly."""

    positions: Floats
    faces: Ints
    region: Ints  # -1 where none
    under: NDArray[np.bool_]  # the assembly's flag: under an outer layer at rest


def _read_layer(glb: Glb, mesh: int, *, need_regions: bool) -> LayerArrays:
    positions: list[tuple[float, float, float]] = []
    faces: list[tuple[int, int, int]] = []
    region: list[int] = []
    under: list[bool] = []
    for primitive in range(primitive_counts(glb, mesh)):
        base = len(positions)
        here = primitive_positions(glb, mesh, primitive)
        positions.extend(here)
        faces.extend((a + base, b + base, c + base) for a, b, c in triangles(glb, mesh, primitive))
        uv2 = primitive_attribute(glb, mesh, primitive, "TEXCOORD_1")
        if uv2 is None:
            if need_regions:
                raise GlbError(f"{glb.path}: mesh {mesh} has no TEXCOORD_1 (the assembly's region channel)")
            region.extend([-1] * len(here))
            under.extend([False] * len(here))
            continue
        # u = region index + 0.5 (parts: -0.5); Blender's v = 1.5 "under" arrives as glTF v = -0.5
        region.extend(math.floor(uv[0]) for uv in uv2)
        under.extend(uv[1] < 0.0 for uv in uv2)
    return LayerArrays(
        np.asarray(positions, dtype=np.float64).reshape(-1, 3),
        np.asarray(faces, dtype=np.int64).reshape(-1, 3),
        np.asarray(region, dtype=np.int64),
        np.asarray(under, dtype=bool),
    )


BODY_CHECK = "BodyCheck"
PART_CHECK = "PartCheck_"


def uncut(node: str) -> str:
    """The name of a drawn mesh's uncut copy (S36-P3b, Q-S36-P3-4): `Body` → `BodyCheck`, `part_<id>` →
    `PartCheck_<id>`. The assembly's finish phase masks covered skin and cuts an inner part's faces under
    outer parts; the copies keep the shells as they stood before, so a poke under an outer layer stays
    visible."""
    if node == "Body":
        return BODY_CHECK
    if node.startswith("part_"):
        return PART_CHECK + node[len("part_") :]
    return node


def shell_names(available: Container[str], names: Sequence[str]) -> tuple[dict[str, str], str]:
    """For each drawn mesh name, the mesh to judge: its uncut copy when every name has one in `available`,
    else the drawn mesh itself; and a `looked_at` line saying which."""
    if all(uncut(n) in available for n in names):
        return {n: uncut(n) for n in names}, "shells: uncut (BodyCheck, PartCheck_<id>)"
    return {n: n for n in names}, "shells: drawn (the outfit has no uncut copies)"


def gate_c2(glb: Glb, manifest: OutfitManifest, tolerances: Tolerances, body: str = "Body") -> GateResult:
    """C-2 on an assembled outfit at rest, part by part in layer order, on the uncut shells (`uncut`) when the
    outfit carries them.

    For each part P that covers regions (hair and rigid parts never cover): the body's vertices in a region
    P covers that lie under P (`beneath`), must lie inside P (generalized winding number), or outside it by
    at most `garment.poke.body`. Each part Q of a lower layer: its vertices under P in a region P covers (a
    part vertex takes the region of its nearest body vertex) must lie inside P, or outside by at most
    `garment.poke.layer`. "Under P" is asked of P itself (S36-P3 F-P3-5), not read from the assembly's
    one-bit flag, which says "under some outer part": skin under a hoodie's hem but above the jeans' waist
    is not the jeans' to enclose. On the uncut shells the body keeps the skin the mask hides (R-11) and an
    inner part keeps the faces cut under outer parts, so a poke under an outer layer is asked about (S36-P3b);
    on the drawn shells it is not. In a region P covers but does not enclose (its manifest `open` list, the
    recipe's `open_regions`), "inside P" is the drape rule (`drape_depth`: on P's inner side) instead of the
    winding number. Never "anything inside anything"."""
    looked = [f"tolerances sha256 {tolerances.sha256}"]
    meshes = node_meshes(glb)
    missing = sorted(name for name in [body, *manifest.parts] if name not in meshes)
    if missing:
        return _cannot_look(GateId.C2, looked, f"the outfit has no mesh node {missing}")
    use, which = shell_names(meshes, [body, *manifest.parts])
    looked.append(which)
    try:
        skin = _read_layer(glb, meshes[use[body]], need_regions=True)
        layers = {name: _read_layer(glb, meshes[use[name]], need_regions=False) for name in manifest.parts}
    except GlbError as exc:
        return _cannot_look(GateId.C2, looked, str(exc))
    try:
        return layer_order(manifest, tolerances, body, skin, layers, looked)
    except ContactUnavailable as exc:
        return _cannot_look(GateId.C2, looked, str(exc))


# Which vertices of an inner surface lie under a covering part (S36-P3 F-P3-5). The assembly's rule for
# "under" (`assemble.py`'s region channel, `COVER_RAY_M`) is a ray along the vertex's outward normal meeting
# the part within 50 mm, started 0.5 mm out. A vertex poking through the part has the part behind it, so the
# inward ray meeting the part within the push-out's reach (40 mm, L-b's SINK_MAX_M) also counts: without it
# C-2 could never see a poke.
UNDER_RAY_M = 0.05
UNDER_START_M = 0.0005
POKE_RAY_M = 0.04
EDGE_M = 1e-4  # 0.1 mm: a closest point this near an open edge lies on it


def open_edges(shell: TriMesh, weld_m: float = 1e-6) -> Floats:
    """The shell's open edges (hems, cuffs, necklines, holes) as (k, 2, 3) segments in metres, after welding
    coincident vertices (a fused download repeats a vertex per UV island, which is not an opening)."""
    key = np.round(shell.vertices / weld_m).astype(np.int64)
    _, first, welded = np.unique(key, axis=0, return_index=True, return_inverse=True)
    faces = welded.reshape(-1)[shell.faces]
    edges = np.sort(np.concatenate([faces[:, [0, 1]], faces[:, [1, 2]], faces[:, [2, 0]]]), axis=1)
    edges = edges[edges[:, 0] != edges[:, 1]]
    unique, count = np.unique(edges, axis=0, return_counts=True)
    places = shell.vertices[first]
    return np.ascontiguousarray(places[unique[count == 1]])


def _past_edge(shell: TriMesh, points: Floats) -> NDArray[np.bool_]:
    """Whether each point's closest point on `shell` lies on an open edge: the point is beyond the shell's
    extent (skin below a hem), not under it."""
    segments = open_edges(shell)
    if segments.shape[0] == 0 or points.shape[0] == 0:
        return np.zeros(points.shape[0], dtype=bool)
    near = contact.closest(shell, points).points
    a, b = segments[:, 0], segments[:, 1]
    edge = b - a
    length = np.maximum(np.einsum("ij,ij->i", edge, edge), 1e-18)
    past = np.zeros(points.shape[0], dtype=bool)
    for start in range(0, points.shape[0], 256):  # (256 x edges) at a time keeps memory bounded
        chunk = near[start : start + 256]
        t = np.clip(np.einsum("pkj,kj->pk", chunk[:, None, :] - a[None], edge) / length, 0.0, 1.0)
        foot: Floats = a[None] + t[..., None] * edge[None]
        gap: Floats = np.linalg.norm(chunk[:, None, :] - foot, axis=2)
        past[start : start + 256] = gap.min(axis=1) < EDGE_M
    return past


def beneath(shell: TriMesh, surface: TriMesh, candidates: Ints) -> NDArray[np.bool_]:
    """For each candidate vertex of `surface`: whether it lies under `shell` (the rule above), and its
    closest point on the shell is not on an open edge (skin past a hem is not under the garment)."""
    if candidates.size == 0:
        return np.zeros(0, dtype=bool)
    normals = contact.vertex_normals(surface)[candidates]
    at = surface.vertices[candidates]
    out = contact.ray_hits(shell, at + normals * UNDER_START_M, normals, UNDER_RAY_M)
    back = contact.ray_hits(shell, at - normals * UNDER_START_M, -normals, POKE_RAY_M)
    hit = out | back
    hit[np.flatnonzero(hit)[_past_edge(shell, at[hit])]] = False
    return hit


def face_normals(mesh: TriMesh) -> Floats:
    """Each face's unit normal, by its vertex order (a degenerate face gets the zero vector)."""
    a, b, c = (mesh.vertices[mesh.faces[:, k]] for k in range(3))
    normals = np.cross(b - a, c - a)
    length = np.linalg.norm(normals, axis=1, keepdims=True)
    return np.asarray(normals / np.where(length > 0.0, length, 1.0), dtype=np.float64)


def drape_depth(shell: TriMesh, points: Floats, normals: Floats) -> tuple[Floats, Floats, Ints]:
    """The drape rule (S36-P3b, Q-S36-P3-3): how far each point lies on the outer side of an open `shell`, in
    mm (negative: on its inner side). `normals` are the points' own outward normals on the surface they belong
    to (the skin, an inner layer). An open shape (an open front, a skirt's hem, a brim) has no inside, so its
    winding number says nothing; but it has two sides, and what it covers lies on its inner side. The side
    is the shell's face normal at the closest point, turned to agree with the point's own normal, so neither
    the shell's winding nor its face order matters. C-2 and rule 5 count a vertex in an open region as poking
    only when the winding number also reads it as outside (`_away`). Returns the depths, the oriented face
    normals and the closest faces."""
    if points.shape[0] == 0:
        empty = np.zeros((0, 3), dtype=np.float64)
        return np.zeros(0, dtype=np.float64), empty, np.zeros(0, dtype=np.int64)
    found = contact.closest(shell, points)
    side = face_normals(shell)[found.faces]
    side = np.where((np.einsum("ij,ij->i", side, normals) < 0.0)[:, None], -side, side)
    depth = np.einsum("ij,ij->i", points - found.points, side) * 1000.0
    return np.asarray(depth, dtype=np.float64), np.ascontiguousarray(side), found.faces


def _away(
    shell: TriMesh, surface: TriMesh, indices: Ints, draped: NDArray[np.bool_]
) -> tuple[contact.Outside, int]:
    """C-2's "outside the shell" for the vertices `indices` of `surface`: the winding number where the shell
    encloses them; where `draped` (an open region of the shell), outside by the winding number **and** on the
    shell's outer side by the drape rule, at the smaller of the two distances. An open jacket still encloses
    the back it wraps, and drapes the chest it leaves open: either answer is "under the part". Returns the
    vertices outside (indexing `indices`) with their distances in mm, and how many were judged by the drape
    rule."""
    closed = np.flatnonzero(~draped)
    opened = np.flatnonzero(draped)
    enclosed = contact.outside(shell, surface.vertices[indices[closed]])
    rows = [closed[enclosed.indices]]
    depths = [enclosed.distances_mm]
    if opened.size:
        normals = contact.vertex_normals(surface)[indices[opened]]
        points = surface.vertices[indices[opened]]
        depth, _side, _faces = drape_depth(shell, points, normals)
        found = contact.closest(shell, points)
        out = (found.winding <= contact.INSIDE_THRESHOLD) & (depth > 0.0)
        rows.append(opened[out])
        depths.append(np.minimum(depth, found.distance_mm)[out])
    return contact.Outside(np.concatenate(rows).astype(np.int64), np.concatenate(depths)), int(opened.size)


def layer_order(
    manifest: OutfitManifest,
    tolerances: Tolerances,
    body: str,
    skin: LayerArrays,
    layers: Mapping[str, LayerArrays],
    looked: list[str],
) -> GateResult:
    """C-2's rule on arrays (see `gate_c2`): the body `skin` and each part's `layers`, keyed by node name."""

    def region_name(index: int) -> str:
        return manifest.regions[index] if 0 <= index < len(manifest.regions) else f"region {index}"

    # a part vertex's region is its nearest body vertex's (the assembly writes regions on the body only)
    part_region = {
        name: skin.region[contact.nearest(layer.positions, skin.positions)] for name, layer in layers.items()
    }
    order = sorted(manifest.parts, key=lambda n: (manifest.parts[n].layer, n))
    reasons: list[str] = []
    tested = 0
    for outer_name in order:
        outer = manifest.parts[outer_name]
        if outer.part_class == "hair" or outer.rigid or not outer.covers:
            continue  # hair hangs, and a rigid part (a bag) sits on the body: neither is a shell around it
        covered = {i for i, r in enumerate(manifest.regions) if r in outer.covers}
        opened = sorted(i for i, r in enumerate(manifest.regions) if r in outer.open_regions)
        shell = TriMesh(layers[outer_name].positions, layers[outer_name].faces)
        inner: list[tuple[str, Ints, str]] = [(body, skin.region, "garment.poke.body")]
        inner += [
            (name, part_region[name], "garment.poke.layer")
            for name in order
            if manifest.parts[name].layer < outer.layer and manifest.parts[name].part_class != "hair"
        ]
        for inner_name, regions, bound_key in inner:
            source = skin if inner_name == body else layers[inner_name]
            surface = TriMesh(source.positions, source.faces)
            in_covered = np.flatnonzero(np.isin(regions, sorted(covered)))
            indices = in_covered[beneath(shell, surface, in_covered)]
            if indices.size == 0:
                continue
            tested += int(indices.size)
            bound = tolerances.bound(bound_key)
            away, draped = _away(shell, surface, indices, np.isin(regions[indices], opened))
            worst = away.worst_mm
            over = away.distances_mm > bound
            rule = f" ({draped} by the drape rule)" if draped else ""
            looked.append(
                f"{inner_name} inside {outer_name}: {indices.size} vertices{rule}, "
                f"{int(over.sum())} past the bound, worst outside {worst:.1f} mm, "
                f"margin {bound - worst:.1f} mm ({bound_key} {bound:g} mm)"
            )
            for region_index in sorted({int(r) for r in regions[indices[away.indices[over]]]}):
                in_region = regions[indices[away.indices]] == region_index
                depth = float(away.distances_mm[in_region & over].max())
                reasons.append(
                    f"{inner_name} pokes {depth:.1f} mm through {outer_name} in {region_name(region_index)} "
                    f"({bound_key} {bound:g} mm)"
                )
    if tested == 0:
        return _cannot_look(GateId.C2, looked, "no vertex lies under a covering part in a covered region")
    return _decide(GateId.C2, looked, reasons)


def run_body_gates(glb: Glb, tolerances: Tolerances, manifest: OutfitManifest | None) -> list[GateResult]:
    """C-1 on every body; C-2 when an outfit manifest is given."""
    results = [gate_c1(glb, tolerances)]
    if manifest is not None:
        results.append(gate_c2(glb, manifest, tolerances))
    return results


# --- C-3 and C-4: items -------------------------------------------------------------------------------------

MOUNTS = ("floor", "surface", "wall", "ceiling")
KNOWN = "known finding: "  # a reason that a recorded finding explains; the command does not fail on it
# A recorded finding still holds while the measured depth stays within this of the recorded one (float noise
# between platforms); a deeper reading is a new failure.
FINDING_SLACK_MM = 0.1


@dataclass(frozen=True)
class ItemSpec:
    """One prop's entry in the library's `contact.json`."""

    mount: str
    resting: frozenset[tuple[str, str]]
    finding_rest_mm: float | None  # C-3: the recorded floor clearance a fix is owed for
    finding_overlaps: Mapping[tuple[str, str], float]  # C-4: recorded pair depths a fix is owed for


def _pair(a: str, b: str) -> tuple[str, str]:
    return (a, b) if a <= b else (b, a)


def load_mounts(path: Path) -> dict[str, ItemSpec]:
    """`contact.json`: {"format": 1, "items": {slug: {"mount", "resting": [[node, node]…], "findings":
    {"C-3": mm, "C-4": [[node, node, mm]…], "lane": "…"}}}}. Unknown mounts and fields are refused by name."""
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"{path}: {exc}") from exc
    if not isinstance(document, dict) or cast(dict[str, object], document).get("format") != 1:
        raise ValueError(f"{path}: expected an object with format 1")
    items = cast(dict[str, object], document).get("items")
    if not isinstance(items, dict):
        raise ValueError(f"{path}: 'items' is not an object")
    specs: dict[str, ItemSpec] = {}
    for slug, raw in cast(dict[str, object], items).items():
        if not isinstance(raw, dict):
            raise ValueError(f"{path}: {slug} is not an object")
        entry = cast(dict[str, object], raw)
        extra = sorted(set(entry) - {"mount", "resting", "findings", "note"})
        if extra:
            raise ValueError(f"{path}: {slug} has unknown field(s) {extra}")
        mount = entry.get("mount")
        if mount not in MOUNTS:
            raise ValueError(f"{path}: {slug} mount {mount!r} is not one of {list(MOUNTS)}")
        resting: set[tuple[str, str]] = set()
        for pair in cast(list[object], entry.get("resting", [])):
            if not isinstance(pair, list) or len(cast(list[object], pair)) != 2:
                raise ValueError(f"{path}: {slug} resting entry {pair!r} is not a pair of node names")
            a, b = (str(n) for n in cast(list[object], pair))
            resting.add(_pair(a, b))
        rest_mm: float | None = None
        overlap_mm: dict[tuple[str, str], float] = {}
        findings = entry.get("findings", {})
        if not isinstance(findings, dict):
            raise ValueError(f"{path}: {slug} findings is not an object")
        recorded = cast(dict[str, object], findings)
        stray = sorted(set(recorded) - {"C-3", "C-4", "lane"})
        if stray:
            raise ValueError(f"{path}: {slug} findings has unknown field(s) {stray}")
        if "C-3" in recorded:
            rest_mm = float(cast(float, recorded["C-3"]))
        for row in cast(list[object], recorded.get("C-4", [])):
            cells = cast(list[object], row)
            overlap_mm[_pair(str(cells[0]), str(cells[1]))] = float(cast(float, cells[2]))
        specs[slug] = ItemSpec(str(mount), frozenset(resting), rest_mm, overlap_mm)
    return specs


def _finding(reasons: list[str], what: str, measured: float, recorded: float | None) -> None:
    if recorded is not None and measured <= recorded + FINDING_SLACK_MM:
        reasons.append(f"{KNOWN}{what} (recorded {recorded:.1f} mm in contact.json)")
    else:
        reasons.append(what)


def gate_c3(
    slug: str, spec: ItemSpec, parts: Sequence[contact.RigidPart], tolerances: Tolerances
) -> GateResult:
    """C-3: a `floor` prop's lowest vertex lies within `item.rest.floor` of y = 0. Other mounts are listed as
    not floor-standing; their placement is judged with the place (step-36 P5)."""
    bound = tolerances.bound("item.rest.floor")
    if spec.mount != "floor":
        return _decide(GateId.C3, [f"{slug}: mount {spec.mount}, not floor-standing (not judged here)"], [])
    if not parts:
        return _cannot_look(GateId.C3, [slug], "the prop has no mesh")
    low = min(contact.clearance(contact.world_vertices(p), 0.0) for p in parts)
    looked = [
        f"{slug}: lowest point {low:+.1f} mm from the floor, margin {bound - abs(low):.1f} mm "
        f"(item.rest.floor {bound:g} mm)"
    ]
    reasons: list[str] = []
    if abs(low) > bound:
        side = "floats" if low > 0 else "sinks"
        _finding(reasons, f"{slug} {side} {abs(low):.1f} mm", abs(low), spec.finding_rest_mm)
    elif spec.finding_rest_mm is not None:
        reasons.append(f"stale finding: {slug} now rests within {bound:g} mm; remove its C-3 finding")
    return _decide(GateId.C3, looked, reasons)


def gate_c4(
    slug: str, spec: ItemSpec, parts: Sequence[contact.RigidPart], tolerances: Tolerances
) -> GateResult:
    """C-4: no two mesh nodes of one prop interpenetrate deeper than `item.overlap.parts`. A pair declared
    `resting` may touch; it still fails beyond the bound."""
    bound = tolerances.bound("item.overlap.parts")
    names = {p.name for p in parts}
    unknown = sorted(n for pair in spec.resting for n in pair if n not in names)
    if unknown:
        return _cannot_look(GateId.C4, [slug], f"a resting pair names a missing node {unknown}")
    found = contact.overlaps(parts)
    looked = [
        f"{slug}: {len(parts)} mesh nodes, {len(parts) * (len(parts) - 1) // 2} pairs, "
        f"{len(found)} in contact (item.overlap.parts {bound:g} mm)"
    ]
    reasons: list[str] = []
    for hit in found:
        key = _pair(hit.a, hit.b)
        kind = "resting" if key in spec.resting else "touching"
        looked.append(
            f"{slug}: {hit.a} / {hit.b} {kind}, depth {hit.depth_mm:.1f} mm, "
            f"margin {bound - hit.depth_mm:.1f} mm"
        )
        if hit.depth_mm > bound:
            _finding(
                reasons,
                f"{slug}: {hit.a} and {hit.b} interpenetrate {hit.depth_mm:.1f} mm",
                hit.depth_mm,
                spec.finding_overlaps.get(key),
            )
    for key, recorded in sorted(spec.finding_overlaps.items()):
        depth = next((h.depth_mm for h in found if _pair(h.a, h.b) == key), 0.0)
        if depth <= bound:
            reasons.append(
                f"stale finding: {slug} {key[0]} / {key[1]} is now {depth:.1f} mm (recorded "
                f"{recorded:.1f} mm); remove it from contact.json"
            )
    return _decide(GateId.C4, looked, reasons)


def prop_files(root: Path) -> dict[str, Path]:
    """The prop library by directory listing: `<root>/<slug>/<slug>.gltf` (or `.glb`) per directory."""
    found: dict[str, Path] = {}
    for directory in sorted(p for p in root.iterdir() if p.is_dir()):
        for suffix in (".gltf", ".glb"):
            candidate = directory / f"{directory.name}{suffix}"
            if candidate.is_file():
                found[directory.name] = candidate
                break
    return found


def run_item_gates(root: Path, tolerances: Tolerances) -> dict[str, list[GateResult]]:
    """C-3 and C-4 over every prop under `root`, with mounts from `<root>/contact.json`. A directory without
    an entry fails C-3 "no mount declared"; an entry without a directory fails too."""
    mounts = load_mounts(root / "contact.json")
    files = prop_files(root)
    directories = sorted(p.name for p in root.iterdir() if p.is_dir())
    results: dict[str, list[GateResult]] = {}
    for slug in directories:
        if slug not in mounts:
            results[slug] = [_cannot_look(GateId.C3, [slug], "no mount declared in contact.json")]
            continue
        if slug not in files:
            results[slug] = [
                _cannot_look(GateId.C3, [slug], f"no {slug}.gltf or {slug}.glb in its directory")
            ]
            continue
        try:
            parts = contact.scene_parts(files[slug])
        except (ContactUnavailable, ValueError, OSError) as exc:
            results[slug] = [_cannot_look(GateId.C3, [slug], str(exc))]
            continue
        results[slug] = [
            gate_c3(slug, mounts[slug], parts, tolerances),
            gate_c4(slug, mounts[slug], parts, tolerances),
        ]
    for slug in sorted(set(mounts) - set(directories)):
        results[slug] = [_cannot_look(GateId.C3, [slug], "declared in contact.json, but no such directory")]
    return results
