# Handoff — S16 PR E-d (Entity Packs)

Continuation aid only; the authority is `step-16-packages.md` §17 (with §17.12). Nothing here
overrides it.

```text
PROJECT / PR        MVP-0 · S16 / PR E-d — Entity Packs: shared item kinds, loaded without a rebuild
PRIMARY DESIGN      .structured-coding/plans/mvp0/step-16-packages.md §17 (DESIGN FROZEN 2026-10-08), §17.12
BINDING             step-16 §§4–9, §14, §15, §16.12; overall.md "Parallel build-out, 2026-10-08"; CLAUDE.md;
                    docs/ENGINEERING_STANDARDS.md, PACKAGE_FORMAT.md, MODULE_SPEC.md, DECISIONS.md ARC-31,
                    ARC-36, ARC-48, ARC-53, ARC-54, ARC-55, ARC-61, DEP-29
BRANCH / WORKTREE   mvp0/pr-ed-entity-packs · /Users/yuema137/mineworld-worktrees/impl-ed (this session only)
BASE                main @ 6ca763d (#93, the E-d freeze, merged); later than the contract's 77a8717
SCOPE               §17.1–17.6 and §17.12, FQ-d1 … FQ-d6 as ruled in §17.0
INVARIANTS          I-E1, I-E2, I-E4, I-E5, I-E6, I-E9; QSE-14; creation order and load.rs beyond PD-36's
                    two functions untouched; PD-q1 … PD-q4
ENDPOINTS           implementation + local validation: authorized (freeze message, §17.11)
                    semantic commits: authorized (§17.11)
                    branch push: authorized (§17.11)
                    PR creation / update, READY FOR OPERATOR REVIEW: authorized (§17.11)
                    CI repair to review readiness: authorized (§17.11)
                    merge: NOT authorized — explicit operator approval only
SEQUENCE            Ed-C1 → C2 → C3 → C4 → (C4b if E-c has not landed `platforms`) → C5; merge origin/main,
                    never rebase
BUDGET              targeted per commit; ED-1's 30-day runs; towns' 300-day runs; one full local gate;
                    CI platforms on the PR; ≈1.5 h
STOPS               any kernel/contracts/persistence/server/clients/systems/cognition change; a digest
                    change; namespaced keys or a new entity type; a platform failure outside S16's files
```

## Current checkpoint

Ed-C0 (design) merged as #93. Session started 2026-10-08 on `6ca763d`, working tree clean. E-c has not
landed (no `platforms` layer on main; no open E-c PR at kickoff). 12d not open.

## Next actions

1. Ed-C1: ARC-71, ARC-54 note, MODULE_SPEC §2/§4.1/§8.1, PACKAGE_FORMAT §5.0/§8; both doc checks.
2. Ed-C2 … C5 per §17.6.

## Environment note

`cargo` is not on the non-interactive shell's PATH: prefix `export PATH=$HOME/.cargo/bin:$PATH`. No
`sed -i`, no `python3 -c`, no heredoc writes.
