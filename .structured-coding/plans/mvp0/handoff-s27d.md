# Handoff — S27-d (MCP stdio adapter)

Continuation aid only. The design `pr-s27d-mcp-stdio.md` (§15 ledger) is the authority.

```text
PROJECT / PR:          MineWorld mvp0, S27-d — stdio MCP adapter and CI play-test (PR #239)
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-s27d-mcp-stdio.md (DESIGN FROZEN 2026-10-15)
BINDING DOCS:          step-27-agent-players.md; pr-s27c-player-library.md; ARC-96, ARC-97, DEP-24, DEP-44, DEP-56
IMPLEMENTATION BRANCH: mvp0/pr-s27d
WORKTREE:              /Users/yuema137/mineworld-worktrees/impl-s27d (one session at a time; resumed by Opus 2026-10-16)
IMPLEMENTATION BASE:   dd357ca6; origin/main @ 8c5d3598 merged in as 1a248cad (documents only)
PREREQUISITES:         #204 (S27-c) and #205 (S27-a) on base; S27-b not merged (optional, R-S27d-b-1 not added)
EXECUTABLE SCOPE:      design §2.1 items 1–7
NEVER:                 design §12 NEVER list (no sed -i, heredoc or python -c on tracked files); no whole-member or
                       workspace local test run (R-LOAD-1); no secrets read
ENDPOINT:              READY FOR OPERATOR REVIEW — DO NOT MERGE
ENDPOINT SOURCE:       design §12 ENDPOINT AUTHORITY (operator dispatch, 2026-10-15); resumed-session brief
                       (2026-10-16): push only complete heads; merge origin/main again once #216 and #224 merge
COMMIT ATTRIBUTION:    Co-Authored-By: Claude Opus 5.5 (1M context) <noreply@anthropic.com>

CURRENT CHECKPOINT:    C0–C7, merge 1a248cad, R1 dfc3b490 (detail echo fix; untitled, compact schemas), R2 d21b4423
                       (answer_lost/not_connected, Alice's record, AC-d-8 comparison, TooFarAway, stricter walk), R3
                       (AC-d-10 rerun, CI record, acceptance table). Head pushed; CI on the exact head is the gate.
SEQUENCE:              C0 … C7 done; R1 … R3 (resumed session) done

EXACT NEXT ACTIONS:
  1. Watch CI on the pushed head (Linux, macOS, Windows). Windows: test-windows' hosted_town flake (#216) and the
     python layer's 100-day timeout (#224) are known and not this PR's; do not chase them.
  2. When #216 and #224 have merged: git merge origin/main; targeted checks; push; record the merge and its CI in §15.
  3. Windows evidence for clients/agent's stdio files (CP-d) is still missing: the Windows python layer stops at
     cognition/lm-controller before reaching clients/agent until #224 lands.
  4. READY FOR OPERATOR REVIEW only with CI green on the exact head; never merge.

STOP CONDITIONS:       design §12 MATERIAL STOPS; any S27-c test red; a result over 8 KiB; the play-test over 180 s on
                       Linux CI

OPEN FOR THE OPERATOR: F-d-R-2 — the tool list is 158.5 KB on the wire (141.7 KB output schemas, repeated across tools
                       because MCP has no shared definitions); further reduction needs a tool-surface decision.

RUNNING PROCESSES:     none locally
```
