"""GLB reader for the character gates and the parity comparison (PIPELINE.md §§6–7).

Standard library only. It reads what the gates read: the JSON chunk, the BIN chunk, accessors with their
component types and normalisation, skins, and embedded image bytes. It knows nothing about a character, a
bound or a verdict.
"""

from __future__ import annotations

import json
import struct
from collections.abc import Mapping
from dataclasses import dataclass
from pathlib import Path
from typing import cast

GLB_MAGIC = 0x46546C67  # "glTF" read as a little-endian uint32
JSON_CHUNK = 0x4E4F534A
BIN_CHUNK = 0x004E4942
TRIANGLES_MODE = 4

# glTF component type -> (struct format character, byte size).
_COMPONENT_FORMAT: dict[int, tuple[str, int]] = {
    5120: ("b", 1),
    5121: ("B", 1),
    5122: ("h", 2),
    5123: ("H", 2),
    5125: ("I", 4),
    5126: ("f", 4),
}
# Normalised integer types: the divisor glTF 2.0 gives for each component type (section 3.11.2).
_NORMALISER: dict[int, float] = {5120: 127.0, 5121: 255.0, 5122: 32767.0, 5123: 65535.0, 5125: 4294967295.0}
_TYPE_WIDTH: dict[str, int] = {"SCALAR": 1, "VEC2": 2, "VEC3": 3, "VEC4": 4, "MAT4": 16}
_PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"


class GlbError(Exception):
    """A file is not a readable GLB, or a part of it is not what the reader needs. The message names it."""


@dataclass(frozen=True)
class Glb:
    """A parsed GLB: the glTF JSON document and the BIN chunk (empty when absent)."""

    path: Path
    document: Mapping[str, object]
    binary: bytes


def read_glb(path: Path) -> Glb:
    """Read and split a GLB file. Refuses a bad magic, a version other than 2, a truncated chunk, an
    unknown chunk type, or a file without a JSON chunk."""
    data = path.read_bytes()
    if len(data) < 12:
        raise GlbError(f"{path}: shorter than the 12-byte GLB header ({len(data)} bytes)")
    magic, version, length = struct.unpack_from("<III", data, 0)
    if magic != GLB_MAGIC:
        raise GlbError(f"{path}: magic is {magic:#010x}, not glTF")
    if version != 2:
        raise GlbError(f"{path}: GLB version {version}, expected 2")
    if length > len(data):
        raise GlbError(f"{path}: header declares {length} bytes but the file has {len(data)} (truncated)")
    offset = 12
    document: dict[str, object] | None = None
    binary = b""
    while offset < length:
        if offset + 8 > length:
            raise GlbError(f"{path}: chunk header at byte {offset} runs past the declared length")
        chunk_length, chunk_type = struct.unpack_from("<II", data, offset)
        body_start = offset + 8
        body_end = body_start + chunk_length
        if body_end > length:
            raise GlbError(
                f"{path}: chunk at byte {offset} ({chunk_length} bytes) runs past the declared length"
            )
        body = data[body_start:body_end]
        if chunk_type == JSON_CHUNK:
            if document is not None:
                raise GlbError(f"{path}: a second JSON chunk at byte {offset}")
            try:
                parsed: object = json.loads(body.decode("utf-8"))
            except (UnicodeDecodeError, json.JSONDecodeError) as exc:
                raise GlbError(f"{path}: JSON chunk is not valid UTF-8 JSON: {exc}") from exc
            if not isinstance(parsed, dict):
                raise GlbError(f"{path}: JSON chunk is not an object")
            document = cast(dict[str, object], parsed)
        elif chunk_type == BIN_CHUNK:
            binary = body
        else:
            raise GlbError(f"{path}: unknown chunk type {chunk_type:#010x} at byte {offset}")
        offset = body_end
    if document is None:
        raise GlbError(f"{path}: no JSON chunk")
    return Glb(path=path, document=document, binary=binary)


