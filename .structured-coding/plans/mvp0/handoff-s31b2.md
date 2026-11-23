# Handoff — PR S31-b2 (`notices` adopts `AttributeOwner`; the `topics` record)

Continuation aid only. The authority is [`pr-s31b2-notices-attributes.md`](pr-s31b2-notices-attributes.md)
(DESIGN FROZEN 2026-10-26); its §15 is the ledger.

| Field | Value |
| --- | --- |
| Worktree | `/tmp/mineworld-recover-s31b2` (delegated recovery owner, 2026-11-22) |
| Branch | `recover/s31b2-notices-attributes` |
| Base | `2b4d36fa` |
| Execution contract | the design's §12 |
| Ends at | Local reviewable commits and PR body, then independent red team coordinated by primary; no publication or merge |

## State

- Executable HEAD `19e30172bc57775d4b1369df7ccdee8226745955`; final review HEAD is the
  subsequent documentation commit reported directly to the coordinator. Five old commits
  were recovered on current base without conflicts. Old WIP
  is read-only; its dirty patch hash and recovery commit mapping are in the primary design §15.
- C4's historical material stop was approved on October 27 at 14:16 (operator ruling in
  `/Users/yuema137/mineworld-demos/progress/pending-decisions.md`). Recovered comparison fix
  filters only `Causation::WorldGenesis`; regression repaired to select the configuration event
  by type instead of accidentally selecting `arrived`. Two comparator tests PASS locally.
- Vocabulary regression for YAML `null`/`true` topic ids now passes using the existing JSON
  serializer. Frozen W1/W2 parity fixtures were independently generated from base `2b4d36fa`;
  recovered implementation matches complete event envelopes. Initial failures and provenance
  are recorded in §15.
- Validation complete: notices 30 tests; worldpack data 9; comparator 2 plus discriminative
  mutation; CLI notices 7 (one filtered). Strengthened create→act→persist→restart disclosure
  regression separately passes with complete pre/post record equality and private disclosure
  assertions. Scoped four-crate all-target clippy with warnings denied passes; formatting,
  documentation checks and reference check (46 files) pass. All heavy commands used
  `cast-slots/with-slot.sh` with the ordinary two slots. No jobs remain running.
- `no_random_subscription` (30-day scenario) is **NOT RUN**, not a passing test.
  AC-b2-8 reference-world milestone/remote CI is **NOT RUN**. No paid API or full workspace suite.
- Implementer technical-debt, modularity, readability and robustness review is complete in
  §15, distinguishing retained dispatcher/loader size from new changes. Independent red-team
  review is pending with the primary. Freeze at the reported final SHA; no further edits until
  review findings or coordinator direction. PR body: `/tmp/mineworld-s31b2-snapshot/pr-body.md`.
- Publication is withheld by the coordinator's current dispatch. No remote CI claimed; no
  subagents or changes to shared plans/dashboard. Preserve the frozen fact schemas, state ownership,
  private disclosure, and excluded paths. Any material expansion is escalated to primary.

## Outstanding follow-ups for other PRs

- R-S31g-b2-1 (`View.topics` in the agent adapter) and R-S31e-b2-1 (topics panel; words for
  `subscriptions-are-attributes`): S31-g and S31-e merged first, so both are open.
- R-S31f2-b2-1, R-S31f-b2-1 as the design's §10 states.
