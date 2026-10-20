"""The garment push-out: move an outer shell out of the surfaces under it, by winding number (S36-P3).

An outfit is shells stacked on a body. Each shell must enclose what it covers: the body's skin and every lower
layer. A single ray along a vertex normal asks the wrong question on an open or folded shell (a hoodie vertex
3 mm inside the tee at the armpit whose sideways ray meets the arm first). The generalized winding number of
the `under` surface at a vertex answers "inside?" even at open cuffs and necks, and the closest point on
`under` says where the vertex must go (`contact.closest`, libigl). This module only moves the shell; the body
and every layer already fixed stay put (I-P3-1).

The rule, per pass:

1. For every shell vertex `v`: `w` = the winding number of `under` at `v`, `c` = the closest point on `under`,
   `d` = the distance.
2. `v` needs moving when `w > 0.5` (inside) or `d < gap` (outside, too close). A vertex inside by more than
   `reach` is not moved and is reported as unreached: it belongs to another limb or a failed registration.
3. The outward direction is `(c - v)/|c - v|` inside and `(v - c)/|v - c|` outside; when `|c - v|` is under
   0.01 mm, the normal of the closest face, with the sign whose side has the lower winding number.
4. The target is `c + n * gap`. The displacement field is smoothed over the shell's edge graph (3 rounds), and
   a needed displacement is never shrunk by the smoothing, so the shell does not crumple into spikes.
5. The other half of C-2: every probed `under` vertex that lies outside the shell by more than `gap - 0.5 mm`
   pushes the three vertices of the shell face closest to it outward along that face's normal by its depth
   plus `gap` (a sharp inner vertex can poke out between three outer vertices that are each outside `under`).

Parameters, not bounds. `gap` (2 mm) is about a cloth's thickness: ARC-138's L-b ruling. `reach` (40 mm) is
L-b's `SINK_MAX_M`. They say where a vertex goes. What passes is C-2's `garment.poke.body` and
`garment.poke.layer` in the tolerance file (I-S36-2).
"""

from __future__ import annotations

from dataclasses import dataclass, field

import numpy as np
from numpy.typing import NDArray

from .. import contact
from ..contact import TriMesh

Floats = NDArray[np.float64]
Ints = NDArray[np.int64]

GAP_M = 0.002  # ARC-138 (L-b): the cloth's thickness
REACH_M = 0.040  # L-b's SINK_MAX_M: deeper than this is another limb, not this layer's skin
PASSES = 3
SMOOTH_ROUNDS = 3
_DEGENERATE_M = 1e-5  # 0.01 mm: below this the closest point gives no direction
_SIDE_PROBE_M = 0.0005  # 0.5 mm: where the two sides of a face are sampled for their winding number
_POKE_SLACK_M = 0.0005  # rule 5 acts on pokes deeper than gap - 0.5 mm
_SETTLED_M = 1e-5  # 0.01 mm: a vertex this close to its target (gap) has arrived (floating-point slack)
_MM = 1000.0


@dataclass(frozen=True)
class PassCount:
    """One pass: vertices moved by rules 1-4, vertices moved by rule 5, vertices left unreached (inside by
    more than `reach`), and the largest displacement in mm."""

    moved: int
    poke_moved: int
    unreached: int
    worst_mm: float


@dataclass(frozen=True)
class PushReport:
    """What the push-out did, pass by pass, and what still fails after it: shell vertices still inside
    `under` (unreached ones excluded) and probed under-vertices still outside the shell beyond the gap."""

    passes: tuple[PassCount, ...]
    unreached: Ints
    still_inside: int
    still_poking: int
    worst_mm: float = field(default=0.0)

    @property
    def moved(self) -> int:
        return sum(p.moved + p.poke_moved for p in self.passes)


