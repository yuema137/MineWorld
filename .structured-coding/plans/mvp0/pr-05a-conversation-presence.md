# PR 05a — ConversationSystem and PresenceSystem

**DESIGN FROZEN** — the design is
[`step-05-vertical-slice.md`](step-05-vertical-slice.md) §2 (PR 05a) under §1.3's frozen
invariants, approved by the operator at `3d834f6`. This file is the **PR-level ledger**: what was
built inside that frozen scope, the evidence, the decisions taken autonomously, and the
limitations. It does not restate or amend the frozen design; where it records a bounded discovery,
it says so in §6.

---

## 1. Why this PR exists

The kernel has been able to compose systems since S3, and every system it has ever run has been a
test stub. This PR is the first two **real System Packs**, and it carries two jobs at once:

| Job | Why it is this PR's |
| --- | --- |
| The vertical slice needs conversation and perception | `AC-15` needs Alice to know somebody spoke to her, and a client to be told who is present and what it may do |
| Every later pack copies these two | They are the worked example, so what they demonstrate — ownership, projection, affordances without hardcoding — is what every later pack will do |

The second job is why the implementation is written to be read and why the architectural claims are
pinned by tests rather than asserted in prose.

## 2. Implementation contract

```text
PROJECT / PR:        MVP0 PR 05a — ConversationSystem and PresenceSystem
PRIMARY DESIGN DOC:  .structured-coding/plans/mvp0/step-05-vertical-slice.md §2 (frozen scope)
                     this file (live ledger: evidence, decisions, limitations)
RELATED / BINDING:   step-05 §1.3 frozen invariants
                     docs/MODULE_SPEC.md §3 (what a System Pack declares)
                     docs/ENGINEERING_RULES.md §§4–9 (spatial grounding, no rule in a renderer)
                     docs/ENGINEERING_STANDARDS.md §§7–8, 17–20 (ownership, amplification, tests)
                     docs/MVP.md §9.2 (Alice remembers by projection), AC-2, AC-12, AC-13
                     kernel/src/{system,view,access,dispatch,registry}.rs
                     contracts/src/{spatial,observation,action,event}.rs
IMPLEMENTATION BASE: main @ 3d834f6 — contracts and kernel merged, 144 tests green
BRANCH:              mvp0/pr-05a-conversation-presence

APPROVED SCOPE:
  systems/conversation/ and systems/presence/ as new crates, their tests and READMEs;
  workspace members and workspace dependency entries; systems/README.md; this ledger.
  NOT the server (05b), the World Pack or CLI (05c), the clients (05d).

FROZEN INVARIANTS:
  - INV-7: each pack writes only the components it owns; cross-domain effects travel as events
  - INV-12 binds the kernel, not a pack: domain vocabulary lives here and nowhere below
  - no kernel or contract change of any kind — if one seemed necessary, stop and report
  - no floats, no engine concepts
  - talk validates through SpatialRequirement::evaluate, never its own distance check
  - affordances come from asking each pack about its own actions, never from PresenceSystem
    knowing what any action is
  - AC-2 must hold with a real system: disabling ConversationSystem answers talk Unavailable
    with nothing else changing
  - fmt, check, clippy -D warnings and the full suite clean at the final HEAD

ENDPOINT AUTHORITY:
  implementation + local validation  authorized   source: operator instruction, this session
  semantic commits                   authorized   source: operator instruction, this session
  branch push                        authorized   source: operator instruction, this session
                                                  ("git push -u origin mvp0/pr-05a-…")
  PR creation                        FORBIDDEN    source: operator instruction, this session
                                                  ("No merge, no PR")
  merge                              FORBIDDEN    explicit operator authorization only

TEST OWNERSHIP:
  INTEGRATION owns everything that matters here. A System Pack's behaviour is only real through
  the pipeline — ActionIntent → route → validate → resolve → record → reduce — so every
  behavioural test dispatches against a composed World rather than calling a method
  (ENGINEERING_STANDARDS §§17, 20). There is no unit test of a getter, a constructor or an enum.
  STRUCTURAL owns two absences no behavioural test can hold: no other pack's vocabulary in
  systems/presence/src, and no floating point in either pack's sources.
  MUTATION owns the two claims that carry weight: the spatial refusal and the disable behaviour.
  GATE 1 (real LLM):     NOT REQUIRED — no LLM-facing semantics exist yet in this repository.
  GATE 2 (real runtime): NOT APPLICABLE — there is no server, client or database in this PR;
  the realistic integration path available at this commit is a headless composed world, which is
  what the tests run. The rendered half arrives with 05b and 05d.

NORMAL STOP CONDITION:
  PR 05a implemented, validated, committed and pushed. NO PR OPENED. NO MERGE.
```

