"""A test-side MWCF v1 writer (the layout of `mwcast/frames.py`) and synthetic bodies for the gait gates.

The synthetic body is a box trunk with two box feet (each foot a `Foot` and a `Toes` box), two box shins and
two box arms, on a skeleton with the humanoid bone names the probe's dumps carry. A walk is a treadmill cycle:
each foot moves backward at the gait speed for its stance half and forward in its swing half, lifted off the
floor in swing. Frames are built from it, then a test perturbs one foot in chosen frames.
"""

from __future__ import annotations

import struct
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np
from numpy.typing import NDArray

Floats = NDArray[np.float64]

BONES = (
    "Hips",
    "Spine",
    "Chest",
    "UpperChest",
    "Neck",
    "Head",
    "LeftUpperArm",
    "LeftLowerArm",
    "LeftHand",
    "RightUpperArm",
    "RightLowerArm",
    "RightHand",
    "LeftUpperLeg",
    "LeftLowerLeg",
    "LeftFoot",
    "LeftToes",
    "RightUpperLeg",
    "RightLowerLeg",
    "RightFoot",
    "RightToes",
)


@dataclass
class Dump:
    """Everything an MWCF file holds, as arrays; `write` lays it out."""

    faces: NDArray[np.int64]
    bone: NDArray[np.int64]  # dominant bone per vertex
    trunk: Floats
    limb: Floats
    ground: NDArray[np.int64]
    meshes: tuple[str, ...] = ("Body",)
    layer: NDArray[np.int64] | None = None
    frames: list[tuple[str, float, float, tuple[float, float], Floats, bool, Floats]] = field(
        default_factory=lambda: []
    )

    @property
    def n(self) -> int:
        return int(self.bone.shape[0])

    def add(
        self, tag: str, bones: Floats, positions: Floats, *, speed: float = 0.0, full: bool = True
    ) -> None:
        self.frames.append(
            (tag, 0.0, speed, (0.0, 1.0), bones, full, positions if full else positions[self.ground])
        )

    def write(self, path: Path, *, magic: bytes = b"MWCF", version: int = 1) -> Path:
        out = bytearray()
        out += magic + struct.pack("<5I", version, self.n, self.faces.shape[0], len(BONES), len(self.frames))
        for name in BONES:
            raw = name.encode()
            out += struct.pack("<H", len(raw)) + raw
        out += self.faces.astype("<u4").tobytes()
        layer = self.layer if self.layer is not None else np.zeros(self.n, dtype=np.int64)
        for i in range(self.n):
            out += struct.pack(
                "<HffHhBB",
                int(self.bone[i]),
                float(self.trunk[i]),
                float(self.limb[i]),
                0,
                -1,
                0,
                int(layer[i]),
            )
        out += struct.pack("<H", len(self.meshes))
        for name in self.meshes:
            raw = name.encode()
            out += struct.pack("<H", len(raw)) + raw
        out += struct.pack("<I", self.ground.shape[0]) + self.ground.astype("<u4").tobytes()
        for tag, floor_y, speed, (fx, fz), bones, full, positions in self.frames:
            raw = tag.encode()
            out += struct.pack("<H", len(raw)) + raw + struct.pack("<4f", floor_y, speed, fx, fz)
            out += bones.astype("<f4").tobytes() + struct.pack("<B", 1 if full else 0)
            out += positions.astype("<f4").tobytes()
        path.write_bytes(bytes(out))
        return path


def box(
    center: tuple[float, float, float], size: tuple[float, float, float]
) -> tuple[Floats, NDArray[np.int64]]:
    """A closed box: 8 corners and 12 outward triangles."""
    c = np.asarray(center, dtype=np.float64)
    h = np.asarray(size, dtype=np.float64) / 2.0
    corners = np.asarray([[x, y, z] for x in (-1, 1) for y in (-1, 1) for z in (-1, 1)], dtype=np.float64)
    faces = np.asarray(
        [
            [0, 1, 3],
            [0, 3, 2],
            [4, 6, 7],
            [4, 7, 5],
            [0, 4, 5],
            [0, 5, 1],
            [2, 3, 7],
            [2, 7, 6],
            [0, 2, 6],
            [0, 6, 4],
            [1, 5, 7],
            [1, 7, 3],
        ],
        dtype=np.int64,
    )
    return c + corners * h, faces


