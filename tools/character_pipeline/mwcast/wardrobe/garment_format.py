"""The runtime garment format as the pilot writes it (design §4.4; `ARC-140`, format part). Stdlib `struct`.

Files (little-endian):

- `binding.bin`  — magic `MWGB`, `u32` version 1, `u32` vertex count `n`, three scale references
  (`u32 a, u32 b, f32 rest` for x, y, z), then `n` records `u32 v0, v1, v2; f32 w0, w1, w2; f32 ox, oy, oz`.
- `hides.bin`    — magic `MWGH`, `u32` count, sorted unique `u32` template vertex ids.
- `corr.bin`     — magic `MWGC`, `u32` body vertex count, per body vertex `u32 t0, t1, t2; f32 b0, b1`
  (the closest template triangle by its three vertex ids; `b2 = 1 - b0 - b1`).
- `template.bin` — magic `MWGT`, `u32` vertex count (19,158 for `hm08`), positions `f32 x, y, z`.
- `template_weights.bin` — magic `MWGW`, `u32` vertex count, `u32` joint count `j`, `j` joint names (`u8`
  length + UTF-8), then per vertex 4 × (`u8` joint index into that table, `f32` weight).

Coordinates are the body mesh's own frame: glTF axes in metres, +Y up, the character facing +Z (the fused Vera
and every pipeline export; checked in C2 against the MPFB2 template, which faces the same way). These are
MakeHuman's own axes at 1/10 scale, so a `.mhclo` binding applies unchanged. The fitted position of garment
vertex `k` on template `T` is MakeHuman's proxy formula (`.mhclo`; step-32 §6.2):

    p_k = Σ_i w_i · T[v_i] + (ox · s_x, oy · s_y, oz · s_z),   s_x = |T[a_x].x − T[b_x].x| / rest_x  (y, z)

The offsets and the `rest` distances are in the `.mhclo`'s units (decimetres); the ratio carries the scale.
"""

from __future__ import annotations

import math
import struct
from collections.abc import Sequence
from dataclasses import dataclass
from pathlib import Path

Vec3 = tuple[float, float, float]

BINDING_MAGIC = b"MWGB"
HIDES_MAGIC = b"MWGH"
CORR_MAGIC = b"MWGC"
TEMPLATE_MAGIC = b"MWGT"
WEIGHTS_MAGIC = b"MWGW"
BINDING_VERSION = 1
WEIGHT_SUM_TOL = 1e-4  # W-1: weights sum to 1 within this (step-32 §7.2, the `.mhclo` format)


class FormatError(Exception):
    """A garment-format file is malformed, or fails W-1."""


@dataclass(frozen=True)
class ScaleRef:
    a: int
    b: int
    rest: float


@dataclass(frozen=True)
class BindRecord:
    verts: tuple[int, int, int]
    weights: Vec3
    offset: Vec3


@dataclass(frozen=True)
class Binding:
    scale: tuple[ScaleRef, ScaleRef, ScaleRef]  # x, y, z
    records: tuple[BindRecord, ...]


@dataclass(frozen=True)
class Corr:
    tri: tuple[int, int, int]
    b0: float
    b1: float


@dataclass(frozen=True)
class TemplateWeights:
    joints: tuple[str, ...]
    influences: tuple[tuple[tuple[int, float], ...], ...]  # per vertex, 4 (joint index, weight)


def _magic(data: bytes, magic: bytes, path: Path) -> None:
    if data[:4] != magic:
        raise FormatError(f"{path}: magic {data[:4]!r}, expected {magic!r}")


def write_binding(path: Path, binding: Binding) -> None:
    out = bytearray(BINDING_MAGIC)
    out += struct.pack("<II", BINDING_VERSION, len(binding.records))
    for ref in binding.scale:
        out += struct.pack("<IIf", ref.a, ref.b, ref.rest)
    for r in binding.records:
        out += struct.pack("<3I3f3f", *r.verts, *r.weights, *r.offset)
    path.write_bytes(bytes(out))


def read_binding(path: Path) -> Binding:
    data = path.read_bytes()
    _magic(data, BINDING_MAGIC, path)
    version, n = struct.unpack_from("<II", data, 4)
    if version != BINDING_VERSION:
        raise FormatError(f"{path}: version {version}, expected {BINDING_VERSION}")
    expected = 12 + 3 * 12 + n * 36
    if len(data) != expected:
        raise FormatError(f"{path}: {len(data)} bytes, expected {expected} for {n} records")
    refs = [ScaleRef(*struct.unpack_from("<IIf", data, 12 + 12 * i)) for i in range(3)]
    records: list[BindRecord] = []
    base = 48
    for k in range(n):
        values = struct.unpack_from("<3I3f3f", data, base + 36 * k)
        records.append(
            BindRecord(
                verts=(int(values[0]), int(values[1]), int(values[2])),
                weights=(float(values[3]), float(values[4]), float(values[5])),
                offset=(float(values[6]), float(values[7]), float(values[8])),
            )
        )
    return Binding(scale=(refs[0], refs[1], refs[2]), records=tuple(records))


