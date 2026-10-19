# Handoff — step-31: PR S31-b implementation (the `notices` System Pack)

A continuation aid, never a design authority. The authority is
[`pr-s31b-notices.md`](pr-s31b-notices.md) (DESIGN FROZEN 2026-10-18), with its live ledger in §15.

```text
PROJECT / PR        MVP-0 · step-31 / PR S31-b — the notices System Pack: topics, subscriptions, notices,
                    the board, the memory renderer
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s31b-notices.md; ledger §15
RELATED / BINDING   step-31-wealth-prices.md §§8, 14, 15, 16; pr-s31a-currency-asks.md §14.1 (Q-S31a-3);
                    ARC-26, ARC-31, ARC-33, ARC-34, ARC-43, ARC-61, ARC-90, ARC-124; CLAUDE.md;
                    docs/ENGINEERING_RULES.md; docs/ENGINEERING_STANDARDS.md
BRANCH / WORKTREE   mvp0/pr-s31b in /Users/yuema137/mineworld-worktrees/impl-s31b (sole writer: the S31+S29
                    lane lead session)
BASE                origin/main @ a1e05874 (2026-10-18)
APPROVED SCOPE      design §2.1 as frozen
ENDPOINT AUTHORITY  implementation + local validation + semantic commits: the coordinator's dispatch of
                    2026-10-18 ("implement locally on branches from main"). Branch push and PR: only once
                    the coordinator says Milestone F's checklist is out, or 12d and 16c have merged.
                    Merge: never by this session without the charter's conditions
SEQUENCE            C0 … C7 (§9)
NEVER               python3 -c; sed -i; curl; git add -A; cargo test --workspace (R-LOAD-1); reading keys
CURRENT CHECKPOINT  C0 committed (this file opened). Next: C1 (ARC-115 in docs/DECISIONS.md, MODULE_SPEC row)
```
