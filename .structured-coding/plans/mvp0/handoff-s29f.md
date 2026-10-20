# Handoff — S29-f, the station in 2D

A continuation aid for `pr-s29f-station-2d.md`; that design and its §15 ledger are the authority.

**Worktree** `/Users/yuema137/mineworld-worktrees/impl-s29-f`, branch `mvp0/pr-s29f`.

**Done.** C1 (docs), C2 (`MineWorldStationState` and its check), C3 (tag set keys, end doorways, shaped
rooms, stateful fittings and furnishings, ordered place names, `wall_m`), C4 (words), C5 (`fit.py`, the
recipe, the HABS reference), C6 (27 candidates, US$ 2.0967 spent; the contact sheet and sidecars committed),
C8's drive (`--drive=station`) and runner (`tools/cli/tests/client_2d_station.rs`, five `#[ignore]` tests).
`renderer/godot.yaml`'s station specs are written but held out of the commits until C7, because
`--check-pack` requires every role they name to be bound.

**Waiting.** Operator gate 2: a verdict on
`presentation/mineworld-default/2D/candidates/station/contact_sheet.png` (§15 lists the recommendation).
The candidate PNGs are in the worktree, uncommitted.

**Next, after the verdict (C7).** For each chosen candidate:

```sh
~/.cache/mineworld-gen/venv/bin/python -m mineworld_gen.fit \
  presentation/mineworld-default/2D/candidates/station/<item>_<n>.png \
  presentation/mineworld-default/2D/art/generated/sib_<item>.png --height-m <h> [--width-m 4.0]
```

run from `tools/asset_generation` with paths adjusted; heights per §5.8 (depot 30.0, halt 4.6, kiosk 2.6,
ticket window 2.4, board 1.4, bench 0.9, track `--width-m 4.0`, pavilion 5.0, arcade pier 4.0). Paste the
printed blocks into `assets/asset_bindings.yaml` `sprites`; bind in set `town`: `facade:station+landmark`,
`facade:station`, `furnishing.kiosk:open`, `furnishing.kiosk:shut`, `furnishing.bench.station`,
`prop:ticket-window:open`, `prop:ticket-window:shut`, `prop:departures-board`, `prop:track`,
`prop:arcade-pier`, `prop:pavilion`; keep the chosen candidates and sidecars, delete the rest; add the
`art/PROVENANCE.md` row with the verdict. Commit with the held `godot.yaml` specs. Then capture the shots
(`--drive=station --capture`, `--drive=farfield --capture`, one world at a time, through `with-slot.sh`) and
run R-1 locally.
