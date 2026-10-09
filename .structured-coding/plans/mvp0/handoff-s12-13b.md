# Handoff — S12 PR 13b (menu interactions through affordances)

A continuation aid, never a design authority. The authority is
[`step-13-client-2d.md`](step-13-client-2d.md) §15 (DESIGN FROZEN 2026-10-08), its rulings and its
execution contract §15.11. Kept as its own file, as 13a's was (§14.10 D-1): lanes run in parallel.

```text
PROJECT / PR        S12 PR 13b — menu interactions through affordances
DESIGN              step-13-client-2d.md §15; contract §15.11; live ledger §15.12
RELATED             step-13 §§1–14; overall.md "Parallel build-out", "Framework, not demo",
                    "One world, two views", "The World Interaction List"; CLAUDE.md;
                    ENGINEERING_RULES §§4, 7–9, 19, 22; server/PROTOCOL.md; clients/protocol/ADOPTION.md;
                    clients/2d/PRESENTATION.md
BRANCH / WORKTREE   mvp0/pr-13b-interactions · /Users/yuema137/mineworld-worktrees/impl-13b
BASE                main @ aa74b32 (≥ 9cf8f8e; IL-a #80, 13a #82, CI #63 on main; 12d and S11-B not)
SCOPE               §15.0–§15.8 as amended by the freeze rulings (QS13b-1 … QS13b-8)
INVARIANTS          I-1 … I-9 (§8.1); no edit to clients/protocol/**, clients/3d-spike/**, server, kernel,
                    contracts, System or World Packs; AC-I1 … AC-I12 as written; ARC-70 + an ARC-47 note
SEQUENCE            C1 → C6 (§15.6)
BUDGET              §15.8: each Godot run ≤ 10 min, > 2 min in the background; ≈ 1 h validation
ENDPOINTS           implement + validate, semantic commits, push, PR create/update, CI repair: authorized
                    (§15.11, primary session's freeze 2026-10-08; kickoff message 2026-10-08).
                    Merge: operator only — DO NOT MERGE.
STOPS               §15.11 material stops; RK-b2 / RK-b3 beyond a consumer change
PLATFORMS           macOS, Linux, Windows (operator, 2026-10-08): no new Unix-only assumption; existing
                    ones recorded as findings with their owning lane
```

## Checkpoint

- Session started 2026-10-08; worktree created from origin/main aa74b32; §15 re-audited (§15.12 E-0).
- Next: C1 (specs and wording).

## Next actions

1. C1: ARC-70 + ARC-47 note; PRESENTATION.md "Wording"; `i18n/en.po`; `--check-pack` parses `.po`.
2. C2 … C6 per §15.6.

## Background processes

Only processes this session started (by PID), never by name.
