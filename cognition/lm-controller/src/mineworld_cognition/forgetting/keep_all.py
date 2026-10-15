"""`keep-all` (pr-fg-b §5.9; QS24-10): keeps everything and leaves the bound to the host. A test fixture for
historical comparisons and the hostile-policy test, never a world's choice: only the host's backstop
limits it (AFGb-4)."""

from __future__ import annotations

from collections.abc import Sequence
from typing import ClassVar

from pydantic import BaseModel, ConfigDict

from mineworld_cognition.memory.retention import Drop, Keep, MemoryItem, Merge, RetentionBudget


class KeepAllParameters(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")


class KeepAll:
    id: ClassVar[str] = "keep-all"
    Parameters = KeepAllParameters

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: KeepAllParameters
    ) -> Sequence[Keep | Merge | Drop]:
        return ()
