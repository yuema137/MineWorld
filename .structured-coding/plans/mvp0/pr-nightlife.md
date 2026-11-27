# Nightlife — shop hours (PR N1) and single-player sleep-until advance (PR N2)

```text
Design revision:        revision 1 (2026-11-27), design lane L2 (Claude, design only)
Lifecycle:              DRAFT — FREEZE-READY (NOT FROZEN; no implementation authorized)
Approved by / evidence: PENDING — operator answers to §0 Q-L1 … Q-L5
Audit base:             origin/main 7718d952 (re-audited; see §2.1)
Start condition (N1):   this design FROZEN by the operator
Start condition (N2):   FROZEN, N1 merged, and the launcher Market Town prerequisite (§1.3) merged
Execution contract:     pr-nightlife-contract.md (two filled contracts, N1 and N2)
Parent:                 operator-playable-mvp-20261123.md §§2–5 (requirements); overall.md
Supersedes as authority: the recovered Codex drafts listed in §1.4 (evidence only, not ledgers)
```

## 0. Decisions for the operator (plain language)

Answer each with "as recommended" or a change. Nothing below is frozen until you say so.

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
- **ARC-69 item 5** (fast-forward is single-player only; `--solo` selects no code path, widens one permission): kept.
  N2's advance is a second permission `--solo` widens; it is not a time-scale change and adds no rescale.
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

### 1.3 Prerequisite owned elsewhere

The launcher must start Market Town with its own save and matching depiction, and show at least one non-Alice resident
walking home, sleeping, waking and commuting. On 7718d952 that fix is still in the active client-stack recovery lane
(ready-queue 2026-11-27 header). It blocks **N1-C3's packaged lifecycle acceptance and all of N2-C4**, nothing else.

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

## 2. Audit (7718d952)

### 2.1 Base

The drafts audited `b457a94d`. `git diff --stat b457a94d 7718d952` over `systems/{sleep,economy,schedule}`, `server`,
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
| `systems/schedule/src/time.rs::TimeOfDay` | strict `"HH:MM"`, seconds since midnight, `of(at) = at mod 86 400` | reused by economy as a **type-only** dependency (`ARC-28` precedent: economy already depends on employment and happenings for types) |
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
| `server/src/runtime/{pacing,control}.rs` | HostClock: anchor, pause, resume, `rescale_at` | add a forward re-anchor at the reached instant, scale unchanged |
| `clients/shared/host/{host_clock,world_tab}.gd`, `clients/shared/HOST.md` | the one shared authenticated admin transport; displayed state only from `welcome`/`clock` frames (`INV-WC-1`) | advance request/cancel/status added here, once, for both clients |
| `clients/2d/scripts/{app.gd,hud/readers.gd,hud/panels.gd}`, `clients/3d-spike/scripts/slice/needs_hud.gd` | 2D asleep panel; 3D `SliceNeedsHud` dims the view while asleep. The 3D client directory is `3d-spike` | show advance progress/Wake; no rule computed |
| `tools/cli/src/hosted.rs` | composition root that builds hosted controllers (`PacedSeat`: walking consult period = stride at scale) | registers the sleep stop adapter |

### 2.3 Residents' night on 7718d952

Market Town installs `schedule`, `sleep`, `needs`, `economy`, `employment`, `consumption`, `calendar`. Routines:
Alice café 05:30 (punctual) → park 14:00 → apartments 18:00; Bob café 07:00 → station 11:00 → store 16:00 →
apartments 20:00; Carol store 07:00 → café 12:00 → apartments 16:00. The proposed hours keep every routine's
shop visit inside opening hours. Hunger rises overnight; S35 has no forced consequence (Q-S35-3), so the risk is
behavioural (walking to a closed shop), not a starvation rule.

## 3. Invariants proposed for freeze

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
  are unaffected. Precedence: `shop-closed` is judged first in `judge_exchange` (after seller resolution), so a closed
  shop never reports `insufficient-funds` or `out-of-stock` instead.
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

### N2 — sleep-until and solo advance

- **N2-I1 Sleep owns sleep.** Only `SleepSystem` creates `Asleep`, the `sleep` Process, `alarm-set` and `woke`.
  `sleep-until { at }` shares `sleep`'s admission, place rule, Interaction-List section, process, alarm privacy
  (holder-only `alarm-set`) and four ways to end. It is a world rule in every world; `--solo` changes nothing in it.
- **N2-I2 Morning.** A new sleep parameter, the morning as minute of day (0…1439, default 420 = 07:00), regional like
  the others. The offered target is the first instant at that time of day strictly after `now + shortest`; it is
  offered complete (exact `at`) only when `at − now ≤ longest` and the place allows sleep, else refused as validate
  would refuse it. Validate admits any `at` with `shortest·60 ≤ at − now ≤ longest·60` seconds (so 23:00:12 → 07:00 is
  exact), refusing out-of-range with `sleep-for-out-of-bound` and a past or present `at` likewise.