def _at(glb: Glb, value: object, where: str) -> Mapping[str, object]:
    if not isinstance(value, dict):
        raise GlbError(f"{glb.path}: {where} is not an object")
    return cast(Mapping[str, object], value)


def _list(glb: Glb, container: Mapping[str, object], key: str) -> list[object]:
    value = container.get(key, [])
    if not isinstance(value, list):
        raise GlbError(f"{glb.path}: '{key}' is not an array")
    return cast(list[object], value)


def _index(glb: Glb, items: list[object], index: int, where: str) -> Mapping[str, object]:
    if not 0 <= index < len(items):
        raise GlbError(f"{glb.path}: {where} index {index} is outside 0..{len(items) - 1}")
    return _at(glb, items[index], f"{where} {index}")


def _int(glb: Glb, container: Mapping[str, object], key: str, where: str) -> int:
    value = container.get(key)
    if isinstance(value, bool) or not isinstance(value, int):
        raise GlbError(f"{glb.path}: {where} '{key}' is not an integer")
    return value


def _int_or(glb: Glb, container: Mapping[str, object], key: str, where: str, default: int) -> int:
    if key not in container:
        return default
    return _int(glb, container, key, where)


def _document_list(glb: Glb, key: str) -> list[object]:
    return _list(glb, glb.document, key)


def _nodes(glb: Glb) -> list[Mapping[str, object]]:
    return [_at(glb, node, "node") for node in _document_list(glb, "nodes")]


def _meshes(glb: Glb) -> list[Mapping[str, object]]:
    return [_at(glb, mesh, "mesh") for mesh in _document_list(glb, "meshes")]


def _primitives(glb: Glb) -> list[Mapping[str, object]]:
    found: list[Mapping[str, object]] = []
    for mesh in _meshes(glb):
        found.extend(_at(glb, p, "primitive") for p in _list(glb, mesh, "primitives"))
    return found


def _first_skin(glb: Glb) -> Mapping[str, object]:
    skins = _document_list(glb, "skins")
    if not skins:
        raise GlbError(f"{glb.path}: the file has no skin")
    return _index(glb, skins, 0, "skin")


def _joint_nodes(glb: Glb) -> list[int]:
    joints: list[int] = []
    for joint in _list(glb, _first_skin(glb), "joints"):
        if isinstance(joint, bool) or not isinstance(joint, int):
            raise GlbError(f"{glb.path}: a skin joint is not a node index")
        joints.append(joint)
    return joints


def _node_name(glb: Glb, node_index: int) -> str:
    node = _index(glb, _document_list(glb, "nodes"), node_index, "node")
    name = node.get("name")
    if not isinstance(name, str):
        raise GlbError(f"{glb.path}: node {node_index} has no name")
    return name


