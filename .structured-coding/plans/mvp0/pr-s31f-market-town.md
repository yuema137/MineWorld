# PR S31-f — The Market Town composition: references, keepers, sellers, the price clerk, notices, pricing and haggling, and the 30-day convergence checkpoint

## DRAFT — not frozen

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S31/S29 lane design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s31f, branch docs/pr-design-s31f,
                        planning base origin/main @ 44f050e3 (#171 12d, #258 decision-log split and #261
                        ARC-153 merged)
Approved by / evidence: none yet. The step's rulings (Q-S31-1 … Q-S31-8, step-31 §18), S31-a's (Q-S31a-1 … 4)
                        and S31-b's (Q-S31b-1 … 4) bind and are not re-asked
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of S31-a, S31-b,
                        S31-c, S31-d, S31-h, S29-b, WE-a (#189) and S30-c (#280)
Start condition:        step-31 §16.5: "Nothing starts before Milestone F's checklist has been handed to the
                        operator (the S29 timing rule). Each PR starts only once its own PR design is
                        frozen." "Last: S31-f, after S31-a…d and S31-h, plus S30-c for §7.4 (otherwise §7.4
                        goes to S31-f2, which is also release-blocking)." State at 2026-10-18: Milestone F's
                        checklist NOT handed over; S31-a FROZEN, not implemented, no PR; S31-b FROZEN,
                        implementation in progress (mvp0/pr-s31b, no PR); S31-c, S31-d, S31-h DRAFT
                        (designs-s31c-h @ e6e89778); S29-b not designed; WE-a #189 open; S30-c #280 open
                        (so §7.4 goes to S31-f2, §5.15). Then the coordinator's dispatch
Execution contract:     §12 (filled; ends at MERGE-READY: pushed, `fast` green on the exact head, reported
                        with PR number and head SHA; the coordinator merges). This design document itself
                        ends at READY FOR OPERATOR REVIEW
Implementer:            Haiku-class for every commit (step-31 §16.2, the S31-f row: "content and measurement
                        against fixed criteria"). No independent reviewer per PR (speed first); acceptance
                        criteria and planted mutations run at the milestone
Lifecycle:              DRAFT
```

**Effort:** `mvp0`, release-blocking (Q-S31-1 re-ruled) · **Step:**
[`step-31-wealth-prices.md`](step-31-wealth-prices.md) (APPROVED 2026-10-15) §§1 (outcomes 2, 4, 6, 7, 8, 9), 6.1,
7.4 (outlined here, built in S31-f2), 8, 9, 10.2, 13, 14, 15 (AC-S31-5, 12, 13, 14, 16–18, 21; X-12), 16.2 (the S31-f
row), 16.5, 18 (Q-S31-3, Q-S31-4) · **Parent:** [`overall.md`](overall.md) · **Depends on:**
[`pr-s31a-currency-asks.md`](pr-s31a-currency-asks.md) (FROZEN), [`pr-s31b-notices.md`](pr-s31b-notices.md) (FROZEN;
its §10 R-S31f-b-1 binds this PR), `pr-s31c-price-survey.md` (DRAFT rev 2, designs-s31c-h), `pr-s31d-reporter-policies.md`
(DRAFT rev 1, commit 1817c712), `pr-s31h-haggling.md` (DRAFT rev 2, designs-s31c-h) · **Coordinates with:** S29-b
(the newsstand; not designed), WE-a (#189, open), S30-c (#280, open) and S30-d · **Followed by:** S31-f2 (the
`wealth-band` attribute, §5.15), then S23's release candidate · **Binding parents:** `CLAUDE.md` §§2, 3, 4 (items 1,
2, 5, 7, 8, 9, 10, 13, 15, 16); `docs/MODULE_SPEC.md` §4.1 (decision list, `ARC-84`); decisions `ARC-23`, `ARC-35`,
`ARC-36`, `ARC-38`, `ARC-61`, `ARC-83`, `ARC-84`, `ARC-111` … `ARC-118`, `ARC-153`.

**Decision ids.** This PR writes **one dated note under `ARC-35`** (`## ARC-35 note — check 3 admits S31's packs
and the decision list (S31-f)`, in `docs/decisions/ARC-35.md`), only if Q-S31f-1 is ruled as recommended. It writes
no new record. The sub-range given for new records (`ARC-186`, `ARC-187`) stays **unused**.

No hosted model is called by this design or by anything it specifies for CI.

### Local ids in this file

| Prefix | Meaning |
| --- | --- |
| `A-n` | audit anchors |
| `F-S31f-n` | findings |
| `SD-S31f-n` | bounded decisions |
| `I-f-n` | invariants |
| `AC-f-n` | acceptance criteria of this PR |
| `M-f-n` | planted mutations of this PR |
| `R-S31f-n` | risks |
| `R-<PR>-f-n` | requirements this PR places on another PR |
| `Q-S31f-n` | operator questions |
| `V1`, `V2`, `V3` | the second worlds and variants (§5.11) |

---

## 0. Binding inputs, exactly restated

- **Operator requirement, 2026-10-15 (step-31 §0.1, verbatim):** "我觉得财富也很重要，我们需要有一般等价物（钱），
  然后每个物品都有标价（不一定是全局统一的，可以是出售的人物决定）" and "必需的消耗品 食材等等，可以有市场建议价格
  （一个专门的npc负责统计，然后给出建议，这个npc后端可以是llm agent，也可以是确定性的算法什么的，这个也是模块化的，
  然后这种有时效性的公共信息可以发送到别的npc的inbox里，每个npc可以有订阅公共信息的属性）……有了trading机制我们这个
  世界才会更加真实". This PR is where the default town shows all of it at once.
- **Operator principles (2026-10-18, relayed in the brief).** One currency per world. Any seller sets its own price,
  and haggling exists. A price clerk publishes suggested prices; its backend is modular. Timely public information
  reaches subscribers' inboxes. Every town has a station (S29). Realistic defaults with cited sources. 2D and 3D
  first-class; every world has a far field; all three OSes; NPCs at the protagonist's bar; MCP agents play by the
  same path.
- **Charter, SPEED FIRST and no hard-coding (2026-10-18, overrides).** "我们不能为了让 default 小世界跑起来，就
  hardcode 一大堆东西，我们要把这些机制做的 robust 才行，不然换了一个世界就跑不了了". Every acceptance criterion
  names at least one second world or a generated variant. Per-PR validation is the build, the tests the commit
  writes, and `fast` CI; acceptance criteria and planted mutations run at the milestone.
- **Scope (step-31 §16.2, S31-f row).** "reference values with sources; keepers; Grace and two others list goods;
  reporter person, routine and book; `configure/notices.yaml`; subscriptions by tag; pricing policies in
  `decisions.yaml`; supply-shock fixture; criteria test update; the `wealth-band` attribute, bands and the
  `market-sampled` fixture (§7.4; S31-f2 if S30-c has not merged); haggling content: two sellers with
  `accept_at_least_permille`, two buyers with `haggle`; the content re-baseline." Depends on "S31-a…d, **S31-h**;
  S29-b if merged; S30-c for §7.4". Checkpoint "**AC-S31-13** (30-day convergence), AC-S31-12, AC-S31-14 (300 d in
  nightly), AC-S31-5, AC-S31-16…18, AC-S31-21". Adversarial "the AC-S31-12 removal runs; X-12". Implementer
  **Haiku**.
- **Q-S31-3 (ruled):** two recorded Market Town re-baselines, "S31-a (vocabulary) and S31-f (content), each one
  commit rebased last with a reason line".
- **Q-S31-4 (ruled):** the reporter is "an existing seated person without a job, not the S29-b seller … S31-f names
  them".
- **R-S31f-b-1 (S31-b §10, binding):** "Market Town's topic id is `prices.ingredients` (step-31 §8.5); its `per_day`
  reads as 'in any 24 h' (Q-S31b-4); its subscriptions use `defaults.by_tag` or sections until the S30 adoption PR."
- **R-S31f-c-1 (S31-c §10, draft):** the reporter "carries `price-survey: { kinds: { categories: [food, drink] } }`
  and a routine through the café, the store and the street; `price-survey` is appended after `weather`".
- **R-S31f-d-1 (S31-d §10, draft):** "Market Town's clerk uses `price-reporter` with the topic S31-b fixes; keepers
  carry a tag if the world wants only them to price (`sell.tags`); a second seller of at least one kind (or the
  supply signal) gives the suggestions something to move toward (F-S31d-9); the composition needs `ARC-35` check 3
  to admit `notices` and `price-survey` and their sections (F-S31d-10)."
- **R-S31f-h-1 (S31-h §10, draft):** "Market Town's haggling sellers and buyers use §5.12's option names. S31-f's
  re-baseline reason line names S31-h if Q-S31h-5 is answered (b)."
- **#261 (ARC-153):** the MCP agent surface is one `act` tool with typed kinds. Nothing in this PR adds an agent
  tool; S31-g's typed kinds are what an agent seat in Market Town uses.
- **Brief finding (2026-10-19):** the LM controller attempts only complete offers today. This PR's default town
  drives every seat with an in-process model (`rule`, `price-reporter`), so nothing here depends on an LM seat
  attempting an incomplete offer. The LM clerk swap (AC-S31-11) is S31-d's, through its filler registry.

---

## 1. Goal

After this PR, `mineworld run worlds/market-town --headless --seed 7 --days 30` shows a town that trades the way
the operator asked:

1. **Every kind has a typical worth.** Each of the twenty item kinds carries a reference value in US cents, with
   its source in the file. It is never charged.
2. **Sellers set their own prices.** The café and the store each have a keeper who may change the shop's asks
   while standing in it: Alice for the café, Felix for the store. Grace, Hana and Ivan sell goods they carry, at
   prices they authored. The café now also sells fresh juice beside the store's bottled juice, so one kind has two
   sellers and two prices.
3. **A clerk publishes suggested prices.** Carol, who already shops at the store every morning and lunches at the
   café, keeps a price book and posts a `prices.ingredients` bulletin four times a day. Her backend is the
   deterministic `price-reporter` model; one line in `decisions.yaml` makes it the LM.
4. **The bulletin reaches subscribers only.** The keepers, Bob, Ivan and the two visitor seats subscribe by their
   tags. Grace, Hana, Dev and Erin do not, and never receive it.
5. **Prices move toward the suggestions, and people haggle.** The keepers follow the current suggestion a bounded
   step at a time. Bob offers 85 % of an ask he finds too high, and Ivan 95 %; the keepers accept at 90 % or more.
   So Ivan's offers are accepted and Bob's are declined.
6. **The 30-day checkpoint measures it,** with criteria fixed in §8 before anything is run, and every criterion is
   also measured on a second world that shares no key, name or amount with Market Town (V2), so a pass cannot
   come from tuning to this town.

**A concrete day.** At 07:00 Carol walks into the store and surveys it: apples 100 (in stock 24), bread 300 (in
stock 2), juice 250. She witnesses Bob's bread purchase at 300. At 08:00 her model posts `bread 330 (basis 4),
apple 95 (basis 5), …`: bread is scarce, so its suggestion is 10 % over the median sale; apples are plentiful, so 5 %
under. At 08:15 Felix, standing in the store, reads the current bulletin and moves bread from 300 to 303. At 11:30
Ivan, in the café, finds the café's juice at 350 against a suggestion of 300 and proposes 332; Alice accepts (332 ≥
315). Bob proposes 297 for the same juice and is declined. No line of code knows any of these names.

---

## 2. Scope

### 2.1 In scope

1. **`worlds/market-town` content** (every change is a section a market pack owns, a configuration file, or the
   decision list):
   - `items/*.yaml`: `economy: { reference: <cents> }` on the twenty kinds, each with its source comment (§5.2);
   - `organizations/cafe-company.yaml`: `shop.keepers: [alice]`; `juice: 350` in `shop.prices`; `juice: 6` in
     `holdings` (§5.3);
   - `organizations/corner-store.yaml`: `shop.keepers: [felix]` (§5.3);
   - `people/alice.yaml`: `job.produces` gains `juice: 2` (§5.3);
   - `people/grace.yaml`, `people/hana.yaml`, `people/ivan.yaml`: `holdings` and `economy.asks` (§5.4);
   - `people/carol.yaml`: `price-survey: { kinds: { categories: [food, drink] } }` (§5.5);
   - `configure/notices.yaml` (new) (§5.6);
   - `decisions.yaml` (new) (§5.7);
   - `world.yaml`: `systems` appends `notices`, `price-survey`; `configure` appends `notices`; the key
     `decisions: decisions.yaml`; its header comment (§5.1);
   - `README.md`: two short sentences (the human README).
2. **`tests/acceptance/tests/ac1_composability.rs`**: check 3 admits S31's two packs, their configuration and
   sections, and the decision list (Q-S31f-1, §5.8).
3. **`tools/cli/tests`**:
   - `market/mod.rs`: the criteria update (job holders read from the town; sale and haggling tallies) (§5.10);
   - `market/variants.rs` (new): the scratch-copy writers for the supply-shock fixture and V1, and the generated
     world V2 (§§5.9, 5.11);
   - `market_composition.rs`: the section counts and the two new removal cases (AC-S31-12);
   - `market_convergence.rs` (new): AC-f-1 … AC-f-8.
4. **`scripts/baselines.txt`**: `[world market-town]` re-recorded with one `reason S31-f …` line (Q-S31-3).
5. **`docs/decisions/ARC-35.md`**: the dated note (Q-S31f-1) and the regenerated `docs/decisions/INDEX.md` if the
   index lists notes (C1 checks; it lists records only today).
6. **This document's ledger** and `handoff-s31f.md`.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Any change under `kernel/`, `contracts/`, `systems/`, `cognition/`, `server/`, `clients/`, `persistence/` | never in this PR (I-f-1). A mechanism this content needs and does not find is a material stop |
| The `wealth-band` attribute, the bands in `configure/economy.yaml`, the `market-sampled` fixture, AC-S31-16…18, X-12 | **S31-f2** (S30-c #280 is open; §5.15 outlines it; its own PR design follows S30-c's and S30-d's merges) |
| Tag-scoped entries in the `pricing:` block (Q-S31f-2) | S31-d (R-S31d-f-1); this PR only writes the YAML |
| The LM clerk swap (AC-S31-11), its filler | S31-d |
| Price tags, the sale panel, the notice board in 2D and 3D; AC-S31-15's drive | S31-e (it may use this PR's content: Grace sells flowers) |
| The agent view and `act` kinds; AC-S31-8 | S31-g |
| The station, the newsstand and its keeper | S29-a / S29-b (§5.12 states what S31-f does if they merged first) |
| A second topic (`text`, a town newspaper) | later content: no seat publishes `text` deterministically, so a topic nobody posts on would be dead weight (SD-S31f-9) |

### 2.3 Dependencies

- **Required before C0:** Milestone F's checklist handed over; this design frozen; S31-a, S31-b, S31-c, S31-d and
  S31-h merged (each supplies a mechanism this content names); the coordinator's dispatch.
- **Q-S31f-1 ruled** before C1 (the check-3 note); **Q-S31f-2 ruled and folded into S31-d** before S31-d's freeze
  (otherwise §5.7's fallback applies, recorded at C0).
- **Either order:** S29-b (§5.12), WE-a C4/C5 (§5.14 R-S31f-3). C0 records both.
- **Not waited for:** S30-c and S30-d (S31-f2 follows them).

---

## 3. Audit (`origin/main @ 44f050e3`, 2026-10-18)

### 3.1 Source anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | Market Town has 12 people, 11 seats (Otto unseated), 2 organizations, 24 item files: 6 food, 4 drink, 10 goods, and Social Café's 4 loose objects (`cafe-ball`, `cafe-box`, `street-ball`, `street-box`, `body:` only) | `worlds/market-town/world.yaml:141-191`; `items/*.yaml` | "Twenty kinds" are the 6 + 4 + 10; the loose objects get no reference |
| A-2 | `systems` ends `… economy, employment, consumption, calendar, weather`; `configure: [calendar, weather]`; no `decisions:` key | `world.yaml:54-105` | §5.1 appends |
| A-3 | Tags: Alice `barista, staff`; Felix `neighbour`; Bob `regular`; Carol, Dev, Erin, Otto `resident` (Dev also `runner`); Grace, Hana `commuter`; Ivan `passerby`; visitor, wanderer `visitor` | `people/*.yaml` `tags:` | Tags are Social Café's fields: check 3 forbids changing them (A-9). Every tag filter in this PR uses these existing tags |
| A-4 | Routines: Carol store 07:00, café 12:00, apartments 16:00; Felix store 08:00, café 13:00; Alice café 05:30, park 14:00; Bob café 07:00, park 11:00, store 15:00; Ivan street 07:00, café 11:00, park 15:00; Grace café 07:00, workplace 11:00; Hana workplace 08:00, café 15:00 | `people/*.yaml` `routine:` | Carol already walks store → (street) → café: the clerk needs **no routine edit** (check 3 would refuse one) |
| A-5 | Jobs: Alice (café 05:30–14:00, wage 120, produces coffee 4, tea 2, croissant 3, cake 2, sandwich 2, soup 2); Felix (store 08:00–13:00, wage 200, produces apple 4, bread 2, milk 2, juice 2) | `people/alice.yaml:63-69`; `people/felix.yaml:35-41` | The supply shock halves Felix's `bread: 2` (§5.9); the café's juice line adds `juice: 2` to Alice's (§5.3) |
| A-6 | Shops: café {coffee 300, tea 250, croissant 250, cake 400, sandwich 600, soup 500}, holdings {coffee 20, tea 10, croissant 10, cake 6, sandwich 6, soup 6}, wallet 20 000; store {apple 100, bread 300, milk 200, juice 250}, holdings {apple 20, bread 10, milk 10, juice 10}, wallet 300 000 | `organizations/cafe-company.yaml:8-18`; `corner-store.yaml:8-17` | Food and drink references are these prices (step §6.1) |
| A-7 | Holdings: Grace {candle, soap, umbrella}; Hana {bread, milk}; Ivan {apple, coffee, mug}; Erin {cake, flowers 2}; Dev {notebook, pen}; Carol {book, pen, tea}; Bob {apple, newspaper} | `people/*.yaml` `holdings:` | §5.4 sizes the sellers' stock within the cap of six |
| A-8 | Item files carry `tags`, `item: { category, name }` (12d added names) | `items/bread.yaml` etc. | §5.2 appends `economy:` after `item:` |
| A-9 | `ARC-35` check 3: `systems` = Social Café's list, the six market packs, then only `GENERIC_PACKS = ["calendar", "weather"]`; `configure` and `configure/` only those (S31-a's Q-S31a-1 ruling adds the market packs' configuration); every other `world.yaml` key equal to Social Café's; every person-file key Social Café has equal, every added key a market-pack section; the pack's top-level entries equal apart from `items/`, `organizations/`, `configure/`, `data/` | `tests/acceptance/tests/ac1_composability.rs:1032-1036, 1041, 1045-1102, 1104-1138, 1140-1206, 1210-1238, 1309-1412` | `notices`, `price-survey`, `configure/notices.yaml`, Carol's `price-survey:` section, the `decisions:` key and `decisions.yaml` are each refused today (F-S31f-1) |
| A-10 | `MARKET_PACKS` is also check 2's list: "only systems/ depends on a market pack; no code outside systems/, worlds/, tests/acceptance/ names one" | `ac1_composability.rs:68-75, 884-921` | Adding `notices`/`price-survey` to `MARKET_PACKS` would make S31-d's `cognition/price-reporter` (which depends on both) fail check 2. The admission belongs to check 3's allow-list, not to `MARKET_PACKS` (SD-S31f-1) |
| A-11 | The decision list: parameters are per model ("one parameter block per model per file"); an assignment is per seat or per class; absent file = every seat `rule`; the file is never seeded | `docs/MODULE_SPEC.md:707-760` (rules 1–7) | One `pricing:` block serves every `rule` seat; per-seat behaviour needs tag scoping (F-S31f-2) |
| A-12 | The market criteria: "exactly two job holders (alice and felix)"; a required tally per 30-day bucket of purchase, items-produced, items-consumed, wage paid to each holder, and `gave: <seat>` for **every** seat; no wallet below the cheapest genesis price; nobody above six | `tools/cli/tests/market/mod.rs:30-37, 525-690` | §5.10 makes the holder count read from the town's files (S29-b adds a third job) |
| A-13 | `market_composition.rs` counts each market section: `("economy", Some("economy"), 14)`, `("employment", Some("job"), 2)`, … | `tools/cli/tests/market_composition.rs:40-47` | References add 20 `economy:` sections (34); the two new packs get removal cases |
| A-14 | Long-run digests: `[world market-town]` six keys; a re-baseline adds `reason <id>: …` | `scripts/baselines.txt:1-5` and the section | C9 |
| A-15 | `tools/cli/tests/fixture/mod.rs` reads routines literally from a pack's people files (`assert_quiet_in`) | `tools/cli/tests/fixture/mod.rs:28-95` | Hosted tests over Market Town that assume quiet stretches read the files, not the decision list (R-S31f-5) |
| A-16 | No world has `decisions.yaml`; the only decision fixtures are `tools/cli/tests/fixtures/decisions/{memory.yaml, memory.json, social-cafe-seed-7-30-days.txt}` | `find worlds -name 'decisions*'` (none) | Market Town's file is the first reference-world decision list |
| A-17 | S30-c is PR #280, open; no `mineworld-attributes-model` crate on main | `gh pr list` 2026-10-18; `grep -r attributes-model --include=Cargo.toml` (none) | §7.4 goes to S31-f2 (step §7.4 "Placement") |
| A-18 | S30-c ruling Q-S30c-2: "an owner's attributes exist only where `attributes` is enabled" | `pr-s30c-attributes-pack.md` §5.10 | S31-f2's sampled path needs a fixture enabling `attributes` and S30-d's sampler (§5.15) |

### 3.2 Findings

- **F-S31f-1 — Check 3 refuses the composition as the step describes it** (A-9, A-10). The step (and S31-c, S31-d)
  put `notices` and `price-survey` in Market Town's `systems`, `configure/notices.yaml` beside it, a `price-survey:`
  section on the clerk, and a `decisions.yaml`. `ARC-35` check 3 refuses each. The fix cannot be "make them market
  packs", because `MARKET_PACKS` also drives check 2, which would then forbid `cognition/price-reporter` from linking
  `mineworld-notices` and `mineworld-price-survey` (S31-d §5.2). It is an operator-ruled measurement → **Q-S31f-1**
  (recommended: check 3's allow-list, not `MARKET_PACKS`, admits the two packs, their configuration and sections,
  and the decision list as host configuration).
- **F-S31f-2 — One `pricing:` block per model cannot express two haggle offers, or keepers that do not list**
  (A-11). S31-d's halves carry one `tags:` filter each (Q-S31d-4). AC-S31-21 needs both accepted and declined
  proposals. With one buyer offer and one seller threshold, the outcome is the same every time: either every
  proposal is accepted or every one is declined. The `list` rule has the same shape problem: it shares the `sell`
  half's tags, so a keeper who buys a scarf would list it, gain an own ask, and from then on stop pricing the shop
  (S31-d F-S31d-6, R-S31d-8). → **Q-S31f-2** (recommended: each half may be a list of tag-scoped entries, first
  match wins; a change of S31-d's parser only).
- **F-S31f-3 — "Bread production halved from day 10" has no data-only form.** A job's `produces` is genesis
  content; nothing changes it mid-run, and a resumed save refuses a changed World Pack. Building a scheduled
  production change would be new code in `employment`, outside this content PR. → **Q-S31f-3** (recommended: the
  shock fixture halves production from genesis; the measured window, days 20–30, is unchanged).
- **F-S31f-4 — The clerk needs no routine change if it is Carol** (A-4). Carol is seated, jobless, observant
  (persona `observant`), and her routine already runs store 07:00 → café 12:00, passing the street. S31-c (W2,
  SD-S31c-13) and S31-d (W2) measured her in that role. Any other jobless person would need a routine edit, which
  check 3 refuses. → **Q-S31f-4** (recommended: Carol; S29-b must not make her the newsstand seller).
- **F-S31f-5 — S31-d's checkpoint café juice is not sustainable content.** S31-d's W2 gives the café a juice ask
  of 350 and 10 juice with no production (F-S31d-9). In a 30-day town the café would sell out within days, after
  which its ask shows `in_stock: 0` forever, the supply signal reads permanent scarcity, and the "second seller"
  disappears. → **Q-S31f-5** (recommended: the café makes fresh juice: Alice's shift produces 2, the café starts
  with 6 and asks 350).
- **F-S31f-6 — Re-baselines.** Q-S31-3 ruled two. S31-h's Q-S31h-5 recommends a composition-only re-baseline in
  S31-h (economy's declaration moves `journal-300s` and `sealed-300s` with no fact moved). S31-f2 will meet the
  same: bands in Market Town's `configure/economy.yaml` change economy's configuration fact. Neither moves
  behaviour. → **Q-S31f-6** (recommended: the ruling counts behaviour re-baselines; composition-only lines are
  recorded separately and measured to move no fact or summary).
- **F-S31f-7 — Market Town will need S31-d's `price-reporter` in its decision list, so `mineworld run` and
  `server --town` read it** (A-11 rule 6). Every host path that runs Market Town without `--decisions` now runs
  Carol on `price-reporter` and every other seat on `rule` with the block. Hosted tests over Market Town that
  count requests, or assume quiet stretches, can move (R-S31f-5). C7 lists them.
- **F-S31f-8 — The newspaper's reference belongs to S29-b's cited price.** S29 §7.2 says the newsstand's price
  "is a realistic default with a cited source (a US daily single copy)". If S29-b merged first, `newspaper`'s
  reference equals its price; otherwise S31-f uses the target in §5.2 and S29-b adopts it (R-S29b-f-1).

### 3.3 What does not exist

No `decisions.yaml` in any reference world; no `configure/notices.yaml`; no reference values; no keepers; no person
asks; no supply-shock fixture; no second market world. Everything this PR names that is not on main comes from
S31-a … S31-d and S31-h, and C0 records each real name.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Choice | Why |
| --- | --- | --- |
| Reference values | Public retail and survey figures, cited per file (§5.2) | Realistic defaults; no data dependency |
| Scratch copies of Market Town | `market_composition.rs`'s copy pattern (`Scratch`, file edits, the real binary) | Already the pattern of S31-a C6, S31-c W2, S31-d W2 |
| The generated second world | S31-c's W5 builder (two shops in a square), extended (§5.11) | One generated-world writer, not two; C0 records whether S31-c made it a shared helper |
| A scenario or economics library (agent-based market frameworks such as Mesa) | **Declined** | MineWorld's own systems are the simulation; a framework would run a second model of the same town |

No dependency is added.

---

## 5. Design

### 5.1 The composition at a glance

```text
worlds/market-town/
  world.yaml            systems += notices, price-survey     configure += notices     decisions: decisions.yaml
  configure/
    notices.yaml        NEW  topic prices.ingredients, publisher carol; subscriptions by tag
    economy.yaml        S31-a's currency (unchanged here; S31-f2 adds wealth_bands)
  decisions.yaml        NEW  carol: price-reporter; rule's pricing block (keepers, sellers, hagglers)
  items/<20 kinds>.yaml       += economy: { reference }
  organizations/cafe-company  += keepers [alice]; juice ask 350; juice 6 held
  organizations/corner-store  += keepers [felix]
  people/alice.yaml           job.produces += juice 2
  people/carol.yaml           += price-survey section
  people/grace|hana|ivan      holdings resized; economy.asks
```

`world.yaml`'s `systems` tail becomes:

```yaml
  - calendar
  - weather
  # Public notices (`ARC-115`): topics, subscriptions by tag, and the board of current notices. It depends on no
  # other system and nothing depends on it. Leave it out, with `configure/notices.yaml`, and the town still trades;
  # nobody receives a bulletin and keepers hold their prices (AC-S31-12).
  - notices
  # The price book (`ARC-116`): Carol's notes of the asks she surveys and the sales she witnesses. It depends on
  # economy, item and presence. Leave it out, with Carol's `price-survey:` section, and nobody keeps a book, so the
  # clerk has nothing to post.
  - price-survey
```

`configure:` appends `- notices`. The manifest gains `decisions: decisions.yaml` with a comment: "Which decision model
drives each seat (`ARC-84`): read by hosts, never seeded. Delete this line and every seat is `rule` with no pricing."

### 5.2 Reference values (step §6.1; `ARC-113`)

**Rule.** Food and drink use their step-10 shop price, which was sized as US retail in cents (step §6.1; S31-a §5.10:
"Market Town's prices are US-sized in cents"). Goods use a US retail figure for one ordinary unit. Each item file
appends, after `item:`:

```yaml
# Reference value (economy, ARC-113): a typical worth in US cents, never charged.
# Source: <publisher or retailer>, <product or series>, <date of the figure>.
economy: { reference: 300 }
```

| Kind | Category | Reference (¢) | Source of the figure |
| --- | --- | --- | --- |
| apple | food | 100 | step-10 shop price (E-9 sizing); BLS CPI Average Price Data "Apples, Red Delicious, per lb." is the cross-check |
| bread | food | 300 | step-10 shop price; cross-check BLS "Bread, white, pan, per lb." |
| cake | food | 400 | step-10 shop price (café slice) |
| croissant | food | 250 | step-10 shop price |
| sandwich | food | 600 | step-10 shop price |
| soup | food | 500 | step-10 shop price |
| coffee | drink | 300 | step-10 shop price (café cup) |
| juice | drink | 250 | step-10 shop price (the store's bottle; the café's fresh juice is an ask, not a second reference) |
| milk | drink | 200 | step-10 shop price |
| tea | drink | 250 | step-10 shop price |
| book | goods | 1 800 | a US trade-paperback list price, typically $17.99–$18.99 |
| candle | goods | 1 000 | a mass-market jar candle at a US general retailer, about $10 |
| flowers | goods | 1 000 | a mixed grocery-store bouquet in the US, about $10 |
| mug | goods | 800 | a plain ceramic mug at a US general retailer, about $8 |
| newspaper | goods | 300 | S29-b's cited single-copy price if merged (F-S31f-8); else a US metropolitan daily's weekday single copy, about $3 |
| notebook | goods | 300 | a 70-sheet spiral notebook, about $3 |
| pen | goods | 150 | a single ballpoint pen bought singly, about $1.50 |
| scarf | goods | 2 500 | a basic knit scarf, about $25 (S31-d's W1 uses the same figure) |
| soap | goods | 200 | one bar of soap, about $2 |
| umbrella | goods | 2 000 | a compact folding umbrella, about $20 |

**Citation procedure (C2).** The goods figures above are targets. For each goods kind, C2 writes one concrete
public source in the comment: the publisher or retailer, the product or price series, and the date seen. If the
source's figure differs from the target by more than 25 %, the cited figure wins and the difference is a recorded
finding in §15. No figure is invented: a kind with no source found keeps its target and the comment says "target,
no public figure recorded", which is a finding. Food and drink cite step-10.

### 5.3 Shops, keepers, and the café's fresh juice (Q-S31-8; Q-S31f-5)

```yaml
# organizations/cafe-company.yaml (economy section; holdings section)
holdings: { coffee: 20, tea: 10, croissant: 10, cake: 6, sandwich: 6, soup: 6, juice: 6 }
economy:
  wallet: 20000
  shop:
    at: cafe
    prices: { coffee: 300, tea: 250, croissant: 250, cake: 400, sandwich: 600, soup: 500, juice: 350 }
    keepers: [alice]
# organizations/corner-store.yaml
economy:
  wallet: 300000
  shop: { at: store, prices: { apple: 100, bread: 300, milk: 200, juice: 250 }, keepers: [felix] }
# people/alice.yaml (employment section)
  produces: { coffee: 4, tea: 2, croissant: 3, cake: 2, sandwich: 2, soup: 2, juice: 2 }
```

- **Keepers are the people already working there.** Alice's shift is 05:30–14:00 in the café; Felix's 08:00–13:00 in
  the store (A-5). A keeper changes asks only while standing in the shop (S31-a SD-S31a-3), so prices move during
  shifts, as shelf tags do.
- **Why the café sells juice.** A suggestion only moves an ask when it differs from it. With one seller per kind and
  every sale at its ask, the median sale equals the ask (S31-d F-S31d-9), and only the supply adjustment can open a
  gap. Two sellers of one kind at different prices give the clerk two sale prices, a median between them, and both
  keepers a reason to move. Fresh juice in a café at a premium over bottled juice in a store is ordinary retail. The
  café produces it (2 a shift) so the second seller lasts the whole run (F-S31f-5). The ask, 350, is 1 400 ‰ of the
  reference, inside the policy's ceiling (2 000 ‰).
- **E-9 sizing.** The café gains revenue (juice) and pays the same wage; its wallet only grows faster. The store's
  juice sales may fall; its wallet (300 000) pays Felix 200 a day for far more than 300 days. AC-f-6 re-measures
  E-9 over 300 days.

### 5.4 Person sellers: Grace and two others (step §16.2 "Grace and two others list goods")

| Seller | Tag | Holdings (≤ 6) | Authored asks (¢) | Why this person |
| --- | --- | --- | --- | --- |
| Grace | `commuter` | candle 1, soap 2, umbrella 1, flowers 2 | flowers 1 000, candle 1 000, umbrella 2 000, soap 200 | the step's named seller; AC-S31-15 buys flowers from her |
| Hana | `commuter` | bread 1, milk 1, flowers 3 | flowers 900 | a second flower seller, cheaper than Grace: "two sellers, two prices" in the real town, so `buy.choose: cheapest` has something to choose |
| Ivan | `passerby` | apple 1, coffee 1, mug 2, candle 1 | mug 800, candle 950 | walks the street and the café all day, so he meets many buyers |

- **Every person ask is at or below its reference value:** a buyer haggles only when the ask exceeds its basis
  (S31-h §5.12), so person asks are bought or not, and haggling stays at the shops, where the keepers answer it
  (SD-S31f-4). A person's ask above its reference would collect proposals nobody answers, which expire after 15
  minutes with no fact.
- **Relisting.** The `list` rule (§5.7) lists any goods these sellers, and the `resident` seats Dev and Erin, come to
  hold at their reference value. Goods therefore circulate: a scarf the visitor gives Hana is on sale the next
  period.
- **No keeper sells on its own account** (S31-d R-S31d-8): Alice and Felix get no `asks:` and are outside the `list`
  entry (§5.7).

### 5.5 The clerk: Carol (Q-S31-4; Q-S31f-4)

- **Who.** Carol Mensah, seated, jobless, persona "observant". Her routine (unchanged) is store 07:00–12:00, café
  12:00–16:00, apartments from 16:00. She passes the street between them.
- **Her book** (`people/carol.yaml`, price-survey's section, appended after `economy:`):

  ```yaml
  # Market Town's price clerk (price-survey, ARC-116): she notes the asks she surveys and the sales she witnesses,
  # food and drink only. Window and depth are the pack's defaults (7 days, 32 notes per kind); she resurveys a
  # place at most once an hour, as a CPI data collector records an outlet once per visit.
  price-survey: { kinds: { categories: [food, drink] } }
  ```

- **Her backend.** `decisions.yaml` assigns `{ seat: carol, model: price-reporter }`. Its daily life is `rule`'s,
  with default pricing (S31-d SD-S31d-11): she never prices or lists. The LM swap is the same line with `model: lm`
  (S31-d AC-d-11, AC-S31-11).
- **What she knows.** Only what she perceives: the store's and café's asks and stock when she surveys, and the sales
  in the place she stands in. She never sees the café's morning sales or the store's afternoon sales. That is the
  realism the step asks for (step §9.2): the USDA AMS Market News reporter and the BLS CPI collector both publish
  from what they observed, not from a central ledger.

### 5.6 `configure/notices.yaml` and subscriptions by tag (R-S31f-b-1)

```yaml
# Market Town's public notices (notices, ARC-115). One topic: the clerk's price bulletin. Its publisher is Carol,
# whose seat runs the price-reporter model (decisions.yaml). The bulletin is advisory: a buyer always pays the
# seller's asking price (ARC-112).
topics:
  prices.ingredients:
    body: prices
    publishers: [carol]
    # A bulletin stays valid for 1 h to 48 h. The clerk posts with 12 h, and each new bulletin supersedes her
    # previous one, so a current suggestion exists from her first post onward (USDA AMS Market News publishes
    # daily reports that stand until the next).
    valid_for: { min: "1h", max: "48h" }
    # She posts four times a day; six in any 24 hours leaves room for a slot that falls exactly 24 h after
    # another (Q-S31b-4's rolling window) without a refusal.
    per_day: 6
defaults:
  # Subscriptions by tag (the attribute until S30's adoption PR, R-S30c-b-1): the two keepers, the two
  # hagglers and the visitor seats. Grace, Hana, Dev and Erin do not subscribe.
  by_tag:
    barista:   [prices.ingredients]   # Alice, the café's keeper
    neighbour: [prices.ingredients]   # Felix, the store's keeper
    regular:   [prices.ingredients]   # Bob
    passerby:  [prices.ingredients]   # Ivan
    visitor:   [prices.ingredients]   # the two player seats: a human or an agent sees the board from the start
```

- **Subscribers:** Alice, Felix, Bob, Ivan, visitor, wanderer; Carol receives each bulletin as its publisher.
- **Non-subscribers** (the negative arm of AC-S31-13 (b)): Grace, Hana, Dev, Erin, Otto.
- Nobody carries a `notices:` person section, so check 3 needs no section admission for `notices` (only for
  `price-survey`).

### 5.7 `decisions.yaml`: the clerk, pricing and haggling (Q-S31-5; Q-S31d-4; Q-S31f-2)

Written in the **tag-scoped entry** form Q-S31f-2 asks S31-d to accept: each half of the `pricing:` block is either
one entry (S31-d's current shape) or a list of up to 8 entries; a seat uses the first entry whose `tags` it meets;
an entry with empty `tags` meets every seat; a seat that meets none uses the half's defaults.

```yaml
# Market Town's decision list (ARC-84): which decision model drives each seat. Read by hosts, never seeded.
# Delete this file and the `decisions:` line in world.yaml, and every seat is `rule` holding its prices.
models:
  rule:
    parameters:
      pricing:
        sell:
          # The keepers follow the clerk's current suggestion a tenth of the gap at a time, every 4 hours, within
          # half and twice the kind's reference value, and accept a haggled price of at least 90 % of the ask.
          - tags: [barista, neighbour]
            policy: toward-suggestion
            every_minutes: 240
            step_permille: 100
            deadband_permille: 30
            floor_permille: 500
            ceiling_permille: 2000
            accept_at_least_permille: 900
          # The goods sellers keep their own prices and put on sale any goods they come to hold, at the
          # reference value.
          - tags: [commuter, passerby, resident]
            policy: hold
            list: { categories: [goods] }
        buy:
          # Bob offers 85 % of an ask that is above the current suggestion (or the reference value without one).
          - tags: [regular]
            choose: cheapest
            haggle: { offer_permille: 850, deadband_permille: 0 }
          # Ivan offers 95 %.
          - tags: [passerby]
            choose: cheapest
            haggle: { offer_permille: 950, deadband_permille: 0 }
          # Everyone else takes the cheapest available offer of the kind they chose to buy.
          - tags: []
            choose: cheapest
  price-reporter:
    parameters:
      topic: prices.ingredients
      first_minute: 480        # 08:00
      last_minute: 1200        # 20:00
      every_minutes: 240       # 08:00, 12:00, 16:00, 20:00
      valid_minutes: 720       # 12 h: the 20:00 bulletin stands until the next morning's
      min_basis: 3
      scarce_below: 3
      scarce_permille: 100
      plenty_above: 20
      plenty_permille: 50
default: rule
assign:
  - { seat: carol, model: price-reporter }
```

- **Who matches what.** Sell: Alice (`barista`) and Felix (`neighbour`) the keeper entry; Grace, Hana (`commuter`),
  Ivan (`passerby`), Dev, Erin (`resident`) the goods entry; Bob, the visitor seats: no sell entry (defaults:
  `hold`, no listing). Buy: Bob 850, Ivan 950, everyone else `cheapest`. Carol is on `price-reporter`, whose
  delegated daily life uses default pricing, so her `resident` tag has no effect.
- **The two haggling sellers** are the keepers (`accept_at_least_permille: 900`); **the two haggling buyers** are Bob
  and Ivan. Ivan's 950 ‰ offers clear 900 ‰ and are accepted; Bob's 850 ‰ offers fall short and are declined. Both
  meet Alice in the café (Bob 07:00–11:00, Ivan 11:00–15:00; Alice there until 14:00). A shop proposal needs only the
  buyer and the keeper in the shop (S31-h §5.9 row `accept-proposal`).
- **Why `deadband_permille: 0` for haggling.** A buyer haggles when the ask exceeds the basis at all. The keepers'
  own deadband (30 ‰) keeps a small gap open, the café's juice ask sits well above a two-seller median, and a
  plentiful kind's suggestion sits 5 % under its ask, so a zero deadband gives a steady flow of proposals without a
  rule that knows any kind.
- **Parameter sources.** The clerk's slots and validity follow the market-news practice above (daily reports,
  standing until the next). `step`, `deadband`, `floor` and `ceiling` are S31-d's defaults. The haggling shares are
  behavioural parameters chosen to straddle the threshold by construction, not physical constants; nothing in the
  real world fixes them, and AC-f-5 measures their effect.
- **Fallback if Q-S31f-2 is declined** (recorded at C0): one entry per half; the keeper entry takes `tags: [barista,
  neighbour, commuter, passerby, resident]` with `list: { categories: [] }` (no relisting; goods are sold from the
  authored asks only); Bob stays on `rule` with `haggle 850`, and Ivan is assigned `motivated` with its own block
  `haggle 950` (one block per model). That changes Ivan's daily life model, and AC-f-1 (e) and AC-f-5 are then
  measured as written, a FAIL recorded.

### 5.8 The check-3 amendment (Q-S31f-1; `ARC-35` note)

`tests/acceptance/tests/ac1_composability.rs`, check 3 only (check 1 and check 2 unchanged):

1. A new constant beside `GENERIC_PACKS`:

   ```rust
   /// Packs a world may enable after the six market packs, each with its configuration and the person sections it
   /// owns (ARC-35 notes of 2026-10-09 and of S31-f). They are not market packs: check 2's `MARKET_PACKS` is unchanged,
   /// so a controller crate may still depend on them.
   const ADMITTED_PACKS: [&str; 4] = ["calendar", "weather", "notices", "price-survey"];
   ```

   `compare_systems`, `compare_configure`, the `configure/` loop and the attachments filter use `ADMITTED_PACKS`
   where they used `GENERIC_PACKS` (the order rule "only after the six, each once" is unchanged).
2. `market_section(key)` also admits a section owned by an `ADMITTED_PACKS` pack (through the same
   `Capability::owning_section` catalog).
3. `compare_manifests` admits the key `decisions` in Market Town only (absent in Social Café), and the top-level
   entry it names (`decisions.yaml`) is admitted like `configure`.
4. The existing hand-written tests (`only_allow_listed_generic_packs_may_follow_the_six_and_only_after_them`,
   `the_world_delta_names_every_difference_by_file_and_key`) gain cases: `notices` before the six is named;
   `decisions` in Social Café is named; a `notices:` section on a Social Café-only key is named.

The note (`docs/decisions/ARC-35.md`): "Check 3 also admits, after the six market packs, `notices` and
`price-survey` with their configuration and owned sections, and the decision list (`decisions:` and its file), which
`ARC-84` makes host configuration, never world state. They are not added to `MARKET_PACKS`, so check 2 is unchanged.
The claim stays: Market Town is Social Café plus installed packs and configuration. Ruled Q-S31f-1."

### 5.9 The supply-shock fixture (AC-S31-13 (d); Q-S31f-3)

- **Writer.** `market/variants.rs::shock(copy: &Path)` edits a scratch copy of `worlds/market-town`: in
  `people/felix.yaml`, the `produces:` line's `bread: 2` becomes `bread: 1`. Nothing else. The test asserts the copy's
  file diff is exactly that one line.
- **From genesis** (Q-S31f-3). The measured window stays days 20–30, so the clerk's 7-day window has seen only shock
  days by then in both readings of the step.
- **Generic form.** The writer takes `(person file, kind, from, to)`, so V2's shock (§5.11) uses the same function with
  its own names. No Market Town key is inside the function.

### 5.10 The criteria test update (`market/mod.rs`; A-12)

1. **Job holders are read, not counted.** `Town::read` gains `jobs: BTreeSet<EntityId>`: the people whose file has a
   top-level `job:` key (the literal reader style of `fixture/mod.rs`). `market_lives` asserts `holders == town.jobs`
   ("every person the town's files employ is hired, and nobody else"), replacing "exactly two". S29-b's newsstand job
   then needs no edit here.
2. **Sales are named.** `MarketFact` gains `Sold { seller, buyer, item, price, place }` (S31-a mirrors it; C0 records
   whether S31-a's C5 already did) and `ProposalAccepted` / `ProposalDeclined` (S31-h's mirror, its C5). The bucket
   tally adds `sold` (all sales) and `sold by a person` (seller is a person); they are printed, not required per bucket
   (a 30-day bucket with no person sale is not a failure of E-9).
3. **Unchanged:** zero `wage-unpaid`, money conserved (S31-a's `genesis_money`), nobody above six, no wallet below the
   cheapest genesis price, every seat gives in every bucket. `PERSON_CAPACITY` stays 6.

### 5.11 Second worlds and variants (charter: no world-specific hard-coding)

| Id | What | Built by | Proves |
| --- | --- | --- | --- |
| **V1** | Market Town relabelled: currency `{ name: euro, symbol: "€", exponent: 2, symbol_at: after }`; topic `market.daily`; clerk Hana (her 08:00 `workplace` segment becomes `store`, Carol's `price-survey:` section moves to Hana, `publishers: [hana]`, `assign: hana`); subscriptions `by_tag` with `commuter` removed from the seller entry and Grace's asks moved to Ivan. Every other byte equal | `variants.rs::relabel` on a scratch copy; the test asserts the diff is exactly these edits | the content, not code, picks the clerk, the topic, the currency and the sellers (S31-d's W3 is the precedent) |
| **V2** | **Harbor Row**, a generated town sharing no key, name or amount with Market Town: places `square`, `bakery`, `kiosk`, `homes`; organizations `oven-coop` (shop at `bakery`: `rye 40`, `bun 25`; keeper `mara`) and `kiosk-co` (shop at `kiosk`: `tea 20`, `rye 46`; keeper `teo`); jobs `mara` (bakery 06:00–14:00, produces `rye 3, bun 2`) and `teo` (kiosk 07:00–15:00, produces `tea 4, rye 1`); clerk `ines` (jobless; routine bakery 06:30, kiosk 10:30, square 14:00, homes 18:00; `price-survey` on `food, drink`); goods sellers `odo`, `pim` (`lamp 150`, `soap 60`); buyers `quin`, `rafa`, `sol`; haggler buyers `rafa` (800 ‰) and `sol` (960 ‰); keeper threshold 880 ‰; currency `{ name: crown, symbol: "cr", exponent: 0, symbol_at: after }`; topic `market.harbor` (publisher `ines`, `per_day 8`, posts at 07:00, 11:00, 15:00, 19:00, valid 12 h); tags `baker`, `vendor`, `trader`, `haggler-low`, `haggler-high`, `townsfolk`; references equal genesis asks; wallets 5 000; wages 30. Systems: `presence, movement, schedule, item, inventory, item-transfer, economy, employment, consumption, notices, price-survey` | `variants.rs::harbor_row(dir)` writes it from scratch (extending S31-c's W5 builder); its `decisions.yaml` uses the same block keys with these tags | every Market Town criterion holds on a world written without any Market Town word, number or person; two sellers of `rye` replace the café's juice; production `rye 3` halves to `1` (the shock writer with its own names) |
| **V3** | Social Café and Lakeside as on the base | none | no S31 fact appears where the packs are not enabled (AC-S31-9's form, re-run with Market Town's new content) |

V2's numbers are chosen to differ from Market Town's on every axis (exponent, slots, thresholds, offer shares,
per-day cap, place count), not tuned to pass: they are fixed here, before any run (`ARC-23`).

### 5.12 The station and the newsstand (S29)

- **If S29-b merged first** (C0 records it): the newsstand's organization gets `keepers: [<its seller>]`; its
  seller's tag joins the keeper sell entry only if the seller subscribes (a keeper without a subscription has no
  suggestion and holds, which is correct for a goods-only shop; SD-S31f-6); `newspaper`'s reference equals S29-b's
  cited price (F-S31f-8). Carol's routine gains a station visit only if S29-a gave her one (it changes both towns'
  person files equally, so check 3 holds); S31-f never edits a routine.
- **If S29-b merges later:** it adds its keeper and reference itself (R-S29b-f-1) and records its own re-baseline.
- **The newsstand seller is never Carol** (Q-S31-4; R-S29b-f-1).

### 5.13 Both clients, the agent view, 3D and the far field

No client, server or agent code changes. The content makes S31-e's and S31-g's surfaces non-empty in the default
town: price tags at both counters and above Grace, Hana and Ivan; the café's juice at 350 beside the store's 250;
the notice board for the visitor seats (subscribed by tag); incoming proposals at the keepers' panels. An MCP agent
in a visitor seat sees the same records and acts through `act`'s typed kinds (`ARC-153`, S31-g). The 3D slice and
the far field are untouched (no geometry, no body, no place).

### 5.14 Persistence, determinism and the re-baseline (Q-S31-3; Q-S31h-5; Q-S31f-6)

- **Deterministic.** Every added behaviour comes from seeded in-process models (`rule`, `price-reporter`) and
  integer rules; a run twice gives byte-identical facts, journal and snapshots; kill-and-resume equals the
  uninterrupted run (AC-f-7).
- **What moves.** Market Town's six baseline keys move: genesis gains references, keepers, asks, `listed`, notices'
  configuration and subscriptions, Carol's book; behaviour gains surveys, bulletins, price changes, person sales,
  proposals. Social Café, Lakeside and bodies-yard do not move (AC-f-8).
- **The re-baseline is C9, rebased last,** from a CI long record of the PR's head after a rebase onto the then main
  (nightly `parity-long` dispatched on the branch, `ci_parity.py baseline write`), with the reason line:

  ```text
  reason S31-f (step-31 §16.2, Q-S31-3, Q-S31f-6): Market Town's trading content — reference values, shop keepers,
  the café's fresh juice, Grace's, Hana's and Ivan's asks, Carol's price book and bulletins (notices, price-survey,
  price-reporter), pricing and haggling in decisions.yaml; every key re-recorded from <run id, commit, platform>
  ```

- **Counting re-baselines (Q-S31f-6).** S31-a's (vocabulary) and S31-f's (content) are the two behaviour
  re-baselines Q-S31-3 ruled. S31-h's (Q-S31h-5 (a)) and S31-f2's are **composition-only**: each is measured to move
  no fact and no summary, only `journal-300s` and `sealed-300s` through the genesis declaration. If Q-S31h-5 is
  ruled (b) instead, S31-h has no line and this reason line also names S31-h (R-S31f-h-1). If WE-a C5 lands between,
  its own line is recorded by WE-a (R-S31f-3).

### 5.15 S31-f2 — the `wealth-band` attribute (outline; its own PR design follows)

S30-c (#280) is open, and S30-c ruled that an owner's attributes exist only where `attributes` is enabled (A-18).
The step therefore moves §7.4 to **S31-f2**, which is release-blocking. Its scope, fixed here so nothing is lost:

- **Economy (generic code, no Market Town knowledge):** `wealth_bands` in `configure/economy.yaml` with the five named
  refusals (`economy-bands-count`, `-first-not-zero`, `-unordered`, `economy-band-opening-outside`,
  `economy-band-id-duplicate`); the shipped declaration `economy.wealth-band` (group `background`, `category` over
  `economy.wealth-bands`, private, derived, owner economy); `AttributeOwner::project` = the band of §7.1 wealth;
  seeding by `attributes-handed` for people without `wallet` (`funded { balance: band.opening }`), refusal
  `economy-wallet-unresolved`; `ARC-114` note.
- **Market Town's bands** (content), with sources:

  ```yaml
  wealth_bands:
    # Below $400: cannot meet a $400 emergency expense from cash (Federal Reserve, Economic Well-Being of U.S.
    # Households in 2023, May 2024: 63 % of adults could).
    - { id: modest,      from: 0,      opening: 20000 }
    # $400 to $8,000: up to the median value of US families' transaction accounts (Federal Reserve, Survey of
    # Consumer Finances 2022, Federal Reserve Bulletin, October 2023: $8,000).
    - { id: comfortable, from: 40000,  opening: 200000 }
    - { id: wealthy,     from: 800000, opening: 2000000 }
  ```

  Market Town's authored wallets (20 000, 200 000) lie in `modest` and `comfortable` and do not move (step §7.4).
- **Fixture `market-sampled`**: Market Town's people without `wallet`, `attributes` enabled, an S30-d profile that
  samples `economy.wealth-band`; AC-S31-16 … 18 and X-12; its second world is V2 with bands in crowns
  (`{0, 15 000, 60 000}` with openings `{5 000, 30 000, 120 000}`).
- **Depends on:** S30-c and S30-d merged, S31-f merged. Implementer: Opus-class for the economy owner commit (owner
  interface and seeding), Haiku for content and fixtures. Composition-only re-baseline of Market Town (Q-S31f-6).

---

## 6. Invariants this PR holds

| Id | Invariant | Checked by |
| --- | --- | --- |
| I-f-1 (I-S31-1, charter) | **Content only.** The diff touches only `worlds/market-town/`, `tools/cli/tests/{market/, market_composition.rs, market_convergence.rs}`, `tests/acceptance/tests/ac1_composability.rs`, `scripts/baselines.txt`, `docs/decisions/ARC-35.md` (+ `INDEX.md` if generated), this document and `handoff-s31f.md` | `git diff --stat main...HEAD` at C10 |
| I-f-2 (charter) | **No code knows Market Town.** `git diff main...HEAD -- systems cognition server clients kernel contracts persistence` is empty, and no test helper outside `market/` names a Market Town key | the diff; `grep` of `variants.rs::shock` and `harbor_row` for Market Town keys (only `relabel` may name them) |
| I-f-3 (I-S31-2) | Single ownership unchanged: the content assigns no state to a second writer | review of each YAML against its owner's section (S31-a/b/c tables) |
| I-f-4 (I-S31-7, I-S31-9) | Bulletins reach exactly the subscribers by tag plus Carol; no non-subscriber's perceived stream holds one | AC-f-2 |
| I-f-5 (I-S31-10) | Removing `notices`, `price-survey`, the decision list, or Carol's publisher entry leaves a town that trades | AC-f-4 |
| I-f-6 (I-S31-9 of the step, AC-S31-9) | Social Café, Lakeside and bodies-yard baselines do not move | AC-f-8 |
| I-f-7 (`ARC-23`) | Every criterion in §8 is fixed before measuring; a miss is a recorded FAIL and a material stop, never a re-tuned run | §15 evidence |

---

## 7. Test ownership

| Level | What | Where |
| --- | --- | --- |
| Composition (structural) | check 3 with the amendment; its hand-written cases | `tests/acceptance/tests/ac1_composability.rs` |
| Real binary, short | `validate` of Market Town, V1, V2 and the shock copy; the removal cases | `tools/cli/tests/market_composition.rs`, `market_convergence.rs` |
| Real binary, 30 days (milestone) | AC-f-1 … AC-f-5, AC-f-7 on Market Town, the shock arm, V1, V2 | `tools/cli/tests/market_convergence.rs` |
| Real binary, 300 days (milestone and nightly) | AC-f-6 (E-9) on Market Town and V2 | `tools/cli/tests/market_town.rs` (existing 300-day case) and `market_convergence.rs` |
| Long-run digests | the re-baseline; AC-f-8 | `scripts/baselines.txt`, nightly `parity-long` |

Per commit: the build and the test file the commit writes, compiled; the 30- and 300-day runs execute at the
milestone (charter).

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`)

Every run is seed 7 unless stated. "MT" is `worlds/market-town` at the PR head; "shock" its §5.9 copy; V1, V2, V3 as
in §5.11. Each criterion names its second world.

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-f-1 | **Convergence checkpoint (MT, 30 days).** (a) at least 20 `notice-published` on `prices.ingredients` by Carol, no refused `publish-notice` in the journal; (b) Bob's perceived stream (agent library `perceived`) holds a bulletin, `mineworld-memory inspect` for Bob shows it, Bob's `notice-board` record at an instant inside its validity lists it and at `valid_until` (and after a later bulletin) does not; Grace's, Hana's, Dev's and Erin's perceived streams hold none; (c) at least two keeper `price-set` facts (Alice's or Felix's, `for: shop`) each move toward the suggestion current at that instant (computed from the facts), none moves away, and each `to` lies within `[500 ‰, 2 000 ‰] × reference`; (d) **shock vs control**: the mean of bread's suggested price over the bulletins of days 20–30 in the shock run exceeds the control run's (MT), and the store's bread ask at day 30 in the shock run is ≥ the control's and ≤ 2 000 ‰ × reference; (e) at least 10 `sold` facts whose seller is a person. **Second world:** V2 meets (a) with ≥ 20 on `market.harbor`, (b) with `rafa` subscribed and `odo` not, (c), (d) with `rye` and `mara`'s halved production, and (e) with ≥ 5; V1 meets (a) and (b) with Hana as clerk on `market.daily` | AC-S31-13; R-S31-3 |
| AC-f-2 | **Audience by tag (MT and V2).** Every `notice-published` audience equals the set of persons whose tags meet `defaults.by_tag` (from the files, by a literal reader) at genesis, adjusted by later `subscribed`/`unsubscribed`, plus the publisher | I-S31-7; R-S31f-b-1 |
| AC-f-3 | **Prices change, by `price-set` only (MT and V2, 30 days).** At least one keeper ask changes; every change of a shop's or a person's ask between two observations is explained by a `price-set`, `listed` or `withdrawn` fact | AC-S31-5 |
| AC-f-4 | **No reporter, still trading (MT and V2, 30 days each).** Four variants: (i) Carol's `publishers` entry replaced by another person who is not on `price-reporter` (the topic stays); (ii) `notices` removed with `configure/notices.yaml`; (iii) `price-survey` removed with Carol's section; (iv) `decisions:` and `decisions.yaml` removed. In each: E-9's criteria hold (`market_lives`), at least one `sold`, no `notice-published` in (i)–(iii), no `price-set` caused by a suggestion in (i)–(iv), and (iv) has no `price-set`, `price-proposed` or `listed` after genesis | AC-S31-12; I-S31-10 |
| AC-f-5 | **NPCs haggle (MT, 30 days).** At least 5 `proposal-accepted` and at least 5 `proposal-declined`; every accepted price ≥ `floor(ask × 900 / 1000)` of the ask current at the proposal; every proposer is Bob or Ivan; every answerer is a keeper; E-9 holds. **Second world:** V2 with ≥ 3 of each, threshold 880 ‰, proposers `rafa`, `sol` | AC-S31-21 |
| AC-f-6 | **E-9 over 300 days (MT and V2; milestone and nightly).** `market_lives` passes with the updated criteria (§5.10): zero `wage-unpaid`, money conserved, nobody above six, no wallet below the cheapest genesis price, every seat gives in every 30-day bucket, every authored job holder hired | AC-S31-14 |
| AC-f-7 | **Determinism (MT 10 days, V2 10 days).** Two runs give byte-identical facts, journal and snapshots; killed at day 5 and resumed equals uninterrupted | step §13; AC-6 |
| AC-f-8 | **Worlds without the packs do not move (V3).** Social Café, Lakeside and bodies-yard keys of the PR's long record equal `scripts/baselines.txt` on main; none states a fact of `notices`' or `price-survey`'s vocabulary | AC-S31-9 |
| AC-f-9 | **Composition (check 3).** `check_3_the_world_delta` passes on MT; its new hand-written cases fail as named on planted inputs (§5.8 item 4); `check_2_the_dependency_structure` passes unchanged | `ARC-35`; Q-S31f-1 |
| AC-f-10 | **Content is sourced and bounded (MT and V2).** Every one of the 20 kinds has a reference with a source comment; every authored ask lies in `[1, PRICE_MAX]` and in `[500 ‰, 2 000 ‰] × reference` (the policy's bounds, so no seller starts outside them); no person's genesis holdings exceed six | step §6.1; realistic defaults |

### 8.1 Planted mutations (each must turn its test red by name, then be restored from a byte copy; run at the milestone)

| Id | Plant | Must fail |
| --- | --- | --- |
| M-f-1 | `decisions.yaml`: the keeper entry's `policy: hold` | AC-f-1 (c), AC-f-3 |
| M-f-2 | `variants.rs::shock` returns the copy unedited | AC-f-1 (d) (the shock and control means are equal) |
| M-f-3 | `configure/notices.yaml`: add `resident: [prices.ingredients]` | AC-f-1 (b) negative arm (Dev's and Erin's streams hold a bulletin) |
| M-f-4 | `decisions.yaml`: Bob's `offer_permille: 950` | AC-f-5 (no `proposal-declined`) |
| M-f-5 | `market/mod.rs`: `town.jobs` read as empty | AC-f-6 (`hired` set differs) |
| M-f-6 | `ac1_composability.rs`: drop `price-survey` from `ADMITTED_PACKS` | AC-f-9 (check 3 names `price-survey`) |
| M-f-7 | `variants.rs::harbor_row` writes `carol` as its clerk key | the V2 key scan of I-f-2 |
| AC-S31-12 removal runs | the four variants of AC-f-4 | step §16.2 "Adversarial" |

X-12 belongs to S31-f2 (§5.15).

---

## 9. Commit plan

### 9.0 The procedure for every commit (Haiku implementer: follow it literally)

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s31f`. Never `cd` elsewhere.

1. Re-read this commit's section and every design section it cites. Re-read every file you will edit, in full, before
   editing it.
2. Make the numbered steps, in order. Do not do a later commit's work. If a step names a file, a key, a field or a
   function that does not exist on the base, stop and record it (§12 material stops). Do not invent one.
3. **Per-commit validation is light (speed first):** the build of `mineworld-cli`, `mineworld validate
   worlds/market-town` when content changed, and the test file the commit writes, compiled (`cargo test -p
   mineworld-cli --test <file> --no-run`) and its short cases run. The 30- and 300-day cases and every mutation run
   **at the milestone**. Record per command: count, wall time, PASS / FAIL / INCONCLUSIVE.
4. Static: `cargo fmt --all --check`; `cargo clippy -p mineworld-cli --all-targets -- -D warnings` when Rust changed;
   `python3 scripts/check_decision_ids.py` when a decision file changed; `python3 scripts/check_doc_headings.py`.
   **R-LOAD-1:** never `cargo test --workspace`, never all of `tools/cli`; any `tools/cli` run goes through
   `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.
5. Review: answer the commit's review questions in one or two sentences each, in §15.
6. `git status --short` and `git diff --stat`: every changed path is in the commit's **Files**; otherwise revert it or
   stop and record why.
7. Update §15 and `handoff-s31f.md` in the same commit.
8. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` ending with the session's attribution
   line. No heredoc writes, no `sed -i`, no `perl -pi`, no `python3 -c`, no `curl`.
9. Stop and report a material stop (§12) instead of improvising.

#### C0 — Base, prerequisites, ledger (Markdown only)

- **Files:** this document (§15), `handoff-s31f.md` (new).
- **Steps:** 1. Record `origin/main`'s commit; the merge commits of S31-a, S31-b, S31-c, S31-d, S31-h (all required:
  else material stop); S29-b, WE-a (#189), S30-c (#280) states. 2. Record the real names of: the reference key
  (`economy.reference`), `shop.keepers`, person `asks`, `price-survey`'s section keys, `configure/notices.yaml`'s keys,
  the `pricing:` block keys and whether S31-d accepted Q-S31f-2's entry lists (else §5.7's fallback applies), the
  `price-reporter` parameter keys, S31-h's option keys, and the `market/mod.rs` mirrors S31-a and S31-h added.
  3. Record Q-S31f-1 … 7 rulings. 4. `mineworld validate worlds/market-town` on the base: PASS.
- **Validation:** none beyond step 4. **Review:** Does any recorded name differ from §5? Each difference is listed
  with the section it changes.
- **Message:** `docs(plan): S31-f — open the ledger at the base`

#### C1 — Check 3 admits S31's packs and the decision list (Q-S31f-1)

- **Files:** `tests/acceptance/tests/ac1_composability.rs`, `docs/decisions/ARC-35.md`, `docs/decisions/INDEX.md`
  (only if `--write-index` changes it).
- **Steps:** §5.8 items 1–4; the `ARC-35` note text of §5.8; `python3 scripts/check_decision_ids.py --write-index`.
- **Validation:** `cargo test -p mineworld-acceptance --test ac1_composability` (all cases; check 3 still passes on
  the base's Market Town, which has none of the new entries); `check_decision_ids.py`.
- **Review:** Is `MARKET_PACKS` unchanged (check 2)? Does every admission name its owner through the build's catalog,
  not a string list of section keys?
- **Message:** `test(acceptance): ARC-35 check 3 admits notices, price-survey and the decision list (S31-f)`

#### C2 — Reference values with sources (step §6.1)

- **Files:** `worlds/market-town/items/{apple, bread, cake, croissant, sandwich, soup, coffee, juice, milk, tea, book,
  candle, flowers, mug, newspaper, notebook, pen, scarf, soap, umbrella}.yaml`.
- **Steps:** append §5.2's block to each, with the source comment per §5.2's citation procedure. Do not touch the
  four loose-object files.
- **Validation:** `mineworld validate worlds/market-town`.
- **Review:** Does every goods comment name a concrete source and date? Is any figure > 25 % off its target (a
  finding)?
- **Message:** `feat(worlds): Market Town's reference values, each with its source (S31-f)`

#### C3 — Keepers, the café's fresh juice, and the goods sellers

- **Files:** `worlds/market-town/organizations/{cafe-company, corner-store}.yaml`, `people/{alice, grace, hana,
  ivan}.yaml`.
- **Steps:** §5.3's three edits; §5.4's holdings and asks; update each file's comment (the café's "sells food and drink
  only" comment is already S31-a's; keep its wording consistent).
- **Validation:** `mineworld validate worlds/market-town`; a 2-day `mineworld run … --headless --seed 7 --days 2`
  prints and exits 0 (smoke).
- **Review:** Is every person at ≤ 6 items? Is every authored ask within §8 AC-f-10's bounds? Do the keepers carry no
  `asks:`?
- **Message:** `feat(worlds): Market Town's shop keepers, the café's fresh juice, and three goods sellers (S31-f)`

#### C4 — Notices and the price book in the composition

- **Files:** `worlds/market-town/world.yaml`, `configure/notices.yaml` (new), `people/carol.yaml`.
- **Steps:** §5.1's `systems` and `configure` lines with comments; §5.6's file; §5.5's section. Update `world.yaml`'s
  header comment: "Market Town is Social Café plus the six market packs, the admitted packs `calendar`, `weather`,
  `notices` and `price-survey`, and configuration (`ARC-35` notes)".
- **Validation:** `mineworld validate worlds/market-town`; `cargo test -p mineworld-acceptance --test
  ac1_composability check_3`.
- **Review:** Are the subscribers exactly Alice, Felix, Bob, Ivan, visitor, wanderer (by the tags in the files)?
- **Message:** `feat(worlds): Market Town posts a price bulletin to subscribers by tag (S31-f)`

#### C5 — The decision list

- **Files:** `worlds/market-town/decisions.yaml` (new), `world.yaml` (the `decisions:` key).
- **Steps:** §5.7's file (or its fallback, per C0). `mineworld decisions worlds/market-town` prints every seat's model:
  Carol `price-reporter`, the rest `rule`.
- **Validation:** `mineworld validate worlds/market-town`; `mineworld decisions worlds/market-town`; check 3 again;
  a 2-day smoke run.
- **Review:** Does every tag in the file exist in some person file (A-3)? Does any entry match a seat it should not?
- **Message:** `feat(worlds): Market Town's decision list — the clerk, pricing and haggling (S31-f)`

#### C6 — Test support: criteria, variants, the generated second world

- **Files:** `tools/cli/tests/market/mod.rs`, `tools/cli/tests/market/variants.rs` (new).
- **Steps:** §5.10 items 1–3; `variants.rs` with `shock(copy, person_file, kind, from, to)`, `relabel(copy)` (V1),
  `harbor_row(dir)` (V2), each asserting its own file diff or its own key set; a `perceived`-stream helper reused from
  S31-c's AC-c-6 test if it is shared, else a local one.
- **Validation:** compile; the short cases: each variant validates (`mineworld validate`), V2's key scan holds no
  Market Town key.
- **Review:** Does `shock` or `harbor_row` contain a Market Town key (I-f-2)?
- **Message:** `test(cli): Market Town criteria read the town's jobs; shock, relabel and Harbor Row writers (S31-f)`

#### C7 — The checkpoint tests

- **Files:** `tools/cli/tests/market_convergence.rs` (new), `tools/cli/tests/market_composition.rs`, and any hosted
  test that F-S31f-7 moves (each listed in §15 with the reason; an assertion is updated only where the decision list
  explains the change).
- **Steps:** AC-f-1 … AC-f-5 and AC-f-7 as `#[test]`s over MT, shock, V1 and V2; `market_composition.rs`: the counts
  (`economy` 34; `holdings` as the files now carry), and the cases for `notices` and `price-survey` removed (AC-f-4
  (ii), (iii)).
- **Validation:** compile; the 2-day smoke of each world variant. The 30-day cases run at the milestone.
- **Review:** Is every threshold in the test the number in §8, written once as a named constant with its AC id?
- **Message:** `test(cli): the 30-day convergence checkpoint on Market Town and Harbor Row (S31-f)`

#### C8 — README

- **Files:** `worlds/market-town/README.md`.
- **Steps:** two sentences: sellers set their own prices (the keepers, Grace, Hana, Ivan; haggling at the shops);
  Carol posts a price bulletin that subscribers receive. Keep the README short (CLAUDE.md §2.1).
- **Message:** `docs(worlds): Market Town's README mentions prices and the bulletin (S31-f)`

#### C9 — Content re-baseline (rebased last; Q-S31-3)

- **Files:** `scripts/baselines.txt`.
- **Steps:** rebase on main; dispatch nightly `parity-long` on the branch; `python3 scripts/ci_parity.py baseline
  write <record> scripts/baselines.txt`; add §5.14's reason line; confirm the other three worlds' keys did not change
  (AC-f-8).
- **Validation:** `git diff scripts/baselines.txt` touches only `[world market-town]`.
- **Message:** `test: re-baseline Market Town for S31-f's trading content (Q-S31-3)`

#### C10 — Close-out

- **Files:** this document (§15), `handoff-s31f.md`.
- **Steps:** I-f-1's and I-f-2's diff scans; push; open the PR; `fast` green on the exact head; report MERGE-READY.

---

## 10. Requirements this PR places on other PRs

- **R-S31d-f-1 (on S31-d).** Each `pricing:` half accepts either one entry or a list of ≤ 8 entries, each with
  `tags:`; a seat uses the first entry whose tags it meets; empty `tags` meets every seat; no match → the half's
  defaults. The list's order is the author's. S31-h's options live inside an entry (Q-S31f-2).
- **R-S31h-f-1 (on S31-h).** `haggle.deadband_permille: 0` is accepted and means "haggle when the ask exceeds the
  basis at all" (§5.12's formula with 0). Proposal facts are mirrored in `market/mod.rs` (S31-h C5).
- **R-S31c-f-1 (on S31-c).** The W5 builder is a shared test helper that V2 extends (or S31-f copies its writer;
  C0 records which).
- **R-S29b-f-1 (on S29-b).** The newsstand seller is not Carol; the newsstand's shop names its seller in `keepers:`;
  `newspaper`'s reference equals its cited price. **R-S29a-f-1 (on S29-a).** If station visits are added to three
  people's routines, include Carol, so the clerk surveys the newsstand.
- **R-S31e-f-1 (on S31-e).** AC-S31-15's "walk to Grace, buy flowers" is reachable in Market Town after S31-f.
- **R-S31g-f-1 (on S31-g).** The visitor seats subscribe by tag, so AC-S31-8's `View.notices` is non-empty in Market
  Town after S31-f.

---

## 11. Bounded decisions (recorded; not operator questions)

| Id | Decision | Why |
| --- | --- | --- |
| SD-S31f-1 | Admit the new packs through check 3's allow-list, not `MARKET_PACKS` | check 2 would otherwise forbid S31-d's crate (A-10) |
| SD-S31f-2 | Food and drink references equal their step-10 shop prices | step §6.1; those prices were sized as US retail |
| SD-S31f-3 | Keepers are the two workers of each shop | they are in the shop during the shift (A-5) |
| SD-S31f-4 | Person asks start at or below their reference | buyers then haggle only at shops, where a keeper answers (S31-h §5.12 basis rule) |
| SD-S31f-5 | Subscriptions by tag only, no person sections | R-S31f-b-1; check 3 needs no `notices` section admission |
| SD-S31f-6 | A keeper without a subscription holds its prices | no suggestion → `toward-suggestion` does nothing, which is correct |
| SD-S31f-7 | `per_day: 6` for four daily slots | Q-S31b-4's rolling window counts a slot exactly 24 h later |
| SD-S31f-8 | `valid_minutes: 720` | a current suggestion exists from the first bulletin onward; supersession keeps only the newest current |
| SD-S31f-9 | No `text` topic in Market Town | no seat would post on it; a topic is added with its publisher's behaviour |
| SD-S31f-10 | Every 30- and 300-day case runs at the milestone | charter (speed first) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S31-f — the Market Town composition and the 30-day convergence checkpoint
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s31f-market-town.md
RELATED / BINDING DOCS:  step-31-wealth-prices.md (§§1, 6.1, 7.4, 8, 9, 10, 13, 14, 15, 16.2, 16.5, 18);
                         pr-s31a-currency-asks.md; pr-s31b-notices.md (§§5.2, 5.4, 10); pr-s31c-price-survey.md
                         (§§5.2, 8, 10); pr-s31d-reporter-policies.md (§§5.2–5.6, 10); pr-s31h-haggling.md (§§5.9,
                         5.12, 10, 14); docs/MODULE_SPEC.md §4.1 (decision list); docs/decisions/ ARC-23, ARC-35,
                         ARC-83, ARC-84, ARC-111 … ARC-118, ARC-153; .structured-coding/standards.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md;
                         /Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s31f, its own, held by one session
BRANCH:                  mvp0/pr-s31f, created from origin/main
START CONDITION:         Milestone F's checklist handed over AND this design frozen AND S31-a, S31-b, S31-c,
                         S31-d, S31-h merged AND the dispatch. NOT met at 2026-10-18
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–6, as §14's rulings shape them                      (operator, pending)
FROZEN INVARIANTS:       step-31 I-S31-1, I-S31-2, I-S31-7, I-S31-9, I-S31-10; §6 I-f-1 … I-f-7; §8 as written
SEQUENCE:                C0 … C10 (§9); C9 rebased last; implementer Haiku-class throughout
ALLOWED COMMANDS:        cargo build / test -p <crate> --test <file> [--no-run] / clippy -p / fmt;
                         target/*/mineworld validate|run|decisions|perceived|replay; mineworld-memory inspect;
                         the cast-slots with-slot.sh wrapper; git; gh (pr create/view/checks, workflow run
                         nightly.yml on the branch for C9 only, run view/download; never merge);
                         python3 scripts/check_doc_headings.py, check_decision_ids.py (--write-index),
                         ci_parity.py baseline write; mkdir -p; cp; cmp; sha256sum/shasum; sed -n; grep
NEVER:                   python3 -c; sed -i; perl -pi; heredoc writes; curl; git add -A; cargo test --workspace
                         or all of tools/cli (R-LOAD-1); reading any key or .env; calling any hosted or local
                         model; editing a path outside I-f-1's allowance; editing a routine, tag, persona or
                         any Social Café field of a person file (check 3); changing a §8 threshold after a run;
                         merging
MATERIAL STOPS:          a prerequisite PR not merged; a name in §5 that does not exist and has no recorded
                         equivalent (C0); a mechanism the content needs that is missing (any change under
                         systems/, cognition/, server/, clients/); Q-S31f-1 not ruled as recommended (check 3
                         would refuse the composition); any §8 criterion failing at the milestone (recorded FAIL
                         with numbers, never tuned); Social Café, Lakeside or bodies-yard baselines moving
PLATFORMS:               Linux, macOS, Windows (CI `fast` per PR; the full matrix at the milestone and on main)
VALIDATION BUDGET:       per PR (speed first): the build, validate, the short cases of the test files each
                         commit writes, CI `fast`, and one nightly `parity-long` dispatch for C9. At the
                         milestone: market_convergence.rs (MT 30 d ×2 with the shock, V1 30 d, V2 30 d ×2,
                         AC-f-4 4 variants × 2 worlds × 30 d, AC-f-7 10 d ×4; ≤ 1 200 s on Linux CI),
                         AC-f-6 300 d ×2 in the nightly, every mutation of §8.1. Gate 1: NOT REQUIRED. Gate 2:
                         NOT REQUIRED. No paid call
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s31f.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                       (source: the operator's freeze of this design — pending)
                         semantic commits: authorized                    (working rules §14)
                         branch push: authorized                         (working rules §21)
                         PR creation / update: authorized                (working rules §21)
                         CI repair to review readiness: authorized       (working rules §21)
                         endpoint: MERGE-READY — pushed, `fast` green on the exact head, reported with
                         the PR number and head SHA
                         merge: by the coordinator only (charter top block, 2026-10-18)
POST-MERGE SYNC OWNER:   lane lead / primary: step-31, overall.md, the progress page; implementation session:
                         this ledger, the merge identity, evidence, deviations
STOP CONDITION:          S31-f MERGE-READY (PR number and head SHA reported) — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S31f-1 | The shock and the control both reach the bread ceiling, so AC-f-1 (d) compares two equal means | the clerk's +10 % applies only below 3 in stock; the keepers move 10 % of the gap per 4 h; a tie is a recorded FAIL and a material stop (I-f-7), not a re-tuned run |
| R-S31f-2 | Fewer than 10 person sales in 30 days (AC-f-1 (e)) | 12 goods units under authored asks at genesis (§5.4), 4 more that Dev and Erin list in their first periods, plus relisting by five sellers; V2 has its own floor; a miss is a recorded FAIL |
| R-S31f-3 | WE-a C5 (#189) lands between and moves Market Town too | WE-a records its own reason line; C9 is rebased last over it |
| R-S31f-4 | A keeper buys goods and, with Q-S31f-2 declined, lists them, then stops pricing the shop (S31-d R-S31d-8) | the entry form keeps keepers out of `list`; the fallback sets `list` empty for everyone |
| R-S31f-5 | Hosted tests over Market Town move because the decision list now applies (F-S31f-7) | C7 lists each, with the decision-list reason; an assertion that encoded "every seat is `rule`" passes `--decisions` of an empty file instead |
| R-S31f-6 | A goods source is hard to find or paywalled | §5.2's procedure: the target stays, marked "no public figure recorded", as a finding |
| R-S31f-7 | Downward drift: accepted haggles pull sale medians under asks, so asks slide toward the floor | floor 500 ‰ × reference; AC-f-1 (c) records every move; only two of eleven buyers haggle |
| R-S31f-8 | S29-b picks Carol | R-S29b-f-1; Q-S31f-4 makes it a ruling |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation | Reason, one line |
| --- | --- | --- | --- |
| **Q-S31f-1 [operator — F-S31f-1, `ARC-35` check 3]** | AC-1 check 3 refuses `notices`, `price-survey`, their config and sections, and `decisions.yaml` in Market Town. Amend check 3 to admit them after the six market packs (a dated `ARC-35` note), keeping check 2's `MARKET_PACKS` unchanged? | **Yes** | Market Town stays "Social Café + installed packs + configuration", and check 2 still lets S31-d's crate depend on the two packs |
| **Q-S31f-2 [operator — F-S31f-2; amends S31-d's Q-S31d-4]** | One `pricing:` block per model cannot give two buyers different haggle offers, or keep keepers out of `list`. Let each half be a list of tag-scoped entries (first match wins), a change to S31-d's parser only? | **Yes, folded into S31-d before its freeze** | Haggling then yields both accepts and declines, and keepers never become sellers, all by data |
| **Q-S31f-3 [operator — F-S31f-3; AC-S31-13 (d)]** | "Bread production halved from day 10" needs new code (no data changes production mid-run). Halve it from genesis instead, and keep measuring days 20–30? | **From genesis** | No new mechanism; by day 20 the clerk's 7-day window sees only shock days either way |
| **Q-S31f-4 [operator — F-S31f-4; Q-S31-4]** | The clerk is Carol (seated, jobless, observant, already walks store → café), and S29-b must not make her the newsstand seller? | **Carol** | No routine edit (check 3 forbids one), and S31-c and S31-d already measured her |
| **Q-S31f-5 [operator — F-S31f-5]** | Make the café a second juice seller: fresh juice at 350, Alice's shift makes 2, the café starts with 6? | **Yes** | Two sellers of one kind give suggestions something to converge on, and production keeps it alive for the whole run |
| **Q-S31f-6 [operator — F-S31f-6; Q-S31-3, Q-S31h-5]** | Q-S31-3 ruled two Market Town re-baselines. S31-h and S31-f2 each move only the journal's genesis declaration. Count only behaviour re-baselines (S31-a, S31-f) as the ruled two, and record S31-h's and S31-f2's as composition-only lines, each measured to move no fact or summary? | **Yes** | It keeps the ruling's intent and lets S31-h and S31-f2 merge independently |
| **Q-S31f-7 [operator — A-17, A-18]** | S30-c is open, and owned attributes need `attributes` enabled and S30-d's sampler. Confirm that the `wealth-band` work is S31-f2, designed after S30-c and S30-d merge, from §5.15's outline? | **Yes** | The step already names S31-f2; designing against an unmerged owner interface would be redone |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | Implementation | Deterministic validation | LLM logic review | SHA |
| --- | --- | --- | --- | --- |
| C0 | [ ] | [ ] | [ ] | |
| C1 | [ ] | [ ] | [ ] | |
| C2 | [ ] | [ ] | [ ] | |
| C3 | [ ] | [ ] | [ ] | |
| C4 | [ ] | [ ] | [ ] | |
| C5 | [ ] | [ ] | [ ] | |
| C6 | [ ] | [ ] | [ ] | |
| C7 | [ ] | [ ] | [ ] | |
| C8 | [ ] | [ ] | [ ] | |
| C9 | [ ] | [ ] | [ ] | |
| C10 | [ ] | [ ] | [ ] | |

### Evidence

(none yet)

### Mutations

(run at the milestone)

### Deviations and findings

(none yet)

---

## 16. Proposed edits elsewhere (made by the PR or owner named)

| Document | Edit | By |
| --- | --- | --- |
| `docs/decisions/ARC-35.md` | the dated note of §5.8 | this PR, C1 (if Q-S31f-1 is ruled yes) |
| `step-31-wealth-prices.md` §15.1 AC-S31-13 (d) | "bread production halved from day 10" → "from genesis", with a dated note citing Q-S31f-3 | lane lead / primary |
| `step-31-wealth-prices.md` §13 "Re-baselines" | a dated note: the two ruled re-baselines are behaviour re-baselines; S31-h and S31-f2 record composition-only lines (Q-S31f-6) | lane lead / primary |
| `step-31-wealth-prices.md` §9.1, §16.2 | the reporter is Carol (Q-S31f-4); §7.4 is S31-f2 (Q-S31f-7) | lane lead / primary |
| `pr-s31d-reporter-policies.md` §5.4 | tag-scoped entry lists in each half (R-S31d-f-1, Q-S31f-2) | S31-d's design session |
| `pr-s31h-haggling.md` §5.12 | `deadband_permille: 0` stated as admitted (R-S31h-f-1) | S31-h's design session |
| S29-a / S29-b designs | R-S29a-f-1, R-S29b-f-1 | their design sessions |
| `overall.md` | S31-f2 added under S31 with its dependency on S30-c and S30-d | primary |
