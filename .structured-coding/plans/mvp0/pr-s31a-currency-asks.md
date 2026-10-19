# PR S31-a — Currency as data, seller asks for Persons and Organizations, and one atomic settlement

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: freeze with the recommended answers)

```text
Design revision:        revision 1 (2026-10-15), FROZEN 2026-10-18
Author:                 S31-a PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s31a, branch docs/pr-design-s31a,
                        planning base origin/main @ dd357ca6 (#222, step-31 APPROVED, merged)
Approved by / evidence: §14.1: Q-S31a-1 … 3 ruled by the operator (answered directly in the primary
                        session), Q-S31a-4 by the primary, 2026-10-15; relayed by the coordinator. Frozen
                        2026-10-18 by the operator in the primary session, every question as recommended
                        (§14.1)
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of 12d (#171,
                        open at 2026-10-18), WE-a (#189, open), S25a (#200, merged) and S29-b
Start condition:        step-31 §16.5: nothing in S31 starts until Milestone F's checklist has been handed
                        to the operator (the S29 timing rule, overall.md "Milestone F"); then 12d (#171)
                        merged (NOT met at 2026-10-18); then this design frozen (met: 2026-10-18); then the
                        primary session's dispatch
Execution contract:     §12 (filled, endpoint authority included; ends at READY FOR OPERATOR REVIEW)
Implementer:            Opus-class (step-31 §16.2, the S31-a row: contract reshaping, settlement proof,
                        many consumers); reviewer the primary session plus an Opus logic review
Lifecycle:              DESIGN FROZEN 2026-10-18 (revision 1)
```

