"""`Player`: play one or more seats of a MineWorld world from a program, exactly as a human client does.

One seat per connection through `mineworld-sdk`'s `ResumingSeat` (INV-S27-1); the server's own observation
rendered compactly with short references (`view.py`); the world's own offers, sent only as the newest
observation offers them (`offers.attempt` / `offers.request`, INV-S27-3); the walk paced as the 2D client
paces it (`mineworld_sdk.pacer`); the world never waiting (`handle.py`'s drain). The Player adds no world rule,
evaluates no requirement, carries world-authored text only as `WorldText` (INV-S27-12) and never shows a
credential (INV-S27-11): the invite is a constructor argument, read only by `join`.

Every operation takes its `*Args` model and returns its `*Result` model (QS27c-3), the same models S27-d
publishes as MCP tool schemas. Acts answer a typed `Outcome`; the other operations raise `PlayerRefusal`
(`unknown_handle`, `unknown_ref`, `seat_lost`, …) or `JoinRefused`, each naming a code only (D-c-1).

The thirteen operations and `close` are the whole tool surface: none pauses, resumes, changes speed, kicks,
releases or grants (INV-S27-10). `inbox` is a read-only status beside them, served by the MCP adapter as a
resource rather than as a tool (S27-d).
"""

from __future__ import annotations

import asyncio
import contextlib
import secrets
from collections.abc import Awaitable, Callable
from functools import partial
from pathlib import Path
from typing import Annotated, Final, cast, get_args

from mineworld_sdk import offers
from mineworld_sdk.offers import NotOffered
from mineworld_sdk.pacer import Pacer, Step, Wait, Walking
from mineworld_sdk.perceived import CursorCell
from mineworld_sdk.reconnect import AnswerLost, NotConnected
from mineworld_sdk.reconnect import SeatLost as SdkSeatLost
from mineworld_sdk.resuming import Connector, ResumingSeat, Seen
from mineworld_sdk.session import TRANSPORT_FAILURES, Answered, RefusedRequest, open_socket
from mineworld_sdk.wire.contract import Accepted as WireAccepted
from mineworld_sdk.wire.contract import (
    ActionRequest,
    LocalPosition,
    Location,
    Orientation,
    PerceivedEntity,
    PlaceId,
)
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, EntityKey, JsonValue
from pydantic import Field

from mineworld_agent.handle import Handle
from mineworld_agent.inbox import INBOX_BOUND, Inbox, offered
from mineworld_agent.models import (
    ActResult,
    AgentModel,
    AnyEvent,
    AttemptArgs,
    ClockArgs,
    ClockResult,
    EventLine,
    EventsArgs,
    EventsResult,
    InboxCounts,
    InboxStatus,
    InspectArgs,
    InspectResult,
    JoinArgs,
    JoinRefusalCode,
    JoinRefused,
    JoinResult,
    LeaveArgs,
    LeaveResult,
    MoveArgs,
    ObserveArgs,
    ObserveResult,
    OfferAvailable,
    Outcome,
    PlayerRefusal,
    Point,
    Refused,
    SayArgs,
    Seated,
    SeatLost,
    SeatLostRefusal,
    SeatsArgs,
    SeatsResult,
    SpokenTo,
    StopArgs,
    ToPerson,
    ToPlace,
    ToPoint,
    View,
    ViewChanged,
    WaitArgs,
    WaitFor,
    WaitResult,
    WakeReason,
    WalkEnded,
    WalkToArgs,
    WorldLine,
    json_bytes,
)
from mineworld_agent.outcomes import outcome_of, unanswered
from mineworld_agent.refs import RefTable, UnknownRef
from mineworld_agent.state import StateDir
from mineworld_agent.status import fetch_status
from mineworld_agent.view import (
    WALKING,
    Caps,
    Incomplete,
    Moment,
    Rendered,
    StaleOffer,
    calendar_of,
    components_of,
    delta_of,
    entity_labels,
    entity_name,
    page_of,
    relocate,
    view_of,
)

__all__ = ["Player", "PlayerConfig"]

