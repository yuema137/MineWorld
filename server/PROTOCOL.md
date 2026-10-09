# The MineWorld client protocol, revision 2

**Audience:** coding agents writing a client, a controller, or another server. This is a
specification, not an introduction; `server/README.md` is the short human orientation.
**Authority:** [`docs/NETWORKING.md`](../docs/NETWORKING.md) decides the model; this document
specifies the frames that implement it. Where the two disagree, `NETWORKING.md` governs and this
document is the defect.
**Implemented by:** `server/src/protocol.rs` and `server/src/protocol/` (the frames),
`server/src/admission.rs` (the invite, the nickname and the resume secret), `server/src/seats.rs` (who
drives each seat, §4.2), `server/src/hosted.rs` (in-server controllers), `server/src/session.rs` (the
sequence), `server/src/app.rs` (the routes). The Rust types are authoritative over the JSON examples here, in the
sense `docs/ARCHITECTURE.md` §13 states. One example of every frame, as the Rust types write it, is
kept in `server/tests/frames/` and checked by `server/tests/frames.rs`; a client in another language
tests its codec against those files.
**Design:** `.structured-coding/plans/mvp0/step-12-server.md` §5 (the revision as specified whole) and
`docs/DECISIONS.md` `ARC-41` (how it is implemented incrementally, and why the wire stays JSON).

---

**Revision 2 arrives in steps.** It is specified whole here and lands over the pull requests of step
S11; a frame or field marked "from S11-x" may be absent before that pull request, and never means
something else (§10, the landing table).

## 1. Transports and routes

```text
GET /health   HTTP    liveness of the process. Answered by the transport, never by the world.
GET /status   HTTP    what the world is (§5.7). Answered by the world itself. Public.
GET /ws       HTTP    upgrade to the live connection.
```

Frames on the WebSocket are **JSON text** (`ARC-41`). A binary frame is refused. Ping and pong are the
transport's own business and carry no protocol meaning.

One server binary serves localhost, a LAN and a cloud container with no semantic difference
(`NETWORKING.md` §§1, 8). The address is configuration. Every connection authenticates the same way on
every one of them, loopback included: there is no unauthenticated mode (§4.1).

On a LAN the connection is plain `ws://`, so the invite crosses the network in clear. TLS is provided by
a gateway or a tunnel in front of the server (`NETWORKING.md` §7), never inside it.

## 2. What a client may say, and what it may never say

A client may send exactly three kinds of frame:

```json
{ "t": "join", "protocol": 2, "invite": "3f9c0a…", "nickname": "Yue", "seat": "visitor",
  "resume": null, "take_over": false, "perceived": { "since": "1873" } }
{ "t": "submit", "token": "c1", "request": { … an ActionRequest (§6) … } }
{ "t": "leave" }
```

| Frame | Fields | Rules |
| --- | --- | --- |
| `join` | `protocol` (integer), `invite` (string), `nickname` (string), `seat` (entity key, required), `resume` (string or `null`, optional, default `null`), `take_over` (boolean, optional, default `false`), `perceived` (object, optional; from S11-C, §5.8) | Valid only before a seat is granted; a second `join` on a seated connection is `already_joined`. An absent `protocol` is read as `1`, so a revision-1 join is answered `protocol_mismatch` rather than `malformed_frame`. An absent `invite` or `nickname` is read as the empty string and answered by §4.1's checks. `resume` re-takes a seat this player's dropped connection held (§4.2); `take_over: true` takes a seat another connection holds (§4.2). `perceived`, when present, is `{ "since": <EventId string> \| null }`: `since` is required inside it and any other field inside it is `malformed_frame`. Without `perceived` the connection is never sent a `perceived` frame. |
| `submit` | `token`, `request` | Unchanged from revision 1. Before a seat is granted: `not_joined`. |
| `leave` | — | Releases the seat at once — no hold (§4.2) — and ends the connection: the server answers `closing { reason: "left" }` and closes. Before a seat is granted it is answered the same way — nothing is released, and a client that asks to go is let go. |

There is no fourth. In particular there is no frame that sets a value, names an observer, widens a
scope, or states a fact about the world. `"my money is now 5000"` is not a frame this protocol has, and
a server answers it:

```json
{ "t": "refused", "code": "unknown_frame",
  "detail": "this protocol has no frame of kind \"set_state\"; a client may only join, submit or leave" }
```

That is `NETWORKING.md` §2 as mechanism: the authority model is not a check applied to a rich message
vocabulary, it is the absence of the vocabulary.

**Unknown fields are refused.** Field order is free, but a known frame carrying a field this revision
does not define is `malformed_frame`. A typo is loud, and a `join` that tries to name an `observer` is
not silently ignored but refused.

## 3. Identity: seats, observers, and why a client names neither

```text
seat       a name a world offers. The authoring key of the entity the seat belongs to.
observer   the EntityId the server resolved that seat to, and the only perspective this
           connection ever receives.
session    which connection this is, for the admin surface. Not a credential.
```

A client names a **seat**; the server answers with the **observer**. No frame carries an observer
id, and a connection holds one seat for its whole life — a second `join` is refused
`already_joined`. This is `INV-13` as construction rather than as a check: perception cannot be
widened by asking, because nothing in the protocol asks for perception.

