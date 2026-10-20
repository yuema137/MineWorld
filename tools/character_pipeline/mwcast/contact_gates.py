"""The static contact gates C-1…C-4 (step-36 §4.1; pr-s36-p1-contact-module.md).

- **C-1** at rest, no limb goes into the trunk (a fused body GLB, bind pose).
- **C-2** at rest, no body surface pokes through the garment covering it, and no inner layer through an
  outer one (an assembled outfit GLB and its manifest).
- **C-3** a floor-standing prop rests on the floor (commit 4); a `ground` prop sinks exactly as deep as it
  declares (S36-P5 commit 4).
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
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
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
LIMB_GROUPS = ("hand", "sleeve", "upper_arm", "thigh", "leg")

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
        if group not in LIMB_GROUPS or skin.trunk_share[i] > MIX_W:
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


def trunk_patch(skin: Skinning, faces: Ints) -> Ints:
    """The faces whose three vertices are trunk-dominant (Spine, Chest, UpperChest) with limb share ≤
    MIX_W."""
    pure = np.array(
        [bone in TORSO and skin.limb_share[i] <= MIX_W for i, bone in enumerate(skin.dominant)], dtype=bool
    )
    keep = pure[faces].all(axis=1)
    return faces[keep]


def _cannot_look(gate: GateId, looked: list[str], why: str) -> GateResult:
    return GateResult(gate, Verdict.FAIL, tuple(looked), (f"cannot look: {why}",))


def _decide(gate: GateId, looked: list[str], reasons: list[str]) -> GateResult:
    return GateResult(gate, Verdict.PASS if not reasons else Verdict.FAIL, tuple(looked), tuple(reasons))


def limb_in_trunk(
    skin: Skinning, faces: Sequence[Sequence[int]], joints: Mapping[str, Vec3], tolerances: Tolerances
) -> GateResult:
    """C-1 over arrays: each limb query vertex inside the trunk patch (winding number > 0.5) deeper than
    `self.limb_in_trunk` (`self.hand_in_trunk` for the hand) fails. Prints the worst depth per group."""
    looked = [f"tolerances sha256 {tolerances.sha256}"]
    face_array = np.asarray(faces, dtype=np.int64).reshape(-1, 3)
    patch = trunk_patch(skin, face_array)
    looked.append(f"trunk patch {patch.shape[0]} of {face_array.shape[0]} faces")
    queries = limb_queries(skin, joints)
    if patch.shape[0] == 0:
        return _cannot_look(GateId.C1, looked, "no trunk-dominant face (Spine, Chest, UpperChest)")
    if not queries:
        return _cannot_look(GateId.C1, looked, "no limb vertex passes G-9's filters")
    trunk = TriMesh(skin.positions, patch)
    reasons: list[str] = []
    for key, indices in queries.items():
        bound_key = "self.hand_in_trunk" if key.endswith(" hand") else "self.limb_in_trunk"
        bound = tolerances.bound(bound_key)
        found = contact.inside(trunk, skin.positions[indices])
        worst = found.worst_mm
        over = int((found.depths_mm > bound).sum())
        looked.append(
            f"{key} in trunk: {indices.shape[0]} vertices, {found.indices.shape[0]} inside, worst "
            f"{worst:.1f} mm, margin {bound - worst:.1f} mm ({bound_key} {bound:g} mm)"
        )
        if worst > bound:
            reasons.append(f"{key} is {worst:.1f} mm inside the trunk ({over} vertices over {bound:g} mm)")
    return _decide(GateId.C1, looked, reasons)


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
            specs[node] = PartSpec(
                float(layer), tuple(str(c) for c in cast(list[object], covers)), str(entry.get("class", ""))
            )
        return OutfitManifest(tuple(str(r) for r in cast(list[object], regions)), specs)


@dataclass(frozen=True)
class _Layer:
    positions: Floats
    faces: Ints
    region: Ints  # -1 where none
    under: NDArray[np.bool_]  # the assembly's flag: under an outer layer at rest


def _read_layer(glb: Glb, mesh: int, *, need_regions: bool) -> _Layer:
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
    return _Layer(
        np.asarray(positions, dtype=np.float64).reshape(-1, 3),
        np.asarray(faces, dtype=np.int64).reshape(-1, 3),
        np.asarray(region, dtype=np.int64),
        np.asarray(under, dtype=bool),
    )


def gate_c2(glb: Glb, manifest: OutfitManifest, tolerances: Tolerances, body: str = "Body") -> GateResult:
    """C-2 on an assembled outfit at rest, part by part in layer order.

    For each part P that covers regions (hair never covers): the body's vertices in a region P covers, that
    the assembly flags as under an outer layer, must lie inside P (generalized winding number), or outside
    it by at most `garment.poke.body`. Each part Q of a lower layer: its vertices flagged under, in a region
    P covers (a part vertex takes the region of its nearest body vertex), must lie inside P, or outside by
    at most `garment.poke.layer`. The drawn body has lost its hidden set (R-11) to the mask, so a hidden
    vertex is never asked about. Never "anything inside anything"."""
    looked = [f"tolerances sha256 {tolerances.sha256}"]
    meshes = node_meshes(glb)
    missing = sorted(name for name in [body, *manifest.parts] if name not in meshes)
    if missing:
        return _cannot_look(GateId.C2, looked, f"the outfit has no mesh node {missing}")
    try:
        skin = _read_layer(glb, meshes[body], need_regions=True)
        layers = {name: _read_layer(glb, meshes[name], need_regions=False) for name in manifest.parts}
    except GlbError as exc:
        return _cannot_look(GateId.C2, looked, str(exc))
    try:
        return _layer_order(manifest, tolerances, body, skin, layers, looked)
    except ContactUnavailable as exc:
        return _cannot_look(GateId.C2, looked, str(exc))


def _layer_order(
    manifest: OutfitManifest,
    tolerances: Tolerances,
    body: str,
    skin: _Layer,
    layers: Mapping[str, _Layer],
    looked: list[str],
) -> GateResult:
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
        if outer.part_class == "hair" or not outer.covers:
            continue
        covered = {i for i, r in enumerate(manifest.regions) if r in outer.covers}
        shell = TriMesh(layers[outer_name].positions, layers[outer_name].faces)
        inner: list[tuple[str, Floats, Ints, str]] = [
            (body, skin.positions, skin.region, "garment.poke.body"),
        ]
        inner += [
            (name, layers[name].positions, part_region[name], "garment.poke.layer")
            for name in order
            if manifest.parts[name].layer < outer.layer and manifest.parts[name].part_class != "hair"
        ]
        for inner_name, positions, regions, bound_key in inner:
            flags = skin.under if inner_name == body else layers[inner_name].under
            mask = flags & np.isin(regions, sorted(covered))
            indices = np.flatnonzero(mask)
            if indices.size == 0:
                continue
            tested += int(indices.size)
            bound = tolerances.bound(bound_key)
            away = contact.outside(shell, positions[indices])
            worst = away.worst_mm
            looked.append(
                f"{inner_name} inside {outer_name}: {indices.size} vertices, worst outside {worst:.1f} mm, "
                f"margin {bound - worst:.1f} mm ({bound_key} {bound:g} mm)"
            )
            over = away.distances_mm > bound
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

MOUNTS = ("floor", "surface", "wall", "ceiling", "ground")
KNOWN = "known finding: "  # a reason that a recorded finding explains; the command does not fail on it
# A recorded finding still holds while the measured depth stays within this of the recorded one (float noise
# between platforms); a deeper reading is a new failure.
FINDING_SLACK_MM = 0.1


@dataclass(frozen=True)
class GroundRest:
    """A `ground` prop's declaration (Q-S36-P1-2): the plane in its own frame that meets the ground, how far
    the mesh goes below that plane by design, and why."""

    rest_mm: float
    embed_mm: float
    why: str


@dataclass(frozen=True)
class ItemSpec:
    """One prop's entry in the library's `contact.json`."""

    mount: str
    resting: frozenset[tuple[str, str]]
    finding_rest_mm: float | None  # C-3: the recorded floor clearance a fix is owed for
    finding_overlaps: Mapping[tuple[str, str], float]  # C-4: recorded pair depths a fix is owed for
    ground: GroundRest | None = None  # set exactly when mount is "ground"
    walk_through: bool = False  # a person may pass through the prop; it needs no solid (S36-P5)
    # contact-scene: recorded (place, rule) worst measurements a fix is owed for (scene_check.py)
    finding_scene: Mapping[tuple[str, str], float] = field(default_factory=lambda: {})


def _pair(a: str, b: str) -> tuple[str, str]:
    return (a, b) if a <= b else (b, a)


_GROUND_FIELDS = ("rest_mm", "embed_mm", "why")
_FIELDS = {"mount", "resting", "findings", "note", "walk_through", "walk_through_why", *_GROUND_FIELDS}


def _number(path: Path, slug: str, entry: Mapping[str, object], key: str) -> float:
    value = entry.get(key)
    if isinstance(value, bool) or not isinstance(value, int | float) or not math.isfinite(value):
        raise ValueError(f"{path}: {slug} {key} {value!r} is not a number")
    return float(value)


def _reason(path: Path, slug: str, entry: Mapping[str, object], key: str) -> str:
    value = entry.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{path}: {slug} needs '{key}', one sentence saying why")
    return value


def _ground(path: Path, slug: str, mount: str, entry: Mapping[str, object]) -> GroundRest | None:
    given = [key for key in _GROUND_FIELDS if key in entry]
    if mount != "ground":
        if given:
            raise ValueError(f"{path}: {slug} has {given}, which only a 'ground' mount takes")
        return None
    missing = [key for key in _GROUND_FIELDS if key not in entry]
    if missing:
        raise ValueError(f"{path}: {slug} mount 'ground' needs {missing}")
    embed = _number(path, slug, entry, "embed_mm")
    if embed < 0:
        raise ValueError(f"{path}: {slug} embed_mm {embed:g} is negative")
    return GroundRest(_number(path, slug, entry, "rest_mm"), embed, _reason(path, slug, entry, "why"))


def _walk_through(path: Path, slug: str, entry: Mapping[str, object]) -> bool:
    if "walk_through" not in entry:
        if "walk_through_why" in entry:
            raise ValueError(f"{path}: {slug} has walk_through_why without walk_through")
        return False
    if entry["walk_through"] is not True:
        raise ValueError(f"{path}: {slug} walk_through must be true when given")
    _reason(path, slug, entry, "walk_through_why")
    return True


def load_mounts(path: Path) -> dict[str, ItemSpec]:
    """`contact.json`: {"format": 1, "items": {slug: {"mount", "resting": [[node, node]…], "findings":
    {"C-3": mm, "C-4": [[node, node, mm]…], "scene": [[place, rule, mm]…], "lane": "…"}}}}. Unknown mounts and
    fields are refused by name.

    A `ground` mount also takes `rest_mm`, `embed_mm` and `why` (all three required); any mount may take
    `walk_through: true` with its own `walk_through_why` (S36-P5 commit 4)."""
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
        extra = sorted(set(entry) - _FIELDS)
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
        stray = sorted(set(recorded) - {"C-3", "C-4", "scene", "lane"})
        if stray:
            raise ValueError(f"{path}: {slug} findings has unknown field(s) {stray}")
        if "C-3" in recorded:
            rest_mm = float(cast(float, recorded["C-3"]))
        for row in cast(list[object], recorded.get("C-4", [])):
            cells = cast(list[object], row)
            overlap_mm[_pair(str(cells[0]), str(cells[1]))] = float(cast(float, cells[2]))
        scene_mm: dict[tuple[str, str], float] = {}
        for row in cast(list[object], recorded.get("scene", [])):
            cells = cast(list[object], row)
            if not isinstance(row, list) or len(cells) != 3:
                raise ValueError(f"{path}: {slug} scene finding {row!r} is not [place, rule, mm]")
            scene_mm[(str(cells[0]), str(cells[1]))] = float(cast(float, cells[2]))
        specs[slug] = ItemSpec(
            str(mount),
            frozenset(resting),
            rest_mm,
            overlap_mm,
            _ground(path, slug, str(mount), entry),
            _walk_through(path, slug, entry),
            scene_mm,
        )
    return specs


def _finding(reasons: list[str], what: str, measured: float, recorded: float | None) -> None:
    if recorded is not None and measured <= recorded + FINDING_SLACK_MM:
        reasons.append(f"{KNOWN}{what} (recorded {recorded:.1f} mm in contact.json)")
    else:
        reasons.append(what)


def gate_c3(
    slug: str, spec: ItemSpec, parts: Sequence[contact.RigidPart], tolerances: Tolerances
) -> GateResult:
    """C-3: a `floor` prop's lowest vertex lies within `item.rest.floor` of y = 0. A `ground` prop's lowest
    vertex lies `embed_mm` below its rest plane `rest_mm`, within the same bound, so the declaration stays
    true when the asset changes. Other mounts are listed as not floor-standing; their placement is judged
    with the place (`scene_check.py`)."""
    bound = tolerances.bound("item.rest.floor")
    if spec.ground is not None:
        return _ground_c3(slug, spec, spec.ground, parts, bound)
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


def _ground_c3(
    slug: str, spec: ItemSpec, ground: GroundRest, parts: Sequence[contact.RigidPart], bound: float
) -> GateResult:
    if not parts:
        return _cannot_look(GateId.C3, [slug], "the prop has no mesh")
    low = min(contact.clearance(contact.world_vertices(p), 0.0) for p in parts)
    depth = ground.rest_mm - low  # how far the lowest point lies below the rest plane
    error = depth - ground.embed_mm
    looked = [
        f"{slug}: ground mount, rest plane {ground.rest_mm:+.1f} mm, lowest point {depth:.1f} mm below it, "
        f"declared embed {ground.embed_mm:.1f} mm, margin {bound - abs(error):.1f} mm "
        f"(item.rest.floor {bound:g} mm)"
    ]
    reasons: list[str] = []
    if abs(error) > bound:
        side = "deeper" if error > 0 else "shallower"
        _finding(
            reasons,
            f"{slug} sinks {depth:.1f} mm below its rest plane, {abs(error):.1f} mm {side} than its declared "
            f"embed {ground.embed_mm:.1f} mm",
            abs(error),
            spec.finding_rest_mm,
        )
    elif spec.finding_rest_mm is not None:
        reasons.append(f"stale finding: {slug} now matches its ground declaration; remove its C-3 finding")
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
