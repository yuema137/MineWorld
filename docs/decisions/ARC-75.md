## ARC-75 — A walk is movement's state; its route is the geometry owner's answer; its strides are embodied requests

**Date** 2026-10-09 · **Approved by** the operator ("add pathfinding; 12d waits for it", 2026-10-08;
realistic defaults; the two time domains, QTW-13) and the primary session at PR 12n's design freeze
(step-11 §21.0: revision 1 with R-12n-1, QN-1 … QN-13, binding note N-1) · **Implements**
[`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§6, 8–9, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §17 ·
**Relates to** `ARC-25`, `ARC-26`, `ARC-27`, `ARC-34`, `ARC-39`, `ARC-62`, `ARC-67`, `DEP-34` ·
**Design** `.structured-coding/plans/mvp0/step-11-bodies.md` §21 (S15, PRs 12n-1 and 12n-2)

**Problem.** `move` is one stride toward a point and sees no walls (`ARC-26`). With walls and furniture
in the towns (`bodies`, `ARC-39`), a controller that strides straight at its target stops short at the
first table, and a café door behind an A-board becomes unreachable (step-11 §19, TD-D7). Somebody has to
go round things. Three rules bound where that lives: no client or controller may compute a world rule
(`ENGINEERING_RULES.md` §8; a client only reports intent, and 2D, 3D and a Python controller must share
the capability without duplicating it, §9); controllers are stateless (`ARC-27`); and embodied pace is
the wall-clock cadence of requests, never a world duration and never a pack reading the time scale
(`ARC-67`).

**Options considered** (step-11 §21.3).

```text
(a) a movement Process that strides once per      wakes in calendar time, so the time scale multiplies
    simulated second                              the walking speed (≈ 16 m/s at 12×); correcting it
                                                  in the pack would read the scale — both rejected by
                                                  ARC-67 (R-12n-1)
(b) a planning library every controller and       every client and controller embeds a planner (GDScript
    client embeds                                 2D and 3D, Python): duplicated logic; a stateless
                                                  controller re-plans every consult
(c) a server plan plus a client preview planner   two planners that can disagree
(d) the walk owned by bodies, or a third pack     walking would vanish without bodies, or the stride and
                                                  passage rule would be duplicated
(e) the route on the server, owned by movement    chosen
    and planned through a catalog the geometry
    owner answers; each stride an embodied request
```

**Choice: (e).**

1. **Two actions of `movement`.** `walk-to { to: Destination }` asks to go somewhere; `Destination` is
   `place(Location)` — a point in the walker's place or in a place reachable through a chain of
   passages, or a place without a position, meaning "enter it" — or `person(PersonId)` in the walker's
   own place. `Destination` is `#[non_exhaustive]` and serialized with one explicit tag per arm, so a
   later arm (a region, a remote place the travel system reaches, an object) is an addition, not a
   breaking change. `walk-step` (no payload) asks for the next stride of one's own walk. Both are offered
   once, against nobody, without a request, like `move` (`ARC-34`): a client or controller composes them.
2. **The walk is world state.** `walk-to`'s resolution plans the first leg and records it as the
   walker's `Walking` component — destination, the places still to enter, the current leg's waypoints,
   the last stride asked, stall and re-plan counts — and states `walk-started { person, destination }`.
   No stride, no `Process`, no world duration: a walk nobody steps waits, and takes no calendar time.
   `Walking` is persisted, restored and replayed like every component, so a resumed world continues the
   walk where it was.
3. **Each stride is a request.** `walk-step` takes at most `WALK_STRIDE` (1 340 mm) toward the next
   waypoint and never past it, or, at a doorway, the crossing into the next place, after which the next
   leg is planned. The stride is checked by the same rule as `move` and stated through presence's
   `arrivals()`, so the registered resolvers resolve it like any other arrival (`ARC-39`). The walk ends
   with `walk-ended { person, destination, outcome }`, outcome `arrived`, `stalled` (three consecutive
   steps with under 50 mm of progress, or more than eight re-plans), `no-route` (a later leg or a
   re-plan found no way), `replaced` (a new `walk-to`) or `stopped` (the walker's own `move` — direct
   control always wins). Arrival is recognised when presence records the walker at the leg's end (or,
   for a person, within `PERSON_APPROACH`, 1 200 mm, of them), in the same dispatch.
4. **The route is the geometry owner's answer.** `movement` owns a catalog (`ARC-62`):
   `Wayfinder { wayfinder_of, route(world, RouteAsk) -> Option<RouteAnswer> }`, `RouteAsk { person,
   place, from, to, avoid }` built only by movement, `RouteAnswer` either waypoints (non-empty, integer
   positions ending at the possibly adjusted goal) or `unreachable`. Its rules are `ARC-62` item 4's:
   write-once and process-wide, pure, keeps nothing, inert (`None`) where its pack holds no state for
   the place. Asked in ascending `SystemId`; the first answer wins; with none, the leg is the straight
   segment. `bodies` is the build's wayfinder (`DEP-34`, `ARC-39` note 5). Movement never names it.
