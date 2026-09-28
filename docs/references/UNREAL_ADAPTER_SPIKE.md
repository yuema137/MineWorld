# Unreal adapter spike — phase one

**Status:** informational. **This document is not authoritative architecture.** It records what an
Unreal client would cost, what the architecture test found, and what Epic's terms say. It changes no
MineWorld contract, and nothing in it overrides [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md),
[`ARCHITECTURE.md`](../ARCHITECTURE.md), [`ENGINEERING_RULES.md`](../ENGINEERING_RULES.md),
[`ENGINEERING_STANDARDS.md`](../ENGINEERING_STANDARDS.md), [`NETWORKING.md`](../NETWORKING.md),
[`VISUAL_FIDELITY.md`](../VISUAL_FIDELITY.md) or [`DECISIONS.md`](../DECISIONS.md). Where this
document and a specification disagree, the specification governs and this file is the defect.

**Audience:** coding agents. Vocabulary is [`CORE_CONCEPTS.md`](../CORE_CONCEPTS.md)'s throughout:
`Entity`, `Person`, `Place`, `Item`, `Relation`, `Process`, `ActionIntent`, `Event`, `System`,
`Controller`, `World`, `Component`, `Observation`, `Kernel`, and the pack types of
[`MODULE_SPEC.md`](../MODULE_SPEC.md). Unreal's own terms are used only when they are Unreal's
(`FRotator`, `ACharacter`, `IWebSocket`, Unreal Unit) and never as MineWorld terms.

| | |
| --- | --- |
| **Authorised by** | [`DECISIONS.md`](../DECISIONS.md) `ARC-18` |
| **Repository state audited at** | `7680290` (`main`) |
| **Phase** | one of two. Phase one is everything that does not need the engine. |
| **Engine installed** | **No.** Unreal is not installed on the machine this was run on, and installing it needs an interactive Epic account login and a very large download. Nothing in this document was verified inside Unreal. |

---

## 1. Why phase one exists at all, and what it is for

`ARC-18` authorises a time-boxed Unreal spike and states plainly which half of it matters:

> **A second renderer is an additional Presentation Adapter**, never a replacement, and this is an
> architecture test as much as a visual one. Presentation independence is a claim this project
> makes; if adding a high-fidelity client requires changes to kernel or System contracts, the claim
> is false and we want to discover that now, while the kernel is small.

and closes:

> **If the spike requires a contract change to proceed, stop and report it.** That result is more
> valuable than the slice.

The architecture test does not need Unreal. It needs the protocol, the contracts, and a client that
is emphatically not the Godot client. That is what §2 and §3 do, and it is why phase one was run
first: if the answer had been *"a contract must change"*, the install would have been the wrong next
step.

The engine is needed for the visual half — the reference-quality slice `ARC-18` §3 describes — and
for nothing in §§2–5 of this document.

---

## 2. The architecture test, part one: what a non-Godot client must implement

### 2.1 Method

Two passes, and neither reads prose for its answer.

**Derived from source.** `server/src/app.rs` for the routes, `server/src/protocol.rs` for the frame
types, `contracts/src/observation.rs`, `contracts/src/action.rs`, `contracts/src/spatial.rs` and
`contracts/src/ids.rs` for the payloads, and `clients/protocol/mineworld/*.gd` for what the one
existing client binding chose to do about each.

**Measured by building one.** [`spike/unreal/probe.py`](../../spike/unreal/probe.py) is a MineWorld
client with no game engine, no WebSocket library, no SDK and no generated stub — the Python standard
library and RFC 6455 spoken by hand. It joins `worlds/social-cafe`, reads an `Observation`, walks up
to Alice by submitting `arrive`, watches the server recompute the `talk` affordance, is accepted, and
then sends six deliberately wrong frames to record the answers. The captured run is
[`spike/unreal/evidence/transcript.log`](../../spike/unreal/evidence/transcript.log).

The measurement is the point: **whatever that file needs is the irreducible cost of a new client, and
whatever it does not need is not part of that cost.** A plan derived only from reading would report
whatever the reader found interesting.

### 2.2 The whole obligation, in eight items

**1. Three HTTP routes and one upgrade** (`server/src/app.rs:42-44`).

```text
GET /health   liveness of the process, answered by the transport
GET /status   what the world is, answered by the world
GET /ws       upgrade to the live connection
```

A client needs `/ws`. `/status` is how it discovers the seats a world offers before joining, and is
the same value the `welcome` frame carries. Captured:
[`spike/unreal/evidence/status.json`](../../spike/unreal/evidence/status.json).

**2. A WebSocket client speaking JSON text frames.** `PROTOCOL.md` §1: *"Frames on the WebSocket are
JSON text. A binary frame is refused."* No subprotocol, no compression requirement, no authentication
in revision 1 (`PROTOCOL.md` §9).

**3. Two frames it may send, and no third** (`server/src/protocol.rs:177-178`, `ClientFrame`, tagged
`"t"`, `rename_all = "snake_case"`).

```json
{ "t": "join",   "seat": "visitor" }
{ "t": "submit", "token": "p1", "request": { … an ActionRequest … } }
```

**4. Four frames it must read** (`server/src/protocol.rs:236-237`, `ServerFrame`): `welcome`,
`observation`, `result`, `refused`. A frame of any other kind is from a future revision and is
ignored rather than guessed at.

**5. The handshake, and the version gate.** `join` → `welcome`. The `welcome` carries
`protocol`, and a client that does not recognise the number **refuses to continue** rather than
guessing at a frame it cannot read (`PROTOCOL.md` §9; `world_client.gd:256-264`). It also carries
`observer` — the `EntityId` the server resolved the seat to. **No frame ever carries an observer id
from the client**, which is how `INV-13` is structural rather than checked (`PROTOCOL.md` §3).

**6. Reading an `Observation`.** Captured verbatim in the transcript; the shape is
`contracts/src/observation.rs`:

```text
observer         EntityId string
at               WorldTime, whole simulated seconds
self_location    Location, or null
entities[]       { id, entity_type, location, tags[], components[] }
relations[]      { relation_type, from, to }
events[]         empty in revision 1
affordances[]    { action_type, target, available, unavailable_reason, requirement }
```

An observation is **exhaustive**: not listed means not perceived, not unchanged. Reconciliation is
add / update / remove against the whole frame, every frame, and there is no second call that returns
more (`observation.gd:12-15`).

**7. Submitting an `ActionRequest`** (`PROTOCOL.md` §6). Four rules, all of which the probe obeys and
all of which the transcript demonstrates being enforced:

```text
no action_id, no issued_at     the server allocates identity and the instant (INV-6)
action_type twice, agreeing    in the envelope and inside payload
actor == this observer         anything else is refused actor_not_observer
actor_location is a report     a 3D client sends where it walked to; null is legal
```

**8. Nine refusal codes to branch on, never on `detail`** (`server/src/protocol.rs:285-286`,
`RefusalCode`): `malformed_frame`, `unknown_frame`, `not_joined`, `already_joined`, `unknown_seat`,
`seat_not_in_world`, `actor_not_observer`, `dispatch_failed`, `world_stopped`. A refusal is not a
`Rejection`: a refusal means the frame was not a request, a rejection means the world considered a
well-formed request and said no, and arrives inside a `result`.

Plus three disciplines that are not frames and are the ones a client gets wrong:

```text
identities are strings              §2.5 of this document
every other number is an integer    positions mm, angles mdeg, times whole seconds
render the newest observation       the world never waits; a backlog is the wrong thing to draw
no world rule in the client         the verdict is in the affordance; a distance check is a defect
```

### 2.3 What that cost actually measured

`spike/unreal/probe.py` is **230 lines of Python** including its module docstring, per-function
documentation, and the deliberate-failure suite that exists only to record error messages. The part
that is protocol — the `Socket` and `Connection` classes — is about 110 lines, of which roughly 50 is
RFC 6455 that Unreal supplies out of the box.

That number is the finding. Joining a MineWorld world, perceiving it, acting in it and being answered
is a small, flat, self-describing JSON conversation. It required no code generation, no schema, no
shared library, no MineWorld dependency, and no knowledge of Godot.

### 2.4 The full loop, from the captured run

The transcript is the evidence; these are the four lines that carry it.

The server resolves the seat and names the observer — a **string**:

```json
{"t":"welcome","protocol":1,"seat":"visitor","observer":"4","world":{…}}
```

Before walking, the server's own verdict on talking to the barista, sent unasked:

```json
{"action_type":"talk","target":"2","available":false,"unavailable_reason":"too_far_away",
 "requirement":{"place":"same_place_as_actor","within_range":3000,
                "requires_line_of_access":false,"requires_target_available":true}}
```

The client walks — in its own space — and reports where, and the server accepts:

```json
{"t":"result","token":"p1","action_id":"1","result":{"accepted":{"events":["5"]}}}
```

The next observation carries the recomputed verdict, and `talk` is then accepted:

```json
{"action_type":"talk","target":"2","available":true,"unavailable_reason":null, …}
{"t":"result","token":"p2","action_id":"2","result":{"accepted":{"events":["6","7"]}}}
```

Nothing in that exchange is Godot, an engine, a renderer or a dimension. The client never measured the
3000 mm; it was told the answer twice, once before it moved and once after.

### 2.5 The six refusals, because an author needs to see them

From the same transcript. Five are a client getting it wrong; the sixth is the authority model.

| What was sent | What came back |
| --- | --- |
| `1500.0` in `actor_location` | `refused` `malformed_frame` — *invalid type: floating point `1500.0`, expected i32* |
| `1500.0` inside a System payload | `result` `rejected` `{"system":{"code":"malformed-payload","detail":"a arrive action payload cannot be read as arrive"}}` |
| unquoted `"actor":5, "target":2` | accepted and resolved — a JSON number is still read as an id on the way in |
| `"actor":"2"` (somebody else) | `refused` `actor_not_observer` |
| envelope `talk`, payload `arrive` | `refused` `malformed_frame` — *an action intent for talk cannot carry a arrive payload* |
| `{"t":"set_state","money":5000}` | `refused` `unknown_frame` — *this protocol has no frame of kind "set_state"; a client may only join or submit* |

Two of those rows matter to an Unreal author specifically and are taken up in §3.4 and §5.2.

---

## 3. The architecture test, part two: is any of it Godot-shaped?

### 3.1 Verdict

**No. After looking in the three places it could hide, I found no Godot concept in the wire format,
in the contracts, or in the kernel.** Three notes below are consequences of Godot being the only
existing client, and none of them is leakage: one is a correct generalisation, one is a per-language
duplication cost, and one is a line in a repository-structure list.

### 3.2 Where I looked, and what was there

**The wire format.** `server/src/protocol.rs` in full: `ClientFrame` (`join`, `submit`),
`ServerFrame` (`welcome`, `observation`, `result`, `refused`), `RefusalCode`'s nine variants,
`WorldSummary`, `SystemSummary`, `WorldInstanceId`, `CorrelationToken`, `WirePayload`. Every field is
a world concept — a seat, an observer, a place, a tag, an affordance, a rejection, a world instance,
a system name — and the frames captured in the transcript contain nothing else. There is no field for
a mesh, a node, a scene, a camera, a viewport, a frame rate, a sprite or a screen coordinate, and no
field whose meaning depends on knowing which engine is reading it.

**The contracts and the kernel.** Grepped `contracts/src`, `kernel/src`, `server/src` and
`systems/*/src` for `mesh|navmesh|sprite|camera|animation|skeleton|texture|shader|pixel|viewport|
collider|physics`. **Seven occurrences, all of them in doc comments, all of them stating what is
excluded:**

```text
contracts/src/spatial.rs:7-8    "render-space position — one client's: a mesh transform, a navmesh
                                 point, an animation anchor, a camera"
contracts/src/spatial.rs:29-30  "No mesh, no navmesh, no collider, no camera, no scene node, no
                                 animation, no skeleton, no physics"
contracts/src/observation.rs:24 a 3D client shows a prompt over what the camera points at
contracts/src/observation.rs:56 how a client uses a tag is its own business
kernel/src/system.rs:4          "an enabled physics process" — CORE_CONCEPTS §13's phrase for a
                                 System, unrelated to a physics engine
```

**Zero in a type name, a field name, a variant or a function name.** The renderer vocabulary appears
in this codebase only in sentences that forbid it.

**Where Godot is named outside `clients/`.** Three places, all doc comments, all provenance:

```text
contracts/src/action.rs:28                  cites the spike's two Godot clients as where the
                                            missing-ActionId defect was measured
contracts/src/ids.rs:37                     cites the Godot client as where the 2^53 corruption
                                            was measured
tools/cli/tests/ac13_semantic_parity.rs:13  records that the fixtures were submitted by the real
                                            Godot client
```

A citation of where a measurement came from is not a dependency. Not one of them changes behaviour,
appears in a type, or would need editing for a second client to work.

### 3.3 Note one — the 64-bit identity rule is not a Godot rule, and the code says so

`clients/protocol/ADOPTION.md` §3.1 justifies string identities with *"Godot's only number type is a
double"*, which reads like a concession to one engine. The contract does not implement it that way.
`contracts/src/ids.rs:192-196` keys on the **serialization format**, not on the client:

```rust
fn serialize_opaque_id<S: Serializer>(raw: u64, serializer: S) -> Result<S::Ok, S::Error> {
    if serializer.is_human_readable() {
        serializer.collect_str(&raw)
    } else {
        serializer.serialize_u64(raw)
    }
}
```

`ids.rs:42-47` states the rule in full: human-readable formats (`serde_json`, YAML, TOML) get a
decimal string; binary formats (`bincode`, `postcard`) get the `u64` they always had. Persistence,
replay and any future binary encoding are untouched — which is exactly the objection `DD-15` raised
and this mechanism answers.

So the rule generalises correctly: **every** JSON client receives strings because JSON's number type
is the problem, not because Godot is. An Unreal client benefits from a decision that was measured in
Godot and implemented against the format. `ids.rs:55-66` also records why the rule lives on the type
rather than at the protocol boundary — a protocol layer cannot reach inside a `ComponentRecord` or an
`EventRecord`, which erase their contents by design and really do carry ids. That reasoning holds for
Unreal identically.

### 3.4 Note two — the one conversion is written once *per language*, not once

`docs/CORE_CONCEPTS.md` §6.1 exists because the renderer spike wrote the axis conversion twice and got
two different sign conventions (`spike/FINDINGS.md` F5). The fix was
`clients/protocol/mineworld/space.gd`, and `ADOPTION.md` §2 closes with: *"do not write a third."*

An Unreal client must write a third, because `space.gd` is GDScript. `ADOPTION.md` §1 is explicit that
the module is *"the folder `mineworld/`, and nothing else"* and is taken by copy or symlink into a
**Godot** project.

This is a real cost and it is **not** an architecture defect: a presentation adapter written in another
language necessarily reimplements its own binding, and that is what a presentation boundary means.
What it does mean is that the guarantee `space.gd` provides is per-language. The thing that actually
prevents divergence is the **normative statement** in `CORE_CONCEPTS.md` §6.1 plus the derivation in
§3.5 below — not the GDScript file.

Two mitigations, in preference order, neither of which changes a contract:

1. `CORE_CONCEPTS.md` §6.1's table is already sufficient to derive any engine's conversion — §3.5
   does exactly that for Unreal without looking at `space.gd`. Keeping the table normative and adding
   the worked Unreal row beside the existing Godot one costs one paragraph.
2. A conversion conformance check — a fixed list of `(LocalPosition, Orientation)` inputs and the
   expected engine-frame outputs — that each client binding runs. This is the mechanism that would
   actually catch a third sign flip, and it is client-side work in each client, not kernel work.

### 3.5 Note three, and the frame question: does the spatial frame survive a left-handed engine?

**Verdict: yes, and better than the Godot conversion suggests it would. The Unreal conversion needs
one axis swap, one scale factor and zero sign flips.** The reason is a property of how
`CORE_CONCEPTS.md` §6.1 is worded, and it is worth stating because it was not obviously deliberate.

The frame (`CORE_CONCEPTS.md` §6.1, lines 306–308):

```text
+x  east        +y  north        +z  up        right-handed
yaw  measured from +y toward +x, so 0 mdeg faces north and 90 000 mdeg faces east
pitch  positive looks up
```

Unreal's frame — **pending confirmation against Epic's documentation, see §3.6** — is left-handed,
`+Z` up, `+X` forward, `+Y` right, one Unreal Unit = 1 cm, and `FRotator::Yaw` rotates about `+Z`
from `+X` toward `+Y`.

Map Unreal's forward onto the world's north:

```text
Unreal.X  =  world.y / 10          north
Unreal.Y  =  world.x / 10          east
Unreal.Z  =  world.z / 10          up
Unreal.Yaw   (degrees) = world.yaw   / 1000
Unreal.Pitch (degrees) = world.pitch / 1000
```

and back:

