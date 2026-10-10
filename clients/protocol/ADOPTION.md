# Adopting the MineWorld client protocol module

**Audience:** whoever is writing a MineWorld client in Godot — the 2D reference client, the 3D
reference client, or a third one. This is a specification, not an introduction;
[`README.md`](README.md) is the orientation.
**Authority:** [`server/PROTOCOL.md`](../../server/PROTOCOL.md) specifies the frames and governs where
the two disagree. This document specifies the module that implements them and the obligations a client
takes on by using it.

---

## 1. What to take, and where to put it

The module is the folder `mineworld/`, and nothing else in this directory. It has three files, no
dependencies, no autoloads, no project settings and no assets.

Two supported ways, and the cost of each:

```text
copy      cp -R clients/protocol/mineworld <your project>/mineworld
          Simple, works everywhere, and diverges the day somebody edits one copy. Re-copy when this
          module changes; the version you have is whatever you last copied.

symlink   ln -s ../../clients/protocol/mineworld <your project>/mineworld
          One source of truth inside one repository, and Godot follows it. Does not survive being
          exported or copied out of the repository on its own.
```

Either way the scripts sit at `res://mineworld/` in your project and the three classes are available
by name: `MineWorldClient`, `MineWorldObservation`, `MineWorldSpace`. A different directory name is
fine — nothing in the module refers to its own path.

Other shared client modules live in `clients/shared/` and are taken the same way; see
[`clients/shared/SETTINGS.md`](../shared/SETTINGS.md).

**A project must be imported once before a headless run.** Godot registers `class_name` in a cache it
builds when it imports a project, so a first `godot --headless --path <project>` may fail to find
`MineWorldClient`. `godot --headless --path <project> --import` builds it. A windowed run through the
editor does it for you. The cache belongs in `.gitignore` (`.godot/`), because a clean checkout must
build its own (`docs/ACCEPTANCE.md` §4.1).

## 2. The whole API

### `MineWorldClient` — a `Node`; add it to your scene

```text
connect_to_world(address: String, seat_name: String,      open a connection and ask for a seat,
                 invite: String, nickname: String,        presenting the server's invite and this
                 take_over := false)                      player's nickname (PROTOCOL.md §4.1);
                                                          take_over takes the seat from another
                                                          connection or a dropped one's hold
                                                          (§4.2). A seat an in-server controller
                                                          drives needs no flag
submit(action_type, target, payload, actor_location)      ask the world for something; returns a token.
                                                          `payload` is any JSON value, normally a
                                                          Dictionary
submit_affordance(affordance, actor_location := null)     submit a complete affordance exactly as
                                                          offered; returns a token, or "" and sends
                                                          nothing when it is not complete
perceive_from(cursor)                                     opt-in, before connect_to_world: also receive
                                                          the reliable `perceived` stream of the facts
                                                          this observer learns (PROTOCOL.md §5.8), from
                                                          `cursor` — null for "from this world's first
                                                          fact" (a world with a save), or a `through`
                                                          kept from before. Every rejoin, reconnect's
                                                          included, continues from perceived_cursor
leave_world()                                             give the seat up and end the connection;
                                                          the server answers `closing` "left"
disconnect_from_world(reason := "…")                      drop the socket
is_seated() -> bool                                       whether requests may be submitted
world_instance() -> String                                which running world this is
```