def accessor(glb: Glb, index: int) -> list[tuple[float, ...]]:
    """Every element of accessor `index`, as tuples of floats. Normalised integers are divided by their
    divisor (`glTF` section 3.11.2); other integers are returned as their value. Sparse and zero-filled
    accessors are refused (none occur in the files this pipeline reads)."""
    accessors = _document_list(glb, "accessors")
    acc = _index(glb, accessors, index, "accessor")
    count = _int(glb, acc, "count", f"accessor {index}")
    component = _int(glb, acc, "componentType", f"accessor {index}")
    kind = acc.get("type")
    if not isinstance(kind, str) or kind not in _TYPE_WIDTH:
        raise GlbError(f"{glb.path}: accessor {index} has an unknown type {kind!r}")
    if component not in _COMPONENT_FORMAT:
        raise GlbError(f"{glb.path}: accessor {index} has an unknown componentType {component}")
    if "bufferView" not in acc:
        raise GlbError(
            f"{glb.path}: accessor {index} has no bufferView (sparse and zero-filled accessors are not read)"
        )
    fmt, size = _COMPONENT_FORMAT[component]
    width = _TYPE_WIDTH[kind]
    normalized = acc.get("normalized") is True
    view_index = _int(glb, acc, "bufferView", f"accessor {index}")
    view = _index(glb, _document_list(glb, "bufferViews"), view_index, "bufferView")
    start = _int_or(glb, view, "byteOffset", f"bufferView {view_index}", 0) + _int_or(
        glb, acc, "byteOffset", f"accessor {index}", 0
    )
    stride = _int_or(glb, view, "byteStride", f"bufferView {view_index}", size * width)
    element = struct.Struct("<" + fmt * width)
    out: list[tuple[float, ...]] = []
    for i in range(count):
        at = start + i * stride
        if at + element.size > len(glb.binary):
            raise GlbError(f"{glb.path}: accessor {index} element {i} reads past the BIN chunk")
        raw = cast(tuple[float, ...], element.unpack_from(glb.binary, at))
        if normalized and component in _NORMALISER:
            divisor = _NORMALISER[component]
            signed = component in (5120, 5122)
            values = tuple(max(v / divisor, -1.0) if signed else v / divisor for v in raw)
            out.append(values)
        else:
            out.append(tuple(float(v) for v in raw))
    return out


def triangle_count(glb: Glb) -> int:
    """Sum over every TRIANGLES primitive of its index count divided by three. A primitive without indices
    counts its POSITION accessor's elements."""
    total = 0
    for primitive in _primitives(glb):
        mode = _int_or(glb, primitive, "mode", "primitive", TRIANGLES_MODE)
        if mode != TRIANGLES_MODE:
            continue
        if "indices" in primitive:
            total += element_count(glb, _int(glb, primitive, "indices", "primitive")) // 3
        else:
            attributes = _at(glb, primitive.get("attributes"), "attributes")
            total += element_count(glb, _int(glb, attributes, "POSITION", "attributes")) // 3
    return total


def element_count(glb: Glb, index: int) -> int:
    """The element count of accessor `index`, read from its declaration without reading its data."""
    acc = _index(glb, _document_list(glb, "accessors"), index, "accessor")
    return _int(glb, acc, "count", f"accessor {index}")


def material_names(glb: Glb) -> list[str]:
    """The name of every material, in document order; an unnamed material is reported as an empty string."""
    names: list[str] = []
    for item in _document_list(glb, "materials"):
        material = _at(glb, item, "material")
        name = material.get("name", "")
        names.append(name if isinstance(name, str) else "")
    return names


def skin_joint_names(glb: Glb) -> list[str]:
    """The names of the first skin's joints, in the skin's own order."""
    return [_node_name(glb, node) for node in _joint_nodes(glb)]


def joint_parents(glb: Glb) -> dict[str, str | None]:
    """Each joint's parent joint by name, from the node children; a root joint maps to None.

    Only joints of the first skin are considered, so a parent outside the skin is reported as None."""
    joints = _joint_nodes(glb)
    joint_set = set(joints)
    parent_of: dict[int, int] = {}
    for node_index, node in enumerate(_nodes(glb)):
        if node_index not in joint_set:
            continue
        for child in _list(glb, node, "children"):
            if isinstance(child, int) and child in joint_set:
                parent_of[child] = node_index
    result: dict[str, str | None] = {}
    for joint in joints:
        parent = parent_of.get(joint)
        result[_node_name(glb, joint)] = None if parent is None else _node_name(glb, parent)
    return result


def _invert4(m: list[list[float]]) -> list[list[float]]:
    """Gauss–Jordan inverse of a 4×4 matrix given as rows, with partial pivoting."""
    a = [row[:] + [1.0 if i == j else 0.0 for j in range(4)] for i, row in enumerate(m)]
    for col in range(4):
        pivot = max(range(col, 4), key=lambda r: abs(a[r][col]))
        if abs(a[pivot][col]) < 1e-12:
            raise GlbError("a bind matrix is singular")
        a[col], a[pivot] = a[pivot], a[col]
        scale = a[col][col]
        a[col] = [v / scale for v in a[col]]
        for r in range(4):
            if r != col and a[r][col] != 0.0:
                factor = a[r][col]
                a[r] = [a[r][k] - factor * a[col][k] for k in range(8)]
    return [row[4:] for row in a]


