"""The static per-outfit gates (design §4.8; `ARC-141`): G-1, G-2, G-3, G-14 per assembled outfit, W-2, W-3,
and the no-stock scan V-1. Stdlib only; every bound is a literal here with its source (I-p-6), not an
argument.

An assembled outfit is one GLB (`blender/mpfb/assemble.py`): the body node `Body` and one skinned node per
part, `part_<id>`, each with one primitive and one material (`MW_Cast` for the body, `MW_Cast_<id>` per part),
and the parts' provenance in `scenes[0].extras.mineworld_provenance` (a JSON object, or a JSON string of one,
keyed by part id). The outfit's manifest gives each part's class (slot or hair) for W-2.
"""

from __future__ import annotations

import json
import math
import re
from collections.abc import Mapping
from dataclasses import dataclass
from typing import cast

from mwcast import standard
from mwcast.gates import GateId, GateResult, Verdict
from mwcast.glb import Glb, accessor, base_colour_image_index, embedded_image, png_size

# W-2 (step-32 §7.2): triangle budget per part class. Source: Vera's 95 k (D+ §5.2) and P-5's measured parts.
PART_BUDGET: dict[str, int] = {
    "body": 60_000,
    "hair": 12_000,
    "top": 10_000,
    "outer": 10_000,
    "legs": 8_000,
    "feet": 4_000,
    "socks": 1_500,
    "head": 4_000,  # an accessory
    "bag": 4_000,  # an accessory
}
MAX_PARTS = 8  # G-2 (step-32 §7.1): body, hair and up to six garments
PART_TEXTURE_PX = (1024, 2048)  # G-3 (step-32 §7.1): garments and hair, as each declares
TEXEL_TOL = 0.15  # W-3 (step-32 §7.2): within ±15 % of Vera's body texel density
FORBIDDEN_ROLES = ("fixture", "fitting-template")  # V-1: hidden data never drawn (R-S32h-p-1)


@dataclass(frozen=True)
class Part:
    node: str
    part_id: str  # "body" for the body
    mesh: int
    triangles: int
    materials: tuple[str, ...]


def _list(glb: Glb, key: str) -> list[dict[str, object]]:
    value = glb.document.get(key, [])
    return cast(list[dict[str, object]], value if isinstance(value, list) else [])


def parts(glb: Glb) -> list[Part]:
    """Every mesh node of the outfit: the body and each `part_<id>`."""
    meshes = _list(glb, "meshes")
    materials = _list(glb, "materials")
    out: list[Part] = []
    for node in _list(glb, "nodes"):
        if "mesh" not in node:
            continue
        name = str(node.get("name", ""))
        if name == "BodyCheck":
            continue  # W-4's unmasked reference body (assemble.py), never drawn
        mesh_index = int(cast(int, node["mesh"]))
        primitives = cast(list[dict[str, object]], meshes[mesh_index]["primitives"])
        tris = 0
        names: list[str] = []
        for prim in primitives:
            if "indices" in prim:
                tris += len(accessor(glb, int(cast(int, prim["indices"])))) // 3
            if "material" in prim:
                names.append(str(materials[int(cast(int, prim["material"]))].get("name", "")))
        part_id = name[len("part_") :] if name.startswith("part_") else "body"
        out.append(Part(node=name, part_id=part_id, mesh=mesh_index, triangles=tris, materials=tuple(names)))
    return out


def _result(gate: GateId, looked: list[str], reasons: list[str]) -> GateResult:
    return GateResult(
        gate=gate,
        verdict=Verdict.FAIL if reasons else Verdict.PASS,
        looked_at=tuple(looked),
        reasons=tuple(reasons),
    )


def gate_g1(glb: Glb) -> GateResult:
    found = parts(glb)
    total = sum(p.triangles for p in found)
    looked = [f"{p.node}: {p.triangles} triangles" for p in found] + [f"total {total}"]
    reasons = [f"{total} triangles > {standard.MAX_TRIANGLES}"] if total > standard.MAX_TRIANGLES else []
    return _result(GateId.G1, looked, reasons)