RESULT_MAX_BYTES: Final = 8192
"""Every result's ceiling, in bytes of compact JSON (step-27 §5.10)."""

WRAPPER_BYTES: Final = 512
"""Room a view leaves for the result around it (a join's handle and seat, an act's outcome)."""

JOIN_PATIENCE: Final = 15.0
"""Wall seconds a join waits for the seat's first observation."""

CAUSED_PATIENCE: Final = 1.0
"""Wall seconds an accepted act waits for the facts it caused to arrive (pr-s27c §5.6)."""

FACT_TO_VIEW_PATIENCE: Final = 1.0
"""Wall seconds a `wait` woken by a fact gives the observation that reflects it to arrive."""

WAIT_EVENTS: Final = 10
"""The most facts one `wait` returns (step-27 §5.4)."""

TALK, WALK_TO, WALK_STEP, MOVE = (
    ActionTypeId("talk"),
    ActionTypeId("walk-to"),
    ActionTypeId("walk-step"),
    ActionTypeId("move"),
)
"""The actions the adapter knows by name (step-27 §4.1; pr-s27c C-8). Everything else is `attempt`."""

_JOIN_CODES: Final = frozenset(get_args(JoinRefusalCode))


class PlayerConfig(AgentModel):
    """How a Player plays: where, and within which bounds. The bounds are the step's defaults and can
    only be lowered (step-27 §5.10)."""

    server: Annotated[str, Field(pattern=r"^wss?://[^/\s]+/ws$")]
    """`ws://host:port/ws`; `/status` is the same host over http."""
    idle_release_s: Annotated[int, Field(ge=0)] = 600
    """A handle with no call for this many wall seconds gives its seat back; 0 = never (step-27 §5.9)."""
    inbox: Annotated[int, Field(ge=1, le=INBOX_BOUND)] = INBOX_BOUND
    max_result_bytes: Annotated[int, Field(ge=2048, le=RESULT_MAX_BYTES)] = RESULT_MAX_BYTES
    state_dir: Path | None = None
    """Where a handle's resume secret and cursor are kept between processes (`state.py`); none if
    `None`."""


type ConnectorFactory = Callable[[], Connector]
type Sleep = Callable[[float], Awaitable[None]]