def joint_world_positions(glb: Glb) -> dict[str, tuple[float, float, float]]:
    """Each joint's world position at bind time: the translation of the inverse of its inverse bind matrix
    (glTF Y-up metres)."""
    skin = _first_skin(glb)
    ibm_index = _int(glb, skin, "inverseBindMatrices", "skin")
    matrices = accessor(glb, ibm_index)
    joints = _joint_nodes(glb)
    if len(matrices) != len(joints):
        raise GlbError(f"{glb.path}: {len(matrices)} inverse bind matrices for {len(joints)} joints")
    positions: dict[str, tuple[float, float, float]] = {}
    for joint, flat in zip(joints, matrices, strict=True):
        # glTF stores matrices column-major: element (row r, column c) is flat[c * 4 + r].
        rows = [[flat[c * 4 + r] for c in range(4)] for r in range(4)]
        world = _invert4(rows)
        positions[_node_name(glb, joint)] = (world[0][3], world[1][3], world[2][3])
    return positions


def _skinned_primitive(glb: Glb) -> Mapping[str, object]:
    for primitive in _primitives(glb):
        attributes = _at(glb, primitive.get("attributes"), "attributes")
        if "JOINTS_0" in attributes:
            return primitive
    raise GlbError(f"{glb.path}: no primitive has JOINTS_0")


def positions(glb: Glb) -> list[tuple[float, float, float]]:
    """POSITION of the first skinned primitive, glTF Y-up."""
    attributes = _at(glb, _skinned_primitive(glb).get("attributes"), "attributes")
    values = accessor(glb, _int(glb, attributes, "POSITION", "attributes"))
    return [(v[0], v[1], v[2]) for v in values]


def triangles(glb: Glb) -> list[tuple[int, int, int]]:
    """The index triples of the first skinned primitive (TRIANGLES; without indices, consecutive vertices)."""
    primitive = _skinned_primitive(glb)
    if "indices" in primitive:
        flat = [int(v[0]) for v in accessor(glb, _int(glb, primitive, "indices", "primitive"))]
    else:
        attributes = _at(glb, primitive.get("attributes"), "attributes")
        flat = list(range(element_count(glb, _int(glb, attributes, "POSITION", "attributes"))))
    return [(flat[i], flat[i + 1], flat[i + 2]) for i in range(0, len(flat) - 2, 3)]


def uvs(glb: Glb) -> list[tuple[float, float]]:
    """TEXCOORD_0 of the first skinned primitive."""
    attributes = _at(glb, _skinned_primitive(glb).get("attributes"), "attributes")
    values = accessor(glb, _int(glb, attributes, "TEXCOORD_0", "attributes"))
    return [(v[0], v[1]) for v in values]


def vertex_weights(glb: Glb) -> list[dict[str, float]]:
    """For each vertex of the first skinned primitive, the weight of each joint with a non-zero weight,
    keyed by the joint's name. JOINTS_0 and WEIGHTS_0 only; a second set (`_1`) is not read here."""
    primitive = _skinned_primitive(glb)
    attributes = _at(glb, primitive.get("attributes"), "attributes")
    joints = accessor(glb, _int(glb, attributes, "JOINTS_0", "attributes"))
    weights = accessor(glb, _int(glb, attributes, "WEIGHTS_0", "attributes"))
    names = skin_joint_names(glb)
    result: list[dict[str, float]] = []
    for joint_row, weight_row in zip(joints, weights, strict=True):
        vertex: dict[str, float] = {}
        for slot in range(4):
            weight = weight_row[slot]
            if weight > 0.0:
                joint = int(round(joint_row[slot]))
                if not 0 <= joint < len(names):
                    raise GlbError(f"{glb.path}: a vertex refers to joint slot {joint}, outside the skin")
                vertex[names[joint]] = vertex.get(names[joint], 0.0) + weight
        result.append(vertex)
    return result


