"""The scene check: the props a client drew against the place's disclosed solids (S36-P5 commit 6).

Input is the dump `slice_probe_props.gd` writes, one `props-<place>.json` per drawn place: the place's
floor and solids in the scene's frame (metres, Y up), the body band, and per drawn prop its model slug,
the members it kept, its global 4×4 and nine support samples (the height of the drawn surface under its
footprint). Each prop's geometry is loaded with `contact.scene_parts`, filtered to its members and moved
by its transform. Three rules:

1. **Inside the solids.** The prop's vertices, and samples every `EDGE_STEP_M` along its edges, that lie
   in the body band and on the walkable floor must lie within `scene.mesh_over_solid` of the union of
   the place's solid footprints, or within that distance of the floor's edge (a wall clock). The prop's
   link is the solid holding most of those points. A prop with none of them in any solid, and some away
   from the floor's edge, is "drawn where no solid is", unless `contact.json` declares it
   `walk_through`.
2. **Apart.** Every pair of props whose bounds meet: `contact.overlaps` over both props' parts; a depth
   beyond `item.overlap.parts` fails, naming both props and parts. Pairs inside one prop are C-4's.
3. **On their support.** `floor` and `surface`: the lowest point lies within `item.rest.floor` of the
   support sample nearest it. `ground`: the rest plane does. `wall` and `ceiling`: listed, not judged.

Invariant I-P5-2: nothing here reads a prop's, place's or world's name to decide. A slug only finds the
model and its `contact.json` entry; the probe's walk stop only scopes a recorded finding. The geometry
is asked through `contact.py` (I-S36-1) and every bound comes from the tolerance file (I-S36-2).
"""

from __future__ import annotations

import json
from collections.abc import Mapping, Sequence
from dataclasses import dataclass
from pathlib import Path
from typing import cast

import numpy as np
from numpy.typing import NDArray

from mwcast import contact
from mwcast.contact import ContactUnavailable, RigidPart
from mwcast.contact_gates import FINDING_SLACK_MM, ItemSpec, load_mounts, prop_files
from mwcast.tolerances import Tolerances

Floats = NDArray[np.float64]

EDGE_STEP_M = 0.02  # rule 1 samples every edge this often, besides the vertices
_MM = 1000.0
# Godot replaces these characters in a node name (`Node.validate_node_name`); a glTF node name is
# compared the same way.
_NODE_NAME_INVALID = '.:@/"%'

RULES = ("inside", "apart", "support")


@dataclass(frozen=True)
class Footprint:
    """A solid in the scene's frame: its footprint (x, z) and its height above the place's floor, metres."""

    x0: float
    z0: float
    x1: float
    z1: float
    height: float


@dataclass(frozen=True)
class DrawnProp:
    slug: str
    name: str  # the node's path in the scene: printed, never read
    parts: tuple[str, ...] | None  # `SliceProps.keep`'s suffixes, or None when the whole file is drawn
    nodes: tuple[str, ...]  # the node's children as drawn (after `keep`)
    transform: Floats
    support: tuple[float | None, ...]


@dataclass(frozen=True)
class DrawnPlace:
    path: Path
    place: str
    base_y: float
    floor: Footprint  # height unused
    solids: tuple[Footprint, ...]
    band_low: float
    band_high: float
    props: tuple[DrawnProp, ...]
    # The probe's stop in its walk ("GE-1 café"), or the place id when the dump has none: what a row
    # names and a recorded finding is scoped by. A place id is renumbered when a world gains a place,
    # and differs between worlds drawn by the same scene; the walk's stop does not.
    scope: str = ""


@dataclass(frozen=True)
class Row:
    """One judged (or listed) fact. `verdict` is PASS, FAIL, KNOWN (a recorded finding) or LISTED (not
    judged)."""

    place: str
    prop: str
    rule: str
    measured: str
    bound: str
    verdict: str
    note: str = ""
    mm: float = 0.0  # the measured value a recorded finding is compared with

    def line(self) -> str:
        tail = f"  {self.note}" if self.note else ""
        return f"{self.place}  {self.prop}  {self.rule}  {self.measured}  {self.bound}  {self.verdict}{tail}"


# --- reading the dump ---------------------------------------------------------------------------------------


