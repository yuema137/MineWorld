# PR FE-b — Market Town food-energy content: sourced prices and kcal, wallets, wages, stock, liquidity proof

**Lifecycle:** `DRAFT — FREEZE-READY`, revision 2 (2026-11-28; revision 1 2026-11-27). Not frozen. Implementation needs the operator's answers to §1 and
a `DESIGN FROZEN` header written by the primary after that ruling; the execution contract is
[`pr-fe-b-contract.md`](pr-fe-b-contract.md) (`DRAFT`).

**Step:** [`step-fe-food-energy.md`](step-fe-food-energy.md) (STEP DESIGN FROZEN 2026-11-27), PR FE-b; owns NE-3, NE-5,
NE-7 on Market Town and all of NE-9, and shares NE-10. **Engine:** FE-a ([`pr-fe-a-food-energy-engine.md`](pr-fe-a-food-energy-engine.md)),
merged as #555. **Requirement:** [`operator-playable-mvp-20261123.md`](operator-playable-mvp-20261123.md) §1, §4 items 3–5, 8.
**History/evidence only:** [`fe-evidence/20261123-content-evidence.md`](fe-evidence/20261123-content-evidence.md) (the
2026-11-23 Codex content draft; where it differs from this document, this document wins).

**Design session:** worktree `/Users/yuema137/mineworld-worktrees/design-fe-b`, branch `docs/pr-design-fe-b`, audit
base `origin/main @ c373b802`. Docs-only.

---

## 1. Freeze questions (plain language, each with a recommendation)

Already decided and not reopened: 2000 kcal per world day (configurable); the new player starts with USD 10,000;
one inventory unit = one whole serving; new saves only (C-FE-2); no per-person kcal (Q-FE-6). Revision 2 answers the
red-team review `_reviews/design-fe-b.md` (FREEZE-AFTER-FIXES); disposition in §9.

**What residents really eat.** A hungry resident on the street walks to the first shop that has food, and that is the
café (lower place id), where the most filling item is the USD 8.00 grilled cheese. So the *likely* diet is café
sandwiches, about **USD 30.80 a day**; the *worst* diet (croissants) is USD 32.00 a day; a store-bread diet would be
USD 4.46 a day. Residents without a job have no income in this town.

| # | Question | Recommendation |
| --- | --- | --- |
| **Q-FEb-1** | **Chance buying: proof versus a livelier town.** Today residents sometimes buy a random thing on offer (coffee, tea, a newspaper). With USD 10,000+ wallets that has no upper limit, so no money guarantee is possible. Choose: **(A)** set it to 0 for Market Town (a `decisions.yaml`). Money becomes provable (§5.2). Cost: in headless or hosted runs **no resident ever buys coffee, tea or a newspaper**; the newsstand sells only to players. Hungry residents still buy food. **(B)** keep it and accept seed-only evidence (no proof). **(C)** a follow-up controller PR adds a per-person daily cap on chance spending, e.g. **USD 1/day**: about three resident coffees or newspapers a day town-wide, and the proof still holds (visitor to day 300 with USD 84 left; savers to day 757). C needs its own design before it ships. | **A now, and commission C as the next controller PR.** C3 records purchases by kind before/after so you see the loss in numbers (§5.4 FB-A1). |
| **Q-FEb-2** | **Black coffee and plain tea count as 0 kcal.** USDA: ~1 kcal per 100 g, ~4 kcal a cup; US labels round under 5 kcal to 0 (21 CFR 101.9(c)(1)). At 4–5 kcal, rule residents would buy coffee after coffee in a café whose food has sold out. Items are labelled "Coffee (black)" and "Tea (plain)". | **Yes, 0 kcal.** |
| **Q-FEb-3** | **"Bread" is a whole 20 oz loaf (~1 500 kcal), eaten at once** (up to ~300 kcal lost to a full bar). Store-fed residents buy loaves; most residents are café-fed and buy sandwiches. Alternative: a bakery roll (no public price; authored). | **Keep the loaf**, stated as a game simplification. |
| **Q-FEb-4** | **Price quality.** Coffee/tea: a readable San Diego County Starbucks-licensed menu (Palomar College; Jan 2025 template, file dated Sept 2025). Bread: BLS West average. Apple, milk, juice, soup: **Target national online prices, no store** (2026-11-23). Croissant and sandwich: San Diego café prices seen only in a search index; their kcal come from Starbucks recipes (a stand-in, not the same product). Cake price: **authored**. The first implementation commit re-checks them and may change a price only below the ceilings that keep the proof (croissant ≤ USD 4.15, sandwich ≤ USD 8.65, any food ≤ 1.66 ¢/kcal); otherwise it comes back to you. The README calls prices "dated US observations, contextual; cake authored", nothing stronger. | **Yes.** |
| **Q-FEb-5** | **Money, and what happens after day 300.** Choose: **(M1) savings**: the visitor keeps USD 10,000; every other resident without a job starts with **USD 25,000** savings; workers earn the San Diego minimum wage USD 17.75/h; shop companies are prefunded for **1 000 days** of wages (café USD 151,000; store and newsstand USD 89,000 each). Result: the visitor (no income) is guaranteed to day 312 (likely ~day 325 on café food, ~6 years on store bread); savers to day 780 worst / ~812 likely; workers and companies indefinitely / to day 1 000. After that a broke resident just stays hungry (no collapse; the controller has no policy for being broke). **(M2) jobs for everyone**: give jobless residents part-time shop jobs so money circulates; sustainable, but it changes routines (S35 rhythm tests) and is its own content design. **(M3)** keep USD 10,000 for all: jobless residents broke at ~day 325, newsstand stops paying Ivan at ~day 305. | **M1 now; M2 as a follow-up content design.** README states how long USD 10,000 lasts. |
| **Q-FEb-6** | **Shelves.** Each shop's most filling food is made in worst-case amounts (sandwich 66 and bread 33 per shift) so residents cannot starve each other; other kinds at ≤ 12 per shift for players. Unsold food piles up (a count); C3 prints shelf counts at days 30 and 300. A player can deliberately buy a shop out; that is allowed. | **Yes.** |
| **Q-FEb-7** | **Where the guarantee does not apply.** It covers rule-driven seats with Market Town's own `decisions.yaml`. A run with `--decisions` (which replaces the file), any `motivated` or LM seat, and human players are outside it. `motivated` seats may buy 0-kcal coffee when hungry (it treats any purchase as food); C3 records their runs without gating; fixing that is a motivated-controller follow-up. | **Accept the scope.** |
| **Q-FEb-8** | **Execution authority once frozen:** a fresh Claude Opus 5.5 session implements up to READY FOR OPERATOR REVIEW; the coordinator runs the heavy 300-day acceptance in a shared slot. No merge, no paid calls, no models. | **Yes.** |

