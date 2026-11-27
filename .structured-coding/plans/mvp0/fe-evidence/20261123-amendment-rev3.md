# Playable-demo nutrition and economy amendment — 2026-11-23

Status: **DRAFT FOR REVIEW — NOT DESIGN FROZEN.** Design only. No implementation, tests, paid calls, public publication, migration or reset authorized by this document. Intended for reconciliation into `.structured-coding/plans/mvp0/operator-playable-mvp-20261123.md`, not a competing permanent authority. This temporary proposal does not amend existing frozen contracts until operator approval.

## 1. Requirement and recommendation

For a newly created Market Town, the configured visitor begins with USD 10,000, purchases realistically priced food/drink/snack servings, consumes one inventory unit per action, and sees authoritative food energy replenish. The example's food-energy use is **2,000 kcal per 86,400 seconds of world time**, configurable per world and optionally by actor class. Other worlds choose their own policy, currency, endowments, prices and optional packs.

**Recommend extending the existing optional `needs` pack with a closed, typed food-energy hunger model.** Retain its unitless hunger model for authored worlds that want the existing behavior. A person has one authoritative hunger state, represented by one model variant; there are never independently ticking kcal and hunger stores. Energy is an authored game resource, not a clinical physiological prediction. No activity, body mass, macronutrient, illness, starvation, forced sleep, speed or death rules are added.

The proposed bar is a bounded **food-energy reserve**, not “calories eaten today,” a metabolic body store, or a recommended food allowance. Full reserve equals one configured day's use. A daily intake diary is outside this amendment; do not label reserve or remaining capacity as daily intake. Food energy decreases continuously across world midnight without resetting. Sleep does not pause this energy use; the old asleep hunger rate applies only to the unitless model. Tiredness and sleep retain their current contracts.

## 2. Evidence and source audit

Read-only checkout: `/Users/yuema137/MineWorld`, branch `main`, HEAD `b457a94d9a1ace6a39005de683cc393886e19288`. Root had existing modified/untracked operator and coordination files; this lane made no repository writes. Source observations are of that dirty working tree, not proof of a clean historical SHA or delivered export identity.

Read `CLAUDE.md` in full; Structured Coding routing/resources for design, project standards; operator playable amendment in full; S35 needs PR contract in full; S35 requirement/removability/consequence/acceptance clauses; actual module contracts and relevant source below. Structured Coding phase is **design amendment**, with a future fresh execution session after freeze. No heavy-job slot was acquired or used.

| Audited owner/source | Established behavior; design implication |
| --- | --- |
| `systems/needs/src/{component,level,section,interactions,event,system,disclose}.rs` | `Needs` and `Nourishment` schema v1; hunger 0–1000 deficit; awake +120/h, asleep +30/h; nourishment is arbitrary deficit reduction. Current coffee has 0, apple 200, sandwich 650. These numbers are not kcal and must not be converted by relabeling. |
| `.structured-coding/plans/mvp0/pr-s35-nb-needs.md`, `step-35-needs-and-sleep.md`, `docs/decisions/ARC-284.md`, `ARC-285.md` | Needs owns both person and item forms of one `needs:` section. Lazy integer state, holder-only needs, no periodic Process or post-genesis needs facts. Q-S35-2 excludes thirst; Q-S35-3 excludes forced consequences; Q-S35-7 preserves Social Cafe/Bodies Yard without needs. |
| `docs/MODULE_SPEC.md` §§4.1–4.2; generated needs/economy schemas | Existing section, parameters, conditional dependencies and disclosure are public pack contracts. Amend these explicitly before changing their implementation. |
| `systems/consumption/src/system.rs` | Stateless `eat`/`drink`, real inventory validation; one action emits inventory's checked `ItemsConsumed`, count 1. Category food versus drink chooses the action. No nutrition or hydration rules belong here. |
| `systems/needs/src/system.rs::fed` | Only needs consumes nourishment after `ItemsConsumed`; no direct inventory or money mutation. Preserve this path. |
| `systems/economy/src/{configuration,currency,section,money}.rs` | Currency metadata is configured and persisted; amounts are integer minor units. Wallet genesis is `Funded`; shops author prices by item; references are not charged prices. `PRICE_MAX` is 10^12; an offered price is positive. No new currency subsystem is needed. |
| `worlds/market-town/configure/{economy,needs}.yaml`, `people/visitor.yaml`, `organizations/{cafe-company,corner-store}.yaml`, item YAMLs | USD exponent 2 already exists. Visitor is 200000 cents ($2000), not requested $10000. Cafe prices include sandwich 175, soup 160, coffee 80 cents; store milk 45 cents but carton size is unspecified. Existing comments explicitly size cheap food for a 300-day horizon; they do not establish retail provenance. |
| `worlds/market-town/people/*.yaml` | Most nonworkers start 200000 cents; Alice/Felix start 20000; wages include 120/200 cents per hour. Higher realistic prices can invalidate old food-affordability assumptions even after the visitor change. Production is configured in job sections. |
| `worldpack/src/load.rs` | Configuration facts precede section facts, but `initial_facts` only collects emissions against the unstarted world. `Seeding::world()` does NOT contain reduced configuration. Model/content checks that need effective parameters must run at the owning genesis reducer after configuration has reduced, not during ordinary section seeding. No kernel/global configuration manager is needed. |
| `persistence/src/{replay,error}.rs`, `tools/cli/src/serve.rs` | Composition/system-version mismatch is refused; older saved system version is `PersistedSystemOutdated`, with no migration. Host checks saved genesis configuration. Do not promise an existing save will gain the new model or load automatically under a bumped system version. |
| `clients/protocol/mineworld/needs.gd`, `clients/3d-spike/scripts/slice/needs_hud.gd` | Current views expose the 1000 deficit scale and draw that bar, not kcal. Shared reader and both clients need an explicit energy record/row. No world-name branch is justified. |
| `cognition/rule-controller/src/needs.rs`, `cognition/motivated/src/considerations.rs` | Rules rank food by disclosed nourishment and use normalized hunger, not raw item files. Keep an owner-derived motivation projection; expose kcal separately for truthful display and agent use. |

Preserve the existing evidence; none of the above is a claim that a new gameplay run passed. The coordinator's independent reviewer reviewed revision 1 with five freeze blockers; revision 2 below records the amendments. No independent re-review of revision 2 is claimed. This lane spawned no children.

## 3. Alternatives and ownership decision

| Alternative | Assessment |
| --- | --- |
| Relabel 0–1000 hunger/nourishment as kcal | Reject: units, daily use, portion and bar direction would all be false. |
| Replace every world with a kcal model | Reject: unnecessarily removes the useful existing unitless model and changes unrelated worlds. |
| Separate optional food-energy System Pack | Plausible for a future distinct physiology/battery domain, but today it either adds a second food need or needs a new cross-owner projection/disable contract with needs. Both subscribe to the same consumption fact; coordinating hunger words and all controllers becomes larger than this requirement. Revisit if an independently useful second energy domain appears. |
| **Closed model choice inside needs** | **Choose:** existing owner, item section, fact subscription, persistence and disclosure. Small pure `food_energy` arithmetic module inside the pack; no registry of arbitrary needs, generic resource framework, restorer catalog or new pack type. |