def scene_extras(glb: Glb) -> Mapping[str, object]:
    """The extras object of the default scene (scene 0), or an empty mapping when there is none."""
    scenes = _document_list(glb, "scenes")
    if not scenes:
        return {}
    scene = _index(glb, scenes, 0, "scene")
    extras = scene.get("extras")
    if extras is None:
        return {}
    return _at(glb, extras, "scene extras")


def embedded_image(glb: Glb, index: int) -> bytes:
    """The bytes of image `index`, read from its bufferView in the BIN chunk."""
    images = _document_list(glb, "images")
    image = _index(glb, images, index, "image")
    view_index = _int(glb, image, "bufferView", f"image {index}")
    view = _index(glb, _document_list(glb, "bufferViews"), view_index, "bufferView")
    start = _int_or(glb, view, "byteOffset", f"bufferView {view_index}", 0)
    length = _int(glb, view, "byteLength", f"bufferView {view_index}")
    if start + length > len(glb.binary):
        raise GlbError(f"{glb.path}: image {index} reads past the BIN chunk")
    return glb.binary[start : start + length]


def png_size(data: bytes) -> tuple[int, int, int, int]:
    """Width, height, bit depth and colour type of a PNG, from its IHDR chunk."""
    if data[:8] != _PNG_SIGNATURE or data[12:16] != b"IHDR":
        raise GlbError("the bytes are not a PNG with an IHDR chunk")
    width, height = struct.unpack_from(">II", data, 16)
    return width, height, data[24], data[25]


def skinned_mesh_count(glb: Glb) -> int:
    """The number of distinct meshes referenced by a node that carries a skin."""
    meshes = {_int(glb, node, "mesh", "node") for node in _nodes(glb) if "skin" in node and "mesh" in node}
    return len(meshes)


def primitive_count(glb: Glb) -> int:
    """The number of primitives across every mesh."""
    return len(_primitives(glb))


def material_documents(glb: Glb) -> list[Mapping[str, object]]:
    """Every material object, in document order."""
    return [_at(glb, item, "material") for item in _document_list(glb, "materials")]


def texture_slots(glb: Glb, material_index: int) -> dict[str, bool]:
    """Which PBR texture slots material `material_index` fills.

    Keys: base colour, normal, occlusion, emissive, metallic-roughness. A missing material is an error."""
    material = _index(glb, _document_list(glb, "materials"), material_index, "material")
    pbr_raw = material.get("pbrMetallicRoughness", {})
    pbr = _at(glb, pbr_raw, "pbrMetallicRoughness")
    return {
        "base colour": "baseColorTexture" in pbr,
        "normal": "normalTexture" in material,
        "occlusion": "occlusionTexture" in material,
        "emissive": "emissiveTexture" in material,
        "metallic-roughness": "metallicRoughnessTexture" in pbr,
    }


def base_colour_image_index(glb: Glb, material_index: int) -> int:
    """The image index behind material `material_index`'s base-colour texture."""
    material = _index(glb, _document_list(glb, "materials"), material_index, "material")
    pbr = _at(glb, material.get("pbrMetallicRoughness", {}), "pbrMetallicRoughness")
    info = _at(glb, pbr.get("baseColorTexture"), "baseColorTexture")
    texture = _index(
        glb, _document_list(glb, "textures"), _int(glb, info, "index", "baseColorTexture"), "texture"
    )
    return _int(glb, texture, "source", "texture")


def attribute_names(glb: Glb) -> list[str]:
    """Attribute names of the first skinned primitive, sorted."""
    attributes = _at(glb, _skinned_primitive(glb).get("attributes"), "attributes")
    return sorted(attributes)
