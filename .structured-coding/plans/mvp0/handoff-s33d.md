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
