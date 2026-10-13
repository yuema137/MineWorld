"""Adapter `kind` → factory (R-P5b-1: P5b adds its adapters here and to `BackendConfig.kind` only).

Each factory imports its adapter when called, never at module import, so a `replay` or `scripted`
process that reads this module still loads no network client (D-P5-5, AP5-6).
"""

from __future__ import annotations

from collections.abc import Callable

from mineworld_cognition.backend.model import ModelBackend
from mineworld_cognition.config import BackendConfig
from mineworld_cognition.secrets import Secret

Factory = Callable[[BackendConfig, Secret | None], ModelBackend]


def _openai_compatible(config: BackendConfig, key: Secret | None) -> ModelBackend:
    from mineworld_cognition.backend.openai_compatible import OpenAICompatibleBackend

    return OpenAICompatibleBackend(config, key)


def _anthropic(config: BackendConfig, key: Secret | None) -> ModelBackend:
    from mineworld_cognition.backend.anthropic_messages import AnthropicMessagesBackend

    return AnthropicMessagesBackend(config, key)


FACTORIES: dict[str, Factory] = {
    "openai-compatible": _openai_compatible,
    "anthropic": _anthropic,
}


def build_backend(config: BackendConfig, key: Secret | None) -> ModelBackend:
    return FACTORIES[config.kind](config, key)
