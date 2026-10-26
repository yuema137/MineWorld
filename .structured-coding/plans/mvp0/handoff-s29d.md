# Handoff — S29-d (the 3D depot exterior)

A continuation aid, never a second design authority: the design and its ledger are
[`pr-s29d-depot-exterior.md`](pr-s29d-depot-exterior.md) §15.

- **Worktree / branch:** `/Users/yuema137/mineworld-worktrees/impl-s29-d`, `mvp0/pr-s29d`, from origin/main
  `68a248d1` (S29-a #348, S29-c #357, S33-c #381, 16c #245, 12e #254, vf-furnishings merged; RL-b2 not needed).
- **Endpoint:** MERGE-READY (pushed, `fast` green on the exact head); the coordinator merges.
- **Where the work is:** `clients/3d-spike/scripts/station/` (kit, tile, depot, views, kit probe),
  `shaders/glazed_tile.gdshader`, `scripts/slice/slice_landmark_check.gd`, the forecourt in `street.gd`, the
  site in `depictions/social-cafe/depiction.gd`, the kit `presentation/mineworld-default/3D/stations/depot.yaml`,
  the references `presentation/mineworld-default/3D/references/station/`.
- **Milestone runs (SPEED FIRST):** AC-d-3 … AC-d-6, AC-d-8, X-6 and M-d-2, M-d-4, M-d-5; commands in §15.
- **For S29-e (R-S29e-d-1):** read every fit-out number from a `fitout` group of the kit (`StationKit` accepts it
  absent today; add its keys to `StationKit.GROUPS` when it lands); use `StationDepot.report` (`origins`,
  `volumes`, `doors`) for the volumes and doors this PR creates; keep new pieces under a `station.` category; add
  `st_5` … `st_8` to `StationViews.shots`. The hall's ceiling at the eaves is a placeholder S29-e replaces.
