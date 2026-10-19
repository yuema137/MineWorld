# PR S31-c — The `price-survey` System Pack: a surveyor's price book, built only from what it perceived

## DRAFT — not frozen

```text
Design revision:        revision 2 (2026-10-19, lane lead: the charter's SPEED FIRST block — the coordinator
                        merges, per-PR validation is light, a generated second world W5 in §8), DRAFT;
                        revision 1 (2026-10-18)
Author:                 S31/S29 lane design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s31c, branch docs/pr-design-s31c,
                        planning base origin/main @ a1e05874 (#273 merged; step-31 APPROVED, S31-a and
                        S31-b FROZEN on main)
Approved by / evidence: none yet. The step's rulings (Q-S31-1 … Q-S31-8, step-31 §18), S31-a's rulings
                        (Q-S31a-1 … 4, pr-s31a §14.1) and S31-b's rulings (Q-S31b-1 … 4) bind and are
                        not re-asked
Implementation base:    main (origin/main at C0); C0 records the commit and the merge state of S31-a,
                        WE-a (#189) and #258 (the decision-log split: merged on main at b787a083, after
                        this planning base)
Start condition:        step-31 §16.5: "Nothing starts before Milestone F's checklist has been handed to
                        the operator (the S29 timing rule). Each PR starts only once its own PR design is
                        frozen." Then S31-a merged (step-31 §16.2, S31-c row: "Depends on S31-a").
                        State at 2026-10-18: Milestone F's checklist NOT handed over; S31-a design FROZEN
                        (2026-10-18), implementation not started, no PR open; this design DRAFT. Then the
                        primary session's (or lane lead's) dispatch
Execution contract:     §12 (filled; ends at MERGE-READY: pushed, `fast` green on the exact head, reported
                        with PR number and head SHA; the coordinator merges, charter SPEED FIRST 2026-10-18)
Implementer:            Haiku-class (step-31 §16.2, the S31-c row: "well-specified pack on a frozen
                        pattern"); reviewer the lane lead, plus an Opus logic review of C3 (the witness
                        rule) at the milestone
Lifecycle:              DRAFT
```

**Effort:** `mvp0`, release-blocking (Q-S31-1 re-ruled) · **Step:**
[`step-31-wealth-prices.md`](step-31-wealth-prices.md) (APPROVED 2026-10-15) §§0.1 (items 2, 3), 1 (outcome 7),
3 ("price book", "reporter"), 9.1, 9.2, 9.3, 13, 14 (I-S31-1, I-S31-2, I-S31-9, I-S31-10), 15 (AC-S31-10;
X-7), 16.2 (the S31-c row), 16.5, 19.1, 19.4 (`ARC-116`, book half) · **Parent:** [`overall.md`](overall.md) ·
**Depends on:** [`pr-s31a-currency-asks.md`](pr-s31a-currency-asks.md) (FROZEN 2026-10-18; `sold`, `Asks`,
the shop listing and the person `asks` view) · **Parallel with:** [`pr-s31b-notices.md`](pr-s31b-notices.md)
(FROZEN; no shared file but `systems/installed`, `docs/MODULE_SPEC.md` §4.1 and the decision log) ·
**Followed by:** S31-d (the `price-reporter` model reads the book this PR defines, §5.10), S31-f (Market
Town's reporter carries the section) · **Binding parents:** `CLAUDE.md` §§2, 3, 4 (items 1, 2, 3, 5, 7, 8, 9,
10, 11, 13, 15, 16); `docs/CORE_CONCEPTS.md` §§12, 15.2; `docs/MODULE_SPEC.md` §4.1; `docs/DECISIONS.md`
`ARC-26`, `ARC-28`, `ARC-31`, `ARC-33`, `ARC-34`, `ARC-36`, `ARC-37`, `ARC-43`, `ARC-61`, `ARC-90`.

