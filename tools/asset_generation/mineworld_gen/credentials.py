"""Credential loading for the generation tooling.

MineWorld does not own a credential convention and must not grow one here. The rules this
module follows:

1. An API key already in the environment always wins. In CI, or for an operator who exports it
   in their shell, nothing else is consulted.
2. Otherwise a user-local secret file may be read. Its path comes from
   `MINEWORLD_SECRETS_FILE`; only if that is unset does the documented default
   `~/.config/mineworld/secrets.env` apply. **That default is a convention, not a dependency** —
   the tool is fully usable with no secret file at all.
3. The file lives outside the repository, and nothing in this package ever writes a key
   anywhere: not to a log, not to a sidecar, not to a manifest, not to stdout.

The parser is deliberately dumb — `KEY=value` lines, optional `export`, `#` comments — because
this is not a dotenv implementation and should not become one.
"""

from __future__ import annotations

import os
from pathlib import Path

__all__ = ["load_api_key", "DEFAULT_SECRETS_FILE", "SECRETS_FILE_ENV"]

SECRETS_FILE_ENV = "MINEWORLD_SECRETS_FILE"
DEFAULT_SECRETS_FILE = "~/.config/mineworld/secrets.env"


def _parse(path: Path, wanted: str) -> str | None:
    try:
        text = path.read_text(encoding="utf-8")
    except OSError:
        return None
    for raw in text.splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("export "):
            line = line[len("export ") :].strip()
        key, sep, value = line.partition("=")
        if not sep or key.strip() != wanted:
            continue
        value = value.strip()
        if len(value) >= 2 and value[0] == value[-1] and value[0] in "\"'":
            value = value[1:-1]
        return value or None
    return None


def load_api_key(var: str = "OPENAI_API_KEY") -> str:
    """Return the API key, or raise with actionable guidance that never echoes a secret."""
    existing = os.environ.get(var)
    if existing:
        return existing

    configured = os.environ.get(SECRETS_FILE_ENV)
    path = Path(configured).expanduser() if configured else Path(DEFAULT_SECRETS_FILE).expanduser()
    found = _parse(path, var)
    if found:
        return found

    raise RuntimeError(
        f"No {var} available.\n"
        f"  Either export {var} in the environment,\n"
        f"  or place a '{var}=...' line in a secret file outside the repository and point\n"
        f"  {SECRETS_FILE_ENV} at it (default when unset: {DEFAULT_SECRETS_FILE}).\n"
        f"  Looked in: {path}"
    )
