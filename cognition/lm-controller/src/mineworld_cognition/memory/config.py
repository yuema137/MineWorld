"""`memory.config`: the `[memory]` table of `cognition.toml` (pr-s28f-home-wiring.md §5.2; step-28 §§5.4, 5.5).

Placement is the operator's: where the seats of one process store their memory, how large the memory file's page
cache is, and which debug checks run. Placement never changes what a seat recalls (I-S28-2), so it lives in
`cognition.toml`, never in a world (QS28-10). The layout itself is fixed (QS28f-2) and there is no record LRU
(QS28f-1).
"""

from __future__ import annotations

from typing import Annotated, Literal

from pydantic import Field

from mineworld_cognition.backend.model import CognitionModel

type Sharing = Literal["none", "home", "service"]
PAGE_CACHE_KIB = 8192
"""The memory file's page cache by default, in KiB (SQLite `cache_size`)."""
PAGE_CACHE_KIB_MIN = 256
PAGE_CACHE_KIB_MAX = 1_048_576


class MemoryConfig(CognitionModel):
    """The `[memory]` table: where seats' memory is stored. Placement never changes what a seat recalls."""

    sharing: Sharing = Field(
        default="home",
        description=(
            "How the seats of this process store memory. `home`: one file per world instance for all seats, "
            "storing each perceived fact once. `none`: one file per seat. `service` is not available yet. "
            "Either choice gives every seat exactly the same memory."
        ),
    )
    page_cache_kib: Annotated[int, Field(ge=PAGE_CACHE_KIB_MIN, le=PAGE_CACHE_KIB_MAX)] = Field(
        default=PAGE_CACHE_KIB,
        description=(
            "The memory file's page cache in KiB, from 256 to 1 048 576 (SQLite `cache_size`). More makes "
            "recall faster and uses more RAM."
        ),
    )
    verify: Literal["off", "pass", "every-commit"] = Field(
        default="off",
        description=(
            "Debug checks on the memory file. `off`: none. `pass`: before each forgetting pass commits. "
            "`every-commit`: before every commit. A failed check stops the seat."
        ),
    )


class SharingModeUnavailable(ValueError):
    """memory.config: `sharing = "service"` is MVP-1's cross-process sharing (step-28 §6.6); refused before it."""

    def __init__(self, mode: str) -> None:
        self.mode = mode
        super().__init__(
            f'memory.config: sharing = "{mode}" is not available yet (cross-process sharing is MVP-1); '
            'use "home" or "none"'
        )
