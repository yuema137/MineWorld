# PR S27-b — `group-activity` offers its four empty-payload actions complete

## DESIGN FROZEN 2026-10-14 (primary session; operator QS27b-1; rulings §14.1)

```text
Design revision:        revision 2 (2026-10-14), FROZEN: revision 1 with §14.1's rulings applied
Author:                 S27-a/b/c PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s27abc, branch docs/s27abc-designs,
                        planning base origin/main @ df77b27
Approved by / evidence: the operator answered QS27b-1 directly in the primary session on 2026-10-14
                        ("允许，只限这个原因"); the primary session ruled QS27b-2, accepted B2's added
                        stale-section stop rule, and froze this design itself (§14.1)
Implementation base:    origin/main at B0; B0 records the commit
Start condition:        now (step-27 §11.4: "Now. Independent"), on the primary session's dispatch, after
                        this design is frozen
Execution contract:     §12, filled at freeze
Implementer:            Haiku-class (step-27 §11.4)
Lifecycle:              DRAFT
```

**Effort:** `mvp0` · **Step:** [`step-27-agent-players.md`](step-27-agent-players.md) §2.2 F-S27-2, §5.5,
§11.2 "PR S27-b", §11.4 · **Parent:** [`overall.md`](overall.md) · **Siblings:**
[`pr-s27a-seat-grants-budget.md`](pr-s27a-seat-grants-budget.md),
[`pr-s27c-player-library.md`](pr-s27c-player-library.md) (independent of both) · **Binding parents:**
`CLAUDE.md` §§2, 4 (items 1, 5, 8, 9, 13); `docs/DECISIONS.md` `ARC-34` (`:2482`), `ARC-35`;
`docs/CORE_CONCEPTS.md` §15 (affordances); `.structured-coding/standards.md` (the baseline reason-line rule).

**Decision records.** None new (step-27 §11.4: "none (dated note under `ARC-34`)").

---

## 0. Binding inputs, exactly restated

- **F-S27-2 (step-27 §2.2):** "Four `group-activity` offers have an empty payload but are not offered
  complete (A-23), so a generic client cannot attempt them without knowing the pack." Resolution:
  "`group-activity` (the owner) offers them complete (`ARC-34`). Additive; S27-b. `invite` stays name-known
  (its `kind` is free text)."
- **Scope (step-27 §11.2):** `systems/group-activity/src/perception.rs`: `accept-invitation`,
  `decline-invitation`, `join-group-activity`, `leave-group-activity` offered with `Offer::complete` and their
  `{}` payload; the pack's tests; any golden observation that lists them; a dated note under `ARC-34`.
- **CP-b (step-27 §11.2):** "A plain SDK client accepts an invitation with `offers.attempt` only;
  `milestone_c`, `ac1_composability` and both 300-day digests are unchanged (offers are not facts; if a digest
  covers observations, the change is reviewed and re-recorded in the same PR)."
- **Adversarial (step-27 §11.2):** "Review plants a payload `{ "x": 1 }`; the pack's own decode test fails."

This design finds that the last two statements do not hold as written (F-S27b-1, F-S27b-2, §3.2); both were
ruled on 2026-10-14 (§14.1: QS27b-1 by the operator, QS27b-2 by the primary session).

---

## 1. Goal

Make the four actions of `group-activity` whose request carries nothing — answering an invitation yes or no,
joining what someone is doing, leaving one's own activity — **complete affordances** (`ARC-34`), so a client
that was never compiled against `group-activity` (the agent adapter of S27-c/d, a plain SDK client, the 3D
client) can attempt them from the observation alone. Nothing else about the pack changes: the same actions,
the same requirements, the same verdicts, the same validation at dispatch.

## 2. Scope

### 2.1 In scope

1. `systems/group-activity/src/perception.rs`: the four offers built with `Offer::complete` instead of
   `Offer::new`, same requirement, same availability, same refusal; the module doc table says so.
2. `systems/group-activity/tests/group_activity.rs`: one new test (§7).
3. `sdk/python/tests/test_real_server.py`: one new real-server test, CP-b (§7).
4. `docs/DECISIONS.md`: a dated note under `ARC-34`.
5. `systems/group-activity/README.md`: one line naming which offers are complete.
6. `scripts/baselines.txt`: only the `group-activity` world sections §5.3 allows (QS27b-1, ruled).

### 2.2 Not in scope

- `invite` stays incomplete: its `kind` is free text (`systems/group-activity/src/action.rs:24-45`).
- No change to `action.rs`, `system.rs`, `interactions.rs`, the requirements, any other System Pack, the
  kernel, the contracts, the server, the protocol, the SDK's source, the 2D or 3D client, or
  `cognition/rule-controller`.
- `#[serde(deny_unknown_fields)]` on the four empty action structs (F-S27b-2): a follow-up, not here (QS27b-2).
- Removing the now-unreached composer entries for the four types from `clients/2d/scripts/intents.gd:18,77`
  (F-S27b-3): a follow-up; they stay correct as a fallback.

### 2.3 Dependencies

None on S27-a or S27-c. S27-d's invitation step (if it has one) depends on this PR (step-27 §11.4).

---

## 3. Audit (`origin/main @ df77b27`)

### 3.1 Source anchors