## 2. Audit of the current state (base `c373b802`)

| Area | Path | Finding |
| --- | --- | --- |
| Food items | `worlds/market-town/items/{apple,bread,milk,juice,coffee,tea,croissant,cake,sandwich,soup}.yaml` | Unitless `needs: { nourishment: { hunger } }` (0–650). Ten kinds; no other food kinds exist. Non-food kinds sold: `newspaper` only. |
| Needs config | `worlds/market-town/configure/needs.yaml` | Unitless rates 120/30 per hour. FE-a's food-energy model (`hunger_model`, `energy_kcal_per_day`; `systems/needs/src/interactions.rs:12`) is on main but unused by any world. |
| People | `worlds/market-town/people/*.yaml` (12) | Wallets: Alice, Felix 20 000; everyone else 200 000 (visitor too). `needs: { hunger: 510, tiredness: 0 }` everywhere. Otto has needs but no seat. |
| Shops | `organizations/{cafe-company,corner-store,station-news}.yaml` | Café: coffee/tea/croissant/cake/sandwich/soup at 80/70/95/110/175/160, wallet 20 000, hours 05:30–22:00. Store: apple/bread/milk/juice 55/130/45/40, wallet 300 000, 06:00–23:00. News: newspaper 200, wallet 200 000, 06:00–18:00. |
| Jobs | `people/{alice,felix,ivan}.yaml` | Alice 05:30–14:00 at 120/h, Felix 08:00–13:00 at 200/h, Ivan 07:00–12:00 at 120/h. `produces` fixed per full shift. |
| Wage/production arithmetic | `systems/employment/src/component.rs:146–167` | Wage `⌊wage × worked ÷ 3600⌋`; production `⌊per_shift × min(worked, shift) ÷ shift⌋` per kind. No par-level restock, no money faucet. |
| Unpaid wages | `systems/economy/src/lib.rs` header | Economy pays a `wage-due` from the employer's wallet or states `wage-unpaid`. |
| Chance buying | `cognition/rule-controller/src/fetch.rs:405` `beyond_chance_budget`; `offered.rs:57–83`; `paced.rs:351` | The chance band attempts a random complete affordance on 20/100 consults; a `buy` is excluded when `price × 1000 > balance × chance_spend_permille` and the seat has `needs`. Exclusion happens **before** the choice, so `0` excludes every positive-price buy. The `pricing` buyer hook runs only on a chosen buy. Haggling affordances are incomplete and never chosen. |
| Fetch | `fetch.rs:215 best`, `:250–300` | Fires only when hunger ≥ `fetch_from` (600) and nothing nourishing is held; buys **one** unit of the most nourishing available kind (ties lowest ItemId); otherwise walks to the lowest-PlaceId shop whose disclosed stock has food for me; "nourishes me" = projected motivation > 0. |
| Policy wiring | `cognition/rule-controller/src/model.rs:33` `RuleParameters`; `tools/cli/src/decisions.rs` | A world's `decisions.yaml` (`world.yaml` key `decisions:`) can set `models: { rule: { parameters: { needs: { … } } } }`. Market Town has no decisions file today. `docs/MODULE_SPEC.md` §4.1 still says "`rule: {}` … no parameters" — **stale** (RuleParameters exists); fixed in C1 (documentation law §2.1 rule 4). |
| 300-day oracle | `tools/cli/tests/market_town.rs`, `tools/cli/tests/market/mod.rs:717 market_lives` | CP-4: every 30-day bucket ≥ 1 purchase, wage paid to each holder, ≥ 1 produced/consumed, every seat gave; whole run: zero `wage-unpaid`, no wallet below the cheapest genesis price, nobody above six items. |
| Needs oracle | `tools/cli/tests/needs_town.rs` | Replays **unitless** hunger from facts (`nourishment.unitless.hunger`, rates 120/30). Market Town switching to food-energy breaks this oracle; it must gain a kcal replay (§6 C2). Thresholds: famished = hunger ≥ 900 for > 24 h; asleep at 03:00 ≥ 850 ‰; ATUS eating TV ≤ 350 ‰; ≥ 2 nourishing items on days 2–3. |
| Literal content in tests | e.g. `tools/cli/tests/client_3d_market.rs:623`, `client_trade.rs:111,146,342` | Tests rewrite `"economy: { wallet: 200000 }"` by string `replacen`; changing wallets turns them into silent no-ops. 109 test/client files mention Market Town. C1 must grep every literal it changes. |