def gate_g2(glb: Glb) -> GateResult:
    found = parts(glb)
    reasons: list[str] = []
    if len(found) > MAX_PARTS:
        reasons.append(f"{len(found)} parts > {MAX_PARTS}")
    for p in found:
        expected = standard.MATERIAL if p.part_id == "body" else f"{standard.MATERIAL}_{p.part_id}"
        if p.materials != (expected,):
            reasons.append(f"{p.node}: materials {list(p.materials)}, expected exactly [{expected}]")
        if p.node in ("Hoodie", "Pack"):
            reasons.append(f"{p.node}: a name human.gd hides (F-S32p-4)")
    return _result(GateId.G2, [f"{p.node}: {list(p.materials)}" for p in found], reasons)


def gate_g3(glb: Glb) -> GateResult:
    found = parts(glb)
    meshes = _list(glb, "meshes")
    looked: list[str] = []
    reasons: list[str] = []
    for p in found:
        prim = cast(list[dict[str, object]], meshes[p.mesh]["primitives"])[0]
        if "material" not in prim:
            reasons.append(f"{p.node}: no material")
            continue
        try:
            image = base_colour_image_index(glb, int(cast(int, prim["material"])))
            width, height, _depth, _kind = png_size(embedded_image(glb, image))
        except Exception as exc:  # noqa: BLE001 - any unreadable texture is this gate's FAIL, with its reason
            reasons.append(f"{p.node}: base colour unreadable ({exc})")
            continue
        looked.append(f"{p.node}: {width}x{height}")
        allowed = (standard.TEXTURE_PX,) if p.part_id == "body" else PART_TEXTURE_PX
        if width != height or width not in allowed:
            reasons.append(f"{p.node}: {width}x{height}, allowed {allowed} square")
    return _result(GateId.G3, looked, reasons)


def gate_w2(glb: Glb, classes: Mapping[str, str]) -> GateResult:
    """W-2 per part; `classes` maps a part id to its slot or `hair`; the body is `body`."""
    looked: list[str] = []
    reasons: list[str] = []
    for p in parts(glb):
        kind = "body" if p.part_id == "body" else classes.get(p.part_id, "")
        budget = PART_BUDGET.get(kind)
        looked.append(f"{p.node} ({kind or 'unknown class'}): {p.triangles} / {budget}")
        if budget is None:
            reasons.append(f"{p.node}: no class in the manifest")
        elif p.triangles > budget:
            reasons.append(f"{p.node}: {p.triangles} triangles > {budget} ({kind})")
    return _result(GateId.W2, looked, reasons)


def _provenance(glb: Glb) -> dict[str, object]:
    scenes = _list(glb, "scenes")
    extras: object = scenes[0].get("extras") if scenes else None
    if not isinstance(extras, dict):
        return {}
    raw: object = cast(dict[str, object], extras).get("mineworld_provenance")
    if isinstance(raw, str):
        raw = cast(object, json.loads(raw))
    return cast(dict[str, object], raw) if isinstance(raw, dict) else {}


def _strings(value: object) -> list[str]:
    if isinstance(value, str):
        return [value]
    if isinstance(value, dict):
        return [s for v in cast(dict[str, object], value).values() for s in _strings(v)]
    if isinstance(value, list):
        return [s for v in cast(list[object], value) for s in _strings(v)]
    return []


def gate_g14(glb: Glb) -> GateResult:
    """Provenance on every part (the body's is #211's G-14 on the body export)."""
    prov = _provenance(glb)
    looked: list[str] = []
    reasons: list[str] = []
    for p in parts(glb):
        if p.part_id == "body":
            continue
        record = prov.get(p.part_id)
        looked.append(f"{p.node}: provenance {'present' if isinstance(record, dict) else 'missing'}")
        if not isinstance(record, dict):
            reasons.append(f"{p.node}: no provenance")
            continue
        rec = cast(dict[str, object], record)
        for key in ("source_type", "pipeline"):
            if key not in rec:
                reasons.append(f"{p.node}: provenance lacks {key}")
        for text in _strings(rec):
            if any(re.search(pattern, text) for pattern in standard.KEY_PATTERNS):
                reasons.append(f"{p.node}: a key-shaped string in the provenance")
    return _result(GateId.G14, looked, reasons)


