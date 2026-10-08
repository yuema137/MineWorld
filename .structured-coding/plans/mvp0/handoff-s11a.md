# Handoff — S11: PR S11-A implementation (handshake and authentication)

A continuation aid, never a design authority. The authority is
[`step-12-server.md`](step-12-server.md) §15 (its freeze record binds), with evidence in §15.10
(`E-SA<n>`) and deviations in §15.11. A file of its own rather than `handoff.md` because six lanes run in
parallel (deviation D-SA1).

```text
PROJECT / PR        MVP-0 · Step 12 (S11) / PR S11-A — handshake and authentication, protocol revision 2
                    part 1
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-12-server.md §15; evidence §15.10; deviations §15.11
RELATED / BINDING   step-12 §§4.1, 5, 6, 7.1, 7.2, 7.7, 8, 9.1, 11.3, 11.4; overall.md "Parallel build-out,
                    2026-10-08"; NETWORKING.md; server/PROTOCOL.md; DECISIONS DEP-3, ARC-23, ARC-25
BRANCH / WORKTREE   mvp0/pr-s11a-handshake in /Users/yuema137/mineworld-worktrees/impl-s11a (sole writer)
BASE                main @ 47c81d1; merge origin/main at each rebase (16b, S12 13a land first)
APPROVED SCOPE      §15.5's change set (plus clients/2d call site + launcher, freeze); A-C1 … A-C8
FROZEN INVARIANTS   no diff under kernel/, contracts/, persistence/, worlds/, worldpack/, systems/,
                    cognition/, authoring/, sdk/, tests/acceptance/; no secret or nickname on the world
                    thread; digests = E-SA0; ac13/ac15/milestones unedited; 16b names unchanged
ENDPOINT AUTHORITY  implementation, commits, push, PR: authorized (freeze message 2026-10-08);
                    merge: operator only
VALIDATION BUDGET   §15.9
STOP CONDITIONS     normal: READY FOR OPERATOR REVIEW — DO NOT MERGE; material: §15.8
CURRENT CHECKPOINT  A-C1 (specs) committed; next A-C2 (protocol.rs split)
NEXT ACTIONS        A-C2 → A-C3 → A-C4 → A-C5 → A-C6 → merge origin/main → A-C7 (Godot, one window at a
                    time) → A-C8
```
