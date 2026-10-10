# Handoff — S10: PR P5b implementation (native Anthropic adapter; subscription bridge)

A continuation aid, never a design authority. The authority is
[`pr-s10-p5b-hosted-subscriptions.md`](pr-s10-p5b-hosted-subscriptions.md) (DESIGN FROZEN 2026-10-09,
with QP5b-1 and QP5b-2), with its live ledger in §11.

```text
PROJECT / PR        MVP-0 · S10 (step-17) / PR P5b — AnthropicMessagesBackend (DEP-33); CliBridgeBackend
                    and the Codex opt-in (ARC-60); no Claude subscription route (QP5b-1)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/pr-s10-p5b-hosted-subscriptions.md; ledger §11
RELATED / BINDING   pr-s10-p5-backends.md (P5a, merged 60a6295, #120); step-17-cognition.md §3.7;
                    docs/ARCHITECTURE.md §9.2; docs/REUSE_POLICY.md; CLAUDE.md
BRANCH / WORKTREE   mvp0/pr-s10-p5b in /Users/yuema137/mineworld-worktrees/impl-s10-p5b (sole writer)
BASE                origin/main @ 02788e6 (#123)
HEAD / FINGERPRINT  see `git log -1` and `git status --short` (kept current at each milestone)
APPROVED SCOPE      design §2.1 as ruled; diff limited to AB-10's list
FROZEN INVARIANTS   §2.3 (I-10, I-16, I-B1 … I-B5); D-B1 … D-B8; P5a's invariants; QS10-19;
                    QP5b-1 (dropped); QP5b-2 (opt-in, off by default, never in an example, gated, last)
ENDPOINT AUTHORITY  implementation + local validation, commits, push, PR create/update, CI repair:
                    authorized (primary session freeze, design §9); merge: operator only
SEQUENCE            C0 … C6; C4 only after C1's OpenAI terms re-read; C5 N/A (QP5b-1)
VALIDATION BUDGET   unit and fake-CLI integration unrestricted; real CLIs, keys, hosted models: none
STOP CONDITIONS     READY FOR OPERATOR REVIEW — DO NOT MERGE; material stops per design §9
CURRENT CHECKPOINT  C0 complete
NEXT ACTIONS        C1: DEP-33, ARC-60, README "Subscriptions"; the OpenAI terms re-read (WebFetch)
```
