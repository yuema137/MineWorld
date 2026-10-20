"""The contact probe: how deep is a point inside a surface, and which rigid parts touch (step-36 §3).

This is the only module that imports `igl` (libigl, DEP-142), `trimesh` or `fcl` (python-fcl,
DEP-143). Invariant I-S36-1 holds that, and `tests/test_contact_licence.py` enforces it.
`igl.copyleft` wraps GPL code and is never imported (DEP-142 "Licence boundary").

The module takes numpy arrays in glTF metres and returns typed results with depths in millimetres. It
knows nothing about a body, a garment, a prop, a bound or a verdict: the gates in `contact_gates.py`
decide.

Why a winding number. A garment shell or a Meshy body is often open (neck, cuffs). Ray parity reads an
inside point as outside when the ray leaves through a hole. The generalized winding number (Jacobson
et al. 2013; the fast variant of Barill et al. 2018) is about 1 inside and about 0 outside, also at
holes, so a threshold of 0.5 answers "inside?" robustly. The distance to the surface is the depth.

The libraries have no complete type information. Each is loaded through `_load`, which turns a missing
library into `ContactUnavailable`, and every value that comes back is converted with `numpy.asarray`
and checked for shape before it leaves this module.
"""

from __future__ import annotations

import importlib
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path
from types import ModuleType
from typing import Any

import numpy as np
from numpy.typing import NDArray

# The winding-number threshold between outside (about 0) and inside (about 1); Jacobson et al. 2013, §4.
INSIDE_THRESHOLD = 0.5
_MM_PER_M = 1000.0

# Import name -> the distribution a developer installs (the name in pyproject.toml and requirements.txt).
_DISTRIBUTION = {"igl": "libigl", "trimesh": "trimesh", "fcl": "python-fcl"}

Floats = NDArray[np.float64]
Ints = NDArray[np.int64]


class ContactUnavailable(Exception):
    """A contact library is not installed. A gate turns this into FAIL "cannot look" (I-S36-4)."""

    def __init__(self, library: str) -> None:
        super().__init__(f"{library} is not installed")
        self.library = library


def _load(module: str) -> ModuleType:
    try:
        return importlib.import_module(module)
    except ImportError as exc:
        raise ContactUnavailable(_DISTRIBUTION[module]) from exc


def _points(values: object, what: str) -> Floats:
    array = np.ascontiguousarray(np.asarray(values, dtype=np.float64))
    if array.ndim != 2 or array.shape[1] != 3:
        raise ValueError(f"{what}: expected an (n, 3) array, got shape {array.shape}")
    if not np.all(np.isfinite(array)):
        raise ValueError(f"{what}: a coordinate is not finite")
    return array


@dataclass(frozen=True)
class TriMesh:
    """A triangle mesh: (n, 3) float64 vertices in metres and (m, 3) int64 faces indexing them.

    Validated on construction: non-empty, finite, every face index in range."""

    vertices: Floats
    faces: Ints

    def __post_init__(self) -> None:
        vertices = _points(self.vertices, "TriMesh vertices")
        faces = np.ascontiguousarray(np.asarray(self.faces, dtype=np.int64))
        if vertices.shape[0] == 0:
            raise ValueError("TriMesh: no vertices")
        if faces.ndim != 2 or faces.shape[1] != 3 or faces.shape[0] == 0:
            raise ValueError(f"TriMesh faces: expected a non-empty (m, 3) array, got shape {faces.shape}")
        low, high = int(faces.min()), int(faces.max())
        if low < 0 or high >= vertices.shape[0]:
            raise ValueError(
                f"TriMesh faces: index range {low}..{high} is outside 0..{vertices.shape[0] - 1}"
            )
        object.__setattr__(self, "vertices", vertices)
        object.__setattr__(self, "faces", faces)


@dataclass(frozen=True)
class Inside:
    """The query points found inside a surface: their indices into the query, and each one's depth in mm."""

    indices: Ints
    depths_mm: Floats

    @property
    def worst_mm(self) -> float:
        return float(self.depths_mm.max()) if self.depths_mm.size else 0.0


@dataclass(frozen=True)
class Outside:
    """The query points found outside a surface: their indices, and each one's distance to it in mm."""

    indices: Ints
    distances_mm: Floats

    @property
    def worst_mm(self) -> float:
        return float(self.distances_mm.max()) if self.distances_mm.size else 0.0


@dataclass(frozen=True)
class Overlap:
    """Two rigid parts that touch or interpenetrate, and the deepest penetration between them in mm."""

    a: str
    b: str
    depth_mm: float


