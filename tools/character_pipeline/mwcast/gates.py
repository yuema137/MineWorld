"""The static gates G-1…G-6 and G-14 (PIPELINE.md §6; ARC-94).

Each gate fills `looked_at` (what it read, before its verdict) and then decides. Bounds come only from
`standard.py` (I-B1-6). A gate that cannot look reports FAIL with the reason; it never passes by default.
"""

from __future__ import annotations

import hashlib
import json
import math
import re
from dataclasses import dataclass
from enum import Enum
from pathlib import Path
from typing import cast

from mwcast import standard
from mwcast.glb import (
    Glb,
    GlbError,
    attribute_names,
    base_colour_image_index,
    embedded_image,
    joint_parents,
    joint_world_positions,
    material_documents,
    png_size,
    positions,
    primitive_count,
    read_glb,
    scene_extras,
    skin_joint_names,
    skinned_mesh_count,
    texture_slots,
    triangle_count,
    vertex_weights,
)
from mwcast.records import FitParams


class GateId(Enum):
    G1 = "G-1"
    G2 = "G-2"
    G3 = "G-3"
    G4 = "G-4"
    G5 = "G-5"
    G6 = "G-6"
    G14 = "G-14"


class Verdict(Enum):
    PASS = "PASS"
    FAIL = "FAIL"


@dataclass(frozen=True)
class GateResult:
    gate: GateId
    verdict: Verdict
    looked_at: tuple[str, ...]
    reasons: tuple[str, ...]


def _result(gate: GateId, looked_at: list[str], reasons: list[str]) -> GateResult:
    verdict = Verdict.PASS if not reasons else Verdict.FAIL
    return GateResult(gate, verdict, tuple(looked_at), tuple(reasons))


def gate_g1(glb: Glb) -> GateResult:
    """G-1: the triangle count of the one skinned mesh is at most MAX_TRIANGLES, and there is exactly one."""
    triangles = triangle_count(glb)
    skinned = skinned_mesh_count(glb)
    looked = [f"triangles {triangles}", f"skinned meshes {skinned}"]
    reasons: list[str] = []
    if skinned != 1:
        reasons.append(f"expected exactly one skinned mesh, found {skinned}")
    if triangles > standard.MAX_TRIANGLES:
        reasons.append(f"triangles {triangles} exceed MAX_TRIANGLES {standard.MAX_TRIANGLES}")
    return _result(GateId.G1, looked, reasons)


def gate_g2(glb: Glb) -> GateResult:
    """G-2: exactly one material, named MATERIAL, with a base-colour texture and no other texture slot; one
    primitive."""
    names = [str(m.get("name", "")) for m in material_documents(glb)]
    primitives = primitive_count(glb)
    looked = [f"materials {names}", f"primitives {primitives}"]
    reasons: list[str] = []
    if len(names) != 1 or names[0] != standard.MATERIAL:
        reasons.append(f"material {', '.join(names) or '(none)'}; expected exactly one, {standard.MATERIAL}")
    if primitives != 1:
        reasons.append(f"expected one primitive, found {primitives}")
    if names:
        slots = texture_slots(glb, 0)
        looked.append(f"texture slots {slots}")
        if not slots["base colour"]:
            reasons.append("the material has no base-colour texture")
        for slot in ("normal", "occlusion", "emissive", "metallic-roughness"):
            if slots[slot]:
                reasons.append(f"the material carries a {slot} texture")
    return _result(GateId.G2, looked, reasons)


def gate_g3(glb: Glb) -> GateResult:
    """G-3: the base-colour image is TEXTURE_PX square, 8-bit RGB or RGBA.

    The compression mode is printed, not judged (Q-S26-5 is S26-f's)."""
    reasons: list[str] = []
    looked: list[str] = []
    try:
        index = base_colour_image_index(glb, 0)
        data = embedded_image(glb, index)
    except GlbError as exc:
        return _result(GateId.G3, ["base-colour image: not readable"], [f"no base-colour image: {exc}"])
    width, height, depth, colour = png_size(data)
    looked.append(f"base-colour image {width}×{height}, bit depth {depth}, colour type {colour}, embedded")
    looked.append("external .import: not applicable (embedded PNG)")
    if (width, height) != (standard.TEXTURE_PX, standard.TEXTURE_PX):
        reasons.append(
            f"base colour is {width}×{height}; expected {standard.TEXTURE_PX}×{standard.TEXTURE_PX}"
        )
    if depth != 8 or colour not in (2, 6):
        reasons.append(f"base colour must be 8-bit RGB or RGBA; found depth {depth}, colour type {colour}")
    return _result(GateId.G3, looked, reasons)


