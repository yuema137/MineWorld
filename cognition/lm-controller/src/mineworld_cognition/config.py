"""The operator's cognition configuration: the only place a backend is named (D-P5-3, D-P5-10, §5.4).

A backend is reached through its **name** in `[backends.<name>]`; a tier is bound to a name or to
`"none"`; a key appears only as the name of an environment variable (`key_env`). Nothing here holds a
key's value, and nothing here is ever inside a world (D-P5-14).
"""

from __future__ import annotations

import ipaddress
import re
from typing import Annotated, Literal
from urllib.parse import urlsplit

from pydantic import BaseModel, ConfigDict, Field, field_validator

StructuredOutput = Literal["json_schema", "json_object", "none"]
Reasoning = Literal["unset", "off", "low", "medium", "high"]
TemperatureMode = Literal["send", "omit"]

KEY_ENV = re.compile(r"^[A-Z_][A-Z0-9_]*$")
"""An environment variable's name. A pasted key (lowercase, `-`) never matches (R-P5-12)."""


class ConfigError(ValueError):
    """The configuration cannot be used. The message names the table and key, never a key's value."""


class ConfigModel(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")


def is_loopback_host(host: str) -> bool:
    if host == "localhost":
        return True
    try:
        return ipaddress.ip_address(host).is_loopback
    except ValueError:
        return False


def check_base_url(value: str) -> str:
    """`http` or `https`, a host, no userinfo, no query, no fragment; `https` unless the host is
    loopback, so a key never crosses a network in clear text (D-P5-10)."""
    parts = urlsplit(value)
    if parts.scheme not in ("http", "https") or not parts.hostname:
        raise ValueError("base_url must be an http or https URL with a host")
    if parts.username is not None or parts.password is not None or "@" in parts.netloc:
        raise ValueError("base_url must not carry credentials (user:password@); use key_env")
    if parts.query or parts.fragment:
        raise ValueError("base_url must not carry a query or a fragment; use key_env for a key")
    if parts.scheme == "http" and not is_loopback_host(parts.hostname):
        raise ValueError("base_url must be https unless its host is a loopback address")
    return value


class BackendConfig(ConfigModel):
    """One `[backends.<name>]` table (D-P5-10). Typed options only: no free-form map."""

    kind: Literal["openai-compatible"]
    base_url: str
    model: Annotated[str, Field(min_length=1, max_length=200)]
    key_env: str = ""
    """The NAME of the environment variable holding the key, or empty when the server needs none."""
    structured_output: StructuredOutput = "json_schema"
    reasoning: Reasoning = "unset"
    temperature: TemperatureMode = "send"
    request_timeout_s: Annotated[int, Field(ge=1, le=3600)] | None = None

    @field_validator("base_url")
    @classmethod
    def _base_url(cls, value: str) -> str:
        return check_base_url(value)

    @field_validator("key_env")
    @classmethod
    def _key_env(cls, value: str) -> str:
        if value and not KEY_ENV.fullmatch(value):
            raise ValueError(
                "key_env is the NAME of an environment variable (A-Z, 0-9, _), never a key's value"
            )
        return value
