# PR S29-b — The station newsstand in Market Town: `station-news`, its shop, its keeper's job, the paper's price

## DESIGN FROZEN — 2026-10-19 (operator approved in the primary session 2026-10-19: every open question answered as recommended)

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S31/S29 lane design sub-agent (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s29b, branch docs/pr-design-s29b,
                        planning base origin/main @ 44f050e3 (#171, 12d "the towns get walls", merged)
Approved by / evidence: none yet. The step's rulings (Q-S29-1 … Q-S29-10, step-29 §13.1), S29-a's rulings
                        (Q-S29a-1 … Q-S29a-7) and step-31's rulings (Q-S31-1 … Q-S31-8, notably Q-S31-4 and
                        Q-S31-8) bind and are not re-asked
Implementation base:    origin/main at C0; C0 records the commit and the merge state of S29-a and S31-a
Start condition:        (1) Milestone F's checklist handed to the operator (step-29 §11.2, operator
                        2026-10-15); (2) S29-a merged (step-29 §11.1, S29-b row: "Depends on S29-a");
                        (3) this design frozen; (4) the dispatch. State at 2026-10-18: (1) NOT met;
                        (2) S29-a is DRAFT (pr-s29a-town-contract.md, revision 3; its freeze follows 12d's
                        merge, now done at 44f050e3, and Milestone F's checklist), no PR open; (3) DRAFT.
                        Reconciled 2026-10-19 with S29-a revision 4 (branch docs/pr-design-s29a-rev4): it
                        changes only the bay's and platform's frames and I-a-7's wording, none of which this
                        design reads (the newsstand is an organization whose shop is `at: station`), so
                        nothing here changes
                        S31-a (FROZEN 2026-10-18; implementation starting on branch mvp0/pr-s31a) is NOT a
                        start condition: §5.3 works in either merge order
Execution contract:     §11 (filled; ends at MERGE-READY, the coordinator merges)
Implementer:            Haiku-class (step-29 §11.1, the S29-b row: "Haiku"); review: the lane lead
Lifecycle:              DESIGN FROZEN 2026-10-19 —
```

**Effort:** `mvp0`, release-blocking (step-29 §11.2: "All seven are in before S23's first public release
candidate") · **Step:** [`step-29-station-landmark.md`](step-29-station-landmark.md) (APPROVED 2026-10-15) §§2.1
(A-7, A-8), 5.1 (the newsstand row), 7.2, 9.3 (`station_visit`'s "`newspaper` is bought there", `e9_economy_300d`),
10 (I-S29-1, I-S29-3), 11.1 (the S29-b row), 13 (Q-S29-6, Q-S29-7), 14.8 (F-S29a-7) · **Parent:**
[`overall.md`](overall.md) · **Depends on:** [`pr-s29a-town-contract.md`](pr-s29a-town-contract.md) (DRAFT: the
`station` place, its kiosk solid, Ivan's station routine, `station_visit.rs`) · **Coordinates with:**
[`pr-s31a-currency-asks.md`](pr-s31a-currency-asks.md) (FROZEN: `ARC-38` item 5 lifted, `keepers:`),
[`step-31-wealth-prices.md`](step-31-wealth-prices.md) §§2.2 (F-S31-6), 8.7, 9.1, 16.4, 17 (R-S31-6), 18
(Q-S31-4, Q-S31-8), S31-f (Market Town content, not yet designed) · **Binding parents:** `CLAUDE.md` §§2, 3, 4
(items 1, 2, 5, 7, 8, 9, 10, 13, 15); `docs/CORE_CONCEPTS.md` §§7, 8, 13.1; `docs/MODULE_SPEC.md` §4.1;
`docs/decisions/ARC-35.md`, `ARC-36.md`, `ARC-37.md`, `ARC-38.md`, `ARC-105.md`.

**Decision ids.** None new. `ARC-190` (the sub-range offered for this PR) is **not used**: the newsstand is
World Pack content on existing mechanisms, and the one decision it touches is `ARC-38` item 5, which S31-a lifts.
If S29-b merges before S31-a, C1 writes one dated note `## ARC-38 note — …` inside `docs/decisions/ARC-38.md`
(§5.3, branch B); no new record. `python3 scripts/check_decision_ids.py` on this branch: PASS (§14).

No hosted model, image model or paid API is called by this design or by anything it specifies.

---

## 0. Binding inputs, exactly restated

- **Scope (step-29 §11.1, S29-b row).** "`station-news` organization, shop and stock; one person's `job`; price
  with source; E-9 300-day run." Depends on "S29-a". Integration checkpoint: "`station_visit` buys a newspaper;
  E-9 holds". Adversarial: "removing `economy` from the world refuses `station-news`'s section by name (the
  existing rule)". Implementer: "**Haiku**".
- **Step-29 §7.2, verbatim in substance.** A new organization `station-news`, "declared like `corner-store`";
  "its shop is at `station`, selling `newspaper`; the price is a realistic default with a cited source (a US
  daily single copy), scaled to Market Town's money unit"; "its holdings and wallet are sized so that the
  300-day E-9 run never drains them"; "**One existing person** gets a morning `job` there (`employer:
  station-news`, `workplace: station`, `produces: { newspaper: n }`). The person is one without a job today
  (ruled, Q-S29-6); S29-b names them." Single ownership: "Only `economy` moves money … Only `inventory` moves
  papers. Only `employment` states the wage due. Nothing new owns anything." "Social Café and Lakeside have no
  `economy`. Their kiosk is drawn **shut**."
- **Step-29 §5.1, newsstand row.** The kiosk is shown open when "the hall discloses a `shop` (Market Town,
  §7.2)", and shut otherwise. The client rule is S29-e's (`station_state.gd`); this PR only makes the hall
  disclose a shop.
- **Rulings that bind and are not re-asked.**
  - Q-S29-6: "an existing person with no job runs the newsstand."
  - Q-S29-7 as read by F-S29a-7 (recorded in step-29 §14.8): the one re-baseline is S29-a's **id shift**;
    "S29-b records its newsstand sales as a behaviour change".
  - Q-S31-4: the market price reporter is "an existing seated person without a job, **not the S29-b seller**".
  - Q-S31-8: "keepers, as `keepers:` in the shop section."
- **Step-31 §16.4, the S29 row (verbatim).** "S29-b's newsstand becomes a normal shop with a keeper, so its
  seller may change the paper's price. F-S31-6 is resolved by lifting `ARC-38` item 5 in S31-a. If S29-b merges
  first, S31-a's C3 converts its `prices` like the others, with no content change. The newsstand seller is a
  natural `town-news` publisher (§8.7)."
- **R-S31-6 (step-31 §17).** "S29-b lands before S31-a with a goods shop → S29-b's design notes F-S31-6;
  either a ruling for S29-b, or S31-a first."
- **R-S29b-a-1 (S29-a §10).** The seller is chosen among the people still jobless after S29-a's C4; `station-news`
  sorts after `corner-store` and organizations are allocated last, so it moves no place, person or item id; its
  shop, job and sales are a behaviour change with its own `reason S29-b:` line.