def _read_json_object(path: Path) -> dict[str, object]:
    value: object = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise ValueError(f"{path}: expected a JSON object")
    return cast(dict[str, object], value)


def gate_g4(glb: Glb, fit: FitParams, run_dir: Path | None) -> GateResult:
    """G-4: (a) the lowest glTF y lies within SOLE_TOL_M of SOLE_Z_M.

    (b) The sole-to-eye height, from the eye line through stage 6's transform (`run_dir/gameready.json`), lies
    within EYE_TOL_M of stature × EYE_RATIO."""
    ys = [p[1] for p in positions(glb)]
    lowest = min(ys)
    looked = [f"lowest glTF y {lowest:.5f}"]
    reasons: list[str] = []
    if abs(lowest - standard.SOLE_Z_M) > standard.SOLE_TOL_M:
        reasons.append(f"soles at y {lowest:.5f}; expected {standard.SOLE_Z_M} within {standard.SOLE_TOL_M}")
    if run_dir is None:
        reasons.append("G-4(b) needs a run directory (gameready.json); it is not judged without one")
    else:
        game = _read_json_object(run_dir / "gameready.json")
        scale = game.get("scale")
        sole_raw = game.get("sole_raw")
        if not isinstance(scale, float | int) or not isinstance(sole_raw, float | int):
            raise ValueError(f"{run_dir / 'gameready.json'}: scale and sole_raw must be numbers")
        eye_from_sole = (fit.gameready.eye_z_raw - float(sole_raw)) * float(scale)
        target = fit.stature_m * standard.EYE_RATIO
        looked.append(f"eye line raw {fit.gameready.eye_z_raw}, sole raw {sole_raw}, scale {scale}")
        looked.append(f"sole-to-eye {eye_from_sole:.5f} m; target {target:.5f} m")
        if abs(eye_from_sole - target) > standard.EYE_TOL_M:
            reasons.append(
                f"sole-to-eye {eye_from_sole:.5f} m is outside {standard.EYE_TOL_M} of {target:.5f} m"
            )
    return _result(GateId.G4, looked, reasons)


def gate_g5(glb: Glb, repo_root: Path, run_dir: Path | None) -> GateResult:
    """G-5: joint names and parents equal the reference skeleton; one JOINTS_0/WEIGHTS_0 set.

    Every weight sum is within WEIGHT_SUM_TOL of 1, and no vertex is unweighted.
    Rolls come from `dump_rig.json`
    when a run directory is given, within ROLL_TOL_RAD of `reference_dump_rig.json`."""
    reference = read_glb(repo_root / standard.REFERENCE_SKELETON)
    names = skin_joint_names(glb)
    reference_names = skin_joint_names(reference)
    parents = joint_parents(glb)
    reference_parents = joint_parents(reference)
    looked = [f"joints {len(names)} (reference {len(reference_names)})"]
    reasons: list[str] = []
    missing = sorted(set(reference_names) - set(names))
    unexpected = sorted(set(names) - set(reference_names))
    if missing or unexpected:
        reasons.append(f"joint names differ: missing {missing}, unexpected {unexpected}")
    mismatched = sorted(
        name for name in set(names) & set(reference_names) if parents.get(name) != reference_parents.get(name)
    )
    looked.append(f"parent relation checked for {len(names)} joints; mismatches {len(mismatched)}")
    if mismatched:
        reasons.append(f"parent differs for {mismatched}")
    attributes = attribute_names(glb)
    looked.append(f"attributes {attributes}")
    extra_sets = [a for a in attributes if a in ("JOINTS_1", "WEIGHTS_1")]
    if extra_sets:
        reasons.append(f"a second weight set is present: {extra_sets}")
    sums = [sum(vertex.values()) for vertex in vertex_weights(glb)]
    unweighted = sum(1 for s in sums if s < standard.UNWEIGHTED_SUM)
    off = sum(1 for s in sums if s >= standard.UNWEIGHTED_SUM and abs(s - 1.0) > standard.WEIGHT_SUM_TOL)
    looked.append(
        f"vertices {len(sums)}; unweighted {unweighted}; weight sum off by more than tolerance {off}"
    )
    if unweighted:
        reasons.append(f"{unweighted} vertices are unweighted")
    if off:
        reasons.append(f"{off} vertices have a weight sum off 1 by more than {standard.WEIGHT_SUM_TOL}")
    if run_dir is None:
        looked.append("rolls: not checked (no run directory)")
    else:
        dump = _read_json_object(run_dir / "dump_rig.json")
        reference_dump = _read_json_object(run_dir / "reference_dump_rig.json")
        worst = 0.0
        for bone, value in dump.items():
            ref_value = reference_dump.get(bone)
            if not isinstance(value, dict) or not isinstance(ref_value, dict):
                reasons.append(f"bone {bone} is missing from a roll dump")
                continue
            roll = float(cast(float, value["roll"]))
            ref_roll = float(cast(float, ref_value["roll"]))
            worst = max(worst, abs(roll - ref_roll))
        looked.append(f"rolls: {len(dump)} bones; largest difference {worst:.2e} rad")
        if worst > standard.ROLL_TOL_RAD:
            reasons.append(f"a roll differs by {worst:.2e} rad; bound {standard.ROLL_TOL_RAD}")
    return _result(GateId.G5, looked, reasons)


