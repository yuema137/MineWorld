# PR 04 — contract fixes from the renderer-integration spike

**DESIGN FROZEN** — frozen by the operator in the implementation session of 2026-09-26. The
operator reviewed `spike/FINDINGS.md`, accepted F2, F3 and F4, and designed the three fixes
below. Scope, invariants and acceptance are frozen; progress, evidence and bounded corrections
stay writable.

---

## 1. Why this PR exists

The renderer-integration spike (merged at `1bcc3be`) ran two real Godot 4.7.2 clients against
the merged contracts and produced measured evidence, not opinion. Three of its findings are
contract defects that are cheap now and expensive after S11 freezes the wire format:

| Fix | Finding | Defect |
| --- | --- | --- |
| 1 | F2 | 64-bit ids inside `ComponentRecord`/`EventRecord` payloads corrupt silently in any JSON client. DD-15's protocol-layer fix structurally cannot reach them. |
| 2 | F3 | `Observation` has no field for relations, and place hierarchy is *defined* as a relation, so a client can render one room and never a world. |
| 3 | F4 | `ActionIntent::new` demands an `ActionId`, which `ids.rs` says nothing outside the kernel may allocate. Both spike clients invented one from a local counter. |

## 2. Implementation contract

```text
PROJECT / PR:        MVP0 PR 04 — contract fixes from the renderer-integration spike
PRIMARY DESIGN DOC:  this file
RELATED / BINDING:   spike/FINDINGS.md (F2, F3, F4, F8.5, R1, R4)
                     .structured-coding/plans/mvp0/step-02-action-event-spatial-contracts.md (DD-15, OBS-1)
                     docs/ENGINEERING_RULES.md, docs/ENGINEERING_STANDARDS.md, docs/CORE_CONCEPTS.md
IMPLEMENTATION BASE: main @ a6ad5c7 — contracts, kernel and the merged spike present, 133 tests green
BRANCH:              mvp0/pr-04-contract-fixes

APPROVED SCOPE:
  contracts/src/{ids,action,observation}.rs and their tests; kernel call sites; the spike
  server; the DD-15 record in the step-02 plan; deletion of spike/server/src/wire.rs.

FROZEN INVARIANTS:
  - no floats anywhere in contracts/src (the structural test in tests/spatial.rs must pass)
  - no domain concepts and no engine concepts enter the contract layer
  - binary (non-human-readable) encodings stay numeric and byte-identical in shape
  - INV-13: an Observation is what one observer was shown, never a window onto everything
  - INV-6 / ids.rs: identity allocation belongs to the kernel, never to a client
  - fmt, check, clippy -D warnings and the full test suite clean at the final HEAD

ENDPOINT AUTHORITY:
  implementation + local validation  authorized   source: operator instruction, this session
  semantic commits                   authorized   source: operator instruction, this session
  branch push                        authorized   source: operator instruction, this session
                                                  ("git push -u origin mvp0/pr-04-contract-fixes")
  PR creation                        FORBIDDEN    source: operator instruction, this session
                                                  ("Do not merge and do not open a PR — I do that")
  merge                              FORBIDDEN    explicit operator authorization only

TEST OWNERSHIP:
  UNIT/CONTRACT owns all three fixes: serialization is deterministic and local, which is
  exactly what §2 of the test rules names as unit-owned. Expected JSON is written by hand.
  MUTATION owns the load-bearing question for Fix 1: break the encoding, confirm red.
  GATE 1 (real LLM):    NOT REQUIRED — no LLM-facing semantics changed.
  GATE 2 (real runtime): NOT RE-RUN — the Godot evidence that motivates these fixes is
  already preserved in spike/evidence/. Re-running the spike clients would re-measure the
  defect, not the fix, because the spike's own wire.rs already worked around it. Recorded as
  a limitation in §6 rather than claimed as evidence.

NORMAL STOP CONDITION:
  PR 04 implemented, validated, committed and pushed. NO PR OPENED. NO MERGE.
```