class Player:
    """Plays seats of one server. Use `async with Player(...) as player:` or call `close()`."""

    def __init__(
        self,
        config: PlayerConfig,
        credential: Invite,
        *,
        connector: ConnectorFactory | None = None,
        wall: Callable[[], float] | None = None,
        sleep: Sleep | None = None,
        on_inbox: Callable[[str], None] | None = None,
    ) -> None:
        self._config = config
        self._credential = credential
        self._connector = connector
        self._wall = wall
        self._sleep = sleep or asyncio.sleep
        self._on_inbox = on_inbox
        """Called with a handle string when that handle's inbox gains facts (S27-d's `MCP` subscription);
        synchronous, and it must not raise."""
        self._handles: dict[str, Handle] = {}
        self._ending: set[asyncio.Task[None]] = set()
        """Lost handles being cleaned up: held so the loop does not drop the tasks."""
        self._states = None if config.state_dir is None else StateDir(config.state_dir)
        self._caps = Caps(max_bytes=config.max_result_bytes - WRAPPER_BYTES)

    async def __aenter__(self) -> Player:
        return self

    async def __aexit__(self, *_: object) -> None:
        await self.close()

    def __repr__(self) -> str:
        return f"Player(server={self._config.server!r}, handles={len(self._handles)})"

    def _now(self) -> float:
        return self._wall() if self._wall is not None else asyncio.get_running_loop().time()

    # ── Seats and joining ─────────────────────────────────────────────────────────────────────────

    async def seats(self, args: SeatsArgs) -> SeatsResult:
        """The world and the seats in it that a player can join, from the server's public status. Needs no
        credential."""
        del args
        summary = await fetch_status(self._config.server)
        return SeatsResult(
            world=WorldLine(
                protocol=summary.protocol,
                at=summary.at,
                time_scale=summary.time_scale,
                paused=summary.paused,
                entities=summary.entities,
                systems=[system.system for system in summary.systems],
                clients=summary.clients,
            ),
            seats=list(summary.seats),
        )

    async def join(self, args: JoinArgs) -> JoinResult:
        """Join the seat named `seat` under `nickname`. The credential is configured outside this call and is
        never an argument. `take_over` asks to take the seat from its current player."""
        key = EntityKey(args.seat)
        saved = None
        if self._states is not None and self._states.path(key).exists():
            summary = await fetch_status(self._config.server)
            saved = self._states.read(key, summary.instance)
        cursor = CursorCell(None if saved is None else saved.since)
        server = self._config.server
        connect: Connector = (
            self._connector() if self._connector is not None else (lambda: open_socket(server))
        )

        async def opened(streamed: bool) -> ResumingSeat:
            return await ResumingSeat.open(
                connect,
                seat=key,
                invite=self._credential,
                nickname=args.nickname,
                cursor=cursor if streamed else None,
                take_over=args.take_over,
                resume=None if saved is None else saved.secret,
            )

        streamed = True
        try:
            try:
                seat = await opened(streamed)
            except SdkSeatLost as lost:
                if lost.reason != "cursor_unavailable" or saved is not None:
                    raise
                # A world without a save keeps no history to stream from its beginning (§5.8):
                # play it as a client without the stream does (D-c-19). The refusal granted nothing.
                streamed = False
                seat = await opened(streamed)
        except SdkSeatLost as lost:
            raise JoinRefused(_join_code(lost.reason)) from None
        except TRANSPORT_FAILURES:
            raise PlayerRefusal("not_connected", "the server could not be reached") from None
        name = secrets.token_hex(16)
        handle = self._handle(seat, key, cursor, name, streamed=streamed, fresh=saved is None)
        try:
            await handle.first_observation(JOIN_PATIENCE)
        except TimeoutError:
            await handle.stop(give_back=True)
            raise PlayerRefusal("not_connected", "no observation arrived") from None
        if handle.lost is not None:
            await handle.stop(give_back=False)
            raise JoinRefused(_join_code(handle.lost))
        self._handles[name] = handle
        welcome = seat.welcome
        return JoinResult(
            handle=name,
            seat=args.seat,
            me=handle.refs.ref(seat.observer),
            took_over=welcome.took_over,
            hold_seconds=welcome.hold_seconds,
            view=self._show(handle, view_of, handle.rendered()),
        )

    def _handle(
        self,
        seat: ResumingSeat,
        key: EntityKey,
        cursor: CursorCell,
        name: str,
        *,
        streamed: bool,
        fresh: bool,
    ) -> Handle:
        refs = RefTable()
        hook = self._on_inbox
        handle = Handle(
            seat=seat,
            key=key,
            refs=refs,
            inbox=Inbox(refs, seat.observer, self._config.inbox),
            cursor=cursor,
            wall=self._now,
            sleep=self._sleep,
            idle_release_s=self._config.idle_release_s,
            states=self._states,
            streamed=streamed,
            fresh=fresh,
            on_inbox=None if hook is None else partial(hook, name),
        )
        handle.pacer = Pacer(
            walking=lambda: _walking(handle),
            paused=lambda: _paused(handle),
            step=lambda: self._step(handle),
            wall=self._now,
            sleep=self._sleep,
        )
        handle.start()
        return handle

    async def leave(self, args: LeaveArgs) -> LeaveResult:
        """Give this seat back to the world now. The seat returns to its default at once, and the handle ends."""
        handle = self._seated(args.handle)
        del self._handles[args.handle]
        await handle.stop(give_back=True)
        return LeaveResult()

    async def close(self) -> None:
        """Leaves every handle."""
        handles, self._handles = list(self._handles.values()), {}
        for handle in handles:
            await handle.stop(give_back=handle.lost is None)
        await asyncio.gather(*self._ending, return_exceptions=True)

    # ── Observe ───────────────────────────────────────────────────────────────────────────────────

    async def observe(self, args: ObserveArgs) -> ObserveResult:
        """The newest observation, as a view, a page of one, or a delta from a view still held."""
        handle = self._seated(args.handle)
        newest = handle.rendered()
        page = args.page
        if page is not None and (paged := handle.held(page.view)) is not None:
            view = self._show(handle, lambda r, m, i, c: page_of(r, page, m, i, c), paged)
            return ObserveResult(view=view)
        if args.since_view is not None and (base := handle.held(args.since_view)) is not None:
            moment = self._moment(handle)
            delta = delta_of(base, newest, moment, _counts(handle), self._caps)
            if delta is not None:
                handle.shown = handle.newest
                return ObserveResult(view=delta)
        return ObserveResult(view=self._show(handle, view_of, newest))

    async def inbox(self, args: Seated) -> InboxStatus:
        """What the handle's inbox holds, read without changing it: unread facts, facts dropped at the bound, and
        the newest fact's reference (S27-d's `inbox` resource and `wait` cue)."""
        held = self._seated(args.handle).inbox
        return InboxStatus(unread=held.unread, dropped=held.dropped, newest=held.newest)

    async def inspect(self, args: InspectArgs) -> InspectResult:
        """What this seat perceives about one entity, `ref`, component by component."""
        handle = self._seated(args.handle)
        entity = _listed(handle, args.ref)
        moment = self._moment(handle)
        return InspectResult(
            at=moment.at,
            time_scale=moment.time_scale,
            paused=moment.paused,
            world_seconds_since_last_call=moment.since,
            ref=args.ref,
            entity_type=entity.entity_type,
            name=entity_name(entity),
            labels=entity_labels(entity),
            components=components_of(entity),
        )

    # ── Events and waiting ────────────────────────────────────────────────────────────────────────

    async def events(self, args: EventsArgs) -> EventsResult:
        """Facts waiting in the inbox, oldest first, one page."""
        handle = self._seated(args.handle)
        moment = self._moment(handle)

        def result(lines: list[EventLine], following: str | None) -> EventsResult:
            return EventsResult(
                at=moment.at,
                time_scale=moment.time_scale,
                paused=moment.paused,
                world_seconds_since_last_call=moment.since,
                events=lines,
                next=following,
                unread=handle.inbox.unread,
                dropped=handle.inbox.dropped,
            )

        lines, following = handle.inbox.events(
            args.after,
            args.limit,
            args.detail,
            fits=lambda ls, n: json_bytes(result(ls, n)) <= self._config.max_result_bytes,
        )
        return result(lines, following)

    async def wait(self, args: WaitArgs) -> WaitResult:
        """Wait until the condition `until` holds, or for `timeout_s` seconds. A fact that arrived since the
        previous call counts at once, so nothing that happened in between is missed. The server keeps no
        wait for this call; the next call starts fresh."""
        handle = self._seated(args.handle)
        start = handle.inbox.read_through
        base = handle.shown
        loop = asyncio.get_running_loop()
        deadline = loop.time() + args.timeout_s
        woke: WakeReason = "timeout"
        while True:
            handle.inbox.changed.clear()
            signal = handle.signal()
            if _woken(handle, args.until, start, base):
                woke = args.until.kind
                break
            if handle.lost is not None:
                break
            remaining = deadline - loop.time()
            if remaining <= 0:
                break
            facts = asyncio.ensure_future(handle.inbox.changed.wait())
            seen = asyncio.ensure_future(signal.wait())
            try:
                await asyncio.wait((facts, seen), timeout=remaining, return_when="FIRST_COMPLETED")
            finally:
                for waiter in (facts, seen):
                    waiter.cancel()
                await asyncio.gather(facts, seen, return_exceptions=True)
        if woke in ("any_event", "spoken_to", "walk_ended"):
            # A fact is sent before the observation that reflects it (`PROTOCOL.md` §5.8): give that
            # observation a moment to arrive, so the result's view_changed and the next observe agree
            # with the fact (D-c-20).
            await _next_observation(handle, FACT_TO_VIEW_PATIENCE)
        handle.touch()
        if handle.lost is not None:
            self._seated(args.handle)  # raises SeatLostRefusal: the seat ended while waiting
        moment = self._moment(handle)
        changed = handle.newest is not None and handle.newest.after(base)

        def result(lines: list[EventLine]) -> WaitResult:
            return WaitResult(
                at=moment.at,
                time_scale=moment.time_scale,
                paused=moment.paused,
                world_seconds_since_last_call=moment.since,
                woke=woke,
                events=lines,
                view_changed=changed,
            )

        lines = handle.inbox.since(
            start,
            WAIT_EVENTS,
            fits=lambda ls, _: json_bytes(result(ls)) <= self._config.max_result_bytes,
        )
        return result(lines)

    # ── Acts ──────────────────────────────────────────────────────────────────────────────────────

    async def attempt(self, args: AttemptArgs) -> ActResult:
        """Take offer `offer` of the view numbered `view`, as the world offered it. The offer is matched against
        the newest observation; when the world no longer offers it, the answer says so as an outcome."""
        handle = self._seated(args.handle)
        newest = handle.rendered()
        shown = handle.held(args.view)
        try:
            if shown is None:
                raise StaleOffer(f"view {args.view} is no longer held")
            affordance = relocate(shown, args.offer, newest)
        except Incomplete:
            return self._act_result(handle, Refused(code="not_offered"))
        except StaleOffer:
            return self._act_result(handle, Refused(code="stale_offer"), fresh=True)
        try:
            request = offers.attempt(newest.observation, affordance)
        except NotOffered:
            return self._act_result(handle, Refused(code="not_offered"))
        return self._act_result(handle, await self._submit(handle, request))

    async def say(self, args: SayArgs) -> ActResult:
        """Say `words` to the person `to`, when the newest view offers talking to that person. The answer says
        whether the world took it; a reply, if any, arrives as an event (see `wait`)."""
        handle = self._seated(args.handle)
        try:
            to = handle.refs.resolve(args.to)
        except UnknownRef:
            return self._act_result(handle, Refused(code="unknown_ref"))
        return await self._request(handle, TALK, to, {"utterance": args.words})

    async def walk_to(self, args: WalkToArgs) -> ActResult:
        """Walk to the place, person or point `to`. Once the world accepts the walk, it goes on one step a second
        until the seat arrives or the walk is stopped."""
        handle = self._seated(args.handle)
        await _stop_pacing(handle)
        try:
            destination = _destination(handle, args.to)
        except UnknownRef:
            return self._act_result(handle, Refused(code="unknown_ref"))
        result = await self._request(handle, WALK_TO, None, {"to": destination})
        if result.outcome.kind == "accepted" and handle.pacer is not None:
            await handle.pacer.start()
        return result

    async def move(self, args: MoveArgs) -> ActResult:
        """Take one stride toward the point `to`, with an optional facing in degrees. It ends any walk."""
        handle = self._seated(args.handle)
        await _stop_pacing(handle)
        try:
            to = _location(handle, args.to, args.facing_deg)
        except UnknownRef:
            return self._act_result(handle, Refused(code="unknown_ref"))
        return await self._request(handle, MOVE, None, {"to": to})

    async def stop(self, args: StopArgs) -> ActResult:
        """Stop where this seat is, keeping its facing. It ends any walk."""
        handle = self._seated(args.handle)
        await _stop_pacing(handle)
        here = handle.rendered().observation.self_location
        if here is None:
            return self._act_result(handle, Refused(code="not_offered"))
        return await self._request(handle, MOVE, None, {"to": here.model_dump(mode="json")})

    async def _request(
        self, handle: Handle, action: ActionTypeId, target: EntityId | None, payload: JsonValue
    ) -> ActResult:
        try:
            request = offers.request(
                handle.rendered().observation, action, target=target, payload=payload
            )
        except NotOffered:
            return self._act_result(handle, Refused(code="not_offered"))
        return self._act_result(handle, await self._submit(handle, request))

    async def _submit(self, handle: Handle, request: ActionRequest) -> Outcome:
        """One request on the handle's own seat, one in flight (the submit lock)."""
        try:
            connection, answer = await handle.send(request)
        except (AnswerLost, NotConnected, SdkSeatLost) as error:
            if isinstance(error, SdkSeatLost):
                handle.lost = handle.lost or error.reason
            return unanswered(error)
        caused: list[EventLine] = []
        if isinstance(answer, Answered):
            handle.last_action = (connection, answer.action_id)
            if isinstance(answer.result, WireAccepted):
                await handle.delivered_through(answer.result.accepted.events, CAUSED_PATIENCE)
                caused = handle.inbox.caused_by(answer.action_id)
            await handle.reflecting(CAUSED_PATIENCE)
        return outcome_of(answer, caused)

    async def _step(self, handle: Handle) -> Step:
        """The pacer's `walk-step`, under the same lock as every other request of the handle."""
        if handle.lost is not None:
            return "stop"
        try:
            request = offers.request(
                handle.rendered().observation, WALK_STEP, target=None, payload={}
            )
        except NotOffered:
            return "stop"
        try:
            connection, answer = await handle.send(request)
        except (AnswerLost, NotConnected):
            return "stepped"
        except SdkSeatLost as lost:
            handle.lost = handle.lost or lost.reason
            return "stop"
        if isinstance(answer, RefusedRequest):
            # Paused: the pacer sees the clock and sends nothing until the resume. Rate limited
            # (S27-a, R-S27c-a-1): the handle's next submit already waits `retry_after_ms`; the walk
            # goes on after it. Anything else ends the walk.
            if answer.code == "paused":
                return "stepped"
            if answer.code == "rate_limited":
                return Wait(handle.backoff())
            return "stop"
        handle.last_action = (connection, answer.action_id)
        return "stepped" if isinstance(answer.result, WireAccepted) else "stop"

    # ── Time ──────────────────────────────────────────────────────────────────────────────────────

    async def clock(self, args: ClockArgs) -> ClockResult:
        """The world's time as this seat sees it, and the calendar when the calendar pack discloses it."""
        handle = self._seated(args.handle)
        moment = self._moment(handle)
        return ClockResult(
            at=moment.at,
            time_scale=moment.time_scale,
            paused=moment.paused,
            world_seconds_since_last_call=moment.since,
            calendar=calendar_of(handle.rendered().observation),
        )

    # ── Shared ────────────────────────────────────────────────────────────────────────────────────

    def _seated(self, name: str) -> Handle:
        """The handle, touched; or the refusal that says why there is none. A lost seat is reported
        once, as `seat_lost { reason }`, and the handle is gone after it."""
        handle = self._handles.get(name)
        if handle is None:
            raise PlayerRefusal("unknown_handle")
        if handle.lost is not None:
            del self._handles[name]
            ending = asyncio.create_task(handle.stop(give_back=False))
            self._ending.add(ending)
            ending.add_done_callback(self._ending.discard)
            raise SeatLostRefusal(handle.lost)
        handle.touch()
        return handle

    def _moment(self, handle: Handle) -> Moment:
        at = handle.rendered().observation.at
        clock = handle.seat.clock
        world = handle.seat.welcome.world
        scale, paused = (
            (clock.time_scale, clock.paused) if clock else (world.time_scale, world.paused)
        )
        since = None if handle.last_at is None else at - handle.last_at
        handle.last_at = at
        return Moment(at=at, time_scale=scale, paused=paused, since=since)

    def _show(
        self,
        handle: Handle,
        shape: Callable[[Rendered, Moment, InboxCounts, Caps], View],
        rendered: Rendered,
    ) -> View:
        view = shape(rendered, self._moment(handle), _counts(handle), self._caps)
        handle.shown = handle.newest
        return view

    def _act_result(self, handle: Handle, outcome: Outcome, *, fresh: bool = False) -> ActResult:
        handle.touch()
        if isinstance(outcome, SeatLost):
            handle.lost = handle.lost or outcome.reason
        view = self._show(handle, view_of, handle.rendered()) if fresh else None
        moment = self._moment(handle)
        return ActResult(
            at=moment.at,
            time_scale=moment.time_scale,
            paused=moment.paused,
            world_seconds_since_last_call=moment.since,
            outcome=outcome,
            view=view,
        )