def _edges(faces: Ints, n: int) -> tuple[Ints, Ints, Floats]:
    """The shell's undirected edge graph as (source, target) pairs in both directions, and vertex degrees."""
    pairs = np.concatenate([faces[:, [0, 1]], faces[:, [1, 2]], faces[:, [2, 0]]])
    pairs = np.unique(np.sort(pairs, axis=1), axis=0)
    source = np.concatenate([pairs[:, 0], pairs[:, 1]])
    target = np.concatenate([pairs[:, 1], pairs[:, 0]])
    degree = np.bincount(source, minlength=n).astype(np.float64)
    return source, target, degree


def _face_normals(vertices: Floats, faces: Ints) -> Floats:
    a, b, c = vertices[faces[:, 0]], vertices[faces[:, 1]], vertices[faces[:, 2]]
    normals = np.cross(b - a, c - a)
    length = np.linalg.norm(normals, axis=1, keepdims=True)
    return np.asarray(normals / np.where(length > 0.0, length, 1.0), dtype=np.float64)


def _directions(
    vertices: Floats, under: TriMesh, found: contact.Closest, inside: NDArray[np.bool_]
) -> Floats:
    """Rule 3: the outward unit direction for every shell vertex."""
    offset = found.points - vertices  # v -> c
    length = np.linalg.norm(offset, axis=1)
    safe = np.where(length > _DEGENERATE_M, length, 1.0)[:, None]
    direction = np.where(inside[:, None], offset / safe, -offset / safe)
    degenerate = np.flatnonzero(length <= _DEGENERATE_M)
    if degenerate.size:
        normals = _face_normals(under.vertices, under.faces)[found.faces[degenerate]]
        near = found.points[degenerate]
        plus = contact.closest(under, near + normals * _SIDE_PROBE_M).winding
        minus = contact.closest(under, near - normals * _SIDE_PROBE_M).winding
        direction[degenerate] = np.where((plus <= minus)[:, None], normals, -normals)
    return np.asarray(direction, dtype=np.float64)


def _smooth(
    displacement: Floats,
    needed: NDArray[np.bool_],
    direction: Floats,
    required: Floats,
    graph: tuple[Ints, Ints, Floats],
) -> Floats:
    """Rule 4: each moved vertex and its 1-ring take the mean of their neighbours' displacements, for
    `SMOOTH_ROUNDS` rounds; a needed vertex keeps at least its required push along its direction."""
    source, target, degree = graph
    field_ = displacement.copy()
    for _ in range(SMOOTH_ROUNDS):
        moving = np.linalg.norm(field_, axis=1) > 0.0
        ring = moving.copy()
        ring[target[moving[source]]] = True
        total = np.zeros_like(field_)
        np.add.at(total, source, field_[target])
        mean = total / np.where(degree > 0.0, degree, 1.0)[:, None]
        field_ = np.where(ring[:, None], mean, field_)
        along = np.einsum("ij,ij->i", field_, direction)
        short = needed & (along < required)
        field_[short] += (required[short] - along[short])[:, None] * direction[short]
    return field_


def _pokes(shell: Floats, faces: Ints, under_points: Floats, gap_m: float) -> Floats:
    """Rule 5: the shell displacement that encloses the probed under-points poking out of the shell."""
    displacement = np.zeros_like(shell)
    if under_points.shape[0] == 0:
        return displacement
    found = contact.closest(TriMesh(shell, faces), under_points)
    depth = found.distance_mm / _MM
    poking = np.flatnonzero((found.winding <= contact.INSIDE_THRESHOLD) & (depth > gap_m - _POKE_SLACK_M))
    if poking.size == 0:
        return displacement
    normals = _face_normals(shell, faces)[found.faces[poking]]
    outward = under_points[poking] - found.points[poking]
    normals = np.where((np.einsum("ij,ij->i", normals, outward) < 0.0)[:, None], -normals, normals)
    push = normals * (depth[poking] + gap_m)[:, None]
    size = np.linalg.norm(push, axis=1)
    # each of the face's three vertices takes the largest push that names it
    vertex = faces[found.faces[poking]].T.reshape(-1)
    pushes = np.tile(push, (3, 1))
    order = np.lexsort((np.tile(size, 3), vertex))  # by vertex, then by size: the largest is last per vertex
    vertex, pushes = vertex[order], pushes[order]
    last = np.append(vertex[1:] != vertex[:-1], True)
    displacement[vertex[last]] = pushes[last]
    return displacement