A seat is driven by **at most one controller at any instant**: one connection, or one in-server
controller, or nobody (§4.2). Who drives a seat is host state, never world state: no binding change is
journaled, states a fact, or moves the world's revision (`docs/DECISIONS.md` `ARC-40`).

`GET /status` lists the seats a world offers. A seat the roster does not contain is refused
`unknown_seat`.

A submitted request whose `actor` is not this connection's observer is refused
`actor_not_observer`. `ActionRequest.actor` is a field a client writes, so without this a client
could ask the world to act as somebody else.

## 4. The sequence

```text
client                                          server
  │  GET /ws  (upgrade)                            │
  │ ─────────────────────────────────────────────► │
  │  { "t": "join", "protocol": 2, "invite": …,    │
  │    "nickname": …, "seat": "visitor" }          │
  │ ─────────────────────────────────────────────► │   §4.1's checks
  │            { "t": "welcome", … }               │   observer assigned; the stream starts
  │ ◄───────────────────────────────────────────── │
  │            { "t": "observation", … }           │   repeatedly, at the server's cadence
  │ ◄───────────────────────────────────────────── │
  │  { "t": "submit", "token": "c1", … }           │
  │ ─────────────────────────────────────────────► │
  │            { "t": "result", … }                │   one answer per submitted request
  │ ◄───────────────────────────────────────────── │
  │  { "t": "leave" }                              │
  │ ─────────────────────────────────────────────► │
  │            { "t": "closing", "reason": "left" } │   then the server closes the socket
  │ ◄───────────────────────────────────────────── │
```

Before a successful `join`, a `submit` is refused `not_joined` and no observation is sent.

### 4.1 Joining: the checks, in order

A `join` is answered by these checks, in this order. The first that fails decides the answer.

```text
1  protocol   not 2            refused protocol_mismatch, closing {protocol_mismatch}, closed
2  invite     wrong or absent  after a fixed delay of 500 ms from the frame's arrival:
                               refused unauthorized, closing {unauthorized}, closed
3  nickname   not a nickname   refused invalid_nickname; the connection stays, and may join again
4  seat       the roster's     refused unknown_seat / seat_not_in_world, as in revision 1; the
                               connection stays
5  perceived  a cursor this    from S11-C, only when the join carries perceived (§5.8): refused
              world can serve  cursor_unavailable; nothing is granted, the connection stays, and
                               may join again. Before control, so that a refused cursor never
                               displaces or resumes anybody
6  control    §4.2's rules     refused invalid_resume / seat_occupied; the connection stays, and
                               may join again
   all pass                    welcome, then (§5.8) the perceived backfill, then the observation
                               stream
```

The protocol is checked first, so a client of another revision is told that rather than that its
credential is wrong; the invite is checked before anything that consults the world, so a connection
without it learns nothing about seats.

**The invite.** One per server process, given by its operator (`mineworld server --invite TOKEN`, or
the environment variable `MINEWORLD_INVITE`) or, when neither is given, generated from the operating
system's random source and printed once when the server starts, as a ready join line:

```text
[mineworld] invite <token> — join with: <address> seat=<seat> invite=<token>
```

A launcher may read the token after `invite ` on the line that begins `[mineworld] invite`. A supplied
invite is never printed; the server prints `[mineworld] join with: <address> seat=<seat> invite=<the
invite you gave>` instead. The invite is compared in constant time, appears in no save, World Pack,
observation, fact, `/status` answer or log line other than that one startup line, and a connection gets
one guess: a wrong invite ends it. A generated invite is 32 lowercase hexadecimal characters; one an
operator gives is 8 to 128 printable ASCII characters without whitespace.