def _floats(value: object, n: int, what: str) -> list[float]:
    if not isinstance(value, list) or len(cast(list[object], value)) != n:
        raise ValueError(f"{what}: expected {n} numbers")
    return [float(cast(float, v)) for v in cast(list[object], value)]


def _object(value: object, what: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise ValueError(f"{what}: expected an object")
    return cast(dict[str, object], value)


def load_place(path: Path) -> DrawnPlace:
    """One `props-<place>.json`. A malformed file is refused by name."""
    try:
        top = _object(json.loads(path.read_text(encoding="utf-8")), str(path))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise ValueError(f"{path}: {exc}") from exc
    if top.get("format") != 1:
        raise ValueError(f"{path}: expected format 1")
    scene = _object(top.get("scene"), f"{path} scene")
    floor = _object(scene.get("floor"), f"{path} scene.floor")
    lo, hi = (
        _floats(floor.get("min"), 2, f"{path} floor.min"),
        _floats(floor.get("max"), 2, f"{path} floor.max"),
    )
    solids: list[Footprint] = []
    for i, raw in enumerate(cast(list[object], scene.get("solids", []))):
        solid = _object(raw, f"{path} solid {i}")
        a = _floats(solid.get("min"), 2, f"{path} solid {i} min")
        b = _floats(solid.get("max"), 2, f"{path} solid {i} max")
        solids.append(Footprint(a[0], a[1], b[0], b[1], float(cast(float, solid.get("height")))))
    band = _object(top.get("band"), f"{path} band")
    props: list[DrawnProp] = []
    for i, raw in enumerate(cast(list[object], top.get("props", []))):
        prop = _object(raw, f"{path} prop {i}")
        parts_raw = prop.get("parts")
        support = [
            None if s is None else float(cast(float, s))
            for s in (cast(list[object], row)[2] for row in cast(list[object], prop.get("support", [])))
        ]
        props.append(
            DrawnProp(
                str(prop.get("slug")),
                str(prop.get("name", f"prop {i}")),
                None if parts_raw is None else tuple(str(p) for p in cast(list[object], parts_raw)),
                tuple(str(n) for n in cast(list[object], prop.get("nodes", []))),
                np.asarray(_floats(prop.get("transform"), 16, f"{path} prop {i} transform")).reshape(4, 4),
                tuple(support),
            )
        )
    return DrawnPlace(
        path,
        str(top.get("place")),
        float(cast(float, scene.get("base_y"))),
        Footprint(lo[0], lo[1], hi[0], hi[1], 0.0),
        tuple(solids),
        float(cast(float, band.get("low"))),
        float(cast(float, band.get("high"))),
        tuple(props),
        str(top.get("label") or top.get("place")),
    )


def dump_files(directory: Path) -> list[Path]:
    """Every `props-*.json` under `directory`, recursively (one subdirectory per world is the CI layout)."""
    return sorted(directory.rglob("props-*.json"))


# --- geometry -----------------------------------------------------------------------------------------------


def _godot_name(name: str) -> str:
    for ch in _NODE_NAME_INVALID:
        name = name.replace(ch, "_")
    return name


def drawn_parts(prop: DrawnProp, library: Sequence[RigidPart], index: int) -> list[RigidPart]:
    """The prop's parts as drawn: filtered to the kept members when `keep` trimmed it, moved by its
    transform, and named `<index>:<slug>:<part>` so two props' parts never share a name."""
    chosen = list(library)
    if prop.parts is not None:
        kept = {_godot_name(n) for n in prop.nodes}
        chosen = [p for p in library if _godot_name(p.name.split("#")[0]) in kept]
    return [RigidPart(f"{index}:{prop.slug}:{p.name}", p.mesh, prop.transform @ p.transform) for p in chosen]


def surface_points(parts: Sequence[RigidPart], step: float = EDGE_STEP_M) -> Floats:
    """Every vertex of the parts, in the scene, and points every `step` metres along every edge."""
    chunks: list[Floats] = []
    for part in parts:
        world = contact.world_vertices(part)
        chunks.append(world)
        faces = part.mesh.faces
        edges = np.unique(
            np.sort(np.vstack([faces[:, [0, 1]], faces[:, [1, 2]], faces[:, [2, 0]]]), axis=1), axis=0
        )
        a, b = world[edges[:, 0]], world[edges[:, 1]]
        counts = np.floor(np.linalg.norm(b - a, axis=1) / step).astype(np.int64)
        long = counts > 0
        if not np.any(long):
            continue
        a, b, counts = a[long], b[long], counts[long]
        owner: NDArray[np.int64] = np.repeat(np.arange(counts.shape[0], dtype=np.int64), counts)
        starts = np.cumsum(counts) - counts
        k = np.arange(owner.shape[0], dtype=np.int64) - starts[owner] + 1
        t: Floats = np.asarray(k / (counts[owner] + 1), dtype=np.float64).reshape(-1, 1)
        chunks.append(a[owner] + (b[owner] - a[owner]) * t)
    if not chunks:
        return np.zeros((0, 3), dtype=np.float64)
    return np.vstack(chunks)


def footprint_distance_mm(points: Floats, solids: Sequence[Footprint]) -> tuple[Floats, NDArray[np.int64]]:
    """For each point (scene x, z), its distance in mm to the nearest solid footprint (0 inside), and the
    index of the solid that contains it (-1: none)."""
    n = points.shape[0]
    if not solids:
        return np.full(n, np.inf), np.full(n, -1, dtype=np.int64)
    box = np.asarray([[s.x0, s.z0, s.x1, s.z1] for s in solids], dtype=np.float64)
    x, z = points[:, 0:1], points[:, 2:3]
    dx = np.maximum(np.maximum(box[None, :, 0] - x, 0.0), x - box[None, :, 2])
    dz = np.maximum(np.maximum(box[None, :, 1] - z, 0.0), z - box[None, :, 3])
    distance = np.hypot(dx, dz) * _MM
    inside = distance <= 0.0
    holder = np.where(inside.any(axis=1), np.argmax(inside, axis=1), -1).astype(np.int64)
    return distance.min(axis=1), holder


# --- the rules ----------------------------------------------------------------------------------------------


def _finding(
    rows: list[Row], spec: ItemSpec | None, place: str, rule: str, measured: float, row: Row
) -> None:
    recorded = None if spec is None else spec.finding_scene.get((place, rule))
    if recorded is not None and measured <= recorded + FINDING_SLACK_MM:
        rows.append(
            Row(
                row.place,
                row.prop,
                row.rule,
                row.measured,
                row.bound,
                "KNOWN",
                f"{row.note}; recorded {recorded:.1f} mm in contact.json".lstrip("; "),
                row.mm,
            )
        )
    else:
        rows.append(row)


def _inside_rule(place: DrawnPlace, label: str, spec: ItemSpec | None, points: Floats, bound: float) -> Row:
    y = points[:, 1] - place.base_y
    f = place.floor
    on_floor = (points[:, 0] > f.x0) & (points[:, 0] < f.x1) & (points[:, 2] > f.z0) & (points[:, 2] < f.z1)
    chosen = points[(y >= place.band_low) & (y <= place.band_high) & on_floor]
    if chosen.shape[0] == 0:
        return Row(place.scope, label, "inside", "no point in the band on the floor", "-", "LISTED")
    if spec is not None and spec.walk_through:
        return Row(place.scope, label, "inside", f"{chosen.shape[0]} points", "-", "LISTED", "walk_through")
    solid_mm, holder = footprint_distance_mm(chosen, place.solids)
    edge_mm = (
        np.minimum(
            np.minimum(chosen[:, 0] - f.x0, f.x1 - chosen[:, 0]),
            np.minimum(chosen[:, 2] - f.z0, f.z1 - chosen[:, 2]),
        )
        * _MM
    )
    held = holder[holder >= 0]
    bound_text = f"scene.mesh_over_solid {bound:g} mm"
    if held.size == 0:
        if bool(np.all(edge_mm <= bound)):
            worst = float(edge_mm.max())
            return Row(
                place.scope,
                label,
                "inside",
                f"{worst:.1f} mm from the floor's edge",
                bound_text,
                "PASS",
                f"wall-held, {chosen.shape[0]} points",
            )
        nearest = float(solid_mm.min())
        return Row(
            place.scope,
            label,
            "inside",
            f"nearest solid {nearest:.1f} mm away",
            bound_text,
            "FAIL",
            f"drawn where no solid is ({chosen.shape[0]} points in the band)",
            nearest,
        )
    link = int(np.bincount(held).argmax())
    off = np.minimum(solid_mm, edge_mm)
    worst = float(off.max())
    over = int((off > bound).sum())
    margin = bound - worst
    note = f"linked to solid {link} ({held.size} of {chosen.shape[0]} points in it), margin {margin:.1f} mm"
    if worst > bound:
        return Row(
            place.scope,
            label,
            "inside",
            f"overhangs solid {link} by {worst:.1f} mm",
            bound_text,
            "FAIL",
            f"{note}; {over} points over",
            worst,
        )
    return Row(place.scope, label, "inside", f"overhang {worst:.1f} mm", bound_text, "PASS", note)


def _support_rule(
    place: DrawnPlace,
    label: str,
    spec: ItemSpec | None,
    prop: DrawnProp,
    parts: Sequence[RigidPart],
    bound: float,
) -> Row:
    if spec is None:
        return Row(place.scope, label, "support", "no mount declared in contact.json", "-", "FAIL")
    if spec.mount in ("wall", "ceiling"):
        return Row(
            place.scope, label, "support", f"mount {spec.mount}", "-", "LISTED", "not judged (no wall sample)"
        )
    hits = [s for s in prop.support if s is not None]
    bound_text = f"item.rest.floor {bound:g} mm"
    if not hits:
        return Row(place.scope, label, "support", "nothing drawn under it", bound_text, "FAIL")
    if spec.ground is not None:
        plane = float((prop.transform @ np.array([0.0, spec.ground.rest_mm / _MM, 0.0, 1.0]))[1])
        what = "rest plane"
    else:
        plane = min(contact.lowest(contact.world_vertices(p)) for p in parts)
        what = "lowest point"
    # The support is the sample nearest the plane: the nine samples cover the prop's bounds, so a corner
    # may meet a neighbour (a mat beside a stool's foot, the cup standing in a saucer, the floor below a
    # gap). A prop rests when one sample meets it; it floats or sinks by the nearest one's distance.
    gap = min(((plane - h) * _MM for h in hits), key=abs)
    side = "above" if gap > 0 else "below"
    verdict = "FAIL" if abs(gap) > bound else "PASS"
    return Row(
        place.scope,
        label,
        "support",
        f"{what} {abs(gap):.1f} mm {side} its support",
        bound_text,
        verdict,
        f"margin {bound - abs(gap):.1f} mm",
        abs(gap),
    )


def _bounds(parts: Sequence[RigidPart]) -> tuple[Floats, Floats]:
    world = np.vstack([contact.world_vertices(p) for p in parts])
    return world.min(axis=0), world.max(axis=0)


def check_place(
    place: DrawnPlace, mounts: Mapping[str, ItemSpec], files: Mapping[str, Path], tolerances: Tolerances
) -> list[Row]:
    """The three rules over one dumped place; one row per prop and rule, and one per failing pair."""
    over_solid = tolerances.bound("scene.mesh_over_solid")
    rest = tolerances.bound("item.rest.floor")
    apart = tolerances.bound("item.overlap.parts")
    library: dict[str, list[RigidPart]] = {}
    rows: list[Row] = []
    drawn: list[tuple[int, DrawnProp, str, list[RigidPart]]] = []
    for i, prop in enumerate(place.props):
        label = f"{prop.slug}#{i}"
        spec = mounts.get(prop.slug)
        if prop.slug not in files:
            rows.append(
                Row(place.scope, label, "inside", f"no model {prop.slug} in the library", "-", "FAIL")
            )
            continue
        if prop.slug not in library:
            library[prop.slug] = contact.scene_parts(files[prop.slug])
        parts = drawn_parts(prop, library[prop.slug], i)
        if not parts:
            rows.append(
                Row(
                    place.scope,
                    label,
                    "inside",
                    f"no member of {list(prop.nodes)} in the model",
                    "-",
                    "FAIL",
                    f"cannot look ({prop.name})",
                )
            )
            continue
        drawn.append((i, prop, label, parts))
        _record(
            rows,
            spec,
            place.scope,
            "inside",
            _inside_rule(place, label, spec, surface_points(parts), over_solid),
        )
        _record(rows, spec, place.scope, "support", _support_rule(place, label, spec, prop, parts, rest))
    rows.extend(_apart_rows(place, drawn, mounts, apart))
    return rows


def _record(rows: list[Row], spec: ItemSpec | None, place: str, rule: str, row: Row) -> None:
    if row.verdict != "FAIL":
        rows.append(row)
        return
    _finding(rows, spec, place, rule, row.mm if row.mm > 0.0 else float("inf"), row)


def _apart_rows(
    place: DrawnPlace,
    drawn: Sequence[tuple[int, DrawnProp, str, list[RigidPart]]],
    mounts: Mapping[str, ItemSpec],
    bound: float,
) -> list[Row]:
    rows: list[Row] = []
    boxes = [_bounds(parts) for _i, _p, _l, parts in drawn]
    for a in range(len(drawn)):
        for b in range(a + 1, len(drawn)):
            (lo_a, hi_a), (lo_b, hi_b) = boxes[a], boxes[b]
            if bool(np.any(hi_a < lo_b) or np.any(hi_b < lo_a)):
                continue
            _ia, prop_a, label_a, parts_a = drawn[a]
            _ib, prop_b, label_b, parts_b = drawn[b]
            spec_a, spec_b = mounts.get(prop_a.slug), mounts.get(prop_b.slug)
            if (spec_a is not None and spec_a.walk_through) or (spec_b is not None and spec_b.walk_through):
                continue  # a person passes through it; so may another prop
            names_a = {p.name for p in parts_a}
            hits = [
                hit
                for hit in contact.overlaps([*parts_a, *parts_b])
                if (hit.a in names_a) != (hit.b in names_a)
            ]
            if not hits:
                continue
            worst = max(hits, key=lambda h: h.depth_mm)
            pair = f"{label_a} / {label_b}"
            measured = f"{worst.depth_mm:.1f} mm deep"
            bound_text = f"item.overlap.parts {bound:g} mm"
            note = f"{worst.a.split(':', 2)[2]} / {worst.b.split(':', 2)[2]}"
            if worst.depth_mm <= bound:
                rows.append(
                    Row(place.scope, pair, "apart", measured, bound_text, "PASS", f"touching: {note}")
                )
                continue
            row = Row(
                place.scope,
                pair,
                "apart",
                measured,
                bound_text,
                "FAIL",
                f"interpenetrate: {note}",
                worst.depth_mm,
            )
            # recorded on either prop of the pair
            spec = spec_a if spec_a is not None and (place.scope, "apart") in spec_a.finding_scene else spec_b
            _finding(rows, spec, place.scope, "apart", worst.depth_mm, row)
    return rows


def stale_rows(
    places: Sequence[DrawnPlace], rows: Sequence[Row], mounts: Mapping[str, ItemSpec]
) -> list[Row]:
    """A recorded scene finding for a place that was dumped, whose prop no longer fails that rule there,
    is stale."""
    dumped = {p.scope for p in places}
    slugs_by_label: dict[tuple[str, str], str] = {}
    for p in places:
        for i, prop in enumerate(p.props):
            slugs_by_label[(p.scope, f"{prop.slug}#{i}")] = prop.slug
    failing: set[tuple[str, str, str]] = set()
    for row in rows:
        if row.verdict not in ("FAIL", "KNOWN"):
            continue
        for label in row.prop.split(" / "):
            slug = slugs_by_label.get((row.place, label))
            if slug is not None:
                failing.add((slug, row.place, row.rule))
    out: list[Row] = []
    for slug, spec in sorted(mounts.items()):
        for (place, rule), recorded in sorted(spec.finding_scene.items()):
            if place in dumped and (slug, place, rule) not in failing:
                out.append(
                    Row(
                        place,
                        slug,
                        rule,
                        f"recorded {recorded:.1f} mm",
                        "-",
                        "FAIL",
                        "stale finding: it passes now; remove it from contact.json",
                    )
                )
    return out


def run(directory: Path, root: Path, tolerances: Tolerances) -> tuple[list[DrawnPlace], list[Row]]:
    """Every dumped place under `directory`, judged with the model library at `root` and its
    `contact.json`."""
    paths = dump_files(directory)
    if not paths:
        raise ValueError(f"{directory}: no props-*.json (run the slice with --world --props first)")
    mounts = load_mounts(root / "contact.json")
    files = prop_files(root)
    places = [load_place(p) for p in paths]
    rows: list[Row] = []
    try:
        for place in places:
            rows.extend(check_place(place, mounts, files, tolerances))
    except ContactUnavailable as exc:
        rows.append(Row("-", "-", "-", f"cannot look: {exc}", "-", "FAIL"))
    rows.extend(stale_rows(places, rows, mounts))
    return places, rows
