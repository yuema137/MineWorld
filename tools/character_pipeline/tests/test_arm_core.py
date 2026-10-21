"""The arms against the body core (PR S36-CC10 commit 2): `core_shell`, `arm_queries`, `arm_depths` and the
pseudonormal confirmation of `contact.inside`, on generated bodies with planted, hand-computable depths.

The second body set: twelve skinned box bodies from the seed "s36-cc10" (trunk, hips, two thighs, two hanging
arms), each with one hand planted a known distance from the side of a thigh or of the hips. Boxes have flat
faces, so the depth of the hand's inner corners is exactly the planted overlap. The torso's corners carry
15 % upper-arm weight, as a fused body's flank does (commit 1's seam)."""

from __future__ import annotations

import math
import random
from dataclasses import dataclass, replace

import numpy as np
import pytest
from contact_shapes import box
from numpy.typing import NDArray

from mwcast import contact
from mwcast.contact_gates import LimbDepth, Skinning, arm_depths, arm_queries, core_shell
from mwcast.tolerances import Tolerances

Floats = NDArray[np.float64]
Ints = NDArray[np.int64]
Vec3 = tuple[float, float, float]

SEED = "s36-cc10"
_SIDES = (("Left", 1.0), ("Right", -1.0))
_CLEAR_GAP_M = 0.015  # the unplanted hand hangs this far from the body


@dataclass(frozen=True)
class Body:
    """One generated body: its parameters (metres, degrees) and the plant."""

    trunk_r: float
    hip_w: float
    thigh_r: float
    abduction_deg: float
    gap_m: float  # the planted hand's gap to the surface beside it; negative: inside
    side: str
    at: str  # "thigh" or "hip"

    @property
    def thigh_side(self) -> float:
        """x of a thigh's lateral face: the thighs stand 30 mm proud of the hips."""
        return self.hip_w + 0.03


class _Mesh:
    def __init__(self) -> None:
        self.vertices: list[Floats] = []
        self.faces: list[Ints] = []
        self.weights: list[dict[str, float]] = []
        self.count = 0

    def add(self, v: Floats, f: Ints, weights: list[dict[str, float]]) -> None:
        self.vertices.append(v)
        self.faces.append(f + self.count)
        self.weights += weights
        self.count += v.shape[0]

    def skin(self) -> tuple[Skinning, Ints]:
        return Skinning.of(np.vstack(self.vertices).tolist(), self.weights), np.vstack(self.faces)


def _segment_box(a: Floats, b: Floats, half: float) -> tuple[Floats, Ints]:
    """A box around the segment a→b in the x-y plane (half-width `half`, z ±`half`), outward-facing."""
    v, f = box((-half, 0.0, -half), (half, 1.0, half))
    axis = b - a
    length = float(np.linalg.norm(axis))
    u = axis / length
    n = np.array([u[1], -u[0], 0.0])  # perpendicular in x-y; (n, u, z) is right-handed, keeping the winding
    placed = a + np.outer(v[:, 0], n) + np.outer(v[:, 1] * length, u) + np.outer(v[:, 2], [0.0, 0.0, 1.0])
    return placed, f


