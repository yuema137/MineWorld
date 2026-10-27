# PR S34-c3 — sit points in the station halls, generated from world data

## Lifecycle

```text
Design revision:        revision 1 (2026-10-27)
Approved by / evidence: none yet — awaiting the operator's rulings (questions in §14)
Implementation base:    origin/main @ 315a50a9 (#513) for the audit; C0 records the exact base
Start condition:        this design frozen, with Q-S34c3-1 and Q-S34c3-2 ruled (they amend frozen
                        step-34 §5.1 and §6.3 and ARC-147). S34-b and S34-c1 are merged (#363, #368)
Execution contract:     §12
Lifecycle:              DRAFT — not frozen. Only the operator freezes this design.
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-34-posture.md`](step-34-posture.md) §§5.1, 6.3,
12–14 (the S34-c row: "sit points in … the station halls (S29)"; AC-S34-6's world half for "both station
halls"; I-S34-6, I-S34-9) · **Previous PRs:** [`pr-s34a-posture-pack.md`](pr-s34a-posture-pack.md) (#301),
[`pr-s34b-bodies-anchoring.md`](pr-s34b-bodies-anchoring.md) (#363),
[`pr-s34c-world-sit-points.md`](pr-s34c-world-sit-points.md) (S34-c1 #368; the format model and the source
of SD-c-1 and SD-c-2) · **Ruling that created this PR:** Q-S34d-4 in
[`pr-s34d-npcs-agents-sit.md`](pr-s34d-npcs-agents-sit.md) §14, and Q-S34f-6 in
[`pr-s34f-3d-sit-stand.md`](pr-s34f-3d-sit-stand.md) §16 · **Record:** a dated note on `ARC-147`
(`docs/decisions/ARC-147.md`). No new id.

**Working name:** S34-c3. **Branch:** `mvp0/pr-s34c3`. One worktree and one session.

**In one sentence.** The station halls of Social Café and Market Town draw 34 oak benches each, about 340
seats, and nobody can sit on any of them. This PR lets a place hold up to 512 sit points, makes posture
offer `sit` only for the points near the person who looks, and adds a tool that writes a place's sit
points from its own furniture data, so the halls get their points from a rule and not by hand.

---

## 0. Binding inputs

- **The operator's ruling (2026-10-27), Q-S34d-4, verbatim recommendation, ruled as recommended:**
  "A separate S34-c3 PR, with a step-level ruling first. Recommended ruling: posture offers `sit` only for
  the points within a fixed radius of the observer (e.g. 5 m), and the cap rises to 512. That changes
  frozen step §6.3, so it is yours to rule. Not in S34-d."
  This design reads that as: the direction is approved (a separate PR; nearby offers; cap 512), and the
  exact step amendment still needs one explicit ruling, because it changes frozen text. Q-S34c3-1 and
  Q-S34c3-2 ask for it in the smallest form.
- **Q-S34f-6 (ruled 2026-10-27):** "a small data PR, S34-c3, under S34-c's frozen rules. S34-f's gate covers
  whatever points the world discloses."
- **S34-c's frozen rules:** SD-c-1 (where a sit point goes: on free floor 260 mm from the seat's front
  face), SD-c-2 (one person per 600 mm of bench, Q-S34-8), SD-c-3 (keys and groups), SD-c-4 (one generic
  test over every world plus a generated variant), I-c-1 (no interpenetration), I-c-2 (removable), I-c-3
  (generic).
- **Lane charter (2026-10-18/19):** no world-specific hard-coding, and every acceptance criterion names a
  second world or a generated variant; no interpenetration; tools first, no hand-tuning; speed first
  (full suites at the milestone).
- **The coordinator's brief:** sit points are generated from world data by a tool, never hand-placed per
  world; say which part of the change is a frozen-contract change that needs the operator.

## 1. Scope

**In:**

1. **Posture: the cap.** `SIT_POINTS_MAX` 64 → 512 (Q-S34c3-2).
2. **Posture: nearby offers.** A standing observer is offered `sit` only for the sit points of their place
   within `OFFER_RADIUS` (5 000 mm) of their own position, in key order, each with its verdict as today
   (Q-S34c3-1). Dispatch is unchanged.
3. **The generator.** A pure function in `posture` that turns a place's `body:` (its floor, solids,
   furnishings and doorway points) and a seating catalogue into a list of sit points, and a `mineworld
   sit-points` subcommand that writes or checks a place file's generated `sit-points:` section (§5.3).
4. **The data.** Generated `sit-points:` sections in `worlds/{social-cafe,market-town}/places/station.yaml`
   (340 points each) and `station-platform.yaml` (6 points each).
5. **Tests.** The generic world test gains a drift check for generated sections; new posture tests for the
   cap and the radius; generator tests on generated variants and on Lakeside (dry run).
6. **Docs.** Dated amendments, as ruled, in `step-34-posture.md` §5.1 and §6.3, `ARC-147`'s note,
   `docs/MODULE_SPEC.md`'s `sit-points:` paragraph; the generated `docs/reference/` pages (the CLI page);
   stale lines named in §2.2. `docs/CORE_CONCEPTS.md` §6.4 states neither the cap nor the offer rule
   (`docs/CORE_CONCEPTS.md:520-560`), so it does not change.

**Out:**

