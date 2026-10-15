# Step 31 — S31: wealth and seller-set prices, a public notice board, and the market price reporter

**Role:** step document for **S31**, a new step that the primary session adds to [`overall.md`](overall.md).
It records two operator requirements of 2026-10-15. It audits what the market packs do today. It then designs:

- the currency as World Pack data;
- seller-set asking prices, for Persons and for Organizations, with one atomic settlement;
- an optional reference value per item kind, kept apart from the asking price;
- wealth as a derived, holder-only view;
- a general publish/subscribe System Pack for time-limited public notices;
- a price survey book and a pluggable market price reporter that publishes suggested prices;
- pluggable pricing policies through which prices converge;
- what agents (MCP), the 2D client and the 3D client see and send.

It ends with the invariants, the acceptance and adversarial criteria, a PR split, the risks and the questions
only the operator can answer.

**Effort:** `mvp0` (placement argued in §16.3; Q-S31-1 asks the operator to confirm it).
**Parent:** [`overall.md`](overall.md).

**Lifecycle:** `STEP DESIGN (DRAFT)`, revision 1, awaiting operator review.

- Approval would fix this step's scope, invariants (§14) and step acceptance (§15.1). Progress, evidence,
  findings and bounded corrections stay writable.
- Each PR is detailed to the commit in its own PR design and frozen in turn (`CLAUDE.md` §3.1). Only a PR
  design carries `DESIGN FROZEN`.
- Nothing in this file authorizes implementation, a paid model call or a merge.

**Author:** S31 design session (Opus), 2026-10-15.

- Worktree: `/Users/yuema137/mineworld-worktrees/design-s31`.
- Branch: `docs/s31-wealth-prices`.
- Audit base: `origin/main @ 58a6f55f` (#204, S27-c merged).

**Ids.** Placeholders only, until the step is approved: `ARC-S31-a` … `ARC-S31-h` and `DEP-S31-a`. On
2026-10-15 the highest ids on `main` are `ARC-108` and `DEP-60`. `ARC-109` and `DEP-61` are already claimed
on the pushed branch `docs/s28a-s27d-designs` (`pr-s28a-memory-home.md`). The first free ids are therefore
`ARC-110` and `DEP-62`, to be re-checked when the step is approved (§19.5).

Local ids in this file:

| Prefix | Meaning |
| --- | --- |
| `SD-S31-n` | step decisions |
| `A-n` | audit anchors |
| `F-S31-n` | findings |
| `AC-S31-n` | step acceptance |
| `X-n` | planted mutations |
| `I-S31-n` | invariants |
| `R-S31-n` | risks |
| `Q-S31-n` | operator questions |

**Parallel-safety.** This file is the only file this session writes. The edits it needs elsewhere are
**proposed** in §19 and made by the PRs that own those files:

- `overall.md` (the primary session);
- `docs/CORE_CONCEPTS.md`, `docs/MODULE_SPEC.md`, `docs/ARCHITECTURE.md`, `docs/DECISIONS.md`;
- the S29, S30 and S27-d designs (each by its own owner; §16.4 lists the interface points).

---

## 0. Binding inputs

### 0.1 Operator requirements, 2026-10-15 (verbatim)

> "我觉得财富也很重要，我们需要有一般等价物（钱），然后每个物品都有标价（不一定是全局统一的，可以是出售的人物决定）"

> "必需的消耗品 食材等等，可以有市场建议价格（一个专门的npc负责统计，然后给出建议，这个npc后端可以是llm agent，也可以是确定性的算法什么的，这个也是模块化的，然后这种有时效性的公共信息可以发送到别的npc的inbox里，每个npc可以有订阅公共信息的属性）。请你把这些也都加入到我们的计划里面。有了trading机制我们这个世界才会更加真实"

In English, for the specification documents (`CLAUDE.md` §2.1 rule 5: English is authoritative, and the
Chinese is quoted as the source):

> *"I think wealth matters too. We need a general equivalent (money), and every item has a price tag (not
> necessarily globally uniform; it can be decided by the person selling it)."*
>
> *"Essential consumables, food ingredients and so on, can have a market suggested price. A dedicated NPC is
> responsible for compiling statistics and giving the suggestion. This NPC's backend can be an LLM agent or a
> deterministic algorithm, and that is modular too. This kind of time-sensitive public information can be
> sent to other NPCs' inboxes, and each NPC can have an attribute for subscribing to public information.
> Please add all of this to our plan. Only with a trading mechanism will our world feel more real."*

The coordinator's restatement of the second requirement, relayed with it, binds the following:

1. A suggested price is advisory. The seller's asking price is still the only price a buyer pays.
2. Suggestions are computed from observed trades, using the sale facts that S31 itself emits.
3. The reporter is a real Person with a role. Its backend is pluggable through the existing decision-model
   registry (`models!`, `external "lm"`), with no new plugin mechanism. It works only from what it can
   legitimately perceive.
4. Public notices are a general publish/subscribe mechanism. A notice has a topic, a publisher, a body, an
   issue time and an expiry. It is owned by one System Pack. Agents over MCP receive notices through S27-d's
   inbox. Received notices become S28 memories. An expired notice stops being current but is still remembered.
5. Subscriptions are a person attribute that S30 owns. S31 specifies the semantics and the shape.
6. Pricing models may read suggestions, so that prices converge.

### 0.2 Standing rules that bind this step

| Rule | Consequence here |
| --- | --- |
| Framework, not game | Currency, prices, reference values, topics, subscriptions and the reporter are World Pack content and configuration. Market Town is one composition (§16.2 S31-f), never a hard-coded scene |
| Modular, not hard-coded | Three independent System Packs (`economy` extended, `notices` new, `price-survey` new) and Controller Packs in the existing registry. Each is removable, and removing one leaves a valid world (AC-S31-9, AC-S31-12) |
| Single ownership (`CLAUDE.md` §4 rule 1) | Only `economy` moves money. Only `inventory` writes holdings. Only `notices` writes notices and subscriptions. Only `price-survey` writes price books |
| Kernel ignorance (§4 rule 2) | No kernel or contract change. The kernel never learns what money, a price, a currency or a notice is (I-S31-1) |
| Spatial requirements belong to System contracts (§4 rule 15) | Each action states its `SpatialRequirement` as data (§5.4). Clients never evaluate one |
| Agents are first-class players | An MCP agent sees the same prices, notices and refusals as anyone in its seat, through the same Observation (§11) |
| Information-rich observations (S27-c R-S27c-INFO) | Prices carry the currency's display data. Refusals carry their named code |
| Realistic defaults (cite the reference) | Currency data follows ISO 4217. Market Town's prices stay those of step-10, which were already sized. The reporter role follows real market-news practice (§9.1) |
| All platforms | Every scenario runs on the Linux, macOS and Windows runners (§15) |
| Local load: CI is the gate (R-LOAD-1) | Long runs are measured in CI. Local runs are INCONCLUSIVE for timing |
| Design on Opus, implement on Haiku | Each PR names its implementer (§16.2) |
| No early compatibility debt (§4 rule 12) | `buy`'s payload and `Shop`'s shape change cleanly; there is no adapter for the old ones (§13) |

---

## 1. The requirement, restated as checkable outcomes

1. **Money is the general equivalent, and the world names it.** Each world with `economy` declares one
   currency: its name, symbol, minor-unit exponent and display placement. No currency name is in code.
2. **Every item for sale has a price, and its seller sets it.** Any Person may list an item it holds at a
   price it chooses. An Organization lists through its shop. Two sellers may ask different prices for one
   kind, and a seller may change a price at any time.
3. **A buyer pays exactly the asking price it saw, or nothing.** A buy at a stale price is refused by name.
   Money and item move together in one revision, or neither moves.
4. **A reference value is optional and is never paid.** It serves wealth estimates and pricing defaults.
5. **Wealth is a view, not a stored value.** It is wallet plus the reference value of holdings, shown to the
   holder only.
6. **Public notices exist and are subscribed to.** A notice has a topic, publisher, body, issue time and
   expiry. It reaches subscribers' perceived streams, which are their inboxes, and their memories. A
   non-subscriber never receives it.
7. **A dedicated reporter publishes suggested prices for essential consumables.** It works from prices and
   sales it perceived. Its backend can be deterministic or an LM, with no contract difference.
8. **Prices converge.** Sellers' pricing policies may read the suggestions they subscribe to. In a seeded
   multi-day Market Town run, suggestions move with supply and at least one seller moves toward them.
9. **Everything is optional.** A world without `economy` loads unchanged. A world without a reporter still
   trades. A world without `notices` still trades and still has prices.
10. **Both clients and the agent view show it.** They show price tags, a sale panel and a wallet. They send
    intents and evaluate no rule.

---

## 2. Audit (`origin/main @ 58a6f55f`, 2026-10-15)

