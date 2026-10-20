# PR N-d — Market Town gets hungry and sleeps: content, supply sizing and the step's acceptance

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: every question answered with its recommendation; Q-LE2-1, Q-Nb-1, Q-Nd-1 and Q-Ne-1 explicitly yes)

```text
Design revision:        revision 1 (2026-10-18)
Parents:                step-35-needs-and-sleep.md (STEP DESIGN FROZEN 2026-10-18) §§5.5, 7, 9, 10.1 row N-d, Q-S35-7,
                        Q-S35-8 (ruled: raise production, set prices, in N-d); pr-le-leave-early.md revision 2 §0.8
                        (commute-town); pr-s35-nc-controllers.md §6 (the test copies this PR replaces)
Depends on:             N-c (and through it N-a, N-b, LE-a)
Implementation base:    origin/main at C0, after N-c has merged
Start condition:        this design FROZEN (Q-Nd-1 ruled) AND N-c merged
Execution contract:     §9
Lifecycle:              DESIGN FROZEN 2026-10-18
```

**What this PR does, in one paragraph.** N-a to N-c build the mechanisms and test them on copies. This PR turns
them on in a real reference world: Market Town enables `sleep` and `needs`, every resident gets needs, every food
gets a nourishment value, and the town makes and prices enough food that people can eat three meals a day for 300
days. It is content only (operator ruling R-LE2-2, 2026-10-18: nothing tuned to Market Town in code). It then runs
the step's acceptance — rhythm, determinism, removal and save growth — on Market Town and on a second world,
`commute-town`, with no code change between them. Under SPEED FIRST the long runs are written here and run at the
step milestone.

---

## 1. Audit (`origin/main @ a1e05874`, 2026-10-18; C0 re-checks on the base)

### 1.1 Market Town today

