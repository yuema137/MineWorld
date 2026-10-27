# PR S35 N-g — walkers do not jam each other (F-Ng-1 … F-Ng-4)

```text
STATUS        DESIGN FROZEN 2026-10-20 (operator approved in the primary session: every question answered as recommended); freeze confirmed by the operator 2026-10-26 (all questions as recommended). Implemented in #453.
step          35 (needs and sleep), step-35-needs-and-sleep.md; milestone-batch fix after N-f
branch        mvp0/pr-s35-ng (implementation, after freeze); this design on docs/pr-design-s35-ng
base          mvp0/pr-s35-nf @ 35ff1325 (#370, open); rebased on main once #370 merges
owns          bodies (stride resolver: who gives way), movement (what a stall is; a walk's start instant)
touches no    kernel, contracts, controllers, clients, world content
decision id   ARC-3xx, from the coordinator's range (placeholder until assigned)
```

## 1. Why this PR exists

After N-f (#370, `ARC-330`), idle people no longer block doorways. Market Town still ends about 30–38 walks a
week `stalled`, several people at once, and two milestone rows still fail:

- AC-S35-8, "nobody famished for more than 24 h": the longest stretch is 48 h (seed 1), 36 h (seed 7) and 67 h
  (seed 42).
- AC-LE-3, "nobody arrives more than an hour early": seed 1 has two early arrivals (seeds 7 and 42 have one and
  two by the same measure).

The hypothesis handed to this design was that walkers block other walkers. §2 checks it against a trace. It holds
for the stalls. It does **not** explain the hunger or the early arrivals: those have two other causes, which §2.3
and §2.4 show and §8 hands to a separate PR.

## 2. The trace

### 2.1 How it was taken

The binary built from `mvp0/pr-s35-nf` @ `35ff1325` ran Market Town headless for 30 days with seeds 1, 7 and 42
(`mineworld run worlds/market-town --headless --seed S --days 30 --save …`, about 30 s each). Every person's
`mineworld perceived … --json` stream was merged and de-duplicated by event id; together these streams hold every
fact the analysis needs. A throwaway script (not checked in) replayed them:

- positions from `arrived`; who is walking from `walk-started` / `walk-ended`; who is asleep from `fell-asleep` /
  `woke`;
- for each `stopped-short`, the person named in `by`, whether *that* person was walking, idle or asleep at that
  instant, and both positions (no `by` = a solid or the floor's edge);
- hunger exactly as `tools/cli/tests/needs_town.rs` replays it (120 an hour awake, 30 asleep, lowered by each
  nourishing `items-consumed`);
- holdings from `stocked`, `items-transferred` and `items-consumed`; wallets from `funded` and `money-transferred`.

The trace reproduces the reported numbers: the same three famished stretches (48.1 h, 36.4 h, 66.7 h) and the same
early arrivals.

### 2.2 F-Ng-1: walkers stall each other, mostly in the 06:00 rush (confirmed)

| | seed 1 | seed 7 | seed 42 |
| --- | --- | --- | --- |
| walks ended `stalled` in 30 days | 162 | 124 | 157 |
| … whose **last** stop was by a person **walking** at the time | 120 (74 %) | 81 (65 %) | 111 (71 %) |
| … with at least one stop by a walking person | 150 (93 %) | 115 (93 %) | 152 (97 %) |
| … whose last stop was by an idle person | 10 | 9 | 10 |
| … whose last stop was by a solid (no `by`) | 23 | 28 | 34 |
| … ended between 06:00 and 06:59 | 103 (64 %) | 66 (53 %) | 96 (61 %) |
| stops (`stopped-short`) per stalled walk, typical | 5–8 | 5–8 | 5–8 |

**Which rule ends the walk.** Movement's step 1 (`systems/movement/src/walk.rs`, `stepped`): three strides in a row
that gain less than `STALL_PROGRESS` (50 mm) reach `STALLS_MAX` (3), and the walk ends `stalled`. A few walks with
nine or ten stops end by `REPLANS_MAX` (8) instead. The typical stalled walk lasts 7–9 minutes.

**Where and who.** A concrete example, seed 42, day 15:

```text
06:06 Ivan  stalled → street, at apartments (2400, 555): stopped by Hana (walking), Bob (walking)
06:07 Dev   stalled → park,   at apartments (2823, 1190): stopped by Ivan (walking) four times
06:08 Bob   stalled → street, at apartments (1834, 271): stopped by Ivan (walking) four times
06:08 Felix stalled → street, at apartments (2238, 585): stopped by Ivan, Erin, Bob (all walking)
```

The apartments' hall is 8 × 6 m with one door at (2000, 400). At 06:00 most of the town's nine residents start a
walk out of it in the same minute. They all aim at the same point and stop one another within a metre of the door.
None of them is idle, so N-f's give-way (R2) moves nobody: `give_way` in `systems/bodies/src/doorway.rs` returns
`None` as soon as the person in the way is walking. The head-on bias (step-11 SD-B10) does not help either,
because these walkers go the **same** way. Each stopped stride re-plans round the person who stopped it, and the
next stride meets someone else. After three strides with no progress, the walk ends.

The second cluster is the store's doorway on the street side (8390, 2800). People walking in meet people walking
out there, for example `d03 06:44 Hana stalled → store at street (8309, 2349), stopped by Bob (walking) at
(8400, 2850)`.

Bursts follow from this: in each seed, 120–141 of the 328–366 walks that did not end `arrived` fell in a
15-minute window with at least one other such walk.

### 2.3 F-Ng-2: famished people have their hands full (not a walking defect)

| | seed 1 Grace | seed 7 Bob | seed 42 Dev |
| --- | --- | --- | --- |
| famished stretch | d02 10:52 → d04 11:00, 48.1 h | d09 15:19 → d11 03:45, 36.4 h | d26 22:05 → d29 16:45, 66.7 h |
| walks that **arrived** at the store during it | 33 | 34 | 108 |
| walks during it that ended `stalled` | 3 | 3 | 3 |
| held during the whole stretch | 6 items: candle, umbrella, newspaper, tea, scarf, pen | 6: tea ×2, coffee, book, soap, mug | 6: notebook, pen, soap, coffee, book, newspaper |
| money | ample | ample | 185 000 |

Every famished person holds six items for the whole stretch, and none of them nourishes (tea and coffee nourish
0). `PERSON_CAPACITY` is 6 (`systems/inventory/src/admit.rs`). Economy offers `buy` as unavailable when the buyer
cannot carry one more (`purchasable` → `admit_transfer` → `can_take`). So at the store the rule's fetch band
(`cognition/rule-controller/src/needs.rs`, `fetch`) sees no available `buy`. It concludes "here sells: the shop has
nothing for me" and does nothing more. The person leaves on their routine, comes back on the next consult, and
goes round this loop 33–108 times. The stretch ends only when they happen to give an item away. No `buy` is ever
refused, because none is ever asked: the journals show `buy accepted` only.

The goods come from gifts (`give`) and other people's errands. Stalls do not cause this, and N-g's walking rules
cannot fix it.

### 2.4 F-Ng-3: early arrivals are errands to an empty shop (not a walking defect)

All the early arrivals are Felix, the store's clerk, whose punctual part is the store at 08:00:

```text
seed 1, d19: 06:15 walk to the store (hungry, no food) → arrived 06:28 → waits → part begins 08:00 (87 min early)
seed 1, d28: 06:30 walk to the store → arrived 06:46 → part begins 08:00 (72 min early)
```

The store sells out every day (24 of 24 apples sold on each day measured). It is restocked only by Felix's own
shift (`people/felix.yaml`, `job.produces`), at 08:00. So the 06:00 fetch errand finds empty shelves. No item is
received, and under F-Nd-7 an early arrival with no receipt counts as a commute arrival. The stall on the way
(seed 1, d28 06:07) only delays him; it does not make him early.

### 2.5 F-Ng-4: one walker is stopped by a solid at the same point again and again

In each seed, 14–19 stops have no `by` (a solid) and leave the walker at exactly street (7074, 1214), on the way to
the store's door. The planned leg passes between two street solids, (6851…7749, 2180…2720) and
(8597…9303, 1797…2503). The planner grows each solid by 310 mm (`systems/bodies/src/route.rs`, `GROWN`) and finds
the gap passable. The stride stops at the same point anyway. The cause is not yet pinned down. Two candidates: the
planner and the stride disagree on clearance, or the planner's corner nodes lie exactly on a grown edge. These
stops make up most of the "last stop by a solid" row in §2.2.

## 3. Established approaches compared

The constraints are the same as N-f's (§2 there): integer positions in the log, byte-identical replay on three
platforms (`ARC-25`), one walker's request resolved at a time, no shared simulation tick, Rapier only behind
quantisation (`DEP-13`), and the operator's no-interpenetration rule (2026-10-19). N-f already rejected ORCA/RVO2,
`dodgy_2d` and social force, because they need a common velocity tick and floats; this PR does not revisit them.

| Approach | Source / tool | Licence | What it does | Fit here | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Cooperative pathfinding with reservations (WHCA\*)** | Silver, "Cooperative Pathfinding", AIIDE 2005; Open-RMF's `mapf` crate (Rust, continuous-time planners) | algorithm; `mapf` Apache-2.0 | Each agent plans in space–time against a shared table of cells others have reserved for the next *W* steps, so plans never collide | Prevents meetings before they happen, but needs new authoritative state (a time-indexed reservation table every walk-step reads and writes) and a space–time A\* per walker on our 50 mm lattice. Reservations go stale whenever a walker re-plans, is pushed, or a controller replaces its walk. `mapf` is `f64` and continuous-time, so it breaks integer replay. It also does not settle who waits at a one-person door: WHCA\* is known to deadlock there without extra priority rules | **Rejected for now**; revisit if §5's rows still fail with priorities |
| **Priority inheritance with backtracking (PIBT)** | Okumura, Machida, Défago, Tamura, IJCAI 2019; the basis of LaCAM (2023) | algorithm; we adopt the rule, not code | One step at a time. Every agent has a unique priority. When a higher-priority agent wants the cell of a lower-priority one, the lower one must move away, inheriting the priority so it can push others in turn. If it cannot move, it backtracks and the higher one tries its next-best move. The priority is the time since the agent last reached a goal, so whoever has waited longest wins. That rule is what gives the "everyone reaches their goal" guarantee on biconnected graphs | One step, local, deterministic with a total order (time, then id), no shared table. It is the walking generalisation of N-f's R2 (idle people give way to walkers): R2 is PIBT with "idle" as the lowest priority. Our resolver already does one-level pushes, nudges and give-way moves with verification (V1–V4) | **Adopt** as R1–R2 |
| **Rule sets: yield, keep right, queue at the door** | Recast/Detour crowd (zlib); Unity `NavMeshAgent.avoidancePriority`; RTS unit pushing (StarCraft II, Age of Empires II); doorway "traffic light" queues in warehouse robotics | zlib / proprietary / behaviour | Fixed avoidance priorities; a side convention for head-on meetings; one agent through a narrow passage at a time while the others wait at a queue point | We already have the head-on bias (SD-B10, effectively a keep-right rule). Fixed per-agent priorities can starve: the same person always waits. A queue at each door needs queue points derived per door, which is more code than PIBT's local rule and covers doors only, not the street-side cluster. A related game convention, letting walkers pass through each other after a few blocked steps (RimWorld, The Sims), is ruled out by the no-interpenetration rule | **Adapt one piece**: waiting behind a walker of higher priority is waiting, not stalling (R2) — Detour's and the RTS engines' "wait while the blocker moves" |

No new crate is adopted. The adopted rule extends code we already own (`give_way` and the stride resolver). The
reuse record is therefore the decision record (§7), which names WHCA\*/`mapf` and the rule sets with the reasons
they were not adopted.

## 4. Recommendation

The rules below are generic. None of them names a place, a person or a world.

### R1 — a walker of lower priority gives way to one of higher priority (bodies)

```text
priority     a walk's priority is (started, person): the earlier `walk-started` instant ranks higher,
             and the lower entity id breaks ties. An idle person ranks below every walker (N-f R2,
             unchanged). A total order, so every pair is decided the same way on every platform.
give way     when a walker's stride is stopped short by a walker of LOWER priority, bodies' resolver
             tries a give-way move for that walker, exactly as N-f R2 does for an idle person:
             nearest free lattice point off the higher walker's corridor, within GIVE_WAY_MAX,
             preferring THRESHOLD_CLEAR from every doorway, reached by a clear segment, the result
             verified (V1–V4); else today's degrade
inheritance  the walker who gives way may in turn move an idle person or a walker of lower priority
             than the one it yields to, at most CHAIN_MAX (2) deep — PIBT's inheritance, bounded by the
             resolver's existing chain limit. A failed chain backtracks: nobody moves and the higher
             walker degrades as today (PIBT's "try the next-best move" is the head-on bias and the
             halvings we already run)
its walk     the walker who gave way keeps its walk. Its next step re-plans from where it stands
             (displaced → re-plan, as today)
facts        presence's `arrived` for the moved walker with participants [moved, mover] and bodies'
             `gave-way { person, for }`, exactly as N-f states them, so sleep (R4 of N-f) and the
             clients need nothing new
```

- **Why the earliest start ranks highest.** It is PIBT's "longest since the last goal", read from facts movement
  already records, so no counter has to be kept. A walk that has gone on longer has been held up longer, so the
  rule cannot starve anyone: every walk eventually becomes the oldest one in its place.
- **Contract.** Movement's public `Walking` gains a read-only accessor `started() -> WorldTime`, stored when the
  walk begins (`Walking` is a movement component; adding a field changes its codec, so the movement component's
  schema version rises by one). Bodies reads it by type, as it already reads `Walking` (N-f R2). No new fact.

### R2 — waiting behind a walker of higher priority is not a stall (movement)

A stride that ended short because a walker of higher priority stopped it (`stopped_by` names someone, and that
person's walk outranks this one) counts as **waiting**, not as a stall: `stalls` does not rise. Waiting is bounded
by its own counter, `WAITS_MAX`, so a walk can never hang:

```text
WAITS_MAX   20 consecutive waiting steps, about 20 strides of the walker ahead: long enough for a
            queue of four or five people to clear a single-file door (stride 1 340 mm; a doorway
            passage is one to two strides per person), short enough that a true deadlock still ends
            the walk `stalled` within minutes
```

`STALLS_MAX` (3) and `REPLANS_MAX` (8) are unchanged for everything else: a walk stopped by a solid, by an idle
person who cannot give way, or by a lower-priority walker who could not move.

### R3 — the stride and the planner agree on what is passable (bodies; F-Ng-4)

Commit 1 pins the street (7074, 1214) stop as a scenario: Market Town's street solids, one walker, the leg to
the store's door, no other people. If the stride's clearance is the cause, the stride sweep and the planner's grown
boxes use the same margin (`GROWN` against `CLEARANCE`), and the fix goes in this PR. If the cause turns out to be
elsewhere and needs a change beyond bodies' resolver or route, the implementer stops and reports it with evidence
(§6.1 stop rule).

### Not in this PR: F-Ng-2 and F-Ng-3

Both are policy and supply defects, not walking defects. §8 states them and recommends PR **N-h**. The rows they
fail (AC-S35-8's famished clause and AC-LE-3) are not N-g's acceptance; N-g must only not make them worse (§5,
AC-Ng-6).

## 5. Acceptance

The thresholds are fixed here, before any run.

| ID | Criterion | World(s) |
| --- | --- | --- |
| AC-Ng-1 | Scenario, one-door room 4 × 3 m: five walkers start for the street in the same step. Before the fix at least one walk ends `stalled` (pinned red in C1). After it, all five end `arrived` within 60 `walk-step`s each | `systems/bodies/tests/scenarios.rs` test room |
| AC-Ng-2 | Scenario, a doorway 1 m wide: two walkers meet head-on in it, one going in and one going out. Both end `arrived`; the one with the later start gives way | same file |
| AC-Ng-3 | Scenario: F-Ng-4's street leg ends `arrived` with no stop by a solid (R3) | Market Town street, scenario copy |
| AC-Ng-4 | Market Town, seeds {1, 7, 42}, 30 days: walks to a place that end `stalled` ≤ 8 per seed (today 124–162), none in a 15-minute window with another, and none in the apartments between 06:00 and 06:59 | Market Town |
| AC-Ng-5 | Generated variant **"same-minute Commute Town"**: the crowded Commute Town of N-f (`punctuality.rs`, `crowded_commute_town`) further rewritten so that every resident's first punctual part starts at the same time, which makes them all leave `home-near` in the same step. 7 days × seeds {1, 7}: no walk ends `stalled` or `no-route`; every part on time; every shift present. Generated at test time, never checked in | crowded Commute Town, generated |
| AC-Ng-6 | No regression: Commute Town and crowded Commute Town pass every row they pass on N-f. Market Town's AC-S35-8 and AC-LE-3 numbers are printed and must not get worse than §2's (longest famished ≤ 67 h; early arrivals ≤ 2 per seed); they become pass/fail in N-h | Commute Town ×2, Market Town |
| AC-Ng-7 | Replay and kill/resume reproduce Market Town seed 7 byte for byte on the three platforms (CI at the milestone) | Market Town |

Per the speed-first rule, a commit runs its own scenario tests and `fast`. AC-Ng-4 to -7 run in the milestone
batch.

## 6. Commit plan

| # | Commit | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C1 | `test(bodies): pin walkers jamming walkers` — AC-Ng-1, -2, -3 scenarios, red, `#[ignore]` with the finding ids | [ ] | [ ] red on N-f's head, with the outcome recorded | [ ] |
| C2 | `feat(movement): a walk knows when it started` — `Walking::started`, component schema version raised | [ ] | [ ] movement tests; replay of a fresh save | [ ] |
| C3 | `feat(bodies): a later walker gives way to an earlier one` — R1 in `give_way` and the resolver | [ ] | [ ] AC-Ng-1, -2 PASS; bodies and N-f scenarios still PASS | [ ] |
| C4 | `fix(movement): waiting behind an earlier walker is not a stall` — R2, `WAITS_MAX` | [ ] | [ ] AC-Ng-1 PASS within its step bound | [ ] |
| C5 | `fix(bodies): the stride and the plan agree on clearance` — R3 | [ ] | [ ] AC-Ng-3 PASS | [ ] |
| C6 | `test(cli): same-minute Commute Town, generated; Market Town stall rows` + docs (`MODULE_SPEC.md` movement and bodies sections, both packs' READMEs, decision record, generated reference) | [ ] | [ ] `check_decision_ids.py`; §5 rows at the milestone | [ ] |

### 6.1 Execution contract (to be filled at freeze)

```text
worktree      /Users/yuema137/mineworld-worktrees/impl-s35-ng, branch mvp0/pr-s35-ng
authority     DESIGN FROZEN <date>; Q-Ng-1 … Q-Ng-6 as ruled; decision id ARC-<n>
validation    speed-first: each commit runs its own tests and `fast`; AC-Ng-4 … -7 in the milestone batch;
              heavy runs through cast-slots/with-slot.sh
stop          any change to a frozen invariant, a public contract beyond §4 (`Walking::started`), scope,
              or R3's cause lying outside bodies' resolver and route
```

### 6.2 Deviations and bounded discoveries

(none yet)

## 7. Records

- `ARC-3xx` "Walkers yield by priority": R1–R3, `WAITS_MAX` and its reasoning, and §3's comparison (WHCA\* and
  `mapf`, PIBT, rule sets and pass-through) with the reason each was or was not adopted. It amends `ARC-330` (R2's
  give-way now also applies to walkers of lower priority) and `ARC-75` (`Walking::started`).
- Step-35 ledger: F-Ng-1 and F-Ng-4 answered by N-g; F-Ng-2 and F-Ng-3 handed to N-h (§8).
- Step-11 non-goal "local avoidance of moving people" is narrowed: moving people now yield to each other by
  priority, but there is still no velocity-based avoidance.

## 8. Handed on: PR N-h (outline only, its own design)

- **F-Ng-2, hands full.** One generic rule for the controller and one fact from the world. (a) The world already
  discloses holdings and nourishment. The `rule` fetch band gains a step: hungry, nothing nourishing held, a `buy`
  offered but unavailable, and holdings at the capacity the world discloses → free one hand, by consuming a held
  consumable that does not nourish (tea, coffee) or else by dropping or giving the lowest-value good. That makes
  the policy data `free_hand_from` plus an order of preference. (b) Inventory should disclose the capacity, so
  that no controller hard-codes 6. Optional: the reason a `buy` is unavailable (`cannot-carry` against
  `sold-out`), so that an LLM seat can tell the cases apart.
- **F-Ng-3, empty shop.** The fetch band should not wait at a shop that has nothing to sell, and an errand that
  bought nothing is still an errand. Two options for N-h's design: (a) F-Nd-7's errand test also counts "arrived
  by a fetch walk" (the walk's purpose disclosed in the decision), or (b) the fetch band skips a shop whose `buy`
  offers are all unavailable and tries the next disclosed shop, else eats nothing and waits elsewhere. Supply
  sizing (restock before the 06:00 rush) is world content and is not the fix; the mechanism must hold in a
  generated variant whose shop opens late.

## 9. Freeze questions (with recommended answers)

**Ruled 2026-10-20 (operator, primary session):** every question below is answered as recommended.

| ID | Question | Recommended answer |
| --- | --- | --- |
| **Q-Ng-1** [OPERATOR] | Adopt PIBT's priority-with-inheritance rule (later walker gives way to an earlier one), and reject WHCA\* reservations and the `mapf` crate for now? | **Yes.** It is a local, one-step, deterministic rule that extends N-f's give-way. WHCA\* needs a new shared time-indexed table, and `mapf` is floating-point |
| **Q-Ng-2** | Priority = earliest `walk-started` first, then lowest entity id? | **Yes.** It is PIBT's "waited longest wins", it cannot starve anyone, and it needs no new counter |
| **Q-Ng-3** [OPERATOR] | One public addition: movement's `Walking::started()` (component schema version raised by one)? | **Yes.** It is the smallest contract change that lets bodies read the priority |
| **Q-Ng-4** | `WAITS_MAX` = 20 waiting steps, a pack constant (not world data)? | **Yes, a constant**, with §4 R2's reasoning; a world-data override can come later if needed |
| **Q-Ng-5** | Include F-Ng-4 (the solid at street 7074, 1214) as R3 in this PR, with a stop if its cause lies outside bodies' resolver and route? | **Yes.** It is a walking defect in the same resolver, and it is pinned first |
| **Q-Ng-6** [OPERATOR] | Hand F-Ng-2 (hands full) and F-Ng-3 (errand to an empty shop) to a separate PR N-h, with AC-S35-8's famished clause and AC-LE-3 owned there, and N-g only not making them worse? | **Yes.** The trace shows neither is caused by walking: the famished people reached the store 33–108 times, and Felix was early because the shelves were empty |
