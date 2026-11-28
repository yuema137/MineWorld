# Handoff — S30-k (attribute localisation)

Continuation aid only. The authority is [`pr-s30k-localisation.md`](pr-s30k-localisation.md) (DESIGN FROZEN
2026-10-27); its §16 ledger holds progress and evidence.

## Current state

- PR: S30-k1 (mechanism), branch `mvp0/pr-s30k1-labels`, worktree `/Users/yuema137/mineworld-worktrees/impl-s30k1-2`
  (the old `mvp0/pr-s30k1` / `impl-s30k1` is superseded; its WIP was rebased here).
- Base: `origin/main @ bdb83457`. C1–C8 committed; see §16 for what is open.
- S30-k2 (zh_Hans data and checkpoint) starts after S30-k1 merges, from §9.2.

## Next action

Close §16's open items (surface oracles, AC-k-9 sizes, mutations, C9 spec sync, review), then C10.

## Facts recorded at C0

- PDN: only its design merged (#499); no `Locale` type exists in code, so C1 adds `Locale` to
  `mineworld-authoring` (R-PDN-k-1: PDN reuses it).
- S31-b2: only its design merged; `notices` does not call `disclose_owned`. The one caller is
  `systems/employment/src/system.rs` (`discloses`).
- S30-j (`mvp0/pr-s30j`) is implemented in parallel and touches `authoring/attributes/src/context.rs`,
  `systems/attributes/src/{data.rs, system.rs}`; S30-k1 keeps its edits there minimal.
