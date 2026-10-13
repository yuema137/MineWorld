"""The frames of protocol revision 2: three a client may send, eight a server sends.

Mirrors `server/src/protocol.rs` (`ClientFrame`, `ServerFrame`, `RefusalCode`) and
`server/src/protocol/{connection,summary,delta}.rs`. Only what `server/PROTOCOL.md` §10 lists as landed
is modelled: S11-A; S11-B (`join.take_over`, `WorldSummary.time_scale`, seat holds and their `resume`
secret, absorbed by P3's C6 under D-P3-5); S11-D (the `clock` frame, `WorldSummary.paused`, the refusal
`paused`, under R-S11-9); S11-C (`join.perceived`, the `perceived` and `delta` frames, `acted_through`,
`observation.events`, the refusals `cursor_unavailable` and `lagged`, the closing reason `lagged`,
modelled in S11-C's own pull request under R-S11-9).

`ClientFrame` is a closed union of `Join`, `Submit` and `Leave`. The encoder accepts nothing else, so
this SDK can say exactly join, submit and leave (`INV-9`).
"""

from __future__ import annotations

from typing import Annotated, Literal

from pydantic import Field, field_serializer

from mineworld_sdk.wire.contract import (
    I64,
    U32,
    U64,
    ActionRequest,
    ActionResult,
    Observation,
    PerceivedEvent,
    WireModel,
)
from mineworld_sdk.wire.delta import ObservationDelta
from mineworld_sdk.wire.ids import (
    ActionIdField,
    ActionTypeIdField,
    CorrelationTokenField,
    EntityIdField,
    EntityKeyField,
    EventIdField,
    EventTypeIdField,
    ResumeSecretField,
    SessionIdField,
    SystemIdField,
    WorldInstanceIdField,
)

PROTOCOL_VERSION = 2
"""The revision this SDK speaks (`PROTOCOL_VERSION` in `server/src/protocol.rs`)."""


class Invite:
    """The server's invite, held so that it cannot leak by accident (D-P3-9).

    Its `repr` and `str` are `Invite(<redacted>)`, so it appears in no log line, exception message or
    model `repr`. Only the encoder of a `join` frame reads it, through `reveal`. The caller builds it,
    from a literal or from an environment variable the caller chooses; the SDK never reads the
    environment itself.
    """

    __slots__ = ("_secret",)

    def __init__(self, secret: str) -> None:
        self._secret = secret

    def reveal(self) -> str:
        """The invite itself. Called by the `join` encoder and nowhere else."""
        return self._secret

    def __repr__(self) -> str:
        return "Invite(<redacted>)"

    __str__ = __repr__

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Invite) and other._secret == self._secret

    def __hash__(self) -> int:
        return hash(self._secret)


# ── What a client may say ─────────────────────────────────────────────────────────────────────────


class PerceivedJoin(WireModel):
    """A `join`'s request for the reliable `perceived` stream (`PROTOCOL.md` §5.8): the `through` of
    the last `perceived` frame processed, or `None` for "from this world's first fact". `since` is
    required inside it, and is written even when `None`."""

    since: EventIdField | None


class Join(WireModel, arbitrary_types_allowed=True):
    """Ask for a seat. A client names a seat, never an observer (`INV-13`).

    `resume` re-takes a seat this player's dropped connection held; `take_over` takes a seat another
    connection holds (`PROTOCOL.md` §4.2). The SDK sends both as given and decides neither: reconnect
    policy is P3b's. `perceived`, when given, asks for the `perceived` stream from its cursor; it is
    omitted from the frame otherwise.
    """

    t: Literal["join"] = "join"
    protocol: Literal[2] = PROTOCOL_VERSION
    invite: Invite
    nickname: str
    seat: EntityKeyField
    resume: ResumeSecretField | None = Field(default=None, repr=False)
    take_over: bool = False
    perceived: PerceivedJoin | None = Field(default=None, exclude_if=lambda value: value is None)

    @field_serializer("invite")
    def _reveal(self, invite: Invite) -> str:
        return invite.reveal()


class Submit(WireModel):
    """Submit a request, with the client's own token for pairing the answer."""

    t: Literal["submit"] = "submit"
    token: CorrelationTokenField
    request: ActionRequest


class Leave(WireModel):
    """Give the seat up and end the connection."""

    t: Literal["leave"] = "leave"


ClientFrame = Annotated[Join | Submit | Leave, Field(discriminator="t")]
"""Everything a client can say. There is no fourth frame (`server/PROTOCOL.md` §2)."""


# ── What a server says ────────────────────────────────────────────────────────────────────────────