```text
world.x = round(Unreal.Y * 10)     world.yaw   = posmod(round(Unreal.Yaw   * 1000), 360000)
world.y = round(Unreal.X * 10)     world.pitch =        round(Unreal.Pitch * 1000)
world.z = round(Unreal.Z * 10)
```

Three things to notice.

**The axis swap is what carries the handedness change.** Exchanging two basis vectors reverses
orientation, so a right-handed `(east, north, up)` becomes a left-handed `(north, east, up)`. Nothing
is mirrored and nothing is negated; the chirality difference between the two engines is absorbed
exactly by the swap, which is the correct and only place for it.

**The scale factor is a clean factor of ten.** The world speaks integer millimetres and one Unreal
Unit is one centimetre, so `uu = mm / 10`. Godot, being metric, divides by 1000. Neither engine needs
a non-uniform import scale, which `DEP-8`'s coherence procedure warns quietly breaks physics, shadow
bias and LOD distances.

**Yaw needs no sign flip, and this is the interesting result.** `space.gd:77-78` has a minus sign the
spike had to discover by turning a character around and looking at it. Unreal has none. The reason is
that §6.1 defines yaw by **naming its two reference axes** — *"measured from +y toward +x"* — rather
than by naming a rotation sense about the vertical, such as *"counter-clockwise about +z"*. An
axis-named definition carries no handedness of its own: "start at north, turn toward east" is the same
physical instruction in a left-handed engine and a right-handed one. A sense-named definition would
have forced every reader to combine it with the handedness statement and get the combination right,
which is precisely the step the spike got wrong twice.

Concretely, MineWorld yaw is a **compass bearing**: clockwise from north, viewed from above. In a
right-handed `+z`-up frame that is the *negative* of the conventional mathematical angle, which is
where Godot's minus sign comes from; in a left-handed `+z`-up frame it is the positive one, which is
why Unreal has none. The spike client's own function name, `bearing_to_godot_yaw`
(`spike/client-3d/main.gd:76`), says the same thing.

Check it against §6.1's own worked example — *a person at `x: 2000, y: 0, z: 0` with `yaw: 180_000`
stands two metres east of the place origin, facing south*:

```text
Unreal position = (X 0, Y 200, Z 0) uu   = 2 m along +Y = 2 m east          ✓
Unreal FRotator(Pitch 0, Yaw 180, Roll 0).Vector() = (cos180, sin180, 0)
                                          = (−1, 0, 0) = −X = −north = south ✓
```

**So the frame does not reveal an assumption baked where it should not be.** It reveals the opposite:
the one place the frame is defined is defined in a form that transfers. What it *does* reveal is that
the property is load-bearing and undocumented as such — §6.1 explains why the frame must be written
down, not why it is written in the axis-naming form. If a later edit "simplified" it to *"yaw is
counter-clockwise about +z"*, the statement would remain true for Godot and would silently become a
trap for any left-handed client.

### 3.6 What in §3.5 is not confirmed

Unreal is not installed, so **none of §3.5 was run**. The two halves have different standing and
should not be reported as one.

The MineWorld half — the frame, the units, the worked example — is confirmed from source and from
frames the running server actually sent.

The Unreal half — left-handedness, `+Z` up, `+X` forward, `+Y` right, 1 uu = 1 cm, `FRotator::Yaw`
turning `+X` toward `+Y`, `FRotator::Pitch` positive looking up — is confirmed against Epic's own
published documentation, quoted in §7.1, and **not** against a running engine. Documentation can be
stale or can describe an editor convention that a runtime API expresses differently, and the pitch
sign in particular is a single bit whose failure mode is a character that looks at the sky when it
should look at the pavement.

So the correct summary is: the derivation is sound and every input to it is cited, and it is
unexecuted. The conformance fixture in §3.4 mitigation 2 and §7.7 is where that is closed, and
running it is the first thing phase two should do after the client connects — before any scene work,
because a wrong frame discovered after a town is built is discovered in the worst possible place.

---

## 4. 64-bit identities and an Unreal client

### 4.1 What the wire already does

The wire format **already accommodates it, and did so before any non-Godot client existed.** Over
JSON every `EntityId`, `EventId`, `ActionId` and `ProcessId` is written as a decimal string, in every
position it appears — including inside a component or event payload, because the rule is on the type
(§3.3). From the captured frames:

```json
"observer":"4"            "target":"2"            "action_id":"2"
"events":["6","7"]        "place":{"entity":"1","entity_type":"place"}
"relations":[{"relation_type":"present-in","from":"2","to":"1"}]
```

Reading back accepts **both** a string and a JSON number (`ids.rs:223-233`: `visit_u64`, `visit_i64`,
`visit_str`), which is why the transcript's unquoted-id row was accepted. A **float** is refused
outright rather than truncated — an id that has already lost precision must fail where it is read
rather than resolve onto whichever entity it rounded to.

### 4.2 What an Unreal client must do

Unreal's JSON module has the same hazard as Godot's and for the same reason: a JSON number lands in a
`double`. The obligations are therefore identical to `ADOPTION.md` §3.1, transposed:

```text
DO      keep every identity as an FString, exactly as it arrived
DO      use it as a TMap key, an actor tag, a UPROPERTY FString, a label
DO      parse to uint64 only with an exact integer parse — FCString::Strtoui64 or
        LexFromString(uint64&, …) — and only when a uint64 is genuinely needed
DO      send it back as a JSON string

NEVER   FJsonValue::AsNumber(), GetNumberField() or TryGetNumberField() on an identity
NEVER   compare two identities numerically, or round-trip one through float/double
NEVER   store one in a float, a double, an int32, or a Blueprint Integer (int32)
```

The Blueprint line deserves emphasis because it is an Unreal-specific trap with no Godot equivalent:
Blueprint's `Integer` is `int32` and its `Integer64` is `int64`, and a designer wiring an entity id
through a Blueprint variable will reach for the wrong one. Keeping identities as `FString` end to end
— never exposing a numeric identity type to Blueprint at all — removes the trap rather than
documenting it.

### 4.3 What was and was not demonstrated

`worlds/social-cafe` allocates ids `1` through `5`, so the captured run does **not** exercise a value
above 2^53 and is not evidence that the discipline works — only that the encoding is string-shaped.
The measurement that matters was made in the renderer spike (`spike/FINDINGS.md` F1, F2:
`9007199254740995` and `9007199254740997` both arriving as `9007199254740996`) and is what the
contract's encoding answers. An Unreal client should carry the same pair as a fixture in its own
conformance check (§3.4 mitigation 2), because a world small enough to debug in is exactly a world
that will never catch this.

---

## 5. Four defects found while running the test

