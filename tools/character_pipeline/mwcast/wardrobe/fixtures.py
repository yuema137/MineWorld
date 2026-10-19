"""Fixture garments and the formula oracle (C3 validation; design §4.8 W-1, W-5, W-9). Stdlib only.

A stock CC0 garment (a test fixture, `DEP-72`) comes as a `.mhclo` bound to the neutral `hm08`. The fixtures
stage (`blender/mpfb/fixtures.py`) exports its neutral fit as `garment.glb` and writes MPFB2's own fits of it
on every fixture body. This module:

- **expands** the `.mhclo` binding to `garment.glb`'s vertex order: the glTF exporter splits a vertex at a UV
  seam, so each exported vertex is matched to the source vertex at the same neutral position (within 1e-5 m);
  a vertex with no match refuses the binding;
- writes `binding.bin` (MakeHuman's axes are the body frame's: no conversion);
- runs the **oracle**: our formula on a fixture body's exact template against MPFB2's own fit of the same
  garment on the same body. Their largest per-vertex distance is the formula's error; it must be below
  `ORACLE_M`.
"""

from __future__ import annotations

from collections.abc import Sequence
from pathlib import Path
from typing import cast

from mwcast.glb import Glb, accessor, read_glb
from mwcast.wardrobe import garment_format as gf

MATCH_M = 1e-5  # two exported positions within this of a source position are the same vertex
ORACLE_M = 1e-4  # 0.1 mm: our formula vs MPFB2's own fit (float32 data; W-9 allows 0.5 mm end to end)


class FixtureError(Exception):
    """A fixture could not be converted or checked."""


def primitive_positions(glb: Glb, mesh: int = 0, primitive: int = 0) -> list[gf.Vec3]:
    meshes = cast(list[dict[str, object]], glb.document["meshes"])
    primitives = cast(list[dict[str, object]], meshes[mesh]["primitives"])
    attributes = cast(dict[str, int], primitives[primitive]["attributes"])
    return [(p[0], p[1], p[2]) for p in accessor(glb, int(attributes["POSITION"]))]


def _key(p: gf.Vec3) -> tuple[int, int, int]:
    return (round(p[0] / MATCH_M), round(p[1] / MATCH_M), round(p[2] / MATCH_M))


def expand_binding(binding: gf.Binding, source: Sequence[gf.Vec3], exported: Sequence[gf.Vec3]) -> gf.Binding:
    """The binding of `source` (one record per source vertex) re-ordered and repeated for `exported`."""
    if len(binding.records) != len(source):
        raise FixtureError(f"{len(binding.records)} binding records for {len(source)} source vertices")
    index: dict[tuple[int, int, int], int] = {}
    for i, p in enumerate(source):
        index.setdefault(_key(p), i)
    records: list[gf.BindRecord] = []
    for k, p in enumerate(exported):
        found = index.get(_key(p))
        if found is None:
            # A rounding boundary: look at the neighbouring cells.
            kx, ky, kz = _key(p)
            for dx in (-1, 0, 1):
                for dy in (-1, 0, 1):
                    for dz in (-1, 0, 1):
                        found = found if found is not None else index.get((kx + dx, ky + dy, kz + dz))
        if found is None:
            raise FixtureError(f"exported vertex {k} at {p} matches no source vertex")
        records.append(binding.records[found])
    return gf.Binding(scale=binding.scale, records=tuple(records))


def convert(mhclo: Path, neutral_fit: Path, garment_glb: Path, out: Path) -> gf.Binding:
    """`.mhclo` + MPFB2's neutral fit (source order) + the exported GLB -> `binding.bin` (body frame)."""
    parsed = gf.parse_mhclo(mhclo.read_text(encoding="utf-8"), str(mhclo))
    source = gf.read_template(neutral_fit)
    exported = primitive_positions(read_glb(garment_glb))
    binding = expand_binding(parsed.binding, source, exported)
    gf.write_binding(out, binding)
    return binding


def oracle(binding: gf.Binding, template: Sequence[gf.Vec3], mpfb_fit: Sequence[gf.Vec3]) -> float:
    """Largest distance between our formula's fit and MPFB2's own fit of the same garment on the same body."""
    return gf.max_distance(gf.fit(binding, template), mpfb_fit)
