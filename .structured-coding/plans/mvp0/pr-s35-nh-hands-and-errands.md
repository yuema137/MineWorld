# PR S35 N-h — full hands, empty shops and the lateness test (F-Ng-2, F-Ng-3, F-Nh-1 … F-Nh-4)

```text
STATUS        DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as
              recommended). Implemented on mvp0/pr-s35-nh; READY FOR OPERATOR REVIEW once CI is green.
step          35 (needs and sleep), step-35-needs-and-sleep.md; milestone-batch fix after N-f and N-g
branch        mvp0/pr-s35-nh (implementation, after freeze); this design on docs/pr-design-s35-nh
base          origin/main @ f92e3cad for the audit; implementation rebases on main after N-f (#370) and
              N-g merge, because AC-Nh-4 and AC-Nh-5 are measured on top of their walking fixes
owns          item-transfer (put-down, pick-up), inventory (a Place as a holder; capacity disclosed),
              needs (nourishment of offered kinds disclosed), economy (a shop's stock disclosed),
              rule-controller (the fetch band; commute reads the disclosed shift), tests
touches no    kernel, contracts crate, movement, bodies, clients' rendering
decision id   ARC-360 (picked above every id in use on 2026-10-20; the coordinator may renumber)
parents       N-g design (#382, `pr-s35-ng-walkers.md` §2.3, §2.4, §8) traced F-Ng-2 and F-Ng-3
```

## 1. Why this PR exists

N-g's trace (#382, §2) found that two milestone rows in Market Town do not fail because of walking:

- **AC-S35-8**, "nobody famished for more than 24 h": the longest stretches are 48 h (seed 1, Grace),
  36 h (seed 7, Bob) and 67 h (seed 42, Dev).
- **AC-LE-3**, "nobody arrives more than an hour early": every early arrival is Felix, the store's clerk.

The coordinator added a third failure: `tools/cli/tests/lateness.rs` fails on Market Town. The default policy
produces `late-for-shift` complaints, and the at-boundary case does not see the expected set of late employees.

This PR finds the generic cause of each and fixes the mechanism, not the town. Every rule below names no place,
person or world, and every acceptance row runs on a second world or a generated variant.

## 2. Audit

### 2.1 What was read

| Area | File / symbol | What it says |
| --- | --- | --- |
| capacity | `systems/inventory/src/admit.rs` `PERSON_CAPACITY` (6), `can_take`, `is_holder` | A Person carries at most six items, every kind together; an Organization has no bound; only a Person or Organization can hold. The doc comment gives the reason: *"nothing in MVP-0 uses an item up"*, so an undriven person absorbed the town's items (step-10 F-39, `ARC-37` item 5) |
| buy | `systems/economy/src/sale.rs` `judge_sale`, `buys` | A `buy` is offered with a refusal code: `out-of-stock` when the seller holds none, `cannot-carry` when `can_take(buyer, 1)` is false, `insufficient-funds`, `not-for-sale`. The code reaches the observer as the affordance's `unavailable_reason` (`contracts/src/observation.rs` `Affordance::unavailable_reason`) |
| give | `systems/item-transfer/src/offer.rs` `gives` | One complete `give` per held kind, to each other living person present; available when the target can take one more. It is the only way a good leaves a person's hands |
| eat / drink | `systems/consumption/src/offer.rs` `meals` | One complete `eat` or `drink` per held kind whose category is food or drink, anywhere. Tea and coffee are drinks: they can be drunk, they nourish 0 |
| nourishment | `systems/needs/src/disclose.rs` | `needs` tells the holder how much each kind **they carry** would nourish them. It says nothing about a kind they do not carry, such as one a shop offers |
| fetch band | `cognition/rule-controller/src/needs.rs` `NeedsPolicy::fetch` | Hungry from `fetch_from`, nothing nourishing held: buy an **available** `buy` of a kind not held, lowest `ItemId`; else, if *here sells*, do nothing; else walk to the lowest-`PlaceId` shop-tagged doorway place; else out of a one-door place. It never reads `unavailable_reason` |
| rule memory | `cognition/rule-controller/src/lib.rs` | The rule controller keeps no world state (only which utterance it answered). So "I already found that shop empty" can only come from a disclosed fact |
| restock | `systems/employment/src/system.rs` `ended`, `component.rs` `Job::produced_for` | A shift's `produces` is stated once, **when the shift ends**, prorated by time worked |
| Felix | `worlds/market-town/people/felix.yaml` | Job at the store 08:00–13:00, `produces { apple 24, bread 18, milk 6, juice 6 }`; routine part at the store from 08:00, `punctual: true` |
| Ivan | `worlds/market-town/people/ivan.yaml` (S29-b, `fbf3edf7`, 2026-10-19) | Job at the station 07:00–12:00 for `station-news`; routine part at the station from 07:00, label `meet`, **not punctual** |
| commute | `cognition/rule-controller/src/commute.rs` `due`, `holds` | A commute is planned only for a routine part with `punctual: true`. The job itself is never read |
| employment disclosure | `systems/employment/src/system.rs` `discloses` | The employee is told their own `Employment`: the job's employer, workplace, `from`, `until` |
| lateness test | `tools/cli/tests/lateness.rs` | AC-LE2-1: zero `late-for-shift` on the default policy in Market Town and Commute Town. AC-LE2-2: at-boundary, the number of distinct late employees equals a literal: **2** for Market Town, 4 for Commute Town |
| early arrivals | `tools/cli/tests/punctuality.rs` (AC-LE-3, F-Nd-7) | An arrival more than an hour before a punctual part's `from` counts as too early unless the person received an item there (an errand) |

