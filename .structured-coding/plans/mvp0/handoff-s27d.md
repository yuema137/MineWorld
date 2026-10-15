# Handoff — S27-d (MCP stdio adapter)

Continuation aid only. The design `pr-s27d-mcp-stdio.md` (§15 ledger) is the authority.

```text
PROJECT / PR:          MineWorld mvp0, S27-d — stdio MCP adapter and CI play-test
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-s27d-mcp-stdio.md (DESIGN FROZEN 2026-10-15)
BINDING DOCS:          step-27-agent-players.md; pr-s27c-player-library.md; ARC-96, ARC-97, DEP-24, DEP-44, DEP-56
IMPLEMENTATION BRANCH: mvp0/pr-s27d
WORKTREE:              /Users/yuema137/mineworld-worktrees/impl-s27d (this session's only)
IMPLEMENTATION BASE:   dd357ca6 (origin/main moved to e6030a36 later; the branch does not rebase)
PREREQUISITES:         #204 (S27-c) merged, ancestor of base: yes. #205 (S27-a): yes. S27-b: not on base (optional, R-S27d-b-1 not added)
EXECUTABLE SCOPE:      design §2.1 items 1–7
NEVER:                 design §12 NEVER list; no full-workspace or whole-member local test run (R-LOAD-1); no secrets read
ENDPOINT:              READY FOR OPERATOR REVIEW — DO NOT MERGE
ENDPOINT SOURCE:       design §12 ENDPOINT AUTHORITY (operator dispatch, 2026-10-15)
COMMIT ATTRIBUTION:    per the operator's dispatch, commits end with Co-Authored-By: Claude Opus 5.5 (see final report)

CURRENT CHECKPOINT:    C0 done (ledger row, DEP-56, this file). Next: C1.
SEQUENCE:              C0 ledger+DEP-56 · C1 dependency + SDK prototype (STOP if P-1/P-2 fail) · C2 library additions ·
                       C3 MCP server · C4 CLI + golden · C5 stdio play-test · C6 same answers/guard/admin · C7 close-out, PR, CI

EXACT NEXT ACTIONS:
  1. C1: pick mcp version by the §4.1 date rule (2026-10-15 → mcp>=2.2,<2.3); uv lock; uv sync --locked.
  2. Record package additions and licences in design §15.
  3. Read installed mcp package for the import paths; record in §15.
  4. Write test_mcp_confined.py with P-1 and P-2 prototypes; run; stop on failure.

STOP CONDITIONS:       design §12 MATERIAL STOPS; a GPL-family transitive licence; P-1 or P-2 failing; any S27-c test red;
                       a result over 8 KiB; the play-test over 180 s on Linux CI

DEVIATIONS SO FAR:     S27-b absent (optional); D-7 `_counts` not present (hook location in `_ingest` unchanged)

RUNNING PROCESSES:     none
CI STATUS:             none pushed yet
