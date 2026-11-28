# World start time — `world.yaml` `begins:` (follow-up to N1, D-N1-4)

```text
Ruling:        operator, 2026-11-27, relayed by the primary coordinator: "新世界早上 7 点开始" — new worlds begin
               at 07:00, not 00:00 (decides pr-nightlife.md §11 D-N1-4).
Scope approval: coordinator, 2026-11-27: implement as a follow-up PR from origin/main after #552 merged;
               design note + ARC first; heavy budget ≈ 2.5 h via cast-slots/with-slot.sh.
Base:          origin/main 2bcae8a9 (#552 merged)
Branch:        mvp0/pr-world-begins @ /Users/yuema137/mineworld-worktrees/impl-world-begins (one session)
Decision:      docs/decisions/ARC-371.md
Lifecycle:     design (this commit) → implementation → validation → READY FOR OPERATOR REVIEW (no merge)
```

## 1. Problem

Every host that creates a world begins it at `WorldTime::EPOCH`, instant 0, which is local midnight of the
calendar's epoch date (`INV-TW-4`). With N1's shop hours (`ARC-364`) a hosted Market Town therefore starts with
every shop closed for 5.5 world hours. The operator ruled that new worlds begin at 07:00.

## 2. Audit (2bcae8a9)

| Seam | Finding |
| --- | --- |
| `tools/cli/src/run.rs` `Begun::new` | `at = WorldTime::EPOCH` for an in-memory world and for a new save; a resumed save reads its genesis instant (`genesis_instant`); days count from genesis (`end = genesis + days·DAY`) |
| `server/src/host/config.rs` `HostConfig::epoch` | default `WorldTime::EPOCH`; `runtime.rs` uses it for an ephemeral world's clock; a persisted world's clock starts at `world.now()` |
| `tools/cli/src/serve.rs` | `epoch = config.epoch`: `pack.load(epoch)` (ephemeral), `persisted(.., epoch)` (new save), and the paced seats' consult lattice origin (`PacedSeat::new(pacing, epoch, ..)`) — on resume too |
| `worldpack` `world.yaml` | the only time key is `hosting: time_scale` — host pacing, "never seeded, ignored by `run`" (`ARC-69` item 9) |
| inspection paths (`validate`, `packs`, `biography`, `perceived`, `create`) | load at `EPOCH` only to read ids or count genesis facts; no world is begun there that is kept |

## 3. Design

1. **`begins: "HH:MM"`**, a new optional top-level `world.yaml` key, owned by `worldpack` (`format.rs`
   `WorldManifest::begins`, checked in `read.rs`; accessor `WorldPack::begins() -> WorldTime`): the time of day
   of genesis on day 0. Absent ⇒ `"00:00"`, today's behaviour — no world that omits it moves. Strict `HH:MM`
   (00:00 … 23:59), refused at load naming the file and key otherwise. Not under `hosting:`, because it is world
   history (it fixes the genesis instant, `run` included), not host pacing. `worldpack` parses it itself; it does
   not depend on the `schedule` System Pack.
2. **Creating hosts use it**: `run` (in memory and a new save), `server` (ephemeral world and new save),
   through `HostConfig::epoch`, set from the pack when the world is created.
3. **Resume keeps the saved start**: a resumed save's genesis instant is read from the save (already so in `run`);
   `server` derives its paced consult origin from the save's genesis instant on resume, so an existing save
   (genesis 0) is driven exactly as before whatever the pack now says. `begins` is not a configuration fact, so
   it is not drift (`ARC-61` item 7); it only applies to worlds created after it is authored.
4. **Invariants kept**: midnights stay multiples of 86 400 s (`INV-TW-4`); genesis simply need not be one. No
   kernel, System Pack or client change. Inspection paths keep `EPOCH` (they begin nothing that persists).
5. **Content**: `worlds/{market-town,social-cafe,lakeside}/world.yaml` `begins: "07:00"`; Bodies Yard unchanged.

## 4. Validation plan

- worldpack: absent / `"07:00"` / malformed (`"7:00"`, `"24:00"`) — load and refusal naming file + key.
- run: a world with `begins: "07:00"` has its genesis facts at 25 200; a resumed pre-`begins` save keeps genesis 0.
- server: a new save of a `begins` world starts its clock at 07:00; a hosted player in Market Town can buy at the
  café right after start (the D-N1-4 behaviour), asserted in the reworked `ac5_takeover` / `milestone_c` against the
  real Market Town (the `MarketCopy::always_open` copies are removed where the 07:00 start makes them unneeded;
  quiet-window checks move to the new start and are re-derived from the routines, never weakened).
- AC-S35-8 re-measured (release, 30 days × seeds 1/7/42); the three worlds' baselines re-recorded after merging
  the latest main; Godot café-buy tests (`client_3d_market`, `client_2d_interact`, `ac13_clients`, `demo_a`) run in a
  heavy slot.

## 5. Ledger

(filled during implementation)