## 3. The three fixes, as designed

### Fix 1 (F2) — 64-bit ids survive JSON

Hand-written `Serialize`/`Deserialize` on `EntityId`, `EventId`, `ActionId` and `ProcessId`,
keyed on `Serializer::is_human_readable()` / `Deserializer::is_human_readable()`:

```text
human-readable (serde_json)    decimal string   "9007199254740993"
non-human-readable (binary)    u64              9007199254740993
```

Deserialization in the human-readable case accepts **both** a decimal string and a JSON
number, so existing fixtures and hand-written test data keep working. A float is rejected
outright: the visitor implements no float method, so serde's default `visit_f64` produces
`invalid type: floating point ...`. That is also why the rejection is implemented by omission
rather than by an explicit method — naming `f64` in `contracts/src` would fail the structural
no-float test, which is the correct outcome for a contract that must not compute in floats.

This answers DD-15's original objection exactly. DD-15 rejected "strings in the contract"
because it "would distort persistence and any binary encoding"; `is_human_readable()` is
precisely the distinction that prevents that. The DD-15 record in the step-02 plan is marked
superseded by evidence.

Consequence: `spike/server/src/wire.rs` is deleted, which is the point of the fix.

### Fix 2 (F3) — `Observation` carries relations

`relations: Vec<Relation>` on `Observation`, with `Observation::relating` to populate it and
`Observation::relations()` to read it. Populated by the same perception step that fills
`entities`: it is what the observer is entitled to see, not the whole graph. The field's doc
comment carries that constraint, because the field is the one place `INV-13` could be widened
by a careless perception system.

### Fix 3 (F4) — a client submits a request; the world makes it an intent

New `ActionRequest<P>` — `actor`, `action_type`, `target`, `payload`, optional
`actor_location` — as what a controller or a client submits. No `ActionId` and no
`WorldTime`, because a client has neither an allocator nor the world's clock.
`ActionIntent::allocate(request, action_id, issued_at)` is how the world turns one into an
intent. `ActionIntent::new` is kept: the kernel's own tests and dispatch's doc example build
intents in process, where the allocator is right there.

`ActionRequest` applies the same envelope/payload agreement check as `ActionIntent`, in
construction and in deserialization, because a hand-built client frame is exactly the input
that can disagree with itself (F8.1).

## 4. Commit plan

| # | Commit | Implementation | Validation | LLM logic review |
| --- | --- | --- | --- | --- |
| 1 | Fix 1: id encoding keyed on `is_human_readable`, and the DD-15 record | `[x]` | `[x]` | `[x]` |
| 2 | Fix 1: delete the spike's `wire.rs` | `[x]` | `[x]` | `[x]` |
| 3 | Fix 2: relations on `Observation` | `[x]` | `[x]` | `[x]` |
| 4 | Fix 3: `ActionRequest` and `ActionIntent::allocate` | `[x]` | `[x]` | `[x]` |

## 5. Live evidence log

### Baseline

```text
command:  cargo test --workspace   @ a6ad5c7
result:   133 passed, 0 failed   PASS
```

### Fix 1 — id encoding

```text
command:  cargo test --workspace   @ commit 1 — 363f0a0
purpose:  a 64-bit id above 2^53 survives JSON; binary stays numeric; a float is refused
result:   139 passed, 0 failed   PASS   (133 at the base, +6 new tests)
```

Fourteen existing tests stated the old shape in hand-written JSON and were updated to the new
one, in `contracts/tests/{action,component,entity,event,identity,observation,relation,spatial}.rs`
and `kernel/tests/two_systems.rs`. Each is a deliberate restatement of the contract, not a
loosened assertion: every one still pins an exact string.

New contract tests in `contracts/tests/identity.rs`, every expected value written by hand:

```text
opaque_identities_are_written_as_decimal_strings_in_json
    7 -> "7", 8 -> "8", 9 -> "9", 10 -> "10", for all four id types, and back.

the_2_53_boundary_survives_json_for_every_opaque_identity
    9007199254740993 / …995 / …997 / …999 / …1001 -> "9007199254740993" etc,
    and back, exactly, for EntityId, EventId, ActionId and ProcessId.
    The same test states what a one-number-type parser does to those five:
      …993 -> …992    …995 -> …996    …997 -> …996    …999 -> …1000    …1001 -> …1000
    Five distinct ids, three distinct doubles, two collisions. Those values are
    derived from IEEE-754 (above 2^53 the representable doubles are two apart, so
    an odd integer ties to the even mantissa), and they match what the spike
    measured from inside Godot (FINDINGS.md F1: alice …995 and mug …997 both
    arriving as …996, collide=true).

an_id_inside_a_component_payload_survives_json
    the exact frame asserted:
      {"entity":"9007199254740995","component_type":"employment",
       "schema_version":1,"payload":"{\"employer\":\"9007199254740993\"}"}
    The id inside the erased payload is a string for the same reason the id in the
    envelope is: the rule travels with the type. Conversation { talking_to } is
    asserted too, so the rule is visibly not keyed on a field name.

an_id_inside_an_event_payload_survives_json
    {"event_type":"item-picked-up","schema_version":1,
     "payload":"{\"item\":\"9007199254740997\"}"}

a_binary_format_encodes_an_opaque_identity_as_a_number
    all four id types write Written::Unsigned(9007199254740993) — serialize_u64,
    not serialize_str — and read back from a u64 through a deserializer that
    refuses deserialize_any, which is what a real binary format does.

json_accepts_both_the_string_and_the_number_form
    "9007199254740993" and 9007199254740993 both load to the same id, so existing
    fixtures keep working. -1, "-1", "" and "seven" are refused.

a_float_is_refused_rather_than_truncated
    9007199254740993.0 fails with "invalid type: floating point"; 1.5, -1.0, 1e3,
    "1.5" and "1e3" are refused for all four types.
```

### Fix 1 — mutation evidence

All three mutations were applied to `contracts/src/ids.rs`, measured, and reverted; the suite was
confirmed back at 139 passing afterwards. Mutation 1 was then **re-run against the committed
code at `363f0a0`** and reproduced identically — 17 red, verbatim `left: "9007199254740993"` /
`right: "\"9007199254740993\""` from `the_2_53_boundary_survives_json_for_every_opaque_identity`
— so the counterfactual belongs to the commit and not only to the working tree.

```text
mutation 1:  undo the fix — serialize_u64 unconditionally, dropping the
             is_human_readable branch on the way out
expected:    every JSON shape test goes red
observed:    17 distinct tests failed, across contracts/tests/{action, component,
             entity, event, identity, observation, relation, spatial}.rs and
             kernel/tests/two_systems.rs. Verbatim from identity.rs:
               assertion `left == right` failed
                 left: "7"
                right: "\"7\""
verdict:     behaviour-changing, caught by many independent tests. REVERTED.

mutation 2:  accept a float by truncating it (add a visit_f64 that casts to u64)
expected:    a_float_is_refused_rather_than_truncated goes red
observed:    2 tests failed. Verbatim:
               a float must not deserialize into an identity:
               EntityId(9007199254740994)
             — i.e. 9007199254740993.0 silently became a *different* entity, which
             is exactly the F1 corruption, now produced by the contract itself.
             no_floating_point_appears_anywhere_in_the_contract_crate also failed,
             because the mutation had to name a float type in contracts/src. Two
             independent guards caught one mutation.
verdict:     behaviour-changing, caught. REVERTED.

mutation 3:  drop the is_human_readable branch on the way in — always
             deserialize_str
expected:    the binary-format test and the both-forms test go red
observed:    6 tests failed: a_binary_format_encodes_an_opaque_identity_as_a_number,
             json_accepts_both_the_string_and_the_number_form,
             an_intent_for_an_action_no_system_provides_is_representable_and_answered_unavailable,
             an_intents_envelope_cannot_disagree_with_its_payload,
             a_persisted_registry_that_lost_an_invariant_is_refused_at_load,
             deserializing_an_inconsistent_registry_fails_rather_than_repairing_it.
             Verbatim:
               ProbeError("a format that is not human-readable cannot answer
               deserialize_any: the type must ask for the shape it wrote")
             which is the failure a real binary codec would produce.
verdict:     behaviour-changing, caught. REVERTED.
```

