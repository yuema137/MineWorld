"""The contract shapes inside frames, as `mineworld-contracts` serializes them to JSON.

Each model mirrors one Rust type field for field; the comment on each names the type. The Rust types
are the authority (`docs/ARCHITECTURE.md` §13.1); `server/tests/frames/` and the real server's frames
check this mirror (AP-1, AP-2).

All models are strict, frozen and refuse unknown fields (D-P3-3): an integer field refuses a float, an
id refuses a number, an enumeration refuses a value the Rust enum lacks. The two cross-field rules the
contract enforces on decode are enforced here too: an affordance's `available` agrees with its
`unavailable_reason`, and a request's `action_type` agrees with its payload's.
"""

from __future__ import annotations

from typing import Annotated, Literal, Self

from pydantic import BaseModel, ConfigDict, Field, model_validator

from mineworld_sdk.wire.ids import (
    ActionIdField,
    ActionTypeIdField,
    ComponentTypeIdField,
    EntityIdField,
    EventIdField,
    EventTypeIdField,
    JsonValue,
    ProcessIdField,
    RejectionCodeField,
    RelationTypeIdField,
    SystemIdField,
    TagField,
)

_I32 = Field(ge=-(2**31), le=2**31 - 1)
I32 = Annotated[int, _I32]
U32 = Annotated[int, Field(ge=0, le=2**32 - 1)]
U64 = Annotated[int, Field(ge=0, le=2**64 - 1)]
I64 = Annotated[int, Field(ge=-(2**63), le=2**63 - 1)]

PITCH_LIMIT = 90_000
"""The steepest legal pitch, in millidegrees (`Orientation::PITCH_LIMIT`)."""

EntityType = Literal["person", "place", "item", "organization"]
"""`EntityType`, snake_case."""


class WireModel(BaseModel):
    """Strict, frozen, closed: the configuration every wire model shares."""

    model_config = ConfigDict(
        strict=True,
        frozen=True,
        extra="forbid",
        serialize_by_alias=True,
        validate_by_alias=True,
        validate_by_name=True,
    )


class PlaceId(WireModel):
    """`PlaceId`: a typed reference whose `entity_type` is always `place`."""

    entity: EntityIdField
    entity_type: Literal["place"]


class LocalPosition(WireModel):
    """`LocalPosition`, in millimetres."""

    x: I32
    y: I32
    z: I32


class Orientation(WireModel):
    """`Orientation`, in millidegrees. A pitch beyond ±90° is refused, as the contract refuses it."""

    yaw: I32
    pitch: Annotated[int, Field(ge=-PITCH_LIMIT, le=PITCH_LIMIT)] | None


class Location(WireModel):
    """`Location`."""

    place: PlaceId
    local: LocalPosition | None = None
    facing: Orientation | None = None


class SpecificPlace(WireModel):
    """`PlaceRequirement::Specific`, externally tagged."""

    specific: PlaceId


PlaceRequirement = Literal["any", "same_place_as_actor"] | SpecificPlace
"""`PlaceRequirement`."""


class SpatialRequirement(WireModel):
    """`SpatialRequirement`. A negative range is refused, as the contract refuses it."""

    place: PlaceRequirement
    within_range: Annotated[int, Field(ge=0, le=2**31 - 1)] | None
    requires_line_of_access: bool
    requires_target_available: bool


class SystemRejectionBody(WireModel):
    """The fields of `Rejection::System`."""

    code: RejectionCodeField
    detail: str | None


class SystemRejection(WireModel):
    """`Rejection::System`, externally tagged: a reason of a system's own."""

    system: SystemRejectionBody


Rejection = (
    Literal[
        "busy",
        "too_far_away",
        "permission_denied",
        "no_supported_interaction",
        "target_unavailable",
        "unavailable",
        "precondition_failed",
    ]
    | SystemRejection
)
"""`Rejection`: the world considered a well-formed request and said no."""


class AcceptedBody(WireModel):
    """The fields of `ActionResult::Accepted`."""

    events: list[EventIdField]


class Accepted(WireModel):
    """`ActionResult::Accepted`: the events the request caused."""

    accepted: AcceptedBody


class Rejected(WireModel):
    """`ActionResult::Rejected`."""

    rejected: Rejection


ActionResult = Accepted | Rejected | Literal["unavailable"]
"""`ActionResult`. `"unavailable"` means no enabled system provides the action type (`INV-10`)."""


class ActionRecord(WireModel):
    """`ActionRecord`: an action's payload, labelled with its type."""

    action_type: ActionTypeIdField
    payload: JsonValue


class ActionRequest(WireModel):
    """`ActionRequest`: what a client asks of the world. No id and no instant (`INV-6`)."""

    actor: EntityIdField
    action_type: ActionTypeIdField
    target: EntityIdField | None = None
    payload: ActionRecord
    actor_location: Location | None = None

    @model_validator(mode="after")
    def _types_agree(self) -> Self:
        # ActionRequestFields: the envelope and the record must name one action type (FINDINGS F8.1).
        if self.action_type != self.payload.action_type:
            raise ValueError(
                f"the request's action_type {self.action_type!r} disagrees with its payload's "
                f"{self.payload.action_type!r}"
            )
        return self


