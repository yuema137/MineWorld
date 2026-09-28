# Handoff — PR 05d implementation context: CLOSED / AWAITING OPERATOR ACTION

**Active PR:** Step 05 / PR 05d — The clients, and `AC-15`
**Effort:** `mvp0`
**Primary design doc (semantic authority):**
`.structured-coding/plans/mvp0/pr-05d-clients-ac15.md`
**Binding parents:** `.structured-coding/plans/mvp0/step-05-vertical-slice.md` §§1.3, 2;
`docs/MVP.md` §9 (`AC-13`, `AC-15`, §9.1, §9.2); `docs/ACCEPTANCE.md`; `docs/CORE_CONCEPTS.md`;
`docs/MODULE_SPEC.md` §5; `docs/ENGINEERING_RULES.md`; `docs/ENGINEERING_STANDARDS.md`; `CLAUDE.md`
**Execution contract:** §2 of the primary design doc

PR 02's handoff content, which stood here until now, is superseded: that PR is merged and its record
lives in `step-02-action-event-spatial-contracts.md`. PR 05c flagged this file as stale and asked
whoever started PR 05d to replace it rather than read it, which is what happened.

## Repository identity

```text
worktree         <scratchpad>/mw-05d   (an isolated worktree; several agents run concurrently, and
                 /Users/yuema137/MineWorld and the sibling worktrees were not touched)
branch           mvp0/pr-05d-clients-ac15
implementation   main @ cc40ad5 — 05a, 05b and 05c merged; 237 tests green at that HEAD
base
current HEAD     the tip of the branch; `git log --oneline` is authoritative — a commit cannot
                 carry its own hash
working tree     clean at each commit; `.godot/` and `target/` are gitignored
remote           origin; the branch is pushed. NO PR, NO MERGE (operator kickoff)
```

## Environment

```text
toolchain   rustc/cargo 1.97.1 pinned by rust-toolchain.toml
PATH        ~/.cargo/bin is NOT on the default non-interactive PATH:
                export PATH="$HOME/.cargo/bin:$PATH"
godot       4.7.2.stable.official.ed1daf0bf, on PATH as `godot`
            headless runs scripts; windowed renders. A project must be imported once before a
            headless run: `godot --headless --path <project> --import`
```

## What this PR delivered

```text
server/src/parity.rs            AC-13's comparison, defined once
server/src/protocol.rs          WorldInstanceId on WorldSummary — AC-15's first evidence line
systems/presence                PerceptionProvider (was InteractionProvider) with `discloses`;
                                observe() returns Observation<Value>
systems/conversation            discloses a person's own history, in that person's own observation
cognition/rule-controller/      the RuleController: an Observation in, an ActionRequest out
tools/cli/src/agent.rs          --agent SEAT, over the same host seam a WebSocket session uses
worlds/social-cafe/             a second visitor, and Alice as a seat
clients/protocol/               the Godot client protocol module, a demonstration scene, evidence
tools/cli/tests/                ac15_one_alice.rs (6 tests), ac13_semantic_parity.rs (2)
docs/MVP_STATUS.md              updated where this PR changed the answer
```

## Semantic commits

```text
feat(server): name the running world, and define AC-13's comparison once
feat(systems): a pack discloses its own state to the observer entitled to it
feat(cognition): a RuleController, so a Person can be driven by an agent
feat(cli): --agent drives a seat, and the café has room for two windows
test(ac-15): there is only one Alice, and the evidence names identity
feat(clients): the Godot client protocol module, and AC-13 from what it sent
docs(mvp0): PR 05d's ledger, evidence and limitations
chore: remove three things nothing uses
fix(clients): say goodbye before the socket stops being polled
fix(systems): a disclosure may only be about the subject it was asked about
```

## Validation state

```text
cargo fmt --all --check                                              clean
cargo check --workspace --all-targets                                clean
cargo clippy --workspace --all-targets --all-features -- -D warnings clean
cargo test --workspace --no-fail-fast                                259 passed, 0 failed
Gate 2 (real binary + real Godot 4.7.2, seven runs)                  PASS — design doc §9.2
AC-15                                                                HOLDS — §9.3, with the
                                                                     counterfactual in §9.4
AC-13                                                                HOLDS — §9.5
Gate 1 (real LLM)                                                    NOT REQUIRED — MVP-0 has no
                                                                     model, and nothing here is
                                                                     LLM-facing
```

## Decisions a later session must not silently undo

```text
a component reaches an observation because its owning pack named it AND named who may see it.
  Walking the component stores would satisfy any test here and defeat INV-13 for every pack after
  (design doc §4)

what may be known is the owning pack's judgement; WHOM a record may be about is not. Perception
  drops any record whose entity is not the subject it asked about, so a pack that lies fails closed.
  Removing that one filter turns `a_pack_that_names_a_third_party_discloses_nothing` red
  (design doc §7.10, found in operator review)

the AC-13 comparison lives in server/src/parity.rs and nowhere else. A test that writes its own
  comparison can drop a field (MVP.md §9's correction, design doc §5.2)

Alice is a seat because the agent occupies one, which is what puts a controller on the same path a
  client uses (design doc §7.3). A human may therefore occupy her: AC-5's direction, and an
  unarbitrated case recorded in §10.6

the AC-13 fixtures in clients/protocol/evidence/ come from the real Godot client. Replacing them
  with hand-written frames would make the test prove the comparison and nothing about a client
```

## Exact next actions

```text
1  operator review of this branch. No PR was opened and nothing was merged.
2  the 2D and 3D visual agents adopt clients/protocol/mineworld/ — ADOPTION.md is written for
   them, and coordination goes through the operator rather than across worktrees
3  after merge: the planning session updates step-05 and overall.md (agent-workflow §10 owner
   rule); this implementation session owns only the PR document
```

## Stop conditions

```text
NORMAL   PR 05d READY FOR OPERATOR REVIEW — reached
NEVER    merge without explicit operator authorization
NEVER    merge, edit or pull from vis/2d-generated-assets or vis/3d-human-pipeline
```
