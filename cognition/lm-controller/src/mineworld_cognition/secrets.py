"""Keys: a redacted holder, and the one place a key is read (D-P5-9, D-P5-14; DEP-32).

A key is resolved only when a live or recording backend is built, from (1) the process environment, or
else (2) the `.env`-style file the operator named in `[secrets] env_file`, read with python-dotenv's
`dotenv_values(path, interpolate=False)`. That call returns a mapping and never touches `os.environ`, so
a file's values never become ambient credentials for a child process or another library. `load_dotenv`
is never called; no file is read that the configuration did not name; there is no default location.
"""

from __future__ import annotations

import os
import stat
from pathlib import Path
from typing import NoReturn

from dotenv import dotenv_values

from mineworld_cognition.errors import ConfigError


class Secret:
    """A key's value. Its `repr` and `str` are redacted; it cannot be pickled or compared by value, so it
    cannot leak through a log line, an exception, a cassette or a save (I-16)."""

    __slots__ = ("_value",)

    def __init__(self, value: str) -> None:
        self._value = value

    def reveal(self) -> str:
        """The value, for the one header that carries it. Never log what this returns."""
        return self._value

    def __repr__(self) -> str:
        return "Secret(<redacted>)"

    __str__ = __repr__

    def __reduce__(self) -> NoReturn:
        raise TypeError("a Secret is never serialized")


def permission_check_applies() -> bool:
    """POSIX modes mean something on macOS and Linux only; on Windows the check is skipped by platform
    (the README says to keep the file under the user's profile)."""
    return os.name != "nt"


def check_env_file(path: Path) -> None:
    """Refuses a key file readable by group or others, as `ssh` refuses a private key (D-P5-14)."""
    if not path.is_file():
        raise ConfigError(f"[secrets] env_file: {path} does not exist or is not a file")
    if permission_check_applies() and stat.S_IMODE(path.stat().st_mode) & 0o077:
        raise ConfigError(
            f"[secrets] env_file: {path} is readable by group or others; run `chmod 600 {path}`"
        )


def resolve_key(name: str, env_file: Path | None, *, backend: str) -> Secret:
    """The value of the variable `name`: the process environment first, then `env_file`."""
    value = os.environ.get(name)
    if value:
        return Secret(value)
    if env_file is not None:
        check_env_file(env_file)
        value = dotenv_values(env_file, interpolate=False).get(name)
        if value:
            return Secret(value)
    where = f"the environment or in {env_file}" if env_file is not None else "the environment"
    raise ConfigError(
        f"backend {backend!r}: {name} is not set in {where}"
        + ("" if env_file is not None else " (no [secrets] env_file is configured)")
    )
