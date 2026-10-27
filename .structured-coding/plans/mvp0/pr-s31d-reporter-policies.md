# PR S31-d — The market price reporter's decision model, the pricing policies, and the LM reporter's path

## DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S31/S29 lane design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s31d, branch docs/pr-design-s31d,
                        planning base origin/main @ 44f050e3 (the lane lead's ruling of 2026-10-19; the
                        branch was fast-forwarded to c61f9a14, which changes no file this design cites).
                        First drafted on a1e05874; every file #258, #261 and #171 moved was re-read and is
                        marked "(main @ 44f050e3)"
Approved by / evidence: none yet. The step's rulings (Q-S31-1 … Q-S31-8, step-31 §18), S31-a's (Q-S31a-1 … 4)
                        and S31-b's (Q-S31b-1 … 4) bind and are not re-asked
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of S31-a, S31-b,
                        S31-c, S31-h and WE-a (#189)
Start condition:        step-31 §16.5: "Nothing starts before Milestone F's checklist has been handed to
                        the operator (the S29 timing rule). Each PR starts only once its own PR design is
                        frozen." "Third wave: S31-d, after S31-b and S31-c." The S31-d row of §16.2 also
                        names S31-a. State at 2026-10-18: Milestone F's checklist NOT handed over; S31-a
                        FROZEN, implementation not started, no PR open; S31-b FROZEN, implementation in
                        progress on mvp0/pr-s31b (C0 and C1 committed at 9d3e75f3), no PR open; S31-c DRAFT
                        (pr-s31c-price-survey.md at 9c4b5296 on docs/pr-design-s31c); this design DRAFT.
                        Then the primary session's (or lane lead's) dispatch
Execution contract:     §12 (filled; ends at MERGE-READY: pushed, `fast` green on the exact head, reported
                        with PR number and head SHA; the coordinator merges)