def gate_v1(glb: Glb) -> GateResult:
    """V-1: every drawn part is a pilot asset: a Meshy task of the pilot, or a tight-fit shell of a pilot
    body. A part naming a stock MPFB2/MakeHuman asset, a fixture or the fitting template, or carrying no
    provenance, fails (I-p-4; AC-p-4)."""
    prov = _provenance(glb)
    looked: list[str] = []
    reasons: list[str] = []
    for p in parts(glb):
        if p.part_id == "body":
            continue
        record = prov.get(p.part_id)
        if not isinstance(record, dict):
            reasons.append(f"{p.node}: no provenance")
            continue
        rec = cast(dict[str, object], record)
        role = str(rec.get("role", ""))
        source = str(rec.get("source_type", ""))
        looked.append(
            f"{p.node}: source {source or '-'}, role {role or '-'}, meshy_task {rec.get('meshy_task')}"
        )
        if role in FORBIDDEN_ROLES:
            reasons.append(f"{p.node}: role {role} is never drawn")
        if source == "stock" or "makehuman" in json.dumps(rec).lower():
            reasons.append(f"{p.node}: a stock MPFB2/MakeHuman asset")
        meshy = rec.get("meshy_task")
        tight = rec.get("fit") == "tight" and bool(rec.get("tight_body"))
        if not (isinstance(meshy, str) and re.fullmatch(standard.UUID_PATTERN, meshy)) and not tight:
            reasons.append(f"{p.node}: neither a pilot Meshy task nor a tight-fit shell of a pilot body")
    return _result(GateId.V1, looked, reasons)


def texel_density(glb: Glb, mesh: int, texture_px: int) -> float:
    """Texels per metre of one mesh's first primitive: sqrt(UV area x texture_px^2 / surface area)."""
    prim = cast(list[dict[str, object]], _list(glb, "meshes")[mesh]["primitives"])[0]
    attributes = cast(dict[str, int], prim["attributes"])
    pos = accessor(glb, int(attributes["POSITION"]))
    uv = accessor(glb, int(attributes["TEXCOORD_0"]))
    idx = [int(i[0]) for i in accessor(glb, int(cast(int, prim["indices"])))]
    area3 = 0.0
    area2 = 0.0
    for k in range(0, len(idx) - 2, 3):
        a, b, c = idx[k], idx[k + 1], idx[k + 2]
        u = [pos[b][i] - pos[a][i] for i in range(3)]
        v = [pos[c][i] - pos[a][i] for i in range(3)]
        cross = (u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0])
        area3 += 0.5 * math.sqrt(sum(x * x for x in cross))
        area2 += 0.5 * abs(
            (uv[b][0] - uv[a][0]) * (uv[c][1] - uv[a][1]) - (uv[c][0] - uv[a][0]) * (uv[b][1] - uv[a][1])
        )
    if area3 <= 0.0:
        return 0.0
    return math.sqrt(area2 * texture_px * texture_px / area3)


def gate_w3(glb: Glb, reference: float | None) -> GateResult:
    """W-3: each part's texel density within ±15 % of the fused Vera's body density (the recipe's literal)."""
    if reference is None:
        return _result(
            GateId.W3, [], ["no reference density in the recipe (C3 derives it once from meshy_d.glb)"]
        )
    meshes = _list(glb, "meshes")
    looked: list[str] = []
    reasons: list[str] = []
    for p in parts(glb):
        if p.part_id == "body":
            continue
        prim = cast(list[dict[str, object]], meshes[p.mesh]["primitives"])[0]
        try:
            image = base_colour_image_index(glb, int(cast(int, prim["material"])))
            width, _h, _d, _k = png_size(embedded_image(glb, image))
            density = texel_density(glb, p.mesh, width)
        except Exception as exc:  # noqa: BLE001 - an unreadable part is this gate's FAIL, with its reason
            reasons.append(f"{p.node}: density unreadable ({exc})")
            continue
        ratio = density / reference
        looked.append(f"{p.node}: {density:.1f} texels/m ({ratio:.2f} of Vera's {reference:.1f})")
        if abs(ratio - 1.0) > TEXEL_TOL:
            reasons.append(f"{p.node}: {ratio:.2f} of the reference, outside ±{TEXEL_TOL:.0%}")
    return _result(GateId.W3, looked, reasons)
