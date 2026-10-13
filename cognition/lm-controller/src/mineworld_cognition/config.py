"""The operator's cognition configuration: the only place a backend is named (D-P5-3, D-P5-10, §5.4).

A backend is reached through its **name** in `[backends.<name>]`; a tier is bound to a name or to
`"none"`; a key appears only as the name of an environment variable (`key_env`). The file is the
operator's — per user, never per world (D-P5-14): a configuration or key file inside a directory that
holds a `world.yaml` is refused. Paths in it are resolved relative to the file. In `replay` and
`scripted` modes the backend tables are validated and never instantiated, and the HTTP adapter is never
imported (D-P5-5).
"""

from __future__ import annotations

import ipaddress
import re
import tomllib
from pathlib import Path
from typing import Annotated, Literal, Self, cast
from urllib.parse import urlsplit

from pydantic import BaseModel, ConfigDict, Field, ValidationError, field_validator, model_validator

from mineworld_cognition.backend.model import ModelBackend
from mineworld_cognition.backend.providers import (
    PRESETS,
    PresetName,
    Reasoning,
    StructuredOutput,
    TemperatureMode,
)
from mineworld_cognition.budget import BudgetPolicy, Clock, MemoryLedger, SqliteLedger, SystemClock
from mineworld_cognition.errors import ConfigError
from mineworld_cognition.gateway import Budget, ModelGateway, Router, Tier
from mineworld_cognition.record import Cassette, RecordingBackend, ReplayBackend

__all__ = [
    "BackendConfig",
    "BudgetConfig",
    "CognitionConfig",
    "ConfigError",
    "Mode",
    "RecordingConfig",
    "SecretsConfig",
    "build_router",
    "load",
]

Mode = Literal["live", "record", "replay", "scripted"]

KEY_ENV = re.compile(r"^[A-Z_][A-Z0-9_]*$")
"""An environment variable's name. A pasted key (lowercase, `-`) never matches (R-P5-12)."""
BACKEND_NAME = r"^[a-z0-9](?:[a-z0-9_-]*[a-z0-9])?$"
NO_BACKEND = "none"


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


Kind = Literal["openai-compatible", "anthropic"]
"""The adapters (`backend/registry.py`). A kind not listed, `claude-code-subscription` among them
(QP5b-1), is refused by name."""

ANTHROPIC_HOST = "api.anthropic.com"


def _is_anthropic_host(host: str) -> bool:
    return host == ANTHROPIC_HOST or host.endswith(".anthropic.com")


class BackendConfig(ConfigModel):
    """One `[backends.<name>]` table (D-P5-10, D-P5-15). Typed options only: no free-form map."""

    kind: Kind = "openai-compatible"
    preset: PresetName | None = None
    """A named provider (`backend/providers.py`): fills every field this table leaves out."""
    base_url: str
    model: Annotated[str, Field(min_length=1, max_length=200)]
    key_env: str = ""
    """The NAME of the environment variable holding the key, or empty when the server needs none."""
    structured_output: StructuredOutput = "json_schema"
    reasoning: Reasoning = "unset"
    temperature: TemperatureMode = "send"
    request_timeout_s: Annotated[int, Field(ge=1, le=3600)] | None = None

    @model_validator(mode="before")
    @classmethod
    def _fill_from_preset(cls, data: object) -> object:
        if not isinstance(data, dict):
            return data
        fields = cast(dict[str, object], data)
        if fields.get("kind") == "anthropic":
            # The Messages API refuses a temperature below 1 on recent models (pr-s10-p5b §3): omit it
            # unless the table asks for it. Presets are OpenAI-compatible rows; `_kind_options` refuses one.
            return {"temperature": "omit", **fields}
        name = fields.get("preset")
        preset = PRESETS.get(name) if isinstance(name, str) else None
        if preset is None:
            return fields  # no preset, or an unknown one, which the field's own validation names
        defaults: dict[str, object] = {
            "key_env": preset.key_env,
            "structured_output": preset.structured_output,
            "temperature": preset.temperature,
            "reasoning": preset.reasoning,
        }
        if preset.base_url is not None:
            defaults["base_url"] = preset.base_url
        return {**defaults, **fields}

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

    @model_validator(mode="after")
    def _kind_options(self) -> Self:
        host = urlsplit(self.base_url).hostname or ""
        if self.kind == "openai-compatible" and _is_anthropic_host(host):
            raise ValueError(
                f'base_url on {host}: use kind = "anthropic" for Claude. Anthropic\'s '
                "OpenAI-compatibility layer ignores response_format and seed (DEP-33)"
            )
        if self.kind == "anthropic":
            if self.preset is not None:
                raise ValueError(
                    'preset: presets are OpenAI-compatible rows; kind = "anthropic" '
                    "takes base_url, model and key_env"
                )
            if self.structured_output == "json_object":
                raise ValueError(
                    'structured_output: kind = "anthropic" supports "json_schema" or "none"'
                )
            if self.reasoning != "unset":
                raise ValueError('reasoning: kind = "anthropic" does not map a reasoning setting')
        return self


class RecordingConfig(ConfigModel):
    mode: Mode
    cassette: str | None = None


class BudgetConfig(BudgetPolicy):
    ledger: str | None = None
    """A SQLite file, so a restart neither resets nor double-counts; unset keeps the ledger in memory."""