### 2.1 Anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | `Wallet { balance: u64 }` on a Person or Organization, in integer minor units; no negative balance can be represented | `systems/economy/src/component.rs:14-35` | Money already exists. S31 adds the currency's identity and changes nothing about amounts |
| A-2 | No currency name, symbol, code or exponent exists anywhere in Rust or YAML | grep of `systems/`, `worlds/`, `docs/` | The currency is a new datum (§4) |
| A-3 | The 2D client formats money as `units / 100` with two decimals, then applies `format.money` = `"{amount}"` (no symbol) | `clients/2d/scripts/hud/words.gd:68-73`; `presentation/mineworld-default/2D/i18n/en.po:158`; `clients/2d/PRESENTATION.md:194` | The exponent 2 is hard-coded in a client. That is a client knowing a world rule (F-S31-3) |
| A-4 | `Shop { operator: OrganizationId, prices: Vec<Price> }` lives on the **Place**. Prices are fixed at genesis from the operator's `economy.shop` section. There is no runtime price change | `component.rs:66-101`; `section.rs` | Only Organizations sell, and their prices never change (F-S31-1) |
| A-5 | A shop on a Person file is refused at seed (`OrganizationId::new` fails) | `section.rs:107-112` | Persons cannot sell |
| A-6 | `buy { item }`: no target, no count (always 1), no price. Requirement `at_place(shop).requiring_target_available()` | `action.rs:18-47` | The payload cannot detect a stale price (F-S31-2) |
| A-7 | `buy` refusals: out of stock, cannot pay and cannot carry all collapse to `TargetUnavailable` (step-10 QS-44); not in a shop or not priced → `NoSupportedInteraction` | `system.rs:88-109`; `tests/buy.rs:142-182` | The required names (`InsufficientFunds`, …) do not exist. `Rejection::System { code, detail }` and `Offer::refused(Rejection)` already exist to carry them (A-15) |
| A-8 | **The atomic exchange already exists for shops.** `resolve` returns `[pay(buyer → operator), inventory::transfer(operator → buyer)]`. Both facts are recorded in one generation of one dispatch. Each owner re-checks its rule at reduction (`ARC-26`) | `system.rs:113-137` | S31 keeps this one-settling-owner pattern and generalizes it (SD-S31-6, §5.7) |
| A-9 | Reduction is not transactional. An error out of `resolve` or `react` after writes is a system bug, not a refused request; nothing rolls back | `kernel/src/dispatch.rs:55-66` | Atomicity rests on validating both rules before resolving, and on the checked constructors (§5.7) |
| A-10 | Persistence commits one revision per transaction: the journal entry and its facts together | `persistence/src/sqlite.rs:1-21`; `persistence/src/lib.rs` (`ARC-25`) | A process killed mid-trade leaves either the whole revision or none of it (§5.7, AC-S31-4) |
| A-11 | `give { item, count }` is Person → Person, same place, within `GIVE_RANGE = 3 000 mm`, target available. Organizations cannot give | `systems/item-transfer/src/action.rs:18-53` | The Person-to-Person sale reuses the same spatial shape (§5.4) |
| A-12 | `inventory` owns `Holdings`. `PERSON_CAPACITY = 6` items of every kind together; Organizations are unbounded. `admit_transfer` and `transfer` are the checked API | `systems/inventory/src/admit.rs:18-112` | A buyer who cannot carry is refused (`cannot-carry`) |
| A-13 | Item kinds carry only `item: { category }`. There is no value or price field. "Items are never perceived": a kind's name is not disclosed | `systems/item/`; `systems/item/src/lib.rs`; F-S27c-9 | A reference value is new (§6). Item names stay pending 12d's `modern-goods` 0.2.0 |
| A-14 | Wages: employment states `wage-due`; economy answers with `money-transferred` or `wage-unpaid` in the next generation (`ARC-28`) | `systems/employment/src/system.rs:216-252`; `economy/src/system.rs:201-226` | Unchanged by S31 |
| A-15 | `enum Rejection { Busy, TooFarAway, PermissionDenied, NoSupportedInteraction, TargetUnavailable, Unavailable, PreconditionFailed, System { code, detail } }` | `contracts/src/action.rs:215-243` | S31's refusals are economy-owned `System` codes plus kernel reasons. No contract change |
| A-16 | `Visibility::{ Public, Place, Participants, Entities(BTreeSet), … }` | `contracts/src/event.rs:296-308` | A notice's audience is `Entities(subscribers ∪ {publisher})`. The kernel already delivers it, with no change (§8.4) |
| A-17 | The S27 agent inbox holds the facts the seat perceived (`perceived` stream, bound 512). S27-d adds `Player.inbox` → `InboxStatus` and the `subscriptions/listen` resource `mineworld://seat/{handle}/inbox` | `clients/agent/src/mineworld_agent/inbox.py:1-12`; `pr-s27d-mcp-stdio.md` §5.7 (frozen on `docs/s28a-s27d-designs`) | A notice delivered as a perceived fact *is* an inbox entry. S31 adds no inbox mechanism (§8.6) |
| A-18 | S28: "perception is the only access right" (I-S28-1). A shared fact record is held per person by receipt; the `item` and `place` scopes are fed from perception | `step-28-shared-memory.md` §§3, 5.2.3 | Notices and sales become memories by being perceived (§8.7, §7.3) |
| A-19 | `DecisionModel` is a pure, stateless function of `(parameters, seed, pace, observation)`. Interior mutability is forbidden. Models register in `models!` (`Rule`, `Motivated` today; `external "lm"` is added by step-17 P6) | `docs/MODULE_SPEC.md` §5.1; `cognition/installed/src/lib.rs:20-22` | A deterministic reporter cannot remember trades between consults. Its observations must be world state disclosed to it (§9.3, SD-S31-11) |
| A-20 | A model may attempt an action whose payload is free-form "by name" (`ARC-34`). `rule` Cargo-depends on System Packs' published types to build such requests | `docs/CORE_CONCEPTS.md` §15.2; `cognition/rule-controller/Cargo.toml` (conversation, movement) | `set-price` and `publish-notice` are attempted by name, with the payload built through the owning pack's type (§10.2) |
| A-21 | WE-a (frozen 2026-10-12, not merged) gives economy its first `configure/economy.yaml` (`price_modifiers`), `Shop.adjustments`, `Shop::effective_price`, facts `prices-adjusted` / `prices-restored`, and `SystemVersion` 2 | `pr-we-a-happenings.md` §6.2 | S31's currency goes into the same file. A modifier applies to an Organization's asking price (§5.8). S31-a rebases over WE-a C4 |
| A-22 | S29-b (approved, not started) adds `station-news`, a shop at `station` selling `newspaper` | `step-29-station-landmark.md` §7.2 | `newspaper` is category `goods`. A shop selling goods contradicts `ARC-38` item 5 (F-S31-6) |
| A-23 | Market Town: 12 people (11 seats; Otto is unseated), two organizations, 20 kinds (6 food, 4 drink, 10 goods). Wallets are 20 000 (Alice, Felix) and 200 000 (others). Shops: café {coffee 300, tea 250, croissant 250, cake 400, sandwich 600, soup 500}; store {apple 100, bread 300, milk 200, juice 250} | `worlds/market-town/**` | The suggested-price topic covers the 10 food and drink kinds (§9.2) |
| A-24 | The market criteria test hard-codes "exactly two job holders (alice and felix)" and `PERSON_CAPACITY = 6` | `tools/cli/tests/market/mod.rs:34, 546` | S29-b and S31-f must update it in the same commit as the content |
| A-25 | Long-run digests live in `scripts/baselines.txt`. A re-baseline adds a `reason <PR id>: …` line in the same PR | `scripts/baselines.txt`; `ARC-83` item 5 | §13 plans the moves |
| A-26 | Social Café, Lakeside and bodies-yard have no `economy` | `worlds/*/world.yaml` | AC-S31-9 checks that they load unchanged |
| A-27 | `happenings` announcements are `Public` facts; WE-b's director needs an Organization seat (QS24-2, MVP-1) | `ARC-88` | Organization seats are not available to S31 in MVP-0 (SD-S31-5) |

### 2.2 Findings

- **F-S31-1 — Only Organizations sell, at fixed prices.** Persons can only give. No runtime action opens a
  listing or changes a price.
- **F-S31-2 — A buy cannot be stale-checked.** The payload names only the kind. A price change between
  observation and dispatch would be charged silently.
- **F-S31-3 — The money's exponent is a client rule.** The 2D client divides by 100. A world whose coin has
  no minor unit would display wrongly in 2D, and an agent cannot know the scale at all.
- **F-S31-4 — The atomic exchange is already right.** One dispatch, one settling owner, two checked
  constructors, one revision. S31 needs no escrow and no new kernel feature, only the generalization of that
  pattern (§5.7).
- **F-S31-5 — Stateless models cannot accumulate statistics.** A deterministic reporter needs its notes as
  world state disclosed to it. An LM reporter could use S28 memory, but then the two backends would see
  different inputs. That would break "swapping the backend changes no contract".
- **F-S31-6 — S29-b's newsstand sells goods.** `ARC-38` item 5 says shops sell consumables only. S31 lifts
  that rule (SD-S31-4), which resolves S29-b's conflict. If S29-b lands first, it needs a ruling of its own,
  or a newspaper kind of a consumable category (R-S31-6).
- **F-S31-8 — A kernel fault is journaled and the world goes on.** `PersistedWorld::dispatch` commits the
  request with the fault as its outcome and no facts (`persistence/src/world.rs:259-281`). Reduction has no
  undo log (A-9). A multi-fact resolve is therefore atomic in the log, and across a crash, unconditionally.
  In memory it is atomic only as long as the owners' checks agree. That holds for today's `buy` as much as
  for S31's sales. §5.7 states how S31 proves the agreement rather than assuming it.
- **F-S31-7 — No public-information channel exists.** Happenings announce publicly. Nothing addresses a
  topic to subscribers, and nothing expires.

---

## 3. Terms

These are economy, notices and price-survey vocabulary, not new ontology terms (`CLAUDE.md` §2.1 rule 3).
Each is a Component, Event or action of a named System Pack.