**The nickname** labels the *player*, never the Person: the Person's name is world data
(`naming`'s `display-name`). It is 1 to 32 Unicode scalar values after trimming leading and trailing
whitespace, with no control character, and is kept trimmed. It is shown to the player's own connection
(`welcome.nickname`) and, from S11-D, to the admin surface — and to nobody else: it is in no
observation, no `/status` answer, no fact and no save. Two players may share a nickname.

### 4.2 Seats: who drives a Person, holding a dropped seat, and taking one over

Every seat is in exactly one of four states, and the server is the only writer of them:

```text
free        nobody drives the Person; it stands
hosted      an in-server controller drives it (mineworld server --agent SEAT, --town)
connected   one connection drives it
held        its connection dropped without `leave`; nobody drives it — not even its in-server
            controller — until the hold ends or the connection's `resume` comes back
```

A seat's **default** is `hosted` when the server runs an in-server controller for it, and `free`
otherwise. A seat returns to its default with a controller **built afresh** at that instant — never one
resumed from memory — so a controller handed a Person back does not answer what was said to whoever
drove it in between.

**Joining (check 5 of §4.1).** After the seat is found in the roster, a `join` is decided by the first of
these rules that applies:

```text
1  resume given      it must equal the seat's held secret, or the secret of the connection that
                     holds the seat now (a half-open socket the server has not yet seen drop):
                     granted, took_over "held"; that older connection, if still open, is sent
                     closing {superseded} and closed. Any other resume: refused invalid_resume.
2  free              granted, took_over "none"
3  hosted            granted, took_over "hosted": the in-server controller yields to a person, with
                     no flag — it is not a connection
4  connected / held  refused seat_occupied — unless take_over is true: then granted, took_over
                     "connection"; the connection holding it, if live, is sent closing {taken_over}
                     and closed, and a held seat's secret stops working
```

Any holder of the server's invite may take any seat with `take_over: true`, a held seat included: MVP-0
has one level of trust, the invite (`NETWORKING.md` §9; `ARC-40`'s limitation). A seat is never driven
by two controllers at once: the displaced connection is unbound before the new one is welcomed.

**Leaving and dropping.** A connection that sends `leave` gives its seat back at once: the seat returns
to its default. A connection whose socket ends without `leave` leaves its seat **held** for the
server's hold (`welcome.hold_seconds`, wall seconds; `mineworld server --hold`, default 30; `0` means no
hold, and a dropped seat returns to its default at once). When the hold ends the seat returns to its
default. During a hold the world goes on — every other Person acts — and the held Person stands.

**Resuming.** `welcome.resume` is a secret for this connection's binding. A client whose socket drops
may join the same seat presenting it, within the hold, and is welcomed `took_over: "held"`, as the same
observer, with a new `session` and a new `resume` (every welcome carries a fresh one; the old one is
dead). The stream restarts with a whole observation. After a server restart every hold and every secret
is gone (none is persisted): a client joins again with its invite, and finds the seat free or hosted.
The secret is sent only in its holder's own `welcome`; it is in no observation, no `/status` answer, no
fact, no save and no log line.

**In-server controllers.** A controller the server runs (`--agent SEAT`: the reactive rule controller;
`--town`: the paced one on every other seat) acts exactly as a connection does: it reads the observation
perception computed for its own Person, and every request it makes takes the session's path — the actor
check, a server-allocated `ActionId` and instant, the journal before the answer. It is not a client:
`/status`'s `clients` does not count it.

## 5. Frames the server sends

| Frame | When |
| --- | --- |
| `welcome` | once, after a granted `join` |
| `observation` | the first frame of the stream; every `keyframe_every`-th frame and the first after a resume (from S11-C); any frame the server chooses |
| `delta` | from S11-C, if `DEP-15` ships deltas: any other frame of the stream (§5.3) |
| `perceived` | from S11-C, only to a connection whose `join` carried `perceived`: the reliable stream of the facts its observer learned (§5.8) |
| `result` | one per `submit` |
| `refused` | a frame was not accepted (§5.5) |
| `closing` | immediately before the server closes the socket (§5.6) |

### 5.1 `welcome`

```json
{ "t": "welcome", "protocol": 2, "seat": "visitor", "observer": "101", "nickname": "Yue",
  "session": "7", "resume": "5f0c2a9e8d7b6c5a4f3e2d1c0b9a8f7e", "hold_seconds": 30,
  "took_over": "hosted", "world": { … a WorldSummary, §5.7 … } }
```

- `observer` is the identity this connection sees the world as, a decimal string (§7).
- `nickname` is the nickname as accepted: trimmed.
- `session` is a `SessionId` as a decimal string: which connection this is, distinct from every other
  connection the process has had. It names this connection on the admin surface (S11-D) and nowhere
  else; it is not a credential.
- `resume` is a 128-bit secret as 32 lowercase hexadecimal characters, with which a client whose socket
  dropped may re-take its seat within `hold_seconds` (§4.2). A fresh one on every welcome. A client keeps
  it to itself: it is the only proof that a later connection is this player's.
- `hold_seconds` is how long this server holds a dropped connection's seat, in wall seconds (an integer;
  `0` means it holds none).
- `took_over` says whether control of the Person changed hands: `"none"` when the seat was free,
  `"hosted"` when an in-server controller was driving it, `"held"` when this join resumed a held seat,
  `"connection"` when this join took the seat from another connection with `take_over: true` (§4.2).
  It never says which controller kind or which player: a client may tell its player "you are now playing
  Alice, who was living on her own", and learns nothing about any other player.
- `world` is the same value `GET /status` returns (§5.7).

A client written before S11-B received `resume: null`, `hold_seconds: 0` and `took_over: "none"` on every
welcome, and seats were not exclusive then (§10).

### 5.2 `observation`

```json
{ "t": "observation", "seq": 12, "revision": 7, "acted_through": "41", "observation": { … } }
```

`revision` is the persisted revision of the world state this observation was computed from: the number
of the last input the world has committed to its save (`docs/DECISIONS.md` `ARC-25`). Monotonic, never
reused, durable before any frame names it. `null` for a world that is not persisted. Two clients whose
observations carry the same revision are looking at one committed state of one world — the fourth line
of `AC-15`'s evidence.

`observation` is `mineworld-contracts`' own `Observation`, serialized by the contract, with
`serde_json::Value` as its payload type. Every field of it is the perception system's answer for
**this** observer: two connected clients receive two different observations, computed separately, and
neither is a filtered copy of a world frame.

