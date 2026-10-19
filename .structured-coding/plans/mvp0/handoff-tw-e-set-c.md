# Handoff — S19 TW-e + S20 SET-c (the world clock in the clients)

Continuation aid only. The design authority is
[`pr-tw-e-set-c-world-clock-clients.md`](pr-tw-e-set-c-world-clock-clients.md); where this file and it
disagree, the design wins.

```text
PROJECT / PR:          MineWorld mvp0, S19 TW-e + S20 SET-c — the world clock in the clients
PRIMARY DESIGN DOC:    .structured-coding/plans/mvp0/pr-tw-e-set-c-world-clock-clients.md (DESIGN FROZEN 2026-10-18)
RELATED / BINDING:     step-19 (§§1.3, 7.5, 8, 10, 11.2, 14.1); step-20 (§§1.5, 3.2, 3.10, 5, 6, 8);
                       pr-tw-c-host-clock.md §§5, 11, 15; clients/shared/SETTINGS.md; server/PROTOCOL.md §§5.9, 11;
                       docs/DECISIONS.md ARC-44, ARC-67, ARC-69, ARC-70, ARC-76, DEP-30, DEP-35; CLAUDE.md
WORKTREE / BRANCH:     /Users/yuema137/mineworld-worktrees/impl-tw-e-set-c, mvp0/pr-tw-e-set-c (one session)
IMPLEMENTATION BASE:   origin/main @ a256e4f8 (after #265 merged as ff654992); only docs moved since the
                       planning base b85f2804
APPROVED SCOPE:        design §3.1, with C2 (POST /admin/stop, Q-WC-1 (a)) and C2b (social-cafe calendar,
                       Q-WC-3 yes) both ruled in
FROZEN INVARIANTS:     design §3.3; SD-WC-1 … SD-WC-8
SEQUENCE:              C0 … C9 (design §10)
VALIDATION BUDGET:     targeted cargo tests and Godot checks through cast-slots/with-slot.sh (2 slots);
                       no local full-workspace cargo test (R-LOAD-1); component-PR push runs only `fast`
DECISION IDS:          ARC-266 … ARC-272
STOP CONDITION:        TRAIN-READY — report head SHA, local validation, review verdict, mutation evidence; DO NOT MERGE
```

## Endpoint authority

| Endpoint | Decision | Source |
| --- | --- | --- |
| implementation + local validation | authorized | design §13, operator freeze 2026-10-18 |
| semantic commits | authorized | design §13 |
| branch push | authorized | design §13 |
| PR creation / update | authorized | design §13 |
| CI repair | authorized | design §13; push runs only `fast` (operator CI policy 2026-10-18, coordinator brief) |
| merge | never by this session; merge train only | design §13; coordinator brief "Never merge" |

## Current checkpoint

C0 in progress (2026-10-18).

## Start-condition finding (C0)

The contract's START CONDITION asks for no open PR editing `app.gd`, `hud/status.gd`, `slice_main.gd`,
`slice_link.gd`, `far_field_view.gd`, `mineworld-2d`, `mineworld-slice`. At C0 open PRs #245 (16c),
#250 (16d), #254 (12e) edit `slice_main.gd`, `slice_link.gd`, `mineworld-slice` and the shared catalogs;
#212 (draft, parked) edits `slice_main.gd`, `mineworld-slice`; #171 and #146 edit `clients/2d/scripts/app.gd`;
#148 edits `tools/cli/src/serve.rs`. Recorded in the design ledger (F-WC-6) and reported to the
coordinator: the coordinator launched this session with those PRs open, so the work proceeds, keeping
edits to those files to calls and small blocks so a later rebase is mechanical.

## Exact next actions

1. C1 specifications.
2. C2 `/admin/stop`; C2b social-cafe calendar.
3. C3 … C9.

## Background jobs

- `cargo build -p mineworld-cli --bin mineworld` through with-slot (log in the session scratchpad
  `build1.log`).
