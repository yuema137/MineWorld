# Handoff — S30-k (attribute localisation)

Continuation aid only. The authority is [`pr-s30k-localisation.md`](pr-s30k-localisation.md) (DESIGN FROZEN
2026-10-27); its §16 ledger holds progress and evidence.

## Current state

- PR: S30-k1 (mechanism), branch `mvp0/pr-s30k1`, worktree `/Users/yuema137/mineworld-worktrees/impl-s30k1`.
- Base: `origin/main @ 315a50a9` (#513).
- S30-k2 (zh_Hans data and checkpoint) starts after S30-k1 merges, from §9.2.

## Next action

Follow §9.1 from the first commit whose row in §16 is not `[x]`.

## Facts recorded at C0

- PDN: only its design merged (#499); no `Locale` type exists in code, so C1 adds `Locale` to
  `mineworld-authoring` (R-PDN-k-1: PDN reuses it).
- S31-b2: only its design merged; `notices` does not call `disclose_owned`. The one caller is
  `systems/employment/src/system.rs` (`discloses`).
- S30-j (`mvp0/pr-s30j`) is implemented in parallel and touches `authoring/attributes/src/context.rs`,
  `systems/attributes/src/{data.rs, system.rs}`; S30-k1 keeps its edits there minimal.