def build(body: Body) -> tuple[Skinning, Ints, dict[str, Vec3]]:
    mesh = _Mesh()
    v, f = box((-body.trunk_r, 1.0, -0.1), (body.trunk_r, 1.45, 0.1))
    mesh.add(v, f, [{"Spine": 0.85, ("LeftUpperArm" if x > 0 else "RightUpperArm"): 0.15} for x in v[:, 0]])
    v, f = box((-body.hip_w, 0.8, -0.11), (body.hip_w, 1.0, 0.11))
    mesh.add(v, f, [{"Hips": 1.0}] * 8)
    tx = body.thigh_side - body.thigh_r
    joints: dict[str, Vec3] = {"Hips": (0.0, 0.92, 0.0), "Spine": (0.0, 1.02, 0.0), "Neck": (0.0, 1.5, 0.0)}
    a = math.radians(body.abduction_deg)
    for side, s in _SIDES:
        x0, x1 = sorted((s * (tx - body.thigh_r), s * (tx + body.thigh_r)))
        v, f = box((x0, 0.5, -body.thigh_r), (x1, 0.8, body.thigh_r))
        mesh.add(v, f, [{side + "UpperLeg": 1.0}] * 8)
        joints |= {side + "UpperLeg": (s * tx, 0.8, 0.0), side + "LowerLeg": (s * tx, 0.5, 0.0)}
        joints[side + "Foot"] = (s * tx, 0.08, 0.0)
        gap = body.gap_m if side == body.side else _CLEAR_GAP_M
        at = body.at if side == body.side else "thigh"
        inner, wrist_y = (body.thigh_side + gap, 0.8) if at == "thigh" else (body.hip_w + gap, 1.0)
        hx0, hx1 = sorted((s * inner, s * (inner + 0.03)))
        v, f = box((hx0, wrist_y - 0.18, -0.02), (hx1, wrist_y - 0.02, 0.02))
        mesh.add(v, f, [{side + "Hand": 1.0}] * 8)
        wrist = np.array([s * (inner + 0.015), wrist_y, 0.0])
        elbow = wrist + 0.27 * np.array([s * math.sin(a), math.cos(a), 0.0])
        shoulder = elbow + 0.28 * np.array([s * math.sin(a / 2), math.cos(a / 2), 0.0])
        lift = (elbow - wrist) / float(np.linalg.norm(elbow - wrist)) * 0.02
        v, f = _segment_box(wrist + lift, elbow, 0.02)
        mesh.add(v, f, [{side + "LowerArm": 1.0}] * 8)
        v, f = _segment_box(elbow, shoulder, 0.025)
        mesh.add(v, f, [{side + "UpperArm": 1.0}] * 8)
        joints |= {
            side + "UpperArm": (float(shoulder[0]), float(shoulder[1]), 0.0),
            side + "LowerArm": (float(elbow[0]), float(elbow[1]), 0.0),
            side + "Hand": (float(wrist[0]), float(wrist[1]), 0.0),
        }
    skin, faces = mesh.skin()
    return skin, faces, joints


def second_body_set() -> list[Body]:
    """Twelve bodies from the seed: every planted gap from +10 mm to -12 mm in 2 mm steps, once each."""
    rng = random.Random(SEED)
    gaps = [g / 1000 for g in range(10, -13, -2)]
    rng.shuffle(gaps)
    bodies: list[Body] = []
    for i, gap in enumerate(gaps):
        trunk_r = rng.uniform(0.12, 0.18)
        bodies.append(
            Body(
                trunk_r=trunk_r,
                hip_w=trunk_r + rng.uniform(0.03, 0.06),
                thigh_r=rng.uniform(0.07, 0.11),
                abduction_deg=rng.uniform(2.0, 10.0),
                gap_m=gap,
                side=rng.choice(("Left", "Right")),
                at="hip" if i % 3 == 2 else "thigh",
            )
        )
    return bodies


def _by_key(depths: list[LimbDepth]) -> dict[str, LimbDepth]:
    return {d.key: d for d in depths}


@pytest.mark.parametrize("body", second_body_set(), ids=lambda b: f"{b.side}-{b.at}-{b.gap_m * 1000:+.0f}mm")
def test_a_generated_body_ends_as_its_planted_depth_says(body: Body) -> None:
    tolerances = Tolerances.load()
    skin, faces, joints = build(body)
    depths = _by_key(arm_depths(skin, faces, joints, tolerances))
    assert set(depths) == {
        f"{side} {group}" for side, _ in _SIDES for group in ("hand", "sleeve", "upper_arm")
    }
    planted = depths[f"{body.side} hand"]
    for key, reading in depths.items():
        if reading is not planted:
            assert reading.worst_mm == 0.0, (key, reading)
    depth_mm = max(0.0, -body.gap_m * 1000)
    if depth_mm <= 5.0:
        assert planted.passed, planted
        assert planted.worst_mm == pytest.approx(depth_mm, abs=0.5)
        return
    assert not planted.passed
    assert planted.worst_mm == pytest.approx(depth_mm, abs=0.5)
    assert planted.bound_key == ("self.arm_in_thigh" if body.at == "thigh" else "self.hand_in_trunk")
    assert planted.over == 4
    assert (
        planted.reason()
        == f"{body.side} hand is {planted.worst_mm:.1f} mm inside the core (4 vertices over 5 mm)"
    )


def test_the_second_body_set_plants_both_outcomes_at_the_thigh_and_at_the_hip() -> None:
    bodies = second_body_set()
    deep = [b for b in bodies if -b.gap_m > 0.005]
    assert {b.at for b in deep} == {"thigh", "hip"}
    assert len(deep) == 4 and len(bodies) - len(deep) == 8