| # | Fact | Where |
| --- | --- | --- |
| B-1 | The four offers are built with `Offer::new`: `leave-group-activity` (no target) at `:51`, `accept-invitation` at `:91-95`, `decline-invitation` at `:97-100`, `join-group-activity` at `:105-109`. `invite` at `:80-83` | `systems/group-activity/src/perception.rs:49-111` |
| B-2 | The four action types are empty structs deriving `Serialize, Deserialize` with no `deny_unknown_fields`: `AcceptInvitation {}` `:49`, `DeclineInvitation {}` `:58`, `JoinGroupActivity {}` `:71`, `LeaveGroupActivity {}` `:80` | `systems/group-activity/src/action.rs:47-85` |
| B-3 | `Offer::complete::<A>(&A, SpatialRequirement) -> Result<Offer, serde_json::Error>` sets the payload to `serde_json::to_value(action)`; `with_target_available` and `refused` keep the payload | `systems/presence/src/interaction.rs:183-191`, `:200-215` |
| B-4 | `answered(offer, permitted)` applies `Offer::refused` on a `Rejection` and returns the offer otherwise; a refused complete offer still carries its payload | `systems/group-activity/src/interactions.rs:202-207`; `systems/presence/src/interaction.rs:209-215` |
| B-5 | The model to copy: economy offers `Offer::complete(&Buy::new(..), ..).expect("a buy encodes").with_target_available(..)` | `systems/economy/src/offer.rs:44-48` |
| B-6 | Payloads are decoded with `serde_json::from_slice`; an unknown field in a `{}` payload is ignored by serde for these structs | `systems/group-activity/src/codec.rs:21-27`; `system.rs:354-360` |
| B-7 | The pack's offer test finds affordances with `offered(view, action, target)` and checks availability only; it never checks a payload | `systems/group-activity/tests/group_activity.rs:302-397` |
| B-8 | Test support: `Cafe::new()`, `cafe.observation(observer, when)`, `cafe.submit(when, actor, target, record)`, `cafe.invite/accept/join/leave`, `cafe.participation(p)`, `cafe.invitations(p)`; `t(offset)` | `systems/group-activity/tests/support/mod.rs:138-243` |
| B-9 | `Affordance::request(actor, encode) -> Option<ActionRequest<Q>>` builds the request of a complete affordance with its own action type, target and payload | `contracts/src/observation.rs:257-271` |
| B-10 | **The paced rule controller attempts available complete affordances**: on 20 of 100 consults that reach its `offered` band, it submits one available complete affordance chosen by a seeded draw. The band runs after the invitation answer, the reply, the agenda walk and the social band | `cognition/rule-controller/src/offered.rs:1-56`; `cognition/rule-controller/src/paced.rs:138-175` |
| B-11 | `mineworld run` and `mineworld server --town` drive every resident with `PacedRuleController` | `tools/cli/src/run.rs:125`; `tests/acceptance/tests/complete_affordances.rs:10-13` |
| B-12 | Three committed long-run baselines cover worlds that install `group-activity`: `lakeside`, `market-town`, `social-cafe` (`bodies-yard` does not install it). The nightly checks them; a PR that changes a world's behaviour updates its section with a `reason` line | `scripts/baselines.txt:1-30`; `worlds/{lakeside,market-town,social-cafe}/world.yaml` (`- group-activity`); `.github/workflows/nightly.yml:218-250` |
| B-13 | No golden frame and no fixture lists these four affordances (`grep` over `*.json`, `*.py`, `*.txt`, `*.log`: no match) | `server/tests/frames/`, `sdk/python/tests/`, `clients/protocol/evidence/` |
| B-14 | The 2D client sends any complete affordance unchanged (`submit_offered`) and composes the four by name only when the affordance is incomplete; both paths send the same `{}` | `clients/2d/scripts/menu.gd:120-136`; `clients/2d/scripts/intents.gd:61-85` |
| B-15 | The SDK's real-server tests may name pack vocabulary as fixtures; the pattern for a complete affordance is `offers.attempt(frame.observation, offer)` | `sdk/python/tests/test_real_server.py:1-5`, `:310-349` |
| B-16 | In market-town, `alice` (6000, 8000) and `bob` (4500, 6100) both start in the café, 2.42 m apart; `invite` reaches 3 m (`INVITE_RANGE`); market-town configures no `group-activity` rules | `worlds/market-town/people/alice.yaml:38-45`, `bob.yaml:22-26`; `systems/group-activity/src/action.rs:22`; `worlds/market-town/configure/` |
| B-17 | The SDK fixture `Server(world)` starts `mineworld server <world>` with no in-server controller, so every seat is free | `sdk/python/tests/realserver.py:50-88` |

### 3.2 Findings

| ID | Finding | Consequence | Resolution |
| --- | --- | --- | --- |
| **F-S27b-1** | CP-b says the 300-day digests are unchanged because "offers are not facts". Offers are not facts, but they **are an input of the paced rule controller** (B-10): a new available complete affordance enters the draw of `offered::attempt`. A member of an activity is now offered `leave-group-activity` complete on every consult, so the `offered` band can make residents leave; non-members are offered `join-group-activity` complete beside members. The residents' histories in `lakeside`, `market-town` and `social-cafe` will very likely move (B-11, B-12). | The step's "unchanged" claim is falsified by audit before any measurement. Moving a baseline is a behaviour change of the residents (`standards.md` baseline rule). | **Operator question QS27b-1** (§14). B2 measures first and stops on any moved column unless QS27b-1 authorizes exactly that cause. |
| **F-S27b-2** | The step's planted mutation "a payload `{ "x": 1 }`; the pack's own decode test fails" is not discriminating: the four structs ignore unknown fields (B-2, B-6), so `{"x":1}` decodes and is accepted, and `Offer::complete` takes a typed value, so `{"x":1}` cannot be planted through it at all. | The step's adversarial criterion would be "passed" by a test that proves nothing. | Replaced by M-b-1 … M-b-3 (§7), which each turn a named test red. Recorded for the primary session (QS27b-2). |
| **F-S27b-3** | After this PR the 2D client never reaches its by-name composer for the four types (B-14): the menu takes the complete path first. | Dead-but-correct code in `intents.gd`; `tests/acceptance/tests/client_rules.rs:40-80` still lists the literals. | Not touched here; listed in §10 as a follow-up for the 2D client's owner. |

