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

Session started 2026-10-08. Worktree created from `origin/main @ 6ca763d`; skill, working rules, test
rules and §16 read; pack repository cloned (it holds GitHub's initial `LICENSE` commit `ac32b96`).
Next: Ec-C1 (docs).

## Environment note

`cargo` is not on the non-interactive shell's PATH: prefix `export PATH=$HOME/.cargo/bin:$PATH`.
No `sed -i`, no `python3 -c`, no heredoc writes, no `awk`/`xargs`/`curl`.
