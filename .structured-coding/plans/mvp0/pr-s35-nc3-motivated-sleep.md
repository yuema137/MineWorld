# PR N-c3 — `motivated` sleeps by the two-process model and goes for food (F-Nc2-1)

## DESIGN FROZEN — 2026-10-19 (operator approved in the primary session: Q-Nc3-1 … Q-Nc3-4 yes as recommended; Q-Nc3-5 not pre-approved — re-ask with the fit's numbers)

Branch (once frozen): `mvp0/pr-s35-nc3`, worktree `/Users/yuema137/mineworld-worktrees/impl-s35-nc3`, from `main`.
Step: [`step-35-needs-and-sleep.md`](step-35-needs-and-sleep.md) §6.3. Controllers PR: [`pr-s35-nc-controllers.md`](pr-s35-nc-controllers.md)
(SD-Nc-7; ledger §N-c2, finding F-Nc2-1). Decision: [`ARC-286`](../../../docs/decisions/ARC-286.md), "Amendment
2026-10-19 (Q-S35-OP1)", paragraph "`motivated` (finding, not changed)".

---

## 1. Why this PR exists

`rule` had three defects in its sleep and food policy (F-Nc-8): it went to bed at any hour once tiredness reached
900, it slept only until "rested" and so woke at 01:00 – 04:00, and it never went for food. The operator ruled
Q-S35-OP1 for `rule`, and N-c2 fixed it with policy data (`exhausted_from`, `sleep_until: night-end`, a fetch band).

`motivated` has the same three defects, for the same causes:

1. Its `Sleep` option carries `rested` (the ATUS hour curve) and `tired` (the need). Both are multiplied, so a person
   with tiredness 900 at 19:00 still scores `Sleep` high enough to win: the hour lowers the score but does not stop
   it.
2. Its `for` is `rule`'s `needs::sleep_minutes` with no `until`, so it is "minutes to rested". A sleep begun at
   19:00 at tiredness 950 ends at 02:36.
3. A door to a `shop` place carries no need. Only a door to an `eat` place and an offered `buy` carry `hungry`. A
   person whose day passes no shop and no café never goes for food.

**Does Q-S35-OP1 cover `motivated`?** No. The ruling names `rule` ("Add the small generic `rule` follow-up"), and
`ARC-286`'s amendment records that `motivated` needs "a new `motivated` parameter and a scored-option question, not
the same small change". The operator rule of 2026-10-19 also binds here: adopt an established model before tuning
thresholds. So this PR is a new design and needs a freeze.

## 2. The established model: two-process sleep regulation

Borbély's two-process model is the standard account of when people fall asleep and wake up:

- A. A. Borbély, "A two process model of sleep regulation", *Human Neurobiology* 1 (1982) 195–204.
- S. Daan, D. G. M. Beersma, A. A. Borbély, "Timing of human sleep: recovery process gated by a circadian
  pacemaker", *American Journal of Physiology* 246 (1984) R161–R183 (the threshold form used here).
- A. A. Borbély, S. Daan, A. Wirz-Justice, T. Deboer, "The two-process model of sleep regulation: a reappraisal",
  *Journal of Sleep Research* 25 (2016) 131–143 (confirms the form is still current).

The model has two parts:

- **Process S** is sleep pressure. It rises while awake and falls while asleep.
- **Process C** is the circadian rhythm. It moves two thresholds up and down over the day: an upper threshold
  `H(t)` and a lower threshold `L(t)`, both shifted by the same circadian wave, `H(t) = H0 + a·C(t)` and
  `L(t) = L0 + a·C(t)`.
- A person falls asleep when S reaches `H(t)`, and wakes when S falls to `L(t)`.

A concrete example. At 19:00, C is near its peak, so `H` is high: a tired person (S = 800) stays awake. At 23:00, C
has fallen and `H` with it: the same person, now at S = 960, falls asleep. At 03:00, C is near its trough, so `L` is
low: S = 450 is still above `L`, and the person sleeps on. At 06:30, C rises and `L` with it: S, now near 40, meets
`L`, and the person wakes.

This gives exactly the three behaviours `motivated` lacks: no evening bedtime unless exhausted, no wake at 03:00,
and an early bedtime after an early rise (S reaches `H` sooner).

### 2.1 How the model maps onto MineWorld

