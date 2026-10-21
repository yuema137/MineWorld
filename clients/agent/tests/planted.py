"""A real post-S31-a observation, with pack records planted into copies of it (pr-s31g §7, the unit layer).

The base is the visitor's first observation in Market Town's café, recorded from the real server after S31-a
(`tests/fixtures/observations/items/market-town.json`, `SOURCE.md` there): it carries `currency`, `wallet`,
`wealth`, the café's `item-catalogue` and priced `buy` payloads. A test plants other records in the shapes C0
pinned from the merged packs (`systems/economy/src/component.rs`, `systems/notices/src/board.rs`). Expected values
are the raw JSON's or literals written in the test, never the adapter's own report (test rules §25).
"""

from __future__ import annotations

import copy
import json
from pathlib import Path
from typing import Any

from mineworld_sdk.wire.contract import Observation

from mineworld_agent.lexicon import Lexicon
from mineworld_agent.models import InboxCounts, View
from mineworld_agent.refs import RefTable
from mineworld_agent.view import Caps, Moment, Rendered, render, view_of

ITEMS = Path(__file__).resolve().parents[3] / "tests" / "fixtures" / "observations" / "items"
MOMENT = Moment(at=100, time_scale=1, paused=False, since=None)
INBOX = InboxCounts(unread=0, dropped=0)

type Raw = dict[str, Any]


def recorded() -> Raw:
    """The visitor's first observation in Market Town's café, as the real server sent it."""
    values: list[Raw] = json.loads((ITEMS / "market-town.json").read_text(encoding="utf-8"))
    return copy.deepcopy(values[0])


def entity(value: Raw, entity_id: str) -> Raw:
    return next(e for e in value["entities"] if e["id"] == entity_id)


def payload(value: Raw, entity_id: str, kind: str) -> Any:  # noqa: ANN401 (a pack's raw JSON)
    components: list[Raw] = entity(value, entity_id)["components"]
    return next((c["payload"] for c in components if c["component_type"] == kind), None)


def plant(value: Raw, entity_id: str, kind: str, body: Any) -> None:  # noqa: ANN401 (raw JSON)
    """Sets (or replaces) component `kind` on `entity_id` with `body`, as the owning pack would disclose it."""
    components: list[Raw] = entity(value, entity_id)["components"]
    components[:] = [c for c in components if c["component_type"] != kind]
    components.append(
        {"entity": entity_id, "component_type": kind, "schema_version": 1, "payload": body}
    )


def remove(value: Raw, entity_id: str, kind: str) -> None:
    """Takes component `kind` off `entity_id`: a pack not installed, or a record not disclosed."""
    components: list[Raw] = entity(value, entity_id)["components"]
    components[:] = [c for c in components if c["component_type"] != kind]


def item(entity_id: str) -> Raw:
    return {"entity": entity_id, "entity_type": "item"}


def person(entity_id: str) -> Raw:
    return {"entity": entity_id, "entity_type": "person"}


def me(value: Raw) -> str:
    return str(value["observer"])


def place(value: Raw) -> str:
    return str(value["self_location"]["place"]["entity"])


def decoded(value: Raw) -> Observation:
    return Observation.model_validate_json(json.dumps(value))


def rendered_of(
    value: Raw, lexicon: Lexicon | None = None, refs: RefTable | None = None, number: int = 1
) -> tuple[Rendered, RefTable]:
    table = RefTable() if refs is None else refs
    return render(decoded(value), table, number, True, lexicon=lexicon), table


def shown(value: Raw, caps: Caps | None = None) -> tuple[View, RefTable]:
    rendered, refs = rendered_of(value)
    return view_of(rendered, MOMENT, INBOX, Caps() if caps is None else caps), refs


def catalogue_names(value: Raw) -> dict[str, str]:
    kinds = payload(value, place(value), "item-catalogue")["kinds"]
    return {k["item"]["entity"]: k["name"] for k in kinds}
