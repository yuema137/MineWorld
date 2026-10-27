# Handoff — PR S31-b2 (`notices` adopts `AttributeOwner`; the `topics` record)

Continuation aid only. The authority is [`pr-s31b2-notices-attributes.md`](pr-s31b2-notices-attributes.md)
(DESIGN FROZEN 2026-10-26); its §15 is the ledger.

| Field | Value |
| --- | --- |
| Worktree | `/Users/yuema137/mineworld-worktrees/impl-s31b2` (one session) |
| Branch | `mvp0/pr-s31b2` |
| Base | `origin/main @ 0b2faf2f` |
| Execution contract | the design's §12 |
| Ends at | READY FOR OPERATOR REVIEW (MERGE-READY to the coordinator); never merge |

## State

- C0 … C3 committed and locally green. C4 committed with one failing real-binary arm (AC-b2-7 restart).
- **MATERIAL STOP at C4** (see the design's §15): resuming a W1-attr save is refused by the configuration
  drift check (`worldpack/src/configure.rs`), because the handed `subscriptions-seeded` is a genesis
  reaction fact. Fix proposed (filter saved facts to `Causation::WorldGenesis`), outside §2.1: awaiting a ruling.
- Not pushed; no PR yet. C5 waits on the ruling.

## Outstanding follow-ups for other PRs

- R-S31g-b2-1 (`View.topics` in the agent adapter) and R-S31e-b2-1 (topics panel; words for
  `subscriptions-are-attributes`): S31-g and S31-e merged first, so both are open.
- R-S31f2-b2-1, R-S31f-b2-1 as the design's §10 states.
