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

Unreal is not installed, so none of §3.5 was run. The MineWorld half is confirmed from source and from
the captured frames. The Unreal half — left-handedness, `+Z` up, `+X` forward, `+Y` right, 1 uu = 1 cm,
`FRotator::Yaw` turning `+X` toward `+Y`, and `FRotator::Pitch` positive looking up — is stated from
Epic's published documentation as reported in §7.3, and the **pitch sign is the one to check first in
phase two**: it is a single bit, it is the kind of thing an engine changes conventions on, and getting
it wrong produces a character that looks at the sky when it should look at the pavement. The
conformance check in §3.4 mitigation 2 is where that bit belongs.

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

## 5. Three defects found while running the test

None blocks an Unreal client. All three are recorded here rather than fixed, because this spike is
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

The three items in §5 are documentation and forward-looking findings. §5.1 and §5.2 are one-line and
one-paragraph corrections to merged specifications. §5.3 is a real design question that belongs with
movement and travel, is engine-neutral, and should not be decided by a renderer spike.

---

## 7. Placeholder — filled in by the remaining sections

Sections 8 onward (the implementation plan, the cost estimates, and the licence findings) are written
in the same pass and appear below.