**Decision ids.** This PR writes **`ARC-116`** (assigned by the step, step-31 §19.4: "The price book as
perception-derived world state, and the reporter as a seat whose backend is its decision model"). This PR
writes the book half and leaves a marked placeholder paragraph for the backend half, which S31-d fills as a
dated note (the `ARC-114` precedent, S31-a SD-S31a-8). ARC-116 is confirmed free on this base
(`grep -n "^## ARC-116" docs/DECISIONS.md`, no match) and on `origin/main @ a8bba658`
(`docs/decisions/ARC-116.md` does not exist). **No new decision id is used**; the reserved `ARC-180` and
`ARC-181` stay free. The record goes where the log lives at C1's base: `docs/decisions/ARC-116.md` plus the
regenerated index, since #258 has merged (SD-S31c-12).

No hosted model is called by this design or by anything it specifies for CI.

### Local ids in this file

| Prefix | Meaning |
| --- | --- |
| `A-n` | audit anchors (this file's own numbering) |
| `F-S31c-n` | findings |
| `SD-S31c-n` | bounded decisions |
| `I-c-n` | invariants of this PR |
| `AC-c-n` | acceptance criteria of this PR |
| `M-c-n` | planted mutations beyond the step's `X-7` |
| `R-S31c-n` | risks |
| `Q-S31c-n` | operator questions |
| `W1 … W4` | the worlds the acceptance runs on (§8) |

---

## 0. Binding inputs, exactly restated

- **Operator requirement, 2026-10-15 (step-31 §0.1, verbatim):** "必需的消耗品 食材等等，可以有市场建议价格（一个专门的npc
  负责统计，然后给出建议，这个npc后端可以是llm agent，也可以是确定性的算法什么的，这个也是模块化的…）" — *"Essential
  consumables, food ingredients and so on, can have a market suggested price. A dedicated NPC is responsible for
  compiling statistics and giving the suggestion. This NPC's backend can be an LLM agent or a deterministic
  algorithm, and that is modular too."* S31-c builds the "compiling statistics" half: the NPC's notebook. S31-d
  builds the "giving the suggestion" half.
- **The coordinator's restatement (step-31 §0.1, items 2 and 3).** "Suggestions are computed from observed
  trades, using the sale facts that S31 itself emits." "The reporter is a real Person with a role. Its backend
  is pluggable through the existing decision-model registry … It works only from what it can legitimately
  perceive."
- **Operator principles, 2026-10-18 (relayed in this task).** One currency per world. Any seller sets its own
  price, and haggling exists. A price clerk publishes suggested prices; its backend is modular, an LLM or
  deterministic, swappable. Agents over MCP play by the same server path and rules.
- **No world-specific hard-coding (operator, 2026-10-18).** "我们不能为了让 default 小世界跑起来，就 hardcode
  一大堆东西". Mechanisms are generic and data-driven, and every acceptance criterion names at least one second
  world or a generated variant (§8 names four).
- **Scope (step-31 §16.2, S31-c row).** "`systems/price-survey`: section, `PriceBook`, `survey`, witnessed
  `sold`, `ARC-116` (book half)". Depends on "S31-a". Checkpoint "fixture world: AC-S31-10". Adversarial "X-7".
  Implementer **Haiku**.
- **Step §9.2 (what the reporter may know).** Asks on display where it stands: yes. Sales it witnessed: yes.
  Stock levels on shop listings: yes. Trades reported to it by sellers: not in S31. A public trade tape:
  declined.
- **Step §9.3 (the book).** "A stateless decision model cannot remember what it saw … So the reporter's notes
  are a notebook in the world, disclosed to its holder, identical for every backend." The pack "depends on
  `economy` … and `presence`". The section is `price-survey: { kinds: { categories: [food, drink] }, depth: 32,
  window: "7d" }`. The component is "written only by `price-survey`'s reduction of its own fact `price-noted`".
  Entries arise by witnessing (`sold` while present) and by surveying (action `survey`, "offered complete when
  the surveyor stands where asks are on display").
- **AC-S31-10.** "The set of `price-noted { source: sale }` equals the set of `sold` facts in the reporter's
  perceived stream. Moving the reporter's routine away from the store removes store sales from its book."
- **X-7.** "`price-survey` notes every `sold` regardless of presence" must fail AC-S31-10.
- **Q-S31a-3 (ruled 2026-10-15).** "A complete offer must be safe for a controller to take at random." `survey`
  is offered complete (step §9.3), so §5.6 makes it safe (Q-S31c-2).
- **R-S31c-a-1 (from S31-a, binding here).** "Name price-survey's component and process so they do not collide
  with WE-a's `price-book` (F-S31a-11)." Applied in §5.3 (SD-S31c-1).
- **Speed first (operator, 2026-10-18, through the lane-lead charter).** A PR merges once implemented and its
  `fast` check is green. Full, acceptance and integration tests run at milestones, and failures are fixed then.
  Designs are still frozen first. §9 and §12 follow this: per commit only the pack test file being written
  runs locally; the real-binary checkpoint (C5) is written in the PR and run once, and is re-run at the
  milestone.

---

## 1. Goal

Give a person a **price book**: a bounded notebook of prices it saw with its own eyes. A world author makes a
person a **surveyor** by giving it a `price-survey:` section that names which kinds of goods it tracks (by item
category, for example `food` and `drink`). From then on, two things fill its book:

1. **Witnessing.** When a sale (`sold`, economy's fact) happens and the surveyor is one of the people who
   perceived it — standing in that place at that instant, by presence's own audience rule — the pack writes the
   sale's item, seller, place, unit price and time into the book. A sale the surveyor did not perceive never
   reaches the book.
2. **Surveying.** The surveyor may take the action `survey` where asks are on display. The pack then copies
   every tracked ask that economy discloses to that surveyor there — the shop's listing with its stock count,
   and each present person seller's asks with availability only — into the book.

The book lives in the world as a component, written only by this pack's reduction of its own fact
`price-noted`, persisted and replayed like any state. It is disclosed to its holder only. Because it is world
state disclosed through the Observation, every backend that drives the surveyor's seat — the deterministic
`price-reporter` model, an LM, a human, an MCP agent — reads exactly the same book (F-S31-5). This PR does not
compute a suggestion and does not publish anything; S31-d does. Nothing changes in `kernel/` or `contracts/`
(I-S31-1), and no reference world enables the pack (S31-f gives Market Town its reporter).

A concrete example. Carol is a surveyor of `food` and `drink`. At 07:10 she stands in the store and Bob buys
bread for 300: her book gains `bread: { at 07:10, store, seller corner-store, 300, sale }`. At 07:20 she takes
`survey`: her book gains one `ask` note per food or drink the store lists, each with its stock count, for
example `milk: { at 07:20, store, corner-store, 200, ask, in_stock 4 }`. At 12:30 Erin buys coffee in the café
while Carol is still on the street: Carol's book does not change, because she did not perceive that sale.

---

## 2. Scope

### 2.1 In scope

1. **`systems/price-survey`** (new crate `mineworld-price-survey`, `SystemId` `price-survey`):
   - `Cargo.toml` (§5.1 lists the dependencies and why).
   - `src/lib.rs`: exports and the pack table in the module docs.
   - `src/span.rs`: `Span`, the `"<n>m" | "<n>h" | "<n>d"` duration of the section (§5.2, SD-S31c-6).
   - `src/section.rs`: `AuthoredSurvey`, `AuthoredKinds`; `impl AuthoredSection` (Person only).
   - `src/component.rs`: `PriceBook`, `KindNotes`, `Note`, `Source`, `Surveyed`; the disclosed view (§5.3).
   - `src/event.rs`: `SurveyBookOpened`, `PriceNoted`, `Noted` (§5.4).
   - `src/action.rs`: `Survey`; `survey_requirement` (§5.6).
   - `src/judge.rs`: `judge_survey` — the one decision `offers` and `validate` both call; `asks_here` (§5.6).
   - `src/witness.rs`: `witnesses` — who of the surveyors perceived a `sold` (§5.5).
   - `src/system.rs`, `src/codec.rs`: declaration, `validate`, `resolve`, `react`, `offers`, `discloses`.
   - `tests/support/mod.rs` and `tests/{section.rs, witness.rs, survey.rs, book.rs}`.
   - `README.md`.
2. **`systems/installed`**: one line in `Cargo.toml`, one in `src/lib.rs` (`PriceSurvey =>
   mineworld_price_survey::PriceSurveySystem`, appended last), as `ARC-33` prescribes. `Cargo.lock`.
3. **`systems/economy`**, read surface only, and only if S31-a's merged code lacks it (C0 checks; §5.9 lists the
   exact items): `pub` getters on `Sold`, `Listing`/`Listed` and the person `asks` view, and their re-export
   from `lib.rs`. No behaviour change, no new fact, no version change (I-c-1).
4. **`tools/cli/tests/price_survey.rs`** (new): the real-binary checkpoint on scratch copies of Market Town (§8
   AC-c-6 … AC-c-9).
5. **`worldpack/tests/refusals.rs`**: one case — the section on an item file refused by name and line (the
   install precedent of S26-a and S31-b).
6. **Documentation:** `ARC-116` (book half); `docs/MODULE_SPEC.md` §4.1 (the `price-survey` section row); the
   generated `docs/reference/` files regenerated by `mineworld reference` (S25a, `ARC-90`).

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| Computing suggested prices (median, supply adjustment), publishing `market-prices` notices, the `price-reporter` model and its `models!` line, the LM cassette, `ARC-116`'s backend half | S31-d |
| Market Town's reporter person, its section, routine and topic | S31-f |
| An agent tool or a `MeLine` line for the book; words for the `survey` codes in the 2D/3D clients | S31-g, S31-e (R-S31g-c-1, R-S31e-c-1) |
| An S28 memory renderer for `price-noted` | not needed: the fact reaches nobody's memory (Q-S31c-1). If the ruling is the alternative, S31-g adds one |
| Trades reported to the reporter by sellers (`trade-reports`), a public trade tape | declined or later (step §9.2) |
| Any change under `kernel/`, `contracts/`, `persistence/`, `server/`, `clients/`, `cognition/`, and under `systems/*` other than `price-survey`, `installed` and §2.1 item 3's read surface of `economy` | never in this PR (I-c-1) |

### 2.3 Dependencies

- **Required before C0:** Milestone F's checklist handed over (operator); this design frozen; S31-a merged
  (`sold`, `Asks`, the shop listing over asks, the person `asks` view exist only after it); the dispatch.
- **Either order, no conflict of content:** S31-b. Both append one line to `systems/installed` and one row to
  `MODULE_SPEC.md` §4.1, and each writes its own decision record. Whichever lands second rebases those lines.
- **Either order, shape rule:** S31-h (haggling). An accepted proposal states the same `sold` (step §5.9), so the
  witness rule covers it with no change here. If S31-h has merged at C3, `witness.rs` gets one more test case (a
  witnessed accepted proposal is noted at the proposed price); otherwise nothing.
- **Either order, modifier rule:** WE-a (#189, open). A shop's disclosed listing already carries the effective
  price when WE-a has merged (S31-a §5.7, step §5.8). The survey copies what the listing says and computes no
  price, so nothing here depends on WE-a.
- **Before S31-d's C-commit that reads the book:** this PR merged. S31-d reads §5.10's surface only.

---

## 3. Audit (`origin/main @ a1e05874`, 2026-10-18)

S31-a is frozen and not implemented. Anchors on S31-a's code therefore cite its design; C0 re-reads them on the
merged code (§9, C0 step 3).

### 3.1 Source anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | Presence owns who learns of a fact. `admits(whereabouts, fact, observer)`: `Place(p)` admits the fact's participants and subjects, and everybody whose whereabouts, after the fact is applied, is `p`. `perceived_by` folds `Whereabouts` over the log and calls `admits`; the server's frames, the `perceived` stream and `mineworld perceived` all call it | `systems/presence/src/audience.rs:100-139` (`ARC-43`) | The witness rule calls this same function, so "the surveyor perceived the sale" is presence's answer, not a second rule (§5.5) |
| A-2 | `Whereabouts::from_world(world)` reads presence's `Presence` components; "a fold from the first fact of a world equals `Whereabouts::from_world` of that world at the same revision" | `audience.rs:61-69, 43-49` | Inside `react`, `from_world` is exactly the fold the audience rule uses, as long as the facts before `sold` in its generation are already reduced (A-4) |
| A-3 | `sold { seller, buyer, item, count: 1, price, place }`, `Visibility::Place(place)`, `about: [buyer, seller]`, participants both; stated first in `settle`, before `money-transferred` and `items-transferred`, in one dispatch | `pr-s31a-currency-asks.md` §5.7 "Resolve", §5.9 | The witness rule hears `sold` and nothing else of economy's; `sold` moves nobody |
| A-4 | Reduction runs each fact of a generation through every subscriber in registration order, fact by fact; what a subscriber emits is reduced in the next generation of the same instant (`BD-7`), caused by the fact it reacted to | `kernel/src/dispatch.rs:604-660` | `price-noted` lands in the sale's revision, `Causation::Event(sold)`. When `react` sees `sold`, every earlier fact of the generation is reduced and no later one is |
| A-5 | A system hears another pack's fact through that pack's published type, with no system dependency (economy hears `wage-due`, `ARC-28`) | `systems/economy/src/system.rs:55-74, 193-196` | `price-survey` subscribes to `Sold`. It still declares a dependency on economy, because a surveyor in a world without economy has nothing to survey (§5.1) |
| A-6 | A declaration's `depending_on` makes the registry refuse a world that enables the pack without its dependency, naming it | `kernel/src/system.rs:138-144`; `kernel/src/registry.rs:213` | AC-c-9's "price-survey without economy is refused by name" |
| A-7 | `PerceptionProvider::{offers, discloses}`; economy discloses the shop's `Listing { operator, listed: [{ item, price, in_stock }] }` under the `shop` component type to whoever perceives the place, built from the operator's `Holdings` at disclosure | `systems/economy/src/system.rs:228-263`; `offer.rs:51-55`; `component.rs:103-165` | The survey reads the listing through economy's own `discloses` (SD-S31c-3), so it copies exactly what the surveyor is shown |
| A-8 | S31-a adds the person `asks` view, `{ asks: [{ item, price, available }] }`, "of a person seller, to whoever perceives them", never a count | `pr-s31a-currency-asks.md` §5.9 table, "Disclosure summary" | A person seller's ask note has `in_stock: None` (§5.3); the survey cannot leak a person's holdings (I-c-6) |
| A-9 | An observation lists "the place you are in, and everybody this pack knows to be in it" | `systems/presence/src/observe.rs:8-14` | "Asks on display where it stands" = the place's shop listing plus the asks of every person present (§5.6) |
| A-10 | `Offer::complete(&action, requirement)`; `refused(reason)` is reported before the requirement | `systems/presence/src/interaction.rs:131-216` | `survey` is one complete offer, refused with the cooldown code while cooling down (§5.6) |
| A-11 | `SpatialRequirement::at_place(place)`; `SamePlaceAsActor` needs a target | `contracts/src/spatial.rs:364-372`; `pr-s31a-currency-asks.md` F-S31a-4 | Step §9.3's "`SamePlaceAsActor`" for a target-less `survey` would always be `PreconditionFailed`; the requirement is `at_place(<the surveyor's place>)` (SD-S31c-4, S31-a SD-S31a-3's precedent) |
| A-12 | Generic controllers attempt an available complete affordance at random (rule: on 20 % of consults; motivated: each available complete affordance is a choice) | `cognition/rule-controller/src/offered.rs:26-52`; `cognition/motivated/src/options.rs:285-289` | A surveyor on `rule` would take `survey` again and again; without a bound its asks would flood the book and evict its sale notes (F-S31c-3, Q-S31c-2) |
| A-13 | `ItemKind { category }` on the Item entity, owned by `item`; `Category` is a validated slug type; inventory depends on item | `systems/item/src/component.rs:9-35`; `category.rs:22-80`; `systems/inventory/src/system.rs:57` | The kinds filter decodes with item's `Category` and reads `ItemKind` (SD-S31c-5) |
| A-14 | `AuthoredSection { SECTION, CARRIED_BY, Authored: DeserializeOwned + JsonSchema, references, seed }`; `Seeding` has the assembled world and no state; "a check that needs state another genesis fact creates belongs in the owner's reduction" | `authoring/src/section.rs:120-200` | "Is any declared kind of this category?" needs `ItemKind`, which is state: it is checked at reduction, which `mineworld validate` runs (§5.2) |
| A-15 | Items' sections seed first, then organizations', then people's (`ARC-36` item 7); `mineworld validate` reduces genesis, so a reduction refusal fails validate by name | `systems/economy/tests/support/mod.rs:1-5`; `tools/cli/src/main.rs:587-589` (S31-b A-6) | When a person's `survey-book-opened` is reduced, every `ItemKind` exists |
| A-16 | `WorldRead::components::<C>()` iterates in `EntityId` order; `WorldRead::at()` is the world time of the read | `kernel/src/view.rs:94, 144-147` | The witness rule visits surveyors in a fixed order; the disclosed book is filtered by `at()` (lazy window) |
| A-17 | `owned_component!` declares a component type id, owner and schema version; a component type id is one namespace across packs | `kernel/src/access.rs:262-279` | `survey-book` (§5.3); WE-a's `price-book` is a `ProcessTypeId` in economy (PR #189 `systems/economy/src/book.rs:16-19`), so no id collides (SD-S31c-1) |
| A-18 | No shared duration type exists; calendar parses its own `Epoch`; S31-b specifies its own `NoticeDuration` (`"<n>m|h|d"`) in `notices` | `systems/calendar/src/configuration.rs:60-100`; `pr-s31b-notices.md` §4, §5.2 | The section's spans are this pack's own small type with the same grammar (SD-S31c-6) |
| A-19 | Installing a pack is one line in `systems/installed/Cargo.toml` and one in `src/lib.rs`; members are `systems/*` | `systems/installed/src/lib.rs:38-62`; `Cargo.toml` `members` | C2's install |
| A-20 | Rule-controller modules read a disclosed record by its component type and decode it with the owning pack's type (`read::disclosed::<C>`), so a model is compiled against a pack's published types | `cognition/rule-controller/src/read.rs:121-127`; `agenda.rs:32-40`; `Cargo.toml` comments | S31-d's model reads `PriceBook` the same way (§5.10) |
| A-21 | Real-binary tests copy a world into a scratch directory, edit text, run `mineworld run … --headless --seed --days --save`, and read the save's facts with `Tables::read` and `format::decode` | `tools/cli/tests/market_composition.rs:66-110`; `headless/mod.rs:196-240`; `market_town.rs:180-190` | C5's harness |
| A-22 | Carol, a seated resident with no job: routine `07:00 store`, `12:00 cafe`, `16:00 apartments`; persona "observant" | `worlds/market-town/people/carol.yaml` | W2's surveyor; W3 moves her `07:00` segment from `store` to `park` |
| A-23 | Market Town's shops sell food and drink only (café: coffee, tea, croissant, cake, sandwich, soup; store: apple, bread, milk, juice), and every NPC buys there under the paced controller | `worlds/market-town/organizations/*.yaml`; step-31 A-23 | In W2 every sale is of a tracked kind, so AC-c-6's set equality is not vacuous |
| A-24 | #258 merged on main (b787a083, after this base): one record per file, `docs/decisions/<ID>.md`, `check_decision_ids.py --write-index` | `git log origin/main`; `scripts/check_decision_ids.py` on origin/main | SD-S31c-12 |

### 3.2 Findings

- **F-S31c-1 — The step's requirement for `survey` cannot be met as written.** `SamePlaceAsActor` needs a target
  (A-11), and `survey` has none. The requirement is `at_place(<the place the surveyor stands in>)`, exactly as S31-a
  resolved the same defect for `for: shop` (SD-S31a-3). Same meaning, bounded → SD-S31c-4.
- **F-S31c-2 — AC-S31-10's set equality is over tracked kinds.** The book keeps only the kinds its section tracks
  (step §9.3: `kinds: { categories: [food, drink] }`). A surveyor that perceives a sale of a `goods` kind does not
  note it, so "the set of `price-noted { source: sale }` equals the set of `sold` facts in the reporter's perceived
  stream" holds only when both sides are restricted to tracked kinds. In Market Town today every sale is food or
  drink (A-23), so the equality is the same there, but the criterion must state the restriction → **Q-S31c-3**.
- **F-S31c-3 — A complete `survey` would flood the book.** Generic controllers take available complete
  affordances at random (A-12), and an LM may loop. Each survey adds one note per tracked ask. With a ring of
  `depth` notes per kind, ten surveys in an hour at the café push every sale note of coffee out of the book. That
  is unsafe in Q-S31a-3's sense. The fix is a per-place cooldown, `resurvey_after`, with the named refusal
  `surveyed-recently` → **Q-S31c-2**.
- **F-S31c-4 — A witnessed note would duplicate a fact the surveyor already has.** If `price-noted` were
  `Entities({surveyor})` (step §9.3: "visible to the surveyor only"), every witnessed sale would reach the
  surveyor's inbox and S28 memory twice: once as `sold`, once as `price-noted`. Q-S31a-2 and Q-S31b-2 ruled the
  same question for bookkeeping facts: the holder learns through its disclosed record, and the inbox keeps only
  what happened in the world → **Q-S31c-1**.
- **F-S31c-5 — The book must re-derive the audience, not approximate it.** "Present at the sale's place" is not
  quite presence's rule: `Place(p)` also admits the fact's participants and subjects wherever they are (A-1).
  Today a buyer or a person seller always stands in the sale's place, so the two agree, but a future sale form
  (a remote `mail-order` pack, step §5.4) would break a hand-written "same place" check. The witness rule
  therefore calls presence's `admits` itself (§5.5), which is exactly AC-S31-10's oracle.
- **F-S31c-6 — `react` sees the generation half-reduced, and that is still exact.** Within one generation the
  kernel reduces fact by fact (A-4). When `price-survey` reacts to `sold`, the facts recorded before `sold` in
  that generation have been reduced and those after it have not. `Whereabouts::from_world` therefore equals the
  audience fold advanced past every fact before `sold` (A-2), and `sold` moves nobody, so it equals the fold
  advanced past `sold` too — the state `admits` is defined against. This holds for any future pack that states
  an `arrived` in the same generation before `sold`. It is argued here and tested by AC-c-6's comparison with
  `perceived_by` over the real log.
- **F-S31c-7 — The book's English name collides with WE-a's process** (F-S31a-11, R-S31c-a-1). WE-a's
  `price-book` is a `ProcessTypeId` holding economy's price modifiers. This pack's component type id is
  `survey-book`; its Rust type keeps the step's name `PriceBook` (different crate, no Rust collision); prose says
  "price book" only for this pack's component → SD-S31c-1.
- **F-S31c-8 — Economy's read surface may be crate-private after S31-a.** Today `listing` is `pub(crate)` (A-7),
  and S31-a's design names `Sold`, `Listing` and the `asks` view without stating their getters' visibility. The
  survey decodes economy's disclosed records with economy's own types, so it needs `Deserialize` and `pub`
  getters on them. C0 checks the merged code; if any is missing, C2 adds it as the read-surface edit of §2.1 item
  3 (bounded: visibility only, no behaviour).

### 3.3 What does not exist

No `price-survey` crate, section, component, fact or action anywhere. No world names a surveyor. No test reads a
price book.

---

## 4. Reuse, both ways (`REUSE_POLICY.md`)

| Need | Candidates | Verdict |
| --- | --- | --- |
| "Did this person perceive this fact?" | write a "same place" check here | **Reused:** presence's `admits` (A-1, `ARC-43`). A second rule would drift from the one the server, the stream and the export all call |
| What the surveyor is shown | read `Asks`, `Shop` and `Holdings` directly | **Reused:** economy's own `discloses` (SD-S31c-3). Reading the raw components would let the survey see a person seller's exact holdings, which economy never discloses (A-8) |
| Item categories | a string list | **Reused:** item's `Category` type and `ItemKind` (A-13) |
| Duration strings (`"7d"`) | `humantime`, `duration-str` | **Declined**, for S31-b's reason (`pr-s31b-notices.md` §4): three suffixes over a positive integer, bounded, are a short `TryFrom<String>` with a message naming the key. Below the `DEP` threshold. The grammar equals S31-b's so authors learn one form (SD-S31c-6) |
| Ring buffer | `ringbuffer`, `arraydeque` | **Not needed:** a `Vec` per kind, appended and truncated from the front, ≤ 64 entries; std only |
| Statistics (median) | `statrs`, `average` | **Not here:** this pack keeps notes and computes nothing. S31-d's model owns any statistic |

No `DEP` record is written: nothing is adopted and no infrastructure is rebuilt.

---

## 5. Design

### 5.1 The crate

```text
systems/price-survey/
  Cargo.toml
  README.md
  src/
    lib.rs        exports; the pack table in the module docs
    span.rs       Span: "<n>m" | "<n>h" | "<n>d", bounded per key
    section.rs    AuthoredSurvey, AuthoredKinds; impl AuthoredSection (Person only)
    component.rs  PriceBook, KindNotes, Note, Source, Surveyed; PriceBook::visible_at
    event.rs      SurveyBookOpened, PriceNoted, Noted
    action.rs     Survey; survey_requirement
    judge.rs      judge_survey, asks_here, tracked
    witness.rs    witnesses
    codec.rs      payload readers, encode
    system.rs     declaration, install, validate, resolve, react, PerceptionProvider
  tests/
    support/mod.rs   the W1 fixture (§8)
    section.rs  witness.rs  survey.rs  book.rs
```

Each file stays under ~300 lines; `system.rs` dispatches only.

**`Cargo.toml` dependencies** (each with a one-line comment in the file, as `systems/economy/Cargo.toml` does):

| Dependency | Why |
| --- | --- |
| `mineworld-authoring` | the `price-survey:` section (`ARC-31`) |
| `mineworld-contracts`, `mineworld-kernel`, `mineworld-sdk` | every pack |
| `mineworld-presence` | `Presence` (where the surveyor stands), `audience::{admits, Whereabouts}` (the witness rule), `PerceptionProvider`, `Offer` |
| `mineworld-economy = { path = "../economy" }` | `Sold` (heard), `EconomySystem` (its `discloses`), `Listing`, the `asks` view type |
| `mineworld-item = { path = "../item" }` | `Category`, `ItemKind` |
| `schemars`, `serde`, `serde_json`, `thiserror` | the section's schema (`ARC-90`), payloads, the span error |
| dev: `mineworld-inventory`, `mineworld-movement`, `serde-saphyr` | the W1 fixture seeds holdings and passages, and decodes sections as the loader does |

**Declaration** (SD-S31c-5):

```rust
SystemDeclaration::of::<Self>()
    .depending_on([EconomySystem::ID, ItemSystem::ID, PresenceSystem::ID])
    .owning::<PriceBook>()
    .providing::<Survey>()
    .emitting::<SurveyBookOpened>()
    .emitting::<PriceNoted>()
    .subscribing_to::<SurveyBookOpened>()
    .subscribing_to::<PriceNoted>()
    .subscribing_to::<Sold>()
```

`SystemVersion` 1. `BIOGRAPHICAL = []` (noting a price is not an event in a life). The pack owns its section
(`owns_section!()`).

### 5.2 The section (`price-survey:` on a person file)

```yaml
# people/carol.yaml — makes Carol a surveyor
price-survey:
  kinds: { categories: [food, drink] }   # required: 1 … 16 distinct item categories
  depth: 32                              # optional, default 32: notes kept per kind, 1 … 64
  window: "7d"                           # optional, default "7d": the oldest note kept, 1h … 30d
  resurvey_after: "1h"                   # optional, default "1h": per place, 1m … 1d (Q-S31c-2)
```

- **Types.** `AuthoredSurvey { kinds: AuthoredKinds, depth: Option<u8>, window: Option<Span>, resurvey_after:
  Option<Span> }`, `AuthoredKinds { categories: Vec<Category> }`, both `deny_unknown_fields`, both deriving
  `Deserialize` and `JsonSchema`, each field with a doc comment (the generated reference prints it).
  `CARRIED_BY = [ContentKind::Person]`. `references()` returns nothing (categories are not entities).
- **Decode-time refusals** (a `try_from` or a custom `Deserialize` whose message names the key; the loader reports
  file, line and column):
  - `kinds.categories`: empty → "kinds.categories: at least one category"; more than 16 → "at most 16"; a
    duplicate → "kinds.categories: `food` is listed twice"; an invalid slug → item's `InvalidCategory` message;
  - `depth`: 0 or above 64 → "depth: 1 to 64, not <n>";
  - `window`: below 1 h or above 30 d → "window: 1h to 30d, not `<text>`";
  - `resurvey_after`: below 1 min or above 1 d → "resurvey_after: 1m to 1d, not `<text>`";
  - a span that does not parse → "`<key>`: a whole number followed by m, h or d, such as \"7d\", not `<text>`".
- **Defaults and their reasons** (realistic defaults, cited):
  - `depth: 32` and `window: "7d"` are the step's example (§9.3). Seven days covers one weekly cycle of a town's
    shopping; the U.S. BLS collects most CPI prices monthly or bimonthly per outlet, and USDA AMS Market News
    reports daily, so a week sits between a daily report and a monthly index (BLS Handbook of Methods, "Consumer
    Price Index: Data Sources"; USDA AMS, "Market News" program description).
  - `resurvey_after: "1h"`: a CPI data collector records an outlet once per visit, not continuously (same BLS
    source). One hour lets a reporter re-check a shop within one morning without flooding the book (F-S31c-3).
- **Seed.** One `survey-book-opened { surveyor, categories (sorted), depth, window_s, resurvey_after_s }`,
  `Visibility::SystemInternal`, `about: [surveyor]`. SystemInternal follows the ruled precedent for bookkeeping
  facts (Q-S31a-2, Q-S31b-2): the holder sees its book at once through the disclosed record, and no inbox or
  memory starts with a line nobody witnessed (SD-S31c-7).
- **Reduction of `survey-book-opened`** writes an empty `PriceBook` on the surveyor. It is refused
  (`FactRefusedByOwner`, which `mineworld validate` reports by name, A-15) when:
  - the subject is not a living Person → `PreconditionFailed`;
  - the person already has a book → `PreconditionFailed`;
  - a category matches no declared kind of the world (no Item with an `ItemKind` of that category) →
    `Rejection::System { code: survey-category-unknown }`, `detail` naming the category. A typo such as `fod`
    would otherwise make a surveyor that silently tracks nothing.

### 5.3 The book (`PriceBook`, component type `survey-book`)

```rust
/// A surveyor's notes: what it tracks, and the prices it saw. On a Person; written only by this pack's
/// reductions of `survey-book-opened` and `price-noted`.
pub struct PriceBook {
    categories: Vec<Category>,      // sorted, distinct, 1 … 16
    depth: u8,                      // 1 … 64
    window: u32,                    // seconds, 3 600 … 2 592 000
    resurvey_after: u32,            // seconds, 60 … 86 400
    kinds: Vec<KindNotes>,          // sorted by item; a kind appears once it has a note
    surveyed: Vec<Surveyed>,        // sorted by place; the last survey of each place
}
pub struct KindNotes { item: ItemId, notes: Vec<Note> }          // oldest first; at most `depth`
pub struct Note {
    at: WorldTime,                  // when it was seen
    place: PlaceId,                 // where
    seller: EntityId,               // a Person, or the Organization that runs the shop
    price: u64,                     // unit price in minor units, as disclosed (≥ 1)
    source: Source,                 // ask | sale
    in_stock: Option<u32>,          // a shop ask: the listing's count; a person's ask and every sale: None
}
pub enum Source { Ask, Sale }       // serde: "ask" | "sale"
pub struct Surveyed { place: PlaceId, at: WorldTime }
```

`owned_component! { component = PriceBook, owner = PriceSurveySystem, component_type = "survey-book",
schema_version = 1 }`. Every field has a `pub const fn` getter; there is no public constructor and no public
mutator (only this pack writes it, I-c-2).

**What the book keeps, and for how long.**

- **One ring per kind.** A kind's notes are appended in fact order. When a kind holds more than `depth` notes, the
  oldest are dropped until it holds `depth`. Notes of different kinds never evict each other.
- **The window.** At every reduction of `price-noted` (time `t`), every note of every kind with `at + window ≤ t`
  is dropped from state, and a kind left with no note is removed. `Surveyed` entries with `at + window ≤ t` are
  dropped too. So state stays bounded by `kinds × depth` notes plus one entry per place.
- **Lazy at disclosure.** The disclosed record is `PriceBook::visible_at(now)`: the same book with every note and
  `Surveyed` entry whose `at + window ≤ now` removed, `now = WorldRead::at()`. A note therefore leaves the
  surveyor's view exactly when it falls out of the window, with no wake and no fact, as S31-b's board does
  (I-S31-8's pattern). Example: with `window: "7d"`, a note at day 1 08:00 is visible at day 8 07:59:59 and gone
  at day 8 08:00:00.
- **Order is part of the shape.** Kinds by `ItemId`; notes oldest first, and within one fact in the fact's note
  order (§5.4); `surveyed` by `PlaceId`. No `HashMap`, no float.

**Disclosure.** To the holder only: `discloses(world, observer, subject)` returns `ComponentRecord::new::<PriceBook>
(subject, to_value(&book.visible_at(world.at())))` when `observer == subject` and the subject holds a book, and
nothing otherwise (I-S31-9, I-c-5). It is disclosed even when it holds no note, so a model can tell "I am a
surveyor with an empty book" from "I am not a surveyor".

**Size.** At the defaults, ten tracked kinds hold at most 320 notes (about 30 KB of JSON) in the holder's
observation. The bound is the author's: `depth` and the categories (R-S31c-2).

### 5.4 Facts

| Fact | Payload | Audience | Reduces to |
| --- | --- | --- | --- |
| `survey-book-opened` | `{ surveyor: PersonId, categories: [Category], depth: u8, window_s: u32, resurvey_after_s: u32 }` | `SystemInternal`, `about: [surveyor]` | an empty `PriceBook` (§5.2) |
| `price-noted` | `{ surveyor: PersonId, place: PlaceId, source: "ask" \| "sale", notes: [Noted { item: ItemId, seller: EntityId, price: u64, in_stock: Option<u32> }] }` | per **Q-S31c-1** (recommended `SystemInternal`), `about: [surveyor]` | notes appended to the surveyor's book (§5.3); for `source: ask`, `surveyed[place] = at` |

- `at` is the envelope's time. A witnessed note's cause is the `sold` it witnessed (`Causation::Event(sold)`,
  A-4): that is how AC-c-6 matches each note to its sale, with no sale id in the payload.
- `notes` is non-empty and sorted by `(item, seller)`; a `source: sale` fact has exactly one note.
- **Reduction re-checks, as the owner** (`ARC-26`): the surveyor holds a book; `notes` is non-empty and sorted;
  every item is a living Item whose `ItemKind` category the book tracks; every price is ≥ 1; a `sale` note has
  `in_stock: None` and exactly one note. A violation is `FactRefusedByOwner { system: price-survey }` with
  `PreconditionFailed` — a defect in whoever stated it, never a refused request.
- Event schema versions are 1. Both event type ids are new (`git grep '"price-noted"'` is empty on the base).

### 5.5 Witnessing (`witness.rs`; X-7)

```text
react(sold envelope e):
  sold     = decode e's payload with economy's Sold
  read     = world.read()
  where    = Whereabouts::from_world(&read)                 once per reaction (F-S31c-6)
  for (surveyor, book) in read.components::<PriceBook>()     EntityId order (A-16)
      if !presence::audience::admits(&where, e, surveyor)    continue     the audience rule itself (F-S31c-5)
      if !tracked(&read, book, sold.item())                  continue     the kinds filter
      emit price-noted { surveyor, place: sold.place(), source: sale,
                         notes: [ { item: sold.item(), seller: sold.seller(), price: sold.price(),
                                    in_stock: None } ] }
```

- `tracked(read, book, item)`: the item is a living Item, it has an `ItemKind`, and the kind's category is in
  `book.categories` (binary search).
- **One fact per surveyor**, not one fact per sale with many surveyors: each fact's `about` is one surveyor, and a
  reduction writes one book.
- **What the surveyor is in the sale does not matter.** A surveyor who is the buyer or the seller perceived the
  sale (it is a participant) and notes it, exactly as the audience rule says. AC-S31-10 asks for the perceived set,
  not "sales of others".
- `sold.count` is 1 in S31-a and S31-h. The note keeps the unit `price`, which is what a suggestion needs; a later
  multi-unit sale still yields one note at its unit price.
- **Removable** (I-S31-10): with no book in the world, the loop does nothing and nothing is stated.

### 5.6 Surveying (`action.rs`, `judge.rs`)

**Action.** `Survey {}` (action type `survey`, owner `price-survey`), payload `{}`. Target: none.
Requirement: `survey_requirement(place) = SpatialRequirement::at_place(place)` with `place` the surveyor's current
place (SD-S31c-4).

**`judge_survey(world, actor) -> Result<Plan { place, notes }, Rejection>`**, the one function `offers`,
`validate` and `resolve` call, in this order:

1. `actor` is a living Person holding a `PriceBook`, else `NoSupportedInteraction` (NotASurveyor).
2. `actor` has a `Presence`, else `PreconditionFailed`; `place` = its place.
3. `notes = asks_here(world, actor, place, book)`:
   - the shop listing, if economy discloses one: `EconomySystem.discloses(world, actor, place)`, the record whose
     component type is economy's `Shop` type, decoded as `Listing`; one note per listed kind that `tracked` admits,
     `seller` = the listing's operator, `in_stock = Some(listed.in_stock)`;
   - every living Person `p` whose `Presence` is in `place`, the actor included, in `EntityId` order:
     `EconomySystem.discloses(world, actor, p)`, the record of the `asks` view type, decoded; one note per ask that
     `tracked` admits, `seller = p`, `in_stock = None` (availability is not a count, A-8);
   - sorted by `(item, seller)`.
4. `notes` is empty → `Rejection::System { code: nothing-to-survey }`.
5. `book.surveyed` holds `place` at `t` and `now < t + resurvey_after` → `Rejection::System { code:
   surveyed-recently }` (Q-S31c-2). At exactly `t + resurvey_after` the survey is accepted again.

`validate` then evaluates `survey_requirement(place)` (SD-S31a-2's order: the pack's codes before the
requirement, so the offer and the dispatch answer the same).

**Resolve.** `judge_survey` again on `world.read()` (dispatch is sequential, so it sees the state `validate`
saw) and one `price-noted { surveyor: actor, place, source: ask, notes }`, audience per Q-S31c-1, `about:
[actor]`. A refusal in resolve is `FactRefusedByOwner` (unreachable after `validate`, as S31-a §5.7 argues).

**Offers** (`PerceptionProvider::offers(world, observer, target)`): with `target = None`, a living person
holding a book, and steps 1–4 passing: one **complete** `Survey {}` offer with `survey_requirement(place)`,
`refused(surveyed-recently)` while step 5 refuses it. Nothing in any other case: a non-surveyor is never offered
`survey`, and a surveyor where nothing is on display is not offered it. Complete is safe because of step 5: a
generic controller that takes it at random gets at most one survey per place per `resurvey_after` (Q-S31a-3,
F-S31c-3).

**Refusals, by name** (codes built with `RejectionCode::from_static`):

| Name | Code / reason | When |
| --- | --- | --- |
| NotASurveyor | kernel `NoSupportedInteraction` | the actor is not a living Person or holds no book |
| Unplaced | kernel `PreconditionFailed` | the actor has no `Presence` |
| NothingToSurvey | `nothing-to-survey` | no tracked ask is disclosed to the actor where it stands |
| SurveyedRecently | `surveyed-recently` | this place was surveyed less than `resurvey_after` ago |
| Malformed | kernel `PreconditionFailed` | a payload that does not decode as `Survey` |
| TooFarAway | kernel, from `SpatialRequirement::evaluate` | not reachable in practice: the requirement names the actor's own place; kept because the evaluator runs |

### 5.7 Ownership, persistence, determinism

- **One writer** (I-S31-2): only `price-survey` writes `PriceBook`, and only in reductions of
  `survey-book-opened` and `price-noted`. The pack writes no other pack's state and states no other pack's fact.
- **Persistence** (`ARC-25`): the book is reduced from logged facts, so a save holds it and a replay rebuilds it.
  Kill-and-resume equals an uninterrupted run because every `price-noted` is in the same revision as its cause
  (A-4). Nothing about the book is a view except the window filter of §5.3.
- **Determinism:** integer seconds and amounts, `EntityId` and `ItemId` order, `Vec` and `BTreeMap` only, no float,
  no draw, no wall clock.
- **Economy is read, never changed.** The survey calls economy's `discloses` (a read of economy's own judgement,
  SD-S31c-3); the witness rule decodes `Sold`. No economy behaviour, fact or version changes (I-c-1).

### 5.8 Both clients and the agent view

- **2D and 3D clients:** no change. A surveyor seat's complete `survey` offer appears in a client's generic
  interaction list like any offered action, and a client sends it as an intent; it evaluates nothing
  (`ENGINEERING_RULES.md` §§8–9). Words for `survey`, `nothing-to-survey` and `surveyed-recently` are S31-e's
  (R-S31e-c-1). The gate questions (`CLAUDE.md` §4): a Minecraft-like 3D client needs nothing new (the action is
  "be in the place"); both clients use the same server path. Yes and yes.
- **MCP agents:** an agent in a surveyor seat is offered the same complete `survey` and is disclosed the same book
  in its observation, through the same server path and rules (agents are first-class players). S31-g decides
  whether `MeLine` names the book (R-S31g-c-1). An agent never sees another person's book (I-c-5).

### 5.9 What this PR needs from economy after S31-a (the cross-PR read surface; checked at C0)

| Item | Needed shape | If missing after S31-a |
| --- | --- | --- |
| `mineworld_economy::Sold` | `pub` type with `pub const fn seller() -> EntityId`, `buyer() -> EntityId`, `item() -> ItemId`, `count() -> u32`, `price() -> u64`, `place() -> PlaceId`; `Deserialize`; `Event` impl | C2 adds the getters and the `lib.rs` re-export |
| `mineworld_economy::{Listing, Listed}` | as today: `operator()`, `listed()`, `item()`, `price()`, `in_stock()`; `Deserialize` (exists, A-7) | none expected |
| The person `asks` view type (S31-a §5.1 names it `AsksView` in `component.rs`) | `pub`, `Deserialize`, `Component` (its `COMPONENT_TYPE` is how the record is found), getters for each ask's `item()`, `price()`, `available()` | C2 adds getters / re-export |
| `EconomySystem: PerceptionProvider` | unchanged (exists) | — |

Any other change economy would need is a material stop (§12).

### 5.10 The read surface S31-d consumes (cross-PR interface)

S31-d's `price-reporter` model, S31-d's pricing policies (if they read it) and an LM backend all read the book
from the surveyor's own Observation. This is the whole contract:

1. **Where.** The observer's own `PerceivedEntity` (`observation.entity(observation.observer())`) carries one
   `ComponentRecord` whose component type is `survey-book`, when and only when the observer is a surveyor.
2. **Rust.** `mineworld_rule_controller::read::disclosed::<mineworld_price_survey::PriceBook>(me)` decodes it
   (A-20). S31-d depends on the `mineworld-price-survey` crate for the type (the cognition → System Pack direction
   `ARC-35` check 2 allows; never the reverse).
3. **Accessors** (all `pub const fn` or `pub fn`, none mutating):

   ```rust
   impl PriceBook {
       pub fn categories(&self) -> &[Category];
       pub const fn depth(&self) -> u8;
       pub const fn window(&self) -> u32;            // seconds
       pub const fn resurvey_after(&self) -> u32;    // seconds
       pub fn kinds(&self) -> &[KindNotes];          // by ItemId; only kinds with ≥ 1 visible note
       pub fn kind(&self, item: ItemId) -> Option<&KindNotes>;
       pub fn surveyed(&self) -> &[Surveyed];        // by PlaceId
       pub fn last_surveyed(&self, place: PlaceId) -> Option<WorldTime>;
   }
   impl KindNotes { pub const fn item(&self) -> ItemId; pub fn notes(&self) -> &[Note]; }  // oldest first
   impl Note {
       pub const fn at(&self) -> WorldTime; pub const fn place(&self) -> PlaceId;
       pub const fn seller(&self) -> EntityId; pub const fn price(&self) -> u64;
       pub const fn source(&self) -> Source; pub const fn in_stock(&self) -> Option<u32>;
   }
   impl Surveyed { pub const fn place(&self) -> PlaceId; pub const fn at(&self) -> WorldTime; }
   pub enum Source { Ask, Sale }
   ```

4. **JSON** (what an LM prompt or a Python reader sees; field names and order are the contract):

   ```json
   { "categories": ["drink", "food"], "depth": 32, "window": 604800, "resurvey_after": 3600,
     "kinds": [ { "item": 41, "notes": [
         { "at": 25800, "place": 7, "seller": 30, "price": 300, "source": "sale", "in_stock": null },
         { "at": 26400, "place": 7, "seller": 30, "price": 300, "source": "ask",  "in_stock": 2 } ] } ],
     "surveyed": [ { "place": 7, "at": 26400 } ] }
   ```

   `WorldTime` serializes as integer seconds and ids as integers, as every pack's payloads do.
5. **Semantics S31-d may rely on.** Every note was perceived by the holder (I-c-3). Every visible note satisfies
   `now − window < at ≤ now`. A kind holds at most `depth` notes. `in_stock` is `Some` only for a shop ask. Ask
   notes of one seller may repeat across surveys (one per survey, at most one survey per place per
   `resurvey_after`), so a statistic that must count each seller once dedups by `(seller, place)` itself.
6. **The action.** S31-d's model attempts the complete `survey` affordance it is offered (it may also build
   `mineworld_price_survey::Survey` and attempt it by name, `ARC-34`). The codes `nothing-to-survey` and
   `surveyed-recently` are `pub const` `RejectionCode`s in `mineworld_price_survey::codes`.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-c-1 (I-S31-1) | No path under `kernel/`, `contracts/`, `persistence/`, `server/`, `clients/`, `cognition/` changes; under `systems/`, only `price-survey`, `installed`, and economy's read surface of §5.9 (visibility and getters only: `git diff` of `systems/economy/src` adds `pub`, getters and re-exports and changes no function body) | C6 diff scan |
| I-c-2 (I-S31-2) | Only `price-survey` writes `PriceBook`, only in reductions of its own two facts | `declaration()` review; `book.rs` |
| I-c-3 (AC-S31-10) | A `price-noted { source: sale }` exists for surveyor `s` and sale `e` if and only if `admits(fold past e, e, s)` and `e`'s item is tracked by `s` | `witness.rs`; AC-c-6 against `perceived_by` over the real log; X-7 |
| I-c-4 | A survey copies only what economy discloses to the surveyor where it stands, and nothing else | `survey.rs`; M-c-1 |
| I-c-5 (I-S31-9) | A book is disclosed to its holder only | `book.rs::only_the_holder_sees_its_book`; M-c-4 |
| I-c-6 (I-S31-9) | No note discloses a person seller's holdings count (`in_stock` is `None` for a person's ask) | `survey.rs`; M-c-1 |
| I-c-7 | A kind holds at most `depth` notes; no visible note is older than `window` | `book.rs`; M-c-2, M-c-3 |
| I-c-8 (Q-S31a-3) | Taking the complete `survey` at random yields at most one survey per place per `resurvey_after` | `survey.rs`; M-c-5 |
| I-c-9 (I-S31-10) | A world without `price-survey`, or with it and no section, states nothing new; every reference world's baselines stay byte-identical | AC-c-9 |
| I-c-10 | `offers`, `validate` and `resolve` call the same `judge_survey`; an offer's verdict equals the dispatch's | `survey.rs::offer_and_dispatch_agree` |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | fmt, clippy `-D warnings`, the `RejectionCode::from_static` literals | CI `fast` |
| Unit | `Span` parse and bounds; section decode refusals at line and column; `PriceBook` ring and window arithmetic at its edges | `systems/price-survey/tests/{section,book}.rs` |
| Integration (pack; real kernel dispatch over the hand-built W1 with presence, movement, item, inventory, economy installed) | AC-c-1 … AC-c-5; X-7, M-c-1 … M-c-6; offers ≡ dispatch | `tests/{section,witness,survey,book}.rs` |
| Real binary | AC-c-6 … AC-c-9 on W2, W3, W4; the W5 arms of AC-c-2, AC-c-3 and AC-c-6 (revision 2) | `tools/cli/tests/price_survey.rs` |
| Long | AC-c-9's "no reference baseline moves": the PR's CI `parity` group, or the milestone run (speed-first) | CI |
| Gate 1 (real LM) | NOT REQUIRED: no LM-facing surface changes (the book is a disclosed record; LM reading is S31-d's cassette) | — |
| Gate 2 (real client) | NOT REQUIRED: no client change | — |

Oracles are literal prices, times, sellers and places written in each test, presence's `perceived_by` over the
real log, or the stored facts; never a value `price-survey` reports about itself (test rules §25).

---

## 8. Acceptance criteria (fixed before measuring, `ARC-23`)

**Worlds.**

- **W1 (pack fixture, hand-built, `tests/support/mod.rs`).** Places `store`, `cafe`, `street`, one passage each
  from the street. Kinds `apple` (food), `bread` (food), `juice` (drink), `pen` (goods). `corner-store` runs the
  store: asks apple 100, bread 300, juice 200, pen 150; holds apple 3, bread 2, pen 2; juice 0. `grace` (a person,
  in the store, 2 000 mm from the counter's origin) holds bread 2 and asks bread 260 and pen 120. `pia` is the
  surveyor (`kinds: { categories: [food, drink] }`, `depth: 4`, `window: "1d"`, `resurvey_after: "1h"`, wallet 2 000).
  `alice` and `bob` are buyers with wallets 2 000. `nat` is a second person with no section. Pia starts in the store
  unless a test moves her.
- **W2 (Market Town copy).** A scratch copy of `worlds/market-town` (A-21's pattern) whose `world.yaml` appends
  `  - price-survey` after `weather`, and whose `people/carol.yaml` appends `price-survey: { kinds: { categories:
  [food, drink] } }`. Nothing else differs (asserted by a file diff, as `market_composition.rs` asserts its
  copies).
- **W3 (W2, routine moved).** W2 with Carol's routine line `{ from: "07:00", place: store, label: shopping }`
  replaced by `{ from: "07:00", place: park, label: shopping }`. Nothing else differs.
- **W4 (Social Café copy).** A scratch copy of `worlds/social-cafe` whose `world.yaml` enables `price-survey` and
  not economy.
- **W5 (a generated second world; revision 2, charter 2026-10-18: "every acceptance criterion names at least one
  second world or a generated variant").** A World Pack the test writes from scratch: two shops in two places
  (`bakery`, `kiosk`) joined by a `square`, two operating organizations, three people with keys and names of the
  test's own (`ines` the surveyor, `tomas` and `wen` the buyers), and three item kinds (`rye`, `tea`, `soap` —
  categories `food`, `drink`, `goods`), each with its own asks. Only `presence`, `movement`, `item`, `inventory`,
  `economy` and `price-survey` are enabled. Nothing in it shares a key, a name or an amount with Market Town or
  with W1, so a rule that holds only for the default town fails here. AC-c-2 (a), (b), (d), AC-c-3 (a) and AC-c-6's
  equality are checked on W5 too, through the real binary (`tools/cli/tests/price_survey.rs`); `git grep` over
  `systems/price-survey` for any Market Town key (`carol`, `grace`, `store`, `cafe`, `bread`) is empty.

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-c-1 | **Section and genesis (W1 and decode cases).** Pia's genesis states one `survey-book-opened`, `SystemInternal`, and her observation of herself holds a `survey-book` record with no kinds and her four settings. Each decode refusal of §5.2 fails at its line naming the key (categories empty, 17 categories, a duplicate, `Food`; depth 0 and 65; window `"59m"`, `"31d"`, `"7x"`; resurvey_after `"0m"`, `"2d"`; an unknown key). A section naming `categories: [fod]` fails reduction with `survey-category-unknown` naming `fod`. The section on an item file is refused by the loader naming the file (worldpack refusals) | §9.3 |
| AC-c-2 | **Witness = audience (W1).** (a) Pia in the store, Alice buys bread from the store at 300: exactly one `price-noted { source: sale }` for Pia, caused by that `sold`, in the same revision, with note `{ bread, corner-store, 300, None }`. (b) Pia on the street, Alice buys apple in the store: no `price-noted`. (c) Pia herself buys an apple from the store: noted (a participant perceives its own sale). (d) Bob buys a `pen` from the store with Pia present: no `price-noted` (goods untracked). (e) Alice buys bread from Grace at 260 with Pia present: noted with seller `grace`, price 260. (f) Nat (no book) is present at every sale: no fact names Nat | AC-S31-10; X-7 |
| AC-c-3 | **Survey (W1).** (a) Pia in the store surveys at t: one `price-noted { source: ask }` whose notes are, in order, `apple/corner-store/100/Some(3)`, `bread/corner-store/300/Some(2)`, `bread/grace/260/None`, `juice/corner-store/200/Some(0)` — no `pen` (untracked), and Grace's note has no count. (b) Again at t + 3 599: refused `surveyed-recently`, offer refused with the same code, state unchanged. (c) At t + 3 600: accepted. (d) Pia on the street (nothing on display): no `survey` offer; dispatch refused `nothing-to-survey`. (e) Nat attempts `survey`: `NoSupportedInteraction`, never offered. (f) A malformed payload: `PreconditionFailed`. (g) In every state of (a)–(f), the offer's verdict equals the dispatch's | §9.3; Q-S31c-2 |
| AC-c-4 | **Bounds (W1, `depth 4`, `window 1d`).** Six bread notes in time order leave the four newest. A note at t is in the disclosed book at t + 86 399 and absent at t + 86 400, with no fact between; the next `price-noted` at or after t + 86 400 removes it from state. Notes of `apple` never evict `bread` notes | §9.3 "depth", "window" |
| AC-c-5 | **Privacy (W1).** Alice's, Grace's and Nat's observations of Pia hold no `survey-book`; Pia's observation of herself does. No note anywhere holds a person's count | I-S31-9 |
| AC-c-6 | **The reporter knows only what it perceived (W2, 30 days, seed 7).** Let `S` = the `sold` facts of tracked kinds in `perceived_by(log, carol, None)` and `N` = the causes of every `price-noted { source: sale }` whose surveyor is Carol. Then `S = N` as sets of event ids, every note's `{ item, seller, price, place }` equals its `sold`'s, and each note shares its cause's revision. Non-vacuity, fixed now: `S` holds at least one store sale and one café sale, and the world's log holds at least one tracked sale Carol did not perceive. `mineworld perceived <W2> --save S --person carol --json` lists exactly the `sold` ids of `S` (the binary path agrees with the library) | **AC-S31-10** (first sentence, as Q-S31c-3 words it) |
| AC-c-7 | **Moving the routine removes store sales (W3, 30 days, seed 7).** No `price-noted { source: sale }` of Carol has `place = store`, while the log holds at least one store sale. AC-c-6's equality holds in W3 too | **AC-S31-10** (second sentence) |
| AC-c-8 | **Persisted and replayed (W2's save).** `mineworld replay <W2> --save S` passes. The save's newest snapshot holds Carol's `PriceBook` equal to the one the test rebuilds by folding W2's `survey-book-opened` and `price-noted` facts up to that snapshot's revision with §5.3's rules written out in the test (a mirror, as `market/mod.rs` mirrors economy) | step §13; `ARC-25` |
| AC-c-9 | **Optional and removable.** Every reference world's keys in the CI `parity` record equal `scripts/baselines.txt` on `main` (no reference world enables the pack). W4 fails `mineworld validate` naming the missing dependency `economy`. W2 with Carol's section deleted (the pack still enabled) runs 30 days and states no fact of `price-survey` and offers nobody `survey` | AC-S31-9; I-S31-10 |
| AC-c-10 | **No kernel, no contract.** I-c-1's diff scan is empty | I-S31-1 |

### 8.1 Planted mutations (each must turn its test red by name, then be restored from a byte copy)

| Id | Plant | Must fail |
| --- | --- | --- |
| X-7 | `witnesses` skips the `admits` check and notes every tracked `sold` for every surveyor | AC-c-2 (b) (pack) and AC-c-6 / AC-c-7 (binary, run once in the PR or at the milestone) |
| M-c-1 | `asks_here` reads a person seller's `Holdings` count into `in_stock` | AC-c-3 (a) (Grace's note has `Some(2)`), AC-c-5 |
| M-c-2 | the ring never truncates to `depth` | AC-c-4 (six notes kept) |
| M-c-3 | `visible_at` returns the book unfiltered | AC-c-4 (the note visible at t + 86 400) |
| M-c-4 | `discloses` returns the book to any observer | AC-c-5 |
| M-c-5 | `judge_survey` skips step 5 | AC-c-3 (b) |
| M-c-6 | `tracked` returns true for every kind | AC-c-2 (d), AC-c-3 (a) (a `pen` note appears) |

---

## 9. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **LLM logic review** separately; `[x]`
needs the work and recorded evidence; an inapplicable item is `N/A` with the audited reason.

### 9.0 The procedure for every commit (Haiku implementer: follow it literally)

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s31c`. Never `cd` elsewhere.

1. Re-read this commit's section and every design section it cites (§5.x). Re-read every file you will edit, in
   full, before editing it.
2. Make the numbered steps, in order. Do not do a later commit's work. If a step names a function or a file that
   does not exist on the base, stop and record it (§12 material stops) — do not invent one.
3. Run the commit's validation. Record per command: test count, wall time, PASS / FAIL / INCONCLUSIVE.
4. Mutations listed for the commit: copy the file aside (`cp f f.orig`), plant the change, run the named test,
   record the red (test name and assertion), restore (`cp f.orig f`, `cmp f f.orig` clean, delete the copy), run
   green again. Pack-level mutations run in the commit (seconds). Binary-level ones run once in C5 if a slot is
   free, else at the milestone (speed-first).
5. Static: `cargo fmt --all --check`; `cargo clippy -p mineworld-price-survey --all-targets -- -D warnings` (and
   `-p mineworld-economy` if C2 touched it, `-p mineworld-installed-systems` in C2). Markdown:
   `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`. **R-LOAD-1:** never
   `cargo test --workspace`, never all of `tools/cli`; one `--test <file>` at a time; any `tools/cli` run goes
   through `/Users/yuema137/mineworld-worktrees/cast-slots/with-slot.sh`.
6. Review: answer the commit's review questions (listed under it) in one or two sentences each, in §15.
7. `git status --short` and `git diff --stat`: every changed path is in the commit's **Files**; otherwise revert
   it or stop and record why.
8. Update §15 (ledger row, evidence, findings) and `handoff-s31c.md` in the same commit.
9. Stage by explicit path (never `git add -A`); `git commit -m "<message>"` ending with the line
   `Co-Authored-By: Claude Haiku 5.5 <noreply@anthropic.com>` (or the session's own attribution line). No heredoc
   writes, no `sed -i`, no `python3 -c`, no `curl`.
10. Stop and report a material stop (§12) instead of improvising.

#### C0 — Base, prerequisites, ledger (Markdown only)

- **Goal:** prove the start condition and the read surface before any code.
- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s31c.md` (new).
- **Steps:**
  - [ ] 1. Record `git rev-parse origin/main` as the base. Quote the dispatch that confirms Milestone F's checklist
    was handed over and this design frozen; either missing → material stop.
  - [ ] 2. Confirm S31-a merged (`git log origin/main --oneline | grep -i "S31-a"`, and `systems/economy/src/sale.rs`
    exists). Not merged → material stop.
  - [ ] 3. Audit §5.9 on the merged economy: open `systems/economy/src/{event.rs, component.rs, lib.rs}`; for each
    row of §5.9 record "present" or "missing: <what>". Record the `asks` view's real type name and component type
    id, and `Sold`'s field names. If `Sold`'s payload differs from A-3 in a way §5.5 cannot read, material stop.
  - [ ] 4. Record whether S31-b, S31-h and WE-a have merged; choose §2.3's branch for each.
  - [ ] 5. Re-read A-1 … A-24 on the base; record moved line numbers (bounded); stop if a statement changed.
  - [ ] 6. Record where the decision log lives (`docs/decisions/` expected, A-24).
- [ ] Implementation. [ ] Validation: the two doc scripts. [ ] Review: §2.3 and §5.9 hold on the base.
- **Commit message:** `docs(plan): S31-c — open the ledger at the base`

#### C1 — Specifications first

- **Goal:** the decision exists before the code it governs (`CLAUDE.md` §2.2).
- **Files:** `docs/decisions/ARC-116.md` (new) and `docs/decisions/INDEX.md` (regenerated) — or
  `docs/DECISIONS.md` if C0 found the single-file log (SD-S31c-12); `docs/MODULE_SPEC.md`.
- **Steps:**
  - [ ] 1. Write `ARC-116` in the log's record format (copy the headings of `docs/decisions/ARC-115.md` if it
    exists, else of the newest record): **problem** (step §0.1 items 2–3; F-S31-5: a stateless model cannot
    remember, and two backends must see the same input); **options** (S28 memory as the notebook — declined, the
    backends would differ; a public trade tape — declined, step §9.2; reported trades — later; **a
    perception-derived book owned by `price-survey`** — chosen); **choice** (§5.2 … §5.6 as ruled by §14: the
    section, the book's shape and bounds, the witness rule through presence's `admits`, the survey through
    economy's `discloses`, the cooldown, the facts' audience); **consequences** (holder-only; removable; the
    `survey-book` name vs WE-a's `price-book`; the span grammar shared with S31-b by convention). End with a
    paragraph headed **Backend half (S31-d)** that says only: "Written by S31-d as a dated note: the reporter as a
    seat whose backend is its decision model."
  - [ ] 2. `MODULE_SPEC.md` §4.1: a row after `economy`, in the table's fixed-width form:
    `price-survey price-survey people  { kinds: { categories: [<category>] }, depth?, window?, resurvey_after? }`
    with the bounds and defaults of §5.2, "Makes the person a surveyor whose price book holds the asks it surveyed
    and the sales it perceived. Disclosed to the holder only (`ARC-116`)".
  - [ ] 3. `python3 scripts/check_decision_ids.py --write-index` (only if `docs/decisions/` exists).
- **Validation:** `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`.
- [ ] Implementation. [ ] Validation. [ ] Review: no new ontology term (`CLAUDE.md` §2.1 rule 3: "surveyor" and
  "price book" are pack vocabulary); every code of §5.2 and §5.6 appears in `ARC-116`; nothing contradicts §11.
- **Commit message:** `docs: the price book — ARC-116, the price-survey pack (S31-c)`

#### C2 — The pack, its section, the book, its disclosure, the install

- **Goal:** a surveyor exists and sees its empty book; nothing writes notes yet.
- **Files:** `systems/price-survey/{Cargo.toml, README.md (stub: one paragraph and the pack table), src/{lib.rs,
  span.rs, section.rs, component.rs, event.rs, codec.rs, system.rs}, tests/{support/mod.rs, section.rs, book.rs}}`;
  `systems/installed/{Cargo.toml, src/lib.rs}`; `Cargo.lock`; `docs/reference/*` as regenerated;
  `worldpack/tests/refusals.rs`; and, only if C0 step 3 recorded a gap, `systems/economy/src/{event.rs,
  component.rs, lib.rs}` (getters and re-exports only).
- **Steps:**
  - [ ] 1. `Cargo.toml` with §5.1's dependencies, each with its one-line comment. Copy the `[package]` block shape
    from `systems/economy/Cargo.toml`; `description` = "MineWorld's price-survey System Pack: a surveyor's price
    book of the asks it surveyed and the sales it perceived, disclosed to the surveyor only."
  - [ ] 2. `span.rs`: `pub struct Span(u32)` (seconds) with `Span::parse(text, min, max, key) -> Result<Span,
    String>`; grammar: one or more ASCII digits, then exactly one of `m` (×60), `h` (×3 600), `d` (×86 400); no
    sign, no space, no fraction; overflow → the same refusal; the messages of §5.2. Unit tests inside the file:
    `"1m"`, `"59m"`, `"1h"`, `"7d"`, `"30d"`, `"0h"`, `"1.5h"`, `"h"`, `"7"`, `"7D"`, `" 7d"`, `"99999999999d"`.
  - [ ] 3. `section.rs`: `AuthoredSurvey`, `AuthoredKinds` (§5.2); the bounds as custom `Deserialize`
    (`#[serde(try_from = "...")]` on a raw mirror struct), each message naming its key; `JsonSchema` with an
    `example`; `impl AuthoredSection for PriceSurveySystem` with `SECTION = SectionName::from_static
    ("price-survey")`, `CARRIED_BY = &[ContentKind::Person]`, `seed` stating one `survey-book-opened` (§5.2) with
    `Visibility::SystemInternal` and `.about(vec![subject])`; refuse a non-person subject with
    `PreconditionFailed` (economy's `seed` shows the pattern, `systems/economy/src/section.rs:97-148`).
  - [ ] 4. `event.rs`: `SurveyBookOpened`, `PriceNoted`, `Noted`, `Source` (serde `rename_all = "snake_case"`),
    each `impl Event` (`"survey-book-opened"`, `"price-noted"`, owner `PriceSurveySystem::ID`, schema 1); getters;
    constructors `pub(crate)`.
  - [ ] 5. `component.rs`: `PriceBook`, `KindNotes`, `Note`, `Surveyed` (§5.3) with serde in the field order of
    §5.10 item 4; `owned_component!` with `"survey-book"`; getters of §5.10 item 3; `pub(crate) fn
    opened(..) -> Self`, `pub(crate) fn note(&mut self, at, place, source, notes: &[Noted])` (append per kind,
    truncate to depth from the front, then prune every kind and `surveyed` by `at + window ≤ at_now`, then for
    `Ask` set `surveyed[place] = at`), and `pub fn visible_at(&self, now: WorldTime) -> PriceBook`.
  - [ ] 6. `system.rs`: `PriceSurveySystem` (`Default`), `SystemIdentity` (`"price-survey"`), `SystemPack`
    (`package!()`, `owns_section!()`), `System` with §5.1's declaration (without `providing::<Survey>()` and
    `subscribing_to::<Sold>()` yet — they arrive in C4 and C3), `install` (`tables.component::<PriceBook>()`),
    `react` for `SurveyBookOpened` (§5.2's reduction, with `survey-category-unknown`) and for `PriceNoted` (§5.4's
    re-checks, then `PriceBook::note`). `PerceptionProvider::discloses` (§5.3); `offers` returns nothing yet.
    `codes` module: `pub const SURVEY_CATEGORY_UNKNOWN`, `NOTHING_TO_SURVEY`, `SURVEYED_RECENTLY` via
    `RejectionCode::from_static`.
  - [ ] 7. Economy read surface, only for the gaps C0 recorded (§5.9): add `pub const fn` getters and `pub use`
    re-exports. Change no function body.
  - [ ] 8. Install: `mineworld-price-survey = { path = "../price-survey" }` in `systems/installed/Cargo.toml`;
    `PriceSurvey => mineworld_price_survey::PriceSurveySystem,` as the last line of `installed!`.
  - [ ] 9. `tests/support/mod.rs`: W1 (§8), built like `systems/economy/tests/support/mod.rs` (items' sections,
    then organizations', then people's, each through its owner's `AuthoredSection::seed`), with helpers `world()`,
    `world_with(pia_section: &str)`, `observe(world, who)`, `submit(world, who, payload)`, `facts_of::<E>(world)`.
  - [ ] 10. `tests/section.rs`: AC-c-1's decode cases and genesis; `tests/book.rs`: AC-c-4 by stating
    `price-noted` facts through a test-only path — if the kernel offers no way to state a fact from a test, drive
    `PriceBook::note` directly in `book.rs` for the ring and window arithmetic and leave the end-to-end arm to C4;
    `only_the_holder_sees_its_book` (AC-c-5's book half, M-c-4).
  - [ ] 11. `worldpack/tests/refusals.rs`: one case — `price-survey:` on an item file refused naming the file and
    line.
  - [ ] 12. Regenerate the reference: `target/debug/mineworld reference --out docs/reference` (after `cargo build
    -p mineworld-cli` through `with-slot.sh`).
- **Validation:** `cargo test -p mineworld-price-survey --test section`; `--test book`; `cargo test -p
  mineworld-price-survey --lib` (span); `cargo test -p mineworld-installed-systems`; `cargo test -p
  mineworld-worldpack --test refusals`; `target/debug/mineworld reference --check`. Mutations M-c-2, M-c-3, M-c-4.
- [ ] Implementation: steps 1–12. [ ] Validation, M-c-2, M-c-3, M-c-4. [ ] Review: the genesis fact is
  `SystemInternal`; `PriceBook` has no public mutator; no `HashMap`, no float; economy's diff (if any) adds getters
  only.
- **Commit message:** `feat(price-survey): surveyors and their price books, from the person's section (S31-c)`

#### C3 — Witnessing sales through presence's audience rule

- **Goal:** a surveyor notes exactly the tracked sales it perceived (AC-S31-10's mechanism).
- **Files:** `systems/price-survey/src/{witness.rs (new), judge.rs (new: `tracked` only), system.rs, lib.rs}`,
  `systems/price-survey/tests/witness.rs` (new).
- **Steps:**
  - [ ] 1. `judge.rs`: `pub(crate) fn tracked(world, book, item) -> bool` (§5.5).
  - [ ] 2. `witness.rs`: `pub(crate) fn witnesses(world: &WorldRead, sold_envelope: &EventEnvelope, sold: &Sold)
    -> Vec<Emission>` exactly as §5.5's pseudocode: one `Whereabouts::from_world`, `components::<PriceBook>()` in
    order, `admits`, `tracked`, one `price-noted { source: sale }` per surveyor with §5.4's audience (as ruled by
    Q-S31c-1) and `.about(vec![surveyor])`.
  - [ ] 3. `system.rs`: add `.subscribing_to::<Sold>()`; in `react`, a `Sold` envelope → decode with economy's type
    → `Ok(witnesses(..))`.
  - [ ] 4. `tests/witness.rs`: AC-c-2 (a)–(f), each asserting the noted fact (or its absence), its cause
    (`Causation::Event(sold id)`), the same revision, and Pia's disclosed book after it. If S31-h has merged, one
    more case: an accepted proposal witnessed is noted at the proposed price.
- **Validation:** `cargo test -p mineworld-price-survey --test witness`. Mutations X-7, M-c-6.
- [ ] Implementation: steps 1–3. [ ] Validation: step 4, X-7, M-c-6. [ ] Review: the rule calls `admits` and no
  hand-written "same place" check exists (`git grep -n "place_of\|== place" systems/price-survey/src` shows only
  `Whereabouts` use); `react` reads, never writes, another pack's state; F-S31c-6's argument still holds for the
  code as written (the `Whereabouts` is built from `world.read()` inside `react`, not cached across reactions).
- **Commit message:** `feat(price-survey): a surveyor notes the sales it perceived, and only those (S31-c)`

#### C4 — The `survey` action, its offer, the cooldown

- **Goal:** a surveyor can copy the asks on display into its book, at most once per place per `resurvey_after`.
- **Files:** `systems/price-survey/src/{action.rs (new), judge.rs, codec.rs, system.rs, lib.rs}`,
  `systems/price-survey/tests/survey.rs` (new).
- **Steps:**
  - [ ] 1. `action.rs`: `Survey {}` (`Serialize`, `Deserialize`, `Action` with `"survey"`, owner
    `PriceSurveySystem::ID`), `pub const fn survey_requirement(place: PlaceId) -> SpatialRequirement` =
    `SpatialRequirement::at_place(place)`.
  - [ ] 2. `judge.rs`: `asks_here` and `judge_survey` exactly as §5.6 steps 1–5, using `EconomySystem.discloses
    (world, actor, subject)` and the component type ids recorded at C0 to find the two economy records.
  - [ ] 3. `system.rs`: `.providing::<Survey>()`; `validate` = decode (`PreconditionFailed` on failure),
    `judge_survey`, then `survey_requirement(place).evaluate(..)`; `resolve` = `judge_survey` again, one
    `price-noted { source: ask }` (audience per Q-S31c-1); `offers` per §5.6.
  - [ ] 4. `tests/survey.rs`: AC-c-3 (a)–(g); `offer_and_dispatch_agree` over the states of (a)–(f); AC-c-4's
    end-to-end arm (six surveys an hour apart with `depth: 4`, then the window edge); AC-c-5's "no person count".
- **Validation:** `cargo test -p mineworld-price-survey --test survey`; rerun `--test witness` and `--test book`.
  Mutations M-c-1, M-c-5.
- [ ] Implementation: steps 1–3. [ ] Validation: step 4, M-c-1, M-c-5. [ ] Review: the survey reads economy only
  through `discloses` (no `component::<Holdings>` or `component::<Asks>` in `systems/price-survey/src`); the offer
  is complete and refused, never absent, during the cooldown; `judge_survey` is the only decision.
- **Commit message:** `feat(price-survey): survey the asks on display, once per place per interval (S31-c)`

#### C5 — The real-binary checkpoint

- **Goal:** AC-S31-10 on a real world, judged by presence's own fold over the real log.
- **Files:** `tools/cli/tests/price_survey.rs` (new).
- **Steps:**
  - [ ] 1. `copy_market_with_surveyor(name, routine_to_park: bool) -> Scratch`: copy Market Town (the
    `market_composition.rs` `copy_dir` pattern; copy the helper into this file rather than share it), append the
    `price-survey` line to `systems:` and the section to `people/carol.yaml`, for W3 replace Carol's `07:00` line;
    assert by a file comparison that nothing else differs.
  - [ ] 2. `fn facts(save) -> Vec<EventEnvelope>` (the `client_2d.rs:47-53` pattern).
  - [ ] 3. `reporter_knows_only_what_it_perceived` (AC-c-6): run W2 30 days seed 7 with `--save`; compute `S` with
    `mineworld_presence::audience::perceived_by` filtered to `sold` of tracked kinds (a kind is tracked when its
    item file's `item.category` is food or drink — read from the copy's `items/*.yaml` text, an independent
    oracle); compute `N` from `price-noted` facts read by slug and JSON field (no dependency on the pack's types);
    assert `S = N`, payload agreement, same revision, non-vacuity; run `mineworld perceived … --person carol
    --json` and compare its `sold` ids with `S`.
  - [ ] 4. `moving_the_routine_removes_store_sales` (AC-c-7) on W3.
  - [ ] 5. `book_is_persisted_and_replayed` (AC-c-8): `mineworld replay`; rebuild Carol's book from the facts with
    a mirror of §5.3's rules written in the test; compare with the snapshot's `survey-book` record.
  - [ ] 6. `optional_and_removable` (AC-c-9's W4 and removal arms).
- **Validation:** `with-slot.sh cargo test -p mineworld-cli --test price_survey` once (four 30-day runs; record
  wall time, INCONCLUSIVE for timing by R-LOAD-1). X-7 on the binary path if a slot is free; otherwise recorded as
  "run at the milestone". AC-c-9's baseline arm comes from CI's `parity` record or the milestone run.
- [ ] Implementation: steps 1–6. [ ] Validation. [ ] Review: the oracle is presence's fold and the item files, not
  the pack; the non-vacuity floors were written before the first run and are not tuned after it (`ARC-23`).
- **Commit message:** `test: a surveyor's book holds exactly the sales it perceived, in a real town (S31-c)`

#### C6 — Close-out

- **Files:** this file (§15), `handoff-s31c.md`, `systems/price-survey/README.md`.
- **Steps:**
  - [ ] 1. README: what the pack owns, the section with its bounds, the facts, the action and codes, what it
    discloses to whom, its dependencies, a link to `ARC-116`.
  - [ ] 2. AC-c-10's diff scan: `git diff --name-only origin/main...HEAD` lists nothing under I-c-1's forbidden
    paths, and `git diff origin/main...HEAD -- systems/economy/src` changes no function body.
  - [ ] 3. Fill §8's evidence; push `mvp0/pr-s31c`; open the PR (body ends with the Claude Code line); CI `fast` on
    the exact head green; repair to green.
- [ ] Implementation. [ ] Validation: `fast` green on the exact head. [ ] Review: deviations recorded; §13's edits
  elsewhere listed for the lane lead.
- **Commit message:** `docs(plan): S31-c close-out`
- **Then:** MERGE-READY: report the PR number and head SHA. The coordinator merges once `fast` is green; the
  implementation session never merges.

---

## 10. Requirements this PR places on other PRs (cross-PR interfaces)

- **R-S31a-c-1 (on S31-a, read surface).** §5.9: `Sold` with `pub` getters and a `lib.rs` re-export; the person
  `asks` view type public, `Deserialize`, with getters; `Listing`/`Listed` as today. If S31-a's implementation
  is still open when this design freezes, the lane lead asks S31-a's implementer to include them; otherwise S31-c
  C2 adds them (visibility only).
- **R-S31d-c-1 (on S31-d, the consumer).** S31-d reads the book only through §5.10: the `survey-book` record on the
  observer, decoded with `mineworld_price_survey::PriceBook`. It never reads S28 memory, the `perceived` stream or
  another component for prices, so the deterministic and the LM backends see the same input (AC-S31-11). It dedups
  repeated ask notes itself if its statistic needs it (§5.10 item 5). It attempts the complete `survey` offer (or
  `Survey` by name) and handles `nothing-to-survey` and `surveyed-recently`. It writes `ARC-116`'s backend half as a
  dated note under the record this PR writes.
- **R-S31f-c-1 (on S31-f).** Market Town's reporter (Q-S31-4: an existing seated person without a job, not the
  S29-b seller) carries `price-survey: { kinds: { categories: [food, drink] } }` and a routine through the café,
  the store and the street; `price-survey` is appended after `weather` in `systems:` (it depends on economy, item
  and presence). AC-c-6's W2 is the measured precedent.
- **R-S31e-c-1 (on S31-e).** Words for the action `survey` and the codes `nothing-to-survey` and
  `surveyed-recently` in `en.po` / `zh_Hans.po`. No client logic.
- **R-S31g-c-1 (on S31-g).** If the agent view names the book, it renders the `survey-book` record of §5.10 and
  infers nothing; `inspect` of another person never shows one (I-c-5).
- **R-S31h-c-1 (on S31-h).** An accepted proposal states the same `sold` (step §5.9); no other shape is needed by
  this pack. A change to `sold`'s audience or payload is a material stop for both.

## 11. Bounded decisions (recorded; not operator questions)

| Id | Decision | Why |
| --- | --- | --- |
| SD-S31c-1 | Component type id `survey-book`; Rust type `PriceBook` | F-S31c-7; R-S31c-a-1 |
| SD-S31c-2 | The witness rule calls presence's `admits` with `Whereabouts::from_world` | F-S31c-5, F-S31c-6; one audience rule (`ARC-43`) |
| SD-S31c-3 | The survey reads economy through `EconomySystem.discloses(world, surveyor, subject)` | I-c-4: it copies exactly what the surveyor is shown and cannot see a person's count; no second disclosure rule |
| SD-S31c-4 | `survey`'s requirement is `at_place(<the surveyor's place>)` | F-S31c-1; S31-a SD-S31a-3 |
| SD-S31c-5 | The pack depends on `economy`, `item` and `presence`, and decodes categories with item's `Category` | it reads all three (A-5, A-13); a dependency it reads is declared, even when another dependency implies it |
| SD-S31c-6 | The section's spans are this pack's own `Span` with S31-b's grammar (`m`, `h`, `d`) | A-18; one grammar for authors; a shared type waits for a third user (`CLAUDE.md` §4 rule 11) |
| SD-S31c-7 | `survey-book-opened` is `SystemInternal` | the ruled precedent (Q-S31a-2, Q-S31b-2) |
| SD-S31c-8 | One ring per kind, both sources together | the step's shape; the cooldown (Q-S31c-2) keeps asks from evicting sales |
| SD-S31c-9 | The window prunes state at each `price-noted` and filters at disclosure | bounded state; exact, wake-free expiry (S31-b's pattern) |
| SD-S31c-10 | The surveyor's own asks and its own sales are noted like anyone's | "every ask disclosed to it there" and "the sales it perceived"; no special case |
| SD-S31c-11 | `survey-category-unknown` is raised at reduction | `ItemKind` is state (A-14, A-15); S31-a SD-S31a-1's pattern |
| SD-S31c-12 | `ARC-116` goes to `docs/decisions/ARC-116.md` (or `DECISIONS.md` if C0 finds the single file) | A-24; S31-b SD-S31b-12 |
| SD-S31c-13 | The real-binary checkpoint uses Carol in a Market Town copy, not S31-f's reporter | S31-f names the reporter; Carol is observant, jobless and walks store → café (A-22) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S31-c — the price-survey System Pack (the price book, witnessing,
                         surveying)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s31c-price-survey.md
RELATED / BINDING DOCS:  step-31-wealth-prices.md (§§0, 1, 3, 9, 13, 14, 15, 16.2, 16.5, 18, 19);
                         pr-s31a-currency-asks.md (§§5.6, 5.7, 5.9, 14.1); pr-s31b-notices.md (§§4, 5.2, 14);
                         docs/CORE_CONCEPTS.md §§12, 15.2; docs/MODULE_SPEC.md §4.1; docs/DECISIONS.md and
                         docs/decisions/ ARC-25, ARC-26, ARC-28, ARC-33, ARC-34, ARC-36, ARC-43, ARC-61, ARC-90;
                         .structured-coding/standards.md; CLAUDE.md; docs/ENGINEERING_STANDARDS.md;
                         docs/ENGINEERING_RULES.md; /Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s31c, its own, held by one session
BRANCH:                  mvp0/pr-s31c, created from origin/main
START CONDITION:         Milestone F's checklist handed to the operator (step-31 §16.5) AND this design
                         frozen by the operator AND S31-a merged AND the dispatch. NOT met at 2026-10-18
IMPLEMENTATION BASE:     main (origin/main at C0); C0 records it
APPROVED SCOPE:          §2.1 items 1–6, as §14's rulings shape them                     (operator, pending)
FROZEN INVARIANTS:       step-31 I-S31-1, I-S31-2, I-S31-9, I-S31-10 (their S31-c parts); §6 I-c-1 … I-c-10;
                         §8 as written; Q-S31a-3 (complete offers safe at random)
SEQUENCE:                C0 … C6 (§9)
ALLOWED COMMANDS:        cargo build / test -p <crate> --test <file> / --lib / clippy -p / fmt;
                         target/*/mineworld *; the cast-slots with-slot.sh wrapper; git; gh (pr create/view/
                         checks, run view/download; never merge); python3 scripts/check_doc_headings.py,
                         check_decision_ids.py (--write-index); mkdir -p; cp; cmp; sha256sum/shasum; sed -n;
                         grep
NEVER:                   python3 -c; sed -i; perl -pi; heredoc writes; curl; git add -A; cargo test --workspace
                         or all of tools/cli (R-LOAD-1); reading any key or .env; calling any model; editing a
                         path outside I-c-1's allowance; changing an economy function body; a nightly dispatch
                         (nothing is re-baselined); merging
MATERIAL STOPS:          Milestone F not handed over; S31-a not merged; `Sold` or the asks view unreadable as
                         §5.5/§5.9 need; economy needing more than getters and re-exports; a needed change under
                         I-c-1's forbidden paths (in particular presence's audience rule); a needed new external
                         dependency; any reference world's baseline moving; AC-c-6's non-vacuity floor failing
                         (recorded as FAIL with the numbers, never tuned); any change to a frozen step invariant
                         or acceptance