class ComponentRecord(WireModel):
    """`ComponentRecord<serde_json::Value>`: one component as perceived, its payload uninterpreted."""

    entity: EntityIdField
    component_type: ComponentTypeIdField
    schema_version: U32
    payload: JsonValue


class PerceivedEntity(WireModel):
    """`PerceivedEntity`. `tags` is the contract's sorted set, as a list."""

    id: EntityIdField
    entity_type: EntityType
    location: Location | None
    tags: list[TagField]
    components: list[ComponentRecord]


class Relation(WireModel):
    """`Relation`. `from` is a Python keyword, so the field is `from_` and travels as `from`."""

    relation_type: RelationTypeIdField
    from_: EntityIdField = Field(alias="from")
    to: EntityIdField


class Affordance(WireModel):
    """`Affordance`: an action the observer may attempt, and the server's verdict on it now.

    `payload` is present only on a **complete affordance** (`ARC-34`): the request payload the offering
    system would accept, unchanged. It is omitted from the encoding when absent, never written `null`.
    """

    action_type: ActionTypeIdField
    target: EntityIdField | None
    available: bool
    unavailable_reason: Rejection | None
    requirement: SpatialRequirement
    payload: JsonValue = Field(default=None, exclude_if=lambda value: value is None)

    @model_validator(mode="after")
    def _availability_agrees(self) -> Self:
        # AffordanceFields: available exactly when there is no reason it is unavailable.
        if self.available != (self.unavailable_reason is None):
            raise ValueError(
                f"an affordance with available={self.available} must "
                f"{'not ' if self.available else ''}carry an unavailable_reason"
            )
        return self


class ActionCause(WireModel):
    """`Causation::Action`: a controller or a client asked."""

    action: ActionIdField


class ProcessCause(WireModel):
    """`Causation::Process`: a process reached a point that produced the fact."""

    process: ProcessIdField


class EventCause(WireModel):
    """`Causation::Event`: another fact, reacted to."""

    event: EventIdField


class SystemTickBody(WireModel):
    """The fields of `Causation::SystemTick`."""

    system: SystemIdField


class SystemTickCause(WireModel):
    """`Causation::SystemTick`: a system acting on its own schedule."""

    system_tick: SystemTickBody


Causation = ActionCause | ProcessCause | EventCause | SystemTickCause | Literal["world_genesis"]
"""`Causation`, externally tagged, snake_case: why a fact happened."""


class PlaceAudience(WireModel):
    """`Visibility::Place`: anyone present in this place."""

    place: PlaceId


class EntitiesAudience(WireModel):
    """`Visibility::Entities`: exactly these entities (the contract's sorted set, as a list)."""

    entities: list[EntityIdField]


Visibility = PlaceAudience | EntitiesAudience | Literal["public", "participants", "system_internal"]
"""`Visibility`: who could have learned of a fact. Whether a person did is the server's judgement."""


class Provenance(WireModel):
    """`Provenance`: which system emitted a fact, and the request that led to it, if any."""

    emitted_by: SystemIdField
    controller_decision: ActionIdField | None


class EventRecord(WireModel):
    """`EventRecord<serde_json::Value>`: a fact's payload, labelled with its type, uninterpreted."""

    event_type: EventTypeIdField
    schema_version: U32
    payload: JsonValue


class PerceivedEvent(WireModel):
    """`PerceivedEvent<serde_json::Value>`: one fact this observer learned (`PROTOCOL.md` §5.2), the
    contract's event envelope with the owning pack's payload as JSON (`null` when it was not JSON).

    An `event_type` this SDK's caller does not know is "something happened", never an error: the
    payload is carried, not interpreted (D-P3-7)."""

    id: EventIdField
    at: I64
    event_type: EventTypeIdField
    subjects: list[EntityIdField]
    participants: list[EntityIdField]
    place: PlaceId | None
    caused_by: Causation
    payload: EventRecord
    visibility: Visibility
    provenance: Provenance

    @model_validator(mode="after")
    def _types_agree(self) -> Self:
        # EventEnvelopeFields: the envelope and the record must name one event type.
        if self.event_type != self.payload.event_type:
            raise ValueError(
                f"the fact's event_type {self.event_type!r} disagrees with its payload's "
                f"{self.payload.event_type!r}"
            )
        return self


class Observation(WireModel):
    """`Observation<serde_json::Value>`: what this connection's observer perceives, and nothing else.

    `events` are the facts this observer learned since its previous frame, oldest first — best effort;
    a caller that must not miss one asks for the `perceived` stream (`PROTOCOL.md` §§5.2, 5.8).
    `entities` are in ascending id order (§5.2)."""

    observer: EntityIdField
    at: I64
    self_location: Location | None
    entities: list[PerceivedEntity]
    relations: list[Relation]
    events: list[PerceivedEvent]
    affordances: list[Affordance]
