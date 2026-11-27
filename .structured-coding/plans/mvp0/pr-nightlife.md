# Nightlife — shop hours (PR N1) and single-player sleep-until advance (PR N2)

## DESIGN FROZEN — 2026-11-27 (operator approved every recommendation, Q-L1 … Q-L5, Q-L4 option A)

```text
Design revision:        revision 2 (2026-11-27): revision 1 + bounded corrections from the red-team pre-freeze
                        review (_reviews/design-nightlife.md, F1–F14; disposition in §10). No approved decision changed.
Approved by / evidence: the operator, 2026-11-27, relayed by the primary coordinator: "同意三个都按推荐冻结"
                        ("agree: freeze all three as recommended" — covering #535, #536 and this design, #537)
Implementation base:    origin/main 8c971f0f7b469f05d1bf6f21fc933501eb22c404 (seams re-checked: only
                        clients/3d-spike/scripts/slice/slice_probe*.gd changed since the 7718d952 audit — not a seam)
Start condition (N1):   met now — N1-C1 and N1-C2 start on main; N1-C3's packaged launcher night check (Gate2)
                        waits for prerequisite P1 (§1.3) and is otherwise recorded INCONCLUSIVE — prerequisite
Start condition (N2):   N1 merged; N2-C1 … N2-C3 then start on main; N2-C4 additionally waits for P1, P2 and P3 (§1.3)
Execution contract:     .structured-coding/plans/mvp0/pr-nightlife-contract.md (contracts N1 and N2)
Parent:                 operator-playable-mvp-20261123.md §§2–5 (requirements); overall.md
Supersedes as authority: the recovered Codex drafts listed in §1.5 (evidence only, not ledgers)
Lifecycle:              FROZEN (2026-11-27); implementation not yet started
```

## 0. Operator rulings (2026-11-27)

Each question below was answered **as recommended** by the operator on 2026-11-27; the "Recommendation" column is
now the ruling. Ruling IDs Q-L1 … Q-L5 are binding for N1 and N2.

| ID | Question | Recommendation |
| --- | --- | --- |
| **Q-L1** | Should shops have opening hours? Proposal: hours are optional data on each shop, owned by the `economy` System (never by a client); a world that writes no hours behaves exactly as today. Market Town gets café 05:30–22:00, corner store 06:00–23:00, station newsstand 06:00–18:00. Sales are unattended (no staff needs to be present). A closed shop refuses every way of buying, including a haggle agreed before closing. | **Yes.** |
| **Q-L2** | A new `sleep-until` action, owned by the `sleep` System: "sleep until 07:00 tomorrow", with the morning configurable per world (default 07:00). The server offers the exact target; clients never calculate it. It is a normal world rule available in every world — in a shared world it is simply an exact alarm. Only the *time advance* (Q-L3/Q-L4) is single-player. The existing `sleep { for }` is unchanged. | **Yes.** |
| **Q-L3** | In a single-player (`--solo`) world, after `sleep-until` succeeds the player may ask the host to run the night faster than real time: every resident, process and event in between is really simulated in order, and it stops at morning or at any earlier wake. If the game is killed during that, the save keeps exactly what was simulated, the operation is recorded as *interrupted*, it never resumes on its own, and we do not promise that residents' future decisions are bit-identical to an uninterrupted run. Shared worlds never jump time. | **Yes.** |
| **Q-L4** | What if another mind is attached — for example Alice driven by the local language model (dialogue lane, Q-D2), or a second human? Option A: the advance is refused with a named reason ("another controller is connected") and the player sleeps normally. Option B: for the duration of the advance those Persons are temporarily driven by their default rule controller, then handed back. | **A in N2** (smallest, deterministic, no change to seat ownership). B becomes its own follow-up PR (N3) after the dialogue lane freezes, if you want sleep advance in the LM profile. Note: with A, sleep advance is unavailable while LM-Alice is attached. |
| **Q-L5** | Delivery order and homes. Proposal: N1 (shop hours) ships in the first playable delivery; N2 (sleep-until + advance) right after. Homes stay the authored shared apartments; private residences and exclusive beds are deferred. | **Yes.** |

Rulings these answers amend, recorded explicitly so nothing is silently overridden:

- **step-35 Q-S35-9** ("no time skip in a shared world; recorded for single-player MVP-1"): the shared-world exclusion
  **stays**. The single-player half moves from MVP-1 into MVP-0 because the operator required it on 2026-11-23
  (`operator-playable-mvp-20261123.md` §5). N2-C1 adds a dated note to step-35 §6 and the Q-S35-9 row.
