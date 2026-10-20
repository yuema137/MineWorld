# Handoff — S29-f, the station in 2D

A continuation aid for `pr-s29f-station-2d.md`; that design and its §15 ledger are the authority.

**Worktree** `/Users/yuema137/mineworld-worktrees/impl-s29-f`, branch `mvp0/pr-s29f`, PR #366.

**Done.** C1–C8: docs, the shared fixture rule, the 2D mechanics, the words, `fit.py` and the recipe, 27
candidates (US$ 2.0967), the promoted sprites with their bindings and specs, the drive and runner, the station
stills and re-captured far-field views of the three towns.

**Open.** Operator gate 2: a verdict on `presentation/mineworld-default/2D/candidates/station/contact_sheet.png`.
The implementer's picks are promoted provisionally (F-S29f-impl-8). To swap one, from the repository root:

```sh
PYTHONPATH=tools/asset_generation ~/.cache/mineworld-gen/venv/bin/python -m mineworld_gen.fit \
  presentation/mineworld-default/2D/candidates/station/<item>_<n>.png \
  presentation/mineworld-default/2D/art/generated/sib_<item>.png --height-m <h>
```

then paste the printed block over the sprite's entry in `assets/asset_bindings.yaml`. The unchosen candidate
PNGs exist only in this worktree. At the milestone: `client_2d_station.rs` R-1…R-5 and M-f-1…M-f-7.
