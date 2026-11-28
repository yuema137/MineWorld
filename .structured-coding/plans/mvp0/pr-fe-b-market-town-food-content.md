# PR FE-b — Market Town food-energy content: sourced prices and kcal, wallets, wages, stock, liquidity proof

**Lifecycle:** `DRAFT — FREEZE-READY` (2026-11-27). Not frozen. Implementation needs the operator's answers to §1 and
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
one inventory unit = one whole serving; new saves only (C-FE-2); no per-person kcal (Q-FE-6).

| # | Question | Recommendation |
| --- | --- | --- |
| **Q-FEb-1** | **Residents stop buying things "by chance".** Today a resident sometimes buys a random thing on offer (a coffee, a newspaper) if it costs at most 0.1 % of their wallet. With USD 10,000 wallets every item passes that test, and such purchases have **no upper bound** over 300 days, so no guarantee that a resident can still afford food is possible. Proposal: Market Town ships a `decisions.yaml` that sets the rule controller's `chance_spend_permille` to **0** for its residents. They still buy food whenever they are hungry, still give, pick up, eat and drink by chance; they no longer buy coffee, tea or newspapers by chance, so in headless runs only the player buys those. With this, "no resident runs out of money in 300 days" becomes arithmetic (§5.2), not hope. The alternative (keep 0.1 % and rely on a few seeded runs) can only show it did not happen on those seeds. Accept? | **Yes, set 0.** Alternative B (keep 1 ‰, empirical only) is weaker and is listed in §5.5. |
| **Q-FEb-2** | **Black coffee and plain tea count as 0 kcal.** USDA measures brewed coffee and tea at about 1 kcal per 100 g, i.e. about 4 kcal for a 12 fl oz cup; US food labels round anything under 5 kcal to 0 (21 CFR 101.9(c)(1)). Starbucks posts 5 kcal for a Tall Pike Place. If coffee counts as food (4–5 kcal), a hungry resident in a café whose food has sold out will buy coffee after coffee (USD 3.50 for 4 kcal) — a real money drain the current controller allows. Proposal: author both at **0 kcal**, as a US label would. They stay drinkable and sellable; they just do not feed. Accept? | **Yes, 0 kcal** (label rounding of the USDA value; documented as such). |
| **Q-FEb-3** | **"Bread" is a whole 20 oz sliced loaf (about 1 500 kcal).** That is how a corner store sells bread, and the price is a public BLS average (West region, Oct 2026). But it means a resident who buys bread eats the whole loaf at once (up to ~300 kcal is lost to the full bar), and because the rule controller always buys the most filling food, bread will dominate store purchases. Alternative: change the item to a single bakery roll (no matching public price, would be an authored estimate). Keep the loaf? | **Yes, keep the loaf**, stated plainly as a game simplification. |
| **Q-FEb-4** | **Café food prices come from San Diego County café menus as seen in a search index on 2026-11-27**, because the cafés' own ordering pages (Toast) refuse automated reading. Coffee and tea prices come from a readable San Diego County Starbucks menu PDF (Palomar College, Sept 2025). Croissant, cake and sandwich prices are therefore **contextual**, and the cake price has no San Diego match at all (it is an authored value inside the observed range). The implementation's first commit re-checks them and may correct a price only within the ceilings that keep the money proof true (§5.2: croissant ≤ USD 4.15, sandwich ≤ USD 8.65, any food ≤ 1.66 ¢/kcal); anything outside comes back to you. Accept these labels and that rule? | **Yes.** |
| **Q-FEb-5** | **Money and wages.** Every person starts with USD 10,000 (the player's amount, for simplicity and fairness); the three workers earn the City of San Diego minimum wage, USD 17.75/h (2026); each shop's company starts with enough cash to pay its worker's full wage for 300 days even if it sells nothing (café USD 46,000; store and newsstand USD 27,000 each). Nothing pays anyone money it did not earn after the start. Accept? | **Yes.** |
| **Q-FEb-6** | **Shelves are sized for the worst case and therefore grow.** Each shift makes enough of each shop's most filling food for *all* eleven active residents to live on that shop alone, so no autonomous resident can be left without food by other residents. Unsold food piles up (counts only; no save-size effect beyond a number). The framework has no "restock to a par level" rule today; adding one is a separate employment change. A player who deliberately buys out both shops can still empty them — that is the player's choice, and it is not prevented. Accept? | **Yes.** |
| **Q-FEb-7** | **Execution authority once frozen:** a fresh Claude Opus 5.5 session in its own worktree implements, validates locally within R-LOAD-1, commits, pushes, opens the PR and repairs CI up to READY FOR OPERATOR REVIEW; the coordinator runs the heavy 300-day acceptance in a shared slot. No merge, no paid calls, no models. Authorize? | **Yes.** |

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
| apple (store) | one small Gala apple; 149 g | **77** | USDA FDC 171688 "Apples, raw, with skin", SR Legacy, pub. 2019-04-01: 52 kcal/100 g; portion "small (2-3/4" dia)" 149 g → 77.5 → 77. API read 2026-11-27. | **79** | Target "Gala Apple – each" (A-15014055), $0.79, described small; read 2026-11-23 (recovered evidence), no store/ZIP. | **Sourced, matched** (small ↔ small; closes the 2026-11-23 size mismatch). |
| bread (store) | one 20 oz white sliced loaf; 567 g | **1508** | USDA FDC 174924 "Bread, white, commercially prepared", 266 kcal/100 g × 567 g = 1508.2. (Cross-check: Wonder label 10 × 140 = 1400, Target 2026-11-23.) | **267** | BLS Average Retail Food Prices, **West region, Oct 2026**, "Bread, white, pan, per lb" $2.135 × 1.25 lb = $2.669 → 267. https://www.bls.gov/regions/mid-atlantic/data/averageretailfoodandenergyprices_usandwest_table.htm read 2026-11-27. (Cross-check: Target Wonder 20 oz $2.69.) | **Sourced, matched** (generic ↔ generic). Q-FEb-3. |
| milk (store) | one 8 fl oz whole-milk carton; 236 ml | **150** | Horizon shelf-stable whole milk, 1 box 236 ml = 150 kcal (manufacturer page, recovered 2026-11-23). Cross-check USDA FDC 171265 61 kcal/100 g × 244 g = 149. | **155** | Target Horizon 6 × 8 fl oz $9.29 ÷ 6 = 154.8 → 155 (normalized multipack), 2026-11-23. (BLS West whole milk $4.546/gal → 28 ¢ per 8 oz is a bulk lower bound, not a single-serve ask.) | **Sourced, normalized multipack.** |
| juice (store) | one 8 fl oz apple-juice bottle; 240 ml | **120** | Mott's label 240 ml = 120 kcal (retailer transcription, 2026-11-23). Cross-check USDA FDC 167771 46 kcal/100 g × 248 g = 114. | **83** | Target Mott's 6 × 8 fl oz $4.99 ÷ 6 = 83.2 → 83, 2026-11-23. | **Sourced, normalized multipack.** |
| coffee (café) | one Tall black brewed coffee; 355 ml | **0** | USDA FDC 171890 "Beverages, coffee, brewed, prepared with tap water" 1 kcal/100 g × 355 g = 3.55 kcal; declared 0 under FDA label rounding (< 5 kcal → 0, 21 CFR 101.9(c)(1)(i)(C)). Starbucks posts 5 kcal for Tall Pike Place (recovered 2026-11-23). | **350** | Palomar College Comet Café (San Marcos, San Diego County) Starbucks licensed menu board, "Freshly Brewed Coffee" Tall $3.50. https://www.palomar.edu/foodservices/wp-content/uploads/sites/292/2025/10/Starbucks-Menu-Boards.pdf, PDF created 2025-09-21, sha256 `6614c03a…f5486a`, read 2026-11-27. | **Sourced.** kcal per Q-FEb-2. |
| tea (café) | one Tall hot brewed tea, unsweetened; 355 ml | **0** | USDA FDC 173227 "Beverages, tea, black, brewed" 1 kcal/100 g → 3.55 → 0 by the same label rule; Starbucks Earl Grey 0 kcal (recovered 2026-11-23). | **325** | Same Palomar PDF: "Hot Brewed Tea Filterbags" Tall $3.25. | **Sourced.** |
| croissant (café) | one butter croissant; 62 g | **250** | Starbucks US Butter Croissant 62 g = 250 kcal (starbucks.com nutrition, recovered 2026-11-23). | **400** | San Diego County café asks for "Butter Croissant" seen via search index 2026-11-27: Nostalgia Coffee Roasters $4.00, Toast Cafe (11455 El Camino Real) $4.00, Madeleine Cafe (1600 Pacific Hwy) $4.50, Nutmeg Bakery $5.00, Con Pane $5.45 (Toast pages; direct read refused HTTP 403). Chosen: lowest observed, $4.00. | **Contextual** (snippet-level; re-check in C0; ceiling $4.15). |
| cake (café) | one slice iced lemon loaf; 115 g | **410** | Starbucks US Iced Lemon Loaf slice 115 g = 410 kcal (recovered 2026-11-23). | **425** | No San Diego match. Range seen: Olive Baking Co (SD) gluten-free lemon loaf $3.75 (search index 2026-11-27); Bentley Univ. Starbucks-licensed "Iced Cake" $4.25 (2025–26 PDF, MA); Disney California Adventure iced lemon pound cake $5.29 (fan guide). | **GAP — authored estimate** inside the observed range. |
| sandwich (café) | one grilled-cheese sandwich; 143 g | **520** | Starbucks US Crispy Grilled Cheese on Sourdough 143 g = 520 kcal (recovered 2026-11-23). Cross-check USDA FNDDS 2705804 grilled cheese, cheddar, white bread 368 kcal/100 g × 143 g = 526. | **800** | SD County café "grilled cheese" asks via search index 2026-11-27: Nostalgia Coffee Roasters $8.00, Toast Cafe $9.00, Higher Grounds (5887 Copley Dr) $9.75, BCB Café (SAN airport) plain $5.95. Chosen: $8.00. | **Contextual** (snippet-level; ceiling $8.65). |
| soup (café) | one 18.6 oz can of chunky chicken-noodle soup, ready to serve; 527 g | **260** | Campbell's Chunky Classic Chicken Noodle, 260 kcal per whole container (manufacturer page, recovered 2026-11-23). | **249** | Target Campbell's Chunky 18.6 oz $2.49, 2026-11-23. | **Sourced** (retail can resold at retail; the café sells the tin, as the item already says). |
| newspaper (news) | — not food | — | — | 200 | Unchanged (Valley News 2024 single copy, already cited in `station-news.yaml`). | Unchanged. |