### 2.2 F-Ng-2, hands full: the causal chain

1. Since `ARC-37`, a person carries at most six items. The bound was set when nothing used an item up.
2. People are given goods (`give`) by others, and the fetch band buys a kind it does not hold **without knowing
   whether it nourishes** (needs discloses nourishment only for carried kinds). So drinks such as coffee and tea
   are bought as "food" and then do not help. Bob's six (tea ×2, coffee, book, soap, mug) and Dev's (coffee
   among goods) show both routes. This contributing route is **F-Nh-2**; C1's trace confirms how many of the six
   came by `buy` against `give`.
3. With six held, every `buy` is offered `refused(cannot-carry)`.
4. The fetch band looks only for an *available* `buy`. It finds none, decides "here sells, so nothing more", and
   does nothing. It never reads the refusal code, which already says *why*.
5. Goods have no way out of a person's hands except `give`, and only to someone present who has room. Drinks
   could be drunk, but the band drinks only what nourishes.
6. The person leaves on their routine, comes back on the next consult, and repeats this 33–108 times (N-g §2.3).

So there are three defects, at three owners: the controller ignores a disclosed reason (rule-controller); the
world offers no generic way to put a good down (item-transfer and inventory); and the world does not disclose
what an offered kind would nourish (needs).

### 2.3 F-Ng-3, the empty shop: the causal chain (with one correction to N-g)

1. The store sells out every day (N-g §2.4: 24 of 24 apples sold each measured day).
2. **Correction to N-g §2.4:** the store is restocked when Felix's shift **ends**, at 13:00, not at 08:00.
   Employment states `items-produced` in `ended` (`system.rs`), prorated by time worked. So the shelves are
   empty from the sell-out until 13:00 the next day.
3. Felix is hungry at 06:00. The fetch band picks the lowest-`PlaceId` shop doorway: the store.
4. He arrives at 06:15–06:46. Every food `buy` is `refused(out-of-stock)`. The band says "here sells: nothing
   more". Because the store is also his punctual 08:00 part, `waiting` holds him there until 08:00.
5. No item is received, so F-Nd-7 counts the arrival as a commute arrival more than an hour early.

The defect is that a person cannot know a shop is empty until they stand in it, and a memoryless controller
cannot remember that it was. Any world whose shop restocks after the morning (a generated variant does exactly
this, AC-Nh-5) reproduces it.

### 2.4 F-Nh-1, `lateness.rs` on Market Town: the test and the content are both wrong

