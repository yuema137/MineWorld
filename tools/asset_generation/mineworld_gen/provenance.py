"""Provenance as data, not prose (`docs/DECISIONS.md` ARC-9).

One sidecar per candidate: `foo.png` is accompanied by `foo.provenance.yaml`. The field names
`source_type`, `generation`, `postprocess` and `license` are spelled exactly as ARC-9 spells
them, because MVP-1 is going to validate them in the pack format. Everything this tool adds
beyond ARC-9 — the reference images, the full parameter set, the seed, the approval record —
is additive and lives under those same keys.

The rule the sidecar exists to enforce: **a generated output is a candidate until a person
accepts it.** `approval.status` starts at `candidate` and nothing in this package ever sets it
to anything else. Promotion is a human act.
"""

from __future__ import annotations

import datetime as _dt
import hashlib
from pathlib import Path
from typing import Sequence

import yaml

__all__ = ["build_provenance", "write_sidecar", "SIDECAR_SUFFIX"]

SIDECAR_SUFFIX = ".provenance.yaml"

_PROMPT_SUMMARY_CHARS = 120


def _digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            h.update(chunk)
    return f"sha256:{h.hexdigest()}"


def build_provenance(
    *,
    provider_name: str,
    model_id: str,
    workflow_version: str,
    prompt: str,
    params: dict[str, object],
    seed: int,
    reference_images: Sequence[Path] = (),
    repo_root: Path | None = None,
    license_note: str = "",
    redistribution_allowed: bool = True,
) -> dict[str, object]:
    """Assemble the ARC-9 record. Reference images are recorded by repo-relative path *and*
    content hash, so a later style audit can tell whether the references themselves moved."""
    refs = []
    for ref in reference_images:
        ref = Path(ref)
        rel = str(ref.resolve().relative_to(repo_root.resolve())) if repo_root else str(ref)
        refs.append({"path": rel, "digest": _digest(ref)})

    summary = " ".join(prompt.split())
    if len(summary) > _PROMPT_SUMMARY_CHARS:
        summary = summary[: _PROMPT_SUMMARY_CHARS - 1].rstrip() + "…"

    return {
        "source_type": "generated",
        "generation": {
            "tool": f"tools/asset_generation ({provider_name})",
            "provider": provider_name,
            "model": model_id,
            "workflow_version": workflow_version,
            "date": _dt.date.today().isoformat(),
            "prompt_summary": summary,
            "prompt": prompt,
            "parameters": params,
            "seed": seed,
            "reference_images": refs,
            # ARC-9 spells this field `human_curated`. It is False on emission by
            # construction: the machine has not curated anything.
            "human_curated": False,
        },
        "postprocess": {
            "cleaned": False,
            "retopology": False,
            "texture_adjusted": False,
            "background_removed": False,
        },
        "approval": {
            "status": "candidate",
            "approved_by": None,
            "approved_date": None,
        },
        "license": {
            "redistribution_allowed": redistribution_allowed,
            "note": license_note,
        },
    }


def write_sidecar(image_path: Path, provenance: dict[str, object]) -> Path:
    sidecar = image_path.with_suffix("")
    sidecar = sidecar.with_name(sidecar.name + SIDECAR_SUFFIX)
    sidecar.write_text(
        yaml.safe_dump(provenance, sort_keys=False, allow_unicode=True, width=100),
        encoding="utf-8",
    )
    return sidecar