## 3. Product table (one inventory unit = the whole serving)

Observation dates are when the source was read. "Primary" = the publisher's own page/file. Prices are US retail asks
without tax or tip. kcal is integer, rounded half-up from the source arithmetic shown.

| Kind (shop) | Serving (label; quantity) | kcal | Source for kcal | Price (¢) | Source for price | Status |
| --- | --- | --- | --- | --- | --- | --- |
| apple (store) | one small Gala apple; 149 g | **77** | USDA FDC 171688 "Apples, raw, with skin", SR Legacy, pub. 2019-04-01: 52 kcal/100 g; portion "small (2-3/4" dia)" 149 g → 77.5 → 77. API read 2026-11-27. | **79** | Target "Gala Apple – each" (A-15014055), $0.79, described small; read 2026-11-23 (recovered evidence), no store/ZIP. | **Sourced — national online, no store** (size matched small ↔ small; closes the 2026-11-23 size mismatch). |
| bread (store) | one 20 oz white sliced loaf; 567 g | **1508** | USDA FDC 174924 "Bread, white, commercially prepared", 266 kcal/100 g × 567 g = 1508.2. (Cross-check: Wonder label 10 × 140 = 1400, Target 2026-11-23.) | **267** | BLS Average Retail Food Prices, **West region, Oct 2026**, "Bread, white, pan, per lb" $2.135 × 1.25 lb = $2.669 → 267. https://www.bls.gov/regions/mid-atlantic/data/averageretailfoodandenergyprices_usandwest_table.htm read 2026-11-27. (Cross-check: Target Wonder 20 oz $2.69.) | **Sourced — BLS West regional average** (generic ↔ generic). Q-FEb-3. |
| milk (store) | one 8 fl oz whole-milk carton; 236 ml | **150** | Horizon shelf-stable whole milk, 1 box 236 ml = 150 kcal (manufacturer page, recovered 2026-11-23). Cross-check USDA FDC 171265 61 kcal/100 g × 244 g = 149. | **155** | Target Horizon 6 × 8 fl oz $9.29 ÷ 6 = 154.8 → 155 (normalized multipack), 2026-11-23. (BLS West whole milk $4.546/gal → 28 ¢ per 8 oz is a bulk lower bound, not a single-serve ask.) | **Sourced — national online, no store; normalized multipack.** |
| juice (store) | one 8 fl oz apple-juice bottle; 240 ml | **120** | Mott's label 240 ml = 120 kcal (retailer transcription, 2026-11-23). Cross-check USDA FDC 167771 46 kcal/100 g × 248 g = 114. | **83** | Target Mott's 6 × 8 fl oz $4.99 ÷ 6 = 83.2 → 83, 2026-11-23. | **Sourced — national online, no store; normalized multipack.** |
| coffee (café) | one Tall black brewed coffee; 355 ml | **0** | USDA FDC 171890 "Beverages, coffee, brewed, prepared with tap water" 1 kcal/100 g × 355 g = 3.55 kcal; declared 0 under FDA label rounding (< 5 kcal → 0, 21 CFR 101.9(c)(1)(i)(C)). Starbucks posts 5 kcal for Tall Pike Place (recovered 2026-11-23). | **350** | Palomar College Comet Café (San Marcos, San Diego County) Starbucks licensed menu board, "Freshly Brewed Coffee" Tall $3.50. https://www.palomar.edu/foodservices/wp-content/uploads/sites/292/2025/10/Starbucks-Menu-Boards.pdf, board template "© 2025 Starbucks Corporation. 1/25" (Jan 2025), PDF metadata 2025-09-22 (UTC; 2025-09-21 PDT), sha256 `6614c03a…f5486a`, read 2026-11-27; drinks only, no food prices. | **Sourced — San Diego County, licensed Starbucks menu.** kcal per Q-FEb-2. |
| tea (café) | one Tall hot brewed tea, unsweetened; 355 ml | **0** | USDA FDC 173227 "Beverages, tea, black, brewed" 1 kcal/100 g → 3.55 → 0 by the same label rule; Starbucks Earl Grey 0 kcal (recovered 2026-11-23). | **325** | Same Palomar PDF: "Hot Brewed Tea Filterbags" Tall $3.25. | **Sourced — San Diego County, licensed Starbucks menu.** |
| croissant (café) | one butter croissant; 62 g | **250** | Starbucks US Butter Croissant 62 g = 250 kcal (starbucks.com nutrition, recovered 2026-11-23). | **400** | San Diego County café asks for "Butter Croissant" seen via search index 2026-11-27: Nostalgia Coffee Roasters $4.00, Toast Cafe (11455 El Camino Real) $4.00, Madeleine Cafe (1600 Pacific Hwy) $4.50, Nutmeg Bakery $5.00, Con Pane $5.45 (Toast pages; direct read refused HTTP 403). Chosen: lowest observed, $4.00. | **Contextual (recipe analogue)**: kcal identity Starbucks, price from independent SD cafés, snippet-level; re-check in C0; ceiling $4.15. |
| cake (café) | one slice iced lemon loaf; 115 g | **410** | Starbucks US Iced Lemon Loaf slice 115 g = 410 kcal (recovered 2026-11-23). | **425** | No San Diego match. Range seen: Olive Baking Co (SD) gluten-free lemon loaf $3.75 (search index 2026-11-27); Bentley Univ. Starbucks-licensed "Iced Cake" $4.25 (2025–26 PDF, MA); Disney California Adventure iced lemon pound cake $5.29 (fan guide). | **Authored estimate** (kcal: Starbucks recipe analogue); no SD price. |
| sandwich (café) | one grilled-cheese sandwich; 143 g | **520** | Starbucks US Crispy Grilled Cheese on Sourdough 143 g = 520 kcal (recovered 2026-11-23). Cross-check USDA FNDDS 2705804 grilled cheese, cheddar, white bread 368 kcal/100 g × 143 g = 526. | **800** | SD County café "grilled cheese" asks via search index 2026-11-27: Nostalgia Coffee Roasters $8.00, Toast Cafe $9.00, Higher Grounds (5887 Copley Dr) $9.75, BCB Café (SAN airport) plain $5.95. Chosen: $8.00. | **Contextual (recipe analogue)**: kcal identity Starbucks, price from independent SD cafés, snippet-level; ceiling $8.65. |
| soup (café) | one 18.6 oz can of chunky chicken-noodle soup, ready to serve; 527 g | **260** | Campbell's Chunky Classic Chicken Noodle, 260 kcal per whole container (manufacturer page, recovered 2026-11-23). | **249** | Target Campbell's Chunky 18.6 oz $2.49, 2026-11-23. | **Sourced — national online, no store** (the café resells the tin at retail, as the item already says). |
| newspaper (news) | — not food | — | — | 200 | Unchanged (Valley News 2024 single copy, already cited in `station-news.yaml`). | Unchanged. |