PLATFORMS:               Linux, macOS, Windows (CI `fast` per PR; the full matrix at the milestone and on main)
VALIDATION BUDGET:       pack test files unrestricted locally, one at a time; tools/cli price_survey.rs once
                         through with-slot.sh (four 30-day Market Town runs; ≤ 300 s on Linux CI expected,
                         timing INCONCLUSIVE locally); binary-level mutation X-7 once or at the milestone.
                         Gate 1: NOT REQUIRED. Gate 2: NOT REQUIRED. No paid call
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s31c.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                                       (source: the operator's freeze of this design — pending)
                         semantic commits: authorized                    (working rules §14)
                         branch push: authorized                         (working rules §21)
                         PR creation / update: authorized                (working rules §21)
                         CI repair to review readiness: authorized       (working rules §21)
                         endpoint: MERGE-READY — the PR pushed with `fast` green on the exact head, its
                         number and head SHA reported
                         merge: NOT by the implementation session and NOT by the lane lead. The coordinator
                         merges (lane-lead charter, SPEED FIRST 2026-10-18: "Merges are done by the
                         coordinator ... the coordinator's auto-merger lands it once `fast` is green")
POST-MERGE SYNC OWNER:   lane lead / primary: step-31, overall.md, the progress page; implementation session:
                         this ledger, the merge identity, evidence, deviations
STOP CONDITION:          S31-c MERGE-READY — the implementation session does not merge
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S31c-1 | S31-a's merged shapes differ from its design (`Sold`'s fields, the asks view's name) | C0 step 3 audits them and records the real names; a shape §5.5 cannot read is a material stop |
| R-S31c-2 | A large book bloats the surveyor's observation (and an LM prompt) | bounded by the author's `depth` (≤ 64) and categories (≤ 16); defaults ≈ 30 KB for ten kinds; S31-d may summarize for its prompt |
| R-S31c-3 | Few sales happen while Carol is present in 30 days, so AC-c-6 is weak | the non-vacuity floors are fixed now; a miss is a recorded FAIL and a material stop, not a tuned run (`ARC-23`) |
| R-S31c-4 | A future pack states `sold` with a narrowed audience, or moves people in the same generation before it | the witness rule calls presence's `admits` and builds `Whereabouts` inside `react` (F-S31c-6); AC-c-6 compares with `perceived_by` over the real log |
| R-S31c-5 | Calling `EconomySystem.discloses` couples this pack to economy's disclosure shapes | that coupling is the point (the surveyor sees what it is shown); the shapes are economy's published types, decoded with economy's own `Deserialize` |
| R-S31c-6 | S31-b and S31-c both append to `systems/installed` and `MODULE_SPEC.md` §4.1 | one line each; the second to land rebases (§2.3) |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S31c-1 [operator — step §9.3 "visible to the surveyor only", F-S31c-4]** | Who perceives a `price-noted` fact? The step says the surveyor. But a witnessed sale is already in the surveyor's inbox and memory as `sold`, so the note would arrive twice; and in Market Town the reporter witnesses dozens of sales a day. | **Nobody (`SystemInternal`).** The surveyor reads its notes through its own book record, which every backend already gets. Its inbox and memory keep only what happened in the world (the `sold`), as ruled for Q-S31a-2 and Q-S31b-2. No memory renderer is needed. The alternative (`Entities({surveyor})`) doubles every witnessed sale in the reporter's inbox (bound 512) and memory. |
| **Q-S31c-2 [operator — step §9.3 "survey … offered complete", Q-S31a-3, F-S31c-3]** | `survey` is offered complete, and generic NPC controllers take complete offers at random, so a surveyor would survey the same shop again and again and push its sale notes out of the book. Add a per-place cooldown? | **Yes: `resurvey_after` (default 1 hour, 1 min … 1 day) per place, refusal code `surveyed-recently`.** It keeps `survey` safe at random, and a real price collector records an outlet once per visit. The alternative, offering `survey` incomplete, means no rule or motivated NPC ever surveys unless S31-d's model asks by name. |
| **Q-S31c-3 [primary — AC-S31-10's wording, F-S31c-2]** | AC-S31-10 says the noted sales equal "the `sold` facts in the reporter's perceived stream". The book keeps only the kinds its section tracks, so a perceived sale of a `goods` kind is not noted. Amend the criterion to "… the `sold` facts **of tracked kinds** in the reporter's perceived stream"? | **Yes.** It is what the step's own `kinds` filter means. In Market Town today every sale is food or drink, so nothing measured changes. AC-c-6 is written that way. |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C6 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Mutations

(none yet; X-7, M-c-1 … M-c-6 as §8.1 lists them)

### Deviations and findings

F-S31c-1 … F-S31c-8 (§3.2) and SD-S31c-1 … 13 (§11), recorded at design time.

---

## 16. Proposed edits elsewhere (made by the PR or owner named)

| File | Edit | By |
| --- | --- | --- |
| `step-31-wealth-prices.md` §9.3 | dated notes: the requirement is `at_place` (F-S31c-1); the `resurvey_after` key and `surveyed-recently` (Q-S31c-2, if ruled); `price-noted`'s audience (Q-S31c-1, if ruled); the component type id `survey-book` (SD-S31c-1) | the primary, at freeze |
| `step-31-wealth-prices.md` §15.1 AC-S31-10 | "… of tracked kinds …" (Q-S31c-3, if ruled) | the primary |
| `docs/MODULE_SPEC.md` §4.1 | the `price-survey` row | this PR, C1 |
| `docs/decisions/ARC-116.md` | the record, book half; backend half by S31-d | this PR, C1; S31-d |
| `pr-s31a-currency-asks.md` / S31-a's implementation | R-S31a-c-1's getters, if the lane lead prefers them there | lane lead |
| `pr-s31d-*.md` (designed in parallel) | consume §5.10 exactly (R-S31d-c-1) | S31-d's designer, reconciled by the lane lead |