@dataclass(frozen=True)
class RigidPart:
    """A named mesh placed in the world by a 4×4 transform (column vectors, metres)."""

    name: str
    mesh: TriMesh
    transform: Floats

    def __post_init__(self) -> None:
        transform = np.ascontiguousarray(np.asarray(self.transform, dtype=np.float64))
        if transform.shape != (4, 4) or not np.all(np.isfinite(transform)):
            raise ValueError(f"RigidPart {self.name}: the transform is not a finite 4×4 matrix")
        object.__setattr__(self, "transform", transform)


def _winding_and_distance(outer: TriMesh, query: Floats) -> tuple[Floats, Floats]:
    igl = _load("igl")
    raw_w: Any = igl.fast_winding_number(outer.vertices, outer.faces, query)
    winding = np.asarray(raw_w, dtype=np.float64).reshape(-1)
    raw_d: Any = igl.point_mesh_squared_distance(query, outer.vertices, outer.faces)
    squared = np.asarray(raw_d[0], dtype=np.float64).reshape(-1)
    if winding.shape[0] != query.shape[0] or squared.shape[0] != query.shape[0]:
        raise RuntimeError("libigl returned a result of the wrong length")
    return winding, np.sqrt(squared) * _MM_PER_M


def inside(outer: TriMesh, points: object) -> Inside:
    """The points (n, 3, metres) whose generalized winding number against `outer` exceeds 0.5, with each one's
    distance to `outer` in mm as its depth."""
    query = _points(points, "inside: points")
    if query.shape[0] == 0:
        return Inside(np.zeros(0, dtype=np.int64), np.zeros(0, dtype=np.float64))
    winding, distance = _winding_and_distance(outer, query)
    mask = winding > INSIDE_THRESHOLD
    return Inside(np.flatnonzero(mask).astype(np.int64), distance[mask])


def outside(outer: TriMesh, points: object) -> Outside:
    """The complement of `inside`: the points whose winding number is at most 0.5, with their distance
    to `outer` in mm."""
    query = _points(points, "outside: points")
    if query.shape[0] == 0:
        return Outside(np.zeros(0, dtype=np.int64), np.zeros(0, dtype=np.float64))
    winding, distance = _winding_and_distance(outer, query)
    mask = winding <= INSIDE_THRESHOLD
    return Outside(np.flatnonzero(mask).astype(np.int64), distance[mask])


@dataclass(frozen=True)
class Closest:
    """For each query point: its distance to a surface in mm, the closest point on it (n, 3, metres), the
    index of the face that point lies on, and the surface's generalized winding number at the query point
    (about 1 inside, about 0 outside)."""

    distance_mm: Floats
    points: Floats
    faces: Ints
    winding: Floats


def closest(outer: TriMesh, points: object) -> Closest:
    """The closest point on `outer` to each query point (libigl's point-mesh distance, which returns the
    point and its face as well as the distance), with the winding number of `outer` at the query point.
    The push-out (`wardrobe/pushout.py`) moves a vertex towards or away from that point."""
    query = _points(points, "closest: points")
    if query.shape[0] == 0:
        empty = np.zeros(0, dtype=np.float64)
        return Closest(empty, np.zeros((0, 3), dtype=np.float64), np.zeros(0, dtype=np.int64), empty)
    igl = _load("igl")
    raw_w: Any = igl.fast_winding_number(outer.vertices, outer.faces, query)
    winding = np.asarray(raw_w, dtype=np.float64).reshape(-1)
    raw: Any = igl.point_mesh_squared_distance(query, outer.vertices, outer.faces)
    squared = np.asarray(raw[0], dtype=np.float64).reshape(-1)
    faces = np.asarray(raw[1], dtype=np.int64).reshape(-1)
    near = np.ascontiguousarray(np.asarray(raw[2], dtype=np.float64).reshape(-1, 3))
    n = query.shape[0]
    if winding.shape[0] != n or squared.shape[0] != n or faces.shape[0] != n or near.shape[0] != n:
        raise RuntimeError("libigl returned a result of the wrong length")
    return Closest(np.sqrt(squared) * _MM_PER_M, near, faces, winding)


def nearest(points: object, targets: object) -> Ints:
    """For each point, the index of the nearest target point (libigl's point-to-point distance)."""
    query = _points(points, "nearest: points")
    pool = _points(targets, "nearest: targets")
    if pool.shape[0] == 0:
        raise ValueError("nearest: no targets")
    if query.shape[0] == 0:
        return np.zeros(0, dtype=np.int64)
    igl = _load("igl")
    elements = np.arange(pool.shape[0], dtype=np.int64).reshape(-1, 1)
    raw: Any = igl.point_mesh_squared_distance(query, pool, elements)
    found = np.asarray(raw[1], dtype=np.int64).reshape(-1)
    if found.shape[0] != query.shape[0]:
        raise RuntimeError("libigl returned a result of the wrong length")
    return found