Each entry of `observation.affordances` is the contract's `Affordance`: `action_type`, `target`,
`available`, `unavailable_reason` and `requirement`, and — only on a **complete affordance** — a
`payload`: the complete request payload the offering system would accept, in the same JSON shape a
client would send inside `request.payload.payload` (§6). The field is **absent**, not `null`, on an
affordance without one. Several complete affordances may share an `action_type` and a `target` — one
per choice the system offers — and differ only in `payload`; a client keeps them apart by their
position in the list, never by looking one up by type and target (`docs/DECISIONS.md` `ARC-34`).

```json
{ "action_type": "ring", "target": null, "available": true, "unavailable_reason": null,
  "requirement": { "place": "any", "within_range": null, "requires_line_of_access": false,
                   "requires_target_available": false },
  "payload": { "bell": "low" } }
```

`seq` counts frames on this connection, from 1. It exists because `Observation.at` is a `WorldTime`
in whole seconds, so several frames of a 10 Hz stream carry the same instant and could not otherwise
be ordered (`spike/FINDINGS.md` F7). A client should treat a lower `seq` as stale. A gap in `seq`
does not occur; frames dropped for a client that is not reading are dropped **before** they are
numbered.

**From S11-C** (absent before it):

- `observation.entities` is in ascending `EntityId` order.
- `observation.events` is the facts this observer learned since the previous frame on this connection,
  oldest first, each a `PerceivedEvent` (below). Which facts an observer learns is perception's
  judgement, made when each fact is recorded, against where people were at that moment
  (`docs/DECISIONS.md` `ARC-43`). **Best effort:** the facts waiting for a connection's next frame are
  bounded (256 by default); when more arrive the oldest are dropped and counted in
  `WorldSummary.events_dropped` (§5.7). Facts waiting for a frame that was dropped because the client was
  not reading are not lost with it: they ride on the next frame that is sent. No fact appears in two
  frames' `events` on one connection. The reliable form of the same facts is the `perceived` stream
  (§5.8). Before S11-C `events` is always empty.
- `acted_through` is the newest `ActionId` the server allocated to a request **submitted on this
  connection** whose dispatch had completed (with any result) before this observation was computed, or
  `null` before the first. It never decreases on a connection. A request refused before an identity
  was allocated (`actor_not_observer`, for example) never sets it. A new connection on a seat — a
  resume or a takeover — starts at `null`: it made none of its predecessor's requests. A client that
  predicts the effect of its own requests (a 3D body that walks before the server answers) reconciles
  with an observation only once `acted_through` reaches the request it predicted.

**A fact on the wire.** A `PerceivedEvent` is the contract's `EventEnvelope`, serialized by the
contract, with the payload record's inner `payload` replaced by the owning pack's payload read as JSON:

```json
{ "id": "1890", "at": 4100, "event_type": "spoke", "subjects": ["5"], "participants": ["5", "7"],
  "place": "3", "caused_by": { … }, "visibility": { "place": "3" }, "provenance": { … },
  "payload": { "event_type": "spoke", "schema_version": 1, "payload": { "utterance": "hello" } } }
```

The example abbreviates; the exact form of every field, including how a `PlaceId` and a `Causation`
are written, is the contract's, and `server/tests/frames/perceived.json` holds one as the Rust types
write it.

Every pack in this repository encodes its payloads as JSON, but that is each pack's convention, not a
contract (`DEP-5`). A payload whose bytes are not JSON is delivered with `payload.payload: null`, and the
server says so once per event type on its own output. A client names an `event_type` from `/status`'s
`states` (§5.7) and treats a type it does not know as "something happened", never as an error.

### 5.3 `delta` (from S11-C)

```json
{ "t": "delta", "seq": 13, "base": 12, "revision": 7, "acted_through": "41",
  "delta": { … an ObservationDelta … } }
```

`acted_through` means what it means on `observation` (§5.2), and is stated whole on every frame.

```json
{ "at": 4112,
  "self_location": { … a Location … },
  "entities": { "upsert": [ PerceivedEntity, … ], "remove": [ "9007199254740995" ] },
  "relations": [ Relation, … ],
  "affordances": [ Affordance, … ],
  "events": [ PerceivedEvent, … ] }
```

Applying a delta whose `base` is the `seq` of the observation the client holds yields the observation
of `seq`, field by field:

| Field | Present | Meaning |
| --- | --- | --- |
| `at` | always | replaces |
| `self_location` | only if changed | replaces; `null` means the observer now has no location |
| `entities.upsert` | only if non-empty | each replaces the entity with the same id, or is added; the result is re-sorted by id |
| `entities.remove` | only if non-empty | ids no longer perceived; removing an id not held is a client-side protocol error |
| `relations` | only if changed | replaces the whole list |
| `affordances` | only if changed | replaces the whole list, order preserved |
| `events` | always | replaces (events are a since-last-frame stream, never accumulated) |

`observer` never changes on a connection and is not in a delta. A delta whose `base` is not the `seq`
the client holds cannot occur on one WebSocket; a client that sees one drops the connection and resumes,
which yields a whole observation. A client must accept a whole `observation` at any time.

**Keyframes.** When deltas ship, the first frame of a connection is a whole `observation`; so is every
`keyframe_every`-th frame (`mineworld server --keyframe-every N`, default 50), the first frame after a
resume, and the first frame after a `perceived` backfill (§5.8). A delta is computed against the last
frame **this connection was sent**, so it is always against a frame the client received.

