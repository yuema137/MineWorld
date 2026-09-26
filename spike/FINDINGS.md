# Renderer-integration spike — findings against the merged contracts

**Branch:** `spike/renderer-integration`, based on `main @ a2b10d9`.
**Question:** do MineWorld's contracts survive contact with a real renderer, before the kernel is
finished and while changing a contract is still cheap?
**Method:** a demo server holding a hand-made world and speaking the real `mineworld-contracts`
types over WebSocket, plus two Godot 4.7.2 clients — 2D top-down and 3D embodied first-person —
that consume the same `Observation` and submit the same `talk`.
**Rule observed throughout:** nothing under `contracts/` or `kernel/` was modified. Every
conclusion below is a finding to be reviewed, not an edit that was made.

Written as the work proceeds. Findings are numbered in the order found and ranked at the end.
A finding is marked `[verified]` only once a run produced the evidence named under it.

---

## What was built

```text
spike/server/      Rust binary. Depends on mineworld-contracts by path; does NOT use the kernel.
                   Its own Cargo workspace, so the root workspace and Cargo.lock are untouched.
  vocabulary.rs    The System Pack vocabulary the spike invents: talk / move-to / pick-up,
                   display-name / body / place-extent / signage, conversation-started / moved.
  world.rs         The hand-made world and every rule in the spike: authoritative state, the
                   per-observer Observation, affordance computation, dispatch, the event log.
  wire.rs          DD-15's wire encoding: 64-bit ids as decimal strings, at the protocol
                   boundary, implemented as a structural mirror of the contract shapes.
  main.rs          axum WebSocket endpoint, the 10 Hz observation loop, intent recording and
                   the AC-13 parity comparison.
spike/client-2d/   Godot 4.7.2 project: top-down renderer, click-to-move, click-to-talk.
spike/client-3d/   Godot 4.7.2 project: CharacterBody3D, gravity, collision, camera look,
                   RayCast3D targeting, walk-near -> look-at -> press E.
spike/evidence/    PNGs, the two submitted intents, and the parity verdict.
```

The world is five entities in one place: `player` (101), `alice` (9007199254740995), `bob` (103),
a chipped mug (9007199254740997), and the `lakeside-cafe` place itself (9007199254740993). Three
of the five ids are deliberately above 2^53 — see F1.

---

## F1 — `DD-15` holds, and the decimal-string encoding does fix it *for the fields it covers*

`EntityId`, `EventId`, `ActionId` and `ProcessId` are `#[serde(transparent)]` over `u64`, so the
contract's own `serde` puts them on the wire as JSON numbers. Godot's `JSON.parse_string` returns
every JSON number as a `float`. The spike chooses ids that make the damage unmissable rather than
theoretical: as IEEE-754 doubles,

```text
9007199254740993  (the place)  ->  9007199254740992
9007199254740995  (Alice)      ->  9007199254740996
9007199254740997  (the mug)    ->  9007199254740996   <- the same value as Alice
```

Two distinct entities collapse into one. A client that bound a rendered body to a parsed number
would raycast the mug and talk to Alice.

The decimal-string encoding at the protocol boundary fixes this for every id the encoder covers,
without touching the contract: `spike/server/src/wire.rs` converts a serialized `Observation`,
`ActionIntent` and `ActionResult` in both directions. DD-15's reasoning — that the fix belongs at
the protocol boundary, not in the contract's `serde` — is sound as far as it goes.

**But see F2, which is the half DD-15 did not anticipate.**

---

## F2 — the wire encoding **cannot** cover component and event payloads, and real payloads will contain `EntityId`s

**Severity: highest. The finding I would act on first.**

`ComponentRecord<P>` and `EventRecord<P>` are documented payload-erasure boundaries: the contract
layer "never interprets" `P`. That is right for the contract. It is fatal for the protocol layer
DD-15 hands the problem to, because the protocol layer cannot interpret `P` either — so it cannot
find the ids inside it.

`wire.rs` therefore stops at the payload. The spike proves the consequence with a deliberate trap:
the `signage` component on the place carries `catalogue_id: 9007199254740999` and
`id: 9007199254741001`, neither of which is an entity id. An encoder that descended into payloads
keying on the field name `id`, or on "any integer above 2^53", would corrupt values that are not
ids at all. So descending is not an option either.

