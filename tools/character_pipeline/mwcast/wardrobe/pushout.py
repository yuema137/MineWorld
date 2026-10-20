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
   In a region the shell covers but does not enclose (the recipe's `open_regions`, S36-P3b), "outside" is
   C-2's drape rule (`contact_gates.drape_depth`: on the shell's outer side), not the winding number.

Parameters, not bounds. `gap` (2 mm) is about a cloth's thickness: ARC-138's L-b ruling. `reach` (40 mm) is
L-b's `SINK_MAX_M`. They say where a vertex goes. What passes is C-2's `garment.poke.body` and
`garment.poke.layer` in the tolerance file (I-S36-2).
"""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path

import numpy as np
from numpy.typing import NDArray

from .. import contact
from ..contact import TriMesh
from ..contact_gates import beneath, drape_depth, face_normals

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
WELD_M = 1e-6  # 0.001 mm: vertices closer than this are one place (a UV seam of a fused download)


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
        normals = face_normals(under)[found.faces[degenerate]]
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


@dataclass(frozen=True)
class Probe:
    """Rule 5's probed `under` vertices: their positions, their own outward normals, and whether each lies in
    a region the shell covers without enclosing it (judged by the drape rule)."""

    points: Floats
    normals: Floats
    draped: NDArray[np.bool_]

    @staticmethod
    def none() -> Probe:
        empty = np.zeros((0, 3), dtype=np.float64)
        return Probe(empty, empty, np.zeros(0, dtype=bool))


def _poking(shell: Floats, faces: Ints, probe: Probe, gap_m: float) -> tuple[Ints, Floats, Ints, Floats]:
    """The probes outside the shell by more than `gap - 0.5 mm` (the winding number where the shell encloses
    them; where it drapes them, the winding number and the drape rule both, at the smaller depth, as in
    C-2's `_away`): their indices, depths (m), closest shell faces and outward face normals."""
    mesh = TriMesh(shell, faces)
    found = contact.closest(mesh, probe.points)
    depth = found.distance_mm / _MM
    normals = face_normals(mesh)[found.faces]
    outward = probe.points - found.points
    normals = np.where((np.einsum("ij,ij->i", normals, outward) < 0.0)[:, None], -normals, normals)
    poking = ~probe.draped & (found.winding <= contact.INSIDE_THRESHOLD) & (depth > gap_m - _POKE_SLACK_M)
    if probe.draped.any():
        rows = np.flatnonzero(probe.draped)
        side_mm, side, _faces = drape_depth(mesh, probe.points[rows], probe.normals[rows])
        outside = found.winding[rows] <= contact.INSIDE_THRESHOLD
        depth[rows] = np.minimum(side_mm / _MM, depth[rows])
        normals[rows] = side
        poking[rows] = (
            outside & (side_mm / _MM > gap_m - _POKE_SLACK_M) & (depth[rows] > gap_m - _POKE_SLACK_M)
        )
    rows = np.flatnonzero(poking)
    return rows, depth[rows], found.faces[rows], normals[rows]


def _pokes(shell: Floats, faces: Ints, probe: Probe, gap_m: float) -> Floats:
    """Rule 5: the shell displacement that encloses the probed under-points poking out of the shell."""
    displacement = np.zeros_like(shell)
    if probe.points.shape[0] == 0:
        return displacement
    poking, depth, closest_faces, normals = _poking(shell, faces, probe, gap_m)
    if poking.size == 0:
        return displacement
    push = normals * (depth + gap_m)[:, None]
    size = np.linalg.norm(push, axis=1)
    # each of the face's three vertices takes the largest push that names it
    vertex = faces[closest_faces].T.reshape(-1)
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


def _still_poking(shell: Floats, faces: Ints, probe: Probe, gap_m: float) -> int:
    if probe.points.shape[0] == 0:
        return 0
    return int(_poking(shell, faces, probe, gap_m)[0].size)


def push_out(
    shell: TriMesh,
    under: TriMesh,
    gap_m: float = GAP_M,
    reach_m: float = REACH_M,
    passes: int = PASSES,
    probe: Ints | None = None,
    draped: NDArray[np.bool_] | None = None,
) -> tuple[Floats, PushReport]:
    """Push `shell`'s vertices out of `under` by the five rules (module docstring), at most `passes` times,
    until nothing needs moving. `probe` is the indices of `under`'s vertices that rule 5 holds inside the
    shell (C-2's flagged-under vertices in the shell's covered regions; default none); `draped` marks the
    probes in regions the shell covers without enclosing (the drape rule; default none). Returns the new shell
    vertices (the faces are unchanged) and the report; `unreached` indexes the shell's own vertices. `under`
    is never moved.

    A garment from a fused download carries a vertex once per UV island, so a seam is two rows of vertices at
    one place. The rules run on the welded shell (one vertex per place, `WELD_M`), so the copies of a seam
    vertex move together and the edge graph that smooths the field runs across the seam."""
    key = np.round(shell.vertices / WELD_M).astype(np.int64)
    _, first, welded = np.unique(key, axis=0, return_index=True, return_inverse=True)
    welded = welded.reshape(-1)
    faces = welded[shell.faces]
    keep = (faces[:, 0] != faces[:, 1]) & (faces[:, 1] != faces[:, 2]) & (faces[:, 2] != faces[:, 0])
    joined = TriMesh(shell.vertices[first], faces[keep])
    if probe is None or probe.size == 0:
        probed = Probe.none()
    else:
        probed = Probe(
            under.vertices[probe],
            contact.vertex_normals(under)[probe],
            np.zeros(probe.size, dtype=bool) if draped is None else np.asarray(draped, dtype=bool),
        )
    moved, report = _push_welded(joined, under, gap_m, reach_m, passes, probed)
    unreached = np.flatnonzero(np.isin(welded, report.unreached)).astype(np.int64)
    out = shell.vertices + (moved - joined.vertices)[welded]
    return np.ascontiguousarray(out), PushReport(
        report.passes, unreached, report.still_inside, report.still_poking, report.worst_mm
    )


def _push_welded(
    shell: TriMesh,
    under: TriMesh,
    gap_m: float,
    reach_m: float,
    passes: int,
    probed: Probe,
) -> tuple[Floats, PushReport]:
    vertices = shell.vertices.copy()
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
        # rule 5 on top: along each poke's direction the vertex moves at least the poke's push, and the
        # smoothed field's other components are kept (replacing it would drop a needed push)
        reach = np.linalg.norm(poke, axis=1)
        poked = reach > 0.0
        if poked.any():
            unit = poke[poked] / reach[poked, None]
            short = np.maximum(reach[poked] - np.einsum("ij,ij->i", moved[poked], unit), 0.0)
            moved[poked] += unit * short[:, None]
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


# --- The assembly's push-out: every part of an outfit, inner layer to outer (S36-P3 commit 2) ---------------


@dataclass(frozen=True)
class PartPlan:
    """One part as the push-out sees it: its id (the arrays' file prefix), layer (`part_layer`), class, the
    region names it covers, and whether its weights are rigid (a bag: pushed out, but it encloses nothing)."""

    id: str
    layer: float
    part_class: str
    covers: tuple[str, ...]
    rigid: bool
    open_regions: tuple[str, ...] = ()  # covered, not enclosed (the recipe's `open_regions`, S36-P3b)


@dataclass(frozen=True)
class _Surface:
    vertices: Floats
    faces: Ints
    region: Ints


def _load(out_dir: Path, name: str) -> tuple[Floats, Ints]:
    vertices = np.asarray(np.load(out_dir / f"{name}_fitted.npy"), dtype=np.float64).reshape(-1, 3)
    faces = np.asarray(np.load(out_dir / f"{name}_faces.npy"), dtype=np.int64).reshape(-1, 3)
    return vertices, faces


def _covered_faces(surface: _Surface, covered: Ints) -> Ints:
    """The faces with a vertex in a covered region, widened by one ring (so a sleeve's edge still sees the
    arm beyond the region's border)."""
    touching = np.isin(surface.region, covered)[surface.faces].any(axis=1)
    ring = np.zeros(surface.vertices.shape[0], dtype=bool)
    ring[surface.faces[touching].reshape(-1)] = True
    return surface.faces[ring[surface.faces].any(axis=1)]


def _unders(
    part: PartPlan, body: _Surface, lower: list[tuple[PartPlan, _Surface]], regions: tuple[str, ...]
) -> list[tuple[str, TriMesh, Ints, NDArray[np.bool_]]]:
    """The surfaces `part` is pushed out of, one at a time from the inside out: the body, then every lower
    part already pushed in layer order, each restricted to the covered regions widened by one ring, with the
    indices of its vertices in the covered regions (rule 5's candidates) and whether each lies in a region
    the part covers without enclosing (the drape rule). Hair is pushed out of the body.

    One at a time, because a winding number adds up over surfaces while a closest point picks one: a hoodie
    vertex 2 mm outside the skin but inside the tee is "inside" the body and tee together, and its closest
    point is the skin it already clears, so a joint surface would never move it out of the tee."""
    nothing = np.zeros(0, dtype=np.int64)
    flat = np.zeros(0, dtype=bool)
    if part.part_class == "hair":
        return [("body", TriMesh(body.vertices, body.faces), nothing, flat)]
    covered = np.array([i for i, name in enumerate(regions) if name in part.covers], dtype=np.int64)
    opened = np.array([i for i, name in enumerate(regions) if name in part.open_regions], dtype=np.int64)
    sources = [("body", body)] + [
        (plan.id, surface)
        for plan, surface in lower
        if plan.part_class != "hair" or part.part_class == "head"  # headwear goes over hair
    ]
    out: list[tuple[str, TriMesh, Ints, NDArray[np.bool_]]] = []
    for name, surface in sources:
        kept = _covered_faces(surface, covered)
        if kept.shape[0] == 0:
            continue
        used = np.zeros(surface.vertices.shape[0], dtype=bool)
        used[kept.reshape(-1)] = True
        candidates = np.flatnonzero(np.isin(surface.region, covered) & used).astype(np.int64)
        draped = np.isin(surface.region[candidates], opened)
        out.append((name, TriMesh(surface.vertices, kept), candidates, draped))
    if not out:  # the part covers nothing the body has: the whole body is still under it
        return [("body", TriMesh(body.vertices, body.faces), nothing, flat)]
    return out


def run(
    out_dir: Path, parts: list[PartPlan], regions: tuple[str, ...], passes: int = PASSES
) -> dict[str, object]:
    """Push every part out of what lies under it, inner layer first (`part_layer` order, ties by id), from the
    fit phase's arrays in `out_dir`; write `<id>_pushed.npy` per part and `pushout_report.json`. The body and
    every part already pushed stay where they are (I-P3-1). Rule 5 holds a candidate inside the part only when
    it lies under the part (`contact_gates.beneath`, the assembly's own rule asked of this part); hair and
    rigid parts enclose nothing, so rule 5 does not run for them."""
    body_vertices, body_faces = _load(out_dir, "body")
    body = _Surface(
        body_vertices,
        body_faces,
        np.asarray(np.load(out_dir / "body_region.npy"), dtype=np.int64).reshape(-1),
    )
    done: list[tuple[PartPlan, _Surface]] = []
    report: dict[str, object] = {}
    for part in sorted(parts, key=lambda p: (p.layer, p.id)):
        vertices, faces = _load(out_dir, part.id)
        steps: dict[str, object] = {}
        for name, under, candidates, draped in _unders(
            part, body, [d for d in done if d[0].layer < part.layer], regions
        ):
            shell = TriMesh(vertices, faces)
            probe: Ints | None = None
            probe_draped: NDArray[np.bool_] | None = None
            if not (part.part_class == "hair" or part.rigid) and candidates.size:
                kept = beneath(shell, under, candidates)
                probe, probe_draped = candidates[kept], draped[kept]
            vertices, result = push_out(shell, under, passes=passes, probe=probe, draped=probe_draped)
            steps[name] = _summary(result, probe)
        np.save(out_dir / f"{part.id}_pushed.npy", vertices)
        region = body.region[contact.nearest(vertices, body.vertices)]
        done.append((part, _Surface(vertices, faces, region)))
        report[part.id] = {"layer": part.layer, "vertices": int(vertices.shape[0]), "out_of": steps}
    (out_dir / "pushout_report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return report


def _summary(result: PushReport, probe: Ints | None) -> dict[str, object]:
    return {
        "moved": result.moved,
        "unreached": int(result.unreached.size),
        "still_inside": result.still_inside,
        "still_poking": result.still_poking,
        "probed": 0 if probe is None else int(probe.size),
        "worst_mm": round(result.worst_mm, 2),
        "passes": [
            {
                "moved": c.moved,
                "poke_moved": c.poke_moved,
                "unreached": c.unreached,
                "worst_mm": round(c.worst_mm, 2),
            }
            for c in result.passes
        ],
    }
