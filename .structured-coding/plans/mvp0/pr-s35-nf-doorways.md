# PR S35 N-f — keep doorways passable (F-Nd-5, F-Nd-6, F-Nc-7)

```text
STATUS        DRAFT — awaiting operator freeze (questions Q-Nf-1 … Q-Nf-6, §8)
step          35 (needs and sleep), step-35-needs-and-sleep.md; milestone-batch fix
branch        mvp0/pr-s35-nf (implementation, after freeze); this design on docs/pr-design-s35-nf
base          origin/main @ 5a8c6f28
owns          movement (walk end), bodies (wayfinder answer, stride resolver), sleep (wake reason)
touches no    kernel, contracts, controllers, clients, world content
```

## 1. The problem, as a causal chain

In Market Town the store and the apartments each have one doorway. Walks there end badly:

1. **A walk to a place ends on its threshold.** `walk-to` with a place destination that has no position is
   "reached on entering its place" (`systems/movement/src/walk.rs`, `arrived`). The walker enters at the
   doorway's point `there`, which the world puts 0.4 m inside the room (`worlds/market-town/places/store.yaml`
   line 19). So every shopper who walked "to the store" stands on the store's doorway point.
2. **The next shopper is nudged onto the people already there.** An entry onto an occupied point nudges them
   aside by at most `NUDGE_MAX` (250 mm) and then lands on the nearest free lattice point (`entry.rs`, E2/E3).
   After two or three arrivals the people form a cluster round the door, with the wall behind them.
3. **Leaving is then blocked.** A walker's stride toward the doorway is stopped by a person in that cluster.
   A nudge cannot move that person, because the wall is behind them (`stride.rs`, nudge pass against the
   walls). The stride degrades to less than 50 mm. After `STALLS_MAX` (3) such strides the walk ends
   `stalled` (`walk.rs`, step 1).
4. **The re-plan gives up.** After a stride that was stopped, the leg is re-planned round the person who
   stopped it (`avoid`, step-11 SD-N9). That person's box, grown by 310 mm, covers the doorway in a narrow
   room, so the wayfinder answers `Unreachable` and the walk ends `no-route` (`walk.rs`, step 4).
5. **The controller retries every consult with the same result.** Alice (seed 7, day 15) gets `no-route` from
   the store from 04:09 to 07:45 and is late (F-Nd-5). People kept out of the store go hungry, people kept out
   of home sleep short (F-Nd-6). AC-S35-8 and AC-S35-9 fail for some seeds.
6. **Sleepers are woken by other people's walks.** A sleeper nudged by a passer-by gets an `arrived`, and sleep
   wakes everyone it reads in an `arrived` as `acted` (`systems/sleep/src/system.rs`, the `Arrived` arm). That
   is F-Nc-7, part (2). D-Nc2-4 moved beds away from doors, which made it rarer, but any nudge still wakes.

Commute Town passes because its homes hold one person each and its street is wide; nobody crowds a door.
So the defect is not in Market Town's data. It is that the movement model lets idle people stand in a
doorway, and gives a walker no way to get past them. Any world with a one-door room and a few visitors hits it.

Commit 1 (§6) pins steps 1–4 with a deterministic scenario before anything is changed, so each claim above
is checked against a red test, not taken from this reading of the code.

## 2. Mature approaches evaluated

The rule (operator, 2026-10-19): compare mature tools and models first, adopt or adapt, and build only after
recording why they do not fit. MineWorld's constraints are fixed by earlier decisions: integer positions in the
event log, byte-identical replay on every platform (`ARC-25`), one walker's request resolved at a time with no
common simulation tick (step-11 §21.3, option c′), and Rapier used only behind quantisation (`DEP-13`).