The frame the server actually sends:

```json
"component_type": "signage",
"entity": "9007199254740993",                                            <- protected (contract field)
"payload": { "catalogue_id": 9007199254740999, "id": 9007199254741001 }  <- NOT protected
```

Now consider what a real System Pack puts in a component payload. `Employment { employer:
EntityId }`. `Conversation { talking_to: EntityId }`. `Ownership { owner: EntityId }`.
`Inventory { contents: Vec<EntityId> }`. Every one is a 64-bit id inside a payload the protocol
layer is forbidden to look into. DD-15's fix protects the envelope and leaves the cargo exposed,
and the failure is silent: the Rust side is correct, the JSON text is correct, only the client's
parsed value is wrong.

This is not a bug in the spike's encoder. It is a structural consequence of erasing the payload at
the contract layer and fixing the encoding at the protocol layer. The two decisions are
individually right and jointly insufficient.

**Smallest change that would fix it:** make the *contract's* id newtypes encoding-aware instead of
making the protocol schema-aware. `serde` already distinguishes the two cases through
`Serializer::is_human_readable()` — `true` for `serde_json`, `false` for `bincode`, `postcard` and
every binary codec. A hand-written `Serialize`/`Deserialize` on `EntityId`, `EventId`, `ActionId`
and `ProcessId` (about fifteen lines each, or one macro) emitting a decimal string when the
serializer is human-readable and a `u64` when it is not would:

- protect ids wherever they appear, including inside any payload a system serializes with
  `serde_json`, because the rule travels with the type instead of with the frame;
- leave every binary encoding byte-identical, so persistence and replay determinism are
  unaffected;
- delete `wire.rs` entirely, along with the obligation to re-audit it whenever a contract type
  gains a field.

DD-15 rejected "make the contract serialize ids as strings" because it "would distort persistence
and any binary encoding to suit one client's parser". `is_human_readable()` answers that
precisely: the binary encoding is not touched. What remains is that a JSON *save* would store
strings — a readability change, not a correctness one, and one that makes JSON saves correct on
the same 2^53 boundary that broke the client.

If that is still rejected, the fallback must be written down explicitly: **no System Pack may put
an `EntityId`, or any integer above 2^53, in a component or event payload.** That is an
implausible restriction, since components referring to other entities is the normal case — which
is itself the argument for the `is_human_readable` fix.

---

## F3 — a `Place` has a location but no *shape*, and `Observation` carries no relations

A 3D walking client must build a floor, four walls and a doorway before it can walk anywhere. A 2D
client must draw a room outline before placing anybody in it. `Location` answers "which place, and
where inside it"; nothing answers "how big is the place, and where does it stop".

The spike worked around this with a `place-extent` component (an axis-aligned box in the place's
own millimetres) owned by an invented `geography` system, and both clients build their geometry
from it with no hardcoded dimensions. That is the *correct* shape of the answer and needs no
contract change: place geometry is a System Pack's state, exactly like a name. So this half is not
a contract defect — it is the contract working.

What *is* a gap is two adjacent facts:

1. **A place entity in an `Observation` has no `Location` of its own.** `Location::in_place` takes
   a `PlaceId`, and giving the café a location inside itself is nonsense, so the spike sends
   `"location": null` for the place. Fine with one place. It means a client cannot lay out two
   places relative to each other from an observation alone — and "walk out of the café and along
   the promenade" is the product's north star (`ENGINEERING_RULES.md` §1).
2. **`Observation` carries no relations at all.** `contracts/src/relation.rs` exists;
   `Observation` has no field for it. Place *hierarchy* is explicitly a relation
   (`spatial.rs`: "that a kitchen is inside a café is a relation (S1), not a field of this type"),
   so the one mechanism that would tell a client the café contains a kitchen is the one mechanism
   an observation cannot deliver. The same hole hides ownership, membership and every other
   relation a client might need to draw.

**Smallest change that would fix it:** add `relations: Vec<Relation>` to `Observation`, filtered by
the same perception system that filters entities and events — one field and one builder method, no
engine concept. Decide it before S11 freezes the observation frame; adding it later changes every
client.

---

## F4 — a client must invent an `ActionId` it has no right to allocate