**Count:** 7 rows fully sourced (apple, bread, milk, juice, coffee, tea, soup; milk/juice via normalized multipacks);
2 contextual (croissant, sandwich: San Diego observations only through a search index); 1 gap (cake price, authored).
kcal is sourced for all ten. Rows read on 2026-11-23 come from the recovered evidence and were not re-read today.

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
- Wallets (Q-FEb-5): every person 1 000 000; café company 4 600 000; corner store 2 700 000; station news 2 700 000.
- Wages 1775 for Alice, Felix, Ivan (City of San Diego minimum wage 2026, https://www.sandiego.gov/sites/default/files/2025-12/2025-12-31-city-of-san-diego-hourly-minimum-wage-will-increase-to-17.75-effective-jan-1-2026.pdf).
  Full shift pays Alice ⌊1775 × 30 600 / 3600⌋ = 15 087; Felix and Ivan ⌊1775 × 18 000 / 3600⌋ = 8 875. 300 days:
  4 526 100 / 2 662 500 / 2 662 500 ≤ the employer openings.
- Production per full shift and opening shelves (Q-FEb-6; arithmetic §5.3): café `produces` sandwich 66, cake 12,
  soup 12, croissant 12, coffee 24, tea 12; opening café holdings twice that. Store `produces` bread 33, milk 12,
  juice 12, apple 24; opening twice that. Newsstand unchanged.
- New `worlds/market-town/decisions.yaml`: `models: { rule: { parameters: { needs: { chance_spend_permille: 0 } } } }`
  and `world.yaml` `decisions: decisions.yaml` (Q-FEb-1). `run`'s header gains the `; decisions <digest> (…)` clause.
- README "Food and money" rewritten from §3–§5 (human orientation; links here); CHANGELOG entry: Market Town saves
  from before FE-b do not load (already true since FE-a/C-FE-2; restated for the content change).
- `docs/MODULE_SPEC.md` decisions example: `rule:` takes parameters (fix the stale line).
- People's genesis holdings stay as they are (their kinds now carry kcal).

## 5. Liquidity and stock proof

### 5.1 Scope

Autonomous seats = the eleven seats driven by `rule` under the pack's `decisions.yaml` (the visitor included when no
client holds it). Otto (no seat) and any seat held by a human, an external agent or another model are outside the
guarantee; their purchases still pass through the same economy and are counted against stock in §5.3.

