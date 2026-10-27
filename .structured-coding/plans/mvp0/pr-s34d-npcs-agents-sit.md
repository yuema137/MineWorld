# PR S34-d — NPCs and agents sit

## Lifecycle

```text
Design revision:        revision 1 (2026-10-26), DESIGN FROZEN 2026-10-27 (operator approved in the primary session: every question answered as recommended)
Approved by / evidence: none yet — awaiting the operator's freeze (questions in §14)
Implementation base:    origin/main @ 0b2faf2f (#382)
Start condition:        this design frozen. S34-c1 is merged (#368). S34-c2 (Lakeside) is not required to
                        start: Lakeside joins every test of this PR by discovery when it lands (Q-S34d-3)
Execution contract:     §12
Lifecycle:              DRAFT — not frozen
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-34-posture.md`](step-34-posture.md) §§8.5, 9, 12–14
(the S34-d row of §14.2; AC-S34-6's "NPCs are seen seated", AC-S34-7; I-S34-1, -2, -4, -5, -6, -7;
Q-S34-11; R-S34-2) · **Previous PRs:** [`pr-s34a-posture-pack.md`](pr-s34a-posture-pack.md) (#301),
[`pr-s34b-bodies-anchoring.md`](pr-s34b-bodies-anchoring.md) (#363),
[`pr-s34c-world-sit-points.md`](pr-s34c-world-sit-points.md) (S34-c1 #368) · **Record:** a dated note on
`ARC-147` (`docs/decisions/ARC-147.md`), and a note on `ARC-151` for the two action types the rule now
decides by name. No new id.

**Working name:** S34-d, in two PRs (Q-S34d-1): **S34-d1** the controllers and the 30-day measurement;
**S34-d2** the agent view and the recorded-agent scenario. **Branches:** `mvp0/pr-s34d1`, `mvp0/pr-s34d2`.
One worktree and one session each. The two touch no common file.

**In one sentence.** Today an NPC sits only when chance leaves it within 750 mm of a sit point, and then
stands up again on a one-in-five draw at the next consult, so in the live server people bob down and up
every few seconds; and an MCP agent is offered `sit` but cannot see where the seat is. This PR measures
both on long seeded runs, gives the rule and motivated controllers one generic "rest where I stay"
behaviour that reads only what the world discloses, and gives the agent view the seats and who sits on
them.

---

## 0. Binding inputs

- **Step row (frozen 2026-10-18), verbatim:** "evaluate the paced and motivated controllers on the 30-day
  runs; Q-S34-11's stillness gate if needed (inside posture); `clients/agent` `disclosed.py` row and words;
  the recorded-agent scenario". Integration checkpoint: "AC-S34-6's 'NPCs are seen seated'; AC-S34-7".
  Adversarial focus: "an agent attempts a stale sit offer (`stale_offer`); a taken point". Implementer:
  Haiku (Q-S34-14).
- **AC-S34-6** (step §13.1): "… a seeded 30-day run per world holds I-S34-1, I-S34-2 and I-S34-7 after every
  request, and NPCs are seen seated."
- **AC-S34-7** (step §13.1): "An MCP agent (recorded, deterministic) walks to the pier bench, sits, is seen
  seated by a human seat and by an NPC's observation, and stands; zero lines change in the adapter's act
  dispatch."
- **Q-S34-11** (ruled as recommended): "Measure first in S34-d's 30-day runs; add the 60 s gate inside
  posture only if people bob. The gate applies to players too."
- **Q-S34c-4** (ruled): the 30-day runs belong to S34-d; S34-c runs a 300-step invariant check.
- **Lane rule, no world-specific hard-coding (operator, 2026-10-18):** mechanisms are generic and
  data-driven. The world discloses facts; each controller kind decides by its own model; consequences are
  events owned by a System. Every acceptance criterion names a second world or a generated variant.
- **Agents are first-class players:** an outside agent sits through the same server path and the same
  rules as a human.
- **Local load, CI is the gate (R-LOAD-1):** no local full-workspace test runs; CI on the exact head
  decides.
- **Tools first, no hand-tuning:** a new constant is set once, with its reason, before measuring. A run
  that misses a threshold is escalated, not tuned until green.

## 1. Scope

**In, S34-d1 (controllers and measurement):**

1. A generic long-run test, `tools/cli/tests/posture_life.rs`, over every World Pack under `worlds/` whose
   `systems:` names `posture`, plus generated variants (§5 SD-d-6). It reads the saved journal and checks
   the invariants after every fact and the "seen seated" and "no bobbing" measures (§5 SD-d-1).
2. A baseline of those measures on the current controllers, recorded in §13 before any controller change
   (Q-S34-11: measure first).
3. The rule controller (`cognition/rule-controller`): `sit` and `stand` join `DECIDED_BY_NAME`, and a new
   rest band decides them (§5 SD-d-3).
4. The motivated model (`cognition/motivated`): `sit` and `stand` leave the generic `Offered` choice, and two
   new options, `seek-seat` and `sit`, with base weights in its settings (§5 SD-d-4).
5. Re-baselines of every golden the change moves: the Social Café 30-day decisions golden, the
   `scripts/baselines.txt` sections of Social Café and Market Town, with a `reason S34-d1` line.

**In, S34-d2 (the agent view and the scenario):**

6. `clients/agent`: three new reads in `disclosed.py`, typed lines in `models.py`, wired in `view.py`
   (§5 SD-d-5): who is seated (`posture-state`), where the seats of my place are (`sit-points`), and which
   seat a `sit` offer is for.
7. A recorded-agent MCP scenario, `clients/agent/tests/test_play_sit_mcp.py`, on the real server, over
   every posture world and a generated variant (§5 SD-d-7), with the stale-offer and taken-seat adversaries.
8. Docs: the `disclosed.py` module table; the agent's reference pages if they list view lines; `ARC-147`'s
   note.

**Out:**

- Any change to the `posture` pack. Q-S34-11's stillness gate is **not** added (F-d-3 shows it would not
  stop the bobbing; Q-S34d-2). If the operator rules otherwise, it is a posture change with its own
  version bump.
- The LM controller (`cognition/lm-controller`): it already sees `sit` and `stand` among its offers like
  any other complete offer (step §9.2). No change.
- Sit points in the station halls of Social Café and Market Town, and Lakeside's sit points (S34-c2,
  S34-c3; F-d-5, F-d-6, Q-S34d-3, Q-S34d-4).
- Every 2D and 3D client change (S34-e, S34-f).
- The MCP adapter's act dispatch (`mcp_server.py` `ACTS`, `player.py`): zero lines change (AC-S34-7).

## 2. Source audit (`origin/main @ 0b2faf2f`)

| # | Fact | Where |
| --- | --- | --- |
| 1 | `posture` is VERSION 2; it provides `sit {point}` and `stand {}`; it stands a seated person up on their own `arrived` off the point or on `walk-started` | `systems/posture/src/system.rs:56`, `:166-230` |
| 2 | One verdict for offer and dispatch: list forbids → `PermissionDenied`; no such point → `NoSupportedInteraction`; seated or walking → `Busy`; farther than reach (750 mm) → `TooFarAway`; taken or crowded → `TargetUnavailable` | `systems/posture/src/system.rs:245-288` (`sit_verdict`) |
| 3 | Offers: standing, one complete `sit {point}` per sit point of my place, in key order, each refused with its verdict; seated, one complete `stand {}` | `systems/posture/src/perception.rs:25-56` |
| 4 | Disclosure: a place's `sit-points` to whoever perceives the place; a person's `posture-state` to whoever perceives the person. `posture-state` serializes as `{ "seated": { "place", "point", "since" } }` | `systems/posture/src/perception.rs:59-79`; `component.rs:22-36` |
| 5 | At most 64 sit points per place | `systems/posture/src/point.rs:14` (`SIT_POINTS_MAX`) |
| 6 | Worlds with `posture`: Social Café and Market Town only (21 sit points each: café 9, park 6, street 6) | `worlds/social-cafe/world.yaml:72`; `worlds/market-town/world.yaml:125`; `docs/decisions/ARC-147.md:44-58` |
| 7 | Lakeside does not enable `posture`; its 29 sit points exist only as the 3D client's fallback depiction file, written in S34's shape for S34-c2 to copy | `worlds/lakeside/world.yaml:46-66`; `presentation/mineworld-default/3D/sit_points/lakeside.yaml`; `tools/cli/tests/lakeside_sit_points.rs:34` |
| 8 | Social Café and Market Town each have a station hall with 34 drawn benches (6 000 mm each) and two platform benches, and no `sit-points:` section | `worlds/market-town/places/station.yaml:25-36`; `station-platform.yaml:27-44` (same in `social-cafe`) |
| 9 | The paced rule's offered band attempts an available complete offer on 20 of 100 consults; constants frozen; "the remedy … is in what its packs offer" | `cognition/rule-controller/src/offered.rs:27-36`, `:56-79` |
| 10 | Action types the rule decides by name are left out of the offered band's pool (`ARC-151`); today 10 types, not `sit` or `stand` | `cognition/rule-controller/src/paced.rs:117-132` |
| 11 | The offered band is the last band before doors and wander, after needs, social, agenda and commute | `cognition/rule-controller/src/paced.rs:228-345` |
| 12 | Draw indices in use: 0–6 (walking), 8–12 (social), 13 (agenda), 14–15 (offered). A new band takes new indices and moves no other band's draws | `paced.rs:38-40`; `social.rs:48-52`; `agenda.rs:30`; `offered.rs:39-42` |
| 13 | Precedent for a controller that walks to a disclosed spot and then acts on an offer: the bed band (`to_bed`, then `sleep`), compiled against `mineworld-sleep` | `cognition/rule-controller/src/needs.rs:600-640`; `rule-controller/Cargo.toml:30` |
| 14 | Motivated: every available complete offer is one `Offered` choice (base weight 200), except `penalize`, `put-down`, `pick-up`; `stay` weighs 300; `Sleep` is a typed option (`ARC-286`) | `cognition/motivated/src/options.rs:45-73`, `:376-388`, `:495-515`; `settings.rs:226-239`; `parameters.rs:51-54` |
| 15 | **The baseline already in the repository:** Social Café, seed 7, pace 900 s, 30 days, all paced rule: `sit accepted 50`, `stand accepted 9`, `sat-down 50`, `stood-up 50` — so 41 of 50 spells ended by walking away | `tools/cli/tests/fixtures/decisions/social-cafe-seed-7-30-days.txt:39-40, 72-74` |
| 16 | A hosted server consults each paced seat every 5 **wall** seconds by default; `run` consults every 900 **world** seconds | `tools/cli/src/main.rs:140-142` (`--cadence`), `:230-233` (`--pace`) |
| 17 | Precedent for a long-run measure read from a saved journal: group-activity lengths over three worlds and two seeds | `tools/cli/tests/activity_length.rs:1-90` |
| 18 | Precedent for generated world variants built at test time | `tools/cli/tests/variants/mod.rs:1-60` |
| 19 | Bodies' `gave-way { person, for }` names the person who stepped aside | `systems/bodies/src/event.rs:304-330` |
| 20 | Agent view: `disclosed.py` reads one pack's component per function and lists them in a module table; `asleep` is the nearest precedent (`MeLine.asleep`, `PersonLine.asleep`) | `clients/agent/src/mineworld_agent/disclosed.py:9-30`, `:528-534`; `view.py:353, 375` |
| 21 | No `posture-state` or `sit-points` read anywhere in `clients/agent` | `git grep -n posture clients/agent` (no match) |
| 22 | An `OfferLine` carries action type, target, verdict, requirement, `complete`, and `about` (entities the payload names). A sit point is not an entity, so a `sit` offer shows no seat | `clients/agent/src/mineworld_agent/models.py:603-629`; `view.py:248-275` |
| 23 | `walk_to` already takes a point (`ToPoint`) | `models.py:244`, `:250` |
| 24 | The `act` tool's dispatch is one table; `attempt` sends a complete offer as is; a stale view gives `stale_offer` | `mcp_server.py:179-192`; `player.py:549`; `models.py:116` |
| 25 | Precedent for a recorded (scripted, no model) agent on the real server over MCP, choosing places by what it observes, with a generated world | `clients/agent/tests/test_play_station_mcp.py:1-12`; `test_play_lakeside.py:1-15`; `realserver.py:69-180` |
| 26 | Nothing of S34-d exists on main: no posture read in either controller, no posture read in the agent, no `posture_life` test | `git grep -n -i "posture\|sit-points" cognition clients/agent` (no match) |

## 3. Findings

- **F-d-1. NPCs sit only by accident.** A `sit` offer is available only within 750 mm of the point (audit 2).
  No band walks anyone toward a seat. So a person sits only when another band happened to leave them that
  close. Audit 15: 50 spells in 30 days among 11 people.
- **F-d-2. The bobbing is real, and its cause is `stand`, not `sit`.** A seated person's only posture offer
  is a complete `stand` (audit 3). The offered band draws it on up to 20 of 100 consults (audit 9). In a
  hosted server a seat is consulted every 5 wall seconds (audit 16), so an NPC who sits stands again after
  about 25 wall seconds on average. That is what a player watching sees.
- **F-d-3. Q-S34-11's stillness gate would not fix it.** The gate delays `sit`, not `stand`. A gate on
  `stand` would trap players, which posture must never do (step §6.2). So the remedy belongs in the
  controllers, not in posture.
- **F-d-4. An agent cannot find the seat.** It sees `sit` offers refused `TooFarAway`, but no position and
  no key (audit 22). It cannot walk to the seat. The place discloses `sit-points` (audit 4), but the view
  does not read it.
- **F-d-5. The station halls have drawn benches and no sit points** (audit 8), against I-S34-9. Their 34
  benches hold about 340 seats, more than the 64-point cap (audit 5). And posture offers one `sit` per point
  of the place (audit 3): 340 offers in every observation in the hall. This needs a step-level ruling and is
  not S34-d's work (Q-S34d-4).
- **F-d-6. Lakeside is not a posture world yet** (audit 7). AC-S34-7 names "the pier bench". S34-c2 has not
  started.
- **F-d-7. The motivated model has the same bobbing path:** `stand` is one more `Offered` choice (audit 14).

## 4. Invariants

- **I-S34-1, -2, -7** (step): checked after every fact of every long run of this PR.
- **I-S34-4** (step): no controller and no agent code computes reach, occupancy or stillness. A controller
  reads the server's verdict on each offer. The only position it reads is the seat's disclosed `at`, to
  walk there.
- **I-S34-5** (step): NPCs and agents sit through the same complete `sit` offer and the same validation as
  a human.
- **I-S34-6** (step): a world without `posture` decides exactly as before. Its controllers see no `sit` or
  `stand` offer, take no new draw, and its baseline does not move.
- **I-d-1. Generic.** No world, place, person or sit-point key appears in code or tests. Worlds are found by
  listing `worlds/` and reading `systems:`; seats are found by reading the observation.
- **I-d-2. Frozen constants stay frozen.** No existing constant of the rule controller changes
  (`ATTEMPTS_OFFERED`, the band thresholds, the draw indices 0–15). New behaviour takes new constants and
  new draw indices.
- **I-d-3. No new tool and no new act kind.** The MCP adapter's tool list and `ACTS` table are unchanged.

## 5. Decisions

- **SD-d-1. What "seen seated" and "no bobbing" mean (measured from the journal).**
  - *Seated person-hour sample:* at every whole world hour from 09:00 to 18:00, the set of persons with a
    seated row (rebuilt from `sat-down` and `stood-up`). A sample **shows seating** when the set holds at
    least one person no player holds (in `run`, nobody is held by a player).
  - *Spell:* from a person's `sat-down` to their next `stood-up`. Its length is counted in world seconds
    and in the run's consults (length ÷ pace).
  - *Bobbing:* a spell that ends by the sitter's own `stand` request (cause `Requested`) within one consult.
  - The thresholds are in §6 and are fixed before measuring (`ARC-23`); Q-S34d-5 asks the operator to
    confirm them.
- **SD-d-2. Measure first.** C1 lands the test with the invariant checks asserted and the measures printed.
  Its numbers on the unchanged controllers go into §13 as the baseline. Only then do C2 and C3 change the
  controllers, and C4 turns the thresholds on.
- **SD-d-3. The rule controller rests where it stays (one new band).**
  - `sit` and `stand` join `DECIDED_BY_NAME`, so the offered band never draws them again (the `ARC-151`
    lesson: one decision, one place). This alone removes F-d-2's bobbing.
  - A new **rest band** sits just before the offered band (after needs, social, agenda and commute). It acts
    only when I am not walking, and either have no agenda or am at my agenda's place:
    1. an **available** `sit` offer in my observation → attempt the first one in observation order, on
       `SITS_BELOW` of 100 draws at draw index 16;
    2. else a `sit` offer refused **only** `TooFarAway` → walk to that seat's disclosed `at` (movement's
       `walk-to`, as the bed band does), the nearest one first, ties by key, on the same draw;
    3. a `sit` offer refused for any other reason (taken, crowded, forbidden) is never chosen.
  - It never decides `stand`. A seated person gets up only by setting off somewhere through another band,
    which posture turns into `stood-up { Moved }` (Q-S34-4).
  - `SITS_BELOW = 50`: an even chance per consult, so a person staying near a free seat sits within two
    consults on average. It is a literal, set once here and never derived from what it produces (`ARC-23`
    rule 2), like `FOLLOWS_AGENDA`.
  - The seat position comes from the place's disclosed `sit-points`, decoded with posture's own type: the
    crate gains a dependency on `mineworld-posture`, as it has on `mineworld-sleep` (audit 13).