`ActionIntent::new` requires an `ActionId` as its first argument. `contracts/src/ids.rs` says of
every runtime id: *"Allocation belongs to the kernel, not to this crate. `EntityId::from_raw`
exists so that the allocator and the persistence layer can rebuild an identity they already own;
it is not a way to invent one."*

A client submitting an intent has no allocator. Both spike clients call the moral equivalent of
`ActionId::from_raw(n)` with an `n` they made up — in GDScript, a local counter. Two clients
connected to one world collide on the first action each submits, and the server cannot tell a
collision from a retransmission.

Consequences already visible here:

- the two clients' `talk` intents **cannot** be identical, because their invented `action_id`s
  differ (F6);
- `Causation::Action(ActionId)` and `Provenance::controller_decision` both point at an id a client
  chose, so the event log's causal chain is anchored on client-supplied identity;
- a server that re-allocates the id on receipt produces an `ActionIntent` the client cannot
  correlate its answer to, since `ActionResult` deliberately carries no `ActionId` (OBS-1).

**Smallest change that would fix it:** `ActionIntent` is the wrong type for a client-to-server
frame. Either (a) the wire protocol carries a *request* with no `action_id` plus a client-chosen
correlation token, and the server builds the `ActionIntent` with an id it allocated — the token is
a protocol concern and never enters the contract; or (b) the contract gains an explicit
unallocated form, `ActionRequest { actor, payload, target, actor_location }`, with
`ActionIntent::allocate(request, action_id, issued_at)` as the only way to get an intent. (a) needs
no contract change and should be written into S11's design now, before two clients exist. (b) is
cleaner and makes "a client cannot invent identity" structural rather than procedural.

The same reasoning applies more weakly to `issued_at: WorldTime`: a client does not have the
world's clock, so both spike clients echo the `at` of the last observation they received — the
time of a past frame, not of the request. Harmless here, wrong in principle, free to fix under (a).

---

## F5 — the spatial contract works for 3D; yaw has no defined zero and no defined handedness

Integer millimetres and millidegrees were the two choices expected to chafe. Mostly they do not:

- **Millimetres are fine.** Godot works in float metres; the conversion is one multiply per axis at
  the boundary (`mm / 1000.0`), done when a body is built and when a position is reported. i32
  millimetres spans ±2 147 km, and a 1 mm quantum is far below what a 2.5 m/s walk resolves at
  60 Hz (about 42 mm per frame).
- **Integer distance comparison is an asset.** `SpatialRequirement::evaluate` compares squared
  distances in `i128` with no square root, so the server's answer and a client's guess cannot
  disagree by a rounding epsilon — precisely the class of bug that makes "the prompt said I could
  but the server said too far" unreproducible.
- **Millidegrees are fine as a unit.** `Orientation::facing` canonicalizes into `[0, 360_000)`,
  which is the range a compass wants; converting to Godot is one divide and a `deg_to_rad`.

The chafe is not the unit. It is that **`Orientation` defines a range and not a frame**:

- which direction is yaw `0`? The contract says "the heading around the vertical axis" and stops.
- does yaw increase clockwise or counter-clockwise seen from above?
- `LocalPosition` says "`x` and `y` span the ground plane and `z` is height" but does not say
  whether the frame is right- or left-handed, so even with a yaw zero fixed the turn direction is
  undetermined.

Not academic. Godot's 3D convention is `-Z` forward, `+Y` up, `rotation.y` increasing
counter-clockwise from `-Z`; Godot's 2D convention is `+Y` **down** with angles increasing
clockwise. The spike had to invent a convention — yaw as a compass bearing, `0` = `+y`, increasing
toward `+x` — and then write *two different* conversions, one per client, each with a sign flip
nothing in the contract could have told the author about. Two independently written clients will
disagree, and the disagreement shows up as NPCs facing backwards: the kind of bug that gets
"fixed" by negating something in a renderer until it looks right.

**Smallest change that would fix it:** three sentences of documentation on `Orientation` and
`LocalPosition` — "`+x` is east, `+y` is north, `+z` is up; the frame is right-handed; yaw is
measured from `+y` toward `+x`", or whatever convention is chosen — plus one doc example turning a
yaw of `90_000` into a unit direction. No type change, no engine concept, and it converts a class
of silent client bug into an obvious conversion. Worth one more sentence stating that `Orientation`
has no roll deliberately, so nobody adds one for a flight system without a decision.

