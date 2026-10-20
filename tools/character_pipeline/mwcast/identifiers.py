"""AI-identifier inspection of downloads and exports (port of `inspect_ids.py`; PIPELINE.md §3; DEP-8).

It reports where AI-generation identifiers sit in a file: a glTF `asset`, `extras` or `extensions` object
(with its location in the document), a PNG text or metadata chunk (tEXt, iTXt, zTXt, caBX, eXIf), a JPEG
APP segment that carries XMP or JUMBF, and every occurrence of `meshy` (case-insensitive) with its location.
A `meshy` in a `name` field is reported as its own kind. It reads, never writes, and it reports locations
and kinds, not bytes.
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import cast

from mwcast.glb import Glb, embedded_image, read_glb

PNG_SIGNATURE = b"\x89PNG\r\n\x1a\n"
PNG_TEXT_CHUNKS = (b"tEXt", b"iTXt", b"zTXt", b"caBX", b"eXIf")
JPEG_INDICATORS = (b"http://ns.adobe.com/xap", b"<x:xmpmeta", b"jumb", b"c2pa")
GLB_EXTRA_KEYS = ("asset", "extras", "extensions")


@dataclass(frozen=True)
class Finding:
    where: str
    kind: str


def _png_findings(data: bytes, where: str) -> list[Finding]:
    found: list[Finding] = []
    offset = len(PNG_SIGNATURE)
    while offset + 8 <= len(data):
        length = int.from_bytes(data[offset : offset + 4], "big")
        chunk = data[offset + 4 : offset + 8]
        if chunk in PNG_TEXT_CHUNKS:
            found.append(Finding(where, f"png chunk {chunk.decode('ascii')}"))
        offset += 12 + length
    return found


def _jpeg_findings(data: bytes, where: str) -> list[Finding]:
    found: list[Finding] = []
    lowered = data.lower()
    for indicator in JPEG_INDICATORS:
        if indicator in lowered:
            found.append(Finding(where, f"jpeg metadata {indicator.decode('ascii', errors='replace')}"))
    return found


def _document_findings(glb: Glb) -> list[Finding]:
    found: list[Finding] = []

    def walk(value: object, path: str) -> None:
        if isinstance(value, dict):
            mapping = cast(dict[str, object], value)
            for key in GLB_EXTRA_KEYS:
                if key in mapping:
                    found.append(Finding(f"{path}.{key}" if path else key, f"glTF {key} object"))
            for key, item in mapping.items():
                here = f"{path}.{key}" if path else key
                if key == "name" and isinstance(item, str) and "meshy" in item.lower():
                    # A human-readable name (a material called MW_Meshy), not a machine-readable identifier.
                    found.append(Finding(here, "meshy in a name"))
                else:
                    walk(item, here)
        elif isinstance(value, list):
            for index, item in enumerate(cast(list[object], value)):
                walk(item, f"{path}[{index}]")
        elif isinstance(value, str) and "meshy" in value.lower():
            found.append(Finding(path, "meshy occurrence"))

    walk(glb.document, "")
    return found


def inspect_glb(path: Path) -> list[Finding]:
    glb = read_glb(path)
    found = _document_findings(glb)
    images = glb.document.get("images")
    if isinstance(images, list):
        for index in range(len(cast(list[object], images))):
            data = embedded_image(glb, index)
            where = f"images[{index}]"
            if data.startswith(PNG_SIGNATURE):
                found.extend(_png_findings(data, where))
            elif data.startswith(b"\xff\xd8"):
                found.extend(_jpeg_findings(data, where))
    return found


def inspect_file(path: Path) -> list[Finding]:
    """The findings for one file. A `.glb` is read structurally; other files get raw-byte checks."""
    if path.suffix.lower() == ".glb":
        return inspect_glb(path)
    data = path.read_bytes()
    found: list[Finding] = []
    if data.startswith(PNG_SIGNATURE):
        found.extend(_png_findings(data, "file"))
    elif data.startswith(b"\xff\xd8"):
        found.extend(_jpeg_findings(data, "file"))
    if b"meshy" in data.lower():
        found.append(Finding("file", "meshy occurrence (raw bytes)"))
    return found