def write_hides(path: Path, ids: Sequence[int]) -> None:
    unique = sorted(set(ids))
    path.write_bytes(HIDES_MAGIC + struct.pack(f"<I{len(unique)}I", len(unique), *unique))


def read_hides(path: Path) -> tuple[int, ...]:
    data = path.read_bytes()
    _magic(data, HIDES_MAGIC, path)
    (count,) = struct.unpack_from("<I", data, 4)
    if len(data) != 8 + 4 * count:
        raise FormatError(f"{path}: {len(data)} bytes for {count} ids")
    ids = struct.unpack_from(f"<{count}I", data, 8)
    if list(ids) != sorted(set(ids)):
        raise FormatError(f"{path}: ids are not sorted and unique")
    return tuple(int(i) for i in ids)


def write_corr(path: Path, corr: Sequence[Corr]) -> None:
    out = bytearray(CORR_MAGIC + struct.pack("<I", len(corr)))
    for c in corr:
        out += struct.pack("<3I2f", *c.tri, c.b0, c.b1)
    path.write_bytes(bytes(out))


def read_corr(path: Path) -> tuple[Corr, ...]:
    data = path.read_bytes()
    _magic(data, CORR_MAGIC, path)
    (n,) = struct.unpack_from("<I", data, 4)
    if len(data) != 8 + 20 * n:
        raise FormatError(f"{path}: {len(data)} bytes for {n} records")
    out: list[Corr] = []
    for k in range(n):
        t0, t1, t2, b0, b1 = struct.unpack_from("<3I2f", data, 8 + 20 * k)
        out.append(Corr(tri=(int(t0), int(t1), int(t2)), b0=float(b0), b1=float(b1)))
    return tuple(out)


def write_template(path: Path, positions: Sequence[Vec3]) -> None:
    out = bytearray(TEMPLATE_MAGIC + struct.pack("<I", len(positions)))
    for p in positions:
        out += struct.pack("<3f", *p)
    path.write_bytes(bytes(out))


def read_template(path: Path) -> tuple[Vec3, ...]:
    data = path.read_bytes()
    _magic(data, TEMPLATE_MAGIC, path)
    (n,) = struct.unpack_from("<I", data, 4)
    if len(data) != 8 + 12 * n:
        raise FormatError(f"{path}: {len(data)} bytes for {n} positions")
    values = struct.unpack_from(f"<{3 * n}f", data, 8)
    return tuple((values[3 * i], values[3 * i + 1], values[3 * i + 2]) for i in range(n))


def write_template_weights(path: Path, weights: TemplateWeights) -> None:
    out = bytearray(WEIGHTS_MAGIC + struct.pack("<II", len(weights.influences), len(weights.joints)))
    for name in weights.joints:
        encoded = name.encode("utf-8")
        out += struct.pack("<B", len(encoded)) + encoded
    for vertex in weights.influences:
        if len(vertex) != 4:
            raise FormatError(f"{path}: every vertex has exactly 4 influences")
        for joint, weight in vertex:
            out += struct.pack("<Bf", joint, weight)
    path.write_bytes(bytes(out))


def read_template_weights(path: Path) -> TemplateWeights:
    data = path.read_bytes()
    _magic(data, WEIGHTS_MAGIC, path)
    n, j = struct.unpack_from("<II", data, 4)
    offset = 12
    joints: list[str] = []
    for _ in range(j):
        (length,) = struct.unpack_from("<B", data, offset)
        joints.append(data[offset + 1 : offset + 1 + length].decode("utf-8"))
        offset += 1 + length
    if len(data) != offset + n * 20:
        raise FormatError(f"{path}: {len(data)} bytes for {n} vertices")
    influences: list[tuple[tuple[int, float], ...]] = []
    for k in range(n):
        values = struct.unpack_from("<BfBfBfBf", data, offset + 20 * k)
        influences.append(tuple((int(values[2 * i]), float(values[2 * i + 1])) for i in range(4)))
    return TemplateWeights(joints=tuple(joints), influences=tuple(influences))


def _scales(binding: Binding, template: Sequence[Vec3]) -> Vec3:
    out: list[float] = []
    for axis, ref in enumerate(binding.scale):
        out.append(abs(template[ref.a][axis] - template[ref.b][axis]) / ref.rest)
    return (out[0], out[1], out[2])


def fit(binding: Binding, template: Sequence[Vec3]) -> list[Vec3]:
    """The `.mhclo` formula: each garment vertex placed on template `T` (the rule `fit_at_load.gd` runs)."""
    sx, sy, sz = _scales(binding, template)
    out: list[Vec3] = []
    for r in binding.records:
        a, b, c = (template[v] for v in r.verts)
        w0, w1, w2 = r.weights
        out.append(
            (
                w0 * a[0] + w1 * b[0] + w2 * c[0] + r.offset[0] * sx,
                w0 * a[1] + w1 * b[1] + w2 * c[1] + r.offset[1] * sy,
                w0 * a[2] + w1 * b[2] + w2 * c[2] + r.offset[2] * sz,
            )
        )
    return out