**Count:** prices — 3 rows from San Diego/West public sources (bread BLS West; coffee, tea Palomar), 4 rows from
Target national online with no store (apple, milk, juice, soup; milk/juice normalized multipacks), 2 contextual recipe
analogues (croissant, sandwich), 1 authored (cake). kcal — all ten sourced (USDA FDC or manufacturer; croissant, cake,
sandwich are Starbucks recipes standing in for local products). Rows read on 2026-11-23 come from the recovered
evidence and were not re-read; C0 re-reads them. Coffee and tea item names become "Coffee (black)" and "Tea (plain)".

Per-kcal price ρ = price ÷ min(kcal, 1200) (¢/kcal; §5.2 explains the 1200): apple 1.026, bread 0.223, milk 1.033,
juice 0.692, croissant **1.600 (max)**, cake 1.037, sandwich 1.538, soup 0.958. Coffee and tea feed nothing and are
never bought by a rule seat under Q-FEb-1.

Fetch ranking under D = 2000 (projection `ceil(1000·min(K,C)/C)`, FE-a §3.1): store bread 754 > milk 75 > juice 60 >
apple 39; café sandwich 260 > cake 205 > soup 130 > croissant 125. Coffee and tea project 0 (not food).

## 4. Authored content (C1)

- `configure/needs.yaml`: `hunger_model: food-energy`, `energy_kcal_per_day: 2000`; tiredness keys, thresholds and
  restore unchanged; the unitless hunger rates removed (FE-a refuses them in food-energy).
- Every person with `needs` (12, Otto included): `needs: { food_energy: { initial_kcal: 1500 }, tiredness: 0 }`.
  1500 keeps genesis (07:00) above the 800 kcal shopping line, so the town does not rush the shelves in its first
  minute; it lies between "full at 2000" and "last ate at 20:00" (2000 − 11 h × 83.3 = 1083).
- Ten item files: `nourishment: { energy_kcal, serving: { label, quantity: { unit, amount } } }` per §3; the comment
  in each file cites its row's sources and status word (Sourced / Contextual / Authored estimate).
- Shop prices per §3 (cents); newspaper unchanged; hours unchanged.
- Wallets (Q-FEb-5, option M1): visitor 1 000 000; Alice, Felix, Ivan 1 000 000; every other person (bob, carol,
  dev, erin, grace, hana, wanderer, otto) 2 500 000; café company 15 100 000; corner store 8 900 000; station news
  8 900 000 (each ≥ 1 000 full shifts of wages with no sales).
