# Handoff — step-31: PR S31-c implementation (the `price-survey` System Pack)

A continuation aid, never a design authority. The authority is
[`pr-s31c-price-survey.md`](pr-s31c-price-survey.md) (DESIGN FROZEN 2026-10-20), with its live ledger in §15.

```text
PROJECT / PR        MVP-0 · step-31 / PR S31-c — the price-survey System Pack: the price book, witnessing
                    sales through presence's audience rule, the `survey` action with its cooldown
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s31c-price-survey.md; ledger §15
BRANCH / WORKTREE   mvp0/pr-s31c in /Users/yuema137/mineworld-worktrees/impl-s31c (sole writer: the S31-c
                    implementer session)
BASE                origin/main @ fec906f2 (2026-10-20)
APPROVED SCOPE      design §2.1 as frozen, §14.1 rulings
ENDPOINT AUTHORITY  implement, validate, commit, push, open the PR, repair `fast`. Never merge (the
                    coordinator merges)
SEQUENCE            C0 … C6 (§9)
VALIDATION          SPEED FIRST: build, fmt, clippy on the touched crates, only the test files this PR
                    writes; every cargo command through cast-slots/with-slot.sh; never --workspace.
                    Long runs, acceptance checks and planted mutations are milestone work
NEVER               python3 -c; sed -i; curl; git add -A; cargo test --workspace (R-LOAD-1); reading keys
CURRENT CHECKPOINT  see §15's commit ledger
```