```text
state      IDLE | OPENING | JOINING | SEATED | CLOSED | RECONNECTING
reconnect  opt-in, false by default. When the socket drops without a `closing`, rejoin the same seat
           with the stored resume at 1 s, 2 s, 4 s … while within hold_seconds of the drop, then once
           without it; emits reconnecting(attempt), then the usual welcomed (took_over "held" if the
           seat was still held). A `closing` or disconnect_from_world never triggers it
seat       the seat asked for
observer   the identity the SERVER resolved it to. A string. Empty until the welcome arrives.
world      the welcome's world summary: instance, at, time_scale, entities, systems, seats, clients,
           counters
latest     the newest MineWorldObservation, or null
sequence   its per-connection frame number, from 1
revision   the persisted revision `latest` was computed from: an int, or null for a world that is not
           persisted. Set from the welcome, then from every observation frame that is not stale
stale_observations  how many arrived out of order and were dropped. Normally 0.
nickname   this player's nickname as the server accepted it (trimmed). Nobody else is shown it.
session    which connection this is, an identity string, for the operator. Not a credential.
took_over  "none" | "hosted" | "held" | "connection": whether control of the Person changed hands —
           the seat was free, an in-server controller drove it, this player's dropped seat was
           resumed, or it was taken from another connection (PROTOCOL.md §4.2)
hold_seconds, resume   the seat's hold after a dropped socket, in wall seconds, and the secret that
           re-takes it (fresh on every welcome). The module never prints, logs or emits the resume
close_reason  the reason of the server's last `closing`, or ""
perceived_cursor   the `through` of the last perceived frame: the cursor a rejoin continues from
deltas_applied, deltas_refused   how many `delta` frames were applied to the observation held, and
           how many could not be (each one drops the connection; with reconnect on it resumes, which
           yields a whole observation). For a status line or a test; act on neither
paused, time_scale   from the newest `clock` frame (PROTOCOL.md §5.9): whether the host has paused
           the world's clock, and world seconds per wall second. While paused, a submit is refused
           `paused`; observations keep arriving
```

The invite is required by every server, loopback included. A server started without `--invite` (or
`MINEWORLD_INVITE`) prints `[mineworld] invite <token> — join with: <address> seat=<seat>
invite=<token>`; a launcher reads the token after `invite ` on that line. The module never prints,
logs or emits the invite.

```text
welcomed(seat, observer, world)             the connection has a seat
observed(observation)                        a fresh view, for this observer only — always whole:
                                             the module applies `delta` frames itself
                                             (mineworld/delta.gd, PROTOCOL.md §5.3), so a client
                                             never sees one
perceived(events, through)                   with perceive_from: facts this observer learned, oldest
                                             first, never dropped or repeated; `through` is the
                                             cursor (PROTOCOL.md §5.8)
resolved(token, action_id, result)           the world's answer to one request
refused(code, token, detail)                 the frame was not accepted; nothing happened. A wrong
                                             invite is `unauthorized`, another revision is
                                             `protocol_mismatch` — each followed by `closing`
closing(reason, detail)                      the server is about to close the connection, and why:
                                             left, unauthorized, protocol_mismatch, world_stopped,
                                             taken_over, superseded, kicked (the operator removed
                                             this connection or released its seat), …
clock_changed(at, time_scale, paused)        how the host paces the clock: once right after the
                                             welcome, then on every pause and resume. Only the
                                             holder of the server's admin token changes it, over
                                             HTTP (PROTOCOL.md §11); this module has no call for it
disconnected(reason)                         the connection ended or could not be made
reconnecting(attempt)                        with reconnect on: the socket dropped and attempt N
                                             (from 1) is about to be made
submitted_request(token, request)            what this client just sent, for a log or a transcript
```

`address` may be `host:port`, `ws://host:port` or a full `ws://host:port/ws`. The module appends the
route.

### `MineWorldObservation` — a reader, never a model

```text
observer() at() self_location() place()
entities() ids() entity(id) tagged(tag) location_of(id)
component(id, component_type) component_value(id, component_type)
own_component(component_type) display_name(id)
relations() events()
affordances(action_type := "", target = null) complete_affordances(action_type := "", target = null)
affordances_about(id) is_complete(affordance)   (static)
affordance(action_type, target := "") may(action_type, target := "")
unavailable_reason(action_type, target := "") requirement(action_type, target := "")
offered_against(target := "")
acted_through()          the newest request this connection submitted that this observation already
                         reflects — the action_id its `resolved` carried — or null (PROTOCOL.md §5.2)
events_of(event_type)    the facts in events() of one event type, oldest first
frame        the dictionary exactly as it arrived (always a whole observation)
```