### 3.3 What does not exist

No test anywhere checks that a `group-activity` affordance carries a payload. No baseline reason line for
`group-activity` offers.

---

## 4. Reuse

Nothing new. `Offer::complete` is presence's existing API (B-3); the test uses `Affordance::request` (B-9).

---

## 5. Design

### 5.1 The change (`perception.rs`)

Each of the four `Offer::new::<A>(req)` becomes `Offer::complete(&A {}, req).expect("an empty payload encodes")`,
keeping every chained `.with_target_available(..)` and every `answered(.., permitted(..))` wrapper exactly as
it is. The `expect` follows the economy pattern (B-5): a struct with no fields always encodes to `{}`.

```text
leave-group-activity   target none     Offer::complete(&LeaveGroupActivity {}, leave_requirement())
accept-invitation      target inviter  Offer::complete(&AcceptInvitation {}, accept_requirement())
decline-invitation     target inviter  Offer::complete(&DeclineInvitation {}, decline_requirement())
join-group-activity    target member   Offer::complete(&JoinGroupActivity {}, join_requirement())
invite                 target invitee  unchanged: Offer::new::<Invite>(invite)  (free-text kind)
```

The module doc table (`perception.rs:3-12`) gains one line: `complete    every offer but invite: its
payload is the empty object` and the import list gains nothing (the four types are already imported, `:28-32`).

### 5.2 Wire effect

Each of the four affordances gains `"payload": {}` in every observation that lists it. Nothing else in any frame
changes. A client that ignores `payload` sees no difference; a client that uses it may now attempt the action
generically. The 2D client's request bytes for these actions are unchanged (B-14).

### 5.3 Baseline and digest impact (decided before measuring)

