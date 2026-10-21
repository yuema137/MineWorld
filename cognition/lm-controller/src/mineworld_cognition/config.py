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

from mineworld_sdk.wire.ids import JsonValue
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
from mineworld_cognition.memory.config import MemoryConfig, SharingModeUnavailable
from mineworld_cognition.record import Cassette, RecordingBackend, ReplayBackend

__all__ = [
    "BUNDLED_RETRY",
    "BackendConfig",
    "BudgetConfig",
    "CognitionConfig",
    "ConfigError",
    "ControllerConfig",
    "LayerConfig",
    "Mode",
    "RecordingConfig",
    "RetryConfig",
    "SeatOverrides",
    "SecretsConfig",
    "build_router",
    "controller_for",
    "layer_policy",
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

    kind: Kind = Field(
        default="openai-compatible",
        description=(
            "`openai-compatible` for a server that speaks the OpenAI chat API: a local server or a "
            "hosted provider. `anthropic` for Claude, through Anthropic's Messages API."
        ),
    )
    preset: PresetName | None = Field(
        default=None,
        description=(
            "A named hosted provider. It fills `base_url`, `key_env`, `structured_output` and "
            '`temperature` when this table leaves them out. Not allowed with `kind = "anthropic"`.'
        ),
    )
    """A named provider (`backend/providers.py`): fills every field this table leaves out."""
    base_url: str = Field(
        description=(
            "The address of the server, for example `http://127.0.0.1:11434/v1` for a local server. "
            "Required unless `preset` supplies it. Use `https` unless the host is a loopback address. "
            "It must not carry a user name, a password, a query or a fragment."
        ),
    )
    model: Annotated[
        str,
        Field(
            min_length=1,
            max_length=200,
            description="The model name the server expects, for example `qwen3.5:4b`.",
        ),
    ]
    key_env: str = Field(
        default="",
        description=(
            "The NAME of the environment variable that holds the API key, for example "
            "`DEEPSEEK_API_KEY`: upper-case letters, digits and `_`, not starting with a digit. Empty "
            "when the server needs no key. A preset fills it when this table leaves it out. The key "
            "itself is never written in this file."
        ),
    )
    """The NAME of the environment variable holding the key, or empty when the server needs none."""
    structured_output: StructuredOutput = Field(
        default="json_schema",
        description=(
            "How the server is asked for structured output. `json_schema`: the server enforces the "
            "output schema. `json_object`: the server returns any JSON object, and MineWorld checks "
            "it against the schema. `none`: no structured-output request is sent."
        ),
    )
    reasoning: Reasoning = Field(
        default="unset",
        description=(
            "The reasoning effort the server should use: `off`, `low`, `medium` or `high`. `unset` "
            'sends no reasoning setting. Not allowed with `kind = "anthropic"`.'
        ),
    )
    temperature: TemperatureMode = Field(
        default="send",
        description=(
            "`send` sends the sampling temperature with each request. `omit` leaves it out, for a "
            "model that refuses a temperature."
        ),
    )
    request_timeout_s: Annotated[int, Field(ge=1, le=3600)] | None = Field(
        default=None,
        description=(
            "Seconds to wait for one answer from this backend, from 1 to 3600. Unset means this "
            "table sets no request timeout."
        ),
    )

    retry: RetryConfig | None = Field(
        default=None,
        description=(
            "The `[backends.<name>.retry.<layer>]` tables: retry layers for the seats that use this backend, "
            "between a seat's own override and `[retry]`."
        ),
    )

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
    """The `[recording]` table: where answers come from, and whether they are recorded."""

    mode: Mode = Field(
        description=(
            "Where answers come from. `live`: the configured backend answers. `record`: the backend "
            "answers and each answer is written to the cassette. `replay`: answers are read from the "
            "cassette, and no network client is built. `scripted`: a backend supplied in code, for tests."
        ),
    )
    cassette: str | None = Field(
        default=None,
        description=(
            "The cassette file, which `record` writes and `replay` reads. Required for `record` and "
            "`replay`. A relative path is resolved against this configuration file."
        ),
    )


class BudgetConfig(BudgetPolicy):
    """The `[budgets]` table: the limits of `BudgetPolicy`, and where the spend is kept."""

    ledger: str | None = Field(
        default=None,
        description=(
            "A SQLite file that keeps the spend, so a restart neither resets it nor counts a call "
            "twice. Unset keeps the spend in memory only. A relative path is resolved against this "
            "configuration file."
        ),
    )
    """A SQLite file, so a restart neither resets nor double-counts; unset keeps the ledger in memory."""


class SecretsConfig(ConfigModel):
    """The `[secrets]` table: an optional `.env`-style file that holds API keys."""

    env_file: str | None = Field(
        default=None,
        description=(
            "A `.env`-style file that holds API keys, read only when named here. A relative path is "
            "resolved against this configuration file. On macOS and Linux the file must not be "
            "readable by its group or by others (`chmod 600`); the loader refuses it otherwise."
        ),
    )


SEAT_KEY = r"^[a-z][a-z0-9-]{0,47}$"
"""A seat key, as the protocol and the decision list spell it (pr-s10-p6 §5.13)."""
POLICY_ID = r"^[a-z][a-z0-9-]{0,47}$"
"""A retry or breaker policy id: its entry-point name (pr-s10-p6 §5.5.2, §5.6)."""
Layer = Literal["transport", "content", "semantic"]
"""The three retry layers of one consult (pr-s10-p6 §5.5)."""


class ControllerConfig(ConfigModel):
    """The `[controller]` table: the consult's bounds and the seat's timing (pr-s10-p6 §5.13)."""

    consult_deadline_s: Annotated[int, Field(ge=1, le=120)] = Field(
        default=20,
        description="Wall seconds from a trigger's receipt to the submit, from 1 to 120.",
    )
    max_calls: Annotated[int, Field(ge=1, le=10)] = Field(
        default=5,
        description="The most model calls one consult may make, from 1 to 10.",
    )
    handback_s: Annotated[int, Field(ge=1, le=600)] = Field(
        default=10,
        description="Wall seconds a seat stays with its fallback model after a handback, from 1 to 600.",
    )
    seat_retry_s: Annotated[int, Field(ge=1, le=3600)] = Field(
        default=60,
        description="Wall seconds between joins while another client holds the seat, from 1 to 3600.",
    )
    recall_episodes: Annotated[int, Field(ge=0, le=8)] = Field(
        default=8,
        description="How many episode lines of memory a consult shows, from 0 to 8.",
    )
    greet_cooldown_s: Annotated[int, Field(ge=0, le=86400)] = Field(
        default=1800,
        description="World seconds before the same person may trigger a greeting again, from 0 to 86400.",
    )
    horizon_minutes: Annotated[int, Field(ge=15, le=480)] = Field(
        default=60,
        description=(
            "World minutes before a punctual next part of the day begins at which the seat is first asked"
            " to think about it (the `upcoming` trigger), from 15 to 480."
        ),
    )
    reask_minutes: Annotated[int, Field(ge=5, le=120)] = Field(
        default=15,
        description=(
            "World minutes between two `upcoming` asks while the seat has not set out, arrived, or seen the"
            " part begin, from 5 to 120."
        ),
    )


class ControllerOverrides(ConfigModel):
    """The same keys as `[controller]`, each optional: a `[seat_overrides.<key>.controller]` table."""

    consult_deadline_s: Annotated[int, Field(ge=1, le=120)] | None = Field(
        default=None, description="Overrides `[controller]` consult_deadline_s for this seat."
    )
    max_calls: Annotated[int, Field(ge=1, le=10)] | None = Field(
        default=None, description="Overrides `[controller]` max_calls for this seat."
    )
    handback_s: Annotated[int, Field(ge=1, le=600)] | None = Field(
        default=None, description="Overrides `[controller]` handback_s for this seat."
    )
    seat_retry_s: Annotated[int, Field(ge=1, le=3600)] | None = Field(
        default=None, description="Overrides `[controller]` seat_retry_s for this seat."
    )
    recall_episodes: Annotated[int, Field(ge=0, le=8)] | None = Field(
        default=None, description="Overrides `[controller]` recall_episodes for this seat."
    )
    greet_cooldown_s: Annotated[int, Field(ge=0, le=86400)] | None = Field(
        default=None, description="Overrides `[controller]` greet_cooldown_s for this seat."
    )
    horizon_minutes: Annotated[int, Field(ge=15, le=480)] | None = Field(
        default=None, description="Overrides `[controller]` horizon_minutes for this seat."
    )
    reask_minutes: Annotated[int, Field(ge=5, le=120)] | None = Field(
        default=None, description="Overrides `[controller]` reask_minutes for this seat."
    )


class LayerConfig(ConfigModel):
    """One retry layer or the breaker: a policy id and its parameters (checked by its registry, C7/C8)."""

    policy: Annotated[str, Field(pattern=POLICY_ID)] = Field(
        description="The policy's id, one of the installed policies (for example `exponential-backoff`)."
    )
    parameters: dict[str, JsonValue] = Field(
        default_factory=dict[str, JsonValue],
        description="The policy's parameters. The policy checks them; an unknown one is refused by name.",
    )


class RetryConfig(ConfigModel):
    """The three retry layers of one table: `[retry]`, `[retry.<layer>]` per table."""

    transport: LayerConfig | None = Field(
        default=None,
        description="The transport layer (backoff, Retry-After); unset uses the bundled default.",
    )
    content: LayerConfig | None = Field(
        default=None,
        description="The content layer (invalid answers, re-asked); unset uses the bundled default.",
    )
    semantic: LayerConfig | None = Field(
        default=None,
        description="The semantic layer (refused proposals, re-planned); unset uses the bundled default.",
    )


class SeatOverrides(ConfigModel):
    """A `[seat_overrides.<key>]` table: per-seat controller bounds and per-seat retry layers (D-P6b-3)."""

    controller: ControllerOverrides = Field(
        default_factory=ControllerOverrides,
        description="The `[seat_overrides.<key>.controller]` table: the seat's own bounds.",
    )
    retry: RetryConfig = Field(
        default_factory=RetryConfig,
        description="The `[seat_overrides.<key>.retry.<layer>]` tables: the seat's own retry layers.",
    )


BUNDLED_RETRY: dict[Layer, LayerConfig] = {
    "transport": LayerConfig(
        policy="exponential-backoff",
        parameters={"attempts": 3, "first_s": 0.5, "factor": 2, "ceiling_s": 8.0, "jitter": 0.5},
    ),
    "content": LayerConfig(policy="fixed", parameters={"attempts": 3}),
    "semantic": LayerConfig(policy="fixed", parameters={"attempts": 2}),
}
"""The bundled defaults of the three layers (pr-s10-p6 §5.5.2, §5.7, ruled QP6-7)."""


def layer_policy(
    config: CognitionConfig, seat: str, backend: str | None, layer: Layer
) -> LayerConfig:
    """Precedence, most specific first: `[seat_overrides.<seat>.retry.<layer>]`, then
    `[backends.<backend>.retry.<layer>]` (the backend bound to `social`), then `[retry.<layer>]`, then
    the bundled default (pr-s10-p6 §5.13)."""
    seat_override = config.seat_overrides.get(seat)
    candidates: list[LayerConfig | None] = [
        None if seat_override is None else getattr(seat_override.retry, layer),
        None
        if backend is None
        or backend not in config.backends
        or config.backends[backend].retry is None
        else getattr(config.backends[backend].retry, layer),
        getattr(config.retry, layer),
    ]
    for candidate in candidates:
        if candidate is not None:
            return candidate
    return BUNDLED_RETRY[layer]


def controller_for(config: CognitionConfig, seat: str) -> ControllerConfig:
    """The seat's controller bounds: `[controller]` with the seat's overrides applied, validated."""
    seat_override = config.seat_overrides.get(seat)
    merged = config.controller.model_dump()
    if seat_override is not None:
        merged.update(seat_override.controller.model_dump(exclude_none=True))
    return ControllerConfig.model_validate(merged)


class CognitionConfig(ConfigModel):
    """The whole `cognition.toml` file: one table per section below, and the backends it names."""

    recording: RecordingConfig = Field(
        description="The `[recording]` table. Required: it names the mode every answer comes from.",
    )
    tiers: dict[Tier, str] = Field(
        default_factory=dict[Tier, str],
        description=(
            "The `[tiers]` table: which backend answers each decision tier. Keys are tier names "
            "(`ordinary_decision`, `social`, `major_decision`, `summarize`; their meaning is in "
            "ARCHITECTURE.md §9.1). Values are a backend name from `[backends.<name>]`, or `none` for a "
            "tier that uses no model. A tier left out uses no model, the same as `none`."
        ),
    )
    budgets: BudgetConfig = Field(
        default_factory=BudgetConfig,
        description="The `[budgets]` table: the most calls and tokens the client may spend.",
    )
    backends: dict[Annotated[str, Field(pattern=BACKEND_NAME)], BackendConfig] = Field(
        default_factory=dict[str, BackendConfig],
        description=(
            "The `[backends.<name>]` tables: one per backend, named by you. A name uses lowercase "
            "letters, digits, `-` and `_`, and does not start or end with `-` or `_`."
        ),
    )
    secrets: SecretsConfig = Field(
        default_factory=SecretsConfig,
        description="The `[secrets]` table: where API keys are read from.",
    )
    server: str | None = Field(
        default=None,
        description=(
            "The protocol address of the server, for example `ws://127.0.0.1:7878/ws`. Required by the "
            "LM controller (`python -m mineworld_cognition`); a loopback address is the usual choice."
        ),
    )
    invite_env: str = Field(
        default="MINEWORLD_INVITE",
        description=(
            "The NAME of the environment variable that holds the server's invite. The invite is read "
            "from the environment only; this file never holds it."
        ),
    )
    nickname: Annotated[str, Field(min_length=1, max_length=32)] = Field(
        default="cognition",
        description="The nickname the controller joins with, from 1 to 32 characters.",
    )
    store: str | None = Field(
        default=None,
        description=(
            "The directory of the controller's own state (memory stores, resume secrets, decision logs). "
            "A relative path is resolved against this configuration file. It must not be inside a world."
        ),
    )
    decisions_json: str | None = Field(
        default=None,
        description=(
            "The JSON that `mineworld decisions … --json` printed for the world the server runs. A relative "
            "path is resolved against this configuration file."
        ),
    )
    seats: list[Annotated[str, Field(pattern=SEAT_KEY)]] | None = Field(
        default=None,
        description=(
            "Optional subset of the `lm` seats in `decisions_json` to join. Every entry must be an `lm` "
            "seat of that file; unset joins all of them."
        ),
    )
    controller: ControllerConfig = Field(
        default_factory=ControllerConfig,
        description="The `[controller]` table: the consult's bounds and the seat's timing.",
    )
    memory: MemoryConfig = Field(
        default_factory=MemoryConfig,
        description="The `[memory]` table: where seats' memory is stored.",
    )
    retry: RetryConfig = Field(
        default_factory=RetryConfig,
        description="The `[retry]` table: the three retry layers of every seat (`[retry.<layer>]`).",
    )
    breaker: LayerConfig | None = Field(
        default=None,
        description="The `[breaker]` table: the per-seat circuit breaker. Unset uses the bundled default.",
    )
    seat_overrides: dict[Annotated[str, Field(pattern=SEAT_KEY)], SeatOverrides] = Field(
        default_factory=dict[str, SeatOverrides],
        description=(
            "The `[seat_overrides.<key>]` tables: per-seat controller bounds and retry layers. The name "
            "`seat_overrides` is used so that `seats` can name the subset of seats (D-P6b-3)."
        ),
    )

    @field_validator("invite_env")
    @classmethod
    def _invite_env(cls, value: str) -> str:
        if not KEY_ENV.fullmatch(value):
            raise ValueError(
                "invite_env is the NAME of an environment variable (A-Z, 0-9, _), never the invite"
            )
        return value

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
    if config.memory.sharing == "service":
        # F-S28f-7: raised here, not in a validator, so the type and its `mode` survive as `__cause__`.
        unavailable = SharingModeUnavailable(config.memory.sharing)
        raise ConfigError(f"{path}: {unavailable}") from unavailable
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
            "store": resolved(config.store),
            "decisions_json": resolved(config.decisions_json),
        }
    )
    if config.secrets.env_file is not None:
        _refuse_inside_world(Path(config.secrets.env_file), "the env_file")
    if config.store is not None:
        _refuse_inside_world(Path(config.store), "the store directory")
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