**Per `DEP-15`.** Whether deltas ship, and in which encoding, is decided by S11-C's measurement
(step-12 §9.3 CP-C1) under a rule frozen before measuring. The three outcomes are specified here so that
the decision changes no other part of this document:

```text
typed        the ObservationDelta above, in "delta"
json-patch   instead of "delta", "patch": an RFC 6902 JSON Patch array that turns the JSON of the
             observation of `base` into the JSON of the observation of `seq`:
             { "t": "delta", "seq": 13, "base": 12, "revision": 7, "acted_through": "41",
               "patch": [ { "op": "replace", "path": "/at", "value": 4113 }, … ] }
keyframes    no delta frame is sent; every frame is a whole observation
             only
```

`DEP-15` records which outcome this revision ships; "whole observations only" is a conforming outcome.

### 5.4 `result`

```json
{ "t": "result", "token": "c1", "action_id": "1",
  "result": { "accepted": { "events": [ "1" ] } } }
```

`token` is the client's own string, echoed. `action_id` is the identity the **server** allocated.
`result` is the contract's `ActionResult`: `accepted` with the events the request caused,
`rejected` with a `Rejection`, or `unavailable` when no enabled system provides the action type at
all (`INV-10` — a different statement from a rejection, and a client should say so differently).

The `ActionId` is here and not inside `ActionResult` deliberately: `contracts/src/action.rs` records
that correlation belongs to the protocol frame, and that a client which needs to recognize *later*
facts caused by its request — a process that produces events after the answer — finds them by
matching their `Causation` against this `action_id`.

### 5.5 `refused`

```json
{ "t": "refused", "token": "c1", "code": "actor_not_observer", "detail": "…" }
```

`token` appears when one could be recovered from the offending frame. `detail` is for a developer;
a client branches on `code` and never on `detail`.

| `code` | Meaning |
| --- | --- |
| `malformed_frame` | Not JSON, no `t`, a `t` that is not a string, or a known frame whose fields do not fit — including an unknown field. |
| `unknown_frame` | A frame kind this protocol does not have — including any message that asserts state. |
| `not_joined` | A request arrived before the connection had a seat. |
| `already_joined` | A second `join` on one connection. |
| `unknown_seat` | The world offers no such seat. |
| `seat_not_in_world` | The roster names an entity this world does not have: a fault in the world's composition, not in the frame. |
| `actor_not_observer` | The request asked the world to act as somebody else. |
| `dispatch_failed` | A system broke its own contract while resolving. The request has no answer; nothing about the frame was wrong. |
| `world_stopped` | The world is no longer running. |
| `protocol_mismatch` | `join.protocol` is not a revision this server speaks. Followed by `closing`. |
| `unauthorized` | A wrong or missing invite, answered after a fixed delay of 500 ms. Followed by `closing`. |
| `invalid_nickname` | Empty after trimming, longer than 32 scalar values, or containing a control character. |
| `seat_occupied` | Another connection holds the seat, or it is held for a dropped connection, and the `join` gave neither a valid `resume` nor `take_over: true` (§4.2). |
| `invalid_resume` | A `resume` that matches neither the seat's hold nor its live connection (expired, superseded, or for another seat); the client may retry without it (§4.2). |
| `cursor_unavailable` | From S11-C: a `join`'s `perceived.since` is a cursor this world cannot serve — older than what it can replay (a world with no save serves only from the join on), newer than the newest fact it has recorded, or one whose history could not be read just now (`detail` says which). Nothing is granted; the connection stays and may join again, for example with `since` set to `null` on a persisted world. |
| `lagged` | From S11-C: this connection's `perceived` stream fell further behind than the server holds for it. Followed by `closing { reason: "lagged" }`; the client rejoins with its `resume` and its last `perceived` cursor, and loses nothing. |

A refusal is **not** a `Rejection`. A rejection means the world considered a well-formed request and
said no, and it arrives inside a `result`. A refusal means the frame was not a request at all. Being
closed is `closing`, never a refusal.

### 5.6 `closing`

```json
{ "t": "closing", "reason": "left", "detail": "…" }
```

Sent immediately before the server closes the connection. A client branches on `reason`; `detail`,
when present, is for a developer. Nothing the client sends after it is acted on. **The client closes
the socket on receiving it**; the server waits up to 2 seconds for that, then sends its own close
frame (status 1000) and drops the connection. The client closes first because some WebSocket
implementations — Godot's `WebSocketPeer` among them — discard frames not yet read when a close frame
arrives with them, which would lose this frame and the refusal before it.

| `reason` | Meaning | Sent from |
| --- | --- | --- |
| `left` | the client sent `leave` | S11-A |
| `unauthorized` | the invite was wrong or missing | S11-A |
| `protocol_mismatch` | the client speaks another revision | S11-A |
| `world_stopped` | the world stopped; there is nothing left to observe | S11-A |
| `kicked` | the operator removed this connection | S11-D |
| `superseded` | the seat was re-taken with this connection's `resume` by a newer connection | S11-B |
| `taken_over` | another connection took the seat with `take_over: true` (§4.2) | S11-B |
| `lagged` | the connection's `perceived` stream overflowed (§5.8); preceded by `refused { code: "lagged" }` | S11-C |
| `server_stopping` | the server process is shutting down | the S11 pull request that wires shutdown into sessions |