**Evidence:** `cargo test -p mineworld-cli --test lateness` on `origin/main` @ `f92e3cad` fails 2 of 3, and
every default-policy complaint is Ivan's (§2.6, L1 and L2).

The cause is the station newsstand, S29-b (`fbf3edf7`, 2026-10-19), merged after `lateness.rs` was written:

- Market Town gained a third employee, Ivan (station, 07:00–12:00). The at-boundary test still asserts the
  literal **2** late employees for Market Town. A literal count of employees is a fact about the content at one
  moment, so any change to the town breaks it.
- Ivan's 07:00 station part is **not punctual**. The rule commute plans only for punctual parts, so on the
  default policy Ivan sets out at 07:00 and arrives one walk late, every day. That is the default-policy
  complaint. It is not a defect in lateness: lateness is right to complain.

So the test is wrong in one place (the literal count), and the behaviour is wrong in another (an employee with
a shift does not commute for it). Fixing only Ivan's file would be the world-specific fix the charter rejects: the
next world author who forgets `punctual: true` on a job's part gets the same lateness. The generic fix (R6) is
that a controller treats the shift its employment discloses as an appointment, as it does a punctual part.

### 2.5 F-Nh-3, the rule's "here sells" also stops a person whose money ran out

The same fetch-band row ("here sells: nothing for me") swallows `insufficient-funds`. It does not fail an
acceptance row today (N-g §2.3: the famished people had ample money), so this PR only makes the band read the
reason and record which reason stopped it (§4, R1); the policy for being broke is not added.

### 2.6 Evidence log

| Run | Command | Result |
| --- | --- | --- |
| L1 | `cargo test -p mineworld-cli --test lateness -- --nocapture --test-threads=1`, `origin/main` @ `f92e3cad`, macOS, through `cast-slots/with-slot.sh` (build 1 m 16 s, tests 15.6 s) | **FAIL 2 of 3.** `on_the_default_policy_nobody_is_late`: Market Town has complaints (line 123). `setting_out_at_the_boundary…`: Market Town "18 late-for-shift, 3 employees", left 3 ≠ right 2 (line 145). The strict-boss case (Commute Town) passes: 12 told, 12 penalties |
| L2 | `mineworld run worlds/market-town --headless --seed 1 --days 7`, then `mineworld perceived --person <p>` for each employee | 7 `late-for-shift` in 7 days: **Ivan 7**, Felix 0, Alice 0 — one a day, Ivan's un-commuted 07:00 station shift. Confirms §2.4 |

The F-Ng-2 and F-Ng-3 numbers are N-g's trace (#382 §2.3–§2.4, on N-f's head). They were not re-run on main,
whose Market Town has since gained the station (S29-a/b); C1 re-measures them on the implementation base.

## 3. Established approaches compared

### 3.1 Carrying: how real people and established games bound what a person holds

