"""The one interface MineWorld's content-production tooling needs from an image generator.

Scope, deliberately narrow (`docs/REUSE_POLICY.md` §6 — do not abstract before a second
implementation exists):

* This is **content-production tooling, not world logic**. It runs on an operator's machine to
  produce candidate art. The server, the clients, the kernel and the systems never import it,
  and no contract references it. It is never a runtime dependency of anything.
* There is exactly one interface and one implementation
  (`openai_images.OpenAIImageProvider`). No registry, no plugin discovery, no factory. When a
  second backend genuinely arrives, that is the moment to add whatever dispatch it needs.

Why the interface exists at all rather than calling the vendor SDK directly at each call site:
ARC-9's roadmap already anticipates the backend changing, and the call site — prompt,
references, params in; image plus provenance out — is the part worth keeping stable. The
fields below are kept vendor-neutral in *meaning*; anything that is genuinely specific to one
vendor goes in `GenerationParams.extra` rather than growing a field here.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Protocol, Sequence, runtime_checkable

__all__ = [
    "GenerationParams",
    "GenerationResult",
    "ImageGenerationProvider",
]


@dataclass(frozen=True)
class GenerationParams:
    """Everything that is not the prompt or the reference images.

    Recorded verbatim in the provenance sidecar, so a candidate can be regenerated from its
    sidecar alone.
    """

    #: "WIDTHxHEIGHT". Providers reject sizes they cannot honour rather than silently resizing.
    size: str = "1024x1024"
    #: Ask for an alpha channel. Essential for props and vegetation; meaningless for scenes.
    transparent_background: bool = False
    #: Relative effort/fidelity. Neutral ladder; providers map it onto their own vocabulary.
    quality: str = "high"
    output_format: str = "png"
    #: Only meaningful for backends that expose one. Recorded as null when unavailable, which
    #: is itself provenance: it says this candidate is not bit-reproducible.
    seed: int | None = None
    #: Provider-specific escape hatch. Recorded in provenance; never interpreted here.
    extra: dict[str, object] = field(default_factory=dict)


@dataclass(frozen=True)
class GenerationResult:
    #: PNG/WEBP bytes exactly as the provider returned them — not re-encoded, so no generation
    #: loss and no silent alpha flattening between here and disk.
    image_bytes: bytes
    #: ARC-9 shaped record. `provenance.write_sidecar` turns this into a file.
    provenance: dict[str, object]
    #: Wall time and, where the provider reports it, billed usage and cost for this one image.
    stats: dict[str, object] = field(default_factory=dict)


@runtime_checkable
class ImageGenerationProvider(Protocol):
    """`generate(prompt, reference_images, params) -> image`, plus the identity provenance needs."""

    #: Stable short name for the backend, e.g. "openai-images".
    name: str
    #: Model identity precise enough to reproduce, e.g. a pinned model or snapshot id.
    model_id: str
    #: Bumped by hand when the request construction changes in a way that changes output.
    workflow_version: str

    def generate(
        self,
        prompt: str,
        reference_images: Sequence[Path] = (),
        params: GenerationParams | None = None,
    ) -> GenerationResult:
        """Produce one candidate image.

        `reference_images` are **style** references: the generator is asked to match their look,
        not to reproduce their content. A provider that cannot do reference conditioning must
        raise rather than quietly falling back to a text-only prompt — a silently text-only
        result is the exact failure ARC-9's human curation step is least likely to catch,
        because it still looks like competent art.
        """
        ...