| Model | MineWorld | Owner |
| --- | --- | --- |
| Process S | the disclosed `tiredness` (0 … 1 000) | the `needs` System (unchanged) |
| rise and fall rates of S | `tiredness_per_hour` 60, `rest_per_hour` 125 | `needs` parameters (unchanged, I-S35-7) |
| Process C with `H0`, `L0`, `a` | two curves over the minute of the day, `sleep_from` (H) and `wake_at` (L) | `motivated` parameters (new) |
| fall asleep when S ≥ H(t) | the `Sleep` option's new consideration `pressure` | `motivated` scoring |
| wake when S ≤ L(t) | `for` = the minutes until S meets `L` | `motivated` option |

One stated difference. In Daan et al. S rises and falls exponentially; in MineWorld `needs` changes tiredness
linearly, and the step froze those rates (I-S35-7). This PR adopts the model's threshold structure (two thresholds
moved by one circadian wave) and keeps `needs`' S as it is. The default threshold curves are fitted once to the
linear S, against the committed ATUS *Sleeping* row (§4.4). They are model data, fitted to population data, not to a
world.

`rule` is not changed. Its night window stays the policy the operator ruled.

## 3. Scope

### 3.1 In scope

- `cognition/motivated`: two new hour-of-day threshold curves, one new consideration, a new sleep length, `hungry`
  on doors to shops, the parameters to override them, and their tests.
- `tools/cli/tests/needs_controllers.rs`: a two-world test for `motivated` seats, the twin of N-c2's
  `rule_people_sleep_through_the_night_and_go_for_food_in_two_worlds`.
- `ARC-286`: an "Amendment (Q-Nc3)" paragraph. Step §6.3 and the N-c ledger: a note each.

### 3.2 Not in scope