5. **Refusals.** `walk-to` is refused `malformed-payload` for a payload it cannot read;
   `NoSupportedInteraction` for an actor that is not a person; `PreconditionFailed` for an actor without
   a presence or a destination presence refuses; `TooFarAway` only for a `person` destination outside
   the walker's place; and **`Rejection::System { code: "no-route" }`** when the destination place has no
   passage chain from the walker's or the first leg's wayfinder answers `unreachable`. The code string
   `no-route` is part of movement's public vocabulary and is never renamed. `walk-step` without a walk
   is `PreconditionFailed`.
6. **Re-planning** happens at a step when the last stride did not end where it was asked (stopped
   short, nudged or shoved), with the person who stopped the walker given as `avoid`; or when a `person`
   destination moved more than 500 mm from where the leg was planned to. A walk re-plans at most eight
   times. A shove or a nudge does not end a walk.
7. **Cadence belongs to the sender.** One `walk-step` per wall second (`EMBODIED_STEP`) is 1.34 m/s
   — Weidmann (1993), the mean free walking speed of pedestrians; Bohannon (1997), 1.27–1.46 m/s
   comfortable gait for adults aged 20–59. A player's client sends it on a timer while its own `walking`
   record is disclosed; a hosted controller is stepped by the host in wall seconds (`ARC-67` item 4); a
   headless `run` steps at a notional cadence. No System Pack reads the cadence, a wall clock or the time
   scale: a walk at 6×, 12× or 24× covers the same ground per wall second.
8. **Disclosure.** To whoever perceives a walking person, movement discloses `walking { destination,
   next }` — the next four waypoints at most. Clients draw intent from the server's own plan; nobody
   plans a route outside the server.

**Accepted limitations.**
- **A sender may step faster than a person walks**, exactly as `move` may be sent back to back
  (`ARC-26`, L-1): a bound per request, not per second.
- **Geometry changed after a plan is noticed only through its effect**: movement cannot see another
  pack's state, so an object kicked across a planned leg is pushed or blocks the stride like any
  object a `move` meets, and a blocked stride re-plans. (Step-11 §21.15 records this against SD-N9's
  wording.)
- **People are not planned round**, except the one who just stopped the walker: passing other people is
  the resolver's nudging and head-on bias (`ARC-39` notes).
- **No overhead geometry, stairs or floors**; no travel between places not joined by passages.

### Amendment 1, 2026-10-20 — a walk to a person ends where to stand, not at the person

**Why.** The promo v2 review (B4, B8, B10) showed a visitor who chose "walk to Alice" end behind the café
counter, beside the barista. Point 3's rule — plan the last leg to the person and stop within
`PERSON_APPROACH` of them — makes the way round the counter's end the only way to arrive, because the
counter's near edge is 1.5–1.7 m from someone standing behind it. A customer stops at the counter.

**Rule.** The last leg of a walk to a person ends at a *stand point*, chosen by the routes the wayfinders
answer and nothing else (movement still never knows what stands in a place):

- candidates: the point `PERSON_APPROACH` short of them on the line from the walker (point 3's old aim),
  then the point `PERSON_APPROACH` from them in each of sixteen fixed compass directions;
- each is routed by the wayfinders, which move a point lying in furniture to the nearest free one — so a
  point inside a counter becomes the counter's edge on one side or the other;
- a route is kept when it ends within **`PERSON_REACH`, 2 100 mm** of them — the far limit of the close
  phase of social distance (Hall 1966, 1.2–2.1 m), the distance of impersonal business across a counter;
- the kept route with the least *walk + remaining gap* wins (route length from the walker, plus the
  end's distance to them); ties go to the smaller gap, then the earlier candidate.

With nothing between the two, the first candidate is the only one whose walk + gap equals the straight
distance, so the walk is exactly the direct approach it was. Arrival is within `PERSON_APPROACH` of the
person, or at the planned stand point while they are within `PERSON_REACH`. The leg is planned again
when the person moves `TARGET_MOVED` from where they stood when it was planned; `Walking` records that
point as `aimed_at` (omitted from a place walk's record, and absent in a walk saved before this
amendment, which keeps the old aim and arrival).

**Evidence.** `systems/bodies/tests/scenarios.rs`
`a_walk_to_a_person_behind_a_counter_ends_on_the_walkers_side`: the café's counter and a generated room
with a table across it; on `main` the café visitor ends at (4 811, 7 645), behind the counter.