- **SD-d-4. The motivated model gets the same two choices as typed options.**
  - `sit` and `stand` leave the generic `Offered` choice (as `put-down` and `pick-up` did).
  - `Choice::Sit(index)`: attempt the available `sit` offer at that index. `Choice::SeekSeat(key)`: walk to
    that free seat's `at` (step 2 of SD-d-3's rule, nearest first).
  - Base weights in `Settings`: `sit: 300` (sitting is a way of staying, so it weighs as `stay`),
    `seek-seat: 250` (a short walk, so it weighs as `wander`). Both names join `parameters.rs`'s `OPTIONS`,
    so a world's decision list can change them. That is how a world tunes its people, not code.
  - No `stand` option: as in SD-d-3, leaving is a walk.
- **SD-d-5. The agent view says who sits, and where the seats are.**
  - `posture-state` → `MeLine.seated` and `PersonLine.seated`: a `SeatedLine { point, words, since }`.
    `words` is the seat's group key written as words (`window-stools` → "window stools"), else the key.
  - `sit-points` of my place → `Here.seats`: a list of `SeatLine { point, words, position }`, in key order.
    `position` is a `Point` in my place's frame, the one `walk_to` takes. No "taken" flag: occupancy is the
    server's verdict on the offer (I-S34-4).
  - A `sit` offer → `OfferLine.seat`: the key its payload names, read by one function in `disclosed.py`
    the way `buy_price` reads a `buy` payload.
  - The module table in `disclosed.py` gains the two rows.