None blocks an Unreal client. All four are recorded here rather than fixed, because this spike is
chartered to report and not to amend merged specifications (`ARC-18`: *"No kernel, contract, World
Pack, persistence or networking change"*). `CLAUDE.md` §2.1 rule 4 requires the contradiction to be
recorded rather than left unrecorded, which is what this section is.

### 5.1 `docs/CORE_CONCEPTS.md` has two sections numbered `## 6.1`

```text
line 298   ## 6.1 The spatial frame, stated because two clients cannot guess it alike
line 316   ## 6.1 Location: semantic place, optionally refined
line 352   ## 6.2 SpatialRequirement: what an action needs of space
```

The spatial-frame section was inserted with a number the document already used. At least three files
cite `§6.1` and mean the frame — `space.gd:7`, `clients/protocol/ADOPTION.md` §2, and
`contracts/src/spatial.rs`'s neighbourhood of citations — so an author writing a new client binding
follows the citation and finds two candidate sections. Renumbering is a one-line docs change and is
not made here because it shifts a numbering other documents cite.

### 5.2 A float is refused loudly at the protocol boundary and quietly inside a System payload

`ADOPTION.md` §3.2 tells a client author:

> Positions are millimetres, angles are millidegrees, times are whole seconds, and the contract
> refuses a float where an integer is declared — `1500.0` comes back as *invalid type: floating point
> 1500.0, expected i32*. […] for your own payload fields, call `MineWorldSpace.millimetres()` or
> `int(round(...))` yourself.

The two halves of that paragraph do not behave the same way, which the transcript measured:

```text
1500.0 in actor_location  →  refused, malformed_frame,
                             "invalid type: floating point `1500.0`, expected i32"
1500.0 in a System payload →  rejected, {"system":{"code":"malformed-payload",
                             "detail":"a arrive action payload cannot be read as arrive"}}
```

The serde diagnostic is discarded at `systems/presence/src/codec.rs:37`:

```rust
serde_json::from_slice(payload).map_err(|_| ContractError::ActionTypeMismatch { … })
```

That discard is **deliberate and documented in place** (`codec.rs:30-34`: a System Pack has only the
contract layer's vocabulary to report in, and both failures are literally *"this payload cannot be
read as `A`"*). So the code is not the defect; `ADOPTION.md` §3.2 over-promises, and the same
over-promise is in `PROTOCOL.md` §7.

It matters here more than it did for Godot. `ADOPTION.md` §3.2 calls the refusal *"loud, which is the
opposite of §3.1's failure"* — and inside a payload it is not loud, it is a generic
`malformed-payload` that names neither the field nor the value. An author debugging a new client in a
new language will spend that difference.

### 5.3 There is no contract for where one Place is relative to another

`LocalPosition` is *"relative to that place's own origin"* (`contracts/src/spatial.rs:114-120`), and
place hierarchy is a `Relation` with no transform (`CORE_CONCEPTS.md` §6.1 line 349:
*"Hierarchy is a Relation, not a field"*). Nothing in `contracts/` or `kernel/` carries a
place-to-place offset, extent, footprint or world origin — grepped for all of those.

The contract is honest about the consequence and handles it correctly:
`SpatialRequirement::evaluate` never compares two `LocalPosition`s across places, and degenerates a
range requirement to *same place* when either side does not model continuous position
(`spatial.rs:495-511`, and the reasoning at `spatial.rs:450-461`).

So this is not a bug. It is a gap that a 3D client hits the moment a world has a second Place and the
player is meant to **walk** between them, which `CLAUDE.md` §1.1 says is the thing the project exists
to enable. With no place-to-place transform in the contract, each client invents one — and that is the
same failure mode `CORE_CONCEPTS.md` §6.1 was written to prevent for axes, one level up.

**It is engine-neutral.** Godot hits it identically. It is not an Unreal finding, it does not block
the Unreal spike, and `worlds/social-cafe` has one Place so nothing hits it today. It is recorded
because a spike whose brief is *"surface architectural holes"* (`CLAUDE.md` §4 rule 13) found one, and
because the natural time to decide it is alongside `MoveIntent` and the travel `Process`
(`ENGINEERING_RULES.md` §6), which do not exist yet.

### 5.4 One `ARC-19` consequence lands on a file `ARC-19` does not name

Found while writing §9, and flagged because it is directly load-bearing for it.

`ARC-19` supersedes `ARC-4`'s facial-fidelity exclusion for the default character and directs in
consequence 3 that mature character tools — *"MetaHuman, Character Creator, Blender with MPFB,
image-to-3D reconstruction, CC0 groom and animation libraries"* — be **evaluated**. Its consequence 1
updates `04_character_closeup.png`'s scoping and its consequence 2 names
[`ART_DIRECTION.md`](../ART_DIRECTION.md) §§3 and 7 as needing amendment. Neither has been applied
yet, which is expected: they are recorded work, not a hidden contradiction.

`presentation/mineworld-default/3D/references/README.md` is a third file carrying the superseded
statement, and `ARC-19` does not name it:

```text
line 29-30   "Its facial photorealism is explicitly not the fidelity target."
line 43      "it explicitly does not depend on photoreal skin, MetaHuman-level facial assets,
              facial scanning, cinematic hair simulation, or a custom character pipeline"
```

That line names MetaHuman by name and reads as a policy exclusion. A reader arriving at §9 of this
document from that README would conclude MetaHuman is ruled out on MineWorld policy grounds, when
`ARC-19` §3 in fact directs the opposite — evaluate it, and do not build a bespoke pipeline instead.
`presentation/mineworld-default/3D/manifest.yaml:52` still carries the superseded
`not_authoritative_for` list too, which `ARC-19` consequence 1 does cover.

Recorded rather than fixed: amending the default presentation pack's own art documentation is
outside a renderer spike's brief and belongs with whoever applies `ARC-19`'s other two consequences.

---

## 6. The verdict `ARC-18` asked for

> **No kernel change, no contract change, no System change, no World Pack change, no persistence
> change and no networking change is required to add an Unreal client.**

The protocol is engine-neutral in fact and not only in intention: a client with no game engine at all
joined the running world, perceived it, moved in it, was refused for distance, moved closer, and was
accepted — and the only thing it needed to know about Godot was nothing. The spatial frame transfers
to a left-handed, Z-up, centimetre-scaled engine with one axis swap and no sign flip, and the 64-bit
identity encoding already protects a client whose JSON parser has one number type, because the rule is
keyed on the format rather than on the engine.

Presentation independence, as `ARC-18` §2 poses it, holds.

The four items in §5 are documentation and forward-looking findings, and none of them is a contract.
§5.1 and §5.2 are a one-line and a one-paragraph correction to merged specifications. §5.4 is a
consequence of `ARC-19` landing on a file `ARC-19` does not name. §5.3 is a real design question —
there is no contract for where one `Place` sits relative to another — which belongs with movement and
travel, is engine-neutral, and should not be decided by a renderer spike.

---

## 7. The implementation plan

This section is the plan `ARC-18` scopes: *"Only client-side work is new: the protocol binding,
`Entity` → actor projection, input → `ActionIntent`, cameras, and the scene."* Each component below
names what it maps to on the MineWorld side, by file, so that the plan can be checked against the
repository rather than believed.

Nothing here has been compiled. Unreal is not installed (§1). Line counts are estimates derived from
the GDScript the same component occupies today and from the probe that measured the protocol.

### 7.1 The engine facts this plan rests on

Every fact below was taken from Epic's own documentation during this spike, with the quoted text
recorded. None was assumed, and none was verified inside a running engine, because Unreal is not
installed.

| Fact | Epic's words | Used by |
| --- | --- | --- |
| Left-handed, `+Z` up | *"In the Unreal Editor the coordinate system is left-handed and uses a Z-up axis."* | §3.5 |
| `+X` forward, `+Y` right, `+Z` up | *"Positive values are forward." / "Positive values are to the right." / "Positive values are upwards."* | §3.5, §7.4 |
| 1 Unreal Unit = 1 cm | *"In Unreal Engine, the primary unit of measurement is one centimeter."* | §3.5, §7.4 |
| `Yaw` is about `+Z`, positive `+X` → `+Y` | `TRotator`: *"Rotation around the up axis (around Z axis), Turning around (0=Forward, +Right, -Left)"*, and Epic's Verse reference: *"a positive angle indicating a clockwise rotation when viewed from above"* | §3.5, §7.4 |
| `Pitch` positive looks up | `TRotator`: *"Rotation around the right axis (around Y axis), Looking up and down (0=Straight Ahead, +Up, -Down)"* | §3.5, §7.4 |
| `IWebSocket` is a **first-party Runtime module**, not a plugin | Module `Runtime/WebSockets`; `Connect()` — *"Initiate a client connection to the server."* | §7.2 |
| A JSON number is stored as a `double` | `FJsonValueNumber` has *"protected `double Value`"*; `GetNumberField` returns `double` | §4.2, §7.3 |
| Epic ships a precision-preserving mode | `FJsonValueNumberString` — *"A Json Number Value, stored internally as a string so as not to lose precision"*, reached via `FJsonSerializer::EFlags::StoreNumbersAsStrings` | §7.3 |
| `FCString::Strtoui64` and `LexFromString(uint64&, …)` exist | `uint64 Strtoui64(const CharType* Start, CharType** End, int32 Base)` | §4.2 |
| Gameplay logic is C++ and Blueprint; Python is editor-only | *"the Python environment is only available in the Unreal Editor, not when your Project is running in the Unreal Engine in any mode, including Play In Editor, Standalone Game, cooked executable"* | §7.2, §8 |

**Every assumption §3.5 made about Unreal is confirmed, including the pitch sign that §3.6 flagged as
the one to check first.** The conversion in §3.5 therefore stands with zero sign flips on yaw and zero
on pitch, and §3.6's caveat is narrowed to: it has been confirmed against Epic's documentation but
not against a running engine, which remains phase two's job and is what the conformance fixture in
§7.7 is for.

Two facts not used by the plan above, because they change it:

**`IWebSocket`'s supported-platform list is not documented.** Epic says only that
`CreateWebSocket` *"Instantiates a new web socket for the current platform"*, and no Epic page
fetched states which platforms, or confirms behaviour in a packaged build. The module is
first-party and Runtime-category, which is strong circumstantial evidence, but it is not a
statement. Two *other* things are Experimental and must not be confused with it:
`WebSocketNetworking` (*"Experimental WebSocket Networking Plugin"*) and `WebSocketMessaging`
(under `Engine\Plugins\Experimental\`). The plan uses `Runtime/WebSockets`; verifying it in a
packaged build is a phase-two item and is priced as first-contact risk in §8.1.

**Runtime glTF import cannot carry a skeleton or animation** — see §7.8, which is the one place
this research changed the plan rather than confirming it.

### 7.2 Component one — the protocol binding

**What it is.** The Unreal equivalent of `clients/protocol/mineworld/`, and the only component that
speaks to the server. Three types, mirroring the three GDScript files, because that decomposition was
already validated against the real protocol and there is no reason to invent a second one.

```text
UMineWorldClient        a UGameInstanceSubsystem      ← clients/protocol/mineworld/world_client.gd
FMineWorldObservation   a reader over one frame       ← clients/protocol/mineworld/observation.gd
FMineWorldSpace         static conversions            ← clients/protocol/mineworld/space.gd
```

**What it talks to.** `server/src/app.rs`'s `/ws` route and `/status`; `server/src/protocol.rs`'s
`ClientFrame` and `ServerFrame`; `contracts/src/observation.rs`'s `Observation`,
`PerceivedEntity` and `Affordance`; `contracts/src/action.rs`'s `ActionRequest` and `ActionResult`.
The specification it implements is [`server/PROTOCOL.md`](../../server/PROTOCOL.md), which governs
where it and this document disagree.

**`UMineWorldClient`** — a `UGameInstanceSubsystem` rather than an `AActor`, because the connection
outlives any level and belongs to the session rather than to the scene. It owns:

```text
IWebSocket            connect, OnConnected, OnMessage, OnClosed, OnConnectionError
State                 Idle | Opening | Joining | Seated | Closed
Seat, Observer        the seat asked for; the EntityId string the SERVER resolved it to
World                 the welcome's world summary, instance included
Latest, Sequence      the newest FMineWorldObservation and its seq; a lower seq is dropped
Submit(...)           fills in actor, writes action_type twice, allocates a correlation token,
                      sends NO action_id and NO issued_at
```

and broadcasts five dynamic multicast delegates matching `world_client.gd`'s signals: `OnWelcomed`,
`OnObserved`, `OnResolved`, `OnRefused`, `OnDisconnected`. Blueprint-exposed, so scene and HUD work
needs no C++.

Two rules this class exists to make unbreakable, both measured in §2.5:

```text
the protocol version gate    a welcome whose `protocol` is not 1 closes the connection rather than
                             guessing at frames it cannot read
no observer in any frame     there is no API here that names an observer, sets a value, widens a
                             scope or asserts a fact, because there is no such frame
```

**`FMineWorldObservation`** — a reader over the `TSharedPtr<FJsonObject>` that arrived, holding the
frame and looking things up in it. Not a model, not a cache, and it decides nothing: the accessor
that matters is `May(ActionType, Target)`, which **reports** `affordances[].available` and never
computes it. A distance comparison anywhere in the Unreal project is a defect
(`ENGINEERING_RULES.md` §§8–9).

Its surface is `observation.gd`'s, transposed: `Observer`, `At`, `SelfLocation`, `Place`, `Entities`,
`Ids`, `Entity`, `Tagged`, `LocationOf`, `Component`, `OwnComponent`, `Relations`, `Events`,
`Affordances`, `Affordance`, `May`, `UnavailableReason`, `Requirement`, `OfferedAgainst`.

**`FMineWorldSpace`** — §3.5's conversion and nothing else. It is the third writing of a conversion
that `CORE_CONCEPTS.md` §6.1 exists to prevent diverging (§3.4), so it ships with the conformance
fixture §7.7 describes rather than with a comment asking the next author to be careful.

### 7.3 Identities, and the one thing to get right before anything else

Covered in full in §4. The plan-level consequence: **the protocol binding never exposes a numeric
identity type to the rest of the project, and never to Blueprint.** `FString` in, `FString` through,
`FString` out. `FMineWorldObservation` returns `FString`; actors carry their id in an `FString`
`UPROPERTY` and as an actor tag; the projection map is `TMap<FString, TWeakObjectPtr<AActor>>`.

There is no accessor anywhere that returns an `int32`, an `int64` or a `double` for an identity, so
`AsNumber()` is not reachable by accident and a Blueprint author cannot wire an id into an `Integer`
pin because no such pin is offered.

### 7.4 Component two — `Entity` → actor projection

**What it is.** The subsystem that turns each `Observation` into a scene. One function, run on
`OnObserved`, reconciling the actor set against the frame.

**What it talks to.** `contracts/src/observation.rs`'s `PerceivedEntity` — `id`, `entity_type`,
`location`, `tags`, `components` — and `contracts/src/spatial.rs`'s `Location` through
`FMineWorldSpace`.

```text
for each PerceivedEntity in the frame
    no actor yet   → spawn one chosen from `entity_type` and `tags`
    actor exists   → update its target transform from `location`
actor not in frame → despawn it
```

The last line is the one that is easy to get wrong and is not negotiable: **an observation is
exhaustive**, so an entity that is not listed was not perceived, and leaving its actor standing
renders a person who is not there.

**What chooses the mesh.** `tags`, and nothing else. `worlds/social-cafe` supplies
`["barista","staff"]`, `["regular"]`, `["visitor"]`, `["cafe","public"]`, and a client picking an
appearance from a tag is making an appearance decision from world data, which
`ADOPTION.md` §4 permits explicitly. A client keeping its own list of which entities exist, or which
actions a world has, is not permitted.

**Movement between frames.** Observations arrive at 10 Hz by default and the world clock is in whole
seconds (`PROTOCOL.md` §8). Other people's actors interpolate toward the position the last
observation gave them; they are not physics bodies and they do not predict. `NETWORKING.md` §4
forbids conflating the observation cadence, the tick and the frame rate, and the correct reading of
that rule here is that interpolation is presentation smoothing over authoritative samples and never a
second simulation.

**The player's own body** is the exception and is covered in §7.5.

### 7.5 Component three — input → `ActionIntent`

**What it is.** Acquisition, and nothing else. The client detects *"the player pressed interact while
looking at Alice"* and submits; the server answers.

**What it talks to.** `PROTOCOL.md` §6's `submit` frame; `systems/presence/src/action.rs`'s `arrive`
and `systems/conversation/src/action.rs`'s `talk`, which are the only two action types
`worlds/social-cafe` provides.

```text
Enhanced Input        Move, Look, Interact, ToggleCamera
ACharacter            the player's body, with UCharacterMovementComponent — local, never
                      authoritative (NETWORKING.md §4: "Local physics: never authoritative")
a camera trace        what the player is looking at; the hit actor's id tag is the target
UMineWorldClient      Submit("talk", Target, {utterance}, ActorLocation)
```

**Moving is submitting `arrive`.** This is worth stating because it is the single most surprising
thing about the current server for a client author, and it is not an Unreal issue. There is no
`MoveIntent` and no travel `Process` — `systems/presence/src/action.rs:17-22` says so in terms:
*"It is not movement. Movement is a `MoveIntent`, a travel `Process` that takes simulated time, and
authoritative state that changes as it runs, and none of those exist yet."* `arrive` *records* an
arrival something else decided. So the client walks its `ACharacter` locally, and submits `arrive`
with the `Location` it walked to, which is what the probe did and what
`spike/client-3d/main.gd` does.

The consequences for the plan, which phase two must not paper over:

```text
the authoritative position updates when the server accepts an `arrive`, not continuously
an affordance is computed against that accepted position, not against where the body is now
so a prediction near a range boundary renders a verdict for a position already left
```

`ADOPTION.md` §4 measured about 250 mm of that on localhost (`FINDINGS.md` F7). Show the server's
answer, not a guess at the next one.

**`actor_location` is a report.** A 3D client sends where it walked to; the server evaluates its own
authoritative state and is free to ignore it (`PROTOCOL.md` §6 rule 3, `ENGINEERING_RULES.md` §8).

**The parity obligation.** `docs/MVP.md` §9 `AC-13` requires a `talk` from a 2D click and a `talk`
from a 3D walk-up-look-at-press to share an identical semantic core — `actor`, `action_type`,
`target`, `payload` — with `actor_location` the only permitted difference. The comparison is
implemented once, in `server/src/parity.rs`. An Unreal client inherits that obligation unchanged and
does not reimplement the comparison.

### 7.6 Components four and five — cameras and the scene

**Cameras.** A `USpringArmComponent` plus a `UCameraComponent` for third person, and a camera at eye
height on the `ACharacter` for first person, toggled by an Enhanced Input action. This is what the
Third Person template already provides, and the camera is a pure presentation choice that the
contracts neither know nor can be told about (`ARCHITECTURE.md` §10).

The one thing the camera does that touches the protocol is the trace in §7.5, and what it produces is
a *target id*, never a verdict.

**The scene.** For the proof of concept, `worlds/social-cafe`'s one Place, built as geometry that
matches the `LocalPosition` extents the observations actually carry — the captured frame puts people
at `x` 1200–4600 mm and `y` 200–4400 mm, so the room is about 5 m × 5 m. `spike/client-3d/main.gd`
builds the equivalent out of slabs.

**The scene is not derived from the contracts, and cannot be.** There is no geometry, no extent and
no footprint in `contracts/src/spatial.rs` — deliberately (`ENGINEERING_RULES.md` §12). A Place's
geometry is Presentation Pack content keyed by the Place's authoring key, exactly as it is in Godot.
§5.3 records the related gap: with more than one Place there is also no contract for where one sits
relative to another.

### 7.7 What the plan adds that the Godot client does not have

One thing, and it is the mitigation §3.4 recommends rather than new scope:

**A frame conformance fixture.** A fixed table of `(LocalPosition, Orientation)` values and their
expected engine-frame results, checked by an automation test in each client binding. It costs a
table and a test, it is the only mechanism that would actually catch a fourth sign flip, and §3.5's
worked example plus §6.1's are its first two rows. It is client-side work in each client and changes
no contract.

### 7.8 The one finding that constrains the plan: Unreal cannot load a rigged character at runtime

This is the most consequential thing the engine research turned up, it was not anticipated, and it
bears on MineWorld's extension model rather than on this spike's slice.

Epic's runtime import path is Interchange, and Epic states it is runtime-capable:

> *"The Interchange Framework is Unreal Engine's import and export framework. It is file format
> agnostic, asynchronous, customizable, and can be used at runtime."*

and that glTF goes through it. But the runtime-import page states a limitation:

> *"This method of import currently does not support Skeletal Mesh or Animation data."*

So an Unreal client can load a glTF **static mesh** at runtime — props, buildings, street furniture
— and **cannot** load a glTF **rigged, animated character** at runtime. A character must be imported
in the editor and cooked into the build.

**Why that matters beyond this spike.** `CLAUDE.md` §1 fixes the extension model as
`install modules → compose world → configure → run`, never `fork source → edit game code`, and
`DEP-8`'s companion decision records glTF as the interchange format packs ship. A community
Presentation Pack supplying its own `Person` appearance is therefore loadable at runtime by a Godot
client and **not** by an Unreal client without a rebuild.

**What this is not.** It is not a contract change and not a reason to stop. `worlds/social-cafe` has
four people and the Unreal slice would cook them in, so nothing in phases one or two touches it. It
is also not necessarily permanent: Epic's limitation is phrased *"currently"*, and a third-party or
custom runtime glTF path is a thing that exists in the Unreal ecosystem.

**What it is** is an asymmetry between the two candidate reference clients on exactly the axis
`CLAUDE.md` says MineWorld is about, and it should be in front of the operator when `ARC-18`'s
outcome is decided rather than discovered afterwards. It also sharpens `ARC-19` consequence 4 —
*"engine-specific Presentation Packs stay isolated from portable MineWorld assets"* — from a
licensing rule into a technical one.

**Unverified counterpart.** I did not verify Godot's runtime glTF capability from a primary source
in this spike, so *"Godot can and Unreal cannot"* is stated as the thing to check, not as a finding.
The Unreal half is cited; the Godot half is not.

### 7.9 Running it on this machine

Recorded because `ARC-18`'s outcome depends partly on whether the install is worth doing, and the
machine that would do it is a known quantity.

```text
Apple M5 · macOS 26.2 · 24 GB unified memory · 551 GB free
```

Against Epic's published macOS requirements: memory is **above the 16 GB minimum and below the
32 GB recommendation**, and the chip is well past the M2 floor the rendering features need. Disk is
ample for an editor install; a from-source build plus derived data is larger and closer to the
margin.

What matters more than the numbers is the **support tier**, which Epic states plainly for macOS:

```text
Lumen, software ray tracing              Apple Silicon M1+          supported
Lumen, hardware ray tracing              Apple Silicon M2+          Experimental
Nanite and Virtual Shadow Maps           Apple Silicon M2+          Beta
```

So the two features most often cited as Unreal's rendering advantage are, on this machine, on
**Beta and Experimental code paths**. That does not make them unusable and it is not an argument
against the spike. It does mean a fidelity comparison run here is a comparison of *Godot on a
supported path* against *Unreal on a beta path*, which `ARC-20` §2's requirement to "compare like
with like" should be read against before `VIS-3D-AB-1` is judged.

Two further constraints from Epic's MetaHuman hardware page, relevant to §9: the stated GPU floor is
*"At least nVIDIA RTX 3070, AMD RX 6800 XT, or Apple M2 Ultra, with 8GB VRAM"* — an M2 **Ultra**, not
an M-series base part — and **MetaHuman Animator's markerless capture is Windows-only**.

---

## 8. What it would cost

Three tiers, separated because they differ by an order of magnitude and merging them is how a spike
produces a number nobody can act on. Confidence is stated per tier, and the thing that would change
the estimate is stated with it.

**A working day means one focused developer-day.** None of these numbers was validated by building
anything in Unreal, because Unreal is not installed.

### 8.1 Tier one — the minimum proof of concept

Connect to the running server, project the same Alice, move, first and third person,
server-authoritative.

| Work | Estimate |
| --- | --- |
| Project setup: Third Person C++ template, `WebSockets`, `Json`, `JsonUtilities` in `Build.cs` | 0.5 d |
| `UMineWorldClient`: socket lifecycle, frames, version gate, tokens, submit | 1.5 d |
| `FMineWorldObservation`: the reader | 1.0 d |
| `FMineWorldSpace`: §3.5's conversion and its conformance fixture | 0.5 d |
| `Entity` → actor projection and reconciliation | 1.0 d |
| Input → intent: Enhanced Input, camera trace, target resolution, submit | 1.0 d |
| Cameras: template spring arm, first-person toggle | 0.25 d |
| Scene: the 5 m room, a body for Alice, a HUD listing affordances and their verdicts | 0.75 d |
| Integration against the real server, and the debugging that only happens there | 1.0 d |
| **Total** | **7–8 working days, call it a week and a half** |

**Confidence: medium-high on the protocol half, medium on the Unreal half.** The protocol half is
measured rather than guessed — the probe is 230 lines including its documentation and its
deliberate-failure suite, and this document hands the next author the frame derivation, the identity
discipline and the six refusals, all of which are otherwise discovered by hitting them.

**What would change it.**

```text
DOWN   starting from the Third Person template and accepting the Mannequin removes most of
       the camera and scene rows
UP     +1–2 d if IWebSocket's connection lifecycle, threading or packaged-build SSL misbehaves;
       this is first-contact risk I cannot price without the engine
UP     ×2–3 if the developer is learning Unreal C++. The build system, module dependencies,
       UObject lifecycle and reflection macros are a real curve and are nothing to do with
       MineWorld
```

### 8.2 Tier two — functional parity with the current Godot 3D demo

The baseline is `spike/client-3d/main.gd` (578 lines): a collidable room, an `ACharacter`-equivalent
with gravity, an eye-height camera, raycast targeting that recovers an entity id from a collider,
a HUD prompt and readout, `talk` and `arrive`, and — per `docs/MVP_STATUS.md` — three camera modes.
`clients/protocol/demo/demo.gd` (459 lines) adds the autopilot and transcript that produce evidence.

Tier one plus:

| Work | Estimate |
| --- | --- |
| The third camera mode | 0.25 d |
| HUD: wording tables for action types and rejection codes, prompt polish | 0.75 d |
| An autopilot mode producing a transcript, for evidence | 1.5 d |
| Evidence capture and a launch script matching `clients/protocol/run.sh` | 1.0 d |
| **Total, including tier one** | **10–12 working days, call it two to two and a half weeks** |

**Confidence: medium.** The scope is knowable — it is a file in this repository — but one item is
genuinely uncertain and is flagged rather than absorbed:

**Automated evidence capture is meaningfully harder in Unreal than in Godot, and I have not tested
how much.** The Godot clients produce `AC-13` fixtures under `godot --headless`
(`clients/protocol/run.sh`). Unreal has no comparably light headless client; the nearest is a
commandlet or a dedicated-client build with `-nullrhi`, which is heavier and slower. The saving grace
is that the parity comparison is computed **server-side** in `server/src/parity.rs`, so what an
Unreal client must contribute is submitted frames and not a headless render. I rate this a real risk
to the *automation* and not to the *evidence*, and phase two should try it early because it is cheap
to try and expensive to discover late.

### 8.3 Tier three — the visual quality of the reference plates

The target is `ARC-20`'s `VIS-3D-UE5-1`: an Unreal slice of scope equivalent to `VIS-3D-GODOT-2` —
character, street, enterable building, interior, lighting, movement, cameras — judged against
`presentation/mineworld-default/3D/references/` under `VISUAL_FIDELITY.md`'s rules.

**The estimate splits into engine work and asset work, and that split is the finding.**

| | Estimate | Engine-dependent? |
| --- | --- | --- |
| Lighting rig, exposure, post-process, sky, and a street blockout in Unreal | 1–2 weeks | **Yes**, and Unreal's defaults start closer to the plates |
| Architecture: box geometry at correct proportions dressed with CC0 materials, one enterable interior | 3–6 weeks | **No** |
| Vegetation, props, street furniture, signage | 1–2 weeks | **No** |
| The character to `04_character_closeup.png` under `ARC-17` and `ARC-19` | see §9 | **Contested — this is the whole question** |
| **Total, excluding the character** | **5–10 weeks** | |

**Confidence: low, and low specifically on the upper bound.** The asset half has never been completed
once in this project in either engine, and `VIS-3D-GODOT-1` failed its first attempt on categorical
identity mismatch (`ARC-17`). An estimate for work whose first attempt failed is a guess with a
range, and calling it anything else would be the kind of confident-sounding report `CLAUDE.md` warns
against.

**The honest comparative statement, which is the part that bears on `ARC-18`'s decision:**

> **Unreal does not reduce the environment asset cost at all.** `DEP-8` established that no
> open-licensed semi-realistic modular building kit exists, so the route is box geometry at correct
> proportions dressed with CC0 brick, plaster, painted wood and roofing from ambientCG and Poly
> Haven. That is Blender work and sourcing work. It is identical in Godot and in Unreal, and the
> five-to-ten-week range above is the same range in either engine.

Where Unreal plausibly shortens the path is narrower than `ARC-18`'s framing implies, and it is
exactly one thing: **the character.** Automated high-fidelity human creation, groom, cloth and
animation retargeting are Unreal's genuine ecosystem advantage, and `ARC-19` §3 already directs that
MetaHuman be evaluated for it rather than a bespoke pipeline built.

Whether that advantage is *usable by MineWorld* is a licence question and not a technical one, which
is §9.

The engine-side advantage that is real and should not be overstated: Unreal's out-of-the-box
lighting, tonemapping and material response start closer to the plates than a from-scratch Godot
Forward+ setup does, which is worth perhaps a week on the first row of the table and nothing on the
others. `ARC-18` already assessed that nothing in the plates is out of Forward+'s reach, and phase
one found no evidence against that assessment.

## 10. Unreal's own licence and cost, verified — and `ARC-18`'s summary corrected

`ARC-18` records a cost summary and says of it: *"This is a summary and the EULA governs."* This
section is the check that sentence invites. **The summary is wrong in one material respect and
incomplete in four others**, and `ARC-18` has been amended in place with a pointer here.

### 10.1 How these findings were obtained, and the limitation that carries

**Stated plainly, because it bears on how much weight these quotes can hold.**
`www.unrealengine.com` and `www.fab.com` sit behind a Cloudflare interactive challenge and returned
**HTTP 403 to every automated request** attempted — plain requests, requests with a full browser
header set, and a rendering proxy — on `/eula/unreal`, `/eula/publishing`, `/eula/creators`,
`/license`, `/eula/content`, `/eula-change-log/unreal` and `fab.com/eula`. Epic's documentation host
`dev.epicgames.com` serves a JavaScript shell to automated fetches for the licensing pages.
`legal.epicgames.com` has no Unreal EULA path.

So the quotations below come from **Internet Archive captures of Epic's own pages**, each with its
capture date. That is an archived copy of an Epic-authored document, which is a great deal better
than a summary or a news article and is **not** the live document.

**There is a specific trap here that this project should record, because it is very likely what
produced an earlier wrong answer.** The PDF a web search returns as *the* Unreal Engine EULA —
`cdn2.unrealengine.com/unreal-engine-end-user-license-agreement-d2812e10c642.pdf` — is live, fetches
with HTTP 200, and its own header says `last-modified: Thu, 24 Mar 2022`. A second CDN copy is dated
August 2022, and the superseded "EULA for Publishing v15" is also still live on Epic's CDN. None of
the 2022 documents contains the words "Seat", "Launch Everywhere", "3.5%" or "Fab". **A confident,
well-cited, four-year-stale answer is the default outcome of researching this topic**, and the
citation looks perfect.

**What a human must still do.** Open `https://www.unrealengine.com/eula/unreal` and
`https://www.unrealengine.com/eula-change-log/unreal` in a normal browser before any licensing
decision rests on this section. The change log has **no Wayback captures at all**, so nothing here
can confirm that no amendment landed in the weeks before this document was written. That gap is
stated rather than papered over.

### 10.2 Which agreement, and is it open source

**One agreement now governs**, and the Publishing / Creators split is gone. From the capture of
`https://www.unrealengine.com/eula/unreal` (2026-08-29), §8(b):

> "With respect to your rights and obligations related to Licensed Technology, this Agreement
> supersedes any prior Unreal Engine End User License Agreement for Publishing, Unreal Engine End
> User License Agreement for Creators, and MetaHuman Creator End User License Agreement you may
> have."

**It is proprietary, and under no OSI-approved licence.** The preamble:

> "This Agreement is a legal document detailing your rights and obligations related to using Epic's
> proprietary computer software program known as Unreal® Engine…"

§7:

> "we own all title, ownership rights, and intellectual property rights in the Licensed Technology."

So `ARC-18`'s *"Epic grants source access under the Epic EULA and is not permissive open source"* is
correct and can be stated more strongly: **source-available proprietary** is the accurate phrase.
`ARC-18` §1's conclusion that the fully open reference client is not negotiable is untouched by
anything in this section.

One helpful specific: Epic's own GitHub FAQ names MIT as acceptable to combine with —
*"Acceptable Non-Copyleft licenses include: Software licensed under the BSD License, MIT License,
Microsoft Public License, or Apache License"* — while §6(c) forbids combining the Engine with GPL,
LGPL (except dynamic linking) or CC-BY-SA. MineWorld's own licence is compatible for *combination*.
That is not the same as making Engine Code redistributable, which §10.4 covers.

### 10.3 The royalty, and the error in `ARC-18`

**`ARC-18` says:** *"Unreal is free below $1M USD trailing-twelve-month revenue. For a product whose
runtime depends on Unreal, the standard model is a 5% royalty on lifetime gross revenue above the
first $1M per product, reducible under Epic's 'Launch Everywhere with Epic' terms."*

**The error: those are two different $1M thresholds, belonging to two different payment regimes, and
the sentence merges them.** Epic itself draws the distinction, on
`https://www.unrealengine.com/license` (capture 2026-09-05):

> "There are two $1 million thresholds and they depend on what you make and how much you make."
>
> "Royalties are determined by the *lifetime gross revenue of the product or title you've created*.
> Once that project has earned $1 million—whether that happens in a month, or three years down the
> line—you'll start paying royalties on your earnings above the first million dollars."
>
> "Unreal Subscription prices are determined by *your annual gross company revenue*. If your company
> has reported earnings of $1 million or more in the last 12 months or fiscal year, you'll need to
> pay for seats."

The trailing-twelve-month figure is the **seat-subscription** test, EULA §6(b)(i):

> "During any period that you, together with any entities in your corporate group, have generated
> less than $1,000,000 USD in gross revenue over the last 12 months, the Seat subscription
> requirement will not apply to your Users."

The royalty test is the Royalty Addendum §4(b)(ii), and it is lifetime and per product:

> "the first $1,000,000 in lifetime gross revenue for each Royalty Product"

**Which regime a MineWorld Unreal client falls under, and it is not the seat one.** EULA §3:

> "(a) If you are developing a Royalty Product (as defined in Section 4(b)), you do not need to pay
> Epic any seat subscription fees for that use of the Licensed Technology. However, you may need to
> pay Epic royalties on the worldwide gross revenue attributable to each Royalty Product.
> (b) For any other use of the Licensed Technology, you will need to purchase seat subscriptions…"

and Epic's pricing page: *"If you're creating a game or application that relies on engine code at
runtime and will be licensed to third party end users, you'll pay royalties and won't be required to
purchase seats."* An Unreal MineWorld client relies on Engine Code at runtime and is licensed to
third-party end users. **Royalty regime, no seats, at any company revenue.**

**The rate.** Royalty Addendum §3(a):

> "The Royalty Rate is equal to 5% of all Royalty Revenue (as defined in Section 4) unless your
> Royalty Product qualifies for a reduced royalty rate."

§4:

> ""Royalty Revenue" means all worldwide gross revenue attributable to each Royalty Product minus any
> allowed exclusions enumerated in Section 4(b)…"

**Four exclusions `ARC-18` omits**, all §4(b): the first $1,000,000 lifetime per product; a quarter
in which the product earns under $10,000; the first $5,000,000 for the Oculus Store; and —

> "revenue generated from sales of your Product on the Fab Marketplace or the Epic Games Store, and
> from any subsequent in-Product purchases making use of Epic's payment services."

**One omission that bears directly on MineWorld's two-client design.** The EULA's main body §4(b):

> "A Royalty Product includes any Ports of that Royalty Product. A "Port" means a Royalty Product
> which (a) is an adaptation of an already released Royalty Product under an existing Unreal Engine
> License Agreement for its release on a different platform…"

So platform builds of one MineWorld world share a single $1M allowance rather than each receiving
their own.

**"Launch Everywhere with Epic" is a rate reduction, not a waiver, and `ARC-18`'s word "reducible"
understates what it costs.** Royalty Addendum §3(b):

> "If your Royalty Product qualifies as a Launch Everywhere with Epic Release (defined below), then
> beginning on the date the Royalty Product qualifies as a Launch Everywhere with Epic Release, the
> Royalty Rate will be reduced to 3.5% of all Royalty Revenue collected going forward across all
> platforms and stores."
>
> "If, at any time, a Royalty Product no longer qualifies as a Launch Everywhere with Epic Release
> the Royalty Rate will revert to 5%…"

Qualifying requires release on the Epic Games Store *before or simultaneously with* every other store
on each platform where EGS exists, plus content parity, feature parity and **marketing parity** —
*"any mention of availability of the Royalty Product on another store in any marketing materials…
will include equivalent mention of availability on Epic Games Store. This requirement applies to
pre-launch marketing."* It applies only to products released on or after 2025-01-01, and it is
revocable.

**The seat regime, which `ARC-18` omits entirely**, is real and current at *"$1,850 per seat per
year"* (pricing page capture). It would reach MineWorld only for a use that is *not* a Royalty
Product — internal tooling, work-for-hire builds, rendered marketing video — and then only once the
corporate group passes $1M over the trailing twelve months. EULA §3 allows both regimes at once:
*"either royalty payments, or seat subscription fees, or a combination"*.

### 10.4 The clause that constrains an open-source MineWorld, and it is not the royalty

`ARC-18` says an Unreal adapter *"would be a separately licensed deliverable"*. That is right, and
the reason is sharper than the royalty. EULA §5(a)(i):

> "You may Distribute Engine Code (including as modified by you) in Source Code or object code to a
> third party who is separately licensed by us to use the same version of the Engine Code that you
> are Distributing."
>
> "Any public Distribution of Engine Tools … must take place through a marketplace operated by Epic
> such as the Fab Marketplace … or through a fork of Epic's GitHub UnrealEngine Network…"

§5(a)(ii) caps public posting:

> "You are permitted to post snippets of Engine Code, up to 30 lines of code in length, online in
> public forums for the sole purpose of discussing the content of the snippet…"

§4:

> "Any Product that you Distribute that incorporates Licensed Technology must incorporate the
> Licensed Technology only in object code and only as an inseparable part of the Product."

**Hard consequence for this repository: no Engine Code may ever appear in a public MineWorld
repository, including modifications MineWorld itself made to it.**

**What can be published under MIT, and the part that is genuinely unresolved.** §7 gives us ownership
of our own code — *"you own all rights, other than rights in the Licensed Technology, in the Products
you develop"* — and Epic's licensing FAQ is relaxed about plugin authors. A **runtime-only** Unreal
module containing only MineWorld-authored code and no copied Engine Code is not Engine Code and is
not Engine Tools, so publishing it under MIT appears permitted.

Two cautions, and I am labelling them as what they are.

*Inference, not a quote.* **Editor-mode tooling is at risk.** §6(d) defines Engine Tools to include
*"(iii) other software that may be used to develop standalone products based on the Licensed
Technology"*, and a MineWorld world-authoring plugin living in the Unreal Editor could be argued into
that limb — which would confine its distribution to Fab or a fork of Epic's GitHub network. The safe
shape is: public MIT repository holds **runtime modules and non-Engine assets only**.

*Unresolved from primary sources.* **Whether compiling against Engine headers affects public MIT
distribution of our own source is not addressed anywhere in the EULA.** The agreement regulates
distribution of Licensed Technology and of Products incorporating it; it says nothing about what
licence you may place on your own separable code, and nothing about `#include`. Only a written
clarification or a custom licence from Epic settles it, and §8(b) confirms a custom licence would
override the EULA. That requires a human to ask Epic and is out of this spike's reach.

### 10.5 Two clauses nobody asked about that MineWorld must see

**Generative AI.** EULA §6(e) forbids:

> "using the Licensed Technology as a training input to any Generative AI Program or as prompt-based
> input where the Generative AI Program trains on input data"

and separately forbids using MetaHuman characters, animation curves *"or any rendered output thereof
if crafted to replicate the functionality of MetaHuman"* to build or enhance any database or to train
or test machine-learning systems.

*Inference, flagged.* MineWorld's cognition layer operates on simulation state behind contracts, not
on Engine assets, so ordinary language-model-driven `Controller` behaviour appears untouched. The
clause bites if Engine assets, Engine output or MetaHuman content were fed to a model that trains on
its input. Given `tools/asset_generation/` exists in this repository, this deserves an explicit line
in `DECISIONS.md` if an Unreal client is adopted, rather than an assumption.

**Contributions.** EULA §7(c) grants Epic a *"non-exclusive, fully-paid, irrevocable, transferable,
sublicensable license"* over anything submitted to Epic's GitHub UnrealEngine network. Relevant
because §5(a)(i) makes that network one of only two legal routes for publicly distributing Engine
Tools.

### 10.6 The corrected record

The paragraph that should replace `ARC-18`'s cost summary, and which has been placed there:

> Unreal Engine is source-available proprietary software under a single agreement, the **Unreal
> Engine End User License Agreement**, which supersedes the former Publishing and Creators EULAs. It
> is under no OSI-approved licence.
>
> Epic operates **two** payment regimes and **two distinct $1M thresholds**, and conflating them is
> the error this record contained. A MineWorld Unreal client relies on Engine Code at runtime and is
> licensed to third-party end users, so it is a **Royalty Product**: **no seat fees at any company
> revenue**, and **5% of worldwide gross revenue attributable to the product**, excluding the first
> **$1,000,000 lifetime per product**, quarters under $10,000, the first $5M on the Oculus Store, and
> Epic Games Store and Fab revenue outright. Ports share one $1M allowance. The rate falls to
> **3.5%** for a "Launch Everywhere with Epic Release" — Epic Games Store release before or
> simultaneous with other stores on each platform, plus content, feature and marketing parity;
> products released on or after 2025-01-01; reverts to 5% on disqualification. The separate
> **$1,850 per seat per year** subscription applies only to uses that are not Royalty Products, and
> only once the corporate group passes **$1M gross revenue over the trailing twelve months**.
>
> **Repository constraint, and it is the binding one:** no Engine Code may appear in any public
> MineWorld repository. Engine Code goes only to same-version Epic licensees; public distribution of
> Engine Tools must go through Fab or a fork of Epic's GitHub `UnrealEngine` network; a Product
> embeds Licensed Technology only in object code as an inseparable part; public posting is capped at
> 30-line snippets. MineWorld-authored **runtime** modules containing no Engine Code may be published
> under MIT — Epic names MIT as an acceptable non-copyleft licence to combine with — but **editor
> tooling is at risk under the Engine Tools definition** and belongs outside the public MIT
> repository. **Unresolved:** whether compiling against Engine headers affects public MIT
> distribution of our own source; only Epic can settle it.

`ARC-18` §1's decision is unaffected in substance: Godot remains the official reference renderer
because it is MIT and MineWorld can stay permissively licensed end to end, and nothing in the current
EULA weakens that reasoning. What changes is that the cost paragraph beneath it now says what Epic's
terms actually say.