def lowest(points: object) -> float:
    """The lowest `y` (glTF up) of the points, in metres."""
    query = _points(points, "lowest: points")
    if query.shape[0] == 0:
        raise ValueError("lowest: no points")
    return float(query[:, 1].min())


def clearance(points: object, floor_y: float) -> float:
    """How far the lowest point is above the floor plane `y = floor_y`, in mm (negative: below it)."""
    return (lowest(points) - floor_y) * _MM_PER_M


def scene_parts(path: Path) -> list[RigidPart]:
    """Every mesh of a glTF scene (`.gltf` or `.glb`), as a rigid part with its world transform, read by
    trimesh (geometry in metres, glTF Y-up). A part is named by its glTF node name. A node whose mesh has
    several primitives arrives as several geometries; trimesh names their scene nodes with a random suffix,
    so they are named `<node>#<k>` instead, in the order of their (deterministic) geometry names."""
    trimesh = _load("trimesh")
    scene: Any = trimesh.load(str(path), force="scene", process=False)
    parents: Any = scene.graph.transforms.parents
    by_node: dict[str, list[tuple[str, Floats, TriMesh]]] = {}
    for leaf in (str(n) for n in scene.graph.nodes_geometry):
        raw: Any = scene.graph[leaf]
        geometry: Any = scene.geometry[raw[1]]
        if not hasattr(geometry, "faces") or len(geometry.faces) == 0:
            continue  # a point cloud or a line set: no surface to rest or overlap
        metadata: Any = getattr(geometry, "metadata", {})
        node = str(parents[leaf]) if metadata.get("from_gltf_primitive") else leaf
        mesh = TriMesh(
            np.asarray(geometry.vertices, dtype=np.float64), np.asarray(geometry.faces, dtype=np.int64)
        )
        by_node.setdefault(node, []).append((str(raw[1]), np.asarray(raw[0], dtype=np.float64), mesh))
    parts: list[RigidPart] = []
    for node, pieces in sorted(by_node.items()):
        pieces.sort(key=lambda piece: piece[0])
        for k, (_geometry, transform, mesh) in enumerate(pieces):
            parts.append(RigidPart(node if len(pieces) == 1 else f"{node}#{k}", mesh, transform))
    return parts


def world_vertices(part: RigidPart) -> Floats:
    """The part's vertices moved by its transform, (n, 3) metres."""
    v = part.mesh.vertices
    homogeneous = np.hstack([v, np.ones((v.shape[0], 1), dtype=np.float64)])
    moved: Floats = (homogeneous @ part.transform.T)[:, :3]
    return np.ascontiguousarray(moved)


def overlaps(parts: Sequence[RigidPart]) -> list[Overlap]:
    """Every pair of parts that touch or interpenetrate, with the deepest contact in mm, sorted by name.

    Uses trimesh's `CollisionManager` over python-fcl; a pair that only touches reports a depth near 0."""
    names = [part.name for part in parts]
    if len(set(names)) != len(names):
        raise ValueError(f"overlaps: part names are not unique: {sorted(names)}")
    if len(parts) < 2:
        return []
    trimesh = _load("trimesh")
    _load("fcl")
    collision: Any = importlib.import_module("trimesh.collision")
    manager: Any = collision.CollisionManager()
    for part in parts:
        mesh: Any = trimesh.Trimesh(vertices=part.mesh.vertices, faces=part.mesh.faces, process=False)
        manager.add_object(part.name, mesh, transform=part.transform)
    result: Any = manager.in_collision_internal(return_names=True, return_data=True)
    contacts: Any = result[2]
    deepest: dict[tuple[str, str], float] = {}
    for contact in contacts:
        pair_names = sorted(str(n) for n in contact.names)
        if len(pair_names) != 2:
            raise RuntimeError(f"python-fcl returned a contact with names {pair_names}")
        key = (pair_names[0], pair_names[1])
        depth = float(contact.depth) * _MM_PER_M
        deepest[key] = max(deepest.get(key, 0.0), depth)
    for pair in result[1]:
        a, b = sorted(str(n) for n in pair)
        deepest.setdefault((a, b), 0.0)
    return [Overlap(a, b, depth) for (a, b), depth in sorted(deepest.items())]