### 5.7 `GET /status` and `welcome.world`: what the world is

```json
{ "protocol": 2, "instance": "1a2b3c4d5e6f70819293a4b5c6d7e8f9", "at": 4112, "time_scale": 1,
  "entities": 41,
  "systems": [ { "system": "conversation", "enabled": true,
                 "provides": [ "talk" ], "states": [ "spoke", "conversation-started" ] } ],
  "seats": [ "visitor", "wanderer", "alice" ], "clients": 2,
  "observations_dropped": 0, "events_dropped": 0, "faults": 0, "revision": 7 }
```

`GET /status` and `welcome.world` carry this one value. It describes a world's **composition** — which
systems it installed, what they do, how many entities it has — and never its state: no entity, no
component and no position appears in it, because a client's knowledge of state arrives only in an
`Observation`. It is public, and it names no player and no nickname.

- `instance` is **which world this is**: allocated when the world is created, reported unchanged for as
  long as that world lives, and identical for every client connected to it. It is not a name, not a
  secret and not derived from the world's content — two worlds created from one World Pack are two
  instances. A **persisted** world stores its instance in its save, and a server that resumes the save
  reports the same instance, because a restarted world is the same world (`docs/MVP.md` `AC-6`). 128
  bits as a lowercase hexadecimal string, for the reason in §7.