- Chairs and stools (SD-c-1's "behind" and "beside" cases) in the generator, and regenerating S34-c1's
  hand-placed café, park and street sections (Q-S34c3-6). Those come with S34-c2, which needs them for
  Lakeside's bakery chairs.
- Lakeside's `posture` and its sit points (S34-c2). This PR only runs the generator on Lakeside as a dry
  run (AC-c3-6) and writes nothing there.
- Changes to the controllers, the agent view and the clients (S34-d, S34-e, S34-f). The one rule they need
  from this PR is stated in §5.5 and ruled by Q-S34c3-3; they implement it.
- Disclosure: `sit-points` stays disclosed whole to whoever perceives the place (§5.2).

## 2. Source audit (`origin/main @ 315a50a9`)

### 2.1 Facts

| # | Fact | Where |
| --- | --- | --- |
| 1 | A place holds at most 64 sit points; `SitPoints::new` refuses the 65th with `TooMany` | `systems/posture/src/point.rs:14`, `:212-218` |
| 2 | The spacing check compares every pair of points (n² / 2 pairs: 130 816 at 512) | `systems/posture/src/point.rs:221-236` |
| 3 | A test pins the cap: 64 accepted, 65 refused with "at most 64" | `systems/posture/tests/section.rs:70-81` |
| 4 | **Offers have no distance filter.** A standing observer gets one complete `sit {point}` for **every** point of the place, in key order, each with `sit_verdict`'s refusal | `systems/posture/src/perception.rs:42-56` |
| 5 | `sit_verdict` checks, in order: list, point exists, seated or walking, reach, occupied, crowded, arrival. The reach test uses the observer's `local`; no `local` answers `TooFarAway` | `systems/posture/src/system.rs:245-288` (reach at `:261-264`) |
| 6 | Each verdict scans every `posture-state` row and every `Presence` in the world (occupied, crowded) | `systems/posture/src/system.rs:265-282` |
| 7 | `reach` is a list parameter: default 750 mm, bounds 300 … 1 500; the compiled default is `REACH` | `systems/posture/src/interactions.rs:38-43`, `:101-105`; `action.rs:17` |
| 8 | `sit-points` is disclosed whole about the place; `posture-state` about a person | `systems/posture/src/perception.rs:59-79` |
| 9 | Posture's `SystemVersion` is 2. A component's shape or a fact's meaning bumps it; offers are neither | `systems/posture/src/system.rs:56`; `docs/MODULE_SPEC.md:586-590` |
| 10 | Both towns enable `posture` | `worlds/social-cafe/world.yaml:72`; `worlds/market-town/world.yaml:125` |
| 11 | Both towns' place files are byte-identical copies, the station files included | `diff -r worlds/social-cafe/places worlds/market-town/places` (no output) |
| 12 | **The hall.** Floor 16 800 × 51 800 mm. 34 benches as `body: furnishings`, keys `bench.w.0…16` and `bench.e.0…16`, kind `bench.station`, facing 0 (north). Each bench is one solid 6 000 × 600 mm, seat 450 mm: west bank x 1 200–7 200, east bank x 9 600–15 600, row k at y 6 000 + 2 400 k … 6 600 + 2 400 k. A kiosk solid at (1 200–3 200, 1 500–3 500). Doorways at (16 400, 25 900), (400, 25 900), (8 400, 400). No `sit-points:` section | `worlds/social-cafe/places/station.yaml:25-36` (comment), `:33-70` (furnishings), `:71-106` (solids), `:19-23` (passage) |
| 13 | **The platform.** Two benches `bench.platform.s` / `.n`, kind `bench.station`, facing 270 000 (west, to the track), solids x 10 100–10 700, 1 900 mm long. No `sit-points:` section | `worlds/social-cafe/places/station-platform.yaml:26-28`, `:42-44` |
| 14 | A furnishing is `{ key, kind, at, facing }`; `facing` is "the way its front faces"; its solids name it; at most 64 furnishings and 64 solids per place. `PlaceShape` exposes `solids()` and `furnishings()` | `systems/bodies/src/furnishing.rs:17`, `:131-145`; `component.rs:87-98`, `:182-189` |
| 15 | Bodies' load check for fixed points (S34-b): inside the floor by `PERSON_RADIUS` (250) and clear of every solid by `PERSON_RADIUS + GAP` (260) | `systems/bodies/src/anchoring.rs:178-230`; `geometry.rs:15`, `:22` |
| 16 | An independent layout test (S36-P5a) checks every person start, doorway point and sit point of every world against the stricter band: inside the floor by 260 and 260 from every solid, reading the files as text | `tools/cli/tests/layout.rs:1-13`; `tools/cli/tests/bodies/mod.rs:1224-1340` |
| 17 | **How S34-c1 derived its points: by hand.** The implementer applied SD-c-1 and SD-c-2 to SD-c-5's proposed table and corrected three cases by hand (BC-c-1…3). No tool was committed. The park's and the street's benches are bare solids with no furnishing | `pr-s34c-world-sit-points.md` §4 SD-c-5, §11; `git show --stat 3b03519e`; `worlds/social-cafe/places/park.yaml:27-46` |
| 18 | S34-c1's bench rule, as data shows it: points at the centre and ±600 mm on a 1 900 mm bench, 260 mm off the front face | `worlds/social-cafe/places/park.yaml:40-46` |
| 19 | The generic world test lists `worlds/`, loads each, checks each point is within reach of a solid, and checks removability; it names no world | `systems/posture/tests/worlds.rs:1-60` |
| 20 | No generator for sit points exists. The closest tools: `scripts/depiction_variant.py` (generated depiction variants), `tools/worldgen/tile.py` (tiles a pack), and S34-f's planned `tools/presentation/sit_point_depictions.py` (depictions from world data, not on main) | `scripts/`; `tools/worldgen/README.md`; `pr-s34f-3d-sit-stand.md` §5.6 |
| 21 | The CLI already has a write-or-check subcommand (`reference --out | --check`), and the CLI crate depends on bodies | `tools/cli/src/main.rs:300-311`; `tools/cli/Cargo.toml:70` |
| 22 | Lakeside does not enable `posture`. Its 29 sit points live only as the 3D client's fallback table | `worlds/lakeside/world.yaml:45-66`; `presentation/mineworld-default/3D/sit_points/lakeside.yaml` |
| 23 | Lakeside's halt bench: kind `bench.platform`, solid x 1 500–6 500, y 0–450 (5 000 mm), facing 0. S33-f's table puts **7** points on it at x 2 200 … 5 800, y 710, facing 0 | `worlds/lakeside/places/station.yaml:25-28`; `lakeside.yaml` (`wall-bench.1…7`) |
| 24 | Lakeside's platform benches: solids x −3 950 … −3 500, 2 000 mm long, facing 90 000. S33-f: three points each at x −3 240, ±600 mm round the centre, facing 90 000 | `worlds/lakeside/places/station-platform.yaml:27-32`; `lakeside.yaml` (`platform-bench-*`) |
| 25 | **Lakeside's pier and square benches face the other way from S33-f's points.** Pier `bench.head` faces 180 000 (south), solid y 300–750, but S33-f's points stand north of it (y 1 010) facing 0. Square `bench.west` faces 180 000 and `bench.east` 270 000, but S33-f's points stand north and east of them, facing 0 and 90 000 | `worlds/lakeside/places/pier.yaml:36-39`; `square.yaml:32-33`, `:40-41`; `lakeside.yaml` |
| 26 | Bodies' furnishings catalogue in the default Presentation Pack draws `bench.station` as a `bench_row` piece with `front_md: 0` | `presentation/mineworld-default/3D/furniture/furnishings.yaml:88` |
| 27 | S34-d's rest band (frozen, not implemented) finds a far seat through a `sit` offer refused only `TooFarAway`; the agent scenario's step 2 does the same | `pr-s34d-npcs-agents-sit.md` §5 SD-d-3 (l. 184-200), SD-d-7 (l. 228-236) |
| 28 | S34-e's shared chain (frozen, not implemented) answers `{}` when the frame has no `sit` offer for the clicked point, and walks only on `too_far_away` | `pr-s34e-2d-seated.md` §5 (l. 163-176) |
| 29 | S34-f's gate fails a disclosed point with no depiction; its C6 tool writes `sit_points/<world>.yaml` from world data for Social Café and Market Town; its no-clipping gate plays every clip on every body at every point | `pr-s34f-3d-sit-stand.md` §5.6, §5.7 items 1 and 4 (l. 290-320), C6 (l. 433) |
| 30 | Nothing of S34-d, S34-e or S34-f is on main (`git grep -n "SITS_BELOW\|seek-seat\|posture" cognition clients/agent/src`: no match; only `lakeside.yaml` under `sit_points/`) | — |
| 31 | The baselines of Social Café and Market Town were last re-recorded at the milestone of 2026-10-20 "after … S34-c1's posture with its sit points" | `scripts/baselines.txt:11`, `:29` |

### 2.2 Stale text and code–spec mismatches found

| # | Where | What | Fix |
| --- | --- | --- | --- |
| S-1 | `docs/MODULE_SPEC.md:906` | "a seventy-first point" is refused; the code refuses the 65th (audit 1) | C1 rewrites the sentence with the new cap |
| S-2 | `pr-s34c-world-sit-points.md` §1 Out; `pr-s34e-2d-seated.md:88` | "the station halls: S29's places are not on main"; they are (audit 12) | dated note in each, C3 |
| S-3 | `step-34-posture.md` §5.1 (l. 274-277), §6.3 (l. 365-371); `ARC-147` items 7 and SD-a-1's cap | the cap and the offer rule as frozen | dated amendments, only as ruled (Q-S34c3-1, -2), C1 |

### 2.3 Findings

- **F-c3-1. The halls need more than 64 points.** At one person per 600 mm (SD-c-2), a 6 000 mm bench holds
  10. 34 benches hold 340 (audit 12). The cap is 64 (audit 1).
- **F-c3-2. 340 offers in every observation.** Posture offers one `sit` per point of the place (audit 4).
  Each offer runs a verdict that scans every seated row and every presence (audit 6). A person in the hall
  would get 340 offers per frame, almost all refused `TooFarAway`, and an agent would read all of them.
- **F-c3-3. Hand placement does not scale.** S34-c1 placed 21 points by hand and needed three corrections
  (audit 17). 340 points by hand would be error-prone and would break the "no hand-tuning" rule.
- **F-c3-4. The hall's data already says everything a rule needs:** each bench is a furnishing with a kind,
  a front facing and one box solid (audit 12, 14). A generator needs only one more fact: which furnishing
  kinds are seats, and how a seat is laid out on them. That fact is in no data file today.
- **F-c3-5. Three frozen designs find far seats through refused offers** (audit 27, 28). Nearby offers
  hide far seats from them. They need one shared rule: a seat that is disclosed but not offered is out of
  offer range, so walk to its disclosed `at` first (§5.5, Q-S34c3-3).
- **F-c3-6. Lakeside's bench data and S33-f's table disagree** (audit 25). Whichever is wrong, a generator
  that reads `facing` will put the pier's and the square's points on the other side of the bench. Not this
  PR's to fix (Q-S34c3-7); the dry run (AC-c3-6) makes it visible with exact numbers.
- **F-c3-7. The rule "centre and 600 mm either side" and "one person per 600 mm" disagree on long
  benches.** S33-f put 7 points on a 5 000 mm bench (audit 23); `floor(5 000 / 600)` is 8. For the hall's
  6 000 mm benches: 9 points with the first rule, 10 with the second (Q-S34c3-4).

## 3. Which parts change a frozen contract

| Change | Frozen text it changes | Needs |
| --- | --- | --- |
| Cap 64 → 512 | step §5.1 ("more than 64 points per place"); S34-a SD-a-1; `ARC-147`; `MODULE_SPEC.md` `sit-points:` | **operator ruling** (Q-S34c3-2) |
| `sit` offered only within 5 m of the observer | step §6.3 ("one complete `sit { point }` offer per sit point, in key order"); `ARC-147` item 7 | **operator ruling** (Q-S34c3-1) |
| Consumers walk to a disclosed seat that is not offered | S34-d SD-d-3 step 2 and SD-d-7 step 2; S34-e's chain (`none → {}`), which S34-f reuses | **operator ruling** (Q-S34c3-3); implemented by those PRs |
| The seating catalogue (which furnishing kinds are seats) | none: new authoring data | operator's choice of home (Q-S34c3-5) |
| The generator and the hall data | none: S34-c's frozen rules (SD-c-1, SD-c-2) applied by a tool | bounded, within S34-c's scope |
| `posture`'s `SystemVersion` | stays 2: no component shape and no fact meaning changes; a save made before loads after (a larger cap accepts every old list) | none |

## 4. Invariants

- **I-S34-1, -2, -7** (step): hold after every request of the seeded runs in both halls.
- **I-S34-4** (step): clients and controllers decide nothing about distance. The radius filter is posture's.
- **I-S34-6** (step): worlds without `posture` (bodies-yard, Lakeside) are byte-identical; their baselines
  do not move.
- **I-S34-9** (step): every reachable drawn bench seat in the halls has a sit point.
- **I-c3-1. No admissible sit is hidden.** `OFFER_RADIUS` is at least the largest `reach` a list may set
  (1 500 mm), checked at compile time. So every point that dispatch would accept is offered.
- **I-c3-2. Offer equals dispatch** for every offered point (ARC-34, X-S34-4's test, unchanged).
- **I-c3-3. Generated, never hand-edited.** Every point this PR adds is the generator's output for that
  place. A generated section carries a marker comment; the drift check regenerates it and fails on any
  difference.
- **I-c3-4. Generic.** No world, place, furnishing key, furnishing kind or sit-point key appears in code or
  tests. Worlds are found by listing `worlds/`. Kinds appear only in the catalogue (data).
- **I-c3-5. No interpenetration.** Every generated point lies inside the floor by 260 mm, 260 mm from every
  solid (the stricter band of audit 16), 510 mm from every other point, and 510 mm from every doorway
  point of its place (a seated person cannot be pushed, S34-b, so a seat on a doorway would block it).
- **I-c3-6. Deterministic.** The generator's output depends only on the place's data and the catalogue;
  the same input gives the same bytes. Offers stay in key order.
- **I-c3-7. Disclosure unchanged.** `sit-points` is disclosed whole; only offers are filtered.

## 5. Design

### 5.1 The cap (C1) — SD-c3-1

`SIT_POINTS_MAX` becomes 512. Nothing else in `SitPoints` changes. The spacing check stays the plain pair
loop: at 512 points it is 130 816 distance checks once at genesis, which is negligible. The section test
moves to "512 accepted, 513 refused naming 512". The schema is unchanged: the generated
`docs/reference/schema/section-sit-points.schema.json` states no item bound.

Why 512 and not "no cap": a cap keeps a typo (a generator loop gone wrong) from building a place with
100 000 points. 512 holds the largest place on main (340) with room, and is the number the operator saw.

### 5.2 Nearby offers (C1) — SD-c3-2

```text
offers(observer, target = none)
    seated                         -> one complete stand {}                       (unchanged)
    standing, has a local position -> for each sit point of my place, in key order,
                                      with distance(my local, point.at) <= OFFER_RADIUS:
                                          one complete sit {point}, with sit_verdict's refusal (unchanged)
    standing, no local position    -> no sit offers
```

- `OFFER_RADIUS` = 5 000 mm, a posture constant beside `REACH`, with a `const` assertion that it is at least
  the list's upper bound for `reach` (1 500 mm) (I-c3-1).
- **Why a constant, not a list parameter.** The ruling's text says "a fixed radius". A parameter would add a
  field to the resolved Interaction List section, which posture states at genesis
  (`posture-interactions-configured`), and so would change a fact's shape and force a `SystemVersion`
  bump (MODULE_SPEC §587). A constant changes no fact. If the operator wants it per world later, it becomes
  a parameter then (Q-S34c3-1 offers that option).
- **Why 5 m.** It is the ruling's example. In the hall, a 5 m circle covers parts of at most five rows;
  with 600 mm seats and a 2 400 mm row pitch that is at most about 55 points (π·5² / (0.6 · 2.4) ≈ 55),
  which AC-c3-3 gates at ≤ 64, the old cap. A person sees every seat they could reach in a few steps.
- **Filter first.** The distance test runs before `sit_verdict`, so the occupied and crowded scans run only
  for nearby points (F-c3-2).
- **No local position.** Today such an observer gets every point refused `TooFarAway` (audit 5). With the
  filter, no point is near, so no `sit` is offered. Dispatch still answers `TooFarAway`, as before.
- **Dispatch is unchanged.** A `sit` request for a far point, sent from an old frame or by hand, answers
  `TooFarAway` from the same `sit_verdict`. So offer and dispatch still agree on every offered point
  (I-c3-2), and an unoffered point is one dispatch would refuse anyway (I-c3-1).
- **Disclosure is unchanged** (I-c3-7). Clients and agents still read every seat's `at` from `sit-points`.
  That is what lets them walk to a far seat (§5.5).

### 5.3 The generator (C2) — SD-c3-3

**What it reads.** One place's `body:` section (floor, solids, furnishings), the place's doorway points
(its passages' `here`, and every other place's `there` into it), and the seating catalogue (§5.4).

**What it writes.** A `sit-points:` section, sorted by key, preceded by one marker comment:

```yaml
# Generated by `mineworld sit-points` from this file's `body: furnishings` and the seating catalogue
# (S34-c3 SD-c3-3). Do not edit by hand: change the furniture or the catalogue and run the tool again.
sit-points:
  - { key: bench.e.0.1, group: bench.e.0, at: { x: 9900, y: 6860 }, facing: 0 }
  # …
```

**The rule for a `bench` seat** (SD-c-1's "open front" case and SD-c-2, written as one procedure):

1. Take the furnishings whose kind the catalogue names as `bench`, in key order.
2. Its box is the rectangle bounding the solids that name it. Its facing must be a multiple of 90 000
   (a box has four sides); otherwise the furnishing is reported "not axis-aligned" and gets no point.
3. The front face is the box side whose outward normal is the facing. `L` is that side's length.
4. `n = floor(L / pitch)` seats (pitch from the catalogue, default 600 mm, SD-c-2; Q-S34c3-4). Seat `i`
   (1 … n) is centred at `centre + (i − (n + 1) / 2) · pitch` along the face, numbered from the seated
   person's left to right. For the hall's benches: 10 seats, 300 mm in from each end.
5. Each candidate `at` is 260 mm (`PERSON_RADIUS + GAP`, read from bodies' constants) out from the front
   face, opposite the seat. `facing` = the furnishing's facing. `key` = `<furnishing key>.<i>`, `group` =
   the furnishing key (SD-c-3's shape, from data).
6. A candidate is **kept** when it lies inside the floor by 260 mm, 260 mm from every solid of the place,
   510 mm from every doorway point, and 510 mm from every point kept before it (furnishings in key order,
   seats in order). Otherwise it is **dropped**, and the report names the furnishing, the seat and the
   reason (`off-floor`, `in-solid <index>`, `on-doorway <passage>`, `too-close <key>`). Nothing is moved:
   a dropped seat is SD-c-1's "no point, recorded" (Q-S34c-5's precedent).
7. The kept list is built with `SitPoints::new`, so the cap, the key grammar and the spacing are posture's
   own checks.

The generator is a pure function in `systems/posture/src/generate.rs`:
`generate(shape: &PlaceShape, doorways: &[LocalPosition], seating: &Seating) -> Generated { points:
SitPoints, dropped: Vec<Dropped> }`. Posture already depends on bodies (audit 14 and posture's
`Cargo.toml`), and it is the owner of the sit-point rules, so the rule lives with its owner.

**The subcommand** in `tools/cli` (precedent: `reference --out | --check`, audit 21):

```text
mineworld sit-points <world> [--place KEY]... [--seating FILE] (--write | --check | --print)
    --print   print each place's generated section and its dropped seats; write nothing
    --write   replace each generated section in place (a section with the marker); add one to a place
              that has seat furnishings and no section; refuse a hand-authored section (no marker),
              naming the file
    --check   regenerate every marked section; exit 1 naming each place and key that differs
```

It loads the world through the real loader (so it reads exactly what the server reads) and touches only
the marked section's lines of the YAML text; every other byte of the file stays.

### 5.4 The seating catalogue (C2) — SD-c3-4

Which furnishing kinds are seats is a fact about the kind, not about one world. It goes in one data file,
by kind, read only by the generator (never by the server):

```yaml
# systems/posture/seating.yaml (recommended home, Q-S34c3-5)
- { kind: bench.station, form: bench }                # pitch defaults to 600 mm (SD-c-2)
- { kind: bench.platform, form: bench }
- { kind: bench.pier, form: bench }
- { kind: bench.park, form: bench }
```

- Only `form: bench` exists in this PR (Q-S34c3-6). A row may set `pitch` (510 … 1 200 mm).
- A kind not in the catalogue is not a seat. The report lists such kinds so a world author sees them.
- `--seating FILE` adds rows or overrides rows by kind, so a world from outside this repository brings its
  own kinds without editing posture.

### 5.5 What consumers do with a seat that is not offered (rule for S34-d, S34-e, S34-f) — SD-c3-5

One rule, stated here once, implemented by the PRs that own the consumers (Q-S34c3-3):

```text
a sit point disclosed in my place's sit-points
    offered, available                  -> attempt it
    offered, refused too_far_away       -> walk to its at, then use the newest offer   (unchanged)
    not offered                         -> out of offer range: walk to its at, then use the newest offer
    offered, refused otherwise          -> do not choose it                            (unchanged)
```

- S34-d's rest band and motivated `SeekSeat`: candidate seats are the disclosed points, nearest first; a
  seat that is offered and refused for any reason but `too_far_away` is skipped. The rest band's "walk"
  step is otherwise unchanged.
- S34-d2's agent scenario, step 2: "pick the first seat whose `sit` offer is refused only `too_far_away`,
  or is available, **or that is listed in `here.seats` with no offer**".
- S34-e's chain: `none -> {}` becomes `none, and the point is disclosed -> {walk: {place, local: at}}`.
- None of these measures a distance (I-S34-4): "not offered" is the server's answer.

### 5.6 The halls' data (C3) — SD-c3-6

Generated, not typed: `mineworld sit-points worlds/social-cafe --place station --place station-platform
--write`, and the same for Market Town. The expected output, from §5.3 on audit 12 and 13 (C3 records the
tool's actual report):

| Place | Furnishings | Seats each | Points | Facing | `at` |
| --- | --- | --- | --- | --- | --- |
| station | `bench.w.0…16` | 10 | 170 | 0 | x 1 500 … 6 900 step 600; y = 6 860 + 2 400 k |
| station | `bench.e.0…16` | 10 | 170 | 0 | x 9 900 … 15 300 step 600; y = 6 860 + 2 400 k |
| station-platform | `bench.platform.s`, `.n` | 3 | 6 | 270 000 | x 9 840; y = centre, ±600 |

Checks by hand on the expected values: row k's points at y 6 860 + 2 400 k are 260 mm from their bench's
front (6 600 + 2 400 k) and 1 540 mm from the next row's back; the outermost points (x 1 500, 15 300) are
1 500 mm from the walls; the nearest doorway point (16 400, 25 900) is about 1 110 mm from `bench.e.8.10`
(15 300, 26 060); the kiosk is 3 360 mm from row 0. No seat is expected to drop. Totals per world: 340 in
the hall, 6 on the platform.

Depictions: if S34-f's C6 tool (`tools/presentation/sit_point_depictions.py`) is on this PR's base, C3 runs
it and commits the regenerated `sit_points/{social-cafe,market-town}.yaml`. If it is not, S34-f's C6 sees
the new points when it runs. Either way no depiction is written by hand.

## 6. Acceptance (decided before measuring, `ARC-23`)

| Id | Criterion | Second world / variant |
| --- | --- | --- |
| AC-c3-1 | A place with 512 sit points loads; 513 is refused naming 512 and 513 | generated variant: a seeded 30 × 40 m floor with 512 points on a 1 200 mm lattice |
| AC-c3-2 | For observers on a 500 mm lattice of free floor, the `sit` offers are exactly the points within 5 000 mm, in key order, each with the verdict dispatch gives | Social Café's and Market Town's halls; the 512-point variant |
| AC-c3-3 | No observer position in either hall is offered more than 64 `sit` offers | Social Café, Market Town (both halls) |
| AC-c3-4 | No admissible sit is hidden: over 1 000 seeded observer positions and seeded `reach` values in 300 … 1 500, every point dispatch accepts is offered | the 512-point variant with a configured `reach` list |
| AC-c3-5 | The generator on seeded random places (benches of random length 600 … 8 000 mm, facings in the four directions, doorways, extra solids) writes sections that load (S34-b), pass the layout test's band (audit 16), keep `n = floor(L/pitch)` where the front is clear, and drop exactly the seats the rule says, naming the reason; the same seed gives the same bytes | generated variants, 200 seeds |
| AC-c3-6 | Lakeside dry run (`--print`, nothing written): the platform benches give exactly S33-f's six (`at`, `facing`); the halt bench gives 8 points at y 710, facing 0; the pier and square benches give the points the rule computes from their stated facings, with dropped seats named. The expected lists are fixed in the test before running | Lakeside |
| AC-c3-7 | Renaming every furnishing key and kind (and the catalogue rows with them) changes only keys and groups, never a position or a facing | variant of Social Café's hall, renamed |
| AC-c3-8 | The halls' committed sections equal the generator's output (`--check`, and the drift check in `tests/worlds.rs`); a hand edit fails naming place and key | Social Café, Market Town |
| AC-c3-9 | In each hall, a scripted person enters by the street door, is offered no `sit` for the bench point farthest from the door, walks to that point's disclosed `at`, is then offered it available, sits, and is seen seated by a bystander; then walks away and is stood up. The test picks the point from the disclosure, by distance, not by key | Social Café, Market Town, and the renamed variant |
| AC-c3-10 | I-S34-1, -2, -7 hold after every request of a seeded 300-step run that starts every person in the hall | Social Café, Market Town |
| AC-c3-11 | Removability (I-c-2): each town with `posture` and every `sit-points:` section removed begins from the same genesis facts, posture's own left out | Social Café, Market Town, the generated variant (existing check 3) |
| AC-c3-12 | Byte identity: bodies-yard and Lakeside baselines do not move; Social Café and Market Town re-record with `reason S34-c3` | bodies-yard, Lakeside |

## 7. Planted mutations and test ownership

### 7.1 Planted mutations (each must turn a named test red)

| Id | Mutation | Expected red |
| --- | --- | --- |
| X-c3-1 | drop the radius filter (offer every point) | AC-c3-2 on the 512-point variant; AC-c3-3 |
| X-c3-2 | `OFFER_RADIUS` 1 000 mm (below the reach bound) | compile-time assertion; with it removed, AC-c3-4 |
| X-c3-3 | measure the radius from the place's origin, not the observer | AC-c3-2 |
| X-c3-4 | sort nearby offers by distance instead of key | AC-c3-2's order check |
| X-c3-5 | `SIT_POINTS_MAX` back to 64 | AC-c3-1; both halls fail to load |
| X-c3-6 | generator offset 250 mm instead of `PERSON_RADIUS + GAP` | AC-c3-5 (layout band); the layout test over both towns |
| X-c3-7 | generator ignores doorway points | AC-c3-5 (a seeded door in front of a bench) |
| X-c3-8 | a kind name compared in code (`kind == "bench.station"`) | AC-c3-7 (renamed kinds) |
| X-c3-9 | move one committed hall point 100 mm by hand | AC-c3-8, naming place and key |
| X-c3-10 | `n = floor(L/pitch) − 1` (the "centre and either side" rule on even lengths) | AC-c3-5; AC-c3-6 (halt 8) |

### 7.2 Test ownership

| Failure class | Owner |
| --- | --- |
| Cap and spacing | `systems/posture/tests/section.rs` |
| Nearby offers, order, offer = dispatch | `systems/posture/tests/offers_nearby.rs` (new) |
| No admissible sit hidden | the compile-time assertion; `offers_nearby.rs` property test |
| Generator rule | `systems/posture/tests/generate.rs` (new): variants, renamed kinds, Lakeside dry run |
| Generated sections do not drift | `systems/posture/tests/worlds.rs` (new check 4); `mineworld sit-points --check` |
| A point inside furniture or off the floor | S34-b's load check; S36-P5a's `tools/cli/tests/layout.rs` (independent, text-read) |
| Walk to a far seat and sit | `systems/posture/tests/shipped.rs` (new hall scenario) |
| CLI behaviour (`--write` refuses a hand section; `--check` exit 1) | `tools/cli/tests/sit_points_cli.rs` (new) |
| Long-run history | `scripts/baselines.txt` via the milestone parity job |
| Depictions for the new points; seat contact | S34-f's C6 tool and C7 gate |
| Controllers and clients walking to unoffered seats | S34-d, S34-e (by Q-S34c3-3) |

## 8. Commit plan

Each commit has **Implementation**, **Validation** and **Review**, each `[ ]` until done with evidence.
Local heavy jobs go through `cast-slots/with-slot.sh`. Speed-first charter: run the new and touched tests
locally; full suites run at the milestone.

### C0 — Design (this file), docs only
- Goal: the reviewable design, before any code.
- [ ] Implementation: this file.
- [ ] Validation: `python3 scripts/check_doc_headings.py`.
- [ ] Review: every audit row cites a line on the base; §3 lists every frozen text touched.

### C1 — Posture: cap 512 and nearby offers (only after Q-S34c3-1 and -2 are ruled)
- Goal: SD-c3-1, SD-c3-2. One commit because the cap without the filter would put 340 offers in a frame.
- Files: `systems/posture/src/point.rs`, `perception.rs`, `action.rs` (`OFFER_RADIUS` beside `REACH`),
  `lib.rs` (export); `tests/section.rs`; `tests/offers_nearby.rs` (new); `docs/MODULE_SPEC.md`
  (`sit-points:` paragraph, S-1); `step-34-posture.md` §5.1, §6.3 and `ARC-147` (dated amendments quoting
  the ruling).
- Unchanged: `sit_verdict`, dispatch, disclosure, `SystemVersion`, every refusal.
- [ ] Implementation: SD-c3-1; SD-c3-2 with the filter before the verdict and the compile-time assertion.
- [ ] Validation: `cargo test -p mineworld-posture` (section, offers_nearby, refusals, sitting, worlds,
  shipped); AC-c3-1, -2 (variant half), -4; X-c3-1…5 planted one at a time, each red, reverted.
- [ ] Review: offer order still key order; an observer without `local` gets no `sit`; the frozen-text
  amendments quote the ruling and change nothing else.

### C2 — The generator, the catalogue and the subcommand
- Goal: SD-c3-3, SD-c3-4.
- Files: `systems/posture/src/generate.rs`, `systems/posture/seating.yaml` (or the ruled home),
  `systems/posture/tests/generate.rs`; `tools/cli/src/sit_points.rs`, `main.rs` (one subcommand),
  `tools/cli/tests/sit_points_cli.rs`; the generated CLI reference page.
- [ ] Implementation: the rule of §5.3 steps 1–7; the subcommand's three modes; the marker; text-only
  replacement of the marked section.
- [ ] Validation: AC-c3-5, -6, -7; X-c3-6, -7, -8, -10 red then reverted; `--write` on a file with a
  hand-authored section is refused naming the file; `mineworld reference --check docs/reference`.
- [ ] Review: no kind, key, world or place in code (grep the diff); every number traces to bodies' or
  posture's constants or the catalogue; the report names every dropped seat.

### C3 — The halls' data, the drift check, depictions, docs
- Goal: SD-c3-6 and the world half of AC-S34-6 for "both station halls".
- Files: `worlds/{social-cafe,market-town}/places/station.yaml`, `station-platform.yaml` (tool output
  only); `systems/posture/tests/worlds.rs` (check 4: drift); `systems/posture/tests/shipped.rs` (hall
  scenario); `presentation/mineworld-default/3D/sit_points/{social-cafe,market-town}.yaml` only if S34-f's
  C6 tool is on the base (its output only); `ARC-147` note; S-2's dated notes; and, if Q-S34c3-3 is ruled,
  a dated note in `pr-s34d-npcs-agents-sit.md` and `pr-s34e-2d-seated.md` pointing to SD-c3-5.
- [ ] Implementation: run the tool with `--write` for both towns; record its report (counts, dropped
  seats) in §13; add the drift check and the hall scenario.
- [ ] Validation: both towns load; AC-c3-3, -8, -9, -10, -11; the layout test over every world;
  X-c3-9 red then reverted; `mineworld sit-points --check` on both towns exits 0.
- [ ] Review: the data diff is exactly the tool's output (rerun `--check`); the two towns' files stay
  byte-identical; bodies-yard and Lakeside untouched.

### C4 — Baselines (at the milestone, per the speed-first charter)
- [ ] Implementation: `scripts/baselines.txt` sections of Social Café and Market Town from a long record of
  the merged head, with `reason S34-c3: the station halls' 346 sit points and nearby sit offers`.
- [ ] Validation: the milestone parity job equals the new baselines; bodies-yard and Lakeside unchanged
  (AC-c3-12).
- [ ] Review: no number edited by hand.

## 9. Failure and edge cases

| Case | Behaviour |
| --- | --- |
| A seat furnishing whose facing is not a multiple of 90 000 | no points; the report says "not axis-aligned"; the world still loads |
| A seat whose front spot is off the floor, in a solid, on a doorway or too close to another point | dropped and named in the report; never moved |
| A place file with a hand-authored `sit-points:` section | `--write` refuses, naming the file; `--check` ignores it (it is not generated) |
| A marked section edited by hand | `--check` and the drift check fail, naming place and key |
| A furnishing kind not in the catalogue | not a seat; listed in the report |
| More than 512 kept points in a place | `SitPoints::new` refuses; the tool exits 1 naming the place and the count |
| An observer with no local position | no `sit` offers; dispatch answers `TooFarAway` as before |
| A `sit` request for a far point (old frame, scripted) | `TooFarAway`, unchanged |
| A world without `posture` | nothing offered, nothing generated into it by this PR; byte-identical |

## 10. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-c3-1 | S34-d, S34-e or S34-f is implemented to the old rule and cannot reach a far seat | Q-S34c3-3 rules SD-c3-5 before they start; C3 writes the dated notes; none of them is on main (audit 30) |
| R-c3-2 | In the café (8.3 × 10.3 m), a person by the door is more than 5 m from some seats, so the rule controller's offered band sees fewer `sit` offers and Social Café's history moves | expected; the baseline re-records with a reason (C4); S34-d's rest band walks to disclosed seats (SD-c3-5) |
| R-c3-3 | S34-f's no-clipping gate plays every clip on every body at every point; 680 more points could make it slow | S34-f owns its gate; this design flags it (Q-S34c3-8) and recommends one run per distinct seat geometry |
| R-c3-4 | The agent view (S34-d2's `Here.seats`) lists 340 seats in the hall, a long text for an LLM | S34-d2 may list nearby seats first or cap the list; noted for S34-d2, not decided here |
| R-c3-5 | The tool's text replacement damages comments or ordering in a place file | the tool touches only the marked block; its CLI test round-trips a file with comments before and after the block |
| R-c3-6 | Lakeside's dry-run expectations are fixed on data that S34-c2 may correct (F-c3-6) | AC-c3-6 asserts what the rule computes from today's data; S34-c2 updates the expectations with its data change |
| R-c3-7 | 340 more points add genesis facts and observation bytes (about 25 KB of `sit-points` per hall observation) | far below the 16 MB frame limit (`sdk/python/src/mineworld_sdk/session.py:110`); measured once in C3's evidence |

## 11. Implementation sequencing

C1 is independent of the data and can land first; it should land before S34-d and S34-e start, so they are
built on nearby offers. C2 needs C1 only for the cap. C3 needs C1 and C2. C4 is at the milestone.

## 12. Execution contract

```text
Repository / worktree:  yuema137/MineWorld · /Users/yuema137/mineworld-worktrees/s34c3-station-sit-points
Branch / base:          mvp0/pr-s34c3 from origin/main after this design freezes
Authorized (on freeze): implement C1–C3, validate locally (cast-slots/with-slot.sh), commit, push, open the PR
Validation budget:      the new and touched tests and both towns' loads; full suites and C4 at the milestone
Implementer / reviewer: Haiku from this commit plan; an independent Opus review of C1 and C2
Stop condition:         MERGE-READY: PR number and head SHA reported to the coordinator
Merge boundary:         the coordinator merges
Escalate:               any change to dispatch, disclosure or SystemVersion; a dropped seat in either hall;
                        a world other than the two towns written; any edit to Lakeside's data
```

## 13. Evidence ledger

(empty until implementation)

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S34c3-1** | Amend frozen step §6.3 and `ARC-147` item 7: posture offers `sit` only for the points within 5 m of the person, in key order, and still discloses every point. A compiled constant, or a per-world list parameter? | **Yes, a constant of 5 000 mm**, never below the largest `reach` (1 500 mm). A parameter would change a genesis fact and force a `SystemVersion` bump; it can become one later if a world needs it |
| **Q-S34c3-2** | Amend frozen step §5.1 and SD-a-1: raise the cap from 64 to 512 sit points per place? | **Yes, 512.** No schema or `SystemVersion` change; every old save stays valid |
| **Q-S34c3-3** | Nearby offers hide far seats from three frozen designs that find seats through refused offers (S34-d's rest band and agent step, S34-e's chain used by S34-f). Adopt SD-c3-5: "a disclosed seat with no offer is out of range: walk to its `at`, then use the offer"? | **Yes.** C3 adds a dated note to S34-d and S34-e; they implement it. None has started, so nothing is reworked |
| **Q-S34c3-4** | Seats per bench: `floor(length / 600)` (10 per 6 m bench, 340 per hall), or S33-f's "centre and 600 mm either side" (9 per 6 m bench, 306 per hall)? | **`floor(length / 600)`**: it is SD-c-2's "one person per 600 mm" and matches every S34-c1 bench. Lakeside's halt then gets 8, not S33-f's 7; S34-c2 adopts the tool's count |
| **Q-S34c3-5** | Where does the seating catalogue (which furnishing kinds are seats) live? | **`systems/posture/seating.yaml`**, read only by the generator, with `--seating FILE` for worlds that bring their own kinds. Not in a World Pack, because the kinds are shared vocabulary; not in presentation, because the server's data must not depend on it |
| **Q-S34c3-6** | Generator scope: benches only now; chairs and stools (S34-c1's "behind" and "beside" cases) and regenerating S34-c1's hand-placed café, park and street sections later? | **Yes, benches only.** S34-c2 adds the chair form for Lakeside's bakery and regenerates S34-c1's sections with the tool. The park's and street's benches first need furnishings (they are bare solids) |
| **Q-S34c3-7** | Lakeside's pier and square benches face the opposite way from S33-f's sit points (F-c3-6). Fix it here? | **No.** S34-c2 checks the drawn benches and corrects whichever is wrong. This PR's dry run records the numbers |
| **Q-S34c3-8** | S34-f's no-clipping gate would run on 680 more points (two towns). Ask S34-f to run its clip gate once per distinct seat geometry (kind, seat offset, seat height) instead of once per point? | **Yes**, as a note to S34-f; its gate stays per point for the drawn-vs-disclosed check, which is cheap |
