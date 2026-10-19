"""Synthetic meshes with hand-computable answers for the contact tests (S36-P1). numpy only: no test imports a
mesh library (I-S36-1)."""

from __future__ import annotations

import math

import numpy as np
from numpy.typing import NDArray

# The 12 outward-facing (counter-clockwise seen from outside) triangles of the unit cube [0, 1]^3, over the
# corners indexed by bits (x, y, z) = (i & 1, i >> 1 & 1, i >> 2 & 1).
_CUBE_FACES = (
    (0, 2, 3), (0, 3, 1),  # z = 0, normal -z
    (4, 5, 7), (4, 7, 6),  # z = 1, normal +z
    (0, 1, 5), (0, 5, 4),  # y = 0, normal -y
    (2, 6, 7), (2, 7, 3),  # y = 1, normal +y
    (0, 4, 6), (0, 6, 2),  # x = 0, normal -x
    (1, 3, 7), (1, 7, 5),  # x = 1, normal +x
)  # fmt: skip


def box(
    low: tuple[float, float, float], high: tuple[float, float, float]
) -> tuple[NDArray[np.float64], NDArray[np.int64]]:
    """A closed axis-aligned box from `low` to `high`, metres, outward-facing."""
    corners = [
        (
            high[0] if i & 1 else low[0],
            high[1] if i >> 1 & 1 else low[1],
            high[2] if i >> 2 & 1 else low[2],
        )
        for i in range(8)
    ]
    return np.array(corners, dtype=np.float64), np.array(_CUBE_FACES, dtype=np.int64)


def open_box(
    low: tuple[float, float, float], high: tuple[float, float, float], *, drop: tuple[int, ...]
) -> tuple[NDArray[np.float64], NDArray[np.int64]]:
    """A box with the faces of the given sides removed (side k is triangles 2k and 2k+1, in the order above:
    -z, +z, -y, +y, -x, +x). An open trunk shell or a sleeve."""
    vertices, faces = box(low, high)
    keep = [row for i, row in enumerate(faces.tolist()) if i // 2 not in drop]
    return vertices, np.array(keep, dtype=np.int64)


def hemisphere(
    radius: float, rings: int = 16, segments: int = 32
) -> tuple[NDArray[np.float64], NDArray[np.int64]]:
    """An open hemispherical shell (the dome y >= 0, open at y = 0), centred at the origin, outward-facing."""
    vertices: list[tuple[float, float, float]] = [(0.0, radius, 0.0)]
    for ring in range(1, rings + 1):
        polar = (math.pi / 2) * ring / rings
        for segment in range(segments):
            azimuth = 2 * math.pi * segment / segments
            vertices.append(
                (
                    radius * math.sin(polar) * math.cos(azimuth),
                    radius * math.cos(polar),
                    radius * math.sin(polar) * math.sin(azimuth),
                )
            )
    faces: list[tuple[int, int, int]] = []

    def at(ring: int, segment: int) -> int:
        return 1 + (ring - 1) * segments + segment % segments

    for segment in range(segments):
        faces.append((0, at(1, segment + 1), at(1, segment)))
    for ring in range(1, rings):
        for segment in range(segments):
            a, b = at(ring, segment), at(ring, segment + 1)
            c, d = at(ring + 1, segment), at(ring + 1, segment + 1)
            faces.append((a, b, d))
            faces.append((a, d, c))
    return np.array(vertices, dtype=np.float64), np.array(faces, dtype=np.int64)
