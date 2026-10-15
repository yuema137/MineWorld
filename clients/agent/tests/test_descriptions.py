"""The information-rich rule (pr-s27d §5.3(a), QS27d-2): every model the thirteen operations and the adapter's own
results reach has a description in its JSON Schema, and every field has one too, except the fields recorded in
`UNDOCUMENTED` below, which may only shrink (pr-s27d §15 records its count)."""

from __future__ import annotations

from typing import TypeAliasType, get_args

from pydantic import BaseModel

from mineworld_agent.models import (
    ActResult,
    AttemptArgs,
    ClockArgs,
    ClockResult,
    EventsArgs,
    EventsResult,
    InboxStatus,
    InspectArgs,
    InspectResult,
    JoinArgs,
    JoinResult,
    LeaveArgs,
    LeaveResult,
    MoveArgs,
    ObserveArgs,
    ObserveResult,
    SayArgs,
    SeatsArgs,
    SeatsResult,
    StopArgs,
    ToolError,
    WaitArgs,
    WaitResult,
    WalkToArgs,
)

ROOTS: tuple[type[BaseModel], ...] = (
    SeatsArgs,
    JoinArgs,
    ObserveArgs,
    InspectArgs,
    EventsArgs,
    WaitArgs,
    SayArgs,
    WalkToArgs,
    MoveArgs,
    StopArgs,
    AttemptArgs,
    ClockArgs,
    LeaveArgs,
    SeatsResult,
    JoinResult,
    ObserveResult,
    InspectResult,
    EventsResult,
    WaitResult,
    ActResult,
    ClockResult,
    LeaveResult,
    InboxStatus,
    ToolError,
)
"""The thirteen operations' arguments and results, the `inbox` status and the error body."""

UNDOCUMENTED: frozenset[str] = frozenset(
    {
        "Accepted.kind",
        "AcquaintanceLine.activities_shared",
        "AcquaintanceLine.exchanges",
        "AcquaintanceLine.familiarity",
        "AcquaintanceLine.first_met",
        "AcquaintanceLine.last_contact",
        "AcquaintanceLine.person",
        "AcquaintanceLine.regard",
        "ActResult.outcome",
        "ActResult.paused",
        "AgendaLine.label",
        "AgendaLine.place",
        "AgendaLine.since",
        "AgendaLine.until",
        "AnyEvent.kind",
        "CalendarLine.day",
        "CalendarLine.month",
        "CalendarLine.year",
        "ClockResult.calendar",
        "ClockResult.paused",
        "ComponentPayload.component",
        "ComponentPayload.envelope",
        "DoingLine.kind",
        "DoingLine.since",
        "Envelope.payload",
        "Envelope.truncated",
        "EventLine.at",
        "EventLine.event_type",
        "EventLine.place",
        "EventLine.ref",
        "EventLine.subjects",
        "EventsResult.dropped",
        "EventsResult.events",
        "EventsResult.paused",
        "EventsResult.unread",
        "ExitLine.leads_to",
        "Here.exits",
        "Here.people",
        "Here.things",
        "InspectResult.components",
        "InspectResult.entity_type",
        "InspectResult.labels",
        "InspectResult.name",
        "InspectResult.paused",
        "InspectResult.ref",
        "InvitationLine.at",
        "InvitationLine.by",
        "InvitationLine.until",
        "ItemCount.count",
        "ItemCount.item",
        "ItemCount.name",
        "JobLine.employer",
        "JobLine.on_shift",
        "JobLine.present",
        "JobLine.until_s",
        "JobLine.workplace",
        "JoinResult.handle",
        "JoinResult.hold_seconds",
        "JoinResult.me",
        "JoinResult.seat",
        "JoinResult.took_over",
        "JoinResult.view",
        "LeaveResult.closing",
        "MeLine.agenda",
        "MeLine.doing",
        "MeLine.facing_deg",
        "MeLine.holdings",
        "MeLine.job",
        "MeLine.name",
        "MeLine.place",
        "MeLine.position",
        "MeLine.ref",
        "MeLine.walking",
        "ObserveResult.view",
        "OfferAvailable.action_type",
        "OfferAvailable.kind",
        "OfferAvailable.target",
        "OfferLine.action_type",
        "OfferLine.available",
        "OfferLine.n",
        "OfferLine.requirement",
        "OfferLine.target",
        "OfferLine.unavailable",
        "PageRef.list",
        "PageRef.start",
        "PageRef.view",
        "PersonLine.doing",
        "PersonLine.labels",
        "PersonLine.name",
        "PersonLine.position",
        "PersonLine.ref",
        "PersonLine.walking",
        "PlaceLine.ref",
        "Point.place",
        "Point.x_mm",
        "Point.y_mm",
        "Point.z_mm",
        "ReasonLine.reason",
        "ReasonLine.system_code",
        "Refused.code",
        "Refused.kind",
        "Rejected.kind",
        "Rejected.reason",
        "Rejected.system_code",
        "RequirementLine.line_of_access",
        "RequirementLine.place",
        "RequirementLine.specific_place",
        "RequirementLine.target_available",
        "RequirementLine.within_mm",
        "SeatLost.kind",
        "SeatLost.reason",
        "SeatsResult.seats",
        "SeatsResult.world",
        "SpokenTo.kind",
        "ThingLine.entity_type",
        "ThingLine.labels",
        "ThingLine.name",
        "ThingLine.position",
        "ThingLine.ref",
        "ToPerson.person",
        "ToPlace.place",
        "ToPoint.point",
        "Unavailable.kind",
        "View.calendar",
        "View.here",
        "View.inbox",
        "View.kind",
        "View.me",
        "View.more",
        "View.offers",
        "View.paused",
        "View.view",
        "View.weather",
        "ViewChanged.kind",
        "ViewDelta.acted_through",
        "ViewDelta.base_view",
        "ViewDelta.inbox",
        "ViewDelta.kind",
        "ViewDelta.more",
        "ViewDelta.paused",
        "ViewDelta.people_gone",
        "ViewDelta.things",
        "ViewDelta.things_gone",
        "ViewDelta.view",
        "WaitResult.paused",
        "WaitResult.view_changed",
        "WaitResult.woke",
        "WalkEnded.kind",
        "WeatherLine.cloud_oktas",
        "WeatherLine.condition",
        "WeatherLine.precipitation_tenth_mm",
        "WeatherLine.wind_from_deg",
        "WorldLine.at",
        "WorldLine.clients",
        "WorldLine.entities",
        "WorldLine.paused",
        "WorldLine.protocol",
        "WorldLine.systems",
        "WorldLine.time_scale",
        "WorldText.kind",
        "WorldText.text",
    }
)
"""`Model.field` names whose field has no docstring. Shrinking only: a field gains a docstring, never loses one."""