| Model | Where it is used | What bounds carrying | What happens when full | Fit here |
| --- | --- | --- | --- | --- |
| **Slots, count only** | MineWorld today (6); Stardew Valley (12–36 slots), Animal Crossing pockets (20–40) | number of things | the game refuses the pick-up; the player drops, sells, eats or stores at home | What we have. The bound is fine; the missing piece is a way out (drop, store) |
| **Slots with stacking** | Minecraft (36 slots, stacks of 64), Terraria | number of kinds, a stack size per kind | drop (`Q`), chests at home | Changes every item's data (stack size); does not by itself free a hand |
| **Weight (encumbrance)** | Skyrim, Fallout (carry weight; overweight = slow); real-world load guidance (pack loads of 10–20 % of body weight for daily carry; NIOSH's 23 kg lifting constant) | total mass | slower, then cannot move | Needs a mass per kind in every world and a movement penalty owned by movement; does not remove the deadlock, only moves it |
| **Volume / grid by size** | Resident Evil, Diablo, Escape from Tarkov; RimWorld and Dwarf Fortress use mass and bulk | shape or volume | drop, rearrange, containers | Most realistic for "an umbrella and a scarf fit in a bag"; most data per kind; out of scope for a fix PR |
| **Carried vs kept at home** | The Sims (personal and household inventory), Animal Crossing (home storage), real life | you carry little and keep most things at home | put it away at home | The realistic pattern. Needs a holder that is a place |

What every one of these has, and MineWorld lacks, is **a way to put something down**. A bound on carrying
without a way to put things down means a full person is stuck forever. The slot bound itself is not the defect.

### 3.2 How to free a hand: options

| Option | Owner | What it adds | For | Against | Verdict |
| --- | --- | --- | --- | --- | --- |
| **(a) Controller only:** drink a non-nourishing drink, or give something to someone present | rule-controller | nothing in the world | No contract change; fixes the three traced people (each holds a drink) | A person holding only goods, alone, stays stuck. A fix that works because the three people happen to hold tea is world-shaped | **Adopt as the first resort**, not the whole fix |
| **(b) Put down / pick up, a Place holds items** | item-transfer (actions), inventory (holdings) | `put-down { item }`, `pick-up { item }`; a living Place may hold items, unbounded; place holdings disclosed to those present | The pattern every game in §3.1 has; generic; also gives "food stocked at home" for free (§3.3); items are conserved, so economy's reference values and the item count stay meaningful | A new holder type; places accumulate items over a month (bounded by the town's item count, which is conserved); clients show them as a list, not as placed objects, until a later visual PR | **Adopt** |
| **(c) Discard:** an item leaves the world | inventory | `discard { item }`, `items-discarded` | Smallest world change | Destroys goods; nobody throws away an umbrella to buy an apple; hides a supply problem | Rejected |
| **(d) Sell surplus to a shop** | economy | shops buy back kinds | Realistic for some goods | Needs every shop to want goods; a shop that does not buy candles leaves the person stuck; economy policy, not a carrying rule | Rejected for N-h; possible later |
| **(e) Weight or volume capacity** | inventory, item data | mass/volume per kind | Realistic | Content for every item of every world; still needs (b) when full | Deferred (Q-Nh-6) |
| **(f) Raise or remove the bound** | inventory | — | Trivial | Brings back step-10 F-39: an undriven person absorbs the town | Rejected |

### 3.3 The empty shop: options

| Option | Owner | What it adds | For | Against | Verdict |
| --- | --- | --- | --- | --- | --- |
| **(a) Restock timing as data** (produce per hour worked instead of at shift end) | employment | `produces` stated each completed hour | Realistic for a bakery | Felix is hungry at 06:00, before any shift; changes every world's supply curve; supply sizing is content, and the mechanism must hold when a shop opens late | Rejected as the fix; Q-Nh-7 |
| **(b) Eat at home / stock at home** | content + (b) of §3.2 | food held by a home place, picked up there | Realistic; free once a Place holds items | Content: a world without stocked homes still fails | Comes with §3.2 (b); not the fix by itself |
| **(c) The shop discloses what it has in stock** to anyone who perceives the place, as a shop window or a "sold out" sign does | economy | the shop's in-stock kinds on the shop place's disclosed record (kinds, not counts) | The world discloses a fact; each controller decides: `rule` skips a shop with nothing that feeds it, `motivated` scores it, an LLM reads it. Fixes the memoryless controller without memory | A new disclosed record; "seeing the shelves from the doorway" is a mild idealisation (real analogues: a window, an online stock check) | **Adopt** |
| **(d) Count a fetch errand as an errand** in F-Nd-7's early-arrival test | test | the walk's purpose | Cheap | Hides the behaviour: Felix still walks to an empty shop and waits 90 minutes hungry | Rejected |
| **(e) Controller memory of empty shops** | rule-controller | state in the controller | — | Breaks the rule controller's "holds no world state" contract (`lib.rs`) and its rebinding guarantee | Rejected |

### 3.4 The un-commuted shift: options