# ── Helpers ───────────────────────────────────────────────────────────────────────────────────────


def _counts(handle: Handle) -> InboxCounts:
    return InboxCounts(unread=handle.inbox.unread, dropped=handle.inbox.dropped)


def _join_code(reason: str) -> JoinRefusalCode:
    return cast(JoinRefusalCode, reason) if reason in _JOIN_CODES else "protocol_violation"


def _listed(handle: Handle, ref: str) -> PerceivedEntity:
    """The entity behind `ref` in the newest observation; `unknown_ref` when the reference is not this
    handle's or the entity is not perceived now — the two cannot be told apart (`CORE_CONCEPTS.md`
    §15.1)."""
    try:
        wanted = handle.refs.resolve(ref)
    except UnknownRef:
        raise PlayerRefusal("unknown_ref") from None
    observation = handle.rendered().observation
    entity = next((e for e in observation.entities if e.id == wanted), None)
    if entity is None:
        raise PlayerRefusal("unknown_ref")
    return entity


def _walking(handle: Handle) -> Walking:
    """My own walk in the newest observation, for the pacer: only once that observation reflects the
    last request (`acted_through`), so a step is never decided on an older one."""
    if handle.newest is None or not handle.reflected():
        return "unseen"
    observation = handle.newest.frame.observation
    me = next((e for e in observation.entities if e.id == observation.observer), None)
    walking = me is not None and any(c.component_type == WALKING for c in me.components)
    return "walking" if walking else "gone"


