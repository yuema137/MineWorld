"""The frame dump reader: what Godot skinned, frame by frame (MWCF v1; PIPELINE.md §15.1, S36-P2).

`godot/pose_sheet.gd --dump-frames <dir>` writes one file per body (the ground probe) or per run (the pose
sheet's captured frames). Godot skins; this module only reads (I-S36-3). The layout, little-endian:

    header   "MWCF" u32 version=1  u32 vertex_count  u32 face_count  u32 bone_count  u32 frame_count
    bones    bone_count × (u16 length, UTF-8 name)
    faces    face_count × 3 × u32                       global vertex indices, all meshes
    labels   vertex_count × (u16 dominant bone, f32 trunk share, f32 limb share,
                             u16 mesh index, i16 region index, u8 under flag, u8 layer)
    meshes   u16 count, count × (u16 length, UTF-8 name)
    ground   u32 ground_count, ground_count × u32 vertex index
    frames   frame_count × (u16 tag length, UTF-8 tag "walk:12", f32 floor_y, f32 speed_mps,
                            f32 fwd_x, f32 fwd_z, bone_count × 3 × f32 bone origins,
                            u8 full, full ? vertex_count × 3 × f32 : ground_count × 3 × f32)

The ground set precedes the frames (a bounded correction to the design's order, which put it last): a frame
that carries the ground set only cannot be sized before the set's length is known.

Positions are glTF/Godot metres, world space, y up. The floor of a frame is `y = floor_y`.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

import numpy as np
from numpy.typing import NDArray

MAGIC = b"MWCF"
VERSION = 1
LABEL = np.dtype(
    [
        ("bone", "<u2"),
        ("trunk", "<f4"),
        ("limb", "<f4"),
        ("mesh", "<u2"),
        ("region", "<i2"),
        ("under", "u1"),
        ("layer", "u1"),
    ]
)

Floats = NDArray[np.float64]
Ints = NDArray[np.int64]


class FramesError(Exception):
    """A dump is not MWCF v1, is truncated, or holds an index out of range. The message names the file."""


@dataclass(frozen=True)
class Frame:
    """One dumped frame. `positions` holds every vertex when `full`, else the ground set's, in its order."""

    tag: str
    floor_y: float
    speed_mps: float
    fwd: Floats  # (2,) horizontal forward (x, z), unit length
    bones: Floats  # (bone_count, 3) world origins
    full: bool
    positions: Floats

    @property
    def clip(self) -> str:
        return self.tag.split(":", 1)[0]

    @property
    def index(self) -> int:
        _, _, number = self.tag.partition(":")
        return int(number) if number.isdigit() else 0


@dataclass(frozen=True)
class Frames:
    path: Path
    bone_names: tuple[str, ...]
    faces: Ints  # (m, 3)
    labels: NDArray[np.void]  # (n,) of LABEL
    mesh_names: tuple[str, ...]
    ground: Ints  # (g,) vertex indices
    frames: tuple[Frame, ...]

    @property
    def vertex_count(self) -> int:
        return int(self.labels.shape[0])

    def dominant(self) -> tuple[str, ...]:
        """Each vertex's dominant bone, by name."""
        return tuple(self.bone_names[int(b)] for b in self.labels["bone"])

    def bone(self, name: str) -> int:
        if name not in self.bone_names:
            raise FramesError(f"{self.path}: the skeleton has no bone {name!r}")
        return self.bone_names.index(name)

    def clip(self, name: str) -> list[Frame]:
        """The frames of one clip ("rest", "stand", "walk", "jog", or a capture id), in index order."""
        return sorted((f for f in self.frames if f.clip == name), key=lambda f: f.index)

    def ground_positions(self, frame: Frame) -> Floats:
        """The ground set's positions in `frame`, whether it carries every vertex or the set only."""
        return frame.positions[self.ground] if frame.full else frame.positions

    def full_positions(self, frame: Frame) -> Floats:
        if not frame.full:
            raise FramesError(f"{self.path}: frame {frame.tag} carries the ground set only")
        return frame.positions

    def joints(self, frame: Frame) -> dict[str, tuple[float, float, float]]:
        return {
            name: (float(p[0]), float(p[1]), float(p[2]))
            for name, p in zip(self.bone_names, frame.bones, strict=True)
        }