### Fix 1 — the spike, with `wire.rs` deleted

The consumer-side evidence, and the only part of this PR that runs outside the test suite.
`spike/server/src/wire.rs` is gone, `mod wire` is gone, and the observation frame carries one
encoding: whatever the contract's own `serde_json` produced.

```text
command:  cargo check   (spike/server, its own workspace)      clean
command:  cargo run -- --dump   (the real server, real contract types)
```

From that dump, verbatim:

```text
observer                         "101"
place entity id                  "9007199254740993"
mug entity id                    "9007199254740997"
affordance targets               [null, "9007199254740995", "103", "9007199254740997"]

ownership component on the mug
  entity                         "9007199254740997"
  payload                        {"owner": "9007199254740995"}      <- F2, protected

signage component on the place
  payload                        {"catalogue_id": 9007199254740999,
                                  "id": 9007199254741001, ...}      <- still numbers
```

The `ownership` line is the measurement this PR exists for. `spike/FINDINGS.md` F2 recorded
`sent 9007199254740999, parsed 9007199254741000, protected=false` for a payload field; an
`EntityId` in that same position is now a decimal string, and the server did nothing to make it
one. The `signage` line is the other half of the same rule and is *correct*: `catalogue_id` is a
large integer that is **not** an identity, so nothing may guess at it — which is precisely why a
field-name-guessing encoder was never an option and why the rule had to go on the type.

A new `Ownership { owner: EntityId }` component was added to the spike's invented vocabulary to
make that comparison available in one frame; it is F2's own third example, and it sits one
component away from the trap that must stay unprotected.

Both Godot clients were updated to the single-encoding frame and their id-encoding report now
derives the corruption from the string itself rather than from a second copy of the frame. They
were **not re-run** — see §6.

### Fix 2 — relations on `Observation`

```text
command:  cargo test --workspace   @ commit 3 — 421f544
result:   141 passed, 0 failed   PASS   (+2 new tests)
```

`relations: Vec<Relation>` sits between `entities` and `events`, with `Observation::relating` to
populate it and `Observation::relations()` to read it. The documented wire shape changed and the
test that pins it changed with it:

```text
{"observer":"41","at":0,"self_location":null,"entities":[],"relations":[],
 "events":[],"affordances":[]}
```

New tests in `contracts/tests/observation.rs`:

```text
an_observation_carries_the_place_structure_a_client_needs_to_render_a_world
    the café (7) contains the kitchen (8) and adjoins the promenade (9) — F3's
    "walk out of the café and along the promenade", expressible for the first
    time. The frame asserted by hand:
      "relations":[{"relation_type":"contains","from":"7","to":"8"},
                   {"relation_type":"adjoins","from":"7","to":"9"}]
    The undirected edge arrives canonically ordered (promenade 9 given first,
    stored 7 -> 9), so an observation transports edges and never re-forms them.
    The test also asserts that entity 9 is NOT in `entities`: being told the café
    adjoins the promenade is not the same as perceiving the promenade.

an_observation_exposes_no_relations_until_one_is_deliberately_added
    a fresh observation lists none, and the file's existing two-person scene
    lists none either — perceiving two people implies no edge between them.
```