def _paused(handle: Handle) -> bool:
    clock = handle.seat.clock
    return clock.paused if clock is not None else handle.seat.welcome.world.paused


def _woken(handle: Handle, until: WaitFor, start: int, base: Seen | None) -> bool:
    match until:
        case AnyEvent() | SpokenTo() | WalkEnded():
            return handle.inbox.matches(until, start)
        case ViewChanged():
            return handle.newest is not None and handle.newest.after(base)
        case OfferAvailable():
            newest = handle.newest
            return newest is not None and offered(until, newest.frame.observation, handle.refs)


async def _next_observation(handle: Handle, patience: float) -> None:
    """Waits for an observation newer than the one held now, at most `patience` wall seconds."""
    held = handle.newest
    with contextlib.suppress(TimeoutError):
        async with asyncio.timeout(patience):
            while True:
                signal = handle.signal()
                if handle.lost is not None or (
                    handle.newest is not None and handle.newest.after(held)
                ):
                    return
                await signal.wait()


async def _stop_pacing(handle: Handle) -> None:
    if handle.pacer is not None:
        await handle.pacer.stop()


def _typed(entity: EntityId, kind: str) -> JsonValue:
    """A typed reference as the packs write `PersonId`/`PlaceId`: `{ entity, entity_type }`."""
    return {"entity": entity, "entity_type": kind}


def _destination(handle: Handle, to: ToPlace | ToPerson | ToPoint) -> JsonValue:
    """movement's `Destination` (`{ place: Location }` or `{ person: PersonId }`, `ARC-75`): a place
    without a position means "enter that place"."""
    match to:
        case ToPlace():
            place = Location(
                place=PlaceId(entity=handle.refs.resolve(to.place), entity_type="place")
            )
            return {"place": place.model_dump(mode="json")}
        case ToPerson():
            return {"person": _typed(handle.refs.resolve(to.person), "person")}
        case ToPoint():
            return {"place": _location(handle, to.point, None)}


def _location(handle: Handle, point: Point, facing_deg: int | None) -> JsonValue:
    """A contract `Location` in the point's place: millimetres, and a yaw in millidegrees."""
    place = PlaceId(entity=handle.refs.resolve(point.place), entity_type="place")
    local = LocalPosition(x=point.x_mm, y=point.y_mm, z=point.z_mm or 0)
    facing = None if facing_deg is None else Orientation(yaw=facing_deg * 1000, pitch=None)
    return Location(place=place, local=local, facing=facing).model_dump(mode="json")