## 3. What was built

Two crates, both workspace members, both depending only on `mineworld-contracts` and
`mineworld-kernel` (and `conversation` on `presence`).

### 3.1 `systems/presence` — `mineworld-presence`

```text
owns        Presence               where a person is: a Location (place + optional position)
            present-in             directed edge Person → Place
provides    arrive                 a person comes to be at a location
emits       arrived                PersonId + Location
subscribes  arrived                react writes Presence and maintains the edge
version     1
```

`observe(world, observer, at, providers) -> Observation` is the query half: perception is a read, so
it is a function over a composed `World` and not a fifth trait method. It fills `self_location`,
the entities present in the observer's place (the place itself first, then everybody in it in
`EntityId` order), the `present-in` edges among them, and the affordances.

### 3.2 `systems/conversation` — `mineworld-conversation`

```text
owns        ConversationHistory    bounded list of Heard { speaker, at, utterance }, max 32
provides    talk                   Talk { utterance }, target = the listener
emits       conversation-started   speaker + listener, Visibility::Participants
            spoke                  speaker + listener + utterance, Visibility::Place
subscribes  spoke                  react appends to the listener's history
depends on  presence               its positions are the ones talk is evaluated against
version     1
```

`talk_requirement()` is `same_place().within(3 000 mm).requiring_target_available()`. It is
evaluated in exactly one place in the crate — one call to `SpatialRequirement::evaluate` in
`validate` — and the same value is handed to a client unevaluated inside an `Affordance`.

### 3.3 Affordances without PresenceSystem knowing any action

The mechanism, which is the architectural core of the PR:

```text
each pack   impl InteractionProvider for its own system type
            fn offers(&self, world: &WorldRead, observer, target: Option<EntityId>) -> Vec<Offer>
            Offer::new::<A>(requirement)          the action type is read off the Action type
                 .with_target_available(bool)     the one judgement only the owner can make

presence    for each target in [None, ..everybody present]:
              for each provider: for each offer:
                drop it if world.systems().provider(offer.action_type()).is_none()   ← the kernel
                otherwise SpatialRequirement::evaluate(here, there, available)       ← one evaluator
                Ok  -> Affordance::available(...)
                Err -> Affordance::unavailable(..., reason)
```

Three consequences, each of which a test pins:

1. `systems/presence/src/` contains no other pack's vocabulary — checked structurally, because the
   claim is an absence;
2. disabling a pack removes its affordances everywhere, because the *route map* is what is asked;
3. the affordance a client is shown and the answer dispatch gives cannot disagree, because there is
   one requirement value and one evaluator on both paths.

## 4. Commit plan

| # | Commit | Implementation | Validation | LLM logic review |
| --- | --- | --- | --- | --- |
| 1 | `feat(systems): PresenceSystem — location, perception and the affordance seam` | `[x]` | `[x]` 10 tests, fmt/clippy/check clean | `[x]` §6 decisions D1, D5, D6, D7 recorded |
| 2 | `feat(systems): ConversationSystem — talk, its facts and Alice's projection` | `[x]` | `[x]` 11 tests incl. AC-2; 165 total | `[x]` §6 decisions D2, D3, D4 recorded |
| 3 | `docs(plan): PR 05a's ledger — evidence, decisions and mutation results` | `[x]` this file | `[x]` 6 mutations applied, 6 killed | `[x]` §5.4, §6 |

Split this way because the first commit is a complete, installable, tested pack on its own, and the
second is the pack that depends on it: a reviewer can read either alone.

## 5. Live evidence log

### 5.1 Baseline

```text
command:  cargo test --workspace        at main @ 3d834f6
result:   PASS — 144 tests, 0 failed
purpose:  the number every later count is measured against
```

### 5.2 Terminal validation at the final HEAD