Economy remains the sole money/price owner; inventory remains the sole holdings/stock owner; consumption remains the interaction; needs owns food energy and derived hunger. World Pack authors instances and policy. Controllers submit existing intents. Kernel/server/clients must not know Market Town, USD 10000, a meal name or its calories.

## 4. Proposed typed authoring contract

Exact proposed field names below are for review, not claims of current support. Unknown, conflicting or inapplicable keys fail by name and file location; do not silently ignore old knobs in a new mode.

`configure/needs.yaml` adds base-only `hunger_model: unitless | food-energy` (default `unitless`) and actor-scoped `energy_kcal_per_day` (typed positive whole kcal/day, bound 1–100000; only legal when the base model is food-energy). Food-energy requires an explicitly authored base daily use; Market Town writes 2000. Existing tiredness/restorer/threshold parameters remain. Existing `hunger_per_hour` and `hunger_asleep_per_hour` may be authored only in unitless mode; explicit use in food-energy mode is rejected. Their compiled unitless defaults do not take effect in food-energy mode. In unitless mode an authored energy-use key is rejected. Actor overrides change daily use and therefore reserve capacity; they cannot change the world model. Use existing fixed actor classes, not new Person attributes or live mutation.

```yaml
# Proposed Market Town configure/needs.yaml (not accepted by current code)
parameters:
  - hunger_model: food-energy
    energy_kcal_per_day: 2000
    tiredness_per_hour: 60
    rest_per_hour: 125
    thresholds: [400, 600, 900]
    restore: { hunger: [eating], tiredness: [sleeping] }
# Optional authored class override, no special person named in code:
# - { actor: high-energy, energy_kcal_per_day: 2400 }
```

Person forms are mutually exclusive for hunger: old `{ hunger: <0..1000>, tiredness: ... }` for unitless; new `{ food_energy: { initial_kcal: <0..effective daily use> }, tiredness: ... }` for food-energy. `needs: {}` means old defaults only in unitless mode; in food-energy mode require explicit `food_energy` initial value rather than silently interpreting old initial hunger 300. Each person remains opt-in through its `needs:` section. A person without a section has no simulated needs.

Market Town writes an initial reserve of **1500 kcal for each existing needs person**, tiredness 700 unless already explicitly authored otherwise. This is an authored starting game state (75% full), not a deduction from the former hunger value. A class override must have an initial reserve in its own valid range. A new unrelated world can start full, empty or between them.

Item nourishment has exactly one model form: old `{ nourishment: { hunger: <0..1000> } }` or new `{ nourishment: { energy_kcal: <0..100000>, serving: ... } }`; reject mixing. All needs nourishment sections must match the world model, including zero-energy drinks. Missing nourishment means no contribution; do not infer energy from food tags or names. Market Town's consumable catalogue must explicitly declare energy, including explicit zeroes.

```yaml
# Illustrative proposed typed serving; 500 is a test/example value, not sourced demo nutrition.
needs:
  nourishment:
    energy_kcal: 500
    serving:
      label: one prepared sandwich
      quantity: { unit: g, amount: 180 }
```

`Serving` has a bounded nonempty localized/display label (1–128 UTF-8 bytes, no control characters), plus a strictly positive integer physical quantity, closed unit `g | ml`. Quantity range 1–100000. Use a measured/authored mass for discrete foods, volume for drinks; the label supplies “one apple,” “two slices,” or “one cup.” These are needs-local typed fields, not new kernel units. Labels are display data, never parsed as rules. Catalogue display names must identify the actual sold portion. Localization must use existing presentation/content conventions, with no translated strings used as keys.

**One inventory unit = one listed, priced, consumed serving.** A 250 ml milk carton is one serving only if its item is authored as that carton; a gallon is not priced as a gallon and consumed as a cup. Choose single-serving milk/juice items in this demo, or explicitly price the whole package and make all its energy one unit. No fractional inventory, unpacking system, remaining-volume state or recipe layer is introduced. Sources per 100 g or multipacks must be normalized to this same portion by documented arithmetic before authoring integer kcal/price. Rounding and retail markup are recorded content choices.

Configuration-only cross-field checks belong in needs configuration resolution. Section decoding validates shape, bounds and carrier independently. Cross-model and effective-person-capacity checks belong in needs' own genesis reducers, after its configuration fact has reduced and actor parameters can be read. `worldpack::initial_facts` merely collects emissions: ordinary `AuthoredSection::seed` cannot read reduced configuration, despite its ordered position. A bad model/initial value must reject world genesis before any playable world is returned or save created; report an owner code naming the authored subject and key, preserving loader file context where available. Syntactic decoding retains line/column; do not promise a source line for runtime cross-field refusal. An assemble-only call is not proof of semantic validity. Keep validated model discriminators in the typed genesis events so the owner can make this check without rereading files. Reuse existing checked owner error paths; no new generic validation layer. Exact CLI error propagation is a pre-freeze audit condition. The existing `ResolvesSubjects` seam was also inspected in `authoring/src/data.rs`: it exposes own configuration and subjects but would require migrating needs to contextual authoring; do not add a dummy data owner just to bypass the proper genesis check.

## 5. Energy arithmetic, bar and consequences

For a person, let `D` be effective configured daily use and `C = D` be reserve capacity in kcal. Internally use named integer types (e.g. `Kcal`, `KcalPerWorldDay`, `EnergyReserve`) and a wide checked integer numerator in units of kcal × 86400. Store `(reserve_numerator, at)` as the food-energy variant of hunger state; tiredness's Level/mode remains separate. Maximum numerator is `100000 × 86400`; intermediate elapsed-time multiplication uses checked/wide arithmetic and clamps without overflow. Never use float or host time.

At world instant `t >= at`:

```
N(t) = max(0, N(at) - D × (t - at))
consume K kcal at t: N' = min(C × 86400, N(t) + K × count × 86400)
available_kcal = floor(N(t) / 86400)
hunger = floor(1000 × (C × 86400 - N(t)) / (C × 86400))
energy_bar_value = floor(1000 × N(t) / (C × 86400)); maximum = 1000
```

The energy reserve, not the projected hunger, is authoritative. Preserve numerator precision across folds so sleep/wake or multiple small meals do not discard fractional daily use. Floor only at disclosure. At cap/zero discard excess credit/debt: no body-fat model and no carried starvation debt. At one instant two distinct successful consumption facts contribute twice; a replayed/duplicate request must obey existing dispatch/journal semantics and must not create a second successful consumption. Do not add an unbounded deduplication cache in needs.

Hand oracle, `D=C=2000`, initial 1500: after 6 world hours reserve 1000; consume a test serving 500 → 1500; at hour 12 reserve 1000. After a whole day with no intake a full 2000 reaches 0. Two further days at zero stay zero; then 500 restores exactly 500. At full reserve consuming 500 still consumes inventory but stays full. Zero-kcal consumption consumes the held unit but neither replenishes nor folds/writes energy solely for that fact.

