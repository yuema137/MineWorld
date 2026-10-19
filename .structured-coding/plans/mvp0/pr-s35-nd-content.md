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
