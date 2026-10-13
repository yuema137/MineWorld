# Handoff — S11: PR S11-B implementation (seats, hold and resume, takeover, hosted controllers, F-13)

A continuation aid, never a design authority. The authority is
[`step-12-server.md`](step-12-server.md) §16 (its freeze record binds), with evidence in §16.10
(`E-SB<n>`) and deviations in §16.11 (`D-SB<n>`). A file of its own because several lanes run in
parallel (D-SA1's precedent).

```text
PROJECT / PR        MVP-0 · Step 12 (S11) / PR S11-B — seats, hold and resume, takeover, hosted
                    controllers, F-13
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-12-server.md §16; evidence §16.10; deviations §16.11
RELATED / BINDING   step-12 §§4.2–4.6, 5, 6, 7.4, 7.5, 8, 9.2, 11, 15; server/PROTOCOL.md revision 2;
                    overall.md "Parallel build-out, 2026-10-08" rulings 1, 3, 6, 7, 9, 10;
                    DECISIONS DEP-14, ARC-41, ARC-23, ARC-25, ARC-27; NETWORKING.md; CLAUDE.md §§2–4
BRANCH / WORKTREE   mvp0/pr-s11b-seats in /Users/yuema137/mineworld-worktrees/impl-s11b (sole writer)
BASE                main @ f842c52 (design commit 1a46de6 on top); merge origin/main before the final
                    gate (E-b, IL-a, test hygiene may land first)
WORKING TREE        clean at 1a46de6 when this file was initialized
APPROVED SCOPE      §16.5's change set; B-C1 … B-C8; SD-B1 … SD-B14 as answered by QS11B-1 … QS11B-6
FROZEN INVARIANTS   I-1 no diff under kernel/, contracts/, persistence/ (nor worlds/, worldpack/, systems/,
                    authoring/, sdk/, tests/acceptance/); I-2/ARC-40 a binding change moves no revision;
                    I-3 one controller per seat; I-5 secrets in no save/observation/fact/status/log;
                    I-6 digests = E-SB0, run.rs untouched; I-8 hosted requests take submit_at; I-9 no
                    controller crate in server/; I-11 nothing waits on the world thread; 16b's GDScript
                    names unchanged; four-argument connect_to_world calls valid
ENDPOINT AUTHORITY  implementation + local validation, commits, push, PR creation: authorized (the
                    primary session's freeze message and S11-B brief, 2026-10-08); CI repair: if a
                    workflow exists on the final head; merge: operator only, never inherited
VALIDATION BUDGET   §16.9 (~1.5 h total; 300-day runs ≤ 6; Godot one window at a time, ≤ 3 per mode;
                    one full gate on the final head, in the background)
STOP CONDITIONS     normal: PR S11-B READY FOR OPERATOR REVIEW — DO NOT MERGE; material: §16.8 (a
                    kernel/contract/persistence edit, a path outside §16.5, a digest change, CP-B4's
                    50 ms bound failing)
CURRENT CHECKPOINT  B-C1 … B-C7 committed and pushed; origin/main (E-b #78, S19 plan #81) merged;
                    QTW-13 amendment applied (D-SB6); digests = E-SB0 (E-SB7); full gate on the
                    head, then the PR as READY FOR OPERATOR REVIEW — DO NOT MERGE.
                    A resumed session (rate limit) audited and kept the 13 uncommitted files (D-SB4).
NEXT ACTIONS        operator review; on merge, the post-merge sync of §16 (this session's) and
                    §§1–14 / overall.md / MVP_STATUS (the primary session's). If IL-a lands first,
                    its drift call goes into tools/cli/src/serve.rs::persisted.
BACKGROUND JOBS     none after the gate
```
