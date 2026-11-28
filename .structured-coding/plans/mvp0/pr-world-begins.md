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

Implementation base 2bcae8a9; origin/main 8745f572 merged in (after #557 bodies v8, #559, #560, #563) before any
re-record. Commits: `167811f9` design + ARC-371; `4e0e7b1d` worldpack key + hosts + content; `c9b3f9be` tests that
assumed genesis at midnight; `c44d61fa` merge main; `e12492b3` two 30-day goldens; then baselines + this ledger.

**Implementation.** `worldpack/src/{format,read,error}.rs` (`begins: Option<String>`, strict parse,
`PackError::Begins`, `WorldPack::begins()`); `tools/cli/src/run.rs` (`Begun::new` creates at `pack.begins()`);
`tools/cli/src/serve.rs` (`epoch_of`: the save's genesis instant when one exists, else `pack.begins()`, into
`HostConfig::epoch`, which feeds the ephemeral clock, a new save's genesis and the paced consult origin); reference
regenerated (`world.schema.json`, `world-pack.md`); `MODULE_SPEC.md` `begins:` entry; three `world.yaml` at 07:00.

**Hidden consumers found by the broad run (worldpack + cli + acceptance, local, 59 min) and their disposition.**

| Test | Assumption | Change (assertion kept) |
| --- | --- | --- |
| `ac5_takeover`, `ac7_many_players`, `milestone_c` | a hosted Market Town needs an always-open copy (#552) | host the **real** Market Town: at 07:00 the café sells; `MarketCopy::always_open` dropped there |
| `ac5_takeover`, `milestone_c`, `milestone_b` | a quiet hour from genesis / the head | genesis/head is 07:00, whose boundaries are already committed; window from +1 s for 1 799 s, re-derived from the routines (Erin 07:30 is the next) |
| `needs_town` AC-S35-8 / AC-Nd-2 | 03:00 / 14:00 / days = offsets from genesis | the calendar's hours inside the run (`midnight + d·DAY + h`, filtered to the run) — exactly `days` samples each |
| `routines` | the first agenda at instant 0 | at the genesis instant |
| `lakeside/history.rs` | the last fact never falls on the run's last instant | ceiling of the span; no bucket opens on the end instant |
| `shop_hours` N1-I7 | "once a night" bucket 12:00–12:00 | keyed by the closed interval (lunch and night are two) |
| `decisions` golden, `lakeside_bodies` doorway golden | 30-day summaries at midnight start | re-captured from this build, with reason lines |
| acceptance `client_rules`, `client_text` | — | **pre-existing on main** (`needs.gd` `capacity`; `loading.*` keys); untouched |

**Validation (local, macOS arm64).** worldpack `hosting` 6/6 (absent/07:00/malformed); re-run of every failing
target on the merged tree: ac5 2/2, ac7 1/1, milestone_b/c/e PASS, needs_town 2/2 (+6 ignored), routines, shop_hours
4/4, guide 46/46, decisions 7/7, lakeside_bodies 4/4 PASS. **AC-S35-8** (release, e12492b3): PASS every row — Market
Town 03:00 1000 ‰, 14:00 0 ‰, sleep 7.54 / 7.55 / 7.53 h, TV 276 / 269 / 283 ‰, longest famished 10.1 / 6.5 /
7.1 h; Commute Town unchanged. **Godot café-buy tests** (`--ignored`, Godot 4.7.2): `ac13_clients` 3/3,
`client_2d_interact` 5/5, `demo_a` 4/4 PASS; `client_3d_market` 1/3 — `a_poor_visitor_still_sees_every_buy` and
`the_3d_market_asks_what_the_2d_market_asks` FAIL. Base check (the same build with Market Town's `begins` removed,
i.e. main's midnight start): both FAIL too (café closed). At 07:00: O-5 expects only tea and croissant affordable
on a 250 wallet, but cake costs 110 — stale against the S35 prices, independent of the start; P-1's 2D run walks to
the counter before buying, the 3D one does not. **Follow-up for the client lane**, not changed here.

**Baselines** (`ci_parity record --profile long --root` one world each, e12492b3, `baseline write`, spliced):

| World | summary-300 | summary-1000 | facts-300s | journal-300s | sealed-300s |
| --- | --- | --- | --- | --- | --- |
| market-town | 401→400 | 1127→1127 | 6032→6247 | 4170→4240 | 263→263 |
| social-cafe | 360→362 | 1085→1086 | 10422→9408 | 7713→6464 | 189→186 |
| lakeside | 364→364 | 1088→1088 | 6515→6824 | 4214→4394 | 131→131 |

(Digests changed in every key; Bodies Yard not written.) Heavy time used ≈ 2 h 40 min wall in slots, of which
≈ 1 h queued behind other lanes.