```text
command:  cargo fmt --all --check
result:   PASS — no diff

command:  cargo check --workspace --all-targets
result:   PASS (subsumed by the clippy run below, which compiles the same targets)

command:  cargo clippy --workspace --all-targets --all-features -- -D warnings
result:   PASS — no warning in any crate

command:  cargo test --workspace
result:   PASS — 165 tests, 0 failed
          144 baseline + 21 new: 10 in systems/presence/tests/presence.rs (9 behavioural +
          1 structural) and 11 in systems/conversation/tests/conversation_and_presence.rs
          (10 behavioural + 1 structural)
```

### 5.3 The acceptance claims, with the tests that carry them

| §2 acceptance | Test | Evidence |
| --- | --- | --- |
| a `talk` from one Person to another is accepted, emits its event, and appears in the target's history | `a_talk_between_two_people_in_one_place_is_accepted_and_remembered` | `Accepted { events: 2 }`; event types `["conversation-started", "spoke"]`; `spoke` is `Visibility::Place(cafe)` with subjects `[bob]`, participants `[alice, bob]`, provenance `conversation`; Bob's history holds 1 entry (`speaker=alice`, `at=t3600`, `"good morning"`); Alice's history is absent |
| the same request from too far away is refused `TooFarAway`, and nothing is written | `the_same_request_from_too_far_away_is_refused_and_writes_nothing` | 9 000 mm apart **in the same place** → `Rejected(TooFarAway)`, 0 events, no history component, and a full state snapshot (identity, presence, relation graph) byte-identical to the snapshot taken before the request; moving Bob to 900 mm makes the identical request `Accepted` |
| disabling `ConversationSystem` makes `talk` return `Unavailable` with nothing else changing | `disabling_conversation_makes_talk_unavailable_with_nothing_else_changing` | two worlds from one `compose(Composition { conversation })`; `provider("talk")` is `Some(conversation)` vs `None`; answer `Accepted` vs `Unavailable`; history `Some` vs `None`; facts `["arrived","arrived","conversation-started","spoke"]` vs `["arrived","arrived"]`; affordances `["arrive","talk"]` vs `["arrive"]`; `EverythingElse` (entities, presence, relations, arrival facts) **equal**; conversation still installed, still owns its declared table, 2 writers; re-enabling restores the action |

Beyond the acceptance list, two tests carry weight on their own:
`the_affordance_a_client_is_shown_and_the_answer_dispatch_gives_are_the_same_answer` (the
`ENGINEERING_RULES.md` §8 property) and
`a_reply_continues_the_conversation_and_a_later_greeting_starts_another` (the one rule this pack
invents).

### 5.4 Mutation evidence

Six mutations, applied to the sources one at a time, full suite run with `--no-fail-fast`, then
reverted. All six compile and all six were killed. Script:
`scratchpad/mutate.py` (session-local, not part of the repository).

| Mutation | What it breaks | Tests that went red |
| --- | --- | --- |
| M1 `validate` skips the evaluator (`Ok(())`) | the spatial refusal itself | 4: too-far, wrong-room, affordance agreement, target-unavailable |
| M2 `INTERACTION_RANGE` 3 000 → 30 000 mm | the declared reach | 2: too-far, affordance agreement |
| M3 requirement drops `.requiring_target_available()` | availability before distance | 1: a person who has left the world |
| M4 perception drops the route-map filter | **the disable behaviour** | 2: the `AC-2` test, and presence's own disable test |
| M5 `react` also records on the speaker | the history records what was *heard* | 3: accepted-and-remembered, reply-continues, ownership counts |
| M6 arrival leaves the old `present-in` edge | one person in two places | 1: edge replacement |

Two mutation classes are **not expressible**, and that is the guarantee rather than a gap: a pack
writing another pack's component does not compile (`WorldView` gates writes on `OwnedBy<S>`), and a
pack emitting an undeclared event type is refused by dispatch at run time. The kernel already pins
both in `kernel/tests/compile_fail/`.

One methodological note: the first mutation run used plain `cargo test --workspace`, which
fail-fasts on the first failing test binary and therefore under-reported the tests each mutation
killed. Re-run with `--no-fail-fast`; the table above is the complete result.

## 6. Decisions, discoveries and limitations

### DECISION D1 — `PresenceSystem` provides an `arrive` action

**Question.** Step-05 §1.2 makes travel a non-goal, so should this pack provide any action at all?

