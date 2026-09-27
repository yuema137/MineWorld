"""MineWorld asset generation — content-production tooling.

Never imported by the server, the clients, the kernel or the systems, and referenced by no
contract. See `provider.py` for the scope rule.
"""

from .provider import GenerationParams, GenerationResult, ImageGenerationProvider
from .provenance import build_provenance, write_sidecar

__all__ = [
    "GenerationParams",
    "GenerationResult",
    "ImageGenerationProvider",
    "build_provenance",
    "write_sidecar",
]
