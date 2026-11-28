# Handoff — Nightlife N2 (sleep-until + solo advance)

Continuation aid only; `pr-nightlife.md` §§0–4, 6 and its §12 N2 ledger are the authority.

```text
Branch / worktree:   mvp0/pr-nightlife-n2 @ /Users/yuema137/mineworld-worktrees/impl-nightlife-n2 (one session)
Implementation base: origin/main bdb83457 (after #552 N1, #549 launcher P2, #561 launcher P3, #566)
Cargo target:        /Users/yuema137/mineworld-worktrees/_targets/nightlife-n2 (APFS clone of fix-main-regressions-2)
Heavy jobs:          only through /Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh
Contract:            pr-nightlife-contract.md "Contract N2"; endpoint = PR READY FOR OPERATOR REVIEW, no merge
```

## State

- N2-C1: committed (e0f22c7b); re-checked after merging origin/main 805498ce (D-N2-8).
- N2-C2: committed (advance contract, lifecycle, durable identity; no loop).
- N2-C3: next. D-N2-6 (non-blocking queue check) is recorded; freeze the oracle fixture first.
- N2-C4: not started.
- Cargo target is deleted after each test cycle (coordinator rule, 2026-11-28): expect a cold rebuild.

## Next action

See the ledger (`pr-nightlife.md` §12) for the current checkpoint.