- **ARC-69 item 5** (fast-forward is single-player only; "`--solo` selects no code path … it only widens one
  permission"): **amended by Q-L3's ruling.** The operator approved a single-player-only advance, which is a new
  execution path (the exact-due loop) reachable only in a `--solo` world. N2-C2 writes an amending ARC record that
  states the new rule: "`--solo` widens two permissions — scales above 24, and the advance operation; the advance
  loop is reachable only through the second; world rules, authority and persistence remain one path." A dated note
  on ARC-69 links it. No rescale is used as an advance.
- **`clients/shared/HOST.md` §3.2** ("a solo world fast-forwarding in S21"): N2-C4 rewrites that reference to the
  N2 operation. **S21 QRT-7 "doze"** (mvp1, in planning) proposes a raised time scale during a ride; N2's operation
  is generic over its stop condition, so S21 may adopt it instead of a rescale when S21 is designed. No S21 edit now.

## 1. Requirement, scope and non-goals

### 1.1 Requirement (operator, 2026-11-23)

`operator-playable-mvp-20261123.md`: a night-time town with home/sleep/shop rhythm (§1); shop schedules and home
assignments are content or owning-system configuration, never kernel or client special cases (§1); acceptance item 7,
"observe a night-to-morning routine … business availability where supported", with night workers a content choice;
§5, single-player sleep advance that interleaves controller decisions, movement and scheduled events, with
deterministic persisted results for cancellation, earlier wake, failure and restart, and **no direct timestamp jump**.

### 1.2 In scope

- **N1** — optional economy-owned daily sales hours; one availability judgement for every Shop sale path; closed
  shops disclosed so controllers do not walk to them; Market Town authoring; bounded night/morning scenario.
- **N2** — sleep-owned `sleep-until { at }` with configured morning; a generic, authenticated, solo-only host
  *advance* operation that executes every due instant in order until a composition-registered stop condition holds;
  durable lifecycle with interrupted-on-restart; 2D and 3D client flow through the shared host module.

### 1.3 Prerequisites owned elsewhere (not N1/N2 scope)

- **P1 — launcher Market Town.** The launcher starts Market Town with its own save and matching depiction, and at
  least one non-Alice resident walks home, sleeps, wakes and commutes. On 8c971f0f this is still in the active
  client-stack recovery lane. Blocks N1-C3's packaged check and N2-C4.
- **P2 — launcher per-run admin token.** On main `tools/launch/src/children.rs` starts the server with
  `env_remove("MINEWORLD_ADMIN_TOKEN")` ("no admin surface is mounted"). The operator reconciliation (§2 "Time
  controls", §3; ARC-78 getrandom exception) assigns the private per-run token to the launcher lane. Blocks N2-C4.
- **P3 — launcher starts single-player worlds `--solo`.** Not done on main. Making the packaged single-player
  world `--solo` also unlocks scales above 24 (ARC-69 item 5); that product consequence is the launcher lane's to
  surface in its own design. Blocks N2-C4 only. N2 does not edit `tools/launch`.

Without P2/P3 the packaged clients show ordinary sleep with a named "advance unavailable" message (N2-C4 item 6);
nothing else in N1/N2 depends on them.

### 1.4 Non-goals

Private homes, residence assignment, exclusive or occupied beds, door locks, lights, eviction, weekdays/holidays,
staffed hours (a keeper's presence), external/LM seats during an advance (Q-L4 B), controller-trace parity across a
crash, any kernel change, any time-scale change used as an advance, any change to Social Café or Bodies Yard (they are
S35 Q-S35-7 removal witnesses; their saves are not migrated or relabelled).

### 1.5 Source drafts imported (read-only evidence)
`/Users/yuema137/mineworld-worktrees/_recovered-docs-20261127/recovered/tmp/`:
`mineworld-night-plan-20261123/.structured-coding/plans/mvp0/pr-nightlife-next-ready.md` (the plan this design
replaces), `mineworld-nightlife-design-20261123.md` (bounded audit), `mineworld-living-world-audit-20261123.md`.
Corrections against them are listed in §8.

## 2. Audit (7718d952; seam drift re-checked on 8c971f0f)

### 2.1 Base

Freeze base `8c971f0f`: `git diff --stat 7718d952 8c971f0f` over the seam paths below (plus `tools/launch`,
`cognition`) touches only `clients/3d-spike/scripts/slice/slice_probe*.gd`, not a seam. The drafts audited
`b457a94d`. `git diff --stat b457a94d 7718d952` over `systems/{sleep,economy,schedule}`, `server`,
`persistence`, `kernel`, `worlds/market-town`, `clients/shared`, `clients/2d/scripts`, `clients/3d-spike/scripts` and
`tools/cli/src` is **empty**: every source claim below was re-read on 7718d952.

### 2.2 Ownership and seams

| Seam (actual path) | Finding | Planned responsibility |
| --- | --- | --- |
| `systems/economy/src/section.rs::AuthoredShop` | `{ at, prices, keepers }`, `deny_unknown_fields`, schemars schema | N1 adds optional `hours`; refusals by name |
| `systems/economy/src/component.rs::Shop` | `{ operator, keepers, adjustments }`; `adjustments` uses `serde(default, skip_serializing_if)` — the precedent for an additive optional field | N1 adds `hours`, same precedent or an explicit schema bump (§3 N1-I4) |
| `systems/economy/src/event.rs::ShopOpened` | genesis fact, `SCHEMA_VERSION 2` | carries authored hours |
| `systems/economy/src/sale.rs` | `resolve_seller` (1–3), `judge_self_trade` (5), `judge_goods` (6–9), `judge_exchange` = 5+6–9, `judge_sale`, `settle_buy`, `buys` | closure judged inside `judge_exchange` |
| `systems/economy/src/haggle.rs::judge_proposal` (l. 133) | calls `judge_self_trade` and `judge_goods` **separately — not `judge_exchange`** | must call the closure judgement too (§8 C2) |
| `systems/economy/src/haggle.rs` l. 256, l. 548 | accepted-proposal settlement and the `accept-proposal` offer use `judge_exchange` | covered by the `judge_exchange` change |
| `systems/economy/src/component.rs::ShopStock` | `shop-stock`: every shop's place and kinds "asked and held ≥ 1" — "what a shop window or a sold-out sign tells a passer-by"; computed, never stored, schema 1 | a closed shop discloses `stocked: []` (§3 N1-I6) |
| `cognition/rule-controller/src/fetch.rs` | hungry band: step 7 walks to a shop that stocks food **or that `shop-stock` does not list**; steps 6/8 stop when no listed shop stocks food | unchanged code if closed shops are listed with nothing stocked; proven by scenario |
| `cognition/motivated/src/{considerations,options}.rs` | hunger raises `eat`/`drink`/`buy` offers (available only); place roles `Eat`/`Shop` by tag drive walks; reads no `shop-stock` | audit in N1-C3; correct only if the scenario shows walks to closed shops (§3 N1-I7) |
| `systems/schedule/src/time.rs::TimeOfDay` | strict `"HH:MM"`, seconds since midnight, `of(at) = at mod 86 400` | reused by economy as a **type-only** dependency. `ARC-28`'s precedent is for *event* types (economy → employment, happenings); reusing a *value* type extends it, stated in N1's ARC record. Fallback if review rejects it: restate `TimeOfDay` in economy as sleep restates `DAY` |
| `cognition/rule-controller/src/{paced,offered}.rs` | in ~20 % of consults the paced rule attempts a random **available complete** affordance not in `DECIDED_BY_NAME` (talk, walks, invitations, activities, put-down, pick-up) | N2-C1 adds `sleep-until` to `DECIDED_BY_NAME` (F1) |
| `cognition/motivated/src/options.rs` (`Offered(usize)`, l. 378–386) | scores any available complete affordance except penalties and the hands | N2-C1 excludes `sleep-until` the same way (F1) |
| `server/src/host.rs` l. 312–324, `runtime.rs` l. 214, 329–348 | a ticker sends `Command::Sweep` every observation interval → `tick_at` → `consult(now)` + `advance(now)`; a session `submit` dispatches at `clock.now()` | N2-I12 fixes each command's meaning during an advance (F2) |
| `server/src/runtime/delivery.rs` l. 10–15 | a subscriber's learned-fact queue is bounded (oldest dropped and counted); a perceived stream past its bound releases the connection `lagged`; nothing waits for a client | N2-I5 narrowed; N2-I13 delivery during an advance (F4) |
| `sdk/rust/src/interactions/mod.rs` l. 157, 215; `docs/decisions/ARC-61.md` item 7 + note 2026-11-27 | a section's `*-interactions-configured` fact is a configuration fact; configuration added, removed or changed is **drift**, refused at resume | N1-I8 / N2-I14 resume policy (F6) |
| `systems/installed/src/lib.rs` `extension` lines (`ARC-62`) | extension catalogs: a trait, its register function, the packs implementing it; the CLI names no domain pack (`mineworld-sleep` is a CLI dev-dependency only) | the advance stop adapters are published the same way (N2-I4, F10) |
| `systems/calendar/src/day.rs::DAY` | calendar restates `DAY` rather than depend on schedule (`INV-TW-4`); instant 0 is local midnight | sleep restates the one constant the same way (no new dependency) |
| `systems/sleep/src/action.rs` | `Sleep { for: u16 minutes }`, `Wake {}`; alarm = now + 60·for | add `SleepUntil { at: WorldTime }` |
| `systems/sleep/src/interactions.rs::SleepParameters` | `parameters!` section: `woken_by_talk`, `longest_minutes 60…840 (840)`, `shortest_minutes 15…120 (15)`; regional per actor/place; version pinned by test | add morning (§3 N2-I2) |
| `systems/sleep/src/offer.rs::offered` | awake: parametric `sleep` (refused as validate would); asleep: complete `wake`, `sleep` refused `Busy` | add a **complete** `sleep-until` offer carrying the exact target |
| `systems/sleep/src/system.rs::validate/resolve` | ordered admission (decode, living Person/no target, not asleep, located, bounds → `sleep-for-out-of-bound`, place → `PermissionDenied`) | `sleep-until` shares the same admission and process |
| `worlds/market-town/configure/sleep.yaml` | states the pack defaults; apartments are `residential` | states the morning |
| `kernel/src/world.rs::next_instant`, `kernel/src/advance.rs::advance_to/step` | ordered queue of due Processes and deferred facts | reused unchanged; the kernel learns nothing |
| `server/src/runtime.rs` (509 lines) `tick_at`, `consult`, `advance`, `submit_at` | `consult(now)` takes one `table.due(now)` snapshot, advances to each due instant, then `slot.decide(&observation, now)` — **horizon `now`**, which skips missed lattice instants by design | the advance loop calls the one-instant path with `now = t` and recomputes due seats after every instant; lives in a new `server/src/runtime/advance.rs` (runtime.rs is already central) |
| `server/src/hosted.rs::HostedSlot` (`next`, `decide`, `answered`, `rescale`) | controller state is not persisted; factories rebuild on bind | reused; no controller checkpoint |
| `server/src/seats.rs::due`, `join` | due = (instant, seat) sorted | `join` refuses decision joins while an advance runs |
| `server/src/host.rs` (`solo`, l. 256–335) | `--solo` fixed at start, loopback only (`ARC-69` item 5) | the advance's first gate |
| `server/src/admin.rs` routes | bearer-checked `/admin/{sessions,seats,clock,stop}`; module contract: "**No route and no field names an instant**" (`ARC-40`) | new `/admin/advance` names **no instant**: the horizon is the sleeper's own alarm, read by the stop adapter |
| `persistence/src/host.rs` | `HostRecord { at, revision, change: PacingChange }`, `PacingChange::{Started, Paused, Resumed, Scaled, Stopped}`, lazily created `host_journal` (`ARC-69` items 6–8) | add the advance lifecycle as host records, never facts |
| `server/src/runtime/world.rs` l. 34–98 (`HostClock`: `now`, `now_at`, `pause_at`, `resume_at`, `rescale_at`); `runtime/pacing.rs` (`resolve`, `DAY_LENGTH_SCALE`) | the clock is in `world.rs`, not `pacing.rs` | add an in-memory hold and a forward re-anchor at the reached instant, scale unchanged |
| `clients/shared/host/{host_clock,world_tab}.gd`, `clients/shared/HOST.md` | the one shared authenticated admin transport; displayed state only from `welcome`/`clock` frames (`INV-WC-1`) | advance request/cancel/status added here, once, for both clients |
| `clients/2d/scripts/{app.gd,hud/readers.gd,hud/panels.gd}`, `clients/3d-spike/scripts/slice/needs_hud.gd` | 2D asleep panel; 3D `SliceNeedsHud` dims the view while asleep. The 3D client directory is `3d-spike` | show advance progress/Wake; no rule computed |
| `tools/cli/src/hosted.rs` (560 lines, above the 500 review trigger) | composition root that builds hosted controllers (`PacedSeat`: walking consult period = stride at scale) | **not** edited for adapters: it names no domain pack (F10) |
| `persistence/src/host.rs` l. 29–60 | `HostRecord`/`PacingChange` are `Copy`; `read_journal` refuses an undecodable record as damaged; the `host_journal` table is created lazily (ARC-69 item 6) | advance lifecycle goes to a separate lazily created table (N2-I8, F12) |

### 2.3 Residents' night on 8c971f0f

Market Town installs `schedule`, `sleep`, `needs`, `economy`, `employment`, `consumption`, `calendar`. Shops: café at
`cafe`, corner store at `store`, newsstand at `station`. All twelve people's routines (`worlds/market-town/people/*.yaml`)
visit shops only inside the ruled hours (café 05:30–22:00, store 06:00–23:00, newsstand 06:00–18:00):

| Shop | Routine visits (from) | Jobs |
| --- | --- | --- |
| café | Alice 05:30 (work), Bob 07:00, Grace 07:00, visitor 08:00, Carol 12:00, Ivan 12:00, Erin 13:00, Felix 13:00, wanderer 13:00, Hana 15:00 | Alice 05:30–14:00 |
| store | Carol 07:00, Felix 08:00 (work), Otto 09:00, Bob 16:00, visitor 16:00 | Felix 08:00–13:00 |
| station | Ivan 07:00 (work), Erin 07:30, Bob 11:00–16:00 | Ivan 07:00–12:00 (station-news) |

Dev, Grace (work at `workplace`), Hana and the parks/street visits involve no shop. Evening home times run 13:00–21:00. Hunger rises overnight; S35 has no forced consequence (Q-S35-3), so the risk is
behavioural (walking to a closed shop), not a starvation rule.

## 3. Frozen invariants

### N1 — shop hours

- **N1-I1 Ownership.** Sales hours are economy's data on `Shop`; only economy reads them to judge a sale. The kernel,
  clients, controllers and other packs never compute business hours. No stored open/closed flag, no minute
  scheduler, no Process: availability is a pure function of `Shop.hours` and the authoritative `world.at()`.
- **N1-I2 Shape.** `hours` optional. Absent ⇒ always open (today's behaviour). `hours: []` ⇒ closed all day.
  Otherwise 1…8 intervals `{ from: "HH:MM", until: "HH:MM" }` (schedule's `TimeOfDay`), half-open `[from, until)`;
  `until < from` wraps midnight; `from == until` is refused; intervals may not overlap after normalising across
  midnight. Time of day = `at mod 86 400` (instant 0 is local midnight, `INV-TW-4`).
- **N1-I3 One judgement.** A closed Shop seller refuses with `shop-closed` on every path: offers, `buy` validate and
  dispatch, `settle_buy`, `propose-price`, the `accept-proposal` offer and accepted-proposal settlement. Settlement
  re-judges at the authoritative instant. On refusal: no `sold`, no money or inventory change. Person-to-person sales
  are unaffected. Full precedence for a Shop seller: `resolve_seller` (not a living buyer → `NoSupportedInteraction`;
  no shop here; `not-for-sale`) → for `buy`, `price-changed` (`judge_sale`) / for `propose-price`, price range →
  **`shop-closed`** → self-trade → proposal limits (propose only) → funds, stock, carry, owners' rules. So a closed
  shop never reports `insufficient-funds` or `out-of-stock`; an unknown item still reports `not-for-sale`.
- **N1-I4 Persistence.** Hours enter through `AuthoredShop` → `ShopOpened` → `Shop`. Old saves and events decode with
  hours absent (always open) either by the additive-optional precedent of `adjustments` or by an explicit version
  bump with a named refusal — chosen in N1-C1 and recorded; never a serde accident.
- **N1-I5 Keepers.** Keeper rights stay listing/price authority; a keeper may re-price a closed shop. No staffing.
- **N1-I6 Disclosure.** While a shop is closed, `shop-stock` lists it with `stocked: []` ("nothing for sale now" — the
  shop window's sign). The component's documented meaning changes from "asked and held" to "asked, held and on sale
  now"; schema unchanged. Inside the shop, offers carry the `shop-closed` refusal for the player's UI.
- **N1-I7 Living town.** In the bounded night scenario no hosted resident submits more than one refused `buy` with
  `shop-closed` per shop per closed interval, and none walks to a closed shop more than once per night. If the
  motivated controller violates this, N1-C3 corrects it in its own module by reading `shop-stock`, not by a client or
  kernel rule.
- **N1-I8 Existing saves.** Policy is the accepted `ARC-61` item 7 rule, unchanged: if hours reach a configuration
  fact compared at resume, a Market Town save created before N1 is **refused at resume by name** with the existing
  drift message (never silently repurposed, never resumed without hours while fresh saves have them); if they do not,
  the save keeps exactly its genesis terms (always open). N1-C1 determines which holds from the code and records it;
  either way a pre-N1 Market Town save fixture is resumed with the N1 binary and its outcome asserted.

### N2 — sleep-until and solo advance

- **N2-I1 Sleep owns sleep.** Only `SleepSystem` creates `Asleep`, the `sleep` Process, `alarm-set` and `woke`.
  `sleep-until { at }` shares `sleep`'s admission, place rule, Interaction-List section, process, alarm privacy
  (holder-only `alarm-set`) and four ways to end. It is a world rule in every world; `--solo` changes nothing in it.
  **Rules are `sleep`'s rules (F8):** `sleep-until` adds no `ActionDecl`; its place ruling and parameters are
  resolved with `Sleep::ACTION_TYPE` exactly as `place.rs::allowed` and `interactions.rs::terms` do today, so every
  existing `{ action: sleep, … }` rule governs it and no world duplicates rules. Test: a `forbid sleep` rule
  (place and actor forms) refuses `sleep-until`; a `permit sleep` rule admits it.
- **N2-I1b Controllers never choose it by chance (F1).** Hosted rule and motivated controllers decide sleep through
  `sleep` by name; `sleep-until` is added to the rule controller's `DECIDED_BY_NAME` and excluded from motivated's
  `Offered` option. Rhythm regression: a Market Town headless day with the new offer has the same asleep share at
  14:00 and 17:00, the same sleep intervals and the same decision trace as the base for the same seed.
- **N2-I2 Morning.** A new sleep parameter, the morning as minute of day (0…1439, default 420 = 07:00), regional like
  the others. The offered target is the first instant at that time of day strictly after `now + shortest`; it is
  offered complete (exact `at`) only when `at − now ≤ longest` and the place allows sleep, else refused as validate
  would refuse it. Validate admits any `at` with `shortest·60 ≤ at − now ≤ longest·60` seconds (so 23:00:12 → 07:00 is
  exact), refusing out-of-range with `sleep-for-out-of-bound` and a past or present `at` likewise.
- **N2-I3 Advance is host state.** The advance is a host operation, never a fact, journal input or revision. It is
  permitted only when: the world was started `--solo`; the request is admin-authenticated; the named seat is bound to
  exactly one decision session; **no other seat is bound to a decision session or inside a hold window** (Q-L4 A —
  other seats may be hosted or free; observer sessions that decide nothing are allowed); the seat's Person is asleep
  through a sleep identified at start; the clock is running; no other advance runs; the world has the named adapter.
  Otherwise a typed refusal before any work. Shared and both-client hosts always refuse.
- **N2-I4 Generic server; adapters as an extension catalog (F10).** The server knows an advance *stop adapter*
  trait, not sleep. The trait lives in a domain-free crate both the server and packs may depend on (`mineworld-sdk`
  or `mineworld-contracts`, chosen by audit in N2-C2 and recorded); the sleep pack implements it; it is published
  through one `ARC-62` `extension` line in `systems/installed`, and reaches the server with the composed world, so
  `tools/cli` names no domain pack. The adapter answers, from authoritative state: the horizon (the sleeper's alarm)
  and whether the identified sleep still holds. A world without `sleep` refuses with `unknown-adapter`. The admin
  route names no instant (`ARC-40`).
- **N2-I5 Exact-due order.** Repeatedly `t = min(kernel.next_instant, earliest hosted consult, horizon)`; advance the
  kernel to `t` (existing queue/reducer order); consult every seat due at `t` in the existing (instant, seat) order
  with `now = t`, through the ordinary perception/submit/answer path; recompute due seats after the instant; after the
  instant is complete, ask the adapter. Stop at the **first completed instant** at which the identified sleep no
  longer holds; never go past an earlier wake. Scale `S`, decision periods, stride periods and speech windows are
  unchanged; no rescale. Durable history, hosted controllers' observations and answers are never dropped. Client
  delivery follows N2-I13.
- **N2-I6 Oracle parity.** For the same save, seed, scale and bindings, the advanced night equals the ordinary host
  driven by an injected wall clock that ticks at every due instant: same intents, events and order, positions, sleep
  intervals, needs, wages, stock, purchases and final instant.
- **N2-I7 Responsiveness.** Work in slices of ≤ 128 completed instants or ≤ 10 ms CPU, whichever first; between slices
  service cancel, disconnect, shutdown and status. Implementation constants, not gameplay time.
- **N2-I8 Durability.** World actions persist through the existing persisted dispatch/advance before progress is
  published. The lifecycle is host state in a **separate, lazily created table** (`host_operations`), not new
  `PacingChange` variants: `PacingChange` stays `Copy` and unchanged, and a pre-N2 binary ignores the unknown table
  exactly as ARC-69 item 6 lets a pre-TW-c binary ignore `host_journal` (no `SAVE_FORMAT` bump). Records:
  `advance-started { operation: OperationId, seat, adapter, prior scale, at, revision }` before the first instant,
  `advance-ended { operation, outcome, at, revision }` after the last. `OperationId` is a typed 128-bit value
  generated by the client (hex on the wire, refused if malformed). `verify` checks the table's coherence like the
  host journal's and never treats it as input. On startup, a started-without-ended operation gets
  `advance-ended { interrupted }` appended, the world resumes at its durable instant with ordinary pacing
  (`ARC-69` item 8), and nothing resumes automatically. A persistence failure fail-stops; a partially reduced in-memory
  advance is never retried and never reported as morning.
- **N2-I9 Idempotence.** The client generates the operation id. A repeated id with the same payload returns the
  recorded result without advancing; a repeated id with a different payload is refused.
- **N2-I10 End states (F3).** Outcomes: `woke` (alarm or any other wake, including a `wake` the sleeper's own session
  submits — the adapter's reason), `cancelled` (the cancel route: finish the current instant and stop; the **server
  submits nothing** — the sleeper stays asleep and the client sends `wake` through its own session if the player
  chose "get up"), `disconnected`/`stopped` (stop at a boundary; the night is never finished in the background),
  `interrupted` (restart), `failed` (fail-stop). After any outcome the HostClock is re-anchored at the reached
  instant with `S` unchanged and no wall time folded in.
- **N2-I12 The world thread during an advance (F2).** The advance runs as a sequence of slices; after each slice the
  world thread re-posts a continuation command to itself (via the existing bounded command channel; if
  `COMMAND_BACKLOG` is full it retries at the next Sweep) so queued commands are served between slices. Meanwhile:

  | Command / route | During an advance |
  | --- | --- |
  | `Sweep` (ticker) | sweep only: release ended holds and deliver observations; **no** `consult`, **no** `advance` |
  | HostClock | held in memory (not a journaled pause); its reading is the last completed instant; re-anchored at the reached instant on the outcome |
  | `GET /admin/clock`, `clock` frames | report that reading and the unchanged scale; one `clock` frame per slice so clients see time move |
  | `Submit` from the sleeper's session | dispatched at the current boundary instant (`world.now()`), through the ordinary path; a `wake` or any act that wakes ends the advance at the adapter's next check (`woke`) |
  | `Submit` from any other session | impossible by N2-I3 (none is seated); if one arrives it is refused `advance-in-progress` |
  | `Join` / takeover of any seat | refused `advance-in-progress` |
  | `Leave` / disconnect of the sleeper | stop at the boundary: `disconnected` |
  | `Control` (`POST /admin/clock`) | refused `advance-in-progress` |
  | `/admin/sessions/{s}/kick`, `/admin/seats/{s}/release` | of the sleeper: stop, `disconnected`; of another seat: refused `advance-in-progress` |
  | `Status`, `GET /admin/advance/{op}` | answered |
  | `/admin/stop`, `Shutdown` | stop at the boundary: `stopped`, then the ordinary graceful stop |

  Tests: a Sweep and a sleeper `wake` each arriving between slices.
- **N2-I13 Client delivery during an advance (F4).** The existing bounded delivery is unchanged: a subscriber's
  learned facts may be dropped and counted as today. To avoid releasing the sleeper `lagged` merely because the
  night runs fast, a slice ends early when the sleeper's perceived stream reaches half its bound, and the next slice
  starts only after the following sweep has drained it. A `lagged` release that still happens ends the advance as
  `disconnected` with reason `lagged`, named. N2-C3 records the queue high-water mark on the reference night.
- **N2-I14 Existing saves (F6).** The morning parameter changes the `sleep-interactions-configured` configuration
  fact, so under the accepted `ARC-61` item 7 rule a Market Town save created before N2 is **refused at resume by name
  as drift** — never silently repurposed. A pre-N2 Market Town save fixture resumed with the N2 binary asserts that
  named refusal; a fresh save gets the morning.
- **N2-I11 No controller checkpoint.** Recovery rebinds controllers through their factories (`ARC-42`, L-TWc-1); only
  durable world history and the interrupted classification are guaranteed.

## 4. Reuse decision

The ordered kernel queue (`next_instant`, `advance_to`), `HostedSlot` cadence, the host journal and SQLite
persistence, the `parameters!` Interaction-List machinery, schedule's `TimeOfDay`, and the shared client admin
transport cover every mechanism. No event engine, scheduler, calendar library, persistence backend or framework is
justified; none is adopted and none is rejected in favour of new infrastructure, so no `DEP` record is required. If
implementation finds a substantial need, it stops for a decision record first (`REUSE_POLICY.md`).

## 5. PR N1 — optional shop hours

Branch `feat/nightlife-n1-shop-hours`, one worktree under `/Users/yuema137/mineworld-worktrees/`.
Integration checkpoint: in Market Town an open shop sells, a closed shop refuses every path with no state change,
residents eat across a night/morning, and save/restart reproduces the hours.

### N1-C1 — typed, persistent shop hours

1. **Goal.** Make hours a typed economy contract before any consumer reads it, so C2 judges against a settled shape.
2. **Scope.** `systems/economy/src/{section,component,event,system,code,lib}.rs`, `systems/economy/Cargo.toml`
   (type-only `mineworld-schedule`), economy pack documentation, regenerated
   `docs/reference/schema/section-economy.schema.json`. Non-goals: sale semantics (C2), content (C3), staffing.
3. **Implementation.**
   - [ ] `Hours` type: 0…8 `[from, until)` intervals of `TimeOfDay`, normalised across midnight; `open_at(WorldTime)`.
   - [ ] `AuthoredShop.hours: Option<Vec<…>>`; refusals naming the source file and interval index.
   - [ ] Thread through `ShopOpened` → `Shop` reducer; decide additive-optional vs version bump (N1-I4) and record.
   - [ ] Determine whether hours reach a resume-compared configuration fact and record the N1-I8 branch.
   - [ ] Regenerate the schema; document defaults and refusals in the pack's module docs.
4. **Validation.** [ ] authoring+seeding integration for absent, `[]`, ordinary, overnight (22:00–06:00), equal
   endpoints, overlap after midnight, ninth interval, malformed `"7:00"`; [ ] save → reopen → equal `Shop` terms;
   [ ] a pre-N1 save fixture (existing economy persistence fixtures) decodes as always-open; [ ] a pre-N1 Market Town
   save resumed with the N1 binary and pack: the N1-I8 outcome (named drift refusal, or old terms kept) asserted;
   [ ] a test-fixture world authoring *different* hours (e.g. a 24 h shop and a split 08:00–12:00/14:00–18:00 shop)
   loads and judges correctly with no code change (operator acceptance 8, F14).
5. **Acceptance.** 22:00–06:00 is open at 23:59:59 and 00:00:00, closed at 06:00:00; every malformed form fails load
   naming file and index; absent hours change no existing economy test outcome.
6. **Failure/edges.** Invalid hours stop world load (never silently always-open); an unsupported old payload is a
   named version refusal; no runtime System dependency on schedule (type-only, checked in `declaration`).
7. **Commands/evidence.** `cargo test -p mineworld-economy` (owning crate only; `R-LOAD-1`: no local full workspace),
   `cargo fmt --all --check`, `cargo clippy -p mineworld-economy -- -D warnings`, `mineworld reference --check
   docs/reference`. NOT RUN (design). Record counts, wall time, schema diff.
8. **Boundary.** Typed configuration and persistence only. [ ] independent logic review of normalisation and
   migration; [ ] staged diff and ledger recorded before commit.

### N1-C2 — one availability judgement and disclosure

1. **Goal.** No sale path, current or alternate, can bypass closure; controllers can see closure without walking in.
2. **Scope.** `systems/economy/src/{sale,haggle,offer,code,component}.rs` (`ShopStock` computation), economy
   sale/haggle integration tests. Depends on C1.
3. **Implementation.**
   - [ ] `judge_open(world, &sale)` for Shop sellers, called first inside `judge_exchange`.
   - [ ] Call it in `haggle.rs::judge_proposal` (which does not use `judge_exchange`).
   - [ ] New code `shop-closed` in `code.rs`; offers carry it.
   - [ ] `ShopStock`: closed shop ⇒ `stocked: []`; update its documented meaning (N1-I6).
   - [ ] Audit every other `Seller::Shop` consumer (`buys`, `settle_buy`, listing, price-survey reads) and record.
4. **Validation.** [ ] hand-computed boundaries (open, `from`, `until`, overnight) through the `buy` offer, direct
   `buy`, `propose-price`, `accept-proposal` offer and accepted settlement; [ ] proposal made open, accepted closed ⇒
   refused; [ ] balances, `Holdings`, absence of `sold` unchanged on refusal; [ ] reopen succeeds; [ ] closed shop with
   zero stock reports `shop-closed`, not `out-of-stock`; [ ] person-to-person sale unaffected; [ ] `shop-stock` for a
   closed shop is empty, open shop unchanged. Negative: no test merely restates getters.
5. **Acceptance.** All Shop paths agree exactly at the close instant; accepted haggles cannot bypass hours; keepers
   still re-price closed shops.
6. **Failure/edges.** Removed shop/stale seller keeps existing precedence; same-instant close vs accept follows the
   kernel's instant order (judged at `world.at()`); no client-only refusal exists.
7. **Commands/evidence.** `cargo test -p mineworld-economy --test buy --test sale --test haggle --test
   haggle_settlement --test settlement` plus the new boundary target; `cargo test -p mineworld-price-survey`. NOT RUN.
8. **Boundary.** One sale rule with evidence. [ ] independent caller-path review covering `judge_proposal`;
   [ ] staged diff/ledger before commit.

### N1-C3 — Market Town hours, residents' night, decision record

1. **Goal.** Configured closures in the reference town without breaking residents' food access or S35 rhythm.
2. **Scope.** `worlds/market-town/organizations/{cafe-company,corner-store,station-news}.yaml`; `cognition/motivated`
   only if N1-I7 fails; Market Town baselines (`scripts/baselines.txt [world market-town]`) re-recorded under the
   golden-token rule; new `docs/decisions/ARC-<next>.md` (shop hours) and `docs/DECISIONS.md`/INDEX row; this ledger.
3. **Implementation.**
   - [ ] Author Q-L1 intervals; keep prices, nutrition, wages, shared apartments.
   - [ ] Bounded headless night scenario (CLI `run`, Market Town, 1 night + morning).
   - [ ] If walks/refusals exceed N1-I7, correct the owning controller and add its regression.
   - [ ] Decision record (states the value-type extension of `ARC-28`); amend `operator-playable-mvp-20261123.md` §3
         "Shop hours" row to point here.
   - [ ] Record citations for the ruled hours as evidence (published hours of comparable cafés, convenience stores and
         station newsstands). The values are operator-ruled content; a source that contradicts them is reported, not
         acted on (realistic-defaults rule, F14).
4. **Validation.** [ ] scenario: a purchase before close, `shop-closed` after, breakfast after opening, at least one
   resident home → sleep → wake → work; [ ] N1-I7 counts from the event log; [ ] Social Café unchanged (removal
   witness, absent hours); [ ] save/reopen reproduces hours; [ ] packaged launcher night check when P1 has landed
   (Gate2, one heavy slot, ≤ 10 min).
   **Long records (F7):** [ ] re-record `scripts/baselines.txt [world market-town]` (`summary-300`, `summary-1000`,
   `facts-300s`, `journal-300s`, `sealed-300s`) and [ ] run AC-S35-8 (`tools/cli/tests/rhythm.rs`, `--ignored`,
   release, 30 days × seeds {1, 7, 42}) **once each on the final head**, in the primary-assigned heavy slot (one
   heavy job at a time; projection recorded before launch; ≤ 60 min wall in total for N1; exceeding it is a material
   stop). The measured AC-S35-8 values are recorded in this ledger; "not relaxed" is a measured PASS, never a claim.
   If CI gains a nightly long-record job on the exact head, its result may replace the local run, recorded as remote.
5. **Acceptance.** Sales close at authored times; residents eat across the night/morning; AC-S35-8 measured PASS on
   the final head — a violation is corrected in content with evidence or escalated, never relaxed.
6. **Failure/edges.** Closure that starves a routine is a content defect; absence of the launcher prerequisite blocks
   only the packaged check, which is recorded `INCONCLUSIVE — prerequisite`, never PASS.
7. **Commands/evidence.** Exact CLI scenario command recorded before running; targeted `mineworld-cli` tests
   (`market_town`, `market_settlement`); CI on exact head is the full gate. NOT RUN.
8. **Boundary.** Completes N1's checkpoint. [ ] independent review of food access and removal witnesses;
   [ ] maintainability review (§7); [ ] staged diff/ledger before commit.

## 6. PR N2 — sleep-until and single-player advance

Branch `feat/nightlife-n2-sleep-advance`, fresh session, base = main after N1 merged (re-audit the seams). N2-C1 …
N2-C3 need nothing else; N2-C4 needs P1, P2 and P3 (§1.3).
Integration checkpoint: in a `--solo` Market Town save, the player sleeps until 07:00 and the host runs a multi-event
night faster than wall time with oracle parity, stopping early on an earlier wake; cancel, kill/reopen and repeat-id
behave as N2-I8 … I14 in both reference clients.

### N2-C1 — `sleep-until`, the morning, and controllers that ignore it

1. **Goal.** An exact, server-computed sleep target, before any host machinery depends on it, with no change to
   residents' behaviour.
2. **Scope.** `systems/sleep/src/{action,interactions,offer,system,lib}.rs`, sleep docs/README section table,
   `worlds/market-town/configure/sleep.yaml`, `docs/reference` regeneration, `docs/decisions/ARC-<next>.md`
   (sleep-until), step-35 dated note on Q-S35-9; `cognition/rule-controller/src/paced.rs` (`DECIDED_BY_NAME`) and
   `cognition/motivated/src/options.rs` (`Offered` exclusion) with their regressions (N2-I1b); offer-list
   tests/goldens that enumerate Market Town offers (`tools/cli/tests/{shown_parity,client_2d_interact,ac13_clients,
   …}.rs` — re-audit); Market Town baselines.
3. **Implementation.**
   - [ ] `SleepUntil { at }` action; `providing::<SleepUntil>()`; **no new `ActionDecl`** — rulings and parameters
         resolved with `Sleep::ACTION_TYPE` (N2-I1, F8).
   - [ ] Morning parameter (N2-I2); version pin test updated; N2-I14 resume outcome recorded.
   - [ ] Shared admission helper for `sleep` and `sleep-until` (one ordered list; the alarm is the only difference).
   - [ ] Complete offer with the exact target; refused offers mirror validate.
   - [ ] Rule controller `DECIDED_BY_NAME` += `sleep-until`; motivated `Offered` excludes it (N2-I1b, F1).
   - [ ] Citation for the 07:00 default recorded as evidence (the ATUS wake-time source step-35 already uses); the
         value is operator-ruled.
4. **Validation.** [ ] 23:00:12 → 07:00:00 alarm exact; [ ] 06:59 and 15:00 out of bound; [ ] past/now `at`;
   [ ] forbidden place; [ ] `forbid sleep` place/actor rules refuse `sleep-until`, `permit sleep` admits it;
   [ ] `Busy` while asleep; [ ] regional morning override; [ ] alarm holder-only in disclosures;
   [ ] all four wake ways end a `sleep-until`; [ ] save mid-sleep → reopen → alarm fires; [ ] pre-N2 Market Town
   save → named drift refusal (N2-I14); [ ] rhythm regression (N2-I1b): same seed, same decisions, asleep share at
   14:00 and 17:00 unchanged; [ ] long records as N1-C3 (baselines re-record + AC-S35-8 once on the final head, same
   slot rule, ≤ 60 min for N2).
5. **Acceptance.** No client arithmetic exists; the offered `at` validates unchanged at dispatch or is refused as stale.
6. **Failure/edges.** Overflowing `at` refused; a world without `sleep` offers nothing; shared worlds offer it too.
7. **Commands/evidence.** `cargo test -p mineworld-sleep`; targeted `mineworld-cli` offer tests. NOT RUN.
8. **Boundary.** World rule plus the two controllers' exclusion, no server. [ ] independent review of admission
   order, privacy and the controllers' unchanged behaviour; [ ] diff/ledger.

### N2-C2 — advance contract, lifecycle and durable identity (no loop)

1. **Goal.** Authenticated, idempotent, recoverable operation and its refusals, reviewable before acceleration.
2. **Scope.** `server/src/admin.rs` (`POST /admin/advance`, `GET /admin/advance/{operation}`, `POST
   /admin/advance/{operation}/cancel`), `server/src/host.rs`, new `server/src/runtime/advance.rs` (state machine),
   the `AdvanceStop` trait in the domain-free crate chosen per N2-I4, the sleep pack's implementation
   (`systems/sleep/src/advance.rs`), one `extension` line in `systems/installed/src/lib.rs`, `server/src/seats.rs`
   refusals, `server/PROTOCOL.md` §§4, 11, `persistence/src/host.rs` + backend (the `host_operations` table),
   `docs/decisions/ARC-<next>.md` (advance; **amends ARC-69 item 5** as §0 states) with a dated note on `ARC-69`.
   `tools/cli/src/hosted.rs` is not edited.
3. **Implementation.**
   - [ ] Typed request `{ operation, seat, until: "<adapter name>" }` — no instant; typed refusals: `not-solo`,
         `unauthenticated`, `seat-mismatch`, `other-controller-connected` (N2-I3 wording, incl. holds),
         `not-asleep`, `paused`, `busy`, `unknown-adapter`, `operation-conflict`, `advance-in-progress`.
   - [ ] `host_operations` table records; legacy saves without it read as empty; recovery appends `interrupted`.
   - [ ] Command/route behaviour of N2-I12 for everything except the loop itself.
   - [ ] Idempotent status by operation id.
   - [ ] Until C3 wires the loop, an admitted request records `advance-started`, moves no time, and ends
         `stopped` with reason `not-yet-executable` — never a false `woke`. C3 removes that branch.
4. **Validation.** [ ] each refusal; [ ] repeated id same/different payload; [ ] shared host and two-session host
   refused; [ ] a seat inside a hold window refused, an observer session allowed; [ ] crash between records →
   reopen → `interrupted`, no re-advance; [ ] `verify` coherence still passes; [ ] a world without `sleep` →
   `unknown-adapter`; [ ] a pre-N2 save (no table) resumes with an empty operation history.
5. **Acceptance.** Rejected requests cause no action and no record; neither the server crate nor `tools/cli` names a
   sleep type.
6. **Failure/edges.** Journal write failure fail-stops; unknown record kind is a named recovery error.
7. **Commands/evidence.** `cargo test -p mineworld-persistence --test host_journal`; `cargo test -p mineworld-server`
   filtered to admin/host/advance; `cargo test -p mineworld-cli --test <hosted target>`. NOT RUN.
8. **Boundary.** [ ] independent auth/privacy/recovery review; [ ] diff/ledger.

### N2-C3 — exact-due execution

1. **Goal.** Execute every intervening decision and process, stopping at the first completed wake instant.
2. **Scope.** `server/src/runtime/advance.rs`, a small extraction in `server/src/runtime.rs` (one-instant consult
   shared with `consult`), `server/src/{hosted,seats}.rs` (earliest due without mutation), `runtime/world.rs`
   (`HostClock` hold and re-anchor), `runtime/delivery.rs` (queue-depth query for N2-I13), runtime tests.
3. **Implementation.**
   - [ ] **First**, freeze the ordinary-host oracle trace (injected wall clock ticking at every due instant) as a
         fixture from the current code; never compute expectations with the new loop.
   - [ ] Loop per N2-I5; slices per N2-I7 with self-posted continuation (N2-I12); delivery per N2-I13;
         durable-before-publish; cancel stops only (N2-I10); re-anchor per N2-I10.
   - [ ] Ordinary `tick_at` semantics unchanged (its jitter-skip behaviour remains).
4. **Validation.** [ ] oracle parity (N2-I6) on a Market Town night with strides, schedule boundaries, wages, a
   purchase and same-instant ties; [ ] early wake at 02:00 by talk stops there; [ ] alarm at target; [ ] mutation
   checks: a jump to morning and `now = horizon` must each fail parity/early-wake; [ ] cancel/disconnect/shutdown
   between slices; [ ] a `Sweep` between slices neither consults nor advances; [ ] a sleeper `wake` between slices
   ends the advance `woke` at the boundary instant; [ ] cancel leaves the sleeper asleep and submits nothing;
   [ ] scale unchanged; [ ] fault mid-advance not retried; [ ] sleeper queue high-water mark recorded.
5. **Acceptance.** Visited instants equal the oracle's, in order; final instant = first completed wake instant;
   cursor never ahead of durable history. **Speed (F14):** a Market Town 23:00 → 07:00 night (scale 12; at ordinary
   pace 40 wall minutes) completes in ≤ 60 wall seconds in a release build on the reference Mac, recorded with
   visited-instant, consult and commit counts (operator reconciliation §7.1). Missing the bound is reported with a
   profile as a material stop, never tuned by skipping work.
6. **Failure/edges.** No next kernel entry; simultaneous due seats; destroyed sleeper; zero-progress controller (bounded
   by slice + horizon); overflow; queue saturation — none spins or reports morning.
7. **Commands/evidence.** `cargo test -p mineworld-server` filtered to runtime/advance; focused CLI hosted cadence
   tests. Record fixture hash, visited-instant count, wall vs world time. NOT RUN.
8. **Boundary.** [ ] independent red-team review (missed stride, horizon skip, first wake, same instant, join race,
   durability); [ ] runtime.rs growth reviewed — orchestration lives in `advance.rs`; [ ] diff/ledger.

### N2-C4 — both clients and real recovery

1. **Goal.** Server-owned sleep/advance/cancel in packaged play; real kill/reopen.
2. **Scope.** `clients/shared/host/{host_clock,world_tab}.gd`, `clients/shared/HOST.md` (§3.2 S21 reference),
   `clients/shared/checks/host_check.gd`, `clients/2d/scripts/{app.gd,hud/readers.gd,hud/panels.gd}`,
   `clients/3d-spike/scripts/slice/needs_hud.gd`. **Not** `tools/launch`: the `--solo` start and the per-run token
   are prerequisites P2/P3 owned by the launcher lane.
3. **Implementation.** [ ] `Sleep until 07:00` from the offer; after success, request advance with a fresh id;
   [ ] progress/asleep state from frames only; [ ] "Wake up" sends `wake` through the player's session (which ends
   the advance `woke`); a separate "stop fast-forward" sends cancel and leaves the player asleep;
   [ ] reconnect polls status by id, never resubmits; [ ] named "unavailable" when refused (shared host, other
   controller, no admin capability).
4. **Validation (Gate2, heavy slot, each ≤ 10 min, total ≤ 60 min).** [ ] 2D and 3D: sleep → advance → morning;
   early wake; cancel; [ ] kill during advance → reopen → interrupted, durable state, no repeated wages/restock;
   [ ] repeat id after reopen; [ ] shared host and second client refused; [ ] day length unchanged afterwards.
5. **Acceptance.** Both clients finish at the authoritative instant with prior pacing; disconnect does not finish the
   night; tokens never appear in observations, logs or the save.
6. **Failure/edges.** Missing admin token ⇒ ordinary sleep only, with a named message; UI waits never become
   wall-time simulation.
7. **Commands/evidence.** Launcher/client lifecycle commands predeclared at the execution head; logs, screenshots,
   world cursor. NOT RUN.
8. **Boundary.** [ ] independent client-authority/privacy/restart/maintainability review; [ ] diff/ledger.

## 7. Test ownership, review and load

- Static: fmt, clippy `-D warnings`, schema/reference consistency. Deterministic integration: hours, sale/haggle
  agreement, sleep bounds, admission, journal replay, oracle parity. Gate1: **N/A** (no model, no prompt).
  Gate2: packaged launcher night (N1-C3) and both-client advance/kill/reopen (N2-C4).
- Local runs are owning-crate targets only; CI on the exact final head is the full gate (`R-LOAD-1`). The only long
  records are the ones N1-C3 and N2-C1 name (baselines re-record + AC-S35-8, once each on the final head, in the
  primary-assigned heavy slot, ≤ 60 min per PR); no other 30/300/1000-day campaign. Record local and remote
  results separately.
- Every ready-PR review records findings on technical debt, module boundaries, readability, robustness, extensibility,
  oversized functions/files (`runtime.rs` 509, `haggle.rs` 558, `sleep/system.rs` 356 are watched), coupling.

## 8. Corrections against the recovered drafts

| # | Draft said | Corrected here |
| --- | --- | --- |
| C1 | Audit base `b457a94d` | Re-audited on `7718d952`; relevant paths unchanged (§2.1) |
| C2 | Closure "in `judge_exchange`" guards every path | `haggle.rs::judge_proposal` bypasses `judge_exchange`; it calls the closure judgement explicitly |
| C3 | Food access only "audited" | Rule fetch band step 7 walks to shops; closed shops are now disclosed `stocked: []` (N1-I6) and walks/refusals are bounded (N1-I7); motivated controller audited |
| C4 | Advance bound to "target" | Admin surface must name no instant (`ARC-40`); horizon comes from the sleeper's alarm via the adapter |
| C5 | `sleep-until` framed as single-player | It is a world rule in every world; only the advance is solo (and that amends `ARC-69` item 5, §0) |
| C6 | Morning in a new "optional minute-of-day configuration" | A `SleepParameters` entry in the existing section; pre-N2 saves refused as drift (N2-I14); offer goldens re-recorded |
| C7 | Economy depends on schedule for `TimeOfDay` (only option) | Kept as type-only (`ARC-28`); sleep restates `DAY` like calendar (`INV-TW-4`) |
| C8 | "External/model sessions are rejected" without consequence | Conflicts with the dialogue lane's LM-Alice profile; surfaced as Q-L4 |
| C9 | N2 as three commits mixing contract and loop | Four commits: sleep rule, lifecycle, loop, clients — each independently reviewable; oracle frozen before the loop |
| C10 | S21 "doze" and HOST.md references unreconciled | Reconciled in §0; HOST.md fixed in N2-C4; S21 may reuse the operation |
| C11 | Launcher prerequisite assumed landing | Still in the active client-stack lane on `7718d952`; blocks only packaged checks |

## 9. Evidence and readiness

Design-only lane. Source audit on `7718d952`, seam drift re-checked on `8c971f0f` (§2.1). No production edit, build,
test or model call was made. Deterministic/runtime/UI checks: NOT RUN. Independent red-team pre-freeze review:
DONE (`/Users/yuema137/mineworld-worktrees/_reviews/design-nightlife.md`, verdict FREEZE-AFTER-FIXES); every finding
dispositioned in §10. Operator freeze: 2026-11-27 (header). Implementation is dispatchable per the header's start
conditions in a fresh session with the filled contract.

## 10. Red-team pre-freeze review — disposition (revision 2)

None of the corrections changes an operator ruling; each is a bounded design correction within Q-L1 … Q-L5.

| Finding | Disposition |
| --- | --- |
| F1 BLOCKER — complete `sleep-until` offer drawn at random by hosted controllers | Fixed: N2-I1b, N2-C1 scope/implementation/validation (rule `DECIDED_BY_NAME`, motivated exclusion, rhythm regression). Keeps Q-L2's complete server-computed offer |
| F2 — world thread during an advance | Fixed: N2-I12 command/route table, HostClock hold, continuation; N2-C3 tests |
| F3 — cancel submitting `wake` for a human | Fixed: N2-I10 — cancel stops only; the sleeper's own `wake` ends the advance; N2-C4 UI split |
| F4 — "never dropped" vs bounded delivery | Fixed: N2-I5 narrowed; N2-I13 delivery rule and `lagged` outcome |
| F5 — launcher not `--solo`, no admin token | Fixed: §1.3 P2, P3 owned by the launcher lane; N2-C4 no longer touches `tools/launch`; the `--solo` product consequence is the launcher design's to surface |
| F6 — existing-save resume | Fixed: N1-I8, N2-I14 apply the accepted `ARC-61` item 7 rule; fixture tests in N1-C1 and N2-C1 |
| F7 — long records unbudgeted | Fixed: N1-C3 / N2-C1 name the runs, slot and ≤ 60 min budget; §7 |
| F8 — split Interaction-List rules | Fixed: option (b) without a second `ActionDecl` — `sleep`'s rules govern `sleep-until` (N2-I1) with a forbid/permit test |
| F9 — ARC-69 item 5 quietly amended | Fixed: §0 records the amendment as a consequence of the Q-L3 ruling; N2-C2's ARC record supersedes item 5's wording |
| F10 — CLI depending on a domain pack | Fixed: N2-I4 — trait in a domain-free crate, sleep implements, `ARC-62` extension line in `systems/installed`; `tools/cli` untouched |
| F11 — admission wording | Fixed: N2-I3 (sessions and hold windows; observers allowed) |
| F12 — journal shape | Fixed: N2-I8 — separate lazily created `host_operations` table, `PacingChange` unchanged, typed `OperationId` |
| F13 — stale claims | Fixed: HostClock location (§2.2), all twelve routines (§2.3), full refusal precedence (N1-I3), `ARC-28` value-type extension stated |
| F14 — speed bound, references, variant world, restamp | Fixed: N2-C3 speed acceptance; citations as evidence in N1-C3/N2-C1; N1-C1 variant-hours fixture; N2 `unknown-adapter` test; header restamped to `8c971f0f` |
| F15 — confirmed sound | No action |