def reachable(annotation: object, unfolded: set[int]) -> set[type[BaseModel]]:
    """Every `BaseModel` class an annotation names, through unions, optionals, lists, `Annotated` and aliases.
    `unfolded` holds the aliases already entered: a recursive alias (the SDK's `JsonValue`) is entered once."""
    if isinstance(annotation, TypeAliasType):
        if id(annotation) in unfolded:
            return set()
        unfolded.add(id(annotation))
        return reachable(annotation.__value__, unfolded)
    if isinstance(annotation, type) and issubclass(annotation, BaseModel):
        return {annotation}
    found: set[type[BaseModel]] = set()
    for argument in get_args(annotation):
        found |= reachable(argument, unfolded)
    return found


def closure(roots: tuple[type[BaseModel], ...]) -> set[type[BaseModel]]:
    seen: set[type[BaseModel]] = set()
    pending = list(roots)
    while pending:
        model = pending.pop()
        if model in seen:
            continue
        seen.add(model)
        # `models.py` defines some models before the models they name (`View` before `WeatherLine`), so their
        # annotations resolve only once the module is complete; rebuilding makes the walk order-independent.
        model.model_rebuild()
        for field in model.model_fields.values():
            pending.extend(reachable(field.annotation, set()))
    return seen


def test_every_reached_model_has_a_schema_description() -> None:
    undescribed = sorted(
        model.__name__
        for model in closure(ROOTS)
        if not model.model_json_schema().get("description")
    )
    assert undescribed == []


def test_every_field_has_a_description_except_the_recorded_ones() -> None:
    undocumented = {
        f"{model.__name__}.{name}"
        for model in closure(ROOTS)
        for name, field in model.model_fields.items()
        if not field.description
    }
    assert undocumented == UNDOCUMENTED


def test_the_recorded_list_names_real_fields() -> None:
    known: set[str] = {
        f"{model.__name__}.{name}" for model in closure(ROOTS) for name in model.model_fields
    }
    assert UNDOCUMENTED <= known