- Wages 1775 for Alice, Felix, Ivan (City of San Diego minimum wage 2026, https://www.sandiego.gov/sites/default/files/2025-12/2025-12-31-city-of-san-diego-hourly-minimum-wage-will-increase-to-17.75-effective-jan-1-2026.pdf).
  Full shift pays Alice ⌊1775 × 30 600 / 3600⌋ = 15 087; Felix and Ivan ⌊1775 × 18 000 / 3600⌋ = 8 875. 300 days:
  1 000 days: 15 087 000 / 8 875 000 / 8 875 000 ≤ the employer openings.
- Production per full shift and opening shelves (Q-FEb-6; arithmetic §5.3): café `produces` sandwich 66, cake 6,
  soup 6, croissant 6, coffee 12, tea 6; opening café holdings twice that. Store `produces` bread 33, milk 6,
  juice 6, apple 12; opening twice that. Newsstand unchanged. Only the two top kinds are sized by the proof; the rest
  are sized for players and as fallback food.
- New `worlds/market-town/decisions.yaml`: `models: { rule: { parameters: { needs: { chance_spend_permille: 0 } } } }`
  and `world.yaml` `decisions: decisions.yaml` (Q-FEb-1). `run`'s header gains the `; decisions <digest> (…)` clause.
- README "Food and money" rewritten from §3–§5 (human orientation; links here): prices are "dated US observations,
  contextual; cake authored"; the player has no income and USD 10,000 lasts ~325 days on café food, ~6 years on store
  bread; CHANGELOG entry: Market Town saves
  from before FE-b do not load (already true since FE-a/C-FE-2; restated for the content change).
- `docs/MODULE_SPEC.md` decisions example: `rule:` takes parameters (fix the stale line).
- `world.yaml` header text corrected: residents buy food when hungry; drinks and newspapers are bought by players
  (and by residents only once Q-FEb-1 option C lands).
- Item names: `Coffee (black)`, `Tea (plain)`.
- People's genesis holdings stay as they are (their kinds now carry kcal).

## 5. Liquidity and stock proof

### 5.1 Scope

Autonomous seats = the eleven seats driven by `rule` under the pack's `decisions.yaml` (the visitor included when no
client holds it). Otto (no seat) and any seat held by a human, an external agent or another model are outside the
guarantee; their purchases still pass through the same economy and are counted against stock in §5.3. Also outside:
any run with `--decisions FILE` (`tools/cli/src/decisions.rs:35` replaces the pack's file, restoring the global 1 ‰),
and every `motivated` seat (`cognition/motivated/src/considerations.rs:66,185`: `FOOD_OFFERS = ["eat","drink","buy"]`
— a hungry motivated seat values any buy, including 0-kcal coffee and newspapers). Existing Market Town tests that pass
`--decisions` (C0 lists them all; known: `needs_town.rs` AC-Nc3 all-motivated, `needs_controllers.rs` AC-Nc3-4,
`lateness.rs`, `punctuality.rs`, `price_reporter_lm.rs`, `motivated.rs`) keep their own files; their money outcomes
are recorded, not guaranteed (§5.4 FB-M1).

### 5.2 Liquidity theorem (per autonomous seat, every prefix of [0, T], T = 300 days)

Premises, each checked (C0 audit, C3 invariant):

- **P1** With `chance_spend_permille: 0`, every positive-price `buy` a rule seat submits is a fetch-band buy
  (`offered.rs` excludes them before choosing; no other band buys).
- **P2** A fetch buy happens only at hunger ≥ 600, i.e. reserve R ≤ 800 kcal (FE-a: hunger = ⌊1000·(C−R)/C⌋), and
  buys one unit.
- **P3** A unit a seat buys is consumed by the eat band (hunger ≥ 600, so headroom ≥ 1200) or not at all; the
  seat never gives it away while hungry (fetch needs "nothing nourishing held"; in `paced.rs::decide` the eat band,
  line 260, runs after only the asleep and due-commute bands, which give nothing away and only let R fall, and before
  every social, fetch and chance band). Useful kcal per purchase ≥ min(K, 1200), **except** after a food gift: if
  another seat gives me a more nourishing unit before I eat the one I bought, I eat the gift, may stop being hungry,
  and the bought unit can later be eaten with less headroom or given away. Each such case adds at most one unit's
  price, and only when a gift (free energy) arrived. The bound is therefore stated with G(t), the number of food units
  received as gifts by the seat up to t; FB-L1 gates on it, and FB-L2's sequencing check is reported, not gated.
- **P4** Prices never exceed the authored asks: no keepers, no person asks for food, the only modifier is the store's
  900 ‰ market-day discount.

Then useful energy over any prefix [0, t] is ≤ D·t + C − R₀ (reserve cannot exceed C), so money spent ≤
ρ_max·(D·t + C − R₀) + p_max·(1 + G(t)) (one unit possibly bought and not yet eaten, plus one per gift episode).
The figures below take G = 0; FB-L1 checks the bound with the measured G. With ρ_max = 1.600, D = 2000,
C = 2000, R₀ = 1500, p_max = 800: at t = 300 days the bound is 1.6 × 600 500 + 800 = **961 600 ¢ < 1 000 000 ¢**.
The visitor therefore keeps ≥ 38 400 ¢ at every prefix to day 300 (and is solvent to day 312 in the worst case);
savers (2 500 000) are solvent to day 780 worst case. The margin closes when
ρ_max > 1.6636, hence Q-FEb-4's ceilings (croissant ≤ 415, sandwich ≤ 865, any food ≤ 1.66 ¢/kcal).
Workers additionally earn up to 15 087 or 8 875 a day; before their first pay (≤ 7 h after genesis) the bound is
≈ 1.6 × (583 + 500) + 800 < 2 600 ¢.

**Employers:** with no receipts at all, each wallet covers 1 000 full shifts (§4), so `wage-unpaid` cannot occur
before day 1 000 and in the 300-day run no employer wallet falls below 10 573 900 ¢ (café), 6 237 500 ¢ (store,
news). Receipts only add. Money is conserved; nothing new creates it (FB-C1 checks it).

### 5.2a Horizons (what happens after the guarantee)

| Holder | Income | Worst case (croissant, 1.60 ¢/kcal) | Likely (café sandwich, 1.54 ¢/kcal) | Store bread (0.22 ¢/kcal) |
| --- | --- | --- | --- | --- |
| visitor (1 000 000) | none | day 312 | ~day 325 | ~day 2 240 |
| jobless residents (2 500 000) | none | day 780 | ~day 812 | ~15 years |
| Alice, Felix, Ivan | 15 087 / 8 875 a day | never while paid | never | never |
| café company | sales | wages to day 1 000 even with no sales; with sales, indefinitely | — | — |
| store, newsstand | sales | day 1 000 with no sales; newsstand has almost none under option A | — | — |

After a holder runs out, `fetch.rs` row 4 does nothing ("no policy for being broke yet"): the seat stays hungry, and
nothing collapses (S35 exclusion). Option M2 (jobs) or C (daily cap) are the follow-ups that turn this into a
circulating economy. C3 prints each seat's wallet and spend rate at day 300 and the projected insolvency day.

### 5.3 Stock argument (per shop, per day)

By P2/P3, between a seat's first and n-th fetch inside one day the reserve starts ≤ 800, gains ≥ (n − 1)·min(K,1200)
and loses ≤ 2000, and ends ≤ 800 again, so (n − 1)·min(K,1200) ≤ 2800. Per seat per day, buying kind K:
**u(K) = ⌊2800/min(K,1200)⌋ + 1** — sandwich 6, bread 3, cake 7, soup 11, croissant 12, milk 19, juice 24, apple 37.

- If all 11 autonomous seats fed only on a shop's top kind, they need 11·u per day: café sandwich 66, store bread 33.
  §4 produces exactly that per full shift, with two days' opening shelf P₀ = 2P. After n days with full attendance the
  shelf is ≥ 2P + nP − (n + 1)P = P > 0, so the top kind of **each** shop alone can feed every autonomous seat even if
  the other shop is closed, empty or bought out. One wholly missed shift can at worst bring that kind to 0 just before
  the next shift; the other shop's top kind still covers everyone then. Lower-ranked kinds are only bought when the
  top kind is out, so they need not meet the bound.
- Fetch walks to another shop when this one has no food for me (`fetch.rs` rows 6–7); both shops overlap
  06:00–22:00. Outside 22:00–05:30 (café) / 23:00–06:00 (store) no shop sells: the worst night gap is 23:00→05:30,
  6.5 h × 83.3 = 542 kcal, so a seat at ≤ 800 at 23:00 may reach ≈ 258 kcal (hunger 871) — not famished (≥ 900).
- Attendance: production is ⌊per_shift × worked ÷ shift⌋; C3 records every shortfall.
- Growth: unsold units accumulate (expected demand is well below the bound). Recorded, not gated (Q-FEb-6).
- A human/agent player can buy out a shop (6-item carry limit, but eat/drop/give lets them continue). That is
  allowed play; the guarantee is for autonomous seats against each other only.

### 5.4 Acceptance runs (thresholds fixed here, before any run; never retuned after a failure)

Run by the coordinator under `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh` (R-LOAD-1); CI on the
exact head is the gate.

| ID | Run | Invariant (every prefix, from the facts) |
| --- | --- | --- |
| FB-L1 | 300 days × seeds 1, 7, 42 | **Gated.** Every autonomous seat's cumulative spend ≤ ρ_max·(D·t + 500) + p_max·(1 + G(t)) at every purchase, G from replayed `items-transferred` gifts of food to the seat (theorem check; ρ_max, p_max from genesis `listed`). No wallet < 79 ¢. Zero `wage-unpaid`. G per seat printed. |
| FB-C1 | same, and every CP-4 run | **Gated.** Σ of all wallets (people and organizations), replayed from `funded` and `money-transferred`, equals the genesis sum after every fact; and equals the newest snapshot's stored wallets (extends CP-4's `state_matches_replay`). |
| FB-L2 | same | P1 **gated**: every buy by an autonomous seat is of a kind with energy_kcal > 0 and happens at replayed R ≤ 800. P3 sequencing **reported** (fact ids of every bought unit not eaten by its buyer before the next buy, and each one's gift cause). |
| FB-S1 | same | Replayed shelves, every instant: during 06:00–22:00 at least one of café sandwich / store bread > 0; café sandwich > 0 inside café hours and store bread > 0 inside store hours on every day with no production shortfall in the preceding two shifts. Per-shift shortfalls printed. Shelf counts at day 300 printed (not gated). |
| FB-N1 | same | Nobody autonomous at hunger ≥ 900 (R ≤ 200) for more than 24 consecutive hours (S35 AC-S35-8 threshold, unchanged). |
| FB-A1 | seed 7, 30 days, at base `c373b802` and at the FE-b head | **Recorded, not gated** (aliveness before/after, Q-FEb-1): purchases per day by kind and shop; non-food purchases (coffee, tea, newspaper) per 30-day bucket; gifts and consumptions per day. |
| FB-H1 | the 300-day seed-7 run | **Recorded:** each seat's wallet and mean daily spend over days 270–300, and the projected insolvency day; each employer's net flow per day. |
| FB-M1 | existing all-motivated Market Town acceptances (`needs_town.rs` AC-Nc3-1…3, `needs_controllers.rs` AC-Nc3-4) and every other Market Town test passing `--decisions` (C0 list) | Their own thresholds unchanged and gated as today. **Recorded:** non-food buys by motivated seats while hungry (the coffee loop), lowest wallet per seat. A FAIL of an existing threshold returns to the operator. |
| CP-4 | existing `market_town.rs` (seed 7 × 300 + 30-day set, kill-resume, seed 8) | Unchanged thresholds, new content. |
| S35 | existing `needs_town.rs` AC-Nd-2, AC-S35-5, AC-S35-8, AC-Nc3, AC-S35-1, AC-Nh-1 on Market Town with the kcal replay | Unchanged thresholds. A FAIL here (e.g. ATUS eating TV > 350 ‰ because meals now follow the reserve) is reported and returned to the operator, never fixed by retuning thresholds or content after the fact. |

### 5.5 Rejected alternatives

- **B: keep `chance_spend_permille: 1`, rely on runs.** Chance buys stop only below 1000 × price (≈ $790 for an
  apple), so a seat can lose ~92 % of $10,000 by chance; the remaining ~$790 cannot fund 300 days (≈ $9,600 worst
  diet). Seeds can show it did not happen; they cannot show it cannot.
- **Huge endowments (2026-11-23 Candidate B)**: Q × Pmax ≈ $215,000 per seat — not realistic, and still unbounded
  without P1.
- **C (follow-up, recommended next): a per-seat daily chance-spend cap** (new rule parameter, e.g.
  `chance_budget_per_day`), so spend ≤ ρ_max·(D·t + C − R₀) + p_max·(1 + G) + B·⌈t⌉. With B = 100 ¢/day the visitor keeps
  ≥ 8 400 ¢ at day 300 and savers are solvent to day 757. It is a rule-controller contract change, so it needs its own
  design; it is not in this content PR. A motivated-controller fix (only nourishing buys count as food) is the
  matching follow-up for Q-FEb-7.
- **M2 (follow-up): part-time jobs for jobless residents** so money circulates through the shops; changes routines
  and therefore the S35 rhythm evidence; its own content design.
- **D: a statistical solvency target** ("≥ X % solvent at day 300") — weakens NE-9; not proposed.
- **Coffee at 4–5 kcal**: ρ = 350/4 = 87.5 ¢/kcal; the bound becomes $525,000 and the coffee loop is real.

## 6. Commit plan

### C0 — Re-audit and source pinning (docs)

1. **Goal:** pin the base, re-verify every premise and every price before content is authored.
2. **Scope:** this document's §2/§3 evidence fields; `fe-evidence/fe-b-sources-<date>.md` (extracted facts, URLs,
   hashes; no copyrighted copies). No source code.