class _Cursor:
    def __init__(self, path: Path, data: bytes) -> None:
        self.path = path
        self.data = data
        self.at = 0

    def take(self, size: int, what: str) -> bytes:
        if size < 0 or self.at + size > len(self.data):
            raise FramesError(f"{self.path}: truncated in {what} (needs {size} bytes at offset {self.at})")
        chunk = self.data[self.at : self.at + size]
        self.at += size
        return chunk

    def u8(self, what: str) -> int:
        return int(np.frombuffer(self.take(1, what), dtype="u1")[0])

    def u16(self, what: str) -> int:
        return int(np.frombuffer(self.take(2, what), dtype="<u2")[0])

    def u32(self, what: str) -> int:
        return int(np.frombuffer(self.take(4, what), dtype="<u4")[0])

    def floats(self, count: int, what: str) -> NDArray[np.float32]:
        return np.frombuffer(self.take(4 * count, what), dtype="<f4")

    def text(self, what: str) -> str:
        length = self.u16(what)
        try:
            return self.take(length, what).decode("utf-8")
        except UnicodeDecodeError as exc:
            raise FramesError(f"{self.path}: {what} is not UTF-8") from exc


def read(path: Path) -> Frames:
    """Read and check a dump: magic, version, every section's length, every index in range."""
    cur = _Cursor(path, path.read_bytes())
    if cur.take(4, "the magic") != MAGIC:
        raise FramesError(f"{path}: not an MWCF dump (bad magic)")
    version = cur.u32("the version")
    if version != VERSION:
        raise FramesError(f"{path}: MWCF version {version}, expected {VERSION}")
    n = cur.u32("the vertex count")
    m = cur.u32("the face count")
    b = cur.u32("the bone count")
    count = cur.u32("the frame count")
    bone_names = tuple(cur.text("a bone name") for _ in range(b))
    faces = np.frombuffer(cur.take(12 * m, "the faces"), dtype="<u4").astype(np.int64).reshape(m, 3)
    if faces.size and int(faces.max()) >= n:
        raise FramesError(f"{path}: a face index {int(faces.max())} is out of range (vertices {n})")
    labels = np.frombuffer(cur.take(LABEL.itemsize * n, "the labels"), dtype=LABEL)
    if labels.size and int(labels["bone"].max()) >= b:
        raise FramesError(f"{path}: a dominant bone index {int(labels['bone'].max())} is out of range ({b})")
    mesh_names = tuple(cur.text("a mesh name") for _ in range(cur.u16("the mesh count")))
    if labels.size and int(labels["mesh"].max()) >= len(mesh_names):
        raise FramesError(f"{path}: a mesh index is out of range ({len(mesh_names)} meshes)")
    g = cur.u32("the ground count")
    ground = np.frombuffer(cur.take(4 * g, "the ground set"), dtype="<u4").astype(np.int64)
    if ground.size and int(ground.max()) >= n:
        raise FramesError(f"{path}: a ground index {int(ground.max())} is out of range (vertices {n})")
    frames: list[Frame] = []
    for _ in range(count):
        tag = cur.text("a frame tag")
        head = cur.floats(4, f"frame {tag}").astype(np.float64)
        bones = cur.floats(3 * b, f"frame {tag} bones").astype(np.float64).reshape(b, 3)
        full = cur.u8(f"frame {tag}") == 1
        rows = n if full else g
        positions = cur.floats(3 * rows, f"frame {tag} positions").astype(np.float64).reshape(rows, 3)
        fwd = np.asarray(head[2:4], dtype=np.float64)
        frames.append(Frame(tag, float(head[0]), float(head[1]), fwd, bones, full, positions))
    if cur.at != len(cur.data):
        raise FramesError(f"{path}: {len(cur.data) - cur.at} bytes after the last frame")
    return Frames(path, bone_names, faces, labels, mesh_names, ground, tuple(frames))