| Option | Owner | For | Against | Verdict |
| --- | --- | --- | --- | --- |
| **(a) Content:** `punctual: true` on Ivan's part | world | One line | World-specific; the next author repeats it | Do it, but not as the fix |
| **(b) A controller treats its disclosed shift as an appointment** | rule-controller `commute`, motivated, LLM prompt | Generic: the world already discloses the job to the employee; a shift is a time and a place one must be at, whatever the routine says | A second source of "be there by" beside the punctual flag (Q-LE-2 chose the flag); the two must agree: the earlier of the two `from`s at the same place wins | **Adopt** |
| **(c) Genesis check:** refuse a world whose job has no punctual routine part at its workplace and `from` | employment reading schedule | Catches the authoring error | Couples employment to schedule; a person with a job and no routine is valid | Rejected |
| **(d) Test derives the employee count** from the run's `hired` facts | test | Removes the literal | — | **Adopt**, alongside (b) |

## 4. Recommendation

### R1 — the fetch band reads why a `buy` is refused (rule-controller)

```text
hungry ≥ fetch_from, nothing nourishing held, standing where a food buy is offered:
  available buy of a kind that nourishes me (R3)        → buy it (the most nourishing; ties: lowest ItemId)
  refused cannot-carry, and that kind nourishes me      → free one hand (R2's order), this consult
  refused out-of-stock / not-for-sale for every food    → here has nothing for me: leave by the fetch walk
                                                          rule below, unless `waiting` holds me (D-Nc2-5)
  refused insufficient-funds                            → nothing more (F-Nh-3; recorded, no new policy)
```

"Free one hand" is policy data, `free_hand: [drink, eat, put-down, give]`, tried in order, each from what the
world offers available right now:

- `drink` / `eat`: the held kind that nourishes **least** (ties: lowest `ItemId`). Drinking a cup of tea to make
  room is what a person would do.
- `put-down`: the held kind of which I hold most, then lowest `ItemId` (R4).
- `give`: to the person present with room, lowest entity id, the same kind order.

An empty list turns the step off. The default list is the order above. A world may reorder it in
`decisions.yaml`. No place, item or person is named.

### R2 — capacity is disclosed (inventory)

Inventory's disclosed `holdings` gains `capacity` (a Person: `PERSON_CAPACITY`; an Organization or a Place:
absent, meaning unbounded). No controller hard-codes 6. The constant stays a published constant (`ARC-26`
note): the weight-or-volume model is Q-Nh-6.

### R3 — nourishment of what a shop offers is disclosed (needs)

`needs`' disclosed `nourishment` lists, beside the carried kinds, every kind named by a `buy` offered to me
where I stand. A controller can then tell an apple from a coffee before buying. The band buys only a kind that
nourishes me (fixing F-Nh-2's route of filling one's hands with drinks).

### R4 — a Place can hold items; `put-down` and `pick-up` (item-transfer, inventory)

```text
holder        inventory: a living Place may hold items, unbounded (like an Organization)
put-down      item-transfer action { item }: one of a held kind from me to the place I stand in;
              requirement: in that place; refused when I hold none. Facts: inventory's
              items-transferred { from: me, to: place }, heard in the place
pick-up       item-transfer action { item }: one from the place I stand in to me; requirement: in that
              place; refused cannot-carry when I cannot take one more, not-here when the place holds none
offered       to a person in a place: one complete put-down per held kind; one complete pick-up per kind
              the place holds, each refused with its reason
disclosed     a place's holdings to everyone perceiving that place (things lying in a room are seen)
```

The fetch band also gains a first source: **food lying here** (a `pick-up` of a kind that nourishes me) before
any `buy`. So a world that stocks a home's pantry feeds its people at home with no further code, and an LLM seat
sees the same offers.

The rule controller never picks up anything else, so goods put down stay where they are. That is bounded:
items are conserved and the town's total does not grow.

### R5 — a shop discloses what it has in stock (economy)

The shop's place carries, to anyone who perceives it, the asked kinds the operator holds at least one of
(kinds only, not counts). The fetch band's walk row then reads:

```text
a doorway to a shop-tagged place whose disclosed stock holds a kind that nourishes me (R3 extended to
disclosed stock kinds)                      → walk there (lowest PlaceId of them)
a shop-tagged doorway with no disclosure    → as today (a world without economy's disclosure)
every shop-tagged doorway discloses nothing for me → no walk: stay on the routine (F-Ng-3)
a place with one doorway                    → walk out through it (unchanged)
```

