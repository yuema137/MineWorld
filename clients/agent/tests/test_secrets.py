"""INV-S27-11 (I-c-10): no credential and no resume secret in any argument, result, `repr` or message.

The model half: no operation's schema has a place for a credential — it is the `Player`'s launch
configuration, never an argument (step-27 §6.3) — and the refusals' messages carry codes only. The `Player`
half (canaries through every operation, the state file) is below the models' tests.
"""

from __future__ import annotations

import json
import re

from mineworld_sdk.wire.ids import JsonValue
from test_world_text import ALL_MODELS

from mineworld_agent.models import JoinRefused, PlayerRefusal, SeatLostRefusal

_CREDENTIAL = re.compile(r"invite|grant|resume|secret|token|credential|password|key", re.I)


def property_names(schema: JsonValue) -> list[str]:
    names: list[str] = []
    if isinstance(schema, dict):
        for key, value in schema.items():
            if key == "properties" and isinstance(value, dict):
                names += list(value)
            names += property_names(value)
    elif isinstance(schema, list):
        for value in schema:
            names += property_names(value)
    return names


def test_no_model_has_a_place_for_a_credential() -> None:
    found: set[tuple[str, str]] = set()
    for model in ALL_MODELS:
        schema: JsonValue = json.loads(json.dumps(model.model_json_schema()))
        found |= {(model.__name__, n) for n in property_names(schema) if _CREDENTIAL.search(n)}
    assert not found, f"a credential-shaped field: {sorted(found)}"


def test_refusals_say_their_code_and_nothing_else() -> None:
    assert str(PlayerRefusal("unknown_handle")) == "unknown_handle"
    assert str(SeatLostRefusal("idle_released")) == "seat_lost: idle_released"
    assert str(JoinRefused("seat_occupied")) == "the join was refused: seat_occupied"
