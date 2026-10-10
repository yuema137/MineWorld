"""Keys: a redacted holder, and the one place a key is read (D-P5-9, D-P5-14; DEP-32)."""

from __future__ import annotations

from typing import NoReturn


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