| Approach | Example tool or source | Licence | What it solves | Fit here | Verdict |
| --- | --- | --- | --- | --- | --- |
| Reciprocal velocity obstacles (ORCA) | RVO2 (UNC, C++); Rust ports `dodgy_2d` (ORCA, used by `landmass`) | RVO2 Apache-2.0; `dodgy_2d` MIT/Apache-2.0 with RVO2's notice | Moving agents that avoid each other, solved together each frame from their velocities | Needs every agent's velocity at one common time step; we have one request at a time and no tick. `f32` geometry, no determinism statement. ORCA only avoids: a person **standing still** in a one-person doorway still blocks it, and ORCA is known to deadlock in bottlenecks. `landmass` was already rejected for the same reasons (step-11 §21.4) | **Rejected** — wrong time model, floats, and it does not address idle blockers |
| Social force model | Helbing and Molnár (1995); used in JuPedSim, Vadere (research simulators) | various (LGPL, MIT) | Crowd flow, bottleneck capacity | Continuous-time float integration of all pedestrians; built to *measure* clogging at exits, not to avoid it | **Rejected** for the same time-model reason |
| Doorway reservation / cooperative pathfinding | WHCA* (Silver, 2005); traffic-light reservation in RTS and warehouse robots | (algorithms) | Two **movers** contending for one narrow cell | Our blockers are not moving; a reservation does not move them. Needs a time-indexed plan per walker — far beyond our per-request walk | **Rejected** now; revisit if moving walkers contend once idle blockers are gone |
| Avoidance priority | Unity `NavMeshAgent.avoidancePriority`; Recast/Detour crowd | proprietary; zlib | Who yields when two agents meet | Detour's crowd also has no idle-agent yielding; priority alone does not move an idle agent | **Adapted** as one rule: a walker outranks a person who is not walking |
| **Moving units push idle units aside** | Standard in RTS engines (StarCraft II, Age of Empires II: a unit walking through idle units pushes them out of its path) | (behaviour, no code) | An idle agent standing in a walker's way | Exactly our blocker. We already have the mechanism, bodies' nudge, but capped at 250 mm and walls-only | **Adopt the model**, as a larger give-way move in bodies' resolver |
| **Arrival spots / waiting areas off the threshold** | The Sims' routing slots; designers' "don't stop in a doorway" rule; NavMesh "arrival areas" | (behaviour) | People who arrive at a place stopping on the doorway | Derivable from data every world already has: the place's floor and its doorway points | **Adopt the model**, generically, from place geometry |