| Term | Meaning |
| --- | --- |
| **currency** | The world's one general equivalent: `name`, `symbol`, `exponent`, `symbol_at`. Configuration of `economy` |
| **amount** | An integer count of the currency's minor units (`u64`), as today |
| **ask** | One seller's standing offer to sell one kind at one unit price: `{ item, price }`. Economy state |
| **asks** | A seller's set of asks: component `asks` on the seller (a Person or an Organization) |
| **shop** | A Place where an Organization's asks are on display and can be bought, without the Organization being present. It replaces today's `Shop` component's price list |
| **keeper** | A Person an Organization's shop names as allowed to change its asks while standing in the shop (SD-S31-5) |
| **asking price** | The price of an ask, after any WE-a modifier. It is the only price a buyer pays |
| **reference value** | An optional authored amount on an item kind: a typical worth. It is never charged (§6) |
| **sale** | One settled exchange: fact `sold`, together with `money-transferred` and inventory's `items-transferred` |
| **wealth** | A derived view: wallet plus the sum, over held kinds with a reference value, of count × reference value (§7) |
| **topic** | A named channel of public notices, declared in `notices` configuration, with its publishers, body kind and validity bounds |
| **notice** | One publication on a topic: `{ id, topic, publisher, body, issued_at, valid_until }` |
| **current** | A notice with `issued_at ≤ now < valid_until` that no later notice from the same publisher on the same topic supersedes |
| **subscription** | A Person's standing choice to receive a topic's notices. Component `subscriptions` |
| **price book** | A surveyor's bounded record of prices it perceived (asks and witnessed sales). Component of `price-survey` |
| **suggested price** | A line of a `market-prices` notice: advisory, published by a reporter |
| **reporter** | A Person named as a publisher of a prices topic, who keeps a price book. Its decision model is its backend |
| **pricing policy** | The part of a seller's decision model that chooses and adjusts its asks (§10) |

---

## 4. The currency as data (`ARC-S31-a`)

### 4.1 Where it is declared (SD-S31-1)

The currency is `economy`'s world configuration (`ARC-61`). It goes in the same file WE-a introduces:

```yaml
# configure/economy.yaml
currency:
  name: US dollar          # 1–64 bytes, display only
  symbol: "$"              # 1–8 bytes, display only
  exponent: 2              # minor units per major unit = 10^exponent; 0 … 4 (ISO 4217 uses 0, 2, 3, 4)
  symbol_at: before        # before | after
price_modifiers: []        # WE-a, unchanged
```

A fantasy world declares, for example, `{ name: crown, symbol: "cr", exponent: 0, symbol_at: after }`. No
currency name, symbol or exponent appears in code. The ISO 4217 minor-unit column is the reference for
`exponent`: USD and EUR have 2, JPY 0, KWD 3 and CLF 4.

- **Required when `economy` is enabled.** A world that enables `economy` without `currency:` is refused,
  by name, with `economy-currency-undeclared`. That makes the scale explicit. There is no hidden default to
  drift from (`CLAUDE.md` §4 rule 12). Only Market Town enables economy today, and S29-b's newsstand lives in
  the same world.
- **The genesis fact** `currency-declared { name, symbol, exponent, symbol_at }` is `Public`. Economy
  discloses a declared, never-carried record `currency` to every observer, following WE-a's
  `happenings-here` pattern (`pr-we-a-happenings.md` §5.7). Every client and agent therefore formats from the
  world, not from a constant.
- **Display text belongs to presentation.** A Presentation Pack may still translate `name` ("美元"). It may
  never change `exponent` or the amount. `format.money` gains `{symbol}` and `{amount}` placeholders, and
  `{amount}` is formatted from the disclosed `exponent`.

### 4.2 One currency, no barter in S31 (SD-S31-2; Q-S31-2)

| Option | Verdict |
| --- | --- |
| (a) **Exactly one currency per world** | **Recommended.** "A general equivalent" is one thing by definition. Amounts stay bare `u64`, and every price compares with every other |
| (b) Several currencies, with a currency id in every amount | Declined for now. It puts an id in every `Price`, `Wallet` and fact for a need nobody has stated, and exchange rates would be a second economy (§4 rule 11). If needed later, it is a clean contract change before stabilization (§4 rule 12) |
| (c) Barter only | Not a currency. A world without `economy` already lets people `give`. An item-for-item `swap` is a separate System Pack later, built on the same settlement pattern (§5.7), and needs nothing from S31 |

### 4.3 What does not change

Wallets, amounts, `money-transferred`, wages and money conservation are unchanged. Renaming the currency
changes only `currency-declared` and the display: no amount, no code (AC-S31-6).

---

## 5. Every item has a price, set by its seller (`ARC-S31-b`)

### 5.1 The model (SD-S31-3)

- **`asks`** is a new economy component on the **seller**, a Person or an Organization:
  `Asks { asks: Vec<Ask { item: ItemId, price: NonZeroU64 }> }`, sorted by item, with at most one ask per kind.
  - An ask covers every unit of that kind the seller holds. Holdings remain inventory's: "in stock" is read
    from `Holdings` when it is shown, exactly as today's listing does (step-10 QS-45). There is no second
    stock count.
  - A seller holding none of a listed kind keeps the ask, shown as out of stock (`in_stock: 0`). It is not
    withdrawn silently.
  - Price bounds: `1 ≤ price ≤ PRICE_MAX = 10^12` minor units, a published constant against overflow in
    wealth sums.
- **`shop`** on a Place becomes `Shop { operator: OrganizationId, keepers: Vec<PersonId> }`. The shop's
  prices are the operator's `Asks`. One truth: the price a person sees at the counter and the price a keeper
  sets are the same record.
- **Genesis.** The `economy:` section keeps its shape and gains two optional keys:

  ```yaml
  # organizations/corner-store.yaml
  economy:
    wallet: 300000
    shop: { at: store, prices: { apple: 100, bread: 300, milk: 200, juice: 250 }, keepers: [felix] }
  # people/grace.yaml
  economy: { wallet: 200000, asks: { flowers: 450 } }
  ```

  `shop.prices` seeds the Organization's asks. `asks:` on a Person seeds that Person's. The genesis facts are
  `shop-opened { place, operator, keepers }` and `listed { seller, item, price }` per ask.
- **Any kind may be sold (SD-S31-4).** `ARC-38` item 5 ("shops sell consumables only") is lifted. What a
  seller lists is content and the seller's choice. The capacity of six still bounds what a person can buy
  (`cannot-carry`), so the reason behind item 5 is kept by inventory's rule rather than by a ban.
- **Who changes an Organization's prices (SD-S31-5).** An Organization holds no seat in MVP-0 (QS24-2 admits
  only WE-b's director, in MVP-1). Its shop therefore names `keepers`: Persons who may `set-price`,
  `list-for-sale` and `withdraw` for the shop while standing in it. The authority is economy's own data, so
  economy gains no dependency on `employment`. A shop with no keepers keeps its genesis prices, apart from
  WE-a modifiers (Q-S31-8).
- **Different sellers, different prices.** The café and a person may both ask for `coffee`, at 300 and 260.
  Nothing relates them. Each buyer sees only the asks of the sellers present to it (§5.4).

### 5.2 Actions (all owned by `economy`)

| Action | Payload | Actor | Effect |
| --- | --- | --- | --- |
| `list-for-sale` | `{ item, price }` | a living Person, for itself; or a keeper standing in its shop, `{ item, price, for: shop }` | adds an ask. Refused if the kind already has one (`already-listed`): use `set-price` |
| `set-price` | `{ item, price }` (+ `for: shop`) | the same | replaces an existing ask's price |
| `withdraw` | `{ item }` (+ `for: shop`) | the same | removes an ask |
| `buy` | `{ item, price }`, target: the seller Person, or none in a shop | a living Person | one unit at exactly `price` (§5.7) |

`price` in `buy` is the asking price the buyer observed. A mismatch is `price-changed`, and nothing moves.
`for: shop` means "the shop of the place I stand in"; economy resolves it, and the payload never names an
Organization.

Haggling is designed in §5.9 and deferred to MVP-1 (Q-S31-6). A buyer's price question needs no action:
asks are perceivable (§5.6).

### 5.3 Events (all owned by `economy`)

| Event | Payload | Audience | Notes |
| --- | --- | --- | --- |
| `currency-declared` | `{ name, symbol, exponent, symbol_at }` | `Public` | genesis |
| `shop-opened` | `{ place, operator, keepers }` | `Public` | genesis; prices now come as `listed` |
| `listed` | `{ seller, item, price }` | `Place(seller's place)`; for a shop, `Place(shop)` | genesis or `list-for-sale` |
| `price-set` | `{ seller, item, from, to }` | the same | `set-price`. It is what a person "sees change" and remembers (§7.3) |
| `withdrawn` | `{ seller, item }` | the same | |
| `sold` | `{ seller, buyer, item, count, price, place }` | `Place(place)` | one per sale. It is the trade fact that a witness, a surveyor and memory perceive |
| `money-transferred` | unchanged | unchanged (`Place` for a sale) | |
| `items-transferred` | inventory's, unchanged | `Participants` | stated by economy through `inventory::transfer` (`ARC-26`) |
| `wage-unpaid`, `funded` | unchanged | unchanged | |

None of these facts is biographical (step-10 QS-49), except that `sold` may be configured biographical later.

### 5.4 Spatial requirements (`CLAUDE.md` §4 rule 15; SD-S31-7)