Daily use is a continuous rate, equal awake and asleep. “Day” means exactly 86400 world seconds, independent of wall pacing, day-length presets, calendar timezone or midnight. No day-reset Process or per-frame event. Sleeping/waking still updates tiredness/mode under existing rules; if the energy branch is folded there, preserve its numerator exactly. Observations do not write. Needs still has no periodic Process, random draw or new runtime fact; its state changes by reacting to the existing facts.

The holder-only disclosed `needs` record retains server-derived normalized `hunger`, `tiredness`, `words` and policy-facing nourishment. Add typed `food_energy` only in food-energy mode, containing `unit: kcal`, `available_kcal`, `capacity_kcal`, `use_kcal_per_world_day`, and the owner-computed bar value/maximum. Expose the model discriminator. Carried-item and existing known-kind nourishment entries additionally include kcal and the serving descriptor. Any old `nourishment.hunger` compatibility projection is explicitly a **normalized motivation score**, computed by needs as `ceil(1000 × min(K,C)/C)` (0 for K=0), not stored hunger reduction or calories. It keeps positive-energy tiny servings positive for food-seeking; do not use it to calculate reserve changes. For this first amendment retain existing food-ranking policy; a new cost/nutrient optimizer is not required.

In food-energy mode both clients replace the hunger deficit bar with a labeled **Food energy** reserve bar and the values `1500 / 2000 kcal; use 2000 kcal/world day`, plus the owner's hunger word if desired. Fuller means more energy. Unitless mode preserves its existing deficit semantics. Tiredness retains its existing independent bar. Clients and agent/LM views render disclosed values; no kcal arithmetic, rates, hunger thresholds or serving defaults in clients. Shared protocol decoder/row data and translation keys serve both clients. Do not show “1500 kcal eaten today” or “500 kcal left to eat.”

At reserve zero, or when derived normalized hunger is at or above the configured hunger threshold, controllers may seek/eat food as before. There is no forced collapse, slower walking, automatic sleep, illness, damage, eviction or purchase. No thirst state is added. A drink with energy replenishes this need; a zero-kcal drink does not. Hydration would require its own approved owning-module contract.

## 6. Economy and sourced content

Keep `configure/economy.yaml` currency name/symbol/exponent/placement as the single source. USD exponent 2 stays. Set **only the new configured player's genesis wallet** in `people/visitor.yaml` to `1000000` minor units. Do not key grants to joining a client, role takeover, reconnection, name or renderer. The current visitor is a pre-authored Person seat: this change is not a new account/player-spawning service. Other worlds author any valid opening balances with existing economy fields.

Prices remain `economy.shop.prices` on each shop's Organization. Reference values remain optional noncharged typical worth. No dollar literals, float dollars, client price tables, special food-price code or economy/nutrition cross-owner writes. Existing happening discounts remain explicit: provenance documents base price; transactions use the actual authoritative quote/discount. Shop opening/staff rules are a separate owning-economy amendment and remain an integration dependency, not implemented by this design.

### Provenance workflow required before claiming retail realism

Author a compact evidence table beside Market Town specifications (proposed agent-facing `docs/market-town-food-provenance.md`; coordinator can instead embed it into the authoritative PR design). For **every** food/drink/snack kind (apple, bread, milk, juice, coffee, tea, croissant, cake, sandwich, soup), record:

- Primary retailer/menu URL and manufacturer/USDA food reference or label URL; product/recipe identity and portion; access date, source date if stated, store/ZIP/country and currency when known.
- Observed ordinary price, package size/count, promotional/member/tax/delivery caveats; distinguish a retailer offer from a marketplace seller. Source excerpt or legally shareable facts sufficient to recheck; no whole-page copying.
- Nutrition basis: kcal per source portion versus per 100 g, adopted mass/volume and explicit conversion; exact package versus edible portion.
- Chosen game serving, integer kcal and minor-unit price, rounding/markup rationale, and whether exact match, normalized package comparison, or authored recipe analogue. A home-grocery unit comparison does not establish prepared cafe menu pricing.
- Status `verified contextual observation | dated observation | authored estimate | unverified`. Only supported rows may be described as sourced plausible retail prices. Unsupported rows block the blanket “realistic retail prices” acceptance claim; they need not block the engine design.

Use primary menu data for cafe servings and primary retailer product pages for packaged groceries. USDA FoodData Central or manufacturer labels are suitable energy sources; FDA guidance establishes how to interpret portions, not a price database. No live scraping, network lookup or price API is needed at world runtime. Values are reviewed, versioned content snapshots, with an explicit locale/date, not a promise of current/local market prices. Do not silently refresh them in existing saves.

Research in this design session opened the FDA's [serving-size explanation](https://www.fda.gov/food/nutrition-facts-label/serving-size-nutrition-facts-label) and [label interpretation guide](https://www.fda.gov/food/nutrition-facts-label/how-understand-and-use-nutrition-facts-label): nutrition values must be interpreted with their serving/package basis. FDA describes 2000 calories as a general guide; this design's 2000 value is independently the operator's chosen game default, not personal medical advice.