def _needs(
    shell: Floats, under: TriMesh, gap_m: float, reach_m: float
) -> tuple[contact.Closest, NDArray[np.bool_], NDArray[np.bool_], NDArray[np.bool_]]:
    found = contact.closest(under, shell)
    distance = found.distance_mm / _MM
    inside = found.winding > contact.INSIDE_THRESHOLD
    unreached = inside & (distance > reach_m)
    needed = (inside | (distance < gap_m - _SETTLED_M)) & ~unreached
    return found, inside, needed, unreached


def _still_poking(shell: Floats, faces: Ints, under_points: Floats, gap_m: float) -> int:
    if under_points.shape[0] == 0:
        return 0
    found = contact.closest(TriMesh(shell, faces), under_points)
    outside = found.winding <= contact.INSIDE_THRESHOLD
    return int(np.count_nonzero(outside & (found.distance_mm / _MM > gap_m - _POKE_SLACK_M)))


def push_out(
    shell: TriMesh,
    under: TriMesh,
    gap_m: float = GAP_M,
    reach_m: float = REACH_M,
    passes: int = PASSES,
    probe: Ints | None = None,
) -> tuple[Floats, PushReport]:
    """Push `shell`'s vertices out of `under` by the five rules (module docstring), at most `passes` times,
    until nothing needs moving. `probe` is the indices of `under`'s vertices that rule 5 holds inside the
    shell (C-2's flagged-under vertices in the shell's covered regions; default none). Returns the new shell
    vertices (the faces are unchanged) and the report. `under` is never moved."""
    vertices = shell.vertices.copy()
    probed = under.vertices[probe] if probe is not None else np.zeros((0, 3), dtype=np.float64)
    graph = _edges(shell.faces, vertices.shape[0])
    counts: list[PassCount] = []
    unreached = np.zeros(vertices.shape[0], dtype=bool)
    worst = 0.0
    for _ in range(passes):
        found, inside, needed, unreached = _needs(vertices, under, gap_m, reach_m)
        poke = _pokes(vertices, shell.faces, probed, gap_m)
        poke_moved = int(np.count_nonzero(np.linalg.norm(poke, axis=1) > 0.0))
        if not needed.any() and poke_moved == 0:
            break
        direction = _directions(vertices, under, found, inside)
        target = found.points + direction * gap_m
        displacement = np.where(needed[:, None], target - vertices, 0.0)
        required = np.einsum("ij,ij->i", displacement, direction)
        moved = _smooth(displacement, needed, direction, required, graph)
        moved[unreached] = 0.0
        # rule 5 on top: a poke's push is kept where it is larger than the smoothed field
        larger = np.linalg.norm(poke, axis=1) > np.linalg.norm(moved, axis=1)
        moved = np.where(larger[:, None], poke, moved)
        size = np.linalg.norm(moved, axis=1)
        pass_worst = float(size.max()) * _MM if size.size else 0.0
        worst = max(worst, pass_worst)
        vertices = vertices + moved
        counts.append(
            PassCount(int(np.count_nonzero(needed)), poke_moved, int(np.count_nonzero(unreached)), pass_worst)
        )
    _found, _inside, needed, unreached = _needs(vertices, under, gap_m, reach_m)
    report = PushReport(
        passes=tuple(counts),
        unreached=np.flatnonzero(unreached).astype(np.int64),
        still_inside=int(np.count_nonzero(needed & _inside)),
        still_poking=_still_poking(vertices, shell.faces, probed, gap_m),
        worst_mm=worst,
    )
    return np.ascontiguousarray(vertices), report
