"""Named provider presets for the one OpenAI-compatible adapter (D-P5-15; §4.1c). Data only, no code path.

A `[backends.<name>]` table may say `preset = "<name>"`; every field it leaves out is filled from the
row, and a field it sets wins. The adapter never sees the preset name, and neither does a request, a
cassette key or a cassette's metadata. No row names a model (model names change monthly; the user names
one) and no row holds a key: `key_env` is a variable's NAME. Adding a provider is one row here and one
row in the README's table (AP5-14 (f)). Each row was checked against the provider's own documentation
on 2026-10-09 (ledger §13.7); what could not be confirmed is marked "verify" in the README.
"""

from __future__ import annotations

from collections.abc import Mapping
from dataclasses import dataclass
from typing import Literal

StructuredOutput = Literal["json_schema", "json_object", "none"]
Reasoning = Literal["unset", "off", "low", "medium", "high"]
TemperatureMode = Literal["send", "omit"]

PresetName = Literal[
    "openai",
    "xai",
    "deepseek",
    "glm",
    "zai",
    "mistral",
    "moonshot",
    "dashscope",
    "gemini",
    "openrouter",
    "groq",
]


@dataclass(frozen=True)
class ProviderPreset:
    name: PresetName
    base_url: str | None
    """An https URL, or None when the URL is per account or region and the user must give it."""
    key_env: str
    structured_output: StructuredOutput
    temperature: TemperatureMode
    reasoning: Reasoning


PRESETS: Mapping[str, ProviderPreset] = {
    preset.name: preset
    for preset in (
        ProviderPreset("openai","https://api.openai.com/v1", "OPENAI_API_KEY", "json_schema", "send", "unset"),
        ProviderPreset("xai", "https://api.x.ai/v1", "XAI_API_KEY", "json_schema", "send", "unset"),
        ProviderPreset("deepseek", "https://api.deepseek.com", "DEEPSEEK_API_KEY", "json_object", "send", "unset"),
        ProviderPreset("glm", "https://open.bigmodel.cn/api/paas/v4", "ZHIPUAI_API_KEY", "json_object", "send", "unset"),
        ProviderPreset("zai", "https://api.z.ai/api/paas/v4", "ZAI_API_KEY", "json_object", "send", "unset"),
        ProviderPreset("mistral", "https://api.mistral.ai/v1", "MISTRAL_API_KEY", "json_schema", "send", "unset"),
        ProviderPreset("moonshot", "https://api.moonshot.ai/v1", "MOONSHOT_API_KEY", "json_object", "omit", "unset"),
        ProviderPreset("dashscope", None, "DASHSCOPE_API_KEY", "json_object", "send", "unset"),
        ProviderPreset("gemini", "https://generativelanguage.googleapis.com/v1beta/openai", "GEMINI_API_KEY", "json_schema", "send", "unset"),
        ProviderPreset("openrouter", "https://openrouter.ai/api/v1", "OPENROUTER_API_KEY", "json_schema", "send", "unset"),
        ProviderPreset("groq", "https://api.groq.com/openai/v1", "GROQ_API_KEY", "json_object", "send", "unset"),
    )
}  # fmt: skip
