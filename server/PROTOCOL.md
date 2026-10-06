# The MineWorld client protocol, revision 1

**Audience:** coding agents writing a client, a controller, or another server. This is a
specification, not an introduction; `server/README.md` is the short human orientation.
**Authority:** [`docs/NETWORKING.md`](../docs/NETWORKING.md) decides the model; this document
specifies the frames that implement it. Where the two disagree, `NETWORKING.md` governs and this
document is the defect.
**Implemented by:** `server/src/protocol.rs` (the frames), `server/src/session.rs` (the sequence),
`server/src/app.rs` (the routes). The Rust types are authoritative over the JSON examples here, in
the sense `docs/ARCHITECTURE.md` §13 states.

---

## 1. Transports and routes

```text
GET /health   HTTP    liveness of the process. Answered by the transport, never by the world.
GET /status   HTTP    what the world is. Answered by the world itself.
GET /ws       HTTP    upgrade to the live connection.
```

Frames on the WebSocket are **JSON text**. A binary frame is refused. Ping and pong are the
transport's own business and carry no protocol meaning.

One server binary serves localhost, a LAN and a cloud container with no semantic difference
(`NETWORKING.md` §§1, 8). The address is configuration.

## 2. What a client may say, and what it may never say

A client may send exactly two kinds of frame:

```json
{ "t": "join",   "seat": "player" }
{ "t": "submit", "token": "c1", "request": { … an ActionRequest … } }
```

There is no third. In particular there is no frame that sets a value, names an observer, widens a
scope, or states a fact about the world. `"my money is now 5000"` is not a frame this protocol has,
and a server answers it:

```json
{ "t": "refused", "code": "unknown_frame",
  "detail": "this protocol has no frame of kind \"set_state\"; a client may only join or submit" }
```

That is `NETWORKING.md` §2 as mechanism: the authority model is not a check applied to a rich
message vocabulary, it is the absence of the vocabulary. The server validates every world change
because there is no path by which a client can describe one.

## 3. Identity: seats, observers, and why a client names neither

```text
seat       a name a world offers. The authoring key of the entity the seat belongs to.
observer   the EntityId the server resolved that seat to, and the only perspective this
           connection ever receives.
```

A client names a **seat**; the server answers with the **observer**. No frame carries an observer
id, and a connection holds one seat for its whole life — a second `join` is refused
`already_joined`. This is `INV-13` as construction rather than as a check: perception cannot be
widened by asking, because nothing in the protocol asks for perception.

`GET /status` lists the seats a world offers. A seat the roster does not contain is refused
`unknown_seat`.

A submitted request whose `actor` is not this connection's observer is refused
`actor_not_observer`. `ActionRequest.actor` is a field a client writes, so without this a client
could ask the world to act as somebody else.

## 4. The sequence

```text
client                                  server
  │  GET /ws  (upgrade)                    │
  │ ─────────────────────────────────────► │
  │  { "t": "join", "seat": "player" }     │
  │ ─────────────────────────────────────► │
  │            { "t": "welcome", … }       │   observer assigned; the stream starts
  │ ◄───────────────────────────────────── │
  │            { "t": "observation", … }   │   repeatedly, at the server's cadence
  │ ◄───────────────────────────────────── │
  │  { "t": "submit", "token": "c1", … }   │
  │ ─────────────────────────────────────► │
  │            { "t": "result", … }        │   one answer per submitted request
  │ ◄───────────────────────────────────── │
```

Before a successful `join`, a `submit` is refused `not_joined` and no observation is sent.

## 5. Frames the server sends

### `welcome`

```json
{ "t": "welcome", "protocol": 1, "seat": "player", "observer": "101",
  "world": { "protocol": 1, "instance": "1a2b3c4d5e6f70819293a4b5c6d7e8f9",
             "at": 0, "entities": 4,
             "systems": [ { "system": "presence", "enabled": true } ],
             "seats": [ "player" ], "clients": 1,
             "observations_dropped": 0, "deferrals_unscheduled": 0, "faults": 0,
             "revision": 7 } }
```

`instance` is **which world this is**: allocated when the world is created, reported unchanged for as
long as that world lives, and identical for every client connected to it. It is not a name, not a
secret and not derived from the world's content — two worlds created from one World Pack are two
instances. A world that is not persisted lives as long as its process, so its instance is allocated
when the world's thread starts. A **persisted** world (`mineworld server <world> --save DIR`) outlives
its process: its instance is stored in its save, and a server that resumes the save reports the same
instance, because a restarted world is the same world (`docs/MVP.md` `AC-6`). It exists because
`docs/MVP.md` §9.1 requires `AC-15`'s evidence to name identity rather than appearance: two clients
each talking to their own world are told two different instances, which is the false success that
criterion exists to exclude. 128 bits as a lowercase hexadecimal string, for the reason in §7.

`revision` is **which persisted revision of the world this answer describes**: the number of the last
input the world has committed to its save ([`docs/DECISIONS.md`](../docs/DECISIONS.md) `ARC-25`).
Monotonic, never reused, and durable before any frame names it — a server never tells a client about a
revision a crash could still lose. `null` for a world that is not persisted, which has no saved
revision to name. An integer, like every non-identity number (§7). Added within revision 1 of this
protocol: a client that does not read it is unaffected.