**Effort:** `mvp0` · **Step:** [`step-31-wealth-prices.md`](step-31-wealth-prices.md) (APPROVED 2026-10-15;
Q-S31-1 … Q-S31-8 ruled, Q-S31-1 and Q-S31-6 re-ruled; §18.1) §§0, 1, 2, 4, 5.1 … 5.8, 6, 7.1 … 7.3, 13, 14
(I-S31-1 … I-S31-6, I-S31-9 … I-S31-11), 15 (AC-S31-1 … 4, 6, 7, 9; X-1, X-2, X-3, X-9, X-11), 16.2 (the S31-a
row), 16.5, 19.1 … 19.4 · **Parent:** [`overall.md`](overall.md) · **Revises the code of:**
[`step-10-market.md`](step-10-market.md) §4.5 (E-C4: economy, `ARC-38`) · **Coordinates with:**
[`pr-we-a-happenings.md`](pr-we-a-happenings.md) (PR #189, open: C4 creates `configure/economy.yaml`),
12d (PR #171, open: item names, economy test support, a Market Town re-baseline), S25a (PR #200, open:
generated configuration and section reference), S29-b (station newsstand) · **Followed by:** S31-b (parallel),
then S31-c, S31-e, S31-g, S31-h · **Binding parents:** `CLAUDE.md` §§2, 3, 4 (items 1, 2, 3, 5, 7, 8, 9, 10,
12, 13, 15, 16); `docs/CORE_CONCEPTS.md` §§7, 8, 12.1, 15.2; `docs/MODULE_SPEC.md` §4.1;
`docs/DECISIONS.md` `ARC-25`, `ARC-26`, `ARC-34`, `ARC-35`, `ARC-37`, `ARC-38`, `ARC-61`.

**Decision ids** (assigned by the step, 2026-10-15; confirmed free on `origin/main @ dd357ca6` by
`grep -n "ARC-11[1-8]\|DEP-63" docs/DECISIONS.md`, no match). This PR writes **`ARC-111`** (currency),
**`ARC-112`** (asks and settlement), **`ARC-113`** (reference value), **`ARC-114`** (wealth view; the
`wealth-band` half is added by S31-f as a dated note, SD-S31a-8) and **`DEP-63`** (money and matching
libraries declined). It writes dated notes under `ARC-38` (item 5 lifted; QS-44 superseded) and under `ARC-35`
(Q-S31a-1, ruled (a)). `ARC-115` … `ARC-118` belong to S31-b … S31-h. No other id is used.

No hosted model is called by this design or by anything it specifies for CI.

---

## 0. Binding inputs, exactly restated

- **Scope (step-31 §16.2, S31-a row).** "C1, specification first: MODULE_SPEC §4.1 economy row, CORE_CONCEPTS
  §8 note, `ARC-111`…`ARC-114`, the `ARC-38` amendment (item 5 lifted; QS-44 superseded). C2: currency
  configuration, `currency-declared`, the record. C3: `Asks`, `Shop{operator, keepers}`, the section keys,
  `listed`. C4: the four actions, codes, offers, disclosure, the wealth record. C5: `sold`, settlement, the
  §5.7 property test and X-11. C6: Market Town `configure/economy.yaml` (currency only); the re-baseline."
  Depends on "WE-a C4 if merged (rebase); 12d merged". Checkpoint "fixture world AC-S31-1…3, AC-S31-6,
  AC-S31-7; Market Town 30 d replays; AC-S31-4". Adversarial "X-1, X-2, X-3, X-9". Implementer **Opus**.
- **Start (step-31 §16.5).** "Nothing starts before Milestone F's checklist has been handed to the operator
  (the S29 timing rule). Each PR starts only once its own PR design is frozen." First wave: S31-a and S31-b
  in parallel.
- **Rulings that bind and are not re-asked (step-31 §18).** Q-S31-1 / Q-S31-6 re-ruled: all of S31 is MVP-0
  and release-blocking. Q-S31-2: one currency per world, no barter. Q-S31-3: **two** recorded Market Town
  re-baselines, S31-a (vocabulary) and S31-f (content), each one commit rebased last with a reason line.
  Q-S31-8: shop keepers, as `keepers:` in the shop section.
- **The coupling the coordinator named.** WE-a (frozen 2026-10-12, PR #189 open at C0–C4) creates
  `configure/economy.yaml` with `price_modifiers`, `Shop.adjustments`, `Shop::effective_price` and economy
  `SystemVersion` 2. S31-a puts `currency:` in the same file. Whichever lands second extends the other's type
  (§5.2, R-S31a-1).
- **R-LOAD-1 (coordinator rule, in force).** Locally: only the targeted test files of the commit, plus
  `cargo fmt --check`, `cargo clippy -p <crate> --all-targets -- -D warnings`. Never a local
  `cargo test --workspace`, never the whole `tools/cli` suite. CI on the exact head is the gate (§12).

---

## 1. Goal

Make the world's money a declared datum and its prices a seller's choice. `economy` gains a required world
configuration naming one currency (name, symbol, minor-unit exponent, symbol placement), disclosed to every
observer so no client or agent assumes a scale. Every seller — a Person for itself, an Organization through
the shop it runs and the keepers that shop names — holds **asks**, one price per kind, which it lists,
re-prices and withdraws by action. A buyer submits the price it saw; economy, the one settling owner, decides
the whole exchange in one dispatch and states `sold`, its own `money-transferred` and inventory's
`items-transferred` together, or refuses by a named code and moves nothing. An item kind may carry an authored
reference value that is never charged; wealth (wallet plus reference values of holdings) is a holder-only view,
computed at disclosure and never stored. Nothing changes in `kernel/` or `contracts/` (I-S31-1). Market Town
gains its currency and nothing else; its re-baseline is the vocabulary change only.

---

## 2. Scope

### 2.1 In scope

1. **`systems/economy`** (the only crate whose behaviour changes):
   - `src/configuration.rs` (new, or extended if WE-a C4 merged first): `EconomyConfiguration { currency:
     Currency, price_modifiers }`; `Currency { name, symbol, exponent, symbol_at }` with its bounds (§5.2).
   - `src/currency.rs` (new): `CurrencyProcess` (an economy-owned Process, never woken) and the disclosed
     `currency` record (§5.3).
   - `src/component.rs`: `Asks`, `Ask`; `Shop { operator, keepers }` (schema 2); `ReferenceValue` on an Item
     entity; the `shop` listing view (shape unchanged, SD-S31a-12); the `asks` and `wealth` views (§5.6, §5.8).
   - `src/section.rs`: the section's new optional keys `asks`, `shop.keepers`, `reference`, and its carriers
     extended to Items (`ARC-36`'s precedent, `systems/bodies/src/section.rs:116`); per-carrier refusals by
     name (§5.4).
   - `src/action.rs`: `Buy { item, price }` with an optional target; `ListForSale`, `SetPrice`, `Withdraw`
     with the optional `for: shop`; `TRADE_RANGE`; the requirements (§5.5).
   - `src/sale.rs` (new): `judge_sale` (the one decision both `offers` and `validate` call) and `settle`
     (the three facts, through `pay` and `inventory::transfer`) (§5.7).
   - `src/listing.rs` (new): the seller actions' judgement and their facts (§5.6).
   - `src/money.rs`: `pay` unchanged; `PRICE_MAX`.
   - `src/event.rs`: `CurrencyDeclared`, `ReferenceDeclared`, `ShopOpened` (schema 2), `Listed`, `PriceSet`,
     `Withdrawn`, `Sold`; the existing four unchanged.
   - `src/offer.rs`, `src/system.rs`, `src/lib.rs`, `src/codec.rs`: the declaration, `validate`, `resolve`,
     `react`, `offers`, `discloses` for the above. `SystemVersion` 1 → 2 (or 2 → 3 after WE-a, §13 step-31).
   - `tests/`: `support/mod.rs` (seeds the currency configuration, references, asks, keepers);
     `buy.rs` (QS-44 superseded); new `currency.rs`, `asks.rs`, `sale.rs`, `settlement.rs`, `wealth.rs`.
   - `README.md`.
2. **`worlds/market-town`**: `configure/economy.yaml` (currency only, ISO 4217 USD), `world.yaml`'s
   `configure:` entry, the stale comment in `organizations/cafe-company.yaml` (F-S31a-14), `README.md`.
   No other content changes (no keepers, no asks, no references: S31-f).
3. **`tools/cli/tests`**: `market/mod.rs` (mirrors of `shop-opened` v2, `listed`, `sold`; the cheapest price
   read from genesis `listed`; the I-S31-3 sale scan), `market_composition.rs` ("without economy" also drops
   the configuration, F-S31a-7), new `market_settlement.rs` (AC-S31-4 kills, AC-S31-6 rename, the
   vocabulary-parity measurement).
4. **`tests/acceptance/tests`**: `ac1_composability.rs` (Q-S31a-1 ruled (a)); new
   `currency_names.rs` (I-S31-11).
5. **`clients/agent/tests/fixtures`**: the three recordings re-recorded and `SOURCE.md` re-hashed (F-S31a-10).
   No `clients/agent/src` change (S31-g).
6. **`scripts/baselines.txt`**: `[world market-town]` re-recorded with one `reason S31-a` line (Q-S31-3).
7. **Documentation**: `docs/DECISIONS.md` (`ARC-111` … `ARC-114`, `DEP-63`, dated notes under `ARC-38` and
   `ARC-35`, Q-S31a-1 ruled (a)); `docs/MODULE_SPEC.md` §4.1 economy row and the `configure/economy.yaml`
   reference; `docs/CORE_CONCEPTS.md` §7 and §8 notes; `docs/ARCHITECTURE.md` one paragraph (step §19.3).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| `notices`, subscriptions, the board record | S31-b |
| `price-survey`, the price book | S31-c |
| The reporter model, pricing policies, the `pricing:` block | S31-d |
| 2D and 3D formatter (the `/100` at `clients/2d/scripts/hud/words.gd:73`), price tags, sale panel, stall, refusal words, client-rules patterns | S31-e |
| Market Town keepers, asks, references, reporter, the second re-baseline, `wealth-band` | S31-f (and S31-f2) |
| Agent `WorldLine.currency`, `MeLine.wealth/asks`, person-seller prices on offers, typed tools | S31-g |
| Haggling | S31-h |
| Economy's S28 renderer (`listed`, `price-set`, `sold` as memories) | S31-g, if S28-d is later (step §16.4); no renderer exists today (F-S31a-18) |
| Any change under `kernel/`, `contracts/`, `persistence/`, `server/`, `cognition/`, `systems/inventory`, `systems/item-transfer` | never in this PR (I-a-1) |

### 2.3 Dependencies

- **Required before C0:** Milestone F's checklist handed over (operator); 12d (#171) merged (step §16.2;
  it edits `systems/economy/tests/support/mod.rs`, Market Town's items and baselines, F-S31a-12).
- **Either order, rebase rule:** WE-a C4 (#189). If merged first, S31-a extends its `EconomyConfiguration`
  and its `Shop.adjustments`, and `effective_price` reads the operator's asks (step §5.8). If S31-a lands
  first, WE-a C4 rebases onto `Asks` (step §19.6).
- **Either order, convention rule:** S25a (#200). If merged first, every section and configuration type this
  PR changes derives the schema S25a's convention requires and its generated `docs/reference/` files are
  regenerated in the same commit (F-S31a-13).
- **Either order, no content change:** S29-b. Its newsstand's `shop.prices` keeps its shape, so C3 seeds it
  like the others; `ARC-38` item 5 is lifted in C1 (F-S31-6).

---

## 3. Audit (`origin/main @ dd357ca6`)

### 3.1 Source anchors

| Id | Fact | Source |
| --- | --- | --- |
| A-1 | `Wallet { balance: u64 }`, written only by reductions of `funded` and `money-transferred` | `systems/economy/src/component.rs:14-35`; `system.rs:151-192` |
| A-2 | `Shop { operator, prices: Vec<Price> }` on the Place; `price(item)` by binary search | `component.rs:66-101` |
| A-3 | The shop listing is disclosed as the `shop` component type with a `Listing { operator, listed: [{ item, price, in_stock }] }` payload, stock read from the operator's `Holdings` | `component.rs:128-165`; `offer.rs:51-55`; `system.rs:256-261` |
| A-4 | Section `economy: { wallet (required), shop?: { at, prices } }`, `deny_unknown_fields`, carried by Person and Organization; a shop on a Person refused `PreconditionFailed` | `section.rs:32-46, 63, 107-112` |
| A-5 | `buy { item }`: no target; `validate` → `NoSupportedInteraction` outside a shop or for an unpriced kind; availability folds three reasons into `TargetUnavailable` | `action.rs:18-59`; `system.rs:88-109` |
| A-6 | `resolve` states `pay(buyer → operator, Place(shop))` and `inventory::transfer(operator → buyer, 1)`; a refusal there is `FactRefusedByOwner` | `system.rs:113-137` |
| A-7 | `admit_payment` returns `PreconditionFailed` for every reason (zero, self, non-holder, short balance) | `money.rs:32-45` |
| A-8 | `admit_transfer`: `PreconditionFailed` (zero, self, non-holder, undeclared, short), `TargetUnavailable` (cannot take); the same function at validate, at `transfer`, at reduction | `systems/inventory/src/admit.rs:62-112` |
| A-9 | `PERSON_CAPACITY = 6`; organizations unbounded | `admit.rs:18, 35-49` |
| A-10 | `give`: Person → Person, `same_place().within(GIVE_RANGE = 3 000 mm).requiring_target_available()`; one complete offer per held kind | `systems/item-transfer/src/action.rs:12, 48-53`; `offer.rs:25-49` |
| A-11 | `Offer::refused(reason)`: "Perception reports the refusal **before** it evaluates the requirement, so the offer and the dispatch answer the same, in the same order" | `systems/presence/src/interaction.rs:207-216` |
| A-12 | `SpatialRequirement::evaluate`: availability, then place, then range; `SamePlaceAsActor` and a range with no target are `PreconditionFailed` | `contracts/src/spatial.rs:470-525` |
| A-13 | `Rejection::System { code: RejectionCode, detail }`; `RejectionCode::from_static` checked at compile time | `contracts/src/action.rs:147-173, 238-243` |
| A-14 | `PackConfiguration { Configuration, FACTS, references, requires, attachments, check, seed }`; nothing lets a pack declare its configuration mandatory | `authoring/src/configuration.rs:97-145` |
| A-15 | `ARC-61` item 6: configuration seeds before sections; item 8: "An owner states its configuration facts `Visibility::SystemInternal` with no subjects … guidance, not a loader rule" | `docs/DECISIONS.md:4580-4612` |
| A-16 | Calendar discloses declared, never-carried records about the observer's place; WE-a keeps economy's configuration in a never-woken Process (`price-book`) because no entity carries it at seeding | `systems/calendar/src/system.rs:278-305`; PR #189 `systems/economy/src/book.rs` |
| A-17 | `mineworld validate` is `WorldPack::read` plus `load(EPOCH)`: genesis is reduced, so a reduction refusal fails validate by name | `tools/cli/src/main.rs:548-550` |
| A-18 | Reduction is not transactional; a `KernelError` after writes is a system defect; the persisted world journals the fault as the request's outcome, with no facts, and goes on | `kernel/src/dispatch.rs:54-66`; `persistence/src/world.rs:259-281` |
| A-19 | The paced rule attempts an available complete affordance, chosen uniformly, on 20 % of consults; `motivated` lists every available complete affordance as a choice | `cognition/rule-controller/src/offered.rs:26-52`; `cognition/motivated/src/options.rs:285-289` |
| A-20 | AC-1 check 3: `configure:` in Market Town may list only `GENERIC_PACKS = ["calendar", "weather"]`, and `configure/` may hold only their files | `tests/acceptance/tests/ac1_composability.rs:1032, 1093-1125, 1321-1330`; `ARC-35` note 2026-10-09 (`DECISIONS.md:2860-2876`) |
| A-21 | The market test mirror reads facts by slug, asserts schema version 1 for each, and requires exactly the mirror's fields; E-9's "cheapest price" is the minimum of `shop-opened.prices` | `tools/cli/tests/market/mod.rs:5-13, 226-289, 546-555, 600-604` |
| A-22 | `market_composition`'s `without("economy")` deletes every `  - economy` line of `world.yaml` and the `economy:` sections, nothing in `configure/` | `tools/cli/tests/market_composition.rs:38-46, 103-110` |
| A-23 | Only Market Town enables economy; no other test world or fixture does | `git grep -ln "^  - economy" -- '*.yaml'` |
| A-24 | The agent library derives `OfferLine.price` from the `shop` listing by item, and `about` from the payload's entity ids | `clients/agent/src/mineworld_agent/view.py:217-234`; `disclosed.py:213-224` |
| A-25 | The agent fixtures are frozen recordings, inputs to tests, with hashes in `SOURCE.md` | `clients/agent/tests/fixtures/SOURCE.md` |
| A-26 | `clients/protocol/evidence/affordances-market-town.log` is written by `run.sh` and grepped, never compared with a golden | `clients/protocol/run.sh:135-140` |
| A-27 | No `server/tests/frames/*.json` holds a shop, wallet or buy | `grep -l "shop\|wallet\|buy" server/tests/frames/*` (no match) |
| A-28 | No property-test crate is a dependency; the tree's idiom is an in-test SplitMix64 | `Cargo.toml`; `sdk/rust/tests/interactions.rs:91-92`; `kernel/tests/long_run.rs:54` |
| A-29 | Long baselines are rewritten from a long record (`ci_parity.py baseline write`), produced by the nightly's `parity-long` group, dispatchable on a branch | `scripts/baselines.txt:1-5`; `.github/workflows/nightly.yml:75-114`; `pr-13c-nightly.md:346-347` |
| A-30 | The café company's file says "It sells food and drink only: goods are given, never sold (step-10 QS-48)" | `worlds/market-town/organizations/cafe-company.yaml:11-13` |
| A-31 | Open PRs touching this ground: #171 (12d), #189 (WE-a), #200 (S25a) | `gh pr list` 2026-10-15 |

### 3.2 Findings

- **F-S31a-1 — Market Town's `configure/economy.yaml` fails AC-1 check 3** (A-20). Economy is one of the six
  market packs, not an allow-listed generic pack, so `configure: [economy]` and the file are refused by
  `compare_configure` and the `configure/` loop. Step-31 does not mention it, and WE-a C5 meets the same check.
  It is an operator-ruled measurement (`ARC-35`) → **Q-S31a-1**.
- **F-S31a-2 — The step makes `currency-declared` and `reference-declared` `Public`; `ARC-61` item 8 says
  configuration facts are `SystemInternal`** (A-15). A `Public` genesis fact enters every seat's perceived
  stream, so every agent inbox and every S28 memory gets one currency fact and twenty reference facts that no
  person ever witnessed. The disclosed `currency` record already reaches every observer → **Q-S31a-2**.
- **F-S31a-3 — An offer's refusal precedes its requirement** (A-11). To keep "the offer and the dispatch
  answer the same, in the same order", `validate` checks economy's named codes **before**
  `SpatialRequirement::evaluate`, the reverse of step §5.7's steps 3 and 4–5. Only the name reported when two
  reasons hold at once changes; AC-S31-3 tests each reason alone → **SD-S31a-2** (bounded).
- **F-S31a-4 — `SamePlaceAsActor` needs a target** (A-12). A keeper's `for: shop` action is target-less, so
  step §5.4's "`SamePlaceAsActor` at the shop's place" would always be `PreconditionFailed`. The requirement is
  `at_place(<shop place>)` → **SD-S31a-3** (bounded).
- **F-S31a-5 — Generic controllers take any available complete affordance at random** (A-19). Step §5.6 makes
  `withdraw` complete. Once S31-f gives Market Town keepers and person sellers, every NPC seller would withdraw
  its asks at random on about one consult in five, and the café's shelf would empty within a day. It is not
  reachable in S31-a's Market Town (no keepers, no person asks), but the offer's shape is S31-a's →
  **Q-S31a-3**. S31-b's complete `subscribe`/`unsubscribe` and S31-h's complete `accept`/`decline` meet the
  same mechanism (§10).
- **F-S31a-6 — The market mirror pins shapes and versions** (A-21). `shop-opened` v2 and the new facts must be
  mirrored in the commit that changes them, and E-9's cheapest price must be read from genesis `listed`.
- **F-S31a-7 — "Without economy" leaves the configuration behind** (A-22). The copy's `world.yaml` loses
  `configure`'s `  - economy` line too (same indentation), and the unlisted `configure/economy.yaml` is then
  refused by `ARC-61` item 5. The test must also delete the file. Its expectation ("runs") is unchanged.
- **F-S31a-8 — No loader rule makes a configuration mandatory** (A-14). `economy-currency-undeclared` is
  therefore raised by economy's own reductions of its section facts (A-17: `validate` reduces genesis) →
  **SD-S31a-1** (bounded). A world that enables economy and authors no `economy:` section anywhere has no
  amount to show and loads; that is recorded as the one narrowing of AC-S31-7.
- **F-S31a-9 — No server frame carries market shapes** (A-27). Step §13's "`server/tests/frames/*.json`
  (market-town frames) moves in S31-a" row has nothing to move: N/A, recorded.
- **F-S31a-10 — The agent fixtures are frozen inputs** (A-25). They do not fail when the server's shapes move;
  they go stale. C6 re-records them so S31-g starts from the current server, and updates `SOURCE.md`'s hashes
  and offer counts.
- **F-S31a-11 — "Price book" is taken twice.** WE-a names its configuration Process `price-book`
  (`ProcessTypeId`, PR #189); step-31 §3 names price-survey's component "price book". Different packs and types,
  one English term. Recorded for S31-c (§10 R-S31c-a-1).
- **F-S31a-12 — 12d is open** (A-31) and edits `systems/economy/tests/support/mod.rs`, Market Town's items and
  its baselines. The step requires it merged first; C0 checks.
- **F-S31a-13 — S25a is open** (A-31) and derives JSON schemas for sections and configurations, with a drift
  check over `docs/reference/`. If it merges first, every type this PR changes follows its convention in the
  same commit.
- **F-S31a-14 — A content comment goes stale** (A-30). C2 rewrites the comment only (no YAML value changes).
- **F-S31a-15 — AC-S31-3's "`TooFarAway` … outside a shop" cannot happen.** A shop `buy` names no shop: the shop
  is "the place I stand in" (step §5.2), so a buyer outside any shop is `NoSuchSeller`
  (`NoSupportedInteraction`, step §5.5, today's `tests/buy.rs:157-163`). `TooFarAway` arises only for a person
  seller beyond `TRADE_RANGE` or in another place → **Q-S31a-4**.
- **F-S31a-16 — The existing buy tests assert QS-44** (`tests/buy.rs:47-53, 141-182`). C5 replaces those
  expectations with the named codes; it is the supersession step §5.5 states.
- **F-S31a-17 — Behaviour should not move in S31-a.** Market Town has no keepers and no person asks after
  S31-a, so the set of available complete affordances is the same as on the base (`buy` per priced kind, `give`,
  `eat`, `drink`, …), and every controller's draws see the same choices. The new offers are incomplete
  (`list-for-sale`), which no NPC controller attempts (A-19). The re-baseline is then the vocabulary only, and
  that is measured (AC-a-9).
- **F-S31a-18 — No economy renderer exists in cognition** (`git grep money-transferred cognition/` → nothing).

---

## 4. Reuse, both ways (`REUSE_POLICY.md`; `DEP-63`)

| Need | Candidates | Verdict |
| --- | --- | --- |
| Currency codes, minor units, formatting | `iso_currency`, `rusty-money`, `currency_rs` | **Declined** (`DEP-63`, step §19.4). Amounts stay bare `u64` minor units (A-1); a world may declare a fantasy coin no ISO table holds; formatting is presentation (S31-e). Only ISO 4217's minor-unit column is reused, as data and as the citation for `exponent ∈ 0 … 4` |
| Order book, matching engine | `orderbook-rs`, exchange crates | **Declined** (`DEP-63`). MineWorld has posted prices settled by one owner, not continuous double auctions |
| Escrow / two-phase commit | none adopted | Step §5.7 option (b) declined there; nothing to adopt |
| Property testing | `proptest`, `quickcheck` | **Not adopted**: the tree's in-test SplitMix64 idiom (A-28) covers 10 000 seeded states with no new dependency, so no decision record is needed |

---

## 5. Design

### 5.1 Module layout (`systems/economy/src/`)

```text
lib.rs            exports; the pack table in the module docs
configuration.rs  EconomyConfiguration, Currency, CurrencyName, CurrencySymbol, Exponent, SymbolAt   (new or WE-a's)
currency.rs       CurrencyProcess (never woken), CurrencyState, the `currency` record                 (new)
component.rs      Wallet; Asks, Ask; Shop { operator, keepers }; ReferenceValue; views: Listing, AsksView, Wealth
section.rs        AuthoredEconomy, AuthoredShop; per-carrier seeding and refusals
action.rs         Buy, ListForSale, SetPrice, Withdraw, ShopSide; TRADE_RANGE; requirements
money.rs          is_holder, balance, admit_payment, pay, PRICE_MAX
sale.rs           Seller, judge_sale, settle                                                          (new)
listing.rs        judge_listing (the three seller actions), their facts                                (new)
offer.rs          offers: buys (shop and person), seller offers
event.rs          the facts of §5.9
system.rs         declaration, validate, resolve, react, PerceptionProvider
codec.rs          payload readers per action
```

Each file stays under ~400 lines; `system.rs` keeps dispatch only and delegates to `sale.rs` and `listing.rs`.

### 5.2 Configuration (`ARC-111`; step §4)

```yaml
# configure/economy.yaml
currency:
  name: US dollar     # 1 … 64 bytes of UTF-8, no control characters; display only
  symbol: "$"         # 1 … 8 bytes, no control characters, no whitespace at either end; display only
  exponent: 2         # 0 … 4 (ISO 4217's minor-unit column: JPY 0, USD 2, KWD 3, CLF 4)
  symbol_at: before   # before | after
price_modifiers: []   # WE-a's key, optional, unchanged
```

- **Decoding is validating** (`ARC-61` item 1; `DEP-10`): `currency` is required in the type, so a file without
  it fails at its line with "missing field `currency`"; each bound is a `try_from` with a message naming the
  key. `deny_unknown_fields`.
- **`FACTS`** = `[currency-declared]` (+ WE-a's `economy-configured`). `seed` states one `currency-declared {
  name, symbol, exponent, symbol_at }`, `SystemInternal`, no subjects (Q-S31a-2, ruled), so
  `ARC-61` item 7's drift check covers renaming a saved world's currency.
- **Where it lives.** Economy's reduction of `currency-declared` starts one `CurrencyProcess` (`ProcessTypeId`
  `economy-currency`), never woken, holding `CurrencyState` — WE-a's `price-book` precedent (A-16): no entity
  exists to carry world configuration at seeding. A second `currency-declared` is refused `FactRefusedByOwner`.
- **Required (SD-S31a-1).** Economy's reductions of `funded`, `shop-opened`, `listed` and `reference-declared`
  refuse with `Rejection::System { code: economy-currency-undeclared }` when no `CurrencyProcess` exists. Since
  configuration is reduced before sections (A-15), a world with any `economy:` section and no
  `configure/economy.yaml` fails `mineworld validate` naming the code (A-17).
- **If WE-a C4 merged first:** `EconomyConfiguration` gains `currency` beside `price_modifiers`; WE-a's
  `economy-configured` and `price-book` stay as they are; `currency` is the only required key.

### 5.3 The `currency` record

`currency { name, symbol, exponent, symbol_at, references: [{ item, value }] }`, a declared component type
never carried by any entity (calendar's precedent, A-16), disclosed **about the observer itself** to every
observer that is a living Person (SD-S31a-5: an observer in no place still learns the scale; the record is the
same for every observer). `references` lists every Item entity carrying a `ReferenceValue`, in item order
(`ARC-113`: a typical worth is common knowledge). With no currency process, nothing is disclosed.

### 5.4 Section (`ARC-112`, `ARC-113`; step §5.1, §6.1)

```yaml
economy: { wallet: 300000, shop: { at: store, prices: { apple: 100 }, keepers: [felix] } }   # organization
economy: { wallet: 200000, asks: { flowers: 450 } }                                         # person
economy: { reference: 300 }                                                                 # item
```

`AuthoredEconomy { wallet: Option<u64>, shop: Option<AuthoredShop>, asks: BTreeMap<EntityKey, NonZeroU64>,
reference: Option<NonZeroU64> }`, `AuthoredShop { at, prices, keepers: Vec<EntityKey> (default empty) }`.
`CARRIED_BY = [Person, Organization, Item]`. `references()` adds each `asks` key and each price key as an Item,
`at` as a Place, each keeper as a Person.

Seeding, per carrier (refusals are `Rejection::System`, which the loader reports naming the file):

| Carrier | Required | Refused, by code |
| --- | --- | --- |
| Person | `wallet` | none → `economy-wallet-unresolved` (S31-f's code, adopted now, SD-S31a-11); `shop` → `economy-shop-on-person`; `reference` → `economy-reference-on-holder` |
| Organization | `wallet` | none → `economy-wallet-unresolved`; `asks` → `economy-asks-on-organization` (it lists through `shop.prices`); `reference` → `economy-reference-on-holder`; a keeper listed twice → `economy-keeper-duplicate`; more than 8 keepers → `economy-keepers-too-many` |
| Item | `reference` | `wallet`, `shop` or `asks` → `economy-item-reference-only` |
| any | — | a price or reference above `PRICE_MAX = 10^12` → `price-out-of-range` |

Genesis facts, in order, per file: `funded`; for a shop, `shop-opened { place, operator, keepers }`
(`Public`, at the place) then one `listed { seller: operator, item, price }` per price (`Place(shop)`); for a
person's `asks`, one `listed { seller: person, item, price }` per ask, audience `Entities({person})`
(SD-S31a-13: a person's place is not yet reduced when sections are seeded); for an item, `reference-declared {
item, value }`, `SystemInternal` (Q-S31a-2, ruled).

### 5.5 Actions, payloads and requirements (step §5.2, §5.4)

| Action | Payload (JSON) | Target | Requirement |
| --- | --- | --- | --- |
| `buy` from a shop | `{ item, price }` | none | `at_place(here).requiring_target_available()` (today's shape, A-5) |
| `buy` from a person | `{ item, price }` | the seller | `same_place().within(TRADE_RANGE).requiring_target_available()`, `TRADE_RANGE = 3 000 mm` (= `GIVE_RANGE`, A-10) |
| `list-for-sale` | `{ item, price }` or `{ item, price, for: "shop" }` | none | `NONE` for oneself; `at_place(here)` for a shop (SD-S31a-3) |
| `set-price` | the same | none | the same |
| `withdraw` | `{ item }` or `{ item, for: "shop" }` | none | the same |

- `price` is `u64`; `buy.price` is compared with the current asking price, so `0` is `price-changed`
  (SD-S31a-7). `for` is `Option<ShopSide>` with `ShopSide::Shop` ↔ `"shop"`, omitted when absent
  (`skip_serializing_if`), so a self-listing encodes without it.
- The `requiring_target_available()` flag is kept as the step states; economy passes `true`, because every
  availability reason now has its named code (§5.7). The requirement is shown to clients unchanged.
- **No compatibility** (`CLAUDE.md` §4 rule 12): an old `{ item }` buy fails decoding → `PreconditionFailed`.

### 5.6 The seller actions (`listing.rs`)

`judge_listing(world, actor, action) -> Result<(Seller, Place?), Rejection>`, used by both `offers` and
`validate`. In order:

1. actor a living Person, else `NoSupportedInteraction`;
2. with `for: shop`: a shop at the actor's place, else `NoSupportedInteraction`; the actor among its keepers,
   else `PermissionDenied` (NotAKeeper); the seller is the operator; else the seller is the actor;
3. `price` in `1 ..= PRICE_MAX`, else `price-out-of-range`;
4. `list-for-sale`: no ask for the kind (`already-listed`), and the seller holds at least one (`not-held`);
   `set-price` and `withdraw`: an ask exists (`not-for-sale`); `set-price` to the same price →
   `price-unchanged` (SD-S31a-14);
5. the requirement (§5.5).

Facts (all reduced by economy into `Asks` on the seller): `listed { seller, item, price }`, `price-set { seller,
item, from, to }`, `withdrawn { seller, item }`; audience `Place(actor's place)` (for a shop, the shop's place),
`about: [seller, actor]`. `Asks` is sorted by item, one entry per kind; a reduction that would duplicate, miss,
or exceed `PRICE_MAX` is `FactRefusedByOwner`.

**Offers** (SD-S31a-6). To a living person, target `None`: one incomplete `list-for-sale` when it, or a shop it
keeps and stands in, holds an unlisted kind; one incomplete `set-price` when it or that shop has an ask; and
one incomplete `withdraw` under the same condition as `set-price` (Q-S31a-3, ruled). Each is
offered at most once per action type, because an incomplete offer carries no payload that could tell "for me"
from "for the shop".

### 5.7 Buying and settlement (`sale.rs`; `ARC-112`; step §5.7)

```text
judge_sale(world, buyer, target, item, price?) -> Result<Sale, Rejection>      one function, offers + validate
  1 buyer a living Person                                                       NoSupportedInteraction
  2 seller: target → a living Person ≠ buyer with an `Asks`;                     NoSupportedInteraction (NoSuchSeller)
            none   → the shop at the buyer's place, seller = its operator        NoSupportedInteraction (NoSuchSeller)
  3 an ask for `item`                                                            not-for-sale
  4 price given (validate only): equals the asking price, after WE-a's modifiers price-changed
  5 buyer is the seller, or a keeper of the shop it would buy from               PreconditionFailed (SelfTrade)
  6 balance(buyer) < price                                                       insufficient-funds
  7 seller holds none of `item`                                                  out-of-stock
  8 !inventory::can_take(buyer, 1)                                               cannot-carry
  9 admit_payment(buyer → seller, price) and admit_transfer(seller → buyer, 1)   must both be Ok; otherwise the
                                                                                 owner's own reason, unmapped
then (validate) the requirement of §5.5 → TooFarAway (person seller only), PreconditionFailed
```

- Steps 6–8 read the same state the owners' functions read, and step 9 asks those functions themselves, so the
  codes cannot disagree with the rules (`ARC-26`): a state step 9 refuses after 6–8 passed is a defect reported
  as the owner's reason.
- **Offers.** A person standing in a shop gets one complete `buy { item, price: asking }` per ask of the operator,
  target `None`; a person perceiving a living person seller gets one per ask of that seller, targeted. Each is
  `refused(code)` when `judge_sale` (without step 4) refuses, otherwise available. Perception then evaluates the
  requirement (A-11), as `validate` does after `judge_sale` (SD-S31a-2).
- **Resolve.** `settle` re-reads the seller and price and states, in this order: `sold { seller, buyer, item,
  count: 1, price, place }` (`Place(place)`, `about: [buyer, seller]`, participants both), `pay(buyer → seller,
  price, Place(place))`, `inventory::transfer(seller → buyer, item, 1)`. A refusal inside `settle` is
  `FactRefusedByOwner` naming the owner (economy for the payment, inventory for the transfer), as today.
- **Reduction.** Economy reduces `money-transferred` (unchanged, A-1); inventory reduces `items-transferred`
  (unchanged, A-8); `sold` writes nothing. Each re-checks its rule; neither changes what the other reads.
- **Why it is atomic** is step §5.7's argument, unchanged. The PR proves the agreement it relies on (§8: AC-a-5,
  the property test) and states the defect path (F-S31-8): X-11.

### 5.8 Wealth (`ARC-114`; step §7.1)

`wealth { amount, unvalued }`, a declared never-carried record disclosed **to the holder only** (about the
observer, when observer = subject). `amount = wallet + Σ over held kinds with a ReferenceValue of count ×
value`, `u64` with saturating arithmetic; `unvalued` = the number of held units of kinds without one. Computed
at disclosure from `Wallet`, inventory's `Holdings` (read; economy depends on inventory) and `ReferenceValue`;
never stored. No settlement path reads `ReferenceValue` (I-S31-6).

### 5.9 Events, components and version

| Item | Change | Schema |
| --- | --- | --- |
| `currency-declared { name, symbol, exponent, symbol_at }` | new, configuration fact | 1 |
| `reference-declared { item, value }` | new, item-section fact → `ReferenceValue` on the item | 1 |
| `shop-opened { place, operator, keepers }` | `prices` removed (they arrive as `listed`) | **2** |
| `listed`, `price-set`, `withdrawn` | new → `Asks` on the seller | 1 |
| `sold { seller, buyer, item, count, price, place }` | new; reduces to nothing | 1 |
| `funded`, `money-transferred`, `wage-unpaid` | unchanged | 1 |
| `Shop { operator, keepers }` | `prices` removed | component **2** |
| `Asks { asks: [Ask { item, price }] }`, `ReferenceValue { value }` | new, owned by economy | 1 |
| `currency`, `wealth` records; `asks` view (`{ asks: [{ item, price, available }] }` of a person seller, to whoever perceives them); `shop` listing view | records new; listing shape unchanged (SD-S31a-12) | — |
| `SystemVersion` | 1 → 2 (2 → 3 after WE-a); old saves refused by name (`ARC-25`) | — |

Disclosure summary: wallet and wealth to the holder only (I-S31-9); a person's asks, with `available: bool` and
never a count, to whoever perceives the person; the shop listing (operator, price, `in_stock`) to whoever
perceives the place, as today; the currency record to every living-person observer.

### 5.10 Market Town in S31-a

`configure/economy.yaml` = `{ currency: { name: US dollar, symbol: "$", exponent: 2, symbol_at: before } }`
(ISO 4217 USD; Market Town's prices are US-sized in cents, step-10 §4.5.3), and `configure:` gains `economy`
after `weather`. Nothing else changes in content except the comment of F-S31a-14.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-a-1 (I-S31-1) | No path under `kernel/`, `contracts/`, `persistence/`, `server/`, `cognition/`, `systems/inventory/`, `systems/item-transfer/` changes | C8 diff scan |
| I-a-2 (I-S31-2) | Only economy writes `Wallet`, `Asks`, `Shop`, `ReferenceValue` and the currency process; only inventory writes `Holdings` | `declaration()` review; the existing owner-write tests |
| I-a-3 (I-S31-3) | Every `sold` shares its cause (the accepted request) with exactly one matching `money-transferred` (buyer → seller, `price`) and one matching `items-transferred` (seller → buyer, `item`, 1); no `money-transferred` caused by a `buy` lacks its `sold`; no request whose outcome is a kernel fault is a `buy` | the sale scan (`market/mod.rs`) over every Market Town save of the tools/cli tests |
| I-a-4 (I-S31-4) | `money-transferred.amount = buy.price = sold.price` | AC-S31-1, AC-S31-2; the scan |
| I-a-5 (I-S31-5) | Σ wallets = Σ `funded` at every revision | existing `genesis_money` / `state_matches_replay`; AC-S31-4 |
| I-a-6 (I-S31-6) | No code path from `validate`, `resolve` or `react` of `buy` reads `ReferenceValue` | X-3; review |
| I-a-7 (I-S31-9, the S31-a part) | Wallet and wealth go to the holder only; a person's holdings count is never disclosed by economy | `wealth.rs`, `asks.rs`; X-9 |
| I-a-8 (I-S31-10, the S31-a part) | Social Café, Lakeside and bodies-yard state nothing new; their baselines do not move | AC-S31-9 |
| I-a-9 (I-S31-11) | No reference world's currency name appears in `systems/`, `clients/`, `cognition/`, `server/` outside tests and fixtures | `currency_names.rs` |
| I-a-10 | `offers` and `validate` call the same `judge_sale` / `judge_listing`; an offer's code equals the dispatch's code for the same state | `sale.rs` test "offer and dispatch agree" over the property states |
| I-a-11 | Market Town's controller choices do not move in S31-a (F-S31a-17) | AC-a-9 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt, clippy `-D warnings`, the compile-time `RejectionCode::from_static` literals | CI `fast`, `core` |
| Unit (economy) | configuration bounds and decode refusals at line and column (`currency.rs`); section per-carrier refusals (`asks.rs`); `judge_sale` order and codes; wealth arithmetic with hand-computed oracles (`wealth.rs`) | `systems/economy/tests/` |
| Integration (economy, real kernel dispatch over the hand-built store) | AC-S31-1, 2, 3; X-1, X-2, X-3, X-9; offers ≡ dispatch; disclosure audiences | `sale.rs`, `asks.rs`, `wealth.rs` |
| Property (economy) | over 10 000 SplitMix64 states, `validate` admitting a sale ⇒ dispatch `Accepted`, three facts, no fault; and the offer's verdict ≡ `validate`'s | `settlement.rs` |
| Defect path | X-11 (planted, reverted) | `settlement.rs` |
| Real binary | Market Town 30/300 days and the composition cases through the updated mirror and the sale scan; AC-S31-4 kills; AC-S31-6 rename; AC-S31-7 validate | `tools/cli/tests/market_town.rs`, `market_composition.rs`, `market_settlement.rs` |
| Measurement (evidence, `#[ignore]`) | AC-a-9 vocabulary parity against the base save | `market_settlement.rs::vocabulary_only` |
| Scans | I-a-9 currency names; AC-1 check 3 as amended (Q-S31a-1) | `tests/acceptance/tests/` |
| Long | AC-S31-9 and the re-baseline, from one `parity-long` record | nightly dispatch on the branch (C7) |
| Gate 1 | NOT REQUIRED: no LLM-facing surface changes in S31-a | — |

Oracles are literal prices, wallets and holdings written in the test, or the base build's save; never a value
economy reports about itself (test rules §25).

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-a-1 | **Two sellers, two prices.** Fixture: the store asks 300 for bread; Grace (a person, `asks: { bread: 260 }`, holding 2) stands 2 m from Alice. Alice buys one from each: two `sold` (300, 260), Alice's wallet falls by exactly 560, store +300, Grace +260, Alice holds 2 bread, each sale's three facts share one cause | AC-S31-1 |
| AC-a-2 | **Stale price.** Bob observes the store's bread at 300; keeper Felix `set-price` 320; Bob submits the observed complete offer unchanged → `System { code: price-changed }`; state bytes before = after | AC-S31-2 |
| AC-a-3 | **Each named refusal, and nothing moves:** `not-for-sale`, `insufficient-funds`, `out-of-stock`, `cannot-carry`, `TooFarAway` (3.1 m from Grace; Grace in another place), `PermissionDenied` (non-keeper `for: shop`), `already-listed`, `price-out-of-range` (0 and `PRICE_MAX + 1`), `not-held`, `price-unchanged`, SelfTrade `PreconditionFailed`, NoSuchSeller `NoSupportedInteraction` (in no shop, no target) — each by dispatch and, where an offer exists, by the offer's refusal, each with state bytes unchanged | AC-S31-3 (wording amended, Q-S31a-4) |
| AC-a-4 | **No half-trade across a kill.** `run worlds/market-town --seed 7 --days 10 --save` SIGKILLed at 20 wall offsets (SplitMix64 seed 31, uniform in [0.05, 0.95] × the control's wall time), each resumed by a new process: every save satisfies I-a-3 and I-a-5, and every survivor's facts, journal and sealed blocks equal the control's byte for byte; at least 5 kills land with a non-empty re-executed tail | AC-S31-4 |
| AC-a-5 | **Property.** Over 10 000 seeded states (wallets 0 … 2 000, seller holdings 0 … 3, buyer carrying 0 … 6, asks 1 … 1 500, buyer at 0 … 6 m, seller a person or a shop), whenever `validate` admits a `buy`, dispatch returns `Accepted` with exactly the three facts and no `KernelError`; and the offer's verdict equals `validate`'s in every state | AC-S31-4 (property) |
| AC-a-6 | **Currency renames with zero code.** A scratch copy of Market Town whose `configure/economy.yaml` is `{ name: crown, symbol: cr, exponent: 0, symbol_at: after }`, 30 days seed 7: its fact log equals the original's except the `currency-declared` payload; the `currency` record differs only in those four fields; no file outside the copy differs | AC-S31-6 (the economy half; client and agent halves are S31-e and S31-g) |
| AC-a-7 | **Undeclared currency.** The same copy without `configure/economy.yaml` (and its `configure:` entry) fails `mineworld validate` naming `economy-currency-undeclared`; each decode bound (exponent 5, a 65-byte name, a 9-byte symbol, `symbol_at: middle`, an unknown key) fails at its line naming the key | AC-S31-7 |
| AC-a-8 | **Worlds without economy.** Social Café, Lakeside and bodies-yard keys of the branch's long record equal `scripts/baselines.txt` on `main`; with economy installed in the build, none states a fact of economy's vocabulary | AC-S31-9 |
| AC-a-9 | **Vocabulary only (F-S31a-17).** Market Town, seed 7, 30 days, base save (C0) vs head: the journal's sequence of `(actor, action type, at, outcome kind)` is equal; every fact outside economy's vocabulary is equal in order and payload (event ids aside); every accepted `buy` on the head has its `sold`. A difference is a finding explained before C7, and a material stop if it shows behaviour moved | Q-S31-3 (what "vocabulary re-baseline" means) |
| AC-a-10 | **Disclosure.** In the fixture: Alice's observation of herself holds `wallet`, `wealth` and `currency`; her observation of Bob and of Grace holds neither `wallet` nor `wealth` nor any holdings count; Grace's `asks` view is visible to Alice beside her and shows `available` only; the shop listing is unchanged in shape | I-S31-9; X-9 |
| AC-a-11 | **Wealth.** With references bread 300, pen 150, Grace holding 2 bread, 1 pen, 1 undeclared-value kind and wallet 1 000: `wealth = { amount: 1 750, unvalued: 1 }`; after she sells one bread for 260, `{ amount: 1 710, unvalued: 1 }` | step §7.1 |
| AC-a-12 | **Re-baseline.** `scripts/baselines.txt` `[world market-town]` rewritten from the branch's long record with one `reason S31-a …` line; the other three worlds unchanged | Q-S31-3 |
| AC-a-13 | **No kernel, no contract.** `git diff --name-only origin/main...HEAD` lists nothing under the paths of I-a-1 | I-S31-1 |

### 8.1 Planted mutations (each must turn its test red by name, then be reverted from a byte copy)

| Id | Plant | Must fail |
| --- | --- | --- |
| X-1 | `settle` omits the `items-transferred` | AC-a-1 (holdings) and the sale scan on `market_town.rs` |
| X-2 | `judge_sale` skips step 4 | AC-a-2 |
| X-3 | `settle` charges `ReferenceValue` instead of the ask | AC-a-1 with references bread 280 ≠ 300 |
| X-9 | `discloses` returns `Wallet` for any observer | AC-a-10 |
| X-11 | inventory's reduction refuses a transfer `admit_transfer` admitted (test-only edit in `systems/inventory`, never committed) | `settlement.rs`: dispatch returns `FactRefusedByOwner { system: inventory }`; persisted, the journal's outcome at that revision is the fault and the scan names the revision |
| M-a-1 | `offers` drops the `refused(code)` and keeps the buy available | AC-a-5 (offer ≡ validate) |
| M-a-2 | `validate` evaluates the requirement before `judge_sale` | AC-a-5 (order) |
| M-a-3 | genesis `listed` for a shop with audience `Public` | AC-a-10 (a person in no shop perceives it) |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason. Targeted
validation per commit; the PR's CI is the one full run.

### 9.0 The procedure for every commit

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s31a`. Never `cd` elsewhere.

1. Re-read this commit's section and the design sections it cites. Re-read every file you will edit, in full.
2. Make the numbered steps, in order. Do not do a later commit's work.
3. Run the commit's validation. Record per command: test count, wall time, PASS / FAIL / INCONCLUSIVE.
4. For each listed mutation: plant, run the named test, record the red (test name), revert from a byte copy
   (`cmp` clean), run green.
5. Static: `cargo fmt --check`; `cargo clippy -p <touched crate> --all-targets -- -D warnings`. Tests:
   `cargo test -p mineworld-economy --test <file>` one file at a time; a `tools/cli` test file only where the
   commit names it as local (`-p mineworld-cli --test <file>`). **R-LOAD-1:** never `cargo test --workspace`,
   never all of `tools/cli`. Markdown: `python3 scripts/ci_layer.py docs`.
6. Review: answer the commit's review questions in §15 in one or two sentences each.
7. `git status --short` and `git diff --stat`: every changed path is in the commit's **Files**, or revert it or
   stop and record why.
8. Update §15 (ledger row, evidence, findings) and the handoff in the same commit.
9. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` with the session's attribution line.
   No heredoc writes, no `sed -i`, no `python3 -c`.
10. Stop and report a material stop (§12) instead of improvising.

#### C0 — Base, prerequisites, ledger (Markdown only)

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s31a.md` (new).
- **Steps:**
  1. Record the base commit. Confirm Milestone F's checklist was handed over (overall.md or the primary's
     dispatch), and 12d (#171) is merged. Either missing → material stop.
  2. Record whether WE-a C4 (#189), S25a (#200) and S29-b are merged; choose the branch of §2.3 for each.
  3. Re-read A-1 … A-31 on the base; record moved lines (bounded) or changed statements (stop if one changes a
     decision here).
  4. Build the base binary (`cargo build -p mineworld-cli`) and write the base Market Town save for AC-a-9
     (`target/debug/mineworld run worlds/market-town --headless --seed 7 --days 30 --save
     target/s31a-base/market-town-30`), recording its sha256 and the commit. Not committed.
  5. Confirm the primary's freeze header exists and §14.1's rulings are unchanged.
- [ ] Implementation. [ ] Validation: `python3 scripts/ci_layer.py docs`. [ ] Review: §2.3 holds on the base.
- **Commit message:** `docs(plan): S31-a — open the ledger at the base`

#### C1 — Specifications first

- **Goal:** the decisions exist before the code they govern (`CLAUDE.md` §2.2).
- **Files:** `docs/DECISIONS.md`, `docs/MODULE_SPEC.md`, `docs/CORE_CONCEPTS.md`, `docs/ARCHITECTURE.md`.
- **Steps:**
  1. `ARC-111` (currency: configuration, one per world, required, the record, the visibility as ruled by
     Q-S31a-2), `ARC-112` (asks on Persons and Organizations, keepers, the four actions, `buy` with the observed
     price, the named codes, one settling owner; amends `ARC-38` items 3 and 5 and supersedes step-10 QS-44),
     `ARC-113` (reference value, never charged), `ARC-114` (wealth view; S31-f adds the band), `DEP-63` (step
     §19.4's text and §4 here), in the file's record format at its end.
  2. Dated notes: under `ARC-38` (item 5 lifted; item 3's `buy { item }` and `TargetUnavailable` superseded by
     `ARC-112`); under `ARC-35` (Q-S31a-1, ruled (a)): `configure/` admits the configuration of the six
     market packs.
  3. `MODULE_SPEC.md` §4.1: the economy row (§5.4's keys and carriers, refusals) and a `configure/economy.yaml`
     entry (§5.2); `CORE_CONCEPTS.md` §7 (an item kind may carry an authored reference value, never a price),
     §8 (an Organization sells through its shop; its keepers change its prices); `ARCHITECTURE.md`, systems
     section: one paragraph, a settling owner states another owner's fact in one dispatch (step §19.3).
- **Validation:** `python3 scripts/ci_layer.py docs` (headings, decision ids).
- [ ] Implementation. [ ] Validation. [ ] Review: no new ontology term (`CLAUDE.md` §2.1 rule 3); every code of
  §5 appears in `ARC-112`; nothing in the docs contradicts a SD of §11.
- **Commit message:** `docs: currency, seller asks and settlement — ARC-111 … ARC-114, DEP-63 (S31-a)`

#### C2 — The currency as configuration, its record, and Market Town's file

- **Goal:** the scale is a world datum; every economy world declares it. Market Town's file lands here so every
  later commit loads it (SD-S31a-4, a bounded reorder of the step's C6).
- **Files:** `systems/economy/{Cargo.toml (unchanged unless S25a), src/{configuration.rs, currency.rs,
  event.rs, system.rs, lib.rs}, tests/{currency.rs, support/mod.rs}}`; `worlds/market-town/{world.yaml,
  configure/economy.yaml, organizations/cafe-company.yaml (comment only), README.md}`;
  `tools/cli/tests/market_composition.rs`; `tests/acceptance/tests/ac1_composability.rs` (Q-S31a-1);
  S25a's generated reference files (only if S25a merged).
- **Steps:**
  1. `Currency` and its bounded field types; `EconomyConfiguration` (new, or WE-a's extended); `impl
     PackConfiguration` and `configures!()` (or extend WE-a's `FACTS` and `seed`).
  2. `CurrencyDeclared`; `CurrencyProcess` started by its reduction; refusals of a second one.
  3. `economy-currency-undeclared` in the reductions of `funded` and `shop-opened` (the others join in C3).
  4. The `currency` record in `discloses` (§5.3, `references` empty until C3).
  5. `SystemVersion` raised (§5.9).
  6. Test support seeds the configuration before the sections, as the loader does.
  7. Market Town: the file (§5.10), the `configure:` entry, the comment of F-S31a-14 (it now says shops may sell
     any kind, `ARC-112`; the café sells food and drink by choice).
  8. `market_composition.rs`: `without("economy")` also deletes `configure/economy.yaml` (F-S31a-7).
  9. `ac1_composability.rs` (Q-S31a-1, ruled (a)) admits a market pack's configuration in `compare_configure` and the
     `configure/` loop; its unit test gains the two mutations (a market pack's file in Social Café; a market pack
     configured but not enabled), each red by name.
- **Validation:** `--test currency` (decode bounds at line and column; record disclosed to two observers, one
  in no place; second `currency-declared` refused; undeclared refused by name on `validate` of a scratch world
  — AC-a-7); `--test buy`, `--test wages`, `--test removable` (unchanged expectations, support seeded);
  `cargo run -p mineworld-cli -- validate worlds/market-town` (prints, exits 0); `-p mineworld-acceptance
  --test ac1_composability` (if (a)). `market_composition.rs` runs in CI only (R-LOAD-1: six 30-day runs).
- [ ] Implementation: steps 1–9. [ ] Validation. [ ] Review: no currency name in Rust (`git grep -n "dollar"
  systems/`); the configuration fact's visibility is the ruled one; the process is never woken.
- **Commit message:** `feat(economy): the currency is world configuration, disclosed to every observer (S31-a)`

#### C3 — Asks, shops with keepers, the section's new keys, reference values

- **Goal:** one truth for prices — the seller's `Asks` — before anything changes them.
- **Files:** `systems/economy/src/{component.rs, section.rs, event.rs, system.rs, offer.rs, action.rs,
  currency.rs, lib.rs}`, `systems/economy/tests/{asks.rs (new), buy.rs, support/mod.rs}`,
  `tools/cli/tests/market/mod.rs`.
- **Steps:**
  1. `Asks`, `Ask`, `ReferenceValue`; `Shop { operator, keepers }` (schema 2); `ShopOpened` v2; `Listed`;
     `ReferenceDeclared`.
  2. Section (§5.4): optional keys, `CARRIED_BY` with `Item`, references, per-carrier refusals, genesis order.
  3. Reductions: `listed` → `Asks` (refuse a duplicate kind); `reference-declared` → `ReferenceValue` (a living
     Item); both refuse `economy-currency-undeclared`.
  4. The shop listing and today's `buy { item }` read the operator's `Asks` (behaviour identical: same prices).
     If WE-a merged: `effective_price` reads the operator's ask.
  5. The currency record's `references`.
  6. `market/mod.rs`: `shop-opened` mirrored at v2 (`place, operator, keepers`); `listed` mirrored (v1); the
     version check per slug; E-9's cheapest price = minimum genesis `listed` price over shops (A-21).
- **Validation:** `--test asks` (each per-carrier refusal by code; genesis order and audiences, M-a-3; a person
  ask survives holding none, `in_stock` view `available: false`); `--test buy` (unchanged behaviour, now from
  `Asks`). `market_town.rs` and `market_composition.rs` in CI.
- [ ] Implementation: steps 1–6. [ ] Validation, M-a-3. [ ] Review: `Shop` holds no price; nothing but economy's
  reductions writes `Asks`; the listing's JSON shape is byte-identical for the same prices (compare a serialized
  listing before/after in `asks.rs`).
- **Commit message:** `feat(economy): asks on the seller, shops with keepers, reference values (S31-a)`

#### C4 — The seller actions, their offers, the asks view and the wealth record

- **Files:** `systems/economy/src/{action.rs, listing.rs (new), event.rs, codec.rs, offer.rs, system.rs,
  component.rs, lib.rs}`, `systems/economy/tests/{asks.rs, wealth.rs (new), support/mod.rs}`.
- **Steps:**
  1. `ListForSale`, `SetPrice`, `Withdraw`, `ShopSide`; `PriceSet`, `Withdrawn`; their codecs.
  2. `judge_listing` (§5.6) shared by `offers` and `validate`; `resolve` states the fact; reductions.
  3. Offers (§5.6, Q-S31a-3 as ruled).
  4. The `asks` view of a person seller; the `wealth` record (§5.8).
- **Validation:** `--test asks` (each seller action accepted and refused by code: `already-listed`,
  `not-held`, `not-for-sale`, `price-unchanged`, `price-out-of-range`, `PermissionDenied`; `for: shop` from a
  keeper outside the shop → no offer, dispatch `NoSupportedInteraction`; audiences); `--test wealth` (AC-a-11
  numbers; holder only; X-9 here as a planted `discloses` change).
- [ ] Implementation: steps 1–4. [ ] Validation, X-9. [ ] Review: an incomplete offer is offered once per type;
  nothing reads a wallet for anyone but its holder; `wealth` never stored (no `owning::<Wealth>`).
- **Commit message:** `feat(economy): sellers list, re-price and withdraw; wealth is a holder view (S31-a)`

#### C5 — Buying from any seller at the observed price, and one settlement

- **Files:** `systems/economy/src/{action.rs, sale.rs (new), event.rs, codec.rs, offer.rs, system.rs, money.rs,
  lib.rs}`, `systems/economy/tests/{buy.rs, sale.rs (new), settlement.rs (new), paced.rs, support/mod.rs}`,
  `tools/cli/tests/market/mod.rs`.
- **Steps:**
  1. `Buy { item, price }`, optional target; `Sold`; `TRADE_RANGE`; `PRICE_MAX` in `money.rs`.
  2. `judge_sale` (§5.7) shared by `offers` and `validate`; `settle` in `resolve`.
  3. Offers: shop buys and person-seller buys with `refused(code)`.
  4. `buy.rs`: QS-44's `TargetUnavailable` expectations replaced by codes (F-S31a-16); the "in no shop" arm
     stays `NoSupportedInteraction`.
  5. `sale.rs`: AC-a-1, AC-a-2, AC-a-3 (buy codes), AC-a-10.
  6. `settlement.rs`: AC-a-5 (in-test SplitMix64, documented seed); X-11 harness (the plant is an uncommitted
     edit in `systems/inventory/src/system.rs`, reverted).
  7. `market/mod.rs`: `sold` mirrored; `sales_are_atomic(facts, journal)` (I-a-3), called from the existing
     market-life checks of `market_town.rs` and `market_composition.rs`.
- **Validation:** `--test sale`, `--test settlement` (record wall time; budget ≤ 60 s on Linux CI), `--test
  buy`, `--test paced` (the unchanged paced controller still buys: E-5). Mutations X-1, X-2, X-3, X-11, M-a-1,
  M-a-2.
- [ ] Implementation: steps 1–3, 7. [ ] Validation: steps 4–6, mutations. [ ] Review: `ReferenceValue` is not
  named in `sale.rs`; the three facts are built only through `pay` and `inventory::transfer`; the order of
  `judge_sale` equals §5.7's table.
- **Commit message:** `feat(economy): buy from any seller at the observed price, settled atomically (S31-a)`

#### C6 — Market Town integration: kills, rename, parity, scans, fixtures

- **Files:** `tools/cli/tests/market_settlement.rs` (new), `tests/acceptance/tests/currency_names.rs` (new),
  `clients/agent/tests/fixtures/{market-town-start.json, market-town-walking.json, market-town-street.json,
  SOURCE.md}`, `clients/protocol/evidence/affordances-market-town.log` (only if re-run, step 5).
- **Steps:**
  1. `market_settlement.rs`: AC-a-4 (kill harness in `run_restart.rs`'s pattern, wall offsets from SplitMix64
     seed 31; the sale scan on every save); AC-a-6 (scratch copy, crown, fact comparison); AC-a-7's
     `validate` arm on the binary.
  2. `vocabulary_only` (`#[ignore]`, reads `S31A_BASE_SAVE`): AC-a-9; run once against C0's base save, record.
  3. `currency_names.rs`: I-a-9 over tracked files (names `US dollar`, `crown` of the reference worlds and test
     fixtures; the `$` symbol is not scannable and is excluded, recorded).
  4. Re-record the agent fixtures with `record.py` (A-25 command); update hashes and offer counts in
     `SOURCE.md`; run `clients/agent` tests that read them, one file at a time
     (`uv run --locked pytest clients/agent/tests/test_info.py`, `test_view.py`, `test_relocate.py`).
  5. `clients/protocol/run.sh` if Godot 4.7 is on the machine; otherwise record "not re-run: Godot absent; the log
     is evidence, not an oracle (A-26)".
- **Validation:** `market_settlement.rs` in CI (budget: AC-a-4 ≤ 300 s per Linux CI run; over → material stop
  with the measured time); locally only `vocabulary_only` and AC-a-6/7 (two 30-day runs; R-LOAD-1 permits a
  single targeted file). Python tests above.
- [ ] Implementation: steps 1–5. [ ] Validation. [ ] Review: AC-a-9's comparison excludes only event ids and
  economy's vocabulary; the kill harness proves each kill happened (status, no summary, short head).
- **Commit message:** `test: Market Town settles atomically across kills and renames its coin with no code (S31-a)`

#### C7 — The vocabulary re-baseline (rebased last)

- **Files:** `scripts/baselines.txt`.
- **Steps:** push the branch; dispatch the nightly on it with `groups=parity-long` (the only producer of a long
  record, A-29; test rules §10 case 2, recorded); download `nightly-parity-long-linux`; `python3
  scripts/ci_parity.py baseline write <record> scripts/baselines.txt`; add `reason S31-a (ARC-112, Q-S31-3):
  economy's vocabulary changed — currency-declared, listed, shop-opened v2, sold, buy { item, price }; AC-a-9:
  controller choices and every non-economy fact unchanged`; confirm the other three worlds did not move
  (AC-a-8). Rebase this commit last before review (step §13).
- [ ] Implementation. [ ] Validation: `baseline check` against the record. [ ] Review: only `[world
  market-town]` moved.
- **Commit message:** `chore(baselines): Market Town re-baselined for economy's new vocabulary (S31-a)`

#### C8 — Close-out

- **Files:** this file (§15), `handoff-s31a.md`, `systems/economy/README.md`.
- **Steps:** the README table (§5); AC-a-13's diff scan; fill §8's evidence; open the PR `mvp0/pr-s31a`; CI on the
  exact head (Linux, macOS, Windows); repair to green.
- [ ] Implementation. [ ] Validation: CI green on the exact head. [ ] Review: deviations recorded.
- **Commit message:** `docs(plan): S31-a close-out`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 10. Requirements this PR places on other PRs

- **R-WEa-a-1 (on WE-a C4/C5, if it lands second).** Extend `EconomyConfiguration` beside `currency`; read
  `effective_price` over the operator's `Asks`; Market Town's file keeps `currency`. WE-a C5 meets AC-1 check 3
  as amended by Q-S31a-1 (a), so its `configure/economy.yaml` is admitted too.
- **R-S31b-a-1 (on S31-b).** Complete `subscribe`/`unsubscribe` offers are taken at random by generic controllers
  (A-19); as ruled (Q-S31a-3), both are offered incomplete (step §8.2, dated note).
- **R-S31c-a-1 (on S31-c).** Name price-survey's component and process so they do not collide with WE-a's
  `price-book` (F-S31a-11).
- **R-S31e-a-1 (on S31-e).** Read the `currency` record (§5.3) and the `asks` view; incomplete
  `list-for-sale`/`set-price`/`withdraw` already appear in the 2D menu as disabled raw entries until then.
- **R-S31f-a-1 (on S31-f).** Adds the `wealth-band` note to `ARC-114`; extends `economy-wallet-unresolved` with the
  band path; mirrors in `market/mod.rs` already read `listed` and `sold`.
- **R-S31g-a-1 (on S31-g).** `OfferLine.price` for person-seller buys comes from the payload's `price`; the
  economy S28 renderer, if S28-d has merged.
- **R-S31h-a-1 (on S31-h).** `accept-proposal`/`decline-proposal` are offered incomplete (Q-S31a-3, ruled; step §5.9 dated note); acceptance reuses `judge_sale`
  steps 5–9 and `settle`.

## 11. Bounded decisions (recorded; not operator questions)

| Id | Decision | Why |
| --- | --- | --- |
| SD-S31a-1 | `economy-currency-undeclared` raised in economy's section-fact reductions; no loader change | F-S31a-8; A-17 |
| SD-S31a-2 | `validate` runs `judge_sale`/`judge_listing` before the requirement | F-S31a-3 (presence's offer contract) |
| SD-S31a-3 | `for: shop` requirement is `at_place(here)` | F-S31a-4 |
| SD-S31a-4 | Market Town's `configure/economy.yaml` lands in C2, not C6 | every commit must load Market Town |
| SD-S31a-5 | Currency in a never-woken Process; the record about the observer itself | A-16; reaches an observer in no place |
| SD-S31a-6 | One incomplete offer per seller action type | an incomplete offer cannot distinguish self from shop |
| SD-S31a-7 | `buy.price: u64`; `0` is `price-changed` | an ask is ≥ 1, so 0 never matches |
| SD-S31a-8 | `ARC-114` written now with the view; the band half by S31-f | §7.4 of the step is S31-f's |
| SD-S31a-9 | Economy's S28 renderer deferred to S31-g | step §16.4; F-S31a-18 |
| SD-S31a-10 | Property test with in-test SplitMix64 | A-28; no dependency |
| SD-S31a-11 | A holder without `wallet` refused `economy-wallet-unresolved` | S31-f's code; same meaning before bands exist |
| SD-S31a-12 | The `shop` listing keeps its JSON shape | the 2D readers (`readers.gd:119-122`) and the agent (A-24) keep working |
| SD-S31a-13 | A person's genesis `listed` is `Entities({person})` | the person's place is not reduced at seeding |
| SD-S31a-14 | `set-price` to the same price → `price-unchanged` | a fact must change something; a refusal must name why |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S31-a — currency as data, seller asks, atomic settlement
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s31a-currency-asks.md
RELATED / BINDING DOCS:  step-31-wealth-prices.md (§§0–7.3, 13, 14, 15, 16.2, 16.5, 18, 19); step-10-market.md
                         §4.5; pr-we-a-happenings.md §6.2; docs/CORE_CONCEPTS.md §§7, 8, 12.1, 15.2;
                         docs/MODULE_SPEC.md §4.1; docs/DECISIONS.md ARC-25, ARC-26, ARC-34, ARC-35, ARC-37,
                         ARC-38, ARC-61; .structured-coding/standards.md; CLAUDE.md;
                         docs/ENGINEERING_STANDARDS.md; docs/ENGINEERING_RULES.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s31a, its own, held by one session
BRANCH:                  mvp0/pr-s31a, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator (step-31 §16.5) AND 12d (#171)
                         merged AND this design frozen (met: operator, 2026-10-18) AND the primary session's
                         dispatch                                                             (step; primary)
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–7, as §14.1's rulings shape them        (operator and primary, 2026-10-15)
FROZEN INVARIANTS:       step-31 I-S31-1 … I-S31-6, I-S31-9 … I-S31-11 (their S31-a parts); §6 I-a-1 … I-a-11;
                         §8 as written; Q-S31-3 (two re-baselines; this PR's is C7, rebased last)
SEQUENCE:                C0 … C8 (§9)
ALLOWED COMMANDS:        cargo build / test -p <crate> --test <file> / clippy -p / fmt; target/*/mineworld *;
                         uv run --locked *; git; gh (pr create/view/checks; workflow run nightly.yml on the
                         branch with groups=parity-long, once per final head; run download); python3 scripts/*;
                         mkdir -p; cmp; sha256sum/shasum; sed -n
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or all
                         of tools/cli locally (R-LOAD-1); reading any key or .env; calling any model; editing
                         kernel/, contracts/, persistence/, server/, cognition/, systems/inventory/,
                         systems/item-transfer/ (X-11's plant is reverted, never staged); merging
MATERIAL STOPS:          Milestone F not handed over or 12d not merged; a needed change under I-a-1's paths;
                         a needed new dependency (DEP); AC-a-9 shows controller choices moved; AC-a-5 finds a
                         state validate admits and a reduction refuses (an owner rule disagreement: a design
                         question, F-S31-8); AC-a-4 over budget; AC-1 failing in a way Q-S31a-1's ruling does
                         not cover; a WE-a or S25a shape that the §2.3 rule does not cover; any change to a
                         frozen step invariant or acceptance
PLATFORMS:               Linux, macOS, Windows (CI core and python jobs on all three)
VALIDATION BUDGET:       economy test files unrestricted locally, one at a time; tools/cli market files in CI
                         only, except AC-a-6/7 and vocabulary_only locally (single files); settlement.rs
                         ≤ 60 s and market_settlement.rs ≤ 300 s on Linux CI; one nightly parity-long dispatch
                         per final executable head (C7). Gate 1: NOT REQUIRED. Gate 2: the real binary runs
                         above. No paid call
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s31a.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                       (source: the operator's freeze of this design, 2026-10-18)
                         semantic commits: authorized                    (working rules §14)
                         branch push: authorized                         (working rules §21)
                         PR creation / update: authorized                (working rules §21)
                         CI repair to review readiness: authorized       (working rules §21)
                         the nightly dispatch of C7: authorized once     (this contract; test rules §10 case 2)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized — never by the implementation session; only with the
                         operator's explicit authorization after the primary's and an Opus logic review
                         (working rules §22)
POST-MERGE SYNC OWNER:   primary: step-31, overall.md, decision ids, the progress page; implementation
                         session: this ledger, the merge identity, evidence, deviations
STOP CONDITION:          S31-a READY FOR OPERATOR REVIEW — DO NOT MERGE
```

### 12.1 R-LOAD-1 note

The host runs many agents. Locally, each commit runs only its named test files, one at a time, plus fmt and
clippy for the touched crate. The heavy real-binary files (`market_town.rs` 300 days, `market_composition.rs`
six 30-day runs, `market_settlement.rs`'s 20 kills) run only in CI; a local timing is INCONCLUSIVE by rule. The
three exceptions (AC-a-6, AC-a-7, `vocabulary_only`) are single targeted files whose claims are about bytes, not
time. CI on the exact final head is the gate; the long record comes from the one C7 dispatch.

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S31a-1 | WE-a, 12d, S25a and S31-a rebase over each other in economy and Market Town | §2.3's rules; C0 records which landed; C7 rebased last (step R-S31-4) |
| R-S31a-2 | The re-baseline hides a behaviour change | AC-a-9 compares choices and non-economy facts with the base save before C7 |
| R-S31a-3 | Agents and the 2D client see new incomplete offers before S31-e/g give them words and tools | they are disabled and never sent (A-19; `client_2d_interact_stub.rs:86-89`); R-S31e-a-1, R-S31g-a-1 |
| R-S31a-4 | The kill test is slow or flaky under CI load | kills at wall offsets with a proven-kill check; budget 300 s; a timing failure is INCONCLUSIVE, never re-tuned silently |
| R-S31a-5 | The property generator misses the disagreeing corner | its ranges straddle every boundary (zero balance, exact price, zero stock, capacity 5/6, 3.0/3.1 m); X-11 proves the harness reports a disagreement |
| R-S31a-6 | Economy grows large | §5.1's split; `system.rs` dispatch only; size triggers reviewed in C5 |
| R-S31a-7 | AC-1 is amended for a reason the operator does not accept | Q-S31a-1 (b) is stated with its cost |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S31a-1 [operator — AC-1 measurement, `ARC-35` check 3]** | AC-1 check 3 lets Market Town's `configure:` list only the generic packs calendar and weather, and refuses any other file in `configure/` (`ac1_composability.rs:1032, 1093-1125, 1321-1330`). Economy is one of the six market packs, so Market Town's required `configure/economy.yaml` fails AC-1 (F-S31a-1). WE-a C5 meets the same check. How should check 3 change? | **(a) Admit the configuration of the six market packs**, in Market Town only, when `systems` enables the pack: a market pack's configuration *is* part of the market delta, so the delta is still measured exactly. One dated note under `ARC-35`, the two loops in `ac1_composability.rs` widened to `MARKET_PACKS ∪ GENERIC_PACKS`, and two mutations (the file in Social Café; configured but not enabled). (b) Not requiring a configuration: the currency would need a default, which step §4.1 rules out, or would move into a section on some entity, which has no natural owner |
| **Q-S31a-2 [operator — step §4.1 / §6.1 vs `ARC-61` item 8]** | Step-31 makes `currency-declared` and `reference-declared` `Public`. `ARC-61` item 8 asks configuration facts to be `SystemInternal`, and a `Public` genesis fact lands in every seat's perceived stream, agent inbox and S28 memory: in Market Town 11 seats × 21 facts that nobody witnessed (F-S31a-2). Keep `Public`? | **No: state both `SystemInternal`** (no subjects), and deliver them only through the `currency` record (§5.3), which every observer already receives and which carries the references. Nothing a client or agent needs is lost; the drift check still covers the currency; S28 memory keeps only what people perceived |
| **Q-S31a-3 [operator / primary — step §5.6, offers]** | The rule and motivated controllers attempt any available complete affordance at random (`offered.rs:26-52`, `options.rs:285-289`). A complete `withdraw` per ask would make every NPC seller withdraw its asks at random once S31-f adds keepers and person sellers (F-S31a-5). Offer `withdraw` complete, as the step says? | **Offer `withdraw` incomplete**, like `set-price`: only a controller that chooses to (a pricing policy, a human, an agent's typed tool) withdraws. The same rule should guide S31-b's `subscribe`/`unsubscribe` and S31-h's `accept`/`decline` (R-S31b-a-1, R-S31h-a-1). The alternative, a per-action opt-out in the generic controllers, changes frozen controller constants (`offered.rs:17-20`, `ARC-35` point 6) |
| **Q-S31a-4 [primary — AC-S31-3's wording]** | AC-S31-3 lists "`TooFarAway` (3.1 m from a person seller; outside a shop)". A shop `buy` names no shop — the shop is the place the buyer stands in — so "outside a shop" is `NoSuchSeller` (`NoSupportedInteraction`, step §5.5; today's behaviour, `tests/buy.rs:157-163`), never `TooFarAway` (F-S31a-15). Amend the criterion? | **Yes:** `TooFarAway` for a person seller 3.1 m away or in another place; `NoSupportedInteraction` for a buyer in no shop with no target. AC-a-3 is written that way. No code or contract differs; only the step's sentence |

### 14.1 Rulings, 2026-10-15 (relayed by the coordinator; binding)

| Question | Ruling | By |
| --- | --- | --- |
| Q-S31a-1 | **(a), as recommended.** Admit the configuration of the enabled market packs, in Market Town only, with a dated `ARC-35` note and the two mutations. WE-a C5's `configure/economy.yaml` is admitted by the same change. Applied: §2.1 item 4, C1 step 2, C2 step 9, §7, R-WEa-a-1 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S31a-2 | **As recommended.** `currency-declared` and `reference-declared` are `SystemInternal` and reach observers only through the `currency` record. Step §4.1 and §6.1 amended with dated notes in this PR. Applied: §5.2, §5.4, C1 step 1 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S31a-3 | **As recommended.** `withdraw` is offered incomplete. The same rule applies to S31-b's `subscribe`/`unsubscribe` and S31-h's `accept-proposal`/`decline-proposal`. Step §5.6, §5.9 and §8.2 amended with dated notes in this PR. Applied: §5.6, C4 step 3, R-S31b-a-1, R-S31h-a-1 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S31a-4 | **Yes.** AC-S31-3's wording amended in the step (dated note): `TooFarAway` for a person seller 3.1 m away or in another place; `NoSupportedInteraction` for a buyer in no shop with no target. Applied: AC-a-3 | Ruled 2026-10-15 (primary) |

No question remains open. **Freeze, 2026-10-18 (operator, in the primary session):** the design is frozen with
every question answered as recommended; the four rulings above stand unchanged.

---

## 15. Ledger (live during implementation)

### 15.0 The execution session (opened at C0, 2026-10-18)

```text
Implementer:       the S31-a implementer (Opus), spawned by the S31+S29 lane lead. The logic review of each
                   commit below is the implementer's own SELF-REVIEW; the lane lead runs the independent one
Worktree, branch:  /Users/yuema137/mineworld-worktrees/impl-s31a, mvp0/pr-s31a; no other session holds it
Base:              origin/main @ 44f050e3 (the merge of #171, 12d), where the branch was created
Scratch:           /Users/yuema137/mineworld-worktrees/lane-s31-s29/scratch-s31a/ (the coordinator's rule: no
                   temporary file under /private/tmp). The base binary and the base save of C0 step 4 live
                   there rather than under target/s31a-base/ (bounded: same content, another directory)
Heavy jobs:        every cargo build, test and clippy, and every long binary run, through
                   /Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh (two slots, machine-wide)
```

**Prerequisites at the base (C0 steps 1, 2 and 5).**

| Item | State at 44f050e3 | Branch of §2.3 taken |
| --- | --- | --- |
| Milestone F's checklist | **not** handed to the operator | The coordinator's reading of the start condition, relayed by the lane lead on 2026-10-18: implement and commit **locally** now; do not push and do not open the PR; the lane lead pushes later. The condition gates publication, not local commits |
| 12d (#171) | merged: 44f050e3 is its merge commit | the precondition holds |
| WE-a (#189) | open, not merged | S31-a lands first. `EconomyConfiguration` is new and holds `currency` only; `SystemVersion` goes 1 → 2; WE-a C4 rebases onto `Asks` (R-WEa-a-1) |
| S25a (#200) | merged (e807c9ea) | every section and configuration type this PR changes derives its JSON Schema with a doc comment on every field, and `docs/reference/` is regenerated in the same commit (F-S31a-13) |
| S29-b | not merged: no newsstand in `systems/` or `worlds/` | no content change; `ARC-38` item 5 is lifted in C1 all the same |
| #258, the decision split | merged | `ARC-111` … `ARC-114` and `DEP-63` are one file each, `docs/decisions/<ID>.md`, with the index regenerated by `scripts/check_decision_ids.py --write-index`; the `ARC-38` and `ARC-35` amendments are dated `## <ID> note — …` headings appended to their records' files. This replaces §9 C1's "`docs/DECISIONS.md` … at its end", which the split made obsolete |
| #261 (`ARC-153`, one MCP `act` tool) | merged | nothing in S31-a's scope depends on it: no `clients/agent/src` change, and C6's re-recording records the server's observations, which #261 does not shape |
| `ARC-154` (speed first: a PR's CI runs `fast` only) | merged | recorded for C7 and C8: a PR head's CI no longer runs the tests; they run at the milestone |
| The freeze header and §14.1 | present and unchanged | — |

**Endpoint authority, as this session executes it.** Implementation, local validation and semantic commits:
authorized (the operator's freeze, 2026-10-18; working rules §14). Branch push, PR creation and update, CI repair
and C7's nightly dispatch: **not authorized in this session** — the coordinator's instruction relayed by the lane
lead on 2026-10-18 ("implement LOCALLY now and commit per the commit plan; DO NOT push and DO NOT open a PR. The
lane lead pushes later") replaces §12's lines for them until the lane lead says otherwise. Merge: never.

**The audit anchors, re-read at the base (C0 step 3).** A-1 … A-3, A-5 … A-14, A-16, A-18, A-19, A-21, A-22 and
A-24 … A-30 hold as written. Moved, with the statement unchanged: A-4 (`section.rs`: `AuthoredEconomy` 33-44,
`CARRIED_BY` 73, the shop-on-a-person refusal 117-122, moved by S25a's schema attributes); A-15 (now
`docs/decisions/ARC-61.md` items 6 and 8, lines 59-77); A-17 (`tools/cli/src/main.rs:587-589`, `read_with` then
`load`); A-20 (`ac1_composability.rs`: `GENERIC_PACKS` 1041, `compare_configure` 1104-1136, the `configure/` loop
1359-1368; the `ARC-35` notes in `docs/decisions/ARC-35.md`); A-31 (#171 and #200 merged, #189 open). Changed:
A-23 (F-S31a-19 below). No change alters a decision of this design.

### Commit ledger

| Commit | SHA | Implementation | Validation | Review (self) |
| --- | --- | --- | --- | --- |
| C0 base, prerequisites, ledger | this commit | [x] §15.0; base binary and base save (Evidence) | [x] `scripts/ci_layer.py docs` | [x] §2.3 holds on the base, as §15.0 records |
| C1 specifications first | this commit | [x] `docs/decisions/ARC-111.md` … `ARC-114.md`, `DEP-63.md` (new, one file each), index regenerated; dated notes appended to `ARC-38.md` and `ARC-35.md`; `MODULE_SPEC.md` §4.1 economy row and `configure/economy.yaml` paragraph; `CORE_CONCEPTS.md` §§7, 8 notes; `ARCHITECTURE.md` §4 paragraph | [x] `scripts/ci_layer.py docs` PASS (headings, 180 ids, index current) | [x] no new ontology term (ask, keeper, reference value, wealth are economy vocabulary, step-31 §3); every code of §5.4–§5.7 appears in `ARC-112`; no doc contradicts an SD of §11 |
| C2 … C8 | — | [ ] | [ ] | [ ] |

### Evidence

- **C0 base binary.** `cargo build -p mineworld-cli` at 44f050e3 (1 min 39 s, dev profile, Darwin arm64), kept as
  `scratch-s31a/base/mineworld-44f050e3`, sha256 `ba7df987fa5326ee42d8934e059621b3c15e7e7f8735ff8af1f962392cd99255`.
- **C0 base save (AC-a-9's reference).** The base binary on a byte copy of the base's `worlds/market-town`
  (`scratch-s31a/base/world/market-town`, `diff -r` identical at 44f050e3), so that later commits' content cannot
  reach it: `run world/market-town --headless --seed 7 --days 30 --save scratch-s31a/base/market-town-30`.
  **Not run: milestone.** The lane lead's update of 2026-10-18 (the charter's "SPEED FIRST" block, operator)
  moves every long run, acceptance measurement and planted mutation to the milestone; the queued run was
  cancelled before it got a slot. The base binary and the world copy stay in scratch, so AC-a-9's base save can
  be written from them at the milestone exactly as above.

**Validation policy from C0 on (lane lead, 2026-10-18, charter "SPEED FIRST").** Locally: the build, fmt, clippy
of the touched crate, and only the test files this PR writes or changes, each through `with-slot.sh`. AC-a-4,
AC-a-6 … AC-a-9, AC-a-12, the `tools/cli` market files, the guide test and the planted mutations of §8.1 are
recorded "not run: milestone". Still no push and no PR.

### Mutations

(none yet; X-1, X-2, X-3, X-9, X-11, M-a-1 … M-a-3 as §8.1 lists them)

### Deviations and findings

F-S31a-1 … F-S31a-18 (§3.2) and SD-S31a-1 … 14 (§11), recorded at design time. SD-S31a-4 reorders the step's C6
content into C2 (bounded: same content, earlier).

- **F-S31a-19 (C0) — A-23 no longer holds: the executable guide's world enables economy.**
  - *Previous assumption:* "only Market Town enables economy; no other test world or fixture does" (A-23, at
    `dd357ca6`).
  - *Audit evidence:* `git grep -ln "^  - economy" -- '*.yaml'` at 44f050e3 lists
    `docs/guide/examples/corner-shop/world.yaml` beside Market Town. Chapters 05, 07 … 10, 12 and 13 of
    `docs/guide/` (and their `zh-CN` mirrors) build and run a `corner-shop` world that enables `economy`, with a
    wallet on each person and a shop on `corner-shop-co`; `tools/cli/tests/guide.rs` executes every command of
    every page and compares the output (`docs/GUIDE_FORMAT.md`).
  - *Corrected understanding:* with the currency required (`ARC-111`, SD-S31a-1, AC-a-7), every `validate` and
    `run` of that world after step 5.4 fails naming `economy-currency-undeclared`, so the guide cannot stay as
    it is without changing a frozen invariant.
  - *Decision (bounded):* the guide's world declares its currency exactly as Market Town does — one
    `configure/economy.yaml` and one `configure:` entry — in its own commit right after C2 (C2b), so that it can
    be reviewed, moved or dropped on its own. No invariant, contract or acceptance criterion changes. Flagged to
    the lane lead, because the guide is a human tutorial (operator ruling QS25-1) and this adds one step to it.
- **F-S31a-20 (C0) — a hidden consumer of the purchase's facts.** `tools/cli/tests/milestone_c.rs:13, 301-305`
  asserts that a purchase records exactly two events. After C5 a purchase records three, `sold` first (§5.7). The
  assertion and its doc line move with C5 (bounded: the frozen order of §5.7, nothing else in that test changes).
