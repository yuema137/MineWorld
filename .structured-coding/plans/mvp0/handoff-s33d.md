# Handoff — PR S33-d (Lakeside's land, water and village in 3D, from a pack data file)

A continuation aid only; the authority is [`pr-s33d-terrain-water-exterior.md`](pr-s33d-terrain-water-exterior.md)
(revision 2, frozen 2026-10-20) and its §15 ledger.

- Worktree `/Users/yuema137/mineworld-worktrees/impl-s33d`, branch `mvp0/pr-s33d`. Stacked on #387 (S33-a),
  #395 (S33-b) and #400 (the stub follow-up), all open under the main merge freeze. Do not merge `origin/main`
  directly; merge those branches' new heads when they move.
- This run's scope: C0…C3. C3 first in priority: 16f-c and 16f-d1 wait on `SliceDataDepiction` and
  `SliceDepictionFile` (16f §13: the data depiction is built with no world key in its constructor; format 1
  accepts an additive optional section later).
- Machine reserved for the operator's video: no local cargo, Godot or Blender. Validation is GitHub CI on the
  PR; checks that do not run per-PR (the Godot `--link` probe, `client_depiction`'s ignored test) are
  INCONCLUSIVE in §15 until the machine is free.
- Depictions live under `clients/3d-spike/scripts/depictions/` (D-S33c-1). Generic builders under
  `scripts/depictions/data/` name no world, place key, tag or species (I-d-9). Furniture never comes from the
  depiction file (`ARC-307`).
- Merge is the coordinator's.

## State (2026-10-21, C13)

- C0…C13 committed on PR #452 (C10 and C11 in one commit). Ready for operator review of the V-gate shots
  (`clients/3d-spike/shots/lakeside/review/`, `shots/lakeside-v7/review/`).
- Open: Q-S33d-7 (S33-a's far-field ID rows at Lakeside's low eyes; the nightly Lakeside 3D far-field check is
  red by them until ruled); AC-d-5 perf counts (run when the machine is idle); the S33-e/f rows of
  `--world=lakeside --geometry` (the line joins `clients-probes` when both land); M-d-4, M-d-5 at the milestone.

## State (2026-10-21, C3)

- PR #452 (draft). C0 `37e34ac8`, C1 `f17aebe2`, C2 `552690f1`, C3 `e3e8f8ee`; #395 (`b6d60223`) and #400
  (`cfb7890d`) merged in. `fast` green on C2 and C3.
- Pending the machine (INCONCLUSIVE in §15): `cargo test -p mineworld-packages --test depiction`, `-p
  mineworld-cli --test packs`, `client_rules`, `client_depiction -- --ignored`, `./mineworld-slice
  --world=lakeside --link`, Social Café / Market Town baseline captures (AC-d-6).
- Next: C4 terrain builder (`scripts/depictions/data/terrain_builder.gd`), adding `terrain`/`paths` to
  `lakeside.yaml`; then C5…C13. New tree forms or cards go into `SliceDepictionFile.FORMS` / `CARDS`.