- **SD-d-6. The long-run test (S34-d1).** For each world found (§I-d-1) and each variant, for seeds 7 and
  11: `mineworld run <world> --headless --seed s --days 30 --save S` with the default decision list, and
  again with every seat put on `motivated` by `--seat-model` (seats read from `world.yaml`), and once at
  `--pace 60 --days 5` (a hosted-like pace). It reads `S` back and checks §6. Generated variants (built by
  `variants/mod.rs`, never checked in):
  - **renamed-seats:** a posture world whose sit-point keys and groups are all rewritten and listed in
    reverse order. A test or controller that names a key fails here.
  - **one-seat:** a posture world with every place's sit points cut to the first one. Contention is high.
  - **no-posture:** a posture world with `posture` and every `sit-points:` section removed. Its summary
    must equal the same world's run on this PR's base with the same removal (I-S34-6).
- **SD-d-7. The recorded-agent scenario (S34-d2).** A scripted agent, no model, through the MCP adapter on
  the real `mineworld server … --town` (so NPCs act), for each posture world found and the renamed-seats
  variant:
  1. observe; if `here.seats` is empty, walk through exits to the nearest place that lists seats (at most
     12 places, as the station scenario searches);
  2. pick the first seat whose `sit` offer is refused only `too_far_away`, or is available;
  3. `walk_to` its `position`; observe; the `sit` offer for that seat is now available;
  4. `act attempt` it → accepted; `me.seated.point` is that seat;
  5. a second seat (a human seat over HTTP) in the same place sees `PersonLine.seated` for the agent;
  6. an NPC present in the place perceived the agent's `sat-down` (read with `perceived_export`); the
     scenario waits until one is present, or fails by name;
  7. `act attempt` the `stand` offer → accepted; `me.seated` is gone; the position is unchanged.
  Adversaries: (a) `attempt` the `sit` offer of a view older than the newest → `stale_offer`; (b) the second
  seat sits on a seat first, then the agent attempts it → refused `target_unavailable`, and the agent is
  not moved.

