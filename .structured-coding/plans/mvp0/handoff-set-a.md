# Handoff — S20 PR SET-a (the shared settings module, language, display, clock and menu)

A continuation aid, never a design authority. The authority is
[`step-20-client-settings.md`](step-20-client-settings.md) §12 (DESIGN FROZEN 2026-10-08), its rulings
(§1.5) and its execution contract (§12.9). Kept as its own file, because lanes run in parallel.

```text
PROJECT / PR        S20 PR SET-a — settings module, language, display, clock, menu, in both clients
DESIGN              step-20-client-settings.md §12; contract §12.9; live ledger §12.10 (+ §12.11 findings)
RELATED             step-20 §§1–11; CLAUDE.md; overall.md "Framework, not demo" item 4, "One world, two
                    views"; step-13 §15 (13b, ARC-70); step-15 (3D slice); step-19 §8; ADOPTION.md §1;
                    DECISIONS DEP-8, ARC-55, ARC-70; ENGINEERING_RULES §§2–3, 10–12
BRANCH / WORKTREE   mvp0/pr-set-a · /Users/yuema137/mineworld-worktrees/impl-set-a (this session only)
BASE                origin/main @ aee8290 (13b merged #103; S11-D, 16a, IL-b on main)
SCOPE               §12.1 (goal, non-goals), files of §12.4
INVARIANTS          INV-SET-1 … 10 (§5); material stops §12.9 (1)–(7); no edit to clients/protocol/mineworld,
                    server, contracts, kernel, System or World Packs; 13b's keys never change
SEQUENCE            C0 → C7 (§12.5)
BUDGET              Godot runs as the checks need; no long town runs; long jobs in the background
ENDPOINTS           implement + validate, semantic commits, push, PR create/update, CI repair: authorized
                    (source: §12.9 "Authority"; primary session's kickoff 2026-10-09).
                    Merge: operator only — DO NOT MERGE.
STOPS               §12.9 material stops (1)–(7)
PLATFORMS           macOS gate; Windows and Linux hand checks are operator checklist items (§8)
```

## Checkpoint

- Worktree created from origin/main @ aee8290; anchors re-verified (§12.11 F-1 … F-12).
- C0/C1 in progress.

## Next actions

1. C1 specs and records; commit; push.
2. C2 store + display + store_check.gd; C3 text layer + catalogs + font + client_text.rs; C4 menu;
   C5 2D; C6 3D; C7 evidence and gate; PR READY FOR OPERATOR REVIEW.

## Background processes

Only processes this session started (by PID), never by name. None now.
