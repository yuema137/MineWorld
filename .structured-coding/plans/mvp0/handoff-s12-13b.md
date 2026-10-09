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

- C1 `6c3cf9f`; C2+C3 `975704f`; C4 `1476472`; C5 `01bc793`, `98bd42e`; C6 `aca9970`; main merged
  `7a17406` (S11-B on main, E-12). Godot suites green on the C5 tree; mutations recorded in §15.6.
- Final gate on the merged head: running (fmt, clippy, workspace tests, Godot suites, slice drive,
  protocol evidence, scans, scratch).

## Next actions

1. Finish the gate; commit the ledger; push; open the PR READY FOR OPERATOR REVIEW (preview, ARC-24),
   stills and operator checklist in the body; `fast` and `test` green on the exact head.
2. Do not merge. After merge: §15 lifecycle and evidence (this session); step/overall: primary session.

## Background processes

Only processes this session started (by PID), never by name.