No new crate. The two adopted models extend code we already own (bodies' nudge and its lattice search), so the
reuse record is a decision record that names the rejected tools and why (§7, `ARC-<id>`), not a `DEP` record.

## 3. Recommendation

Three generic rules and one wake rule. None names a place, a person or a world.

### R1 — a walk to a place ends clear of the threshold (movement + bodies)

When a walk's destination is a place without a position, the walk does not end on entering. It plans one more
leg, inside the destination, to an **arrival spot** the wayfinder chooses, and ends there.

```text
arrival spot  the first free point of the 50 mm lattice, in the entry's E3 order (distance, then y, then x),
              measured from the doorway point `there`, that is
                - at least THRESHOLD_CLEAR from every doorway point of the place (all doors, not only
                  the one entered by), and
                - free by the route planner's own test (inside the shrunk floor, outside every grown box)
              none such → the doorway point itself (today's behaviour)
THRESHOLD_CLEAR  1 200 mm, centre to centre: one person standing (R 250) plus a passing lane of one
              person's width (0.6 m) plus the gap. Reference: the minimum clear door width of 815 mm
              (ADA 2010 §404.2.3) plus a body's radius each side; Fruin (1971) level-of-service A
              space per standing pedestrian ≈ 1.2 m².
```

- **Contract.** Movement's `Wayfinder` trait (`ARC-75`) gains one method,
  `fn arrival_spot(&self, world, place, entered_at) -> Option<LocalPosition>`, default `None`. Bodies answers it
  from `PlaceShape`, `LooseObjects` and movement's `Passages`, which it already reads. Without bodies, or in a
  place with no shape, the answer is `None` and the walk ends on entering, as today.
- **Disclosure unchanged.** The walk's extra leg is an ordinary leg of `Walking`, disclosed as today.
- **Why movement ends the walk and bodies picks the point:** movement owns walks and passages; bodies owns
  geometry (step-11 §21.3, a1). This keeps that split.

### R2 — a person who is not walking gives way to one who is (bodies)

When a stride is stopped by a person who has no walk in progress (no `Walking` component, read by type, as
bodies already reads `Passages`), the resolver tries a **give-way move** before degrading:

```text
give-way      the blocker moves to the nearest free lattice point (E3 order, from where they stand)
              that is outside the walker's swept corridor (start → target, half-width R + GAP),
              at least THRESHOLD_CLEAR from every doorway point, within GIVE_WAY_MAX of them,
              and reachable from them by a clear straight segment
GIVE_WAY_MAX  1 200 mm (one step aside and back; the same comfortable step as THRESHOLD_CLEAR)
accepted iff  the result verifies (V1–V4), as every other candidate does; else today's degrade
facts         presence's `arrived` for the blocker, caused by the walker's request, like a nudge,
              plus bodies' new fact `gave-way { person, for }` so that other packs can tell it apart
```

- A walker never gives way by this rule, so two walkers meeting still use the head-on bias (step-11 SD-B10).
- A seated or lying person (posture) gives way like a standing one. The rule reads only "walking or not".
- This is the RTS rule adopted as stated. It also clears doorways that R1 cannot: a person who stopped in a
  doorway for another reason (a conversation, an interrupted walk).

### R3 — a re-plan round the blocker falls back once to the plain plan (movement)

When the re-plan with `avoid` answers `Unreachable`, movement plans once more without `avoid` before it ends the
walk `no-route`. The next stride then meets the blocker, and R2 moves them. `no-route` stays for places that are
really unreachable.

### R4 — another person's walk does not wake a sleeper as `acted` (sleep; F-Nc-7)

Sleep's `Arrived` arm wakes a sleeper only when the arrival is their own: the `arrived` names the sleeper and is
**not** paired with a `gave-way` or a nudge of that sleeper in the same request. A sleeper who is moved aside by
someone else keeps sleeping. (Recommended over waking them with a new reason `disturbed`: a nudge of 250 mm or a
step aside is not a reason to break a night's sleep, and AC-S35-8 measures sleep length.) Talking (`spoke`) and
the alarm still wake as today.

Bodies already emits nudges inside one request's emission list; R4 needs bodies to state them. `gave-way` (R2)
covers give-way moves. For a plain nudge, bodies emits the same `gave-way` fact with the nudge's distance, so
sleep reads one fact for both (Q-Nf-4).

## 4. Worlds it must pass on

| World | Why | Must hold |
| --- | --- | --- |
| Market Town, seeds {1, 7, 42}, 30 days, both packs | the failing world | AC-S35-8 and AC-S35-9 rows; zero walks ended `no-route` whose place is reachable; `woke { acted }` only for the sleeper's own walk |
| Commute Town (`tools/cli/tests/worlds/commute-town`) | the second world, already passing | every row it passes today still passes; its digest may change only where R1 moves a walk's end |
| **Commute Town, crowded** — generated at test time, never checked in | a one-door variant built to crowd: the test copies Commute Town into a temp dir, moves every resident into `home-near` and shrinks the bakery's floor to 4 × 3 m round its one doorway, by rewriting the copied YAML | 7 days × seeds {1, 7}: no walk ends `stalled` or `no-route` at a doorway; all shifts present (AC-LE-1); AC-S35-9 holds |
| Bodies Yard | the bodies regression world | its existing tests pass unchanged except where a give-way replaces a degraded stride, which the PR lists |

## 5. Acceptance

| ID | Criterion | World(s) |
| --- | --- | --- |
| AC-Nf-1 | A scenario test with a one-door room, three idle people on the threshold cluster and one walker leaving: before the fix it ends `stalled` or `no-route` (pinned in commit 1); after, the walker leaves within 10 `walk-step`s | test room in `systems/bodies/tests/scenarios.rs` |
| AC-Nf-2 | A walk to a shaped place ends at least `THRESHOLD_CLEAR` from every doorway point when a free point exists; ends on entering when the place has no shape (inert without bodies) | Market Town store; Bodies Yard |
| AC-Nf-3 | A sleeper moved aside by another person's walk is still asleep after it; one who walks is woken as `acted` (unchanged) | crowded Commute Town |
| AC-Nf-4 | §4's rows | all four |
| AC-Nf-5 | Replay and kill/resume reproduce the history byte for byte on the three platforms (CI at the milestone) | Market Town seed 7 |

## 6. Commit plan

| # | Commit | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C1 | `test(bodies): pin the doorway jam` — AC-Nf-1 scenario, expected to fail, marked `#[ignore]` with the finding id | [ ] | [ ] red on main, naming `stalled`/`no-route` | [ ] |
| C2 | `feat(movement,bodies): a walk to a place ends clear of the threshold` — R1, `Wayfinder::arrival_spot`, `THRESHOLD_CLEAR` | [ ] | [ ] AC-Nf-2; movement + bodies tests | [ ] |
| C3 | `feat(bodies): idle people give way to walkers` — R2, `gave-way` fact, `GIVE_WAY_MAX`; nudges also state `gave-way` | [ ] | [ ] AC-Nf-1 un-ignored and green | [ ] |
| C4 | `fix(movement): fall back once to the plain plan` — R3 | [ ] | [ ] unit test: avoid-plan unreachable → walk continues | [ ] |
| C5 | `fix(sleep): another person's push does not wake a sleeper` — R4 (F-Nc-7 part 2) | [ ] | [ ] AC-Nf-3 | [ ] |
| C6 | `test(cli): crowded Commute Town, generated` + docs (`MODULE_SPEC.md` movement and bodies sections, the packs' READMEs, decision record, reference regenerated) | [ ] | [ ] `check_decision_ids.py`; §4 rows at the milestone | [ ] |

Per the speed-first rule, a commit runs its own tests and `fast`; §4's 30-day rows run in the milestone batch.

## 7. Records

- `ARC-<id>` (id from the coordinator's range): "Doorways stay passable" — R1 … R4, the constants and their
  references, and §2's comparison (ORCA/RVO2, `dodgy_2d`, social force, WHCA*, Detour) with the reason each was
  not adopted. Amends `ARC-75` (one `Wayfinder` method) and `ARC-39` (a give-way candidate before degrade).
- Step-35 ledger: F-Nd-5, F-Nd-6 and F-Nc-7 (2) answered by N-f.
- Step-11 non-goal "local avoidance of moving people" stays a non-goal: R2 moves only people who are not walking.

## 8. Freeze questions (with recommended answers)

| ID | Question | Recommended answer |
| --- | --- | --- |
| **Q-Nf-1** [OPERATOR] | Adopt the two behaviour models (arrival spot off the threshold; idle people give way) and reject ORCA/RVO2 and social-force crates for this defect? | **Yes.** ORCA needs a shared tick and floats and does not move a person who is standing still; both adopted models are standard in games and extend code we already have |
| **Q-Nf-2** [OPERATOR] | Add one method to the public `Wayfinder` trait (`arrival_spot`, default `None`)? | **Yes.** It is the smallest contract change; without bodies nothing changes |
| **Q-Nf-3** | `THRESHOLD_CLEAR` and `GIVE_WAY_MAX` both 1 200 mm, constants of the packs (not world data)? | **Yes, constants**, with the cited references. A world-data override can come later if a world needs it |
| **Q-Nf-4** | A new bodies fact `gave-way { person, for, distance }` for both give-way moves and nudges, so sleep can tell a push from the sleeper's own walk? | **Yes.** It also lets clients animate a step aside |
| **Q-Nf-5** [OPERATOR] | A sleeper moved by someone else keeps sleeping (rather than waking as `disturbed`)? | **Keep sleeping.** Talking and the alarm still wake them |
| **Q-Nf-6** | Second worlds: Commute Town plus a test-generated crowded one-door variant, no new checked-in world? | **Yes**, as §4 |