The disclosure is economy's, read by the controller by slug (no crate link, `ARC-35` check 2).

### R6 — a disclosed shift is an appointment (rule-controller commute; motivated; LLM prompt)

The commute treats the employee's disclosed shift (`employment`: workplace, `from`) as a punctual part at the
workplace for that day, when the routine does not already have one there at or before `from`. The earlier `from`
at the same place wins. Applies only when `employment` is installed and disclosed to me. The motivated
controller reads the same record; the LLM prompt lists the shift under "be there by".

`lateness.rs` AC-LE2-2 derives each world's employee set from its `hired` facts instead of a literal.
Ivan's file also gets `punctual: true` (content consistency), but AC-Nh-3 is run on a generated variant in
which every job's routine part has its `punctual` flag removed, so the fix cannot depend on the flag.

## 5. Acceptance

Thresholds are fixed here, before any run. Every row is measured at the milestone batch except where marked.

| ID | Criterion | World(s) |
| --- | --- | --- |
| AC-Nh-1 | AC-S35-8's famished clause: nobody at hunger ≥ 900 for more than 24 consecutive hours; and no `buy` refused `cannot-carry` is followed by the same person's next consult doing nothing (each is followed by a free-hand action or an accepted `buy`) | Market Town, 30 days × seeds {1, 7, 42}; Commute Town 30 days × {1, 7, 42}; generated **"full-hands"** variant (below) |
| AC-Nh-2 | AC-LE-3: no arrival more than an hour before a punctual part's `from`; no fetch walk to a shop whose disclosed stock holds nothing that nourishes the walker | Market Town × {1, 7, 42}; generated **"late-shop"** variant (below) |
| AC-Nh-3 | `lateness.rs` green: AC-LE2-1 zero complaints on the default policy; AC-LE2-2 every employee (from `hired`) late at least once at-boundary | Market Town, Commute Town, generated **"no-flag"** variant (below) |
| AC-Nh-4 | No regression: every AC-S35-8 row other than the famished clause, AC-LE2-1 and N-g's AC-Ng-4 / AC-Ng-6 rows pass as they do on N-g's head | Market Town, Commute Town |
| AC-Nh-5 | Scenario tests (per commit): a full person at a shop with an apple and a held tea drinks the tea and buys the apple; a person holding six goods alone puts one down and buys; a place's holdings are disclosed to a second person present, who picks one up; a shop with no stock is not walked to | `systems/*/tests`, `cognition/rule-controller` tests |
| AC-Nh-6 | Replay and kill/resume reproduce Market Town seed 7 byte for byte on the three platforms (CI at the milestone) | Market Town |

Generated variants, built at test time and never checked in, from the same files the tests already copy:

- **full-hands Commute Town:** every resident starts holding `PERSON_CAPACITY` goods (no food, no drink), so
  the drink-first route cannot be what feeds them.
- **late-shop Commute Town:** every shop's stock starts at zero and the only restock is a shift ending at
  13:00, so the morning's shops are empty.
- **no-flag Market Town:** every routine part's `punctual` flag is removed; jobs are unchanged.

## 6. Commit plan

