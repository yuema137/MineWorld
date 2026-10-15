"""Builds small synthetic GLB files in memory, so the gates and the reader run on hand-computable data."""

from __future__ import annotations

import json
import struct
from collections.abc import Mapping, Sequence

_IDENTITY_COLUMN_MAJOR = (1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0)


def _pad(data: bytearray, boundary: int, fill: bytes) -> None:
    while len(data) % boundary:
        data.extend(fill)


def build_glb(
    *,
    triangles: int = 1,
    materials: Sequence[str] = ("MW_Test",),
    joints: Sequence[tuple[str, str | None]] = (("Hips", None),),
    joint_positions: Mapping[str, tuple[float, float, float]] | None = None,
    ibm_overrides: Mapping[str, Sequence[float]] | None = None,
    weights_u16: bool = False,
    extras: Mapping[str, object] | None = None,
    image_png: bytes | None = None,
) -> bytes:
    """A skinned GLB with `triangles` triangles per material (one primitive per material).

    Joint `i` is node `i`, named and parented as `joints` says. Each vertex is bound to joint 0 with weight 1.
    Inverse bind matrices are translations to `joint_positions` (default: the origin), unless `ibm_overrides`
    gives a column-major 4×4 for a joint by name. `weights_u16` writes WEIGHTS_0 as normalised
    UNSIGNED_SHORT."""
    positions_map = dict(joint_positions or {})
    overrides = dict(ibm_overrides or {})
    vertex_count = 3 * triangles
    binary = bytearray()
    views: list[dict[str, object]] = []
    accessors: list[dict[str, object]] = []

    def add_view(payload: bytes) -> int:
        _pad(binary, 4, b"\x00")
        offset = len(binary)
        binary.extend(payload)
        views.append({"buffer": 0, "byteOffset": offset, "byteLength": len(payload)})
        return len(views) - 1

    def add_accessor(
        payload: bytes, *, component: int, count: int, kind: str, normalized: bool = False
    ) -> int:
        view = add_view(payload)
        entry: dict[str, object] = {
            "bufferView": view,
            "componentType": component,
            "count": count,
            "type": kind,
        }
        if normalized:
            entry["normalized"] = True
        accessors.append(entry)
        return len(accessors) - 1

    positions = b"".join(struct.pack("<3f", k * 0.001, 0.0, 0.0) for k in range(vertex_count))
    position_acc = add_accessor(positions, component=5126, count=vertex_count, kind="VEC3")
    uv_acc = add_accessor(
        b"".join(struct.pack("<2f", 0.5, 0.5) for _ in range(vertex_count)),
        component=5126,
        count=vertex_count,
        kind="VEC2",
    )
    joint_acc = add_accessor(
        bytes([0, 0, 0, 0]) * vertex_count, component=5121, count=vertex_count, kind="VEC4"
    )
    if weights_u16:
        weight_bytes = struct.pack("<4H", 65535, 0, 0, 0) * vertex_count
        weight_acc = add_accessor(
            weight_bytes, component=5123, count=vertex_count, kind="VEC4", normalized=True
        )
    else:
        weight_acc = add_accessor(
            struct.pack("<4f", 1.0, 0.0, 0.0, 0.0) * vertex_count,
            component=5126,
            count=vertex_count,
            kind="VEC4",
        )
    index_acc = add_accessor(
        struct.pack(f"<{3 * triangles}I", *range(3 * triangles)) if triangles else b"",
        component=5125,
        count=3 * triangles,
        kind="SCALAR",
    )
    ibm_bytes = bytearray()
    for name, _parent in joints:
        if name in overrides:
            matrix = [float(v) for v in overrides[name]]
            if len(matrix) != 16:
                raise ValueError(f"ibm override for {name} needs 16 values")
        else:
            x, y, z = positions_map.get(name, (0.0, 0.0, 0.0))
            matrix = list(_IDENTITY_COLUMN_MAJOR)
            matrix[12], matrix[13], matrix[14] = -x, -y, -z
        ibm_bytes.extend(struct.pack("<16f", *matrix))
    ibm_acc = add_accessor(bytes(ibm_bytes), component=5126, count=len(joints), kind="MAT4")

    images: list[dict[str, object]] = []
    if image_png is not None:
        view = add_view(image_png)
        images.append({"bufferView": view, "mimeType": "image/png"})

    nodes: list[dict[str, object]] = []
    children: dict[int, list[int]] = {}
    for index, (name, parent) in enumerate(joints):
        nodes.append({"name": name})
        if parent is not None:
            parent_index = [n for n, _ in joints].index(parent)
            children.setdefault(parent_index, []).append(index)
    for index, node in enumerate(nodes):
        if index in children:
            node["children"] = children[index]
    mesh_node = len(nodes)
    nodes.append({"name": "Body", "mesh": 0, "skin": 0})
    primitives: list[dict[str, object]] = []
    for material_index in range(len(materials)):
        primitives.append(
            {
                "attributes": {
                    "POSITION": position_acc,
                    "JOINTS_0": joint_acc,
                    "WEIGHTS_0": weight_acc,
                    "TEXCOORD_0": uv_acc,
                },
                "indices": index_acc,
                "material": material_index,
            }
        )
    scene: dict[str, object] = {"nodes": [*range(mesh_node + 1)]}
    if extras is not None:
        scene["extras"] = dict(extras)
    document: dict[str, object] = {
        "asset": {"version": "2.0"},
        "scene": 0,
        "scenes": [scene],
        "nodes": nodes,
        "meshes": [{"primitives": primitives}],
        "skins": [{"joints": list(range(len(joints))), "inverseBindMatrices": ibm_acc}],
        "accessors": accessors,
        "bufferViews": views,
        "buffers": [{"byteLength": len(binary)}],
        "materials": [{"name": name} for name in materials],
    }
    if images:
        document["images"] = images
    return _assemble(document, bytes(binary))


def _assemble(document: dict[str, object], binary: bytes) -> bytes:
    json_chunk = bytearray(json.dumps(document, separators=(",", ":")).encode("utf-8"))
    _pad(json_chunk, 4, b" ")
    bin_chunk = bytearray(binary)
    _pad(bin_chunk, 4, b"\x00")
    total = 12 + 8 + len(json_chunk) + 8 + len(bin_chunk)
    header = struct.pack("<III", 0x46546C67, 2, total)
    return (
        header
        + struct.pack("<II", len(json_chunk), 0x4E4F534A)
        + bytes(json_chunk)
        + struct.pack("<II", len(bin_chunk), 0x004E4942)
        + bytes(bin_chunk)
    )