Implementer:            split by commit (step-31 §16.2, the S31-d row: "Haiku for the deterministic model and
                        policies from a commit plan; Opus for the LM cassette wiring"): Haiku-class for C0 …
                        C5 and C8; Opus-class for C6 and C7. No independent reviewer per PR (speed first);
                        acceptance criteria and planted mutations run at the milestone
Lifecycle:              DRAFT
```

**Effort:** `mvp0`, release-blocking (Q-S31-1 re-ruled) · **Step:**
[`step-31-wealth-prices.md`](step-31-wealth-prices.md) (APPROVED 2026-10-15) §§0.1 (items 3, 6), 1 (outcomes 7, 8),
3 ("reporter", "suggested price", "pricing policy"), 9.1, 9.4, 10, 13, 14 (I-S31-1, I-S31-8, I-S31-10), 15 (AC-S31-11,
X-6; AC-6 through the template test), 16.2 (the S31-d row), 16.5, 19.4 (`ARC-116` backend half, `ARC-117`) ·
**Parent:** [`overall.md`](overall.md) · **Depends on:** [`pr-s31a-currency-asks.md`](pr-s31a-currency-asks.md)
(FROZEN: asks, the shop listing, the `asks` view, the `currency` record, `set-price`, `list-for-sale`, `buy`),
[`pr-s31b-notices.md`](pr-s31b-notices.md) (FROZEN: `publish-notice`, `notice-board`, `PriceLine`; its §10
R-S31d-b-1 binds this PR), `pr-s31c-price-survey.md` (DRAFT, commit 9c4b5296: the `survey-book` record and its
§5.10 read surface) · **Coordinates with:** `pr-s31h-haggling.md` (DRAFT, commit 4d5a81f6 on
docs/pr-design-s31h: its §5.12 options live in this PR's `pricing:` block; its Q-S31h-2 and Q-S31h-4) ·
**Followed by:** S31-f (Market Town's reporter, keepers and policies), S31-g (agent `act` kinds) · **Binding
parents:** `CLAUDE.md` §§2, 3, 4 (items 1, 3, 4, 5, 7, 8, 9, 10, 11, 13, 15, 16); `docs/CORE_CONCEPTS.md` §§14,
15.2; `docs/MODULE_SPEC.md` §5.1; decisions `ARC-27`, `ARC-34`, `ARC-35`, `ARC-58`, `ARC-72`, `ARC-84`, `ARC-85`,
`ARC-153`.

**Decision ids.** This PR writes:

- **`ARC-117`** (assigned by the step, step-31 §19.4: "Pricing policies are decision-model parameters and models,
  defaulting to `hold`");
- **a note under `ARC-116`**, `## ARC-116 note — the backend half (S31-d)`, inside the record S31-c writes
  (S31-c §2.2 and R-S31d-c-1: "It writes `ARC-116`'s backend half as a dated note under the record this PR
  writes");
- **`ARC-182`** (new, from the sub-range the lane lead gave: controllers read the market packs' records and build
  their requests by slug, through wire mirrors that an owner-side test holds equal) and a note under **`ARC-35`**
  that points to it — both only if Q-S31d-1 is ruled (a);
- **`ARC-183`** (new, from the same sub-range: the LM controller fills an incomplete offer through a typed filler
  from a closed registry) and a note under **`ARC-72`** — both only if Q-S31d-2 is ruled as recommended.

`DEP-100` stays unused: nothing is adopted (§4). Every id above is free on origin/main @ 44f050e3
(`git cat-file -e origin/main:docs/decisions/<ID>.md` fails for `ARC-116`, `ARC-117`, `ARC-182`, `ARC-183` and
`DEP-100`). Since #258 the log is one file per record, `docs/decisions/<ID>.md`, with a generated `INDEX.md`
(`python3 scripts/check_decision_ids.py --write-index`); a note on an existing record is a
`## <ID> note — …` heading inside that record's file (for example `docs/decisions/ARC-110.md:58` on main).

No hosted model is called by this design or by anything it specifies for CI. The LM half is tested with a
scripted model and a cassette recorded from that scripted model, so no recording costs money.

### Local ids in this file

| Prefix | Meaning |
| --- | --- |
| `A-n` | audit anchors (this file's own numbering) |
| `F-S31d-n` | findings |
| `SD-S31d-n` | bounded decisions |
| `I-d-n` | invariants of this PR |
| `AC-d-n` | acceptance criteria of this PR |
| `M-d-n` | planted mutations beyond the step's `X-6` |
| `R-S31d-n` | risks |
| `Q-S31d-n` | operator questions |
| `W1 … W6` | the worlds the acceptance runs on (§8) |

---

## 0. Binding inputs, exactly restated

- **Operator requirement, 2026-10-15 (step-31 §0.1, verbatim):** "必需的消耗品 食材等等，可以有市场建议价格（一个专门的npc
  负责统计，然后给出建议，这个npc后端可以是llm agent，也可以是确定性的算法什么的，这个也是模块化的…）" — *"Essential
  consumables, food ingredients and so on, can have a market suggested price. A dedicated NPC is responsible for
  compiling statistics and giving the suggestion. This NPC's backend can be an LLM agent or a deterministic
  algorithm, and that is modular too."* S31-c builds the notebook. **S31-d builds "giving the suggestion", in two
  backends that a world swaps by configuration.**
- **Operator principles, 2026-10-18 (relayed in this task).** "The price clerk publishes suggested prices through
  the `notices` pack, and its backend is modular — deterministic or an LLM, swappable by configuration with no
  code change to the world. Core tests must stay LM-independent (recorded cassette for the LM path; no live
  calls, no money)." Also: one currency per world; any seller sets its own price; haggling exists; agents over MCP
  play by the same server path and rules.
- **No world-specific hard-coding (operator, 2026-10-18, lane-lead charter).** "Mechanisms are generic and
  data-driven … rule = configurable policy data, motivated = scored option, LLM = decides from the disclosed
  facts … Every acceptance criterion names at least one second world or a generated variant."
- **The coordinator's restatement (step-31 §0.1, items 3 and 6).** "The reporter is a real Person with a role. Its
  backend is pluggable through the existing decision-model registry (`models!`, `external "lm"`), with no new
  plugin mechanism. It works only from what it can legitimately perceive." "Pricing models may read suggestions,
  so that prices converge."
- **Scope (step-31 §16.2, the S31-d row).** "`cognition/price-reporter` (`models!` line);
  `mineworld_rule_controller::price` (`hold`, `toward-suggestion`, `scarcity`, `list-goods`, `buyer`); the
  `pricing:` block in `rule` and `motivated`; template determinism test (AC-6); LM cassette for AC-S31-11;
  `ARC-116` (backend half), `ARC-117`." Depends on "S31-a, S31-b, S31-c". Checkpoint "fixture world 10 d: notices
  published, a policy follows; backend swap". Adversarial "X-6".
- **Q-S31-5 (ruled 2026-10-15).** "The shared `pricing:` block, default `hold`." A custom algorithm is still a model
  in `models!`.
- **AC-S31-11.** "The reporter seat on `price-reporter` and on `lm` (cassette replay) both publish notices that
  validate. The notices' type and audience are identical; only body values differ. No Rust or protocol file
  differs between the two runs."
- **X-6.** "`toward-suggestion` reads remembered notices instead of the board" must fail "the expired-suggestion
  policy test".
- **R-S31d-b-1 (S31-b §10, binding).** "The reporter publishes `{ topic, body: { prices: [...] }, valid_for }`
  through `publish-notice` by name (ARC-34); a pricing policy reads only the `notice-board` record (current
  notices), never remembered ones (X-6); the deterministic and the LM backend submit the same request shape,
  validated by the same `judge_publish` (AC-S31-11)."
- **R-S31d-c-1 (S31-c §10, consumed exactly).** "S31-d reads the book only through §5.10: the `survey-book` record on
  the observer, decoded with `mineworld_price_survey::PriceBook`. It never reads S28 memory, the `perceived` stream
  or another component for prices … It dedups repeated ask notes itself if its statistic needs it … It attempts
  the complete `survey` offer … and handles `nothing-to-survey` and `surveyed-recently`."
- **The lane lead's update, 2026-10-18 (binding for this design).** (1) #261 merged, `ARC-153`: the MCP agent
  surface is one `act` tool with typed kinds. (2) #258 merged: one file per decision record. (3) #171 (12d)
  merged: Market Town's items and bodies changed; re-audit. (4) S31-h's finding: the LM controller attempts only
  complete offers, and `publish-notice` is incomplete; "either design that mechanism in S31-d, or put a numbered
  question with a recommendation". (5) S31-h's §5.12 options live in this PR's `pricing:` block; "state the
  interface from your side". This design does both (4) and (5): §5.8 and §5.6.
- **Speed first (operator, 2026-10-18, lane-lead charter, top block; the brief's "SPEED FIRST and no
  hard-coding").** "MERGE-READY: pushed, `fast` green on the exact head; reported with PR number and head SHA; the
  coordinator merges." Per PR: the build and the tests the commit writes, plus `fast` CI; no independent reviewer,
  no mutation run, no full or targeted suite. The acceptance criteria (§8) and planted mutations (§8.1) are kept,
  written in the PR, and **run at the milestone**, where failures are fixed in one batch. Designs are still frozen
  first. Every acceptance criterion names a second world or a generated variant besides Market Town (§8).

---

## 1. Goal

Give a world a **price clerk** whose "brain" is swappable, and give sellers a configurable way to **react to what
the clerk publishes**.

1. **The deterministic clerk.** A new decision model, `price-reporter`, drives a seat like the rule controller
   does (it walks its routine, answers people, greets). On top of that it does two jobs. Where asks are on display,
   it takes the `survey` the world offers (S31-c). At a few configured times of day it reads its own price book,
   computes one suggested price per kind it has enough notes on, and posts them as one `prices` notice on its topic
   through `publish-notice` (S31-b).
2. **The LM clerk.** The same seat can be driven by the `lm` model instead. Once a world day, the LM controller asks
   its language model to write the bulletin from the same price book, checks every line, and submits the same
   `publish-notice` request. Swapping one for the other is one line in `decisions.yaml`. No world file and no Rust or
   protocol file changes.
3. **Pricing policies.** `rule` and `motivated` gain a typed `pricing:` parameter block. Its default, `hold`, changes
   nothing, so every existing world decides byte for byte as before. A world that chooses `toward-suggestion` makes
   its sellers move their asks a bounded step toward the current suggestion; `scarcity` moves them by their own
   stock; `list` lists held goods at their reference value; `buy.choose: cheapest` makes buyers take the cheapest
   available offer of a kind.

A concrete example. In a copy of Market Town, Carol is the clerk. Every morning she shops in the store and lunches in
the café, so her book fills with the sales she sees. At 08:00 her model reads the book: juice sold at 250 in the store
and at 350 in the café, so the median sale is 300. It posts `juice 300 (basis 5)` on `prices.ingredients`. Felix, the
store's keeper, subscribes. At his next pricing slot he sees the current suggestion 300 against his ask 250, and moves
10 % of the way: `set-price juice 255`. Alice, the café's keeper, moves hers from 350 to 345. Over days the two asks
meet. If the world's `decisions.yaml` says `carol: lm` instead, the same book reaches a language model, its bulletin
is checked line by line, and the same kind of notice reaches the same subscribers.

Nothing changes in `kernel/`, `contracts/`, `persistence/` or `server/` (I-S31-1), and no reference world changes
(S31-f composes Market Town).

---

## 2. Scope

### 2.1 In scope

1. **`cognition/rule-controller`** (`mineworld-rule-controller`):
   - `src/price/mod.rs`: the module's entry points `seller_band`, `on_chosen_buy`, `suggestion` (§5.4 … §5.6).
   - `src/price/block.rs`: `Pricing`, `Sell`, `Buy`, `SellPolicy`, `BuyChoice`, `ListRule`; the hand-written decoder
     (§5.4).
   - `src/price/wire.rs`: the wire mirrors of the market packs' records and requests (§5.7; Q-S31d-1).
   - `src/price/slots.rs`: the slot arithmetic shared with `price-reporter` (§5.3).
   - `src/price/sell.rs`, `src/price/buy.rs`: the policies (§5.5).
   - `src/price_tests.rs`: unit and observation tests (W1).
   - `src/model.rs`: `Rule`'s parameters become `RuleParameters { pricing }`; `src/paced.rs` and `src/offered.rs`: the
     two call sites (§5.6); `Cargo.toml`: `mineworld-notices` (the board's types; not a market pack) and
     `schemars` (the block's JSON Schema, which `ModelParameters` requires).
2. **`cognition/motivated`**: `MotivatedParameters` gains the `pricing` key; `model.rs` calls the seller band after
   the reply and before scoring; `options.rs` passes a chosen `buy` through `on_chosen_buy` (§5.6).
3. **`cognition/price-reporter`** (new crate `mineworld-price-reporter`): `Cargo.toml`, `clippy.toml`, `README.md`,
   `src/{lib.rs, model.rs, parameters.rs, suggest.rs}`, `src/tests.rs` (§5.2, §5.3).
4. **`cognition/installed`**: one line in `Cargo.toml`, one in `src/lib.rs` (`PriceReporter =>
   mineworld_price_reporter::PriceReporter`, after `Motivated`, before `external "lm"`).
5. **`cognition/lm-controller`** (Python): the filler registry and the `publish-notice` filler; the `due` trigger;
   the extended choice, validation and intent; the cognition TOML table `[fill.publish-notice]`; tests, a committed
   cassette recorded from a scripted model, and a scripted reporter process (§5.8).
6. **`systems/economy/tests/controller_wire.rs`** (new): the drift guard that holds the wire mirrors equal to
   economy's, inventory's and item's real types (§5.7). Test code only; no economy source changes.
7. **`tools/cli/tests/price_reporter.rs`** (new, deterministic checkpoint) and
   **`tools/cli/tests/price_reporter_lm.rs`** (new, the swap; run by the python layer) (§8).
8. **`tools/cli/tests/packs.rs`**: the expected controller packs gain `mineworld-price-reporter`.
9. **Documentation:** `docs/decisions/ARC-117.md`, the `ARC-116` note, and (as ruled) `ARC-182.md`, `ARC-183.md`
   with notes under `ARC-35` and `ARC-72`; `docs/decisions/INDEX.md` regenerated; `docs/MODULE_SPEC.md` §5.1; the
   generated `docs/reference/decisions.md`, `docs/reference/schema/decisions.schema.json` and
   `docs/reference/cognition-toml.md`.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Market Town's reporter, keepers, references, topic, subscriptions and policies; the 30-day convergence checkpoint AC-S31-13 | S31-f |
| The haggling options `buy.haggle` and `sell.accept_at_least_permille`, and `haggle_instead` / `answer_proposals` | S31-h, at the call sites §5.6 fixes (Q-S31h-2, "whichever merges second") |
| LM fillers for economy's actions (`set-price`, `list-for-sale`, `withdraw`) and S31-h's four | a follow-up (or S31-h); the registry of §5.8 takes each as one module and one line |
| MCP agent `act` kinds for `publish-notice`, `subscribe`, `unsubscribe` and the economy actions | S31-g (`ARC-153`: one `act` tool, closed typed kinds) |
| Words for any code in the 2D / 3D clients | S31-e |
| A reporter that also sells, or a seller policy on the reporter seat | not in S31 (SD-S31d-11) |
| Any change under `kernel/`, `contracts/`, `persistence/`, `server/`, `clients/`, `systems/*/src` | never in this PR (I-d-1) |

### 2.3 Dependencies

- **Required before C0:** Milestone F's checklist handed over; this design frozen; S31-a, S31-b and S31-c merged
  (the `asks` view, the `currency` record, `publish-notice`, `notice-board` and `survey-book` exist only after them);
  the dispatch.
- **Either order, interface rule:** S31-h. If S31-h merged first, its options and two functions are in `price/`
  already, under the names §5.6 fixes (R-S31h-d-1); this PR keeps them and calls them. If S31-d merges first, it
  leaves the two call sites and the reserved keys of §5.6, and S31-h's C6 attaches there.
- **Either order, modifier rule:** WE-a (#189). A shop listing shows the price after modifiers. C0 records whether
  WE-a has merged and how a modifier shows; §5.5 then skips a shop ask while one is active (R-S31d-4).
- **Rulings that shape commits:** Q-S31d-1 (C2, C3), Q-S31d-2 and Q-S31d-3 (C6, C7), Q-S31d-4 (C2).

---

## 3. Audit (`origin/main @ a1e05874`; files moved by #258, #261, #171 re-read on `origin/main @ 44f050e3`)

S31-a and S31-c are designs, not code. Anchors on their code cite their designs; C0 re-reads them on the merged code.
S31-b's anchors cite its implementation in progress (`/Users/yuema137/mineworld-worktrees/impl-s31b`, read only).

### 3.1 Source anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | `DecisionModel` is a pure function of `(parameters, seed, pace, observation)`: `&self`, no interior mutability, no clock, randomness only from `Draw` | `cognition/decision/src/model.rs:86-108`; `docs/MODULE_SPEC.md` §5.1 | The reporter cannot remember its last bulletin; schedules are computed from world time (§5.3) |
| A-2 | `Rule` is parameterless: `type Parameters = ()`, and `()` refuses every key ("an empty mapping: this model takes no parameters") | `cognition/rule-controller/src/model.rs:24-36`; `cognition/decision/src/parameters.rs:38-69` | Adding `pricing:` to `rule` changes its parameter type (§5.4) |
| A-3 | A seat's parameters are its **model's** block, not the seat's: "its parameters are the file's block for that model, else defaults" | `docs/MODULE_SPEC.md:776` (main @ 44f050e3); `cognition/decision/src/read.rs:269, 353-390` | Step §10.1's example `felix: { model: rule, parameters: { pricing: … } }` is not a valid file (F-S31d-4) |
| A-4 | `Decoded::canonical` is the parameters' JSON, used only by the printed `decisions` digest ("for a person to compare … not evidence") | `cognition/decision/src/models.rs:31-38`; `resolve.rs:268-292` | `rule`'s canonical changes from `null` to the defaulted block; no baseline reads it (`scripts/baselines.txt` has no decisions line) |
| A-5 | The installed models are `Rule`, `Motivated`, `external "lm"`; the template test runs every in-process model on Social Café twice and across a resume, byte for byte | `cognition/installed/src/lib.rs:20-24`; `tools/cli/tests/decision_models.rs:51-103` | `price-reporter` is held to AC-6 with no test edited |
| A-6 | `packs list` test expects exactly `["mineworld-rule-controller", "mineworld-motivated"]` as controller packs | `tools/cli/tests/packs.rs:116-120` | C4 updates it |
| A-7 | The paced rule decides: invitation → reply → agenda → social → offered band → walking roll. Draw indices 0–6 and 8–15 are taken; 7 is free | `cognition/rule-controller/src/paced.rs:36-56, 131-199` | The pricing bands take no draw at all, so no existing draw moves (§5.6) |
| A-8 | The offered band attempts a random available **complete** affordance on 20 % of consults; its constants are frozen (`ARC-35` point 6) | `cognition/rule-controller/src/offered.rs:26-52` | `buy.choose` changes which buy is submitted after the band chose one, never the constants |
| A-9 | `motivated`: invitation → reply → score every choice; `Choice::Offered(index)` for each available complete affordance | `cognition/motivated/src/model.rs:48-76`; `options.rs:285-289, 340` | The seller band runs after the reply; the buy hook sits in `options::request` |
| A-10 | `read::available::<A>` reads the server's verdict; `read::disclosed::<C>` decodes a record by its component type | `cognition/rule-controller/src/read.rs:111-127` | Every policy reads verdicts and records, and computes no rule (`ENGINEERING_RULES.md` §8) |
| A-11 | `act::request` and `act::record` build a request from any `Action` type | `cognition/rule-controller/src/act.rs:160-176` | Requests are built through typed values, mirrors included |
| A-12 | `trait Action { ACTION_TYPE; OWNER }`, `trait Component { COMPONENT_TYPE; OWNER; SCHEMA_VERSION }`; `payload_for` refuses another label and a newer schema | `contracts/src/action.rs:327-332`; `contracts/src/component.rs:91-98, 206-215` | A mirror is a type with the owner's labels; a newer owner schema is refused, not guessed (§5.7) |
| A-13 | **`ARC-35` check 2**, enforced at HEAD by `check_2_the_dependency_structure`: the direct dependents of each market pack (`item`, `inventory`, `item-transfer`, `economy`, `employment`, `consumption`) are `systems/*` crates only; no normal or build path leads to a market pack from `kernel`, `contracts`, `persistence`, `server`, `authoring`, `sdk` or **`rule-controller`**; no `*.rs` or `Cargo.toml` outside `systems/`, `worlds/`, `tests/acceptance/` names a market crate | `docs/decisions/ARC-35.md:30-37` (main @ 44f050e3); `tests/acceptance/tests/ac1_composability.rs:68, 651-662, 884-913` (main @ 44f050e3) | `rule-controller` may not depend on `mineworld-economy` or `mineworld-item`, and no `cognition/` crate may name them (F-S31d-1) |
| A-14 | `ARC-35` item 6: "World-level tests read the market by type slugs, never through a market crate … into test-local mirrors: unknown fields refused, schema version 1 asserted" | `docs/decisions/ARC-35.md:182-186` (main @ 44f050e3) | The sanctioned precedent for reading the market by slug (§5.7) |
| A-15 | The shop listing `{ operator, listed: [{ item, price, in_stock }] }` under component type `shop`, on the place, to whoever perceives it | `systems/economy/src/component.rs:97-165`; S31-a §5.9 (shape kept, SD-S31a-12) | Mirror `ListingSeen` |
| A-16 | S31-a: the `asks` view `{ asks: [{ item, price, available }] }` of a person seller; the `currency` record `{ name, symbol, exponent, symbol_at, references: [{ item, value }] }` to every living-person observer; `set-price` / `list-for-sale` payload `{ item, price }` or `{ item, price, for: "shop" }`; one incomplete offer per seller action type; `set-price` to the same price is `price-unchanged` | `pr-s31a-currency-asks.md` §§5.3, 5.5, 5.6, 5.9; SD-S31a-6, SD-S31a-14 | Mirrors and the keeper inference (F-S31d-6) |
| A-17 | inventory's `holdings` record `{ held: [{ item, count }] }` to its holder | `systems/inventory/src/component.rs:36-48, 91-93` | Mirror `HoldingsSeen` (own stock for `scarcity` and `list`) |
| A-18 | Since 12d, `item` discloses an `item-catalogue` record on every perceived place: every declared kind, its category and its name, `{ kinds: [{ item, category, name }] }` | `systems/item/src/system.rs:89-113`, `component.rs:45-111` (main @ 44f050e3) | A controller can tell a `goods` kind from `food` (`sell.list`), and the LM can name kinds (F-S31d-8) |
| A-19 | Market Town since 12d: 24 item keys, 20 kinds plus four loose objects (`cafe-ball`, `cafe-box`, `street-ball`, `street-box`) that are Item entities with a `body:` and no `item:` section; `bodies` sits before `persona` | `worlds/market-town/world.yaml:145-169`, `items/cafe-ball.yaml` (main @ 44f050e3) | Scratch copies are made from main at C0; a loose object is never a tracked kind |
| A-20 | Market Town people: Carol (`resident`) routine store 07:00, café 12:00, apartments 16:00; Felix store 08:00 (job at corner-store from 08:00), café 13:00; Alice (`barista`, `staff`) café 05:30, park 14:00; Hana (`commuter`) workplace 08:00, café 15:00, apartments 19:00. The store asks apple 100, bread 300, milk 200, juice 250; the café coffee 300, tea 250, croissant 250, cake 400, sandwich 600, soup 500 | `worlds/market-town/people/*.yaml`, `organizations/*.yaml` (main @ 44f050e3) | W2, W3 and W6 (§8) |
| A-21 | `run` consults seat `k` at `genesis + k + m·PACE`, `PACE = 900 s`; a consult answers a line heard in `(t − pace, t]` | `tools/cli/src/run.rs:48, 632-641`; `paced.rs:20-24` | The same half-open window places every slot in exactly one consult (§5.3) |
| A-22 | S31-b (implementation in progress): `PublishNotice::new(topic: TopicId, body: NoticeBody, valid_for: u32)`; `NoticeBody::Prices(Vec<PriceLine>)`; `PriceLine::new(item: EntityId, price: u64, basis: u16)`; `NOTICE_PRICE_MAX = 10^12`; `PRICE_LINES_MAX = 32`; `NoticeBoardRecord { notices: Vec<Notice> }` under `notice-board`; `Notice::{topic, publisher, body, issued_at, valid_until}`; `publish-notice` offered incomplete to a publisher; `TopicId` a dotted slug | `impl-s31b/systems/notices/src/{action.rs:18-50, body.rs:13-60, component.rs:62-75, board.rs:42-101, topic.rs:41-71}` | The reporter builds `PublishNotice` with notices' own types (notices is not a market pack: allowed) |
| A-23 | S31-c §5.10: `read::disclosed::<mineworld_price_survey::PriceBook>(me)`; accessors `kinds()`, `kind(item)`, `KindNotes::{item, notes}`, `Note::{at, place, seller, price, source, in_stock}`, `Source::{Ask, Sale}`; ask notes repeat across surveys; `survey` offered **complete**, refused `surveyed-recently` during the cooldown | `pr-s31c-price-survey.md` §§5.3, 5.6, 5.10 (commit 9c4b5296) | `suggest` (§5.2) and the survey band; `price-survey` is not a market pack, so `price-reporter` may depend on it |
| A-24 | The LM controller's model answer is `SocialChoice { act: say \| attempt \| nothing, to, words, offer, recalls }`; `attempt` names a numbered **complete** offer; `Situation.offers` lists only available affordances with a payload | `cognition/lm-controller/src/mineworld_cognition/controller/choice.py:38-66`; `situation.py:73-79`; `validate.py:45-90`; `intent.py:29-57` | The LM cannot submit `publish-notice` today (F-S31d-2; the lane lead's item 4) |
| A-25 | `ARC-72`: "A model's answer becomes a `talk` utterance or the unchanged payload of a complete affordance the server offered, and nothing else"; it "never builds a movement request" | `docs/decisions/ARC-72.md:26` (main @ 44f050e3); `docs/MODULE_SPEC.md:1904` (main) | Filling an incomplete offer amends `ARC-72` (Q-S31d-2); an LM seat never walks (F-S31d-7) |
| A-26 | `mineworld_sdk.offers.request(observation, action_type, target=, payload=)` builds a request with the caller's payload, provided some affordance offers that type on that target as available — incomplete offers included | `sdk/python/src/mineworld_sdk/offers.py:55-84` | The LM's filled request goes through the SDK, as `ARC-72` requires |
| A-27 | Triggers are `spoken_to` (a `spoke` to me) and `arrived` (a remembered person entered my place); the LM controller has no time-based trigger | `controller/triggers.py:22-58` | The `due` trigger (§5.8.4; Q-S31d-3) |
| A-28 | `build_request(…, output_schema=None)` defaults to `SOCIAL_CHOICE_SCHEMA`; the request's canonical JSON, schema included, is the cassette key (`ARC-58`) | `controller/prompt.py:61-103` (main @ 44f050e3); `backend/canonical.py:45-65` | A changed schema on every request would miss every committed cassette; the fill schema appears only when a filler is listed (I-d-8) |
| A-29 | A cassette can be recorded from a scripted backend at no cost and replayed strictly; FG-c's `lm-summarise.jsonl` is the precedent | `tests/test_lm_summarise_replay.py:1-110`; `record.py:1-14` | The LM cassette of AC-S31-11 (AC-d-12) |
| A-30 | Milestone D's live test runs the shipped controller as a process with a scripted model, a pure function of the request; ignored in `core`, run by the python layer | `tools/cli/tests/milestone_d.rs:1-40, 120-152`; `tests/milestone_d_cognition.py:1-40` | The swap test `price_reporter_lm.rs` follows it (C7) |
| A-31 | `ARC-153`: the MCP tools are nine; `act` is a closed union of typed kinds; QS27-9's ban on a generic free-form act stands | `docs/decisions/ARC-153.md` (main @ 44f050e3) | S31-d adds no MCP surface; S31-g adds typed kinds (R-S31g-d-1). The LM filler registry follows the same principle: closed and typed |
| A-32 | `ARC-35` check 3 admits only the six market packs plus `GENERIC_PACKS = ["calendar", "weather"]` in Market Town's `systems`, and only market-owned sections in its files | `tests/acceptance/tests/ac1_composability.rs:1041-1095` (main @ 44f050e3) | Not this PR's (scratch copies are not Market Town), but S31-f's composition will need an `ARC-35` note (F-S31d-10) |
| A-33 | The server fast-forwards above 24× only with `--solo` (loopback); a paced seat is consulted every `--cadence` wall seconds | `tools/cli/src/main.rs:135-152` | The swap test runs `--solo --time-scale 1440`: one world day per wall minute |

### 3.2 Findings

- **F-S31d-1 — The step puts market knowledge where `ARC-35` forbids a crate dependency.** Step §10.1 places the
  policies in `mineworld_rule_controller::price` and builds `set-price` "through the owning pack's type" (step A-20).
  `rule-controller` is a framework crate in `ARC-35` check 2 (A-13): no normal path from it may reach `economy` or
  `item`, and no `cognition/` file may name `mineworld-economy` or `mineworld-item`. The check runs on every build,
  not only at the 11d/11e merges. So the step's plan fails CI as written. S31-h's §5.12 ("requests are built through
  economy's published types") fails the same way. → **Q-S31d-1** (recommended: read and build by slug through wire
  mirrors, the precedent `ARC-35` item 6 already sets for tests, with an owner-side test that holds the mirrors
  equal).
- **F-S31d-2 — The LM cannot publish today.** `publish-notice` is offered incomplete (Q-S31a-3), and the LM attempts
  only complete offers (A-24). `ARC-72` allows nothing else (A-25). → **Q-S31d-2** (recommended: a closed registry of
  typed **fillers**, one per incomplete action type; S31-d ships the registry and the `publish-notice` filler; S31-h's
  four and economy's seller actions follow as one module each). This answers S31-h's Q-S31h-4.
- **F-S31d-3 — The LM has no clock.** It acts only when spoken to or when a known person arrives (A-27). A clerk must
  publish on its own. → **Q-S31d-3** (recommended: a `due` trigger on calendar's `day-began`, raised only for a seat
  that a filler lists, so once per world day per reporter).
- **F-S31d-4 — Parameters are per model, not per seat.** A decision list configures each model once (A-3). Every seat
  on `rule` therefore gets the same `pricing:` block. Without a way to target sellers, a world could not make Felix
  follow suggestions and leave everyone else alone, nor make buyers haggle while sellers hold. → **Q-S31d-4**
  (recommended: the block has a `sell:` half and a `buy:` half, each with a `tags:` filter read from the seat's own
  disclosed tags). S31-h's per-seat example has the same defect.
- **F-S31d-5 — The step's buyer default is misnamed.** Step §10.1: "`buyer: cheapest | nearest` … `nearest` is the
  default and is today's behaviour". Today the offered band submits whichever available complete affordance its draw
  picks (A-8), which is not the nearest seller. The default is named `any` here (SD-S31d-9).
- **F-S31d-6 — A controller cannot see who keeps a shop.** The listing view keeps its shape (SD-S31a-12) and carries
  no `keepers`. Economy offers `set-price` (incomplete) when the person **or** a shop it keeps and stands in has an
  ask (A-16). So "in a shop with a listing, `set-price` offered, and no asks of my own" means "I keep this shop". A
  keeper who also sells on its own account cannot be told apart, and prices only its own asks (SD-S31d-6, R-S31d-8).
- **F-S31d-7 — An LM seat never walks.** `ARC-72` and `MODULE_SPEC.md` §5.1: the LM controller "never chooses
  movement" (A-25). An LM clerk therefore perceives only the place it stands in: its book holds that place's sales and
  surveys. The deterministic clerk walks its routine. → **Q-S31d-5** (recommended: accept for MVP-0; the swap test
  places the clerk where trade happens).
- **F-S31d-8 — Since 12d, kinds are named and categorised in every place's observation** (A-18). The step's
  `list-goods` can be a category filter in data (`sell.list.categories: [goods]`), not a code rule, and the LM prompt
  can name kinds without showing an entity id (step-17 I-10).
- **F-S31d-9 — Suggestions only move when sellers disagree or supply is off.** With one seller per kind and every sale
  at its ask, the sale median equals the ask, so `toward-suggestion` sees no gap. Movement comes from a second seller
  of the kind or from the supply adjustment. The checkpoint world therefore gives the café a juice ask beside the
  store's, by construction (§8 W2), so the criterion is not vacuous.
- **F-S31d-10 — S31-f's Market Town will meet `ARC-35` check 3.** Adding `notices` and `price-survey` to Market Town's
  `systems`, and their sections to person files, is outside check 3's allow-list (A-32). Not this PR's to change; it
  is reported to the lane lead for S31-f.
- **F-S31d-11 — S31-c's §5.10 JSON example shows ids as integers.** Contracts serialize an opaque id as a decimal
  string in JSON and a typed reference (`ItemId`, `PlaceId`) as a tagged object (`contracts/src/ids.rs:31-57, 186-198,
  732-733, 854-855`). The Python filler reads the real encoding; the example should be corrected (§16).
- **F-S31d-12 — After 12d a loose object is an Item entity.** S31-b's `judge_body` admits "a living entity of type
  `Item`" as a price line's item, so `cafe-ball` passes. S31-d's producers never name one (the book tracks kinds only;
  the LM filler admits only book kinds). Reported to the lane lead for S31-b (§16).

### 3.3 What does not exist

No `price-reporter` model, no `pricing:` block, no `price` module, no reader of the `notice-board` or `survey-book`
records anywhere in `cognition/`. No LM path for an incomplete offer, no time-based LM trigger. No world names a
reporter.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Candidates | Verdict |
| --- | --- | --- |
| Median and simple statistics | `statrs`, `average`, `medians` | **Declined:** they compute in floats; MineWorld's models are integer-only (`motivated`'s `#![deny(clippy::float_arithmetic)]`) and must be bit-identical on three platforms. An integer median over ≤ 64 notes is a sort and an index. Below the `DEP` threshold |
| Request building for the market packs | the packs' own crates | **Not allowed** by `ARC-35` check 2 (A-13) → wire mirrors (Q-S31d-1) |
| Scheduling "at 08:00 and 12:00" | `cron`, `chrono` schedules | **Declined:** a model has no clock and keeps no state (A-1); a slot is an integer comparison on world seconds, the same idiom as the rule's answer window (A-21) |
| LLM structured output for the bulletin | an agent framework, function-calling libraries | **Not applicable:** the LM controller already has the provider-neutral gateway, cassettes and strict JSON schemas (`ARC-57`, `ARC-58`, `DEP-27`); a filler adds one schema fragment and one validator |
| Pricing algorithms (dynamic pricing libraries) | none suited: they optimise revenue against demand curves the world does not disclose | **Declined:** the step asks for bounded, explainable policies over disclosed facts; anything richer is a new model in `models!` |

No `DEP` record is written: nothing is adopted and no infrastructure is rebuilt.

---

## 5. Design

### 5.1 Where each part lives, and why

```text
cognition/rule-controller/src/price/     pricing policies, shared by rule and motivated (step §10.1)
    block.rs   the pricing: block            sell.rs   seller policies     buy.rs   buyer choice
    slots.rs   slot arithmetic (also the reporter's)    wire.rs  market-pack mirrors (Q-S31d-1)
cognition/price-reporter/                the deterministic clerk, a model in models!
cognition/lm-controller/…/controller/fill/   the LM's fillers; publish_notice.py first (Q-S31d-2)
systems/economy/tests/controller_wire.rs the drift guard: mirrors = owners' types
```

The dependency picture after this PR, with the `ARC-35` rule it keeps:

```text
mineworld-price-reporter ──► mineworld-rule-controller ──► conversation, movement, group-activity, naming,
        │                            │                     schedule, notices (none is a market pack)
        ├──► mineworld-price-survey ─┼──► economy, item  (a path from price-reporter is allowed: it is not
        └──► mineworld-notices       │                    a framework crate, and it names no market crate)
                                     └──✗ economy, item  (forbidden: A-13; the module reads them by slug)
```

### 5.2 The deterministic clerk: `price-reporter`

**Crate.** `cognition/price-reporter` (`mineworld-price-reporter`), a Controller Pack with its own package identity
(`ARC-53`). Dependencies, each with a one-line comment in `Cargo.toml`: `mineworld-contracts`, `mineworld-decision`,
`mineworld-rule-controller` (daily life, `read`, `act`, `price::slots`), `mineworld-price-survey` (`PriceBook`,
`Survey`, its codes), `mineworld-notices` (`PublishNotice`, `NoticeBody`, `PriceLine`, `TopicId`,
`NOTICE_PRICE_MAX`, `PRICE_LINES_MAX`), `mineworld-packages`, `schemars`, `serde`, `serde_json`. No market crate is
named (A-13). `clippy.toml` copies `motivated`'s (interior mutability banned), and `lib.rs` carries
`#![deny(clippy::float_arithmetic)]`.

**The model.** `PriceReporter`, `ID = "price-reporter"`, `Parameters = ReporterParameters`. One consult:

```text
1  publish band   if a publish slot lies in (t − pace, t]       (§5.3)
                  and the server offers publish-notice, available  (read::available::<PublishNotice>, target none)
                  and my own survey-book record decodes             (read::disclosed::<PriceBook>(me))
                  and suggest(book, parameters) is not empty        (§5.3)
                  → publish-notice { topic, body: prices(lines), valid_for: valid_minutes × 60 }
2  survey band    if the server offers survey complete and available (read::available::<Survey>, target none)
                  → that affordance's own request (Affordance::request, as offered.rs does)
3  daily life     PacedRuleController::new(seed, pace).decide(observation)   — rule, unchanged, pricing hold
step              PacedRuleController::new(seed, pace).step(observation)     — rule's, unchanged
```

- **No draw.** Bands 1 and 2 take no draw; band 3 is the rule's own stream (`Draw::new`), so a seat that has no book
  and no publish offer decides exactly as `rule` (AC-d-4).
- **Why publish first.** A slot is one consult in a day of 96; the rule's reply window may lose one line on a
  slot consult. Survey is the second band because `surveyed-recently` keeps it to once per place per
  `resurvey_after` (S31-c Q-S31c-2), so it takes few consults.
- **Refusals are the world's.** If the server refuses the publish (for example `over-daily-limit`), that is
  journaled and the next slot tries again; the model never retries within a slot (SD-S31d-3).

**Parameters** (`models: { price-reporter: { parameters: { … } } }`), decoded by hand like `MotivatedParameters`,
each refusal at its key, `deny_unknown_fields`, every key optional:

| Key | Type | Default | Bounds | Meaning |
| --- | --- | --- | --- | --- |
| `topic` | topic id (S31-b's `TopicId` rule) | `prices.ingredients` | a dotted slug, 1 … 64 bytes | where the bulletin is posted |
| `first_minute` | minute of day | 480 (08:00) | 0 … 1 439 | the first publish slot |
| `last_minute` | minute of day | 1 200 (20:00) | `first_minute` … 1 439 | no slot after this |
| `every_minutes` | minutes | 240 | 15 … 1 440 | slots at `first`, `first + every`, … ≤ `last` |
| `valid_minutes` | minutes | 360 | 1 … 43 200 | the notice's `valid_for`; the topic's own bounds are the server's check |
| `min_basis` | count | 3 | 1 … 64 | notes a line must rest on (§5.3) |
| `scarce_below` | count | 3 | 0 … 1 000 (0: off) | supply adjustment threshold, low |
| `scarce_permille` | per mille | 100 | 0 … 1 000 | added when scarce |
| `plenty_above` | count | 20 | 1 … 100 000 | supply adjustment threshold, high |
| `plenty_permille` | per mille | 50 | 0 … 900 | subtracted when plentiful |

`last_minute < first_minute` is refused naming `last_minute`. The default topic is the id S31-b's R-S31f-b-1 fixes for
Market Town; a world names its own. Example: `first 480, last 1200, every 240` gives slots at 08:00, 12:00, 16:00 and
20:00, four a day, within S31-b's example `per_day: 4`.

**Why it is a model and not a world rule** (step §9.4): the median and the supply rule are this model's parameters.
Another model may publish a mean, a trimmed median or an LM's judgement through the same request.

### 5.3 The suggestion rule and the slot arithmetic

**`suggest(book: &PriceBook, p: &ReporterParameters) -> Vec<PriceLine>`** (pure, integer, in `suggest.rs`):

```text
for each KindNotes k in book.kinds()                      (by ItemId; already window-filtered at disclosure, A-23)
    sales  = prices of k's notes with source Sale                         every sale is its own observation
    asks   = for each (seller, place): the newest Ask note                S31-c §5.10 item 5: repeats are deduped
    stock  = in_stock of those deduped ask notes that have Some(count)    a person's ask has None
    if   len(sales) ≥ min_basis: base = median(sales), basis = len(sales)
    elif len(asks)  ≥ min_basis: base = median(asks prices), basis = len(asks)
    else: no line for k
    if stock is not empty:
        s = median(stock)
        if scarce_below > 0 and s < scarce_below: base = base × (1000 + scarce_permille) / 1000
        elif s > plenty_above:                    base = base × (1000 − plenty_permille) / 1000
    price = clamp(base, 1, NOTICE_PRICE_MAX)                            floor division in u128, then u64
    line  = PriceLine::new(k.item().entity_id(), price, basis as u16)
if more than 32 lines: keep the 32 with the largest basis (ties: lower item id first), then sort by item again
```

- **Median.** Sort ascending; odd count → the middle value; even count → `floor((a + b) / 2)` of the two middle
  values. Example: `[300, 310, 320, 330]` → 315.
- **Example (W1a).** Bread: sales 300, 320, 300 (basis 3) → median 300. Asks: corner-store at the store, newest
  `in_stock: Some(1)`; Grace at the store, `None`. Stock median 1 < 3 → 300 × 1 100 / 1 000 = 330. Line
  `{ bread, 330, 3 }`. Apple: one sale, one deduped ask → no line.
- **Integer floors are visible at small scales** (W1b, a currency with exponent 0): base 3, scarce +100 ‰ →
  `3 × 1100 / 1000 = 3`. The adjustment rounds away; that is the documented integer rule, not a defect.

**Slots** (`price::slots`, shared with the pricing bands, so both use one rule):

```text
in_window(t, pace, s)          = t − pace < s ≤ t                         the rule's answer window (A-21)
publish_due(t, pace, p)        = some s = d·86 400 + m·60 with d ∈ {day(t) − 1, day(t)} and
                                 m ∈ {first, first + every, …} ≤ last, and in_window(t, pace, s)
period_index(t, pace, every)   = floor((t mod (every × 60)) / pace)       the pricing bands (§5.5)
```

- A world second `s` lies in exactly one window of a seat consulted every `pace` (A-21), so each publish slot fires
  exactly once, with no memory. Example: `pace = 900`, seat offset 3; the consult at `t = 28 803` (08:00:03) holds
  the 08:00 slot (`28 803 − 900 < 28 800 ≤ 28 803`); the consult at 27 903 does not.
- `period_index` gives consecutive consults consecutive indices within one period, so each index is handled at most
  once per period, and exactly once when `pace` divides the period (`900` divides every multiple of 15 minutes).
- WorldTime 0 is midnight of the calendar's epoch date (`systems/calendar/src/configuration.rs:60-100`: the epoch is a
  date), so `t mod 86 400` is the local time of day. In a world without calendar the same arithmetic still runs on
  world seconds.

### 5.4 The `pricing:` block (`price/block.rs`; Q-S31-5 ruled; Q-S31d-4)

```yaml
# decisions.yaml — configured once per model (A-3), so the block says which seats it applies to.
models:
  rule:
    parameters:
      pricing:
        sell:                           # how a seat prices what it sells: its own asks, and a shop's it keeps
          policy: toward-suggestion     # hold | toward-suggestion | scarcity            default hold
          every_minutes: 240            # 60 … 10 080: the pricing period                default 240
          step_permille: 100            # 1 … 500: how far one move goes                 default 100
          deadband_permille: 30         # 0 … 500: a gap smaller than this is ignored    default 30
          floor_permille: 500           # 1 … 1 000: lowest ask, × the reference value   default 500
          ceiling_permille: 2000        # 1 000 … 10 000: highest ask, × reference       default 2 000
          low_below: 2                  # 0 … 1 000 (scarcity): fewer in stock → raise   default 2
          full_above: 10                # 1 … 100 000 (scarcity): more in stock → lower  default 10
          list: { categories: [] }      # 0 … 16 item categories: list held kinds of these at their reference value
          tags: []                      # 0 … 8 tags: only seats carrying one; empty: every seat on this model
          # S31-h adds: accept_at_least_permille (1 … 1 000; absent: off)
        buy:
          choose: any                   # any | cheapest                                 default any
          tags: []                      # 0 … 8 tags, as above
          # S31-h adds: haggle: { offer_permille: 1 … 999, deadband_permille: 0 … 1 000 }  (absent: off)
```

- **Types.** `Pricing { sell: Sell, buy: Buy }`; `Sell { policy: SellPolicy, every_minutes: u32, step_permille: u32,
  deadband_permille: u32, floor_permille: u32, ceiling_permille: u32, low_below: u32, full_above: u32, list: ListRule,
  tags: Vec<Tag> }`; `SellPolicy { Hold, TowardSuggestion, Scarcity }` (kebab-case); `ListRule { categories:
  Vec<Category text> }` (each the item category slug rule, read as text: `rule-controller` may not name `item`'s
  `Category` type, A-13; the rule is restated and the drift guard checks it, §5.7); `Buy { choose: BuyChoice, tags:
  Vec<Tag> }`; `BuyChoice { Any, Cheapest }`. All `Default`, `Clone`, `PartialEq`, `Debug`, `Serialize`,
  `JsonSchema` with ranges and doc comments (the generated reference prints them, `ARC-90`).
- **Decoding** is by hand, as `MotivatedParameters` does (`cognition/motivated/src/parameters.rs:1-8`): unknown keys
  refused listing the known ones, each bound refused at its value with "'<key>' is <low> … <high>, not <value>", a key
  written twice refused, `deny_unknown_fields` at every level. Cross-field: `floor_permille ≤ 1 000 ≤
  ceiling_permille` holds by the bounds.
- **`rule`.** `Rule::Parameters` becomes `RuleParameters { pricing: Pricing }` (one key, `pricing`, optional). `rule:
  {}` still decodes to the defaults; any other key is refused naming `pricing` (A-2). `Rule::decide` calls
  `PacedRuleController::decide_priced(observation, &parameters.pricing)`; `PacedRuleController::decide(observation)`
  stays, as `decide_priced(observation, &Pricing::default())`, for its existing callers
  (`tools/cli/src/hosted.rs:342`).
- **`motivated`.** `MotivatedParameters` gains the key `pricing` (its `KEYS` list and its decoder), held beside the
  derived `Settings`.
- **Default parity.** With the defaults, `seller_band` returns `None` and `on_chosen_buy` returns its input, and
  neither takes a draw. Every world without a `pricing:` block decides byte for byte as before (AC-d-6).
- **The tag filter** reads the seat's own tags from its own `PerceivedEntity` (`observation.entity(me)?.tags()`;
  presence discloses them, `systems/presence/src/observe.rs:130-139`). Example: `sell.tags: [keeper]` with
  `buy.choose: cheapest` and empty `buy.tags` makes only tagged keepers price, and every `rule` seat buy cheapest.

### 5.5 The policies (`price/sell.rs`, `price/buy.rs`)

**What a seller can price** (read from its own observation, never computed):

- **its own asks:** economy's `asks` view on its own entity (mirror `AsksSeen`), each priced with `set-price
  { item, price }`;
- **the shop's asks, when it keeps the shop it stands in:** the place's `shop` listing (`ListingSeen`) when the
  server offers `set-price` available **and** the seat has no asks of its own (F-S31d-6), each priced with
  `set-price { item, price, for: "shop" }`.

The **priceable list** is own asks by item, then the shop's asks by item.

**`seller_band(observation, pace, pricing) -> Option<ActionRequest>`**, at most one request per consult:

```text
skip unless my own tags meet sell.tags
j = period_index(t, pace, sell.every_minutes)                                        (§5.3)
j = 0  → list:   if sell.list.categories is not empty and the server offers list-for-sale available,
                 the first held kind (by item id) with count ≥ 1 whose category (from the place's
                 item-catalogue) is listed, that has no ask of mine, and that has a reference value
                 → list-for-sale { item, price: reference }
j ≥ 1  → ask number j − 1 of the priceable list (none: nothing), priced by sell.policy below
```

Every price a policy proposes is then bounded: `lo = max(1, ref × floor / 1000)`, `hi = max(lo, ref × ceiling /
1000)`, `new = clamp(new, lo, hi)` and `new ≤ 10^12`; if `new = p`, nothing is sent (economy would refuse
`price-unchanged`, A-16). A kind with **no reference value** (absent from the `currency` record's `references`) is
never moved: without a reference there is no bound (SD-S31d-7).

| Policy | For the ask `(item, p)`, with reference `ref` |
| --- | --- |
| `hold` | nothing, ever |
| `toward-suggestion` | `s` = `suggestion(observation, item)` (below); none → nothing. If `|s − p| × 1000 ≤ deadband × p` → nothing. Else `d = (s − p) × step / 1000`, truncated toward zero; `d = 0` becomes `sign(s − p)`; `new = p + d`, bounded |
| `scarcity` | `stock` = the listing's `in_stock` (shop ask) or my `holdings` count of the item (own ask). `stock < low_below` → `new = p + max(1, p × step / 1000)`; `stock > full_above` → `new = p − max(1, p × step / 1000)`; else nothing. Bounded. It reads no notice, so it works without `notices` |

**`suggestion(observation, item) -> Option<Suggested { price, basis, issued_at, publisher, topic }>`** (shared with
S31-h): among the `prices` lines for `item` in the observer's own `notice-board` record, the line of the notice with the
newest `issued_at`; ties go to the lower publisher id, then the lower topic id. It reads **only** the board, which lists
current notices only (S31-b §5.3, I-S31-8), so an expired suggestion is never used (X-6, R-S31d-b-1).

Examples (W1a, `step 100`, `deadband 30`, `floor 500`, `ceiling 2000`):

- Felix keeps the store; bread ask 300, current suggestion 330, reference 300. `|30| × 1000 = 30 000 > 30 × 300 =
  9 000` → `d = 30 × 100 / 1000 = 3` → `set-price { bread, 303, for: shop }` at the consult whose index is bread's.
- Suggestion 309: `9 000 ≤ 9 000` → nothing (inside the deadband).
- Ask 590, suggestion 2 000, reference 300: `d = 141` → 731 → bounded to 600. At 600 the next move is bounded to 600 →
  nothing.
- `scarcity`, shop bread `in_stock 1 < 2` → 330; `in_stock 12 > 10` → 270.
- `list` with `categories: [goods]`: Grace holds a scarf (category `goods`, reference 2 500) with no ask → `list-for-sale
  { scarf, 2500 }` at index 0. She also holds an apple (`food`) → not listed.

**`on_chosen_buy(observation, pricing, chosen: &Affordance) -> Option<ActionRequest>`** (the buyer hook): if the chosen
affordance is a `buy` (`wire::BUY`) and `buy.choose` is `cheapest` and my tags meet `buy.tags`, return the request of the
available complete `buy` affordance for the same item (decoded with `wire::BuySeen { item, price }`) with the lowest price,
the first in observation order on a tie. Otherwise return the chosen affordance's own request, unchanged. Example: the
offered band drew "buy bread from the store at 300"; Grace, 2 m away, offers bread at 260 → the request buys Grace's.

### 5.6 Call sites, and the interface with S31-h

| Model | Seller band | Buyer hook |
| --- | --- | --- |
| `rule` (`paced.rs`) | in `decide_priced`, right after the invitation answer and the reply, before the agenda: `if let Some(r) = price::seller_band(observation, self.pace, pricing) { return Some(r) }` | in `offered::attempt(observation, draw, pricing)`: the drawn affordance passes through `price::on_chosen_buy` |
| `motivated` (`model.rs`, `options.rs`) | in `decide`, after `reply`, before `score_all` | in `options::request` for `Choice::Offered(index)` |
| `price-reporter` | none (SD-S31d-11) | none (`rule` with default pricing) |

**The interface S31-h attaches to** (answering its R-S31d-h-1 and Q-S31h-2 from this side):

- **Module and names.** `cognition/rule-controller/src/price/` (a directory, not S31-h's assumed `price.rs`). S31-h adds
  `price/haggle.rs` with `pub fn answer_proposals(observation, pricing) -> Option<ActionRequest>` and `pub fn
  haggle_instead(observation, pricing, buy: ActionRequest) -> Option<ActionRequest>`.
- **Where its options live.** `sell.accept_at_least_permille` (its seller option) and `buy.haggle { offer_permille,
  deadband_permille }` (its buyer option), so each obeys its half's `tags:` (Q-S31d-4). Its flat names
  `pricing.haggle` and `pricing.accept_at_least_permille` change to these.
- **Order inside the bands.** `seller_band` runs `answer_proposals` first (an open proposal is answered before prices
  move), then the slot schedule. `on_chosen_buy` applies `cheapest` first, then `haggle_instead` on the result.
- **Basis.** `haggle_instead` reads the suggestion through `price::suggestion` (the same tie rule as §5.5), then the
  reference value from `wire::CurrencySeen`.
- **Requests.** S31-h builds `propose-price`, `accept-proposal` and `decline-proposal` through `wire` mirrors (`wire.rs`
  gains three), not through economy's types (F-S31d-1, Q-S31d-1).
- **If S31-h merged first,** C2 and C3 of this PR keep its file and fold its options into the halves above; if this PR
  merges first, the two calls are present as `// S31-h:` call sites that do nothing, and the decoder refuses the two
  keys until S31-h adds them (no silent key).

### 5.7 Wire mirrors and the drift guard (`price/wire.rs`; Q-S31d-1, recommended (a))

`rule-controller` may not depend on a market crate (A-13). It reads the market packs' disclosed records and builds their
requests through **mirrors**: small `pub` types in `price::wire` with the owner's labels.

| Mirror | Implements | Label | Shape (as S31-a / the owner discloses it) |
| --- | --- | --- | --- |
| `AsksSeen` | `Component` | the `asks` view's component type, owner `economy` | `{ asks: [{ item, price, available }] }` |
| `ListingSeen` | `Component` | `shop`, owner `economy` | `{ operator, listed: [{ item, price, in_stock }] }` |
| `CurrencySeen` | `Component` | `currency`, owner `economy` | `{ name, symbol, exponent, symbol_at, references: [{ item, value }] }` |
| `HoldingsSeen` | `Component` | `holdings`, owner `inventory` | `{ held: [{ item, count }] }` |
| `CatalogueSeen` | `Component` | `item-catalogue`, owner `item` | `{ kinds: [{ item, category, name }] }` |
| `SetPrice`, `ListForSale` | `Action` | `set-price`, `list-for-sale`, owner `economy` | `{ item, price, for?: "shop" }` (`for` omitted when absent) |
| `BuySeen` | decode only | `buy` | `{ item, price }` |

- Each mirror declares the owner's **schema version**; `payload_for` refuses a newer one (A-12), so an owner that bumps
  its schema makes the policy do nothing rather than guess.
- `deny_unknown_fields` on every mirror. The exact component type ids and field names are those of the merged S31-a
  code, recorded at C0 (S31-a §5.9 names the `asks` view without its id).
- **The drift guard** lives where both sides may be named: `systems/economy/tests/controller_wire.rs` (economy already
  dev-depends on `mineworld-rule-controller`, `systems/economy/Cargo.toml`). In a hand-built world it builds each
  real record with its owner's code (economy's `discloses`, inventory's `Holdings`, item's `ItemCatalogue`),
  decodes it with the mirror, and compares field by field; and it encodes each mirror request and decodes it with
  economy's own codec. A renamed field or a bumped schema turns it red (M-d-6).
- **Why this is honest, not a loophole** (the reasoning `ARC-182` records): `ARC-35` check 2 keeps a framework crate from
  *linking* the market, so the market stays removable and the controller compiles without it. A slug mirror keeps that:
  the controller still builds, and decides exactly as before, in a world without economy (the records are absent and
  the offers are not made). It is the same choice `ARC-35` item 6 made for world-level tests, plus a guard that the
  tests did not need. The alternative, amending check 2, is in Q-S31d-1.

### 5.8 The LM backend: typed fillers for incomplete offers (Q-S31d-2, Q-S31d-3)

#### 5.8.1 The idea, plainly

Today a language model may do three things: say something, attempt a numbered **complete** offer, or nothing. A
complete offer already carries its exact payload, so the model only picks. `publish-notice` is **incomplete**: the
world says "you may post", but the post's content is the poster's. A **filler** is a small typed part of the LM
controller that knows one incomplete action: when to list it, how to explain it to the model, which JSON shape the
model must answer with, how to check that answer, and how to turn it into the request payload. The model fills in
typed fields; it never writes a payload.

Example. Carol's observation offers `publish-notice` and discloses her book. The prompt gains:

```text
TO FILL
F1. publish-notice: post suggested prices on prices.ingredients. Answer "fill": {"offer": "F1", "notice":
    {"lines": [{"item": <a kind from MY PRICE BOOK>, "price": <whole cents>, "basis": <how many of your notes it
    rests on>}]}}, at most 32 lines.

MY PRICE BOOK (amounts in cents; US dollar, $ before: 300 = $3.00)
Bread: sales 300, 320, 300 (3 notes); asks Corner Store 300 (stock 1), Grace Okafor 260
Juice: sales 250, 250, 350, 350, 340 (5 notes); asks Corner Store 250 (stock 4), Café Company 350 (stock 9)
```

The model answers `{"act": "fill", "to": null, "words": "", "offer": null, "recalls": [], "fill": {"offer": "F1",
"notice": {"lines": [{"item": "Juice", "price": 300, "basis": 5}]}}}`. The filler checks that `Juice` is a kind in the
book, that 300 is within 1 … 10^12 and that 5 ≤ the 5 notes shown, then builds `{"topic": "prices.ingredients", "body":
{"prices": [{"item": "<Juice's id>", "price": 300, "basis": 5}]}, "valid_for": 21600}` and submits it through
`mineworld_sdk.offers.request` (A-26). The notices pack judges it exactly as it judges the deterministic clerk's.

#### 5.8.2 Parts (`cognition/lm-controller/src/mineworld_cognition/controller/fill/`)

```text
fill/__init__.py        FILLERS: the closed registry, action type → Filler; listed(observation, names, config)
fill/base.py            Filler (a Protocol), Fillable (what one listed filler shows), FillContent, the shared checks
fill/publish_notice.py  PublishNoticeFiller: the first filler
```

A `Filler` has, all pure (no clock, no randomness, no environment, `ARC-72`):

| Member | `publish-notice` filler |
| --- | --- |
| `action_type` | `publish-notice` |
| `listed(observation, names, config) -> Fillable \| None` | the observation offers `publish-notice` available with target none (A-26's verdict); the seat's own `survey-book` record decodes with at least one kind; `[fill.publish-notice] topic` is configured. Else `None` |
| `section(fillable) -> str` | the `TO FILL` line and the `MY PRICE BOOK` block, in labels only: kinds by the place's `item-catalogue` names (A-18), duplicates suffixed ` (2)`; a kind with no name is `kind 1`, `kind 2`… in item order; sellers by the name book, an organization as "<place label> shop"; never an entity id (step-17 I-10). Amounts in minor units, with the `currency` record's name and symbol when disclosed |
| `schema() -> dict` | `{"type": ["object", "null"], "additionalProperties": false, "required": ["lines"], "properties": {"lines": {"type": "array", "maxItems": 32, "items": {"type": "object", "additionalProperties": false, "required": ["item", "price", "basis"], "properties": {"item": {"type": "string"}, "price": {"type": "integer", "minimum": 1}, "basis": {"type": "integer", "minimum": 1}}}}}}` under the key `notice` |
| `check(content, fillable) -> FillPayload \| ContentFailure` | 1 … 32 lines (`no_lines`); each `item` a kind label shown (`unknown_item`); no label twice (`duplicate_item`); `1 ≤ price ≤ 10^12` (`price_out_of_range`); `1 ≤ basis ≤` the notes shown for that kind (`basis_out_of_range`). The payload's lines are sorted by item id |
| `payload(fill) -> (target, JSON)` | target none; `{"topic", "body": {"prices": [...]}, "valid_for": valid_minutes × 60}` |
| `due_events` | `{"day-began"}` (Q-S31d-3) |

`config` is the cognition TOML table `[fill.publish-notice]` (`topic`, required for the filler to list; `valid_minutes`,
default 360, 1 … 43 200), operator configuration, never a world file (`MODULE_SPEC.md` §5.1: the LM's settings live in
`cognition.toml`).

#### 5.8.3 What changes in the consult, and what does not

- **Nothing listed, nothing changes.** When no filler lists itself, the situation, the prompt, the schema
  (`SOCIAL_CHOICE_SCHEMA`), the parse, the validation, the request and the decision-log record are byte-identical to
  today (I-d-8). Every committed cassette and golden keeps its key (AC-d-14). That holds for every existing world: no
  world offers `publish-notice` yet.
- **Something listed.** The prompt gains one rule line in the system message (`'Use "fill" only with an F-number
  listed under TO FILL, and give exactly the fields it asks for.'`) and the `TO FILL` and filler sections in the user
  message, before `WHAT I REMEMBER`. The output schema is `SOCIAL_CHOICE_SCHEMA` with `"fill"` added to `act`'s enum and
  one required, nullable property `fill: {"type": ["object", "null"], "additionalProperties": false, "required":
  ["offer", <each listed filler's key>], "properties": {"offer": {"type": "string", "enum": ["F1", …]}, <key>: <its
  schema>}}`. Every object is closed and every property required, so the schema stays valid for strict providers
  (`backend/strict_schema.py`).
- **Parse and validate.** `parse_choice` takes the listed fillables and validates against the extended model when
  there are any. `validate` gains the `fill` row: an F-number not listed is `unknown_fill`; otherwise the filler's
  `check`. The new reasons extend `ContentReason`; `FailureRecord` reads old logs unchanged.
- **Intent.** A `Fill` proposal (`kind: "fill"`, the action type, the target, the checked payload, the recalls) becomes
  `offers.request(observation, action_type, target=, payload=)`. `NotOffered` is the semantic failure `not_offered`, as
  for `attempt`. A server refusal (`over-daily-limit`, `body-invalid`) re-plans through the existing semantic layer.
- **The consult engine** (`consult.py`) is otherwise unchanged: the same layers, budget, breaker, deadline and
  hand-back. A consult that falls back hands the seat back, and the server's fallback model (for the clerk:
  `price-reporter`) drives it (`MODULE_SPEC.md` §5.1).

#### 5.8.4 The `due` trigger (Q-S31d-3)

- **Detection.** `detect(…, due_events)` gains a third kind: for each perceived fact whose type is in `due_events`,
  `Trigger(kind="due", event=fact.id, speaker=None, words="")`. The seat runner passes the union of `due_events` over the
  fillers listed for its newest observation, so a seat that lists no filler never gets a `due` trigger.
- **Prompt.** `WHAT JUST HAPPENED` reads "A new day began." for a `day-began` trigger.
- **Cost.** One model call per world day per clerk seat (calendar states one `day-began` a day), within the existing
  budget and breaker.
- **Determinism.** The trigger carries the fact's id, so the sampling seed (`sha256("<seat>:<event>")`) and the
  cassette key are fixed for a given world (`prompt.py:56-58`, main @ 44f050e3).
- **Pending slot.** A newer trigger replaces a waiting one (`triggers.py`, `Pending`); a `due` trigger replaced by a
  `spoken_to` one is lost for that day. The replacing consult still lists the filler, so the model may still post
  (R-S31d-9).

#### 5.8.5 The registry covers the other incomplete actions (S31-h's Q-S31h-4)

Each further incomplete action is one filler module and one registry line: S31-h's `propose-price` (`{item, price}`,
target the seller), `accept-proposal` and `decline-proposal` (a proposal number from the `proposals` record), and
`withdraw-proposal`; economy's `set-price`, `list-for-sale`, `withdraw`. S31-d ships only `publish-notice`; the others
follow in S31-h or a small follow-up PR (§2.2). The registry is closed and typed, the same principle `ARC-153` kept for
MCP (QS27-9: no generic free-form act).

### 5.9 Swapping the backend: configuration only (AC-S31-11)

```yaml
# deterministic clerk
models:
  price-reporter: { parameters: { topic: prices.ingredients, first_minute: 480, last_minute: 1200, every_minutes: 240 } }
assign:
  - { seat: carol, model: price-reporter }
```

```yaml
# LM clerk: the same world; while no LM process holds Carol, her fallback is the deterministic clerk
models:
  lm: { fallback: price-reporter }
  price-reporter: { parameters: { topic: prices.ingredients, first_minute: 480, last_minute: 1200, every_minutes: 240 } }
assign:
  - { seat: carol, model: lm }
```

```toml
# cognition.toml (operator configuration, never in a world)
[fill.publish-notice]
topic = "prices.ingredients"
valid_minutes = 360
```

Both clerks submit `publish-notice { topic, body: prices, valid_for }`, judged by the same `judge_publish` (S31-b §5.7);
the notice's type, topic, publisher and audience (the subscribers at that instant plus the publisher) are the same
whatever made it. Only the lines differ. An MCP agent in the clerk's seat posts through S31-g's typed `act` kind
(R-S31g-d-1) and is judged the same way.

### 5.10 Ownership, persistence, determinism

- **Nothing new is owned.** S31-d adds no component, fact, action or record. Every effect is a request to an owner
  (`notices`, `economy`, `price-survey`) that re-validates it at dispatch (`INV-10`).
- **Persistence.** A model's requests are journaled; replay re-executes the journal and never calls a model
  (`ARC-25`). Kill-and-resume equals an uninterrupted run, because every in-process band is a pure function of
  `(parameters, seed, pace, observation)` (A-1, AC-d-8 (e)).
- **Determinism.** Integers only (u128 intermediates, floor division), `Vec` and `BTreeMap` order, no float, no
  `HashMap`, no clock; the seller band, the buyer hook and the clerk's bands take no draw.
- **The LM path** is reproducible at the request level only under cassette replay (`ARC-58`; `MODULE_SPEC.md` §5.1's
  determinism table); its world facts are timing-dependent like every hosted LM seat's.

### 5.11 Both clients and the agent view

- **2D and 3D clients:** no change. A notice reaches the board panel S31-e builds; a `price-set` or `listed` reaches
  the price tags. Clients evaluate no rule. The gate questions (`CLAUDE.md` §4): a Minecraft-like 3D client needs
  nothing new, and both clients use the same server path. Yes and yes.
- **MCP agents:** no change here. An agent may sit in the clerk's seat (agents are first-class players) and post
  through S31-g's `act` kind, judged by the same `judge_publish`; an agent seller sets its own prices through S31-g's
  kinds. `ARC-153`'s closed union is untouched by this PR (R-S31g-d-1).

### 5.12 The decision texts this PR writes (C1)

- **`ARC-116` note — the backend half (S31-d).** The clerk is a seat; its backend is the seat's decision model. The
  bundled `price-reporter` reads the book through `read::disclosed::<PriceBook>`, publishes at configured slots with
  the median-and-supply rule of §5.3, surveys when offered, and is otherwise `rule`. The LM backend reads the same book
  through the `publish-notice` filler (`ARC-183`) once per world day. Both submit the same `publish-notice` request.
  Swapping is a `decisions.yaml` line (and, for the LM, `cognition.toml`).
- **`ARC-117` — Pricing policies are decision-model parameters and models, defaulting to `hold`.** Problem (step
  §0.1 item 6, Q-S31-5), options (a third model per seat; a world rule in economy — declined: it would read trades
  nobody perceived; a shared block — chosen), the choice (§5.4 … §5.6: halves with tag filters because parameters are
  per model; the slot schedule; bounds by the reference value; the board as the only source of suggestions; the
  buyer hook), consequences (default parity; S31-h's options; F-S31d-6's keeper limit).
- **`ARC-182` — Controllers read the market packs by slug through wire mirrors that an owner-side test holds equal**
  (if Q-S31d-1 is ruled (a)), with a note under `ARC-35`: check 2 is unchanged; a framework crate still links no
  market pack; item 6's slug precedent extends to controller code, with the drift guard of §5.7.
- **`ARC-183` — The LM controller fills an incomplete offer through a typed filler from a closed registry** (if
  Q-S31d-2 is ruled as recommended), with a note under `ARC-72`: the model's answer may now also become the payload a
  filler builds from checked, typed fields; still only through `mineworld_sdk.offers`; still never movement; a filler
  may raise a `due` consult on named perceived facts.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-d-1 (I-S31-1) | No path under `kernel/`, `contracts/`, `persistence/`, `server/`, `clients/`, `worlds/`, and no `systems/*/src` changes; under `systems/`, only `systems/economy/tests/controller_wire.rs` is added | C8 diff scan |
| I-d-2 (`ARC-35` check 2) | No `cognition/` crate depends on or names a market crate; no normal path leads from `rule-controller` to one | `tests/acceptance` `check_2_the_dependency_structure` (CI); C8 scan |
| I-d-3 (default parity) | With no `pricing:` block, `rule` and `motivated` submit exactly what they submitted before, consult for consult, and take no new draw | AC-d-6; the template test |
| I-d-4 (X-6, I-S31-8) | A policy reads suggestions only from the `notice-board` record | AC-d-5 (f); X-6 |
| I-d-5 | Every price a policy proposes lies in `[max(1, ref × floor / 1000), ref × ceiling / 1000]` and ≤ 10^12; a kind with no reference value is never moved | AC-d-5 (d), (e); AC-d-8 (c) |
| I-d-6 | The clerk publishes at most once per configured slot and only from its own book | AC-d-2; AC-d-8 (a), (b) |
| I-d-7 (AC-S31-11) | Whatever backend drives the clerk, its notice is a `publish-notice` request judged by `judge_publish`; type, topic, publisher and audience rule are the same | AC-d-11 |
| I-d-8 | When no filler is listed, every LM request, schema and decision record is byte-identical to before this PR | AC-d-14 |
| I-d-9 | An LM answer reaches the world only as a `talk`, a complete offer's own payload, or a filler-built payload from checked, typed fields, through `mineworld_sdk.offers` | AC-d-13; review of `intent.py` |
| I-d-10 | The mirrors equal their owners' types and schema versions | AC-d-7 |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt, clippy `-D warnings` (with `float_arithmetic` denied in `price-reporter`), ruff, pyright | CI `fast` |
| Unit (Rust, W1 observations) | the block's decoding at key and value; `suggest`; slots; each policy; the buyer hook; the bands' order | `cognition/rule-controller/src/price_tests.rs`; `cognition/price-reporter/src/tests.rs` |
| Contract | the mirrors against the owners' real types | `systems/economy/tests/controller_wire.rs` |
| Python unit | the registry, the filler's listing, section, schema, check and payload; the `due` trigger; byte-identity of unlisted requests | `cognition/lm-controller/tests/test_fill_*.py`, `test_price_reporter_replay.py` |
| Real binary (deterministic) | AC-d-4, AC-d-8, AC-d-9, AC-d-10 | `tools/cli/tests/price_reporter.rs` |
| Real binary (LM, python layer) | AC-d-11 | `tools/cli/tests/price_reporter_lm.rs` + `cognition/lm-controller/tests/price_reporter_cognition.py` |
| Long | AC-d-6 (b): every reference baseline unchanged, from the PR's CI `parity` record or the milestone run | CI |
| Gate 1 (a real LM) | NOT REQUIRED: the LM half is checked with a scripted model and a cassette recorded from it (no cost). An operator-approved local-model recording is a later, separate step | — |
| Gate 2 (a real client) | NOT REQUIRED: no client change | — |

Oracles are literal books, boards, asks and times written in each test, facts read back from the save by slug
(`ARC-35` item 6), and a mirror of §5.3 written out in the test; never a value the module reports about itself (test
rules §25).

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`)

**Worlds.**

- **W1 (hand-built observations, Rust unit tests).** `Observation`s built with literal records: Carol's `survey-book`,
  `notice-board` notices, the store's `shop` listing, `asks` views, `holdings`, `currency` (US dollar, exponent 2) and
  the place's `item-catalogue` (bread, apple, juice: `food`/`drink`; scarf, pen: `goods`). **W1b** is the same set with
  every amount divided by 100 (rounded up) and the currency `{ name: crown, symbol: cr, exponent: 0, symbol_at: after }`.
- **W2 (deterministic checkpoint).** A scratch copy of `worlds/market-town` from the C0 base (`market_composition.rs`'s
  copy pattern), with only these edits, asserted by a file diff: `systems:` appends `notices`, `price-survey`;
  `configure:` appends `notices`; `configure/notices.yaml` declares `prices.ingredients` (`prices`, publishers
  `[carol]`, `valid_for` 1h … 48h, `per_day` 4); `people/carol.yaml` appends `price-survey: { kinds: { categories:
  [food, drink] } }`; `people/alice.yaml` and `people/felix.yaml` append `notices: { subscribe:
  [prices.ingredients] }`; both shops gain `keepers` (`alice` for the café, `felix` for the store); the 10 food and
  drink item files gain `economy: { reference: <their step-10 shop price> }` and juice's is 250; **the café also asks
  juice 350 and holds 10 juice** (F-S31d-9). Decisions file (test-local, `--decisions`): Carol on `price-reporter`
  (topic `prices.ingredients`, 08:00 … 20:00 every 240, valid 360); `rule` with `sell: { policy: toward-suggestion }`
  (the other keys default).
- **W3 (second world).** W2 with: currency `{ name: euro, symbol: "€", exponent: 2, symbol_at: after }`; the topic
  renamed `market.daily` with publisher `hana`; the survey section and subscriptions moved accordingly (Hana surveys;
  Alice and Felix subscribe to `market.daily`); Hana's `08:00` routine segment moved from `workplace` to `store`, so she
  sees the store's juice in the morning and the café's from 15:00 (A-20); Hana on `price-reporter` (topic
  `market.daily`, 16:00 … 20:00 every 120, valid 840, so the 20:00 notice is still current when the keepers are back at
  their counters in the morning); **Alice on `motivated`** with the same `sell:` block; Felix on `rule`.
- **W4 (removal variants of W2).** (a) `notices` removed with its configuration and sections; (b) `price-survey`
  removed with Carol's section; (c) the decisions file without any `pricing:` block.
- **W5 (Social Café).** `worlds/social-cafe` as on the base.
- **W6 (the swap).** W2 with Carol and the visitor placed in the store at genesis (their `location:` lines). **W6b**
  is the same edit on W3: Hana and the visitor placed in the store, topic `market.daily`, the euro.
- **W7 (Lakeside).** `worlds/lakeside` as on the base: no economy, no notices; a second world for delegation.

Every criterion below names at least one world or variant besides Market Town's copies (charter, "no world-specific
hard-coding"): W1b, W3, W5, W6b or W7, or a second hand-built fixture.

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-d-1 | **The suggestion rule (W1, W1b).** (a) bread sales 300, 320, 300, stock median 1 → `{bread, 330, 3}`; (b) sales 300, 310, 320, 330, no stock → 315, basis 4; (c) one sale; asks corner-store@store 300 with stock 5 (and an older 290, ignored), grace@store 260, cafe-company@cafe 280 with stock 9 → 280, basis 3; (d) two sales and two asks → no line; (e) stock median 25 → 300 × 950 / 1000 = 285; (f) base 1 with plenty → 1; (g) 33 qualifying kinds → 32 lines, the smallest-basis kind dropped (ties: higher id dropped), sorted by item; (h) W1b: sales 3, 3, 4, scarce → 3 | §9.4 |
| AC-d-2 | **Publishing slots (W1).** Over one day of consult instants `t = k + m·900` for seat offsets `k = 0` and `k = 3`: exactly four `publish-notice` requests, at the consults holding 08:00, 12:00, 16:00 and 20:00; none when the offer is absent or unavailable, none without a book, none when `suggest` is empty. Each request decodes with notices' `PublishNotice`: topic, lines sorted by item, `valid_for` 21 600. Variant: `first 960, last 1200, every 120` → 16:00, 18:00, 20:00 only | §9.4; R-S31d-b-1 |
| AC-d-3 | **The survey band (W1, W1b).** `survey` offered complete and available → the request equals the affordance's own; offered refused (`surveyed-recently`) or absent → the consult equals `rule`'s | S31-c §5.10 item 6 |
| AC-d-4 | **Delegation (W5, W7).** Social Café and Lakeside, seed 7, 10 days each, every seat on `price-reporter`: facts, journal and snapshots equal the same run with every seat on `rule`, byte for byte. The template test (A-5) passes with the new model installed (AC-6) | AC-6 |
| AC-d-5 | **The policies (W1, W1b).** (a) `hold`: no request in any observation of the suite; (b) toward: `set-price {bread, 303, for: shop}` at bread's index only; (c) suggestion 309 → nothing; (d) ask 590, suggestion 2 000 → 600; at 600 → nothing; (e) no reference → nothing; (f) **X-6:** the board is empty and the observation's events hold an expired `notice-published` with bread 330 → nothing; (g) two current notices on bread → the newest `issued_at`; a tie → the lower publisher id; (h) scarcity: shop `in_stock` 1 → 330, 12 → 270; an own ask with 0 held → raised; (i) list `[goods]`: scarf listed at 2 500 at index 0; apple (food) and a goods kind without a reference not listed; (j) `sell.tags: [keeper]` and Felix untagged → nothing; (k) keeper inference: listing, `set-price` offered, no own asks → shop request; own asks present → only own asks; not in the shop → nothing; (l) `buy.choose: cheapest` → Grace's 260 instead of the store's 300; a tie → the first in observation order; `any` → unchanged; `buy.tags` respected; (m) (b) and (l) again through `motivated` | §10.1; X-6 |
| AC-d-6 | **Default parity.** (a) `rule: {}` and `motivated` without `pricing` decode to defaults; `rule: { x: 1 }` is refused at `x` naming `pricing`; each bound of §5.4 is refused at its value; (b) every reference world's baselines (Social Café, Lakeside, bodies-yard, Market Town) in the milestone's `parity` record equal `scripts/baselines.txt` on main; (c) the template test (A-5) passes | R-S31-5; AC-S31-9 |
| AC-d-7 | **Mirrors equal their owners (contract).** Each mirror of §5.7 decodes its owner's real record built by the owner's code in two hand-built worlds (a shop world in US dollars; a world with a crown of exponent 0, a person seller's `asks` view and no shop), field for field; `SetPrice` and `ListForSale` encode to payloads economy's codec decodes; the schema versions are equal | Q-S31d-1 |
| AC-d-8 | **The checkpoint (W2, seed 7, 10 days).** (a) at least 10 `notice-published` by Carol on `prices.ingredients`, and no refused `publish-notice` in the journal; (b) every line of every notice equals the test's own recomputation from Carol's book, rebuilt by folding her `price-noted` facts up to that instant (S31-c AC-c-8's mirror) with §5.3's rule written out in the test; (c) at least one keeper `price-set` (Alice or Felix) moves toward the suggestion current at that instant (computed from the facts), none moves away, and every `to` lies within the bounds of I-d-5; (d) no keeper `price-set` is explained only by a notice whose `valid_until ≤` its instant; (e) the run twice, and killed at day 5 and resumed, gives byte-identical facts, journal and snapshots | §16.2 checkpoint; AC-6 |
| AC-d-9 | **Second world (W3, seed 7, 10 days).** (a) at least 5 notices by Hana on `market.daily`; (b) as AC-d-8 (b); (c) at least one `price-set` by Alice (on `motivated`) toward the current suggestion, none away, all bounded | no hard-coding |
| AC-d-10 | **Removable (W4, 5 days each; (a) also on W3).** (a) no `notice-published`, no `price-set` from a suggestion, at least one `sold`; (b) no `notice-published`, at least one `sold`; (c) no `price-set`, `listed` or `withdrawn` by any seat after genesis | AC-S31-12; I-S31-10 |
| AC-d-11 | **Backend swap (W6; AC-S31-11).** Arm D: `mineworld run` 2 days with Carol on `price-reporter`. Arm L: `mineworld server <W6> --save S --solo --time-scale 1440` with Carol on `lm` (fallback `price-reporter`) and the scripted reporter process (§9 C7) holding her seat; a protocol client on the visitor's seat buys bread three times beside Carol; the test waits ≤ 150 s wall for Carol's first notice. Then: L's notice and D's notices have the same event type, topic and publisher; each audience equals the topic's subscribers at that instant plus Carol (from the save's subscription facts); L's body is `prices`, and every line's item is a kind in Carol's book at that instant; the request's actor is Carol's seat, submitted by the cognition process (journal); the two arms' inputs differ only in the decisions file (the world directory's hash is equal). The same two arms run on W6b (Hana, `market.daily`, the euro) with the same assertions | AC-S31-11 |
| AC-d-12 | **The cassette (offline).** `tests/cassettes/price-reporter.jsonl`, recorded once from the scripted reporter through `RecordingBackend`, replays two fixture `due` consults strictly (one US-dollar clerk on `prices.ingredients`; one crown clerk with exponent 0 on `market.daily` whose kinds have no catalogue names, so the labels are `kind 1`, `kind 2`): the same request keys, the same filled request as the committed expected one; a changed prompt line is `CassetteMiss`, never a fallback | AC-S31-11 "cassette replay"; `ARC-58` |
| AC-d-13 | **The filler's checks (both fixtures of AC-d-12).** Each content failure by name: `unknown_fill`, `no_lines`, `unknown_item`, `duplicate_item`, `price_out_of_range` (0; 10^12 + 1), `basis_out_of_range` (0; one more than shown); a world refusal re-plans; `NotOffered` is `not_offered`. A seat whose observation lists no filler gets no `due` trigger and no fill schema | step §9.4 "validated identically"; I-d-9 |
| AC-d-14 | **Nothing else in the LM moved (Social Café's Milestone D run and FG-c's memory stream).** `SOCIAL_CHOICE_SCHEMA`'s golden, `tests/fixtures/prompt_request.json`, `test_milestone_d_replay.py` and `test_lm_summarise_replay.py` pass unchanged; for every existing fixture observation, `build_request` is byte-identical to the base's | I-d-8 |
| AC-d-15 | **Structure (build-wide: it holds for every world, because no world is read).** `check_2_the_dependency_structure` passes; I-d-1's diff scan is empty; `cognition/price-reporter/Cargo.toml` names no market crate | I-S31-1; `ARC-35` |

### 8.1 Planted mutations (each must turn its test red by name, then be restored from a byte copy)

| Id | Plant | Must fail |
| --- | --- | --- |
| X-6 | `toward-suggestion` reads the observation's perceived `notice-published` events instead of the board | AC-d-5 (f); AC-d-8 (d) at the milestone |
| M-d-1 | `suggest` takes the mean | AC-d-1 (b), (c) |
| M-d-2 | `suggest` keeps every ask note (no dedup) | AC-d-1 (c) |
| M-d-3 | `in_window` closes both ends (`t − pace ≤ s ≤ t`) | AC-d-2 (offset 0: a slot fires twice) |
| M-d-4 | the deadband is ignored | AC-d-5 (c) |
| M-d-5 | the bounds are ignored | AC-d-5 (d); I-d-5 |
| M-d-6 | `ListingSeen` renames `in_stock` | AC-d-7 |
| M-d-7 | the filler admits an item not shown in the book | AC-d-13 (`unknown_item`) |
| M-d-8 | the fill property is added to every request's schema, listed or not | AC-d-14 |
| M-d-9 | the clerk publishes on every consult in a slot's hour | AC-d-2; AC-d-8 (a) (refused `over-daily-limit` in the journal) |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]` needs the
work and recorded evidence; an inapplicable item is `N/A` with the audited reason. The implementer of each commit is
named in its heading.

### 9.0 The procedure for every commit (Haiku implementer: follow it literally)

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s31d`. Never `cd` elsewhere.

1. Re-read this commit's section and every design section it cites. Re-read every file you will edit, in full, before
   editing it.
2. Make the numbered steps, in order. Do not do a later commit's work. If a step names a function, a file, an id or a
   field that does not exist on the base, stop and record it (§12 material stops). Do not invent one.
3. **Per-commit validation is light (speed first):** build the touched crates and run the unit or Python test file
   the commit itself writes. Record per command: test count, wall time, PASS / FAIL / INCONCLUSIVE. The
   "Validation" lines below name everything a commit's tests cover; the real-binary files (`price_reporter.rs`,
   `price_reporter_lm.rs`), `check_2_the_dependency_structure` and the reference check are **written** in their
   commit and **run at the milestone**, except that a commit must still compile them.
4. Mutations listed for a commit are **run at the milestone**: `cp f f.orig`, plant, run the named test, record the
   red (test name, assertion), `cp f.orig f`, `cmp f f.orig` clean, delete the copy, run green again.
5. Static: `cargo fmt --all --check`; `cargo clippy -p <touched crate> --all-targets -- -D warnings`; for Python,
   `uv run --locked ruff check <files>` and `uv run --locked pyright <files>`. Markdown:
   `python3 scripts/check_decision_ids.py` (`--write-index` after adding a record). **R-LOAD-1:** never `cargo test
   --workspace`, never all of `tools/cli`; one `--test <file>` at a time; any `tools/cli` run through
   `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.
6. Review: answer the commit's review questions in one or two sentences each, in §15.
7. `git status --short` and `git diff --stat`: every changed path is in the commit's **Files**; otherwise revert it or
   stop and record why.
8. Update §15 and `handoff-s31d.md` in the same commit.
9. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` with the session's attribution line. No
   heredoc writes, no `sed -i`, no `perl -pi`, no `python3 -c`, no `curl`.
10. Stop and report a material stop (§12) instead of improvising.

#### C0 — Base, prerequisites, ledger (Markdown only) — Haiku

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s31d.md` (new).
- **Steps:**
  - [ ] 1. Record `git rev-parse origin/main`. Quote the dispatch that confirms Milestone F's checklist was handed over
    and this design frozen; either missing → material stop.
  - [ ] 2. Confirm S31-a, S31-b and S31-c merged (`git log origin/main --oneline`, and
    `systems/{economy/src/sale.rs, notices/src/lib.rs, price-survey/src/lib.rs}` exist). Any missing → material stop.
  - [ ] 3. Record the real names: the `asks` view's component type id and schema version; the `currency` record's id,
    fields and version; `shop`'s listing fields and version; `holdings`' and `item-catalogue`'s fields and versions;
    `set-price`, `list-for-sale`, `buy` payload fields; `PriceBook`'s path and accessors; `Survey`; `PublishNotice`,
    `NoticeBoardRecord`, `PriceLine`, `TopicId`, `NOTICE_PRICE_MAX`. A shape §5 cannot read → material stop.
  - [ ] 4. Record whether S31-h and WE-a merged; choose §2.3's branch for each; for WE-a, record how a shop's active
    modifier shows in an observation, if at all (R-S31d-4).
  - [ ] 5. Record the rulings of §14.1 and which C2/C3/C6 branch they select.
  - [ ] 6. Re-read A-1 … A-33 on the base; record moved line numbers; stop if a statement changed.
- [ ] Implementation. [ ] Validation: `python3 scripts/check_decision_ids.py`. [ ] Review: §2.3 holds on the base.
- **Commit message:** `docs(plan): S31-d — open the ledger at the base`

#### C1 — Decisions and the specification first — Haiku

- **Goal:** the decisions exist before the code they govern (`CLAUDE.md` §2.2).
- **Files:** `docs/decisions/ARC-117.md` (new), `docs/decisions/ARC-116.md` (the note), and as ruled
  `docs/decisions/{ARC-182.md, ARC-183.md}` (new) with notes in `docs/decisions/{ARC-35.md, ARC-72.md}`;
  `docs/decisions/INDEX.md` (regenerated); `docs/MODULE_SPEC.md` §5.1.
- **Steps:**
  - [ ] 1. Write each record in the log's format (copy the headings of `docs/decisions/ARC-153.md`), from §5.12's
    texts and the sections they cite. Notes are `## <ID> note — <title> (2026-MM-DD, S31-d)` headings at the end of the
    record's file.
  - [ ] 2. `MODULE_SPEC.md` §5.1: the registration example gains `PriceReporter =>
    mineworld_price_reporter::PriceReporter`; a paragraph "The bundled clerk, `price-reporter`" (§5.2's parameters and
    bands); the `pricing:` block under `rule` and `motivated` (§5.4, linking `ARC-117`); in the LM Controller bullet,
    one sentence on fillers and the `due` trigger (linking `ARC-183`).
  - [ ] 3. `python3 scripts/check_decision_ids.py --write-index`.
- **Validation:** `python3 scripts/check_decision_ids.py`; `python3 scripts/check_doc_headings.py`.
- [ ] Implementation. [ ] Validation. [ ] Review: no new ontology term (`CLAUDE.md` §2.1 rule 3: "clerk", "filler",
  "slot" are pack and controller vocabulary); every code and key of §5 appears in its record; nothing contradicts §11.
- **Commit message:** `docs: pricing policies and the price clerk's backends — ARC-117, ARC-116 note (S31-d)`

#### C2 — The `pricing:` block, the wire mirrors, the drift guard — Haiku

- **Goal:** `rule` and `motivated` accept the block and decide exactly as before; the mirrors exist and are held equal.
- **Files:** `cognition/rule-controller/{Cargo.toml, src/lib.rs, src/model.rs, src/paced.rs, src/offered.rs,
  src/price/{mod.rs, block.rs, wire.rs, slots.rs}, src/price_tests.rs}`; `cognition/motivated/src/{parameters.rs,
  model.rs, options.rs, parameters_tests.rs}`; `systems/economy/tests/controller_wire.rs` (new); `Cargo.lock`;
  `docs/reference/{decisions.md, schema/decisions.schema.json}` as regenerated.
- **Steps:**
  - [ ] 1. `price/block.rs`: the types and the hand decoder of §5.4 (copy the decoder pattern of
    `cognition/motivated/src/parameters.rs`); doc comments on every field.
  - [ ] 2. `price/slots.rs`: `in_window`, `publish_due`, `period_index` (§5.3), `pub`.
  - [ ] 3. `price/wire.rs`: the mirrors of §5.7 with C0's recorded ids, fields and versions; `pub`.
  - [ ] 4. `price/mod.rs`: `seller_band` and `on_chosen_buy` returning `None` / the input for every policy (the
    policies arrive in C3); `suggestion` (§5.5).
  - [ ] 5. `model.rs`: `RuleParameters { pricing }` with `ModelParameters` (one key); `Rule::decide` →
    `decide_priced`. `paced.rs`: `decide_priced` with the seller band at §5.6's place; `decide` delegates with the
    default. `offered.rs`: `attempt(observation, draw, pricing)` passes the drawn affordance through `on_chosen_buy`.
  - [ ] 6. `motivated`: the `pricing` key in `KEYS` and the decoder; the call sites of §5.6.
  - [ ] 7. `systems/economy/tests/controller_wire.rs`: AC-d-7 (copy the world-building helpers of
    `systems/economy/tests/support/mod.rs`).
  - [ ] 8. Regenerate the reference: `target/debug/mineworld reference --out docs/reference` (after `cargo build -p
    mineworld-cli` through `with-slot.sh`).
- **Validation:** `cargo test -p mineworld-rule-controller --lib` (AC-d-6 (a)); `cargo test -p mineworld-motivated --lib`;
  `cargo test -p mineworld-economy --test controller_wire` (AC-d-7; M-d-6); `cargo test -p mineworld-acceptance --test
  ac1_composability check_2_the_dependency_structure` (AC-d-15); `target/debug/mineworld reference --check`.
- [ ] Implementation: steps 1–8. [ ] Validation, M-d-6. [ ] Review: no market crate named under `cognition/`
  (`git grep -n "mineworld[-_]\(economy\|item\|inventory\)" cognition/` empty); no draw added (`git grep -n "below(\|
  offset(" cognition/rule-controller/src/price` empty); `offered.rs`'s constants unchanged.
- **Commit message:** `feat(rule, motivated): a pricing: block, default hold, read through market wire mirrors (S31-d)`

#### C3 — The policies — Haiku

- **Goal:** `toward-suggestion`, `scarcity`, `list` and `cheapest` behave as §5.5 says.
- **Files:** `cognition/rule-controller/src/price/{mod.rs, sell.rs (new), buy.rs (new)}`,
  `cognition/rule-controller/src/price_tests.rs`, `cognition/motivated/src/tests.rs`.
- **Steps:**
  - [ ] 1. `sell.rs`: the priceable list (own asks, then the kept shop's, F-S31d-6), the index rule, the three
    policies, the bounds, `list` at index 0 (§5.5).
  - [ ] 2. `buy.rs`: `on_chosen_buy` (§5.5).
  - [ ] 3. `price_tests.rs`: AC-d-5 (a)–(l) and AC-d-2's slot arithmetic for `period_index`, on W1 and W1b;
    `motivated/src/tests.rs`: AC-d-5 (m).
- **Validation:** `cargo test -p mineworld-rule-controller --lib price`; `cargo test -p mineworld-motivated --lib`.
  Mutations X-6, M-d-4, M-d-5.
- [ ] Implementation: steps 1–2. [ ] Validation: step 3, X-6, M-d-4, M-d-5. [ ] Review: the board is the only source
  of suggestions; every proposed price passes the bound function; no float, no `HashMap`.
- **Commit message:** `feat(rule, motivated): sellers move toward current suggestions within bounds; buyers may choose the cheapest (S31-d)`

#### C4 — The clerk, `price-reporter` — Haiku

- **Goal:** the deterministic clerk exists, is installed, and decides as `rule` where it has nothing to report.
- **Files:** `cognition/price-reporter/{Cargo.toml, clippy.toml, README.md, src/{lib.rs, model.rs, parameters.rs,
  suggest.rs, tests.rs}}`; root `Cargo.toml` (`members`, `[workspace.dependencies]`); `cognition/installed/{Cargo.toml,
  src/lib.rs}`; `tools/cli/tests/packs.rs`; `Cargo.lock`; `docs/reference/*` as regenerated.
- **Steps:**
  - [ ] 1. The crate (§5.2's dependencies, each with its comment; `[package]` shape copied from
    `cognition/motivated/Cargo.toml`; `clippy.toml` copied from `cognition/motivated/clippy.toml`).
  - [ ] 2. `parameters.rs`: `ReporterParameters` and its hand decoder (§5.2's table).
  - [ ] 3. `suggest.rs`: §5.3, with `median` as a private function.
  - [ ] 4. `model.rs`: `PriceReporter` and the three bands of §5.2, using `price::slots::publish_due`.
  - [ ] 5. Install: `cognition/installed` (one line each, after `Motivated`); `packs.rs` expects
    `["mineworld-rule-controller", "mineworld-motivated", "mineworld-price-reporter"]`.
  - [ ] 6. `tests.rs`: AC-d-1, AC-d-2, AC-d-3 on W1 and W1b.
  - [ ] 7. Regenerate the reference.
- **Validation:** `cargo test -p mineworld-price-reporter`; `cargo test -p mineworld-installed-models`;
  `with-slot.sh cargo test -p mineworld-cli --test packs`; `target/debug/mineworld reference --check`. Mutations M-d-1,
  M-d-2, M-d-3, M-d-9.
- [ ] Implementation: steps 1–5, 7. [ ] Validation: step 6, M-d-1, M-d-2, M-d-3, M-d-9. [ ] Review: no market crate
  named; no draw outside `rule`'s; the publish band never fires twice for one slot.
- **Commit message:** `feat(cognition): price-reporter, the price clerk that surveys and publishes suggested prices (S31-d)`

#### C5 — The deterministic checkpoint — Haiku

- **Goal:** AC-d-4, AC-d-8, AC-d-9, AC-d-10 on the real binary.
- **Files:** `tools/cli/tests/price_reporter.rs` (new).
- **Steps:**
  - [ ] 1. `copy_world(variant) -> Scratch` for W2, W3, W4 (a)–(c), W6: copy Market Town from the base (copy the
    `copy_dir` helper from `market_composition.rs` into this file), apply the literal edits of §8, assert by a file
    comparison that nothing else differs; write each decisions file as literal text.
  - [ ] 2. A slug reader of the save's facts (the `client_2d.rs` and `market/mod.rs` patterns): `notice-published`,
    `price-noted`, `price-set`, `sold`, `listed`, `withdrawn`, `subscribed`, `subscriptions-seeded`; no market or S31
    crate named (`ARC-35` item 6).
  - [ ] 3. `delegation_equals_rule` (AC-d-4, W5).
  - [ ] 4. `clerk_publishes_and_keepers_follow` (AC-d-8 (a)–(d), W2), with §5.3's rule written out in the test as the
    oracle; `same_history_twice_and_across_a_resume` (AC-d-8 (e)).
  - [ ] 5. `second_world` (AC-d-9, W3).
  - [ ] 6. `removable` (AC-d-10, W4 (a)–(c)).
- **Validation:** per PR, the file compiles (`with-slot.sh cargo test -p mineworld-cli --test price_reporter
  --no-run`); the run itself and X-6 on the binary path are at the milestone (speed first).
- [ ] Implementation: steps 1–6. [ ] Validation. [ ] Review: the floors (10, 1, 5, 1) were written before the first
  run and are not tuned after it (`ARC-23`); every oracle is the facts or the test's own mirror.
- **Commit message:** `test: the clerk publishes from its book and keepers follow current suggestions, in two towns (S31-d)`

#### C6 — The LM filler path — Opus

- **Goal:** an LM seat can fill `publish-notice` from its book, once a world day, and nothing else in the LM moves.
- **Files:** `cognition/lm-controller/src/mineworld_cognition/controller/{fill/__init__.py, fill/base.py,
  fill/publish_notice.py, choice.py, validate.py, intent.py, prompt.py, situation.py (only if the fillables travel
  there; prefer `ConsultInput`), triggers.py, consult.py, seat.py, log.py}`, `config.py`; `cognition/lm-controller/tests/
  {test_fill_registry.py, test_fill_publish_notice.py, test_price_reporter_replay.py, fixtures/reporter_observation.json,
  fixtures/fill_choice.schema.json, cassettes/price-reporter.jsonl}`; `docs/reference/cognition-toml.md` as
  regenerated.
- **Steps:**
  - [ ] 1. `fill/base.py` and `fill/__init__.py`: the `Filler` protocol, `Fillable`, the closed registry (§5.8.2).
  - [ ] 2. `fill/publish_notice.py`: listing, section, schema, checks, payload, `due_events` (§5.8.2), reading the
    `survey-book`, `item-catalogue`, `currency` records with the real JSON encodings (F-S31d-11).
  - [ ] 3. The consult changes of §5.8.3 and the trigger of §5.8.4, each guarded so that an unlisted situation takes the
    old code path byte for byte.
  - [ ] 4. `config.py`: `[fill.publish-notice]` (`topic`, `valid_minutes`); regenerate the cognition TOML reference.
  - [ ] 5. Tests: AC-d-13 and AC-d-14 (`test_fill_*.py`); the fixture observation (§5.10 JSON of S31-c with the real
    encodings); the schema golden.
  - [ ] 6. The cassette: a scripted reporter backend (a pure function of the request: for each kind in `MY PRICE BOOK`,
    the median of the sale prices shown, basis the notes shown); record once with `PRICE_REPORTER_RECORD=1` through
    `RecordingBackend(binding="local", model="scripted")`; commit it; `test_price_reporter_replay.py` replays it strictly
    (AC-d-12), FG-c's pattern (A-29).
- **Validation:** `uv run --locked pytest cognition/lm-controller/tests/test_fill_registry.py
  cognition/lm-controller/tests/test_fill_publish_notice.py cognition/lm-controller/tests/test_price_reporter_replay.py
  cognition/lm-controller/tests/test_milestone_d_replay.py cognition/lm-controller/tests/test_lm_summarise_replay.py
  cognition/lm-controller/tests/test_controller_consult.py`; ruff; pyright; the cognition reference check. Mutations
  M-d-7, M-d-8.
- [ ] Implementation: steps 1–4, 6's recording. [ ] Validation: step 5, the replay, M-d-7, M-d-8. [ ] Review: no entity
  id in any prompt; the model never writes a payload; every request goes through `mineworld_sdk.offers`; no network
  client is built in tests (`recording.mode` is `scripted` or `replay`).
- **Commit message:** `feat(cognition): an LM seat fills an incomplete offer through a typed filler; the price clerk's first (S31-d)`

#### C7 — The backend swap on the real binary — Opus

- **Goal:** AC-d-11: the same world, two backends, one kind of notice.
- **Files:** `tools/cli/tests/price_reporter_lm.rs` (new); `cognition/lm-controller/tests/price_reporter_cognition.py`
  (new: the shipped controller with C6's scripted reporter model, as `milestone_d_cognition.py` is for Milestone D).
- **Steps:**
  - [ ] 1. Arm D: `mineworld run <W6> --headless --seed 7 --days 2 --save D --decisions <deterministic>`; read Carol's
    notices.
  - [ ] 2. Arm L: `mineworld decisions <W6> --decisions <lm> --json > seats.json`; `mineworld server <W6> --save L --solo
    --time-scale 1440 --decisions <lm>`; start `price_reporter_cognition.py --config cognition.toml` (scripted mode, the
    sink pattern of `milestone_d.rs:186-220`, `[fill.publish-notice] topic = "prices.ingredients"`); a protocol client on
    the visitor's seat submits three complete `buy` offers for bread; wait ≤ 150 s wall for Carol's `notice-published`;
    SIGKILL both; read the save.
  - [ ] 3. The comparisons of AC-d-11, with the world directory's hash taken before and after both arms.
  - [ ] 4. `#[ignore = "needs the Python workspace (uv sync --locked): run by the python layer"]`, as `milestone_d.rs`.
- **Validation:** per PR, the file compiles (`--no-run` through `with-slot.sh`) and the scripted process passes ruff
  and pyright; the python layer runs it at the milestone (`scripts/ci_layer.py`).
- [ ] Implementation: steps 1–4. [ ] Validation. [ ] Review: no model is reachable (the sink accepts nothing); the only
  differing input is the decisions file; the notice's audience is read from subscription facts, not from the pack.
- **Commit message:** `test: the price clerk on a scripted LM and on price-reporter post the same kind of notice (S31-d)`

#### C8 — Close-out — Haiku (Opus reviews C6/C7 evidence)

- **Files:** this file (§15), `handoff-s31d.md`, `cognition/price-reporter/README.md`, `cognition/rule-controller/README.md`
  (one paragraph on `price`).
- **Steps:**
  - [ ] 1. READMEs: what the clerk does, its parameters, how to swap it for `lm` (§5.9), links to `ARC-116`, `ARC-117`,
    `ARC-183`.
  - [ ] 2. AC-d-15's scan: `git diff --name-only origin/main...HEAD` lists nothing under I-d-1's forbidden paths.
  - [ ] 3. Fill §8's evidence; push `mvp0/pr-s31d`; open the PR (body ends with the Claude Code line); CI `fast` green on
    the exact head; repair to green.
- [ ] Implementation. [ ] Validation: `fast` green on the exact head. [ ] Review: deviations recorded; §16 listed for the
  lane lead.
- **Commit message:** `docs(plan): S31-d close-out`
- **Then:** MERGE-READY: report the PR number and head SHA. The coordinator merges; the implementation session never
  does. The milestone runs §8 and §8.1.

---

## 10. Requirements this PR places on other PRs (cross-PR interfaces)

- **R-S31a-d-1 (on S31-a's implementation).** The `asks` view, the `currency` record, the `shop` listing and the
  `set-price` / `list-for-sale` / `buy` payloads keep the shapes S31-a §§5.3, 5.5, 5.9 state; C0 records their real ids.
  A later change to any of them updates `price::wire` and `controller_wire.rs` in the same PR (the guard turns red
  otherwise).
- **R-S31c-d-1 (on S31-c).** `PriceBook` derives `Deserialize` and implements `Component` (`survey-book`), with §5.10's
  accessors; `Survey` is `pub`; the codes are `pub const`. §5.10 item 4's JSON example uses the contract's real id
  encodings (F-S31d-11).
- **R-S31h-d-1 (on S31-h).** Place the options under the halves (`buy.haggle`, `sell.accept_at_least_permille`); put
  `answer_proposals` and `haggle_instead` in `price/haggle.rs`; call them at §5.6's places; build its requests through
  `price::wire` mirrors (three more), not economy's types; per-model configuration (Q-S31d-4) replaces its per-seat
  example; its LM rows arrive as four fillers in the registry of §5.8 (not "with no further change").
- **R-S31f-d-1 (on S31-f).** Market Town's clerk uses `price-reporter` with the topic S31-b fixes; keepers carry a tag
  if the world wants only them to price (`sell.tags`); a second seller of at least one kind (or the supply signal)
  gives the suggestions something to move toward (F-S31d-9); the composition needs `ARC-35` check 3 to admit `notices`
  and `price-survey` and their sections (F-S31d-10).
- **R-S31g-d-1 (on S31-g).** Typed `act` kinds for `publish-notice` (and `subscribe`, `unsubscribe`, the economy seller
  actions) under `ARC-153`, each one `Player` operation; S31-d adds none.
- **R-S31e-d-1 (on S31-e).** Nothing new: the clerk's notices and the keepers' `price-set` already reach the board panel
  and the price tags.

## 11. Bounded decisions (recorded; not operator questions)

| Id | Decision | Why |
| --- | --- | --- |
| SD-S31d-1 | The clerk is a separate model that delegates daily life to `rule` | step §9.4; `motivated`'s precedent (SD-DLb-1); AC-d-4 proves the delegation |
| SD-S31d-2 | Slots are the half-open consult window `(t − pace, t]` | the rule's answer window (A-21); stateless and exactly once |
| SD-S31d-3 | A refused publish is not retried within its slot | one journaled refusal, not a burst; the next slot tries again |
| SD-S31d-4 | Sales take precedence over asks; asks are deduped per (seller, place) to the newest note | S31-c §5.10 item 5; a sale is a price someone paid |
| SD-S31d-5 | Lines are capped at 32 by largest basis | S31-b's `PRICE_LINES_MAX`; the best-founded lines survive |
| SD-S31d-6 | A keeper is inferred from `set-price` being offered while the seat has no own asks | F-S31d-6; no economy change |
| SD-S31d-7 | A kind with no reference value is never repriced or listed | the bounds need it (step §10.1 "× reference") |
| SD-S31d-8 | One request per consult; each priceable ask has its own index in the period | a model returns at most one request (A-1) |
| SD-S31d-9 | The buyer default is named `any` (today's draw), not `nearest` | F-S31d-5 |
| SD-S31d-10 | `sell.list` is a category list in data, empty by default | F-S31d-8; no category name in code |
| SD-S31d-11 | The clerk seat has no pricing block | a clerk that also sells is not an S31 need; a world wanting both uses another model later |
| SD-S31d-12 | The LM's `valid_for` and topic come from `cognition.toml`, not from the model | the model judges content, not policy; fewer failure modes |
| SD-S31d-13 | The fill schema and sections appear only when a filler is listed | existing cassettes and goldens keep their keys (I-d-8) |
| SD-S31d-14 | `ARC-117` in its own file; the `ARC-116` backend half as a note in S31-c's record | #258's one-file-per-record rule; S31-c §2.2 |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S31-d — the price clerk's deterministic model, the pricing policies,
                         and the LM clerk's filler path
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s31d-reporter-policies.md
RELATED / BINDING DOCS:  step-31-wealth-prices.md (§§0, 1, 3, 9, 10, 13, 14, 15, 16.2, 16.5, 18, 19);
                         pr-s31a-currency-asks.md (§§5.3, 5.5, 5.6, 5.9, 14.1); pr-s31b-notices.md (§§5.3, 5.5,
                         5.6, 5.8, 10); pr-s31c-price-survey.md (§§5.3, 5.6, 5.10, 10); pr-s31h-haggling.md
                         (§§5.12, 5.15, 10, 14); docs/MODULE_SPEC.md §5.1; docs/decisions/ ARC-25, ARC-27,
                         ARC-34, ARC-35, ARC-53, ARC-58, ARC-72, ARC-84, ARC-85, ARC-90, ARC-153;
                         .structured-coding/standards.md; CLAUDE.md; docs/ENGINEERING_STANDARDS.md;
                         docs/ENGINEERING_RULES.md; /Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s31d, its own, held by one session
BRANCH:                  mvp0/pr-s31d, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator (step-31 §16.5) AND this design frozen
                         by the operator AND S31-a, S31-b and S31-c merged AND the dispatch. NOT met at
                         2026-10-18
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–9, as §14's rulings shape them                     (operator, pending)
FROZEN INVARIANTS:       step-31 I-S31-1, I-S31-8, I-S31-10 (their S31-d parts); §6 I-d-1 … I-d-10; §8 as
                         written; Q-S31a-3 (incomplete offers stay incomplete); ARC-35 check 2 as it stands
SEQUENCE:                C0 … C8 (§9); implementers: Haiku C0–C5, C8; Opus C6, C7
ALLOWED COMMANDS:        cargo build / test -p <crate> --test <file> / --lib / clippy -p / fmt;
                         target/*/mineworld *; the cast-slots with-slot.sh wrapper; uv run --locked
                         pytest|ruff|pyright on named files; git; gh (pr create/view/checks, run view/download;
                         never merge); python3 scripts/check_doc_headings.py, check_decision_ids.py
                         (--write-index), ci_layer.py --list; mkdir -p; cp; cmp; sha256sum/shasum; sed -n; grep
NEVER:                   python3 -c; sed -i; perl -pi; heredoc writes; curl; git add -A; cargo test --workspace
                         or all of tools/cli (R-LOAD-1); reading any key or .env; calling any hosted or local
                         model (the LM half uses the scripted backend only); editing a path outside I-d-1's
                         allowance; naming a market crate under cognition/; changing ARC-35's checks; a nightly
                         dispatch (nothing is re-baselined); merging
MATERIAL STOPS:          Milestone F not handed over; S31-a, S31-b or S31-c not merged; a record or payload
                         shape §5 cannot read; a needed change under I-d-1's forbidden paths; a needed new
                         external dependency (DEP); any reference world's baseline moving; AC-d-14 showing an
                         existing LM request moved; AC-d-8's or AC-d-9's floors failing (recorded FAIL with the
                         numbers, never tuned); any change to a frozen step invariant or acceptance
PLATFORMS:               Linux, macOS, Windows (CI `fast` per PR; the full matrix at the milestone and on main)
VALIDATION BUDGET:       per PR (speed first): the build, the unit / contract / Python test files each commit
                         writes, one at a time, and CI `fast`. At the milestone: tools/cli price_reporter.rs
                         (W2 10 d ×3, W3 10 d, W4 5 d ×4, W5 and Lakeside 10 d ×2; ≤ 600 s on Linux CI),
                         price_reporter_lm.rs in the python layer (≤ 300 s wall), check_2, the parity
                         record, and every mutation of §8.1. Gate 1: NOT REQUIRED. Gate 2: NOT REQUIRED. No
                         paid call
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s31d.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                       (source: the operator's freeze of this design — pending)
                         semantic commits: authorized                    (working rules §14)
                         branch push: authorized                         (working rules §21)
                         PR creation / update: authorized                (working rules §21)
                         CI repair to review readiness: authorized       (working rules §21)
                         endpoint: MERGE-READY — pushed, `fast` green on the exact head, reported with
                         the PR number and head SHA
                         merge: by the coordinator only (charter top block, 2026-10-18); never by the
                         implementation session or the lane lead
POST-MERGE SYNC OWNER:   lane lead / primary: step-31, overall.md, the progress page; implementation session:
                         this ledger, the merge identity, evidence, deviations
STOP CONDITION:          S31-d MERGE-READY (PR number and head SHA reported) — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S31d-1 | Q-S31d-1 is ruled (b) | this design is revised before its freeze: `wire.rs` and the drift guard go, the policies import economy's and item's types after `ARC-35` check 2 is amended in the same PR; the policies themselves do not change |
| R-S31d-2 | Feedback oscillation (step R-S31-2) | deadband, a step ≤ 50 %, the book's 7-day window, bounds by the reference; AC-d-8 (c) records every move |
| R-S31d-3 | The book is too thin in 10 days for the floors | the second-seller construction (F-S31d-9); floors fixed now; a miss is a recorded FAIL and a material stop, not a tuned run (`ARC-23`) |
| R-S31d-4 | A WE-a modifier makes the listing show a price the keeper's `set-price` does not set (the base), so the move overshoots | C0 records how a modifier shows; the policy skips a shop ask while one is shown; if none is shown, the overshoot is a recorded limitation until WE-a's design discloses it |
| R-S31d-5 | A market pack changes a record and the mirrors drift | AC-d-7 in `systems/economy/tests` runs on every economy change; `payload_for` refuses a newer schema (I-d-10) |
| R-S31d-6 | An LM clerk floods or posts nonsense | one `due` consult a day; S31-b's `per_day`; the filler's checks; the board names the publisher (step R-S31-7) |
| R-S31d-7 | S31-a's or S31-c's merged shapes differ from their designs | C0 records the real names; a shape §5 cannot read is a material stop |
| R-S31d-8 | A keeper who also sells on its own account never prices the shop (F-S31d-6) | recorded; S31-f gives keepers no personal asks, or a later economy disclosure of `keepers` lifts it |
| R-S31d-9 | A `due` trigger is replaced by a `spoken_to` one and that day's bulletin is lost | the replacing consult lists the filler too; the next day's `day-began` raises another |
| R-S31d-10 | Hosted consult instants are not exactly `pace` apart, so a slot is missed or doubled under load | the same assumption as the rule's answer window (`ARC-27`); `per_day` caps a double; a miss waits for the next slot |
| R-S31d-11 | The swap test is slow or flaky under CI load | `--solo --time-scale 1440`; a bounded wait; a timing failure is INCONCLUSIVE, never re-tuned silently |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation | Reason, one line |
| --- | --- | --- | --- |
| **Q-S31d-1 [operator — F-S31d-1, `ARC-35` check 2]** | The ruled `pricing:` block lives in `rule-controller`, which `ARC-35` check 2 forbids from linking `economy` or `item`; no `cognition/` crate may even name them. How should controllers read prices and send `set-price`? (a) read the records and build the requests **by slug**, through small mirror types, with a test in `systems/economy` that holds the mirrors equal to the real types; (b) amend check 2 so that cognition crates may depend on the market packs | **(a), recorded as `ARC-182`** with a note under `ARC-35`. S31-h's haggling uses the same mirrors | It keeps the market removable and check 2 unchanged, and `ARC-35` item 6 already reads the market by slug in tests |
| **Q-S31d-2 [operator — F-S31d-2, `ARC-72`; answers S31-h's Q-S31h-4]** | An LM seat can only talk or attempt a complete offer, so it cannot post a notice (`publish-notice` is incomplete). Add a closed registry of typed **fillers**, one per incomplete action, where the model fills checked fields and the controller builds the payload? S31-d ships the registry and the `publish-notice` filler; S31-h's four and economy's seller actions follow as one module each | **Yes, recorded as `ARC-183`** with a note under `ARC-72` | One typed mechanism serves every incomplete action, the model never writes a payload, and no existing LM request or cassette changes |
| **Q-S31d-3 [operator — F-S31d-3]** | The LM has no clock. When does an LM clerk post? (a) once per world day, on calendar's `day-began`, only for a seat offered `publish-notice`; (b) at every change of its agenda; (c) at fixed hours, like the deterministic clerk | **(a)** | One model call a day per clerk bounds cost, and the deterministic clerk covers fixed hours |
| **Q-S31d-4 [operator / primary — F-S31d-4]** | A decision list sets parameters per model, not per seat, so every `rule` seat gets one `pricing:` block. Split the block into `sell:` and `buy:` halves, each with a `tags:` filter on the seat's own tags (S31-h's options move under them)? The alternative is per-seat parameters in `decisions.yaml`, a change to `ARC-84` in another PR | **Yes, the halves with tag filters** | It targets keepers, sellers and buyers with data alone, and changes no decision-list contract |
| **Q-S31d-5 [operator — F-S31d-7]** | An LM seat never walks (`ARC-72`), so an LM clerk only sees trade where it stands. Accept that for MVP-0, with the deterministic clerk as Market Town's default and the swap tested on a clerk standing in the store? | **Accept** | Letting an LM seat walk is a larger change to `ARC-72` that S31 does not need |

### 14.1 Rulings

Ruled 2026-10-20 by the operator in the primary session (the freeze header above; relayed in the
implementation dispatch of 2026-10-26): **every question is answered as recommended.**

- Q-S31d-1 → (a): wire mirrors read and build by slug, `ARC-182` with a note under `ARC-35` (C1, C2, C3).
- Q-S31d-2 → yes: a closed registry of typed fillers, `ARC-183` with a note under `ARC-72` (C1, C6).
- Q-S31d-3 → (a): the `due` trigger on calendar's `day-began` (C6).
- Q-S31d-4 → yes: `sell:` / `buy:` halves, each with a `tags:` filter (C2, C3).
- Q-S31d-5 → accept: an LM clerk sees only where it stands; the swap test places the clerk in the store (C7).

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 | (this commit) | [x] | [x] `check_decision_ids.py` PASS | [x] §2.3 holds (below) |
| C1 … C8 | — | [ ] | [ ] | [ ] |

### Evidence

**C0 — base and prerequisites (2026-10-26).**

1. Base: `origin/main` = `0b2faf2f989c2497b809e45adf4dfb6720a13ca9` (GitHub `commits/main` agrees). Dispatch: the
   coordinator's implementation brief of 2026-10-26 ("Execute its §9 commit plan under its §12 contract"; the
   design "DESIGN FROZEN 2026-10-20; all questions ruled as recommended — record that at C0"). Milestone F's
   checklist: the dispatch is the coordinator's statement that the start condition holds (the S29 timing rule is
   the coordinator's to judge; the operator's "autonomous until milestone" ruling of 10-19 runs past F).
2. Merged on the base: S31-a (#380), S31-b (#349), S31-c (#424, W5 #435), S31-e (#422), S31-g (#418).
   `systems/economy/src/sale.rs`, `systems/notices/src/lib.rs`, `systems/price-survey/src/lib.rs` exist.
3. Real names (all read on the base):
   - economy (`SystemId "economy"`): `asks` view = component type `asks`, schema 1, `{ asks: [{ item, price,
     available }] }` (`AsksView`, `component.rs:345-380`, disclosed about a person seller to whoever perceives
     it, `system.rs:669`); `currency` schema 1, `{ name, symbol, exponent, symbol_at, references: [{ item, value
     }] }` (`currency.rs:66-80`), to each living Person about itself; the shop listing = component type `shop`,
     **schema 2** (WE-a bumped `Shop`; the listing view is disclosed under `Shop`'s labels, `system.rs:675`),
     `{ operator, listed: [{ item, price, in_stock }] }` (`component.rs:300-330`); `set-price` / `list-for-sale`
     `{ item, price, for?: "shop" }` (`deny_unknown_fields`, `ShopSide` snake_case, `action.rs:57-110`); `buy`
     `{ item, price }`.
   - inventory (`"inventory"`): `holdings` schema 1, `{ held: [{ item, count }] }` plus **`capacity` when
     bounded** (`system.rs:144-175`, `DisclosedHoldings` flattens) — the mirror admits the optional key.
   - item (`"item"`): `item-catalogue` schema 1, `{ kinds: [{ item, category, name }] }` (`component.rs:44-111`).
   - price-survey: `mineworld_price_survey::{PriceBook, KindNotes, Note, Source, Survey}`; `PriceBook` is
     `survey-book` schema 1 with `kinds()`, `kind(item)`, `KindNotes::{item, notes}`, `Note::{at, place, seller,
     price, source, in_stock}`; `Survey {}` is `survey`, `deny_unknown_fields`; codes in `judge::codes`.
   - notices: `PublishNotice::new(TopicId, NoticeBody, u32)`, `NoticeBody::Prices(Vec<PriceLine>)`,
     `PriceLine::new(EntityId, u64, u16)`, `NOTICE_PRICE_MAX = 10^12`, `PRICE_LINES_MAX = 32`,
     `NoticeBoardRecord { pub notices }` (`notice-board`, schema 1), `Notice::{topic, publisher, body,
     issued_at, valid_until}`, `TopicId::new`.
4. S31-h (#426) is **OPEN**, owned by another agent: §2.3's "S31-d merges first" branch — `price/` carries the
   two `// S31-h:` call sites that do nothing, and the decoder refuses `buy.haggle` and
   `sell.accept_at_least_permille` until S31-h adds them. WE-a (#189) is **MERGED**: a shop's active adjustment
   shows **only as the adjusted `price` in the listing**; the listing does not disclose `adjustments`, and the
   `shop` record on a place is the listing view (`component.rs:310-325`, `Listing::of` applies
   `Shop::adjust`). R-S31d-4's second branch therefore holds: the policy cannot tell an adjusted listing price
   from the ask, and the overshoot (or a `price-unchanged` refusal) while a happening runs is a **recorded
   limitation** until economy discloses the adjustment.
5. Rulings: §14.1 (all as recommended): C2/C3 take the wire-mirror branch, C6 the filler registry and the
   `day-began` trigger.
6. Anchor drift (statements unchanged unless noted): A-2 **changed** — `rule` is no longer parameterless: since
   S35/LE it has `RuleParameters { commute, boss, needs }` (`model.rs:27-40`) with builder methods
   `commuting/bossing/needing` on `PacedRuleController`. Bounded: `pricing` becomes the fourth key and a
   `pricing(..)` builder replaces §5.4's `decide_priced` (same behaviour, same default parity). A-3:
   `MODULE_SPEC.md:816`. A-5: `cognition/installed/src/lib.rs:20-24` unchanged;
   `tools/cli/tests/decision_models.rs:52`. A-6: `packs.rs:119`. A-7 **moved**: `paced.rs` now has more hard
   rungs (asleep, commute, eat, boss, bed, fetch); draw indices 0–6, 8–15 still taken, 7 free; the seller
   band goes after the boss rung and before bed (no draw, so position cannot move a draw). A-8: `offered.rs`
   `attempt(observation, draw, decided_by_name, excluded)`; constants unchanged. A-9: `motivated::decide`
   takes `&Settings`; the `pricing` block is carried in `Settings`. A-13: `ac1_composability.rs:651-662,
   884-917`. A-21: `run.rs:48`. A precedent for slug reading already exists in `rule-controller`:
   `fetch.rs:100-140` reads `holdings`, `shop-stock`, `buy` by slug (S35 N-h).

### Mutations

(none yet; X-6, M-d-1 … M-d-9 as §8.1 lists them)

### Deviations and findings

F-S31d-1 … F-S31d-12 (§3.2) and SD-S31d-1 … 14 (§11), recorded at design time.

---

## 16. Proposed edits elsewhere (made by the PR or owner named)

| File | Edit | By |
| --- | --- | --- |
| `step-31-wealth-prices.md` §9.4 | dated note: the deterministic rule as §5.3 specifies it (sales before asks, ask dedup, `min_basis` on each); the LM backend through a filler and a `day-began` trigger (Q-S31d-2, Q-S31d-3, if ruled) | the primary, at freeze |
| `step-31-wealth-prices.md` §10.1 | dated note: parameters are per model (F-S31d-4); the block's `sell:` / `buy:` halves with `tags:` (Q-S31d-4); `list-goods` becomes `sell.list.categories`; the buyer default is `any` (SD-S31d-9); the policies live in `rule-controller` read through wire mirrors (Q-S31d-1) | the primary |
| `step-31-wealth-prices.md` §11 | dated note: "typed tools" become typed `act` kinds under `ARC-153` | the primary (S31-g's design) |
| `step-31-wealth-prices.md` AC-S31-11 | "on `lm` (cassette replay)" reads "on `lm` driven by a scripted model, with its cassette replayed offline" (AC-d-11, AC-d-12) | the primary |
| `pr-s31c-price-survey.md` §5.10 item 4 | the JSON example with the contract's id encodings (F-S31d-11) | S31-c's designer / the lane lead |
| `pr-s31h-haggling.md` §§5.12, 5.15, 10 | R-S31h-d-1: option placement, `price/haggle.rs`, wire mirrors, per-model configuration, four LM fillers | S31-h's designer / the lane lead |
| `pr-s31b-notices.md` / its implementation | F-S31d-12: a loose object passes `judge_body` as a price item since 12d (bounded; S31-d's producers never name one) | the lane lead, for S31-b |
| S31-f's design | F-S31d-10: `ARC-35` check 3 must admit `notices`, `price-survey` and their sections in Market Town | the lane lead, for S31-f |
| `docs/decisions/ARC-117.md`, `ARC-116.md` note, `ARC-182.md`, `ARC-183.md`, `ARC-35.md` note, `ARC-72.md` note, `INDEX.md`; `docs/MODULE_SPEC.md` §5.1 | §5.12's texts | this PR, C1 |