3. **Plan:** [ ] record base; [ ] re-diff §2 paths since `c373b802`, re-audit changed seams; [ ] re-read every §3 source
   reachable; try direct reads of the Toast pages; [ ] correct a price only within Q-FEb-4 ceilings, recording
   old/new; [ ] audit P1–P4 in code (band order in `paced.rs`, `needs.rs` eat before chance; no keepers; no person asks
   on food; P3 gift case written out); [ ] grep literal Market Town content in tests/clients (`200000`, `hunger: 510`,
   `nourishment: { hunger`, prices) and list each user; [ ] list every Market Town test that passes `--decisions` or
   `--seat-model`, its decision file, and the expected effect of FE-b on it (FB-M1).
4. **Validation:** document audit by the reviewer; arithmetic in §5 recomputed from the pinned table.
5. **Acceptance:** every row's status is Sourced/Contextual/Authored with date; ρ_max ≤ 1.6636; each premise has a
   code citation or is escalated.
6. **Failures:** a premise false in code → stop (material). A price outside ceilings → stop and ask.
7. **Evidence:** —  8. **Boundary:** docs only.

### C1 — Market Town content

1. **Goal:** author §4 exactly.
2. **Scope:** `worlds/market-town/{configure/needs.yaml, items/*.yaml (10), people/*.yaml (12), organizations/*.yaml (3),
   decisions.yaml (new), world.yaml, README.md}`, `docs/MODULE_SPEC.md` (decisions example), `CHANGELOG.md`. Tests whose
   literal strings C0 listed are updated in the same commit so none becomes a silent no-op.