def subdivided_box(
    center: tuple[float, float, float], size: tuple[float, float, float], k: int
) -> tuple[Floats, NDArray[np.int64]]:
    """A box plus a k×k×k grid of points inside it: the faces use only the 8 corners, and the extra points
    carry a limb's query vertices."""
    pts, faces = box(center, size)
    c = np.asarray(center, dtype=np.float64)
    h = np.asarray(size, dtype=np.float64) / 2.0
    grid = np.linspace(-0.9, 0.9, k)
    extra = np.asarray([[x, y, z] for x in grid for y in grid for z in grid], dtype=np.float64)
    return np.vstack([pts, c + extra * h]), faces


@dataclass
class Body:
    dump: Dump
    rest: Floats  # (n, 3)
    parts: dict[str, NDArray[np.int64]]  # part name -> its vertex indices
    joints: Floats  # (len(BONES), 3) at rest


def body() -> Body:
    """The synthetic body at rest: trunk box from 0.9 to 1.4 m, arms hanging 0.25 m out, feet on the floor."""
    pieces: list[tuple[str, str, Floats, NDArray[np.int64]]] = []
    trunk_pts, trunk_faces = box((0.0, 1.15, 0.0), (0.36, 0.5, 0.22))
    pieces.append(("trunk", "Chest", trunk_pts, trunk_faces))
    for side, x in (("Left", 0.1), ("Right", -0.1)):
        pts, faces = box((x, 0.03, 0.03), (0.09, 0.06, 0.14))
        pieces.append((f"{side} foot", f"{side}Foot", pts, faces))
        pts, faces = box((x, 0.02, 0.15), (0.09, 0.04, 0.08))
        pieces.append((f"{side} toes", f"{side}Toes", pts, faces))
        pts, faces = box((x, 0.25, 0.0), (0.08, 0.3, 0.08))
        pieces.append((f"{side} shin", f"{side}LowerLeg", pts, faces))
        arm_x = 0.3 if side == "Left" else -0.3
        pts, faces = subdivided_box((arm_x, 1.12, 0.0), (0.06, 0.06, 0.06), 3)
        pieces.append((f"{side} hand", f"{side}Hand", pts, faces))
    positions: list[Floats] = []
    faces_all: list[NDArray[np.int64]] = []
    bones: list[int] = []
    parts: dict[str, NDArray[np.int64]] = {}
    n = 0
    for name, bone, pts, faces in pieces:
        positions.append(pts)
        faces_all.append(faces + n)
        bones += [BONES.index(bone)] * pts.shape[0]
        parts[name] = np.arange(n, n + pts.shape[0], dtype=np.int64)
        n += pts.shape[0]
    rest = np.vstack(positions)
    bone_arr = np.asarray(bones, dtype=np.int64)
    trunk_share = np.asarray([1.0 if BONES[b] == "Chest" else 0.0 for b in bone_arr], dtype=np.float64)
    ground = np.flatnonzero(rest[:, 1] <= rest[:, 1].min() + 0.30).astype(np.int64)
    joints = np.zeros((len(BONES), 3), dtype=np.float64)
    at = {
        "Hips": (0.0, 0.9, 0.0),
        "Spine": (0.0, 0.95, 0.0),
        "Chest": (0.0, 1.1, 0.0),
        "UpperChest": (0.0, 1.3, 0.0),
        "Neck": (0.0, 1.5, 0.0),
        "Head": (0.0, 1.6, 0.0),
    }
    for side, s in (("Left", 1.0), ("Right", -1.0)):
        at[f"{side}UpperArm"] = (0.2 * s, 1.42, 0.0)
        at[f"{side}LowerArm"] = (0.27 * s, 1.27, 0.0)
        at[f"{side}Hand"] = (0.3 * s, 1.15, 0.0)
        at[f"{side}UpperLeg"] = (0.1 * s, 0.9, 0.0)
        at[f"{side}LowerLeg"] = (0.1 * s, 0.45, 0.0)
        at[f"{side}Foot"] = (0.1 * s, 0.08, 0.0)
        at[f"{side}Toes"] = (0.1 * s, 0.03, 0.13)
    for name, p in at.items():
        joints[BONES.index(name)] = p
    dump = Dump(
        faces=np.vstack(faces_all),
        bone=bone_arr,
        trunk=trunk_share,
        limb=1.0 - trunk_share,
        ground=ground,
    )
    return Body(dump, rest, parts, joints)


