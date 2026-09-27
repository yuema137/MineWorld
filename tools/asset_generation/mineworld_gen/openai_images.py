"""The first — and, per `docs/REUSE_POLICY.md` §6, currently the only — `ImageGenerationProvider`.

OpenAI's Images API. The capability that decides the design is that `/v1/images/edits` accepts
an **array** of input images, which is what makes genuine reference-guided generation possible:
MineWorld's four canonical 2D references go in with every request, and the prompt's job is to
say "match this style, do not copy this content".

Model selection is deliberately *not* hardcoded from memory. `DEFAULT_MODEL` is pinned to a
verified-current identifier and is overridable per call and by environment variable, because an
obsolete model id is a silent failure rather than a loud one.

Licence position for outputs: OpenAI's terms assign the customer ownership of output images and
permit commercial use and redistribution, so candidates produced here can be committed to
MineWorld's MIT repository. The constraint that remains is factual, not legal — outputs must
still pass ARC-9 human curation before becoming assets. See `../README.md`.
"""

from __future__ import annotations

import base64
import os
import time
from pathlib import Path
from typing import Sequence

from .credentials import load_api_key
from .provider import GenerationParams, GenerationResult

__all__ = ["OpenAIImageProvider", "DEFAULT_MODEL", "MODEL_ENV"]

#: Verified against the live model catalogue on 2026-09-27, not recalled from training data.
#: `gpt-image-2.5-flare` is the fast high-quality tier; `gpt-image-2.5-sunburst` is the most
#: capable and slower. Override per call, or with MINEWORLD_IMAGE_MODEL.
DEFAULT_MODEL = "gpt-image-2.5-flare"
MODEL_ENV = "MINEWORLD_IMAGE_MODEL"

WORKFLOW_VERSION = "1.0.0"

#: USD per 1M tokens, from the published pricing table (2026-09-27). Used only to report an
#: estimate alongside each candidate; the invoice is authoritative.
PRICE_PER_MTOK = {"image_input": 8.0, "text_input": 5.0, "image_output": 30.0}

LICENSE_NOTE = (
    "Generated with the OpenAI Images API. Under OpenAI's terms of use the customer owns the "
    "output images and may use and redistribute them commercially, so these candidates may be "
    "committed to and redistributed from MineWorld's MIT-licensed repository. No model weights "
    "are redistributed. Reference images are MineWorld's own committed style references."
)


class OpenAIImageProvider:
    """`generate(prompt, reference_images, params) -> GenerationResult`.

    With references it calls `images.edit` (multi-image reference conditioning); without them it
    calls `images.generate`. Callers do not choose — passing references is the choice.
    """

    name = "openai-images"
    workflow_version = WORKFLOW_VERSION

    def __init__(self, model: str | None = None, api_key: str | None = None) -> None:
        self.model_id = model or os.environ.get(MODEL_ENV) or DEFAULT_MODEL
        self._api_key = api_key or load_api_key()
        self._client = None

    @property
    def client(self):
        if self._client is None:
            from openai import OpenAI

            self._client = OpenAI(api_key=self._api_key)
        return self._client

    # ------------------------------------------------------------------------------ generate

    def generate(
        self,
        prompt: str,
        reference_images: Sequence[Path] = (),
        params: GenerationParams | None = None,
    ) -> GenerationResult:
        from .provenance import build_provenance

        params = params or GenerationParams()
        refs = [Path(p) for p in reference_images]
        for ref in refs:
            if not ref.is_file():
                raise FileNotFoundError(f"reference image not found: {ref}")

        request: dict[str, object] = {
            "model": self.model_id,
            "prompt": prompt,
            "size": params.size,
            "quality": params.quality,
            "output_format": params.output_format,
            "background": "transparent" if params.transparent_background else "opaque",
        }
        request.update(params.extra)

        started = time.perf_counter()
        if refs:
            handles = [p.open("rb") for p in refs]
            try:
                response = self.client.images.edit(image=handles, **request)
            finally:
                for handle in handles:
                    handle.close()
        else:
            response = self.client.images.generate(**request)
        elapsed = time.perf_counter() - started

        datum = response.data[0]
        if not getattr(datum, "b64_json", None):
            raise RuntimeError("provider returned no image payload")
        image_bytes = base64.b64decode(datum.b64_json)

        usage = _usage_dict(getattr(response, "usage", None))
        cost = _estimate_cost(usage)
        stats = {
            "wall_seconds": round(elapsed, 2),
            "usage": usage,
            "cost_usd_estimate": cost,
            "bytes": len(image_bytes),
        }

        recorded = {
            "size": params.size,
            "quality": params.quality,
            "background": request["background"],
            "output_format": params.output_format,
            "endpoint": "images.edit" if refs else "images.generate",
            "revised_prompt": getattr(datum, "revised_prompt", None),
            **({"extra": params.extra} if params.extra else {}),
        }

        provenance = build_provenance(
            provider_name=self.name,
            model_id=self.model_id,
            workflow_version=self.workflow_version,
            prompt=prompt,
            params=recorded,
            # The Images API exposes no seed, so candidates are not bit-reproducible. Recording
            # null states that honestly rather than inventing a number that reproduces nothing.
            seed=params.seed,
            reference_images=refs,
            repo_root=_repo_root(),
            license_note=LICENSE_NOTE,
            redistribution_allowed=True,
        )
        provenance["generation"]["runtime"] = {
            "wall_seconds": stats["wall_seconds"],
            "cost_usd_estimate": cost,
            "usage": usage,
        }
        return GenerationResult(image_bytes=image_bytes, provenance=provenance, stats=stats)


def _usage_dict(usage) -> dict[str, object]:
    if usage is None:
        return {}
    if hasattr(usage, "model_dump"):
        return usage.model_dump(exclude_none=True)
    return dict(usage)


def _estimate_cost(usage: dict[str, object]) -> float | None:
    if not usage:
        return None
    details = usage.get("input_tokens_details") or {}
    text_in = float(details.get("text_tokens", 0) or 0)
    image_in = float(details.get("image_tokens", 0) or 0)
    if not (text_in or image_in):
        image_in = float(usage.get("input_tokens", 0) or 0)
    out = float(usage.get("output_tokens", 0) or 0)
    total = (
        text_in * PRICE_PER_MTOK["text_input"]
        + image_in * PRICE_PER_MTOK["image_input"]
        + out * PRICE_PER_MTOK["image_output"]
    ) / 1e6
    return round(total, 5)


def _repo_root() -> Path | None:
    for parent in Path(__file__).resolve().parents:
        if (parent / ".git").exists() or (parent / "Cargo.toml").exists():
            return parent
    return None