| # | Commit | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C1 | `test(cli): pin full hands, empty shops, the un-commuted shift` — the three generated variants; AC-Nh-1 … -3 as `#[ignore]` red tests with finding ids; a trace count of how the famished six were acquired (`buy` vs `give`, F-Nh-2) | [ ] | [ ] red on the base with outcomes recorded | [ ] |
| C2 | `fix(test): lateness counts the employees it hired` — AC-LE2-2 from `hired` | [ ] | [ ] `lateness.rs` boundary case PASS on Market Town | [ ] |
| C3 | `feat(inventory): a place holds items; capacity disclosed` — `is_holder` takes a living Place; `can_take` unbounded for it; holdings record `capacity`; place holdings disclosed to perceivers; component schema version raised | [ ] | [ ] inventory tests; replay of a fresh save | [ ] |
| C4 | `feat(item-transfer): put down and pick up` — R4 actions, offers, refusals, facts | [ ] | [ ] AC-Nh-5 rows 2 and 3 | [ ] |
| C5 | `feat(needs, economy): disclose what an offered kind nourishes and what a shop has in stock` — R3, R5 | [ ] | [ ] needs and economy disclosure tests | [ ] |
| C6 | `feat(rule): the fetch band reads the reason, frees a hand, picks up food, skips an empty shop` — R1, R4's first source, R5's walk rule; `free_hand` policy data and its schema | [ ] | [ ] AC-Nh-5 rows 1 and 4; rule-controller tests | [ ] |
| C7 | `feat(rule, motivated, lm): a disclosed shift is an appointment` — R6; Ivan's `punctual: true` | [ ] | [ ] AC-Nh-3 on the no-flag variant | [ ] |
| C8 | docs — `MODULE_SPEC.md` (inventory, item-transfer, needs, economy, rule sections), packs' READMEs, generated reference, decision record, step-35 ledger | [ ] | [ ] `check_decision_ids.py`; §5 rows at the milestone | [ ] |

### 6.1 Execution contract (to be filled at freeze)

```text
worktree      /Users/yuema137/mineworld-worktrees/impl-s35-nh, branch mvp0/pr-s35-nh
authority     DESIGN FROZEN 2026-10-20; Q-Nh-1 … Q-Nh-7 as recommended; decision id ARC-360
validation    speed-first: each commit runs its own tests and `fast`; AC-Nh-1 … -4, -6 in the milestone batch;
              heavy runs through cast-slots/with-slot.sh
stop          any change to a frozen invariant, a public contract beyond §4 (Place holder, put-down/pick-up,
              the three disclosures, `free_hand`), or scope
```

### 6.2 Deviations and bounded discoveries

- **D-Nh-1 (bounded).** R3 named kinds offered where I stand; R5's walk rule needs the nourishment of kinds
  stocked by shops elsewhere, so `needs` discloses `kinds`, every food kind of the world (a superset).
- **D-Nh-2 (bounded).** R5 named "to anyone who perceives the shop's place"; perception lists only the place one
  stands in, so `shop-stock` is disclosed to each living Person about itself (the currency-record pattern).
- **D-Nh-3 (bounded).** A shop the observer keeps lists no stock in `shop-stock`: a keeper's `buy` is refused
  there, and without it a shopkeeper walks back into his own shop for food he may not buy.
- **D-Nh-4 (bounded).** With `shop-stock` disclosed and no shop anywhere stocking food for me, the one-door
  rule does not fire either (else home ↔ street ping-pong); with food somewhere, it does.
- **D-Nh-5 (bounded).** `put-down` / `pick-up` are decided by name only: excluded from the rule's offered band
  (`DECIDED_BY_NAME`, `ARC-151`) and from the motivated model's generic option, so nobody drops things by chance.
- **D-Nh-6 (bounded).** `punctuality.rs` asserted literal shift counts too (14 for Market Town; S29-b made it
  21): it now derives parts and shifts from `routine-assigned` and `hired`, like `lateness.rs`.
