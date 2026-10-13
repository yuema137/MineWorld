"""The operator-run model spike (QS10-2; pr-s10-p5-backends.md C7 and AP5-S). Never run by CI or an agent.

    uv run python cognition/lm-controller/tools/model_spike.py --config FILE --models a,b,c --out DIR

It asks each model, listed **smallest first**, the 40 fixed scenarios of `spike_scenarios.json` once,
through `ModelGateway` with a recorder, using the backend bound to the `social` tier in FILE (only its
`model` is replaced). It writes `DIR/spike-<model>.jsonl` (a cassette per model) and `DIR/report.json`.
The default is the first model that meets every criterion below; if none does, the report says so and the
thresholds are not changed (QS10-2's criteria are fixed). A non-loopback `base_url` is refused unless
`--allow-remote` is given.
"""

from __future__ import annotations

import argparse
import asyncio
import hashlib
import json
import os
import platform
import re
import sys
import time
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from typing import Annotated, Literal
from urllib.parse import urlsplit

from mineworld_sdk.wire.ids import EntityKey, JsonValue
from pydantic import BaseModel, ConfigDict, Field, ValidationError

from mineworld_cognition.backend.model import (
    CompletionRequest,
    Message,
    ModelBackend,
    Sampling,
)
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger, SystemClock
from mineworld_cognition.config import BackendConfig, CognitionConfig, is_loopback_host, load
from mineworld_cognition.gateway import Budget, Completed, Failed, ModelGateway, Refused
from mineworld_cognition.record import RecordingBackend

SCENARIOS = Path(__file__).resolve().parent / "spike_scenarios.json"

# AP5-S, fixed before any run. Changing one after a run is a material stop (§11).
SCENARIO_COUNT = 40
FIRST_ATTEMPT_VALID_MIN = 38
P95_LATENCY_MAX_MS = 15_000
WORDS_MAX_BYTES = 480

SCHEMA: JsonValue = {
    "type": "object",
    "properties": {
        "act": {"enum": ["say", "wait", "leave"]},
        "to": {"type": ["string", "null"]},
        "words": {"type": "string", "maxLength": WORDS_MAX_BYTES},
        "recalls": {"type": "array", "items": {"type": "string"}},
    },
    "required": ["act", "to", "words", "recalls"],
    "additionalProperties": False,
}


class SpikeChoice(BaseModel):
    """The `SocialChoice`-shaped answer the schema asks for, checked locally (the authority, R-P5-2)."""

    model_config = ConfigDict(strict=True, extra="forbid")

    act: Literal["say", "wait", "leave"]
    to: str | None
    words: str
    recalls: list[str]


class Scenario(BaseModel):
    model_config = ConfigDict(strict=True, extra="forbid")

    id: Annotated[str, Field(pattern=r"^s\d\d$")]
    persona: str
    heard: str


class ScenarioFile(BaseModel):
    model_config = ConfigDict(strict=True, extra="forbid")

    about: str
    scenarios: Annotated[
        list[Scenario], Field(min_length=SCENARIO_COUNT, max_length=SCENARIO_COUNT)
    ]


def scenarios() -> list[Scenario]:
    return ScenarioFile.model_validate_json(SCENARIOS.read_bytes()).scenarios


def scenario_request(scenario: Scenario) -> CompletionRequest:
    system = (
        f"You are {scenario.persona}, living in a small town. Answer only with JSON matching the "
        "schema: act is say, wait or leave; to is who you speak to, or null; words is what you say "
        "(empty unless act is say); recalls lists what you remember that matters here."
    )
    return CompletionRequest(
        purpose="decide",
        messages=(Message(role="system", text=system), Message(role="user", text=scenario.heard)),
        output_schema=SCHEMA,
        sampling=Sampling(temperature_milli=700, max_output_tokens=256, seed=7),
    )


def schema_valid(text: str) -> bool:
    try:
        choice = SpikeChoice.model_validate_json(text)
    except ValidationError:
        return False
    return len(choice.words.encode("utf-8")) <= WORDS_MAX_BYTES