| Thing | Where | Value |
| --- | --- | --- |
| Systems | `worlds/market-town/world.yaml` | Social Café's list, then the six market packs, then `calendar`, `weather`; `configure: [calendar, weather]` |
| People | `population:` | 12 (alice, bob, carol, dev, erin, felix, grace, hana, ivan, otto, visitor, wanderer); 11 seats (otto is not seated) |
| Jobs | `people/alice.yaml`, `people/felix.yaml` | Alice: café, 05:30–14:00, 120/h, `produces { coffee 4, tea 2, croissant 3, cake 2, sandwich 2, soup 2 }`. Felix: store, 08:00–13:00, 200/h, `produces { apple 4, bread 2, milk 2, juice 2 }`. One shift a day each (employment's `next`, every day) |
| Wallets | people files | Alice 20 000, Felix 20 000, everyone else 200 000 minor units |
| Shops | `organizations/cafe-company.yaml`, `corner-store.yaml` | café prices: coffee 300, tea 250, croissant 250, cake 400, sandwich 600, soup 500; store: apple 100, bread 300, milk 200, juice 250. Wallets 20 000 and 300 000 |
| Food and drink | `items/*.yaml` `item.category` | food: apple, bread, cake, croissant, sandwich, soup; drink: coffee, juice, milk, tea |

### 1.2 The AC-1 composability check

`tests/acceptance/tests/ac1_composability.rs` check 3 holds Market Town to "Social Café plus the six market packs,
`items/`, `organizations/` and the sections they own, plus allow-listed generic packs" — `GENERIC_PACKS =
["calendar", "weather"]`, with the comment "Each is added by the PR that brings its pack" (`ARC-35` note of
2026-10-09, operator ruling on S19 TW-a). A people-file key Market Town adds must be owned by a market pack
(`market_section`); `needs:` would be named a failure, and `configure/needs.yaml`, `configure/sleep.yaml` are
refused unless their pack is allow-listed. **So enabling the packs in Market Town requires adding `sleep` and
`needs` to `GENERIC_PACKS`, and letting check 3 accept sections owned by an allow-listed generic pack** (Q-Nd-1).

### 1.3 Baselines

`scripts/baselines.txt` has one section per world with reason lines; `scripts/ci_parity.py baseline write <long
record>` rewrites it from a long record. Market Town's keys will all move (genesis gains `needs-assigned` facts and
behaviour changes); Social Café, Lakeside and Bodies Yard must not move (AC-S35-2 (a)).

---

## 2. Supply and prices (Q-S35-8, ruled: in N-d's content)

### 2.1 Demand, from the step's defaults

```text
hunger per day  = 16 h awake × 120 + 8 h asleep × 30          = 2 160 per person
town            = 12 people × 2 160                            = 25 920 per day
```

The step's nourishment values (§5.5): soup 600, sandwich 650, bread 500, cake 400, croissant 350, apple 200,
juice 100, milk 150, coffee 0, tea 0. A day's hunger is therefore about four food items, roughly breakfast, lunch,
dinner and a snack.

### 2.2 Supply today, and the new `produces`

| | croissant | cake | sandwich | soup | apple | bread | milk | juice | nourishment per day |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| today | 3 | 2 | 2 | 2 | 4 | 2 | 2 | 2 | 6 650 (26 % of demand) |
| **N-d** | **8** | **6** | **10** | **10** | **24** | **18** | **6** | **6** | **33 000 (127 % of demand)** |

`33 000 = 8×350 + 6×400 + 10×650 + 10×600 + 24×200 + 18×500 + 6×150 + 6×100`. The 27 % surplus covers a shift cut
short and uneven buying. Coffee and tea production stays as it is (they nourish nothing). Genesis shelf holdings
stay as they are: the first morning's shift restocks before the first rush.

### 2.3 Prices, so an endowment lasts 300 days

A jobless resident starts with 200 000. To last 300 days on food alone they may spend at most 666 a day; N-d aims
at 600 a day (a 10 % margin for drinks and the odd duplicate buy), i.e. **at most 0.28 minor units per point of
nourishment** (600 ÷ 2 160).

| Item | nourishment | old price | **N-d price** | per point |
| --- | --- | --- | --- | --- |
| soup | 600 | 500 | **160** | 0.27 |
| sandwich | 650 | 600 | **175** | 0.27 |
| bread | 500 | 300 | **130** | 0.26 |
| cake | 400 | 400 | **110** | 0.28 |
| croissant | 350 | 250 | **95** | 0.27 |
| apple | 200 | 100 | **55** | 0.28 |
| milk | 150 | 200 | **45** | — |
| juice | 100 | 250 | **40** | — |
| coffee | 0 | 300 | **80** | — |
| tea | 0 | 250 | **70** | — |

A diet entirely of the dearest food per point (0.28) costs 605 a day, 181 500 over 300 days, under 200 000.
Employees are covered by wages: Alice earns about 1 020 a day (8.5 h × 120), Felix about 1 000 (5 h × 200).
**L-13** (no income without a job) stays a recorded gap; this sizing keeps it from starving anyone inside 300 days.
Employers: the café's wage bill is about 1 020 a day against food sales that, at these prices, exceed it once
two or three people eat there daily; the store's 300 000 opening wallet covers Felix's 1 000 a day for 300 days
even with no sales.

**All of these are content in the world's files, chosen by this sizing and fixed before any run.** No code reads a
Market Town number. A different world sizes its own (the arithmetic of §2.1–§2.3 is in the World Pack's README as an
authoring note, so an author can repeat it).

---

## 3. Content changes (Market Town only)

1. `world.yaml`: append `sleep`, `needs` to `systems:` (after `weather`); append `sleep`, `needs` to `configure:`.
2. `configure/sleep.yaml`: the reference IL section (`rules` default forbid `sleep`, permit `place: residential`;
   `parameters` at the step's defaults). `configure/needs.yaml`: parameters at the step's defaults,
   `restore: { hunger: [eating], tiredness: [sleeping] }`.
3. `needs: {}` on every person file (genesis defaults: hunger 300, tiredness 700).
4. `nourishment: { hunger: … }` on the ten food and drink item files (§2.3 values).
5. `produces` in `alice.yaml`, `felix.yaml` (§2.2); prices in the two organization files (§2.3).
6. `apartments` is tagged `residential` (C0 confirms; if not, the tag is added — content).
7. `README.md` of the world: the sizing note.

Social Café, Lakeside and Bodies Yard are not edited (Q-S35-7: removal witnesses; Lakeside is a later content PR).

---

## 4. The second world

`commute-town` (LE revision 2 §0.8) is a test-only World Pack under `tools/cli/tests/worlds/`. This PR gives it the
same kind of content — `sleep`, `needs`, `needs: {}`, nourishment on its foods, its own `produces` and prices sized
by §2's arithmetic for its own population — so that every criterion below runs on two towns with different
geometry, populations, shift times and a manager. Nothing in any crate names either world.

---

## 5. Acceptance (step §9; thresholds fixed by the step and never retuned, I-S35-7)

| Id | Criterion | Worlds | When |
| --- | --- | --- | --- |
| **AC-S35-1** | Determinism: 30 days, seed 7, two runs and a kill-and-resume run give equal fingerprints | Market Town, commute-town; Linux, macOS, Windows | milestone |
| **AC-S35-2** | (a) Social Café, Lakeside, Bodies Yard digests unchanged; (b) Market Town with `sleep`, `needs`, their `configure/` files and every `needs:`/`nourishment:` section removed (a test copy), and the supply/price edits kept, gives the same `run` output as the base with the same supply/price edits | (a) three worlds; (b) Market Town | milestone |
| **AC-S35-5** | Over 30 days `needs` states exactly one `needs-assigned` per person and nothing else; no `needs` Process | Market Town, commute-town | PR (7 days) and milestone (30) |
| **AC-S35-8** | Rhythm, all `rule`, 30 days × seeds {1, 7, 42}, seated people: asleep at 03:00 ≥ 850 ‰; at 14:00 ≤ 100 ‰; mean sleep per person per day in [7.0 h, 9.5 h]; TV distance of food `items-consumed` by hour vs the ATUS *Eating and drinking* row ≤ 350 ‰; nobody at hunger ≥ 900 for more than 24 consecutive hours | Market Town, commute-town | milestone |
| **AC-S35-9** | LE's AC-LE2-1 still 100 % with both packs | Market Town, commute-town | milestone |
| **AC-S35-12** | 300-day saved run keeps AC-SAVE-1's bound (≤ 0.25 MiB/day, days 100–300) | Market Town | milestone |
| **AC-Nd-1** | AC-1 check 3 green with the extended allow list; a `needs:` section in Social Café still refused | — | PR |
| **AC-Nd-2** | Smoke: 3 days, seed 7: every seated person sleeps once a night and eats at least twice a day; no `deduction-unpaid`/`purchase-refused` for lack of stock on days 2–3 | Market Town, commute-town | PR (~5 s) |

A FAIL at the milestone stops for the operator with the numbers; nothing is retuned (I-S35-7).

---

## 6. Test files

- `tools/cli/tests/needs_town.rs` — AC-Nd-2 and AC-S35-5 (7 days) by default; AC-S35-1, -2, -5 (30 days), -8, -9
  `#[ignore]` for the milestone, each printing PASS/FAIL with its numbers before asserting (the `rhythm.rs`
  pattern), reading facts from saves and the ATUS row from `cognition/motivated/data/atus-a3-2025.tsv`.
