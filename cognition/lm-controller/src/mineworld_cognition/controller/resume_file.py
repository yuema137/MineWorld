"""The seat's resume secret, persisted beside its store (pr-s10-p6 §5.10, Q-P3b-4; C11).

The file holds the 32 lowercase hexadecimal characters of the current `welcome.resume`, and nothing else. It is
written after every welcome by writing `<path>.tmp` and replacing `<path>` (atomic on all three platforms), mode
`0o600` on POSIX. It is read before the join, deleted when the seat is given back or lost, and never logged,
never in a `repr`, never in the decision log or the store (I-P6-8, AP6-9). A file that is absent or not well
formed reads as `None`: a stale secret costs one join without `resume` (the SDK retries without it, row 7).
"""

from __future__ import annotations

import os
import re
from pathlib import Path

from mineworld_sdk.wire.ids import ResumeSecret

_WELL_FORMED = re.compile(r"^[0-9a-f]{32}$")


def read(path: Path) -> ResumeSecret | None:
    try:
        text = path.read_text(encoding="ascii")
    except (OSError, UnicodeDecodeError):
        return None
    if not _WELL_FORMED.fullmatch(text):
        return None
    return ResumeSecret(text)


def write(path: Path, secret: ResumeSecret) -> None:
    """Replaces `path` with `secret` atomically. Refuses anything that is not the secret's wire form."""
    if not _WELL_FORMED.fullmatch(secret):
        raise ValueError("a resume secret is 32 lowercase hexadecimal characters")
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(path.name + ".tmp")
    temporary.write_text(secret, encoding="ascii")
    if os.name == "posix":
        os.chmod(temporary, 0o600)
    os.replace(temporary, path)


def delete(path: Path) -> None:
    path.unlink(missing_ok=True)
