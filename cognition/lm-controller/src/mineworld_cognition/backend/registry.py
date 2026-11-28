"""Adapter `kind` → factory (R-P5b-1: P5b adds its adapters here and to `BackendConfig.kind` only).

Each factory imports its adapter when called, never at module import, so a `replay` or `scripted`
process that reads this module still loads no network client (D-P5-5, AP5-6).
"""

from __future__ import annotations

from collections.abc import Callable

from mineworld_sdk.wire.ids import JsonValue

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


def serves_model(config: BackendConfig, timeout_s: float) -> bool | None:
    """Whether the backend's local service lists the configured model (`GET <base_url>/models`, the
    OpenAI-compatible listing; a supervised preflight, `docs/reference/cognition-supervised.md` §3): a listing,
    no inference. `None` for a kind with no such listing; `False` when the service does not answer."""
    if config.kind != "openai-compatible":
        return None
    import json
    import urllib.error
    import urllib.request

    url = config.base_url.rstrip("/") + "/models"
    try:
        with urllib.request.urlopen(url, timeout=timeout_s) as response:
            listed: JsonValue = json.loads(response.read())
    except (urllib.error.URLError, OSError, ValueError):
        return False
    models = listed.get("data") if isinstance(listed, dict) else None
    if not isinstance(models, list):
        return False
    return any(isinstance(entry, dict) and entry.get("id") == config.model for entry in models)