## 6. Acceptance (decided before measuring, `ARC-23`)

| Id | Criterion | Worlds | Test |
| --- | --- | --- | --- |
| AC-d-1 | I-S34-1 (no two seated on one point) and I-S34-2 (seated means on the point, same place, same facing) hold after every fact of every run | every posture world (Social Café, Market Town; Lakeside when S34-c2 lands), renamed-seats, one-seat | `posture_life.rs` |
| AC-d-2 | I-S34-7: no `gave-way` names a seated person, and no `person-shoved` targets one | as AC-d-1 | `posture_life.rs` |
| AC-d-3 | **NPCs are seen seated:** default decisions, pace 900, 30 days: at least 50 % of the 09:00–18:00 samples show seating, in each world and seed | every posture world, renamed-seats | `posture_life.rs` |
| AC-d-4 | The same with every seat on `motivated` | every posture world, renamed-seats | `posture_life.rs` |
| AC-d-5 | **No bobbing:** zero bobbing spells (SD-d-1) in every run, at pace 900 and at pace 60; the median spell is at least two consults | every posture world, renamed-seats, one-seat | `posture_life.rs` |
| AC-d-6 | I-S34-6: the no-posture variant's run summary equals the base's; bodies-yard's and Lakeside's `scripts/baselines.txt` sections do not move | no-posture variant; bodies-yard; Lakeside (until S34-c2) | `posture_life.rs`; parity job |
| AC-d-7 | AC-S34-7: SD-d-7 steps 1–7 pass | every posture world; renamed-seats | `test_play_sit_mcp.py` |
| AC-d-8 | Adversaries (a) `stale_offer` and (b) `target_unavailable`, the agent unmoved | as AC-d-7 | `test_play_sit_mcp.py` |
| AC-d-9 | Zero lines change in `mcp_server.py` and `player.py`; the closed tool list and `act` kinds are unchanged | — | `git diff --stat` in review; the existing `test_mcp_tools.py` |
| AC-d-10 | The view's new lines read only what the server sent: expected values come from a recorded observation's raw JSON, never from `view.py` | a recorded Social Café observation and a recorded renamed-seats observation | `test_view.py` additions |