def check_binding(binding: Binding, template_count: int) -> list[str]:
    """W-1: every record names three distinct template vertices in range, weights sum to 1 ± 1e-4, every scale
    reference names two distinct vertices in range with a positive rest distance. Returns the failures."""
    failures: list[str] = []
    for axis, ref in zip("xyz", binding.scale, strict=True):
        if not (0 <= ref.a < template_count and 0 <= ref.b < template_count) or ref.a == ref.b:
            failures.append(
                f"scale {axis}: vertices {ref.a}, {ref.b} invalid for a {template_count}-vertex template"
            )
        if not ref.rest > 0 or not math.isfinite(ref.rest):
            failures.append(f"scale {axis}: rest distance {ref.rest} is not positive")
    for k, r in enumerate(binding.records):
        rigid = len(set(r.verts)) == 1 and r.weights == (1.0, 0.0, 0.0)  # a `.mhclo` one-vertex record
        if (len(set(r.verts)) != 3 and not rigid) or not all(0 <= v < template_count for v in r.verts):
            failures.append(f"vertex {k}: template triangle {r.verts} invalid")
        total = sum(r.weights)
        if abs(total - 1.0) > WEIGHT_SUM_TOL or not all(math.isfinite(w) for w in (*r.weights, *r.offset)):
            failures.append(f"vertex {k}: weights sum to {total:.6f}")
    return failures


@dataclass(frozen=True)
class Mhclo:
    """What the pipeline reads from a `.mhclo`: the binding and `delete_verts` (MakeHuman's hidden set)."""

    binding: Binding
    delete_verts: tuple[int, ...]


def parse_mhclo(text: str, source: str) -> Mhclo:
    """Parse MakeClothes' `.mhclo`: `x_scale`/`y_scale`/`z_scale`, the `verts 0` block (three-vertex
    records; a one-vertex record is rigid, weight 1 and zero offset), and the `delete_verts` block (ids and
    `a - b` ranges)."""
    scales: dict[str, ScaleRef] = {}
    records: list[BindRecord] = []
    deletes: list[int] = []
    section = ""
    for number, raw in enumerate(text.splitlines(), start=1):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        words = line.split()
        if words[0] in ("x_scale", "y_scale", "z_scale"):
            scales[words[0][0]] = ScaleRef(int(words[1]), int(words[2]), float(words[3]))
            continue
        if words[0] == "verts":
            section = "verts"
            continue
        if words[0] == "delete_verts":
            section = "delete"
            continue
        if section == "verts" and words[0].lstrip("-").isdigit():
            if len(words) == 9:
                v = (int(words[0]), int(words[1]), int(words[2]))
                w = (float(words[3]), float(words[4]), float(words[5]))
                o = (float(words[6]), float(words[7]), float(words[8]))
                records.append(BindRecord(verts=v, weights=w, offset=o))
            elif len(words) == 1:
                i = int(words[0])
                records.append(BindRecord(verts=(i, i, i), weights=(1.0, 0.0, 0.0), offset=(0.0, 0.0, 0.0)))
            else:
                raise FormatError(f"{source}: line {number}: a verts record has {len(words)} fields")
            continue
        if section == "delete":
            tokens = line.replace(" - ", "-").split()
            for token in tokens:
                if "-" in token:
                    low, high = token.split("-")
                    deletes.extend(range(int(low), int(high) + 1))
                elif token.isdigit():
                    deletes.append(int(token))
            continue
        section = "" if section == "verts" else section
    if set(scales) != {"x", "y", "z"}:
        raise FormatError(f"{source}: x_scale, y_scale and z_scale are required")
    binding = Binding(scale=(scales["x"], scales["y"], scales["z"]), records=tuple(records))
    return Mhclo(binding=binding, delete_verts=tuple(sorted(set(deletes))))


def remap_joints(weights: TemplateWeights, order: Sequence[str]) -> TemplateWeights:
    """The weights with their joint table replaced by `order` (the body's glTF skin joint order), so that a
    joint index is a bind index of the body's Skin. A joint the body does not have refuses."""
    missing = sorted(set(weights.joints) - set(order))
    if missing:
        raise FormatError(f"the body's skin has no joints {missing}")
    position = {name: k for k, name in enumerate(order)}
    influences = tuple(
        tuple((position[weights.joints[j]], w) if w > 0.0 else (0, 0.0) for j, w in vertex)
        for vertex in weights.influences
    )
    return TemplateWeights(joints=tuple(order), influences=influences)


def max_distance(a: Sequence[Vec3], b: Sequence[Vec3]) -> float:
    """W-9's measure: the largest per-vertex distance between two fits of one garment."""
    if len(a) != len(b):
        raise FormatError(f"vertex counts differ: {len(a)} and {len(b)}")
    return max((math.dist(p, q) for p, q in zip(a, b, strict=True)), default=0.0)