- AC-S35-12 joins the existing save-retention long test as a Market Town case (`#[ignore]`).
- The test-copy helper of N-c is replaced by the real content for Market Town; its removal-copy form stays for
  AC-S35-2 (b).

---

## 7. Commit plan

Each commit tracks Implementation, Validation and Review separately.

### C0 — Base audit (Markdown only)
- [ ] Record the base; confirm N-a/N-b section names, `residential` on `apartments`, the AC-1 check's current text,
      and that commute-town exists (LE-a). Re-run §2.1's arithmetic against N-b's merged defaults.
- [ ] Implementation · [ ] Validation · [ ] Review

### C1 — AC-1 allow list (after Q-Nd-1)
Scope: `tests/acceptance/tests/ac1_composability.rs` (`GENERIC_PACKS` + `sleep`, `needs`; check 3 accepts sections
owned by an allow-listed generic pack), its unit tests; `ARC-35` note (a dated note in `ARC-35`'s file under
`docs/decisions/`, no new id; `scripts/check_decision_ids.py`). Validation: `cargo test -p mineworld-acceptance --test ac1_composability` (fast parts).
- [ ] Implementation · [ ] Validation · [ ] Review

### C2 — Market Town content
Scope: §3, items 1–7. Validation: `mineworld validate worlds/market-town`; AC-Nd-1; a 3-day run.
- [ ] Implementation · [ ] Validation · [ ] Review (every number equals §2's tables)

### C3 — commute-town content
Scope: §4. Validation: `mineworld validate`; a 3-day run.
- [ ] Implementation · [ ] Validation · [ ] Review

### C4 — Tests
Scope: §6. Validation: the default tests; `--ignored` tests compile and one 7-day ignored case runs locally under
`with-slot.sh`.
- [ ] Implementation · [ ] Validation · [ ] Review

### C5 — Baselines and docs
Scope: `scripts/baselines.txt` Market Town section re-recorded from a run with one reason line ("step-35 N-d: Market
Town enables sleep and needs; production and prices sized per pr-s35-nd §2; every key moved"); the other three
sections untouched; `worlds/market-town/README.md`; step-35 §15's `overall.md` edit is the coordinator's.
- [ ] Implementation · [ ] Validation · [ ] Review

**Size.** ~0.3 kLOC YAML, ~0.5 kLOC tests, ~0.1 kLOC docs. **M.** Opus for C0–C2 (sizing), Haiku for C3–C5 from this
plan.

---

## 8. Questions

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-Nd-1** [OPERATOR] | Market Town can only enable `sleep` and `needs` if AC-1's check 3 allow-lists them as generic packs and accepts the sections they own (`needs:` on people, `nourishment:` on items), as was done for `calendar` and `weather` (`ARC-35` note, 2026-10-09). Extend it? | **Yes**, by a dated `ARC-35` note. The check still measures the market delta exactly; the two packs are generic, not market packs |

The concrete numbers of §2 are content decisions within ruling Q-S35-8 and are not asked.

---

## 9. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, step-35 PR N-d
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s35-nd-content.md
WORKTREE / BRANCH:       /Users/yuema137/mineworld-worktrees/impl-s35-nd   mvp0/pr-s35-nd   (one session)
START CONDITION:         this design FROZEN (Q-Nd-1 ruled); N-c merged
APPROVED SCOPE:          §§3, 4, 6; C1 per Q-Nd-1
FROZEN:                  §2's numbers; step §9's thresholds
ALLOWED COMMANDS:        cargo * (per crate; heavy jobs via cast-slots/with-slot.sh); git; gh (pr create/update);
                         python3 scripts/*; target/*/mineworld *
NEVER:                   python3 -c; sed -i; perl -pi; curl/wget; any code change outside tests/acceptance and
                         tools/cli/tests; editing Social Café, Lakeside or Bodies Yard; a local full-workspace run
MATERIAL STOPS:          a §2 number changed after a run; a digest of the three witness worlds moving
VALIDATION (SPEED FIRST): validate, the smoke tests, `fast`; the long runs at the step milestone
ENDPOINT AUTHORITY:      implementation, commits, push, PR; the lane lead merges after `fast` is green
STOP CONDITION:          PR open, `fast` green, ledger current
```

---

## 10. Ledger

- 2026-10-18: design written from `origin/main @ a1e05874`; arithmetic from the files listed in §1.1.

### C0 — base audit (implementation session, 2026-10-19)

- **Implementation base: stacked on `mvp0/pr-s35-nc @ 036f7fbb`** (PR #307, which carries N-a #297, N-b #305 and
  LE-a/b/c #296/#302/#303; none merged to main yet). **Bounded deviation D-Nd-1:** the start condition "N-c
  merged" is replaced, by the lane lead's decision under SPEED FIRST, with "stacked on N-c"; the lane lead rebases
  after the stack lands. No scope change.
- Names confirmed on the base: `sleep` and `needs` packs under `systems/`; the person section is `needs:` (`{}`
  for the genesis levels 300 / 700, `systems/needs/src/section.rs`); the item section is
  `needs: { nourishment: { hunger: N } }` (ARC-285, Q-Nb-1 (a)), so §3 item 4's "`nourishment:` on the item
  files" is written as that one-line `needs:` section. `configure/sleep.yaml` takes `parameters`
  (`longest_minutes` 840, `shortest_minutes` 15, `woken_by_talk` true) and `rules`; `configure/needs.yaml` takes
  `parameters` (`hunger_per_hour` 120, `hunger_asleep_per_hour` 30, `tiredness_per_hour` 60, `rest_per_hour` 125,
  `thresholds` [400, 600, 900], `restore`) — `docs/reference/interaction-list.md`.
- `places/apartments.yaml` carries `residential`: no tag added. `tools/cli/tests/worlds/commute-town` exists
  (LE-a), with three `residential` homes, one food kind (`bread`) and no shop and no `consumption`.
- AC-1 check 3's text on the base: `GENERIC_PACKS = ["calendar", "weather"]`; added person/place keys and
  item/organization keys must satisfy `market_section`; `configure/<key>.yaml` only for allow-listed packs.
- §2.1 re-run against N-b's merged defaults: `hunger_per_hour` 120, `hunger_asleep_per_hour` 30 →
  16 × 120 + 8 × 30 = 2 160 per person per day; 12 × 2 160 = 25 920. §2.2's 33 000 and §2.3's 0.28 per point
  stand. Implementation [x] · Validation N/A (Markdown) · Review [x].
- **F-Nd-1 (bounded, C2):** §3 item 2 describes `configure/sleep.yaml` as "default forbid `sleep`, permit
  `place: residential`". A rule's `place:` names a class, which needs a `configure/classes.yaml` (a file AC-1's
  allow list does not admit). The pack's own rule already says exactly this when the section has no rule
  (`systems/sleep/src/place.rs`: unsaid default → places tagged `residential`), so the file states the
  parameters at their defaults and no rule; the comment in the file says where the residential rule lives.

### C1–C4 — progress (2026-10-19)

| C | Commit | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C1 AC-1 allow list | `ebb41f0a` | [x] `GENERIC_PACKS` + `sleep`, `needs`; `admitted_section`; `ARC-35` note | [x] `cargo test -p mineworld-acceptance --test ac1_composability` PASS 16/16 (incl. the new `a_needs_section_in_social_cafe_is_still_refused`); `check_decision_ids.py` PASS | [x] checks 1–2 untouched |
| C2 Market Town | `1a5409ac` | [x] §3 items 1–5, 7 (item 6 not needed) | [x] `mineworld validate` PASS (193 genesis facts); 3-day run: 0 faults, 50 `fell-asleep`, 67 `items-consumed`, no `buy` refused; AC-1 check 3 PASS with the content | [x] every number equals §2's tables |
| C3 Commute Town | `4e39407b` | [x] `consumption`, `sleep`, `needs` appended (unconfigured: the packs' defaults); bread 500; a bakery shop (bread 130, shelf 10); produces 8 + 8 + 6 = 22 loaves (11 000 = 127 % of 4 × 2 160); wallets 1 000 → 200 000 (F-Nd-2) | [x] `mineworld validate` PASS; 3-day run: 0 faults, 17 `fell-asleep`, 31 `items-consumed` | [x] |
| C4 Tests | `9f6e4053` | [x] `needs_town.rs` (AC-Nd-2, AC-S35-5 default; AC-S35-5 30 d, -8, -1, -2 (b) ignored); `fixture/needs.rs` is now the removal copy; `needs_controllers.rs` runs the real worlds (the "copy" is gone; seated = needs ∩ walked, since Otto now has needs) | see below — **FAIL, material stop S-Nd-1** | [x] thresholds as frozen; nothing retuned |

- **F-Nd-2 (bounded, C3):** Commute Town's people started with 1 000 and earn 400–500 a day against ~560 a day of
  bread; by §2.3's own rule (an endowment lasts 300 days) the wallets become 200 000, as Market Town's jobless.
  `lateness.rs`'s strict-boss and boundary cases still pass (deductions are now paid, not `deduction-unpaid`).
- **D-Nd-2 (bounded):** AC-S35-12 is `market_town.rs`'s existing ASRB-2 bound (≤ 0.25 MiB per simulated day over
  days 30–300 of Market Town, now with both packs), not a new case in `save_retention.rs`; AC-S35-9 is
  `punctuality.rs`'s and `needs_controllers.rs`'s existing tests, which now run on the worlds with both packs.
- **F-Nd-3 (bounded):** a refused purchase is not a fact, and an out-of-stock kind is simply not offered, so
  AC-Nd-2's "no purchase refused for lack of stock" is read from `run`'s `requests buy rejected` lines (none) and
  the `deduction-unpaid` facts of days 2–3 (none).

### Validation evidence (local, macOS arm64, each through `cast-slots/with-slot.sh`)

| Command | Result |
| --- | --- |
| `cargo test -p mineworld-acceptance --test ac1_composability` | PASS 16/16 |
| `mineworld validate` Market Town · Commute Town | PASS · PASS |
| `cargo test -p mineworld-cli --test needs_town` (default) | AC-S35-5 (7 days) **PASS** both worlds (one `needs-assigned` per person, no `needs` fact after genesis). AC-Nd-2 **FAIL**: Market Town 9 of 11 seated people and Commute Town 3 of 4 fail "asleep at 03:00 of nights 2–3 and ≥ 2 meals on days 2–3" (e.g. Dev: meals per day [0, 0, 0]; Alice: asleep [true, false]); no `buy` refused, no `deduction-unpaid` |
| `… --test needs_town the_towns_sleep -- --ignored` (AC-S35-8, 30 d × seeds 1, 7, 42) | **FAIL** (table below) |
| `cargo test -p mineworld-cli --test needs_controllers` (AC-Nc-4, default) | **FAIL**: Market Town Dev (#10) ate 0 times in 3 days |
| `cargo test -p mineworld-cli --test punctuality` (default, 7 d, seed 1) | **FAIL**: Market Town 12 of 14 parts on time — Alice +149 min, Felix +43 min (AC-S35-9 at risk) |
| `cargo test -p mineworld-cli --test lateness` (default) | **FAIL** `on_the_default_policy_nobody_is_late` (Market Town complaints); the strict-boss and boundary cases PASS |
| `cargo test -p mineworld-cli --test market_town` (300 d; 30 d × 2, kill after day 15 and resume, seed 8) | **PASS** (183 s): the market lives 300 days with both packs, ASRB-1/-2 growth bound holds (AC-S35-12), same seed = same history incl. kill-and-resume (AC-S35-1 for Market Town on macOS) |
| `cargo test -p mineworld-cli --test market_composition` | **FAIL** (expected, F-Nd-4): "without consumption" is now refused at load — `configure/needs.yaml` … needs `consumption` — exactly step-35 §7's row; the test's expectation for that case must become "refused, naming consumption" (a tools/cli/tests edit for the milestone batch) |
| `cargo clippy -p mineworld-cli -p mineworld-acceptance --tests -D warnings` · `cargo fmt --check` | PASS · PASS |

- **F-Nd-4 (bounded, for the milestone batch):** `market_composition.rs`'s consumption case expects "runs"; with
  `needs` enabled the loader refuses a world without `consumption` (step §7), so the expectation is stale, not the
  code. Not edited here (it would mask nothing, but SPEED FIRST batches it with the milestone fixes).
- **Baselines (deferred, lane re-records once at the step milestone; C5's re-record not done here):** Market Town:
  every `summary-*`, `facts-300s`, `journal-300s`, `sealed-300s` moves (genesis gains `needs-assigned`,
  `nourishment-assigned`, the two `*-interactions-configured`; behaviour, prices and production change). Social
  Café, Lakeside, Bodies Yard: N-d edits none of their files, so N-d moves none of their digests (only LE's
  pre-registered genesis-row move, F-LE-4). Commute Town is test-only and has no baseline section.

AC-S35-8, all `rule`, seated people, 30 days (thresholds: 03:00 ≥ 850 ‰, 14:00 ≤ 100 ‰, sleep 7.0–9.5 h, TV ≤ 350 ‰,
famished ≤ 24 h):

| World, seed | asleep 03:00 | asleep 14:00 | sleep h/day | TV eating | longest famished | meals/person/day |
| --- | --- | --- | --- | --- | --- | --- |
| Market Town 1 | **318 ‰** | 0 ‰ | 7.08 | 273 ‰ | **709.8 h** | 1.47 |
| Market Town 7 | **342 ‰** | 0 ‰ | 7.18 | 298 ‰ | **525.2 h** | 1.46 |
| Market Town 42 | **224 ‰** | 0 ‰ | 7.03 | 292 ‰ | **710.6 h** | 1.45 |
| Commute Town 1 | **300 ‰** | **600 ‰** | 7.84 | **480 ‰** | **26.3 h** | 2.79 |
| Commute Town 7 | **300 ‰** | **608 ‰** | 7.84 | **487 ‰** | 22.9 h | 2.90 |
| Commute Town 42 | **300 ‰** | **608 ‰** | 7.84 | **513 ‰** | 20.7 h | 2.97 |

Per person (Market Town seed 7): Dev 1 meal in 30 days (famished 525 h); Erin, Grace 19–28 meals (famished
95–115 h); Alice, Bob, Carol, Felix, Vera 54–86 meals. Everyone sleeps 35–76 times in 30 days (fragmented sleep).

### MATERIAL STOP S-Nd-1 (2026-10-19) — AC-S35-8 / AC-S35-9 fail on the frozen defaults; content cannot fix it

Supply is **not** the cause: no `buy` was refused in any run, and the best-fed people eat 2–3 meals a day from the
same shelves (wallets over 30 days were not read; at ≤ 605 a day a 200 000 endowment cannot run out). The two causes are the controller policy and the needs defaults, both frozen (I-S35-7), which is F-Nc-8 made
measurable:

1. **Sleep drifts (03:00 asleep 224–342 ‰).** With `tiredness_per_hour` 60 and `rule`'s `exhausted_from` 900, a
   person who wakes rested is exhausted 15 h later and the "≥ 900 at any hour" bed rule fires at ~18:00–21:00;
   `for` is "minutes to rested", so they wake at ~01:00–04:00 and sleep again in fragments (35–76 sleeps a month).
   In Commute Town early shifts add afternoon naps (14:00 asleep ~600 ‰).
2. **`rule` does not go and buy food (famished up to 710 h).** Step §6.2 has `rule` rely on the 20 % offered band
   for `buy`, which only fires where a shop is. A person whose routine never passes a shop (Market Town's Dev) eats
   once a month; the meal timing (TV) follows routines, not hunger.

Content cannot repair either within AC-1 check 3 (a person's `routine:` must equal Social Café's) and §2's frozen
numbers. **Recommended answer (for the operator, smallest generic change, in a follow-up to N-c, no world named):**
(a) `rule`'s bed band: drop the "≥ `exhausted_from` at any hour" trigger to `exhausted_from` 1 000 (or require the
night window unless ≥ 1 000), and let `for` run to the window's end (06:00) when no punctual part comes first —
a policy-data change in `rule`'s `needs` block; (b) a generic `rule` "fetch food" band: hunger ≥ `eat_from` and no
nourishing kind held → walk to a place where a shop is disclosed, as policy data; (c) re-run AC-S35-8 and -9 with
no change to the step's thresholds or to `needs`' rates. Until ruled, N-d's content and tests stay committed as the
evidence; the four failing default tests above are the stop's evidence, not to be ignored or retuned.

### Re-run after Q-S35-OP1 (2026-10-19) — stop S-Nd-1 answered by PR N-c2

Ruling Q-S35-OP1 (quoted in `step-35-needs-and-sleep.md` §0.1's ledger and `ARC-286`'s amendment) was implemented
as PR N-c2 (`mvp0/pr-s35-nc2`, head `23eed980`) and merged here. N-c2's own re-run findings D-Nc2-4 (a bed spot
clear of the doors) and D-Nc2-5 (wake two paces ahead; no errand while waiting for a punctual part) are in
`pr-s35-nc-controllers.md` §N-c2. N-d content changed by one tag: Commute Town's bakery is `[bakery, shop]` (F-Nc2-2).
No threshold, rate or baseline was changed or re-recorded.

| Commit | Implementation | Validation | Review |
| --- | --- | --- | --- |
| merges of N-c2 | [x] `8b48a5af` and later merges; conflicts in `fixture/needs.rs`, `needs_controllers.rs` kept N-d's (real worlds) | [x] below | [x] |
| bakery tag | [x] `tools/cli/tests/worlds/commute-town/places/bakery.yaml` | [x] below | [x] content only |
| F-Nd-4 | [x] `market_composition.rs`: "without consumption" now expects a refusal naming `consumption` | [x] `--test market_composition` PASS (81 s) | [x] the step-§7 row, not a code change |

AC-S35-8, all `rule`, seated people, 30 days (thresholds unchanged: 03:00 ≥ 850 ‰, 14:00 ≤ 100 ‰, sleep 7.0–9.5 h,
TV ≤ 350 ‰, famished ≤ 24 h). Before = the S-Nd-1 table above; after = N-c2 head `23eed980`:

| World, seed | asleep 03:00 before → after | 14:00 | sleep h/day | TV eating | longest famished | meals/person/day |
| --- | --- | --- | --- | --- | --- | --- |
| Market Town 1 | 318 → **893 ‰** | 0 → 9 ‰ | 7.08 → 7.15 | 273 → 305 ‰ | 709.8 → **37.6 h** | 1.47 → 6.52 |
| Market Town 7 | 342 → **793 ‰** | 0 → 0 ‰ | 7.18 → **6.12** | 298 → 280 ‰ | 525.2 → **45.5 h** | 1.46 → 6.39 |
| Market Town 42 | 224 → **903 ‰** | 0 → 3 ‰ | 7.03 → 7.24 | 292 → 278 ‰ | 710.6 → **21.3 h** | 1.45 → 6.55 |
| Commute Town 1 | 300 → **1 000 ‰** | 600 → **0 ‰** | 7.84 → 7.31 | 480 → **244 ‰** | 26.3 → **1.3 h** | 2.79 → 5.13 |
| Commute Town 7 | 300 → **1 000 ‰** | 608 → **0 ‰** | 7.84 → 7.35 | 487 → **266 ‰** | 22.9 → 1.7 h | 2.90 → 5.19 |
| Commute Town 42 | 300 → **1 000 ‰** | 608 → **0 ‰** | 7.84 → 7.36 | 513 → **253 ‰** | 20.7 → 1.0 h | 2.97 → 5.24 |

Verdict: Commute Town **PASS** on all three seeds; Market Town seed 42 **PASS**; seed 1 **FAIL** (famished 37.6 h);
seed 7 **FAIL** (03:00 793 ‰, 6.12 h, famished 45.5 h). Not retuned.

AC-S35-9 (AC-LE-1 with both packs: every shift present, nobody late), 30 days, `mineworld run` counts:

| World | seed 1 | seed 7 | seed 42 |
| --- | --- | --- | --- |
| Market Town (60 shifts) | 60 present, 0 late | **56 present, 4 late** | **1 late** (present not read) |
| Commute Town (120 shifts) | 0 late | 0 late | 0 late |

Before: only the 7-day smoke was measured (Market Town 12 of 14 parts on time). After: the 7-day smoke is 14/14
(Market Town) and 35/35 (Commute Town) on time, but AC-LE-3 counts arrivals more than 60 min early — Commute Town
2 (Ben, Cy at the bakery), Market Town 30 d seed 1: 3 (Felix at the store) — a person who fetches food at the shop
that is also their workplace and is then held there by the commute policy (`holds`).

| Test (default unless noted) | Before | After |
| --- | --- | --- |
| AC-Nd-2 smoke (`needs_town` people_sleep_each_night…) | FAIL (9/11, 3/4) | **PASS** |
| AC-Nc-4 (`needs_controllers` rule_people_sleep_at_night…) | FAIL (Dev ate 0) | **PASS** |
| `lateness` on_the_default_policy_nobody_is_late | FAIL | **PASS** (3/3) |
| `punctuality` 7 d | FAIL (12/14) | **FAIL** — all on time, AC-LE-3: 2 early in Commute Town |
| `punctuality` 30 d × 3 (ignored) | not run | **FAIL** at Market Town seed 1, AC-LE-3 (3 early) |
| AC-Nc-3 30 d × 3 (ignored) | not run | **FAIL** at Market Town seed 7 (56/60, 4 late) |
| `market_composition` | FAIL (F-Nd-4) | **PASS** |

**Remaining causes (diagnosed, not fixed; for the milestone batch):**
- **F-Nd-5:** Market Town seed 7, day 15: Alice, out at the store at 04:09 on a night food errand, gets `no-route`
  from the store every consult until 07:45 (late 3 days running; the store's doorway is blocked by bodies). The
  same crowding shows as `stalled` walks and `woke { acted }` (F-Nc-7): a `bodies`/movement question about people
  standing in a one-door place, not a controller threshold.
- **F-Nd-6:** Market Town's famished stretches > 24 h (37.6 h, 45.5 h) and seed 7's 6.1 h sleep follow from the same
  blocked walks (a person stuck out of reach of the store, or kept from home at night); Commute Town, with a wide
  street and no crowding, passes every row.
- **F-Nd-7:** AC-LE-3's "no arrival more than 60 min early" counts a fetch errand to a shop that is also the next
  shift's place. Either the measure reads only commute arrivals, or `fetch` avoids a shop that is the next punctual
  part's place within the commute cap — an operator choice (recommended: the former; arriving early to buy breakfast
  at one's own bakery is not a lateness defect).

### Operator rulings, 2026-10-19 (after the re-run)

- **F-Nd-7 ruled: AC-LE-3 counts only commute arrivals.** `punctuality.rs` now treats an early arrival as an
  errand, and does not judge it, when the person received items (`items-transferred` to them) between that arrival
  and the part's `from`. This is a generic test, with no place or world named. Errand arrivals are reported in
  `Measured.errands`.
- **D-Nc2-4 and D-Nc2-5 kept** (see `step-35-needs-and-sleep.md` §0.1's ledger).
- **Re-run, 7 days, after the cascade merge of origin/main (630c1c3a).** Market Town seed 1: 14/14 on time and
  14/14 shifts present. One errand arrival was set aside. One commute arrival is still early: person #12 at
  place #4, 74 min before `from` (t197135 against t201600). AC-LE-3 therefore stays FAIL.
  - This is finding **F-Nd-8**, for the milestone: the cause of the early commute is not yet identified.
  - Commute Town is not reached in this test, because the assertion stops at Market Town first.