| Action | Requirement | Why |
| --- | --- | --- |
| `buy` from a shop | `at_place(shop).requiring_target_available()` (today's) | the shop is the place; the operator need not be present |
| `buy` from a Person | `same_place().within(TRADE_RANGE).requiring_target_available()`, `TRADE_RANGE = 3 000 mm` | a hand-to-hand sale, the same reach as `give` (A-11) and conversation |
| `list-for-sale`, `set-price`, `withdraw` for oneself | `NONE` | a person decides its own prices wherever it is, like `send message` |
| the same `for: shop` | `SamePlaceAsActor` at the shop's place | a keeper changes the shelf tags in the shop |

**Remote listings are out of scope.** Buying from someone not present needs delivery: items would have to
travel. Delivery is a transport Process (S21, MVP-1). A future `mail-order` pack would declare `NONE` and a
delivery Process, on the same settlement (§5.7). S31 states the non-goal so that no client invents one.

### 5.5 Refusals, by name

Economy-owned codes are `Rejection::System { code }` (A-15). Kernel reasons are used where they fit.

| Name | Code / reason | When | Checked by |
| --- | --- | --- | --- |
| NotForSale | `not-for-sale` | the seller has no ask for the kind, or the shop does not list it | validate |
| PriceChanged | `price-changed` | `buy.price` ≠ the current asking price | validate |
| InsufficientFunds | `insufficient-funds` | buyer's balance < price (`admit_payment`) | validate; offer |
| OutOfStock | `out-of-stock` | the seller holds none (`admit_transfer` from-side) | validate; offer |
| CannotCarry | `cannot-carry` | the buyer is at capacity (`can_take`) | validate; offer |
| TooFarAway | kernel `TooFarAway` | not in the shop's place, or beyond `TRADE_RANGE` | `SpatialRequirement::evaluate` |
| SelfTrade | kernel `PreconditionFailed` | buyer = seller, or a keeper buying for the shop it keeps | validate |
| NotAKeeper | kernel `PermissionDenied` | `for: shop` by a person the shop does not name | validate |
| AlreadyListed | `already-listed` | `list-for-sale` of a kind already asked | validate |
| PriceOutOfRange | `price-out-of-range` | `price` = 0 or > `PRICE_MAX` | validate |
| NoSuchSeller | kernel `NoSupportedInteraction` | the target is not a living Person, or no shop is here and no target was given | validate |
| NotHeld | `not-held` | `list-for-sale` of a kind the person holds none of | validate |

This amends step-10 QS-44, which collapsed three reasons into `TargetUnavailable` because an offer could then
carry only one. `Offer::refused(Rejection)` now carries the named code, and clients map codes to words
(`CORE_CONCEPTS.md` §12.1).

### 5.6 Offers and disclosure

- **Buy offers are complete affordances** (`ARC-34`), with the price in the payload. A person standing in a
  shop is offered one `buy { item, price }` per listed kind. A person within `TRADE_RANGE` of a seller is
  offered one per ask of that seller, targeted at the seller. Each is available or refused with its named
  code. A controller that submits an offer it observed is refused `price-changed` if the price moved since.
  That is the honest outcome, and it is the test of F-S31-2.
- **Seller offers.** A person holding a kind it has not listed is offered `list-for-sale` (incomplete: the
  price is the seller's choice, attempted by name, `ARC-34`). A seller with asks is offered `set-price`
  (incomplete) and `withdraw` (complete). A keeper standing in its shop is offered the same `for: shop`.
- **Disclosure.**
  - A **shop's listing** (operator; per kind the asking price and how many in stock) goes to whoever
    perceives the place, as today.
  - A **Person's asks** (kind, price, in stock 0/≥1, never the exact count) go to whoever perceives the
    seller. Their holdings stay private (`ARC-37`): a buyer learns only whether one is available.
  - **Wallets** go to the holder only (unchanged, `INV-13`).
  - **Reference values** are public (§6.1).

### 5.7 Atomic settlement: one settling owner (SD-S31-6)

**Choice: economy is the one settling system for a sale. It decides the whole exchange in one dispatch and
states both facts. Each owner still reduces and re-checks its own** (`ARC-26`, `ARC-38` item 3). This is
today's `buy` (A-8), generalized.

```text
buy { item, price } ── validate (read-only) ──────────────────────────────────────────────────────
   1 seller resolved (target Person, or this place's shop → operator)       NoSuchSeller
   2 ask exists; price equals the asking price (after modifiers)             NotForSale, PriceChanged
   3 SpatialRequirement::evaluate                                            TooFarAway
   4 admit_payment(buyer → seller, price)                                    InsufficientFunds, SelfTrade
   5 inventory::admit_transfer(seller → buyer, item, 1)                      OutOfStock, CannotCarry
resolve ──────────────────────────────────────────────────────────────────────────────────────────
   [ sold{…}, money-transferred{buyer→seller, price}, items-transferred{seller→buyer, item, 1} ]
   built through pay() and inventory::transfer(), which ask steps 4 and 5 again
reduce (same generation, same instant) ─────────────────────────────────────────────────────────
   economy writes both Wallets          inventory writes both Holdings      (each re-checks)
persist ─────────────────────────────────────────────────────────────────────────────────────────
   one revision = the journal entry + all three facts, in one SQLite transaction (A-10)
```

Why this is atomic:

1. **Before anything is written,** validate asks both owners' rules through their own functions
   (`admit_payment`, `admit_transfer`). Validate cannot write (`BD-6`).
2. **Within the dispatch,** the checked constructors ask the same functions again. Each owner's reduction
   asks them a third time. All three read the same state at the same instant, and no other dispatch
   interleaves (dispatch is sequential). Reducing `money-transferred` writes only wallets, and reducing
   `items-transferred` writes only holdings, so neither changes what the other's check reads. A
   reduction-time refusal is therefore unreachable except through a bug in one of the two `admit_*`
   functions. **What such a bug would do is stated, not hidden** (F-S31-8): the kernel has no undo log
   (A-9), and the persisted world journals a kernel fault as the request's outcome, with no facts, and goes
   on (`persistence/src/world.rs:259-281`). The wallets written before the refusal would stay written in
   memory. S31 does not change the kernel (I-S31-1). It closes the gap by evidence instead:
   - a property test over seeded random states asserts that `validate` admitting a sale implies both
     reductions accepting it;
   - the I-S31-3 scan runs over every scenario save;
   - a planted reduction refusal must be reported as `FactRefusedByOwner` naming `inventory`, and must be
     flagged by the scan (X-11).
3. **Across a crash,** the revision is one transaction. A process killed at any wall instant leaves the save
   at the last committed revision. Replay re-executes the journal (`ARC-25`). A sale is either entirely in the
   log or entirely absent.

**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) **One settling owner (economy), both facts in one dispatch** | **Chosen.** Already built and tested for shops. Single ownership holds: economy writes wallets, inventory writes holdings. No new state, no new kernel feature |
| (b) Two-phase escrow: economy reserves money, inventory reserves the item, then both commit | Declined. It adds reserved balances and reserved stock (two new kinds of state, each with an expiry), a commit and abort protocol, and recovery for an orphaned reservation. It solves cross-time trades (haggling, delivery), which S31 resolves at a single instant instead (§5.9) |
| (c) Event choreography: economy pays, inventory reacts by moving the item | Declined. The item move would happen in a later generation, and could be refused after the money had moved. That is exactly the half-trade the requirement forbids, and inventory would be reacting to economy's vocabulary, against `ARC-26` item 1 |

`item-transfer`'s `give` is unchanged and still owns nothing. Economy keeps its system dependencies on
`inventory` and `presence` only.

### 5.8 WE-a's price modifiers

A modifier (`prices-adjusted`) is the world author's configured policy for an Organization's shop. The
asking price of an Organization's ask is `effective_price(base ask, active adjustments)`, through WE-a's one
function. A keeper's `set-price` changes the base, and modifiers still apply on top. `buy.price` is compared
with the effective price, so a modifier starting between observation and dispatch is `price-changed`. That is
honest. Person asks have no modifiers. If S31-a lands before WE-a C4, WE-a C4 rebases on `Asks`; the formula
is unchanged.

### 5.9 Haggling (designed; deferred to MVP-1, Q-S31-6)

`propose-price { item, price }` targeted at a seller records a `proposal` (economy state, `valid_for` ≤ 15 world
minutes) that only the seller perceives. The seller answers `accept-proposal` or `decline-proposal`.
**Acceptance settles exactly like `buy`, at that instant, with every check re-run.** Nothing is reserved, so a
proposal accepted after the buyer spent the money is refused `insufficient-funds`. This keeps §5.7 and needs no
escrow. The flow is the same shape as conversation's invitations. It is MVP-1 because no NPC policy needs it to
show converging prices, and it doubles economy's action surface.

---

## 6. Reference value vs asking price (`ARC-S31-c`)

### 6.1 The reference value (SD-S31-8)

- An item file may carry, in `economy`'s section (whose carriers extend to Items, as `ARC-36` extended
  `ARC-31`), `economy: { reference: <amount ≥ 1> }`. It is optional.
- Genesis fact `reference-declared { item, value }`, `Public`, disclosed with the currency record. A typical
  worth is common knowledge; it reveals nobody's state.
- **It is never charged.** No code path reads it during `buy`. A planted mutation that charges it fails
  (X-3).
- **Uses:**
  - the wealth view (§7);
  - a pricing policy's default and bounds (§10.1);
  - context for cognition: an LM seat is told "typical worth" beside asks.
- **Authored, not derived.** A value derived from trades is a suggestion, which is a reporter's published
  notice (§9), not world truth. The world never computes "the" market price. That would hard-code an
  aggregator into economy and let every controller read trades it never perceived.
- **Market Town:** each of the 20 kinds gets a reference value. Food and drink use their step-10 shop price.
  Goods use a cited US retail figure scaled to the same unit, sourced in the item file's comment (realistic
  defaults). S31-f fills the values.

### 6.2 Who chooses prices

The seat's controller always chooses: a decision model for an NPC, a human through a client, or an agent
over MCP. Economy validates only the bounds. §10 specifies the pluggable policies.

---

## 7. Wealth (`ARC-S31-d`)

### 7.1 The view (SD-S31-9)

`wealth(holder) = wallet + Σ over held kinds k with a reference value: count(k) × reference(k)`, in minor
units with saturating arithmetic. It is also returned as `unvalued: n`, the number of held units with no
reference value.

- **Economy computes it at disclosure,** as today's `Listing` is computed: it reads `Holdings` (it depends on
  inventory) and never stores it. It is a declared, never-carried record `wealth`, disclosed to the holder
  only.
- **It is not stored, so it cannot drift.** Property (MVP-1) adds a term when a property pack exists. Until
  then the formula has exactly these two terms.
- **S30** presents wealth as a derived person attribute, read from this record. S30 owns the attribute view.
  Economy owns the number (§16.4).

### 7.2 Who sees whose wealth

| What | Who |
| --- | --- |
| Wallet, wealth | the holder only |
| Another person's asks and their availability | whoever perceives that person |
| Prices in a shop | whoever perceives the place |
| A sale (`sold`), a price change (`price-set`) | whoever is in that place at that instant |
| How affluent someone *looks* | not in S31. An appearance cue belongs to S26 / S30 presentation. It must be authored, never derived from a wallet (no leak path from `wallet` to appearance) |