**Evidence.** `ComponentStore::insert` requires the owning system's `WriteToken`, which only
`World::install` grants and which never leaves the kernel; `System::install` is handed a
`Declarations` that "exposes declaration and nothing else — no rows". There is therefore **no
mechanism at this commit by which anything other than `PresenceSystem` itself can put a position
into the world**, and a world where nobody has a location cannot exercise a spatial requirement at
all.

**Choice.** Provide one action, `arrive`, that records where a person is. It is not movement: no
`Process`, no simulated duration, no interpolation, and its documentation says so at length.
`SpatialRequirement::NONE`, because a world with no movement system has nothing to walk, and a
distance requirement here would refuse the only mechanism a World Pack (05c) has for placing its
people.

**Consequence.** `arrive` is unrestricted, and a world that wants arrival to obey rules gains them
by declaring them in `arrive_requirement()` — where `validate` already evaluates them — rather than
by adding a check somewhere else. Recorded as a limitation in L3.

### DECISION D2 — `ConversationSystem` declares a dependency on `PresenceSystem`

`talk`'s requirement is evaluated against `Location`s, and the authoritative location is
`Presence`, which this pack may read and may never write. Reading it needs the Rust type, hence a
crate dependency, and `MODULE_SPEC.md` §3 constraint 2 then requires the *declared* dependency too.

The alternative — evaluating against `ActionIntent::actor_location()`, which a client reports — was
rejected: `ENGINEERING_RULES.md` §8 puts the decision on the server, and the contract's own
documentation says a server "is free to ignore it". The report is ignored, and the ignoring is
documented at the call site.

When a movement system later owns position, the dependency moves there and this pack changes one
`use`. That is the intended shape, not a wart.

### DECISION D3 — the history is written in `react`, never in `resolve`