- `needs` rates, thresholds and words; `rule`; the kernel and contracts; LM seats.
- Per-person chronotype (Q-S35-11 keeps it for S30's follow-up). The curves are classed parameters, so a
  `night-shift` class can already move its own thresholds.
- AC-DLb-4 (the social-café rhythm criterion). It is re-measured and reported, not gated (§5).

### 3.3 Invariants

- **I-Nc3-1** Without `Needs` disclosed, every consult decides exactly as before: no new consideration is added and
  no draw is taken (I-Nc-1; AC-S35-2).
- **I-Nc3-2** No world is named. Every input is a disclosure: my `needs` record (levels, rates, `nourishment`),
  my holdings, the offers, the hour, the doorways and their `to_tags`.
- **I-Nc3-3** No new draw. Scoring stays the existing compensation and top-band draw.
- **I-Nc3-4** A due commute and the alarm keep precedence over sleep, as today (SD-Nc-4, D-Nc2-5).

## 4. Design

### 4.1 SD-Nc3-1 — the threshold curves

Two new curves in `Settings.curves`, input the **minute of the day** (0 … 1 439), output a tiredness level
(0 … 1 000):

- `sleep_from` is `H(t)`: the tiredness at which sleep appeals at that time of day.
- `wake_at` is `L(t)`: the tiredness at which a sleeper wakes at that time of day.

Both are built from one circadian wave `C(t)` (a cosine sampled at 24 points, peak `c_peak`) by the model's own
form: `H(t) = min(1 000, H0 + a·C(t))`, `L(t) = max(0, L0 + a·C(t))`. The cap at 1 000 means a fully exhausted
person can always sleep, which replaces any "exhausted at any hour" override.

Default values (fitted at C1 by `curves_fit_atus_sleeping` in §4.4, then committed as numbers):

| Parameter | Default (to be confirmed by the fit) | Source |
| --- | --- | --- |
| `c_peak` | 18:30 | the evening "wake maintenance zone" (Strogatz, Kronauer, Czeisler, *Am. J. Physiol.* 253 (1987) R172–R178); C's trough 12 h later at 06:30 |
| `a` | 110 | Daan et al.'s ratio `a / (H0 − L0)` = 0.12 / 0.50 = 0.24, applied to MineWorld's gap |
| `H0` | 895 (so `H` runs 785 … 1 000 after the cap; `H(22:30)` ≈ 950) | fitted: the night's onset at 22:00 – 23:00 for a 06:30 riser at 60 per hour |
| `L0` | 150 (so `L` runs 40 … 260) | fitted: the wake at 06:00 – 07:00 after a 22:30 onset at 125 per hour |

The three fitted values are the only ones the fit chooses; the shape (one wave, two thresholds, a fixed gap) is the
model's. The fit runs once, against ATUS, for `needs`' default rates; it is not run per world. A world that changes
`needs`' rates may override `sleep_from` and `wake_at` like any curve.

### 4.2 SD-Nc3-2 — `pressure`: when sleep wins

New consideration `pressure`, input the margin `m = tiredness − H(now)`, with a response curve (overridable):
`m ≤ −300 → 0`, `−100 → 300`, `0 → 1 000`. With `Needs` disclosed, `Sleep` carries `pressure` **in place of**
`rested` and `tired`, since the model's `H(t)` already holds both the hour and the need, and multiplying the old two
again would count each twice. Without `Needs`, `Sleep` is not offered (it needs the `sleep` pack), so nothing changes.

A door to a `home` place carries `pressure` on `tiredness + homeward` (`homeward`, default 60, one hour's rise at the
default rate), in place of `tired`. So a person heads home about an hour before sleep would win, instead of leaving
whenever tired.

### 4.3 SD-Nc3-3 — `for`: when the sleeper wakes

`motivated` stops calling `rule`'s `needs::sleep_minutes`. Its own `sleep_for` steps forward from now in 15-minute
steps, lowering S by `rest_per_hour / 4` each step (the disclosed rate), and stops at the first step where
`S ≤ wake_at(t)`. The result is bounded, as today, by the world's longest sleep and by the commute's leave-by
(SD-Nc-4). All integer, no draw.

Example: at 22:30, S = 960. S falls to 40 by 05:52; `wake_at(05:45)` ≈ 45 and `wake_at(06:00)` ≈ 42, so the sleeper
wakes at 06:00 with `for` = 450 minutes.

### 4.4 SD-Nc3-4 — the fit and its test

`curves_fit_atus_sleeping` (a unit test in `cognition/motivated`) simulates one person with no commute at
`needs`' default rates for 14 days under the curves, from a midnight start at the genesis levels, and asserts the
asleep share at each hour against the committed ATUS *Sleeping* row: ≥ 850 ‰ at 03:00, ≤ 100 ‰ at 14:00, mean sleep
in [7.0 h, 9.5 h] (AC-S35-8's bounds). The defaults of §4.1 are the values that pass it; they are committed as
literal numbers with this test as their record.

### 4.5 SD-Nc3-5 — going for food

A door to a place with the `shop` role carries `hungry` when nothing nourishing is held (the `needs` record's
`nourishment` list, the same disclosure `rule`'s fetch band reads). An offered `buy` already carries `hungry`
(SD-Nc-7). Roles are matched by tags as today (`roles.shop`), so a world whose shops carry another tag names it in
`roles`. There is no fallback "walk out of a one-door place": `motivated` already scores every door, and `hungry`
on the shop door is what was missing.

### 4.6 Parameters

`MotivatedParameters` gains `sleep_from`, `wake_at` (minute-of-day curves) and `pressure` in `OVERRIDABLE`, a new
input class "minute of the day" (0 … 1 439) for the first two, and `homeward` (0 … 600) in the `needs` group.
Unknown names stay rejected.

## 5. Acceptance (fixed before any run)

Each criterion runs on **Market Town (test copy, every seat `motivated`) and `commute-town` (LE's generated
variant)**, with no code or parameter change between them.

| ID | Criterion | Mutation that must fail it |
| --- | --- | --- |
| AC-Nc3-1 | AC-S35-8's sleep bounds for `motivated` seats over 30 days × seeds {1, 7, 42}: asleep at 03:00 ≥ 850 ‰; at 14:00 ≤ 100 ‰; mean sleep in [7.0 h, 9.5 h] | `sleep_from` flat at 600 (no circadian gate) |
| AC-Nc3-2 | Fragmentation: at most 2 sleep episodes per person per night (18:00 – 12:00) in 95 % of person-nights | `for` back to "minutes to rested" |
| AC-Nc3-3 | Food: no seated person at hunger ≥ 900 for more than 24 consecutive hours (AC-S35-8's last clause) | `hungry` removed from shop doors |
| AC-Nc3-4 | Punctuality: AC-S35-9 (180 of 180 shifts present) with `motivated` seats | alarm ignored |
| AC-Nc3-5 | Removal: without the packs, all-`motivated` `run` output unchanged (AC-Nc-1) | `pressure` added when `Needs` is absent |
| — | `curves_fit_atus_sleeping` (§4.4) | — |

Per SPEED FIRST, C1 – C3 run the tests they write and `fast`; AC-Nc3-1 … -4 and the mutations run at the milestone.
AC-DLb-4 is re-measured then and reported, not gated.

## 6. Commit plan

- **C1** — curves, `pressure`, parameters, the fit test (`cognition/motivated/src/{settings,curves,parameters,considerations}.rs`).
- **C2** — `sleep_for` by `wake_at`; `hungry` on shop doors (`options.rs`, `considerations.rs`, `needs_tests.rs`).
- **C3** — two-world test in `tools/cli/tests/needs_controllers.rs`; `ARC-286` amendment; step §6.3 note; ledger.

Size: about 350 lines of Rust and tests. Implementer: Opus (the fit needs judgement), one session, one worktree.

## 7. Risks

| Risk | Handling |
| --- | --- |
| The fit cannot pass with one wave on a linear S | Report with the fit's numbers as a material stop; do not add ad-hoc thresholds |
| Early risers (alarm at 04:30, S ≈ 250 left) reach `H` near 17:15 and, with the draft numbers, meet `L` near 23:45, then stay awake: S is linear here, while the model's exponential S falls fast from a high level | `curves_fit_atus_sleeping` adds an early-riser case at C1; AC-Nc3-2 measures it. If one wave cannot pass both cases, a material stop with the fit's numbers (no ad-hoc band) |
| `rest` stops shaping `Sleep` and AC-DLb-4 moves | Reported; AC-DLb-4 is already FAIL and outside this PR |

## 8. Questions for the operator (each with a recommended answer)

| ID | Question | Recommended |
| --- | --- | --- |
| **Q-Nc3-1** | Fix `motivated`'s night by the two-process model (Borbély 1982; Daan et al. 1984) as data — two threshold curves from one circadian wave — rather than copying `rule`'s night window? | **Yes** |
| **Q-Nc3-2** | Fit `H0`, `L0` and the wave's peak once against the committed ATUS *Sleeping* row at `needs`' default rates (one test, committed numbers), not per world? | **Yes** |
| **Q-Nc3-3** | With `Needs` disclosed, `Sleep` and doors home score by `pressure` alone, replacing `rested` × `tired` (which would count the hour and the need twice)? | **Yes** |
| **Q-Nc3-4** | `motivated` goes for food by `hungry` on doors to `shop`-role places when nothing nourishing is held (no extra fallback band)? | **Yes** |
| **Q-Nc3-5** (contingency) | If the C1 fit shows the early-riser case cannot pass on `needs`' linear S, may a follow-up give `needs` the model's exponential recovery (fall per hour proportional to the level, mean kept at 125 per hour over a night) — a change to frozen I-S35-7? | **Ask again with the fit's numbers**; do not pre-approve |

---

## 9. Ledger

- 2026-10-19 — drafted by the step-35 follow-up lane from F-Nc2-1 and the operator rule "adopt established models
  first" (relayed by the coordinator, 2026-10-19).
- 2026-10-20 — execution (impl-s35-nc3). **SD-Nc3-5 implemented** (`hungry` on a door to a `shop`-role place while
  the `needs` record lists no nourishing kind held; unit test `a_hungry_person_holding_no_food_is_drawn_to_a_shop_door`).
  **SD-Nc3-1 … -4 stopped at the C1 fit (§7 risk confirmed; Q-Nc3-5 to the operator).** The fit simulated one
  person at `needs`' default rates for 60 days (last 28 measured) with the curves sampled exactly as the Rust
  `two_process` module builds them, three cases: free-running, alarm 06:30, alarm 04:30; pass = asleep at 03:00
  ≥ 850 ‰, at 14:00 ≤ 100 ‰, mean sleep 7.0 – 9.5 h, ≤ 1.1 sleeps a day. With the design's `a` = 110 the best
  set is `H0` 990, `L0` 10, peak 15:00 (free-running 22:15 – 06:15, 06:30 alarm 22:15 – 06:00, 04:30 alarm
  20:45 – 04:30), but it is a knife edge: every neighbour one grid step away (±10 in `H0` or `L0`, ±1 h in the
  peak) passes the first two cases and fails the early riser, who drifts to a day sleep (e.g. 13:45 – 21:30) or
  splits his night. Freeing `a` (outside Daan et al.'s ratio 0.24) gives `H0` 980, `L0` 260, `a` 300, peak 16:00,
  with 4 of 7 neighbours passing; the failing ones put the early riser to bed at 17:30 and wake him at 01:15 — the
  §7 failure. The work so far (curves, `pressure`, `sleep_for` by `wake_at`, parameters, the Rust fit test and the
  scratch fit scripts) is on branch `wip/s35-nc3-sleep` @ `953cfe23`, not for merge.