**Facts in an observation.** `events()` is what this observer learned since its previous frame,
oldest first, each a `PerceivedEvent` dictionary — judged by the server, never filtered here. It is
best effort: a client that falls behind may miss some (counted in the server's `events_dropped`). A
client that must not miss one asks for the `perceived` stream with `perceive_from`. An `event_type` a
client does not know is "something happened", never an error.

**Predicting your own requests.** A client that moves its body before the server answers reconciles
with an observation only once `acted_through()` has reached the `action_id` of the request it
predicted: earlier observations do not yet reflect it.

`may()` **reports** the server's verdict. It does not compute one, and neither may you.

**Listing affordances.** `affordances()` with no argument is the whole list, in the server's order.
`action_type` narrows it to one type (`""` matches any); `target` narrows it to one target — `null`
matches any target, `""` matches the affordances directed at nobody, and an identity string matches
exactly that target, as every other method here spells it. `affordances_about(id)` is every affordance
that concerns one entity: its `target` is `id`, or its `payload` is an object with a top-level value
that refers to `id` — the identity string itself, or the contract's typed reference
`{ "entity": id, "entity_type": … }`, which is how a typed identity such as an item's travels
(`buy`'s payload is `{ "item": { "entity": "22", "entity_type": "item" } }`). That is how a
target-less `buy { item }` or `kick { object }` is found from the thing it is about, without the
module or the client knowing either action. Nothing deeper is searched.

**Complete affordances.** An affordance may carry a `payload` key: the complete request the offering
system would accept (`PROTOCOL.md` §5, `docs/DECISIONS.md` `ARC-34`). `is_complete(affordance)` says
whether the key is present — a present `null` is a complete affordance whose payload is `null`;
absent means incomplete. `complete_affordances(…)` is the complete subset of `affordances(…)`.

Several complete affordances routinely share an action type and a target, one per choice offered, and
differ only in `payload`: keep them apart by their position in the list, never by looking one up.
`affordance(action_type, target)` returns only the **first** match, so it is for incomplete
affordances, where type and target name one entry.

Submit a complete affordance with `submit_affordance(affordance)`: it sends the affordance's
`action_type`, `target` and `payload` exactly as offered, labelled like any other request, without
anybody knowing what the action is. It does this **whether or not the affordance is available** — the
server answers, and an unavailable one comes back rejected with its reason. "Unchanged" means the JSON
the server sent: Godot parses every number as a double, so an offered `"count": 1` arrives as `1.0`,
and `submit_affordance` sends every whole number in the payload as the integer it was on the wire —
re-sending `1.0` is refused (§3.2). Never rebuild a complete affordance's request with `submit` for
that reason. It refuses, returning `""`
and sending nothing, only an affordance that is not complete: there is no payload to send unchanged,
and inventing one would be the client knowing the action.

So a client shows two kinds of entry. A complete affordance it shows and submits unchanged, knowing
nothing about the action. An incomplete one — `talk`'s utterance, `move`'s position — it submits only
by composing the payload from the player's input, which means knowing how to *ask* for that action
type, never whether it is allowed. An offered incomplete affordance the client has no composer for is
shown as one it cannot perform, never guessed at.

**Disclosed components.** `component()` returns a payload only when it is a JSON object, and `{}`
otherwise. `component_value()` returns the payload exactly as it arrived — an object, an array, a
number — or `null` when that component was not disclosed about that entity; read a listing that is
not an object through it.

`display_name(id)` is what a person is called: the `naming` System Pack's `display-name` record,
payload `{ "name": "Alice Moreau" }`, disclosed about everybody the observer perceives
(`docs/DECISIONS.md` `ARC-31`). It answers `""` when the world told this observer no name — show
something else the frame carries then, such as the id, and never a name of your own making.

### `MineWorldSpace` — static, and the only place the axes are converted

```text
to_2d(local) -> Vector2          to_3d(local) -> Vector3       metres, in Godot's frame
yaw_to_2d_radians(facing)        yaw_to_3d_radians(facing)     pitch_to_radians(facing)
from_2d(point, height := 0.0)    from_3d(point)                integer millimetres, world frame
yaw_from_2d_radians(radians)     yaw_from_3d_radians(radians)  canonical millidegrees
location(place, local := null, facing_mdeg := null)            a Location as the protocol carries one
millimetres(metres) millidegrees(degrees)                      the rounding, for your own payloads
```

The frame is `docs/CORE_CONCEPTS.md` §6.1: `+x` east, `+y` north, `+z` up, right-handed, yaw measured
from `+y` toward `+x`. The renderer spike wrote this conversion twice and got two different sign
conventions (`spike/FINDINGS.md` F5); do not write a third.

## 3. The four rules a client must not break

### 3.1 An identity is a string

Keep every `EntityId`, `EventId`, `ActionId` and `ProcessId` as the string it arrives as. Godot's only
number type is a double, so `9007199254740995` and `9007199254740997` parse to the *same* value: two
entities silently become one, and a client raycasts the mug and talks to Alice. That was measured, not
imagined (`spike/FINDINGS.md` F1, F2; `PROTOCOL.md` §7).

Use them as dictionary keys, as node metadata (`set_meta`) and as labels. Never `int()` one, never
compare one numerically, never round-trip one through a float.

### 3.2 Every other number is an integer

Positions are millimetres, angles are millidegrees, times are whole seconds, and the contract refuses
a float where an integer is declared — `1500.0` comes back as *invalid type: floating point 1500.0,
expected i32*. `MineWorldSpace` rounds and casts what it produces; for your own payload fields, call
`MineWorldSpace.millimetres()` or `int(round(...))` yourself (`FINDINGS.md` F9).

That refusal is the contract working. It is loud, which is the opposite of §3.1's failure.

### 3.3 No world rule in the client

The server decides distance, availability, permission, willingness and whether a system provides an
action at all. A client detects *"the player pressed interact while targeting Alice"* and submits;
the server answers with the resolved action or with a rejection naming the reason
(`docs/ENGINEERING_RULES.md` §§4, 7-9).

So:

```text
allowed      show a prompt when may("talk", alice) is true, and grey it out when it is false
allowed      show "too far away" from unavailable_reason(), in your own wording and language
allowed      show "needs 3 m" from requirement().within_range, unevaluated
allowed      submit a complete affordance unchanged, available or not
NOT allowed  compare two positions and decide whether to submit
NOT allowed  keep a list of which actions exist in this world
NOT allowed  refuse to submit because you concluded it would fail
```

Submitting something the server refuses is correct behaviour. Deciding it yourself is the defect.

### 3.4 Render the newest observation, not a backlog

An observation is whole and self-contained, and the world never waits for a client: one that stops
reading loses frames. Draw `latest`; do not accumulate. A gap in `sequence` is not an error
(`PROTOCOL.md` §8). On the wire most frames are deltas against the previous one (`DEP-15`); the module
applies them and emits the whole observation, so this rule is unchanged — and a delta that does not
apply is never patched around: the module resumes and is sent a whole frame. Facts are different:
`events()` is a since-the-last-frame stream, so a client that draws only the newest frame and needs
every fact uses the `perceived` stream (§2), not `events()`.

## 4. What a client may decide for itself

Everything about appearance, and nothing about validity:

```text
wording and language for an action type and a rejection — the contract carries no display text
                                                           deliberately (DD-13)
a sprite, a mesh, a colour, a label — chosen from an entity's `tags`, which are world data
a camera, a layout, a scale, an animation, an input mapping
whether to show an unavailable action greyed out or hide it
prediction: moving a body locally before the server confirms it
```

On the last one, know what you are doing: an affordance is computed against the server's
*authoritative* position, so near a boundary a predicted body renders a verdict for a position it has
already left. The spike measured about 250 mm of it on localhost (`FINDINGS.md` F7). Show the server's
answer, not your guess at the next one.

### 4.1 Moving: submit `move`, and report often enough

Moving a body is prediction; moving a *person* is a request. Submit `move` with the `Location` the body
walked to (`PROTOCOL.md` §6.2):

```gdscript
world.submit("move", null, { "to": MineWorldSpace.location(place, MineWorldSpace.from_2d(here)) })
```

The server accepts a move of at most **2 000 mm** from the position it last accepted, or through a
doorway the world declares, and answers anything else `too_far_away`. That makes one rule binding on
every client that moves continuously:

```text
report before your body has travelled 2 000 mm since the last ACCEPTED position
```

A walking or jogging 3D body must report several times a second, not once: at 2.6 m/s, once a second
is refused every time. A 2D client that walks to a click splits the walk into strides of at most
2 000 mm. Neither is a world rule in the client — the number is a request size, and a client that
ignores it is refused by the server, not by itself. On `too_far_away`, move the body back to the
position the next observation shows.

### 4.2 Walking somewhere: submit `walk-to`, then step once a wall second

To send a person somewhere — a click on the floor, "Walk to <name>" — do not plan a route and do not
split it into strides. Ask the server, which plans it round the world's walls, furniture and objects
(`DECISIONS.md` `ARC-75`):

```gdscript
world.submit("walk-to", null, { "to": { "place": MineWorldSpace.location(place, MineWorldSpace.from_2d(here)) } })
world.submit("walk-to", null, { "to": { "person": person_ref } })
```

A refusal is the server's answer: `too_far_away` for a person in another place, and the code `no-route`
for a destination no way leads to. Once accepted, the walk is the player's own `walking` record in the
next observation — its destination and the next waypoints — and it moves nobody by itself. While that
record is disclosed, send one `walk-step` (no payload) per wall second:

```text
while your own `walking` record is disclosed: one walk-step every 1 s of wall time
```

Each step carries the person at most 1 340 mm along the route — 1.34 m/s, a person's walking pace — and
the server answers it like a `move`. The timer is pacing, the same kind of rule as the `move` reporting
rule above: it is never a rule about the world, and the server neither knows nor checks how often you
send. Stop when the record is gone (the walk ended: arrived, stalled, no route, replaced or stopped). A
`move` of your own — WASD — ends the walk. Draw the record if you like (a route line, a heading); never
compute one.

## 5. The shape of a client, as this module expects it

```text
_ready        add a MineWorldClient, connect the signals,
              connect_to_world(address, seat, invite, nickname)
welcomed      you now have `observer`; build whatever needs to exist once
observed      reconcile your scene with the frame: add what is new, update what moved, remove what
              is no longer listed — an observation is exhaustive, so "not listed" means "not
              perceived", not "unchanged"
input         acquire intent — a click, a raycast, a key — and call submit()
resolved      show the answer; find later facts caused by your request by matching this action_id
refused       branch on `code`, never on `detail`
```

`submit` fills in the actor, writes `action_type` in both of the places the contract requires it, and
sends no `action_id` and no `issued_at` — the server allocates identity and the instant, and a client
that invented an `ActionId` would collide with the other client on its first action (`INV-6`,
`FINDINGS.md` F4).

## 6. What this module does not do yet

```text
remembering facts    the `perceived` stream delivers them and perceived_cursor resumes it; keeping
                      them (a memory, a transcript) across process restarts is yours — store the
                      cursor with them and pass it to perceive_from next time
admin calls           pausing, resuming, kicking and releasing are the host's, over HTTP with the
                      admin token (PROTOCOL.md §11); a launcher that holds one makes those calls
                      itself. The module only reads the `clock` frame
prediction, smoothing, interpolation    yours, and deliberately not here
a scene graph          yours entirely: this module has no opinion about how a world looks
```

### 6.1 Reconnecting: the pattern, as guidance

By default the module stays policy-free: a dropped connection ends with `disconnected`, and
`connect_to_world` accepts a call from `CLOSED`, so a client reconnects with the API it already has.
A client that sets `reconnect = true` (§2) instead gets the module's own short policy — rejoin with
the stored `resume` within the server's hold, then once without it — and still owns everything below
once that gives up. What a client's own policy must get right, as the 2D reference client does it
(`clients/2d/scripts/link.gd`):

```text
1  on disconnected    keep showing the last observation, marked stale; retry with capped back-off
                      (0.5 s doubling to 8 s), indefinitely while the player waits
2  re-join            the SAME seat, with the same invite and nickname
3  on welcomed        compare world.instance with the previous one:
                        same       the world went on; keep what you learned about its layout; the
                                   revision may be higher
                        different  the world was replaced; drop every drawn entity and every cache
4  refused            final for that attempt, shown by code — except world_stopped, which retries
5  never replay       a walk in progress is abandoned at disconnect; a request whose result never
                      arrived is reported "unknown — check the world", not resubmitted: the server
                      allocates identity, so a retry could do the thing twice (INV-6)
```

Point 5 is the one that matters most. Reconnecting is safe because a client's whole state is
replaced by the next observation (§3.4); resubmitting is not, because the world may already have
done what was asked.

A future protocol revision raises `MineWorldClient.PROTOCOL`. This module refuses to continue when a
server answers with a number it does not recognize, rather than guessing at a frame it cannot read.
