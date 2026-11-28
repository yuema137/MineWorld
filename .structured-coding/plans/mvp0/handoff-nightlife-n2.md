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
- N2-C3: committed; PR opened for red-team on C1–C3.
- N2-C4: next (clients/shared/host, 2D app/HUD, 3D needs HUD; Gate2 in a heavy slot).
- Cargo target is deleted after each test cycle (coordinator rule, 2026-11-28): expect a cold rebuild.

## Next action

See the ledger (`pr-nightlife.md` §12) for the current checkpoint.

## C4 audit notes (2026-11-28, on 9b0503ee) — start here

- Shared module: `clients/shared/host/host_clock.gd` (MineWorldHostClock, the only HTTP sender; token from
  `MINEWORLD_ADMIN_TOKEN`, `transport` seam for checks) and `world_tab.gd`. Plan: a sibling `advance.gd`
  (`MineWorldAdvance`): `start(seat)` generates a 32-hex id (`Crypto.generate_random_bytes(16).hex_encode()`),
  `POST /admin/advance {operation, seat, until: "sleep"}`, polls `GET /admin/advance/{op}` while running,
  `cancel()` → `.../cancel`, `on_reconnected()` polls by id and never resubmits; refusal codes →
  `ui.night.unavailable.<code>`; no token → `ui.night.no_admin`. Extend `checks/host_check.gd` with a scripted
  transport. HOST.md §3.2 "a solo world fast-forwarding in S21" → the N2 operation (§0 of the design).
- 2D: `clients/2d/scripts/app.gd::_on_resolved` (accepted `sleep-until` → `advance.start(options.seat)`);
  asleep panel built in `hud/readers.gd::_panel` (`button` = Wake up…) and drawn in `hud/panels.gd::_box` —
  add a "Stop fast-forward" button and a progress row from `clock` frames only; `host_clock` made at app.gd:325.
- 3D: `scripts/slice/slice_clock.gd` owns the host clock; `needs_hud.gd::_show_asleep` shows the wake hint;
  `offer_menu.gd` is where a `sleep-until` offer would be chosen — audit whether the 3D self menu offers it.
- Locale keys go to `clients/shared/settings/locale/{messages.pot,en.po,zh_Hans.po}` (checked by
  `tools/cli` `client_text.rs`).
- The server's `advance_in_progress` refusal and `/admin/advance` contract: `server/PROTOCOL.md` §11.4.
- Gate2: heavy slot only (`cast-slots/with-slot.sh`), ≤ 10 min per run, ≤ 60 min total.