WALK_FRAMES = 30
WALK_MPS = 1.2


def walk_offsets(
    side: str, k: int, frames: int = WALK_FRAMES, speed: float = WALK_MPS
) -> tuple[float, float]:
    """A foot's (z, lift) at frame k of a treadmill cycle: backward at `speed` in stance (half the cycle, lift
    0), forward in swing with a lift of up to 6 cm. The right foot is half a cycle behind the left."""
    phase = ((k + (0 if side == "Left" else frames // 2)) % frames) / frames
    period = frames / 30.0
    stride = speed * period / 2.0
    if phase < 0.5:
        return stride / 2.0 - stride * (phase / 0.5), 0.0
    t = (phase - 0.5) / 0.5
    return -stride / 2.0 + stride * t, 0.06 * float(np.sin(np.pi * t))


def build(
    tmp: Path,
    *,
    sink: dict[tuple[str, str, int], float] | None = None,
    lift: dict[tuple[str, str, int], float] | None = None,
    arm_in: float = 0.0,
    name: str = "body.mwcf",
) -> Path:
    """A dump with rest, 30 stand frames (every 10th full), a 30-frame walk and a 30-frame jog (the walk's
    feet). `sink[(clip, side, k)]` lowers that foot by metres in that frame; `lift` raises it; `arm_in` moves
    the left hand that far into the trunk in every walk and jog frame."""
    b = body()
    d = b.dump
    sink = sink or {}
    lift = lift or {}

    def frame(clip: str, k: int, moving: bool) -> tuple[Floats, Floats]:
        pos = b.rest.copy()
        joints = b.joints.copy()
        for side in ("Left", "Right"):
            z, up = walk_offsets(side, k) if moving else (0.0, 0.0)
            side_key = side.lower()
            up += lift.get((clip, side_key, k), 0.0) - sink.get((clip, side_key, k), 0.0)
            for part in (f"{side} foot", f"{side} toes"):
                idx = b.parts[part]
                pos[idx, 2] += z
                pos[idx, 1] += up
            for bone in (f"{side}Foot", f"{side}Toes"):
                joints[BONES.index(bone), 2] += z
                joints[BONES.index(bone), 1] += up
        if moving and arm_in:
            pos[b.parts["Left hand"], 0] -= arm_in
            joints[BONES.index("LeftHand"), 0] -= arm_in
            joints[BONES.index("LeftLowerArm"), 0] -= arm_in
        return joints, pos

    joints, pos = frame("rest", 0, False)
    d.add("rest:0", joints, pos)
    for k in range(30):
        joints, pos = frame("stand", k, False)
        d.add(f"stand:{k}", joints, pos, full=k % 10 == 0)
    for clip in ("walk", "jog"):
        for k in range(WALK_FRAMES):
            joints, pos = frame(clip, k, True)
            d.add(f"{clip}:{k}", joints, pos, speed=WALK_MPS)
    return d.write(tmp / name)