### 7.3 The memory of prices seen (S28)

`listed`, `price-set`, `withdrawn` and `sold` are perceived facts. They therefore enter S28 shared records by
receipt (I-S28-1). With S28-d's `Rendered.anchors`, economy's renderer anchors each to its `item` and its
place, so the `item` scope recalls "coffee: 300 at the café (2 days ago), 260 from Grace (today)". No new
memory mechanism is added. Recall shows how old a seen price is, from the record's `at`.

---

## 8. Public notices: a publish/subscribe System Pack (`ARC-S31-e`)

### 8.1 The pack (SD-S31-10)

`systems/notices` (`mineworld-notices`, `SystemId` `notices`) owns topics' configuration, notices and
subscriptions. It depends on no other System (§8.3 says how a price line is checked), and nothing depends on it. It is a
general mechanism: price bulletins are one topic.

```yaml
# configure/notices.yaml
topics:
  market-prices:
    body: prices                     # prices | text   (the body-kind catalog, §8.3)
    publishers: [carol]              # person keys
    valid_for: { min: "1h", max: "48h" }
    per_day: 4                       # at most this many notices per publisher per day
  town-news:
    body: text
    publishers: [hana]
    valid_for: { min: "6h", max: "7d" }
    per_day: 1
defaults:                            # default subscriptions by person tag (§8.5)
  by_tag: { barista: [market-prices], staff: [market-prices] }
```

```yaml
# people/alice.yaml (notices' section; optional; overrides the tag defaults)
notices: { subscribe: [market-prices, town-news] }
```

Refusals at load: an unknown topic in a section or default; a publisher that is not a person in the world; a
`valid_for` minimum above its maximum or outside 1 min … 30 d; `per_day` outside 1 … 96. Each names its key.

### 8.2 Actions

| Action | Payload | Requirement | Refusals |
| --- | --- | --- | --- |
| `publish-notice` | `{ topic, body, valid_for }` | `NONE` (a bulletin is sent, like `send message`) | `not-a-publisher` (`PermissionDenied`), `unknown-topic`, `body-mismatch`, `validity-out-of-range`, `over-daily-limit`, `body-too-large` |
| `subscribe` | `{ topic }` | `NONE` | `unknown-topic`, `already-subscribed` |
| `unsubscribe` | `{ topic }` | `NONE` | `not-subscribed` |

`publish-notice` is offered incomplete to a publisher (the body is free-form, `ARC-34`). `subscribe` and
`unsubscribe` are offered complete, one per topic.

### 8.3 Notice and body

```text
Notice     { id: NoticeId (allocated by notices, from its own counter), topic: TopicId, publisher: PersonId,
             body: NoticeBody, issued_at: WorldTime, valid_until: WorldTime }
NoticeBody = Prices(Vec<PriceLine { item: ItemId, price: NonZeroU64, basis: u16 }>)   ≤ 32 lines, sorted by item
           | Text(NoticeText)                                                         1 … 280 bytes, no control chars
```

- `basis` is how many observations the publisher says the line rests on. It is the publisher's own claim and
  is shown as such.
- **Strong typing, not a JSON blob** (`CLAUDE.md` §4 rule 7). The body-kind catalog is code in `notices`.
  Adding a kind (`departures` for S21) is one variant in this crate plus its renderer, reviewed as a
  vocabulary change. A topic chooses one kind.
- `notices` does not know what a price means. `Prices` is a typed list of item and amount; notices checks that
  each item names a living entity of type `Item`, which is kernel identity and readable without any system
  dependency. A world with `notices` and no `item` can still publish `text` notices.

### 8.4 Delivery: the audience is the subscribers (SD-S31-10)

On `publish-notice`, `notices` states `notice-published { notice }` with
`Visibility::Entities(subscribers of the topic at that instant ∪ { publisher })`, `about: [publisher]`. The
kernel's existing audience rule delivers it to exactly those perceived streams (A-16). Nothing else learns of
it. A person who subscribes later does not receive old notices, but does see the topic's current notices in
the board record (§8.6).

### 8.5 Subscriptions: semantics, and the shape S30 owns the view of

- **Component** `Subscriptions { topics: Vec<TopicId> }` (sorted, ≤ 32) on a Person, written only by `notices`
  in reductions of its own facts: `subscribed { person, topic }` and `unsubscribed { person, topic }`, each
  visible to the person only.
- **Genesis:** the person's `notices.subscribe` if present, otherwise the union of `defaults.by_tag` over the
  person's tags. That is how a cook subscribes to ingredient prices: a role tag, not code.
- **Disclosed to the holder only.** Others cannot see what someone reads.
- **S30 interface.** S30 shows it as the person attribute `subscriptions: [topic]`. The writer is `notices`.
  If S30 rules that every person attribute is owned by one attributes pack, `notices` reads it read-only
  instead and keeps the actions. Either way there is one writer (Q-S31-7).

### 8.6 Current vs remembered

- **Current.** `notices` discloses a declared, never-carried record `notice-board` to each observer. It lists
  the current notices of the topics the observer subscribes to (§3 "current"), computed from world time at
  disclosure. It is lazy, so there is no expiry wake and no expiry fact. An expired notice therefore leaves
  the board at exactly `valid_until`, deterministically.