3. **Plan:** [ ] items; [ ] needs config; [ ] people; [ ] orgs; [ ] decisions file + key; [ ] README/CHANGELOG/MODULE_SPEC;
   [ ] literal test updates.
4. **Validation:** `mineworld validate worlds/market-town`; `mineworld decisions worlds/market-town --json` shows
   `chance_spend_permille: 0` for every seat; targeted crates' tests that load Market Town (`food_energy`, `needs_sections`,
   `client_trade`, `client_3d_market`), via the slot wrapper.
5. **Acceptance:** loader accepts; each kind's `nourishment-assigned` carries the §3 kcal/serving; each shop's genesis
   `listed` prices equal §3; wallets/wages per §4.
6. **Failures:** a test pinned to old numbers → update with the reason in the commit, never delete the assertion.
7–8. Evidence recorded; content and its directly dependent fixtures only.

### C2 — Oracles for food-energy Market Town

1. **Goal:** make the existing S35 oracle read kcal and add the NE-9 proof checks.
2. **Scope:** `tools/cli/tests/needs_town.rs` (kcal replay for food-energy worlds, unitless kept for Commute Town;
   famished = R ≤ 200, i.e. hunger ≥ 900); new `tools/cli/tests/food_liquidity.rs` (FB-L1, FB-L2, FB-S1, FB-N1, reusing
   `market/mod.rs` `Ledger` and fact decoding; ignored, milestone); a fast default test computing §5.2's bound from the
   loaded genesis facts (prices, kcal, wallets, wages, production) and asserting margin ≥ 0 and the 11·u stock rule.