A primary [Walmart organic whole milk product page](https://www.walmart.com/ip/44391006) was reachable. Search/index output surfaced a gallon listing with a price and energy-per-serving claim, but lacked a chosen local store and an independently checked complete serving basis. It is a **research lead, not approved demo price/nutrition evidence**. It does not justify the current ambiguous “carton” item. No final replacement menu or current-price claim is made here; completing the ten-row evidence table is an explicit pre-content-freeze deliverable, not an unresolved operator preference.

### Preserve the existing economy horizon honestly

Retail prices and changed kcal portions invalidate old price-per-hunger arithmetic. Do not leave comments claiming the old 300-day guarantee. Before content freeze, size a feasible 2000-kcal daily basket under six inventory-unit capacity, compute `daily_cost × 300`, and check every nonworker endowment, worker starting cash/wages, organization funding and production (including discounts and buying availability as applicable). Visitor $10000 alone does not solve NPC affordability. Record any needed nonplayer endowment/production amendments as explicit authored content changes for review; do not introduce money faucets, free food, forced grants or counterfeit nutrition to pass rhythm checks.

Retain S35's 300-day affordability/supply and rhythm requirements unless separately amended. If realistic prices cannot meet those with current authored wages/endowments, return the exact content sizing proposal to the coordinator before freeze. This is an identified content dependency, not permission to weaken acceptance or retune frozen rhythm bounds after a failed run.

## 7. Persistence, restart and optional composition

This contract is for **new world creation**. Use the exact revision-2 N3 version table and SDK seam; record those choices in the frozen implementation contract. Do not keep a stale system version to bypass persistence checks. Existing v1 saves stay unchanged and can be resumed with the matching retained package/source and their original world configuration. The new version must refuse incompatible saves clearly and nondestructively; no implicit hunger→kcal conversion, no automatic re-genesis, no wallet reset, and no replacing an existing save file after refusal. Automatic migration is not included. This limitation must be visible in the launch/package handoff before the operator chooses a new demo save.

A new save under the amended version must replay with its recorded parameters, initial reserves, servings, currency, prices, wallets and inventories. On restart, the state numerator and last instant survive exactly; only elapsed **world time** applies, with no wall-time offline depletion. No new genesis funding is applied. Changing authored genesis configuration while reusing a save follows existing drift/composition refusal, rather than applying new rates retroactively. Compare logically committed contents on incompatible load refusal as specified by revision 2 N5; physical database/WAL bytes are not invariant.

Creation-time alternatives:

1. Food-energy world: needs food-energy config, person energy forms and matching item energy/serving forms; consumption/inventory present for eating restoration.
2. Unitless world: same existing needs owner and its unitless forms, no energy keys/records/rows.
3. No needs: remove enabled needs, configure entry/file and all its person/item sections. Existing economy/inventory/consumption/sleep still work within their own dependencies; no fabricated hunger or kcal HUD.
4. No consumption: remove nourishment sections and eating restorer (`restore.hunger: []`) or refuse by name. Energy may deplete as an explicit authored no-restoration world. No hidden substitute action.
5. No sleep: remove sleep configuration and set `restore.tiredness: []` explicitly as current needs requires; energy still depletes normally.
6. No economy: remove economy config/sections and dependent content/settings as existing contracts require; held food may still be consumed if inventory/consumption remain. Do not require a wallet to gain food energy.

Do not promise live hot-removal, runtime currency/rate changes, or destructive conversion of an existing save. Reconfiguration/removal is creation-time composition for this amendment. Preserve Social Cafe and Bodies Yard as delivered removal witnesses; use scratch/generated variants of an enabled world for positive/negative cases rather than enriching either witness.

## 8. Required acceptance and verification ownership

All rows are **NOT RUN** in this design-only session. Independent review, implementation, packaged integration and operator trial are separate states.

| ID | Observable acceptance / adversary | Canonical owner |
| --- | --- | --- |
| NE-1 | Real loader rejects mixed/unknown models, wrong carrier, energy without serving/positive quantity, initial above effective capacity, zero/oversized daily use, unitless knobs in energy mode, energy keys in unitless mode, and required packs absent. Names offending file/key for decode checks and authored subject/key for genesis checks; permitted variants complete genesis. No listener or new persisted save is created on failure. Include actor-class override 2400 versus base 2000. | Needs configuration/section and CLI loader integration |
| NE-2 | Literal 1500→1000 (6h), +500→1500, →1000 (12h) oracle; full 2000 exhausts after 24h; partial seconds preserved across folds; cap/empty/zero; same-time meals, wide elapsed multiplication and batched count safe. Splitting a positive interval into folds agrees with unsplit calculation. | Pure energy arithmetic plus real consumption integration, not getter tests |
| NE-3 | In a new demo save, visitor wallet is 1000000; purchase one sourced serving at quoted price P: visitor -P, seller +P, seller stock -1, player count +1 exactly once. Consume it: inventory -1, reserve +K capped at capacity, no money change. Unaffordable/out-of-stock/unsupported action leaves balances, inventory and reserve unchanged. | Existing market/consumption integration extended for new needs |
| NE-4 | Consume explicit zero-kcal drink: inventory -1, no energy credit and no energy-only state fold/write. A caloric drink restores precisely its authored serving. Capped consumption remains legal and consumes stock. No thirsty or collapse side effect. | Needs reactions/market scenario |
| NE-5 | Save after purchase, after meal, during fractional depletion and sleep; close/reopen and kill/resume agree with uninterrupted authoritative state/events, currency, holdings, wallet, energy numerator and future readouts. No regrant, duplicate consumption or offline wall-time depletion. Incompatible old-version/config load refuses and does not alter the old save. | Persistence lifecycle on actual host/CLI, not mocked save files |
| NE-6 | 1/2/4-hour real day presets give equal energy at equal world instants; no midnight refill; observation/read frequency does not change state/facts; energy uses the same rate asleep and awake. No needs Process or runtime fact added. | Deterministic host/needs integration |
| NE-7 | Headless, 2D, 3D and agent views see identical owner-provided kcal, rate, portion and bar values for the same observer/instant. Another person cannot see holder reserve. Both actual clients show correctly localized energy units and replenishing bar; existing unitless rows stay semantically unchanged. Renderer detached/models absent do not change authoritative scripted outcomes. | Shared observation/row parity + actual client lifecycle/visual evidence |
| NE-8 | Second generated world uses a fictional currency with exponent 0, different balance and 2400-kcal use/portions, without kernel/client edits. Unitless variant works; removing needs and its data leaves no energy record/row and no rule behavior/draw change caused by absent needs. Social Cafe/Bodies Yard baseline facts remain unchanged. Bad partial removal refuses by name. | Real composition/removal scenarios |
| NE-9 | Ten catalogue evidence rows have matched retail/menu serving and kcal basis; a complete reachable daily basket fits capacity; documented 300-day affordability/production arithmetic holds for all applicable people. Subsequent S35 rhythm/long-save checks retain their existing bounds; changed Market Town baselines have explicit reasons. Never silently revise a failed fixed threshold. | Content audit then existing integration/rhythm owners; heavy runs only coordinator-scheduled |
| NE-10 | Inspect maintained function/file size, ownership and consumer diffs; no world switches, duplicated client rules, money grants, general resource manager, compatibility bypass or new dependency. Independent reviewer evaluates arithmetic, restart refusal and price/portion provenance before readiness. | Coordinator independent red-team |

Counterfactuals to pin: relabel old 650 as kcal; apply rate per wall hour; discard fractional energy on sleep; grant money on join; give energy to zero drink; consume a gallon but credit one cup; disclose others' reserve; compute bar/thresholds in a client; silently accept unmatched model keys. Use a small selected mutation where it proves the relevant integration assertion, not a mutation census.

Static formatting/types/schemas own trivial syntax and field shape; integration owns cross-owner atomicity and refusal semantics; real host/client lifecycle owns save/restart and displayed behavior. No real-model inference is needed to establish nutrition correctness. Agent/LM prompt rendering needs deterministic checks because displayed facts change; actual model dietary planning is not a requirement here, so real-LLM Gate is not required for this lane. Preserve the separate contextual conversation/memory Gate in the broader playable plan.

## 9. Bounded implementation route after review (not an execution grant)

Use one isolated worktree/owner and a fresh session for the eventual PR. This amendment can be one coherent PR with these semantic commits; if lifecycle size requires a split, each child still needs a meaningful integrated checkpoint, and no old v1 save compatibility claim is added by implication.

### C0 — Reconcile authoritative docs and content evidence

1. Goal: settle model/bar/save contract and source the authored menu before any behavior code.
2. Scope: authoritative playable/S35 amendment, ARC-284/285 dated notes, MODULE_SPEC sections, provenance evidence and reviewed content sizing. No unrelated launcher, clock, sleep-skip or conversation changes.
3. Implementation: [ ] coordinator incorporates this proposal once; [ ] gather ten evidence rows; [ ] record new-world and version/refusal policy; [ ] fill execution authority/base/budget; [ ] independent review and explicit freeze.
4. Validation: [ ] read against actual module and persistence seams; [ ] hand-check serving normalization and basket/cash/production arithmetic.
5. Acceptance: no unresolved model/bar/ownership choice, no unsupported all-menu realism claim, and an operator-approved concrete version/save policy.
6. Failure cases: missing source stays unverified; failed sizing returns concrete authored revision, never lowers existing acceptance implicitly.
7. Evidence: read-only `rg`/source inspection and primary-source browsing; record access/source context. No test execution required to freeze an evidence-backed design.
8. Boundary: documentation/design only; no implementation until freeze.

### C1 — Needs food-energy model and authoritative integration

1. Goal: implement the approved model behind the existing owner with deterministic consumption/restart semantics.
2. Scope: `systems/needs/src/{component,level,section,interactions,event,system,disclose}.rs`, a small `food_energy.rs` if justified; tests under needs; affected CLI integration and generated schemas/reference. No consumption/inventory/economy rule change unless new audited evidence requires a material amendment.
3. Implementation: [ ] typed variants and named units; [ ] precision-preserving arithmetic; [ ] validated model-specific config/forms; [ ] genesis/version changes; [ ] owner-derived disclosure/motivation fields; [ ] conditional dependency checks via existing seams.
4. Validation: [ ] NE-1/2/4/6; [ ] real market/persistence integration NE-3/5; [ ] unitless/removal parity NE-8; [ ] focused format/check/clippy for affected crates.
5. Acceptance: exact oracles, owner boundaries and restart/refusal hold; no runtime fact/Process added.
6. Failure cases: invalid config fails before play; incompatible save fails without mutation; any changed public contract outside freeze returns to coordinator.
7. Intended commands: targeted `cargo test -p mineworld-needs`; selected existing CLI `needs_sections`, `needs_town`, market integration targets after source re-audit; touched-crate clippy and repository fmt. Use existing slot wrapper for builds/tests; record exact commands/counts/wall time and NOT RUN explicitly. Do not run workspace suite locally by default.
8. Boundary: behavior, owning documentation/schema updates and their meaningful checks, no content price tuning against results. [ ] Implementation; [ ] validation; [ ] independent logic review, with separate evidence.

### C2 — Content, shared views and packaged player loop

1. Goal: make the reviewed serving/price/new-player configuration visible and playable through both clients.
2. Scope: Market Town needs/economy/person/item/organization content and source-backed notes; shared protocol needs reader, 2D reader/HUD and 3D needs HUD, agent typed records/views, relevant prompt rendering/catalog strings and parity fixtures. Audit exact consumer paths before enumerating edits; reuse existing shared readers/catalog. Generated reference and package parity artifacts belong to their existing owners.
3. Implementation: [ ] visitor 1000000 genesis only; [ ] all demo needs people/consumables match energy model; [ ] price/portion and any explicitly reviewed supply/endowment changes; [ ] truthful kcal/rate/serving row in shared views; [ ] hide absent mode/system; [ ] integrate matching source/export in coordinator's package workflow.
4. Validation: [ ] NE-3/5/7/8/9, with selected row parity and actual 2D/3D packaged startup/buy/eat/drink/restart; [ ] existing S35 rhythm/300-day checks scheduled once at integrated head through shared job limits; [ ] Python/GDScript checks only for touched consumers and existing client-rule guard.
5. Acceptance: actual quoted-price transaction and visible energy replenishment survive restart; other currency/world/removal works; all required provenance and unchanged witnesses verified. Report implemented/integrated/reviewed/operator-tried independently.
6. Failure cases: client malformed record must not invent units/default calories; wrong export/source identity is inconclusive, not pass; absent model has no fake energy row. Price affordability failures are owner content defects or material sizing revisions, not excuse for forced grants.
7. Intended commands/evidence: existing `needs_rows_parity`, `client_3d_market`, market scenario targets after re-audit; existing agent/prompt tests; actual client launch and screenshots/logs tied to exact source/export. Coordinator selects canonical CI, avoiding repeated equivalent full runs. No model/paid calls; no long/heavy run until dispatched under shared limits.
8. Boundary: configured demo plus reusable presentation consumers, no social-cafe enrichment, new spawning service, hydration, clinical model or generic economy abstraction. [ ] Implementation; [ ] validation; [ ] independent logic/maintainability review separately.

## 10. Kickoff outline and remaining review conditions

Fresh executor reads CLAUDE in full and Structured Coding execute routing, the **approved** integrated amendment plus original binding S35 docs, ENGINEERING_RULES/STANDARDS, owning contracts and full working/test rules. Record exact merged base and isolated worktree ownership; audit changed source/consumer seams again. Coordinator dispatch identifies owning paths, reviewer, acceptance commands, shared heavy-job policy and explicit publication/commit boundaries; this temporary design-only dispatch supplies none of those execution grants.

Before freeze the coordinator needs: (a) operator acceptance of the reserve-bar/mode proposal and explicit old-save version barrier, (b) completed matched serving/price table and viable content sizing, (c) exact version/schema and owner-check implementation audit, (d) independent red-team disposition. These are concrete review/artifact conditions; 2000 kcal/day, USD10000 genesis and optional composition are already requirements and need no repeated preference question. If the operator requires new binaries to resume old v1 saves, a separately reviewed migration/compatibility design is necessary; do not smuggle it into this lane.

Delivery in this lane: this temporary draft only. No code, test, commit, push, PR, public write, paid service, model run, heavy job or child agent. Primary coordinator owns authoritative repository incorporation and review scheduling.

## 11. Revision 2 — independent review disposition and concrete seam amendments

Revision date: 2026-11-23. Read the entire independent review `/tmp/mineworld-nutrition-independent-review-20261123.md`. Its verdict is **direction accepted, NOT FREEZE READY**; this revision does not convert that verdict into approval. This section supersedes conflicting implementation-location, version, affordability, byte-preservation and single-PR wording above. Earlier requirements and failed assumptions remain recorded for audit.

Additional read-only audit: `tools/cli/src/{serve,run,main}.rs`; `persistence/src/{world,sqlite,backend,error}.rs`; `kernel/src/{dispatch,error}.rs`; `sdk/rust/src/interactions/{mod,lookup,section,resolve}.rs`; `authoring/src/configuration.rs`; `worldpack/src/{configure,error}.rs`; `cognition/rule-controller/src/{paced,offered,fetch,needs}.rs`. No code or tests were run. The five findings are addressed by proposed contracts below; their implementation and independent re-review are still required.

### N1 — Delay backend allocation until valid genesis and encoded initialization exist

**Choose a generic lazy backend factory in persistence**, rather than a second genesis preflight or contextual needs authoring. Existing `PersistentWorld::create(backend, world, creation)` snapshots before genesis, applies genesis once, builds manifest/input/revision/snapshot bytes, then initializes the backend. Refactor that sequence into a common internal path and add:

```
PersistentWorld::create_with_backend<F>(world, creation, make_backend: F)
  -> Result<(PersistentWorld, Vec<EventEnvelope>), PersistError>
where F: FnOnce() -> Result<Box<dyn PersistenceBackend>, PersistError>
```

The closure is invoked **exactly once, only after** the unstarted candidate's snapshot, successful in-memory genesis, manifest encoding and genesis-row/snapshot encoding all succeed. It then calls `backend.initialize` once using that already computed row; no second genesis, snapshot restoration, re-execution or temporary database. The candidate `World` is moved into the operation; on any failure no `PersistentWorld` is returned. Old `create` remains the existing API with the same behavior for callers that deliberately supply an allocated backend, delegating to the common path through a closure returning that backend. Document that the old API cannot promise absence of an already-created backend on genesis failure.

Change both actual CLI new-save callers: `tools/cli/src/serve.rs::persisted` (PowerLoss) and `tools/cli/src/run.rs::Begun::new` (ProcessCrash). They assemble first and pass a closure that invokes `SqliteBackend::create`. Before this operation they must not create the save directory, open SQLite or print a successful creation message. Providers are returned only on success. `mineworld validate` already calls `pack.load` (main.rs), so it executes genesis without a backend. Server bind remains after successful host construction. A model/capacity failure therefore leaves an absent destination directory absent, or an existing empty caller-owned destination unchanged, and opens no listener.

Backend/storage failure is a different disposition: if the closure or initialize fails after valid genesis, report `Storage`/existing error, return no world/listener and never claim that initialization succeeded or auto-resume a usable save. This amendment does **not** promise absence of SQLite artifacts after I/O failure or power loss during backend initialization; retain existing recovery semantics and surface that limitation. Do not delete any save to simulate rollback. The factory still uses existing create's SaveExists refusal; do not turn the earlier `exists` branch into overwrite authority or claim this work fixes the pre-existing interprocess create race.

Acceptance additions: one invalid initial-capacity fixture and one invalid model fixture exercise actual `server --save` and `run --save`; both exit nonzero with the diagnostic from N2 and leave no `world.sqlite`/WAL/SHM artifacts or new directory when the path was absent. A focused factory test proves closure invocation count 0 for genesis/encoding failure and 1 for success, with real initialized save replay proving genesis happened once. Existing create/replay integration pins old entry-point behavior. No whole persistence rewrite or needs-aware host logic.

### N2 — Separate invalid authored genesis from a broken checked runtime fact

Current `KernelError::FactRefusedByOwner` explicitly means a system violated a checked-constructor contract; using it as the ordinary authored-content error is wrong. **Propose one domain-neutral kernel construction error**:

```
KernelError::InvalidGenesisContent {
  system: SystemId,
  subject: EntityKey,
  code: RejectionCode,
  field: String,     // bounded diagnostic field path, never a rule selector
  detail: String,    // bounded human-readable diagnostic
}
```

It carries no needs/currency/food semantics and introduces no validation callback, generalized data layer or runtime rejection path. This is an explicit minimal amendment to S35's former “kernel unchanged” invariant, requiring review; it is not hidden in a needs-only scope. Need owner reducers validate all of a genesis assignment **before inserting that assignment's component** and return this variant for authoring contradictions when `event.caused_by()` is `Causation::WorldGenesis` (use the actual envelope accessor on the execution base). Codes are `needs-model-mismatch` for wrong person/item model and `needs-initial-energy-out-of-range` for initial reserve above the actor's effective capacity. Diagnostics include `system needs`, authored entity key, `food_energy.initial_kcal` or `nourishment.energy_kcal`/`hunger`, supplied model/value and expected model/capacity. Obtain the subject key from `EntityRecord::key`, not a YAML read or display name. Missing subject identity, undecodable payload, wrong schema and impossible event ordering remain contract/corruption errors, not user errors.

The dispatcher already records all initial emissions and then reduces them nontransactionally; no rollback is promised. On this construction error discard the **entire candidate**, including earlier in-memory reductions, and stop initialization. N1 ensures it has no backend yet. `PackError::Composition` transparently wraps kernel construction failure; `PersistError::Kernel` does likewise, and existing CLI `described`/`HostError::build` paths must retain the nested owner/code/subject/field detail in visible output. No fabricated file/line: section decoder errors retain actual source location; this genesis cross-field error carries a subject/key. World creation cannot continue after it.

Only fresh authored genesis gets the user-input diagnostic classification. During replay of persisted genesis, an identical failure means the saved history cannot be reconstructed and resume/verify fails closed; it is not permission to amend facts. A non-genesis assignment contradicting state returns the existing checked-owner violation/contract fault and retains existing runtime fault/journal policy. Do not reuse `InvalidGenesisContent` for an ordinary action rejection or change dispatch into a partially rollbackable transaction.

Pin visible CLI output for both N1 fixtures, including code, owner, subject and field; pin no success message/listener/save. Add a replay fixture whose incompatible assignment fails closed and returns no resumed world. Error documentation explicitly distinguishes these cases. No promise that every wrapper prints the same prefix beyond the identifying fields.

### N3 — Exact versions and a small SDK validation/version seam

The current generic `SectionConfigured<S>` and `Interactions<S>` hardcode schema 1; a needs system bump alone is insufficient. Extend `InteractionSection` with defaulted constants `CONFIGURED_SCHEMA_VERSION: EventSchemaVersion = 1` and `COMPONENT_SCHEMA_VERSION: ComponentSchemaVersion = 1`; use those in the two generic implementations in `lookup.rs`. Other owners inherit exactly 1 and their declarations/facts remain unchanged. Needs overrides both to 2. The generic schema/reference generator and component declarations must obtain the same associated constants, not separate literals.

| Contract | Proposed version |
| --- | --- |
| NeedsSystem system version | 2 |
| stored Needs component and holder-disclosed needs record | 2 |
| Nourishment component | 2 |
| NeedsAssigned event | 2 |
| NourishmentAssigned event | 2 |
| needs-interactions-configured event (`SectionConfigured<NeedsSystem>`) | 2 |
| needs-interactions component (`Interactions<NeedsSystem>`) | 2 |
| other System Packs, economy/inventory/consumption facts and save container format | unchanged |

Holder disclosure continues to use the needs component type/version; a second contradictory v1 view is not emitted. Unitless needs created by version 2 uses the version-2 typed model too; retaining unitless behavior is not a promise that version-1 persisted bytes load in version 2. All schema-qualified consumers/fixtures must accept/verify v2, rather than matching only the old type string and claiming compatibility.

**Configuration presence validation:** current `HeldSection::check` only calls generic resolve; there is no owning cross-field hook. Add a default no-op `InteractionSection::validate_authored(section: &Section<Self>, resolved: &Resolved<Self>) -> Result<(), ConfigurationRefusal>`. `resolve` first performs existing generic class/ambiguity resolution, builds `Resolved`, invokes this hook once, then returns the result. Thus check and seed paths cannot disagree. The hook may inspect authored optional fields via `Partial::set_names`/the actual typed option fields; the resolved value is read-only. Add `ConfigurationRefusal::Owner { entry: Option<EntryAt>, code: RejectionCode, detail: String }`; map this exhaustively in `worldpack/src/configure.rs` to an owner-named configuration error retaining the actual file path and entry index when available. No false line/column claim after decoding. All existing owners use no-op behavior.

Needs' hook selects the effective model from `resolved.base.parameters`. Inspect each source parameter entry in authored order, including reference-list default/extends chain (existing `Section::chain`/`reference_lists`) and region entries, **before optional-field presence is lost**; default compiled fields do not count as authored. Reject authored unitless hunger rate keys anywhere in a food-energy configuration even if shadowed; reject any energy-use key in a unitless configuration. Require a base energy-use entry for food-energy, including an explicit inherited reference-list base entry if such a list is added later. Actor energy-use overrides are legal only in food-energy. The existing FIELD_ROLES check keeps model base-only and disallows placed energy-rate fields. Defaults/regions/class ambiguity retain existing resolve priority. The only current needs reference list is empty `default`; do not introduce new lists to satisfy this amendment. Configuration-only failures use `needs-model-parameter-conflict` or `needs-energy-daily-use-required`, naming the actual EntryAt/field. Model choice from the resolved base plus explicit source-presence scan makes contradictory inputs reject even when later entries overwrite them.

Effective capacity checking for a person still happens in N2 genesis reduction through existing actor-scoped parameter lookup after configuration reduction; no actor/class matching algorithm is duplicated in section decoding or the hook. Authoring scalar bounds use existing typed bounds. Add one inherited-list stub regression for the generic hook's propagation, plus focused needs actor/default/conflict cases, not duplicate tests of serde.

**Error precedence on incompatible old saves:** current serve/run opens existing SQLite, checks saved configuration, then composes/resumes. Keep that order. A configuration drift/decode error may occur before system version/composition refusal. Accept the first applicable explicit incompatibility, identified by owning code/path/system; do not promise `PersistedSystemOutdated` specifically for every old save. There is no retry-as-fresh, reset, migration or schema-number spoofing.

### N4 — Reachable diet, capacity and liquidity proof replaces the cheap-basket argument

Source audit confirms `fetch::best` chooses the greatest disclosed nourishment among available offers, ties lowest ItemId; `NeedsPolicy::eat` does likewise. `fetch` runs only when hungry and no positive nourishing item is held, may free hands, and stops on insufficient funds. `paced::DECIDED_BY_NAME` does **not** exclude eat/drink/buy, so generic `offered::attempt` can consume or purchase below the hunger threshold. `beyond_chance_budget` limits each chance purchase to `chance_spend_permille` of current wallet; the default 1 is not a total daily spending budget. Free-hand, gifts, cap waste and random consumption defeat an unqualified useful-energy-only bound. This is now an explicit source fact, not an assumed hunger-only diet.

**Chosen route: split engine/model freeze from Market Town content freeze.** Keep the controller algorithm unchanged in this amendment. A sourced low-cost basket proves author feasibility only; it is not acceptance evidence for the autonomous diet. Do not insert a planner/optimizer, globally exclude consumption from generic offers, or set new policy defaults implicitly. Any policy configuration revision must be separately specified with before/after behavior and included in content freeze, not tuned after a failed run.

Before the content PR can freeze, deliver a table for every enabled controller seat/class (human visitor excluded from an autonomous guarantee once occupied by a human). The table records resolved controller/model, cadence and hosting scale, initial holdings/nonfood slots, initial energy, thresholds, all purchase/consume/give/free-hand routes, price modifiers/repricing policies, job wages, actual first wage timing, initial seller capital and production. Include every candidate kind the unchanged policy can select at each reachable shop; a highest-energy expensive choice cannot be omitted because another shop is cheaper. Human/model choices are not bounded by a deterministic rule-controller guarantee; their actual purchase correctness remains NE-3/5/7, while a model fallback must use the documented rule profile.

Use the following sufficient bounds or a tighter independently reviewed deterministic reachability bound. For a fixed H=300-day acceptance schedule, let Q_p(t) be a proven upper bound on all economic-action consultation opportunities for seat p through time t, counting the actual headless/host consult schedule (including walk-step opportunities if they can choose an economic action). At most one successful request per opportunity. Let Pmax_p(t) be the maximum reachable charged price including all modifiers/possible permitted repricing; if no finite audited bound below engine PRICE_MAX is established, use PRICE_MAX and reject the sizing as impractical rather than assume menu prices stay fixed. Then a conservative bound is `all purchase outflow <= Q_p(t) × Pmax_p(t)`. It covers chance purchases, waste and cap excess even without guaranteed useful kcal. Add any other reachable outgoing transfer/proposal settlement to this bound; if none, demonstrate why with available offers/policy. This conservative bound may be too expensive to use for realistic endowments; that is an exposed limitation requiring a tighter policy/reachability proposal before content freeze, not authority to inflate balances silently.

A tighter food-bound may count `useful_kcal >= min(K, deficit at consumption)` only on paths with a demonstrated positive deficit lower bound (for the eat band at hunger 600, deficit is at least 0.6C under owner projection). It **cannot** charge all consumption to that bound because generic/free-hand consumption can have zero useful energy. Count such waste purchases separately by their actual reachable consultation/action bound. Include normalized nourishment rounding/capping in ranking and ItemId ties; per-person capacity/day override changes both rankings and useful-energy estimates. Six inventory units include clothes/books/initial food and every nonfood kind; cannot assume six free meal slots.

For **every prefix**, not just day 300, prove `initial_wallet + guaranteed_received_wages/proceeds - bounded_outgoing >= reserve needed to buy the next reachable required food`. Do not count unearned wages, uncollected jobs or hypothetical sale revenue as guaranteed income. In particular show pre-first-pay liquidity for Alice/Felix/Ivan. For each shop prove initial wallet covers all wage outflows before guaranteed receipts and each prefix remains solvent; predicted customer receipts are not guaranteed unless derived from the same reachable policy bound. Stock proof records per-kind initial stock + guaranteed production at the actual completion times - maximum reachable withdrawals, with at least one purchasable positive-energy candidate whenever the rule must fetch. Size morning stock before first production and correlated demand from all residents together; total daily kcal production alone is insufficient. Required journey/opening/commute timing still must satisfy the separate living-world/hour contracts and unchanged S35 rhythm checks.

The content-freeze deliverable must contain **concrete minor-unit balances/wages, item prices/kcal/portions, stock/production counts and policy configuration**, with these prefix calculations and the ten provenance rows. No final numbers beyond required visitor 1000000 cents and configured use 2000 have been invented in this revision. Existing NPC endowments/wages remain unaudited for the new price model and are **not accepted**. This is a blocked content artifact, not a new unresolved user preference: a content author must propose the numbers for independent review. If the reachable-policy proof cannot be made practical, present the smallest explicit controller-policy amendment as a new material design; do not claim engine freeze approves it. Long rhythm/300-day gates remain the empirical acceptance after this pre-freeze argument, with their original thresholds unchanged.

### N5 — Logical save preservation, not immutable SQLite/WAL representation

**Choose logical preservation of committed contents** for incompatible-save refusal. The earlier “unchanged save bytes” wording is withdrawn. Existing open uses a read/write SQLite connection and sets journal_mode=WAL; valid connection/close/recovery can change database/WAL/SHM representation. Do not promise physical hashes, modification times, file presence or checkpoint layout are identical after refusal.

For compatibility/configuration refusal require: no initialize, no append/commit of a new world revision, no re-genesis, no new funded/nourishment/consumed facts, no deletion/replacement/truncation by application code, no changed manifest/head/committed journal/facts/snapshot/blob contents or host-journal settings. SQLite's own recovery/checkpoint representation changes are allowed only insofar as they preserve those logically committed contents. User originals remain untouched during development; tests use scratch copies of the **complete** database plus WAL/SHM family made consistently (SQLite backup for a live source, or the existing crash fixture's closed/quiesced snapshot protocol). Never copy only the main file from an active WAL database, never delete WAL as a repair, and never use launcher fresh/reset after incompatibility.

