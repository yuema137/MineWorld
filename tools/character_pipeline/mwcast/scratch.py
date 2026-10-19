"""Scratch root resolution (PIPELINE.md §3; I-B1-5).

Every pipeline output goes to a scratch root outside the repository. A scratch root inside the repository is
refused, because a committed download, intermediate or texture is exactly what the repository must never hold.
"""

from __future__ import annotations

from collections.abc import Mapping
from pathlib import Path

SCRATCH_ENV = "MINEWORLD_CAST_SCRATCH"


class ScratchError(Exception):
    """No scratch root was given, or the one given is inside the repository."""


def resolve_scratch(arg: str | None, environ: Mapping[str, str], repo_root: Path) -> Path:
    """Return the scratch root: `arg` if given, else the `MINEWORLD_CAST_SCRATCH` value, else refuse.

    The resolved path must not be the repository root or lie inside it."""
    raw = arg if arg else environ.get(SCRATCH_ENV, "")
    if not raw:
        raise ScratchError(
            f"no scratch root: give --scratch or set {SCRATCH_ENV} to a folder outside the repository"
        )
    path = Path(raw).expanduser().resolve()
    root = repo_root.resolve()
    if path == root or root in path.parents:
        raise ScratchError(f"scratch root {path} is inside the repository {root}; choose a folder outside it")
    return path