## 7. Planted mutations (each must turn a named test red, then be reverted)

| Id | Mutation | Expected red |
| --- | --- | --- |
| X-d-1 | remove `sit` and `stand` from `DECIDED_BY_NAME` | AC-d-5 (bobbing spells reappear at pace 60) |
| X-d-2 | the rest band also takes offers refused `TargetUnavailable` | AC-d-1 or the rule unit test "a taken seat is never chosen" |
| X-d-3 | the rest band reads a sit point by a fixed key | renamed-seats variant: AC-d-3 |
| X-d-4 | the rest band runs in a world without `posture` (draws index 16 whenever it is consulted) | AC-d-6 (no-posture summary differs) |
| X-d-5 | `disclosed.py` reads `posture-state` from the observer instead of the subject | AC-d-7 step 5 |
| X-d-6 | `OfferLine.seat` drops the key | AC-d-7 step 3 (no offer matches the chosen seat) |
| X-d-7 | the I-S34-2 check in `posture_life.rs` skips facing | a unit test of the checker with a planted wrong-facing journal |

## 8. Commit plan

Each commit tracks **Implementation**, **Validation** and **Review** separately, each `[ ]` until done with
evidence in §13.

### S34-d1

#### C0 — Design (this file), docs only
- [ ] Implementation: this file.
- [ ] Validation: `python3 -I scripts/check_doc_headings.py` PASS.
- [ ] Review: scope equals the step's S34-d row, apart from the questions in §14.