@dataclass(frozen=True)
class DepthSpan:
    """The body along a joint's depth ray: the back-most and front-most surface (glTF z; +z is the front)."""

    vertices: int
    back_z: float
    front_z: float

    def margin(self, joint_z: float) -> float:
        """How far the joint lies inside the span, in metres; negative when it is in front of or behind it."""
        return min(joint_z - self.back_z, self.front_z - joint_z)


def depth_span(
    points: list[tuple[float, float, float]], joint: tuple[float, float, float]
) -> DepthSpan | None:
    """G-6(b)'s inside test (R-B1-G6): the vertices in the joint's slab (±SLAB_HALF_M in y) whose x lies
    within DEPTH_RAY_HALF_M of the joint's, i.e. a ray cast along the depth axis through the joint. None when
    the ray meets no surface. Nothing is clipped by distance, so a deep section (a body fused with a backpack)
    is measured whole."""
    jx, jy, _ = joint
    zs = [
        z
        for x, y, z in points
        if abs(y - jy) <= standard.SLAB_HALF_M and abs(x - jx) <= standard.DEPTH_RAY_HALF_M
    ]
    if not zs:
        return None
    return DepthSpan(vertices=len(zs), back_z=min(zs), front_z=max(zs))


def gate_g6(glb: Glb, fit: FitParams) -> GateResult:
    """G-6: (a) the four limb segments on each side, as a fraction of stature, within ±WINTER_TOL of Winter.

    (b) Every BODY_JOINT lies inside the body: between the back-most and the front-most surface on a ray cast
    along the depth axis through it (`depth_span`, R-B1-G6). Margins are printed per joint."""
    world = joint_world_positions(glb)
    stature = fit.stature_m
    looked: list[str] = []
    reasons: list[str] = []
    for segment, proximal, distal in standard.LIMB_SEGMENTS:
        a = world[proximal]
        b = world[distal]
        ratio = math.dist(a, b) / stature
        expected = standard.WINTER_RATIOS[segment]
        relative = abs(ratio - expected) / expected
        looked.append(f"{segment} {proximal}→{distal}: {ratio:.4f} (Winter {expected}, off {relative:.1%})")
        if relative > standard.WINTER_TOL:
            reasons.append(
                f"{segment} {proximal}→{distal} ratio {ratio:.4f} is off Winter {expected} by {relative:.1%}"
            )
    points = positions(glb)
    for joint in standard.BODY_JOINTS:
        span = depth_span(points, world[joint])
        if span is None:
            looked.append(f"{joint}: the depth ray meets no surface")
            reasons.append(f"{joint}: no mesh surface on the depth ray through it (outside the body)")
            continue
        margin = span.margin(world[joint][2])
        looked.append(
            f"{joint}: margin {margin * 1000:.1f} mm; back z {span.back_z:.4f}, front z {span.front_z:.4f} "
            f"over {span.vertices} vertices"
        )
        if margin < 0.0:
            reasons.append(f"{joint} lies outside the body by {-margin * 1000:.1f} mm")
    return _result(GateId.G6, looked, reasons)


