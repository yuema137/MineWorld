"""The typed retention interface (pr-fg-b §5.2): what a `ForgettingPolicy` is shown and what it returns.

A policy is a pure function from frozen values to decisions. It never receives a store, a connection, a
clock, a path or a gateway, and it never writes: the host validates its plan and executes it (FGb-3).
Importance is the host's (§5.4); recency is the policy's own arithmetic on `budget.now - item.last_at`,
never time of retrieval (ARC-87). The helpers below are integer-only, so two runs, two platforms and two
rebuilds compute the same decisions.
"""

from __future__ import annotations

import hashlib
from collections.abc import Sequence
from typing import Annotated, ClassVar, Literal, Protocol

from mineworld_sdk.wire.ids import EntityIdField
from pydantic import Field

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.memory.records import Citation, ItemId

ItemLevel = Literal[0, 1, 2, 3]
"""0 an L0 record, 1 an episode, 2 a chapter, 3 a stable fact."""

Importance = Annotated[int, Field(ge=0, le=1000)]
PERMILLE = 1000


class MemoryItem(CognitionModel):
    """One eligible item of a seat's store, as the host shows it to a policy."""

    item: ItemId
    level: ItemLevel
    container: ItemId | None
    """The unit it nests in: an L0 record's episode; an episode's week chapter, if one exists; otherwise
    `None`, and the item is top-level."""
    first_at: int
    """World seconds of its earliest fact."""
    last_at: int
    day: Annotated[int, Field(ge=0)]
    """`first_at // 86 400`."""
    kind: str
    """An L0 record's event type; otherwise `episode`, `chapter` or `stable`."""
    counterparts: tuple[EntityIdField, ...]
    place: EntityIdField | None
    mine: bool
    """An L0 record: I was the actor. An episode or chapter: one of its records was mine."""
    importance: Importance
    """Computed by the host at the pass (§5.4); never by a policy."""
    citation: Citation
    """The perceived facts it stands for."""
    text: str
    """Names-only text (P4-5): a gist, a summary, or a stable fact's line."""
    bytes: Annotated[int, Field(ge=1)]
    """Its own rows' lines in the canonical dump: what removing it alone subtracts from the store."""


class RetentionBudget(CognitionModel):
    store_bytes: Annotated[int, Field(ge=0)]
    """Bytes the surviving items should fit in."""
    now: int
    """World seconds: the start of the day whose first fact triggered the pass (§5.6)."""
    calls_left: Annotated[int, Field(ge=0)]
    """Model calls this pass may make: 0 for every no-model policy."""


class Keep(CognitionModel):
    kind: Literal["keep"] = "keep"
    item: ItemId


class Merge(CognitionModel):
    """Items merged into their shared container, which survives and already cites them: L0 records into
    their episode (`into=1`), or episodes into their chapter (`into=2`)."""

    kind: Literal["merge"] = "merge"
    items: Annotated[tuple[ItemId, ...], Field(min_length=1)]
    into: Literal[1, 2]


class Drop(CognitionModel):
    """A top-level item removed with everything nested in it; its citation joins the forgotten set."""

    kind: Literal["drop"] = "drop"
    item: ItemId


RetentionDecision = Annotated[Keep | Merge | Drop, Field(discriminator="kind")]


class ForgettingPolicy[P: CognitionModel](Protocol):
    """A retention policy: found through the entry-point group `mineworld.forgetting` under its `id`.

    `Parameters` is a strict model whose every field is bounded and has a default; the host validates a
    world's parameters into it once, when the policy is bound, and passes the result to every `plan`."""

    id: ClassVar[str]
    Parameters: type[P]

    def plan(
        self, items: Sequence[MemoryItem], budget: RetentionBudget, params: P
    ) -> Sequence[Keep | Merge | Drop]: ...


def half_life_permille(elapsed_s: int, half_life_s: int) -> int:
    """Retention in per mille after `elapsed_s`: 1 000 halved once per whole half-life (`1000 >> k`),
    linear between whole half-lives; 0 from the tenth on. Negative ages count as 0."""
    elapsed = max(0, elapsed_s)
    whole, rest = divmod(elapsed, half_life_s)
    high, low = PERMILLE >> whole, PERMILLE >> (whole + 1)
    return high - (high - low) * rest // half_life_s


def hyperbolic_permille(elapsed_s: int, scale_s: int) -> int:
    """Retention in per mille after `elapsed_s` on a hyperbolic (power-law, exponent 1) curve:
    `1000 · s // (s + Δ)`, so one scale of age leaves 500."""
    return PERMILLE * scale_s // (scale_s + max(0, elapsed_s))


def tie_key(seed: int, item: MemoryItem) -> int:
    """A seeded tie-break that is a pure function of the item: the sha256 of `"<seed>:<first Event ID>"`
    (UTF-8) read as a big-endian integer. The "random" key of priority sampling, without RNG state."""
    first = item.citation.ranges[0].first
    digest = hashlib.sha256(f"{seed}:{first}".encode()).digest()
    return int.from_bytes(digest, "big")