- **Remembered.** The `notice-published` fact stays in the perceiver's inbox (S27-c's bounded inbox) and in
  S28 memory. The renderer (`notices`' S28 renderer, S31-b) prints validity:
  `"Market prices from Carol, valid until Tue 18:00"`. Recall adds `(expired)` when the consult's time is past
  `valid_until`. The record is the same; only its presentation is time-relative.
- **Agents (S27-d).** An agent receives the notice as an inbox fact. `on_inbox` fires and
  `mineworld://seat/{h}/inbox` publishes `ResourceUpdated` (A-17). `View.notices` lists the board record:
  current notices only, each with `valid_until`. The adapter renders, and it infers nothing (INV-S27-2).

### 8.7 Other topics (non-preclusion)

| Topic | Publisher | Body | When |
| --- | --- | --- | --- |
| `market-prices` | the reporter (§9) | `prices` | S31 |
| `town-news` / newspaper | a person, e.g. S29's newsstand seller | `text` | content only, S31-f optional |
| `weather-warnings` | `weather` (a System) | `text` | later: a System publisher needs `notices` to accept System-stated notices through a checked constructor (`ARC-26`), and `weather` to depend on it. Designed, not built |
| `departures` | `transport` (S21) | `departures` (new body kind) | MVP-1, with S21 |
| happenings | `happenings` | `text` | later; happenings' own public announcements remain as they are |

---

## 9. The market price reporter (`ARC-S31-f`)

### 9.1 The role, and the real practice it follows

The reporter is a Person whose role is to watch the market and publish what prices are. It follows the
real-world practice:

- **USDA Agricultural Marketing Service Market News** (since 1915). Its reporters observe trading at terminal
  markets and shipping points and publish daily price reports, which buyers and sellers use as reference
  prices.
- **BLS Consumer Price Index data collectors** visit outlets and record the prices on display.

Both are a person who observes and then publishes. Neither has access to a central record of every trade.

In Market Town, the reporter is an **existing seated person without a job** (Q-S31-4), given a routine that
visits the café, the store, the street and (after S29) the station. It is named a publisher of
`market-prices` and carries a price book. Removing the reporter means the world has no suggestions. Trading
is unaffected (AC-S31-12).

### 9.2 What it may know (visibility; SD-S31-11)

| Source | Admitted? | Why |
| --- | --- | --- |
| Asks on display where it stands (shop listings, nearby sellers' asks) | **Yes** | already disclosed to anyone present (§5.6) |
| Sales it witnessed (`sold` with `Place` audience while it was there) | **Yes** | perceived like any witness |
| Stock levels on shop listings (`in_stock`) | **Yes** | disclosed on the listing; the supply signal |
| Trades reported to it by sellers | Not in S31 | It needs a reporting action and trust semantics. It is a natural later topic (`trade-reports`), where sellers publish, the reporter subscribes and the same pack carries it |
| A public trade tape (every sale world-wide) | **Declined** | It would make every sale public by default (against `INV-13`), make the reporter redundant, and put an aggregator in the world |

### 9.3 The price book: perception as world state (`price-survey`, SD-S31-11)

A stateless decision model cannot remember what it saw (A-19, F-S31-5). An LM could use S28 memory, but then
the two backends would see different inputs. So the reporter's notes are a **notebook in the world**, disclosed
to its holder, identical for every backend:

- **Pack** `systems/price-survey` (`mineworld-price-survey`). It depends on `economy` (it reads listings and
  hears `sold`) and `presence` (it reads who stands where).
- **Section** `price-survey: { kinds: { categories: [food, drink] }, depth: 32, window: "7d" }` on a person
  makes that person a surveyor. `depth` is entries kept per kind (1 … 64). `window` is the oldest entry kept.
- **Component** `PriceBook { entries: per kind, a ring of { at, place, price, source: ask | sale, in_stock:
  Option<u32>, seller } }`. It is written only by `price-survey`'s reduction of its own fact `price-noted`
  (visible to the surveyor only).
- **How entries arise:**
  1. **Witnessing.** `price-survey` subscribes to economy's `sold`. When the surveyor is present at the sale's
     place at that instant (presence, read), it states `price-noted { source: sale }`. That is exactly the
     sale's audience, re-derived. A test asserts that the set of sales noted equals the set the surveyor
     perceived (AC-S31-10).
  2. **Surveying.** Action `survey`, requirement `SamePlaceAsActor`, offered complete when the surveyor stands
     where asks are on display. It notes every ask disclosed to it there (`source: ask`, with `in_stock`). It
     is an action, so it costs the reporter's time and appears in the journal.
- **Disclosed to the holder only.** It appears in the reporter's Observation for any backend.
- **Removable.** Without `price-survey`, nobody keeps a book. An LM reporter could still publish from S28
  memory, with a weaker test oracle. The bundled deterministic model offers nothing and publishes nothing.

### 9.4 The backend is the seat's decision model

| Backend | Model id | What it does |
| --- | --- | --- |
| Deterministic | `price-reporter` (new in-process `DecisionModel`, `cognition/price-reporter`, registered in `models!`) | Daily life through `rule`'s shared `read`/`act` modules (the `motivated` precedent, SD-DLb-1). Survey band: at a place with asks, `survey` once per visit. Publish band: at configured hours, build a `Prices` body and attempt `publish-notice` by name (A-20). For each kind with ≥ `min_basis` entries in the window: `base = median(sale prices)`, or the median of asks if there are fewer than `min_basis` sales. Then a supply adjustment: if the median noted `in_stock` is below `scarce_below`, `+scarce_permille`; above `plenty_above`, `−plenty_permille`. Integer, floor, clamped to `[1, PRICE_MAX]`. Parameters are bounded through `parameters!` |
| LM | `lm` (`external "lm"`, step-17 P6) | Sees the same Observation: price book, board record, affordances. It composes the body itself. Its request is validated identically. Tests use cassette replay (`ARC-58`) |
| Any other | a user's `DecisionModel` or external Controller Pack | the same contract |

**Swapping the backend changes no contract.** The reporter's inputs are its Observation. Its output is one
`publish-notice` request. The notices pack validates it the same way whoever made it (AC-S31-11).

The median and the supply rule are *one* bundled policy, a model's parameters. They are not a world rule:
another model may publish a mean, a trimmed median or an LM's judgement.

---

## 10. Pricing policies and convergence (`ARC-S31-g`)

### 10.1 Pricing is a decision-model concern (SD-S31-12; Q-S31-5)

- **Where.** A seller's prices are chosen by the controller of the seat that may change them: the person
  itself, or the shop's keeper for the shop.
- **Bundled policies** live in one shared module, `mineworld_rule_controller::price` (the `read`/`act`
  precedent). They are selected by a typed, bounded `pricing:` parameter block that both `rule` and
  `motivated` accept:

  ```yaml
  # decisions.yaml (World Pack), per seat or class
  felix: { model: rule, parameters: { pricing: { policy: toward-suggestion, every: "4h",
            step_permille: 100, deadband_permille: 30, floor_permille: 500, ceiling_permille: 2000 } } }
  ```

  | Policy | Behaviour |
  | --- | --- |
  | `hold` (**default**) | never changes a price. Every existing world behaves exactly as before (AC-S31-9) |
  | `toward-suggestion` | every `every`, for each own ask whose kind has a *current* suggested price on the seat's board: if it differs by more than `deadband`, move `step` of the way toward it. Bounded by `[floor, ceiling] × reference` |
  | `scarcity` | from its own stock only: low stock raises the price by `step`, a full shelf lowers it, within the same bounds. It needs no notices pack |
  | `list-goods` (a flag) | a person holding a good it has not listed lists it at its reference value (rounded), so person-to-person sales arise without an LM |

- **Buyers.** The same block's `buyer: cheapest | nearest` chooses among available `buy` offers for the same
  kind. `nearest` is the default and is today's behaviour.
- **Anything else is a model.** A custom pricing algorithm is a new `DecisionModel` in `models!`, with no new
  plugin mechanism. An LM chooses prices in language, from the same affordances.
- **An expired suggestion is never used.** The policy reads the board record, which lists only current
  notices (X-6).

### 10.2 Why prices converge, and how it is checked

Suggestions are medians of sales and asks the reporter perceived, adjusted for perceived stock. Sellers under
`toward-suggestion` move toward them. When the content changes supply (S31-f's fixture halves bread
production), noted stock falls, the bread suggestion rises, and keepers move their bread asks up. The
checkpoint (§15.1 AC-S31-13) measures exactly that against a control run with the same seed.

---

## 11. Agents over MCP (S27)

| Surface | Change | PR |
| --- | --- | --- |
| `WorldLine` | `currency { name, symbol, exponent, symbol_at }` | S31-g |
| `MeLine` | `wallet` (as today), `wealth { amount, unvalued }`, `asks`, `subscriptions` | S31-g |
| `here.people[]` | `asks: [{ item, price, available }]` for sellers present | S31-g |
| `offers[]` | `price` (exists, A-17 R-S27c-INFO), now also on person-to-person buys; `unavailable` carries the named code | S31-g |
| `View.notices` | current notices from the board record, each with `valid_until` | S31-g |
| inbox | `notice-published`, `sold`, `price-set` read by name, like `spoke` (pr-s27c C-8) | S31-g |
| tools | `list_for_sale(item, price)`, `set_price(item, price)`, `withdraw(item)`, `publish_notice(topic, body, valid_for)`, `subscribe(topic)`, `unsubscribe(topic)`: typed tools with `outputSchema`, as S27-d's | S31-g (after S27-d) |
| amounts | every amount is shown as minor units and as `display` (formatted from the disclosed currency) | S31-g |

**What an agent never sees:** another person's wallet, wealth, holdings counts, subscriptions or price book.
The scenario test asserts their absence (AC-S31-8).

---

## 12. Clients (2D and 3D)

The rules are evaluated on the server only. Clients render disclosed records and send intents
(`ENGINEERING_RULES.md` §§8–9). The client-rules scan (`scripts/check_client_rules.py`) gains two patterns: a
client comparing a wallet with a price, and a client reading `reference` to compute a price. Either fails CI
(X-8).

| Element | 2D (`clients/2d`, default 2D pack) | 3D (slice) |
| --- | --- | --- |
| Money format | `words.money()` reads the disclosed `currency` record. The `/100` is removed. `format.money` = `"{symbol}{amount}"` / `"{amount} {symbol}"` from `symbol_at` | the same, in the shared GDScript formatter |
| Wallet HUD | the existing `panel.wallet` shows the formatted balance and, on hover, wealth | a corner HUD with the same two numbers |
| Price tags | per listed kind on the shop counter sprite; a small tag above a person with asks | `Label3D` tags at shop counters and over a selling person, distance-faded |
| Sale panel | interact with a shop or seller: rows "item · price · available or reason" from offers; *Buy* sends the complete offer | the same panel, the existing interaction prompt opens it |
| My stall | list, change price, withdraw: a price field in major units, converted with the disclosed exponent; the server validates | the same |
| Refusals | `price-changed`, `insufficient-funds`, … mapped to words in `en.po` / `zh_Hans.po` | the same keys |
| Notices | an inbox badge; a notice board panel listing current notices with "valid until" | the same, in the HUD |

Both clients use one shared formatter and one panel model, so no game logic is duplicated (§4 gate questions:
yes and yes). 3D work is limited to the HUD and tags, with no new geometry.

---

## 13. Persistence, determinism and digests

- **Everything is facts, reduced by owners** (`ARC-25`). Asks, shops, wallets, subscriptions, price books and
  notices are reduced from logged facts. A replay reproduces them. Kill-and-resume equals an uninterrupted run
  (AC-6). Wealth and the notice board are views, never stored.
- **Determinism.** No floats (integer permille, floor division). Order: asks by item; offers by seller id,
  then item; notices by id; board by `(topic, publisher)`. The reporter model and the pricing policies are pure
  functions of the observation and the seed (`Draw::salted` with documented indices if they draw at all).
- **Versions.** `economy` 2 → 3 (after WE-a's 2), or 1 → 2 if WE-a has not landed. A save written before is
  refused by name (`ARC-25`). `notices` 1 and `price-survey` 1 are new.
- **Golden fixtures that move:**

  | Fixture | Moves in | Why |
  | --- | --- | --- |
  | `scripts/baselines.txt` `[world market-town]` (all six values) | S31-a; S31-f | `buy` payload, `sold`, `listed`, `currency-declared`; then content (references, keepers, reporter, notices, policies) |
  | `clients/agent/tests/fixtures/market-town-*.json` | S31-a; S31-g | disclosure shapes |
  | `clients/protocol/evidence/affordances-market-town.log` | S31-a | offers carry price and codes |
  | `server/tests/frames/*.json` (market-town frames) | S31-a | |
  | `tools/cli/tests/market/mod.rs` criteria | S31-f | A-24 |
  | `social-cafe`, `lakeside`, `bodies-yard` baselines | **never** | AC-S31-9 |

- **Re-baselines (Q-S31-3).** The honest minimum, without a long-lived integration branch, is **two recorded
  re-baselines of Market Town**:
  - S31-a C-last: the vocabulary change;
  - S31-f C-last: the content.

  Each is one commit, rebased last, with a `reason` line. Recommended alternative for exactly one: S31-a's
  re-baseline is skipped by merging S31-a…S31-e onto `mvp0/s31-integration`, with each PR reviewed there, and
  re-baselining once when that branch merges to `main`. That costs one long-lived branch.

---

## 14. Invariants (proposed)

| Id | Invariant |
| --- | --- |
| I-S31-1 | **No kernel and no contract change.** The diff in `kernel/` and `contracts/` is empty. Money, currency, prices and notices are System Pack vocabulary |
| I-S31-2 | **Only `economy` writes `Wallet`, `Asks` and `Shop`. Only `inventory` writes `Holdings`. Only `notices` writes notices and `Subscriptions`. Only `price-survey` writes `PriceBook`.** Checked by `declaration()` review and the existing ownership tests |
| I-S31-3 | **A sale is atomic.** Every `sold` fact shares its revision with exactly one matching `money-transferred` and one matching `items-transferred`, and neither of those exists for a sale without the other two. Checked over every Market Town save in the scenarios |
| I-S31-4 | **A buyer pays exactly the asking price it submitted.** `money-transferred.amount = buy.price = sold.price` |
| I-S31-5 | **Money is conserved.** The sum of wallets equals the sum of `funded`, at every revision |
| I-S31-6 | **The reference value is never charged.** No settlement path reads it |
| I-S31-7 | **Notices reach exactly the subscribers.** A `notice-published` audience equals the topic's subscribers at that instant plus the publisher |
| I-S31-8 | **The current board never shows an expired notice.** Memory and inbox may keep it, marked by time |
| I-S31-9 | **Nothing private leaks.** Wallets, wealth, holdings counts, subscriptions and price books are disclosed to their holder only |
| I-S31-10 | **Optional throughout.** A world without `economy`, `notices` or `price-survey`, or without a reporter, loads and runs. Its behaviour is unchanged for packs it does not enable |
| I-S31-11 | **No currency name in code.** A grep of `systems/`, `clients/`, `cognition/` and `server/` for the reference worlds' currency names and symbols finds only tests and fixtures |
| I-S31-12 | **Rules on the server only.** The client-rules scan passes with its two new patterns |

---

## 15. Acceptance and adversarial criteria

### 15.1 Step acceptance (fixed before measuring)

| Id | Criterion |
| --- | --- |
| AC-S31-1 | **Two sellers, two prices, both settle.** In a fixture world, the store asks 300 for bread and Grace asks 260. A buyer buys one from each. Both `sold` facts exist with their own prices, and the wallets differ by exactly 560 |
| AC-S31-2 | **Stale price refused.** A buyer observes 300. The keeper sets 320. The buyer submits the observed complete offer. It is refused `price-changed`, and the revision changes no wallet and no holding |
| AC-S31-3 | **Each named refusal is reachable and changes nothing:** `not-for-sale`, `insufficient-funds`, `out-of-stock`, `cannot-carry`, `TooFarAway` (3.1 m from a person seller; outside a shop), `PermissionDenied` (non-keeper), `already-listed`, `price-out-of-range`, `not-held` |
| AC-S31-4 | **No half-trade across a kill.** `mineworld run worlds/market-town` at seed 7 is killed with SIGKILL at 20 seeded wall offsets during a 10-day run with heavy trading, and resumed each time. Every save satisfies I-S31-3 and I-S31-5, and the final digest equals the uninterrupted run's. Plus the §5.7 property test: over 10 000 seeded states, `validate` admitting a sale implies both reductions accept it |
| AC-S31-5 | **Prices change over time.** In Market Town 30 days, at least one keeper's ask changes, through `price-set` facts only |
| AC-S31-6 | **The currency renames with zero code.** Changing `configure/economy.yaml` to `{ name: crown, symbol: cr, exponent: 0, symbol_at: after }` in a copy changes only `currency-declared`, the agent `WorldLine` and the client display (2D shot). The diff outside the world's directory is empty |
| AC-S31-7 | **Undeclared currency is refused.** `economy` without `currency:` fails `mineworld validate` with `economy-currency-undeclared` |
| AC-S31-8 | **MCP view.** An agent seat in Market Town sees the café's listed prices with currency display, sees Grace's asks when beside her, and buys from both. `inspect` on another person discloses no wallet, wealth, holdings counts, subscriptions or price book |
| AC-S31-9 | **Worlds without economy are unchanged.** Social Café, Lakeside and bodies-yard baselines are byte-identical before and after every S31 PR. With all S31 packs installed in the build, a world that does not enable them states nothing new |
| AC-S31-10 | **The reporter knows only what it perceived.** The set of `price-noted { source: sale }` equals the set of `sold` facts in the reporter's perceived stream. Moving the reporter's routine away from the store removes store sales from its book |
| AC-S31-11 | **Backend swap changes no contract.** The reporter seat on `price-reporter` and on `lm` (cassette replay) both publish notices that validate. The notices' type and audience are identical; only body values differ. No Rust or protocol file differs between the two runs |
| AC-S31-12 | **No reporter, still trading.** Market Town with the reporter's `publishers` entry removed (or `notices` removed) runs 30 days: sales, wages and E-9's criteria hold, and no `notice-published` exists |
| AC-S31-13 | **Integration checkpoint: prices converge with supply.** Market Town, 30 days, seed 7, S31-f content. (a) At least 20 `market-prices` notices. (b) A subscribed NPC's inbox (agent library) and S28 memory (`mineworld-memory inspect`) show a bulletin, and the board shows it only while current. (c) At least one keeper's ask moves toward the current suggestion at least twice. (d) Under the supply-shock fixture (bread production halved from day 10), bread's suggested price on day 20–30 exceeds the control run's, and the store's bread ask follows it within `ceiling`. (e) At least 10 person-to-person sales |
| AC-S31-14 | **E-9 still holds** over 300 days (zero `wage-unpaid`, money conserved, nobody above six, no wallet below the cheapest price), with the criteria test updated for S29-b and S31-f content |
| AC-S31-15 | **Both clients.** 2D and 3D drives: walk to the store, read the price tag, buy, see the wallet drop by the price. Walk to Grace, buy flowers. Refusals show their words. Recorded shots on three OSes |

### 15.2 Planted mutations (each must fail by name, then be reverted)

| Id | Plant | Must fail with |
| --- | --- | --- |
| X-1 | `resolve` emits `money-transferred` but drops `items-transferred` | I-S31-3 scan in AC-S31-1 and AC-S31-4 |
| X-2 | validate skips the `price-changed` check | AC-S31-2 |
| X-3 | settlement charges the reference value instead of the asking price | I-S31-4 / I-S31-6 test |
| X-4 | `notice-published` audience set to `Public` | I-S31-7: a non-subscriber's perceived stream holds the notice (AC-S31-13 (b) negative arm) |
| X-5 | the board record omits the `valid_until` filter | "expired notice shown as current" test |
| X-6 | `toward-suggestion` reads remembered notices instead of the board | expired-suggestion policy test |
| X-7 | `price-survey` notes every `sold` regardless of presence | AC-S31-10 |
| X-8 | 2D client greys out *Buy* by comparing wallet with price | client-rules scan |
| X-9 | economy discloses `Wallet` to any observer | AC-S31-8 |
| X-10 | the 2D formatter keeps `/100` | AC-S31-6's `exponent: 0` shot shows "0.03 cr" |
| X-11 | inventory's reduction refuses a transfer that `admit_transfer` admitted (test-only) | `FactRefusedByOwner { system: inventory }` reported, and the I-S31-3 scan names the revision |

---

## 16. PR split, placement and interactions

### 16.1 Dependency graph

```text
WE-a C4 (economy config, if first) ┐
S29-b (newsstand, if first) ───────┤
                                   ▼
             S31-a  currency + asks + settlement + refusals ─────┬──────────────► S31-g agent view
                     (economy; vocabulary re-baseline)           │                (after S27-d)
                                   │                             │
S31-b  notices pack ───────────────┼─────────────┐               ├──► S31-e clients 2D + 3D
(independent of S31-a)             │             │               │
                                   ▼             ▼               │
                    S31-c  price-survey     S31-d  price-reporter model + pricing policies
                                   │             │               │
                                   └──────┬──────┘               │
                                          ▼                      │
                     S31-f  Market Town composition + 30-day checkpoint + content re-baseline
                                          │
                                          ▼
                              S31-h  haggling (MVP-1)
```

### 16.2 The PRs

| PR | Scope | Depends on | Integration checkpoint | Adversarial | Implementer |
| --- | --- | --- | --- | --- | --- |
| **S31-a** asks and settlement | <ul><li>C1, specification first: MODULE_SPEC §4.1 economy row, CORE_CONCEPTS §8 note, `ARC-S31-a`/`-b`/`-c`/`-d`, the `ARC-38` amendment (item 5 lifted; QS-44 superseded).</li><li>C2: currency configuration, `currency-declared`, the record.</li><li>C3: `Asks`, `Shop{operator, keepers}`, the section keys, `listed`.</li><li>C4: the four actions, codes, offers, disclosure, the wealth record.</li><li>C5: `sold`, settlement, the §5.7 property test and X-11.</li><li>C6: Market Town `configure/economy.yaml` (currency only); the re-baseline</li></ul> | WE-a C4 if merged (rebase); 12d merged | fixture world AC-S31-1…3, AC-S31-6, AC-S31-7; Market Town 30 d replays; AC-S31-4 | X-1, X-2, X-3, X-9 | **Opus** (contract reshaping, settlement proof, many consumers) |
| **S31-b** notices | `systems/notices`: configuration, section, three actions, `notice-published`, `subscribed`/`unsubscribed`, board record, S28 renderer (Python, with `Rendered.anchors` once S28-d lands), `ARC-S31-e` | none (parallel with S31-a) | fixture world: publisher, two subscribers, one non-subscriber; expiry at the boundary; kill-and-resume | X-4, X-5 | **Opus** (new pack; audience and time semantics) |
| **S31-c** price-survey | `systems/price-survey`: section, `PriceBook`, `survey`, witnessed `sold`, `ARC-S31-f` (book half) | S31-a | fixture world: AC-S31-10 | X-7 | **Haiku** (well-specified pack on a frozen pattern) |
| **S31-d** reporter and policies | `cognition/price-reporter` (`models!` line); `mineworld_rule_controller::price` (`hold`, `toward-suggestion`, `scarcity`, `list-goods`, `buyer`); the `pricing:` block in `rule` and `motivated`; template determinism test (AC-6); LM cassette for AC-S31-11; `ARC-S31-f` (backend half), `ARC-S31-g` | S31-a, S31-b, S31-c | fixture world 10 d: notices published, a policy follows; backend swap | X-6 | **Haiku** for the deterministic model and policies from a commit plan; **Opus** for the LM cassette wiring |
| **S31-e** clients | shared formatter; HUD; tags; sale panel; stall; notice board; i18n; client-rules patterns; drives and shots | S31-a (S31-b for the board) | AC-S31-15 | X-8, X-10 | **Haiku** (2D); **Opus** review of 3D frames |
| **S31-f** Market Town | reference values with sources; keepers; Grace and two others list goods; reporter person, routine and book; `configure/notices.yaml`; subscriptions by tag; pricing policies in `decisions.yaml`; supply-shock fixture; criteria test update; the content re-baseline | S31-a…d; S29-b if merged | **AC-S31-13** (30-day convergence), AC-S31-12, AC-S31-14 (300 d in nightly), AC-S31-5 | the AC-S31-12 removal runs | **Haiku** (content and measurement against fixed criteria) |
| **S31-g** agent view | §11 surfaces and tools; the AC-S31-8 scenario over MCP | S31-a, S31-b; **S27-d merged** | AC-S31-8 over MCP | X-9 (through the agent) | **Haiku** |
| **S31-h** haggling | §5.9 | S31-f | MVP-1 | | Opus |

### 16.3 Placement: MVP-0 (Q-S31-1)

- **Recommendation:** S31-a…S31-g in **MVP-0**, after Milestone F's checklist is handed over (the S29 timing
  rule), and in before S23's first public release candidate. S31-h goes to MVP-1.
- **Why MVP-0.** The operator ties realism to trading. The work is System Packs and content, with no kernel
  and no transport. Market Town is MVP-0's market world, and S27's agents already buy there.
- **The cost.** Seven PRs. Their critical path is S31-a → S31-c/d → S31-f. S31-b, S31-e and S31-g run in
  parallel worktrees.
- **If the release must not wait.** S31-a, S31-e and S31-g are the release subset. S31-b, c, d and f follow
  the release within MVP-0.

### 16.4 Interactions

| Step | Interaction |
| --- | --- |
| **S29** | S29-b's newsstand becomes a normal shop with a keeper, so its seller may change the paper's price. F-S31-6 is resolved by lifting `ARC-38` item 5 in S31-a. If S29-b merges first, S31-a's C3 converts its `prices` like the others, with no content change. The newsstand seller is a natural `town-news` publisher (§8.7). `departures` is a future body kind |
| **S27** | S27-c's `price` on offers is kept. S27-d's inbox and `subscriptions/listen` are reused unchanged. S31-g adds typed tools following S27-d's conventions |
| **S28** | Notices, sales and price changes are perceived facts, so they become shared records by receipt. S28-d's `Rendered.anchors` gives item-anchored price memory. The renderers are S31-b's (notices) and S31-a's (economy, filed with S31-g if S28-d is later) |
| **S30** | S30 owns the person-attribute view. Wealth is a derived attribute read from economy's `wealth` record. `subscriptions` is an attribute whose writer is `notices` (Q-S31-7). Role-based default subscriptions read person tags today, and S30's role attribute if S30 defines one |
| **WE-a** | §5.8: modifiers apply to Organization asks through `effective_price`. Both share `configure/economy.yaml` |
| **S21 / step-21** | `fares` are asks of a transport Organization at a ticket office. `fare-due` is answered like `wage-due`. Remote purchase plus delivery is a future pack on §5.7's settlement |
| **DL-c (needs, MVP-1)** | Hunger makes food purchases needs-driven, so the "essential consumables" topic gains real demand |

---

## 17. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S31-1 | Persons listing goods fill hands (capacity 6) and stall the give loop | `list-goods` lists only goods held; E-9's six-cap criterion is re-run in S31-f; buyers' policy buys goods only below `ceiling × reference` |
| R-S31-2 | Feedback oscillation: sellers chase suggestions that chase sellers | deadband, step ≤ 10 %, the reporter's window of 7 days, and bounds; AC-S31-13 records the per-day series so oscillation is visible |
| R-S31-3 | The supply signal is too weak in 12 people for a measurable shift | the supply-shock fixture is strong by construction (halved production); the criterion is fixed before measuring, and a FAIL is recorded, not tuned away (`ARC-23`) |
| R-S31-4 | Re-baselines collide with S29-a C5, 12d and WE-a C5 | each re-baseline commit rebased last, scheduled by the primary in a quiet window |
| R-S31-5 | `rule`'s determinism template breaks with the `pricing:` block | the default `hold` produces no requests; the template test runs every installed model unchanged |
| R-S31-6 | S29-b lands before S31-a with a goods shop | S29-b's design notes F-S31-6; either a ruling for S29-b, or S31-a first |
| R-S31-7 | LM reporter's bulletins are implausible | validation bounds the body (declared kinds, price range); a `body-mismatch` refusal; the board shows the publisher, so trust is the reader's |
| R-S31-9 | A future change makes the two owners' checks disagree, so that a kernel fault leaves a half-applied sale in memory (F-S31-8) | the §5.7 property test and the I-S31-3 scan run in CI on every economy or inventory change. A transactional reduction is a kernel question, and S31 does not open it (I-S31-1) |
| R-S31-8 | Item names are not disclosed (F-S27c-9), so price tags and bulletins name kinds by key | 12d's `modern-goods` 0.2.0 names; until then tags show the presentation pack's item word |

---

## 18. Questions for the operator (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S31-1** | **Placement.** Is S31 in MVP-0 (after Milestone F's checklist, before the first public release), or in MVP-1? | **MVP-0**, with S31-h (haggling) in MVP-1. If the release must not wait for all of it, the release subset is S31-a + S31-e + S31-g (§16.3) |
| **Q-S31-2** | **One currency per world**, and no barter in S31? | **Yes.** One general equivalent. Multiple currencies and barter are later packs that need nothing from S31 (§4.2) |
| **Q-S31-3** | **Re-baselines.** Two recorded Market Town re-baselines (S31-a vocabulary, S31-f content), or one, through an integration branch? | **Two**, each one commit rebased last with a reason line. An integration branch delays review of S31-a's settlement change for weeks (§13) |
| **Q-S31-4** | **The reporter.** An existing seated person without a job (as Q-S29-6 ruled for the newsstand), or a new person? | **An existing person**, not the S29-b seller: no new character (S26 bytes), no new seat, fewer moving ids. S31-f names them |
| **Q-S31-5** | **Pricing policies.** Bundled policies as a `pricing:` parameter block shared by `rule` and `motivated`, with anything else a new decision model; or only new decision models? | **The shared block**, defaulting to `hold`, so that the town's existing models can trade without a third model per seat. A custom algorithm is still a model in `models!` (§10.1) |
| **Q-S31-6** | **Haggling** (propose, accept, decline): in S31 (MVP-0), or MVP-1? | **MVP-1.** Convergence does not need it, and it doubles economy's action surface. The design (§5.9) keeps the settlement unchanged |
| **Q-S31-7** | **Subscriptions under S30.** Does `notices` own the `subscriptions` component (S30 shows it as an attribute), or does S30's attributes pack own it (`notices` reads it)? | **`notices` owns it.** The pack that acts on a datum writes it (single ownership), and a world without S30 still has subscriptions. S30's view reads it, as it reads wealth from economy |
| **Q-S31-8** | **Shop keepers.** Organizations change prices only through named keepers standing in the shop (until WE-b's Organization seats, MVP-1)? | **Yes.** `keepers:` in the shop section: economy-owned authority, with no dependency on employment. Market Town: Alice for the café, Felix for the store |

---

## 19. Proposed edits elsewhere (made by the PRs named)

### 19.1 `docs/MODULE_SPEC.md` §4.1 (S31-a C1; S31-b; S31-c)

- **economy row:**
  - `currency` (configuration);
  - `shop: { at, prices, keepers }` on organizations;
  - `asks: { item: price }` on people;
  - `reference: <amount>` on items.
- **New rows:** `notices` (people: `{ subscribe: [topic] }`) and `price-survey` (people).
- **`configure/notices.yaml` and `configure/economy.yaml`:** the field references, generated per S25 25b.

### 19.2 `docs/CORE_CONCEPTS.md`

- **§7 Item, a note:** an item kind may carry an authored reference value, which is never a price.
- **§8 Organization, a note:** an Organization sells through a shop, and its keepers change its prices.

Neither note adds an ontology term.

### 19.3 `docs/ARCHITECTURE.md`

One paragraph in the systems section: a settling owner may state another owner's fact in one dispatch. This
is `ARC-26` restated for exchanges, with §5.7's three-reason argument.

### 19.4 `docs/DECISIONS.md` (placeholders; real ids at approval, from `ARC-110` / `DEP-62`)

| Placeholder | Title |
| --- | --- |
| `ARC-S31-a` | The currency is economy configuration: one per world, ISO-4217-shaped, required with economy |
| `ARC-S31-b` | Seller-set asks for Persons and Organizations; `buy` carries the observed price; named refusals; one settling owner (amends `ARC-38` items 3 and 5, step-10 QS-44) |
| `ARC-S31-c` | An optional authored reference value, never charged |
| `ARC-S31-d` | Wealth is a holder-only view, never stored |
| `ARC-S31-e` | `notices`: topics, subscriptions, audience = subscribers, lazy expiry |
| `ARC-S31-f` | The price book as perception-derived world state, and the reporter as a seat whose backend is its decision model |
| `ARC-S31-g` | Pricing policies are decision-model parameters and models, defaulting to `hold` |
| `ARC-S31-h` | Haggling settles at acceptance without escrow (MVP-1) |
| `DEP-S31-a` | Money libraries (`rusty-money`, `iso_currency`) declined: amounts are bare `u64` and formatting is presentation, so a dependency would add ISO tables that a fantasy currency cannot use. Order-book and matching-engine crates declined: MineWorld has posted prices, not an exchange |

### 19.5 `overall.md` (the primary session)

- Add S31 to §3 with output, dependencies, acceptance (AC-S31-13 as the checkpoint) and adversarial
  criteria (§15.2).
- Add its row in the step-document map.
- Record the two re-baselines beside the "Digests" ruling.
- Record the id assignment.

### 19.6 Other designs (their owners)

- **S29-b:** F-S31-6 and the keeper key.
- **S30:** the `wealth` record and the `subscriptions` attribute shape (§8.5, Q-S31-7).
- **S27-d:** nothing changes; S31-g extends after it merges.
- **WE-a C4:** if it lands after S31-a, it rebases onto `Asks`, and `effective_price` reads the operator's
  asks.