- `systems[]`, in registration order: `system`, `enabled` (a disabled system's actions are
  `unavailable`), `provides` (the action types the system provides) and `states` (the event types it
  declares it may state, including another system's vocabulary it is declared to state). A client uses
  `provides` to know whether to offer a verb at all, and `states` to name an `event_type` it receives.
- `at` is the world's clock as of the answer, and `time_scale` how many world seconds pass per wall
  second while the world is hosted (an integer ≥ 1; `mineworld server --time-scale`, default 1). It is a
  deployment setting, not world state: the same save hosted at another scale is the same world.
- `seats`: the seats a client may ask for.
- `clients`: connections that hold a seat. In-server controllers are not clients and are not counted
  (§4.2).
- `observations_dropped`: frames the server did not send because a client was not reading them — the
  world never waits for a client.
- `events_dropped`: from S11-C, facts lost from `observation.events` to a full per-connection queue
  (§5.2) — never from the `perceived` stream, which does not drop (§5.8). `0` before S11-C, when no fact
  is delivered and none can be dropped.
- `faults`: dispatches, and advances of the world's clock, in which a system broke its own contract.
- `revision`: the persisted head, or `null` for a world that is not persisted (§5.2).

Removed from revision 1: `deferrals_unscheduled`, always `0` since S4.

`GET /health` answers `{ "status": "ok", "protocol": 2 }` without consulting the world.

### 5.8 `perceived`: the reliable stream of what an observer learned (from S11-C)

```json
{ "t": "perceived", "through": "1907", "events": [ PerceivedEvent, … ] }
```

A client that must not miss a fact — a memory, a transcript, a cognition process — asks for this
stream in its `join` with `perceived: { "since": <cursor> }`, where the cursor is the `through` of the
last `perceived` frame it processed, or `null` for "from the beginning of this world". It carries the
same facts, judged by the same rule, as `observation.events` (§5.2), but none is ever dropped.

- **`events`** are facts this observer learned, each a `PerceivedEvent` (§5.2), in ascending `EventId`:
  never reordered, never dropped, never duplicated within a connection.
- **`through`** is the newest `EventId` the server has considered for this connection, whether this
  observer learned of it or not. A client stores it as its cursor. A frame is sent when it has at least
  one event, and once at the end of a backfill even when it has none.
- **The head.** When the join is granted the server notes the newest `EventId` its world has recorded,
  the connection's head. Facts recorded after it reach the connection live. If `since` is `null` or
  older than the head, the server first sends the **backfill** — the facts in `(since, head]` this
  observer learned, in frames of at most 256 events, the last with `through` equal to the head — and only
  then the live stream and the observation stream.
- **Where a backfill comes from.** A persisted world replays it from its save, judged by the same rule
  as the live stream; that is the function `mineworld perceived` runs, so a client's stream and the
  export of the same save agree fact for fact. A world with no save keeps no history beyond its
  observers' queues, and serves a `perceived` stream only from the join on: it answers any `since`
  older than its head `cursor_unavailable` (§5.5), and `since` equal to the head is the way to ask it
  for the live stream alone. Any `since` newer than the head is not a cursor of this world and is
  answered `cursor_unavailable` everywhere.
- **Order.** Every `perceived` frame carrying a fact recorded at revision R is sent before any
  `observation` or `delta` frame whose `revision` is R or later. A client that reads frames in order
  therefore never sees the world change before it has been told the facts that changed it.
- **Flow.** The world does not wait for a client. The server holds up to 4 096 facts not yet sent to a
  connection's `perceived` stream; while some are waiting, that connection's observation frames are
  skipped (and counted in `observations_dropped`) rather than sent ahead of them. A connection that
  falls further behind than that is sent `refused { code: "lagged" }` and `closing { reason: "lagged" }`,
  and rejoins with its `resume` and its cursor. Nothing is lost; the price of not reading is a
  reconnect, never a gap.
- **Resuming.** A client whose socket dropped rejoins with `resume` (§5.1) and `perceived: { since:
  <its cursor> }`, and receives exactly the facts after the cursor: those recorded while it was gone come
  in the backfill.

## 6. Submitting a request

```json
{ "t": "submit", "token": "c1", "request": {
    "actor": "101",
    "action_type": "talk",
    "target": "9007199254740995",
    "payload": { "action_type": "talk", "payload": { "topic": "greeting" } },
    "actor_location": null } }
```

The `request` object is the contract's `ActionRequest`, and the contract is the only thing that
decides whether it is well formed. Four consequences a client must know:

1. **No identity and no instant.** There is no `action_id` and no `issued_at` to send: the server
   allocates both (`INV-6`). A client that invents an `ActionId` is the defect
   `spike/FINDINGS.md` F4 measured — two clients collide on their first action.
2. **`action_type` appears twice** and the two must agree, in the envelope and inside `payload`.
   The contract refuses a frame whose halves disagree (`FINDINGS.md` F8.1).
3. **`actor_location` is a report, not an assertion.** A 3D client sends the position it walked to;
   a 2D client that models no position sends `null`. The server evaluates spatial requirements
   against its own authoritative state and is free to ignore the report
   (`ENGINEERING_RULES.md` §8).
4. **`payload` is the owning system's own shape.** The server carries it without interpreting it;
   the system that provides the action decodes it. It reaches that system as the JSON bytes of what
   was sent.

**Submitting a complete affordance.** A client may submit a complete affordance (§5.2) unchanged,
without knowing the action: `request.action_type` and `request.payload.action_type` are the
affordance's `action_type`, `request.payload.payload` is the affordance's `payload`, and
`request.target` is the affordance's `target`. The offer decides nothing: the server validates the
request exactly as any other, and the world may have changed since the observation. A client still
decides nothing new — whether to offer the player the choice is presentation, and whether it is
valid is the server's.

```json
{ "t": "submit", "token": "c9", "request": {
    "actor": "101",
    "action_type": "ring",
    "target": null,
    "payload": { "action_type": "ring", "payload": { "bell": "low" } },
    "actor_location": null } }
```

### 6.1 What two clients of different dimensions must agree about

`docs/MVP.md` §9 `AC-13` requires a `talk` from a 2D click and a `talk` from a 3D
walk-up-look-at-press to have an **identical semantic core**, and names the fields:

```text
identical        actor · action_type · target · payload
may differ       actor_location — a 3D client reports the position it walked to, a 2D client
                 that models no position sends null
```

The comparison is implemented **once**, in `server/src/parity.rs` (`semantic_core` and
`differing_fields`), so that an acceptance test states which difference it permits rather than
writing its own comparison and forgetting a field. `spike/FINDINGS.md` F6 measured three differing
fields; PR 04 removed `action_id` and `issued_at` from what a client submits, so one is left.

A client author's obligation is the short version of the same thing: everything after acquisition is
identical. How the player expressed the wish — a click, a raycast, a menu — changes nothing in the
frame except the position the client is able to report.

### 6.2 Moving, and the reporting rule

A person moves only by a `move` request, provided by the `movement` system when a world enables it
(`DECISIONS.md` `ARC-26`). There is no other way for a client to change where anybody is.

```json
{ "t": "submit", "token": "c7", "request": {
    "actor": "5",
    "action_type": "move",
    "target": null,
    "payload": { "action_type": "move", "payload": { "to": {
        "place": { "entity": "1", "entity_type": "place" },
        "local": { "x": 3200, "y": 600, "z": 0 },
        "facing": { "yaw": 270000, "pitch": null } } } },
    "actor_location": null } }
```

The server, not the client, decides whether the move is possible, against the person's
authoritative position:

```text
same place       accepted if `to` is at most MAX_STRIDE = 2 000 mm from the authoritative position
another place    accepted only through a passage the world declares, within MAX_STRIDE of the
                 doorway on this side, landing within MAX_STRIDE of it on the other
otherwise        rejected too_far_away — reaching a place that does not open onto this one is travel
```

**The reporting rule.** Because the bound is per request, a client must **report a `move` before its
body has travelled `MAX_STRIDE` since the last position the server accepted**. The bound is not a
speed limit, and a client that reports too rarely is refused for moving legally: a body jogging at
about 2.6 m/s that reports once a second asks to move 2.6 m and is answered `too_far_away`, after
which every later report is further still. A client that is refused reconciles: it moves its body
back to the position the next observation shows, and continues from there.

Turning on the spot is a `move` to the same position with a new `facing`. A world that models no
continuous position moves within a place and through passages with `local: null`.

## 7. Numbers, and the one rule a client with a single number type must follow

**Identities are decimal strings.** `EntityId`, `EventId`, `ActionId`, `ProcessId` and `SessionId` are
written as `"9007199254740995"`, in every position they appear, including inside a component or event
payload. `mineworld-contracts` does this itself for its identities, keyed on whether the format is
human-readable; the transport adds nothing. The reason is measured rather than theoretical: a JSON
parser whose only number type is a double turns `9007199254740995` and `9007199254740997` into the
same value, so two entities become one (`spike/FINDINGS.md` F1, F2).

A client must parse an id as a **string** and keep it as one, or as a 64-bit integer. Parsing it as
a number is the silent corruption above. Deserialization also accepts a JSON number, so a
hand-written fixture keeps working; a client should still send strings.

**Every other numeric field is an integer**, and the contract refuses a float where one is declared:
positions are millimetres, angles are millidegrees, times are whole seconds, `protocol` and
`hold_seconds` are integers. A client whose language has one number type must round and cast before
encoding — GDScript's `1500.0` is refused with `invalid type: floating point 1500.0, expected i32`
(`FINDINGS.md` F9). That is the contract working: a loud refusal rather than a silent truncation.

## 8. Rates, and what a client must not conclude from them

```text
observation cadence   the server's, configurable, 10 Hz by default
world clock           whole simulated seconds; several frames share one instant; time_scale (§5.7)
                      world seconds pass per wall second
rendering             the client's own business entirely
```

`NETWORKING.md` §4 forbids conflating these. An observation is not a tick, a tick is not a frame,
and nothing in this protocol replicates at a rendering rate. A client that stops reading loses
observations and the world does not wait for it: the server drops frames for a slow client rather
than blocking the world, which is why `seq` may jump for a client that was not keeping up and why
a client must render the newest observation it has rather than accumulating them.

## 9. What revision 2 deliberately does not have

```text
a subscribe / scope frame      a connection perceives its observer (INV-13); nothing to widen
admin frames on the socket     admin is HTTP (S11-D); a frame would widen the client vocabulary
binary or compressed frames    JSON only (ARC-41); Godot's WebSocketPeer cannot negotiate
                               permessage-deflate
an observer-naming field       there is none, and a join that tries to carry one is malformed
other players' nicknames       a nickname is not world data (§4.1)
accounts, matchmaking          NETWORKING.md §9: an invite and a nickname are the whole of MVP
                               authentication
```

## 10. Revisions, and the landing table

`protocol` is `2`. A client states the revision it speaks in its `join`; a server that does not speak
it answers `refused { code: "protocol_mismatch" }`, then `closing`, and closes. A client that receives
a `welcome.protocol` it does not know refuses to continue rather than guess. A future revision raises
`protocol`.

Revision 2 is **specified whole and implemented incrementally** (`docs/DECISIONS.md` `ARC-41`). It
lands over the pull requests of step S11 (S11-A … S11-E). While the step is in flight, a server may
**omit** a frame or a field this document marks as arriving later, and it never gives a frame or a
field it does send a different meaning. A client written against this document is therefore correct
against every intermediate `main`.

The table says what each pull request lands and what a server sends before it. Fields named by the
coordination rulings of `.structured-coding/plans/mvp0/overall.md` "Parallel build-out, 2026-10-08" are
specified in this document by their owning pull request when it lands them; until then a `join`
carrying one of them is `malformed_frame` (§2, unknown fields), so a client sends such a field only to a
server whose `PROTOCOL.md` lists it as landed.

| Frame / field | Lands in | Before it lands |
| --- | --- | --- |
| `join` with `protocol`, `invite`, `nickname`, `seat`, `resume`; `leave`; `closing` (`left`, `unauthorized`, `protocol_mismatch`, `world_stopped`); `welcome.nickname`, `welcome.session`; refusal codes `protocol_mismatch`, `unauthorized`, `invalid_nickname`, `invalid_resume`; `WorldSummary` revision 2 | S11-A | — |
| `welcome.resume` (a secret), `welcome.hold_seconds` (non-zero), `welcome.took_over` (`hosted`, `held`, `connection`); seat exclusivity, holds, `seat_occupied`; `closing.superseded`, `closing.taken_over`; in-server controllers not counted in `clients` (§4.2) | S11-B — **landed** | `resume: null`, `hold_seconds: 0`, `took_over: "none"`; seats are not exclusive |
| `join.take_over` (an explicit takeover flag, §2, §4.2) | S11-B, coordination ruling 1 — **landed** | a `join` carrying it is `malformed_frame` |
| `WorldSummary.time_scale` (world seconds per wall second, in `welcome.world` and `/status`, §5.7) | S11-B, coordination ruling 1 — **landed** | absent; the hosted clock runs one simulated second per wall second |
| `observation.events` (facts this observer learned, §5.2); `observation.entities` in id order; `events_dropped` non-zero | S11-C | `events` empty; entity order unspecified; `events_dropped: 0` |
| `delta` frames and periodic keyframes, `--keyframe-every` (§5.3) | S11-C, in the encoding `DEP-15` records, or not at all | whole `observation` frames only |
| `acted_through` on the `observation` (and `delta`) frame (§5.2) | S11-C, coordination ruling 1 | absent |
| the `perceived` stream (§5.8): `join.perceived`, the `perceived` frame, refusals `cursor_unavailable` and `lagged`, `closing.lagged` | S11-C, coordination ruling 2 | no `perceived` frame; a `join` carrying `perceived` is `malformed_frame` |
| `/admin` routes; `closing.kicked` | S11-D | no admin route |