`MVP.md` §9.2 requires Alice's memory to be *a projection of the objective event log*. So `resolve`
writes nothing at all and `react`, subscribed to this pack's own `spoke`, is the single writer of
`ConversationHistory` — which the kernel explicitly supports ("how a system reduces the facts it
emitted into the state it owns"). The property this buys is the one a later cognition pack needs: a
world replayed from its log remembers the same conversations. The same discipline is applied to
`Presence` for the same reason.

### DECISION D4 — `conversation-started` is decided by a five-minute silence

§2 requires both `ConversationStarted` and `Spoke`, which forces a rule about when a conversation
*starts*. Emitting it on every `talk` would make it say nothing; emitting it once per pair forever
would be wrong the next day. So: an exchange starts a conversation unless either party remembers
hearing from the other within `CONVERSATION_GAP` (300 simulated seconds).

Both directions are consulted, and that is the subtle half — a reply is recorded on the *speaker's*
history, so a one-directional check would make every reply start a second conversation. Pinned by
`a_reply_continues_the_conversation_and_a_later_greeting_starts_another`, which fails if either
direction is dropped.

The constant is this pack's policy and becomes a configuration field in S7.

### DECISION D5 — `present-in` exists alongside `Presence`

The same fact at two resolutions, which invites the duplication objection. Kept because
`Observation` carries relations precisely so that containment and adjacency are *stated* rather than
inferred by a client (`spike/FINDINGS.md` F3), and because "who is in this place" should be a
question the relation graph answers. Both are written in one function, `PresenceSystem::react`, so
they cannot drift, and M6 is the mutation that proves the drift is caught.

### DECISION D6 — a payload this pack cannot read is `Rejection::System`, not one of the five

A malformed payload is a fact about the *request*, not about the world, and a client shown
`PreconditionFailed` would look for a precondition. Both packs answer
`System { code: "malformed-payload", detail }`, which the contract layer designed for exactly this
and which a client that does not recognize the code renders as "refused". It also exercises the
system-code path end to end, which the worked example should.

### DECISION D7 — an observation carries no events and no component records

`Observation::with_events` and `PerceivedEntity::with_components` are left empty. Deciding which
recorded facts a person *learned of* means a per-observer position in the log and `Visibility`
honoured over time, which belongs to S10 and S11; and exposing a component because it happens to
exist is how an observation becomes a window onto everything. Both are limitations (L1, L2), not
omissions.

### DISCOVERY — a System Pack cannot name its own failure in `KernelError`

**Audit.** `KernelError` is a closed enum in the kernel with `#[from] ContractError`, and
`ContractError` is a closed enum in the contract layer that is `Clone + PartialEq` and so cannot
carry a `serde_json::Error`. A pack's `resolve` and `react` return `Result<_, KernelError>`, so a
pack has no way to report "my own codec failed" or "my validation and my resolution disagree".

**How this PR lives inside it, without touching the kernel:**

```text
encode          serde_json::to_vec on a struct of integers, strings and typed ids cannot fail;
                one documented expect in each pack's codec, with the reasoning at the call site
decode          ContractError::{Action,Event}TypeMismatch — literally true (this payload cannot
                be read as A), and the only thing a caller can act on
validate        the ContractError's Display becomes the detail of Rejection::System (D6), so no
                information is lost where a client developer needs it
resolve         a case validate already excluded is reported as
                KernelError::ActionNotResolvedBySystem, which the kernel documents as meaning
                exactly "a bug in this system" — rather than panicking a running world
```

**Not escalated**, because nothing in the frozen scope is blocked and no kernel change is required.
Reported to the operator as a candidate for a future kernel decision: either a
`KernelError::System { system, detail }` variant, or an explicit statement that packs must map their
failures onto the existing vocabulary as above.

### DISCOVERY — destroying an entity emits no event, so no pack can reduce it

`World::destroy_entity` transitions the lifecycle and *returns* the cleared edges to its caller; it
emits nothing. A System Pack therefore cannot learn that a person is gone, and
`a_person_who_has_left_the_world_is_unavailable_rather_than_far_away` records the consequence
honestly: a destroyed Bob keeps his `Presence`, is still perceived, and is offered as
`TargetUnavailable` — correct as far as it goes, and not the same as being removed from the world's
perception.

Recorded rather than worked around. Inventing a `Destroyed` event in this pack would be this pack
declaring a fact about the kernel's own lifecycle, which is not its to declare.

### LIMITATION L1 — a controller cannot yet read Alice's history through an observation

`ConversationHistory` is exposed to nobody: perception exposes no component records (D7), so the
projection is readable only by code holding the `World`. That is enough for 05a's acceptance and for
a server (05b) that assembles a controller's context, and it is *not* enough for a client to display
"Alice remembers you". The step that needs it decides how — most likely a world-level perception
configuration naming which components an observer may see of itself and of others.

### LIMITATION L2 — no `PerceivedEvent` reaches a client from this pack

See D7. A client learns what happened from the facts a dispatch returns, which the transport (05b)
holds.

### LIMITATION L3 — `arrive` is unrestricted, and there is no movement

See D1. Anybody may be anywhere, immediately. The slice deliberately has no travel; when it does,
that system drives arrival and this pack keeps owning the state.

### LIMITATION L4 — line of access is declared nowhere and evaluated nowhere

`talk` does not require it, and the contract layer would return `Ok` if it did (`DD-7`): a wall
between two people does not yet stop a conversation. The requirement gains the clause on the day a
world geometry provider exists, in one place, with no client change.

### LIMITATION L5 — the memory bound and the conversation rule interact

`REMEMBERED_AT_MOST` is 32, and the "is this conversation ongoing" question is answered from what is
still remembered. A person spoken to by 32 others inside five minutes would have an exchange counted
as a new conversation. Documented on `last_heard_from`, and the safe direction: it records a fact
rather than omitting one.

### OUT OF SCOPE, FOUND WHILE WORKING — nothing in `docs/` contradicts this PR

`MODULE_SPEC.md` §3, `ENGINEERING_RULES.md` §§7–9 and `MVP.md` §9.2 were all checked against the
implementation as built. No specification needed correcting, and none was edited. `MVP_STATUS.md`
and the step and overall documents are the planning session's to update after merge, per this
effort's convention.

## 7. Closeout

```text
final executable HEAD:   see the branch tip of mvp0/pr-05a-conversation-presence
validation at that HEAD: fmt PASS · clippy -D warnings PASS · cargo test --workspace PASS (165)
mutation evidence:       6 applied, 6 killed (§5.4)
PR:                      NOT OPENED — forbidden by operator instruction this session
merge:                   NOT PERFORMED — requires explicit operator authorization
state:                   PR 05a CLOSED / AWAITING OPERATOR REVIEW
```
