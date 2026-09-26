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
| 2 | Fix 1: delete the spike's `wire.rs` | `[ ]` | `[ ]` | `[ ]` |
| 3 | Fix 2: relations on `Observation` | `[ ]` | `[ ]` | `[ ]` |
| 4 | Fix 3: `ActionRequest` and `ActionIntent::allocate` | `[ ]` | `[ ]` | `[ ]` |

## 5. Live evidence log

### Baseline

```text
command:  cargo test --workspace   @ a6ad5c7
result:   133 passed, 0 failed   PASS
```

### Fix 1 — id encoding

```text
command:  cargo test --workspace   @ commit 1 (fe/ids)
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
confirmed back at 139 passing afterwards.

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

### Fix 2 — relations on `Observation`

PENDING.

### Fix 3 — `ActionRequest`

PENDING.

### Terminal validation

PENDING.

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
