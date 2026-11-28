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
Lifecycle:              FROZEN (2026-11-27); N1 merged (#552); N2 implementation in progress (mvp0/pr-nightlife-n2, §12)
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
   - [x] `Hours` type: 0…8 `[from, until)` intervals of `TimeOfDay`, normalised across midnight; `open_at(WorldTime)`.
         → `systems/economy/src/hours.rs` (`Hours`, `Opening`, `HoursError`, `MAX_OPENINGS`); `Hours::new` refuses a
         ninth, an empty interval and an overlap after splitting a wrapping interval into `[from, 86 400) ∪ [0, until)`;
         valid on every decode (`Deserialize` re-runs `new`), the `Segments` pattern of schedule.
   - [x] `AuthoredShop.hours: Option<Vec<…>>`; refusals naming the source file and interval index.
         → `AuthoredShop.hours: Option<Hours>`; refused at decode (serde custom error: "hours interval 1 opens and
         closes at 09:00", "hours intervals 0 and 1 overlap", "at most 8 intervals, but these have 9", schedule's
         `"7:00"` message); the loader names the file (asserted: `day-co.yaml`).
   - [x] Thread through `ShopOpened` → `Shop` reducer; decide additive-optional vs version bump (N1-I4) and record.
         → **Additive-optional, no bump** (decision D-N1-1 in §11). `ShopOpened` (schema 2) and `Shop` (schema 2) gain
         `#[serde(default, skip_serializing_if = "Option::is_none")] hours`, the `adjustments` precedent (added by WE-a
         C4 `4dca8647` without a bump). Economy `SystemVersion` stays 5. A shop without hours encodes byte-identically
         to the pre-N1 encoding, so worlds without hours keep their digests.
   - [x] Determine whether hours reach a resume-compared configuration fact and record the N1-I8 branch.
         → They do **not**: hours are an organization's `economy:` **section** (content), seeded as `shop-opened`;
         economy's `configuration_facts` are `currency-declared` and `economy-configured` only (`system.rs` `FACTS`).
         `ARC-61` item 7 compares configuration facts only, and its "Accepted limitations" state content drift is
         not checked. **N1-I8 branch: the save keeps exactly its genesis terms (always open).**
   - [x] Regenerate the schema; document defaults and refusals in the pack's module docs.
         → `mineworld reference --out docs/reference` (section-economy.schema.json +62, sections.md +28: `hours`,
         `Hours`, `Opening`, `TimeOfDay`); `section.rs`/`lib.rs` docs, `systems/economy/README.md`,
         `docs/MODULE_SPEC.md` economy row.
4. **Validation.** [x] authoring+seeding integration for absent, `[]`, ordinary, overnight (22:00–06:00), equal
   endpoints, overlap after midnight, ninth interval, malformed `"7:00"`; [x] save → reopen → equal `Shop` terms;
   [x] a pre-N1 save fixture (existing economy persistence fixtures) decodes as always-open; [x] a pre-N1 Market Town
   save resumed with the N1 binary and pack: the N1-I8 outcome (named drift refusal, or old terms kept) asserted;
   [x] a test-fixture world authoring *different* hours (e.g. a 24 h shop and a split 08:00–12:00/14:00–18:00 shop)
   loads and judges correctly with no code change (operator acceptance 8, F14).
   Evidence: `systems/economy/tests/hours.rs` (4 tests, real loader + real SQLite):
   `authored_hours_reach_every_shop_and_judge_its_day` (always/[]/06–23/22–06/split, 9 hand-computed instants on
   day 0 and day 7); `malformed_hours_stop_the_load_naming_the_file_and_the_interval` (equal, overlap after
   midnight, ninth, `"7:00"`, unknown key); `hours_are_the_same_after_save_and_reopen`;
   `a_save_from_before_hours_resumes_with_its_shops_always_open` (a save created from the pack without hours —
   byte-identical to a pre-N1 save, since an hours-less shop encodes as before — passes `check_configuration`
   against the pack *with* hours, resumes, and sells at 23:30 in the shop the pack now closes at 23:00). No economy
   persistence fixture existed (bounded deviation D-N1-2): the pre-N1 save is produced in-test. The Market Town
   variant of the N1-I8 check runs in N1-C3 (`tools/cli`). Unit: `hours::tests` (boundaries 05:59:59 / 06:00:00 /
   23:59:59 / 00:00:00; overlap after wrap; touching intervals allowed).
5. **Acceptance.** 22:00–06:00 is open at 23:59:59 and 00:00:00, closed at 06:00:00; every malformed form fails load
   naming file and index; absent hours change no existing economy test outcome. **Met** (unit + integration above;
   all 16 pre-existing economy test targets unchanged and passing).
6. **Failure/edges.** Invalid hours stop world load (never silently always-open); an unsupported old payload is a
   named version refusal; no runtime System dependency on schedule (type-only, checked in `declaration`).
   **Met**: `EconomySystem::declaration` unchanged (`depending_on([Inventory, Presence])`); `mineworld-schedule` is a
   type-only Cargo dependency, as employment's.
7. **Commands/evidence.** `cargo test -p mineworld-economy` (owning crate only; `R-LOAD-1`: no local full workspace),
   `cargo fmt --all --check`, `cargo clippy -p mineworld-economy -- -D warnings`, `mineworld reference --check
   docs/reference`. **Run 2026-11-27 on the C1 tree (local, macOS arm64, debug-optimized test profile)**: economy
   17 targets PASS (5 lib + 64 integration tests incl. 4 new); clippy `--all-targets -D warnings` PASS; fmt PASS;
   `reference --check` PASS (46 files). Mutation: removing the overlap check fails
   `overlap_is_judged_after_wrapping…` (caught).
8. **Boundary.** Typed configuration and persistence only. [x] logic review of normalisation and migration
   (author self-review at commit: adjacent-pair overlap after sorting is complete because any overlap of span *i*
   with a later *j* implies *i*'s successor starts before *i* ends; a wrap's two pieces are disjoint; no version
   bump is safe because decode re-validates and absent hours skip serialization. The independent review of the
   whole PR is recorded in §11); [x] staged diff and ledger recorded before commit.

### N1-C2 — one availability judgement and disclosure

1. **Goal.** No sale path, current or alternate, can bypass closure; controllers can see closure without walking in.
2. **Scope.** `systems/economy/src/{sale,haggle,offer,code,component}.rs` (`ShopStock` computation), economy
   sale/haggle integration tests. Depends on C1.
3. **Implementation.**
   - [x] `judge_open(world, &sale)` for Shop sellers, called first inside `judge_exchange`.
         → `sale.rs::judge_open` ("step 4½"): a `Seller::Shop` whose `Shop::open_at(world.at())` is false →
         `shop-closed`; a person seller has no hours; a shop that is gone falls through to the existing steps.
         `judge_exchange` = `judge_open` → `judge_self_trade` → `judge_goods`; `judge_sale` keeps `price-changed`
         before it, so the N1-I3 precedence holds for `buy`.
   - [x] Call it in `haggle.rs::judge_proposal` (which does not use `judge_exchange`).
         → after `resolve_seller` and the price range, before self-trade and the limits (N1-I3 for propose).
   - [x] New code `shop-closed` in `code.rs`; offers carry it.
         → `code::SHOP_CLOSED`. `buy` offers carry it through `judge_sale`; the `accept-proposal` offer through
         `judge_exchange`; the shop-directed `propose-price` offer explicitly (`haggle::offers`, closed before the
         keeper/self-trade refusal, the dispatch's order).
   - [x] `ShopStock`: closed shop ⇒ `stocked: []`; update its documented meaning (N1-I6).
         → `offer.rs::shop_stock` reads `world.at()` (the observation's read is at the world's now); doc on
         `component.rs::ShopStock`: "asks, holds at least one of, and has on sale now". Schema 1 unchanged.
   - [x] Audit every other `Seller::Shop` consumer (`buys`, `settle_buy`, listing, price-survey reads) and record.
         → `buys` and `validate` → `judge_sale` (covered); `settle_buy` re-judges `judge_sale` at the resolution's
         instant (covered); accepted-proposal settlement → `judge_acceptance` → `judge_exchange` (covered);
         `listing.rs` (list/set-price/withdraw) judges seller authority only — unchanged by design (N1-I5);
         the shop `Listing` disclosure (operator, prices, stock) is unchanged: prices in a window are still
         readable while closed, and `price-survey` (`judge.rs` reads `Listing`) surveys asks, never buys —
         unaffected. No other crate reads `Shop` to sell (`git grep mineworld_economy::` outside economy:
         price-survey only).
4. **Validation.** [x] hand-computed boundaries (open, `from`, `until`, overnight) through the `buy` offer, direct
   `buy`, `propose-price`, `accept-proposal` offer and accepted settlement; [x] proposal made open, accepted closed ⇒
   refused; [x] balances, `Holdings`, absence of `sold` unchanged on refusal; [x] reopen succeeds; [x] closed shop with
   zero stock reports `shop-closed`, not `out-of-stock`; [x] person-to-person sale unaffected; [x] `shop-stock` for a
   closed shop is empty, open shop unchanged. Negative: no test merely restates getters.
   Evidence: `systems/economy/tests/closed.rs` (5 tests): `every_way_of_buying_agrees_at_the_open_and_close_instants`
   (08:00–20:00 at 07:59:59 / 08:00:00 / 19:59:59 / 20:00:00 and 22:00–06:00 at 21:59:59 / 23:59:59 / 00:00:00 /
   06:00:00: `buy` offer, `propose-price` offer, `shop-stock`, direct `buy` with state/wallet/holdings unchanged and
   no facts on refusal, direct `propose-price`); `a_proposal_made_open_cannot_be_accepted_after_closing` (19:50
   proposal accepted 19:55 settles; accepted 20:02 refused in offer and dispatch, nothing moves; next day 08:00 a
   buy sells); `closed_comes_after_the_seller_and_price_and_before_funds_and_stock` (out-of-stock and
   insufficient-funds become shop-closed when closed; not-for-sale and price-changed precede it);
   `a_person_still_sells_and_a_keeper_still_reprices_in_a_closed_shop`; `a_shop_without_hours_is_always_open`.
   `hours.rs::hours_are_the_same_after_save_and_reopen` now also asserts the reopened save refuses `shop-closed`.
5. **Acceptance.** All Shop paths agree exactly at the close instant; accepted haggles cannot bypass hours; keepers
   still re-price closed shops. **Met** (above).
6. **Failure/edges.** Removed shop/stale seller keeps existing precedence; same-instant close vs accept follows the
   kernel's instant order (judged at `world.at()`); no client-only refusal exists. **Met**: `judge_open` answers
   `Ok` when the `Shop` component is gone; every judgement reads `world.at()`; no client or controller code changed.
7. **Commands/evidence.** `cargo test -p mineworld-economy --test buy --test sale --test haggle --test
   haggle_settlement --test settlement` plus the new boundary target; `cargo test -p mineworld-price-survey`.
   **Run 2026-11-27 on the C2 tree (local, macOS arm64)**: `cargo test -p mineworld-economy -p
   mineworld-price-survey` — every target PASS (economy 18 targets incl. `closed` 5/5 and `hours` 4/4;
   price-survey all PASS); clippy `-p mineworld-economy --all-targets -D warnings` PASS; fmt PASS.
   Mutations (each reverted): M1 drop `judge_open` from `judge_proposal` → `every_way…` FAILS (caught);
   M2 drop it from `judge_exchange` → 3 tests FAIL (caught); M3 move it after `judge_goods` →
   `closed_comes_after…` FAILS (caught); M4 `shop_stock` ignores hours → `every_way…` FAILS (caught).
8. **Boundary.** One sale rule with evidence. [x] caller-path review covering `judge_proposal` (author's
   caller-path audit in item 3; every `judge_self_trade` / `judge_goods` caller now has `judge_open` before it:
   `judge_exchange` and `judge_proposal` are the only two — `git grep judge_goods` — independent PR review in
   §11); [x] staged diff/ledger before commit.
### N1-C3 — Market Town hours, residents' night, decision record

1. **Goal.** Configured closures in the reference town without breaking residents' food access or S35 rhythm.
2. **Scope.** `worlds/market-town/organizations/{cafe-company,corner-store,station-news}.yaml`; `cognition/motivated`
   only if N1-I7 fails; Market Town baselines (`scripts/baselines.txt [world market-town]`) re-recorded under the
   golden-token rule; new `docs/decisions/ARC-<next>.md` (shop hours) and `docs/DECISIONS.md`/INDEX row; this ledger.
3. **Implementation.**
   - [x] Author Q-L1 intervals; keep prices, nutrition, wages, shared apartments.
         → `worlds/market-town/organizations/{cafe-company,corner-store,station-news}.yaml`: one `hours` line each
         (05:30–22:00, 06:00–23:00, 06:00–18:00); nothing else changed.
   - [x] Bounded headless night scenario (CLI `run`, Market Town, 1 night + morning).
         → `tools/cli/tests/shop_hours.rs`: `mineworld run worlds/market-town --headless --seed 7 --days 2 --save <scratch>`
         (command fixed in the test), under `rule` (default) and under every seat `motivated`
         (`--decisions cognition/motivated/examples/social-cafe-all-motivated.yaml`).
   - [x] If walks/refusals exceed N1-I7, correct the owning controller and add its regression.
         → **Not exceeded; no controller edited.** rule: 0 `shop-closed` refusals; closed-shop entries
         `{(person #10, place #2, night 0): 1, (…, night 1): 1}` — one entry per night into one shop before it opens,
         the pattern of the café employee's punctual pre-05:30 arrival. motivated: identical entries, 0 refusals,
         0 purchases — `motivated` makes no purchase in this 2-day Market Town window on the base content either
         (same 5 492 facts with and without hours; pre-existing behaviour of that model, not N1's). The motivated
         run stays as a regression (`motivated_residents_do_not_haunt_closed_shops`).
   - [x] Decision record (states the value-type extension of `ARC-28`); amend `operator-playable-mvp-20261123.md` §3
         "Shop hours" row to point here. → `docs/decisions/ARC-364.md` (item 7 states the `ARC-28` value-type
         extension) + INDEX row; dated note on `ARC-360` (shop-stock meaning, N1-I6); the reconciliation row amended.
   - [x] Record citations for the ruled hours as evidence. → In each organization file: café — US coffeehouse
         chains commonly open 05:30–06:00 and close 20:00–22:00 (Starbucks general hours, about.starbucks.com /
         store locator); store — 7-Eleven named in 1946 for its 07:00–23:00 day (Britannica, "7-Eleven",
         https://www.britannica.com/money/7-Eleven); newsstand — no independent source recorded (operator-ruled
         content, stated so). No source contradicts the ruled values.
4. **Validation.** [x] scenario: a purchase before close, `shop-closed` after, breakfast after opening, at least one
   resident home → sleep → wake → work; [x] N1-I7 counts from the event log; [x] Social Café unchanged (removal
   witness, absent hours); [x] save/reopen reproduces hours; [ ] packaged launcher night check when P1 has landed
   (Gate2, one heavy slot, ≤ 10 min) — **INCONCLUSIVE — prerequisite P1** (launcher lane; not run).
   Evidence (local, 2026-11-27, macOS arm64): `cargo test -p mineworld-cli --test shop_hours` 3/3 PASS —
   ruled hours read back from `shop-opened`; 98 shop sales, none outside its shop's hours (hours parsed in-test
   from the fact, independent of economy's code); 29 evening sales (16:00–22:00), 15 day-1 breakfast sales
   (05:30–10:00); a resident woke on day 1 then started a shift present. "`shop-closed` after": no resident
   attempted a closed shop in this window (residents see the sign through `shop-stock`); the refusal itself is
   proven on every path in `systems/economy/tests/closed.rs`. N1-I7 entries are counted from the stored
   `person-entered-place` facts; refusals from the run report (aggregated by code, format-checked). Social Café
   and Bodies Yard author no hours and their files are untouched; an hours-less shop encodes byte-identically
   (D-N1-1), so their histories cannot move. Save/reopen: `hours.rs::hours_are_the_same_after_save_and_reopen`;
   pre-N1 Market Town save: `shop_hours.rs::a_market_town_save_from_before_hours_resumes_without_drift`.
   Behaviour is otherwise unchanged in this window: the same request counts with and without hours (rule,
   seed 7, 2 days: buy 98, eat 92, sleep 33 …); only the genesis `shop-opened` bytes differ.
   **Long records (F7):** [ ] re-record `scripts/baselines.txt [world market-town]` — **NOT DONE: deferred to the
   primary** (dispatch rule "report before re-recording any world golden"). Every Market Town key will move
   (genesis `shop-opened` bytes); the baselines are read by the nightly, not by `fast`. [ ] AC-S35-8 — see §11.
5. **Acceptance.** Sales close at authored times; residents eat across the night/morning; AC-S35-8 measured PASS on
   the final head — see §11.
6. **Failure/edges.** Closure that starves a routine is a content defect (none observed: every routine visits shops
   inside the hours, §2.3); absence of the launcher prerequisite blocks only the packaged check, recorded
   `INCONCLUSIVE — prerequisite`.
7. **Commands/evidence.** See item 4 and §11 (targeted `mineworld-cli` run).
8. **Boundary.** Completes N1's checkpoint. [x] independent review (red-team subagent, read-only, §11 R-1…R-8);
   [x] maintainability review (§11); [x] staged diff/ledger before commit.

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
   - [x] `SleepUntil { at }` action; `providing::<SleepUntil>()`; **no new `ActionDecl`** — rulings and parameters
         resolved with `Sleep::ACTION_TYPE` (N2-I1, F8).
         → `action.rs::SleepUntil` (`sleep-until`, `at: WorldTime`); `system.rs` declaration; `place.rs` / `terms`
         unchanged (already keyed by `Sleep::ACTION_TYPE`).
   - [x] Morning parameter (N2-I2); version pin test updated; N2-I14 resume outcome recorded.
         → `morning_minute: u16 = 420, 0 ..= 1439`; `SleepSystem` v2, `sleep-interactions-configured` /
         `sleep-interactions` schema 2 (D-N2-2); pre-N2 save refused by name (`sleep_until.rs`).
   - [x] Shared admission helper for `sleep` and `sleep-until` (one ordered list; the alarm is the only difference).
         → `system.rs::may_sleep(world, actor, target, Asked::{For, Until})` → `(PlaceId, alarm)`; `alarm_within`.
   - [x] Complete offer with the exact target; refused offers mirror validate.
         → `offer.rs::offered` + `morning_after`; order located → bounds → place, as validate.
   - [x] Rule controller `DECIDED_BY_NAME` += `sleep-until`; motivated `Offered` excludes it (N2-I1b, F1).
         → `rule-controller/src/paced.rs` (+4 lines), `motivated/src/options.rs` (+3 lines); nothing else in either.
   - [x] Citation for the 07:00 default recorded as evidence (the ATUS wake-time source step-35 already uses); the
         value is operator-ruled. → ATUS A-3 2025 *Sleeping* 659 ‰ (06:00 hour) → 425 ‰ (07:00 hour): the share asleep
         falls through one half in the 07:00 hour; in `worlds/market-town/configure/sleep.yaml` and `ARC-373`.
4. **Validation.** [x] 23:00:12 → 07:00:00 alarm exact; [x] 06:59 and 15:00 out of bound; [x] past/now `at`;
   [x] forbidden place; [x] `forbid sleep` place/actor rules refuse `sleep-until`, `permit sleep` admits it;
   [x] `Busy` while asleep; [x] regional morning override; [x] alarm holder-only in disclosures;
   [x] all four wake ways end a `sleep-until`; [x] save mid-sleep → reopen → alarm fires; [x] pre-N2 Market Town
   save → named drift refusal (N2-I14); [x] rhythm regression (N2-I1b): same seed, same decisions, asleep share at
   14:00 and 17:00 unchanged (evidence §12); [ ] long records as N1-C3 (baselines re-record + AC-S35-8 once on the final head, same
   slot rule, ≤ 60 min for N2).
5. **Acceptance.** No client arithmetic exists; the offered `at` validates unchanged at dispatch or is refused as stale.
6. **Failure/edges.** Overflowing `at` refused; a world without `sleep` offers nothing; shared worlds offer it too.
7. **Commands/evidence.** `cargo test -p mineworld-sleep`; targeted `mineworld-cli` offer tests. Run: §12.
8. **Boundary.** World rule plus the two controllers' exclusion, no server. [ ] independent review of admission
   order, privacy and the controllers' unchanged behaviour (PR-level red-team, §12); [x] diff/ledger.

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

## 11. N1 execution ledger (implementation session, 2026-11-27)

```text
Branch / worktree:   mvp0/pr-nightlife-n1 @ /Users/yuema137/mineworld-worktrees/impl-nightlife-n1
                     (the contract names feat/nightlife-n1-shop-hours; the primary's dispatch named
                     mvp0/pr-nightlife-n1 — bounded deviation D-N1-0, name only)
Implementation base: origin/main 8c971f0f (freeze base) at start; origin/main c161c120 merged in 2026-11-27
                     (after #537 merged; 50 commits, none in economy, market-town organizations or §2.2 seams' N1 paths)
Ledger carriage:     PR #537 (this design, docs-only) was not merged at the start; this PR carries the frozen
                     design and contract byte-identical to 86e65cf6 plus the N1 evidence, so the ledger lives
                     with the code it records. Whichever of #537 / N1 merges second takes the other's text.
Scope:               N1-C1, N1-C2, N1-C3 (N1-C3's packaged launcher check waits for P1). N2 not started.
```

### Decisions and bounded deviations

| ID | Question | Evidence | Decision |
| --- | --- | --- | --- |
| D-N1-0 | Branch name | dispatch from the primary | `mvp0/pr-nightlife-n1`, one worktree, one session |
| D-N1-1 | N1-I4: additive-optional or a version bump | `component.rs` `Shop.adjustments` added by WE-a C4 (`4dca8647`) with `serde(default, skip_serializing_if)` and no schema bump; `codec` decodes the payload with serde | Additive-optional; `ShopOpened` stays schema 2, `Shop` schema 2, economy `SystemVersion` 5. Hours are re-validated on every decode, so a stored invalid value is refused, never read as always-open |
| D-N1-2 | "Existing economy persistence fixtures" for a pre-N1 save | none exist (`persistence/tests/fixtures` holds format-2/3 only; economy tests build saves in-test) | The pre-N1 save is created in-test from the same pack without hours; it is byte-identical to a pre-N1 binary's save of that pack because an hours-less shop encodes exactly as before |
| D-N1-3 | N1-I8 branch | §5 N1-C1 item 4 above (hours are content; `ARC-61` compares configuration facts only) | "the save keeps exactly its genesis terms (always open)", asserted |

### Progress

| Checkpoint | Commit | State |
| --- | --- | --- |
| Frozen design + contract carried | `953f3331` (#537 later merged the same bytes; main merged in, no force-push) | done |
| N1-C1 typed, persistent shop hours | `39e9b0cd` | done |
| N1-C2 one availability judgement and disclosure | `d102ab5c` | done |
| N1-C3 Market Town hours, scenario, ARC-364 | `d0381dd2` (+ merge `4807b618`) | done |
| Hidden consumers: midnight-hosted buys, fact mirror | (this commit) | test adaptations; Godot client tests escalated (D-N1-4) |

### Independent review (red-team subagent, read-only, on the C1+C2 commits and the C3 working tree)

Verdict READY-AFTER-FIXES. Confirmed sound: every Shop sale path passes `judge_open` with the N1-I3
precedence; `Hours` normalisation and overlap completeness; additive-optional persistence and the N1-I8 branch;
`shop-stock` reads the same instant as the offers. Findings and disposition:

| # | Finding | Disposition |
| --- | --- | --- |
| R-1 MAJOR | C3 uncommitted; baselines untouched although Market Town genesis bytes move; AC-S35-8 unmeasured | C3 committed here; AC-S35-8 run on the final head (below); baselines **deferred to the primary** by the dispatch rule (report before re-recording a world golden) — open item, nightly-only |
| R-2 MINOR | N1-I7 refusal bound aggregate, interval count off by the day-0 morning, stdout parse could silently read 0 | Bound corrected to 3 × (nights + 1); every refused-buy line must parse its count (format change fails loudly); per-shop keying not possible from the run report — recorded limitation; the per-path refusal is proven in `closed.rs` |
| R-3 MINOR | CLI resume test cannot fail on behaviour | Its doc now states the discriminating part (a drift refusal would fail the resume) and points to `hours.rs`, which asserts the sale after the new closing time on a real save |
| R-4 MINOR | closed vs self-trade untested | Added to `closed.rs`: Felix's `propose-price` and `buy` offers are `shop-closed` at 23:00 and `PreconditionFailed` at 09:00 |
| R-5 NIT | byte-identity not golden-tested | Covered by the unchanged Social Café / Bodies Yard / Lakeside histories (their files untouched; the nightly baselines compare them) |
| R-6 NIT | MODULE_SPEC wording | Reworded |
| R-7 NIT | observation instant assumption | Stated in `offer.rs::shop_stock`'s doc |
| R-8 NIT | `haggle.rs` 564 lines (> 500 trigger) | Existing debt (558 on main); +6 lines here. No business-hours logic outside economy |

### Maintainability review (§7)

New: `hours.rs` (≈ 250 lines incl. tests, one concept); `sale.rs` +15, `haggle.rs` +6, `offer.rs` +6,
`component.rs` +20. No function over 50 lines added; no new abstraction beyond the `Hours` value type; ownership
unchanged (economy alone); kernel, clients and controllers untouched. Existing debt `haggle.rs` > 500 noted, not grown
materially.

### Discovery D-N1-4 — hosted Market Town begins at 00:00, when every authored shop is closed

```text
Previous assumption (§2.3):
  Closures are safe because every resident routine visits shops inside the hours.
Audit evidence:
  The targeted mineworld-cli run (below) failed ac5_takeover, ac7_many_players and milestone_c: each hosts
  Market Town and has a player buy at the café right after genesis or at the 2-day head — both midnight
  (instant 0 is local midnight, INV-TW-4). The buy offer is (correctly) refused `shop-closed`; the tests wait
  20 s for an available buy and fail. market_town, market_settlement, milestone_c and newsstand also failed on
  the test-side `shop-opened` mirror (`tests/market/mod.rs`), which refuses unknown fields by design.
Corrected understanding:
  Hosted play — tests and the packaged game — starts the town at midnight, so a player cannot buy anything
  for 5.5 world hours (≈ 27 wall minutes at scale 12). This is what Q-L1's hours mean; it was not visible in
  the residents-only audit.
Implementation consequence (bounded, test-only):
  - the mirror accepts the optional `hours` field (shape check stays strict);
  - `market::AlwaysOpen` — a scratch copy of Market Town with no `hours` lines — for the three default-run
    tests whose subject is takeover / many players / work-earn-buy, not hours. Their assertions are unchanged.
Not done (escalated): the Godot client acceptance tests that buy at the café in hosted Market Town at start
  (`client_3d_market`, `client_2d_interact`, `ac13_clients`, `demo_a`; `#[ignore]`d locally, run by the client
  lanes) will meet the same closed café. They belong to the active client lane (S34-e) and the packaged
  first-minute experience is a product decision (start the hosted day later, give the opening scene an
  open shop, or adapt the tests as above) — for the primary/operator.
Validation consequence:
  ac5_takeover 2/2, ac7_many_players 1/1, milestone_c 1/1, market_settlement 3/3 (+1 ignored), market_town 1/1,
  newsstand 6/6 PASS after the adaptations (local).
```

### Validation record (local, macOS arm64, 2026-11-27)

| Check | Head | Result |
| --- | --- | --- |
| `cargo test -p mineworld-economy -p mineworld-price-survey` | C2 tree | PASS (all targets) |
| `cargo clippy -p mineworld-economy -p mineworld-cli --all-targets -D warnings`, `cargo fmt --all --check`, `mineworld reference --check` | final tree | PASS |
| `shop_hours` 3/3 | C3 tree | PASS |
| Targeted `mineworld-cli` (21 targets touching Market Town, `--no-fail-fast`) | 4807b618 | 5 targets FAILED (D-N1-4); after the adaptations each of the 5 re-run PASS; the other 16 PASS (Godot client targets are `#[ignore]` locally — not run) |
| **AC-S35-8** (`needs_town::the_towns_sleep_at_night_and_eat_at_meal_times`, release, `--ignored`, 30 days × seeds 1/7/42, Market Town + Commute Town; projection ≤ 15 min, actual 924 s wall incl. slot wait and release build, 67 s test) | 4807b618 | **PASS** every row. Market Town: asleep 03:00 1000 ‰ (all seeds); 14:00 0 ‰; sleep 7.45 / 7.43 / 7.47 h; TV(food, ATUS) 275 / 275 / 273 ‰; longest famished 15.8 / 5.8 / 7.0 h. Later commits are test-only, so the measurement stands for the final head's behaviour |
| `scripts/baselines.txt [world market-town]` re-record | — | NOT DONE — deferred to the primary (dispatch rule); every Market Town key moves (genesis `shop-opened` bytes) |
| Packaged launcher night check (Gate2) | — | INCONCLUSIVE — prerequisite P1 |
| CI `fast` | 4807b618 | PASS (run 46081089009); re-run on the final head below |

### Coordinator rulings (2026-11-27, after the first hand-back)

1. Re-record `scripts/baselines.txt [world market-town]` in this PR (the genesis `shop-opened` change is the frozen
   design's own consequence); Market Town only; Lakeside was re-recorded by #553 and is not touched.
2. D-N1-4 (hosted day starts at 00:00 with every shop closed) goes to the operator; no product change here. The
   always-open test copies stay. **Follow-up for the client lane (S34-e):** the Godot acceptance tests that buy at
   the café in hosted Market Town at start will meet `shop-closed` — `tools/cli/tests/client_3d_market.rs`,
   `client_2d_interact.rs`, `ac13_clients.rs`, `demo_a.rs` (all `#[ignore]` locally; run by the client lanes).
   `market::AlwaysOpen` (in `tools/cli/tests/market/mod.rs`) is available to them if the operator keeps 00:00.
3. `fast` green on the final head.

### Market Town baselines re-recorded (coordinator ruling 1)

```text
Run:     ci_parity.py record --binary target/release/mineworld --root <root> --profile long, where <root> holds only
         worlds/market-town (+ entities, presentation/mineworld-default) as links into this checkout; commit 9af926e7
         (merge with main 29401a26); Darwin/arm64; 1 401 s in the shared heavy slot (projection ≤ 40 min, budget 60).
Write:   ci_parity.py baseline write <record> <copy of baselines.txt>; only the [world market-town] keys spliced back,
         with a reason line; Social Café, Bodies Yard and Lakeside untouched (Lakeside: #553).
Check:   baseline check <record> scripts/baselines.txt — market-town's 6 keys equal; the only failures are the three
         worlds the single-world record does not carry (expected).
```

| Key | Before (milestone F) | After (9af926e7) |
| --- | --- | --- |
| summary-300 | 399 4a7c7a1d… | 401 bb959647… |
| summary-1000 | 1123 4448c2d4… | 1127 8d27edd3… |
| summary-300s | 398 5074ddc0… | 400 c43a95c0… |
| facts-300s | 6135 460d89d0… | 6032 0bb704b8… |
| journal-300s | 4540 c19fd50b… | 4170 b825bc57… |
| sealed-300s | 528 e97e994d… | 263 9e90d6b6… |

The before keys date from the milestone F re-record (2026-10-20); the after keys therefore also absorb main's
Market Town drift since then (summaries +2/+4 lines), not only N1's genesis `shop-opened` bytes. A full-world
nightly record on the merged head is the cross-check.

### Review minors (coordinator, red-team `_reviews/pr552.md`)

- **N1-I7 now met, not just absent**: `shop_hours.rs::residents_meet_a_lunchtime_closure_and_neither_buy_there_nor_haunt_it`
  — a copy of Market Town with the café closed 12:00–14:00: 8 entries into the closed café over 2 days, 0 sales
  and 0 `shop-closed` refusals inside the window, 16 café sales 14:00–22:00. Mutation M5 (drop `judge_open`):
  the test fails ("sold at shop 2 at 49502 s, outside its hours") — caught. `motivated` buys nothing in Market
  Town on base content either (7 days: no `buy`, no `eat` request), so its run remains a bound on walks; a
  purchase cannot be attempted by that model without changing it (out of N1 scope).
- **Helpers deduplicated**: `tools/cli/tests/market/mod.rs::MarketCopy` (`always_open`, `with_hours`) replaces
  `AlwaysOpen` and `shop_hours.rs`'s private copy-and-strip.
- **Locale**: `reason.shop-closed` in `clients/shared/settings/locale/en.po` ("the shop is closed") and
  `zh_Hans.po` ("店铺已打烊"); `messages.pot` regenerated (`MINEWORLD_WRITE_POT=1`); `mineworld-acceptance
  --test client_text` 6/6 PASS.

### Operator ruling on D-N1-4 (2026-11-27, relayed by the coordinator) — audit and material stop

Ruling: "新世界早上 7 点开始" — new worlds begin at 07:00, not 00:00.

**Audit (head 125b98fc, main a8a74659).** The genesis instant is not authored anywhere: every host hard-codes
`WorldTime::EPOCH` (instant 0 = local midnight of the calendar epoch, `INV-TW-4`) — `tools/cli/src/run.rs`
`Begun::new` (l. 318), `server/src/host/config.rs` `HostConfig::epoch` default (l. 96, used by `serve.rs` to create
a save and to load an ephemeral world), `tools/cli/src/create.rs` l. 72, and the inspection paths
(`main.rs` l. 776, `biography.rs`, `packs.rs`, `perceived.rs`). Resume reads genesis from the save. `run --days N`
already counts from genesis (`run.rs` l. 132). `world.yaml`'s only time key is `hosting: time_scale` (host
pacing, "never seeded, ignored by `run`", `ARC-69` item 9), so a start time does not belong there.

**Smallest correct change (proposed, not implemented).**
1. A new optional `world.yaml` key `begins: "HH:MM"` (schedule's `TimeOfDay` convention), the time of day of
   genesis on day 0; absent = `"00:00"` (today's behaviour, no world's bytes move). Owned by `worldpack`
   (`format.rs` field + `WorldPack::begins()`), documented in `MODULE_SPEC.md` beside `hosting:`, recorded as a
   new ARC (amends no invariant: midnights stay multiples of 86 400 s; genesis simply need not be one).
2. Every host that **creates** a world uses `pack.begins()` instead of `WorldTime::EPOCH`: `run.rs`,
   `serve.rs`/`HostConfig` (when creating or loading ephemeral), `create.rs`; the inspection paths likewise.
3. Content: `worlds/{market-town,social-cafe,lakeside}/world.yaml` `begins: "07:00"`; Bodies Yard unchanged.

**Why this stops here (contract N1 MATERIAL STOP; CLAUDE.md §2.2, §3.1).**
- It adds a public `world.yaml` contract outside N1's frozen scope (N1-C1…C3 touch economy and Market Town
  organizations only); a contract addition needs its design text and ARC before code.
- Ripple measured by audit: three worlds' baselines move (three long records ≈ 25 min each ≈ 75 min, above N1's
  60-min heavy budget); tests that assert a quiet first hour after genesis (`ac5_takeover`, `milestone_c` via
  `fixture::assert_quiet_in(…, 0, 3600)`) break at 07:00, where Bob, Grace, Carol and Ivan's routine boundaries
  fall, and need reworking, not a constant change; AC-S35-8 must be re-measured (the first night is gone);
  ~30 test files load worlds at `EPOCH` in memory and need a per-file audit; the Godot café-buy tests
  (`client_3d_market`, `client_2d_interact`, `ac13_clients`, `demo_a`) are the ruling's beneficiaries and must be
  run in a heavy slot.
- Recommendation: merge N1 as is (its `MarketCopy` test copies keep the three Rust tests honest), and open a
  small follow-up PR "world start time" with the design above (one commit for the key + hosts, one for content
  and goldens), budget ≈ 2 h heavy. Alternative, if the operator wants it inside #552: approve the scope and a
  2 h heavy budget, and this session continues.

### Merge with main a2be71e4 (after #553 Lakeside, #555 FE-a needs v2, #544 …) — Market Town re-recorded again

Merged without conflict (no rebase). Other worlds' sections are main's. `[world market-town]` re-recorded from a
long-profile record of the merged tree (`ae34d202`, Market Town only, 1 674 s) and written by `baseline write`:

| Key | #552 before merge | After merge (ae34d202) |
| --- | --- | --- |
| summary-300 | 401 bb959647… | 401 65c42943… |
| summary-1000 | 1127 8d27edd3… | 1127 0264a9eb… |
| summary-300s | 400 c43a95c0… | 400 a357530d… |
| facts-300s | 6032 0bb704b8… | unchanged |
| journal-300s | 4170 b825bc57… | unchanged |
| sealed-300s | 263 9e90d6b6… | 263 6867e8ea… |

On the merged tree: economy `closed` 5/5, `hours` 4/4; `shop_hours` 4/4; `ac5_takeover` 2/2, `ac7_many_players`
1/1, `milestone_c` 1/1, `newsstand` 6/6 PASS. AC-S35-8 re-run (release, `ae34d202`): PASS, every row as before
(Market Town 1000 ‰ at 03:00, 0 ‰ at 14:00, 7.45 / 7.43 / 7.47 h, TV 275 / 275 / 273 ‰; Commute Town PASS).

## 12. N2 execution ledger (implementation session, 2026-11-27)

```text
Branch / worktree:   mvp0/pr-nightlife-n2 @ /Users/yuema137/mineworld-worktrees/impl-nightlife-n2 (the contract names
                     feat/nightlife-n2-sleep-advance; the primary's dispatch named mvp0/pr-nightlife-n2 — D-N2-0, name only)
Implementation base: origin/main bdb83457 (N1 #552, launcher P2 #549 and P3 #561 merged; #564 world start 07:00 not
                     depended on). Seams re-audited on bdb83457: sleep, rule/motivated controllers, server runtime/hosted/
                     seats/admin/control/delivery/world, persistence host journal, systems/installed — as §2.2 states.
Handoff:             .structured-coding/plans/mvp0/handoff-nightlife-n2.md
Cargo target:        /Users/yuema137/mineworld-worktrees/_targets/nightlife-n2 (own; heavy commands through with-slot.sh)
```

### Decisions and bounded deviations

| ID | Question | Evidence | Decision |
| --- | --- | --- | --- |
| D-N2-0 | Branch name | dispatch from the primary | `mvp0/pr-nightlife-n2`, one worktree, one session |
| D-N2-1 | Name and unit of the morning parameter | `parameters!` admits integer bounds only (`sdk/rust/src/interactions/section.rs`) | `morning_minute: u16 = 420, 0 ..= 1439` (minute of the day), as N2-I2 states |
| D-N2-2 | Version bump for the morning (N2-I14) | `parameters!`'s full struct has no serde default, so an old resolved payload lacks a field; `ARC-370` bumped `needs` the same way; `kernel/src/system.rs` "a system that changes the schema of state it owns raises this number" | `SleepSystem` 2, `CONFIGURED_SCHEMA_VERSION` 2, `COMPONENT_SCHEMA_VERSION` 2. Only Market Town configures `sleep` (Commute Town, a test world, installs it unconfigured: its saves meet `PersistedSystemOutdated`) |
| D-N2-3 | Pre-N2 save fixture | the design asks for a Market Town fixture; `tools/cli/tests/fixtures/pre-fe-save` is the precedent | a real genesis-only Market Town save from the base binary (999 424 bytes, `fixtures/pre-n2-market-town/SOURCE.md`) |
| D-N2-4 | Rhythm regression's form ("same decision trace as the base") | a two-day run is 4 s; the base binary was built before any edit | a frozen copy of the base build's two-day summary (`run-seed-7-days-2.txt`) compared line for line by `sleep_until.rs`; the journal-row identity (below) recorded once as evidence; the 300-day `journal-300s` key is the long cross-check |
| D-N2-5 | Refused `sleep-until` offers | validate's order | refused offers are parametric (no `at`); only an available offer carries a payload |

### N2-C1 evidence (local, macOS arm64, 2026-11-27)

| Check | Result |
| --- | --- |
| `cargo test -p mineworld-sleep` (all targets incl. new `until` 10/10) | PASS |
| `cargo test -p mineworld-rule-controller -p mineworld-motivated` | PASS (50 + 90 lib tests) |
| `cargo test -p mineworld-cli --test sleep_until --test sleep_sections --test needs_sections --test food_energy --test shop_hours` | PASS (2, 2, 6, 7, 4) |
| `cargo clippy -p mineworld-sleep -p mineworld-rule-controller -p mineworld-motivated -p mineworld-cli --all-targets -D warnings`; `cargo fmt --all --check`; `mineworld reference --check docs/reference` | PASS |
| Rhythm regression, base `bdb83457` binary vs this tree: `mineworld run worlds/market-town --headless --seed 7 --days 2 --save …` | summary identical except the save path, wall time and fact fingerprint (the configuration fact); **all 6 458 journal rows after genesis byte-identical** (`sqlite3 … select revision, at, action_id, hex(entry) from journal where revision > 1`, sha256 `da2935d1…` both) |
| Counterfactual (F1): the same run with `sleep-until` removed from `DECIDED_BY_NAME` | differs: revision 6 564 vs 6 459; `sleep-until accepted 6`, `sleep accepted 27` (base 33) — the exclusion is load-bearing |
| Mutation M1: `may_sleep` skips the place rule for `sleep-until` | `until.rs` 2 tests FAIL (caught) |
| Long records (baselines re-record + AC-S35-8) | NOT RUN yet — once on the final head (§7, contract budget) |
