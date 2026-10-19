# Handoff — PR S31-a (currency as data, seller asks, atomic settlement)

A continuation aid for this PR only. It is never a design authority: the authority order is repository and git
truth, then [`pr-s31a-currency-asks.md`](pr-s31a-currency-asks.md) (its §15 ledger is live), then the binding
parents, then this file.

```text
PROJECT / PR:          MineWorld mvp0, S31-a — currency as data, seller asks, atomic settlement
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-s31a-currency-asks.md (DESIGN FROZEN 2026-10-18)
BINDING DOCS:          step-31-wealth-prices.md; step-10-market.md §4.5; CLAUDE.md;
                       docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md; .structured-coding/standards.md;
                       /Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md
WORKTREE / BRANCH:     /Users/yuema137/mineworld-worktrees/impl-s31a, mvp0/pr-s31a
IMPLEMENTATION BASE:   origin/main @ 44f050e3 (12d #171 merged; WE-a #189 open; S25a #200 merged; S29-b not
                       merged; #258 and #261 merged)
SCRATCH:               /Users/yuema137/mineworld-worktrees/lane-s31-s29/scratch-s31a/
```

## Approved scope and frozen invariants

The design's §2.1 items 1–7 as §14.1's rulings shape them; invariants I-a-1 … I-a-11 (§6); acceptance AC-a-1 …
AC-a-13 (§8) with the planted mutations of §8.1. Nothing changes under `kernel/`, `contracts/`, `persistence/`,
`server/`, `cognition/`, `systems/inventory/` or `systems/item-transfer/` (X-11's plant is reverted, never staged).

## Endpoint authority, with sources

| Endpoint | Decision | Source |
| --- | --- | --- |
| implementation + local validation | authorized | the operator's freeze, 2026-10-18 |
| semantic commits | authorized | working rules §14 |
| branch push, PR creation and update, CI repair | **not now** | the coordinator, relayed by the lane lead 2026-10-18: "implement LOCALLY now and commit per the commit plan; DO NOT push and DO NOT open a PR. The lane lead pushes later" |
| C7's nightly `parity-long` dispatch | **not now** (it needs a pushed branch) | the same instruction |
| merge | never by this session | working rules §22 |

## Validation budget (R-LOAD-1)

Locally: only each commit's targeted test files, one at a time, plus `cargo fmt --all --check` and `cargo clippy -p
<crate> --all-targets -- -D warnings`, all through `cast-slots/with-slot.sh`. Never `cargo test --workspace`,
never all of `tools/cli`. Long Market Town runs and nightly-class runs are CI's: INCONCLUSIVE locally when not run.
The local exceptions the design names: AC-a-6, AC-a-7 and `vocabulary_only` (single files, 30-day runs). No model
call, no paid call, no key read.

## Implementation sequence and current checkpoint

C0 … C8 of §9. C7 (the re-baseline from a CI `parity-long` record) needs a pushed branch, so this session stops at
the commit before it and reports what remains.

- **Current checkpoint:** C0 committed (ledger opened, base save written).
- **Next actions:** C1 — the decision records `ARC-111` … `ARC-114`, `DEP-63`, the `ARC-38` and `ARC-35` notes,
  `MODULE_SPEC.md` §4.1, `CORE_CONCEPTS.md` §§7–8, `ARCHITECTURE.md` §4; then C2 (currency) and C2b (the guide's
  world declares its currency, F-S31a-19).

## Stop conditions

A material stop of §12 (a needed change under I-a-1's paths, a new dependency, AC-a-9 showing controller choices
moved, AC-a-5 finding an owner-rule disagreement, AC-1 failing outside Q-S31a-1's ruling, any change to a frozen
invariant or acceptance), or the end of local work before C7: report to the lane lead and stop.

## Background jobs

None running. The base save of C0 is deferred to the milestone (lane lead's speed-first update, 2026-10-18);
its base binary and world copy are in `scratch-s31a/base/`.

## Validation policy (lane lead, 2026-10-18)

Only the build, fmt, clippy and the test files this PR writes or changes run locally; acceptance runs, long runs
and planted mutations are "not run: milestone". No push, no PR (it waits for Milestone F's checklist or 16c's
merge). No world-specific hard-coding: a path that only works for Market Town is a stop.