### 5.2 Liquidity theorem (per autonomous seat, every prefix of [0, T], T = 300 days)

Premises, each checked (C0 audit, C3 invariant):

- **P1** With `chance_spend_permille: 0`, every positive-price `buy` a rule seat submits is a fetch-band buy
  (`offered.rs` excludes them before choosing; no other band buys).
- **P2** A fetch buy happens only at hunger ≥ 600, i.e. reserve R ≤ 800 kcal (FE-a: hunger = ⌊1000·(C−R)/C⌋), and
  buys one unit.
- **P3** A unit a seat buys is consumed by the eat band (hunger ≥ 600, so headroom ≥ 1200) or not at all; the
  seat never gives it away while hungry (fetch needs "nothing nourishing held"; in `paced.rs::decide` the eat band,
  line 260, runs after only the asleep and due-commute bands, which give nothing away and only let R fall, and before
  every social, fetch and chance band). At most one bought unit is uneaten at any time. Useful kcal per purchase
  ≥ min(K, 1200).
- **P4** Prices never exceed the authored asks: no keepers, no person asks for food, the only modifier is the store's
  900 ‰ market-day discount.

Then useful energy over any prefix [0, t] is ≤ D·t + C − R₀ (reserve cannot exceed C), so money spent ≤
ρ_max·(D·t + C − R₀) + p_max (one unit possibly bought and not yet eaten). With ρ_max = 1.600, D = 2000,
C = 2000, R₀ = 1500, p_max = 800: at t = 300 days the bound is 1.6 × 600 500 + 800 = **961 600 ¢ < 1 000 000 ¢**.
Every non-worker, including the visitor, therefore keeps ≥ 38 400 ¢ at every prefix. The margin closes when
ρ_max > 1.6636, hence Q-FEb-4's ceilings (croissant ≤ 415, sandwich ≤ 865, any food ≤ 1.66 ¢/kcal).
Workers additionally earn up to 15 087 or 8 875 a day; before their first pay (≤ 7 h after genesis) the bound is
≈ 1.6 × (583 + 500) + 800 < 2 600 ¢.