TookOver = Literal["none", "hosted", "held", "connection"]
"""`TookOver`: whether control of the Person changed hands when this connection joined."""

ClosingReason = Literal[
    "left",
    "kicked",
    "superseded",
    "unauthorized",
    "protocol_mismatch",
    "world_stopped",
    "server_stopping",
    "taken_over",
    "lagged",
]
"""`ClosingReason`: why the server is about to close the connection."""

RefusalCode = Literal[
    "malformed_frame",
    "unknown_frame",
    "not_joined",
    "already_joined",
    "unknown_seat",
    "seat_not_in_world",
    "actor_not_observer",
    "dispatch_failed",
    "world_stopped",
    "protocol_mismatch",
    "unauthorized",
    "invalid_nickname",
    "seat_occupied",
    "invalid_resume",
    "paused",
    "cursor_unavailable",
    "lagged",
]
"""`RefusalCode`: the ways a frame fails to be a request at all. Not a `Rejection`."""


class SystemSummary(WireModel):
    """`SystemSummary`: one installed system, as `/status` and `welcome` name it."""

    system: SystemIdField
    enabled: bool
    provides: list[ActionTypeIdField]
    states: list[EventTypeIdField]


class WorldSummary(WireModel):
    """`WorldSummary`: what the world is — its composition, never its state."""

    protocol: U32
    instance: WorldInstanceIdField
    at: I64
    time_scale: U32
    paused: bool
    entities: U64
    systems: list[SystemSummary]
    seats: list[EntityKeyField]
    clients: U64
    observations_dropped: U64
    events_dropped: U64
    faults: U64
    revision: U64 | None


class Welcome(WireModel):
    """The seat is this connection's, and `observer` is who it sees the world as."""

    t: Literal["welcome"]
    protocol: U32
    seat: EntityKeyField
    observer: EntityIdField
    nickname: str
    session: SessionIdField
    resume: ResumeSecretField | None = Field(repr=False)
    """The secret that re-takes this seat after a dropped socket; never shown in a `repr`."""
    hold_seconds: U32
    took_over: TookOver
    world: WorldSummary


class Clock(WireModel):
    """How the host is pacing the world's clock (`PROTOCOL.md` §5.9): sent once right after `welcome`,
    before the first observation, and again on every pause and resume. While `paused`, every `submit`
    is refused `paused`."""

    t: Literal["clock"]
    at: I64
    time_scale: U32
    paused: bool


class ObservationFrame(WireModel):
    """One observation of this connection's observer. `seq` orders frames sharing one world instant."""

    t: Literal["observation"]
    seq: U64
    revision: U64 | None
    acted_through: ActionIdField | None
    """The newest request submitted on this connection that this observation already reflects, or
    `None` before the first (`PROTOCOL.md` §5.2)."""
    observation: Observation


class Delta(WireModel):
    """What changed since the frame numbered `base` — always the previous one (`PROTOCOL.md` §5.3).
    A session applies it to the observation it holds; its caller only ever sees whole observations."""

    t: Literal["delta"]
    seq: U64
    base: U64
    revision: U64 | None
    acted_through: ActionIdField | None
    delta: ObservationDelta


class Perceived(WireModel):
    """Facts this connection's observer learned, on the reliable stream its `join` asked for
    (`PROTOCOL.md` §5.8): ascending, never dropped, never repeated. `through` is the cursor."""

    t: Literal["perceived"]
    through: EventIdField
    events: list[PerceivedEvent]


class Result(WireModel):
    """The world's answer to one submitted request, paired by the client's own token."""

    t: Literal["result"]
    token: CorrelationTokenField
    action_id: ActionIdField
    result: ActionResult


class Refused(WireModel):
    """A frame was not accepted, and nothing happened. `token` and `detail` are omitted when absent."""

    t: Literal["refused"]
    token: CorrelationTokenField | None = Field(
        default=None, exclude_if=lambda value: value is None
    )
    code: RefusalCode
    detail: str | None = Field(default=None, exclude_if=lambda value: value is None)


class Closing(WireModel):
    """The server is about to close this connection, and says why. `detail` is omitted when absent."""

    t: Literal["closing"]
    reason: ClosingReason
    detail: str | None = Field(default=None, exclude_if=lambda value: value is None)


ServerFrame = Annotated[
    Welcome | Clock | ObservationFrame | Delta | Perceived | Result | Refused | Closing,
    Field(discriminator="t"),
]
"""Everything a client ever receives on revision 2 (S11-A, S11-B, S11-C, S11-D)."""