Evidence must include both a normally closed save and an existing crash-tail/WAL fixture, each copied safely to scratch. Establish a consistent logical before-view of manifest, all committed revision/fact/snapshot/block rows, head and host metadata; attempt incompatible config/version open using actual CLI; compare a consistent logical after-view. A complete logical row comparison is stronger than counts or just the head; ignore physical pages/ordering and WAL layout. If opening/checking the scratch fixture already normalizes SQLite, record that stage explicitly; it cannot substitute for testing the real rejected-open path. Existing replay verification of the copy after rejection with its matching binary/config demonstrates retained committed history. Invalid/corrupt WAL recovery may fail closed and is not classified as a valid incompatibility pass. No read-only compatibility preflight or storage-format change is introduced by this chosen scope.

### Additional bounded details fixed by review

- Use i128 intermediates **before** subtracting signed WorldTime endpoints and before multiplying D×elapsed or K×count×86400. At backwards-time read return an explicit invariant/contract failure (no saturating invented duration); normal callers must uphold monotonic time. Include max count, signed endpoint difference and one-second/fold-boundary oracles. Clamp a valid positive result before narrowing.
- Sleep mode has one truth in needs: retain it in tiredness's existing Level and make `Needs::mode()` read that Level. The energy variant carries numerator+instant only. Unitless hunger still folds its own level but cannot independently define asleep state; existing sleep reactions turn both unitless levels together. No independent energy asleep flag.
- Food-energy disclosed v2 shape is a tagged model record. Define `food_energy` fields as unsigned bounded integers plus literal unit `kcal` and owner-supplied bar value/max (max positive, value within max); serving is `label` plus `quantity {unit:g|ml,amount}` as above. Shared readers reject malformed/incomplete energy records as unavailable energy information and draw no energy or fallback deficit bar; they may still render separately valid tiredness. Missing energy in a claimed food-energy record is not unitless mode. Legacy schema1 observations remain supported only by their explicitly existing unitless decoder; no schema2 record is silently downgraded.
- Preserve the existing bounded plain serving label for now, treated like item display content. The current item names are plain authored text; do not invent a translatable-message object under a string. Unit/rate/UI labels use existing en/zh_Hans catalog keys; content label localization beyond current item conventions is separate work. No client infers calories from translated label text.

