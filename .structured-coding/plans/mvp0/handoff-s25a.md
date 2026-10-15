# Handoff — S25 PR 25a (the generated reference, Rust side)

A continuation aid, never a design authority. The authority is
[`pr-s25a-reference.md`](pr-s25a-reference.md) (contract §8, ledger §9) under
[`step-25-creator-docs.md`](step-25-creator-docs.md) §10.3.

```text
PROJECT / PR        MVP-0 · S25 / PR 25a — the generated reference, Rust side
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s25a-reference.md
BRANCH / WORKTREE   mvp0/pr-s25a in /Users/yuema137/mineworld-worktrees/impl-s25a (sole writer)
EXTERNAL            /Users/yuema137/mineworld-worktrees/ext-fishing (yuema137/mineworld-pack-fishing)
BASE                origin/main @ f2f0a71
ENV                 PATH="$HOME/.cargo/bin:$PATH"; CARGO_TARGET_DIR=/tmp/impl-s25a-target
ENDPOINT AUTHORITY  implementation, commits, push, PR, CI repair, fishing tag: authorized (primary's
                    dispatch 2026-10-14); merge: operator only
STOP CONDITIONS     READY FOR OPERATOR REVIEW with CI fast/test/test-macos/test-windows/platforms/python
                    green on the exact head; material stops per contract §8
TOOL DISCIPLINE     no sed -i, no inline python, no curl, no heredoc writes
```

## Current checkpoint

C0, C1, C2+C3 committed by the first implementer (stopped, cancelled 2026-10-14; this session replaced
it in the same worktree, sole writer). C4 and C5 committed by this session; origin/main @ 31ba514
merged; mutations M1–M4 killed; parity compared with the base. Under R-LOAD-1 the workspace gate is CI.
Next: push, open the PR, CI green on the exact head (fast, test, test-macos, test-windows, platforms,
python), record the run ids in the ledger, READY FOR OPERATOR REVIEW. Never merge. A detached worktree
`/Users/yuema137/mineworld-worktrees/s25a-parity-base` (origin/main @ 31ba514) was used for the base
parity record and can be removed.