---

## F6 — "the same `ActionIntent`" cannot mean field-identical, and `AC-13` needs to say which

`overall.md` records the throwaway harness as producing "byte identical" intents, comparing
`{action_type, actor, target}`. Against the real contract the two clients' intents **cannot** be
field-identical, and three fields are the reason:

| field | 2D | 3D | why it must differ |
| --- | --- | --- | --- |
| `action_id` | invented by the client | invented by the client | F4 — no client may allocate |
| `issued_at` | the last observation's `at` | the last observation's `at` | different frames |
| `actor_location` | absent | present | a 2D client models no continuous position |

`actor_location` is the interesting one, because it is *correct* for it to differ: the contract's
own documentation says "a 3D client sends the position it walked to, a 2D client that models no
position sends none". So the acceptance criterion as written is unachievable against the contract
designed to satisfy it.

The spike therefore records **two** verdicts in `spike/evidence/parity.json`:
`identical_whole_intent` and `identical_semantic_core`, the semantic core being
`{actor, action_type, target, payload}` — what the world is being asked for, stripped of who asked
and when.

**Smallest change that would fix it:** none in the contract; a correction in the plan. `AC-13`
should read "the two clients produce `ActionIntent`s with an identical semantic core — the same
`actor`, `action_type`, `target` and payload — resolved by the same system to the same
`ActionResult`", and should name the fields permitted to differ. Better still, S11 defines the
comparison once, so the S12/S14 acceptance test cannot quietly compare a different set of fields
than the one agreed.

---

## F7 — an `Affordance` is computed against server position and carries no marker of when

`Affordance` does what `DD-11` promises (evidence under `§ Runs`). The gap is temporal rather than
structural. The server evaluates the requirement against *its own* authoritative position. A 3D
client's body is a local prediction that leads the server's by up to one round trip, so near the
2.5 m boundary the client renders an affordance computed for a position it has already left.

The contract gives a client no way to notice. `Observation.at` is the world clock in *seconds*
(`WorldTime` is `i64` seconds), so every observation inside one simulated second is stamped
identically: a client cannot order two frames, let alone tell how stale an affordance is. There is
no sequence number and no "computed at this actor position".

**Smallest change that would fix it:** nothing in `Affordance`. Either give the protocol frame a
monotonic sequence number (S11's business, no contract change), or — if sub-second simulated
resolution is wanted for other reasons — reconsider `WorldTime`'s second granularity before the
S4 scheduler fixes it. A 10 Hz observation stream against a 1 Hz clock is a mismatch worth noticing
now rather than after the scheduler is written.

---

## F8 — smaller chafes, none blocking

1. **`action_type` is carried twice in an `ActionIntent`**, once in the envelope and once inside the
   `ActionRecord`; they must agree or `ActionIntentPayloadMismatch` rejects the frame. In Rust
   `ActionIntent::new` reads it off the record so they cannot disagree; a GDScript client
   hand-builds JSON and can. The contract catching it is the system working, but it is a redundancy
   every non-Rust client pays for. One line in S11's protocol spec, not a contract change.
2. **`P = Vec<u8>` is the wrong default for a JSON transport.** With the default payload type a
   serialized `ComponentRecord` carries its payload as a JSON array of byte integers, unusable for
   a client. `Observation<serde_json::Value>` fixes it completely, and the generic parameter exists
   precisely so it can — the contract being right. Recorded only because it is not obvious; S11
   should state the instantiation it uses.
3. **`Rejection` has no `Display`, deliberately (`DD-13`)**, so both clients carry a
   variant-to-English map. Correct, and it worked. `Rejection::System { code, detail }` means every
   client also needs a fallback path; both spike clients have one.
4. **`Tags` are what the clients used to choose colours**, keeping an appearance decision in the
   client where it belongs. A small vindication of tags as an open vocabulary.
5. **`WorldTime` and `SimDuration` are `i64`** and share the 2^53 problem in principle. A world
   clock in seconds will not reach 2^53, so this is theoretical — but if F2's `is_human_readable`
   fix is adopted, covering these two costs nothing.

---

## Runs

To be filled in as each run completes.

## Ranking

To be filled in at the end.