### Revised delivery boundaries and review checklist

Replace §9's optional one-PR suggestion with two separately frozen checkpoints:

**Engine/model PR:** needs typed model + schema2 disclosure, SDK two version constants and default owner-validation hook/refusal plumbing, generic lazy backend factory in persistence and both CLI callers, generic invalid-genesis error and transparent propagation, synthetic composition/serving and scripted purchase/consume/restart tests, shared decoder and truthful rows in both clients/agent views using synthetic data. No Market Town retail-realism or 300-day economy claim. Kernel changes are limited to the domain-neutral error type/documentation; no needs semantics, clock/controller/currency behavior or generic resource manager enters kernel. All these owning paths are now explicit scope additions requiring review, not incidental fixes to C1.

Engine semantic commits (each requires separate [ ] implementation / [ ] deterministic validation / [ ] independent review evidence): (1) infrastructure contracts + SDK version/validation seam with unchanged-owner parity; (2) lazy persistence creation and CLI error/no-save lifecycle; (3) needs arithmetic/forms/schema2/reactions and scripted persistence/composition; (4) shared typed consumers and actual two-client synthetic energy display. For every commit retain §9's eight-field format in the canonical PR ledger before freeze, exact audited files/callers, targeted commands and failure cases from N1–N5. No heavy/inference run in this design lane.