- **D-Nh-7 (bounded).** The second clause of AC-Nh-1 ("no `cannot-carry` refusal followed by a do-nothing
  consult") is an observation fact, not a log fact; it is covered by the rule-controller scenario tests, and
  the full-hands variant's milestone row measures its outcome (everyone eats, nobody famished > 24 h).
- **D-Nh-8 (bounded).** Found on Market Town seed 1: from the street the station estimate was short enough that
  the commute was no longer due, the fetch band sent a hungry Ivan to the store, and from the store it was due
  again — to and fro until 07:00. `Commute::due` now stays due while my own walk already goes to the punctual
  part's place (once set out, keep going), in every world.
- **D-Nh-9 (bounded, after merge #429).** The milestone run 39509874835 failed AC-1 check 3: Ivan's Market Town
  routine must equal Social Café's. The `punctual: true` content edit is reverted; R6 keeps his shift.
- **D-Nh-10 (bounded, after merge #429).** The full-hands variant installs `item-transfer` (Commute Town has
  none, so nobody could put a thing down: 0 meals); `punctuality.rs` saves are unique per run (two tests of
  one binary collided on one scratch name).
- **Findings (milestone, local macOS, 30 days):** AC-Nh-1 PASS (Market Town longest famished 7.5 / 6.5 / 9.2 h for
  seeds 1/7/42, was 48 / 36 / 67 h; Commute Town 0.5 / 1.5 / 1.8 h; full-hands 1.8 h ×3). AC-Nh-2: Market Town
  seed 1 0 too early; late-shop variant seed 1 **3 too early** (F-Nh-5, open: arrivals 1.2–2.5 h before a
  punctual part with no errand). AC-S35-8 sleep rows FAIL on Market Town seeds 1, 7 (5.81 h, 6.89 h; 757 ‰
  asleep at 03:00 on seed 1) — not N-h's rows; Social Café seed 1 one part +41 min (F-Nh-6, open).

## 7. Records

- `ARC-3xx` "Putting things down, seeing shelves, keeping shifts": §3's comparisons, R1–R6, and why
  discard, sell-back, weight capacity, restock timing and controller memory were not adopted. It amends
  `ARC-37` item 5 (the bound's reason: a full person now has a way out), `ARC-26` (a Place is a holder),
  `ARC-286` (the fetch band) and `ARC-280` / Q-LE-2 (a disclosed shift is an appointment beside the flag).
- Step-35 ledger: F-Ng-2, F-Ng-3, F-Nh-1 … F-Nh-3 and N-g §2.4's restock-time correction.

## 8. Not in this PR

- Weight or volume capacity (Q-Nh-6), stack sizes.
- Rendering items lying in a place as placed objects in the 2D and 3D clients (they appear in the place's
  list first); the no-interpenetration check for placed items comes with that visual PR.
- A policy for being broke (F-Nh-3), selling surplus to shops.
- Changing when employment states production (Q-Nh-7).

## 9. Freeze questions (with recommended answers)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

| ID | Question | Recommended answer |
| --- | --- | --- |
| **Q-Nh-1** [OPERATOR] | Keep the six-item bound, and fix "hands full" by giving the world a way to put things down (`put-down` / `pick-up`, a Place can hold items), rather than discarding items, selling them back, or removing the bound? | **Yes.** Every carrying model in §3.1 has a way to put things down; ours lacks only that. Discard destroys goods, sell-back needs every shop to buy, and no bound brings back F-39 |
| **Q-Nh-2** | The rule's free-hand order `drink, eat, put-down, give`, as policy data `free_hand` a world may reorder or empty? | **Yes.** Drinking the tea first is what a person does; putting down covers a person holding only goods |
| **Q-Nh-3** [OPERATOR] | `needs` discloses what a kind nourishes for kinds offered to me for sale here, not only kinds I carry? | **Yes.** Without it the band buys coffee as food and fills its hands (F-Nh-2) |
| **Q-Nh-4** [OPERATOR] | A shop discloses which asked kinds it has in stock (kinds, not counts) to anyone who perceives its place, so a controller skips an empty shop? | **Yes.** It is the only fix that works for a memoryless controller in a world whose shop opens late; counting the errand as an errand only hides the hungry wait |
| **Q-Nh-5** [OPERATOR] | A shift disclosed by employment counts as an appointment for the commute (beside the `punctual` flag, earlier `from` wins), and `lateness.rs` counts employees from `hired`? | **Yes.** The lateness failure is Ivan's job (S29-b) with no punctual part plus a literal "2" in the test; fixing only Ivan's file is the world-specific fix the charter rejects |
| **Q-Nh-6** | Weight- or volume-based capacity: not in N-h? | **Not now.** It needs data on every item of every world and still needs put-down when full; record it as a later option |
| **Q-Nh-7** | Leave production at shift end (no hourly restock) in N-h? | **Yes.** Felix is hungry before any shift starts; restock timing is content, and AC-Nh-2's late-shop variant proves the mechanism without it |