class SecretsConfig(ConfigModel):
    env_file: str | None = None


class CognitionConfig(ConfigModel):
    recording: RecordingConfig
    tiers: dict[Tier, str] = Field(default_factory=dict[Tier, str])
    budgets: BudgetConfig = Field(default_factory=BudgetConfig)
    backends: dict[Annotated[str, Field(pattern=BACKEND_NAME)], BackendConfig] = Field(
        default_factory=dict[str, BackendConfig]
    )
    secrets: SecretsConfig = Field(default_factory=SecretsConfig)

    @model_validator(mode="after")
    def _consistent(self) -> Self:
        for tier, name in self.tiers.items():
            if name != NO_BACKEND and name not in self.backends:
                raise ValueError(f"tiers.{tier}: no [backends.{name}] table defines {name!r}")
        mode = self.recording.mode
        if mode in ("replay", "record") and self.recording.cassette is None:
            raise ValueError(f"recording.cassette: mode {mode!r} needs a cassette")
        if mode == "record" and len(self.bound_backends()) > 1:
            raise ValueError("recording.mode: record mode records one backend binding at a time")
        return self

    def bound_backends(self) -> list[str]:
        """The backend names some tier is bound to, each once, in tier order."""
        return list(dict.fromkeys(name for name in self.tiers.values() if name != NO_BACKEND))


def world_directory_of(path: Path) -> Path | None:
    """The nearest directory at or above `path` that holds a `world.yaml`, if any."""
    for directory in path.resolve().parents:
        if (directory / "world.yaml").is_file():
            return directory
    return None


def _refuse_inside_world(path: Path, what: str) -> None:
    world = world_directory_of(path)
    if world is not None:
        raise ConfigError(
            f"{what} {path} is inside the world directory {world}; configuration and keys belong "
            "to the operator, never to a world, which travels with them (D-P5-14)"
        )


def _describe(error: ValidationError) -> str:
    # Location and message only: pydantic's own text repeats the input, which could be a pasted key.
    return "; ".join(
        ".".join(str(part) for part in item["loc"]) + ": " + item["msg"] for item in error.errors()
    )


def load(path: Path) -> CognitionConfig:
    """Reads, validates and resolves a configuration file. Relative paths are resolved against it."""
    _refuse_inside_world(path, "the configuration file")
    try:
        with path.open("rb") as file:
            data = tomllib.load(file)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ConfigError(f"{path}: {error}") from None
    try:
        config = CognitionConfig.model_validate(data)
    except ValidationError as error:
        raise ConfigError(f"{path}: {_describe(error)}") from None
    base = path.resolve().parent

    def resolved(value: str | None) -> str | None:
        return None if value is None else str(base / value)

    config = config.model_copy(
        update={
            "recording": config.recording.model_copy(
                update={"cassette": resolved(config.recording.cassette)}
            ),
            "budgets": config.budgets.model_copy(
                update={"ledger": resolved(config.budgets.ledger)}
            ),
            "secrets": SecretsConfig(env_file=resolved(config.secrets.env_file)),
        }
    )
    if config.secrets.env_file is not None:
        _refuse_inside_world(Path(config.secrets.env_file), "the env_file")
    return config


def build_router(
    config: CognitionConfig,
    *,
    clock: Clock | None = None,
    scripted: ModelBackend | None = None,
) -> Router:
    """Builds every tier's gateway. Only `live` and `record` import the adapter or read a key."""
    policy = BudgetPolicy.model_validate(config.budgets.model_dump(exclude={"ledger"}))
    ledger = (
        MemoryLedger()
        if config.budgets.ledger is None
        else SqliteLedger(Path(config.budgets.ledger))
    )
    budget = Budget.of(policy, ledger, clock or SystemClock())
    mode = config.recording.mode
    gateways: dict[str, ModelGateway] = {}
    if mode == "replay":
        assert config.recording.cassette is not None
        shared = ModelGateway(ReplayBackend(Cassette.load(Path(config.recording.cassette))), budget)
        gateways = {name: shared for name in config.bound_backends()}
    elif mode == "scripted":
        if scripted is None:
            raise ConfigError("recording.mode: scripted mode needs a backend supplied in code")
        shared = ModelGateway(scripted, budget)
        gateways = {name: shared for name in config.bound_backends()}
    else:
        from mineworld_cognition.backend.registry import build_backend
        from mineworld_cognition.secrets import resolve_key

        env_file = None if config.secrets.env_file is None else Path(config.secrets.env_file)
        for name in config.bound_backends():
            backend_config = config.backends[name]
            key = (
                resolve_key(backend_config.key_env, env_file, backend=name)
                if backend_config.key_env
                else None
            )
            backend = build_backend(backend_config, key)
            if mode == "record":
                assert config.recording.cassette is not None
                backend = RecordingBackend(
                    backend,
                    Path(config.recording.cassette),
                    binding=name,
                    model=backend_config.model,
                )
            gateways[name] = ModelGateway(backend, budget)
    tiers: dict[Tier, ModelGateway | None] = {
        tier: None if name == NO_BACKEND else gateways[name] for tier, name in config.tiers.items()
    }
    return Router(tiers, budget)