- **N2-I3 Advance is host state.** The advance is a host operation, never a fact, journal input or revision. It is
  permitted only when: the world was started `--solo`; the request is admin-authenticated; exactly one decision
  session is seated and it holds the named seat; every other seat is a hosted in-server controller (Q-L4 A); the
  seat's Person is asleep through a sleep that is identified at start; the clock is running; no other advance runs.
  Otherwise a typed refusal before any work. Shared and both-client hosts always refuse.
- **N2-I4 Generic server.** The server knows an advance *stop adapter* (registered by name at the composition root),
  not sleep. The adapter answers, from authoritative state: the horizon (the sleeper's alarm) and whether the
  identified sleep still holds. The admin route names no instant (`ARC-40`).
- **N2-I5 Exact-due order.** Repeatedly `t = min(kernel.next_instant, earliest hosted consult, horizon)`; advance the
  kernel to `t` (existing queue/reducer order); consult every seat due at `t` in the existing (instant, seat) order
  with `now = t`, through the ordinary perception/submit/answer path; recompute due seats after the instant; after the
  instant is complete, ask the adapter. Stop at the **first completed instant** at which the identified sleep no
  longer holds; never go past an earlier wake. Scale `S`, decision periods, stride periods and speech windows are
  unchanged; no rescale. Visual frames may be coalesced; durable history, event perception, controller observations
  and answers are never dropped.
- **N2-I6 Oracle parity.** For the same save, seed, scale and bindings, the advanced night equals the ordinary host
  driven by an injected wall clock that ticks at every due instant: same intents, events and order, positions, sleep
  intervals, needs, wages, stock, purchases and final instant.
- **N2-I7 Responsiveness.** Work in slices of ≤ 128 completed instants or ≤ 10 ms CPU, whichever first; between slices
  service cancel, disconnect, shutdown and status. Implementation constants, not gameplay time.
- **N2-I8 Durability.** World actions persist through the existing persisted dispatch/advance before progress is
  published; host records `advance-started { operation, seat, adapter, prior scale }` before the first instant and
  `advance-ended { operation, outcome, at }` after the last. On startup, a started-without-ended operation gets
  `advance-ended { interrupted }` appended, the world resumes at its durable instant with ordinary pacing
  (`ARC-69` item 8), and nothing resumes automatically. A persistence failure fail-stops; a partially reduced in-memory
  advance is never retried and never reported as morning.
- **N2-I9 Idempotence.** The client generates the operation id. A repeated id with the same payload returns the
  recorded result without advancing; a repeated id with a different payload is refused.
- **N2-I10 End states.** Outcomes: `woke` (alarm or other wake — the adapter's reason), `cancelled` (finish the
  current instant, stop, then submit a normal `wake` for the sleeper), `disconnected`/`stopped` (stop at a boundary;
  the night is never finished in the background), `interrupted` (restart), `failed` (fail-stop). After any outcome
  the HostClock is re-anchored at the reached instant with `S` unchanged and no wall time folded in. While an advance
  runs, `/admin/clock` edits and decision joins/takeovers are refused.
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
   - [ ] Regenerate the schema; document defaults and refusals in the pack's module docs.
4. **Validation.** [ ] authoring+seeding integration for absent, `[]`, ordinary, overnight (22:00–06:00), equal
   endpoints, overlap after midnight, ninth interval, malformed `"7:00"`; [ ] save → reopen → equal `Shop` terms;
   [ ] a pre-N1 save fixture (existing economy persistence fixtures) decodes as always-open.
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
   - [ ] Decision record; amend `operator-playable-mvp-20261123.md` §3 "Shop hours" row to point here.
4. **Validation.** [ ] scenario: a purchase before close, `shop-closed` after, breakfast after opening, at least one
   resident home → sleep → wake → work; [ ] N1-I7 counts from the event log; [ ] Social Café unchanged (removal
   witness, absent hours); [ ] save/reopen reproduces hours; [ ] packaged launcher night check when §1.3 has landed
   (Gate2, one heavy slot, ≤ 10 min).
5. **Acceptance.** Sales close at authored times; residents eat across the night/morning; S35 AC-S35-8 rhythm bounds
   are not relaxed — a violation is corrected in content with evidence or escalated.
6. **Failure/edges.** Closure that starves a routine is a content defect; absence of the launcher prerequisite blocks
   only the packaged check, which is recorded `INCONCLUSIVE — prerequisite`, never PASS.
7. **Commands/evidence.** Exact CLI scenario command recorded before running; targeted `mineworld-cli` tests
   (`market_town`, `market_settlement`); CI on exact head is the full gate. NOT RUN.
8. **Boundary.** Completes N1's checkpoint. [ ] independent review of food access and removal witnesses;
   [ ] maintainability review (§7); [ ] staged diff/ledger before commit.

## 6. PR N2 — sleep-until and single-player advance

Branch `feat/nightlife-n2-sleep-advance`, fresh session, base = main after N1 and §1.3 merged (re-audit the seams).
Integration checkpoint: in a `--solo` Market Town save, the player sleeps until 07:00 and the host runs a multi-event
night faster than wall time with oracle parity, stopping early on an earlier wake; cancel, kill/reopen and repeat-id
behave as N2-I8…I10 in both reference clients.

### N2-C1 — `sleep-until` and the morning (sleep pack only)

1. **Goal.** An exact, server-computed sleep target, before any host machinery depends on it.
2. **Scope.** `systems/sleep/src/{action,interactions,offer,system,lib}.rs`, sleep docs/README section table,
   `worlds/market-town/configure/sleep.yaml`, `docs/reference` regeneration, `docs/decisions/ARC-<next>.md`
   (sleep-until), step-35 dated note on Q-S35-9; offer-list tests/goldens that enumerate Market Town offers
   (`tools/cli/tests/{shown_parity,client_2d_interact,ac13_clients,…}.rs` — re-audit). Controllers unchanged
   (they read `sleep` by type).
3. **Implementation.**
   - [ ] `SleepUntil { at }` action; `providing::<SleepUntil>()`; ActionDecl so the section's rules apply to it.
   - [ ] Morning parameter (N2-I2); old resolved sections decode with the default; version pin test updated.
   - [ ] Shared admission helper for `sleep` and `sleep-until` (one ordered list; the alarm is the only difference).
   - [ ] Complete offer with the exact target; refused offers mirror validate.
4. **Validation.** [ ] 23:00:12 → 07:00:00 alarm exact; [ ] 06:59 and 15:00 out of bound; [ ] past/now `at`;
   [ ] forbidden place; [ ] `Busy` while asleep; [ ] regional morning override; [ ] alarm holder-only in disclosures;
   [ ] all four wake ways end a `sleep-until`; [ ] save mid-sleep → reopen → alarm fires; [ ] old save decodes.
5. **Acceptance.** No client arithmetic exists; the offered `at` validates unchanged at dispatch or is refused as stale.
6. **Failure/edges.** Overflowing `at` refused; a world without `sleep` offers nothing; shared worlds offer it too.
7. **Commands/evidence.** `cargo test -p mineworld-sleep`; targeted `mineworld-cli` offer tests. NOT RUN.
8. **Boundary.** World rule only, no server. [ ] independent review of admission order and privacy; [ ] diff/ledger.

### N2-C2 — advance contract, lifecycle and durable identity (no loop)

1. **Goal.** Authenticated, idempotent, recoverable operation and its refusals, reviewable before acceleration.
2. **Scope.** `server/src/admin.rs` (`POST /admin/advance`, `GET /admin/advance/{operation}`, `POST
   /admin/advance/{operation}/cancel`), `server/src/host.rs`, new `server/src/runtime/advance.rs` (state machine and
   `AdvanceStop` trait), `server/src/seats.rs::join` refusal, `server/PROTOCOL.md` §§4, 11, `persistence/src/host.rs`
   (lifecycle records), `tools/cli/src/hosted.rs` (register `sleep` adapter reading `mineworld_sleep::Asleep`),
   `docs/decisions/ARC-<next>.md` (advance) with a dated note on `ARC-69`.
3. **Implementation.**
   - [ ] Typed request `{ operation, seat, until: "<adapter name>" }` — no instant; typed refusals: `not-solo`,
         `unauthenticated`, `seat-mismatch`, `other-controller-connected`, `not-asleep`, `paused`, `busy`,
         `unknown-adapter`, `operation-conflict`.
   - [ ] Journal records `advance-started` / `advance-ended`; legacy journals decode; recovery appends `interrupted`.
   - [ ] Idempotent status by operation id.
   - [ ] Until C3 wires the loop, an admitted request records `advance-started`, moves no time, and ends
         `stopped` with reason `not-yet-executable` — never a false `woke`. C3 removes that branch.
4. **Validation.** [ ] each refusal; [ ] repeated id same/different payload; [ ] shared host and two-session host
   refused; [ ] crash between records → reopen → `interrupted`, no re-advance; [ ] `verify` coherence still passes.
5. **Acceptance.** Rejected requests cause no action and no record; the server crate names no sleep type.
6. **Failure/edges.** Journal write failure fail-stops; unknown record kind is a named recovery error.
7. **Commands/evidence.** `cargo test -p mineworld-persistence --test host_journal`; `cargo test -p mineworld-server`
   filtered to admin/host/advance; `cargo test -p mineworld-cli --test <hosted target>`. NOT RUN.
8. **Boundary.** [ ] independent auth/privacy/recovery review; [ ] diff/ledger.

### N2-C3 — exact-due execution

1. **Goal.** Execute every intervening decision and process, stopping at the first completed wake instant.
2. **Scope.** `server/src/runtime/advance.rs`, a small extraction in `server/src/runtime.rs` (one-instant consult
   shared with `consult`), `server/src/{hosted,seats}.rs` (earliest due without mutation), `runtime/pacing.rs`
   (re-anchor), runtime tests.
3. **Implementation.**
   - [ ] **First**, freeze the ordinary-host oracle trace (injected wall clock ticking at every due instant) as a
         fixture from the current code; never compute expectations with the new loop.
   - [ ] Loop per N2-I5; slices per N2-I7; durable-before-publish; cancel ⇒ normal `wake`; re-anchor per N2-I10.
   - [ ] Ordinary `tick_at` semantics unchanged (its jitter-skip behaviour remains).
4. **Validation.** [ ] oracle parity (N2-I6) on a Market Town night with strides, schedule boundaries, wages, a
   purchase and same-instant ties; [ ] early wake at 02:00 by talk stops there; [ ] alarm at target; [ ] mutation
   checks: a jump to morning and `now = horizon` must each fail parity/early-wake; [ ] cancel/disconnect/shutdown
   between slices; [ ] scale unchanged; [ ] fault mid-advance not retried.
5. **Acceptance.** Visited instants equal the oracle's, in order; final instant = first completed wake instant;
   cursor never ahead of durable history.
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
   `clients/3d-spike/scripts/slice/needs_hud.gd`, launcher solo credential plumbing (re-audit `tools/launch`).
3. **Implementation.** [ ] `Sleep until 07:00` from the offer; after success, request advance with a fresh id;
   [ ] progress/asleep state from frames only; Wake ⇒ cancel; [ ] reconnect polls status by id, never resubmits;
   [ ] named "unavailable" when refused (shared host, other controller).
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
- Local runs are owning-crate targets only; CI on the exact final head is the full gate (`R-LOAD-1`). No 30/300-day
  campaigns. Record local and remote results separately.
- Every ready-PR review records findings on technical debt, module boundaries, readability, robustness, extensibility,
  oversized functions/files (`runtime.rs` 509, `haggle.rs` 558, `sleep/system.rs` 356 are watched), coupling.

## 8. Corrections against the recovered drafts

| # | Draft said | Corrected here |
| --- | --- | --- |
| C1 | Audit base `b457a94d` | Re-audited on `7718d952`; relevant paths unchanged (§2.1) |
| C2 | Closure "in `judge_exchange`" guards every path | `haggle.rs::judge_proposal` bypasses `judge_exchange`; it calls the closure judgement explicitly |
| C3 | Food access only "audited" | Rule fetch band step 7 walks to shops; closed shops are now disclosed `stocked: []` (N1-I6) and walks/refusals are bounded (N1-I7); motivated controller audited |
| C4 | Advance bound to "target" | Admin surface must name no instant (`ARC-40`); horizon comes from the sleeper's alarm via the adapter |
| C5 | `sleep-until` framed as single-player | It is a world rule in every world (`--solo` selects no code path, `ARC-69` item 5); only the advance is solo |
| C6 | Morning in a new "optional minute-of-day configuration" | A `SleepParameters` entry in the existing section; old sections decode with the default; offer goldens re-recorded |
| C7 | Economy depends on schedule for `TimeOfDay` (only option) | Kept as type-only (`ARC-28`); sleep restates `DAY` like calendar (`INV-TW-4`) |
| C8 | "External/model sessions are rejected" without consequence | Conflicts with the dialogue lane's LM-Alice profile; surfaced as Q-L4 |
| C9 | N2 as three commits mixing contract and loop | Four commits: sleep rule, lifecycle, loop, clients — each independently reviewable; oracle frozen before the loop |
| C10 | S21 "doze" and HOST.md references unreconciled | Reconciled in §0; HOST.md fixed in N2-C4; S21 may reuse the operation |
| C11 | Launcher prerequisite assumed landing | Still in the active client-stack lane on `7718d952`; blocks only packaged checks |

## 9. Evidence and readiness

Design-only lane. Source audit on `7718d952` (paths in §2.2). No production edit, build, test or model call was made.
Deterministic/runtime/UI checks: NOT RUN. Independent red-team review of this design: PENDING (primary assigns).
Ready for operator freeze once §0 is answered; implementation is not dispatchable until the header says `DESIGN
FROZEN` with the operator's date and the contract's endpoint sources are filled from that answer.