No kernel call site: the kernel does not build `Observation`s yet (`kernel/src/view.rs` says so
explicitly — an observation is S10/S11's). The spike server compiles unchanged and its frame now
carries `"relations": []`, which is correct: it has exactly one place, and F3 itself says the
second place is where the next surprise lives.

`INV-13` review: the field is a list of what was exposed, like every other field on the type.
There is no accessor that takes an identity and searches the world, no way to ask for more, and
the doc comment on the field states the constraint a perception system must honour — an edge is
here because this observer may know it, and a system that put every edge in a world here would be
making the same mistake as one that listed every entity.

### Fix 3 — `ActionRequest`

```text
command:  cargo test --workspace   @ commit 4 — 0d2398d
result:   144 passed, 0 failed   PASS   (+3 new tests)
command:  cargo test   (spike/server, its own workspace)
result:   2 passed, 0 failed   PASS   (the spike had no tests before)
```

`ActionRequest<P>` carries `actor`, `action_type`, `target`, `payload` and an optional
`actor_location`, and applies the same envelope/payload agreement check as `ActionIntent` in
construction and in deserialization. `ActionIntent::allocate(request, action_id, issued_at)` is the
only way to get an intent from a request. `ActionIntent::new` is kept for code inside the world, and
its doc now says which of the two a caller wants and why.

New contract tests in `contracts/tests/action.rs`:

```text
a_client_submits_a_request_and_the_world_makes_it_an_intent
    the request carries no action_id and no issued_at; allocate adds
    ActionId(9007199254741099) and WorldTime 32418, and every other field is
    carried across unchanged — asserted field by field, because "the world does
    not edit what was asked for" is the claim. Allocating twice from one request
    yields two ids and one identical payload, which is the collision F4 measured,
    now impossible.

a_request_is_stored_as_its_documented_shape_and_cannot_disagree_with_itself
    the frame, asserted by hand:
      {"actor":"101","action_type":"give_item","target":"9007199254740995",
       "payload":{"action_type":"give_item","payload":"{\"item\":18517}"},
       "actor_location":null}
    and the test asserts that the text contains neither "action_id" nor
    "issued_at": there is nothing in this frame for a client to invent. A frame
    whose envelope and payload name different action types is refused, which is
    the check that matters most here because a request is what a non-Rust client
    hand-builds.

two_clients_that_acquire_a_request_differently_submit_the_same_request
    a click and a camera ray produce requests equal in actor, action_type, target
    and payload, differing only in actor_location — which the contract intends.
    With ActionRequest, AC-13's list of unavoidable differences drops from three
    fields to one.
```

And from the spike, which is the client side of the same claim:

```text
a_client_frame_with_no_identity_becomes_an_intent_the_world_identified
    the exact JSON client-2d/main.gd now builds, written out by hand in the test,
    deserializes into an ActionRequest with target 9007199254740995 intact; the
    world then supplies ActionId(9007199254741001) and WorldTime 32400, neither of
    which appeared in the frame.

two_clients_sending_the_same_frame_receive_two_identities
    the same frame twice yields 9007199254741001 and 9007199254741002 with an
    identical payload — the collision that used to be a client's local counter.
```

`SpikeWorld::allocate` is now the spike's only allocator, and both Godot clients send
`{"t":"request", ...}` frames with no `action_id` and no `issued_at`. The server's reply carries
`action_id` in its **own** frame, which is the protocol doing the job §6 argues is the protocol's.

### Terminal validation

At the final HEAD:

```text
cargo fmt --all --check                                               clean
cargo check --workspace --all-targets                                 clean
cargo clippy --workspace --all-targets --all-features -- -D warnings  clean
cargo test --workspace                                                144 passed, 0 failed
spike/server: cargo clippy --all-targets -- -D warnings               clean
spike/server: cargo test                                              2 passed, 0 failed
spike/server: cargo run -- --dump                                     inspected, quoted above
```

146 tests in total against 133 at the base: +13, and 14 existing assertions restated.

## 6. Decisions, discoveries and limitations

### DECISION — `ActionResult` does **not** gain the allocated `ActionId`

QUESTION Now that the world allocates the `ActionId` (Fix 3), OBS-1's reasoning said
correlation belongs to whoever paired request with answer. Does the answer now have to carry
the id?

EVIDENCE INSPECTED `contracts/src/action.rs` (`ActionResult`'s own rationale),
`kernel/src/dispatch.rs` (`World::dispatch(&intent, at) -> Dispatched`), F4 and OBS-1.

FINDING The premise of OBS-1 survives Fix 3, for a reason that is visible in the kernel's
signature: `dispatch` takes the `ActionIntent` *by reference* and returns `Dispatched`. The
party that calls dispatch is the party that allocated the id — after Fix 3 that is structurally
so, because `ActionIntent::allocate` is the only way an intent comes into being from a request.
So every producer of an `ActionResult` already holds the id the result answers, and a field here
would restate it on every in-process call while still not preventing a mismatched pair.

For the client, two further facts close the gap F4 worried about:

```text
request <-> answer    a protocol concern: the client's own correlation token in the frame,
                      which is F4's own recommendation (a) and must not enter the contract
answer  <-> facts     already carried: Accepted { events: Vec<EventId> } names the very
                      facts the request caused, so a client finds its own events without
                      ever learning the ActionId
```

DECISION Not added. Recorded here rather than absorbed silently, and the `ActionResult` doc
comment now states the case in its post-Fix-3 form instead of the pre-Fix-3 one.

IMPLEMENTED CONSEQUENCE The spike's reply frame now carries `"action_id"` next to `"result"` — the
protocol doing what the contract declines to do, in the one place where a client genuinely needs the
world's answer to a question the contract does not answer for it. That is the shape S11 should
inherit, and it cost one field in a protocol frame rather than a field in the kernel's vocabulary.

LIMITATION, carried to S11 One case remains genuinely open: an action that starts a `Process`
produces later events whose `Causation` is `Action(id)` or `Process(id)`, and a client that
never learned its `ActionId` cannot attribute those to its own request. `Accepted { events }`
does not cover them, because they do not exist yet when the answer is written. The fix is one
field in S11's protocol frame — the server holds the id it allocated — not a field in the
contract. S11 must decide it deliberately; this PR names it so that S11 cannot inherit it by
accident.

### DISCOVERY — the no-float rule shaped Fix 1's float rejection

The structural test scans `contracts/src` for the literal `f32`/`f64` outside comments. An
explicit `visit_f64` that rejected a float would therefore have failed it. Serde's default
`visit_f64` already refuses with `invalid type: floating point`, so the rejection is correct by
omission — and the test that asserts it is what keeps that from being an accident. Recorded
because the obvious implementation is the forbidden one.

### DISCOVERY — the spike's F2 trap now behaves differently, and that is the fix landing

`spike/server/src/vocabulary.rs` carries a deliberate trap: a `signage` component whose payload
holds `catalogue_id: 9007199254740999` and `id: 9007199254741001`, *neither of which is an
entity id*. They are still plain integers after this PR, and still corrupt in a Godot client.
That is correct and is the evidence that Fix 1 keys on the *type* rather than on the field name:
an encoder that protected them would be guessing. What changed is that a real `EntityId` in the
same position is now protected. The trap's `protected=false` line in the spike logs remains
true of the catalogue number and is no longer true of an id.

### DECISION — the kernel gains no `World::submit`, and the spike allocates for itself

QUESTION `ids.rs` says identity allocation belongs to the kernel. After Fix 3, who calls
`ActionIntent::allocate`?

EVIDENCE INSPECTED `kernel/src/dispatch.rs` (`World::dispatch(&intent, at)`, `EventIds` as the
kernel's own monotonic allocator for event identity), and this PR's approved scope.

FINDING The kernel already owns an identity allocator for events and would be the natural owner of
one for actions, but adding `World::submit(request)` is new kernel functionality rather than a call
site this change breaks — the frozen scope is "expect to update kernel call sites", and `dispatch`
compiles unchanged because `ActionIntent::new` is kept. Adding a submission entry point would also
decide where the action-identity counter lives, which is S3/S11's question and touches persistence
(an action counter that a replay must reproduce).

DECISION Not added. The contract now makes the only path from a request to an intent explicit, so
whichever layer sites the allocator cannot get it wrong. The spike allocates for itself, in
`SpikeWorld::allocate`, which is what a spike with no kernel is for. Carried to S11 as the one
remaining question: where the action-identity counter lives, and whether a replay must reproduce it.

### OUT OF SCOPE, FOUND WHILE WORKING — `overall.md`'s AC-13 text is stale

`a6ad5c7` corrected `AC-13` in `docs/MVP.md`, but
`.structured-coding/plans/mvp0/overall.md` still describes it at line 339 as "the *same*
`ActionIntent`" and at line ~358 as "byte identical". That contradicts `MVP.md`, and Fix 3 makes the
correct wording stronger still: `action_id` and `issued_at` are no longer in the value a client
submits at all, so of F6's three unavoidable differences only `actor_location` remains. Not corrected
here — `overall.md` is the parent ledger and is updated at merge by its owner (working rules §19,
§28). Reported so it is not inherited silently.

### LIMITATION — the binary side is tested with a recording codec, not with a real binary crate

No binary serde format is a dependency of this workspace, and adding one is a material decision
(working rules §7) that this PR does not need. The property under test is exactly
`is_human_readable() == false ⇒ a number`, so `contracts/tests/support` implements the smallest
serializer and deserializer that report `false` and record what they were handed. That is the
discriminator itself rather than a simulation of one; what it does *not* prove is that any
particular crate — bincode, postcard — encodes byte-identically to before. The id types call
`serialize_u64` on that path exactly as `#[serde(transparent)]` over `u64` did, so nothing about
the byte shape changed, but a real-codec parity test is a follow-up for whichever PR adopts a
binary format for persistence (S5).

### LIMITATION — no Godot client was re-run

See §2, TEST OWNERSHIP. The Godot evidence in `spike/evidence/` measured the defect; the spike's
`wire.rs` then worked around it at the protocol boundary, so a re-run would show the same green
result for a different reason and prove nothing about the contract. The honest verification that
Fix 1 works from the far side is a Godot round trip against a server with `wire.rs` *deleted*,
which is S11's acceptance test and is where risk R-9 already assigns it. This PR's claim is the
narrower one it can prove: the contract now emits a decimal string in JSON for every id, in
every position, including inside a payload.

---

## 7. Closeout

**Status: READY FOR OPERATOR REVIEW — NOT MERGED, NO PR OPENED.**

```text
branch          mvp0/pr-04-contract-fixes  (pushed to origin)
base            main @ a6ad5c7
executable head 0d2398d — this section is the only later commit and changes no code,
                so the validation above belongs to 0d2398d and to this head equally
```

Semantic commits:

```text
363f0a0  fix(contracts): 64-bit ids survive a JSON parser that has only doubles
37fda11  spike(renderer): delete wire.rs — the contract now protects its own ids
421f544  feat(contracts): an Observation can carry the relations an observer was shown
0d2398d  feat(contracts): a client submits an ActionRequest; the world makes it an intent
```

Public contract changes, for the reviewer to weigh as a set:

```text
EntityId, EventId, ActionId, ProcessId   JSON encoding changes from number to decimal string.
                                         Reading accepts both, so no fixture breaks. Binary
                                         encodings unchanged.
Observation                              gains relations: Vec<Relation>, Observation::relating,
                                         Observation::relations(). The wire shape changes.
ActionRequest                            new type, exported.
ActionIntent::allocate                   new constructor. ActionIntent::new unchanged.
ActionResult                             unchanged; documentation restated.
```

Not done, deliberately, each with its reason in §6: no `ActionId` on `ActionResult`; no
`World::submit` on the kernel; no `overall.md` edit; no Godot re-run; no real binary codec.