**Market Town content/integration PR:** depends on the engine capability; requires complete price/portion provenance and N4 reachable-policy/prefix-liquidity/stock artifact, explicit new-save launcher instructions, concrete reviewed content and policy values, required visitor 1000000-cent genesis and use 2000, actual both-client/source-export shopping/consume/restart evidence, and scheduled existing long acceptance. Do not label this checkpoint frozen/complete solely because the engine PR is ready. Coordinator owns one authoritative amendment; these temporary files remain review inputs.

Updated acceptance: NE-1 adds the actual run/server no-backend/listener checks and exact diagnostic identifiers; NE-5 means N5's logical preservation and accepted first-incompatibility precedence, with real closed+WAL copies; NE-8 pins unchanged generic owners' schema1 declarations; NE-9 requires N4's reachable-choice/prefix proof, not a cheap-basket existence argument. All are NOT RUN. Independent review completed on revision1 with five blockers; revision2 awaits the same reviewer's disposition.

Residual **operator** decisions are now only acceptance of the concrete reserve/mode/new-save-only contract (including old-package retention), and the explicit minimal infrastructure additions/phase split. Do not re-ask approved 2000kcal configurable or USD10000 genesis. Residual **engineering artifacts** are canonical eight-field engine PR commit design and exact source audit on its execution base, implementable source-backed content table/financial-stock proof, complete caller/schema consumer mapping and independent re-review. They are work to complete before the respective freeze, not preferences to defer back to the user. No implementation authorization, freeze or public delivery is asserted.

Revision 3 preparation (2026-11-23): corrected §5 threshold direction after independent revision-2 architectural PASS for operator decision. Canonical engine PR draft and filled proposal contract are prepared in `/tmp/mineworld-nutrition-engine-design-20261123/.structured-coding/plans/mvp0/`; no freeze or implementation grant. Original requirement/proposal and review history are preserved.

Canonical review package prepared (still unfrozen): `pr-nutrition-engine-20261123.md` and `pr-nutrition-engine-contract-20261123.md` in the isolated planning clone. It supplies five eight-field commit plans C0–C4, a full caller/consumer table with raw rg evidence and source hash manifest, exact proposed wire row replacement identifier preserving R7, unchanged fee/diet policies and a four-point operator design package. Engine/content scope and current design-only execution authority remain explicit. No execution grant inferred.
