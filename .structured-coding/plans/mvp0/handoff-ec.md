# Handoff — S16 PR E-c (a System Pack from outside the repository)

Continuation aid only; the authority is `step-16-packages.md` §16 (with §16.12). E-b's handoff
(`handoff-eb.md`) is closed; its state is in §15.

```text
PROJECT / PR        MVP-0 · S16 / PR E-c — a System Pack from outside the repository
PRIMARY DESIGN      .structured-coding/plans/mvp0/step-16-packages.md §16 (DESIGN FROZEN 2026-10-08)
BINDING             step-16 §§3–9, §14, §15; overall.md "Parallel build-out"; CLAUDE.md;
                    ENGINEERING_STANDARDS, REUSE_POLICY, PACKAGE_FORMAT, MODULE_SPEC, DECISIONS
BRANCH / WORKTREE   mvp0/pr-ec-third-party · /Users/yuema137/mineworld-worktrees/impl-ec
EXTERNAL CLONE      /Users/yuema137/mineworld-worktrees/ext-fishing (yuema137/mineworld-pack-fishing, main)
BASE                main @ 6ca763d (#93 merged; IL-a, E-b, CI on main)
SCOPE               §16.1–16.6 and §16.12 with FQ-c1 … FQ-c10 as ruled in §16.0
ENDPOINTS           implement, commit, push this branch, open/update the PR READY FOR OPERATOR
                    REVIEW, CI repair: authorized (freeze message, §16.11). Push commits and tags to the
                    pack repository's main, never force-push: authorized (operator, FQ-c5, and the
                    kickoff message). Merge: NOT authorized.
SEQUENCE            Ec-C1 → C2 → C3 → C4 (external, 3-OS CI) → C5 → C6 → C7 → C7b → C8
STOPS               edit to ac1_composability.rs or ARC-35; kernel/contracts/persistence/server/
                    clients/cognition/worldpack/src change; a digest change; the install needing a
                    third line; a platform failure whose fix leaves S16's files
VALIDATION BUDGET   targeted per commit; towns' 300-day runs; EC-3 vendor outside the tree; one
                    full local gate; CI fast/test/platforms on the PR; ≈2 h
```

## Current checkpoint

Ec-C1 … C6 committed and pushed (`1b59dba`, `d56729c`, `e127e1c`, `5a1d33e` ledger of C4, `b5aebd8`
install, `51f0382` proofs). The pack is `b40e71f` on the pack repository's `main`, tag `v0.1.0`, its CI
green on ubuntu/macos/windows (run 37907882591). Working on Ec-C7 (`deny.toml`, Dockerfile, `fast`) and
C7b (`platforms` layer, `.github/actions/native` per step-14 §13.0.3, job on `macos-26`,
`windows-2025`). origin/main has moved (13b plan #88, S11b #83): merge before C8's gate. Local
`/tmp/ec-check/market-town` is the EC-1 scratch world (delete at close). Local pack-dev config:
`/Users/yuema137/mineworld-worktrees/ext-fishing-local.toml` and target `ext-fishing-target` (delete at
close).

## Environment note

`cargo` is not on the non-interactive shell's PATH: prefix `export PATH=$HOME/.cargo/bin:$PATH`.
No `sed -i`, no `python3 -c`, no heredoc writes, no `awk`/`xargs`/`curl`.
