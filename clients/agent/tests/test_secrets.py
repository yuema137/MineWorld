"""INV-S27-11 (I-c-10): no credential and no resume secret in any argument, result, `repr` or message.

The model half: no operation's schema has a place for a credential — it is the `Player`'s launch
configuration, never an argument (step-27 §6.3) — and the refusals' messages carry codes only. The `Player`
half (canaries through every operation, the state file) is below the models' tests.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import EntityKey, JsonValue
from test_world_text import ALL_MODELS

from mineworld_agent.models import JoinRefused, PlayerRefusal, SeatLostRefusal
from mineworld_agent.player import Player, PlayerConfig
from mineworld_agent.state import SeatState, StateDir

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


CANARY_INVITE = "mwagent-canary-invite-3c9f1e7a2b5d"
CANARY_RESUME = "c4a1e2c4a1e2c4a1e2c4a1e2c4a1e2c4"


def test_a_player_and_its_state_never_show_the_credential_or_the_secret(tmp_path: Path) -> None:
    # The Player half (I-c-10): the invite is held, never shown; the state file's model hides the
    # secret in its repr; the file itself is the one place the secret is written (AC-c-9's library half,
    # the play-test checks the rest on the real server).
    config = PlayerConfig(server="ws://127.0.0.1:9/ws", state_dir=tmp_path)
    player = Player(config, Invite(CANARY_INVITE))
    state = SeatState(instance="1" * 32, resume=CANARY_RESUME, cursor="3")
    shown = " ".join([repr(player), str(player), repr(config), repr(state), str(state)])
    assert CANARY_INVITE not in shown and CANARY_RESUME not in shown
    states = StateDir(tmp_path)
    states.write(EntityKey("visitor"), state)
    assert CANARY_RESUME in states.path(EntityKey("visitor")).read_text(encoding="utf-8")
    assert CANARY_RESUME not in repr(states)