_UUID = re.compile(standard.UUID_PATTERN)
_SHA256 = re.compile(standard.SHA256_PATTERN)
_KEY_PATTERNS = tuple(re.compile(p) for p in standard.KEY_PATTERNS)


def _strings(value: object, where: str) -> list[tuple[str, str]]:
    if isinstance(value, str):
        return [(where, value)]
    if isinstance(value, dict):
        found: list[tuple[str, str]] = []
        for key, item in cast(dict[str, object], value).items():
            found.extend(_strings(item, f"{where}.{key}"))
        return found
    if isinstance(value, list):
        found = []
        for index, item in enumerate(cast(list[object], value)):
            found.extend(_strings(item, f"{where}[{index}]"))
        return found
    return []


def gate_g14(glb: Glb, repo_root: Path) -> GateResult:
    """G-14: the provenance block carries the required keys with their types.

    Each reference's SHA-256 matches its file when the path exists in the repository, and no string matches an
    AI-identifier key pattern (AC-S26-9)."""
    extras = scene_extras(glb)
    raw = extras.get("mineworld_provenance")
    looked = ["scenes[0].extras.mineworld_provenance"]
    if not isinstance(raw, dict):
        return _result(GateId.G14, looked, ["mineworld_provenance is missing"])
    provenance = cast(dict[str, object], raw)
    reasons: list[str] = []
    missing = [key for key in standard.PROVENANCE_KEYS if key not in provenance]
    looked.append(f"provenance keys {sorted(provenance)}")
    if missing:
        reasons.append(f"missing: {', '.join(missing)}")
    if provenance.get("source_type") != "generated":
        reasons.append('source_type is not "generated"')
    if provenance.get("ai_generated") is not True:
        reasons.append("ai_generated is not true")
    generator = provenance.get("generator")
    if not isinstance(generator, str) or not generator:
        reasons.append("generator is empty or not a string")
    task = provenance.get("meshy_task")
    if not isinstance(task, str) or _UUID.fullmatch(task) is None:
        reasons.append("meshy_task is not a UUID")
    references = provenance.get("references")
    if references is not None:
        if not isinstance(references, list):
            reasons.append("references is not a list")
        else:
            for index, item in enumerate(cast(list[object], references)):
                reasons.extend(_check_reference(index, item, repo_root))
    if "reference_generator" in provenance and not isinstance(provenance["reference_generator"], str):
        reasons.append("reference_generator is not a string")
    pipeline = provenance.get("pipeline")
    if "pipeline" in provenance:
        if not isinstance(pipeline, dict):
            reasons.append("pipeline is not an object")
        else:
            block = cast(dict[str, object], pipeline)
            for key in ("version", "fit_sha256", "blender"):
                if not isinstance(block.get(key), str):
                    reasons.append(f"pipeline.{key} is missing or not a string")
    for key, text in _strings(provenance, "mineworld_provenance"):
        for pattern in _KEY_PATTERNS:
            if pattern.search(text):
                reasons.append(f"{key} matches an AI-identifier key pattern")
    return _result(GateId.G14, looked, reasons)


def _check_reference(index: int, item: object, repo_root: Path) -> list[str]:
    if not isinstance(item, dict):
        return [f"references[{index}] is not an object"]
    entry = cast(dict[str, object], item)
    path = entry.get("path")
    sha = entry.get("sha256")
    problems: list[str] = []
    if not isinstance(path, str):
        problems.append(f"references[{index}].path is not a string")
        return problems
    if not isinstance(sha, str) or _SHA256.fullmatch(sha) is None:
        problems.append(f"references[{index}].sha256 is not 64 hex characters")
        return problems
    target = repo_root / path
    if target.is_file():
        actual = hashlib.sha256(target.read_bytes()).hexdigest()
        if actual != sha:
            problems.append(f"references[{index}] sha256 does not match {path}")
    return problems


def run_static_gates(glb: Glb, fit: FitParams, repo_root: Path, run_dir: Path | None) -> list[GateResult]:
    """Every static gate in order: G-1…G-6, G-14. G-4(b) and G-5's rolls need `run_dir`."""
    return [
        gate_g1(glb),
        gate_g2(glb),
        gate_g3(glb),
        gate_g4(glb, fit, run_dir),
        gate_g5(glb, repo_root, run_dir),
        gate_g6(glb, fit),
        gate_g14(glb, repo_root),
    ]
