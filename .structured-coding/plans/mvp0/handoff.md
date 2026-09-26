# Handoff — PR 02 implementation context

**Active PR:** Step 02 / PR 02 — Action, Event, Observation and Spatial contracts
**Effort:** `mvp0`
**Primary design doc (semantic authority):**
`.structured-coding/plans/mvp0/step-02-action-event-spatial-contracts.md`
**Binding parents:** `.structured-coding/plans/mvp0/overall.md`, `docs/CORE_CONCEPTS.md`,
`docs/ENGINEERING_RULES.md` §§4-12 and §15, `docs/ENGINEERING_STANDARDS.md`, `CLAUDE.md`
**Execution contract:** §9 of the primary design doc

PR 01's handoff content is superseded: that PR is merged into `main` at `e85c889` and its record
lives in `step-01-entity-component-contracts.md`.

## Repository identity

```text
repository       /Users/yuema137/MineWorld
branch           mvp0/pr-02-action-event-spatial
implementation   main @ c8f2934 (S1 merged; mineworld-contracts exists and is green)
base
current HEAD     the most recent commit on the branch; `git log --oneline` is authoritative
working tree     clean at each commit; the only untracked path is target/ (gitignored)
remote           none (decision D-9: local-only until S13)
```

## Environment

```text
toolchain   rustc/cargo 1.97.1 pinned by rust-toolchain.toml, rustfmt 1.9.0, clippy 0.1.97
PATH        ~/.cargo/bin is NOT on the default non-interactive PATH.
            Every command must start with: export PATH="$HOME/.cargo/bin:$PATH"
            A bare "command not found" is a PATH defect in the session, never a failed check.
verify      cargo fmt --all --check
            cargo check --workspace --all-targets
            cargo clippy --workspace --all-targets --all-features -- -D warnings
            cargo test -p mineworld-contracts
```

## Approved scope

§1.1 of the design: `contracts/src/action.rs`, `event.rs`, `spatial.rs`, `observation.rs`, the
spatial and affordance vocabulary in `docs/CORE_CONCEPTS.md`, and the `contracts/README.md`
inventory. Non-goals in §1.2 — dispatch, the System trait, the scheduler, persistence, concrete
actions, perception systems and network encoding all belong to later steps.

## Frozen invariants

```text
no floating point anywhere in these contracts; positions are i32 millimetres and angles
  i32 millidegrees, and a structural test asserts the crate contains no f32/f64
no domain concept: talk, give_item, Cafe appear only in test fixtures, as opaque identifiers
no engine concept: no mesh, navmesh, camera, scene tree, animation, skeleton, physics
BTreeMap / BTreeSet / Vec only (clippy.toml denies HashMap/HashSet workspace-wide)
an Event cannot be constructed without a Causation and a Visibility
an Observation exposes only the entities it lists — no store handle, no global accessor
ActionRecord and EventRecord are the only erasure boundaries, mirroring
  ComponentRecord<P = Vec<u8>>: generic over the payload, the codec belongs to persistence
INV-2, INV-10, INV-12, INV-13, INV-15 as stated in §1.3
```

## Endpoint authority

```text
implementation + local validation   authorized      (operator kickoff, 2026-09-25)
semantic commits                    authorized      (operator kickoff, explicit)
branch push                         NOT authorized  (operator kickoff; decision D-9)
PR creation / remote CI             N/A — no remote exists (decision D-9)
merge into main                     explicit operator authorization only
```

## Implementation sequence

```text
C1  action contracts          — ActionTypeId, Action, ActionRecord, ActionIntent,
                                ActionResult, Rejection, RejectionCode
C2  event contracts           — EventTypeId, Event, EventRecord, EventEnvelope,
                                Causation, Visibility, Provenance
C3  spatial contract          — Millimetres, Millidegrees, LocalPosition, Orientation,
                                Location, SpatialRequirement + evaluate;
                                also adds ActionIntent.actor_location (deviation D-1)
C4  observation contract      — PerceivedEntity, PerceivedEvent, Affordance, Observation
C5  specification sync        — docs/CORE_CONCEPTS.md, contracts/README.md, this ledger
```

## Current checkpoint

```text
C1  DONE   contracts/src/action.rs, two IdentifierKind variants and two ContractError
           variants in error.rs, check_identifier widened to pub(crate) in ids.rs,
           module wiring and re-exports in lib.rs, contracts/tests/action.rs (5 tests),
           identity.rs extended for the two new declaration names.
           All four §6 commands clean; 36 integration tests + 2 doc-tests; three
           mutations confirmed the new guards are load-bearing (§7.2).
C2  DONE   contracts/src/event.rs, IdentifierKind::EventTypeId and two ContractError
           variants, module wiring, contracts/tests/event.rs (6 tests) and a new
           compile-fail case pinning that a recorded event cannot be reassigned.
           All four §6 commands clean; 42 integration tests + 3 doc-tests; two
           mutations confirmed the guards are load-bearing (§7.2).
C3  DONE   contracts/src/spatial.rs (Millimetres, Millidegrees, LocalPosition,
           Orientation, Location, PlaceRequirement, SpatialRequirement + evaluate),
           two ContractError variants, ActionIntent.actor_location and from_location
           (deviation D-1 closed), contracts/tests/spatial.rs (8 tests including the
           no-float structural guard), action.rs tests extended to pin the reported
           location. All four §6 commands clean; 50 integration tests + 3 doc-tests;
           five mutations, one of which survived and led to a stronger degeneracy
           test (§7.2 M9).
C4  next
C5  not started

background jobs: none
open items: O-1 (event payloads have no schema version — S5 owns it),
            O-2 (no ActionDeclaration value — S3 owns it), both in §7.3
```

## Exact next actions

1. `contracts/src/observation.rs` per §5 C4: `PerceivedEntity`, `PerceivedEvent`, `Affordance`,
   `Observation` — no store handle, no global accessor, and no way to reach an entity the
   observation does not list.
2. `contracts/tests/observation.rs`, and a compile-fail case if one carries information.
3. Record C4 in §7, then commit.
4. C5: `docs/CORE_CONCEPTS.md` §§6, 12 and 15, `contracts/README.md`, §7.4 gate re-check.

## Stop conditions

```text
STOP and report when:
  a frozen invariant in §1.3 would have to change
  a gate question in §3.1 cannot be answered yes against the written code
  a public specification statement is falsified
  the PR reaches READY FOR OPERATOR REVIEW

NORMAL STOP: PR 02 ready for operator review on
mvp0/pr-02-action-event-spatial — DO NOT MERGE.
```