#### C1 — The measurement, before any controller change
- Goal: SD-d-2. Know the baseline before changing anything.
- Scope: `tools/cli/tests/posture_life.rs` (new); `tools/cli/tests/variants/mod.rs` (three new variants);
  a small journal reader for posture facts beside `social::facts_of`. No production code.
- [ ] Implementation: world discovery by `systems:`; the runs of SD-d-6; the invariant checks (AC-d-1,
  AC-d-2) asserted; the measures of SD-d-1 printed per world, seed and model.
- [ ] Validation: CI on the commit's head (R-LOAD-1). Record the printed baseline in §13. A local run of one
  world, one seed, 3 days only, to check the reader.
- [ ] Review: no world, place or key in the test; the checker's expected values come from the journal, not
  from posture's code (test rules §25).

#### C2 — The rule controller's rest band
- Goal: SD-d-3.
- Scope: `cognition/rule-controller/src/rest.rs` (new), `paced.rs` (one band call, `DECIDED_BY_NAME`),
  `Cargo.toml` (`mineworld-posture`), `rest_tests.rs` (new). No existing constant changes.
- [ ] Implementation: the band; the two action types by name; draw index 16.
- [ ] Validation: unit tests on hand-built observations: an available seat is sat on; a too-far free seat
  is walked to, nearest first; a taken seat is never chosen; walking or away from the agenda → no rest; no
  `sit` offer → no draw taken (the draw sequence equals the base's). The existing `offered_tests.rs` and
  `paced_tests.rs` pass unchanged.
- [ ] Review: the band reads only the observation; no reach or occupancy computed; I-d-2.

#### C3 — The motivated model's two options
- Goal: SD-d-4.
- Scope: `cognition/motivated/src/options.rs`, `settings.rs`, `parameters.rs`, `Cargo.toml`; tests beside the existing ones.
- [ ] Implementation: `Choice::Sit`, `Choice::SeekSeat`; `sit`/`stand` out of `Offered`; the two base
  weights; the two option names.
- [ ] Validation: unit tests as in C2; a decision list that sets `sit: 0` and `seek-seat: 0` makes nobody
  sit (data controls it); the parameters test lists 15 options.
- [ ] Review: no world data in defaults; options appear only where `sit` offers exist.

#### C4 — Thresholds on, re-baselines, mutations
- [ ] Implementation: AC-d-3…AC-d-6 asserted in `posture_life.rs`; the Social Café 30-day decisions golden
  re-captured from CI; `scripts/baselines.txt` for social-cafe and market-town with `reason S34-d1`.
- [ ] Validation: CI on the exact head green; X-d-1…X-d-4 and X-d-7 planted one at a time, each red,
  reverted, each recorded in §13.
- [ ] Review: bodies-yard's and Lakeside's baseline sections untouched; the re-baseline reason names this PR.

#### C5 — Docs
- [ ] Implementation: dated notes on `ARC-147` (NPCs sit through a rest band; why no stillness gate) and on
  `ARC-151` (`sit`, `stand` decided by name); the rule controller's module docs.
- [ ] Validation: `check_doc_headings.py` and `check_decision_ids.py` PASS.
- [ ] Review: notes state the measured baseline and the after numbers from §13.

### S34-d2

#### C1 — The view's lines
- Scope: `clients/agent/src/mineworld_agent/disclosed.py`, `models.py`, `view.py`; a recorded observation
  fixture captured from a real Social Café server and one from the renamed-seats variant
  (`tests/fixtures/`, with `SOURCE.md` lines); `tests/test_view.py`.
- [ ] Implementation: SD-d-5's three reads and lines; the module table rows.
- [ ] Validation: view tests with expected values from the fixtures' raw JSON (AC-d-10); X-d-5 and X-d-6
  planted, red, reverted; `ruff` and `pyright` clean as the agent package's checks require.
- [ ] Review: no occupancy computed; unknown shapes give `None`, as every other read.

#### C2 — The recorded-agent scenario
- Scope: `clients/agent/tests/test_play_sit_mcp.py` (new); the renamed-seats variant built in the test from
  a posture world, as `test_play_station_mcp.py` builds its generated world.
- [ ] Implementation: SD-d-7 steps 1–7 and adversaries (a), (b).
- [ ] Validation: CI's `real_server` job on the exact head; AC-d-7, AC-d-8; AC-d-9 by `git diff --stat
  origin/main -- clients/agent/src/mineworld_agent/mcp_server.py clients/agent/src/mineworld_agent/player.py`
  empty.
- [ ] Review: the script holds no world, place or key; every oracle is a server answer.

#### C3 — Docs
- [ ] Implementation: the agent reference pages, if generated from the models, regenerated; `ARC-147` note
  line for the view.
- [ ] Validation: the reference-page tests (`test_reference_pages.py`, `test_markdown_reference.py`) pass.
- [ ] Review: the words match the view's field names.

## 9. Test ownership

| Failure class | Owner |
| --- | --- |
| A controller sits on a taken seat, or computes reach | C2/C3 unit tests (deterministic observations) |
| A controller bobs | `posture_life.rs` (AC-d-5), pace 60 and 900 |
| NPCs never seen seated | `posture_life.rs` (AC-d-3, AC-d-4) |
| Seat exclusivity or seated-on-point broken over a long run | `posture_life.rs` (AC-d-1, AC-d-2) |
| A world without posture changes | no-posture variant (AC-d-6); the parity job's baselines |
| The view misreads a component | `test_view.py` against raw fixtures (AC-d-10) |
| An agent cannot reach, sit or stand through the real server | `test_play_sit_mcp.py` (AC-d-7, AC-d-8) |
| Real-model behaviour (LM controller) | not owned here: no LM is attached in tests (step §9.3) |
| Gate 1 (real LLM) | N/A: the scenario is recorded and deterministic |
| CI impact | `tools/cli` tests (long runs: the heaviest new cost, §11 R-d-2), `cognition/*` units, the agent's `real_server` job |

## 10. Is any of this material?

- **The rest band and the motivated options change controller behaviour**, which the step expected only
  "if needed" (§9.2). F-d-1 and F-d-2 show it is needed, and that the step's named remedy (the stillness
  gate) does not reach the cause (F-d-3). This is the operator's call: Q-S34d-2.
- **The agent view reads `sit-points` and the offer's key**, beyond the step's one `posture-state` row. The
  step's §9.3 already says the agent "`walk_to`s within reach when needed"; without the seat's position it
  cannot. This is a bounded completion of the step's intent, recorded here.
- Everything else follows the frozen step.

## 11. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-d-1 | People sit too much: the café fills, and staff sit on customers' stools | AC-d-3 is a floor, not a target; the weights are world data for the motivated model; a follow-up may read the agenda's purpose. Recorded, not tuned here |
| R-d-2 | CI time: 2 worlds × 2 seeds × 2 models × 30 days plus variants | Run the 30-day set in the milestone profile, as the parity job does; keep a 3-day set in the PR profile. Q-S34d-6 |
| R-d-3 | Lakeside lands (S34-c2) and fails AC-d-3 with a layout of its own | It joins by discovery; a failure there is S34-c2's evidence, raised, not tuned |
| R-d-4 | The scenario waits for an NPC that never comes to the place | A bounded wait (world time), then a named failure; the server runs `--town` so NPCs move |
| R-d-5 | The re-baselines hide an unrelated change | One reason line per world; the summary diff is read in review |

## 12. Execution contract

```text
PROJECT / PR:           MineWorld mvp0 · S34-d1 (controllers, measurement) and S34-d2 (agent view, scenario)
PRIMARY DESIGN DOC:     .structured-coding/plans/mvp0/pr-s34d-npcs-agents-sit.md
RELATED / BINDING DOCS: step-34-posture.md; pr-s34a, pr-s34b, pr-s34c; docs/decisions/ARC-147.md, ARC-151.md;
                        docs/ENGINEERING_RULES.md; docs/ENGINEERING_STANDARDS.md
IMPLEMENTATION BASE:    origin/main at freeze (≥ 0b2faf2f)
WORKTREES / BRANCHES:   /Users/yuema137/mineworld-worktrees/s34d1 · mvp0/pr-s34d1
                        /Users/yuema137/mineworld-worktrees/s34d2 · mvp0/pr-s34d2
IMPLEMENTER:            Haiku (Q-S34-14); Opus reviews C2 and C3 of S34-d1
APPROVED SCOPE:         §1 "In"; nothing in §1 "Out"
FROZEN INVARIANTS:      §4
VALIDATION BUDGET:      unit tests and one-world 3-day runs locally; every long run and the real-server
                        scenario in CI only (R-LOAD-1); no real LLM; no spend
ENDPOINT AUTHORITY:     implementation + local validation: authorized on freeze (source: freeze ruling)
                        semantic commits, push, PR create/update, CI repair: authorized on freeze (same)
                        merge: explicit operator authorization only; the coordinator merges
STOP CONDITION:         each PR READY FOR OPERATOR REVIEW: PR number and head SHA reported to the coordinator,
                        CI green on that exact head
ESCALATE:               AC-d-3, AC-d-4 or AC-d-5 missed after C4 (report the numbers; do not tune);
                        any change to systems/posture or to mcp_server.py / player.py
POST-MERGE SYNC OWNER:  the implementation session owns this file's ledger; the coordinator owns the step
                        and overall updates
```

## 13. Evidence ledger

(empty until implementation)

## 14. Operator questions (each with a recommended answer)

**Ruled 2026-10-27 (operator, primary session):** every question below is answered as recommended.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S34d-1** | One PR or two? The controller half and the agent half share no file | **Two, in parallel:** S34-d1 (controllers, measurement) and S34-d2 (agent view, scenario), one Haiku implementer each |
| **Q-S34d-2** | NPCs bob because the rule draws `stand` by chance (F-d-2), and they rarely sit because nothing walks them to a seat (F-d-1). The step's stillness gate does not reach either cause (F-d-3). Change the controllers? | **Yes.** `sit` and `stand` decided by name (no chance `stand`); one rest band in the rule (`SITS_BELOW` 50, draw 16), two options in motivated (`sit` 300, `seek-seat` 250). No change to posture; no existing constant changes. Measure the baseline first (C1) |
| **Q-S34d-3** | AC-S34-7 names Lakeside's pier bench, but Lakeside is not a posture world (S34-c2 not started). Wait for it? | **No.** S34-d tests every posture world it finds, plus a generated variant. Lakeside joins by discovery when S34-c2 lands; S34-c2's CI then runs this scenario on the pier. Schedule S34-c2 next in the lane |
| **Q-S34d-4** | The station halls of Social Café and Market Town draw 34 benches (about 340 seats) with no sit points. The cap is 64 per place, and posture offers one `sit` per point. Who fixes it? | **A separate S34-c3 PR, with a step-level ruling first.** Recommended ruling: posture offers `sit` only for the points within a fixed radius of the observer (e.g. 5 m), and the cap rises to 512. That changes frozen step §6.3, so it is yours to rule. Not in S34-d |
| **Q-S34d-5** | Thresholds for "seen seated" and "no bobbing" (§6) | **Accept:** at least 50 % of 09:00–18:00 hourly samples show a seated NPC; zero spells ended by the sitter's own `stand` within one consult; median spell at least two consults. Missed → escalate with numbers, not tune |
| **Q-S34d-6** | The full 30-day set is heavy for every PR's CI | **30-day set in the milestone profile, 3-day set in the PR profile**, as the parity job splits default and long |