# --- an open flared garment over a hanging arm (F-CC10-3) ---------------------------------------------------
# A planar open hem cannot read a point outside the garment as inside: seen from above its plane, the missing
# cap only lowers the winding number. What the sign confirmation answers is a distant sheet that wraps an arm:
# a cloak hanging from the chest over both arms, open at the hem. Every arm vertex is inside the cloak's hull
# (winding number about 1), while the surface nearest it is the body's, whose front faces it.


def _frustum(
    top: tuple[float, float], bottom: tuple[float, float], segments: int = 48
) -> tuple[Floats, Ints]:
    """An open surface of revolution about y from (y, r) `top` to `bottom`, outward-facing, 6 rings."""
    rings = 6
    vertices: list[tuple[float, float, float]] = []
    for k in range(rings + 1):
        t = k / rings
        y = top[0] + (bottom[0] - top[0]) * t
        r = top[1] + (bottom[1] - top[1]) * t
        vertices += [
            (r * math.cos(2 * math.pi * i / segments), y, r * math.sin(2 * math.pi * i / segments))
            for i in range(segments)
        ]
    faces: list[tuple[int, int, int]] = []
    for k in range(rings):
        for i in range(segments):
            a, b = k * segments + i, k * segments + (i + 1) % segments
            faces += [(a, b, b + segments), (a, b + segments, a + segments)]
    return np.array(vertices, dtype=np.float64), np.array(faces, dtype=np.int64)


_CLOAKED = Body(0.15, 0.19, 0.09, 4.0, _CLEAR_GAP_M, "Left", "thigh")
_HEM_Y = 0.62 - 0.06  # 60 mm below the hand


def _cloaked(gap_m: float) -> tuple[Skinning, Ints, dict[str, Vec3]]:
    skin, faces, joints = build(replace(_CLOAKED, gap_m=gap_m))
    v, f = _frustum((1.45, 0.48), (_HEM_Y, 0.52))
    positions = np.vstack([skin.positions, v])
    weights = [{"Chest": 1.0}] * v.shape[0]
    cloak = Skinning.of(v.tolist(), weights)
    merged = Skinning(
        positions,
        skin.dominant + cloak.dominant,
        np.concatenate([skin.trunk_share, cloak.trunk_share]),
        np.concatenate([skin.limb_share, cloak.limb_share]),
    )
    return merged, np.vstack([faces, f + skin.positions.shape[0]]), joints


def test_a_cloaked_arm_beside_the_thigh_passes_because_the_nearest_surface_faces_it() -> None:
    tolerances = Tolerances.load()
    skin, faces, joints = _cloaked(_CLEAR_GAP_M)
    depths = _by_key(arm_depths(skin, faces, joints, tolerances))
    assert all(d.passed and d.worst_mm == 0.0 for d in depths.values()), depths
    # the winding number alone reads every hand vertex inside the cloak
    core = contact.TriMesh(skin.positions, core_shell(skin, faces))
    hand = arm_queries(skin, joints)["Left hand"]
    assert contact.inside(core, skin.positions[hand]).indices.size == hand.size


def test_a_cloaked_hand_planted_8_mm_into_the_thigh_fails_8_mm() -> None:
    skin, faces, joints = _cloaked(-0.008)
    hand = _by_key(arm_depths(skin, faces, joints, Tolerances.load()))["Left hand"]
    assert not hand.passed
    assert hand.worst_mm == pytest.approx(8.0, abs=0.5)
    assert hand.bound_key == "self.arm_in_thigh"


def test_a_point_15_mm_outside_an_open_flared_skirt_is_outside_with_or_without_the_sign() -> None:
    # the design's literal open hem: a Hips skirt flaring from the waist to a hem 60 mm below the hand, over
    # the body's hips and thighs; the winding number beside it stays under 0.5 (F-CC10-8)
    skin, faces, _joints = build(_CLOAKED)
    top, bottom = (1.0, 0.24), (_HEM_Y, 0.3)
    skirt_v, skirt_f = _frustum(top, bottom)
    vertices = np.vstack([skin.positions, skirt_v])
    core = contact.TriMesh(vertices, np.vstack([faces, skirt_f + skin.positions.shape[0]]))

    def radius(y: float) -> float:
        return top[1] + (bottom[1] - top[1]) * (y - top[0]) / (bottom[0] - top[0])

    beside = np.array([[radius(y) + 0.015, y, 0.0] for y in (0.62, 0.7, 0.78)])
    assert contact.inside(core, beside).indices.size == 0
    assert contact.inside(core, beside, confirm_sign=True).indices.size == 0