| Artefact | Expected | Rule |
| --- | --- | --- |
| `scripts/baselines.txt` `[world bodies-yard]` | unchanged (no `group-activity`) | any move: **material stop** |
| `[world lakeside]`, `[world market-town]`, `[world social-cafe]` | may move (F-S27b-1) | moved only for the cause "the paced rule controller's `offered` band draws among the four new complete affordances"; allowed (QS27b-1, ruled 2026-10-14: "允许，只限这个原因"), re-recorded with the reason line below; any other cause: **material stop** |
| `summary-*`, `facts-300s`, `journal-300s` of a moving world | all may move together | a world whose `summary-*` moves but whose cause is not the one above: material stop |
| Golden frames (`server/tests/frames/`), SDK golden tests | unchanged (B-13) | any change: material stop |
| `ac1_composability`, `milestone_c` | pass unchanged (they do not pin these worlds' social histories; audited `tests/acceptance/tests/ac1_composability.rs`, `tools/cli/tests/milestone_c.rs`) | a failure: audit; if caused by F-S27b-1 behaviour, record and stop |

The reason line, exactly (the file's format, `scripts/baselines.txt:2-4`), one per moved world section:

```text
reason S27-b #<PR>: group-activity offers accept-invitation, decline-invitation, join-group-activity and leave-group-activity complete (F-S27-2, ARC-34 note 2026-10-14); the paced rule controller's offered band now draws among them, so residents' social histories moved (QS27b-1)
```

Sanity of the moved worlds (fixed now, checked in B2, reported, never tuned): over the long profile's 300-day
run of each moved world, the count of `group-activity-started` facts is **> 0** and every seat is still active
in every bucket (the `run` output's existing lines). A world where activities stop starting is a material stop:
the change would have broken the town's social life, which no ruling of QS27b-1 covers.

---

## 6. Invariants

| Id | Invariant | Maps to | Checked by |
| --- | --- | --- | --- |
| I-b-1 | The four affordances carry exactly the payload `{}` the pack accepts; `invite` carries none | INV-S27-3 (same rules: a generic requester is answered as a named one), F-S27-2 | B1 test (a) |
| I-b-2 | Requirement, availability and refusal of every offer are unchanged | INV-S27-3 | B1 test (b); the existing `offers_and_disclosure_say_what_the_pack_knows_and_no_more` still passes |
| I-b-3 | A request built from the affordance alone is accepted exactly as the named one is | INV-S27-1 (one path), INV-S27-3 | B1 test (c); CP-b Python test |
| I-b-4 | No file outside §2.1 changes; no System other than `group-activity` changes | INV-S27-8 (removable: nothing agent-specific enters a pack) | `git diff --stat` in B3 |
| I-b-5 | Nothing in this PR names MCP, agent or adapter | INV-S27-9 | `git diff` review in B3 |

---

## 7. Test ownership and mutations

| Layer | Owns |
| --- | --- |
| Static | `cargo fmt`, `cargo clippy -p mineworld-group-activity -D warnings`; ruff/pyright on the SDK test file |
| Unit / pack integration (Rust, no server) | `systems/group-activity/tests/group_activity.rs` new test `the_four_answers_are_offered_complete_and_attempted_unchanged`: (a) payload of each of the four is `json!({})` and `invite`'s is `None`; (b) each keeps its availability; (c) each request built by `Affordance::request` from the observation alone is accepted and has its effect (accept: both have a `Participation`; decline: the invitation is gone; join: the joiner has a `Participation`; leave: the leaver has none) |
| Real lifecycle (Python, real binary) | `sdk/python/tests/test_real_server.py` new test `test_an_invitation_is_answered_and_left_through_complete_affordances` (CP-b): market-town, `bob` invites `alice` with `offers.request`; `alice` accepts and later leaves with `offers.attempt` only |
| Long-run behaviour | `ci_parity.py record --profile long` + `baseline check` (B2), and the nightly afterwards |
| Gate 1 (real LLM) | NOT REQUIRED: no LLM-facing change |
| CI impact | `systems/group-activity/**` → Rust full fallback in the selector; `sdk/python/tests/**` → the `python` job on three OSes |

Mutations (plant, see red with the test's name, revert, see green; never commit one):

| Id | Mutation | Must turn red |
| --- | --- | --- |
| M-b-1 | `accept-invitation` built with `Offer::new::<AcceptInvitation>(..)` again | `the_four_answers_are_offered_complete_and_attempted_unchanged` (a) |
| M-b-2 | `invite` built with `Offer::complete(&Invite::new(ActivityKind::new("coffee")…), invite)` | the same test, the `invite` assertion of (a) |
| M-b-3 | `leave-group-activity` built complete with `accept_requirement()` in place of `leave_requirement()` | the same test (b): `leave`'s `requirement()` no longer equals `leave_requirement()` (same-place vs none) |

---

## 8. Acceptance criteria

| Id | Criterion |
| --- | --- |
| AB-1 | In a hand-built café, each of the four affordances, when offered, carries `payload == {}`; `invite` carries none. |
| AB-2 | For each of the four, `Affordance::request(observer, encode)` submitted unchanged is `Accepted` and has the effect of AB-test (c). |
| AB-3 | Against the real market-town binary, a plain SDK client answers an invitation and leaves the activity with `offers.attempt` only; both are `Answered` with `accepted`. |
| AB-4 | `cargo test -p mineworld-group-activity`, `-p mineworld-rule-controller`, and the listed `mineworld-cli` tests (B2 step 3) pass. |
| AB-5 | §5.3 holds against B2's reference (§14.1 note): `bodies-yard` unchanged; any moved section is moved only for F-S27b-1's cause (QS27b-1, ruled), with the reason line; each moved world still starts activities. |
| AB-6 | M-b-1, M-b-2 (and M-b-3 or its recorded equivalent) each turned a named test red. |

---

## 8a. Integration checkpoint CP-b (as this design restates it)

AB-3 plus AB-5. The step's "digests unchanged" half is replaced by AB-5, as ruled under QS27b-1 (F-S27b-1, §14.1).

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately;
`[x]` needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason. Targeted
validation per commit; the PR's CI is the one full run (test rules §§8–9). Written for a small-model
implementer: numbered steps, exact paths, exact commands.

### 9.0 The procedure for every commit

Run every command from the worktree root `/Users/yuema137/mineworld-worktrees/impl-s27b`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full,
   before editing it.
2. Make the numbered implementation steps, in order. Do not do work that belongs to a later commit.
3. Run the commit's validation commands. Record for each: the command, the test count, the wall time, and
   PASS / FAIL / INCONCLUSIVE (a test not run is not a pass).
4. For each mutation the commit lists: plant it, run the named test, record that it turned red (with the
   failing test's name), revert it, run the test again green. Never commit a planted mutation.
5. Static checks:
   - Rust: `cargo fmt --all --check`; `cargo clippy -p mineworld-group-activity --all-targets -- -D warnings`.
   - Python (B1 only): `uv run --locked ruff check sdk/python`; `uv run --locked ruff format --check
     sdk/python`; `uv run --locked pyright sdk/python`.
   - Markdown: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
6. Review: answer each review question of the commit in §15 in one or two sentences.
7. `git status --short` and `git diff --stat`: every changed path must be in the commit's **Files** list. If
   another path changed, revert it or stop and record why.
8. Update §15 (ledger row, evidence, findings) and the handoff file in the same commit.
9. Stage by explicit path (`git add <path> …`, never `git add -A`), then `git commit -m "<message>"` with the
   message given, then a blank line and the attribution line your session specifies. Separate commands; no
   heredoc, no `sed -i`, no `python3 -c`.
10. Stop and report (a material stop, §12) instead of improvising if a step cannot be done as written.

#### B0 — Open the ledger and handoff (Markdown only)

- **Goal.** Start S27-b against the frozen design. **Files:** this file (§15);
  `.structured-coding/plans/mvp0/handoff-s27b.md` (new).
- Steps:
  1. `git log --oneline -1`; record the base commit in §15 and in the handoff.
  2. Confirm this file's header reads `DESIGN FROZEN` and §12 is filled. If not: stop.
  3. Copy the QS27b-1 and QS27b-2 rulings from §14.1 into the handoff verbatim.
  4. Create `handoff-s27b.md` with: project/PR, primary design, binding docs, branch, base, HEAD,
     working-tree fingerprint (`git status --short` output), approved scope, frozen invariants, endpoint
     authority with sources (copy §12), sequence B0–B3, budget, checkpoint, next actions, stop conditions.
- [ ] Implementation: steps 1–4.
- [ ] Validation: both Markdown checks.
- [ ] Review: no authority line widened; the rulings copied verbatim.
- **Commit message:** `docs(plan): S27-b — open the ledger and handoff`

#### B1 — The four offers complete, with their tests and the ARC-34 note

- **Goal.** F-S27-2 resolved at its owner. **Files:** `systems/group-activity/src/perception.rs`,
  `systems/group-activity/tests/group_activity.rs`, `systems/group-activity/README.md`,
  `sdk/python/tests/test_real_server.py`, `docs/DECISIONS.md`, this file (§15), `handoff-s27b.md`.
- Steps:
  1. Read `systems/economy/src/offer.rs:30-50` (the pattern) and `systems/presence/src/interaction.rs:153-215`.
  2. In `perception.rs`, replace line 51's `Offer::new::<LeaveGroupActivity>(leave_requirement())` with
     `Offer::complete(&LeaveGroupActivity {}, leave_requirement()).expect("an empty payload encodes")`.
  3. Replace `Offer::new::<AcceptInvitation>(accept_requirement())` (`:92`) with
     `Offer::complete(&AcceptInvitation {}, accept_requirement()).expect("an empty payload encodes")`, keeping
     `.with_target_available(active)` after it and the `answered(…, permitted(…))` around it.
  4. Replace `Offer::new::<DeclineInvitation>(decline_requirement())` (`:98`) the same way
     (`&DeclineInvitation {}`), keeping `.with_target_available(active)`.
  5. Replace `Offer::new::<JoinGroupActivity>(join_requirement())` (`:106`) the same way
     (`&JoinGroupActivity {}`), keeping its `.with_target_available(…)` and `answered(…)`.
  6. Do **not** change line 81's `Offer::new::<Invite>(invite)`.
  7. In the module doc block (`:3-12`), add after the `offers` rows the line
     `//! complete    every offer but invite, whose kind is free text: the payload is the empty object (ARC-34)`.
     Remove now-unused imports only if `cargo clippy` says so.
  8. In `tests/group_activity.rs`, add the test `the_four_answers_are_offered_complete_and_attempted_unchanged`
     after `offers_and_disclosure_say_what_the_pack_knows_and_no_more`. Use only the support helpers of B-8
     and `Affordance::request` (B-9). Shape:
     - (setup) `let mut cafe = Cafe::new(); cafe.invite(10, cafe.alice, cafe.bob);`
     - (a, b) `let bobs = cafe.observation(cafe.bob, 11);` find `accept-invitation` and `decline-invitation`
       targeting `cafe.alice` with `offered(..)`; assert each `.payload() == Some(&serde_json::json!({}))`
       and `.is_available()`; find `invite` targeting `cafe.alice` and assert `.payload().is_none()`.
     - (c accept) build `let request = accept.request(cafe.bob, |v| serde_json::to_vec(v).expect("encodes")).expect("complete");`
       then `let (result, _) = cafe.submit(t(12), request.actor(), request.target(), request.payload().clone());`
       assert accepted (the file's `accepted(&result)` helper, `:20`), and that both Alice and Bob have a
       `Participation` (`cafe.participation(..).is_some()`).
     - (c join) `let carols = cafe.observation(cafe.carol, 13);` find `join-group-activity` targeting
       `cafe.bob`, assert payload `{}`, submit it the same way at `t(14)`, assert accepted and Carol has a
       `Participation`.
     - (c leave) `let carols = cafe.observation(cafe.carol, 15);` find `leave-group-activity` with target
       `None`, assert payload `{}`, submit at `t(16)`, assert accepted and Carol has no `Participation`.
     - (c decline) on a fresh café: `let mut other = Cafe::new(); other.invite(10, other.alice, other.bob);`
       find `decline-invitation` targeting `other.alice` in `other.observation(other.bob, 11)`, assert
       payload `{}`, submit it at `t(12)` the same way, assert accepted, and assert
       `other.invitations(other.bob).and_then(|held| held.from(person(&other, other.alice)).cloned()).is_none()`
       (`Invitations::from` takes a `PersonId`, `src/component.rs:99`; the test file's `person(cafe, entity)`
       helper, `tests/group_activity.rs:37`, makes one).
     - (b, for every one of the four) also assert the affordance's `requirement()` equals the pack's named
       function: `accept_requirement()`, `decline_requirement()`, `join_requirement()`, `leave_requirement()`
       (all exported, `src/lib.rs:57-60`), so the change provably kept each requirement.
     - If a helper name differs from the above, use the real one and record the difference in §15 (bounded).
  9. In `sdk/python/tests/test_real_server.py`, add `test_an_invitation_is_answered_and_left_through_complete_affordances`
     after `test_a_complete_affordance_is_attempted_unchanged_and_round_trips` (`:310`), with a docstring
     "CP-b (pr-s27b): bob invites alice by name; alice answers and leaves through complete affordances only."
     Shape, using the file's own helpers (`seated`, `until`, `run`, `offers`):
     - `server = mineworld("market-town")`; `alice, _ = await seated(server, "alice")`; `bob, _ = await
       seated(server, "bob")`.
     - Bob: wait with `until(bob, "invite to alice available", …)` for an affordance with `action_type ==
       "invite"`, `target == alice.observer`, `available`. Submit `offers.request(frame.observation,
       ActionTypeId("invite"), target=alice.observer, payload={"kind": "coffee"})`; assert `Answered` and
       accepted.
     - Alice: wait for an available affordance with `action_type == "accept-invitation"`, `target ==
       bob.observer` and `payload is not None`; submit `offers.attempt(frame.observation, affordance)`;
       assert `Answered` and `isinstance(outcome.result, Accepted)`.
     - Alice: wait for an available affordance `leave-group-activity` with `target is None` and a payload;
       `offers.attempt` it; assert accepted.
     - Close both sessions with `async with` exactly as the neighbouring tests do.
  10. In `docs/DECISIONS.md`, at the end of the `ARC-34` record (the paragraph before the next `## ` heading
      after `:2482`), add a dated note:
      `**Note, 2026-10-14 (S27-b, F-S27-2).** group-activity offers accept-invitation, decline-invitation,
      join-group-activity and leave-group-activity complete, with the empty payload each accepts, so a
      requester that does not know the pack can attempt them (step-27 §5.5). invite stays incomplete: its kind
      is free text. Consequence recorded in pr-s27b §3.2 F-S27b-1: the paced rule controller's offered band now
      draws among them.`
  11. In `systems/group-activity/README.md`, after the `depends on` block, add one line: "Every offer but
      `invite` is complete (`ARC-34`): its payload is the empty object, so any client can attempt it from
      the observation alone."
- Validation:
  - `cargo test -p mineworld-group-activity` → record count, time, PASS/FAIL.
  - `cargo build -p mineworld-cli` then
    `uv run --locked pytest sdk/python/tests/test_real_server.py -k invitation_is_answered` → record.
  - Mutations M-b-1, M-b-2, M-b-3 (§7) against `cargo test -p mineworld-group-activity
    the_four_answers_are_offered_complete_and_attempted_unchanged`.
- [ ] Implementation: steps 1–11.
- [ ] Validation: the two commands, the three mutations, the static checks of §9.0 step 5.
- [ ] Review: (1) is every `with_target_available` and `answered` wrapper unchanged? (2) does any expected
  value in the test come from the code under test rather than from `json!({})` and the pack's own
  effects? (3) does the Python test reach `accept-invitation` only through `offers.attempt`?
- **Commit message:** `feat(group-activity): offer accept, decline, join and leave complete (S27-b, F-S27-2)`

#### B2 — Behaviour impact: targeted suites and the long-run baselines

- **Goal.** Measure F-S27b-1 and apply §5.3. **Files:** this file (§15), `handoff-s27b.md`, and
  `scripts/baselines.txt` only for §5.3's allowed cause (QS27b-1, ruled).
- Steps:
  1. `cargo test -p mineworld-rule-controller` → record.
  2. `cargo test -p mineworld-acceptance --test ac1_composability --test complete_affordances` → record.
  3. `cargo test -p mineworld-cli --test social_composition --test social_sections --test interaction_runs
     --test market_town --test milestone_c --test routines` → record each test binary's result.
  4. If step 3 fails: read the failing assertion. If it fails because residents now leave/join/decline
     differently (F-S27b-1), record it and **stop** (material: a test's expectation of town behaviour). If it
     fails for any other reason, it is a defect of B1: fix B1's change, re-run, record.
  5. **Choose the reference** (coordinator note of 2026-10-14, §14.1: `main`'s baselines are stale, F-12d-1).
     Read `git log --oneline -- scripts/baselines.txt` on the base. If the base contains the post-12d re-record
     of `main` (its reason lines name 12d / F-12d-1), the reference is `scripts/baselines.txt` as committed.
     Otherwise the reference is a long record of **this PR's base commit**: `git worktree add
     target/scratch-s27b/base <B0's base SHA>`, `cargo build --release -p mineworld-cli --manifest-path
     target/scratch-s27b/base/Cargo.toml`, and `python3 scripts/ci_parity.py record --binary
     target/scratch-s27b/base/target/release/mineworld --profile long --out target/scratch-s27b/base.long`
     (≤ 20 min); remove that worktree afterwards (`git worktree remove target/scratch-s27b/base`). Record which
     reference was used.
  6. `cargo build --release -p mineworld-cli`; `mkdir -p target/scratch-s27b`;
     `python3 scripts/ci_parity.py record --binary target/release/mineworld --profile long --out target/scratch-s27b/s27b.long`
     (≤ 20 min). Compare with the reference: committed baselines → `python3 scripts/ci_parity.py baseline check
     target/scratch-s27b/s27b.long`; the base record → `python3 scripts/ci_parity.py compare
     target/scratch-s27b/base.long target/scratch-s27b/s27b.long` (read the script's usage first; if `compare`
     takes other arguments, use its documented form and record it). Read the verdict lines; record every moved
     key per world. A key that differs between the committed baselines and the base record is F-12d-1's
     staleness, not this PR's: never attribute it to S27-b and never re-baseline it here.
  7. Apply §5.3 to the keys step 6 attributes to this PR: `bodies-yard` moved → **stop**. A `group-activity`
     world moved → (QS27b-1 ruled yes, only for this cause) check the sanity rule of §5.3 (count
     `group-activity-started` facts in that world's 300-day save with `target/release/mineworld inspect
     <save> --last 0` or the record's own counts; read `scripts/ci_parity.py` to find where the 300-day save
     is kept, record how you counted). Zero → **stop**.
  8. Only when step 7 allows: `python3 scripts/ci_parity.py baseline write target/scratch-s27b/s27b.long
     scripts/baselines.txt`, then open `scripts/baselines.txt` and add the reason line of §5.3 (with `<PR>`
     left as `#<PR>` until the PR exists; B3 fills it) to each moved world's section, keeping every existing
     reason line. `git diff scripts/baselines.txt` must touch **only** the moved `group-activity` world sections
     (and only their keys and the added reason line): if `baseline write` also rewrote a section for F-12d-1's
     staleness, restore that section by hand from `origin/main` and record it. If a moved `group-activity` section is
     itself also stale (it differs between the committed file and the base record), the two causes cannot be
     separated in one rewrite: **stop** and report to the primary session (re-baseline after the 12d re-record
     lands, or on its instruction). Then, if the reference was the
     committed file, `python3 scripts/ci_parity.py baseline check target/scratch-s27b/s27b.long` → PASS; if it
     was the base record, the check may still report the stale sections — record them as F-12d-1, not as a
     failure of this PR.
  9. Optional, recorded either way: if `godot` is on PATH, `cargo test -p mineworld-cli --test client_2d_interact
     -- --ignored --test-threads=1` → PASS / FAIL / NOT RUN (no Godot).
- [ ] Implementation: steps 7–8 (or `N/A`: no column moved).
- [ ] Validation: steps 1–3, 5–6, 9, each recorded.
- [ ] Review: every moved key has the F-S27b-1 cause and no other; no existing reason line was dropped.
- **Commit message (only if a file other than the ledger changed):** `test(baselines): re-record group-activity worlds after complete offers (S27-b, QS27b-1)`;
  otherwise `docs(plan): S27-b — behaviour impact measured, baselines unchanged`.

#### B3 — Close-out

- **Files:** this file (§15), `handoff-s27b.md`, `scripts/baselines.txt` (only to replace `#<PR>`).
- Steps:
  1. `git diff --stat origin/main...HEAD`: check I-b-4 (only §2.1's paths); record.
  2. `git push -u origin mvp0/pr-s27b`; `gh pr create --title "feat(group-activity): complete offers for the four empty-payload actions (S27-b)" --body "<what changed, the evidence, any re-baselined world with its reason line>"`
     with the PR-body attribution line your session specifies.
  3. If B2 wrote reason lines: replace `#<PR>` with the PR number, commit with the message below, push.
  4. Fill §8's evidence for each criterion; mark parent synchronisation pending for the primary session (§11).
  5. Wait for CI on the exact head; repair routine failures (working rules §21).
- [ ] Implementation: steps 1–4.
- [ ] Validation: CI green on the exact head on Linux, macOS and Windows.
- [ ] Review: deviations recorded; no `[x]` without evidence.
- **Commit message:** `docs(plan): S27-b close-out` (and, for step 3, `test(baselines): name the S27-b PR in its reason lines`).
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-S27d-b-1 (on S27-d).** The adapter's `attempt` reaches the four actions; S27-d may add the invitation
  step to its play-test once this PR has merged.
- **R-2D-b-1 (on the 2D client's owner, follow-up).** F-S27b-3: the four by-name composers in
  `clients/2d/scripts/intents.gd` and their literals in `tests/acceptance/tests/client_rules.rs` may be removed
  once every supported server offers them complete.
- **R-GA-b-1 (follow-up, QS27b-2).** `#[serde(deny_unknown_fields)]` on the four empty action structs, so a
  stray field is `malformed-payload` rather than ignored.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-27-agent-players.md` §11.2 CP-b | replace "digests unchanged" with AB-5 as ruled under QS27b-1 (F-S27b-1) |
| `step-27-agent-players.md` §11.2 Adversarial | replace the `{ "x": 1 }` mutation with M-b-1 … M-b-3 (F-S27b-2) |
| `overall.md` | S27-b frozen / merged as … |

---

## 12. Execution contract (filled at freeze)

```text
PROJECT / PR:            MineWorld mvp0, S27-b — group-activity offers its empty-payload actions complete
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s27b-group-activity-offers.md
RELATED / BINDING DOCS:  step-27-agent-players.md (§§2.2, 5.5, 11.2, 11.4); docs/DECISIONS.md ARC-34, ARC-35;
                         .structured-coding/standards.md (baseline reason lines); CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s27b, its own, held by one session
BRANCH:                  mvp0/pr-s27b, created from origin/main
START CONDITION:         this design frozen, then the primary session's dispatch (step-27 §11.4)
IMPLEMENTATION BASE:     origin/main at B0; B0 records the commit
APPROVED SCOPE:          §2.1 items 1–6
FROZEN INVARIANTS:       §6 I-b-1 … I-b-5; §5.3's baseline rules as ruled under QS27b-1
SEQUENCE:                B0, B1, B2, B3 (§9)
ALLOWED COMMANDS:        cargo *; uv run --locked *; uv sync --locked; git; gh (never merge);
                         python3 scripts/*; target/*/mineworld *; mkdir -p; sed -n; godot (B2 step 9 only)
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; edits outside APPROVED SCOPE;
                         editing a baseline section §5.3 does not allow; editing a test's expectation of town
                         behaviour
MATERIAL STOPS:          §5.3's stops; B2 step 4's stop; any change needed outside §2.1; any change to an
                         action, requirement or System other than the four offers
PLATFORMS:               Linux, macOS, Windows (the PR's CI: Rust layers and the python job on all three)
VALIDATION BUDGET:       unit and integration tests unrestricted locally; one long parity record (≤ 20 min);
                         CI: the PR's runs. Gate 1: NOT REQUIRED (no LLM-facing change)
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s27b.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized    (primary, at freeze — unresolved
                                                                           until then)
                         semantic commits: authorized                     (working rules §14)
                         branch push: authorized                          (working rules §21)
                         PR creation / update: authorized                 (working rules §21)
                         CI repair to review readiness: authorized        (working rules §21)
                         merge: never by the implementation session; only with the operator's explicit
                         authorization (working rules §22)
POST-MERGE SYNC OWNER:   primary session: step-27, overall.md; implementation session: this ledger
STOP CONDITION:          S27-b READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-b-1 | Residents leave activities more often, so towns feel less social (F-S27b-1) | QS27b-1; §5.3's sanity rule (activities still start); the paced band's constants are frozen and not retuned (`offered.rs:17-20`) |
| R-b-2 | A CLI test pins a social count that now moves | B2 step 4 stops rather than editing an expectation |
| R-b-3 | The Python CP-b test is flaky because bob's `invite` is not yet offered available | `until(…)` waits on the stream (20 s); alice and bob start 2.42 m apart, inside 3 m (B-16) |
| R-b-4 | The long parity record exceeds 20 min on a slow machine | run once; record the time; CI's nightly re-checks after merge |

---

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **QS27b-1 [operator — resident behaviour, baselines]** | Making the four offers complete lets the paced rule controller's `offered` band (20 % of consults that reach it) attempt them, so residents of `lakeside`, `market-town` and `social-cafe` will sometimes leave, join or decline through that band, and those worlds' long-run baselines will likely move (F-S27b-1). The step expected "unchanged". May S27-b re-record exactly those sections with the §5.3 reason line? | **Yes, narrowly**, as QP6-2 was ruled for P6a: only `group-activity` worlds, only for this cause, with the reason line, and only while each moved world still starts activities (§5.3). The alternative — keeping the four incomplete and teaching the adapter their payloads by name — would make the agent player depend on a pack's vocabulary, which `ARC-34` and step-27 §4.1 exist to avoid; the other alternative — changing the rule controller to ignore them — edits a frozen controller (`offered.rs:17-20`) for a world-content reason **Ruled: allowed, only for this cause (§14.1).** |
| **QS27b-2 [primary — adversarial criterion]** | The step's adversarial "plant `{ "x": 1 }`; the decode test fails" cannot fail (F-S27b-2). Accept M-b-1 … M-b-3 in its place, and the `deny_unknown_fields` hardening as follow-up R-GA-b-1 rather than in this PR? | **Yes.** Hardening the four structs changes what the pack accepts at dispatch, a separate behaviour decision; the mutations here prove the new test is load-bearing **Ruled yes (§14.1).** |

### 14.1 Rulings, 2026-10-14 (relayed by the coordinator; binding)

| Question | Ruling | By |
| --- | --- | --- |
| QS27b-1 | **"允许，只限这个原因"** — allowed, only for this cause: re-baseline the `group-activity` worlds only for F-S27b-1's cause, with §5.3's reason line and its sanity rule (each moved world still starts activities) | operator, answered directly in the primary session |
| QS27b-2 | **Yes.** M-b-1 … M-b-3 replace the step's `{ "x": 1 }` mutation; `deny_unknown_fields` on the four structs is a follow-up finding (R-GA-b-1) | primary session |

Coordinator note, same day (binding on B2): `main`'s long-run baselines are stale (F-12d-1); a re-record of
`main` lands after 12d. B2's measure-first comparison is made against that re-recorded baseline when it is on
the base, and otherwise against a long record of B2's own base commit (§9 B2 steps 5–8).

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| B0 | — (this commit) | [x] | [x] | [x] |
| B1 | — (this commit) | [x] | [x] | [x] |
| B2 | — | [ ] | [ ] | [ ] |
| B3 | — | [ ] | [ ] | [ ] |

### Evidence

- B0 (2026-10-14): base `origin/main` = `708bc56` (merge of PR #196, which carries this frozen design).
  Header reads `DESIGN FROZEN`; §12 is filled (checked). Rulings QS27b-1 and QS27b-2 copied verbatim into
  `handoff-s27b.md`. Markdown checks run in B0 (see handoff).
- B1 (2026-10-14), load average ~200 throughout (R-LOAD-1: no local workspace run):
  - `cargo test -p mineworld-group-activity`: 3 + 11 + 1 tests, all PASS (≈7 min incl. compile).
  - Mutations (each red on the named test, reverted, then green):
    M-b-1 accept-invitation via `Offer::new`: FAIL at payload assertion (`:422`), reverted.
    M-b-2 invite complete: FAIL at "invite stays incomplete", reverted.
    M-b-3 leave with `accept_requirement()`: FAIL at requirement assertion (`:470`), reverted.
  - `cargo fmt --all --check`: PASS. `cargo clippy -p mineworld-group-activity --all-targets -- -D warnings`: PASS.
  - `cargo build -p mineworld-cli` (≈12 min under load): PASS.
  - `MINEWORLD_BIN=/tmp/impl-s27b-target/debug/mineworld uv run --locked pytest sdk/python/tests/test_real_server.py -k invitation_is_answered`: 1 passed.
  - `ruff check`, `ruff format --check`, `pyright sdk/python`: PASS (0 errors).
  - `check_doc_headings.py`, `check_decision_ids.py`: PASS.
  - Test names: `the_four_answers_are_offered_complete_and_attempted_unchanged` (Rust),
    `test_an_invitation_is_answered_and_left_through_complete_affordances` (Python).

### Assumptions to verify in implementation

| Id | Assumption | Verified in | If false |
| --- | --- | --- | --- |
| A-b-1 | The `Cafe` fixture's names (`alice`, `bob`, `carol`, `far`) and accessors match B-8 | B1 step 8 | use the real names; record |
| A-b-2 | `Invitations::from(person)` is the accessor for "an invitation from them" | B1 step 8 (read `component.rs`) | use the real accessor; record |
| A-b-3 | In market-town with no controllers, bob is offered `invite` to alice as available at the start | B1 step 9 | walk bob toward alice with `offers.request` of `move` first (the file's `TALK`/`MOVE` pattern, `:30-37`); record |

### Mutations

(none run yet)

### Deviations and findings

(planning findings are §3.2)

- **D-b-0 (B0, bounded, recorded).** The header block reads `DESIGN FROZEN 2026-10-14` but its metadata
  line `Lifecycle: DRAFT` was not updated at freeze. The operator's dispatch states the design is frozen and
  §12 is filled, so B0 proceeds; the stale `Lifecycle` line is for the primary session to correct in its
  next design-side commit. No authority line was changed by the implementer.

- **D-b-1 (bounded correction, recorded, 2026-10-15; operator dispatch on resume).** Pre-existing defect found
  when the lane resumed: `perception.rs` offered `accept-invitation` and `decline-invitation` as available for
  an invitation past its `until`, although dispatch refuses it (`PreconditionFailed`). The module doc claimed
  perception could not see the clock; `WorldRead::at` exists and is the world's clock. Fix: the invitation
  must be open at `world.at()` to be offered (`invited_by_them`). Effect on I-b-2: the availability of the
  two answers changes for expired invitations only; requirements, refusals and the other offers are unchanged.
  Regression test `an_expired_invitation_is_not_offered_as_an_answer` (`systems/group-activity/tests/group_activity.rs`):
  red on the pre-fix code (FAIL at "an expired invitation is not offered accept"), green after. The test moves
  the world's clock with `World::advance_to` before observing, because perception reads `WorldRead::at`
  (= `World::now`) and not the `at` argument of `observe`. Finding for the kernel/presence owner: `observe`
  takes an `at` that its providers do not see (the observation's `at` and the read's `at` can differ in tests).
  This correction may move the B2 long-run measurement for a cause other than F-S27b-1 (an expired accept
  is no longer attempted); B2 must attribute every moved key before any re-record (§5.3).