3. **Plan:** [ ] kcal replay with a literal unit test (FE-a NE-2 oracle values); [ ] the fast bound test; [ ] the ignored
   300-day checks, PASS/FAIL lines printed before asserts.
4. **Validation:** fast tests locally (slot wrapper); the bound test must FAIL on a scratch copy with croissant at 450
   (negative); the kcal replay checked against a 3-day run's reserve readings from the owner's observation.
5. **Acceptance:** oracles agree with the owner's disclosed reserve to ±1 kcal at sampled instants.
6–8. Test files only.

### C3 — Market Town acceptance (NE-3, NE-5, NE-7, NE-9)

1. **Goal:** real-path evidence on the real world.
2. **Scope:** NE-3 buy/eat at visitor 1 000 000 ¢ through the CLI/server; NE-5 save after purchase/meal/mid-depletion/
   sleep, close/reopen and kill/resume equality; NE-7 2D and 3D client rows (en, zh_Hans) and agent/LM rows on the real
   Market Town package (stills under `fe-evidence/fe-b/`), including a 0-kcal "Coffee (black)" row; a **player-broke
   row**: a visitor whose wallet is below the price sees `insufficient-funds` in 2D and 3D and nothing changes; the
   §5.4 runs by the coordinator (FB-L1, FB-C1, FB-L2, FB-S1, FB-N1, FB-A1, FB-H1, FB-M1, CP-4, S35).
3. **Plan:** [ ] NE-3/NE-5 tests on Market Town (extend `food_energy.rs`), incl. the insufficient-funds row; [ ] NE-7
   captures; [ ] request slot runs; [ ]
   record results per ID.
4. **Validation:** as listed; missing Godot → INCONCLUSIVE, never PASS.
5. **Acceptance:** every gated §5.4 ID PASS with numbers and every recorded ID's numbers in the evidence; else stop and
   report.
6–8. Evidence and tests only.

### C4 — Ledger and review

[ ] independent red-team review (NE-10: ownership, no money grants, no client rules, size growth, provenance); [ ]
findings disposition; [ ] step-fe §3/§4 state, README, progress page (primary).

## 7. Risks

- Visitor margin at day 300 is 3.8 % worst case (38 400 ¢); any price change re-runs the fast bound test.
- Under option A the newsstand and café drinks have no autonomous customers (FB-A1 quantifies it); option C is the fix.
- After day 312 (visitor) / 780 (savers) a holder may run out and stay hungry (§5.2a); M2/C are the fixes.
- `motivated` seats and `--decisions` runs are outside the guarantee; motivated may buy 0-kcal items when hungry.
- S35 rhythm thresholds may fail because meal timing now follows the reserve; that returns to the operator.
- Whole-loaf eating (Q-FEb-3) and shelf growth of the two top kinds (Q-FEb-6) are visible in play.

## 8. Size estimate

Content ≈ 30 YAML files (+ ~250 / − ~150 lines incl. comments); tests ≈ +850 lines (`food_liquidity.rs` ~500 incl. FB-C1/A1/H1,
needs_town kcal replay ~150, bound test ~150, NE-3 broke row ~50) and literal-fixture edits; docs ≈ +150. Five commits. No production Rust outside
tests.

## 9. Review disposition (red team, `_reviews/design-fe-b.md`, head `fe4b4f77`, FREEZE-AFTER-FIXES)

| # | Finding | Disposition in revision 2 |
| --- | --- | --- |
| 1 | Chance 0 kills newsstand/café-drink trade, unmeasured | Q-FEb-1 is now a choice A/B/C with numbers; C (daily cap) with its bound; FB-A1 before/after metric; `world.yaml` header corrected (§4). |
| 2 | 300-day horizon is a cliff; café-first diet ≈ worst case | Café-first diet stated (§1); Q-FEb-5 options M1/M2/M3 with insolvency days; M1 recommended (savers 2 500 000, 1 000-day employer prefunds); §5.2a horizons; FB-H1 projection; player-broke NE-3 row; README states the visitor's horizon. |
| 3 | Guarantee lost under `--decisions` and `motivated` | Q-FEb-7 and §5.1 name both; C0 lists every such test; FB-M1 runs the all-motivated acceptances and records the coffee loop. |
| 4 | P3 gift gap | P3 restated with G(t); FB-L1 gates on the bound with G; FB-L2 sequencing reported only. |
| 5 | Source labels overstate | §3 status words per row (national online, no store; regional average; recipe analogue; authored); PDF template Jan 2025 and metadata 2025-09-22 corrected. |
| 6 | Non-top shelves grow for no buyer | Non-top kinds cut to ≤ 12 per shift (coffee 12, others 6; apple 12); shelf counts at days 30 and 300 printed. |
| 7 | 0-kcal coffee OK | Names "Coffee (black)", "Tea (plain)"; 0-kcal capture row in NE-7. |
| 8 | No conservation test | FB-C1 gated wallet-sum invariant. |