def p95_ms(latencies: list[int]) -> int | None:
    """Nearest-rank 95th percentile."""
    if not latencies:
        return None
    ordered = sorted(latencies)
    return ordered[-(-95 * len(ordered) // 100) - 1]


@dataclass
class ModelResult:
    model: str
    first_attempt_valid: int = 0
    completed: int = 0
    failures: dict[str, int] | None = None
    latencies_ms: list[int] | None = None

    def summary(self, reasoning: str) -> dict[str, JsonValue]:
        latencies = self.latencies_ms or []
        failures = self.failures or {}
        p95 = p95_ms(latencies)
        meets = (
            self.first_attempt_valid >= FIRST_ATTEMPT_VALID_MIN
            and p95 is not None
            and p95 <= P95_LATENCY_MAX_MS
        )
        return {
            "model": self.model,
            "scenarios": SCENARIO_COUNT,
            "first_attempt_schema_valid": self.first_attempt_valid,
            "completed": self.completed,
            "failures": dict(failures),
            "latency_p50_ms": None
            if not latencies
            else sorted(latencies)[(len(latencies) - 1) // 2],
            "latency_p95_ms": p95,
            "reasoning_setting": reasoning,
            # A server that rejects the reasoning field answers 400 to every call (http_status:400).
            "reasoning_accepted": failures.get("http_status:400", 0) == 0,
            "meets_criteria": meets,
        }


BackendFactory = Callable[[BackendConfig], ModelBackend]


def _live_factory(config: CognitionConfig, name: str) -> BackendFactory:
    from mineworld_cognition.backend.registry import build_backend
    from mineworld_cognition.secrets import resolve_key

    env_file = None if config.secrets.env_file is None else Path(config.secrets.env_file)

    def factory(backend: BackendConfig) -> ModelBackend:
        key = resolve_key(backend.key_env, env_file, backend=name) if backend.key_env else None
        return build_backend(backend, key)

    return factory


def _machine() -> dict[str, JsonValue]:
    memory: int | None = None
    if sys.platform != "win32":  # os.sysconf is POSIX only; pyright narrows on sys.platform
        try:
            memory = os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES")
        except (ValueError, OSError):
            memory = None
    return {
        "os": platform.platform(),
        "machine": platform.machine(),
        "processor": platform.processor(),
        "cpus": os.cpu_count(),
        "memory_bytes": memory,
        "python": sys.version.split()[0],
    }


async def _run_model(
    backend_config: BackendConfig, name: str, factory: BackendFactory, out: Path
) -> ModelResult:
    safe = re.sub(r"[^A-Za-z0-9._-]", "_", backend_config.model)
    recorder = RecordingBackend(
        factory(backend_config),
        out / f"spike-{safe}.jsonl",
        binding=name,
        model=backend_config.model,
    )
    # The spike's own ceilings: 40 calls must not be refused by a seat's everyday budget.
    policy = BudgetPolicy(
        calls_per_wall_hour=10_000,
        tokens_per_wall_day=10_000_000,
        max_in_flight=1,
        call_timeout_s=120,
    )
    gateway = ModelGateway(recorder, Budget.of(policy, MemoryLedger(), SystemClock()))
    result = ModelResult(backend_config.model, failures={}, latencies_ms=[])
    assert result.failures is not None and result.latencies_ms is not None
    for scenario in scenarios():
        started = time.monotonic()
        outcome = await gateway.complete(EntityKey("spike"), scenario_request(scenario))
        elapsed = int((time.monotonic() - started) * 1000)
        match outcome:
            case Completed(completion=answer):
                result.completed += 1
                result.latencies_ms.append(elapsed)
                result.first_attempt_valid += schema_valid(answer.text)
            case Failed(failure=failure):
                label = failure.reason + ("" if failure.status is None else f":{failure.status}")
                result.failures[label] = result.failures.get(label, 0) + 1
            case Refused():
                result.failures["refused"] = result.failures.get("refused", 0) + 1
    await gateway.aclose()
    return result


def run_spike(
    config: CognitionConfig,
    models: list[str],
    out: Path,
    *,
    allow_remote: bool = False,
    factory: BackendFactory | None = None,
) -> dict[str, JsonValue]:
    name = config.tiers.get("social")
    if name is None or name == "none":
        raise SystemExit("the configuration binds no backend to the social tier")
    base = config.backends[name]
    host = urlsplit(base.base_url).hostname or ""
    if not is_loopback_host(host) and not allow_remote:
        raise SystemExit(
            f"{base.base_url} is not a loopback address; pass --allow-remote to use it"
        )
    out.mkdir(parents=True, exist_ok=True)
    build = factory or _live_factory(config, name)
    results: list[dict[str, JsonValue]] = []
    for model in models:
        result = asyncio.run(_run_model(base.model_copy(update={"model": model}), name, build, out))
        results.append(result.summary(base.reasoning))
    default = next((r["model"] for r in results if r["meets_criteria"] is True), None)
    report: dict[str, JsonValue] = {
        "criteria": {
            "first_attempt_schema_valid_min": FIRST_ATTEMPT_VALID_MIN,
            "latency_p95_max_ms": P95_LATENCY_MAX_MS,
            "scenarios": SCENARIO_COUNT,
            "scenarios_sha256": hashlib.sha256(SCENARIOS.read_bytes()).hexdigest(),
        },
        "server": {"backend": name, "structured_output": base.structured_output},
        "machine": _machine(),
        "models": list(results),
        "default": default,
        "verdict": "default chosen"
        if default
        else "no candidate meets the criteria: to the operator",
    }
    (out / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    return report


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0] if __doc__ else None)
    parser.add_argument("--config", type=Path, required=True)
    parser.add_argument("--models", required=True, help="comma-separated, smallest first")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--allow-remote", action="store_true")
    arguments = parser.parse_args(argv)
    models = [model.strip() for model in str(arguments.models).split(",") if model.strip()]
    report = run_spike(
        load(arguments.config), models, arguments.out, allow_remote=bool(arguments.allow_remote)
    )
    print(json.dumps({"default": report["default"], "verdict": report["verdict"]}))
    return 0


if __name__ == "__main__":
    sys.exit(main())
