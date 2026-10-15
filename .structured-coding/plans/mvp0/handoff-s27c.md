# Handoff — PR S27-c (the `Player` core library, `clients/agent/`, no MCP)

A continuation aid only. The authority is `pr-s27c-player-library.md` (design, contract §12, ledger §15).

```text
PROJECT / PR         MVP-0 · S27 / PR S27-c — the Player core library (clients/agent, mineworld-agent, no MCP)
PRIMARY DESIGN DOC   .structured-coding/plans/mvp0/pr-s27c-player-library.md
RELATED / BINDING    step-27-agent-players.md (§§3–7, 9, 11, 12, 16); pr-s10-p3-python-sdk.md; pr-s10-p3b-perceived.md;
                     pr-s10-p6-lm-controller.md §5.2.2 (contrast); server/PROTOCOL.md; clients/protocol/ADOPTION.md §4;
                     docs/CORE_CONCEPTS.md §§6.1, 14, 15; docs/DECISIONS.md ARC-34, ARC-56, ARC-67, ARC-75, DEP-24,
                     DEP-26, DEP-44; .structured-coding/standards.md; CLAUDE.md; ENGINEERING_STANDARDS; ENGINEERING_RULES
BRANCH / WORKTREE    mvp0/pr-s27c · /Users/yuema137/mineworld-worktrees/impl-s27c (one writer: this session)
BASE                 origin/main @ f772a18 (S27-a not merged; S27-b not merged)
FROZEN               DESIGN FROZEN 2026-10-14, revision 2 (primary; operator QS27c-2; rulings §14.1)
APPROVED SCOPE       §2.1 items 1–4 (item 3, ResumingSeat.clock, as ruled QS27c-1)
FROZEN INVARIANTS    step-27 INV-S27-1 … INV-S27-13 as §6 maps them (I-c-1 … I-c-14); step-27 §5's surface; §8 as
                     ruled (AC-c-7 replaces AC-S27-7, QS27c-2); no mcp; no new dependency; no path under server/,
                     contracts/, kernel/, systems/, worlds/, cognition/
ENDPOINT AUTHORITY   implementation + local validation: authorized (primary, at freeze; dispatch 2026-10-14)
                     semantic commits: authorized (working rules §14)
                     branch push, PR creation/update, CI repair: authorized (working rules §21)
                     merge: never by this session; operator's explicit authorization only (§22)
SEQUENCE             C0 … C9 (§9)
VALIDATION BUDGET    unit unrestricted; each real-binary test file ≤ 90 s on Linux; CI: the PR's runs; Gate 1 NOT
                     REQUIRED (no model is called)
BINARY               export PATH="$HOME/.cargo/bin:$PATH"; export CARGO_TARGET_DIR=/tmp/impl-s27c-target;
                     cargo build -p mineworld-cli; MINEWORLD_BIN=/tmp/impl-s27c-target/debug/mineworld
STOP                 S27-c READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## Current checkpoint

C0–C5 committed (`5f3d48e` … `4b94d28`), the gap table (`2525c34`), and C5a: ruling R-S27c-INFO applied
(`disclosed.py`; `MeLine`/`PersonLine`/`OfferLine`/`View` fields; acquaintances and invitations paged; 12
mutations red by name; ledger §15 "Ruling R-S27c-INFO"). This session replaced the stopped implementer (one
writer). Coordinator ruling R-LOAD-1 in force (only touched tests locally, ruff/format/pyright; never a local
`cargo test --workspace`; CI on the exact head is the workspace gate).

## Next actions

1. C6 (`b9f571f`), C7 (`b57ea61`), C8 (`abd37a4`, real binary, all four files PASS on macOS), C9 ledger
   (F-S27c-9: 12d not merged; AC-c-12 per R-LOAD-1) done.
0. **MATERIAL STOP (CI, head 94e5b70, PR #204).** `python (ubuntu-24.04)` failed at
   `cognition/lm-controller/tests/test_structural_isolation.py::test_ci_runs_pytest_once_per_member`:
   it asserts exactly 2 pytest lines in `ci_layer.py --list python|python-smoke`, and the member makes 3.
   The fix is in `cognition/`, which the contract forbids. It needs the operator's authorization (or another
   way to wire the member). The proposed fix: one line per `PYTHON_MEMBERS`, with the agent line asserted.
   Because the layer stops at that point, the agent tests have not yet run on CI.
2. Push; `gh pr create`; CI on the exact head (fast, test, test-macos, test-windows, platforms, python ×3);
   repair if red; then READY FOR OPERATOR REVIEW — never merge.
3. After 12d merges (separately authorised): merge `origin/main`, check item names (ruling 2).