`observations_dropped` counts frames the server did not send because a client was not reading them:
the world never waits for a client, so a client that falls behind loses observations rather than
delaying anybody. `deferrals_unscheduled` counted work a system asked to happen later while the
server had no scheduler; since S4 the world's own schedule holds and fires every deferral, so it is
always `0`, and it remains in this revision only because removing a field is a protocol change.
`faults` counts dispatches, and advances of the world's clock, in which a system broke its own
contract.

`world` is the same value `GET /status` returns. It describes a world's **composition** — which
systems it installed, how many entities it has — and never its state: no entity, no component and
no position appears in it, because a client's knowledge of state arrives only in an `Observation`.

### `observation`

```json
{ "t": "observation", "seq": 12, "revision": 7, "observation": { … } }
```

`revision` is the persisted revision of the world state this observation was computed from, with the
meaning and the `null` given for `world.revision` above. Two clients whose observations carry the same
revision are looking at one committed state of one world — the fourth line of `AC-15`'s evidence.

`observation` is `mineworld-contracts`' own `Observation`, serialized by the contract, with
`serde_json::Value` as its payload type. Every field of it is the perception system's answer for
**this** observer: two connected clients receive two different observations, computed separately,
and neither is a filtered copy of a world frame.

`seq` counts frames on this connection, from 1. It exists because `Observation.at` is a `WorldTime`
in whole seconds, so several frames of a 10 Hz stream carry the same instant and could not otherwise
be ordered (`spike/FINDINGS.md` F7). A client should treat a lower `seq` as stale. A gap in `seq`
does not occur; frames dropped for a client that is not reading are dropped **before** they are
numbered.

### `result`

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

### `refused`

```json
{ "t": "refused", "token": "c1", "code": "actor_not_observer", "detail": "…" }
```

`token` appears when one could be recovered from the offending frame. `detail` is for a developer;
a client branches on `code` and never on `detail`.

| `code` | Meaning |
| --- | --- |
| `malformed_frame` | Not JSON, no `t`, a `t` that is not a string, or a known frame whose fields do not fit. |
| `unknown_frame` | A frame kind this protocol does not have — including any message that asserts state. |
| `not_joined` | A request arrived before the connection had a seat. |
| `already_joined` | A second `join` on one connection. |
| `unknown_seat` | The world offers no such seat. |
| `seat_not_in_world` | The roster names an entity this world does not have: a fault in the world's composition, not in the frame. |
| `actor_not_observer` | The request asked the world to act as somebody else. |
| `dispatch_failed` | A system broke its own contract while resolving. The request has no answer; nothing about the frame was wrong. |
| `world_stopped` | The world is no longer running. |

A refusal is **not** a `Rejection`. A rejection means the world considered a well-formed request and
said no, and it arrives inside a `result`. A refusal means the frame was not a request at all.

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

**Identities are decimal strings.** `EntityId`, `EventId`, `ActionId` and `ProcessId` are written
as `"9007199254740995"`, in every position they appear, including inside a component or event
payload. `mineworld-contracts` does this itself, keyed on whether the format is human-readable; the
transport adds nothing. The reason is measured rather than theoretical: a JSON parser whose only
number type is a double turns `9007199254740995` and `9007199254740997` into the same value, so two
entities become one (`spike/FINDINGS.md` F1, F2).

A client must parse an id as a **string** and keep it as one, or as a 64-bit integer. Parsing it as
a number is the silent corruption above. Deserialization also accepts a JSON number, so a
hand-written fixture keeps working; a client should still send strings.

**Every other numeric field is an integer**, and the contract refuses a float where one is declared:
positions are millimetres, angles are millidegrees, times are whole seconds. A client whose language
has one number type must round and cast before encoding — GDScript's `1500.0` is refused with
`invalid type: floating point 1500.0, expected i32` (`FINDINGS.md` F9). That is the contract working:
a loud refusal rather than a silent truncation.

## 8. Rates, and what a client must not conclude from them

```text
observation cadence   the server's, configurable, 10 Hz by default
world clock           whole simulated seconds; several frames share one instant
rendering             the client's own business entirely
```

`NETWORKING.md` §4 forbids conflating these. An observation is not a tick, a tick is not a frame,
and nothing in this protocol replicates at a rendering rate. A client that stops reading loses
observations and the world does not wait for it: the server drops frames for a slow client rather
than blocking the world, which is why `seq` may jump for a client that was not keeping up and why
a client must render the newest observation it has rather than accumulating them.

## 9. What is not in revision 1

```text
authentication          NETWORKING.md §9 scopes the MVP to an invite token plus a nickname; this
                        revision has the seat name and no token
unsubscribing / scoping a client receives its observer's observations for the life of the
                        connection
state deltas            every observation is whole. NETWORKING.md §5 anticipates deltas; a whole
                        observation is correct and a delta is an optimization, in that order
admin commands          NETWORKING.md §5's admin class has no frames yet
binary encodings        JSON only, and DEP-3 records that a compact encoding is replaceable behind
                        this same boundary
```

A future revision raises `protocol`. A client that does not recognize the number should refuse to
connect rather than guess.