**Employers:** with no receipts at all, each wallet covers 300 full shifts (§4), so `wage-unpaid` cannot occur and no
employer wallet falls below 73 900 ¢ (café), 37 500 ¢ (store, news) — all above the cheapest genesis price (79 ¢),
which is CP-4's existing bound. Receipts only add. Money is conserved; nothing new creates it.

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
| FB-L1 | 300 days × seeds 1, 7, 42 | Every autonomous seat's cumulative spend ≤ 1.6·(D·t + 500) + 800 at every purchase (theorem check). No wallet < 79 ¢. Zero `wage-unpaid`. |
| FB-L2 | same | P1: every buy by an autonomous seat is of a kind with energy_kcal > 0 and happens at replayed R ≤ 800. P3: the bought unit is consumed by its buyer before that buyer's next buy, at R ≤ 800. Any exception is a FAIL naming the fact ids. |
| FB-S1 | same | Replayed shelves, every instant: during 06:00–22:00 at least one of café sandwich / store bread > 0; café sandwich > 0 inside café hours and store bread > 0 inside store hours on every day with no production shortfall in the preceding two shifts. Per-shift shortfalls printed. Shelf counts at day 300 printed (not gated). |
| FB-N1 | same | Nobody autonomous at hunger ≥ 900 (R ≤ 200) for more than 24 consecutive hours (S35 AC-S35-8 threshold, unchanged). |
| CP-4 | existing `market_town.rs` (seed 7 × 300 + 30-day set, kill-resume, seed 8) | Unchanged thresholds, new content. |
| S35 | existing `needs_town.rs` AC-Nd-2, AC-S35-5, AC-S35-8, AC-Nc3, AC-S35-1, AC-Nh-1 on Market Town with the kcal replay | Unchanged thresholds. A FAIL here (e.g. ATUS eating TV > 350 ‰ because meals now follow the reserve) is reported and returned to the operator, never fixed by retuning thresholds or content after the fact. |

