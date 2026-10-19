# Handoff — S27-e (MCP over Streamable HTTP, several agents per adapter)

Continuation aid only. The design `pr-s27e-streamable-http.md` (§15 ledger) is the authority.

```text
PROJECT / PR:          MineWorld mvp0, S27-e — the MCP adapter over Streamable HTTP, several agents per adapter
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-s27e-streamable-http.md (DESIGN FROZEN 2026-10-18)
BINDING DOCS:          step-27-agent-players.md; pr-s27d-mcp-stdio.md; pr-s27c-player-library.md; ARC-96, ARC-97, DEP-56
IMPLEMENTATION BRANCH: mvp0/pr-s27e
WORKTREE:              /Users/yuema137/mineworld-worktrees/impl-s27e (one session)
IMPLEMENTATION BASE:   origin/main @ a1e05874 (S27-d merged as #239, 42bc3ceb)
EXECUTABLE SCOPE:      design §2.1 items 1–6
NEVER:                 design §12 NEVER list; no whole-member or workspace test run (R-LOAD-1); heavy jobs only
                       through cast-slots/with-slot.sh; no secrets read
ENDPOINT:              READY FOR OPERATOR REVIEW — DO NOT MERGE
COMMIT ATTRIBUTION:    Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
PARALLEL CHANGE:       PR #261 (S27-d1, ARC-153) merged: nine tools, the five acts are kinds of `act`; merged into
                       this branch (dbc6d888) and adapted.

SEQUENCE:              C0 ledger + DEP-56 note · C1 prototypes · C2 pool · C3 server + gate · C4 CLI ·
                       C5 HTTP play-test · C6 mixed play + endpoint · C7 close-out
CURRENT CHECKPOINT:    C7 done; PR open, MERGE-READY for the coordinator (see §15)
STOP CONDITIONS:       design §12 MATERIAL STOPS
RUNNING PROCESSES:     none
```
