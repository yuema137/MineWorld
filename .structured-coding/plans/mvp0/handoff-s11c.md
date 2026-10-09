# Handoff — S11: PR S11-C implementation (facts in observations, perceived, acted_through, deltas)

A continuation aid, never a design authority. The authority is
[`step-12-server.md`](step-12-server.md) §17 (its freeze record binds), with evidence in §17.12
(`E-SC<n>`) and deviations in §17.13 (`D-SC<n>`).

```text
PROJECT / PR        MVP-0 · Step 12 (S11) / PR S11-C — facts in observations, the perceived stream,
                    acted_through, deltas
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-12-server.md §17; evidence §17.12; deviations §17.13
RELATED / BINDING   step-12 §§4.7, 4.8, 5, 6, 7.3, 7.8, 8, 9.3, 15, 16 (on mvp0/pr-s11b-seats until #83
                    merges), 19; server/PROTOCOL.md rev 2; overall.md rulings 1, 2, 4, 6, 9, 10, QIL-8;
                    step-17 §§3.3, 10, 11.1; DECISIONS ARC-23, 25, 28, 40, 41, 42; CLAUDE.md §§2–4
BRANCH / WORKTREE   mvp0/pr-s11c-perception in /Users/yuema137/mineworld-worktrees/impl-s11c (sole writer)
BASE                origin/main @ ec38570 merged in b314dc4 (S11-B #83 merged as 15b05a9); draft PR #95
APPROVED SCOPE      §17.6; C-C1 … C-C10 with C-C3b; SD-C1 … SD-C14 as ruled; §17.14
FROZEN INVARIANTS   I-1 no kernel/contracts/persistence diff; I-6 digests = E-SC0, observe and run.rs
                    untouched; I-7 facts only through EventPerception; I-9 no pack in server/; I-10 if
                    deltas ship; I-11 no history read or wait on the world thread; I-12 ac13/ac15 green;
                    every existing module name and call valid
ENDPOINT AUTHORITY  implementation, commits, push, PR create/update, CI repair: authorized (primary
                    session's freeze 2026-10-08, §17.11); merge: operator only
VALIDATION BUDGET   §17.11 (CP-C1 ≤ two 60 s runs; CA-13 once; digests ≤ four; one full gate)
STOP CONDITIONS     normal: READY FOR OPERATOR REVIEW — DO NOT MERGE; material: §17.11 MATERIAL STOP;
                    coordinator: S11-B still open 3 h after the wait began → stop and report
CURRENT CHECKPOINT  C-C1 … C-C5 committed and pushed (head 375b090); E-SC0 … E-SC6 recorded
NEXT ACTIONS        C-C6: tools/cli perceive.rs PackEventPerception, history.rs SavedHistory, serve.rs
                    wiring, main.rs < 500 (516 now); tests CA-2, CA-3, CA-4 (binary), CA-6, CA-11,
                    CA-12 persisted; then C-C7 (entity order, delta.rs, CP-C1), C-C8, C-C9 (Godot),
                    C-C10 (CA-13, digests, full gate, READY)
```