- **Charter, SPEED FIRST and no hard-coding (operator 2026-10-18; lane-lead brief, binding).**
  - The contract ends at **MERGE-READY**: pushed, `fast` green on the exact head, reported with PR number and
    head SHA. **The coordinator merges.**
  - Per-PR validation is the build plus the tests the commits write. The E-9 300-day run, the step's acceptance
    criteria and planted mutations are kept and **run at the milestone**, where failures are fixed in one batch.
  - Mechanisms are generic and data-driven. The newsstand is content on the generic shop, keeper and job
    mechanisms. **Every acceptance criterion names at least one second world or a generated variant.**
- **Main moved on 2026-10-19.** Decision records are one file each under `docs/decisions/`. The MCP surface is
  one `act` tool with typed kinds (`ARC-153`); nothing here adds an agent tool. 12d gave the towns `bodies` and
  item names (re-audited in §3).

---

## 1. Goal

Give Market Town's newspaper a seller. Today `newspaper` is an item kind that two people hold and nobody sells
(step-29 A-8). After this PR, an organization `station-news` runs a shop in the station hall that sells the paper
at a price taken from a real US daily, one existing person, **Ivan**, works its morning shift and is paid for it,
and the shift prints the day's papers into the newsstand's stock. In Social Café and Lakeside nothing changes:
they have no `economy`, so their halls disclose no shop and their kiosks stay shut.

The chain, in plain steps:

1. The organization file declares a shop at `station` with one price (economy's section) and a stock of papers
   (inventory's section).
2. At genesis, `economy` states `shop-opened` at `station`; from then on every observer standing in the hall
   perceives the shop's listing (operator, price, how many in stock).
3. A person in the hall is offered `buy newspaper`; if they buy, `economy` moves the money to `station-news` and
   `inventory` moves one paper to them, in one dispatch (`ARC-38`).
4. Ivan's `job:` makes `employment` start a shift at 07:00 at `station`; time he is present is worked; at 12:00
   the shift ends, `employment` states `wage-due`, `economy` pays it from `station-news`'s wallet, and
   `inventory` produces the prorated papers into `station-news`'s holdings.

No code changes outside tests. No System, kernel, contract, client or cognition file is touched (I-S29-1).

---

## 2. Scope

### 2.1 In scope

1. **Content (C1).** `worlds/market-town/organizations/station-news.yaml` (new); `worlds/market-town/world.yaml`
   (the `organizations:` list and the comments that count organizations and jobs);
   `worlds/market-town/people/ivan.yaml` (one appended `job:` section and its comment; nothing else in the file);
   `worlds/market-town/README.md` (the counts of organizations and job holders: human text).
2. **The tests the content moves (C1, same commit, so `main` stays green).** `tools/cli/tests/market/mod.rs`:
   the job-holder assertion reads the holders from the pack instead of the literal "exactly two (alice and
   felix)" (step-31 A-24); `tools/cli/tests/market_composition.rs`: the carrier counts of `economy` (14 → 15) and
   `employment` (2 → 3).
3. **Branch B only (S31-a not merged at C0, §5.3).** `docs/decisions/ARC-38.md`: one dated note (C1).
4. **New tests (C2).** `tools/cli/tests/newsstand.rs` (new): a generated variant world written by the test, and
   the Market Town and Social Café checks of §7. `tools/cli/tests/station_visit.rs` (S29-a's file): one test,
   "the paper is bought at the station".

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| The station places, the kiosk solid, Ivan's `07:00 station meet` routine, `station_visit.rs` | S29-a |
| The kiosk drawn open or shut (`station_state.gd`), the hall's 3D and 2D fit-out, the `prop:kiosk` sprite | S29-e, S29-f |
| `PlaceLine.roles` and the agent scenario that sees `buy newspaper` in the hall (step-29 §9.4 item 3) | S29-g |
| `keepers:` on the café and the store; asks, reference values, the reporter, notices topics | S31-f |
| A `town-news` topic with Ivan as its publisher (§5.8, Q-S29b-3) | S31-f (recommended) |
| The long baselines (`scripts/baselines.txt`) re-recorded for the behaviour change | the milestone batch (§5.9) |
| A way for a bought paper to leave the world (read, recycle) | not here (§3.2 F-S29b-4, Q-S29b-4) |
| Any change under `kernel/`, `contracts/`, `systems/`, `server/`, `clients/`, `cognition/`, `sdk/` | never (I-S29-1) |

### 2.3 Dependencies

- **S29-a merged (required before C0).** It creates `worlds/market-town/places/station.yaml` (the hall, tags
  `station`, `public`, `landmark`, with "the kiosk (design), south-west of the benches" as a `body` solid,
  `pr-s29a-town-contract.md` §5.5), gives Ivan the routine `07:00 station meet, 12:00 cafe chat, 16:00 park walk,
  20:00 apartments home` in both towns (§5.6 there), and creates `tools/cli/tests/station_visit.rs` (C6 there).
  Missing → material stop.
- **S31-a, either order (§5.3).** Branch A: S31-a merged → the shop names `keepers: [ivan]`. Branch B: S31-a not
  merged → no `keepers:` key (the section refuses unknown keys), the `ARC-38` note, and R-S31f-b-1 hands the
  keeper to S31-f.
- **S31-b, S31-c, S31-d, S31-f: not required.** None changes what this PR writes.

---

## 3. Audit (`origin/main @ 44f050e3`, 2026-10-18; after 12d)

### 3.1 Source anchors

| Id | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | Market Town has two organizations (`cafe-company`, `corner-store`), 12 people (11 seats; Otto is not a seat), 24 item keys (20 kinds plus Social Café's four loose objects), six places | `worlds/market-town/world.yaml` `organizations:`, `population:`, `items:`, `places:`, `seats:` | `station-news` is the third organization. It sorts after `corner-store` and organizations are allocated last (`worldpack/src/load.rs`, S29-a A-17), so **no id moves** |
| A-2 | An organization file: `tags`, `note`, `holdings:` (inventory), `economy: { wallet, shop: { at, prices } }` | `worlds/market-town/organizations/corner-store.yaml`, `cafe-company.yaml` | `station-news.yaml` is written in the same shape |
| A-3 | `newspaper`: `tags: [goods, reading]`, `item: { category: goods, name: Newspaper }`. Bob and Otto each hold one at genesis | `worlds/market-town/items/newspaper.yaml`; `people/bob.yaml:30`; `people/otto.yaml:33` | The kind exists and has a name (12d). No item file changes |
| A-4 | Economy's section: `deny_unknown_fields` on the section and on `AuthoredShop`; a shop on a Person is refused; the shop's place must be a Place and every priced key an Item | `systems/economy/src/section.rs:34, 48, 77-121` | A `keepers:` key is refused on `main` today; it exists only after S31-a's C3 (§5.3) |
| A-5 | **No code enforces `ARC-38` item 5** ("shops sell consumables only"). `section.rs` checks no item category; `buy` reads only the price and the stock | `systems/economy/src/{section.rs, system.rs:113-137}`; `git grep -n "category" systems/economy/src` (no match) | A shop selling `goods` loads and sells today. The conflict F-S31-6 names is with a **decision record**, not with code (F-S29b-2) |
| A-6 | `ARC-38` item 5: "Shops sell consumables only, and their stock is produced by the shift (QS-48). Goods keep circulating by `give` (`ARC-37`); a bought good would occupy one of a person's six places for good." | `docs/decisions/ARC-38.md` (Choice, item 5) | The rule's reason is the six-item capacity, which a paper meets (F-S29b-4) |
| A-7 | Employment's section: `job: { employer, workplace, from, until, wage, produces }`; `employer` an Organization, `workplace` a Place, `produces` keys Items; `from < until` within one day | `systems/employment/src/section.rs:22-150`; `docs/decisions/ARC-38.md` (accepted limitations) | Ivan's job is one section in his file |
| A-8 | A shift's pay and production are prorated to the time worked; a kind is produced only when its prorated share is at least one | `systems/employment/src/system.rs:188-248` ("one `items-produced` per kind whose prorated share is at least one") | A late arrival is paid and produces less. No content tunes that |
| A-9 | `buy` needs no staff: a buyer in a place with a shop buys from the operator's stock whether or not an employee is present (today's café and store behave the same) | `systems/economy/src/system.rs:88-137` | The newsstand sells whenever someone is in the hall. That is the existing model, stated, not changed (F-S29b-5) |
| A-10 | **AC-1 check 3 (`ARC-35`).** For a person file present in both Social Café and Market Town, every Social Café key must be present in Market Town **with an equal value**; Market Town may only add sections a market pack owns (`job:`, `economy:`, `holdings:`) | `tests/acceptance/tests/ac1_composability.rs:1208-1235` (`compare_content`), `1395-1405` | **The seller's `routine`, `tags`, `note` and `persona` cannot change in Market Town alone.** The job must fit a routine both towns share (F-S29b-1) |
| A-11 | Ivan: seated, no job, `tags: [passerby]`, persona "chatty, curious … stops to talk to anyone who will", holds `{ apple: 1, coffee: 1, mug: 1 }`, wallet 200 000. After S29-a his routine starts `07:00 station meet` and ends that part at 12:00, in both towns | `worlds/market-town/people/ivan.yaml`; `pr-s29a-town-contract.md` §5.6 | Ivan is at the station 07:00–12:00 every day in both towns, so a 07:00–12:00 job there needs **no routine change** |
| A-12 | Wages and wallets in Market Town: Alice 120/h, 05:30–14:00, café wallet 20 000; Felix 200/h, 08:00–13:00, store wallet 300 000 ("sized so that it can pay Felix every day for the whole measured run") | `people/alice.yaml` `job:`; `people/felix.yaml` `job:`; `organizations/*.yaml` | The town's wage scale is content sized to E-9 (`ARC-38` item 6), not a real wage. Q-S29b-2 |
| A-13 | Prices are integer minor units with exponent 2 (the 2D client divides by 100; S31-a makes the exponent declared): apple 100, coffee 300, bread 300 | `clients/2d/scripts/hud/words.gd:73` (step-31 F-S31-3); `organizations/*.yaml` | 100 minor units = one US dollar in this town. A $2.00 paper is 200 |
| A-14 | E-9 / CP-4 criteria (`market_lives`): every 30-day bucket has a purchase, a wage paid to **every job holder**, an `items-produced`, an `items-consumed` and a give by every seat; zero `wage-unpaid`; no wallet below "the cheapest price any shop opened with"; nobody above six items. It asserts "**exactly two** job holders (alice and felix), read from `hired`" | `tools/cli/tests/market/mod.rs:511-680`, the literal at `541-547`; `tools/cli/tests/market_town.rs:216-256` (the 300-day run) | A third job holder fails the literal at once. C1 replaces it with the holders the pack declares (no new literal) |
| A-15 | `market_composition` strips each market pack and its section; it asserts the number of files carrying each section: `economy` 14, `employment` 2 | `tools/cli/tests/market_composition.rs:39-46, 103-121` | `station-news.yaml` and `ivan.yaml` add one carrier each: 15 and 3. The "without economy" case expects the world to **run** with the sections stripped; the step's adversarial case (economy disabled, sections kept) is a different, new check (AC-b-6) |
| A-16 | MODULE_SPEC §4.1 rule 6: a world that does not enable a section's owner refuses the section, by name | `docs/MODULE_SPEC.md` §4.1; `market_composition.rs:5-7` | "Removing `economy` refuses `station-news`'s section by name" is the existing rule; AC-b-6 plants it |
| A-17 | Long baselines: `[world market-town]` in `scripts/baselines.txt`, each move with a `reason <PR>:` line | `scripts/baselines.txt:1-5, 33-37` | Sales and a third shift move Market Town's history (§5.9) |
| A-18 | The step-31 example names `town-news` with `publishers: [hana]`, and S31-c's fixture uses Carol as the surveyor | `step-31-wealth-prices.md:678-686`; `pr-s31c-price-survey.md` (branch `docs/pr-design-s31c`, 9c4b5296) A-22, R-S31f-c-1 | Examples only; nothing is ruled. Ivan is neither (Q-S31-4 holds) |
| A-19 | The rule and motivated controllers attempt only **complete** offers; `publish-notice` is offered incomplete (S31-b), and S31-d is asked to design "attempt an incomplete offer by name" (brief, material finding of the S31-h design) | `cognition/rule-controller/src/offered.rs:26-52`; `pr-s31b-notices.md` §5 | A `town-news` publisher seat publishes nothing until S31-d's mechanism and a policy exist (§5.8) |

### 3.2 Findings

- **F-S29b-1. The seller's routine cannot change in Market Town alone.** AC-1 check 3 compares each shared person
  file's keys between the towns (A-10). A morning job at the station for someone whose routine does not already
  put them there in both towns would need a Market-Town-only routine, which AC-1 refuses, or a Social Café routine
  change to "go and work at a kiosk that is shut there", which would pretend (step-29 §5.2). **Ivan is the one
  jobless seated person whose routine (after S29-a) already holds the hall 07:00–12:00 in both towns** (Erin's
  holds it 07:30–13:00 and is the alternative). So the job fits the day he already keeps, as Alice's and Felix's
  do (`alice.yaml`: "her routine already has her at the café from 05:30 to 14:00, so the job fits the day she
  keeps"). → Q-S29b-1.
- **F-S29b-2. F-S31-6 is a conflict with a decision, not with code.** No code enforces `ARC-38` item 5 (A-5). If
  S29-b merges before S31-a, the newsstand loads and sells, and the only defect is an unrecorded departure from
  `ARC-38`. A dated note records it (§5.3 branch B). No ruling is needed beyond the step's own §7.2 and §16.4,
  which already approve a goods shop; Q-S29b-5 states the reading for the record.
- **F-S29b-3. `keepers:` does not exist on `main`.** The section refuses unknown keys (A-4). So "make the keeper
  explicit" has two forms, one per merge order (§5.3).
- **F-S29b-4. A bought paper never leaves the world.** `consumption` uses only `food` and `drink`; goods are never
  consumed (`ARC-38` item 4). Each sale moves a paper from an unbounded organization into a person bounded at six.
  Papers can be given on, and a holder at six is refused further buys (`cannot-carry`), so E-9's "nobody above six"
  holds by inventory's rule. But over 300 days the station's regulars may fill their hands with papers and stop
  buying food. This is exactly the reason `ARC-38` item 5 gave. A sink ("read and discard") is a System change
  (`consumption` or a new pack), which I-S29-1 forbids here. → Q-S29b-4 (recommended: accept, measure at the
  milestone, record the measured holdings).
- **F-S29b-5. The kiosk sells without its seller (existing model).** `buy` needs no employee present (A-9). The
  newsstand is therefore "open" in the world whenever the hall has a shop, which is what step-29 §5.1's client rule
  shows. Staffed opening hours would be a change to `economy` for every shop; not here.
- **F-S29b-6. The E-9 test hard-codes the job holders** (A-14), which the operator's no-hard-coding rule forbids
  carrying forward. C1 replaces the literal by the pack's own declaration (the people whose file carries `job:`),
  so a world with any number of jobs is measured the same way.
- **F-S29b-7. The town's wages are not real wages** (A-12). Q-S29b-2.

### 3.3 What does not exist

No organization other than `cafe-company` and `corner-store`. No shop sells `newspaper`. No `station` place on
`main` (S29-a). No test world other than Market Town enables `economy` (`git grep -ln "^  - economy" -- '*.yaml'`).

---

## 4. Terms

No new term. **Newsstand** and **kiosk** are plain English for one thing: the shop `station-news` runs in the
station hall. In this file **seller** is Ivan, the Person whose `job:` names `station-news`; after S31-a he is
also the shop's **keeper** (step-31 §3: "A Person an Organization's shop names as allowed to change its asks
while standing in the shop"). A keeper is not an employee by definition; Ivan is both because the content says
so twice, once in each owner's section (economy's `keepers`, employment's `job`), and neither pack reads the
other's (SD-S31-5).

---

## 5. Design

### 5.1 The organization (C1)

`worlds/market-town/organizations/station-news.yaml`, written in `corner-store.yaml`'s shape:

```yaml
# Station News — the organization that runs the newsstand in the station hall (step-29 §7.2). Its key is
# not the hall's: an organization and the place it runs are two entities (docs/decisions/ARC-36.md).
tags: [business, newsstand]
note: Runs the newsstand in the station hall. Ivan keeps it in the mornings.

# What it has on the kiosk at genesis (inventory's `holdings`, ARC-37). Ivan's morning shift brings in the
# day's papers (his `job:` in people/ivan.yaml).
holdings: { newspaper: 12 }

# The opening balance and the shop it runs (economy's section, ARC-38). The price is a US daily's weekday
# single copy: $2.00 (Valley News, Lebanon NH, from 1 April 2024, publisher's note
# https://www.vnews.com/valley-news-publishers-note-54430519), at this town's 100 minor units to the dollar.
# The wallet pays Ivan's whole shift every day for the measured 300 days with no sale at all (step-10 E-9):
# 120 an hour x 5 hours x 300 days = 180 000.
economy:
  wallet: 200000
  shop:
    at: station
    prices: { newspaper: 200 }
    # Branch A only (S31-a merged, §5.3):
    keepers: [ivan]
```

- **Branch B** (S31-a not merged at C0) omits the `keepers:` line and its comment, and the comment above the
  shop says instead: "Ivan will keep this shop's prices once shops name keepers (S31-a; R-S31f-b-1)."
- **Tags.** `business` is the tag both existing organizations carry; `newsstand` is the role, as `cafe` and
  `store` are theirs. Tags are content (`ARC-36`); no System reads them.
- **No place file changes.** The shop is authored on its operator (`ARC-38`, QS-43), so `station.yaml` (S29-a's)
  is untouched; AC-1 check 3 compares place files too (A-10).

### 5.2 The seller's job (C1)

Inserted in `worlds/market-town/people/ivan.yaml` right after `economy:` and before `persona:`, the place
`felix.yaml` and `alice.yaml` put their `job:`:

```yaml
# Market Town: his job (employment's section, ARC-38; see alice.yaml). His routine already has him in the
# station hall from 07:00 to 12:00 (the town's meeting point, step-29 §7.1), and in Social Café too, so the
# job fits the day he keeps in both towns (docs/decisions/ARC-35.md, AC-1 check 3). He keeps the newsstand
# for Station News; a full shift brings in the day's papers.
job:
  employer: station-news
  workplace: station
  from: "07:00"
  until: "12:00"
  wage: 120
  produces: { newspaper: 8 }
```

- **The seller is Ivan** (Q-S29b-1, F-S29b-1). Jobless after S29-a (R-S29b-a-1's list), seated, not the
  reporter (Q-S31-4; R-S31f-b-2), and his persona ("stops to talk to anyone who will") reads as a newsagent's
  without any edit, which AC-1 would refuse anyway.
- **The shift, 07:00–12:00.** A morning shift (step-29 §7.2: "a morning `job`"), equal to his station segment, so
  his routine is unchanged. He walks from the apartments after 07:00 and arrives later (S29-a F-S29a-5: the
  station is the farthest place); pay and papers are prorated to the time present (A-8). Nothing is tuned to
  hide that.
- **The wage, 120 an hour**, the café's rate (A-12): the lowest wage the town already pays, for the least skilled
  of its three jobs. It is the town's scale, not a real one (Q-S29b-2).
- **Production, 8 papers per full shift.** Sized so the stock does not run out at the measured demand. The step
  fixes no number; the milestone's 300-day run records the newsstand's stock at days 30, 150 and 300, and a stock
  of zero on more than half of the days is recorded as a finding (AC-b-5), never tuned silently (`ARC-23`).
- **No `holdings:` change, no `economy:` change** for Ivan. His three held items stay.

### 5.3 The keeper, in either merge order (F-S29b-3, R-S31-6)

| C0 finds | Branch | What C1 writes | Who makes Ivan the keeper |
| --- | --- | --- | --- |
| S31-a merged | **A** | `keepers: [ivan]` in the shop (S31-a's section key, §5.4 there) | this PR |
| S31-a not merged | **B** | no `keepers:` key; one dated note in `docs/decisions/ARC-38.md` (below); the organization file's comment names Ivan as the keeper to be | **S31-f**, which writes Market Town's keepers (`step-31` §16.2, S31-f row: "keepers"); R-S31f-b-1 |

- **Why not a stub key in branch B.** The section refuses unknown keys (A-4), and S31-a is frozen with "S29-b …
  no content change" (`pr-s31a-currency-asks.md` §2.3). Writing `keepers:` before S31-a would fail genesis; asking
  S31-a to add it would change a frozen design. S31-f already edits every Market Town shop to add keepers, so one
  more line there is the smallest change.
- **The `ARC-38` note (branch B only), exact text:**

  ```markdown
  ## ARC-38 note — 2026-10-?? (S29-b): a newsstand sells goods before item 5 is lifted

  Market Town's `station-news` shop sells `newspaper` (category `goods`), approved by step-29 §7.2 and step-31
  §16.4 (F-S31-6, R-S31-6). Item 5 ("shops sell consumables only") is a content rule that no code enforces; this
  shop departs from it until S31-a lifts it (`ARC-112`). Item 5's reason, that a bought good keeps one of a
  person's six places, still holds for the paper and is measured at the milestone (pr-s29b-newsstand.md
  F-S29b-4).
  ```

  Then `python3 scripts/check_decision_ids.py --write-index` (the note is a heading inside the record's own file,
  so the index does not change; the script confirms it).
- **Branch A writes no note.** S31-a's C1 lifts item 5 with its own note.
- **Behaviour is the same in both branches.** Before S31-f, Market Town has no NPC pricing policy, so a keeper
  never changes a price; `keepers:` only grants the right (step-31 SD-S31-5).

### 5.4 The price and its source

- **Value:** `newspaper: 200` = $2.00.
- **Source:** the Valley News (Lebanon, New Hampshire; a US daily) raised its weekday single-copy price to $2.00
  from 1 April 2024, in a publisher's note (`https://www.vnews.com/valley-news-publishers-note-54430519`). A US daily
  weekday single copy at $2.00 sits between the Providence Journal ($2.00 weekday, undated report) and Newsday
  ($2.50), the other figures found the same day.
- **What was not found:** the San Diego Union-Tribune's current single-copy price (Market Town's weather is San
  Diego's, `ARC-68`). Its audited statement for the six months ended 30 September 2020 (Alliance for Audited
  Media, published by Poynter) gives circulation but no price. If the operator prefers the local paper's price,
  it replaces this value with one line and its source (Q-S29b-2 covers the scale question; the value itself is
  content).
- **Scale:** 100 minor units to the dollar (A-13). E-9's "cheapest price any shop opened with" stays 100 (apple),
  because 200 > 100 (A-14).

### 5.5 Sizing for E-9 (the 300-day criterion; measured at the milestone)

| Quantity | Value | Why |
| --- | --- | --- |
| Wallet | 200 000 | the wage bill with zero sales is 120 × 5 × 300 = 180 000, so `wage-unpaid` cannot occur whatever the demand (E-9: zero `wage-unpaid`) |
| Opening stock | 12 | papers on the kiosk before the first shift ends |
| Production | 8 per full shift, prorated | §5.2 |
| Price | 200 | §5.4 |

Ivan's wage income (≤ 600 a day) only raises his wallet, so "no wallet below 100" is not put at risk by the job;
buyers spend 200 per paper, which E-9's wallet criterion measures over 300 days.

### 5.6 Ownership, events, refusals, space

- **Owners, unchanged.** `economy`: the wallet, the shop, the listing, `money-transferred`, `wage-unpaid`
  (and after S31-a: the asks, `listed`, `sold`). `inventory`: the newsstand's papers and every buyer's.
  `employment`: `hired`, `shift-started`, `shift-ended`, `wage-due`, and `items-produced` through inventory's
  checked constructor (`ARC-26`). `schedule`: Ivan's routine. Nothing new owns anything (step-29 §7.2).
- **Events this content causes, all existing:** genesis `funded { station-news }`, `shop-opened { station }`
  (branch A: v2 with `keepers`, plus one `listed`), inventory's genesis fact for the 12 papers, `hired { ivan }`; then
  daily `shift-started`, `shift-ended`, `wage-due`, `money-transferred` (wage), `items-produced { newspaper }`;
  per sale `money-transferred` (buyer → station-news) and `items-transferred` (and `sold` after S31-a).
- **Refusals, all existing and by name:** a buyer who cannot pay, cannot carry, or finds no stock is refused by
  `economy`'s and `inventory`'s rules (on `main`: `TargetUnavailable`; after S31-a: `insufficient-funds`,
  `cannot-carry`, `out-of-stock`); a buy outside a shop is `NoSupportedInteraction`.
- **Space.** A shop `buy` requires the buyer to stand in the shop's place, `station` (economy's own requirement;
  the client only reports intent, `CLAUDE.md` §4 rule 15). After S31-a, a keeper's `for: shop` actions require
  Ivan to stand in `station` (`at_place`, S31-a SD-S31a-3).

### 5.7 Both clients and the agent view

Nothing in `clients/` changes. Every client already reads the `shop` listing the hall now discloses:

- **2D:** the HUD's `shop` reader shows the listing in the hall (`clients/2d/scripts/hud/readers.gd`); the kiosk
  sprite's open state is S29-f's rule.
- **3D:** the kiosk's open state is S29-e's `station_state.gd`; until then the hall is not drawn (S29-a F-S29a-6).
- **Agents (MCP, S27):** an agent seat in the hall perceives the listing and is offered `buy newspaper` as a
  typed kind of the one `act` tool (`ARC-153`); no new kind. Step-29 §9.4 item 3 is S29-g's scenario.

### 5.8 The `town-news` publisher (step-31 §8.7; Q-S29b-3)

Ivan is a natural publisher of a `town-news` topic: he keeps the paper. Making him one needs `notices` enabled in
Market Town, a `configure/notices.yaml` with a `town-news` topic, `ARC-35`'s allow-list to admit that
configuration, and subscriptions. All of that is Market Town's notices composition, which step-31 gives to S31-f
("`configure/notices.yaml`; subscriptions by tag"). And a rule-controlled Ivan would still publish nothing,
because `publish-notice` is offered incomplete and NPC controllers attempt only complete offers until S31-d's
"attempt an incomplete offer by name" mechanism lands (A-19). **Recommendation: S31-f**, with R-S31f-b-3 below.
S29-b adds nothing for notices.

### 5.9 Persistence, determinism and the behaviour change

- **No id moves** (A-1): `station-news` is the last organization key.
- **Behaviour moves:** genesis gains the organization's facts and Ivan's `hired`; the run gains a third shift and
  newsstand sales. Every `[world market-town]` digest in `scripts/baselines.txt` moves; `social-cafe`, `lakeside`
  and `bodies-yard` must not (they have no economy and no file changes here).
- **Speed-first placement:** the PR's `fast` CI does not run the long profile. The long baselines are
  re-recorded in the **milestone batch** from one long record, with this reason line, which the PR's ledger
  (§15) carries verbatim for that batch:

  ```text
  reason S29-b (step-29 §7.2, F-S29a-7): Market Town's station newsstand — organization station-news (shop at
  station, newspaper 200, stock 12), Ivan's 07:00–12:00 job (produces 8) — adds genesis facts, a third shift and
  newsstand sales; no id moved; social-cafe, lakeside and bodies-yard did not move
  ```

- **Determinism.** Content only; the run is seeded as before. Save → restart → resume identity is S29-a's and
  CP-4's existing check, re-run at the milestone (AC-b-4).

---

## 6. Invariants

| Id | Invariant | Checked by |
| --- | --- | --- |
| I-b-1 (I-S29-1) | `git diff --name-only origin/main...HEAD` lists nothing under `kernel/`, `contracts/`, `systems/`, `server/`, `clients/`, `cognition/`, `sdk/` | C3 diff scan |
| I-b-2 | No world-specific code: no test or code names `station-news`, `ivan` or `newspaper` as a rule; the job-holder count and the shop's existence are read from the pack | review of C1/C2; `git grep -n "station-news" -- '*.rs'` lists only `newsstand.rs`'s Market Town data lookups, never a constant in shared helpers |
| I-b-3 | Single ownership (step-29 §7.2): money only by `economy`, papers only by `inventory`, the wage due only by `employment` | the existing owners; AC-b-2's fact check |
| I-b-4 | AC-1 holds: Ivan's shared keys are unchanged; only `job:` is added | `tests/acceptance` `check_3_the_world_delta` (CI `fast` runs it if it is in the layer; else at the milestone) |
| I-b-5 (I-S29-3) | Social Café and Lakeside halls disclose no shop | AC-b-3 |
| I-b-6 | No id moves in any world | AC-b-7 |

---

## 7. Acceptance criteria and planted mutations (fixed before measuring)

**When they run.** C2's tests run in the PR's CI if the `core` layer lists them (`python3 scripts/ci_layer.py
--list core`); the 300-day run, the planted mutations and the AC-1 check run at the **milestone** (charter).
Every criterion names a second world or a generated variant.

**The generated variant, `kiosk-yard`.** `newsstand.rs` writes it into a scratch directory, with no key shared
with Market Town: systems `presence, movement, schedule, item, inventory, economy, employment`; places
`concourse` (tags `station`, `public`, `landmark`), `booking-office` (`station`, `ticket-office`, `public`),
`platform-1` (`station`, `stop`, `platform`, `public`, `outdoor`), `lane`, joined as S29-a's town is;
`settlement: { kind: town, station: concourse, landmark: concourse }`; item `gazette` (`category: goods`);
organization `paper-kiosk` with `holdings: { gazette: 6 }`, `economy: { wallet: 50000, shop: { at: concourse,
prices: { gazette: 150 } } }` (branch A adds `keepers: [mo]`); people `mo` (routine `06:00 concourse kiosk`, job
`paper-kiosk` at `concourse` 06:00–11:00, wage 100, `produces: { gazette: 3 }`) and `lu` (routine `06:00 concourse
wait`, wallet 5 000, starts in `concourse`); both seats. Nothing in it is Market Town's.

| Id | Criterion | Worlds | Step ref | When |
| --- | --- | --- | --- | --- |
| AC-b-1 | `mineworld interactions <world> --place <hall> --json` offers `buy` of the authored kind at the authored price | Market Town (`station`, `newspaper`, 200) **and** `kiosk-yard` (`concourse`, `gazette`, 150) | step-29 §5.1, §9.4 item 3 (static half) | PR (C2) |
| AC-b-2 | **Sold at the station.** A seeded run has ≥ 1 action-caused `money-transferred` from a person to the hall's operator, paired in the same revision with an `items-transferred` of the authored kind from that operator to the same person; and ≥ 1 `wage-due` for the job holder answered by `money-transferred` | `kiosk-yard` 3 days seed 1 (PR); Market Town 30 days seed 7 via `station_visit` (milestone) | step-29 §9.3 ("`newspaper` is bought there"), S29-b checkpoint | PR + milestone |
| AC-b-3 | **No shop without economy.** The hall lists no `buy` and discloses no `shop` | Social Café **and** Lakeside (`mineworld interactions … --place station --json`) | step-29 §5.1 (kiosk shut), I-S29-3 | PR (C2) |
| AC-b-4 | **E-9 holds** over 300 days seed 7 with three job holders read from the pack (zero `wage-unpaid`, money conserved, nobody above six, no wallet below the cheapest price, every bucket's counts incl. "wage paid to ivan"); CP-4's kill-and-resume identity holds | Market Town (milestone); the same `market_lives` on `kiosk-yard` 30 days (PR, with its own holders) | step-29 §9.3 `e9_economy_300d`; AC-S31-14's S29-b half | PR (variant) + milestone |
| AC-b-5 | **The paper's life, recorded (F-S29b-4).** The 300-day run prints, at days 30, 150, 300: papers held per person, the newsstand's stock, and the number of `cannot-carry`/`TargetUnavailable` refusals of `buy`. Recorded, not asserted beyond AC-b-4; a person at six papers for 30 consecutive days is a recorded finding for Q-S29b-4 | Market Town (milestone); `kiosk-yard` 30 days (PR, printed) | Q-S29b-4 | PR + milestone |
| AC-b-6 | **Adversarial (step's row).** A copy with `  - economy` removed from `systems:` but the sections kept is refused by `mineworld validate`, naming the `economy` section and the file `organizations/station-news.yaml` (or, first in key order, the first carrier the loader reports, which the test reads from the message and checks is a carrier) | Market Town copy **and** `kiosk-yard` copy (`organizations/paper-kiosk.yaml`) | step-29 §11.1 S29-b adversarial | PR (C2) |
| AC-b-7 | **No id moves.** Every place, person and item id of Market Town equals the base's, and `station-news` is the highest organization id | Market Town; `kiosk-yard` (its organization is the last id) | R-S29b-a-1 | PR (C2) |
| AC-b-8 | I-b-1's diff scan is empty | the PR | I-S29-1 | PR (C3) |

| Mutation | Plant | Must fail | When |
| --- | --- | --- | --- |
| X-b-1 | `prices: { newspaper: 200 }` → `{ newspaper: 250 }` in `station-news.yaml` (and 150 → 175 in the variant) | AC-b-1 names the price | milestone |
| X-b-2 | Ivan's job `workplace: station` → `workplace: cafe` (and Mo's → `lane` in the variant) | `the_newsstand_job_is_at_the_shop` names the person and both places | milestone |
| X-b-3 | restore the literal "exactly two job holders" in `market/mod.rs` | AC-b-4 on Market Town fails, naming three holders read from `hired` | milestone |
| X-b-4 | in AC-b-6's copies, strip the `economy:` section from the operator's file only | AC-b-6 still refuses, naming another carrier: the check reads whichever carrier the loader names, not one file | milestone |

`the_newsstand_job_is_at_the_shop` (C2, PR): for every person whose `job:` names an employer that runs a shop, the
job's `workplace` is that shop's place — a content check over any world, run on Market Town and `kiosk-yard`. It
catches X-b-2 and holds Alice's and Felix's jobs to the same rule.

---

## 8. Commit plan

Each commit tracks **implementation**, **deterministic validation** and **review** as separate items; `[x]`
needs the work and its evidence. Per-PR validation is the build plus the commit's own tests (charter); no local
`cargo test --workspace`, no whole-member suite (R-LOAD-1).

Run every command from `/Users/yuema137/mineworld-worktrees/impl-s29b`, with absolute paths. Never `sed -i`,
`python3 -c`, heredoc writes, `curl` or `git add -A`.

#### C0 — Base, prerequisites, ledger (Markdown only)

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s29b.md` (new).
- Steps:
  1. Record `git rev-parse origin/main`. Record the dispatch's statement that Milestone F's checklist was handed
     over. Missing → stop.
  2. S29-a merged: `git log origin/main --oneline -- worlds/market-town/places/station.yaml` non-empty and
     `tools/cli/tests/station_visit.rs` exists. Else stop.
  3. Ivan's routine on the base: `sed -n 1,20p worlds/market-town/people/ivan.yaml` and the same for
     `worlds/social-cafe/people/ivan.yaml` show `07:00 station` until `12:00` in both. Else stop (F-S29b-1 no
     longer holds; Q-S29b-1 is re-asked).
  4. S31-a merged? `git grep -n "keepers" systems/economy/src/section.rs`. Non-empty → branch A; empty → branch B.
     Record it.
  5. Re-read A-1 … A-17 on the base; record moved lines (bounded) or changed facts (stop if a decision here
     changes). In particular the literal at `market/mod.rs` (A-14) and the carrier counts (A-15), which S31-a may
     have moved.
- [ ] Implementation: steps 1–5. [ ] Validation: `python3 scripts/check_doc_headings.py`, `python3
  scripts/check_decision_ids.py` PASS. [ ] Review: every start condition quoted with its evidence.
- **Commit:** `docs(plan): S29-b — open the ledger at the base`

#### C1 — The newsstand: organization, job, and the tests the content moves

- **Files:** `worlds/market-town/organizations/station-news.yaml` (new); `worlds/market-town/world.yaml`;
  `worlds/market-town/people/ivan.yaml`; `worlds/market-town/README.md`; `tools/cli/tests/market/mod.rs`;
  `tools/cli/tests/market_composition.rs`; branch B only: `docs/decisions/ARC-38.md`; this file (§15).
- Steps:
  1. `station-news.yaml` exactly as §5.1, in the branch C0 chose.
  2. `world.yaml`: `organizations:` gains `- station-news` after `corner-store`; the comment above it says
     "The three organizations that run the shops" and names the newsstand; the header's "`job:` for Alice and
     Felix" becomes "`job:` for Alice, Felix and Ivan", and "two of them work" becomes "three of them work". Comment
     lines only besides the list entry (AC-1 compares values, A-10).
  3. `ivan.yaml`: append §5.2's `job:` block and comment after `economy:`. No other line changes.
  4. `README.md`: "the café's and the store's shops for the two organizations" → "the café's, the store's and the
     station newsstand's shops for the three organizations"; "`job:` for two" → "`job:` for three"; "Alice a job at
     the café and Felix one at the store" gains "and Ivan one at the station newsstand"; "two of them work" →
     "three of them work".
  5. `market/mod.rs`: `Town` gains `pub job_holders: BTreeSet<EntityId>`, read in `Town::read` from the pack's
     people whose file carries a `job:` section (through `WorldPack`'s authored sections, or by reading each
     `people/<key>.yaml` with `serde_saphyr` and testing for the key, the method `ac1_composability.rs::read_yaml`
     uses); `market_lives` asserts `holders == town.job_holders` with the message "the job holders read from
     `hired` are the people whose file carries `job:`"; the doc comment's "exactly two" becomes "every person
     whose file carries `job:`".
  6. `market_composition.rs`: `("economy", Some("economy"), 15)`, `("employment", Some("job"), 3)`.
  7. Branch B: §5.3's note in `ARC-38.md`; `python3 scripts/check_decision_ids.py --write-index`.
- Validation (one at a time): `cargo build -p mineworld-cli`; `target/debug/mineworld validate worlds/market-town
  --packs entities --packs presentation/mineworld-default` (reads; prints three organizations' worth of entities);
  `target/debug/mineworld validate worlds/social-cafe …` (unchanged output); `cargo test -p mineworld-cli --test
  market_composition` is a 30-day multi-run test — **not run locally** (milestone); `cargo clippy -p mineworld-cli
  --all-targets -- -D warnings`; `cargo fmt --all --check`.
- [ ] Implementation: steps 1–7. [ ] Validation: the two `validate` runs, clippy, fmt. [ ] Review: no shared key
  of `ivan.yaml` changed (`git diff` shows only added lines after `economy:`); no literal names a job holder.
- **Commit:** `feat(worlds): Market Town's station newsstand — Station News, its shop and Ivan's morning job (S29-b)`

#### C2 — The newsstand's tests, on Market Town and a generated variant

- **Files:** `tools/cli/tests/newsstand.rs` (new); `tools/cli/tests/station_visit.rs`; this file (§15).
- Steps:
  1. `newsstand.rs`, with `mod headless; mod market;`:
     - `fn kiosk_yard(scratch) -> PathBuf`: writes §7's variant (every file literal; branch A adds `keepers`).
     - `the_hall_offers_the_paper` (AC-b-1): `interactions` on Market Town `--place station` and on `kiosk-yard`
       `--place concourse`; parse JSON; assert a `buy` of the kind at the price, both read from the pack's
       organization file (not literals in the assertion).
     - `no_shop_without_economy` (AC-b-3): Social Café and Lakeside, `--place station`: no `buy`, no `shop`.
     - `the_variant_sells_and_pays` (AC-b-2, AC-b-4 variant, AC-b-5 variant): `run kiosk-yard --headless --seed 1
       --days 3 --save`, read facts (`headless::Tables`), assert the pairing and the wage; then `--days 30` and
       `market_lives(&town, &facts, 30)`; print AC-b-5's table.
     - `economy_removed_refuses_the_sections` (AC-b-6): both copies; `validate` fails; the message names `economy`
       and a carrier file.
     - `no_id_moves` (AC-b-7): Market Town's place, person and item ids from `Town::read` equal a list computed
       from the pack's key order (S29-a A-17's rule), and `station-news`'s id is the maximum organization id; the
       variant likewise.
     - `the_newsstand_job_is_at_the_shop` (§7).
  2. `station_visit.rs`: `market_town_buys_the_paper_at_the_station` — on the 30-day seed-7 save the file
     already makes, ≥ 1 sale of the kind `station`'s operator lists, paired as AC-b-2 (milestone; mark `#[ignore]`
     only if the file's convention ignores its 30-day runs, and record which CI layer runs it).
- Validation: `cargo test -p mineworld-cli --test newsstand` (the variant runs are short; record the count and
  PASS/FAIL); clippy; fmt.
- [ ] Implementation: steps 1–2. [ ] Validation: `newsstand` PASS. [ ] Review: every criterion names a second
  world or the variant; no expected value is computed by the code under test.
- **Commit:** `test(cli): the station newsstand sells, pays and is absent without economy, in Market Town and a generated town (S29-b)`

#### C3 — Close-out: MERGE-READY

- **Files:** this file (§15), `handoff-s29b.md`.
- Steps: AC-b-8's diff scan; push `mvp0/pr-s29b`; `gh pr create` (body ending with the generated-with line); wait
  for `fast` on the exact head (poll at most every 5 minutes); repair to green; report **MERGE-READY** with the PR
  number and head SHA. Hand the milestone list (AC-b-2 Market Town half, AC-b-4, AC-b-5, X-b-1 … X-b-4, the
  baseline reason line of §5.9) to the lane lead in the report.
- [ ] Implementation. [ ] Validation: `fast` green on the exact head. [ ] Review: deviations recorded.
- **Commit:** `docs(plan): S29-b close-out`
- **Then:** MERGE-READY — the coordinator merges.

---

## 9. Requirements this PR places on other PRs

- **R-S31f-b-1 (S31-f; branch B only).** Market Town's `station-news` shop gains `keepers: [ivan]` in the same
  commit that adds `keepers: [alice]` and `keepers: [felix]`.
- **R-S31f-b-2 (S31-f).** The market price reporter is not Ivan (Q-S31-4). Ivan keeps the newsstand 07:00–12:00
  every day; the reporter's routine may pass the station (step-31 §9.1) like anyone's.
- **R-S31f-b-3 (S31-f; if Q-S29b-3 is ruled as recommended).** S31-f decides whether Market Town gets a
  `town-news` topic; if it does, Ivan is its publisher (step-31 §8.7), and his notices are published only once
  S31-d's "attempt an incomplete offer by name" exists with a policy that uses it (A-19).
- **R-S31a-b-1 (S31-a, informational; no change to the frozen design).** If S29-b merges first, S31-a's C3 finds
  three shops, a 15-carrier `economy` count in `market_composition.rs`, and `market_lives` reading job holders
  from the pack; its C3 step 6 (E-9's cheapest price from genesis `listed`) covers the newsstand unchanged.
- **R-S29e-b-1 / R-S29f-b-1 (S29-e, S29-f).** The kiosk's open state reads the hall's disclosed `shop`, never the
  organization key `station-news`.
- **R-milestone-b-1 (the milestone batch).** Re-record `[world market-town]` long baselines with §5.9's reason
  line; run AC-b-2 (Market Town), AC-b-4, AC-b-5 and X-b-1 … X-b-4.

## 10. Edits to parent documents (owner: the primary session or the lane lead)

| Document | Edit |
| --- | --- |
| `step-29-station-landmark.md` §7.2 | "S29-b names them" → "Ivan (S29-b, Q-S29b-1): his routine already holds the hall 07:00–12:00 in both towns, so AC-1 check 3 holds" |
| `step-31-wealth-prices.md` §8.1 example, §16.4 | the `town-news` example's publisher, if Q-S29b-3 is ruled (`hana` is an example only) |
| `overall.md` | S29-b state |

---

## 11. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S29-b — the station newsstand in Market Town
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s29b-newsstand.md
RELATED / BINDING DOCS:  step-29-station-landmark.md (§§5.1, 7.2, 9.3, 10, 11, 13, 14.8); pr-s29a-town-contract.md
                         (§§5.5, 5.6, 10); step-31-wealth-prices.md (§§8.7, 9.1, 16.4, 17, 18);
                         pr-s31a-currency-asks.md (§§2.3, 5.4); docs/decisions/ARC-35, ARC-36, ARC-37, ARC-38;
                         docs/MODULE_SPEC.md §4.1; CLAUDE.md; docs/ENGINEERING_STANDARDS.md;
                         /Users/yuema137/mineworld-worktrees/LANE_LEAD_CHARTER.md (SPEED FIRST)
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s29b, its own, held by one session
BRANCH:                  mvp0/pr-s29b, created from origin/main
START CONDITION:         Milestone F's checklist handed over AND S29-a merged AND this design frozen, then the
                         dispatch (step-29 §11.2)
IMPLEMENTATION BASE:     origin/main at C0; C0 records it and the branch (A or B, §5.3)
APPROVED SCOPE:          §2.1 items 1–4, as the rulings of §13 shape them
FROZEN INVARIANTS:       I-S29-1, I-S29-3; §6 I-b-1 … I-b-6; §7 as written
SEQUENCE:                C0 … C3 (§8)
IMPLEMENTER:             Haiku-class, from this plan; review by the lane lead
ALLOWED COMMANDS:        cargo build/test/clippy/fmt -p <crate> (never --workspace; one --test at a time);
                         target/*/mineworld *; python3 scripts/check_doc_headings.py, check_decision_ids.py,
                         ci_layer.py --list; git; gh (pr create/view, run view; never merge); cp; cmp; mkdir -p;
                         sed -n; grep
NEVER:                   python3 -c; sed -i; heredoc writes; curl; git add -A; cargo test --workspace or a whole
                         member suite (R-LOAD-1); reading any key or .env; any model call; edits outside
                         APPROVED SCOPE; any change to a shared key of a person file
MATERIAL STOPS:          a start condition missing; Ivan's routine not 07:00–12:00 at station in both towns; a
                         needed change under kernel/, contracts/, systems/, server/, clients/, cognition/, sdk/;
                         an id moving (AC-b-7); a new dependency
PLATFORMS:               Linux, macOS, Windows (CI)
VALIDATION BUDGET:       per PR: the build and the newsstand tests (the variant runs ≤ 2 min); `fast` CI. At the
                         milestone: the 300-day run, the planted mutations, AC-1, the long baselines
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s29b.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized once the start condition holds
                         semantic commits, branch push, PR creation, CI repair: authorized
                         endpoint: MERGE-READY (pushed; `fast` green on the exact head; PR number and head SHA
                         reported)
                         merge: by the coordinator only
POST-MERGE SYNC OWNER:   lane lead: step-29 §7.2 (§10); implementation session: this ledger
STOP CONDITION:          S29-b MERGE-READY — the coordinator merges
```

---

## 12. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-b-1 | Papers fill the regulars' hands over 300 days and food buys stop (F-S29b-4) | AC-b-5 records it at the milestone; E-9 (AC-b-4) is the gate; Q-S29b-4 decides the sink |
| R-b-2 | S31-a lands between C0 and C1, so the branch is wrong | C1 re-runs C0 step 4 before writing; a changed branch is a bounded correction recorded in §15 |
| R-b-3 | S29-a changes Ivan's routine before merging | C0 step 3 stops; Q-S29b-1 is re-asked with Erin as the alternative |
| R-b-4 | The milestone's E-9 run fails "wage paid to ivan" in some 30-day bucket because he never reaches the station before 12:00 | a shift with no worked time states no `wage-due` (A-8). A whole bucket without one is unlikely (S29-a measures station arrivals, AC-a-7), but if it happens the fix cannot be a later shift end, because the shift must stay inside his shared routine (F-S29b-1). It is then a material finding back to the operator with the measured arrival times |
| R-b-5 | The price source is a New England daily, not San Diego's | stated in §5.4; one-line replacement if a local source is found |

---

## 13. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S29b-1** | **Who sells the paper?** AC-1 forces the seller's routine to equal Social Café's, so only someone already at the station in the morning in both towns can take the job without a routine change. | **Ivan** (07:00–12:00 at the station after S29-a; chatty; jobless; not the reporter). Alternative: Erin (07:30–13:00). |
| **Q-S29b-2** | **Wage scale.** The town pays Alice 120/h and Felix 200/h, far below a real wage (California's state minimum wage, $16.90/h for 2026 as announced by the state, would be 1 690). Pay Ivan on the town's scale or a real wage? | **The town's scale, 120/h.** One real wage beside two toy ones would make Ivan the town's richest person by day 30 and skew S31's wealth bands; rescaling every wage is town-wide content for S31-f. |
| **Q-S29b-3** | **`town-news` publisher.** Should S29-b make Ivan a `town-news` publisher (step-31 §8.7), or leave it to S31-f? | **S31-f.** It owns Market Town's notices configuration; and an NPC publisher publishes nothing until S31-d's "attempt an incomplete offer by name" lands. |
| **Q-S29b-4** | **Papers never leave the world** (goods are not consumed), so regulars may fill their six slots with papers. Add a sink now? | **Not in S29-b.** A sink is a System change (forbidden here). Measure it at the milestone (AC-b-5); if hands fill, a later PR adds "read and discard" to `consumption` with a ruling. |
| **Q-S29b-5** | **If S29-b merges before S31-a**, may the newsstand sell goods with only a dated note under `ARC-38` (no code enforces item 5)? | **Yes.** Step-29 §7.2 and step-31 §16.4 already approve the shop; the note records the departure until S31-a lifts item 5. |

### 13.1 Rulings

(none yet)

---

## 14. Checks run on this design

- `python3 scripts/check_decision_ids.py` (2026-10-18, on this branch): "175 decision ids, all distinct; index
  current" — PASS. No record added.
- `python3 scripts/check_doc_headings.py`: it scans `docs/**` only, so it does not cover this file; run anyway:
  "243 numbered sections across 251 documents, none duplicated" — PASS.

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 … C3 | — | [ ] | [ ] | [ ] |

### Evidence

(none yet)

### Milestone hand-over

(the reason line of §5.9; AC-b-2 Market Town half, AC-b-4, AC-b-5, X-b-1 … X-b-4)

### Deviations and findings

F-S29b-1 … F-S29b-7 (§3.2), recorded at design time.
