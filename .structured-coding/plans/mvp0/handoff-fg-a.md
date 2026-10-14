# Handoff — PR FG-a (implementation session)

A continuation aid, never a design authority. The authority is
[`pr-fg-a-relationship-forgetting.md`](pr-fg-a-relationship-forgetting.md) (frozen §§1–14, live §§10, 15, 16).

```text
PROJECT / PR        MVP-0 · S24 / PR FG-a — System-owned forgetting in relationships and conversation
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-fg-a-relationship-forgetting.md
RELATED / BINDING   mvp1/step-24-decision-and-memory.md §§1.2, 2.4, 3.2, 8.2, 10, 11.2, 14.1; overall.md;
                    pr-il-e-social.md; DECISIONS ARC-5, ARC-23, ARC-25, ARC-28, ARC-61 … ARC-65, ARC-86,
                    DEP-10; MODULE_SPEC §4.2; CLAUDE.md §§2–4
BRANCH              mvp0/pr-fg-a, worktree /Users/yuema137/mineworld-worktrees/impl-fg-a (one session)
BASE                origin/main @ 6306977 (after DOC-24 #175, DL design #177)
APPROVED SCOPE      §1, §4, §5 as answered by §13.1; FGA-C1 … FGA-C9; kernel K1 only; SDK S1–S4 only
FROZEN INVARIANTS   §14 of the design (no decay fact/Process/row; only lowers toward floor; cap bound;
                    edges = entries; holder-only same-shape disclosure; no model; unconfigured summaries
                    and facts byte-identical; integer only; §11 no-diff list)
ENDPOINT AUTHORITY  implementation + local validation: authorized (operator go 2026-10-13, relayed);
                    commits, push, PR create/update, CI repair: authorized (design §14, working rules
                    §§14, 21); merge: operator only, merge commit
VALIDATION BUDGET   design §14 (≈ 1 hour of runs); operator: full local `cargo test --workspace
                    --no-fail-fast` before any push that changes tests
STOP CONDITIONS     NORMAL: READY FOR OPERATOR REVIEW — DO NOT MERGE. MATERIAL: design §14.
                    Milestone pause announced by the operator: commit locally and stop.
ENVIRONMENT         export PATH="$HOME/.cargo/bin:$PATH"; export CARGO_TARGET_DIR=/tmp/impl-fg-a-target;
                    python scripts via `uv run --locked python scripts/...` (system python3 is 3.9)
SCRATCH             /Users/yuema137/mineworld-worktrees/scratch-fg-a-impl (base binary, records, saves)
```

## Current checkpoint

See the design's §10 checkboxes and §15 ledger; this file records only what they do not.

## Exact next actions

1. E-FGA-0: base long record and four 30-day saves with the base release binary (scratch dir above).
2. FGA-C1 docs, then C2 (kernel), C3 (SDK), C4 … C9.
