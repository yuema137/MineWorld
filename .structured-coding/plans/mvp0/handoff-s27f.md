# Handoff — PR S27-f (guide chapter 14, the agent tool reference, the guide resources)

A continuation aid only. The design authority is [`pr-s27f-agent-guide-reference.md`](pr-s27f-agent-guide-reference.md)
(DESIGN FROZEN 2026-10-18); its §15 is the ledger.

```text
WORKTREE:       /Users/yuema137/mineworld-worktrees/impl-s27f (one session)
BRANCH:         mvp0/pr-s27f, stacked on mvp0/pr-s27e @ 5e9cb04d (S27-e, PR #290, open); retargets to main
                once #290 merges (coordinator's direction, 2026-10-18)
SEQUENCE:       C0 ledger · C1 chapter + mirror · C2 transcript · C3 reference + index · C4 resources ·
                C5 links · C6 review fixes · C7 close-out
LOCAL RULES:    R-LOAD-1 and the lane charter's SPEED FIRST: targeted tests only; cargo through
                cast-slots/with-slot.sh with CARGO_TARGET_DIR=/tmp/impl-s27f-target;
                MINEWORLD_BIN=/tmp/impl-s27f-target/debug/mineworld
ENDPOINT:       PR open against mvp0/pr-s27e, READY FOR OPERATOR REVIEW; never merged by this session
PENDING:        AC-f-8 (the primary session's sign-off of the zh-CN mirror and the plain-language read-through)
```

## State

See §15's commit ledger for each commit's SHA and evidence. The next action after the last recorded commit is the
next commit of the sequence above.
