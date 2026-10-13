# Handoff — S10: PR P3b implementation (the SDK's perceived stream, cursor and resume)

A continuation aid, never a design authority. The authority is
[`pr-s10-p3b-perceived.md`](pr-s10-p3b-perceived.md) (DESIGN FROZEN 2026-10-10, primary session; contract
§11, rulings §12.1), with its live ledger in §13.

```text
PROJECT / PR        MineWorld mvp0 · S10 (step-17) / PR P3b — the SDK's perceived stream, cursor and
                    resume (ResumingSeat, PerceivedStream, CursorSource; DEP-44)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s10-p3b-perceived.md; ledger §13
RELATED / BINDING   step-17-cognition.md (§3.4.3, §3.6, §10 IC-1, §15); pr-s10-p3-python-sdk.md;
                    pr-s10-p4-memory.md (R-P3b-1); step-12-server.md §§16–17; server/PROTOCOL.md rev 2
                    §§4, 5; docs/REUSE_POLICY.md; docs/ENGINEERING_STANDARDS.md; CLAUDE.md
BRANCH / WORKTREE   mvp0/pr-s10-p3b in /Users/yuema137/mineworld-worktrees/impl-p3b (sole writer;
                    source: the primary session's freeze, design §11)
BASE                origin/main @ f4ed913 (merge of #149, the frozen design); includes #142 (7eed282)
HEAD / FINGERPRINT  see `git log -1` and `git status --short`; the final PR head and its CI run are in
                    the PR body (a commit cannot carry its own run)
APPROVED SCOPE      design §2.1, as frozen; diff limited to sdk/python/**, docs/DECISIONS.md,
                    .structured-coding/** (AP3b-15)
FROZEN INVARIANTS   §2.3 (I-1, I-2, I-10, I-11, INV-9, INV-13, P3b-1 … P3b-6); D-P3b-1 … D-P3b-10;
                    DEP-44; §4.5's table; AP3b-1 … AP3b-17; §12.1's rulings
ENDPOINT AUTHORITY  implementation + local validation: authorized (primary session freeze)
                    semantic commits: authorized (primary session freeze)
                    branch push, PR create/update, CI repair: authorized (working rules §21 default,
                    not narrowed by the freeze instruction)
                    merge: explicit operator authorization only
SEQUENCE            C0 … C6 (design §8) — all committed
VALIDATION BUDGET   unit, static and local real-binary tests unrestricted; Gate 1 NOT REQUIRED; no
                    model, no key, no paid API; CI: the PR's runs only, no manual dispatch
NEVER               python3 -c; sed -i; awk; xargs; curl; heredoc writes; reading
                    ~/.config/mineworld/secrets.env
MATERIAL STOPS      any Rust, server, protocol or golden-frame change; any new dependency (uv.lock
                    unchanged); a change to §4.5's table or P3b-1 … P3b-6; any hosted-API use
STOP CONDITIONS     READY FOR OPERATOR REVIEW — DO NOT MERGE; material stops above
CURRENT CHECKPOINT  C0 … C6 committed; local evidence complete (§13.2, §13.4, §13.6); the PR's CI
                    on the final head decides READY FOR OPERATOR REVIEW. No material stop met;
                    deviations DV-P3b-1 … 8 are bounded.
NEXT ACTIONS        push; open the PR; watch CI on the exact head; repair routine failures; then
                    READY FOR OPERATOR REVIEW — DO NOT MERGE. After merge: record the merge identity
                    in §13; the S10 planning session updates step-17 §15 and overall.md (DEP-44,
                    F-P3b-4/5 for P6).
BACKGROUND JOBS     none
```
