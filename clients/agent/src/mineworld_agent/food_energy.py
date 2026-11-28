"""My food-energy reserve, as `needs` discloses it to me (`docs/DECISIONS.md` ARC-370; FE-a design §3.8).

In a world whose `needs` section chooses the food-energy hunger model, the holder's `needs` record (schema 2)
carries a `food_energy` block: the whole kcal available now, the capacity, and the kcal used per world day.
An agent reads those three numbers; it computes none of them, and no bar (an agent reads numbers). The
adapter names the need already, so `replaces_need` is not repeated here. A block that is missing, malformed or
out of bound reads as nothing, never as a guess.
"""

from __future__ import annotations

from typing import Annotated

from mineworld_sdk.wire.ids import JsonValue
from pydantic import BaseModel, ConfigDict, Field, ValidationError

KCAL_MAX = 100_000
"""The largest kcal number the `needs` pack states (`MODULE_SPEC.md` §4.2)."""

NEEDS_SCHEMAS = (1, 2)
"""The `needs` record schemas this adapter reads; any other is not read at all."""


class FoodEnergyLine(BaseModel):
    """My food-energy reserve in kcal, as `needs` computed it for me now."""

    # The same strict, frozen, closed configuration as every `AgentModel` (models.py), stated here so this
    # module does not import models.py, which imports it.
    model_config = ConfigDict(
        strict=True, frozen=True, extra="forbid", use_attribute_docstrings=True
    )

    available_kcal: Annotated[int, Field(ge=0)]
    """Whole kcal I have now; at most `capacity_kcal`."""
    capacity_kcal: Annotated[int, Field(ge=1, le=KCAL_MAX)]
    """The most I can hold: my daily use."""
    use_kcal_per_world_day: Annotated[int, Field(ge=1, le=KCAL_MAX)]
    """How many kcal I use per world day (86 400 world seconds), asleep or awake."""


def food_energy(record: dict[str, JsonValue] | None) -> FoodEnergyLine | None:
    """The `food_energy` block of my `needs` record, or `None` when there is none or it does not read."""
    if record is None or record.get("hunger_model") != "food-energy":
        return None
    block = record.get("food_energy")
    if not isinstance(block, dict) or block.get("unit") != "kcal":
        return None
    numbers = {
        key: block.get(key) for key in ("available_kcal", "capacity_kcal", "use_kcal_per_world_day")
    }
    if not all(
        isinstance(value, int) and not isinstance(value, bool) for value in numbers.values()
    ):
        return None
    try:
        line = FoodEnergyLine.model_validate(numbers)
    except ValidationError:
        return None
    return line if line.available_kcal <= line.capacity_kcal else None