### 5.5 Rejected alternatives

- **B: keep `chance_spend_permille: 1`, rely on runs.** Chance buys stop only below 1000 × price (≈ $790 for an
  apple), so a seat can lose ~92 % of $10,000 by chance; the remaining ~$790 cannot fund 300 days (≈ $9,600 worst
  diet). Seeds can show it did not happen; they cannot show it cannot.
- **Huge endowments (2026-11-23 Candidate B)**: Q × Pmax ≈ $215,000 per seat — not realistic, and still unbounded
  without P1.
- **A controller change** (e.g. a reserved food budget for chance buys) would keep random buying with a proof, but it
  is a rule-controller contract change outside this content PR; possible later, separately designed.
- **Coffee at 4–5 kcal**: ρ = 350/4 = 87.5 ¢/kcal; the bound becomes $525,000 and the coffee loop is real.

## 6. Commit plan

### C0 — Re-audit and source pinning (docs)

1. **Goal:** pin the base, re-verify every premise and every price before content is authored.
2. **Scope:** this document's §2/§3 evidence fields; `fe-evidence/fe-b-sources-<date>.md` (extracted facts, URLs,
   hashes; no copyrighted copies). No source code.
3. **Plan:** [ ] record base; [ ] re-diff §2 paths since `c373b802`, re-audit changed seams; [ ] re-read every §3 source
   reachable; try direct reads of the Toast pages; [ ] correct a price only within Q-FEb-4 ceilings, recording
   old/new; [ ] audit P1–P4 in code (band order in `paced.rs`, `needs.rs` eat before chance; no keepers; no person asks
   on food); [ ] grep literal Market Town content in tests/clients (`200000`, `hunger: 510`, `nourishment: { hunger`,
   prices) and list each user.
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
   Market Town package (stills under `fe-evidence/fe-b/`); the §5.4 runs by the coordinator.
3. **Plan:** [ ] NE-3/NE-5 tests on Market Town (extend `food_energy.rs`); [ ] NE-7 captures; [ ] request slot runs; [ ]
   record results per ID.
4. **Validation:** as listed; missing Godot → INCONCLUSIVE, never PASS.
5. **Acceptance:** every §5.4 ID PASS with numbers; else stop and report.
6–8. Evidence and tests only.

### C4 — Ledger and review

[ ] independent red-team review (NE-10: ownership, no money grants, no client rules, size growth, provenance); [ ]
findings disposition; [ ] step-fe §3/§4 state, README, progress page (primary).

## 7. Risks

- Thin margin (3.8 %) on the visitor: any price change re-runs the fast bound test.
- S35 rhythm thresholds may fail because meal timing now follows the reserve; that returns to the operator.
- Bread dominance and whole-loaf eating (Q-FEb-3) are visible in play.
- Shelf growth (Q-FEb-6) is visible in shop listings over long runs.
- Removing chance buys (Q-FEb-1) means headless residents never buy coffee, tea or newspapers; CP-4's "≥ 1 purchase
  per bucket" is still met by food fetches.

## 8. Size estimate

Content ≈ 30 YAML files (+ ~250 / − ~150 lines incl. comments); tests ≈ +700 lines (`food_liquidity.rs` ~400, needs_town
kcal replay ~150, bound test ~150) and literal-fixture edits; docs ≈ +150. Five commits. No production Rust outside
tests.
