"""The configuration error every module of this package shares."""

from __future__ import annotations


class ConfigError(ValueError):
    """The configuration cannot be used. The message names the table, the key, a variable or a path —
    never a key's value (I-16)."""
